/*************************************************************************
 * Copyright (c) 2012 AT&T Intellectual Property
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v1.0
 * which accompanies this distribution, and is available at
 * http://www.eclipse.org/legal/epl-v10.html
 *
 * Contributors: See CVS logs. Details at http://www.graphviz.org/
 *************************************************************************/

/* This code is derived from the Java implementation by Luc Maisonobe */
/* Copyright (c) 2003-2004, Luc Maisonobe
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with
 * or without modification, are permitted provided that
 * the following conditions are met:
 *
 *    Redistributions of source code must retain the
 *    above copyright notice, this list of conditions and
 *    the following disclaimer.
 *    Redistributions in binary form must reproduce the
 *    above copyright notice, this list of conditions and
 *    the following disclaimer in the documentation
 *    and/or other materials provided with the
 *    distribution.
 *    Neither the names of spaceroots.org, spaceroots.com
 *    nor the names of their contributors may be used to
 *    endorse or promote products derived from this
 *    software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
 * CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED
 * WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
 * PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
 * THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF
 * USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER
 * IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 * NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
 * USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use crate::polyline::Ppolyline_t;
use crate::vec2::Ppoint_t;
use core::f64::consts::PI;

//use crate::pathplan;

// #include "render.h"
// #include "pathplan.h"

const TWOPI: f64 = 2.0 * PI;

fn fmin(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

fn fmax(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

#[derive(Clone, Default)]
struct Ellipse {
    /* center */
    c: Vec2f,

    /* semi-major and -minor axes */
    a: f64,
    b: f64,

    /* Orientation of the major axis with respect to the x axis. */
    theta: f64,
    cosTheta: f64,
    sinTheta: f64,

    /* Start and end angles of the arc. */
    eta1: f64,
    eta2: f64,

    /* Position of the start and end points. */
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,

    /* Position of the foci. */
    xF1: f64,
    yF1: f64,
    xF2: f64,
    yF2: f64,

    /* x of the leftmost point of the arc. */
    xLeft: f64,

    /* y of the highest point of the arc. */
    yUp: f64,

    /* Horizontal width and vertical height of the arc. */
    width: f64,
    height: f64,

    f: f64,
    e2: f64,
    g: f64,
    g2: f64,
}

fn computeFoci(ep: &mut Ellipse) {
    let d = (ep.a * ep.a - ep.b * ep.b).sqrt();
    let dx = d * ep.cosTheta;
    let dy = d * ep.sinTheta;

    ep.xF1 = ep.c.x - dx;
    ep.yF1 = ep.c.y - dy;
    ep.xF2 = ep.c.x + dx;
    ep.yF2 = ep.c.y + dy;
}

/* Compute the locations of the endpoints. */
fn computeEndPoints(ep: &mut Ellipse) {
    let aCosEta1 = ep.a * ep.eta1.cos();
    let bSinEta1 = ep.b * ep.eta1.sin();
    let aCosEta2 = ep.a * ep.eta2.cos();
    let bSinEta2 = ep.b * ep.eta2.sin();

    // start point
    ep.x1 = ep.c.x + aCosEta1 * ep.cosTheta - bSinEta1 * ep.sinTheta;
    ep.y1 = ep.c.y + aCosEta1 * ep.sinTheta + bSinEta1 * ep.cosTheta;

    // end point
    ep.x2 = ep.c.x + aCosEta2 * ep.cosTheta - bSinEta2 * ep.sinTheta;
    ep.y2 = ep.c.y + aCosEta2 * ep.sinTheta + bSinEta2 * ep.cosTheta;
}

