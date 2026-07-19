// Separate test file: nodes/distance_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/distance_test.rs"] mod
// distance_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::PointPair;
    use crate::distance::distance;
    use std::collections::HashMap;

    fn pp(from_lon: f64, from_lat: f64, to_lon: f64, to_lat: f64) -> PointPair {
        PointPair { from_lon, from_lat, to_lon, to_lat }
    }

    // Independent oracle: the haversine great-circle distance on a sphere,
    // implemented from scratch here (NOT via geo). geo returns the geodesic
    // (ellipsoidal) distance, which differs from the spherical value by at most
    // ~0.5% — a tight enough bound to catch axis swaps, unit errors, or a wrong
    // formula, while being a genuinely separate implementation.
    fn haversine_oracle(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
        const R: f64 = 6_371_008.8; // mean Earth radius, meters
        let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
        let dphi = (lat2 - lat1).to_radians();
        let dlam = (lon2 - lon1).to_radians();
        let a = (dphi / 2.0).sin().powi(2)
            + phi1.cos() * phi2.cos() * (dlam / 2.0).sin().powi(2);
        2.0 * R * a.sqrt().asin()
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

    // Golden: one degree of latitude along the equatorial meridian, (0,0)->(0,1),
    // is ~110.574 km on WGS-84 (a documented reference value, independent of geo).
    #[test]
    fn test_one_degree_of_latitude() {
        let ax = test_context();
        let out = distance(&ax, pp(0.0, 0.0, 0.0, 1.0)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.meters - 110_574.0).abs() < 5.0, "got {}", out.meters);
    }

    // Golden: JFK (-73.7781, 40.6413) -> LAX (-118.4085, 33.9416) ~= 3,983 km.
    #[test]
    fn test_jfk_to_lax() {
        let ax = test_context();
        let out = distance(&ax, pp(-73.7781, 40.6413, -118.4085, 33.9416)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.meters - 3_983_000.0).abs() < 5_000.0, "got {}", out.meters);
    }

    #[test]
    fn test_identical_points_are_zero() {
        let ax = test_context();
        let out = distance(&ax, pp(2.35, 48.85, 2.35, 48.85)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.meters < 1e-6, "got {}", out.meters);
    }

    #[test]
    fn test_non_finite_is_structured_error() {
        let ax = test_context();
        let out = distance(&ax, pp(f64::NAN, 0.0, 0.0, 0.0)).unwrap();
        assert_eq!(out.error, "NON_FINITE_COORD");
        assert_eq!(out.meters, 0.0);
    }

    #[test]
    fn test_out_of_range_latitude_is_structured_error() {
        let ax = test_context();
        let out = distance(&ax, pp(0.0, 91.0, 0.0, 0.0)).unwrap();
        assert_eq!(out.error, "OUT_OF_RANGE");
    }

    // Oracle: geo's geodesic distance agrees with the independent haversine
    // distance to within 0.5% across a spread of global pairs.
    #[test]
    fn test_agrees_with_independent_haversine_oracle() {
        let ax = test_context();
        let cases = [
            (2.3522, 48.8566, 13.4050, 52.5200),    // Paris -> Berlin
            (-0.1276, 51.5072, -73.9857, 40.7484),  // London -> NYC
            (139.6917, 35.6895, 151.2093, -33.8688),// Tokyo -> Sydney
            (-118.2437, 34.0522, -122.4194, 37.7749),// LA -> SF
        ];
        for (lo1, la1, lo2, la2) in cases {
            let got = distance(&ax, pp(lo1, la1, lo2, la2)).unwrap();
            assert_eq!(got.error, "");
            let oracle = haversine_oracle(lo1, la1, lo2, la2);
            let rel = (got.meters - oracle).abs() / oracle;
            assert!(rel < 0.005, "geo={} haversine={} rel={}", got.meters, oracle, rel);
        }
    }

    // Published physical constant (independent of geo): the WGS-84 quarter
    // meridian, equator (0,0) to pole (0,90), is 10,001,965.7 m.
    #[test]
    fn test_quarter_meridian_wgs84_constant() {
        let ax = test_context();
        let out = distance(&ax, pp(0.0, 0.0, 0.0, 90.0)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.meters - 10_001_965.7).abs() < 2.0, "got {}", out.meters);
    }
}
