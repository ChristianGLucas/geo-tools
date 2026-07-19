// Separate test file: nodes/convex_hull_test.rs. The generated service wires
// it into the crate via `#[cfg(test)] #[path="nodes/convex_hull_test.rs"] mod
// convex_hull_test;`. It reaches the node + SDK through `crate::` paths (this is
// a sibling module of the node, not a child — so `super::*` would not resolve).
#[cfg(test)]
mod tests {
    use crate::axiom_context::*;
    use crate::gen::messages::Geometry;
    use crate::convex_hull::convex_hull;
    use std::collections::HashMap;

    fn geom(geojson: &str) -> Geometry {
        Geometry { geojson: geojson.to_string(), error: String::new() }
    }
    fn polygon_ring(geojson: &str) -> Vec<(f64, f64)> {
        let v: serde_json::Value = serde_json::from_str(geojson).unwrap();
        assert_eq!(v["type"], "Polygon");
        v["coordinates"][0]
            .as_array().unwrap()
            .iter()
            .map(|c| (c[0].as_f64().unwrap(), c[1].as_f64().unwrap()))
            .collect()
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

    // Golden: the hull of four corners plus an interior point is the square —
    // the interior point (2,2) must be dropped.
    #[test]
    fn test_hull_drops_interior_point() {
        let ax = test_context();
        let out = convex_hull(&ax, geom(
            r#"{"type":"MultiPoint","coordinates":[[0,0],[4,0],[4,4],[0,4],[2,2]]}"#,
        )).unwrap();
        assert_eq!(out.error, "");
        let ring = polygon_ring(&out.geojson);
        // Closed ring of the 4 hull corners = 5 positions.
        assert_eq!(ring.len(), 5, "ring {:?}", ring);
        assert!(!ring.iter().any(|&(x, y)| x == 2.0 && y == 2.0), "interior point kept: {:?}", ring);
        for corner in [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)] {
            assert!(ring.contains(&corner), "missing corner {:?} in {:?}", corner, ring);
        }
    }

    // A single point / collinear points can't form a polygon hull: report
    // DEGENERATE rather than emitting an invalid <4-position ring.
    #[test]
    fn test_degenerate_hull_is_structured_error() {
        let ax = test_context();
        assert_eq!(convex_hull(&ax, geom(r#"{"type":"Point","coordinates":[3,4]}"#)).unwrap().error, "DEGENERATE");
        assert_eq!(convex_hull(&ax, geom(r#"{"type":"LineString","coordinates":[[0,0],[1,1],[2,2]]}"#)).unwrap().error, "DEGENERATE");
    }

    #[test]
    fn test_error_paths() {
        let ax = test_context();
        assert_eq!(convex_hull(&ax, geom("")).unwrap().error, "EMPTY_INPUT");
        assert_eq!(convex_hull(&ax, geom("xx")).unwrap().error, "INVALID_GEOJSON");
    }
}
