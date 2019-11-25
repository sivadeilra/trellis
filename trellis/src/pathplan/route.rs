use super::solvers::solve3;
use super::*;
use log::debug;

const EPSILON1: f64 = 1.0E-3;
const EPSILON2: f64 = 1.0E-6;

// #define ABS(a) ((a) >= 0 ? (a) : -(a))

fn DISTSQ(a: Ppoint_t, b: Ppoint_t) -> f64 {
    (((a).x - (b).x) * ((a).x - (b).x)) + (((a).y - (b).y) * ((a).y - (b).y))
}

const POINTSIZE: usize = core::mem::size_of::<Ppoint_t>();

fn LT(pa: Ppoint_t, pbp: &Ppoint_t) -> bool {
    ((pa.y > pbp.y) || ((pa.y == pbp.y) && (pa.x < pbp.x)))
}
fn GT(pa: Ppoint_t, pbp: &Ppoint_t) -> bool {
    ((pa.y < pbp.y) || ((pa.y == pbp.y) && (pa.x > pbp.x)))
}

struct p2e_t {
    pp: *mut Ppoint_t,
    ep: *mut Pedge_t,
}

struct elist_t {
    ep: *mut Pedge_t,
    next: *mut elist_t,
    prev: *mut elist_t,
}

#[derive(Clone, Default)]
struct Context {
    ops: Vec<Ppoint_t>,
    // int opn; // opn = ops.capacity()
    // int opl; // opl = ops.size()
}

#[derive(Clone, Debug)]
enum RouteError {}

/* Proutespline:
 * Given a set of edgen line segments edges as obstacles, a template
 * path input, and endpoint vectors evs, construct a spline fitting the
 * input and endpoing vectors, and return in output.
 * Return 0 on success and -1 on failure, including no memory.
 */
fn Proutespline(
    edges: &[Pedge_t],
    input: &Ppolyline_t,
    evs: &mut [Ppoint_t],
    output: &mut Ppolyline_t,
) -> Result<(), RouteError> {
    /*
        Ppoint_t p0, p1, p2, p3;
        Ppoint_t* pp;
        Pvector_t v1, v2, v12, v23;
        int ipi, opi;
        int ei, p2ei;
        Pedge_t* e0p, * e1p;
    */

    /* unpack into previous format rather than modify legacy code */
    let inps: &[Ppoint_t] = &input.ps;
    let inpn = inps.len();

    /*
        if (!(p2es = (p2e_t*)malloc(sizeof(p2e_t) * (p2en = edgen * 2)))) {
            prerror("cannot malloc p2es");
            return -1;
        }
        for (ei = 0, p2ei = 0; ei < edgen; ei++) {
            if (edges[ei].a.x == edges[ei].b.x
                && edges[ei].a.y == edges[ei].b.y)
                continue;
            p2es[p2ei].pp = &edges[ei].a;
            p2es[p2ei++].ep = &edges[ei];
            p2es[p2ei].pp = &edges[ei].b;
            p2es[p2ei++].ep = &edges[ei];
        }
        p2en = p2ei;
        qsort(p2es, p2en, sizeof(p2e_t), cmpp2efunc);
        elist = NULL;
        for (p2ei = 0; p2ei < p2en; p2ei += 2) {
            pp = p2es[p2ei].pp;
    #if DEBUG >= 1
            fprintf(stderr, "point: %d %lf %lf\n", p2ei, pp->x, pp->y);
    #endif
            e0p = p2es[p2ei].ep;
            e1p = p2es[p2ei + 1].ep;
            p0 = (&e0p->a == p2es[p2ei].pp) ? e0p->b : e0p->a;
            p1 = (&e0p->a == p2es[p2ei + 1].pp) ? e1p->b : e1p->a;
            if (LT(p0, pp) && LT(p1, pp)) {
                listdelete(e0p), listdelete(e1p);
            } else if (GT(p0, pp) && GT(p1, pp)) {
                listinsert(e0p, *pp), listinsert(e1p, *pp);
            } else {
                if (LT(p0, pp))
                    listreplace(e0p, e1p);
                else
                    listreplace(e1p, e0p);
            }
        }
    */

    /* generate the splines */
    evs[0] = normv(evs[0]);
    evs[1] = normv(evs[1]);

    let mut context = Context::default();
    context.ops.push(inps[0]);
    reallyroutespline(&mut context, edges, inps, evs[0], evs[1])?;

    output.ps = context.ops;

    /*
        fprintf(stderr, "edge\na\nb\n");
        fprintf(stderr, "points\n%d\n", inpn);
        for (ipi = 0; ipi < inpn; ipi++)
            fprintf(stderr, "%f %f\n", inps[ipi].x, inps[ipi].y);
        fprintf(stderr, "splpoints\n%d\n", opl);
        for (opi = 0; opi < opl; opi++)
            fprintf(stderr, "%f %f\n", ops[opi].x, ops[opi].y);
    */

    Ok(())
}

