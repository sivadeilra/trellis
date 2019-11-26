pub fn MIN<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

pub fn MAX<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

pub fn AVG(a: f64, b: f64) -> f64 { (a + b) / 2.0 }

pub fn SGN(a: f64) -> i32 { if a < 0.0 { -1 } else { 1 } }

pub fn CMP(a: f64 , b: f64) -> i32 { 
    match () {
        () if a < b => -1,
        () if a > b => 1,
        _ => 0
    }
}

pub fn BETWEEN(a: f64, b: f64, c: f64) -> bool {
    a <= b && b <= c
}	

pub const M_PI: f64 = core::f64::consts::PI;
pub const SQRT2: f64 = core::f64::consts::SQRT_2;

pub fn ROUND(f: f64) -> i32 {
    if f >= 0.0 {
        (f + 0.5) as i32
    } else {
        (f - 0.5) as i32
    }
}
pub fn RADIANS(deg: f64) -> f64 { deg / 180.0 * M_PI }
pub fn DEGREES(rad: f64) -> f64 { rad / M_PI * 180.0 }

pub fn SQR(a: f64) -> f64 { a * a }

