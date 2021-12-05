use core::{mem, slice};

macro_rules! impl_num_parse {
    ($($fn_name:ident, $typ:tt::$conv:tt,)*) => {
        $(
            fn $fn_name (&mut self) -> Option<$typ> {
                let bytes = self.advance(mem::size_of::<$typ>())?;
                Some($typ::$conv(unsafe { *(bytes.as_ptr() as *const [u8; mem::size_of::<$typ>()]) }))
            }
        )*
    }
}

macro_rules! impl_mint_parse {
    ($($fn_name:ident, $typ:ty, [ $($c:expr),* ],)*) => {
        $(
            fn $fn_name (&mut self) -> Option<$typ> {
                Some([ $($c(self)?),* ].into())
            }
        )*
    }
}

pub(crate) trait Bytes: AsRef<[u8]> {
    fn advance(&mut self, n: usize) -> Option<&[u8]>;

    impl_num_parse! {
        parse_u16_le, u16::from_le_bytes,
        parse_u16_be, u16::from_be_bytes,
        parse_u16_ne, u16::from_ne_bytes,
        parse_i16_le, i16::from_le_bytes,
        parse_i16_be, i16::from_be_bytes,
        parse_i16_ne, i16::from_ne_bytes,
        parse_u32_le, u32::from_le_bytes,
        parse_u32_be, u32::from_be_bytes,
        parse_u32_ne, u32::from_ne_bytes,
        parse_i32_le, i32::from_le_bytes,
        parse_i32_be, i32::from_be_bytes,
        parse_i32_ne, i32::from_ne_bytes,
        parse_u64_le, u64::from_le_bytes,
        parse_u64_be, u64::from_be_bytes,
        parse_u64_ne, u64::from_ne_bytes,
        parse_i64_le, i64::from_le_bytes,
        parse_i64_be, i64::from_be_bytes,
        parse_i64_ne, i64::from_ne_bytes,
        parse_u128_le, u128::from_le_bytes,
        parse_u128_be, u128::from_be_bytes,
        parse_u128_ne, u128::from_ne_bytes,
        parse_i128_le, i128::from_le_bytes,
        parse_i128_be, i128::from_be_bytes,
        parse_i128_ne, i128::from_ne_bytes,
        parse_f32_le, f32::from_le_bytes,
        parse_f32_be, f32::from_be_bytes,
        parse_f32_ne, f32::from_ne_bytes,
        parse_f64_le, f64::from_le_bytes,
        parse_f64_be, f64::from_be_bytes,
        parse_f64_ne, f64::from_ne_bytes,
    }

    impl_mint_parse! {
        parse_vector2_f16_le, [F16; 2], [Self::parse_f16_le, Self::parse_f16_le],
        parse_vector2_f32_le, [f32; 2], [Self::parse_f32_le, Self::parse_f32_le],
        parse_vector3_f32_le, [f32; 3], [Self::parse_f32_le, Self::parse_f32_le, Self::parse_f32_le],
        parse_vector4_f16_le, [F16; 4], [
            Self::parse_f16_le,
            Self::parse_f16_le,
            Self::parse_f16_le,
            Self::parse_f16_le
        ],
        parse_vector4_f32_le, [f32; 4], [
            Self::parse_f32_le,
            Self::parse_f32_le,
            Self::parse_f32_le,
            Self::parse_f32_le
        ],
        parse_quaternion_f32_le, mint::Quaternion<f32>, [
            Self::parse_f32_le,
            Self::parse_f32_le,
            Self::parse_f32_le,
            Self::parse_f32_le
        ],
        parse_colmatrix3x4_f32_le, mint::ColumnMatrix3x4<f32>, [
            Self::parse_vector3_f32_le,
            Self::parse_vector3_f32_le,
            Self::parse_vector3_f32_le,
            Self::parse_vector3_f32_le
        ],
        parse_colmatrix4_f32_le, mint::ColumnMatrix4<f32>, [
            Self::parse_vector4_f32_le,
            Self::parse_vector4_f32_le,
            Self::parse_vector4_f32_le,
            Self::parse_vector4_f32_le
        ],
    }

    fn parse_u8(&mut self) -> Option<u8> {
        Some(self.advance(1)?[0])
    }
    fn parse_i8(&mut self) -> Option<i8> {
        Some(self.parse_u8()? as i8)
    }

    fn parse_f16_le(&mut self) -> Option<F16> {
        Some(F16::new_unchecked(self.parse_u16_le()?))
    }
    fn parse_f16_be(&mut self) -> Option<F16> {
        Some(F16::new_unchecked(self.parse_u16_be()?))
    }
    fn parse_f16_ne(&mut self) -> Option<F16> {
        Some(F16::new_unchecked(self.parse_u16_ne()?))
    }

