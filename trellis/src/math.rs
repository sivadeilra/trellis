pub fn fmin(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn fmax(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

use core::ops::Mul;

pub fn sqr<T>(a: T) -> T
where
    T: Copy + Mul<T, Output = T>,
{
    a * a
}
