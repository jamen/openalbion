//! A 3D PGA implementation closely inspired from the references at https://bivector.net/tools.html
//! The difference between them is mostly trivial code style.

#![allow(non_upper_case_globals)]

use std::ops::{Index,IndexMut,Not,Add,Sub,Mul,BitXor,BitAnd,BitOr};
use std::fmt;

#[derive(Default,Debug,Clone,Copy,PartialEq)]
pub struct Mv ([f64; 16]);

//
// Basis vectors
//

const basis: &'static [&'static str] = &["1", "e0", "e1", "e2", "e3", "e01", "e02", "e03", "e12", "e31", "e23", "e021", "e013", "e032", "e123", "e0123"];

pub const e0: Mv = Mv([0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e1: Mv = Mv([0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e2: Mv = Mv([0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e3: Mv = Mv([0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e01: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e02: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e03: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e12: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e31: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e23: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
pub const e021: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
pub const e013: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
pub const e032: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
pub const e123: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
pub const e0123: Mv = Mv([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

impl Mv {
    pub fn new(f: f64, i: usize) -> Self {
        let mut result: Self = Default::default();
        result[i] = f;
        result
    }

    pub fn point(x: f64, y: f64, z: f64) -> Self {
        e123 + x * e032 + y * e013 + z * e021
    }

    pub fn line_from_points(p1: Self, p2: Self) -> Self {
        p1 & p2
    }

    pub fn reverse(&self) -> Self {
        Mv([self[0], self[1], self[2], self[3], self[4], -self[5], -self[6], -self[7], -self[8],
            -self[9], -self[10], -self[11], -self[12], -self[13], -self[14], self[15]])
    }
}

impl Index<usize> for Mv {
    type Output = f64;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Mv {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

//
// Products
//

// Geometric product: ab
impl Mul for Mv {
    type Output = Mv;
    fn mul(self, b: Self) -> Self {
        let a = self;
        Mv([
            b[0]*a[0]+b[2]*a[2]+b[3]*a[3]+b[4]*a[4]-b[8]*a[8]-b[9]*a[9]-b[10]*a[10]-b[14]*a[14],
            b[1]*a[0]+b[0]*a[1]-b[5]*a[2]-b[6]*a[3]-b[7]*a[4]+b[2]*a[5]+b[3]*a[6]+b[4]*a[7]+b[11]*a[8]+b[12]*a[9]+b[13]*a[10]+b[8]*a[11]+b[9]*a[12]+b[10]*a[13]+b[15]*a[14]-b[14]*a[15],
            b[2]*a[0]+b[0]*a[2]-b[8]*a[3]+b[9]*a[4]+b[3]*a[8]-b[4]*a[9]-b[14]*a[10]-b[10]*a[14],
            b[3]*a[0]+b[8]*a[2]+b[0]*a[3]-b[10]*a[4]-b[2]*a[8]-b[14]*a[9]+b[4]*a[10]-b[9]*a[14],
            b[4]*a[0]-b[9]*a[2]+b[10]*a[3]+b[0]*a[4]-b[14]*a[8]+b[2]*a[9]-b[3]*a[10]-b[8]*a[14],
            b[5]*a[0]+b[2]*a[1]-b[1]*a[2]-b[11]*a[3]+b[12]*a[4]+b[0]*a[5]-b[8]*a[6]+b[9]*a[7]+b[6]*a[8]-b[7]*a[9]-b[15]*a[10]-b[3]*a[11]+b[4]*a[12]+b[14]*a[13]-b[13]*a[14]-b[10]*a[15],
            b[6]*a[0]+b[3]*a[1]+b[11]*a[2]-b[1]*a[3]-b[13]*a[4]+b[8]*a[5]+b[0]*a[6]-b[10]*a[7]-b[5]*a[8]-b[15]*a[9]+b[7]*a[10]+b[2]*a[11]+b[14]*a[12]-b[4]*a[13]-b[12]*a[14]-b[9]*a[15],
            b[7]*a[0]+b[4]*a[1]-b[12]*a[2]+b[13]*a[3]-b[1]*a[4]-b[9]*a[5]+b[10]*a[6]+b[0]*a[7]-b[15]*a[8]+b[5]*a[9]-b[6]*a[10]+b[14]*a[11]-b[2]*a[12]+b[3]*a[13]-b[11]*a[14]-b[8]*a[15],
            b[8]*a[0]+b[3]*a[2]-b[2]*a[3]+b[14]*a[4]+b[0]*a[8]+b[10]*a[9]-b[9]*a[10]+b[4]*a[14],
            b[9]*a[0]-b[4]*a[2]+b[14]*a[3]+b[2]*a[4]-b[10]*a[8]+b[0]*a[9]+b[8]*a[10]+b[3]*a[14],
            b[10]*a[0]+b[14]*a[2]+b[4]*a[3]-b[3]*a[4]+b[9]*a[8]-b[8]*a[9]+b[0]*a[10]+b[2]*a[14],
            b[11]*a[0]-b[8]*a[1]+b[6]*a[2]-b[5]*a[3]+b[15]*a[4]-b[3]*a[5]+b[2]*a[6]-b[14]*a[7]-b[1]*a[8]+b[13]*a[9]-b[12]*a[10]+b[0]*a[11]+b[10]*a[12]-b[9]*a[13]+b[7]*a[14]-b[4]*a[15],
            b[12]*a[0]-b[9]*a[1]-b[7]*a[2]+b[15]*a[3]+b[5]*a[4]+b[4]*a[5]-b[14]*a[6]-b[2]*a[7]-b[13]*a[8]-b[1]*a[9]+b[11]*a[10]-b[10]*a[11]+b[0]*a[12]+b[8]*a[13]+b[6]*a[14]-b[3]*a[15],
            b[13]*a[0]-b[10]*a[1]+b[15]*a[2]+b[7]*a[3]-b[6]*a[4]-b[14]*a[5]-b[4]*a[6]+b[3]*a[7]+b[12]*a[8]-b[11]*a[9]-b[1]*a[10]+b[9]*a[11]-b[8]*a[12]+b[0]*a[13]+b[5]*a[14]-b[2]*a[15],
            b[14]*a[0]+b[10]*a[2]+b[9]*a[3]+b[8]*a[4]+b[4]*a[8]+b[3]*a[9]+b[2]*a[10]+b[0]*a[14],
            b[15]*a[0]+b[14]*a[1]+b[13]*a[2]+b[12]*a[3]+b[11]*a[4]+b[10]*a[5]+b[9]*a[6]+b[8]*a[7]+b[7]*a[8]+b[6]*a[9]+b[5]*a[10]-b[4]*a[11]-b[3]*a[12]-b[2]*a[13]-b[1]*a[14]+b[0]*a[15],
        ])
    }
}

// Outer product: a ∧ b
impl BitXor for Mv {
    type Output = Mv;
    fn bitxor(self, b: Self) -> Self {
        let a = self;
        Mv([
            b[0]*a[0],
            b[1]*a[0]+b[0]*a[1],
            b[2]*a[0]+b[0]*a[2],
            b[3]*a[0]+b[0]*a[3],
            b[4]*a[0]+b[0]*a[4],
            b[5]*a[0]+b[2]*a[1]-b[1]*a[2]+b[0]*a[5],
            b[6]*a[0]+b[3]*a[1]-b[1]*a[3]+b[0]*a[6],
            b[7]*a[0]+b[4]*a[1]-b[1]*a[4]+b[0]*a[7],
            b[8]*a[0]+b[3]*a[2]-b[2]*a[3]+b[0]*a[8],
            b[9]*a[0]-b[4]*a[2]+b[2]*a[4]+b[0]*a[9],
            b[10]*a[0]+b[4]*a[3]-b[3]*a[4]+b[0]*a[10],
            b[11]*a[0]-b[8]*a[1]+b[6]*a[2]-b[5]*a[3]-b[3]*a[5]+b[2]*a[6]-b[1]*a[8]+b[0]*a[11],
            b[12]*a[0]-b[9]*a[1]-b[7]*a[2]+b[5]*a[4]+b[4]*a[5]-b[2]*a[7]-b[1]*a[9]+b[0]*a[12],
            b[13]*a[0]-b[10]*a[1]+b[7]*a[3]-b[6]*a[4]-b[4]*a[6]+b[3]*a[7]-b[1]*a[10]+b[0]*a[13],
            b[14]*a[0]+b[10]*a[2]+b[9]*a[3]+b[8]*a[4]+b[4]*a[8]+b[3]*a[9]+b[2]*a[10]+b[0]*a[14],
            b[15]*a[0]+b[14]*a[1]+b[13]*a[2]+b[12]*a[3]+b[11]*a[4]+b[10]*a[5]+b[9]*a[6]+b[8]*a[7]+b[7]*a[8]+b[6]*a[9]+b[5]*a[10]-b[4]*a[11]-b[3]*a[12]-b[2]*a[13]-b[1]*a[14]+b[0]*a[15],
        ])
    }
}

// Regressive product: a ∨ b
impl BitAnd for Mv {
    type Output = Mv;
    fn bitand(self, b: Self) -> Self {
        let a = self;
        Mv([
            b[0]*a[15]+b[1]*a[14]+b[2]*a[13]+b[3]*a[12]+b[4]*a[11]+b[5]*a[10]+b[6]*a[9]+b[7]*a[8]+b[8]*a[7]+b[9]*a[6]+b[10]*a[5]-b[11]*a[4]-b[12]*a[3]-b[13]*a[2]-b[14]*a[1]+b[15]*a[0],
            b[1]*a[15]+b[5]*a[13]+b[6]*a[12]+b[7]*a[11]+b[11]*a[7]+b[12]*a[6]+b[13]*a[5]+b[15]*a[1],
            b[2]*a[15]-b[5]*a[14]+b[8]*a[12]-b[9]*a[11]-b[11]*a[9]+b[12]*a[8]-b[14]*a[5]+b[15]*a[2],
            b[3]*a[15]-b[6]*a[14]-b[8]*a[13]+b[10]*a[11]+b[11]*a[10]-b[13]*a[8]-b[14]*a[6]+b[15]*a[3],
            b[4]*a[15]-b[7]*a[14]+b[9]*a[13]-b[10]*a[12]-b[12]*a[10]+b[13]*a[9]-b[14]*a[7]+b[15]*a[4],
            b[5]*a[15]+b[11]*a[12]-b[12]*a[11]+b[15]*a[5],
            b[6]*a[15]-b[11]*a[13]+b[13]*a[11]+b[15]*a[6],
            b[7]*a[15]+b[12]*a[13]-b[13]*a[12]+b[15]*a[7],
            b[8]*a[15]+b[11]*a[14]-b[14]*a[11]+b[15]*a[8],
            b[9]*a[15]+b[12]*a[14]-b[14]*a[12]+b[15]*a[9],
            b[10]*a[15]+b[13]*a[14]-b[14]*a[13]+b[15]*a[10],
            b[11]*a[15]+b[15]*a[11],
            b[12]*a[15]+b[15]*a[12],
            b[13]*a[15]+b[15]*a[13],
            b[14]*a[15]+b[15]*a[14],
            b[15]*a[15],
        ])
    }
}

// Dot product: a · b
impl BitOr for Mv {
    type Output = Mv;
    fn bitor(self, b: Self) -> Self {
        let a = self;
        Mv([
            b[0]*a[0]+b[2]*a[2]+b[3]*a[3]+b[4]*a[4]-b[8]*a[8]-b[9]*a[9]-b[10]*a[10]-b[14]*a[14],
            b[1]*a[0]+b[0]*a[1]-b[5]*a[2]-b[6]*a[3]-b[7]*a[4]+b[2]*a[5]+b[3]*a[6]+b[4]*a[7]+b[11]*a[8]+b[12]*a[9]+b[13]*a[10]+b[8]*a[11]+b[9]*a[12]+b[10]*a[13]+b[15]*a[14]-b[14]*a[15],
            b[2]*a[0]+b[0]*a[2]-b[8]*a[3]+b[9]*a[4]+b[3]*a[8]-b[4]*a[9]-b[14]*a[10]-b[10]*a[14],
            b[3]*a[0]+b[8]*a[2]+b[0]*a[3]-b[10]*a[4]-b[2]*a[8]-b[14]*a[9]+b[4]*a[10]-b[9]*a[14],
            b[4]*a[0]-b[9]*a[2]+b[10]*a[3]+b[0]*a[4]-b[14]*a[8]+b[2]*a[9]-b[3]*a[10]-b[8]*a[14],
            b[5]*a[0]-b[11]*a[3]+b[12]*a[4]+b[0]*a[5]-b[15]*a[10]-b[3]*a[11]+b[4]*a[12]-b[10]*a[15],
            b[6]*a[0]+b[11]*a[2]-b[13]*a[4]+b[0]*a[6]-b[15]*a[9]+b[2]*a[11]-b[4]*a[13]-b[9]*a[15],
            b[7]*a[0]-b[12]*a[2]+b[13]*a[3]+b[0]*a[7]-b[15]*a[8]-b[2]*a[12]+b[3]*a[13]-b[8]*a[15],
            b[8]*a[0]+b[14]*a[4]+b[0]*a[8]+b[4]*a[14],
            b[9]*a[0]+b[14]*a[3]+b[0]*a[9]+b[3]*a[14],
            b[10]*a[0]+b[14]*a[2]+b[0]*a[10]+b[2]*a[14],
            b[11]*a[0]+b[15]*a[4]+b[0]*a[11]-b[4]*a[15],
            b[12]*a[0]+b[15]*a[3]+b[0]*a[12]-b[3]*a[15],
            b[13]*a[0]+b[15]*a[2]+b[0]*a[13]-b[2]*a[15],
            b[14]*a[0]+b[0]*a[14],
            b[15]*a[0]+b[0]*a[15],
        ])
    }
}

//
// Vectors ops
//

macro_rules! multivector_op {
    (($a:expr) $tt:tt ($b:expr)) => {
        Mv([$a[0] $tt $b[0], $a[1] $tt $b[1], $a[2] $tt $b[2], $a[3] $tt $b[3], $a[4] $tt $b[4], $a[5] $tt $b[5],
            $a[6] $tt $b[6], $a[7] $tt $b[7], $a[8] $tt $b[8], $a[9] $tt $b[9], $a[10] $tt $b[10], $a[11] $tt $b[11],
            $a[12] $tt $b[12], $a[13] $tt $b[13], $a[14] $tt $b[14], $a[15] $tt $b[15] ])
    }
}

impl Add<Mv> for Mv {
    type Output = Mv;
    fn add(self, rhs: Self) -> Self {
        multivector_op!((self) + (rhs))
    }
}

impl Sub for Mv {
    type Output = Mv;
    fn sub(self, rhs: Self) -> Self {
        multivector_op!((self) - (rhs))
    }
}


impl Not for Mv {
    type Output = Mv;
    fn not(self) -> Self::Output {
        self.reverse()
    }
}

//
// Scalar ops
//

macro_rules! scalar_op {
    (($a:expr) $tt:tt ($b:expr)) => {
        Mv([$a[0] $tt $b, $a[1], $a[2], $a[3], $a[4], $a[5], $a[6], $a[7], $a[8], $a[9], $a[10], $a[11], $a[12],
            $a[13], $a[14], $a[15] ])
    }
}

macro_rules! scalar_op_all {
    (($a:expr) $tt:tt ($b:expr)) => {
        Mv([$a[0] $tt $b, $a[1] $tt $b, $a[2] $tt $b, $a[3] $tt $b, $a[4] $tt $b, $a[5] $tt $b, $a[6] $tt $b,
            $a[7] $tt $b, $a[8] $tt $b, $a[9] $tt $b, $a[10] $tt $b, $a[11] $tt $b, $a[12] $tt $b, $a[13] $tt $b,
            $a[14] $tt $b, $a[15] $tt $b ])
    }
}

macro_rules! scalar_inverse_op {
    (($a:expr) $tt:tt ($b:expr)) => {
        Mv([$a $tt $b[0], $b[1], $b[2], $b[3], $b[4], $b[5], $b[6], $b[7], $b[8], $b[9], $b[10], $b[11], $b[12],
            $b[13], $b[14], $b[15] ])
    }
}

macro_rules! scalar_inverse_op_all {
    (($a:expr) $tt:tt ($b:expr)) => {
        Mv([$a $tt $b[0], $a $tt $b[1], $a $tt $b[2], $a $tt $b[3], $a $tt $b[4], $a $tt $b[5], $a $tt $b[6],
            $a $tt $b[7], $a $tt $b[8], $a $tt $b[9], $a $tt $b[10], $a $tt $b[11], $a $tt $b[12], $a $tt $b[13],
            $a $tt $b[14], $a $tt $b[15] ])
    }
}

impl Add<f64> for Mv {
    type Output = Mv;
    fn add(self, rhs: f64) -> Self { scalar_op!((self) + (rhs)) }
}

impl Add<Mv> for f64 {
    type Output = Mv;
    fn add(self, rhs: Mv) -> Mv { scalar_inverse_op!((self) + (rhs)) }
}

impl Sub<f64> for Mv {
    type Output = Mv;
    fn sub(self, rhs: f64) -> Self { scalar_op!((self) - (rhs)) }
}

impl Sub<Mv> for f64 {
    type Output = Mv;
    fn sub(self, rhs: Mv) -> Mv { scalar_inverse_op!((self) - (rhs)) }
}

impl Mul<f64> for Mv {
    type Output = Mv;
    fn mul(self, rhs: f64) -> Self { scalar_op_all!((self) * (rhs)) }
}

impl Mul<Mv> for f64 {
    type Output = Mv;
    fn mul(self, rhs: Mv) -> Mv { scalar_inverse_op_all!((self) * (rhs)) }
}

//
// Display
//

// impl fmt::Display for Mv {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if self[0] != 0.0 { f.write_fmt(format_args!("{}", self[0]))? }
//         if self[1] != 0.0 { f.write_fmt(format_args!("{}e{}", self[1], basis[1]))? }
//         Ok(())
//     }
// }