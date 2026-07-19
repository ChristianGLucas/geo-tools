// Shared GeoJSON <-> geo-types helpers for geo-tools nodes.
//
// The generated service.rs only wires the node files listed in axiom.yaml as
// crate modules, so shared code cannot live at the crate root. Each node that
// needs these helpers includes this file as its own submodule with
// `#[path = "geoutil.rs"] mod geoutil;` (the `#[path]` is resolved relative to
// nodes/). Compiling it once per including node is harmless: every function is
// pure and returns only external (geo/geojson) or std types.
#![allow(dead_code)]

use geo::{CoordsIter, Geometry};
use geojson::GeoJson;
use std::str::FromStr;

/// Max accepted length of a GeoJSON input string (bytes). Checked on the RAW
/// input before any parse, bounding allocation/parse cost up front.
pub const MAX_GEOJSON_LEN: usize = 1_000_000;
/// Max total coordinate count in a parsed geometry. Bounds superlinear ops
/// (Ramer–Douglas–Peucker, convex hull) against a crafted huge input.
pub const MAX_COORDS: usize = 100_000;

/// True if a point's coordinates are finite and within WGS-84 range.
pub fn point_in_range(lon: f64, lat: f64) -> Result<(), &'static str> {
    if !lon.is_finite() || !lat.is_finite() {
        return Err("NON_FINITE_COORD");
    }
    if lat.abs() > 90.0 || lon.abs() > 180.0 {
        return Err("OUT_OF_RANGE");
    }
    Ok(())
}

/// Parse a GeoJSON string into a geo-types `Geometry<f64>`, enforcing the input
/// caps and rejecting non-finite coordinates. A bare geometry, or a single
/// Feature wrapping one, is accepted; a FeatureCollection is rejected as
/// ambiguous. Returns a stable error token on any failure.
pub fn parse_geometry(geojson: &str) -> Result<Geometry<f64>, &'static str> {
    // Length cap on the RAW bytes, before trim or parse.
    if geojson.len() > MAX_GEOJSON_LEN {
        return Err("INPUT_TOO_LONG");
    }
    let s = geojson.trim();
    if s.is_empty() {
        return Err("EMPTY_INPUT");
    }
    let gj = GeoJson::from_str(s).map_err(|_| "INVALID_GEOJSON")?;
    let geom: Geometry<f64> = match gj {
        GeoJson::Geometry(g) => Geometry::try_from(g).map_err(|_| "INVALID_GEOJSON")?,
        GeoJson::Feature(f) => match f.geometry {
            Some(g) => Geometry::try_from(g).map_err(|_| "INVALID_GEOJSON")?,
            None => return Err("EMPTY_INPUT"),
        },
        GeoJson::FeatureCollection(_) => return Err("WRONG_GEOMETRY_TYPE"),
    };
    // Single pass: count coordinates and verify each is finite.
    let mut n = 0usize;
    for c in geom.coords_iter() {
        n += 1;
        if !c.x.is_finite() || !c.y.is_finite() {
            return Err("NON_FINITE_COORD");
        }
    }
    if n > MAX_COORDS {
        return Err("TOO_MANY_COORDS");
    }
    Ok(geom)
}

/// Serialize a geo-types geometry to a compact GeoJSON string.
pub fn to_geojson<G>(geom: &G) -> String
where
    for<'a> geojson::Geometry: From<&'a G>,
{
    geojson::Geometry::from(geom).to_string()
}