/* Compute the bounding box. */
fn computeBounds(ep: &mut Ellipse) {
    let bOnA = ep.b / ep.a;
    let mut etaXMin;
    let mut etaXMax;
    let mut etaYMin;
    let mut etaYMax;

    if ep.sinTheta.abs() < 0.1 {
        let tanTheta = ep.sinTheta / ep.cosTheta;
        if ep.cosTheta < 0.0 {
            etaXMin = -(tanTheta * bOnA).atan();
            etaXMax = etaXMin + PI;
            etaYMin = 0.5 * PI - (tanTheta / bOnA).atan();
            etaYMax = etaYMin + PI;
        } else {
            etaXMax = -(tanTheta * bOnA).atan();
            etaXMin = etaXMax - PI;
            etaYMax = 0.5 * PI - (tanTheta / bOnA).atan();
            etaYMin = etaYMax - PI;
        }
    } else {
        let invTanTheta = ep.cosTheta / ep.sinTheta;
        if ep.sinTheta < 0.0 {
            etaXMax = 0.5 * PI + (invTanTheta / bOnA).atan();
            etaXMin = etaXMax - PI;
            etaYMin = (invTanTheta * bOnA).atan();
            etaYMax = etaYMin + PI;
        } else {
            etaXMin = 0.5 * PI + (invTanTheta / bOnA).atan();
            etaXMax = etaXMin + PI;
            etaYMax = (invTanTheta * bOnA).atan();
            etaYMin = etaYMax - PI;
        }
    }

    etaXMin -= TWOPI * ((etaXMin - ep.eta1) / TWOPI).floor();
    etaYMin -= TWOPI * ((etaYMin - ep.eta1) / TWOPI).floor();
    etaXMax -= TWOPI * ((etaXMax - ep.eta1) / TWOPI).floor();
    etaYMax -= TWOPI * ((etaYMax - ep.eta1) / TWOPI).floor();

    ep.xLeft = if etaXMin <= ep.eta2 {
        (ep.c.x + ep.a * etaXMin.cos() * ep.cosTheta - ep.b * etaXMin.sin() * ep.sinTheta)
    } else {
        fmin(ep.x1, ep.x2)
    };
    ep.yUp = if etaYMin <= ep.eta2 {
        (ep.c.y + ep.a * etaYMin.cos() * ep.sinTheta + ep.b * etaYMin.sin() * ep.cosTheta)
    } else {
        fmin(ep.y1, ep.y2)
    };
    ep.width = (if etaXMax <= ep.eta2 {
        (ep.c.x + ep.a * etaXMax.cos() * ep.cosTheta - ep.b * etaXMax.sin() * ep.sinTheta)
    } else {
        fmax(ep.x1, ep.x2)
    }) - ep.xLeft;
    ep.height = (if etaYMax <= ep.eta2 {
        (ep.c.y + ep.a * etaYMax.cos() * ep.sinTheta + ep.b * etaYMax.sin() * ep.cosTheta)
    } else {
        fmax(ep.y1, ep.y2)
    }) - ep.yUp;
}

fn initEllipse(
    cx: f64,
    cy: f64,
    a: f64,
    b: f64,
    theta: f64,
    lambda1: f64,
    lambda2: f64,
) -> Ellipse {
    let mut ep: Ellipse = Default::default();

    ep.c = point(cx, cy);
    ep.a = a;
    ep.b = b;
    ep.theta = theta;

    ep.eta1 = f64::atan2(lambda1.sin() / b, lambda1.cos() / a);
    ep.eta2 = f64::atan2(lambda2.sin() / b, lambda2.cos() / a);
    ep.cosTheta = theta.cos();
    ep.sinTheta = theta.sin();

    // make sure we have eta1 <= eta2 <= eta1 + 2*PI
    ep.eta2 -= TWOPI * ((ep.eta2 - ep.eta1) / TWOPI).floor();

    // the preceding correction fails if we have exactly eta2 - eta1 = 2*PI
    // it reduces the interval to zero length
    if (lambda2 - lambda1 > PI) && (ep.eta2 - ep.eta1 < PI) {
        ep.eta2 += TWOPI;
    }

    computeFoci(&mut ep);
    computeEndPoints(&mut ep);
    computeBounds(&mut ep);

    /* Flatness parameters */
    ep.f = (ep.a - ep.b) / ep.a;
    ep.e2 = ep.f * (2.0 - ep.f);
    ep.g = 1.0 - ep.f;
    ep.g2 = ep.g * ep.g;

    ep
}

type ErrArray = [[[f64; 4]; 4]; 2];

// coefficients for error estimation
// while using quadratic Bezier curves for approximation
// 0 < b/a < 1/4
static COEFFS2_LOW: ErrArray = [
    [
        [3.92478, -13.5822, -0.233377, 0.0128206],
        [-1.08814, 0.859987, 0.000362265, 0.000229036],
        [-0.942512, 0.390456, 0.0080909, 0.00723895],
        [-0.736228, 0.20998, 0.0129867, 0.0103456],
    ],
    [
        [-0.395018, 6.82464, 0.0995293, 0.0122198],
        [-0.545608, 0.0774863, 0.0267327, 0.0132482],
        [0.0534754, -0.0884167, 0.012595, 0.0343396],
        [0.209052, -0.0599987, -0.00723897, 0.00789976],
    ],
];

