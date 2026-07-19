// Separate test file: nodes/contains_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/contains_test.rs"] mod
// contains_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::ContainsInput;
    use crate::contains::contains;
    use std::collections::HashMap;

    const SQUARE: &str = r#"{"type":"Polygon","coordinates":[[[0,0],[4,0],[4,4],[0,4],[0,0]]]}"#;

    fn ci(geojson: &str, lon: f64, lat: f64) -> ContainsInput {
        ContainsInput { geojson: geojson.to_string(), lon, lat }
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

    #[test]
    fn test_interior_point_is_contained() {
        let ax = test_context();
        let out = contains(&ax, ci(SQUARE, 2.0, 2.0)).unwrap();
        assert_eq!(out.error, "");
        assert!(out.contains);
    }

    #[test]
    fn test_exterior_point_is_not_contained() {
        let ax = test_context();
        let out = contains(&ax, ci(SQUARE, 5.0, 5.0)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.contains);
    }

    // Boundary is excluded (interior-only containment).
    #[test]
    fn test_boundary_point_is_not_contained() {
        let ax = test_context();
        let out = contains(&ax, ci(SQUARE, 0.0, 2.0)).unwrap();
        assert_eq!(out.error, "");
        assert!(!out.contains);
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(contains(&ax, ci(r#"{"type":"Point","coordinates":[0,0]}"#, 0.0, 0.0)).unwrap().error, "WRONG_GEOMETRY_TYPE");
        assert_eq!(contains(&ax, ci(SQUARE, 200.0, 0.0)).unwrap().error, "OUT_OF_RANGE");
        assert_eq!(contains(&ax, ci("nope", 1.0, 1.0)).unwrap().error, "INVALID_GEOJSON");
    }
}
