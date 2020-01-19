pub mod decode;
pub mod encode;
pub mod reader;

use chrono::naive::NaiveDateTime;
use std::fs::File;

#[derive(Debug,PartialEq)]
pub struct Wad {
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entries_count: u32,
    pub entries_offset: u32,
    pub entries: Vec<WadEntry>,
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

pub struct WadReader<'a> {
    pub source: &'a File,
    pub entry: WadEntry,
}