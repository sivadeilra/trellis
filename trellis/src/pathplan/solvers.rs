use core::f64::consts::PI;

fn cbrt(x: f64) -> f64 {
    if x < 0.0 {
        -1.0 * (-x).powf(1.0 / 3.0)
    } else {
        x.powf(1.0 / 3.0)
    }
}

const EPS: f64 = 1.0E-7;
const fn AEQ0(x: f64) -> bool {
    (x < EPS) && (x > -EPS)
}

pub fn solve3(coeff: &[f64; 4]) -> (usize, [f64; 4]) {
    let mut roots: [f64; 4] = [0.0, 0.0, 0.0, 0.0];
    let mut rootn: usize = 0;

    let a = coeff[3];
    let b = coeff[2];
    let c = coeff[1];
    let d = coeff[0];
    if AEQ0(a) {
        return solve2(coeff);
    }
    let b_over_3a = b / (3.0 * a);
    let c_over_a = c / a;
    let d_over_a = d / a;

    let p = b_over_3a * b_over_3a;
    let q = 2.0 * b_over_3a * p - b_over_3a * c_over_a + d_over_a;
    let p = c_over_a / 3.0 - p;
    let disc = q * q + 4.0 * p * p * p;

    if disc < 0.0 {
        let r = 0.5 * (-disc + q * q).sqrt();
        let theta = f64::atan2((-disc).sqrt(), -q);
        let temp = 2.0 * cbrt(r);
        roots[0] = temp * (theta / 3.0).cos();
        roots[1] = temp * ((theta + PI + PI) / 3.0).cos();
        roots[2] = temp * ((theta - PI - PI) / 3.0).cos();
        rootn = 3;
    } else {
        let alpha = 0.5 * (disc.sqrt() - q);
        let beta = -q - alpha;
        roots[0] = cbrt(alpha) + cbrt(beta);
        if disc > 0.0 {
            rootn = 1;
        } else {
            roots[1] = -0.5 * roots[0];
            roots[2] = -0.5 * roots[0];
            rootn = 3;
        }
    }

    for r in roots[..rootn].iter_mut() {
        *r -= b_over_3a;
    }

    (rootn, roots)
}

fn solve2(coeff: &[f64; 4]) -> (usize, [f64; 4]) {
    let a = coeff[2];
    let b = coeff[1];
    let c = coeff[0];
    if AEQ0(a) {
        return solve1(coeff);
    }
    let b_over_2a = b / (2.0 * a);
    let c_over_a = c / a;

    let disc = b_over_2a * b_over_2a - c_over_a;
    if disc < 0.0 {
        (0, [0.0; 4])
    } else if disc == 0.0 {
        (1, [-b_over_2a, 0.0, 0.0, 0.0])
    } else {
        let root0 = -b_over_2a + disc.sqrt();
        let root1 = -2.0 * b_over_2a - root0;
        (2, [root0, root1, 0.0, 0.0])
    }
}

fn solve1(coeff: &[f64; 4]) -> (usize, [f64; 4]) {
    let a = coeff[1];
    let b = coeff[0];
    if AEQ0(a) {
        if AEQ0(b) {
            return (4, [0.0; 4]);
        } else {
            return (0, [0.0; 4]);
        }
    } else {
        (1, [-b / a, 0.0, 0.0, 0.0])
    }
}
