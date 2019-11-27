/* geometric types and macros (e.g. points and boxes) with application to, but
 * no specific dependence on graphs */

use crate::common::arith::{BETWEEN, ROUND};
use crate::math::{fmax, fmin};
use crate::vec2::Vec2;

pub type point = Vec2<i32>;
pub type pointf = Vec2<f64>;

// was named "box"
pub struct boxi {
    pub LL: point,
    pub UR: point,
}

pub struct boxf {
    pub LL: pointf,
    pub UR: pointf,
}

/* true if point p is inside box b */
pub fn INSIDE(p: pointf, b: &boxf) -> bool {
    BETWEEN(b.LL.x, p.x, b.UR.x) && BETWEEN(b.LL.y, p.y, b.UR.y)
}

/* true if boxes b0 and b1 overlap */
pub fn OVERLAP(b0: &boxf, b1: &boxf) -> bool {
    (b0.UR.x >= b1.LL.x) && (b1.UR.x >= b0.LL.x) && (b0.UR.y >= b1.LL.y) && (b1.UR.y >= b0.LL.y)
}

/* true if box b0 completely contains b1*/
pub fn CONTAINS(b0: &boxf, b1: &boxf) -> bool {
    (b0.UR.x >= b1.UR.x) && (b0.UR.y >= b1.UR.y) && (b0.LL.x <= b1.LL.x) && (b0.LL.y <= b1.LL.y)
}

/* expand box b as needed to enclose point p */
pub fn EXPANDBP(b: &mut boxf, p: pointf) {
    b.LL.x = fmin(b.LL.x, p.x);
    b.LL.y = fmin(b.LL.y, p.y);
    b.UR.x = fmax(b.UR.x, p.x);
    b.UR.y = fmax(b.UR.y, p.y);
}

/* expand box b0 as needed to enclose box b1 */
pub fn EXPANDBB(b0: &mut boxf, b1: &boxf) {
    b0.LL.x = fmin(b0.LL.x, b1.LL.x);
    b0.LL.y = fmin(b0.LL.y, b1.LL.y);
    b0.UR.x = fmax(b0.UR.x, b1.UR.x);
    b0.UR.y = fmax(b0.UR.y, b1.UR.y);
}

/* clip box b0 to fit box b1 */
pub fn CLIPBB(b0: &mut boxf, b1: &boxf) {
    b0.LL.x = fmax(b0.LL.x, b1.LL.x);
    b0.LL.y = fmax(b0.LL.y, b1.LL.y);
    b0.UR.x = fmin(b0.UR.x, b1.UR.x);
    b0.UR.y = fmin(b0.UR.y, b1.UR.y);
}

use crate::math::sqr as SQR;

pub fn LEN2(a: f64, b: f64) -> f64 {
    SQR(a) + SQR(b)
}
pub fn LEN(a: f64, b: f64) -> f64 {
    LEN2(a, b).sqrt()
}

pub fn DIST2(p: pointf, q: pointf) -> f64 {
    LEN2(p.x - q.x, p.y - q.y)
}
pub fn DIST(p: pointf, q: pointf) -> f64 {
    DIST2(p, q).sqrt()
}

pub const POINTS_PER_INCH: f64 = 72.0;
pub const POINTS_PER_PC: f64 = (POINTS_PER_INCH / 6.0);
pub const POINTS_PER_CM: f64 = (POINTS_PER_INCH * 0.393700787);
pub const POINTS_PER_MM: f64 = (POINTS_PER_INCH * 0.0393700787);

pub fn POINTS(a_inches: f64) -> i32 {
    ROUND((a_inches) * POINTS_PER_INCH)
}
pub fn INCH2PS(a_inches: f64) -> f64 {
    a_inches * POINTS_PER_INCH
}
pub fn PS2INCH(a_points: f64) -> f64 {
    a_points / POINTS_PER_INCH
}

pub fn P2PF(p: point) -> pointf {
    pointf {
        x: p.x as f64,
        y: p.y as f64,
    }
}

pub fn PF2P(pf: pointf) -> point {
    point {
        x: ROUND(pf.x) as i32,
        y: ROUND(pf.y) as i32,
    }
}

pub fn B2BF(b: &boxi) -> boxf {
    boxf {
        LL: P2PF(b.LL),
        UR: P2PF(b.UR),
    }
}

pub fn BF2B(bf: &boxf) -> boxi {
    boxi {
        LL: PF2P(bf.LL),
        UR: PF2P(bf.UR),
    }
}

pub fn APPROXEQ(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

pub fn APPROXEQPT(p: pointf, q: pointf, tol: f64) -> bool {
    DIST2(p, q) < SQR(tol)
}

/* some common tolerance values */
pub const MILLIPOINT: f64 = 0.001;
pub const MICROPOINT: f64 = 0.000001;
