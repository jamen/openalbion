// Derived from minilzo-rs (https://github.com/badboy/minilzo-rs) which has the license:
//
// The MIT License (MIT)
//
// Copyright (c) 2016 Jan-Erik Rediger
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::os::raw::{c_uchar,c_ulong,c_void,c_int};
use std::ptr;
use std::mem::MaybeUninit;

extern "C" {
    pub fn lzo1x_1_compress(
        src: *const c_uchar, src_len: c_ulong,
        dst: *mut c_uchar, dst_len: *mut c_ulong,
        wrkmem: *mut c_void
    ) -> c_int;
    pub fn lzo1x_decompress_safe(
        src: *const c_uchar, src_len: c_ulong,
        dst: *mut c_uchar, dst_len: *mut c_ulong,
        wrkmem: *mut c_void
    ) -> c_int;
}

pub const LZO1X_1_MEM_COMPRESS: usize = 16384 * 8;

#[derive(Debug, PartialEq)]
pub enum Error {
    Error,
    OutOfMemory,
    NotCompressible,
    InputOverrun,
    OutputOverrun,
    LookbehindOverrun,
    EOFNotFound,
    InputNotConsumed,
    NotYetImplemented,
    InvalidArgument,
    InvalidAlignment,
    OutputNotConsumed,
    InternalError,
}

impl Error {
    pub fn from_code(code: i32) -> Error {
        match code {
             -1 => Error::Error,
             -2 => Error::OutOfMemory,
             -3 => Error::NotCompressible,
             -4 => Error::InputOverrun,
             -5 => Error::OutputOverrun,
             -6 => Error::LookbehindOverrun,
             -7 => Error::EOFNotFound,
             -8 => Error::InputNotConsumed,
             -9 => Error::NotYetImplemented,
            -10 => Error::InvalidArgument,
            -11 => Error::InvalidAlignment,
            -12 => Error::OutputNotConsumed,
            -99 => Error::InternalError,
            _ => Error::Error,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub fn compress(indata: &[u8]) -> Result<Vec<u8>, Error> {
    let mut wrkmem: MaybeUninit<[u8; LZO1X_1_MEM_COMPRESS]> = MaybeUninit::uninit();

    let inlen = indata.len();
    let outlen = inlen + inlen / 16 + 64 + 3;
    let mut outdata = Vec::with_capacity(outlen);

    unsafe {
        let r = lzo1x_1_compress(
            indata.as_ptr(),
            inlen as c_ulong,
            outdata.as_mut_ptr(),
            &outlen as *const _ as *mut _,
            wrkmem.as_mut_ptr() as *mut _);

        if r == 0 {
            outdata.set_len(outlen);
            return Ok(outdata)
        }

        return Err(Error::from_code(r))
    }
}

pub fn decompress(indata: &[u8], newlen: usize) -> Result<Vec<u8>, Error> {
    let inlen = indata.len();
    let mut outdata = Vec::with_capacity(newlen);

    unsafe {
        let r = lzo1x_decompress_safe(
            indata.as_ptr(),
            inlen as c_ulong,
            outdata.as_mut_ptr(),
            &newlen as *const _ as *mut _,
            ptr::null_mut());

        if r == 0 {
            outdata.set_len(newlen);
            return Ok(outdata)
        }

        return Err(Error::from_code(r))
    }
}