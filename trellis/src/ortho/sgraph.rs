
use log::debug;

use super::fpq::PQ;

#[derive(Clone, Default)]
pub struct snode {
    pub n_val: i32,
    pub n_idx: i32,
    pub n_dad: *mut snode,
    pub n_edge: *mut sedge,
    pub n_adj: i16,
    pub save_n_adj: i16,
    pub cells: [*mut cell; 2],

    /* edges incident on this node 
     * -- stored as indices of the edges array in the graph
     */
    /// index into sgraph.adj
    pub adj_edge_list: usize,

    pub index: i32,
    pub isVert: bool,  /* true if node corresponds to vertical segment */
}

#[derive(Clone, Default)]
pub struct sedge {
    /// weight of edge
    pub weight: f64,

    /// paths using edge
    pub cnt: i32,

    /// end-points of the edge 
    /// -- stored as indices of the nodes vector in the graph
    pub v1: i32,
    pub v2: i32,
}

#[derive(Default)]
pub struct sgraph {
    pub nnodes: usize,
    pub nedges: usize,
    pub save_nnodes: usize,
    pub save_nedges: usize,
    pub nodes: Vec<snode>,
    pub edges: Vec<sedge>,

    // contains values for snode::adj_edge_list
    pub adj: Vec<i32>,
} 

// #include "fPQ.h"

/*#if 0
/* Max. number of maze segments around a node */
static int MaxNodeBoundary = 100; 

typedef struct {
    int left, right, up, down;
} irect;

/* nodes in the search graph correspond to line segments in the 
 * grid formed by n_hlines horizontal lines and n_vlines vertical lines.
 * The vertical segments are enumerated first, top to bottom, left to right.
 * Then the horizontal segments left to right, top to bottom. For example,
 * with an array of 4 vertical and 3 horizontal lines, we have
 *
 * |--14--|--15--|--16--|
 * 1      3      5      7
 * |--11--|--12--|--13--|
 * 0      2      4      6
 * |-- 8--|-- 9--|--10--|
 */
static irect
get_indices(orthograph* OG,int i, int j)
{
    irect r;
    int hl = OG->n_hlines-1;
    int vl = OG->n_vlines-1;
	r.left = i*hl + j;
	r.right = r.left + hl;
	r.down = (vl+1)*hl + j*vl + i;
	r.up = r.down + vl;
    return r;
}

static irect
find_boundary(orthograph* G, int n)
{
    rect R = G->Nodes[n];
    irect r;
    int i;

    for (i = 0; i < G->n_vlines; i++) {
        if (R.left == G->v_lines[i]) {
            r.left = i;
            break;
        }
    }
    for (; i < G->n_vlines; i++) {
        if (R.right == G->v_lines[i]) {
            r.right = i;
            break;
        }
    }
    for (i = 0; i < G->n_hlines; i++) {
        if (R.down == G->h_lines[i]) {
            r.down = i;
            break;
        }
    }
    for (; i < G->n_hlines; i++) {
        if (R.up == G->h_lines[i]) {
            r.up = i;
            break;
        }
    }
    return r;
}
#endif*/

pub fn gsave(G: &mut sgraph) {
    G.save_nnodes = G.nnodes;
    G.save_nedges = G.nedges;
    for i in 0..G.nnodes {
	    G.nodes[i].save_n_adj =  G.nodes[i].n_adj;
    }
}

pub fn reset(G: &mut sgraph) {
    G.nnodes = G.save_nnodes;
    G.nedges = G.save_nedges;
    for i in 0..G.nnodes {
        G.nodes[i].n_adj = G.nodes[i].save_n_adj;
    }
    for i in G.nnodes .. G.nnodes + 2 {
        G.nodes[i].n_adj = 0;
    }
}

pub fn initSEdges(g: &mut sgraph, maxdeg: usize) {
    g.adj = vec![0; 6 * g.nnodes + 2 * maxdeg];
    g.edges = (0.. 3 * g.nnodes + maxdeg).map(|_| sedge::default()).collect();
    let mut adj: usize = 0;
    for i in 0..g.nnodes {
	    g.nodes[i].adj_edge_list = adj;
	    adj += 6;
    }
    for i in g.nnodes..g.nnodes + 2 {
	    g.nodes[i].adj_edge_list = adj;
	    adj += maxdeg;
    }
}

