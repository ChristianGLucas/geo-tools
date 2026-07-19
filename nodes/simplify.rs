use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ SimplifyInput, Geometry };
use geo::{CoordsIter, Geometry as GeoGeometry, Simplify};

#[path = "geoutil.rs"]
mod geoutil;

/// Largest vertex count Simplify will process. Ramer–Douglas–Peucker recurses,
/// and an adversarial shape (e.g. a dense sawtooth) recurses O(n) deep and costs
/// O(n²); this per-node cap keeps worst-case CPU well under a second and is far
/// tighter than the shared MAX_COORDS, which only bounds count, not recursion.
const MAX_SIMPLIFY_COORDS: usize = 10_000;

/// Stack for the RDP worker thread. A native stack overflow aborts the whole
/// process, so the recursion runs on a roomy dedicated stack — with the vertex
/// cap above, recursion depth cannot come close to exhausting it.
const RDP_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Ramer–Douglas–Peucker simplification of a line or polygon geometry: drops
/// vertices that lie within `epsilon` (in degrees) of the retained shape.
/// Larger epsilon removes more points. Point/MultiPoint inputs pass through
/// unchanged. Inputs with more than 10,000 vertices return TOO_MANY_COORDS.
/// Emits the result in the canonical `Geometry` envelope.
pub fn simplify(
    ax: &dyn AxiomContext,
    input: SimplifyInput,
) -> Result<Geometry, Box<dyn std::error::Error>> {
    let _ = ax;
    if !input.epsilon.is_finite() || input.epsilon < 0.0 {
        return Ok(Geometry { geojson: String::new(), error: "OUT_OF_RANGE".into() });
    }
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Geometry { geojson: String::new(), error: e.into() }),
    };
    if geom.coords_iter().count() > MAX_SIMPLIFY_COORDS {
        return Ok(Geometry { geojson: String::new(), error: "TOO_MANY_COORDS".into() });
    }
    let eps = input.epsilon;
    // Run the recursive RDP on a dedicated large-stack thread so a deep
    // recursion can never overflow the (smaller) request-handler stack and abort
    // the process. The vertex cap above bounds both the depth and the CPU cost.
    let worker = std::thread::Builder::new()
        .stack_size(RDP_STACK_BYTES)
        .spawn(move || -> GeoGeometry<f64> {
            match geom {
                GeoGeometry::LineString(ls) => GeoGeometry::LineString(ls.simplify(eps)),
                GeoGeometry::MultiLineString(mls) => GeoGeometry::MultiLineString(mls.simplify(eps)),
                GeoGeometry::Polygon(p) => GeoGeometry::Polygon(p.simplify(eps)),
                GeoGeometry::MultiPolygon(mp) => GeoGeometry::MultiPolygon(mp.simplify(eps)),
                other => other,
            }
        })?;
    let out = match worker.join() {
        Ok(g) => g,
        Err(_) => return Ok(Geometry { geojson: String::new(), error: "SIMPLIFY_FAILED".into() }),
    };
    Ok(Geometry { geojson: geoutil::to_geojson(&out), error: String::new() })
}
