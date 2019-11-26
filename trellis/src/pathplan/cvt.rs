use super::vis::*;
use super::*;
use crate::polyline::Ppoly_t;
use crate::vec2::Ppoint_t;

pub type ilcoord_t = Ppoint_t;

/* open a visibility graph
 * Points in polygonal obstacles must be in clockwise order.
 */
pub fn Pobsopen(obs: &[&Ppoly_t]) -> vconfig_t {
    /* get storage */
    let n: usize = obs.iter().map(|p| p.ps.len()).sum();

    let mut rv: vconfig_t = vconfig_t {
        P: Vec::with_capacity(n),
        start: vec![0; obs.len() + 1],
        next: vec![0; n],
        prev: vec![0; n],
        N: n,
        Npoly: obs.len(),
        ..<vconfig_t as Default>::default()
    };

    /* build arrays */
    let mut i: usize = 0;
    for (poly_i, poly) in obs.iter().enumerate() {
        let start = i;
        rv.start[poly_i] = start as i32;
        let end = start + poly.ps.len() - 1;
        for pt_i in poly.ps.iter() {
            rv.P.push(*pt_i);
            rv.next[i] = i as i32 + 1;
            rv.prev[i] = i as i32 - 1;
            i += 1;
        }
        rv.next[end] = start as i32;
        rv.prev[start] = end as i32;
    }
    rv.start[obs.len()] = i as i32;
    super::visibility::visibility(&mut rv);
    rv
}

/*
void Pobsclose(vconfig_t * config)
{
    if (config->vis) {
        free(config->vis[0]);
        free(config->vis);
    }
    delete config;
}
*/

/*

/* route a polyline from p0 to p1, avoiding obstacles.
 * if an endpoint is inside an obstacle, pass the polygon's index >=0
 * if the endpoint is not inside an obstacle, pass POLYID_NONE
 * if the endpoint location is not known, pass POLYID_UNKNOWN
 */
int Pobspath(vconfig_t * config, Ppoint_t p0, int poly0, Ppoint_t p1,
         int poly1, Ppolyline_t * output_route)
{
    int i, j, *dad;
    size_t opn;
    COORD *ptvis0, *ptvis1;

#ifdef GASP
    gasp_print_obstacles(config);
#endif
    ptvis0 = ptVis(config, poly0, p0);
    ptvis1 = ptVis(config, poly1, p1);

#ifdef GASP
    gasp_print_point(p0);
    gasp_print_point(p1);
#endif
    dad = makePath(p0, poly0, ptvis0, p1, poly1, ptvis1, config);

    opn = 1;
    for (i = dad[config->N]; i != config->N + 1; i = dad[i])
    opn++;
    opn++;
    std::vector<Ppoint_t> ops{ opn };

    j = opn - 1;
    ops[j--] = p1;
    for (i = dad[config->N]; i != config->N + 1; i = dad[i])
    ops[j--] = config->P[i];
    ops[j] = p0;
    assert(j == 0);

#ifdef DEBUG
    printVconfig(config);
    printVis("p", ptvis0, config->N + 1);
    printVis("q", ptvis1, config->N + 1);
    printDad(dad, config->N + 1);
#endif

    if (ptvis0)
    free(ptvis0);
    if (ptvis1)
    free(ptvis1);

    output_route->ps = std::move(ops);
#ifdef GASP
    gasp_print_polyline(output_route);
#endif
    free(dad);
    return TRUE;
}

int Pobsbarriers(vconfig_t* config, std::vector<Pedge_t>* barriers, int* n_barriers) {
    barriers->clear();
    barriers->resize(config->N);

    for (int i = 0; i < config->N; i++) {
        int j = config->next[i];
        (*barriers)[i] = Pedge_t{
            config->P[i],
            config->P[j]
        };
    }
    return 1;
}

#ifdef DEBUG
static void printVconfig(vconfig_t * cp)
{
    int i, j;
    int *next, *prev;
    Ppoint_t *pts;
    array2 arr;

    next = cp->next;
    prev = cp->prev;
    pts = cp->P;
    arr = cp->vis;

    printf("this next prev point\n");
    for (i = 0; i < cp->N; i++)
    printf("%3d  %3d  %3d    (%3g,%3g)\n", i, next[i], prev[i],
           pts[i].x, pts[i].y);

    printf("\n\n");

    for (i = 0; i < cp->N; i++) {
    for (j = 0; j < cp->N; j++)
        printf("%4.1f ", arr[i][j]);
    printf("\n");
    }
}

