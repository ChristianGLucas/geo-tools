use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ PointPair, Bearing };
use geo::{Bearing as _, Geodesic, Point};

#[path = "geoutil.rs"]
mod geoutil;

/// Initial bearing (forward azimuth) from the first point to the second, in
/// degrees clockwise from true north, normalized to [0, 360). Uses the geodesic
/// (WGS-84) model. Returns a structured `error` (NON_FINITE_COORD or
/// OUT_OF_RANGE) for invalid coordinates.
pub fn bearing(
    ax: &dyn AxiomContext,
    input: PointPair,
) -> Result<Bearing, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = geoutil::point_in_range(input.from_lon, input.from_lat) {
        return Ok(Bearing { degrees: 0.0, error: e.into() });
    }
    if let Err(e) = geoutil::point_in_range(input.to_lon, input.to_lat) {
        return Ok(Bearing { degrees: 0.0, error: e.into() });
    }
    let a = Point::new(input.from_lon, input.from_lat);
    let b = Point::new(input.to_lon, input.to_lat);
    let deg = Geodesic.bearing(a, b);
    let normalized = ((deg % 360.0) + 360.0) % 360.0;
    Ok(Bearing { degrees: normalized, error: String::new() })
}
