use crate::axiom_context::AxiomContext;
use crate::gen::messages::Geometry;
use geo::{Centroid, Geometry as GeoGeometry};

#[path = "geoutil.rs"]
mod geoutil;

/// The centroid of any geometry, emitted as a GeoJSON Point in the canonical
/// `Geometry` envelope. For a polygon this is the area centroid; for a line the
/// length-weighted centroid; for points the mean position. An empty geometry
/// yields EMPTY_GEOMETRY.
pub fn centroid(
    ax: &dyn AxiomContext,
    input: Geometry,
) -> Result<Geometry, Box<dyn std::error::Error>> {
    let _ = ax;
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Geometry { geojson: String::new(), error: e.into() }),
    };
    match geom.centroid() {
        Some(pt) => {
            let out: GeoGeometry<f64> = GeoGeometry::Point(pt);
            Ok(Geometry { geojson: geoutil::to_geojson(&out), error: String::new() })
        }
        None => Ok(Geometry { geojson: String::new(), error: "EMPTY_GEOMETRY".into() }),
    }
}
