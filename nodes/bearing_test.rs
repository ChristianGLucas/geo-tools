// Separate test file: nodes/bearing_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/bearing_test.rs"] mod
// bearing_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::PointPair;
    use crate::bearing::bearing;
    use std::collections::HashMap;

    fn pp(from_lon: f64, from_lat: f64, to_lon: f64, to_lat: f64) -> PointPair {
        PointPair { from_lon, from_lat, to_lon, to_lat }
    }

    // Independent oracle: the spherical initial-bearing formula, implemented from
    // scratch here (NOT via geo). geo returns the geodesic/ellipsoidal azimuth,
    // which agrees with the spherical value to a fraction of a degree over the
    // ranges tested — enough to catch axis swaps, unit errors, or a wrong formula.
    fn spherical_initial_bearing(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
        let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
        let dlon = (lon2 - lon1).to_radians();
        let y = dlon.sin() * phi2.cos();
        let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * dlon.cos();
        let deg = y.atan2(x).to_degrees();
        ((deg % 360.0) + 360.0) % 360.0
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

    // Exact cardinal-direction goldens (independent of any library — basic geography).
    #[test]
    fn test_cardinal_directions() {
        let ax = test_context();
        assert!((bearing(&ax, pp(0.0, 0.0, 0.0, 10.0)).unwrap().degrees - 0.0).abs() < 1e-6);   // N
        assert!((bearing(&ax, pp(0.0, 0.0, 10.0, 0.0)).unwrap().degrees - 90.0).abs() < 1e-6);  // E (along equator)
        assert!((bearing(&ax, pp(0.0, 0.0, 0.0, -10.0)).unwrap().degrees - 180.0).abs() < 1e-6);// S
        assert!((bearing(&ax, pp(0.0, 0.0, -10.0, 0.0)).unwrap().degrees - 270.0).abs() < 1e-6);// W (along equator)
    }

    // Oracle: geo's geodesic bearing agrees with the independent spherical formula.
    #[test]
    fn test_agrees_with_independent_spherical_oracle() {
        let ax = test_context();
        let cases = [
            (2.3522, 48.8566, 13.4050, 52.5200),   // Paris -> Berlin
            (-0.1276, 51.5072, -73.9857, 40.7484), // London -> NYC
            (139.6917, 35.6895, 151.2093, -33.8688),// Tokyo -> Sydney
        ];
        for (lo1, la1, lo2, la2) in cases {
            let got = bearing(&ax, pp(lo1, la1, lo2, la2)).unwrap();
            assert_eq!(got.error, "");
            let oracle = spherical_initial_bearing(lo1, la1, lo2, la2);
            let mut diff = (got.degrees - oracle).abs();
            if diff > 180.0 { diff = 360.0 - diff; } // wrap-around
            assert!(diff < 0.5, "geo={} oracle={} diff={}", got.degrees, oracle, diff);
        }
    }

    #[test]
    fn test_bearing_is_normalized_range() {
        let ax = test_context();
        let d = bearing(&ax, pp(0.0, 0.0, -0.0001, -10.0)).unwrap().degrees; // just west of due south
        assert!((0.0..360.0).contains(&d), "got {}", d);
        assert!(d > 180.0 && d < 181.0, "expected ~180-181, got {}", d);
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(bearing(&ax, pp(0.0, 0.0, f64::INFINITY, 0.0)).unwrap().error, "NON_FINITE_COORD");
        assert_eq!(bearing(&ax, pp(0.0, 0.0, 200.0, 0.0)).unwrap().error, "OUT_OF_RANGE");
    }
}
