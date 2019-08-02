
use crate::V;
use crate::graph::Graph;
use crate::ramp_table::RampTable;
use crate::vec_option::VecOption;
use log::debug;

const NO_GRAPH: u32 = !0u32;

pub fn find_disjoint_subgraphs(graph: &Graph) -> DisjointSubgraphs 
{
    // First we traverse all edges of the graph and create the graph_alias and v_graph tables.
    // This creates a set of "possible subgraphs". After this step, v_graph will contain one
    // entry for each "possible subgraph" and v_graph will contain a mapping from vertex
    // to a possible-subgraph-number.
    debug!("step 1: assigning verts to possibly-disjoint-subgraphs");
    let mut v_graph: Vec<u32> = vec![NO_GRAPH; graph.num_verts()];
    let mut graph_alias: Vec<u32> = vec![];
    for (from, to) in graph.iter_edges_flattened() {
        let g_from = v_graph[from as usize];
        let g_to = v_graph[to as usize];
        match (g_from, g_to) {
            (NO_GRAPH, NO_GRAPH) => {
                let new_g = graph_alias.len() as u32;
                graph_alias.push(new_g);
                v_graph[from as usize] = new_g;
                v_graph[to as usize] = new_g;
                debug!("v{} (none) --> v{} (none), assigned to new graph {}", from, to, new_g);
            }
            (NO_GRAPH, g_to) => {
                debug!("v{from} (none) --> v{to} (in g{g_to}), assigning v{from} to g{g_to}", from=from, to=to, g_to=g_to);
                v_graph[from as usize] = g_to;
            }
            (g_from, NO_GRAPH) => {
                debug!("v{from} (in g{g_from}) --> v{to} (none), assigning v{to} to g{g_from}", from=from, to=to, g_from=g_from);
                v_graph[to as usize] = g_from;
            }
            (g_from, g_to) => {
                if g_from == g_to {
                    // already in same graph
                    debug!("v{from} (in g{g_from}) --> v{to} (in g{g_to}), both in same graph", from=from, to=to, g_from=g_from, g_to=g_to);
                } else {
                    // We've found a connection between these two possibly-disjoint graphs.
                    // It might not be the only connection, of course. Remember the connection
                    // by editing graph_alias[] so that both g_low and g_high "point" to one
                    // of the graphs. By convention, we make the higher one point to the lower.
                    let g_low = g_from.min(g_to);
                    let g_high = g_from.max(g_to);
                    graph_alias[g_high as usize] = g_low;
                    debug!("v{from} (in g{g_from}) --> v{to} (in g{g_to}), in different graphs. assigning g{g_high} --> g{g_low}", from=from, to=to, g_from=g_from, g_to=g_to, g_high=g_high, g_low=g_low);
                }
            }
        }
    }

    // Now we have a bunch of possibly-disjoint subgraphs and a mapping v_graph that maps from
    // vertex to a possibly-disjoint subgraph. For each possibly-disjoint subgraph, determine
    // the "leader".

    // make sure each entry in graph_alias points to the lowest entry
    debug!("step 2: find leaders for possibly-disjoint-subgraphs");
    let mut num_subgraphs: usize = 0;
    let mut remap: Vec<u32> = vec![!0u32; graph_alias.len()];
    for i in 0..graph_alias.len() {
        let mut leader = i as u32;
        loop {
            let next = graph_alias[leader as usize];
            assert!(next <= leader);
            if next == leader {
                break;
            }
            leader = next;
        }
        graph_alias[i] = leader;
        if i as u32 == leader {
            // If the leader for this PDS is itself, then this PDS is a leader.
            // Assign a new "final" subgraph number to it and store it in the
            // 'remap' table. 'remap' maps from PDS number to final subgraph number,
            // but only for the leaders.
            remap[i] = num_subgraphs as u32;
            debug!("g{} is leader G{}", i, num_subgraphs);
            num_subgraphs += 1;
        } else {
            // this is an "alias"
        }        
    }
    debug!("number of subgraphs: {}", num_subgraphs);

    debug!("step 3: remapping v_layer table");
    for vg in v_graph.iter_mut() {
        if *vg == NO_GRAPH {
            // This vertex is not connected.
        } else {
            let leader = graph_alias[*vg as usize];
            let g = remap[leader as usize];
            assert!(g != !0u32);
            *vg = g;
        }
    }

    debug!("step 4: counting the number of verts in each subgraph");
    let mut verts_per_subgraph: Vec<u32> = vec![0; num_subgraphs];
    for &vg in v_graph.iter() {
        if vg != NO_GRAPH {
            verts_per_subgraph[vg as usize] += 1;
        }
    }
    for (i, &count) in verts_per_subgraph.iter().enumerate() {
        debug!("G{}: {}", i, count);
    }

    debug!("step 5: building ramp table");


    // Build 'verts_per_subgraph', which counts the number of verts in each subgraph.
    let mut verts_per_subgraph: Vec<u32> = vec![0; num_subgraphs as usize];
    for &value in v_graph.iter() {
        if value != NO_GRAPH {
            verts_per_subgraph[value as usize] += 1;
        }
    }

    // Build the index table.
    let mut output_index: Vec<u32> = Vec::with_capacity(num_subgraphs as usize + 1);
    let mut sum: u32 = 0;
    for &value in verts_per_subgraph.iter() {
        output_index.push(sum);
        sum += value;
    }
    output_index.push(sum);
    // <-- sum is now the number of verts in the output. This may be less than the
    // number of verts in the input, if there are isolated verts.

    // Clone 'output_index' as 'output_pos', so that we can know where in output_values
    // to write verts.
    let mut output_pos = output_index.clone();

    // Build the values table
    let mut output_values: VecOption<u32> = VecOption::new_repeat_none(sum as usize);

    for (v, &graph) in v_graph.iter().enumerate() {
        if graph != NO_GRAPH {
            let pos_ptr = &mut output_pos[graph as usize];
            output_values.set_some(*pos_ptr as usize, v as u32);
            *pos_ptr += 1;
        }
    }
    let output_values = output_values.some_into_vec();
    assert_eq!(output_values.len(), sum as usize);

    // Check that all of our positions ended up where we expected them to.
    for i in 0..num_subgraphs {
        assert!(output_pos[i] == output_index[i + 1]);
    }

    assert!(output_values.iter().all(|v| *v != !0u32));

    let subgraphs = RampTable {
            index: output_index,
            values: output_values
        };

    println!("Disjoint subgraphs: {:#?}", subgraphs);

    DisjointSubgraphs {
        subgraphs
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use log::info;

    #[test]
    fn find_disjoint_subgraphs_test() {
        init_test();

        fn case(description: &str, graph: &Graph) {
            info!("testing: {}", description);
            find_disjoint_subgraphs(graph);
        }

        case("empty", 
            &graph_from_paths(&[]));

        case("simple acyclic",
            &graph_from_paths(&[
                &[1, 2, 3, 4, 5],
            ])
        );

        case("backward",
            &graph_from_paths(&[
                &[5, 4, 3, 2, 1],
            ])
        );

        case("two",
            &graph_from_paths(&[
                &[1, 2, 3, 4, 5],
                &[10, 11, 12, 13, 14],
            ])
        );

    }

}


pub struct DisjointSubgraphs {

    /// Contains a set of G -> [V]. Each key is a subgraph. Each set of values for each key
    /// contains the vertices that are in that subgraph.
    pub subgraphs: RampTable<V>,

}