    fn parse_vector3_packed(&mut self) -> Option<[f32; 3]> {
        Some(Vector3Packed::new_unchecked(self.parse_u32_le()?).into())
    }

    fn parse_str(&mut self, n: usize) -> Option<&str> {
        core::str::from_utf8(self.advance(n)?).ok()
    }

    fn parse_while(&mut self, pred: impl Fn(&&u8) -> bool) -> Option<&[u8]> {
        let len = self.as_ref().iter().take_while(pred).count();
        let out = self.advance(len + 1)?;
        let out = &out[..len];
        Some(out)
    }

    fn parse_until_nul(&mut self) -> Option<&[u8]> {
        self.parse_while(|x| **x != b'\0')
    }

    fn parse_str_until_nul(&mut self) -> Option<&str> {
        let out = self.parse_until_nul()?;
        let out = core::str::from_utf8(out).ok()?;
        Some(out)
    }

    fn parse_with_u32_le_prefix(&mut self) -> Option<&[u8]> {
        let prefix = self.parse_u32_le()?;
        let out = self.advance(prefix as usize)?;
        Some(out)
    }

    fn parse_str_with_u32_le_prefix(&mut self) -> Option<&str> {
        let out = self.parse_with_u32_le_prefix()?;
        let out = core::str::from_utf8(out).ok()?;
        Some(out)
    }
}

impl Bytes for &[u8] {
    fn advance(&mut self, n: usize) -> Option<&[u8]> {
        let len = self.len();
        let ptr = (*self).as_ptr();

        if n > len {
            None
        } else {
            unsafe {
                let out = slice::from_raw_parts(ptr, n);
                *self = slice::from_raw_parts(ptr.add(n), len - n);
                Some(out)
            }
        }
    }
}

impl Bytes for &mut [u8] {
    fn advance(&mut self, n: usize) -> Option<&[u8]> {
        let len = self.len();
        let ptr = (*self).as_mut_ptr();

        if n > len {
            None
        } else {
            unsafe {
                let out = slice::from_raw_parts_mut(ptr, n);
                *self = slice::from_raw_parts_mut(ptr.add(n), len - n);
                Some(out)
            }
        }
    }
}

macro_rules! impl_num_put {
    ($($fn_name:ident, $typ:tt::$conv:tt,)*) => {
        $(
            fn $fn_name(&mut self, val: $typ) {
                self.put(&$typ::$conv(val))
            }
        )*
    }
}

// macro_rules! impl_mint_put {
//     ($($fn_name:ident, $typ:ty, [ $($c:expr),* ],)*) => {
//         $(
//             fn $fn_name (&mut self, x: $typ) {
//                 let mut i = x.into();
//                 $($c(self, i.next());)*
//             }
//         )*
//     }
// }

pub(crate) trait BytesMut: AsRef<[u8]> + AsMut<[u8]> {
    fn put(&mut self, val: &[u8]);

    fn put_u8(&mut self, val: u8) {
        self.put(&[val])
    }

    fn put_i8(&mut self, val: i8) {
        self.put(&[val as u8])
    }

    impl_num_put! {
        put_u16_be, u16::to_be_bytes,
        put_u16_le, u16::to_le_bytes,
        put_u16_ne, u16::to_ne_bytes,
        put_i16_le, i16::to_le_bytes,
        put_i16_be, i16::to_be_bytes,
        put_i16_ne, i16::to_ne_bytes,
        put_u32_le, u32::to_le_bytes,
        put_u32_be, u32::to_be_bytes,
        put_u32_ne, u32::to_ne_bytes,
        put_i32_le, i32::to_le_bytes,
        put_i32_be, i32::to_be_bytes,
        put_i32_ne, i32::to_ne_bytes,
        put_u64_le, u64::to_le_bytes,
        put_u64_be, u64::to_be_bytes,
        put_u64_ne, u64::to_ne_bytes,
        put_i64_le, i64::to_le_bytes,
        put_i64_be, i64::to_be_bytes,
        put_i64_ne, i64::to_ne_bytes,
        put_u128_le, u128::to_le_bytes,
        put_u128_be, u128::to_be_bytes,
        put_u128_ne, u128::to_ne_bytes,
        put_i128_le, i128::to_le_bytes,
        put_i128_be, i128::to_be_bytes,
        put_i128_ne, i128::to_ne_bytes,
        put_f32_le, f32::to_le_bytes,
        put_f32_be, f32::to_be_bytes,
        put_f32_ne, f32::to_ne_bytes,
        put_f64_le, f64::to_le_bytes,
        put_f64_be, f64::to_be_bytes,
        put_f64_ne, f64::to_ne_bytes,
    }

