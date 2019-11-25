// port of graphviz path planner

#![allow(non_camel_case_types)]

pub mod array2;
pub mod cvt;
pub mod in_poly;
pub mod route;
pub mod shortest;
pub mod shortestpth;
pub mod solvers;
pub mod vis;
pub mod visibility;

pub type COORD = f64;

pub const POLYID_NONE: i32 = -1111;
pub const POLYID_UNKNOWN: i32 = -2222;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pxy_t {
    pub x: f64,
    pub y: f64,
}

impl core::ops::Add<Pxy_t> for Pxy_t {
    type Output = Pxy_t;
    fn add(self, other: Pxy_t) -> Self::Output {
        Self::Output {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl core::ops::Sub<Pxy_t> for Pxy_t {
    type Output = Pxy_t;
    fn sub(self, other: Pxy_t) -> Self::Output {
        Self::Output {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl core::ops::Mul<f64> for Pxy_t {
    type Output = Pxy_t;
    fn mul(self, other: f64) -> Self::Output {
        Self::Output {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl core::ops::Mul<Pxy_t> for f64 {
    type Output = Pxy_t;
    fn mul(self, other: Pxy_t) -> Self::Output {
        Self::Output {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl core::ops::Div<f64> for Pxy_t {
    type Output = Pxy_t;
    fn div(self, other: f64) -> Self::Output {
        Self::Output {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

pub type Ppoint_t = Pxy_t;
pub type Pvector_t = Pxy_t;

#[derive(Clone, Debug, Default)]
pub struct Ppoly_t {
    pub ps: Vec<Ppoint_t>,
}

pub type Ppolyline_t = Ppoly_t;

#[derive(Clone, Debug, Default)]
pub struct Pedge_t {
    pub a: Ppoint_t,
    pub b: Ppoint_t,
}
