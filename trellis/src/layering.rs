#![doc = r###"

Layering assigns vertices to layers

* Find a topological sort of the vertices, based on the edge list.

* Using the topological sort, assign vertices to layers. This gives us v_layer[v].

* Using v_layer[v], create a proper graph, consisting of virtual vertices and edges.

* For each pair of layers in the proper graph, permute the vertices in one layer
  while keeping the other layer stable. Traverse up and down the layers.

For each vert, we keep track of its position in its layer. This gives us a permutation
within each layer. The goal is to modify those permutations to meet our goals.

Let v_pos[v] be the horizontal position of each vert in its layer.
Then we need to be able to find the per-layer edges for each vertex.

Let each edge be a tuple (vf, vt). Then each layer has an edge list, [(vf, vt)].
This could be a RampTable, of Layer --> [(vf, ft)]. Each pass, then, sorts the
edge list in-place, so that the edge list is sorted by (v_pos[vf]) or v_pos[vt],
depending on which direction the sort is happening in.

Let's say that layer[0] is at the 'bottom' of the page, and numbers increase as
you go up the page.



"###]

use crate::V;
use crate::error::Error;
use crate::graph::Graph;
use crate::ramp_table::{RampTable, RampTableBuilder};
use log::{debug};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerMap {
    /// The number of layers in the layer map. This is always >= 1.
    pub num_layers: usize,

    /// A table that gives which layer each vertex has been assigned to.
    /// v_layer[vertex] = layer
    pub v_layer: Vec<u32>,

    /// A RampTable which maps from Layer --> [Verts]
    pub layer_verts: RampTable<V>,
}

/// Scans the graph and assigns every vertex to a layer. The layer assignment
/// ensures that all edges point "down". That is, if the graph contains an edge `f -> t`
/// then `v_layer[f] > v_layer[t]`.
///
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
pub fn create_proper_graph(graph: &Graph) -> Result<(), Error> {
    let topo_order = crate::topo_sort::topo_sort_reverse(graph)?;
    if topo_order.is_empty() {
        debug!("there are no connected edges in the graph.");
        return Ok(());
    }

    debug!("num verts in topo sort: {}", topo_order.len());

    // Create the layer map and assign every vertex to layer 0.
    // We  store this information in two forms:
    //      * v_layer[v] gives the layer that 'v' is in.
    //      * layer_verts_builder is building a mapping from Layer --> Verts
    let mut max_layer: u32 = 0;
    let mut v_layer: Vec<V> = vec![0; graph.num_verts()]; // v_layer is probably obsolete
    let mut layer_verts_builder: RampTableBuilder<u32> = RampTableBuilder::new();
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
        layer_verts_builder.push(from_layer, from);
        max_layer = max_layer.max(from_layer);
    }
    // Find the max layer that has been assigned.
    let num_layers = (max_layer + 1) as usize;
    debug!("create_proper_graph: num_layers = {}", num_layers);

    // We're going to construct a RampTable (what else?) that maps each layer
    // to the edges that connect the vertices in that layer to the following layer.
    // Let LE be this map: the Layer Edges map.  It is a RampTable<(V, V)>.
    // It maps L --> (from, to), where 'from' is a vertex in layer L and 'to'
    // is a vertex in layer `L + 1`.

    debug!("building proper edges:");
    let mut layer_edges_builder: RampTableBuilder::<(V, V)> = RampTableBuilder::new();
    let mut next_virt_v: u32 = graph.num_verts() as u32;
    for (from, to) in graph.iter_edges_flattened() {
        // from_layer should always be greater than to_layer.
        // We will build our layers from the bottom up, in increasing numeric order.
        // That means we will iterate from to_layer to from_layer.
        let from_layer = v_layer[from as usize];
        let to_layer = v_layer[to as usize];
        debug!("  v{from} in L{from_layer} --> v{to} in L{to_layer}", from=from, from_layer=from_layer, to=to, to_layer=to_layer);
        assert!(from_layer > to_layer);
        let mut prev_v = from;
        let mut layer = from_layer - 1;
        while layer > to_layer  {
            // Allocate a new "virtual" vertex, using next_virt_v.
            let virt_v = next_virt_v;
            next_virt_v += 1;
            layer_verts_builder.push(layer, virt_v);
            layer_edges_builder.push(layer, (prev_v, virt_v));
            debug!("    ... edge: v{from}* in L{layer} --> v{to} in L{next_layer}", layer=layer, from=prev_v, to=virt_v, next_layer=layer + 1);
            prev_v = virt_v;
            layer -= 1;
        }
        debug!("    ... edge: v{from} in L{layer} --> v{to} in L{next_layer}", layer=to_layer, from=prev_v, to=to, next_layer = to_layer + 1);
        layer_edges_builder.push(to_layer, (prev_v, to));
    }

    // layer_edges maps layer --> (from, to), where from=L[i], to=L[i + 1];
    let layer_edges: RampTable::<(V, V)> = layer_edges_builder.finish();

    // layer_verts maps layer --> v
    let layer_verts: RampTable::<V> = layer_verts_builder.finish();
    assert_eq!(layer_verts.num_keys(), num_layers);

    let num_proper_verts = next_virt_v as usize;

    // Build v_pos.
    debug!("assigning initial v_pos to vertices:");
    // v_pos now contains the horizontal position of every vertex.
    // For degenerate vertices (not connected to any edge), v_pos[v] == !0.
    // v_pos is the horizontal 
    let mut v_pos: Vec<u32> = vec![!0u32; num_proper_verts];
    for (layer, verts) in layer_verts.iter().enumerate() {
        for (x, &v) in verts.iter().enumerate() {
            v_pos[v as usize] = x as u32;
        }
    }

    debug!("proper edges:");
    for (layer, edges) in layer_edges.iter().enumerate().rev() {
        debug!("  L{} to L{} : {:?}", layer, layer + 1, edges);
    }

    debug!("STARTING GRAPH");
    dump_proper_graph(&v_pos, &layer_verts, &layer_edges);

    min_crossings_up(&mut v_pos, &layer_edges);
    min_crossings_down(&mut v_pos, &layer_edges);

    debug!("FINAL GRAPH");
    dump_proper_graph(&v_pos, &layer_verts, &layer_edges);

    Ok(())
}

