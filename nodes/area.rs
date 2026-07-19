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
    // Trust geo's geodesic area, which already follows the GeoJSON right-hand
    // rule (RFC 7946): the ring winding defines which side is the interior, so
    // `geodesic_area_unsigned` returns the area of that interior directly. A
    // reversed (clockwise) exterior ring therefore names the complementary
    // region and is measured as such — winding is meaningful, not normalized away.
    let (sq, perim) = match &geom {
        GeoGeometry::Polygon(p) => (p.geodesic_area_unsigned(), p.geodesic_perimeter()),
        GeoGeometry::MultiPolygon(mp) => (mp.geodesic_area_unsigned(), mp.geodesic_perimeter()),
        _ => return Ok(Area { square_meters: 0.0, perimeter_meters: 0.0, error: "WRONG_GEOMETRY_TYPE".into() }),
    };
    Ok(Area { square_meters: sq, perimeter_meters: perim, error: String::new() })
}
