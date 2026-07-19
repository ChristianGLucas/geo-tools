# geo-tools

Composable **geospatial geometry** nodes for the [Axiom](https://axiomide.com)
marketplace, published as `christiangeorgelucas/geo-tools`. Measure, transform,
and test GeoJSON geometries — distance, bearing, length, area, centroid,
bounding box, convex hull, simplification, and point-in-polygon — entirely
offline and deterministically.

Written in **Rust**, wrapping one battle-tested, permissively-licensed library:

| Concern | Library | License |
|---|---|---|
| Geodesic measurement & computational geometry | [`geo`](https://github.com/georust/geo) (the georust project) | MIT OR Apache-2.0 |
| GeoJSON parsing / serialization | [`geojson`](https://github.com/georust/geojson) | MIT OR Apache-2.0 |

Every node is **stateless**, **offline** (no network, no API keys, no signup),
and **deterministic**. Coordinates everywhere are `[longitude, latitude]` in
decimal degrees on the **WGS-84** datum (GeoJSON / RFC 7946 axis order). All
measurements use the **geodesic** (ellipsoidal, Karney) model, so distances are
in meters and areas in square meters — the professionally-correct model rather
than a spherical approximation.

## The canonical `Geometry` envelope

Geometry flows between nodes as a single message — a GeoJSON geometry object
serialized as a JSON string:

```json
{ "type": "Point", "coordinates": [-73.9857, 40.7484] }
```

Every geometry-producing node emits this envelope and every geometry-consuming
node accepts it, so nodes chain by passing `geojson` straight through. On
failure a producer leaves `geojson` empty and sets a machine-readable `error`
token (e.g. `INVALID_GEOJSON`, `WRONG_GEOMETRY_TYPE`, `EMPTY_GEOMETRY`).

**proto3 JSON note:** default scalar values (`false`, `""`, `0`) are omitted from
the JSON emitted over the HTTP bridge, so a consumer must treat a missing
`error` as success, a missing `geojson` as "no geometry produced", and a missing
`contains`/numeric field as its zero value.

## Nodes

| Node | Input → Output | Purpose |
|---|---|---|
| `Distance` | `PointPair` → `Distance` | Geodesic distance in meters between two points |
| `Bearing` | `PointPair` → `Bearing` | Initial bearing (forward azimuth), degrees clockwise from north |
| `Destination` | `DestinationInput` → `Geometry` | Point reached from an origin along a bearing + distance |
| `Length` | `Geometry` → `Length` | Total geodesic length of a LineString/MultiLineString |
| `Area` | `Geometry` → `Area` | Geodesic area (m²) and perimeter (m) of a Polygon/MultiPolygon |
| `Centroid` | `Geometry` → `Geometry` | Centroid of any geometry, as a Point |
| `BoundingBox` | `Geometry` → `BoundingBox` | Axis-aligned bounds + the box as a GeoJSON Polygon |
| `ConvexHull` | `Geometry` → `Geometry` | Smallest convex polygon enclosing all vertices |
| `Simplify` | `SimplifyInput` → `Geometry` | Ramer–Douglas–Peucker simplification (epsilon in degrees) |
| `Contains` | `ContainsInput` → `Contains` | Whether a point lies inside a polygon (interior only) |

Every input is validated: point-coordinate nodes (`Distance`, `Bearing`,
`Destination`, `Contains`) and every geometry's coordinates return
`NON_FINITE_COORD` or `OUT_OF_RANGE` (|lat|>90 or |lon|>180) rather than a bad
number — so a geodesic op never silently yields `NaN`. Geometry inputs are also
bounded: a geometry with more than 100,000 coordinates returns `TOO_MANY_COORDS`,
and `Simplify` (whose recursion is the costliest path) caps at 10,000 vertices,
so a crafted input cannot exhaust memory or CPU. The node also rejects a GeoJSON
string over 1 MB with `INPUT_TOO_LONG` as defense in depth, though over the HTTP
bridge the platform gateway may reject an oversized body with a 413 before the
node runs. A single Feature wrapping one geometry is accepted; a FeatureCollection
is rejected as ambiguous.

Errors do not propagate across a flow edge: if a producing node fails it emits
empty `geojson`, and a downstream node then reports `EMPTY_INPUT` rather than the
original cause — inspect each node's own `error` when debugging a chain.

### Caveats (honest edges)

- **`Area` follows the GeoJSON right-hand rule (RFC 7946).** Ring winding is
  meaningful: a counter-clockwise exterior ring measures the enclosed region; a
  clockwise ring describes — and measures — the complementary region. Use CCW
  exteriors for the common case.
- **`ConvexHull`** computes the hull planar-ly on lon/lat (correct for local
  extents; near the poles or across the antimeridian a planar hull can differ
  from the true spherical hull). Fewer than three non-collinear points cannot
  form a polygon and return `DEGENERATE`.
- **Garbage-in cases are not specially detected** (they do not crash, but the
  result is only as meaningful as the input): a `Bearing` whose origin is exactly
  a geographic pole is at the azimuth singularity, and `Area` of a
  self-intersecting (bow-tie) polygon returns geo's signed-loop cancellation.
  Supply valid, simple geometries.

## Correctness

The test suite (`axiom test`) enforces every accuracy claim with **independent
oracles** — code that does not go through `geo`, so the suite never checks the
library against itself:

- **`Distance`** is cross-checked against a from-scratch **haversine**
  implementation (agreement within 0.5% of the geodesic value) and against the
  published **WGS-84 quarter-meridian** constant (10,001,965.7 m).
- **`Bearing`** is cross-checked against a from-scratch **spherical
  initial-bearing** formula and against exact cardinal directions (N/E/S/W).
- **`Destination`** is verified by a **round-trip invariant**: travelling out
  along a `(bearing, distance)` and then measuring back with the independent
  `Distance` and `Bearing` solvers reproduces both inputs (a consistency check
  across three separate algorithms that trusts no golden).
- **`Length`** and **`Area`** are cross-checked against independent haversine /
  spherical-polygon formulas.
- The transform/predicate nodes (`Centroid`, `BoundingBox`, `ConvexHull`,
  `Simplify`, `Contains`) assert exact geometric goldens (e.g. the centroid of a
  square is its center; the hull drops interior points; a boundary point is not
  contained).

## Composability

Geometry-producing nodes emit the same `Geometry` envelope the geometry-consuming
nodes accept, so they chain by mapping `geojson → geojson`. A runnable proof flow
ships with this package at `flows/geo-hull-area.flow.yaml`:
`ConvexHull → Area` — wrap a set of points in their convex hull, then measure the
hull's geodesic area. It compiles and runs end to end.

## Development

```bash
axiom validate     # static checks
axiom test         # unit tests (goldens + independent oracles + error paths)
axiom dev          # local HTTP bridge (prints the port it binds)
```

## License

MIT — © 2026 Christian George Lucas. Built for the Axiom marketplace.
`geo` and `geojson` are dual MIT/Apache-2.0 licensed. See `THIRD_PARTY_NOTICES.md`.
