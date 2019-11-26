use super::visibility::wind;
use crate::vec2::Vec2;

pub fn in_poly(ps: &[Vec2<f64>], q: Vec2<f64>) -> bool {
    for (i, p) in ps.iter().enumerate() {
        // point index; i1 = i-1 mod n
        let i1 = (i + ps.len() - 1) % ps.len();
        if wind(ps[i1], *p, q) == 1 {
            return false;
        }
    }
    true
}
