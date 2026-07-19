// Separate test file: nodes/length_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/length_test.rs"] mod
// length_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::Geometry;
    use crate::length::length;
    use std::collections::HashMap;

    fn geom(geojson: &str) -> Geometry {
        Geometry { geojson: geojson.to_string(), error: String::new() }
    }

    // Independent oracle: haversine sum over the segments of a line.
    fn haversine_len(pts: &[(f64, f64)]) -> f64 {
        const R: f64 = 6_371_008.8;
        let mut total = 0.0;
        for w in pts.windows(2) {
            let (lon1, lat1) = w[0];
            let (lon2, lat2) = w[1];
            let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
            let dphi = (lat2 - lat1).to_radians();
            let dlam = (lon2 - lon1).to_radians();
            let a = (dphi / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dlam / 2.0).sin().powi(2);
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

    // Golden: one degree of latitude, (0,0)->(0,1), ~110,574 m.
    #[test]
    fn test_one_degree_line() {
        let ax = test_context();
        let out = length(&ax, geom(r#"{"type":"LineString","coordinates":[[0,0],[0,1]]}"#)).unwrap();
        assert_eq!(out.error, "");
        assert!((out.meters - 110_574.0).abs() < 5.0, "got {}", out.meters);
    }

    // Oracle: geodesic length agrees with independent haversine sum within 0.5%.
    #[test]
    fn test_multi_segment_agrees_with_haversine() {
        let ax = test_context();
        let pts = [(2.3522, 48.8566), (13.4050, 52.5200), (37.6173, 55.7558)]; // Paris->Berlin->Moscow
        let gj = format!(
            r#"{{"type":"LineString","coordinates":[[{},{}],[{},{}],[{},{}]]}}"#,
            pts[0].0, pts[0].1, pts[1].0, pts[1].1, pts[2].0, pts[2].1
        );
        let out = length(&ax, geom(&gj)).unwrap();
        assert_eq!(out.error, "");
        let oracle = haversine_len(&pts);
        let rel = (out.meters - oracle).abs() / oracle;
        assert!(rel < 0.005, "geo={} haversine={} rel={}", out.meters, oracle, rel);
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(length(&ax, geom(r#"{"type":"Point","coordinates":[0,0]}"#)).unwrap().error, "WRONG_GEOMETRY_TYPE");
        assert_eq!(length(&ax, geom("not json")).unwrap().error, "INVALID_GEOJSON");
        assert_eq!(length(&ax, geom("")).unwrap().error, "EMPTY_INPUT");
    }
}
