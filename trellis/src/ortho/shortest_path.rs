
type Weight = i32;

// Each entry in the priority queue represents a path.
struct Path {
    vert: V, // vertex
    // weight: Weight, // sum of weights
    prev_path: i32, // index of previous Path entry (in FindPathState::paths)
}

struct PQEntry {
    path: i32,
    weight: Weight,
}

fn left(parent: usize) -> usize { parent * 2 + 1 }
fn right(parent: usize) -> usize { parent * 2 + 2 }
fn parent(child: usize) -> usize { (child - 1) / 2 }

struct FindPathState {
    // contains paths in progress
    pq: Vec<PQEntry>,

    /// contains all paths that we have explored
    /// length is proportional to number of edges in graph
    paths: Vec<Path>,
}

impl FindPathState {
    // is_ordered(parent, child) indicates whether a particular parent/child relationship is
    // well-ordered or not.
    fn pq_is_ordered(parent: &PQEntry, child: &PQEntry) -> bool {
        parent.weight < child.weight
    }

    // Repair ordering constraints, by moving an element up the tree (toward the root)
    fn pq_reorder_up(&mut self, index: usize) {
        let mut i: usize = index;
        while i > 0 {
            let parent = parent(i);
            if !Self::pq_is_ordered(&self.pq[parent], &self.pq[i]) {
                self.pq.swap(parent, i);
                i = parent;
            } else {
                break;
            }
        }
    }

    // Repair ordering constraints, by moving an element down the tree.
    fn pq_reorder_down(&mut self, index: usize) {
        let mut i: usize = index;
        while i < self.pq.len() {
            let i_key = &self.pq[i];
            let left = left(i);
            if left < self.pq.len() && !Self::pq_is_ordered(i_key, &self.pq[left]) {
                self.pq.swap(i, left);
                i = left;
                continue;
            }
            let right = right(i);
            if right < self.pq.len() && !Self::pq_is_ordered(i_key, &self.pq[right]) {
                self.pq.swap(i, right);
                i = right;
                continue;
            }
            break;
        }
    }

    fn pq_remove(&mut self) -> PQEntry {
        assert!(!self.pq.is_empty());
        let result = self.pq.swap_remove(0);
        self.pq_reorder_down(0);
        result
    }

    fn pq_insert(&mut self, value: PQEntry) {
        let index = self.pq.len();
        self.pq.push(value);
        self.pq_reorder_up(index);
    }
}

pub type V = i32;

pub fn find_shortest_path<Edges, EdgeIterator>(
    edges: Edges,
    from_vert: V,
    to_vert: V) -> Option<Vec<V>>
    where 
        Edges: Fn(V) -> EdgeIterator,
        EdgeIterator: Iterator<Item = (V, Weight)>
{
    let mut state: FindPathState = FindPathState {
        pq: Vec::new(),
        paths: Vec::new(),
    };

    // start search
    state.paths.push(Path {
        vert: from_vert,
        // weight: 0,
        prev_path: -1, // no previous path
    });
    state.pq.push(PQEntry {
        path: 0, // index of the initial item pushed into paths
        weight: 0
    });

    struct BestPath {
        path: i32,
        weight: Weight,
    }

    let mut best_path: Option<BestPath> = None;

    loop {
        // iterate on search
        // the entry at pq[0] is the shortest path
        if state.pq.is_empty() {
            break;
        }

        let current_path = state.pq_remove();
        let current_vert = state.paths[current_path.path as usize].vert;
        let current_path_weight = current_path.weight;

        // Has this path reached the goal vertex?
        if current_vert == to_vert {
            if best_path.as_ref().map(move |best| best.weight > current_path_weight).unwrap_or(true) {
                best_path = Some(BestPath { path: current_path.path, weight: current_path.weight });
            }
        } else {
            // The current_path has not yet reached the goal.
            // Find all the forward edges from this path and generate
            // new path entries.
            for (next_vert_index, edge_weight) in edges(current_vert) {
                let next_path_index = state.paths.len();
                let next_path_weight = current_path.weight + edge_weight;
                state.paths.push(Path {
                    vert: next_vert_index as i32,
                    // weight: next_path_weight,
                    prev_path: current_path.path
                });
                state.pq_insert(PQEntry { path: next_path_index as i32, weight: next_path_weight });
            }
        }
    }

    if let Some(ref best) = &best_path {
        let mut path_verts: Vec<i32> = Vec::new();
        let mut pi = best.path;
        loop {
            path_verts.push(state.paths[pi as usize].vert);
            if pi == from_vert as i32 {
                break;
            }
            pi = state.paths[pi as usize].prev_path;
        }
        Some(path_verts)
    } else {
        None
    }
}

use crate::ramp_table::RampTable;

/*
pub fn find_shortest_path_ramp_table<Edges, EdgeIterator>(
    edges: &RampTable<f32>,
    from_vert: V,
    to_vert: V) -> Option<Vec<V>>
    where 
        Edges: Fn(V) -> EdgeIterator,
        EdgeIterator: Iterator<Item = (V, Weight)>
{
    find_shortest_path(|from| {
        from.edges(from).map(|t| (t, 1.0))
    }, from_vert, to_vert)
}
*/

#[test]
fn test() {

}