// coefficients for error estimation
// while using quadratic Bezier curves for approximation
// 1/4 <= b/a <= 1
static COEFFS2_HIGH: ErrArray = [
    [
        [0.0863805, -11.5595, -2.68765, 0.181224],
        [0.242856, -1.81073, 1.56876, 1.68544],
        [0.233337, -0.455621, 0.222856, 0.403469],
        [0.0612978, -0.104879, 0.0446799, 0.00867312],
    ],
    [
        [0.028973, 6.68407, 0.171472, 0.0211706],
        [0.0307674, -0.0517815, 0.0216803, -0.0749348],
        [-0.0471179, 0.1288, -0.0781702, 2.0],
        [-0.0309683, 0.0531557, -0.0227191, 0.0434511],
    ],
];

/// safety factor to convert the "best" error approximation
/// into a "max bound" error
static SAFETY2: [f64; 4] = [0.02, 2.83, 0.125, 0.01];

/// coefficients for error estimation
/// while using cubic Bezier curves for approximation
/// 0 < b/a < 1/4
static COEFFS3_LOW: ErrArray = [
    [
        [3.85268, -21.229, -0.330434, 0.0127842],
        [-1.61486, 0.706564, 0.225945, 0.263682],
        [-0.910164, 0.388383, 0.00551445, 0.00671814],
        [-0.630184, 0.192402, 0.0098871, 0.0102527],
    ],
    [
        [-0.162211, 9.94329, 0.13723, 0.0124084],
        [-0.253135, 0.00187735, 0.0230286, 0.01264],
        [-0.0695069, -0.0437594, 0.0120636, 0.0163087],
        [-0.0328856, -0.00926032, -0.00173573, 0.00527385],
    ],
];

/// coefficients for error estimation
/// while using cubic Bezier curves for approximation
/// 1/4 <= b/a <= 1
static COEFFS3_HIGH: ErrArray = [
    [
        [0.0899116, -19.2349, -4.11711, 0.183362],
        [0.138148, -1.45804, 1.32044, 1.38474],
        [0.230903, -0.450262, 0.219963, 0.414038],
        [0.0590565, -0.101062, 0.0430592, 0.0204699],
    ],
    [
        [0.0164649, 9.89394, 0.0919496, 0.00760802],
        [0.0191603, -0.0322058, 0.0134667, -0.0825018],
        [0.0156192, -0.017535, 0.00326508, -0.228157],
        [-0.0236752, 0.0405821, -0.0173086, 0.176187],
    ],
];

/// safety factor to convert the "best" error approximation
/// into a "max bound" error
static SAFETY3: [f64; 4] = [0.001, 4.98, 0.207, 0.0067];

/* Compute the value of a rational function.
 * This method handles rational functions where the numerator is
 * quadratic and the denominator is linear
 */
fn rational_function(x: f64, c: &[f64; 4]) -> f64 {
    (x * (x * c[0] + c[1]) + c[2]) / (x + c[3])
}

/* Estimate the approximation error for a sub-arc of the instance.
 * degree specifies degree of the Bezier curve to use (1, 2 or 3)
 * tA and tB give the start and end angle of the subarc
 * Returns upper bound of the approximation error between the Bezier
 * curve and the real ellipse
 */
