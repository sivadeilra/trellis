use super::*;
use log::debug;

const ISCCW: i32 = 1;
const ISCW: i32 = 2;
const ISON: i32 = 3;

const DQ_FRONT: i32 = 1;
const DQ_BACK: i32 = 2;

struct pointnlink_t {
    pp: usize, // an index into the 'ps' points array

    /// an index into pnls
    link: usize,
}

const NULL_TRIANGLE_INDEX: usize = core::usize::MAX;
const NULL_PNLS: usize = core::usize::MAX;

struct tedge_t {
    /// an index into pnls
    pnl0p: usize,
    /// an index into pnls
    pnl1p: usize,

    // ltp = left triangle pointer, and is an index into 'tris'
    ltp: usize,
    // rtp = right triangle pointer, and is an index into 'tris'
    rtp: usize,
}

struct triangle_t {
    mark: i32,
    e: [tedge_t; 3],
}

#[derive(Default)]
struct deque_t {
    /// contains indices into pnls
    pnlps: Vec<usize>,

    fpnlpi: usize,
    lpnlpi: usize,
    apex: usize,
}

pub enum ShortestPathError {
    SourcePointNotInAnyTriangle,
    DestPointNotInAnyTriangle,
}

/* Pshortestpath:
 * Find a shortest path contained in the polygon polyp going between the
 * points supplied in eps. The resulting polyline is stored in output.
 * Return 0 on success, -1 on bad input, -2 on memory allocation problem.
 */
