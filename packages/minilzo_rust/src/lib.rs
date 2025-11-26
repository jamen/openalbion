// These minilzo bindings are a modified version of [minilzo-rs](https://github.com/badboy/minilzo-rs).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::error::Error;
use core::{fmt, mem::MaybeUninit, ptr};

use libc::{c_int, c_uchar, c_ulong, c_void};

unsafe extern "C" {
    fn lzo1x_1_compress(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: *mut c_ulong,
        wrkmem: *mut c_void,
    ) -> c_int;
    fn lzo1x_decompress_safe(
        src: *const c_uchar,
        src_len: c_ulong,
        dst: *mut c_uchar,
        dst_len: *mut c_ulong,
        wrkmem: *mut c_void,
    ) -> c_int;
}

const LZO1X_1_MEM_COMPRESS: usize = 16384 * 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LzoError {
    Error = -1,
    OutOfMemory = -2,
    NotCompressible = -3,
    InputOverrun = -4,
    OutputOverrun = -5,
    LookbehindOverrun = -6,
    EOFNotFound = -7,
    InputNotConsumed = -8,
    NotYetImplemented = -9,
    InvalidArgument = -10,
    InvalidAlignment = -11,
    OutputNotConsumed = -12,
    InternalError = -99,
    Unknown,
}

impl LzoError {
    pub fn from_code(code: i32) -> LzoError {
        match code {
            -1 => LzoError::Error,
            -2 => LzoError::OutOfMemory,
            -3 => LzoError::NotCompressible,
            -4 => LzoError::InputOverrun,
            -5 => LzoError::OutputOverrun,
            -6 => LzoError::LookbehindOverrun,
            -7 => LzoError::EOFNotFound,
            -8 => LzoError::InputNotConsumed,
            -9 => LzoError::NotYetImplemented,
            -10 => LzoError::InvalidArgument,
            -11 => LzoError::InvalidAlignment,
            -12 => LzoError::OutputNotConsumed,
            -99 => LzoError::InternalError,
            _ => LzoError::Unknown,
        }
    }
}

impl fmt::Display for LzoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Error => "error",
            Self::OutOfMemory => "out of memory",
            Self::NotCompressible => "not compressible",
            Self::InputOverrun => "input overrun",
            Self::OutputOverrun => "output overrun",
            Self::LookbehindOverrun => "lookbehind overrun",
            Self::EOFNotFound => "eof not found",
            Self::InputNotConsumed => "input not consumed",
            Self::NotYetImplemented => "not yet implemented",
            Self::InvalidArgument => "invalid argument",
            Self::InvalidAlignment => "invalid alignment",
            Self::OutputNotConsumed => "output not consumed",
            Self::InternalError => "internal error",
            Self::Unknown => "unknown error",
        })
    }
}

impl Error for LzoError {}

pub fn compress(indata: &[u8]) -> Result<Vec<u8>, LzoError> {
    let inlen = indata.len() as c_ulong;
    let mut outlen = inlen + inlen / 16 + 64 + 3;
    let mut outdata = Vec::with_capacity(outlen as usize);
    let mut wrkmem: MaybeUninit<[u8; LZO1X_1_MEM_COMPRESS]> = MaybeUninit::uninit();

    let error_code = unsafe {
        lzo1x_1_compress(
            indata.as_ptr(),
            inlen,
            outdata.as_mut_ptr(),
            &mut outlen as *mut _,
            wrkmem.as_mut_ptr() as *mut _,
        )
    };

    if error_code != 0 {
        return Err(LzoError::from_code(error_code));
    }

    outdata.truncate(outlen as usize);

    Ok(outdata)
}

pub fn decompress(indata: &[u8], outlen: usize) -> Result<Vec<u8>, LzoError> {
    let inlen = indata.len() as c_ulong;
    let mut outdata = Vec::with_capacity(outlen);
    let mut outlen = outlen as c_ulong;

    let error_code = unsafe {
        lzo1x_decompress_safe(
            indata.as_ptr(),
            inlen,
            outdata.as_mut_ptr(),
            &mut outlen as *mut _,
            ptr::null_mut(),
        )
    };

    if error_code != 0 {
        return Err(LzoError::from_code(error_code));
    }

    outdata.truncate(outlen as usize);

    Ok(outdata)
}
