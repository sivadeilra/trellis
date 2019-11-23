use crate::graph::Graph;
use crate::V;
use std::collections::HashMap;

pub fn init_test() {
    drop(env_logger::try_init());
}

pub struct GraphBuilder {
    // from -> to
    edges: HashMap<V, Vec<V>>,
}
impl GraphBuilder {
    pub fn edge(&mut self, from: V, to: V) {
        self.edges.entry(from).or_default().push(to);
    }
    pub fn from(&mut self, from: V) -> GraphBuilderFrom<'_> {
        GraphBuilderFrom {
            builder: self,
            from,
        }
    }
    pub fn path(&mut self, verts: &[V]) {
        for w in verts.windows(2) {
            self.edge(w[0], w[1]);
        }
    }
    pub fn build(self) -> Graph {
        let mut sorted = self.edges.iter().collect::<Vec<(&V, &Vec<V>)>>();
        sorted.sort_by_key(|e| e.0);
        let mut graph = Graph::new();
        for (&from, to_list) in sorted.iter() {
            // Fill in any empty 'from' entries.
            while graph.num_verts() < from as usize {
                graph.finish_from();
            }
            for &to in to_list.iter() {
                graph.push_to(to);
            }
            graph.finish_from();
        }

        let num_verts = self
            .edges
            .iter()
            .map(|(&from, to_list)| {
                (from + 1).max(to_list.iter().map(|&v| v + 1).max().unwrap_or(0))
            })
            .max()
            .unwrap_or(0);

        while graph.num_verts() < num_verts as usize {
            graph.finish_from();
        }

        crate::graph::assert_graph_is_well_formed(&graph);
        graph
    }
}

pub struct GraphBuilderFrom<'a> {
    from: V,
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

pub fn graph_builder() -> GraphBuilder {
    GraphBuilder {
        edges: HashMap::new(),
    }
}

pub fn graph_from_paths(paths: &[&[V]]) -> Graph {
    let mut b = graph_builder();
    for &path in paths.iter() {
        b.path(path);
    }
    b.build()
}
