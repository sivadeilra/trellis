pub type COORD = f64;

use super::*;
use crate::vec2::Ppoint_t;

// use super::array2::Array2;

/* TRANSPARENT means router sees past colinear obstacles */
#[cfg(TRANSPARENT)]
fn INTERSECT(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t, d: Ppoint_t, e: Ppoint_t) -> bool {
    intersect1(a, b, c, d, e)
}

#[cfg(not(TRANSPARENT))]
fn INTERSECT(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t, d: Ppoint_t, _e: Ppoint_t) -> bool {
    intersect(a, b, c, d)
}

/* area2:
 * Returns twice the area of triangle abc.
 */
pub fn area2(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t) -> COORD {
    (a.y - b.y) * (c.x - b.x) - (c.y - b.y) * (a.x - b.x)
}

/* wind:
 * Returns 1, 0, -1 if the points abc are counterclockwise,
 * collinear, or clockwise.
 */
pub fn wind(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t) -> i32 {
    let w = (a.y - b.y) * (c.x - b.x) - (c.y - b.y) * (a.x - b.x);
    /* need to allow for small math errors.  seen with "gcc -O2 -mcpu=i686 -ffast-math" */
    if w > 0.0001 {
        1
    } else {
        if w < -0.0001 {
            -1
        } else {
            0
        }
    }
}

/*
#if 0				/* NOT USED */
/* open_intersect:
 * Returns true iff segment ab intersects segment cd.
 * NB: segments are considered open sets
 */
static int open_intersect(Ppoint_t a, Ppoint_t b, Ppoint_t c, Ppoint_t d)
{
    return (((area2(a, b, c) > 0 && area2(a, b, d) < 0) ||
         (area2(a, b, c) < 0 && area2(a, b, d) > 0))
        &&
        ((area2(c, d, a) > 0 && area2(c, d, b) < 0) ||
         (area2(c, d, a) < 0 && area2(c, d, b) > 0)));
}
#endif
*/

/* inBetween:
 * Return true if c is in (a,b), assuming a,b,c are collinear.
 */
pub fn inBetween(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t) -> bool {
    if a.x != b.x {
        /* not vertical */
        ((a.x < c.x) && (c.x < b.x)) || ((b.x < c.x) && (c.x < a.x))
    } else {
        ((a.y < c.y) && (c.y < b.y)) || ((b.y < c.y) && (c.y < a.y))
    }
}

/* TRANSPARENT means router sees past colinear obstacles */
#[cfg(TRANSPARENT)]
/* intersect1:
 * Returns true if the polygon segment [q,n) blocks a and b from seeing
 * each other.
 * More specifically, returns true iff the two segments intersect as open
 * sets, or if q lies on (a,b) and either n and p lie on
 * different sides of (a,b), i.e., wind(a,b,n)*wind(a,b,p) < 0, or the polygon
 * makes a left turn at q, i.e., wind(p,q,n) > 0.
 *
 * We are assuming the p,q,n are three consecutive vertices of a barrier
 * polygon with the polygon interior to the right of p-q-n.
 *
 * Note that given the constraints of our problem, we could probably
 * simplify this code even more. For example, if abq are collinear, but
 * q is not in (a,b), we could return false since n will not be in (a,b)
 * nor will the (a,b) intersect (q,n).
 *
 * Also note that we are computing w_abq twice in a tour of a polygon,
 * once for each edge of which it is a vertex.
 */
fn intersect1(a: Ppoint_t, b: Ppoint_t, q: Ppoint_t, n: Ppoint_t, p: Ppoint_t) -> bool {
    let w_abq = wind(a, b, q);
    let w_abn = wind(a, b, n);

    /* If q lies on (a,b),... */
    if w_abq == 0 && inBetween(a, b, q) {
        (w_abn * wind(a, b, p) < 0) || (wind(p, q, n) > 0)
    } else {
        let w_qna = wind(q, n, a);
        let w_qnb = wind(q, n, b);
        /* True if q and n are on opposite sides of ab,
         * and a and b are on opposite sides of qn.
         */
        ((w_abq * w_abn) < 0) && ((w_qna * w_qnb) < 0)
    }
}

/* intersect:
 * Returns true if the segment [c,d] blocks a and b from seeing each other.
 * More specifically, returns true iff c or d lies on (a,b) or the two
 * segments intersect as open sets.
 */
#[cfg(not(TRANSPARENT))]
fn intersect(a: Ppoint_t, b: Ppoint_t, c: Ppoint_t, d: Ppoint_t) -> bool {
    let a_abc = wind(a, b, c);
    if (a_abc == 0) && inBetween(a, b, c) {
        return true;
    }
    let a_abd = wind(a, b, d);
    if (a_abd == 0) && inBetween(a, b, d) {
        return true;
    }
    let a_cda = wind(c, d, a);
    let a_cdb = wind(c, d, b);

    /* True if c and d are on opposite sides of ab,
     * and a and b are on opposite sides of cd.
     */
    (a_abc * a_abd < 0) && (a_cda * a_cdb < 0)
}

