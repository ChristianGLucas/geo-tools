use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ PointPair, Distance };
use geo::{Distance as _, Geodesic, Point};

#[path = "geoutil.rs"]
mod geoutil;

/// Geodesic distance in meters between two [longitude, latitude] points on the
/// WGS-84 ellipsoid (Karney's algorithm — the shortest path over the curved
/// Earth, accurate to sub-millimeter). Returns a structured `error`
/// (NON_FINITE_COORD or OUT_OF_RANGE) instead of a numeric result for bad input.
pub fn distance(
    ax: &dyn AxiomContext,
    input: PointPair,
) -> Result<Distance, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = geoutil::point_in_range(input.from_lon, input.from_lat) {
        return Ok(Distance { meters: 0.0, error: e.into() });
    }
    if let Err(e) = geoutil::point_in_range(input.to_lon, input.to_lat) {
        return Ok(Distance { meters: 0.0, error: e.into() });
    }
    let a = Point::new(input.from_lon, input.from_lat);
    let b = Point::new(input.to_lon, input.to_lat);
    Ok(Distance { meters: Geodesic.distance(a, b), error: String::new() })
}
