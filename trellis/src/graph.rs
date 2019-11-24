use crate::ramp_table::RampTable;
use crate::V;

/// Represents a set of edges; a graph.
///
/// `Graph` describes only the structure of a graph, specifically the set of edges.
/// This type does not provide any way to store per-vertex or per-edge information.
/// If an app needs to store information associated with vertices or edges, then
/// that information should be stored in parallel vectors, using the vertex index
/// and edge index to relate information in `Graph` to that parallel information.
///
/// `Graph` uses a `RampTable` for its representation. This allows for very efficient
/// forward traversal. `Graph` does not provide any efficient way to do reverse
/// traversal, however. If efficient reverse traversal is needed, then the
/// RampTable `transpose` function can be used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    pub edges: RampTable<V>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: RampTable::new(),
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
        self.edges
            .iter()
            .enumerate()
            .map(move |(from, edges)| edges.iter().map(move |&to| (from as V, to)))
            .flatten()
    }

    pub fn iter_from_edges(&self) -> impl Iterator<Item = (V, &'_ [V])> + '_ {
        self.edges
            .iter()
            .enumerate()
            .map(|(from, to_list)| (from as V, to_list))
    }

    /// Given a graph in RampTable form, produce a new graph that is its transposition.
    pub fn transpose(&self) -> Graph {
        let graph = &self.edges;
        let nv = graph.num_keys();

        let mut t_index = vec![0; nv + 1];
        // We can build the index table directly, by counting the in-degree of every vertex.

        for &to in graph.all_values().iter() {
            t_index[to as usize + 1] += 1;
        }

        // Now integrate.
        let mut sum: u32 = 0;
        for ii in t_index.iter_mut() {
            sum += *ii;
            *ii = sum;
        }

        // Build the values table.
        // counts: For each output vertex, the number of edges written to it.
        const PLACEHOLDER: u32 = !0u32;
        let mut counts: Vec<u32> = vec![0; graph.num_keys()];
        let mut t_values: Vec<u32> = vec![PLACEHOLDER; graph.num_values()];
        for (from, to_list) in graph.iter().enumerate() {
            for &to in to_list.iter() {
                let counts_ptr = &mut counts[to as usize];
                let t_values_ptr = &mut t_values[(t_index[to as usize] + *counts_ptr) as usize];
                assert_eq!(*t_values_ptr, PLACEHOLDER);
                *t_values_ptr = from as u32;
                *counts_ptr += 1;
            }
        }

        Graph {
            edges: RampTable {
                index: t_index,
                values: t_values,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use log::info;

    #[test]
    fn transpose_test() {
        fn case(description: &str, graph: &Graph) {
            info!("{} -----", description);
            info!("input graph: {:#?}", graph);
            let t_graph = graph.transpose();
            info!("transposed graph = {:#?}", t_graph);
        }
        init_test();
        case("empty", &graph_from_paths(&[]));
        case(
            "simple",
            &graph_from_paths(&[&[1, 2, 3], &[10, 11, 12, 13, 14]]),
        );
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
