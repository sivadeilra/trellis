use super::array2::Array2;
use super::vis::vconfig_t;
use super::visibility::directVis;
use super::*;

const UNSEEN: COORD = core::i32::MAX as f64;

/* shortestPath:
 * Given a VxV weighted adjacency matrix, compute the shortest
 * path vector from root to target. The returned vector (dad) encodes the
 * shorted path from target to the root. That path is given by
 * i, dad[i], dad[dad[i]], ..., root
 * We have dad[root] = -1.
 *
 * Based on Dijkstra's algorithm (Sedgewick, 2nd. ed., p. 466).
 *
 * This implementation only uses the lower left triangle of the
 * adjacency matrix, i.e., the values a[i][j] where i >= j.
 */
pub fn shortestPath(root: i32, target: i32, V: usize, wadj: &Array2<f64>) -> Vec<i32> {
    shortestPathGeneric(root, target, V, move |i, j| wadj[(i, j)])
}
pub fn shortestPathGeneric<Wadj: Fn(usize, usize) -> f64>(
    root: i32,
    target: i32,
    V: usize,
    wadj: Wadj,
) -> Vec<i32> {
    /* initialize arrays */
    let mut dad: Vec<i32> = vec![-1; V];
    let mut vl: Vec<COORD> = vec![-UNSEEN; V + 1]; /* One extra for sentinel, which is at index 0 */
    vl[0] = -(UNSEEN + 1.0); /* Set sentinel */
    const val: usize = 1; // an offset into vl; rewrite val[x] as vl[val + x]

    let mut min = root;

    /* use (min >= 0) to fill entire tree */
    while min != target {
        let k = min;
        vl[val + k as usize] *= -1.0;
        min = -1;
        if vl[val + k as usize] == UNSEEN {
            vl[val + k as usize] = 0.0;
        }

        for t in 0..V {
            if vl[val + t] < 0.0 {
                /* Use lower triangle */
                let wkt = if k >= t as i32 {
                    wadj(k as usize, t)
                } else {
                    wadj(t, k as usize)
                };
                let newpri = -(vl[val + k as usize] + wkt);
                if wkt != 0.0 && (vl[val + t] < newpri) {
                    vl[val + t] = newpri;
                    dad[t] = k;
                }
                if vl[val + t] > vl[(val as i32 + min) as usize] {
                    min = t as i32;
                }
            }
        }
    }

    dad
}

/* makePath:
 * Given two points p and q in two polygons pp and qp of a vconfig_t conf,
 * and the visibility vectors of p and q relative to conf,
 * compute the shortest path from p to q.
 * If dad is the returned array and V is the number of polygon vertices in
 * conf, then the path is V(==q), dad[V], dad[dad[V]], ..., V+1(==p).
 * NB: This is the only path that is guaranteed to be valid.
 * We have dad[V+1] = -1.
 *
 */
pub fn makePath(
    p: Ppoint_t,
    pp: i32,
    pvis: &[COORD],
    q: Ppoint_t,
    qp: i32,
    qvis: &[COORD],
    conf: &vconfig_t,
) -> Vec<i32> {
    let V = conf.N;
    if directVis(p, pp, q, qp, conf) {
        let mut dad: Vec<i32> = vec![0; V + 2];
        dad[V] = V as i32 + 1;
        dad[V + 1] = -1;
        dad
    } else {
        let conf_vis = &conf.vis;
        shortestPathGeneric(V as i32 + 1, V as i32, V + 2, |i, j| {
            if i < conf_vis.rows {
                conf_vis[(i, j)]
            } else {
                match i - conf_vis.rows {
                    0 => qvis[j],
                    1 => pvis[j],
                    _ => panic!("out of bounds"),
                }
            }
        })
    }
}