fn Pshortestpath(polyp: &Ppoly_t, eps: &[Ppoint_t]) -> Result<Ppolyline_t, ShortestPathError> {
    let mut ps_vec: Vec<Ppoint_t> = Vec::with_capacity(polyp.ps.len() + 2);
    ps_vec.copy_from_slice(&polyp.ps);

    let mut pnls: Vec<pointnlink_t> = Vec::new();
    // pnlps contains indices into pnls
    let mut pnlps: Vec<usize> = Vec::new();
    pnls.reserve(polyp.ps.len());
    pnlps.reserve(polyp.ps.len());

    let mut dq = deque_t::default();
    dq.pnlps.reserve(polyp.ps.len() * 2);
    dq.fpnlpi = dq.pnlps.len() / 2;
    dq.lpnlpi = dq.fpnlpi - 1;

    {
        /* make sure polygon is CCW and load pnls array */
        let mut minx: f64 = polyp.ps[0].x;
        let mut minpi: usize = 0;
        for (pi, pip) in polyp.ps.iter().enumerate() {
            if minx > pip.x {
                minx = pip.x;
                minpi = pi;
            }
        }

        let p2 = polyp.ps[minpi];
        let p1 = polyp.ps[if minpi == 0 {
            polyp.ps.len() - 1
        } else {
            minpi - 1
        }];
        let p3 = polyp.ps[if minpi == polyp.ps.len() - 1 {
            0
        } else {
            minpi + 1
        }];
        if (p1.x == p2.x && p2.x == p3.x) && (p3.y > p2.y) || ccw(p1, p2, p3) != ISCCW {
            for pi in (0..polyp.ps.len()).rev() {
                if pi < polyp.ps.len() - 1 && polyp.ps[pi] == polyp.ps[pi + 1] {
                    continue;
                }
                let pnll = pnls.len();
                pnls.push(pointnlink_t {
                    pp: pi,
                    link: pnll % polyp.ps.len(),
                });
                pnlps.push(pnll);
            }
        } else {
            for pi in 0..polyp.ps.len() {
                if pi > 0 && polyp.ps[pi] == polyp.ps[pi - 1] {
                    continue;
                }
                let pnll = pnls.len();
                pnls.push(pointnlink_t {
                    pp: pi,
                    link: pnll % polyp.ps.len(),
                });
                pnlps.push(pnll);
            }
        }
    }

    /*#if defined(DEBUG) && DEBUG >= 1
        fprintf(stderr, "points\n%d\n", pnls.size());
        for (const auto& p : pnls) {
            fprintf(stderr, "%f %f\n", p.pp->x, p.pp->y);
        }
    #endif*/

    /* generate list of triangles */
    let mut tris: Vec<triangle_t> = Vec::new();
    triangulate(&mut tris, &ps_vec, &pnls, &mut pnlps);

    /*#if defined(DEBUG) && DEBUG >= 2
        fprintf(stderr, "triangles\n%d\n", tris.size());
        for (const triangle_t& tri : tris) {
            for (const auto& e : tri.e) {
                fprintf(stderr, "%f %f\n", e.pnl0p->pp->x, e.pnl0p->pp->y);
            }
        }
    #endif*/

    /* connect all pairs of triangles that share an edge */
    for trii in 0..tris.len() {
        for trij in trii + 1..tris.len() {
            connect_tris(&mut tris, &pnls, trii, trij);
        }
    }

    let find_tri = |p: Ppoint_t| {
        tris.iter()
            .position(|tri| point_in_tri(&ps_vec, &pnls, tri, p))
    };

    // find first triangle
    let ftrii = if let Some(i) = find_tri(eps[0]) {
        i
    } else {
        debug!("source point not in any triangle");
        return Err(ShortestPathError::SourcePointNotInAnyTriangle);
    };

    // find last triangle
    let ltrii = if let Some(i) = find_tri(eps[1]) {
        i
    } else {
        debug!("destination point not in any triangle");
        return Err(ShortestPathError::DestPointNotInAnyTriangle);
    };

    /* mark the strip of triangles from eps[0] to eps[1] */
    if !mark_tri_path(&mut tris, ftrii, ltrii) {
        debug!("cannot find triangle path");
        /* a straight line is better than failing */
        return Ok(Ppolyline_t {
            ps: vec![eps[0], eps[1]],
        });
    }

    /* if endpoints in same triangle, use a single line */
    if ftrii == ltrii {
        return Ok(Ppolyline_t {
            ps: vec![eps[0], eps[1]],
        });
    }

    let ep0_index = ps_vec.len();
    ps_vec.push(eps[0]);
    let ep1_index = ps_vec.len();
    ps_vec.push(eps[1]);
    // From this point on, ps does not change.
    let ps: &[Ppoint_t] = &ps_vec;

    /* build funnel and shortest path linked list (in add2dq) */
    let ep0_nls_index = pnls.len();
    pnls.push(pointnlink_t {
        pp: ep0_index,
        link: NULL_PNLS,
    });
    let ep1_nls_index = pnls.len();
    pnls.push(pointnlink_t {
        pp: ep1_index,
        link: NULL_PNLS,
    });

    add2dq(&mut dq, DQ_FRONT, &mut pnls, ep0_nls_index);
    dq.apex = dq.fpnlpi;
    let mut trii = ftrii;
    loop {
        tris[trii].mark = 2;
        let trip = &tris[trii];

        /* find the left and right points of the exiting edge */
        let lpnlp: usize; // index into pnls
        let rpnlp: usize; // index into pnls
        let ei = trip
            .e
            .iter()
            .position(|e| e.rtp != NULL_TRIANGLE_INDEX && tris[e.rtp].mark == 1);
        if ei.is_none() {
            /* in last triangle */
            if ccw(
                eps[1],
                ps[pnls[dq.pnlps[dq.fpnlpi]].pp],
                ps[pnls[dq.pnlps[dq.lpnlpi]].pp],
            ) == ISCCW
            {
                lpnlp = dq.pnlps[dq.lpnlpi];
                rpnlp = ep1_nls_index;
            } else {
                lpnlp = ep1_nls_index;
                rpnlp = dq.pnlps[dq.lpnlpi];
            }
        } else {
            let ei = ei.unwrap();
            let pnlp = trip.e[(ei + 1) % 3].pnl1p;
            if ccw(
                ps[pnls[trip.e[ei].pnl0p].pp],
                ps[pnls[pnlp].pp],
                ps[pnls[trip.e[ei].pnl1p].pp],
            ) == ISCCW
            {
                lpnlp = trip.e[ei].pnl1p;
                rpnlp = trip.e[ei].pnl0p;
            } else {
                lpnlp = trip.e[ei].pnl0p;
                rpnlp = trip.e[ei].pnl1p;
            }
        }

        /* update deque */
        if trii == ftrii {
            add2dq(&mut dq, DQ_BACK, &mut pnls, lpnlp);
            add2dq(&mut dq, DQ_FRONT, &mut pnls, rpnlp);
        } else {
            if dq.pnlps[dq.fpnlpi] != rpnlp && dq.pnlps[dq.lpnlpi] != rpnlp {
                /* add right point to deque */
                let splitindex = finddqsplit(&mut dq, ps, &pnls, rpnlp);
                splitdq(&mut dq, DQ_BACK, splitindex);
                add2dq(&mut dq, DQ_FRONT, &mut pnls, rpnlp);
                /* if the split is behind the apex, then reset apex */
                if splitindex > dq.apex {
                    dq.apex = splitindex;
                }
            } else {
                /* add left point to deque */
                let splitindex = finddqsplit(&mut dq, ps, &pnls, lpnlp);
                splitdq(&mut dq, DQ_FRONT, splitindex);
                add2dq(&mut dq, DQ_BACK, &mut pnls, lpnlp);
                /* if the split is in front of the apex, then reset apex */
                if splitindex < dq.apex {
                    dq.apex = splitindex;
                }
            }
        }
        let next_trii = None;
        for ei in 0..3 {
            let rtp = trip.e[ei].rtp;
            if rtp != NULL_TRIANGLE_INDEX && tris[rtp].mark == 1 {
                next_trii = Some(rtp);
                break;
            }
        }
        if let Some(i) = next_trii {
            trii = i;
        } else {
            break;
        }
    }

    /*#if defined(DEBUG) && DEBUG >= 1
        fprintf(stderr, "polypath");
        for (pnlp = &epnls[1]; pnlp; pnlp = pnlp->link)
            fprintf(stderr, " %f %f", pnlp->pp->x, pnlp->pp->y);
        fprintf(stderr, "\n");
    #endif*/

    let mut num_output = 0;
    {
        let mut pnlp: usize = ep1_nls_index;
        while pnlp != NULL_PNLS {
            num_output += 1;
            pnlp = pnls[pnlp].link;
        }
    }
    let mut ops: Vec<Ppoint_t> = Vec::with_capacity(num_output);
    {
        let mut pnlp = ep1_nls_index;
        while pnlp != NULL_PNLS {
            ops.push(ps[pnls[pnlp].pp]);
            pnlp = pnls[pnlp].link;
        }
    }
    ops.reverse();
    Ok(Ppolyline_t { ps: ops })
}