pub fn createSGraph(nnodes: i32) -> sgraph {
    sgraph {
        /* create the nodes vector in the search graph */
        nodes: Vec::new(), // N_NEW(nnodes, snode);
        edges: Vec::new(),
    }
}

pub fn createSNode(g: &mut sgraph) -> *mut snode {
    let np: *mut snode = g.nodes + g.nnodes;
    np.index = g.nnodes;
    g.nnodes += 1;
    return np;
}

fn addEdgeToNode(g: &mut sgraph, np: &mut snode, _e: &mut sedge, idx: usize) {
    g.adj[np.adj_edge_list + np.n_adj as usize] = idx as i32;
    np.n_adj += 1;
}

/// returns (edge_index, &mut sedge)
pub fn createSEdge<'a>(g: &'a mut sgraph, v1: &mut snode, v2: &mut snode, wt: f64) -> (usize, &'a mut sedge) {
    let idx = g.nedges;
    g.nedges += 1;

    g.edges.push(sedge {
        v1: v1.index,
        v2: v2.index,
        weight: wt,
        cnt: 0,
    });

    let e = &mut g.edges[idx];

    addEdgeToNode(g, v1, e, idx);
    addEdgeToNode(g, v2, e, idx);

    (idx, &mut g.edges[idx])
}
 
pub fn freeSGraph(_g: sgraph) {
    /*
    free (g->nodes[0].adj_edge_list);
    free (g->nodes);
    free (g->edges);
    free (g);
    */
}

/* shortest path:
 * Constructs the path of least weight between from and to.
 * 
 * Assumes graph, node and edge type, and that nodes
 * have associated values N_VAL, N_IDX, and N_DAD, the first two
 * being ints, the last being a node*. Edges have a E_WT function 
 * to specify the edge length or weight.
 * 
 * Assumes there are functions:
 *  agnnodes: graph -> int           number of nodes in the graph
 *  agfstnode, agnxtnode : iterators over the nodes in the graph
 *  agfstedge, agnxtedge : iterators over the edges attached to a node
 *  adjacentNode : given an edge e and an endpoint n of e, returns the
 *                 other endpoint.
 * 
 * The path is given by
 *  to, N_DAD(to), N_DAD(N_DAD(to)), ..., from
 */

const UNSEEN: i32 = core::i32::MIN;

fn adjacentNode<'a>(g: &'a mut sgraph, e: &mut sedge, n: &mut snode) -> &'a mut snode {
    if e.v1 == n.index {
	    &mut g.nodes[e.v2 as usize]
    } else {
	    &mut g.nodes[e.v1 as usize]
    }
}

// from_index is an index into g.nodes
// to_index is an index into g.nodes
pub fn shortPath(g: &mut sgraph, from_index: usize, to: &mut snode) -> i32 {
    for x in 0..g.nnodes {
        let temp: &mut snode = &mut g.nodes[x];
        temp.n_val = UNSEEN;
    }
    
    let mut pq: PQ<*mut snode> = PQ::new();
    pq.insert(from);
    g.nodes[from_index].n_dad = null();
    from.n_val = 0;
    
    loop {
        let n = if let Some(n) = pq.remove() { n } else {
            break;
        };
        let n = &mut *n;
    	debug!("process {}", n.index);
	    N_VAL(n) *= -1;
        if n == to {
            break;
        }
        for y in 0..n.n_adj {
            let e: &mut sedge = &mut g.edges[g.adj[n.adj_edge_list + y] as usize];
            let adjn: *mut snode = adjacentNode(g, e, n);
            if N_VAL(adjn) < 0 {
                let d = -(N_VAL(n) + E_WT(e));
                if N_VAL(adjn) == UNSEEN {
                    debug!("new {} ({})", adjn.index, -d);
                    N_VAL(adjn) = d;
                    pq.insert(adjn);
                    N_DAD(adjn) = n;
                    N_EDGE(adjn) = e;
                } else {
                    if N_VAL(adjn) < d {
                        debug!("adjust {} ({})\n", adjn->index, -d);
                        PQupdate(adjn, d);
                        N_DAD(adjn) = n;
                        N_EDGE(adjn) = e;
                    }
                }
            }
        }
    }

    /* PQfree(); */
    return 0;
}