/* in_cone:
 * Returns true iff point b is in the cone a0,a1,a2
 * NB: the cone is considered a closed set
 */
fn in_cone(a0: Ppoint_t, a1: Ppoint_t, a2: Ppoint_t, b: Ppoint_t) -> bool {
    let m = wind(b, a0, a1);
    let p = wind(b, a1, a2);

    if wind(a0, a1, a2) > 0 {
        m >= 0 && p >= 0 /* convex at a */
    } else {
        m >= 0 || p >= 0 /* reflex at a */
    }
}

/*				/* NOT USED */
/* in_open_cone:
 * Returns true iff point b is in the cone a0,a1,a2
 * NB: the cone is considered an open set
 */
static int in_open_cone(Ppoint_t a0, Ppoint_t a1, Ppoint_t a2, Ppoint_t b)
{
    int m = wind(b, a0, a1);
    int p = wind(b, a1, a2);

    if (wind(a0, a1, a2) >= 0)
    return (m > 0 && p > 0);	/* convex at a */
    else
    return (m > 0 || p > 0);	/* reflex at a */
}
*/

/* dist2:
 * Returns the square of the distance between points a and b.
 */
fn dist2(a: Ppoint_t, b: Ppoint_t) -> COORD {
    let delx = a.x - b.x;
    let dely = a.y - b.y;
    delx * delx + dely * dely
}

/* dist:
 * Returns the distance between points a and b.
 */
fn dist(a: Ppoint_t, b: Ppoint_t) -> COORD {
    dist2(a, b).sqrt()
}

fn inCone(i: usize, j: usize, pts: &[Ppoint_t], nextPt: &[i32], prevPt: &[i32]) -> bool {
    in_cone(
        pts[prevPt[i] as usize],
        pts[i],
        pts[nextPt[i] as usize],
        pts[j],
    )
}

/* clear:
 * Return true if no polygon line segment non-trivially intersects
 * the segment [pti,ptj], ignoring segments in [start,end).
 */
fn clear(
    pti: Ppoint_t,
    ptj: Ppoint_t,
    start: usize,
    end: usize,
    V: usize,
    pts: &[Ppoint_t],
    nextPt: &[i32],
    prevPt: &[i32],
) -> bool {
    for k in 0..start {
        if INTERSECT(
            pti,
            ptj,
            pts[k],
            pts[nextPt[k] as usize],
            pts[prevPt[k] as usize],
        ) {
            return false;
        }
    }

    for k in end..V {
        if INTERSECT(
            pti,
            ptj,
            pts[k],
            pts[nextPt[k] as usize],
            pts[prevPt[k] as usize],
        ) {
            return false;
        }
    }

    true
}

use super::vis::vconfig_t;

/* compVis:
 * Compute visibility graph of vertices of polygons.
 * Only do polygons from index startp to end.
 * If two nodes cannot see each other, the matrix entry is 0.
 * If two nodes can see each other, the matrix entry is the distance
 * between them.
 */
fn compVis(conf: &mut vconfig_t, start: usize) {
    let V = conf.N;
    let pts = &conf.P;
    let nextPt = &conf.next;
    let prevPt = &conf.prev;
    let wadj = &mut conf.vis;

    for i in start..V {
        /* add edge between i and previ.
         * Note that this works for the cases of polygons of 1 and 2
         * vertices, though needless work is done.
         */
        let previ = prevPt[i];
        let d = dist(pts[i], pts[previ as usize]);
        wadj[(i, previ as usize)] = d;
        wadj[(previ as usize, i)] = d;

        /* Check remaining, earlier vertices */
        let mut j;
        if previ == i as i32 - 1 {
            j = i - 2;
        } else {
            j = i - 1;
        }
        loop {
            if inCone(i, j, pts, nextPt, prevPt)
                && inCone(j, i, pts, nextPt, prevPt)
                && clear(pts[i], pts[j], V, V, V, pts, nextPt, prevPt)
            {
                /* if i and j see each other, add edge */
                let d = dist(pts[i], pts[j]);
                wadj[(i, j)] = d;
                wadj[(j, i)] = d;
            }

            if j == 0 {
                break;
            }
            j -= 1;
        }
    }
}

/* visibility:
 * Given a vconfig_t conf, representing polygonal barriers,
 * compute the visibility graph of the vertices of conf.
 * The graph is stored in conf->vis.
 */