/* triangulate polygon */
fn triangulate(
    tris: &mut Vec<triangle_t>,
    ps: &[Ppoint_t],
    pnls: &[pointnlink_t],
    pnlps: &mut [usize],
) {
    let pnln = pnlps.len();
    if pnln > 3 {
        for pnli in 0..pnln {
            let pnlip1 = (pnli + 1) % pnln;
            let pnlip2 = (pnli + 2) % pnln;
            if isdiagonal(ps, pnli, pnlip2, pnls, pnlps) {
                loadtriangle(tris, pnlps[pnli], pnlps[pnlip1], pnlps[pnlip2]);
                for pnli in pnlip1..pnln - 1 {
                    pnlps[pnli] = pnlps[pnli + 1];
                }
                triangulate(tris, ps, pnls, &mut pnlps[..pnlps.len() - 1]);
                return;
            }
        }
        debug!("triangulation failed");
    } else {
        loadtriangle(tris, pnlps[0], pnlps[1], pnlps[2]);
    }
}

/* check if (i, i + 2) is a diagonal */
fn isdiagonal(
    ps: &[Ppoint_t],
    pnli: usize,
    pnlip2: usize,
    pnls: &[pointnlink_t],
    pnlps: &[usize],
) -> bool {
    let pnln = pnlps.len();
    /* neighborhood test */
    let pnlip1 = (pnli + 1) % pnln;
    let pnlim1 = (pnli + pnln - 1) % pnln;
    let ips = |i: usize| ps[pnls[pnlps[i]].pp];
    /* If P[pnli] is a convex vertex [ pnli+1 left of (pnli-1,pnli) ]. */
    let res = if ccw(ips(pnlim1), ips(pnli), ips(pnlip1)) == ISCCW {
        ccw(ips(pnli), ips(pnlip2), ips(pnlim1)) == ISCCW
            && ccw(ips(pnlip2), ips(pnli), ips(pnlip1)) == ISCCW
    } else {
        /* Assume (pnli - 1, pnli, pnli + 1) not collinear. */
        ccw(ips(pnli), ips(pnlip2), ips(pnlip1)) == ISCW
    };
    if !res {
        return false;
    }

    /* check against all other edges */
    for pnlj in 0..pnln {
        let pnljp1 = (pnlj + 1) % pnln;
        if !(pnlj == pnli || pnljp1 == pnli || pnlj == pnlip2 || pnljp1 == pnlip2) {
            if intersects(ips(pnli), ips(pnlip2), ips(pnlj), ips(pnljp1)) {
                return false;
            }
        }
    }
    return true;
}

