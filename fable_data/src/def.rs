// TODO: Convert def.xml and the .def files into a bunch of structures used for parsing .bin.

// mod xml;

// pub use xml::*;

use std::io::Read;

use crate::Bytes;

// TODO: DefNames?
pub struct NamesBin {
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub names: Vec<(u32, String)>,
}

// TODO: Def?
pub struct Bin {}

impl NamesBin {
    pub fn decode(mut data: &[u8]) -> Option<NamesBin> {
        let unknown_1 = data.parse_u32_le()?;
        let unknown_2 = data.parse_u32_le()?;
        let names_count = data.parse_u32_le()?;
        let unknown_3 = data.parse_u32_le()?;
        let unknown_4 = data.parse_u32_le()?;

        let mut names = Vec::new();

        while names.len() < names_count as usize {
            let unknown_6 = data.parse_u32_le()?;
            let name = data.parse_str_until_nul()?.to_owned();
            names.push((unknown_6, name));
        }

        Some(NamesBin {
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            names,
        })
    }
}

impl Bin {
    pub fn decode(mut data: &[u8]) -> Option<Bin> {
        let _starting_len = data.len();

        let _unknown_1 = data.parse_u8()?;
        let _unknown_2 = data.parse_u32_le()?;
        let _unknown_3 = data.parse_u32_le()?;
        let entries_count = data.parse_u32_le()?;

        let mut entries = Vec::new();

        while entries.len() < entries_count as usize {
            let unknown_1 = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let unknown_3 = data.parse_u32_le()?;
            if unknown_1 == 0xFFFF {
                // println!("{:?} {:?} {:?}", unknown_1, unknown_2, unknown_3);
            }
            entries.push((unknown_1, unknown_2, unknown_3));
        }

        // println!("{:?}", entries);

        let chunks_count = data.parse_u32_le()?;
        let _unknown_4 = data.parse_u32_le()?;

        // println!("{:?} {:?}", chunks_count, unknown_4);

        let mut chunks_table = Vec::new();

        while chunks_table.len() < chunks_count as usize {
            let offset = data.parse_u32_le()?;
            let unknown_1 = data.parse_u32_le()?;
            chunks_table.push((offset, unknown_1));
        }

        chunks_table.sort_by(|a, b| a.0.cmp(&b.0));

        // let chunks_offset = starting_len - data.len();

        let mut bytes = Vec::new();

        for (i, (start, idk)) in chunks_table.iter().copied().enumerate() {
            println!("{:?}", idk);

            let compressed_chunk = match chunks_table.get(i + 1).and_then(|x| Some(x.0)) {
                Some(end) => &data[start as usize..end as usize],
                None => continue,
            };

            let mut decompressor = flate2::bufread::ZlibDecoder::new(compressed_chunk);

            // let mut chunk = Vec::new();

            decompressor.read_to_end(&mut bytes).ok()?;

            // println!("{:?}", &bytes[..1024]);

            break;

            // let initial_offset = data.parse_u16_le()?;

            // println!("{:?} {:?}", initial_offset, file_count);

            // println!("{:?}", &chunk[initial_offset as usize..]);

            // println!("{:?}", chunk.len());
        }

        // let stdout = std::io::stdout();
        // let mut handle = stdout.lock();
        // handle.write_all(&bytes).ok()?;

        // for (offset, unknown_1) in chunks_table.iter() {

        // }

        // for (offset, unknown_1) in chunks_table.iter() {

        // }

        // println!("{:?}", chunks_table);

        Some(Bin {})
    }
}

// // use std::io::{Read,Seek};
// // use crate::Error;

// /// Compiled def format.
// ///
// /// ## Format Description
// ///
// /// WIP
// pub struct DefBin {
//     pub header: DefBinHeader,
//     pub entries: Vec<DefBinNameLookup>,
// }

// // Header
// //
// // [1] Byte - 00 (Indicates to use Names.Bin as Library)
// // [4] Bytes - File Indicator
// // [4] Bytes - Platform Indicator (Xbox / PC)
// // [4] Bytes - Number of Entries
// pub struct DefBinHeader {
//     pub use_names_bin: u8,
//     pub file_indicator: u32,
//     pub platform_indicator: u32,
//     pub entries_count: u32,
// }

// // Names Lookup
// //
// // Each Row is 12 Bytes long. Loop until Number of Entries is met.
// //
// // [4] Bytes - Definition Name Offset in Names.Bin
// // [4] Bytes - »PC File Name offset in Names.Bin »Xbox Enumerator for specific file.
// // [4] Bytes - Counter Based on Definition Used
// //
// // *Exceptions For File Name or Enumerator Bytes:
// // PC If the entry Equals (FF FF FF FF) Name is Defined outside of Names.Bin and does not require parsing.
// // Xbox If the enumeration equals (00 00 00 00) Then Enumerator is Defined in Names.Bin
// //
// pub struct DefBinNameLookup {
//     pub definition_offset: u32,
//     pub file_name_offset: u32,
//     pub counter: u32,
// }

// // Second Table Header
// //
// //
// // [4] Bytes - Number of Compressed Chunks (Actual Compressed Chunks is always one less)
// // [4] Bytes - Null
// //
// pub struct DefSecondTableHeader {
//     pub compressed_chunks_count: u32,
//     pub unknown1: u32,
// }

// // Second Table Lookup (Compressed)
// //
// //
// // Each Row is 8 Bytes long, loop until number of compressed chunks is met.
// //
// // [4] Bytes - Offset to Compressed Chunk
// // [4] Bytes - Last File Number Contained in Chunk (Running Counter)
// //
// // *Each Offset is based after the second table ends. (Equals (Number of Entries * 12 + 13 byte header) + (Number of Compressed Chunks * 8 + 8 byte header))
// //
// //
// // Each Compressed Chunk is Zlib Compressed
// //
// pub struct DefSecondTableRow {
//     pub compressed_chunk_offset: u32,
//     pub last_file_number: u32,
// }

// // Decompressed
// //
// // [2] Bytes - Offset
// //
// // *If you divide initial Offset by 2 it equals number of files
// // Each File uses indicator bytes that are defined in the NullDefs (Based on Definition From First Table)
// //
// // // *Additional Notes: PC .Bin Files require Names.Bin to be parsed into each file offset. Files are listed in order in the First Table.
// // Bin Entries
// //
// pub struct DefSecondTableRowDecompressed {
//     pub offset: u16,
// }

// impl DefBin {
//     // pub fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
//     //     todo!()
//     // }
// }
