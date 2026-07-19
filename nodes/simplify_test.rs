// Separate test file: nodes/simplify_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/simplify_test.rs"] mod
// simplify_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::SimplifyInput;
    use crate::simplify::simplify;
    use std::collections::HashMap;

    fn si(geojson: &str, epsilon: f64) -> SimplifyInput {
        SimplifyInput { geojson: geojson.to_string(), epsilon }
    }
    fn linestring_len(geojson: &str) -> usize {
        let v: serde_json::Value = serde_json::from_str(geojson).unwrap();
        assert_eq!(v["type"], "LineString");
        v["coordinates"].as_array().unwrap().len()
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

    // Golden: a near-collinear middle vertex is dropped at a large-enough epsilon.
    #[test]
    fn test_drops_collinear_vertex() {
        let ax = test_context();
        let line = r#"{"type":"LineString","coordinates":[[0,0],[5,0.0001],[10,0]]}"#;
        let out = simplify(&ax, si(line, 0.01)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(linestring_len(&out.geojson), 2, "{}", out.geojson);
    }

    // Golden: epsilon 0 keeps every vertex.
    #[test]
    fn test_zero_epsilon_keeps_all() {
        let ax = test_context();
        let line = r#"{"type":"LineString","coordinates":[[0,0],[5,0.0001],[10,0]]}"#;
        let out = simplify(&ax, si(line, 0.0)).unwrap();
        assert_eq!(out.error, "");
        assert_eq!(linestring_len(&out.geojson), 3);
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(simplify(&ax, si(r#"{"type":"LineString","coordinates":[[0,0],[1,1]]}"#, -1.0)).unwrap().error, "OUT_OF_RANGE");
        assert_eq!(simplify(&ax, si("bad", 0.1)).unwrap().error, "INVALID_GEOJSON");
    }

    // Build a dense "sawtooth" LineString of `n` points alternating y=+4/-4.
    fn sawtooth(n: usize) -> String {
        let mut c = String::from("[0,0]");
        for i in 1..n {
            let x = i as f64 * 0.001;
            let y = if i % 2 == 0 { 4.0 } else { -4.0 };
            c.push_str(&format!(",[{},{}]", x, y));
        }
        format!(r#"{{"type":"LineString","coordinates":[{}]}}"#, c)
    }

    // Simplify enforces its own tight vertex cap (10,000) so the recursive RDP
    // cannot be driven into a costly/deep run: > 10,000 vertices -> TOO_MANY_COORDS.
    #[test]
    fn test_vertex_cap() {
        let ax = test_context();
        assert_eq!(simplify(&ax, si(&sawtooth(10_001), 0.00001)).unwrap().error, "TOO_MANY_COORDS");
    }

    // Regression for the recursive-RDP stack overflow: a dense sawtooth at the
    // cap boundary that keeps every vertex (tiny epsilon) drives deep recursion,
    // which must COMPLETE on the large worker stack rather than abort the process.
    #[test]
    fn test_deep_recursion_does_not_crash() {
        let ax = test_context();
        let out = simplify(&ax, si(&sawtooth(10_000), 0.00001)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.geojson.is_empty());
    }
}