#[derive(Default, Clone)]
struct tna_t {
    pub t: f64,
    pub a: [Ppoint_t; 2],
}

fn reallyroutespline(
    context: &mut Context,
    edges: &[Pedge_t],
    inps: &[Ppoint_t],
    ev0: Ppoint_t,
    ev1: Ppoint_t,
) -> Result<(), RouteError> {
    // tns.size() == inpn, one entry per point
    // TODO: be smarter about reusing this; move to context?
    let mut tnas: Vec<tna_t> = Vec::with_capacity(inps.len());
    tnas.extend((0..inps.len()).map(|_| tna_t::default()));

    tnas[0].t = 0.0;
    for i in 1..inps.len() {
        tnas[i].t = tnas[i - 1].t + dist(inps[i], inps[i - 1]);
    }
    let last_t = tnas[tnas.len() - 1].t;
    for tna in tnas[1..].iter_mut() {
        tna.t /= last_t;
    }
    for tna in tnas.iter_mut() {
        tna.a[0] = ev0 * B1(tna.t);
        tna.a[1] = ev1 * B2(tna.t);
    }

    let Spline {
        sp0: p1,
        sv0: v1,
        sp1: p2,
        sv1: v2,
    } = mkspline(inps, &tnas, ev0, ev1)?;

    if splinefits(&mut context.ops, edges, p1, v1, p2, v2, inps) {
        return Ok(());
    }

    let cp1: Ppoint_t = add(p1, scale(v1, 1.0 / 3.0));
    let cp2: Ppoint_t = sub(p2, scale(v2, 1.0 / 3.0));

    let mut maxd: f64 = -1.0;
    let mut maxi: i32 = -1;
    for i in 1..inps.len() - 1 {
        let t = tnas[i].t;
        let p = Ppoint_t {
            x: B0(t) * p1.x + B1(t) * cp1.x + B2(t) * cp2.x + B3(t) * p2.x,
            y: B0(t) * p1.y + B1(t) * cp1.y + B2(t) * cp2.y + B3(t) * p2.y,
        };
        let d = dist(p, inps[i]);
        if d > maxd {
            maxd = d;
            maxi = i as i32;
        }
    }
    let spliti = maxi as usize;
    let splitv1 = normv(sub(inps[spliti], inps[spliti - 1]));
    let splitv2 = normv(sub(inps[spliti + 1], inps[spliti]));
    let splitv = normv(add(splitv1, splitv2));
    reallyroutespline(context, edges, &inps[..spliti + 1], ev0, splitv)?;
    reallyroutespline(context, edges, &inps[spliti..], splitv, ev1)?;
    Ok(())
}

