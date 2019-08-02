use crate::error::Error;
use crate::graph::Graph;
use crate::V;
use log::debug;

/// Reads an edge list and produces a topological sort of the graph.
/// 
/// On success, the output vector contains a list of vertices, in order from sinks
/// to sources. That is, for any `i` and `j` indices in the returned order `order`,
/// if `i < j` then there are no paths from vertex `order[i]` to `order[j]`.
/// The `topo_sort` function returns the opposite order (sources to sinks).
/// 
/// It is legal for the input to contain disjoint subgraphs. If the input does contain
/// more than one disjoint subgraph, then there are no guarantees about the order of
/// the returned vertex list with respect to these disjoint subgraphs. That is, if the
/// input contains two disjoint subgraphs G1 and G2, then the output vertex list may
/// contain vertexes drawn from G1, then G2, then G1 again, then G2 again, etc. They
/// are not partitioned.
/// 
/// It is legal for the input to contain degenerate vertices, meaning vertices that have
/// no edges (in-degree = 0 and out-degree = 0). The output will _not_ contain these vertices.
pub fn topo_sort_reverse(graph: &Graph) -> Result<Vec<u32>, Error> {
    debug!("topo_sort");
    let nv = graph.num_verts();

    debug!("nv = {}", nv);

    // Verts visited. these are known to be acyclic, and have been written
    // to the output vector.
    let mut visited = vec![false; nv];

    // We build the output of the function in this vector. It contains a
    // permutation of vertexes.
    let mut topo_order: Vec<V> = Vec::with_capacity(graph.num_verts());

    // Work stack contains the set of verts and the remaining forward edges for each
    // vert that we need to traverse.
    let mut work_stack: Vec<(V, core::slice::Iter<V>)> = Vec::new();

    // verts currently being examined
    let mut in_work_set: Vec<bool> = vec![false; nv];

    // Iterate through vertices. For each vertex, do depth-first forward search.
    // When a sink is found, mark the path to it as "done".
    // If we find a vertex that is in the "workset", then we found a loop.
    for (source, source_tos) in graph.iter_from_edges() {
        assert!(!in_work_set[source as usize]);
        assert!(work_stack.is_empty());

        if visited[source as usize] {
            debug!("v{} already visited", source);
            continue;
        }

        if source_tos.is_empty() {
            // Don't report isolated verts (totally disconnected verts).
            // If this vert is actually a sink (has at least one in-edge but no out-edges),
            // then we will discover it via another node.
            debug!("v{} has no edges, ignoring (for now)", source);
            continue;
        }

        /* also good:
        in_work_set[source] = true;
        work_stack.push((source as u32, source_tos.iter()));
        'work_stack_loop: while let Some((v, mut v_edges)) = work_stack.pop() {
            while let Some(&next_v) = v_edges.next() {
                if in_work_set[next_v as usize] {
                    // We have found a cycle.
                    return Err(Error::FoundCycle);
                }
                if visited[next_v as usize] {
                    // do nothing, because we have already checked subgraph at next_v
                } else {
                    // We need to descend into this forward edge.
                    work_stack.push((v, v_edges));
                    work_stack.push((next_v, graph.entry_values(next_v as usize).iter()));
                    continue 'work_stack_loop;
                }
            }
            // We have finished traversing the forward edges for v.
            // This means that v is now "done".
            in_work_set[v as usize] = false;
            visited[v as usize] = true;
            topo_order.push(v);
        }
        */

        debug!("v{} starting traversal", source);
        let mut v = source as u32;
        let mut v_edges = source_tos.iter();
        in_work_set[v as usize] = true;
        loop {
            assert!(in_work_set[v as usize]);
            for (wv, _) in work_stack.iter() {
                assert!(
                    in_work_set[*wv as usize],
                    "verts in work_stack should also be set in in_work_set"
                );
            }
            if let Some(&next_v) = v_edges.next() {
                if in_work_set[next_v as usize] {
                    debug!("... found cycle");
                    // We have found a cycle.
                    return Err(Error::FoundCycle);
                }
                if visited[next_v as usize] {
                    debug!("... v{} --> v{}, already seen v{}", v, next_v, next_v);
                // do nothing, because we have already checked subgraph at next_v
                } else {
                    // We need to descend into this forward edge.
                    debug!("... v{} --> v{}, descending", v, next_v);
                    work_stack.push((v, v_edges));
                    in_work_set[next_v as usize] = true;
                    v = next_v;
                    v_edges = graph.edges_from(next_v).iter();
                }
            } else {
                // We're done with the subgraph under v.
                debug!("... v{} is now done, popping stack", v);
                topo_order.push(v);
                visited[v as usize] = true;
                in_work_set[v as usize] = false;
                // move to previous entry on stack.
                if let Some((prev_v, prev_v_edges)) = work_stack.pop() {
                    debug!("... popped to v{}", prev_v);
                    assert!(in_work_set[prev_v as usize]);
                    assert!(!visited[prev_v as usize]);
                    v = prev_v;
                    v_edges = prev_v_edges;
                } else {
                    debug!("... done.");
                    // All done.
                    break;
                }
            }
        }
    }

    assert!(
        in_work_set.iter().all(|v| !*v),
        "in_work_set should all be false"
    );
    assert!(work_stack.is_empty());

    for &v in topo_order.iter() {
        assert!(v < graph.num_verts() as u32);
    }

    Ok(topo_order)
}

pub fn topo_sort(graph: &Graph) -> Result<Vec<u32>, Error> {
    let mut order = topo_sort_reverse(graph)?;
    order.reverse();
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn topo_sort_test() {
        init_test();

        fn case(
            description: &str,
            steps: impl Fn(&mut GraphBuilder),
            expected_result: Result<Vec<u32>, Error>,
        ) {
            let mut g = graph_builder();
            steps(&mut g);
            let graph = g.build();
            let result = topo_sort(&graph);
            assert_eq!(
                result, expected_result,
                "(actual : expected) {:?}, graph: {:#?}",
                description, graph
            );
        }

        case("empty graph", |_| {}, Ok(vec![]));

        case(
            "self-edge",
            |g| {
                g.from(0).to(0);
            },
            Err(Error::FoundCycle),
        );

        case(
            "linear path",
            |g| {
                g.path(&[1, 2, 3, 4, 5]);
            },
            Ok(vec![1, 2, 3, 4, 5]),
        );

        case(
            "linear path reversed",
            |g| {
                g.path(&[5, 4, 3, 2, 1]);
            },
            Ok(vec![5, 4, 3, 2, 1]),
        );

        case(
            "simple loop",
            |g| {
                g.from(0).to(1).to(2).to(0);
            },
            Err(Error::FoundCycle),
        );

        case(
            "single edge",
            |g| {
                g.from(0).to(1);
            },
            Ok(vec![0, 1]),
        );

        case(
            "tree",
            |g| {
                g.path(&[0, 1, 2]);
                g.path(&[0, 3, 4]);
                g.path(&[0, 5, 6]);
            },
            Ok(vec![0, 5, 6, 3, 4, 1, 2]),
        );

        case(
            "lots of small loops",
            |g| {
                g.path(&[1, 2, 3, 4, 5]);
                g.path(&[5, 4, 3, 2, 1]);
            },
            Err(Error::FoundCycle),
        );
    }

}
