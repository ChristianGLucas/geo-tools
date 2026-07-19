use crate::axiom_context::AxiomContext;
use crate::gen::messages::Geometry;
use geo::{ConvexHull, CoordsIter, Geometry as GeoGeometry, MultiPoint, Point};

#[path = "geoutil.rs"]
mod geoutil;

/// The convex hull of a geometry's vertices — the smallest convex polygon
/// enclosing every coordinate — emitted as a GeoJSON Polygon in the canonical
/// `Geometry` envelope. Computed planar-ly on lon/lat (fine for local extents).
/// An empty geometry yields EMPTY_GEOMETRY.
pub fn convex_hull(
    ax: &dyn AxiomContext,
    input: Geometry,
) -> Result<Geometry, Box<dyn std::error::Error>> {
    let _ = ax;
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Geometry { geojson: String::new(), error: e.into() }),
    };
    let pts: Vec<Point<f64>> = geom.coords_iter().map(Point::from).collect();
    if pts.is_empty() {
        return Ok(Geometry { geojson: String::new(), error: "EMPTY_GEOMETRY".into() });
    }
    let hull = MultiPoint::new(pts).convex_hull();
    let out: GeoGeometry<f64> = GeoGeometry::Polygon(hull);
    Ok(Geometry { geojson: geoutil::to_geojson(&out), error: String::new() })
}
