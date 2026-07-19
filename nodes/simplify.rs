use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ SimplifyInput, Geometry };
use geo::{Geometry as GeoGeometry, Simplify};

#[path = "geoutil.rs"]
mod geoutil;

/// Ramer–Douglas–Peucker simplification of a line or polygon geometry: drops
/// vertices that lie within `epsilon` (in degrees) of the retained shape.
/// Larger epsilon removes more points. Point/MultiPoint inputs pass through
/// unchanged. Emits the result in the canonical `Geometry` envelope.
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
    let eps = input.epsilon;
    let out: GeoGeometry<f64> = match geom {
        GeoGeometry::LineString(ls) => GeoGeometry::LineString(ls.simplify(eps)),
        GeoGeometry::MultiLineString(mls) => GeoGeometry::MultiLineString(mls.simplify(eps)),
        GeoGeometry::Polygon(p) => GeoGeometry::Polygon(p.simplify(eps)),
        GeoGeometry::MultiPolygon(mp) => GeoGeometry::MultiPolygon(mp.simplify(eps)),
        other => other,
    };
    Ok(Geometry { geojson: geoutil::to_geojson(&out), error: String::new() })
}
