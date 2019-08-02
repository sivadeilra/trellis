
use crate::V;
use crate::ramp_table::RampTable;

/// Represents a set of edges; a graph.
/// 
/// The `E` type parameter contains per-edge information. By default, edges contain no information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    pub edges: RampTable<V>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: RampTable::new()
        }
    }

    pub fn edges(&self) -> &RampTable<V> {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut RampTable<V> {
        &mut self.edges
    }

    pub fn push_to(&mut self, to: V) {
        self.edges.push_value(to);
    }

    pub fn finish_from(&mut self) {
        self.edges.finish_key();
    }

    pub fn num_verts(&self) -> usize {
        self.edges.num_keys()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.num_values()
    }

    pub fn edges_from(&self, from: V) -> &[V] {
        self.edges.entry_values(from as usize)
    }

    pub fn iter_edges_flattened(&self) -> impl Iterator<Item = (V, V)> + '_ {
        self.edges.iter()
            .enumerate()
            .map(move |(from, edges)| edges.iter().map(move |&to| (from as V, to)))
            .flatten()
    }

    pub fn iter_from_edges(&self) -> impl Iterator<Item = (V, &'_ [V])> + '_ {
        self.edges.iter().enumerate().map(|(from, to_list)| (from as V, to_list))
    }
}

pub fn assert_graph_is_well_formed(graph: &Graph) {
    let nv = graph.num_verts();
    for (_from, to_list) in graph.iter_from_edges() {
        for &to in to_list.iter() {
            assert!(to < nv as u32, "all vertices should be in bounds");
        }
    }
}