    // impl_mint_put! {
    //     put_vector2_f32_le, [f32; 2], [Self::put_f32_le, Self::put_f32_le],
    //     put_vector3_f32_le, [f32; 3], [Self::put_f32_le, Self::put_f32_le, Self::put_f32_le],
    //     put_vector4_f32_le, [f32; 4], [
    //         Self::put_f32_le,
    //         Self::put_f32_le,
    //         Self::put_f32_le,
    //         Self::put_f32_le
    //     ],
    //     put_quaternion_f32_le, mint::Quaternion<f32>, [
    //         Self::put_f32_le,
    //         Self::put_f32_le,
    //         Self::put_f32_le,
    //         Self::put_f32_le
    //     ],
    //     put_colmatrix3x4_f32_le, mint::ColumnMatrix3x4<f32>, [
    //         Self::put_vector3_f32_le,
    //         Self::put_vector3_f32_le,
    //         Self::put_vector3_f32_le,
    //         Self::put_vector3_f32_le
    //     ],
    //     put_colmatrix4_f32_le, mint::ColumnMatrix4<f32>, [
    //         Self::put_vector4_f32_le,
    //         Self::put_vector4_f32_le,
    //         Self::put_vector4_f32_le,
    //         Self::put_vector4_f32_le
    //     ],
    // }
}

impl BytesMut for &mut [u8] {
    fn put(&mut self, val: &[u8]) {
        let n = val.len();
        let len = self.len();
        let ptr = (*self).as_mut_ptr();

        if len >= n {
            unsafe {
                let write = slice::from_raw_parts_mut(ptr, n);
                write.copy_from_slice(val);
                *self = slice::from_raw_parts_mut(ptr.add(n), len - n);
            }
        }
    }
}

/// A 3d vector where X and Y are 11-bit floats and Z is a 10-bit float.
#[derive(Copy, Clone)]
pub struct Vector3Packed(u32);

impl Vector3Packed {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3Packed(
            ((y * 1023.0) as u32 & 0x7ff | ((z * 511.0) as u32) << 11) << 11
                | (x * 1023.0) as u32 & 0x7ff,
        )
    }
    pub fn new_unchecked(x: u32) -> Self {
        Vector3Packed(x)
    }
}

impl From<[f32; 3]> for Vector3Packed {
    fn from(x: [f32; 3]) -> Vector3Packed {
        Vector3Packed::new(x[0], x[1], x[2])
    }
}

