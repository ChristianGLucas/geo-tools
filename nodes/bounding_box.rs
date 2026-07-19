use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ Geometry, BoundingBox };
use geo::{BoundingRect, Geometry as GeoGeometry};

#[path = "geoutil.rs"]
mod geoutil;

/// Axis-aligned bounding box of a geometry, as numeric bounds in decimal degrees
/// plus the same box as a GeoJSON Polygon in `geojson` (ready to chain into the
/// geometry nodes). An empty geometry yields EMPTY_GEOMETRY.
pub fn bounding_box(
    ax: &dyn AxiomContext,
    input: Geometry,
) -> Result<BoundingBox, Box<dyn std::error::Error>> {
    let _ = ax;
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(empty_bbox(e)),
    };
    match geom.bounding_rect() {
        Some(rect) => {
            let min = rect.min();
            let max = rect.max();
            let poly: GeoGeometry<f64> = GeoGeometry::Polygon(rect.to_polygon());
            Ok(BoundingBox {
                min_lon: min.x,
                min_lat: min.y,
                max_lon: max.x,
                max_lat: max.y,
                geojson: geoutil::to_geojson(&poly),
                error: String::new(),
            })
        }
        None => Ok(empty_bbox("EMPTY_GEOMETRY")),
    }
}

fn empty_bbox(err: &str) -> BoundingBox {
    BoundingBox {
        min_lon: 0.0, min_lat: 0.0, max_lon: 0.0, max_lat: 0.0,
        geojson: String::new(), error: err.into(),
    }
}