type LayerEdges = RampTable<(u32, u32)>;

// "down" means in decreasing numeric order of the layers
// the upper layer L[i + 1] is stable, while the lower layer L[i] changes
fn min_crossings_down(v_pos: &mut [u32], layer_edges: &LayerEdges) {
    debug!("min_crossings_down -----");
    let num_crossings_before = count_all_crossings(v_pos, layer_edges);
    for (layer, edges) in layer_edges.iter().enumerate().rev() {
        debug!("layers: L{} x L{}", layer, layer + 1);
        let mut local_edges: Vec<(u32, u32)> = Vec::with_capacity(edges.len());
        local_edges.extend(edges.iter().map(|&(lower, upper)| (upper, lower)));
        min_crossings_core(v_pos, &mut local_edges);
    }
    let num_crossings_after = count_all_crossings(v_pos, layer_edges);
    debug!("  change in num_crossings: {} --> {}", num_crossings_before, num_crossings_after);
}

// "up" means in increasing numeric order of the layers
// the lower layer L[i] is stable, while the upper layer L[i + 1] changes
fn min_crossings_up(v_pos: &mut [u32], layer_edges: &LayerEdges) {
    debug!("min_crossings_up -----");
    let num_crossings_before = count_all_crossings(v_pos, layer_edges);
    for (layer, edges) in layer_edges.iter().enumerate() {
        debug!("layers: L{} x L{}", layer, layer + 1);
        let mut local_edges: Vec<(u32, u32)> = Vec::with_capacity(edges.len());
        local_edges.extend(edges.iter().map(|&(lower, upper)| (lower, upper)));
        min_crossings_core(v_pos, &mut local_edges);
    }
    let num_crossings_after = count_all_crossings(v_pos, layer_edges);
    debug!("  change in num_crossings: {} --> {}", num_crossings_before, num_crossings_after);
}

