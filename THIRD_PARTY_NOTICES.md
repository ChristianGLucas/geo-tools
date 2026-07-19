# Third-party notices

`geo-tools` is MIT-licensed (© 2026 Christian George Lucas). It builds on the
open-source Rust crates below. The full transitive dependency tree was resolved
with `cargo license` against the committed `Cargo.lock` and independently
verified to be entirely permissive.

## Primary libraries

| Crate | Role | License |
|---|---|---|
| [`geo`](https://github.com/georust/geo) | Geodesic measurement & computational geometry (distance, bearing, destination, length, geodesic area, centroid, bounding rect, convex hull, RDP simplify, containment) | MIT OR Apache-2.0 |
| [`geo-types`](https://github.com/georust/geo) | Core geometry primitive types | MIT OR Apache-2.0 |
| [`geographiclib-rs`](https://github.com/georust/geographiclib-rs) | Karney's geodesic algorithms (the ellipsoidal engine behind `geo`'s geodesic ops) | MIT |
| [`geojson`](https://github.com/georust/geojson) | GeoJSON parsing and serialization | MIT OR Apache-2.0 |

## License verification

Every crate in the resolved tree carries one of: **MIT**, **Apache-2.0**,
**BSD-2-Clause**, **BSD-3-Clause**, **ISC**, **Zlib**, **Unicode-3.0**, or
**Unlicense** — each individually, or as a permissive `OR`/`AND` combination.

No crate is copyleft-only. The single lockfile entry naming a copyleft option,
`r-efi` (`Apache-2.0 OR LGPL-2.1-or-later OR MIT`), is (a) a triple-licensed
crate from which the permissive Apache-2.0/MIT option is taken, and (b) a
target-gated entry that is not part of the Linux build graph (`cargo tree -i
r-efi` is empty), so no LGPL code is ever compiled or linked.

The Axiom Rust service runtime (`tonic`, `prost`, `tokio`, `hyper`, `tower`,
`tracing`, and their dependencies) is likewise MIT / Apache-2.0 throughout.
