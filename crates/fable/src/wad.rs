//! The world archive.
//!
//! See [`Wad`] and [`WadReader`].
//!
//! These archives contain only [`lev`] and [`tng`] files.
//!
//! Usually there's only one occurence at `Data/Levels/FinalAlbion.wad`.
//!
//! To read the archive, you use the header to find the entries and the entries to find the contents.
//!
//! ## Format Description
//!
//! | Section | Description                                              |
//! |---------|----------------------------------------------------------|
//! | Header  | The header.                                              |
//! | Blob    | The file contents in contiguous chunks.                  |
//! | Entries | The entries list with file locations and metadata.       |
//!
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
//! The entries start at the header's first entry offset and are repeated until the entry count.
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
//! | Created At  | `[7; u32]` | 28        | Creation timestamp.                              |
//! | Accessed At | `[7; u32]` | 28        | Access timestamp.                                |
//! | Written At  | `[5; u32]` | 20        | Write timestamp.                                 |
//!
//! ### Timestamps
//!
//! This is a description for the "created at" and "accessed at" fields. The "written at" field is the same but only percise to minutes.
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
//!
//! [`lev`]: ../lev/index.html
//! [`tng`]: ../tng/index.html
//! [`Wad`]: ./struct.Wad.html
//! [`WadReader`]: ./struct.WadReader.html

pub mod decode;
pub mod encode;
pub mod reader;

use chrono::naive::NaiveDateTime;

use std::fs::File;
use std::io::Take;

#[derive(Debug,PartialEq)]
pub struct Wad {
    pub header: WadHeader,
    pub entries: Vec<WadEntry>
}

#[derive(Debug,PartialEq)]
pub struct WadHeader {
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entries_count: u32,
    pub entries_offset: u32,
}

#[derive(Debug,PartialEq)]
pub struct WadEntry {
    pub id: u32,
    pub offset: u32,
    pub length: u32,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub accessed_at: NaiveDateTime,
    pub written_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct WadReader<'a> {
    pub source: Take<&'a mut File>,
    pub entry: WadEntry,
    pub position: u64,
}