mod decode;
mod encode;

use std::io::BufReader;

#[derive(Debug,PartialEq)]
pub struct Stb {
    pub header: StbHeader,
    pub entries_header: StbEntriesHeader,
    pub entries: Vec<StbEntry>,
}

#[derive(Debug,PartialEq)]
pub struct StbHeader {
    pub version: u32,
    pub header_size: u32,
    pub files_count: u32,
    pub levels_count: u32,
    pub entries_offset: u32,
}

#[derive(Debug,PartialEq)]
pub struct StbEntriesHeader {
    pub start: u32,
    pub levels_count: u32,
}

#[derive(Debug,PartialEq)]
pub struct StbEntry {
    pub listing_start: u32,
    pub id: u32,
    pub offset: u32,
    pub length: u32,
    pub name_1: String,
    pub name_2: String,
    pub extras: Option<StbEntryExtras>,
}

#[derive(Debug,PartialEq)]
pub struct StbEntryExtras {
    pub field_1: u32,
    pub field_2: u32,
    pub field_3: u32,
    pub field_4: u32,
}

#[derive(Debug)]
pub struct StbReader<Source> {
    pub source: BufReader<Source>,
    pub entry: StbEntry,
}