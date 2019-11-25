use super::*;

const ISCCW: i32 = 1;
const ISCW: i32 = 2;
const ISON: i32 = 3;

fn dpd_ccw(p1: Ppoint_t, p2: Ppoint_t, p3: Ppoint_t) -> i32 {
    let d = (p1.y - p2.y) * (p3.x - p2.x) - (p3.y - p2.y) * (p1.x - p2.x);
    if d > 0.0 {
        ISCW
    } else {
        if d < 0.0 {
            ISCCW
        } else {
            ISON
        }
    }
}

/* Ptriangulate:
 * Return 0 on success; non-zero on error.
 */
fn Ptriangulate<F: FnMut(Ppoint_t, Ppoint_t, Ppoint_t)>(
    polygon: &Ppoly_t,
    output: &mut F,
) -> Result<(), ()> {
    let pointn = polygon.ps.len();
    let mut pointp: Vec<usize> = (0..pointn).collect();
    let ps = &polygon.ps;
    triangulate(ps, &mut pointp, output);
    Ok(())
}

/* triangulate:
 * Triangulates the given polygon.
 * Throws an exception if no diagonal exists.
 */
fn triangulate<F: FnMut(Ppoint_t, Ppoint_t, Ppoint_t)>(
    ps: &[Ppoint_t],
    pointp: &mut [usize],
    output: &mut F,
) {
    let pointn = pointp.len();
    if pointn > 3 {
        for i in 0..pointn {
            let ip1 = (i + 1) % pointn;
            let ip2 = (i + 2) % pointn;
            if dpd_isdiagonal(i, ip2, ps, pointp) {
                output(ps[pointp[i]], ps[pointp[ip1]], ps[pointp[ip2]]);
                let mut j = 0;
                for i in 0..pointn {
                    if i != ip1 {
                        pointp[j] = pointp[i];
                        j += 1;
                    }
                }
                triangulate(ps, &mut pointp[0..pointn - 1], output);
                return;
            }
        }
        panic!("booo");
    } else {
        output(ps[pointp[0]], ps[pointp[1]], ps[pointp[2]]);
    }
}

/* check if (i, i + 2) is a diagonal */
fn dpd_isdiagonal(i: usize, ip2: usize, ps: &[Ppoint_t], pointp: &[usize]) -> bool {
    let pointn = pointp.len();

    /* neighborhood test */
    let ip1 = (i + 1) % pointn;
    let im1 = (i + pointn - 1) % pointn;
    /* If P[i] is a convex vertex [ i+1 left of (i-1,i) ]. */
    let res: bool;
    if dpd_ccw(ps[pointp[im1]], ps[pointp[i]], ps[pointp[ip1]]) == ISCCW {
        res = (dpd_ccw(ps[pointp[i]], ps[pointp[ip2]], ps[pointp[im1]]) == ISCCW)
            && (dpd_ccw(ps[pointp[ip2]], ps[pointp[i]], ps[pointp[ip1]]) == ISCCW);
    } else {
        /* Assume (i - 1, i, i + 1) not collinear. */
        res = dpd_ccw(ps[pointp[i]], ps[pointp[ip2]], ps[pointp[ip1]]) == ISCW;
        /*
                &&
                        (dpd_ccw (pointp[ip2], pointp[i], pointp[im1]) != ISCW));
        */
    }
    if !res {
        return false;
    }

    /* check against all other edges */
    for j in 0..pointn {
        let jp1 = (j + 1) % pointn;
        if !((j == i) || (jp1 == i) || (j == ip2) || (jp1 == ip2)) {
            if dpd_intersects(
                ps[pointp[i]],
                ps[pointp[ip2]],
                ps[pointp[j]],
                ps[pointp[jp1]],
            ) {
                return false;
            }
        }
    }
    return true;
}

/* line to line intersection */
fn dpd_intersects(pa: Ppoint_t, pb: Ppoint_t, pc: Ppoint_t, pd: Ppoint_t) -> bool {
    if dpd_ccw(pa, pb, pc) == ISON
        || dpd_ccw(pa, pb, pd) == ISON
        || dpd_ccw(pc, pd, pa) == ISON
        || dpd_ccw(pc, pd, pb) == ISON
    {
        dpd_between(pa, pb, pc)
            || dpd_between(pa, pb, pd)
            || dpd_between(pc, pd, pa)
            || dpd_between(pc, pd, pb)
    } else {
        let ccw1 = dpd_ccw(pa, pb, pc) == ISCCW;
        let ccw2 = dpd_ccw(pa, pb, pd) == ISCCW;
        let ccw3 = dpd_ccw(pc, pd, pa) == ISCCW;
        let ccw4 = dpd_ccw(pc, pd, pb) == ISCCW;
        (ccw1 ^ ccw2) & (ccw3 ^ ccw4)
    }
}

fn dpd_between(pa: Ppoint_t, pb: Ppoint_t, pc: Ppoint_t) -> bool {
    let pba = pb - pa;
    let pca = pc - pa;
    if dpd_ccw(pa, pb, pc) != ISON {
        false
    } else {
        (pca.x * pba.x + pca.y * pba.y >= 0.0)
            && (pca.x * pca.x + pca.y * pca.y <= pba.x * pba.x + pba.y * pba.y)
    }
}