fn estimateError(ep: &mut Ellipse, degree: i32, etaA: f64, etaB: f64) -> f64 {
    let eta = 0.5 * (etaA + etaB);

    if degree < 2 {
        // start point
        let aCosEtaA = ep.a * etaA.cos();
        let bSinEtaA = ep.b * etaA.sin();
        let xA = ep.c.x + aCosEtaA * ep.cosTheta - bSinEtaA * ep.sinTheta;
        let yA = ep.c.y + aCosEtaA * ep.sinTheta + bSinEtaA * ep.cosTheta;

        // end point
        let aCosEtaB = ep.a * etaB.cos();
        let bSinEtaB = ep.b * etaB.sin();
        let xB = ep.c.x + aCosEtaB * ep.cosTheta - bSinEtaB * ep.sinTheta;
        let yB = ep.c.y + aCosEtaB * ep.sinTheta + bSinEtaB * ep.cosTheta;

        // maximal error point
        let aCosEta = ep.a * eta.cos();
        let bSinEta = ep.b * eta.sin();
        let x = ep.c.x + aCosEta * ep.cosTheta - bSinEta * ep.sinTheta;
        let y = ep.c.y + aCosEta * ep.sinTheta + bSinEta * ep.cosTheta;
        let dx = xB - xA;
        let dy = yB - yA;
        return (x * dy - y * dx + xB * yA - xA * yB).abs() / (dx * dx + dy * dy).sqrt();
    } else {
        let x = ep.b / ep.a;
        let dEta = etaB - etaA;
        let cos2 = (2.0 * eta).cos();
        let cos4 = (4.0 * eta).cos();
        let cos6 = (6.0 * eta).cos();

        // select the right coefficient's set according to degree and b/a
        let coeffs: &ErrArray;
        let safety: &[f64; 4];
        if degree == 2 {
            coeffs = if x < 0.25 {
                &COEFFS2_LOW
            } else {
                &COEFFS2_HIGH
            };
            safety = &SAFETY2;
        } else {
            coeffs = if x < 0.25 {
                &COEFFS3_LOW
            } else {
                &COEFFS3_HIGH
            };
            safety = &SAFETY3;
        }

        let c0 = rational_function(x, &coeffs[0][0])
            + cos2 * rational_function(x, &coeffs[0][1])
            + cos4 * rational_function(x, &coeffs[0][2])
            + cos6 * rational_function(x, &coeffs[0][3]);

        let c1 = rational_function(x, &coeffs[1][0])
            + cos2 * rational_function(x, &coeffs[1][1])
            + cos4 * rational_function(x, &coeffs[1][2])
            + cos6 * rational_function(x, &coeffs[1][3]);

        return rational_function(x, safety) * ep.a * (c0 + c1 * dEta).exp();
    }
}

type Vec2f = crate::vec2::Vec2<f64>;

fn point(x: f64, y: f64) -> Ppoint_t {
    Ppoint_t { x, y }
}

pub trait PathTarget {
    fn move_to(&mut self, p: Vec2f);
    fn line_to(&mut self, p: Vec2f);
    fn curve_to(&mut self, p1: Vec2f, p2: Vec2f, p3: Vec2f);
    fn end_path(&mut self, close: bool);
}

/* Code to append points to a Bezier path
 * Assume initial call to moveTo to initialize, followed by
 * calls to curveTo and lineTo, and finished with endPath.
 */

impl PathTarget for Ppolyline_t {
    fn move_to(&mut self, p: Vec2f) {
        self.ps = Vec::new();
        self.ps.push(p);
    }
    fn line_to(&mut self, p: Vec2f) {
        let curp = *self.ps.last().unwrap();
        self.curve_to(curp, p, p);
    }
    fn curve_to(&mut self, p1: Vec2f, p2: Vec2f, p3: Vec2f) {
        self.ps.push(p1);
        self.ps.push(p2);
        self.ps.push(p3);
    }

    fn end_path(&mut self, close: bool) {
        if close {
            let p0 = self.ps[0];
            self.line_to(p0);
        }
    }
}

use core::ops::Range;

struct F64Steps {
    start: f64,
    step: f64,
    range: Range<i32>
}

impl Iterator for F64Steps {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        let i = self.range.next()?;
        Some(self.start + (i as f64) * self.step)
    }
}

impl F64Steps {
    pub fn from_range(start: f64, end: f64, steps: i32) -> Self {
        assert!(steps >= 1);
        let step = (end - start) / (steps as f64);
        F64Steps {
            start,
            step,
            range: 0..steps
        }
    }
}