struct Spline {
    sp0: Ppoint_t,
    sv0: Ppoint_t,
    sp1: Ppoint_t,
    sv1: Ppoint_t,
}
fn mkspline(
    inps: &[Ppoint_t],
    tnas: &[tna_t],
    ev0: Ppoint_t,
    ev1: Ppoint_t,
) -> Result<Spline, RouteError> {
    assert!(inps.len() > 0);
    let mut scale0 = 0.0;
    let mut scale3 = 0.0;
    let mut c: [[f64; 2]; 2] = [[0.0, 0.0], [0.0, 0.0]];
    let mut x: [f64; 2] = [0.0, 0.0];
    for i in 0..inps.len() {
        c[0][0] += dot(tnas[i].a[0], tnas[i].a[0]);
        c[0][1] += dot(tnas[i].a[0], tnas[i].a[1]);
        c[1][0] = c[0][1];
        c[1][1] += dot(tnas[i].a[1], tnas[i].a[1]);
        let tmp = sub(
            inps[i],
            add(
                scale(inps[0], B01(tnas[i].t)),
                scale(inps[inps.len() - 1], B23(tnas[i].t)),
            ),
        );
        x[0] += dot(tnas[i].a[0], tmp);
        x[1] += dot(tnas[i].a[1], tmp);
    }
    let det01 = c[0][0] * c[1][1] - c[1][0] * c[0][1];
    let det0X = c[0][0] * x[1] - c[0][1] * x[0];
    let detX1 = x[0] * c[1][1] - x[1] * c[0][1];
    if det01.abs() >= 1.0e-6 {
        scale0 = detX1 / det01;
        scale3 = det0X / det01;
    }
    if det01.abs() < 1.0e-6 || scale0 <= 0.0 || scale3 <= 0.0 {
        let d01 = dist(inps[0], inps[inps.len() - 1]) / 3.0;
        scale0 = d01;
        scale3 = d01;
    }
    Ok(Spline {
        sp0: inps[0],
        sv0: scale(ev0, scale0),
        sp1: inps[inps.len() - 1],
        sv1: scale(ev1, scale3),
    })
}

fn dist_n(ps: &[Ppoint_t]) -> f64 {
    let mut rv = 0.0;
    for p in ps.windows(2) {
        rv +=
            ((p[1].x - p[0].x) * (p[1].x - p[0].x) + (p[1].y - p[0].y) * (p[1].y - p[0].y)).sqrt();
    }
    rv
}

fn splinefits(
    ops: &mut Vec<Ppoint_t>,
    edges: &[Pedge_t],
    pa: Ppoint_t,
    va: Pvector_t,
    pb: Ppoint_t,
    vb: Pvector_t,
    inps: &[Ppoint_t],
) -> bool {
    let mut first = true;
    let forceflag = inps.len() == 2;

    /*#if 0
        let d = sqrt((pb.x - pa.x) * (pb.x - pa.x) +
            (pb.y - pa.y) * (pb.y - pa.y));
        a = d, b = d;
    #else*/
    let mut a = 4.0;
    let mut b = 4.0;
    //#endif
    loop {
        let sps: [Ppoint_t; 4] = [pa, pa + a * va / 3.0, pb - b * vb / 3.0, pb];

        /* shortcuts (paths shorter than the shortest path) not allowed -
         * they must be outside the constraint polygon.  this can happen
         * if the candidate spline intersects the constraint polygon exactly
         * on sides or vertices.  maybe this could be more elegant, but
         * it solves the immediate problem. we could also try jittering the
         * constraint polygon, or computing the candidate spline more carefully,
         * for example using the path. SCN */

        if first && (dist_n(&sps[..]) < (dist_n(inps) - EPSILON1)) {
            return false;
        }
        first = false;

        if splineisinside(edges, &sps[..]) {
            for pi in 1..4 {
                ops.push(sps[pi]);
            }
            debug!("success: {} {}", a, b);
            return true;
        }
        if a == 0.0 && b == 0.0 {
            if forceflag {
                for pi in 1..4 {
                    ops.push(sps[pi]);
                }
                debug!("forced straight line: {} {}", a, b);
                return true;
            }
            break;
        }
        if a > 0.01 {
            a /= 2.0;
            b /= 2.0;
        } else {
            a = 0.0;
            b = 0.0;
        }
    }
    debug!("failure");
    false
}

fn splineisinside(edges: &[Pedge_t], sps: &[Ppoint_t]) -> bool {
    for ei in edges.iter() {
        let lps = [ei.a, ei.b];
        let mut roots: [f64; 4] = [0.0; 4];
        let rootn = splineintersectsline(sps, &lps[..], &mut roots[..]);
        if rootn == 4 {
            continue;
        }
        for rooti in 0..rootn {
            if roots[rooti] < EPSILON2 || roots[rooti] > 1.0 - EPSILON2 {
                continue;
            }
            let t = roots[rooti];
            let td = t * t * t;
            let tc = 3.0 * t * t * (1.0 - t);
            let tb = 3.0 * t * (1.0 - t) * (1.0 - t);
            let ta = (1.0 - t) * (1.0 - t) * (1.0 - t);
            let ip = ta * sps[0] + tb * sps[1] + tc * sps[2] + td * sps[3];
            if DISTSQ(ip, lps[0]) < EPSILON1 || DISTSQ(ip, lps[1]) < EPSILON1 {
                continue;
            }
            return false;
        }
    }
    return true;
}

