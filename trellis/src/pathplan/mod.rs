// port of graphviz path planner

pub mod array2;
pub mod cvt;
pub mod in_poly;
pub mod route;
pub mod shortest;
pub mod shortestpth;
pub mod solvers;
pub mod triang;
pub mod vis;
pub mod visibility;

pub type COORD = f64;

pub const POLYID_NONE: i32 = -1111;
pub const POLYID_UNKNOWN: i32 = -2222;

use crate::vec2::Vec2;

#[derive(Clone, Debug, Default)]
pub struct Pedge_t {
    pub a: Vec2<f64>,
    pub b: Vec2<f64>,
}