impl From<Vector3Packed> for [f32; 3] {
    fn from(v: Vector3Packed) -> [f32; 3] {
        // Khronos data format spec for 11/10bit floats:
        // https://www.khronos.org/registry/DataFormat/specs/1.2/dataformat.1.2.html#11bitfp

        // From Talchas#7429 on Discord:
        // if raw_exponent == 0 {
        //     special cases for 0 and denorms
        // } else {
        //     new_exponent = raw_exponent + 2**new-2**old; new_mantissa = old_mantissa <<
        // (new_m_bits - old_m_bits) }

        // let xe = (v.0 >> 6) & 0b11111;
        // let xm = (v.0 >> 0) & 0b111111;

        // let ye = (v.0 >> 17) & 0b11111;
        // let ym = (v.0 >> 11) & 0b111111;

        // let ze = (v.0 >> 27) & 0b11111;
        // let zm = (v.0 >> 22) & 0b11111;

        // let x = match (xe, xm) {
        //     (0, 0) => 0.0f32,
        //     (31, 0) => f32::INFINITY,
        //     (0, x) if x != 0 => {} // denorm?
        //     (31, x) if x != 0 => f32::from_bits(f32::NAN.to_bits() & (x << 17)),
        //     (x, m) => {}
        // };

        // Attempt 4
        //

        let xe = (v.0 >> 6) & 0b11111;
        let xm = (v.0 >> 0) & 0b111111;

        let ye = (v.0 >> 17) & 0b11111;
        let ym = (v.0 >> 11) & 0b111111;

        let ze = (v.0 >> 27) & 0b11111;
        let zm = (v.0 >> 22) & 0b11111;

        let mut exponent;
        let mut mantissa = xm;

        if xe == 0x1f {
            exponent = 0x7f800000 | (xm << 17);
        } else if xm != 0 {
            exponent = 1;
            loop {
                exponent = exponent.wrapping_sub(1);
                mantissa <<= 1;
                if (mantissa & 0x40) == 0 {
                    break;
                }
            }
            mantissa &= 0x1f;
            // let l = xm.leading_zeros();
            // exponent = 1_u32.wrapping_sub(l);
            // mantissa = (xm << l) & 0x3f;
        } else {
            exponent = 112_u32.wrapping_neg();
        }

        let x = f32::from_bits(((exponent.wrapping_add(112)) << 23) | (mantissa << 17));

        mantissa = ym;

        if ye == 0x1f {
            exponent = 0x7f800000 | (ym << 17);
        } else if ym != 0 {
            exponent = 1;
            loop {
                exponent = exponent.wrapping_sub(1);
                mantissa <<= 1;
                if (mantissa & 0x40) == 0 {
                    break;
                }
            }
            mantissa &= 0x1f;
            // let l = ym.leading_zeros();
            // exponent = 1_u32.wrapping_sub(l);
            // mantissa = (ym << l) & 0x3f;
        } else {
            exponent = 112_u32.wrapping_neg();
        }

        let y = f32::from_bits(((exponent.wrapping_add(112)) << 23) | (mantissa << 17));

        mantissa = zm;

        if ze == 0x1f {
            exponent = 0x7f800000 | (zm << 17);
        } else if zm != 0 {
            exponent = 1;
            loop {
                exponent = exponent.wrapping_sub(1);
                mantissa <<= 1;
                if (mantissa & 0x20) == 0 {
                    break;
                }
            }
            mantissa &= 0x1f;
            // let l = zm.leading_zeros();
            // exponent = 1_u32.wrapping_sub(l);
            // mantissa = (zm << l) & 0x1f;
        } else {
            exponent = 112_u32.wrapping_neg();
        }

        let z = f32::from_bits(((exponent.wrapping_add(112)) << 23) | (mantissa << 18));

        [x, y, z]

        // Attempt 3
        //

        // let x = f32::from_bits(
        //     (((v.0 & 0b00000000000000000000010000000000) != 0) as u32).wrapping_neg()
        //         & 0b11111111111111111111100000000000
        //         | v.0 & 0b00000000000000000000011111111111,
        // ) * 0.00097752;

        // let y = f32::from_bits(
        //     ((((v.0 >> 11) & 0b00000000000000000000010000000000) != 0) as u32).wrapping_neg()
        //         & 0b11111111111111111111100000000000
        //         | (v.0 >> 11) & 0b00000000000000000000011111111111,
        // ) * 0.00097752;

        // let z = f32::from_bits(
        //     (((v.0 >> 22 & 0b00000000000000000000001000000000) != 0) as u32).wrapping_neg()
        //         & 0b11111111111111111111100000000000
        //         | v.0 >> 22 & 0b00000000000000000000011111111111,
        // ) * 0.00195695;

        // println!(
        //     "1. {:0>32b} {:0>32b} {:0>32b}\n\
        //      2. {} {} {}\n",
        //     x.to_bits(),
        //     y.to_bits(),
        //     z.to_bits(),
        //     x,
        //     y,
        //     z
        // );

        // [x, y, z]

        // Attempt 2
        //

        // println!("{:0>8x}", v.0);

        // let ze = (v.0 & 0b11111000000000000000000000000000) >> 27;
        // let zm = (v.0 & 0b00000111110000000000000000000000) >> 22;

        // let ye = (v.0 & 0b00000000001111100000000000000000) >> 17;
        // let ym = (v.0 & 0b00000000000000011111100000000000) >> 11;

        // let xe = (v.0 & 0b00000000000000000000011111000000) >> 6;
        // let xm = v.0 & 0b00000000000000000000000000111111;

        // let z = if ze == 0x1f {
        //     println!("On this?");
        //     f32::from_bits(0x7f800000 | (zm << 18))
        // } else {
        //     let mut m = zm;
        //     let e = if ze != 0 {
        //         ze
        //     } else if zm != 0 {
        //         let mut e = 1u32;
        //         loop {
        //             e = e.wrapping_sub(1);
        //             m <<= 1;
        //             if (m & 40) == 0 {
        //                 break e;
        //             }
        //         }
        //     } else {
        //         112_u32.wrapping_neg()
        //     };
        //     f32::from_bits((e.wrapping_add(112) << 23) | (m << 18))
        // };

        // let y = if ye == 0x1f {
        //     println!("On this?");
        //     f32::from_bits(0x7f800000 | (ym << 17))
        // } else {
        //     let mut m = ym;
        //     let e = if ye != 0 {
        //         ye
        //     } else if ym != 0 {
        //         let mut e = 1u32;
        //         loop {
        //             e = e.wrapping_sub(1);
        //             m <<= 1;
        //             if (m & 40) == 0 {
        //                 break e;
        //             }
        //         }
        //     } else {
        //         112_u32.wrapping_neg()
        //     };
        //     f32::from_bits((e.wrapping_add(112) << 23) | (m << 17))
        // };

        // let x = if xe == 0x1f {
        //     println!("On this?");
        //     f32::from_bits(0x7f800000 | (xm << 17))
        // } else {
        //     let mut m = xm;
        //     let e = if xe != 0 {
        //         xe
        //     } else if xm != 0 {
        //         let mut e = 1u32;
        //         loop {
        //             e = e.wrapping_sub(1);
        //             m <<= 1;
        //             if (m & 40) == 0 {
        //                 break e;
        //             }
        //         }
        //     } else {
        //         112_u32.wrapping_neg()
        //     };
        //     f32::from_bits((e.wrapping_add(112) << 23) | (m << 17))
        // };

        // let z = f32::from_bits(
        //     ((((v.0 & 0b11111000000000000000000000000000) >> 27) + 112) << 23)
        //         | ((v.0 & 0b00000111110000000000000000000000) >> 22),
        // );

        // let x = f32::from_bits(
        //     ((((v.0 & 0b00000000001111100000000000000000) >> 17) + 112) << 23)
        //         | ((v.0 & 0b00000000000000011111100000000000) >> 11),
        // );

        // let y = f32::from_bits(
        //     ((((v.0 & 0b00000000000000000000011111000000) << 6) + 112) << 23)
        //         | (v.0 & 0b00000000000000000000000000111111),
        // );

        // println!(
        //     "1. {:0>32b} {:0>32b} {:0>32b}\n\
        //      2. {} {} {}\n",
        //     x.to_bits(),
        //     y.to_bits(),
        //     z.to_bits(),
        //     x,
        //     y,
        //     z
        // );

        // [x, y, z]

        // Attempt 1
        //

        // let a1 = v.0 & 0b11111111111;
        // let a2 = v.0 >> 11 & 0b11111111111;
        // let a3 = v.0 >> 22;

        // let b1 = v.0 & 0b10000000000;
        // let b2 = v.0 >> 11 & 0b10000000000;
        // let b3 = v.0 >> 22 & 0b1000000000;

        // let c1 = ((b1 != 0) as u32).wrapping_neg() & 0b11111111111111111111100000000000;
        // let c2 = ((b2 != 0) as u32).wrapping_neg() & 0b11111111111111111111100000000000;
        // let c3 = ((b3 != 0) as u32).wrapping_neg() & 0b11111111111111111111110000000000;

        // let d1 = c1 | a1;
        // let d2 = c2 | a2;
        // let d3 = c3 | a3;

        // let e1 = d1 as f32 * 0.00097752;
        // let e2 = d2 as f32 * 0.00097752;
        // let e3 = d3 as f32 * 0.00195695;

        // println!(
        //     "1. {:0>11b} {:0>11b} {:0>10b}\n\
        //      2. {:0>11b} {:0>11b} {:0>10b}\n\
        //      3. {:0>32b} {:0>32b} {:0>32b}\n\
        //      4. {:0>32b} {:0>32b} {:0>32b}\n\
        //      5. {:0>32b} {:0>32b} {:0>32b}\n\
        //      6. {} {} {}\n",
        //     a1,
        //     a2,
        //     a3,
        //     b1,
        //     b2,
        //     b3,
        //     c1,
        //     c2,
        //     c3,
        //     d1,
        //     d2,
        //     d3,
        //     e1.to_bits(),
        //     e2.to_bits(),
        //     e3.to_bits(),
        //     e1,
        //     e2,
        //     e3,
        // );

        // [e1, e2, e3]
    }
}

/// A 16-bit float but the only thing you can do with it is make a 32-bit float
#[derive(Copy, Clone)]
pub struct F16(u16);

impl F16 {
    pub fn new(x: f32) -> Self {
        F16(((x + 8.0).round() * 2048.0) as u16)
    }
    pub fn new_unchecked(x: u16) -> Self {
        F16(x)
    }
}

impl From<f32> for F16 {
    fn from(x: f32) -> F16 {
        F16::new(x)
    }
}

impl From<F16> for f32 {
    fn from(x: F16) -> f32 {
        x.0 as f32 * 0.00048828 - 8.0
    }
}

pub struct F32Inspect(f32);

use core::fmt;

impl fmt::Debug for F32Inspect {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = self.0.to_bits();
        write!(
            fmt,
            "{:b}_{:0>8b}_{:0>23b}",
            x >> 31,
            (x >> 23) & 0xff,
            (x) & 0x7fffff
        )
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_f16_round_trip() {}
}