fn splineintersectsline(sps: &[Ppoint_t], lps: &[Ppoint_t], roots: &mut [f64]) -> usize {
    let xcoeff = [lps[0].x, lps[1].x - lps[0].x];
    let ycoeff = [lps[0].y, lps[1].y - lps[0].y];
    let mut rootn: usize = 0;
    if xcoeff[1] == 0.0 {
        if ycoeff[1] == 0.0 {
            let mut scoeff = points2coeff(sps[0].x, sps[1].x, sps[2].x, sps[3].x);
            scoeff[0] -= xcoeff[0];
            let (xrootn, xroots) = solve3(&scoeff);

            let mut scoeff = points2coeff(sps[0].y, sps[1].y, sps[2].y, sps[3].y);
            scoeff[0] -= ycoeff[0];
            let (yrootn, yroots) = solve3(&scoeff);

            if xrootn == 4 {
                if yrootn == 4 {
                    return 4;
                } else {
                    for j in 0..yrootn {
                        addroot(yroots[j], roots, &mut rootn);
                    }
                }
            } else if yrootn == 4 {
                for i in 0..xrootn {
                    addroot(xroots[i], roots, &mut rootn);
                }
            } else {
                for i in 0..xrootn {
                    for j in 0..yrootn {
                        if xroots[i] == yroots[j] {
                            addroot(xroots[i], roots, &mut rootn);
                        }
                    }
                }
            }
            return rootn;
        } else {
            let mut scoeff = points2coeff(sps[0].x, sps[1].x, sps[2].x, sps[3].x);
            scoeff[0] -= xcoeff[0];
            let (xrootn, xroots) = solve3(&scoeff);
            if xrootn == 4 {
                return 4;
            }
            for i in 0..xrootn {
                let tv = xroots[i];
                if tv >= 0.0 && tv <= 1.0 {
                    let scoeff = points2coeff(sps[0].y, sps[1].y, sps[2].y, sps[3].y);
                    let mut sv = scoeff[0] + tv * (scoeff[1] + tv * (scoeff[2] + tv * scoeff[3]));
                    sv = (sv - ycoeff[0]) / ycoeff[1];
                    if (0.0 <= sv) && (sv <= 1.0) {
                        addroot(tv, roots, &mut rootn);
                    }
                }
            }
            return rootn;
        }
    } else {
        let rat = ycoeff[1] / xcoeff[1];
        let mut scoeff = points2coeff(
            sps[0].y - rat * sps[0].x,
            sps[1].y - rat * sps[1].x,
            sps[2].y - rat * sps[2].x,
            sps[3].y - rat * sps[3].x,
        );
        scoeff[0] += rat * xcoeff[0] - ycoeff[0];
        let (xrootn, xroots) = solve3(&scoeff);
        if xrootn == 4 {
            return 4;
        }
        for i in 0..xrootn {
            let tv = xroots[i];
            if tv >= 0.0 && tv <= 1.0 {
                let scoeff = points2coeff(sps[0].x, sps[1].x, sps[2].x, sps[3].x);
                let mut sv = scoeff[0] + tv * (scoeff[1] + tv * (scoeff[2] + tv * scoeff[3]));
                sv = (sv - xcoeff[0]) / xcoeff[1];
                if (0.0 <= sv) && (sv <= 1.0) {
                    addroot(tv, roots, &mut rootn);
                }
            }
        }
        return rootn;
    }
}

fn points2coeff(v0: f64, v1: f64, v2: f64, v3: f64) -> [f64; 4] {
    [
        v0,
        3.0 * (v1 - v0),
        3.0 * v0 + 3.0 * v2 - 6.0 * v1,
        v3 + 3.0 * v1 - (v0 + 3.0 * v2),
    ]
}

fn addroot(root: f64, roots: &mut [f64], rootnp: &mut usize) {
    if root >= 0.0 && root <= 1.0 {
        roots[*rootnp] = root;
        *rootnp += 1;
    }
}

