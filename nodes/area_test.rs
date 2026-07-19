// Separate test file: nodes/area_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/area_test.rs"] mod
// area_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::Geometry;
    use crate::area::area;
    use std::collections::HashMap;

    fn geom(geojson: &str) -> Geometry {
        Geometry { geojson: geojson.to_string(), error: String::new() }
    }

    // Independent oracle: the classic spherical ring-area formula (as used by
    // mapbox's geojson-area), on a sphere — separate from geo's ellipsoidal
    // Karney method. `ring` is (lon,lat), first point NOT repeated.
    fn spherical_ring_area(ring: &[(f64, f64)]) -> f64 {
        const R: f64 = 6_378_137.0;
        let n = ring.len();
        if n < 3 { return 0.0; }
        let mut total = 0.0;
        for i in 0..n {
            let (lo1, la1) = ring[i];
            let (lo2, la2) = ring[(i + 1) % n];
            total += (lo2 - lo1).to_radians() * (2.0 + la1.to_radians().sin() + la2.to_radians().sin());
        }
        (total * R * R / 2.0).abs()
    }

    fn haversine_perimeter(ring_closed: &[(f64, f64)]) -> f64 {
        const R: f64 = 6_371_008.8;
        let mut total = 0.0;
        for w in ring_closed.windows(2) {
            let (lo1, la1) = w[0];
            let (lo2, la2) = w[1];
            let (p1, p2) = (la1.to_radians(), la2.to_radians());
            let a = ((la2 - la1).to_radians() / 2.0).sin().powi(2)
                + p1.cos() * p2.cos() * ((lo2 - lo1).to_radians() / 2.0).sin().powi(2);
            total += 2.0 * R * a.sqrt().asin();
        }
        total
    }

    // TESTS — delete this block when done ─────────────────────────────────────
    // Tests are required to publish this package. The publish pipeline runs your
    // tests as a quality gate — a package will not be published if tests fail or
    // do not meet the minimum requirements.
    //
    // Requirements checked before publishing:
    //   - At least one test per node
    //   - All tests must pass
    //   - Output fields must be meaningfully asserted — not just Ok-checked
    //
    // The generated test below is a starting point. Replace the TODO with real
    // assertions: given a specific input, what should the output fields contain?
    //
    // Run your tests locally at any time:  axiom test

    struct TestLogger;
    impl AxiomLogger for TestLogger {
        fn debug(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn info(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn warn(&self, _m: &str, _a: &HashMap<&str, String>) {}
        fn error(&self, _m: &str, _a: &HashMap<&str, String>) {}
    }
    struct TestSecrets;
    impl AxiomSecrets for TestSecrets {
        fn get(&self, _n: &str) -> (String, bool) { (String::new(), false) }
    }
    struct EmptyFlow { pos: FlowPosition }
    impl FlowReflection for EmptyFlow {
        fn nodes(&self) -> &[ReflectionNode] { &[] }
        fn edges(&self) -> &[ReflectionEdge] { &[] }
        fn loop_edges(&self) -> &[ReflectionEdge] { &[] }
        fn position(&self) -> &FlowPosition { &self.pos }
        fn graph_id(&self) -> &str { "" }
    }
    struct TestReflection { flow: EmptyFlow }
    impl Reflection for TestReflection { fn flow(&self) -> &dyn FlowReflection { &self.flow } }
    struct TestFlowMut;
    impl FlowMutation for TestFlowMut {
        fn add_node(&self, _p: &str, _v: &str, _c: Option<CanvasPosition>) -> u32 { 0 }
        fn add_edge(&self, _s: u32, _d: u32, _c: Option<EdgeCondition>) {}
    }
    struct TestMutation { flow: TestFlowMut }
    impl Mutation for TestMutation { fn flow(&self) -> &dyn FlowMutation { &self.flow } }

    // Mock AxiomContext a node author edits to drive a specific test scenario.
    struct TestContext {
        log: TestLogger, secrets: TestSecrets,
        reflection: TestReflection, mutation: TestMutation,
    }
    impl AxiomContext for TestContext {
        fn log(&self) -> &dyn AxiomLogger { &self.log }
        fn secrets(&self) -> &dyn AxiomSecrets { &self.secrets }
        fn execution_id(&self) -> &str { "test-execution-id" }
        fn flow_id(&self) -> &str { "test-flow-id" }
        fn tenant_id(&self) -> &str { "test-tenant-id" }
        fn reflection(&self) -> &dyn Reflection { &self.reflection }
        fn mutation(&self) -> &dyn Mutation { &self.mutation }
    }
    fn test_context() -> TestContext {
        TestContext {
            log: TestLogger, secrets: TestSecrets,
            reflection: TestReflection { flow: EmptyFlow { pos: FlowPosition::default() } },
            mutation: TestMutation { flow: TestFlowMut },
        }
    }

    // Oracle: geodesic area agrees with the independent spherical formula within
    // 1%, and perimeter with independent haversine sum within 0.5%. A 1x1 degree
    // box near the equator.
    #[test]
    fn test_unit_box_area_and_perimeter() {
        let ax = test_context();
        let out = area(&ax, geom(
            r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#,
        )).unwrap();
        assert_eq!(out.error, "");
        let ring = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let oracle_area = spherical_ring_area(&ring);
        let rel_a = (out.square_meters - oracle_area).abs() / oracle_area;
        assert!(rel_a < 0.01, "geo={} oracle={} rel={}", out.square_meters, oracle_area, rel_a);

        let closed = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)];
        let oracle_perim = haversine_perimeter(&closed);
        let rel_p = (out.perimeter_meters - oracle_perim).abs() / oracle_perim;
        assert!(rel_p < 0.005, "geo={} oracle={} rel={}", out.perimeter_meters, oracle_perim, rel_p);
    }

    // Unsigned area is winding-order independent: a clockwise ring gives the
    // same magnitude as the counter-clockwise one above.
    #[test]
    fn test_winding_order_independent() {
        let ax = test_context();
        let ccw = area(&ax, geom(r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#)).unwrap();
        let cw = area(&ax, geom(r#"{"type":"Polygon","coordinates":[[[0,0],[0,1],[1,1],[1,0],[0,0]]]}"#)).unwrap();
        assert!((ccw.square_meters - cw.square_meters).abs() < 1.0);
        // Both resolve to the small enclosed region (~1.2e10 m²), not the Earth complement.
        assert!(ccw.square_meters > 1.0e10 && ccw.square_meters < 2.0e10, "got {}", ccw.square_meters);
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(area(&ax, geom(r#"{"type":"LineString","coordinates":[[0,0],[1,1]]}"#)).unwrap().error, "WRONG_GEOMETRY_TYPE");
        assert_eq!(area(&ax, geom("{bad")).unwrap().error, "INVALID_GEOJSON");
    }
}
