#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::ramp_table::RampTable;
use core::u16;

pub mod cgraph;
pub mod common;
pub mod disjoint;
// pub mod dot_parser;
pub mod error;
pub mod find_chains;
pub mod graph;
pub mod gvc;
pub mod layering;
pub mod math;
pub mod pathplan;
pub mod polyline;
pub mod ramp_table;
pub mod topo_sort;
pub mod vec2;
pub mod vec_option;

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
