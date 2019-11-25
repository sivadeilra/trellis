use super::visibility::wind;
use super::*;

pub fn in_poly(ps: &[Ppoint_t], q: Ppoint_t) -> bool {
    for (i, p) in ps.iter().enumerate() {
        // point index; i1 = i-1 mod n
        let i1 = (i + ps.len() - 1) % ps.len();
        if wind(ps[i1], *p, q) == 1 {
            return false;
        }
    }
    true
}
