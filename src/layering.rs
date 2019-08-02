use crate::error::Error;
use crate::graph::Graph;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerMap {
    /// The number of layers in the layer map. This is always >= 1.
    pub num_layers: usize,

    /// A table that gives which layer each vertex has been assigned to.
    /// v_layer[vertex] = layer
    pub v_layer: Vec<u32>,
}

/// Scans the graph and assigns every vertex to a layer. The layer assignment
/// ensures that all edges point "down". That is, if the graph contains an edge `f -> t`
/// then `v_layer[f] > v_layer[t]`.
pub fn create_layer_map(graph: &Graph) -> Result<LayerMap, Error> {
    let topo_order = crate::topo_sort::topo_sort_reverse(graph)?;
    let nv = graph.num_verts();

    // Create the layer map and assign every vertex to layer 0.
    let mut v_layer: Vec<u32> = vec![0; nv];
    for &from in topo_order.iter() {
        let to_list = graph.edges_from(from);
        let from_layer: u32 =
            if let Some(layer_max) = to_list.iter().map(|&v| v_layer[v as usize]).max() {
                layer_max + 1
            } else {
                // No inputs; this is a source. Assign it to layer zero.
                0
            };
        v_layer[from as usize] = from_layer;
    }

    // Find the max layer that has been assigned.
    let num_layers = v_layer.iter().copied().max().unwrap_or(0) as usize + 1;
    Ok(LayerMap {
        v_layer,
        num_layers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn create_layer_map_test() {
        fn case(description: &str, graph: &Graph) {
            let layer_map = create_layer_map(graph);
            println!(
                "--- {}\ngraph: {:#?}\nlayer_map: {:#?}\n",
                description, graph, layer_map
            );
        }

        case("empty", &Graph::new());

        case("self-edge", &graph_from_paths(&[&[0, 0]]));

        case("linear path", &graph_from_paths(&[&[1, 2, 3, 4, 5]]));

        case(
            "two linear paths, not connected",
            &graph_from_paths(&[&[10, 11, 12, 13, 14], &[20, 21, 22, 23, 24]]),
        );

        case(
            "two linear paths, connected at source",
            &graph_from_paths(&[&[1, 10, 11, 12], &[1, 20, 21, 22]]),
        );

        case(
            "two linear paths, connected at sink",
            &graph_from_paths(&[&[10, 11, 12, 1], &[20, 21, 22, 1]]),
        );

        case(
            "two linear paths, connected at middle",
            &graph_from_paths(&[&[10, 11, 1, 12, 13], &[20, 21, 1, 22, 23]]),
        );
    }
}

/// Given a graph, constructs a new graph that contains "virtual" edges and nodes,
/// called a "proper graph". In a proper graph, each edge crosses exactly one layer.
///
/// All of the vertex numbers of the input graph are preserved in the proper graph.
/// However, edge numbers are not preserved, because the edges in the proper graph
/// are usually virtual edges. No effort is made to preserve edge numbers. Because
/// this function creates virtual vertices, those are numbered after the original
/// vertices (in the output).
///
/// The algorithm first scans the input and determines the number of edges that will
/// be created for each vertex. This is used to construct a RampTable. Placing the
/// edges is relatively easy.
///
pub fn create_proper_graph(graph: &Graph, layers: &LayerMap) -> Graph {
    let nv = graph.num_verts();
    let v_layer = &layers.v_layer;
    let mut next_virt_v: u32 = nv as u32;

    // Scan the input graph and determine how many edges the proper graph
    // will contain. This allows us to allocate the output graph buffers
    // at their final size, avoiding reallocations.
    let proper_num_virt_v: usize = graph
        .iter_from_edges()
        .map(|(from, to_list)| {
            let from_layer = v_layer[from as usize];
            to_list
                .iter()
                .map(move |&to| {
                    let to_layer = v_layer[to as usize];
                    assert!(from_layer > to_layer);
                    from_layer - to_layer - 1
                })
                .sum::<u32>()
        })
        .sum::<u32>() as usize;

    let expected_num_vert_proper = graph.num_verts() + proper_num_virt_v;
    let expected_num_edge_proper = graph.num_edges() + proper_num_virt_v;

    let mut proper_from: Vec<u32> = Vec::with_capacity(expected_num_edge_proper);
    let mut proper_to: Vec<u32> = Vec::with_capacity(expected_num_edge_proper);

    let mut add_edge = |f, t| {
        proper_from.push(f);
        proper_to.push(t);
    };

    for (from, to) in graph.iter_edges_flattened() {
        let from = from as u32;
        let from_layer = layers.v_layer[from as usize];
        let to_layer = layers.v_layer[to as usize];
        assert!(from_layer > to_layer);
        let span = from_layer - to_layer;
        let mut prev_v = from;
        for i in 0..span - 1 {
            let virt_v = next_virt_v;
            next_virt_v += 1;
            add_edge(prev_v, virt_v);
            prev_v = virt_v;
        }
        add_edge(prev_v, to);
    }

    // Because of the iteration order, we know that values in 'proper_form'
    // are in non-decreasing order. Next, we transform that into an index
    // table for a RampTable.

    let nv_proper = next_virt_v;

    let index = convert_runs_to_index(proper_from.iter().copied(), nv_proper as usize);

    unimplemented!();
}

/*
#[test]
fn convert_runs_to_index_test() {
    fn case(input: &[u32], len: usize, expected: &[u32]) {
        let actual = convert_runs_to_index(input.iter().copied(), len);
        assert_eq!(
            actual.as_slice(),
            expected,
            "len: {} input: {:?}",
            len,
            input
        );
    }

    case(&[0, 0, 1, 4, 7, 7], 10, &[0, 2]);
}
*/

fn convert_runs_to_index(iter: impl Iterator<Item = u32>, len: usize) -> Vec<u32> {
    let mut index: Vec<u32> = Vec::with_capacity(len + 1);
    index.push(0);
    let mut base: u32 = 0;
    for key in iter {
        assert!((key as usize) < len);
        while index.len() <= key as usize {
            index.push(base);
        }
        base += 1;
    }
    while index.len() < len {
        index.push(base);
    }
    index
}

// Idea for language: Si# or Si++
