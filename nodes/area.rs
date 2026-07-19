use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ Geometry, Area };
use geo::{GeodesicArea, Geometry as GeoGeometry};

#[path = "geoutil.rs"]
mod geoutil;

/// Geodesic area (m²) and perimeter (m) of a Polygon or MultiPolygon on the
/// WGS-84 ellipsoid. `square_meters` is the unsigned area, so ring winding order
/// does not affect the sign. Non-polygon geometries yield WRONG_GEOMETRY_TYPE.
pub fn area(
    ax: &dyn AxiomContext,
    input: Geometry,
) -> Result<Area, Box<dyn std::error::Error>> {
    let _ = ax;
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Area { square_meters: 0.0, perimeter_meters: 0.0, error: e.into() }),
    };
    let (raw, perim) = match &geom {
        GeoGeometry::Polygon(p) => (p.geodesic_area_unsigned(), p.geodesic_perimeter()),
        GeoGeometry::MultiPolygon(mp) => (mp.geodesic_area_unsigned(), mp.geodesic_perimeter()),
        _ => return Ok(Area { square_meters: 0.0, perimeter_meters: 0.0, error: "WRONG_GEOMETRY_TYPE".into() }),
    };
    // geo interprets ring winding per GeoJSON's right-hand rule: a reversed
    // (clockwise) exterior ring names the *complementary* region, so its unsigned
    // area comes back as ~(Earth − region). Normalize to the smaller enclosed
    // region so the result is winding-order independent. This is exact for any
    // region up to half the Earth's surface; a genuine polygon larger than that
    // must follow the right-hand rule (CCW exterior) to measure correctly.
    const EARTH_AREA_M2: f64 = 5.100_656_217_240_886e14; // WGS-84 ellipsoid surface
    let sq = if raw > EARTH_AREA_M2 / 2.0 { (EARTH_AREA_M2 - raw).max(0.0) } else { raw };
    Ok(Area { square_meters: sq, perimeter_meters: perim, error: String::new() })
}