/// pnlap, pnlbp, pnlcp are all indices into pnls
fn loadtriangle(tris: &mut Vec<triangle_t>, pnlap: usize, pnlbp: usize, pnlcp: usize) {
    let tri_index = tris.len();
    tris.push(triangle_t {
        mark: 0,
        e: [
            tedge_t {
                pnl0p: pnlap,
                pnl1p: pnlbp,
                ltp: tri_index,
                rtp: NULL_TRIANGLE_INDEX,
            },
            tedge_t {
                pnl0p: pnlbp,
                pnl1p: pnlcp,
                ltp: tri_index,
                rtp: NULL_TRIANGLE_INDEX,
            },
            tedge_t {
                pnl0p: pnlcp,
                pnl1p: pnlap,
                ltp: tri_index,
                rtp: NULL_TRIANGLE_INDEX,
            },
        ],
    });
}

/* connect a pair of triangles at their common edge (if any) */
fn connect_tris(tris: &mut [triangle_t], pnls: &[pointnlink_t], tri1: usize, tri2: usize) {
    for ei in 0..3 {
        for ej in 0..3 {
            let tri1p = &tris[tri1];
            let tri2p = &tris[tri2];
            let t1e = &tri1p.e[ei];
            let t2e = &tri2p.e[ej];
            if (pnls[t1e.pnl0p].pp == pnls[t2e.pnl0p].pp
                && pnls[t1e.pnl1p].pp == pnls[t2e.pnl1p].pp)
                || (pnls[t1e.pnl0p].pp == pnls[t2e.pnl1p].pp
                    && pnls[t1e.pnl1p].pp == pnls[t2e.pnl0p].pp)
            {
                tri1p.e[ei].rtp = tri2;
                tri2p.e[ej].rtp = tri1;
            }
        }
    }
}

/* find and mark path from trii, to trij */
fn mark_tri_path(tris: &mut [triangle_t], trii: usize, trij: usize) -> bool {
    if tris[trii].mark != 0 {
        return false;
    }

    tris[trii].mark = 1;
    if trii == trij {
        return true;
    }

    let trip = &tris[trii];
    for ei in 0..3 {
        let e_rtp = trip.e[ei].rtp;
        if e_rtp != NULL_TRIANGLE_INDEX && mark_tri_path(tris, e_rtp, trij) {
            return true;
        }
    }
    tris[trii].mark = 0;
    return false;
}