// Minimizes crossings between two layers. The edges are given by (v_stable, v_moving) pairs.
// The v_pos[v_stable] values are not changed.
// The v_pos[v_moving] values are changed in order to minimize crossings.
fn min_crossings_core(v_pos: &mut [u32], edges: &mut [(u32, u32)]) {
    debug!("min_crossings_core: num_edges = {}", edges.len());
    dump_crossings(v_pos, edges, "before");

    // Sort the edges between these two layers (L[i] and L[i + 1]) by the position of the 'from' vertex.
    // Then scan through the array, finding sequential runs that have the same 'from' value.
    // Find the average of the 'from_v_pos' value for each 'from' vertex, and replace all of the values
    // (for that value of 'from') with that average.
    edges.sort_by_key(|&(_stable, v_moving)| v_moving);
    debug!("  sorted edges: {:?}", edges);
    let mut barys: Vec<(u32, u32)> = Vec::with_capacity(edges.len()); // not correct, but an upper bound
    for ee in crate::iters::iter_runs_by_key(edges, |&(_, v_moving)| v_moving) {
        let (_, v_moving) = ee[0];
        let pos_sum: f32 = ee.iter().map(|&(v_stable, _)| v_pos[v_stable as usize] as f32).sum();
        let pos_avg: f32 = pos_sum / (ee.len() as f32);
        debug!("    v_moving v{} at avg pos {}", v_moving, pos_avg);
        let scaled_avg: u32 = (pos_avg * 1000.0f32) as u32;
        barys.push((v_moving, scaled_avg));
    }
    barys.sort_by_key(|&(_v_moving, pos_avg)| pos_avg);
    debug!("  barys: {:?}", barys);
    for (i, &(v_moving, _)) in barys.iter().enumerate() {
        debug!("    v_stable v{} --> pos {}", v_moving, i);
        v_pos[v_moving as usize] = i as u32;
    }

    dump_crossings(v_pos, edges, "after");
}

#[derive(Copy, Clone)]
struct Crossing {
    e1_from_v: u32,
    e1_from_pos: u32,
    e1_to_v: u32,
    e1_to_pos: u32,

    e2_from_v: u32,
    e2_from_pos: u32,
    e2_to_v: u32,
    e2_to_pos: u32,
}
impl core::fmt::Debug for Crossing {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let from_order = self.e1_from_pos > self.e2_from_pos;
        let to_order = self.e1_to_pos > self.e2_to_pos;
        let crossed = from_order ^ to_order;
        write!(fmt, "v{}@{}_v{}@{} x v{}@{}_v{}@{} --> {}",
        self.e1_from_v, self.e1_from_pos, self.e1_to_v, self.e1_to_pos,
        self.e2_from_v, self.e2_from_pos, self.e2_to_v, self.e2_to_pos,
        if crossed { "crossed" } else { "---" }
        )
    }
}

/*
fn iter_crossings<'a, 'b>(v_pos: &'a [u32], edges: &'b [(u32, u32)]) -> impl Iterator<Item = Crossing> + 'a + 'b {
    edges.iter().enumerate().flat_map(move |(i, &(e1_from_v, e1_to_v))| {
        let e1_from_pos = v_pos[e1_from_v as usize];
        let e1_to_pos = v_pos[e1_to_v as usize];
        edges[..i].iter().flat_map(move |&(e2_from_v, e2_to_v)| {
            let e2_from_pos = v_pos[e2_from_v as usize];
            let e2_to_pos =  v_pos[e2_to_v as usize];
            let from_order = e1_from_pos > e2_from_pos;
            let to_order = e1_to_pos > e2_to_pos;
            let crossed = from_order ^ to_order;
            if crossed {
                Some(Crossing {
                    e1_from_v,
                    e1_from_pos,
                    e1_to_v,
                    e1_to_pos,
                    e2_from_v,
                    e2_from_pos,
                    e2_to_v,
                    e2_to_pos,
                })
            } else {
                None
            }
        })
    })
}
*/

