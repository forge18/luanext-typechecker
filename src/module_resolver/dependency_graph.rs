use super::error::{ModuleError, ModuleId};
use rustc_hash::{FxHashMap, FxHashSet};

/// Type of dependency edge between modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Type-only dependency (e.g., `import type { Foo } from './bar'`)
    /// These dependencies can be circular without causing compilation issues
    TypeOnly,
    /// Runtime value dependency (e.g., `import { foo } from './bar'`)
    /// These dependencies must be acyclic for proper initialization order
    Value,
}

/// An edge in the dependency graph with metadata
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Target module ID
    pub target: ModuleId,
    /// Kind of dependency
    pub kind: EdgeKind,
}

/// Dependency graph for module compilation ordering
#[derive(Debug)]
pub struct DependencyGraph {
    /// Adjacency list: module_id -> list of dependency edges
    edges: FxHashMap<ModuleId, Vec<DependencyEdge>>,
    /// All known modules
    nodes: FxHashSet<ModuleId>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: FxHashMap::default(),
            nodes: FxHashSet::default(),
        }
    }

    /// Add a module and its dependencies to the graph
    pub fn add_module(&mut self, id: ModuleId, dependencies: Vec<(ModuleId, EdgeKind)>) {
        self.nodes.insert(id.clone());

        let edges: Vec<DependencyEdge> = dependencies
            .into_iter()
            .map(|(target, kind)| {
                self.nodes.insert(target.clone());
                DependencyEdge { target, kind }
            })
            .collect();

        self.edges.insert(id, edges);
    }

    /// Perform topological sort to determine compilation order
    ///
    /// Returns modules in dependency order (dependencies first)
    /// or an error if a circular dependency is detected
    pub fn topological_sort(&self) -> Result<Vec<ModuleId>, ModuleError> {
        let mut sorted = Vec::new();
        let mut visited = FxHashSet::default();
        let mut visiting = FxHashSet::default();

        for node in &self.nodes {
            if !visited.contains(node) {
                self.visit(
                    node,
                    &mut visited,
                    &mut visiting,
                    &mut sorted,
                    &mut Vec::new(),
                )?;
            }
        }

        Ok(sorted)
    }

    /// DFS visit for topological sort with cycle detection
    ///
    /// Only follows Value edges. Type-only edges are ignored.
    fn visit(
        &self,
        node: &ModuleId,
        visited: &mut FxHashSet<ModuleId>,
        visiting: &mut FxHashSet<ModuleId>,
        sorted: &mut Vec<ModuleId>,
        path: &mut Vec<ModuleId>,
    ) -> Result<(), ModuleError> {
        if visiting.contains(node) {
            // Circular dependency detected - extract cycle from path
            let cycle_start = path.iter().position(|n| n == node).unwrap();
            let mut cycle: Vec<ModuleId> = path[cycle_start..].to_vec();
            cycle.push(node.clone());
            return Err(ModuleError::CircularDependency { cycle });
        }

        if visited.contains(node) {
            return Ok(());
        }

        visiting.insert(node.clone());
        path.push(node.clone());

        // Visit dependencies - ONLY follow Value edges
        if let Some(edges) = self.edges.get(node) {
            for edge in edges {
                if edge.kind == EdgeKind::Value {
                    self.visit(&edge.target, visited, visiting, sorted, path)?;
                }
            }
        }

        path.pop();
        visiting.remove(node);
        visited.insert(node.clone());
        sorted.push(node.clone());

        Ok(())
    }

    /// Get direct dependencies of a module with edge kinds
    pub fn get_dependencies(&self, id: &ModuleId) -> Option<&Vec<DependencyEdge>> {
        self.edges.get(id)
    }

    /// Get only value dependencies (runtime imports)
    pub fn get_value_dependencies(&self, id: &ModuleId) -> Vec<ModuleId> {
        self.edges
            .get(id)
            .map(|edges| {
                edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::Value)
                    .map(|e| e.target.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get only type-only dependencies
    pub fn get_type_dependencies(&self, id: &ModuleId) -> Vec<ModuleId> {
        self.edges
            .get(id)
            .map(|edges| {
                edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::TypeOnly)
                    .map(|e| e.target.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if the graph contains a module
    pub fn contains(&self, id: &ModuleId) -> bool {
        self.nodes.contains(id)
    }

    /// Get all modules in the graph
    pub fn modules(&self) -> impl Iterator<Item = &ModuleId> {
        self.nodes.iter()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_id(name: &str) -> ModuleId {
        ModuleId::new(PathBuf::from(name))
    }

    #[test]
    fn test_simple_topological_sort() {
        let mut graph = DependencyGraph::new();

        // a depends on b, b depends on c
        graph.add_module(make_id("c"), vec![]);
        graph.add_module(make_id("b"), vec![(make_id("c"), EdgeKind::Value)]);
        graph.add_module(make_id("a"), vec![(make_id("b"), EdgeKind::Value)]);

        let sorted = graph.topological_sort().unwrap();

        // c should come before b, b before a
        let c_pos = sorted.iter().position(|id| id.as_str() == "c").unwrap();
        let b_pos = sorted.iter().position(|id| id.as_str() == "b").unwrap();
        let a_pos = sorted.iter().position(|id| id.as_str() == "a").unwrap();

        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = DependencyGraph::new();

        // a depends on b and c, both b and c depend on d
        graph.add_module(make_id("d"), vec![]);
        graph.add_module(make_id("b"), vec![(make_id("d"), EdgeKind::Value)]);
        graph.add_module(make_id("c"), vec![(make_id("d"), EdgeKind::Value)]);
        graph.add_module(
            make_id("a"),
            vec![
                (make_id("b"), EdgeKind::Value),
                (make_id("c"), EdgeKind::Value),
            ],
        );

        let sorted = graph.topological_sort().unwrap();

        // d should come before b and c, both should come before a
        let d_pos = sorted.iter().position(|id| id.as_str() == "d").unwrap();
        let b_pos = sorted.iter().position(|id| id.as_str() == "b").unwrap();
        let c_pos = sorted.iter().position(|id| id.as_str() == "c").unwrap();
        let a_pos = sorted.iter().position(|id| id.as_str() == "a").unwrap();

        assert!(d_pos < b_pos);
        assert!(d_pos < c_pos);
        assert!(b_pos < a_pos);
        assert!(c_pos < a_pos);
    }

    #[test]
    fn test_circular_dependency_detected() {
        let mut graph = DependencyGraph::new();

        // a depends on b, b depends on c, c depends on a (cycle)
        graph.add_module(make_id("a"), vec![(make_id("b"), EdgeKind::Value)]);
        graph.add_module(make_id("b"), vec![(make_id("c"), EdgeKind::Value)]);
        graph.add_module(make_id("c"), vec![(make_id("a"), EdgeKind::Value)]);

        let result = graph.topological_sort();

        assert!(result.is_err());
        if let Err(ModuleError::CircularDependency { cycle }) = result {
            assert!(cycle.len() >= 3);
            // Verify cycle contains a, b, c
            assert!(cycle.iter().any(|id| id.as_str() == "a"));
            assert!(cycle.iter().any(|id| id.as_str() == "b"));
            assert!(cycle.iter().any(|id| id.as_str() == "c"));
        } else {
            panic!("Expected CircularDependency error");
        }
    }

    #[test]
    fn test_self_dependency() {
        let mut graph = DependencyGraph::new();

        // a depends on itself
        graph.add_module(make_id("a"), vec![(make_id("a"), EdgeKind::Value)]);

        let result = graph.topological_sort();

        assert!(result.is_err());
        if let Err(ModuleError::CircularDependency { cycle }) = result {
            assert_eq!(cycle.len(), 2); // [a, a]
            assert_eq!(cycle[0].as_str(), "a");
        } else {
            panic!("Expected CircularDependency error");
        }
    }

    #[test]
    fn test_no_dependencies() {
        let mut graph = DependencyGraph::new();

        graph.add_module(make_id("a"), vec![]);
        graph.add_module(make_id("b"), vec![]);
        graph.add_module(make_id("c"), vec![]);

        let sorted = graph.topological_sort().unwrap();

        assert_eq!(sorted.len(), 3);
        // Order doesn't matter since there are no dependencies
    }

    #[test]
    fn test_type_only_cycle_allowed() {
        let mut graph = DependencyGraph::new();

        // a -> b (type), b -> c (type), c -> a (type) - cycle but all type-only
        graph.add_module(make_id("a"), vec![(make_id("b"), EdgeKind::TypeOnly)]);
        graph.add_module(make_id("b"), vec![(make_id("c"), EdgeKind::TypeOnly)]);
        graph.add_module(make_id("c"), vec![(make_id("a"), EdgeKind::TypeOnly)]);

        // Should succeed since cycle is type-only
        let result = graph.topological_sort();
        assert!(result.is_ok());
    }

    #[test]
    fn test_value_cycle_rejected() {
        let mut graph = DependencyGraph::new();

        // a -> b (value), b -> c (value), c -> a (value) - cycle with runtime deps
        graph.add_module(make_id("a"), vec![(make_id("b"), EdgeKind::Value)]);
        graph.add_module(make_id("b"), vec![(make_id("c"), EdgeKind::Value)]);
        graph.add_module(make_id("c"), vec![(make_id("a"), EdgeKind::Value)]);

        // Should fail - runtime circular dependency
        let result = graph.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_type_dependency_ignored_in_sort() {
        let mut graph = DependencyGraph::new();

        // a -> b (value), b -> c (type), a -> c (type)
        // Topological order should be: c, b, a (or b, c, a)
        // The type-only edge a -> c should be ignored
        graph.add_module(make_id("c"), vec![]);
        graph.add_module(make_id("b"), vec![(make_id("c"), EdgeKind::TypeOnly)]);
        graph.add_module(
            make_id("a"),
            vec![
                (make_id("b"), EdgeKind::Value),
                (make_id("c"), EdgeKind::TypeOnly),
            ],
        );

        let sorted = graph.topological_sort().unwrap();

        let a_pos = sorted.iter().position(|id| id.as_str() == "a").unwrap();
        let b_pos = sorted.iter().position(|id| id.as_str() == "b").unwrap();

        // a should come after b (respects value edge)
        assert!(b_pos < a_pos);
        // c position doesn't matter relative to a (type-only edge ignored)
    }

    #[test]
    fn test_get_value_dependencies() {
        let mut graph = DependencyGraph::new();

        graph.add_module(
            make_id("a"),
            vec![
                (make_id("b"), EdgeKind::Value),
                (make_id("c"), EdgeKind::TypeOnly),
                (make_id("d"), EdgeKind::Value),
            ],
        );

        let value_deps = graph.get_value_dependencies(&make_id("a"));

        assert_eq!(value_deps.len(), 2);
        assert!(value_deps.contains(&make_id("b")));
        assert!(value_deps.contains(&make_id("d")));
        assert!(!value_deps.contains(&make_id("c")));
    }

    #[test]
    fn test_get_type_dependencies() {
        let mut graph = DependencyGraph::new();

        graph.add_module(
            make_id("a"),
            vec![
                (make_id("b"), EdgeKind::Value),
                (make_id("c"), EdgeKind::TypeOnly),
                (make_id("d"), EdgeKind::Value),
            ],
        );

        let type_deps = graph.get_type_dependencies(&make_id("a"));

        assert_eq!(type_deps.len(), 1);
        assert!(type_deps.contains(&make_id("c")));
    }
}