pub fn visibility(conf: &mut vconfig_t) {
    conf.vis = super::array2::allocArray(conf.N, 2);
    compVis(conf, 0);
}

/* polyhit:
 * Given a vconfig_t conf, as above, and a point,
 * return the index of the polygon that contains
 * the point, or else POLYID_NONE.
 */
pub fn polyhit(conf: &vconfig_t, p: Ppoint_t) -> i32 {
    for i in 0..conf.Npoly {
        let start = conf.start[i] as usize;
        let end = conf.start[i + 1] as usize;
        if super::in_poly::in_poly(&conf.P[start..end], p) {
            return i as i32;
        }
    }
    POLYID_NONE
}

/* ptVis:
 * Given a vconfig_t conf, representing polygonal barriers,
 * and a point within one of the polygons, compute the point's
 * visibility vector relative to the vertices of the remaining
 * polygons, i.e., pretend the argument polygon is invisible.
 *
 * If pp is NIL, ptVis computes the visibility vector for p
 * relative to all barrier vertices.
 */
pub fn ptVis(conf: &vconfig_t, mut pp: i32, p: Ppoint_t) -> Vec<COORD> {
    let V = conf.N;
    let pts = &conf.P;
    let nextPt = &conf.next;
    let prevPt = &conf.prev;

    let mut vadj: Vec<COORD> = vec![0.0; V + 2];

    if pp == POLYID_UNKNOWN {
        pp = polyhit(conf, p);
    }
    let start;
    let end;
    if pp >= 0 {
        start = conf.start[pp as usize] as usize;
        end = conf.start[pp as usize + 1] as usize;
    } else {
        start = V;
        end = V;
    }

    for k in 0..start {
        let pk = pts[k];
        if in_cone(pts[prevPt[k] as usize], pk, pts[nextPt[k] as usize], p)
            && clear(p, pk, start, end, V, pts, nextPt, prevPt)
        {
            /* if p and pk see each other, add edge */
            let d = dist(p, pk);
            vadj[k] = d;
        } else {
            vadj[k] = 0.0;
        }
    }

    for k in start..end {
        vadj[k] = 0.0;
    }

    for k in end..V {
        let pk = pts[k];
        if in_cone(pts[prevPt[k] as usize], pk, pts[nextPt[k] as usize], p)
            && clear(p, pk, start, end, V, pts, nextPt, prevPt)
        {
            /* if p and pk see each other, add edge */
            let d = dist(p, pk);
            vadj[k] = d;
        } else {
            vadj[k] = 0.0;
        }
    }

    vadj[V] = 0.0;
    vadj[V + 1] = 0.0;

    vadj
}

/* directVis:
 * Given two points, return true if the points can directly see each other.
 * If a point is associated with a polygon, the edges of the polygon
 * are ignored when checking visibility.
 */
pub fn directVis(p: Ppoint_t, pp: i32, q: Ppoint_t, qp: i32, conf: &vconfig_t) -> bool {
    let V = conf.N;
    let pts = &conf.P;
    let nextPt = &conf.next;
    let prevPt = &conf.prev;

    let s1: usize;
    let e1: usize;
    let s2: usize;
    let e2: usize;

    if pp < 0 {
        s1 = 0;
        e1 = 0;
        if qp < 0 {
            s2 = 0;
            e2 = 0;
        } else {
            s2 = conf.start[qp as usize] as usize;
            e2 = conf.start[qp as usize + 1] as usize;
        }
    } else if qp < 0 {
        s1 = 0;
        e1 = 0;
        s2 = conf.start[pp as usize] as usize;
        e2 = conf.start[pp as usize + 1] as usize;
    } else if pp <= qp {
        s1 = conf.start[pp as usize] as usize;
        e1 = conf.start[pp as usize + 1] as usize;
        s2 = conf.start[qp as usize] as usize;
        e2 = conf.start[qp as usize + 1] as usize;
    } else {
        s1 = conf.start[qp as usize] as usize;
        e1 = conf.start[qp as usize + 1] as usize;
        s2 = conf.start[pp as usize] as usize;
        e2 = conf.start[pp as usize + 1] as usize;
    }

    for k in 0..s1 {
        if INTERSECT(
            p,
            q,
            pts[k],
            pts[nextPt[k] as usize],
            pts[prevPt[k] as usize],
        ) {
            return false;
        }
    }
    for k in e1..s2 {
        if INTERSECT(
            p,
            q,
            pts[k],
            pts[nextPt[k] as usize],
            pts[prevPt[k] as usize],
        ) {
            return false;
        }
    }
    for k in e2..V {
        if INTERSECT(
            p,
            q,
            pts[k],
            pts[nextPt[k] as usize],
            pts[prevPt[k] as usize],
        ) {
            return false;
        }
    }
    true
}
