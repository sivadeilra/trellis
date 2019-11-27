pub struct paird {
    pub p1: f64,
    pub p2: f64,
}

pub struct pair {
    pub a: i32,
    pub b: i32,
}

pub struct pair2 {
    pub t1: pair,
    pub t2: pair,
}

pub enum bend {
    B_NODE,
    B_UP,
    B_LEFT,
    B_DOWN,
    B_RIGHT,
}

/* Example : segment connecting maze point (3,2)
 * and (3,8) has isVert = 1, common coordinate = 3, p1 = 2, p2 = 8
 */
pub struct segment {
    pub isVert: bool,
    pub flipped: bool,
    pub comm_coord: f64, /* the common coordinate */
    pub p: paird,        /* end points */
    pub l1: bend,
    pub l2: bend,
    pub ind_no: i32,   /* index number of this segment in its channel */
    pub track_no: i32, /* track number assigned in the channel */
    pub prev: *mut segment,
    pub next: *mut segment,
}

pub struct route {
    pub n: i32,
    pub segs: *mut segment,
}

pub struct channel {
    //  pub link: Dtlink_t,
    pub p: paird,                    /* extrema of channel */
    pub cnt: i32,                    /* number of segments */
    pub seg_list: *mut *mut segment, /* array of segment pointers */
    pub G: *mut rawgraph,
    pub cp: *mut cell,
}

/*#if 0
typedef struct {
  int i1, i2, j;
  int cnt;
  int* seg_list;  /* list of indices of the segment list */

  rawgraph* G;
} hor_channel;

typedef struct {
    hor_channel* hs;
    int cnt;
} vhor_channel;

typedef struct {
  int i, j1, j2;
  int cnt;
  int* seg_list;  /* list of indices of the segment list */

  rawgraph* G;
} vert_channel;

typedef struct {
    vert_channel* vs;
    int cnt;
} vvert_channel;
#endif*/

// #define N_DAD(n) (n)->n_dad
