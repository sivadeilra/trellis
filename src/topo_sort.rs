use crate::error::Error;
use crate::find_sources;
use crate::ramp_table::RampTable;

/// Reads an edge list and produces a topological sort of the graph.
pub fn topo_sort_reverse(graph: &RampTable<u32>) -> Result<Vec<u32>, Error> {
    let nv = graph.num_keys();

    // Verts visited. these are known to be acyclic, and have been written
    // to the output vector.
    let mut visited = vec![false; nv];

    // We build the output of the function in this vector. It contains a
    // permutation of vertexes.
    let mut topo_order: Vec<u32> = Vec::with_capacity(graph.num_keys());

    // Work stack contains the set of verts and the remaining forward edges for each
    // vert that we need to traverse.
    let mut work_stack: Vec<(u32, core::slice::Iter<u32>)> = Vec::new();

    // verts currently being examined
    let mut in_work_set: Vec<bool> = vec![false; nv];

    // Iterate through vertices. For each vertex, do depth-first forward search.
    // When a sink is found, mark the path to it as "done".
    // If we find a vertex that is in the "workset", then we found a loop.
    for (source, source_tos) in graph.iter().enumerate() {
        assert!(!in_work_set[source]);
        assert!(work_stack.is_empty());

        if visited[source] {
            continue;
        }

        if source_tos.is_empty() {
            // Don't report isolated verts (totally disconnected verts).
            // If this vert is actually a sink (has at least one in-edge but no out-edges),
            // then we will discover it via another node.
            continue;
        }

        in_work_set[source] = true;
        work_stack.push((source as u32, source_tos.iter()));

        while let Some((v, mut v_edges)) = work_stack.pop() {
            if let Some(&next_v) = v_edges.next() {
                if in_work_set[next_v as usize] {
                    // We have found a cycle.
                    return Err(Error::FoundCycle);
                }
                if visited[next_v as usize] {
                    work_stack.push((v, v_edges)); // put iterator back
                } else {
                    // We need to descend into this forward edge.
                    work_stack.push((v, v_edges));
                    work_stack.push((next_v, graph.entry_values(next_v as usize).iter()));
                }
            } else {
                // We have finished traversing the forward edges for v.
                // This means that v is now "done".
                in_work_set[v as usize] = false;
                visited[v as usize] = true;
                topo_order.push(v);
                // Continue to the next item in the work stack.
            }
        }
    }

    Ok(topo_order)
}

pub fn topo_sort(graph: &RampTable<u32>) -> Result<Vec<u32>, Error> {
    let mut order = topo_sort_reverse(graph)?;
    order.reverse();
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ramp_table::RampTable;
    use std::collections::HashMap;

    type Graph = RampTable<u32>;

    struct GraphBuilder {
        // from -> to
        edges: HashMap<u32, Vec<u32>>,
    }
    impl GraphBuilder {
        pub fn edge(&mut self, from: u32, to: u32) {
            self.edges.entry(from).or_default().push(to);
        }
        pub fn from(&mut self, from: u32) -> GraphBuilderFrom<'_> {
            GraphBuilderFrom {
                builder: self,
                from,
            }
        }
        pub fn path(&mut self, verts: &[u32]) {
            for w in verts.windows(2) {
                self.edge(w[0], w[1]);
            }
        }
        pub fn build(self) -> Graph {
            let mut sorted = self.edges.iter().collect::<Vec<(&u32, &Vec<u32>)>>();
            sorted.sort_by_key(|e| e.0);
            let mut graph = Graph::new();
            for (&from, to_list) in sorted.iter() {
                // Fill in any empty 'from' entries.
                while graph.num_keys() < from as usize {
                    graph.finish_key();
                }
                graph.push_entry_copy(to_list);
            }

            let num_verts = self.edges.iter().map(|(&from, to_list)| {
                (from + 1).max(to_list.iter().map(|&v| v + 1).max().unwrap_or(0))
            }).max().unwrap_or(0);

            while graph.num_keys() < num_verts as usize {
                graph.finish_key();
            }

            graph
        }
    }

    struct GraphBuilderFrom<'a> {
        from: u32,
        builder: &'a mut GraphBuilder,
    }

    impl<'a> GraphBuilderFrom<'a> {
        pub fn to(self, to: u32) -> Self {
            self.builder.edge(self.from, to);
            Self {
                builder: self.builder,
                from: to,
            }
        }
    }

    fn graph_builder() -> GraphBuilder {
        GraphBuilder {
            edges: HashMap::new(),
        }
    }

    fn example_graph() -> Graph {
        let mut g = graph_builder();
        g.from(0).to(1).to(5);
        g.build()
    }

    #[test]
    fn topo_sort_test() {
        fn case(description: &str,
            steps: impl Fn(&mut GraphBuilder), expected_result: Result<Vec<u32>, Error>) {
            let mut g = graph_builder();
            steps(&mut g);
            let graph = g.build();
            let result = topo_sort(&graph);
            assert_eq!(result, expected_result, "(actual : expected) {:?}, graph: {:#?}", description, graph);
        }

        case("empty graph", |_| {}, Ok(vec![]));

        case("self-edge", |g| {
            g.from(0).to(0);
        }, Err(Error::FoundCycle));

        case("linear path", |g| {
            g.path(&[1, 2, 3, 4, 5]);
        }, Ok(vec![1, 2, 3, 4, 5]));

        case("linear path reversed", |g| {
            g.path(&[5, 4, 3, 2, 1]);
        }, Ok(vec![5, 4, 3, 2, 1]));

        case("simple loop", |g| {
            g.from(0).to(1).to(2).to(0);
        }, Err(Error::FoundCycle));

        case("single edge", |g| {
            g.from(0).to(1);
        }, Ok(vec![0, 1]));

        case("tree", |g| {
            g.path(&[0, 1, 2]);
            g.path(&[0, 3, 4]);
            g.path(&[0, 5, 6]);
        }, Ok(vec![0, 5, 6, 3, 4, 1, 2]));
    }

}
