use super::array2::Array2;
use super::*;
use crate::vec2::Ppoint_t;

pub type COORD = f64;

const OBSCURED: f64 = 0.0;
/*
#define EQ(p,q)		((p.x == q.x) && (p.y == q.y))
#define NEQ(p,q)	(!EQ(p,q))
#define NIL(p)		((p)0)
#define	CW			0
#define	CCW			1
*/

#[derive(Default, Clone, Debug)]
pub struct vconfig_t {
    pub Npoly: usize,
    pub N: usize,         /* number of points in walk of barriers */
    pub P: Vec<Ppoint_t>, /* barrier points */
    pub start: Vec<i32>,
    pub next: Vec<i32>,
    pub prev: Vec<i32>,

    pub vis: Array2<COORD>,
}
