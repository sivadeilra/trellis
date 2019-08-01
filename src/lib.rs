use crate::ramp_table::RampTable;
use std::u16;

pub mod error;
pub mod layering;
pub mod ramp_table;
pub mod topo_sort;

#[cfg(test)]
mod testing;

/* very general
use core::ops::AddAssign;
use core::convert::TryFrom;

pub fn add_degree_mut<E, D>(edges: &[E], degree: &mut [D])
where E: Copy, usize: TryFrom<E>,
D: AddAssign<D> + From<u8>
{
    for &e in edges.iter() {
        degree[usize::try_from(e).ok().unwrap()] += D::from(1u8);
    }
}
*/

/// Returns true if any degree saturated.
pub fn add_degree(e: &[V], degree: &mut [Degree]) -> bool {
    let mut saturated = false;
    for &v in e.iter() {
        let d = &mut degree[v as usize];
        let (new_d, overflow) = d.overflowing_add(1);
        if overflow {
            saturated = true;
        } else {
            *d = new_d;
        }
    }
    saturated
}

pub fn find_degree(e: &[V], nv: V) -> Vec<Degree> {
    let mut degree: Vec<Degree> = vec![0; nv as usize];
    add_degree(e, &mut degree);
    degree
}

#[test]
fn add_degree_mut_test() {
    let edges: Vec<u32> = vec![3, 1, 0, 0, 2, 0, 1];
    let mut degree = vec![0; 10];
    add_degree(&edges, &mut degree);
    assert_eq!(degree, [3, 2, 1, 1, 0, 0, 0, 0, 0, 0]);
}

// edge type; an index into an edge list
pub type E = u32;

// vertex type; an index into a vertex list
pub type V = u32;

// degree of a vertex (in-degree or out-degree)
pub type Degree = u16;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn find_referenced_edges(e: &[V], nv: usize) -> Vec<bool> {
    let mut result = vec![false; nv];
    for &v in e.iter() {
        result[v as usize] = true;
    }
    result
}

/// Returns a vector with 'true' for each vertex is a source (has no inbound edges).
pub fn find_sources(edges: &RampTable<u32>) -> Vec<bool> {
    let nv = edges.num_keys();
    let mut sources = vec![true; nv];
    for &edge in edges.all_values().iter() {
        sources[edge as usize] = false;
    }
    sources
}

/// Returns a vector indicating which verts are sinks (have no outgoing edges).
pub fn find_sinks(edges: &RampTable<u32>) -> Vec<bool> {
    edges.iter().map(|to_list| to_list.is_empty()).collect()
}

/// Given a graph in RampTable form, produce a new graph that is its transposition.
pub fn transpose_graph(graph: &RampTable<u32>) -> RampTable<u32> {
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

    RampTable {
        index: t_index,
        values: t_values,
    }
}

// the traversal does not include the starting node
// each node is iterated before its children
pub fn traverse_depth_first_preorder(
    graph: &RampTable<u32>,
    start: u32,
) -> impl Iterator<Item = u32> + '_ {
    struct DepthFirst<'a> {
        graph: &'a RampTable<u32>,
        stack: Vec<core::slice::Iter<'a, u32>>,
    }
    impl<'a> Iterator for DepthFirst<'a> {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(last) = self.stack.last_mut() {
                if let Some(&next) = last.next() {
                    self.stack
                        .push(self.graph.entry_values(next as usize).iter());
                    return Some(next);
                } else {
                    // done with this level
                    // pop to the next
                    self.stack.pop();
                }
            }
            return None;
        }
    }

    DepthFirst {
        graph,
        stack: vec![graph.entry_values(start as usize).iter()],
    }
}

pub fn assert_graph_is_well_formed(graph: &RampTable<u32>) {
    let nv = graph.num_keys();
    for (i, edges) in graph.iter().enumerate() {
        for &to in edges.iter() {
            assert!(to < nv as u32, "all vertices should be in bounds");
        }
    }
}