/// Approximate an elliptical arc via Beziers of given degree
/// threshold indicates quality of approximation
/// if isSlice is true, the path begins and ends with line segments
/// to the center of the ellipse.
/// Returned path must be freed by the caller.
fn gen_elliptic_path<P: PathTarget>(ep: &mut Ellipse, degree: i32, threshold: f64, isSlice: bool, path: &mut P) {
    // find the number of Bezier curves needed
    let mut found = false;
    let mut n = 1;
    while !found && (n < 1024) {
        let dEta = (ep.eta2 - ep.eta1) / n as f64;
        if dEta <= 0.5 * PI {
            let mut etaB = ep.eta1;
            found = true;
            for _ in 0..n {
                let etaA = etaB;
                etaB += dEta;
                found = estimateError(ep, degree, etaA, etaB) <= threshold;
                if !found {
                    break;
                }
            }
        }
        n = n << 1;
    }

    let mut etaB = ep.eta1;
    let dEta = (ep.eta2 - ep.eta1) / n as f64;
    let mut xB;
    let mut yB;
    let mut xBDot;
    let mut yBDot;
    let alpha;
    {
        let cosEtaB = etaB.cos();
        let sinEtaB = etaB.sin();
        let aCosEtaB = ep.a * cosEtaB;
        let bSinEtaB = ep.b * sinEtaB;
        let aSinEtaB = ep.a * sinEtaB;
        let bCosEtaB = ep.b * cosEtaB;
        xB = ep.c.x + aCosEtaB * ep.cosTheta - bSinEtaB * ep.sinTheta;
        yB = ep.c.y + aCosEtaB * ep.sinTheta + bSinEtaB * ep.cosTheta;
        xBDot = -aSinEtaB * ep.cosTheta - bCosEtaB * ep.sinTheta;
        yBDot = -aSinEtaB * ep.sinTheta + bCosEtaB * ep.cosTheta;

        if isSlice {
            path.move_to(ep.c);
            path.line_to(point(xB, yB));
        } else {
            path.move_to(point(xB, yB));
        }

        let t = (0.5 * dEta).tan();
        alpha = dEta.sin() * ((4.0 + 3.0 * t * t).sqrt() - 1.0) / 3.0;
    }

    for _ in 0..n {
        let xA = xB;
        let yA = yB;
        let xADot = xBDot;
        let yADot = yBDot;

        etaB += dEta;
        let cosEtaB = etaB.cos();
        let sinEtaB = etaB.sin();
        let aCosEtaB = ep.a * cosEtaB;
        let bSinEtaB = ep.b * sinEtaB;
        let aSinEtaB = ep.a * sinEtaB;
        let bCosEtaB = ep.b * cosEtaB;
        xB = ep.c.x + aCosEtaB * ep.cosTheta - bSinEtaB * ep.sinTheta;
        yB = ep.c.y + aCosEtaB * ep.sinTheta + bSinEtaB * ep.cosTheta;
        xBDot = -aSinEtaB * ep.cosTheta - bCosEtaB * ep.sinTheta;
        yBDot = -aSinEtaB * ep.sinTheta + bCosEtaB * ep.cosTheta;

        match degree {
            1 => {
                path.line_to(point(xB, yB));
            }
            #[cfg(DO_QUAD)]
            2 => {
                let k = (yBDot * (xB - xA) - xBDot * (yB - yA)) / (xADot * yBDot - yADot * xBDot);
                quadTo(path, xA + k * xADot, yA + k * yADot, xB, yB);
            }
            _ => {
                path.curve_to(
                    point(xA + alpha * xADot, yA + alpha * yADot),
                    point(xB - alpha * xBDot, yB - alpha * yBDot),
                    point(xB, yB),
                );
            }
        }
    }

    path.end_path(isSlice);
}

/// Return a cubic Bezier for an elliptical wedge, with center ctr, x and y
/// semi-axes xsemi and ysemi, start angle angle0 and end angle angle1.
/// This includes beginning and ending line segments to the ellipse center.
pub fn ellipticWedge<P: PathTarget>(
    ctr: Ppoint_t,
    xsemi: f64,
    ysemi: f64,
    angle0: f64,
    angle1: f64,
    path: &mut P,
) {
    let mut ell = initEllipse(ctr.x, ctr.y, xsemi, ysemi, 0.0, angle0, angle1);
    gen_elliptic_path(&mut ell, 3, 0.00001, true, path);
}

#[test]
fn test_ellipse() {
    let mut ell = initEllipse(200.0, 200.0, 100.0, 50.0, 0.0, PI / 4.0, 3.0 * PI / 2.0);
    let mut path: Ppolyline_t = <Ppolyline_t as Default>::default();
    gen_elliptic_path(&mut ell, 3, 0.00001, true, &mut path);

    println!("newpath {} {} moveto", path.ps[0].x, path.ps[0].y);
    for curve in path.ps[1..].chunks(3) {
        println!(
            "{:?} {:?} {:?} curveto", curve[0], curve[1], curve[2]);
    }
    println!("stroke showpage");
}
