//! An archive of [`lev`] and [`tng`] files
//!
//! ## Format
//!
//! ### Header
//!
//! | Field                  | Type        | Byte Size | Description                     |
//! |------------------------|-------------|-----------|---------------------------------|
//! | Magic number           | `[4; char]` | 4         | `"BBBB"` string.                |
//! | Version                | `[3; u32]`  | 12        | File version number.            |
//! | Block size             | `u32`       | 4         | Chunk size. Usually `2048`.     |
//! | Entry count            | `u32`       | 4         | Amount of entries.              |
//! | Entry count repeated   | `u32`       | 4         |                                 |
//! | First entry offset     | `u32`       | 4         | Offset to the first entry .     |
//!
//! ### Entry
//!
//! An entry's layout is:
//!
//! | Field       | Type       | Byte Size | Description                                      |
//! |-------------|------------|-----------|--------------------------------------------------|
//! | Unknown     | `[16; u8]` | 16        | (Maybe hash-related.)                            |
//! | File Id     | `u32`      | 4         | Index number. (This is implicit tho lol?)        |
//! | Unknown     | `u32`      | 4         |                                                  |
//! | File size   | `u32`      | 4         | File size in the blob.                           |
//! | File offset | `u32`      | 4         | File offset in the blob.                         |
//! | Unknown     | `u32`      | 4         |                                                  |
//! | Path size   | `u32`      | 4         | Size of the path string that follows.            |
//! | Path string | `String`   | Path size | File path                                        |
//! | Unknown     | `[16; u8]` | 16        | (Maybe some kind of metadata like perms.)        |
//! | Created     | `[7; u32]` | 28        | Creation timestamp.                              |
//! | Accessed    | `[7; u32]` | 28        | Access timestamp.                                |
//! | Written     | `[5; u32]` | 20        | Write timestamp.                                 |
//!
//! The write timestamp only goes to minutes for reasons unknown
//!
//! ### Timestamp
//!
//! | Field       | Type  | Byte Size |
//! |-------------|-------|-----------|
//! | Year        | `u32` | 4         |
//! | Month       | `u32` | 4         |
//! | Day         | `u32` | 4         |
//! | Hour        | `u32` | 4         |
//! | Minute      | `u32` | 4         |
//! | Second      | `u32` | 4         |
//! | Millisecond | `u32` | 4         |

#![feature(slice_take)]

mod compile;
mod parse;

pub use compile::*;
pub use parse::*;

#[derive(Debug, PartialEq, Eq)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub repeated_entry_count: u32,
    pub first_entry_offset: u32,
}

pub enum WadHeaderPart {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    RepeatedEntryCount,
    FirstEntryOffset,
}

impl WadHeader {
    pub const TOTAL_BYTE_SIZE: usize = 32;
}

#[derive(Debug, PartialEq, Eq)]
pub struct WadEntry<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub offset: u32,
    pub length: u32,
    pub unknown_3: u32,
    pub path: &'a [u8],
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

pub enum WadEntryPart {
    Unknown1,
    Id,
    Unknown2,
    Offset,
    Length,
    Unknown3,
    PathLen,
    Path,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

impl<'a> WadEntry<'a> {
    pub const PARTIAL_BYTE_SIZE: usize = 132;
}
