use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ Geometry, Length };
use geo::{Geodesic, Geometry as GeoGeometry, Length as _};

#[path = "geoutil.rs"]
mod geoutil;

/// Total geodesic length in meters of a LineString or MultiLineString on the
/// WGS-84 ellipsoid (the sum of its segment lengths). Non-line geometries yield
/// WRONG_GEOMETRY_TYPE; malformed input yields the parse error token.
pub fn length(
    ax: &dyn AxiomContext,
    input: Geometry,
) -> Result<Length, Box<dyn std::error::Error>> {
    let _ = ax;
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Length { meters: 0.0, error: e.into() }),
    };
    let meters = match &geom {
        GeoGeometry::Line(l) => Geodesic.length(l),
        GeoGeometry::LineString(ls) => Geodesic.length(ls),
        GeoGeometry::MultiLineString(mls) => Geodesic.length(mls),
        _ => return Ok(Length { meters: 0.0, error: "WRONG_GEOMETRY_TYPE".into() }),
    };
    Ok(Length { meters, error: String::new() })
}
