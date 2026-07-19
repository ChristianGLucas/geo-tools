use crate::axiom_context::AxiomContext;
use crate::gen::messages::{ ContainsInput, Contains };
use geo::{Contains as _, Geometry, Point};

#[path = "geoutil.rs"]
mod geoutil;

/// Whether a test point lies inside a Polygon or MultiPolygon (interior only —
/// a point exactly on the boundary is not contained). Non-polygon geometries
/// yield WRONG_GEOMETRY_TYPE; bad coordinates yield NON_FINITE_COORD /
/// OUT_OF_RANGE.
pub fn contains(
    ax: &dyn AxiomContext,
    input: ContainsInput,
) -> Result<Contains, Box<dyn std::error::Error>> {
    let _ = ax;
    if let Err(e) = geoutil::point_in_range(input.lon, input.lat) {
        return Ok(Contains { contains: false, error: e.into() });
    }
    let geom = match geoutil::parse_geometry(&input.geojson) {
        Ok(g) => g,
        Err(e) => return Ok(Contains { contains: false, error: e.into() }),
    };
    let point = Point::new(input.lon, input.lat);
    let inside = match &geom {
        Geometry::Polygon(p) => p.contains(&point),
        Geometry::MultiPolygon(mp) => mp.contains(&point),
        _ => return Ok(Contains { contains: false, error: "WRONG_GEOMETRY_TYPE".into() }),
    };
    Ok(Contains { contains: inside, error: String::new() })
}