static void printVis(char *lbl, COORD * vis, int n)
{
    int i;

    printf("%s: ", lbl);
    for (i = 0; i < n; i++)
    printf("%4.1f ", vis[i]);
    printf("\n");
}

static void printDad(int *vis, int n)
{
    int i;

    printf("     ");
    for (i = 0; i < n; i++) {
    printf("%3d ", i);
    }
    printf("\n");
    printf("dad: ");
    for (i = 0; i < n; i++) {
    printf("%3d ", vis[i]);
    }
    printf("\n");
}
#endif

#ifdef GASP

static Ppoint_t Bezpt[1000];
static int Bezctr;

static void addpt(Ppoint_t p)
{
    if ((Bezctr == 0) ||
    (Bezpt[Bezctr - 1].x != p.x) || (Bezpt[Bezctr - 1].y != p.y))
    Bezpt[Bezctr++] = p;
}

#define W_DEGREE 5
static ilcoord_t Bezier(ilcoord_t * V, int degree, double t,
            ilcoord_t * Left, ilcoord_t * Right)
{
    int i, j;			/* Index variables  */
    ilcoord_t Vtemp[W_DEGREE + 1][W_DEGREE + 1];

    /* Copy control points  */
    for (j = 0; j <= degree; j++) {
    Vtemp[0][j] = V[j];
    }

    /* Triangle computation */
    for (i = 1; i <= degree; i++) {
    for (j = 0; j <= degree - i; j++) {
        Vtemp[i][j].x =
        (1.0 - t) * Vtemp[i - 1][j].x + t * Vtemp[i - 1][j + 1].x;
        Vtemp[i][j].y =
        (1.0 - t) * Vtemp[i - 1][j].y + t * Vtemp[i - 1][j + 1].y;
    }
    }

    if (Left != NIL(ilcoord_t *))
    for (j = 0; j <= degree; j++)
        Left[j] = Vtemp[j][0];
    if (Right != NIL(ilcoord_t *))
    for (j = 0; j <= degree; j++)
        Right[j] = Vtemp[degree - j][j];
    return (Vtemp[degree][0]);
}

static void append_bezier(Ppoint_t * bezier)
{
    double a;
    ilcoord_t left[4], right[4];

    a = fabs(area2(bezier[0], bezier[1], bezier[2]))
    + fabs(area2(bezier[2], bezier[3], bezier[0]));
    if (a < .5) {
    addpt(bezier[0]);
    addpt(bezier[3]);
    } else {
    (void) Bezier(bezier, 3, .5, left, right);
    append_bezier(left);
    append_bezier(right);
    }
}

FILE *GASPout = stderr;

static void gasp_print_point(Ppoint_t p)
{
    fprintf(GASPout, "%3g %3g\n", p.x, p.y);
}

void gasp_print_obstacles(vconfig_t * conf)
{
    int i, j;
    Ppoly_t poly;

    fprintf(GASPout, "%d\n", conf->Npoly);
    for (i = 0; i < conf->Npoly; i++) {
    poly.ps = &(conf->P[conf->start[i]]);
    poly.pn = conf->start[i + 1] - conf->start[i];
    fprintf(GASPout, "%d\n", poly.pn);
    for (j = 0; j < poly.pn; j++)
        gasp_print_point(poly.ps[j]);
    }
}

void gasp_print_bezier(Ppolyline_t * route)
{
    int i;

    Bezctr = 0;
    for (i = 0; i + 3 < route->pn; i += 3)
    append_bezier(route->ps + i);
    fprintf(GASPout, "%d\n", Bezctr);
    for (i = 0; i < Bezctr; i++)
    gasp_print_point(Bezpt[i]);
    Bezctr = 0;
}

void gasp_print_polyline(Ppolyline_t * route)
{
    int i;

    fprintf(GASPout, "%d\n", route->pn);
    for (i = 0; i < route->pn; i++)
    gasp_print_point(route->ps[i]);
}
#endif
*/
