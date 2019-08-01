use std::collections::HashMap;
use crate::ramp_table::RampTable;

pub fn init_test() {
    drop(env_logger::try_init());
}

pub type Graph = RampTable<u32>;

pub struct GraphBuilder {
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

        crate::assert_graph_is_well_formed(&graph);
        graph
    }
}

pub struct GraphBuilderFrom<'a> {
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

pub fn graph_builder() -> GraphBuilder {
    GraphBuilder {
        edges: HashMap::new(),
    }
}

pub fn graph_from_paths(paths: &[&[u32]]) -> Graph {
    let mut b = graph_builder();
    for &path in paths.iter() {
        b.path(path);
    }
    b.build()
}


