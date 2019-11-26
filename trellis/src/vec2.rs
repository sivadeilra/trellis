#[derive(Copy, Clone, Default, PartialEq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

use core::ops::{Add, Div, Mul, Sub};

impl<T: Add<T, Output = T> + Copy> Add<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T>;
    fn add(self, other: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Sub<T, Output = T> + Copy> Sub<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T>;
    fn sub(self, other: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Mul<T, Output = T> + Copy> Mul<T> for Vec2<T> {
    type Output = Vec2<T>;
    fn mul(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

/*
impl<T: Mul<T, Output=T>> Mul<Vec2<T>> for T {
    type Output = Vec2<T>;
    fn mul(self, other: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self * other.x,
            y: self * other.y,
        }
    }
}
*/

impl Mul<Vec2<f64>> for f64 {
    type Output = Vec2<f64>;
    fn mul(self, other: Vec2<f64>) -> Self::Output {
        Vec2 {
            x: self * other.x,
            y: self * other.y,
        }
    }
}

impl<T: Div<T, Output = T> + Copy> Div<T> for Vec2<T> {
    type Output = Vec2<T>;
    fn div(self, other: T) -> Self::Output {
        Vec2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

pub type Pxy_t = Vec2<f64>;
pub type Ppoint_t = Pxy_t;
pub type Pvector_t = Pxy_t;

impl Vec2<f64> {
    pub fn from_angle(theta: f64) -> Self {
        let (s, c) = theta.sin_cos();
        Self {
            x: c,
            y: s,
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Vec2<T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "({:?}, {:?})", self.x, self.y)
    }
}