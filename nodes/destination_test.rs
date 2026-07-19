// Separate test file: nodes/destination_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/destination_test.rs"] mod
// destination_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::{ DestinationInput, PointPair };
    use crate::destination::destination;
    use crate::distance::distance;
    use crate::bearing::bearing;
    use std::collections::HashMap;

    fn di(lon: f64, lat: f64, bearing_deg: f64, distance_m: f64) -> DestinationInput {
        DestinationInput { lon, lat, bearing_deg, distance_m }
    }

    // Pull [lon, lat] out of a GeoJSON Point string.
    fn point_coords(geojson: &str) -> (f64, f64) {
        let v: serde_json::Value = serde_json::from_str(geojson).unwrap();
        assert_eq!(v["type"], "Point");
        let c = v["coordinates"].as_array().unwrap();
        (c[0].as_f64().unwrap(), c[1].as_f64().unwrap())
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

    // Golden: heading due north from (0,0) for ~110,574 m lands at ~(0, 1).
    #[test]
    fn test_due_north_one_degree() {
        let ax = test_context();
        let out = destination(&ax, di(0.0, 0.0, 0.0, 110_574.0)).unwrap();
        assert_eq!(out.error, "");
        let (lon, lat) = point_coords(&out.geojson);
        assert!(lon.abs() < 1e-6, "lon {}", lon);
        assert!((lat - 1.0).abs() < 1e-3, "lat {}", lat);
    }

    // Independent oracle by round-trip invariant: Destination, Distance, and
    // Bearing are three separate solvers in geo. Going out along (bearing,
    // distance) and measuring back must reproduce both — a consistency check
    // that trusts no golden and no single algorithm.
    #[test]
    fn test_roundtrip_distance_and_bearing() {
        let ax = test_context();
        let cases = [
            (2.3522, 48.8566, 45.0, 200_000.0),
            (-73.9857, 40.7484, 137.5, 5_000.0),
            (151.2093, -33.8688, 300.0, 1_234_567.0),
            (0.0, 0.0, 90.0, 500_000.0),
        ];
        for (lon, lat, brg, dist) in cases {
            let out = destination(&ax, di(lon, lat, brg, dist)).unwrap();
            assert_eq!(out.error, "");
            let (dlon, dlat) = point_coords(&out.geojson);
            let back = distance(&ax, PointPair { from_lon: lon, from_lat: lat, to_lon: dlon, to_lat: dlat }).unwrap();
            assert!((back.meters - dist).abs() < 0.5, "dist got {} want {}", back.meters, dist);
            let b = bearing(&ax, PointPair { from_lon: lon, from_lat: lat, to_lon: dlon, to_lat: dlat }).unwrap();
            let mut diff = (b.degrees - brg).abs();
            if diff > 180.0 { diff = 360.0 - diff; }
            assert!(diff < 1e-3, "bearing got {} want {}", b.degrees, brg);
        }
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(destination(&ax, di(0.0, 91.0, 0.0, 100.0)).unwrap().error, "OUT_OF_RANGE");
        assert_eq!(destination(&ax, di(0.0, 0.0, 0.0, -5.0)).unwrap().error, "OUT_OF_RANGE");
        assert_eq!(destination(&ax, di(0.0, 0.0, f64::NAN, 100.0)).unwrap().error, "NON_FINITE_COORD");
    }
}