fn normv(v: Pvector_t) -> Pvector_t {
    let d2 = v.x * v.x + v.y * v.y;
    if d2 > 1.0e-6 {
        v / d2.sqrt()
    } else {
        v
    }
}

fn add(p1: Ppoint_t, p2: Ppoint_t) -> Ppoint_t {
    p1 + p2
}

fn sub(p1: Ppoint_t, p2: Ppoint_t) -> Ppoint_t {
    p1 - p2
}

fn dist(p1: Ppoint_t, p2: Ppoint_t) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

fn scale(p: Ppoint_t, c: f64) -> Ppoint_t {
    p * c
}

fn dot(p1: Ppoint_t, p2: Ppoint_t) -> f64 {
    p1.x * p2.x + p1.y * p2.y
}

fn B0(t: f64) -> f64 {
    let tmp = 1.0 - t;
    tmp * tmp * tmp
}

fn B1(t: f64) -> f64 {
    let tmp = 1.0 - t;
    3.0 * t * tmp * tmp
}

fn B2(t: f64) -> f64 {
    let tmp = 1.0 - t;
    3.0 * t * t * tmp
}

fn B3(t: f64) -> f64 {
    t * t * t
}

fn B01(t: f64) -> f64 {
    let tmp = 1.0 - t;
    tmp * tmp * (tmp + 3.0 * t)
}

fn B23(t: f64) -> f64 {
    let tmp = 1.0 - t;
    t * t * (3.0 * tmp + t)
}

/*#if 0
static int cmpp2efunc(const void* v0p, const void* v1p) {
    p2e_t* p2e0p, * p2e1p;
    double x0, x1;

    p2e0p = (p2e_t*)v0p, p2e1p = (p2e_t*)v1p;
    if (p2e0p->pp->y > p2e1p->pp->y)
        return -1;
    else if (p2e0p->pp->y < p2e1p->pp->y)
        return 1;
    if (p2e0p->pp->x < p2e1p->pp->x)
        return -1;
    else if (p2e0p->pp->x > p2e1p->pp->x)
        return 1;
    x0 = (p2e0p->pp == &p2e0p->ep->a) ? p2e0p->ep->b.x : p2e0p->ep->a.x;
    x1 = (p2e1p->pp == &p2e1p->ep->a) ? p2e1p->ep->b.x : p2e1p->ep->a.x;
    if (x0 < x1)
        return -1;
    else if (x0 > x1)
        return 1;
    return 0;
}

static void listdelete(Pedge_t* ep) {
    elist_t* lp;

    for (lp = elist; lp; lp = lp->next) {
        if (lp->ep != ep)
            continue;
        if (lp->prev)
            lp->prev->next = lp->next;
        if (lp->next)
            lp->next->prev = lp->prev;
        if (elist == lp)
            elist = lp->next;
        free(lp);
        return;
    }
    if (!lp) {
        prerror("cannot find list element to delete");
        abort();
    }
}

static void listreplace(Pedge_t* oldep, Pedge_t* newep) {
    elist_t* lp;

    for (lp = elist; lp; lp = lp->next) {
        if (lp->ep != oldep)
            continue;
        lp->ep = newep;
        return;
    }
    if (!lp) {
        prerror("cannot find list element to replace");
        abort();
    }
}

static void listinsert(Pedge_t* ep, Ppoint_t p) {
    elist_t* lp, * newlp, * lastlp;
    double lx;

    if (!(newlp = (elist_t*)malloc(sizeof(elist_t)))) {
        prerror("cannot malloc newlp");
        abort();
    }
    newlp->ep = ep;
    newlp->next = newlp->prev = NULL;
    if (!elist) {
        elist = newlp;
        return;
    }
    for (lp = elist; lp; lp = lp->next) {
        lastlp = lp;
        lx = lp->ep->a.x + (lp->ep->b.x - lp->ep->a.x) * (p.y -
            lp->ep->a.y) /
            (lp->ep->b.y - lp->ep->a.y);
        if (lx <= p.x)
            continue;
        if (lp->prev)
            lp->prev->next = newlp;
        newlp->prev = lp->prev;
        newlp->next = lp;
        lp->prev = newlp;
        if (elist == lp)
            elist = newlp;
        return;
    }
    lastlp->next = newlp;
    newlp->prev = lastlp;
    if (!elist)
        elist = newlp;
}
#endif
*/
