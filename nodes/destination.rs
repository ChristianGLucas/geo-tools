use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ DestinationInput, Geometry };
use geo::{Destination as _, Geodesic, Geometry as GeoGeometry, Point};

#[path = "geoutil.rs"]
mod geoutil;

/// The destination point reached by starting at an origin and travelling a
/// geodesic distance (meters) along a forward azimuth (degrees clockwise from
/// true north) on the WGS-84 ellipsoid. Emits the point as a GeoJSON Point in
/// the canonical `Geometry` envelope, so it chains straight into the other
/// geometry nodes. Sets `error` for non-finite, out-of-range, or negative input.
pub fn destination(
    ax: &dyn AxiomContext,
    input: DestinationInput,
) -> Result<Geometry, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = geoutil::point_in_range(input.lon, input.lat) {
        return Ok(Geometry { geojson: String::new(), error: e.into() });
    }
    if !input.bearing_deg.is_finite() || !input.distance_m.is_finite() {
        return Ok(Geometry { geojson: String::new(), error: "NON_FINITE_COORD".into() });
    }
    if input.distance_m < 0.0 {
        return Ok(Geometry { geojson: String::new(), error: "OUT_OF_RANGE".into() });
    }
    let origin = Point::new(input.lon, input.lat);
    let dest: Point<f64> = Geodesic.destination(origin, input.bearing_deg, input.distance_m);
    let geom: GeoGeometry<f64> = GeoGeometry::Point(dest);
    Ok(Geometry { geojson: geoutil::to_geojson(&geom), error: String::new() })
}