/* add a new point to the deque, either front or back */
/// pnlp is an index into pnls
fn add2dq(dq: &mut deque_t, side: i32, pnls: &mut [pointnlink_t], pnlp: usize) {
    if side == DQ_FRONT {
        if dq.lpnlpi - dq.fpnlpi >= 0 {
            /* shortest path links */
            pnls[pnlp].link = dq.pnlps[dq.fpnlpi];
        }
        dq.fpnlpi -= 1;
        dq.pnlps[dq.fpnlpi] = pnlp;
    } else {
        if dq.lpnlpi - dq.fpnlpi >= 0 {
            /* shortest path links */
            pnls[pnlp].link = dq.pnlps[dq.lpnlpi];
        }
        dq.lpnlpi += 1;
        dq.pnlps[dq.lpnlpi] = pnlp;
    }
}

fn splitdq(dq: &mut deque_t, side: i32, index: usize) {
    if side == DQ_FRONT {
        dq.lpnlpi = index;
    } else {
        dq.fpnlpi = index;
    }
}

// pnlp is an index into pnls
fn finddqsplit(dq: &deque_t, ps: &[Ppoint_t], pnls: &[pointnlink_t], pnlp: usize) -> usize {
    for index in dq.fpnlpi..dq.apex {
        if ccw(
            ps[pnls[dq.pnlps[index + 1]].pp],
            ps[pnls[dq.pnlps[index]].pp],
            ps[pnls[pnlp].pp],
        ) == ISCCW
        {
            return index;
        }
    }

    // for (int index = dq.lpnlpi; index > dq.apex; index--) {
    for index in (dq.apex + 1..dq.lpnlpi + 1).rev() {
        if ccw(
            ps[pnls[dq.pnlps[index - 1]].pp],
            ps[pnls[dq.pnlps[index]].pp],
            ps[pnls[pnlp].pp],
        ) == ISCW
        {
            return index;
        }
    }

    dq.apex
}

/* ccw test: CCW, CW, or co-linear */
fn ccw(p1p: Ppoint_t, p2p: Ppoint_t, p3p: Ppoint_t) -> i32 {
    let d = (p1p.y - p2p.y) * (p3p.x - p2p.x) - (p3p.y - p2p.y) * (p1p.x - p2p.x);
    if d > 0.0 {
        ISCCW
    } else {
        if d < 0.0 {
            ISCW
        } else {
            ISON
        }
    }
}

/* line to line intersection */
fn intersects(pap: Ppoint_t, pbp: Ppoint_t, pcp: Ppoint_t, pdp: Ppoint_t) -> bool {
    if ccw(pap, pbp, pcp) == ISON
        || ccw(pap, pbp, pdp) == ISON
        || ccw(pcp, pdp, pap) == ISON
        || ccw(pcp, pdp, pbp) == ISON
    {
        between(pap, pbp, pcp)
            || between(pap, pbp, pdp)
            || between(pcp, pdp, pap)
            || between(pcp, pdp, pbp)
    } else {
        let ccw1 = ccw(pap, pbp, pcp) == ISCCW;
        let ccw2 = ccw(pap, pbp, pdp) == ISCCW;
        let ccw3 = ccw(pcp, pdp, pap) == ISCCW;
        let ccw4 = ccw(pcp, pdp, pbp) == ISCCW;
        (ccw1 ^ ccw2) & (ccw3 ^ ccw4)
    }
}

/* is pbp between pap and pcp */
fn between(pap: Ppoint_t, pbp: Ppoint_t, pcp: Ppoint_t) -> bool {
    let p1 = pbp - pap;
    let p2 = pcp - pap;
    if ccw(pap, pbp, pcp) != ISON {
        return false;
    }
    (p2.x * p1.x + p2.y * p1.y >= 0.0) && (p2.x * p2.x + p2.y * p2.y <= p1.x * p1.x + p1.y * p1.y)
}

fn point_in_tri(ps: &[Ppoint_t], pnls: &[pointnlink_t], tri: &triangle_t, pp: Ppoint_t) -> bool {
    let mut sum = 0;
    for ei in 0..3 {
        if ccw(
            ps[pnls[tri.e[ei].pnl0p].pp],
            ps[pnls[tri.e[ei].pnl1p].pp],
            pp,
        ) != ISCW
        {
            sum += 1;
        }
    }
    sum == 3 || sum == 0
}
