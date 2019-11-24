use crate::graph::Graph;
use crate::ramp_table::RampTable;
use log::debug;

const VKIND_NONE: u8 = 0;
const VKIND_ONE: u8 = 1;
const VKIND_MANY: u8 = 0xff;

fn pack_degrees(in_degree: u8, out_degree: u8) -> u8 {
    (out_degree << 4) | in_degree
}

// (in_degree, out_degree)
fn unpack_degrees(packed: u8) -> (u8, u8) {
    ((packed & 0xf), (packed >> 4))
}
fn increment_degree(degree: u8) -> u8 {
    if degree == VKIND_NONE {
        VKIND_ONE
    } else {
        VKIND_MANY
    }
}

fn is_chain(degrees: u8) -> bool {
    degrees == pack_degrees(VKIND_ONE, VKIND_ONE)
}

pub fn find_chains(graph: &Graph) -> RampTable<u32> {
    // bits 0..3 for 'from' vertex
    // bits 4..7 for 'to' vertex
    let mut v_degrees: Vec<u8> = vec![pack_degrees(VKIND_NONE, VKIND_NONE); graph.num_verts()];
    for (from, to) in graph.iter_edges_flattened() {
        if from == to {
            v_degrees[from as usize] = pack_degrees(VKIND_MANY, VKIND_MANY);
            continue;
        }

        // handle 'from' side of this edge
        let vk = &mut v_degrees[from as usize];
        let (mut fd, td) = unpack_degrees(*vk);
        *vk = pack_degrees(increment_degree(fd), td);

        // handle 'to' side of this edge
        let vk = &mut v_degrees[to as usize];
        let (fd, mut td) = unpack_degrees(*vk);
        *vk = pack_degrees(fd, increment_degree(td));
    }

    // Chain -> [Start vert, Verts in chain, end vert]
    let mut chains: RampTable<u32> = RampTable::new();

    for (from, to_list) in graph.iter_from_edges() {
        // If 'from' is in a chain, then we cannot start a chain.
        if is_chain(v_degrees[from as usize]) {
            continue;
        }
        for &to in to_list.iter() {
            if is_chain(v_degrees[to as usize]) {
                // Found a chain. Walk its length.
                debug!("found chain starting at v{}", from);
                chains.push_value(from);
                let mut chain_vert = to;
                loop {
                    if is_chain(v_degrees[chain_vert as usize]) {
                        chains.push_value(chain_vert);

                        // Find the next vert. By definition, there should be exactly one.
                        let chain_to_list = graph.edges_from(chain_vert);
                        assert_eq!(chain_to_list.len(), 1);
                        let next_vert = chain_to_list[0];
                        debug!("... next v{}", next_vert);
                        chain_vert = next_vert;
                    } else {
                        // push the end
                        debug!("... end v{}", chain_vert);
                        chains.push_value(chain_vert);
                        break;
                    }
                }
                chains.finish_key();
            }
        }
    }

    debug!("chains: \n{:#?}", chains);

    debug!("done.");
    return chains;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use log::info;

    #[test]
    fn find_chains_test() {
        fn case(description: &str, graph: &Graph) {
            info!("{} -----", description);
            find_chains(graph);
        }
        init_test();

        case("empty", &graph_from_paths(&[]));

        case("simple", &graph_from_paths(&[&[1, 2, 3, 4, 5]]));

        case(
            "double",
            &graph_from_paths(&[&[1, 2, 3, 4, 5], &[1, 10, 11, 12, 5]]),
        );
    }
}