fn dump_crossings(v_pos: &[u32], edges: &[(u32, u32)], description: &str) {
    debug!("crossings: num_edges = {}, for {}", edges.len(), description);
    let mut num_crossings: u32 = 0;
    for (i, &(e1_from, e1_to)) in edges.iter().enumerate() {
        let e1_from_pos = v_pos[e1_from as usize];
        let e1_to_pos = v_pos[e1_to as usize];
        for &(e2_from, e2_to) in edges[..i].iter() {
            let e2_from_pos = v_pos[e2_from as usize];
            let e2_to_pos =  v_pos[e2_to as usize];
            let from_order = e1_from_pos > e2_from_pos;
            let to_order = e1_to_pos > e2_to_pos;
            let crossed = from_order ^ to_order && e1_from != e2_from && e1_to != e2_to;
            debug!("    v{}@{}_v{}@{} x v{}@{}_v{}@{} --> {}",
                e1_from, e1_from_pos, e1_to, e1_to_pos,
                e2_from, e2_from_pos, e2_to, e2_to_pos,
                if crossed { "XXX" } else { "-" },
            );
            num_crossings += crossed as u32;
        }
    }
    debug!("  num_crossings = {}", num_crossings);
}

fn dump_proper_graph(v_pos: &[u32], proper_verts: &RampTable<V>, proper_edges: &LayerEdges) {
    debug!("dump_proper_graph:");
    let num_layers = proper_verts.num_keys();
    debug!("  num_layers = {}", num_layers);
    for (layer, layer_verts) in proper_verts.iter().enumerate().rev() {
        // show verts for this layer, at their positions
        let mut sorted_verts = Vec::new();
        sorted_verts.extend(layer_verts.iter().map(|&v| (v, v_pos[v as usize])));
        sorted_verts.sort_by_key(|&(v, v_pos)| v_pos);
        debug!("  L{}: {:?}", layer,
            sorted_verts.iter().map(|&(v, _pos)| v).collect::<Vec<u32>>());
        if layer < proper_edges.num_keys() {
            let layer_edges: &[(V, V)] = proper_edges.entry_values(layer);
            let num_crossings = count_crossings(v_pos, layer_edges);
            debug!("    edges: {:?} (num_crossings: {})", layer_edges, num_crossings);
        }
    }
}

/// Count all crossings among all pairs of consecutive layers.
fn count_all_crossings(v_pos: &[u32], layer_edges: &LayerEdges) -> u32 {
    let mut num_crossings: u32 = 0;
    for edges in layer_edges.iter() {
        num_crossings += count_crossings(v_pos, edges);
    }
    num_crossings
}

/// Count crossings between two layers.
fn count_crossings(v_pos: &[u32], edges: &[(V, V)]) -> u32 {
    let mut num_crossings: u32 = 0;
    for (i, &(e1_from, e1_to)) in edges.iter().enumerate() {
        let e1_from_pos = v_pos[e1_from as usize];
        let e1_to_pos = v_pos[e1_to as usize];
        for &(e2_from, e2_to) in edges[..i].iter() {
            if e1_from != e2_from && e1_to != e2_to {
                let e2_from_pos = v_pos[e2_from as usize];
                let e2_to_pos =  v_pos[e2_to as usize];
                let from_order = e1_from_pos > e2_from_pos;
                let to_order = e1_to_pos > e2_to_pos;
                let crossed = from_order ^ to_order;
                num_crossings += crossed as u32;
            }
        }
    }
    num_crossings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn create_proper_graph_test() {
        fn case(description: &str, graph: &Graph) {
            let layer_map = create_proper_graph(graph);
            println!(
                "--- {}\ngraph: {:#?}\nlayer_map: {:#?}\n",
                description, graph, layer_map
            );
        }

        init_test();

        case("simple tree", &graph_from_paths(&[
            &[1, 10, 11, 12, 13, 14],
            &[1, 20, 21, 22, 23, 24],
            &[1, 14],
            &[1, 24],
            &[10, 23],
        ]));

/*
        case("diamonds", &graph_from_paths(&[
            &[1, 2, 3],
            &[1, 4, 3],

            &[3, 5, 6],
            &[3, 7, 6],

            &[1, 6],
        ]));
        */

        /*
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
        */
    }
}
