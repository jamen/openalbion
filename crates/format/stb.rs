use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, PartialEq)]
pub struct Stb {
    pub header: StbHeader,
    pub entries_header: StbEntriesHeader,
    pub entries: Vec<StbEntry>,
}

#[derive(Debug, PartialEq)]
pub struct StbHeader {
    pub version: u32,
    pub header_size: u32,
    pub files_count: u32,
    pub levels_count: u32,
    pub entries_offset: u32,
}

#[derive(Debug, PartialEq)]
pub struct StbEntriesHeader {
    pub start: u32,
    pub levels_count: u32,
}

#[derive(Debug, PartialEq)]
pub struct StbEntry {
    pub listing_start: u32,
    pub id: u32,
    pub offset: u32,
    pub length: u32,
    pub name_1: String,
    pub name_2: String,
    pub extras: Option<StbEntryExtras>,
}

#[derive(Debug, PartialEq)]
pub struct StbEntryExtras {
    pub field_1: u32,
    pub field_2: u32,
    pub field_3: u32,
    pub field_4: u32,
}

/// Temporary comments from fabletlcmod.com.
///
/// ```txt
/// Stb .Lev
///
/// For Easier Editing Open STATIC_MAP_COMMON_HEADER Last Entry in .Stb Archive
///
/// Static Map Common Header
///
/// [4] Bytes - Number of Entries
///
/// Then You Need to Search for Filename (Data\Levels\FinalAlbion\xxxxx.Lev)
/// Once Desired Name is found there is an offset jump following name.
///
/// [4] Bytes - Offset Jump to File Data
///
/// [4] Bytes - Start Data (01 00 00 00)
/// [4] Bytes - Level ID
/// [4] Bytes - Unknown (Relative to Dimension of Map)
/// [4] Bytes - Map Dimension (X)
/// [4] Bytes - Map Dimesnion (Y)
/// [4] Bytes - Map World Offset (X)
/// [4] Bytes - Map World Offset (Y)
/// [4] Bytes - Null
/// [4] Bytes - Current File Offset Push
/// [4] Bytes - Current File Offset Push
///
/// [4] Bytes - New Section Start (0A 00 00 00)
/// [4] Bytes - Always (04 00 00 00)
/// [4] Bytes - Repeat of Unknown Data Above (Relative to Dimension of Map)
/// [4] Bytes - Checksum (Same as one used in .Wad Lev file)
/// [4] Bytes - Unknown (Always B3 16 34 7C?)
/// [4] Bytes - Unknown (Always 00 00 32 00)
/// [4] Bytes - Map World Offset Starting Position (X) {Float}
/// [4] Bytes - Map World Offset Starting Position (Y) {Float}
/// [4] Bytes - Map World Offset Starting Position (Z) {Float}
/// [4] Bytes - Map World Offset Ending Position (X) {Float}
/// [4] Bytes - Map World Offset Ending Position (Y) {Float}
/// [4] Bytes - Map World Offset Ending Position (Z) {Float}
/// [4] Bytes - Current File Offset Push
/// [4] Bytes - Current File Offset Push
///
/// [4] Bytes - New Section Start (01 00 00 00) (.Lev File Offsets)
/// [4] Bytes - Offset to First Table In Desired .Lev
/// [4] Bytes - Offset to Second Table In Desired .Lev
/// [4] Bytes - Second Table Size
/// [1] Byte - Indicator For Offset Push
/// [4] Bytes - Current File Offset Push
///
///
/// [4] Bytes - Section Length In Bytes *(Does not Start until after Next 4 Bytes)
/// [4] Bytes - New Section Start (01 E3 E3 12) (Flora Placement?)
/// [4] Bytes - World Placement (X) {Float}
/// [4] Bytes - World Placement (Y) {Float}
/// [4] Bytes - Height Placement (Z) {Float}
/// [4] Bytes - Unknown Placement (Rotation?) {Float}
/// [4] Bytes - Unknown Placement {Float}
/// [4] Bytes - Sections for Flora Data (In Static Map Header) (If Present)
/// [4] Bytes - Offset to Third Section (If Present) in Current .Lev
/// [4] Bytes - Size of Third Section (If Present) in Current .Lev
/// [4] Bytes - Offset in Third Section table (Third Section Offset + these bytes)
/// [4] Bytes - Null
/// [2] Bytes - Map Dimension (X)
/// [2] Bytes - Map Dimension (Y)
///
/// [1] Byte - Indicator for Internal Loop Within Last Segment
/// [4] Bytes - Number of Sections to loop *
///
/// *If Zero Data Ends Here for Current Map. If there is Value follow the next segment
/// Flora Loop
///
/// [4] Bytes - BIG ID #1
/// [4] Bytes - BIG ID #2
/// [4] Bytes - BIG ID #3
/// [4] Bytes - Placement (X)? {Float}
/// [4] Bytes - Placement (Y)? {Float}
/// [4] Bytes - Unknown (Area?) (Int)
/// [4] Bytes - Null (Int)
/// [4] Bytes - Unknown (Rotation/Angle?) {Float}
/// [4] Bytes - Unknown (Rotation/Angle?) {Float}
/// [4] Bytes - Unknown (Rotation/Angle?) {Float}
/// [4] Bytes - Placement on Map (X) (Secondary?) {Float}
/// [4] Bytes - Placement on Map (Y) (Secondary?) {Float}
/// [4] Bytes - Number of Sections (Int)
/// [4] Bytes - Unknown (type?) (Int)
/// [1] Byte - Boolean? Unknown
/// [1] Byte - Boolean? Unknown
/// [1] Byte - Boolean? Unknown
/// [1] Byte - Boolean? Unknown
/// [4] Bytes - Unknown (Int)
/// .Lev File Spec
/// First Table (Height)
///
/// Each Row is 36 Bytes long. Number of Rows is relative to ((X*Y) / 256) (dimensions of map divided by area of section /// always 16×16)
///
/// [4] Bytes - Offset
/// [4] Bytes - Compressed Size
/// [4] Bytes - World Map Starting Position (X) {Float}
/// [4] Bytes - World Map Starting Position (Y) {Float}
/// [4] Bytes - World Map Starting Position (Z) {Float}
/// [4] Bytes - World Map Ending Position (X) {Float}
/// [4] Bytes - World Map Ending Position (Y) {Float}
/// [4] Bytes - World Map Ending Position (Z) {Float}
/// [4] Bytes - Unknown
///
/// *Each Compressed Package is LZO Compressed following this format:
/// [4] Bytes - Decompressed Size
/// [4] Bytes - Compressed Size
/// Decompressed Data
///
/// [2] Bytes - Number of Sections
///
/// Looped
/// [2] Bytes - Number of Verts
/// [2] Bytes - Number of Faces
/// [1] Byte - Null
/// [4] Bytes - Textures.Big ID of Image used. (This may be only for Proc Textures) These are the actual Mesh
/// [4] Bytes - Textures.Big ID of Image used. (This may be repeated from above)
/// [4] Bytes - Textures.Big ID of Bump used.
/// [4] Bytes - Possibly another texture link
/// [4] Bytes - Possibly another texture link
/// [4] Bytes - Possibly another texture link
/// [1] Byte - Null
/// Vert Listing
///
/// 15 Byte rows
///
/// [2] Bytes - Global X Position
/// [2] Bytes - Global Y Position
/// [4] Bytes - Height {Float}
/// [4] Bytes - Packed Normal {PackedXYZ}
/// [1] Byte - 00 or FF (indicates next X or Y value?)
/// [1] Bytes - TU offset, Actual TU = (TU + (this-127)/127)
/// [1] Bytes - TU offset, Actual TU = (TU + (this-127)/127)
/// Faces
///
/// 2 Bytes
/// [2] Bytes - Number of Faces
/// Second Table (Unused Graphics?)
///
/// Originally believed to be the mesh, could be used for internal editor only.
/// [2] Bytes - Placement (X)
/// [2] Bytes - Placement (Y)
/// [2] Bytes - Tile Value (X) (Based on Dimension)*
/// [2] Bytes - Tile Value (Y) (Based on Dimension)*
///
/// *If Value is size of Map then this is the lowest LOD
///
/// [1] Byte - Indicator (01 - Indicates that it contains Offsets. Else it is only 47 Bytes in Length)
/// [1] Byte - Indicator (Section Start)
/// [1] Byte - Indicator (Section End)
/// [4] Bytes - Offset to Compressed Chunk
/// [4] Bytes - Size of Entire Table (If Value is present this indicates a start of a new LOD)
/// [4] Bytes - Offset Push Within Table (Next Section to Read, Table treated as internal file!)
/// [4] Bytes - World Map Start Position (X) {Float}
/// [4] Bytes - World Map Start Position (Y) {Float}
/// [4] Bytes - World Map Start Position (Z) {Float}
/// [4] Bytes - World Map Ending Position (X) {Float}
/// [4] Bytes - World Map Ending Position (Y) {Float}
/// [4] Bytes - World Map Ending Position (Z) {Float}
/// Offset Rows
///
/// If Indicator = 01 above then loop until starting indicator equals (stop indicator above)
/// 13 Byte Rows
///
/// [1] Byte - Indicator (Row start based on LOD)
/// [4] Bytes - Offset to compressed chunk
/// [4] Bytes - Compressed Chunk Size
/// [4] Bytes - Offset to specific compressed Data (Offset to compressed chunk + Value of These Bytes)
///
/// *Additional Notes Number of LODs seems dependent on First Table Rows. Example Creature Hub 4 Rows in first Table. // Graphics It Contains (4 High Quality, 6 Medium, 1 Low)
///
/// *Each Compressed Package is LZO Compressed following this format:
/// [4] Bytes - Decompressed Size
/// [4] Bytes - Compressed Size
/// Decompressed Data
///
/// [2] Bytes - Image Dimension (X) it takes up on Map *
/// [2] Bytes - Image Dimension (Y it takes up on Map *
///
/// *If Dimension equals map dimensions then it is the lowest LOD
///
/// [2] Bytes - Tile Placement of Image (Based on Map Dimensions) (X)
/// [2] Bytes - Tile Placement of Image (Based on Map Dimensions) (Y)
/// [1] Byte - Null
/// [1] Byte - Indicator for LOD (not an actual value!)
/// [3] Bytes - Unkown Based on LOD
/// [1] Byte - Indicator new section start
/// [1] Byte - Image Dimensions (X) (Actual Image Size, for editing)
/// [1] Byte - Image Dimensions (Y) (Actual Image Size, for editing)
/// [1] Byte - Indicator new section start
/// [2] Bytes - Image Dimensions Repeat (X) as 2 bytes
/// [2] Bytes - Image Dimensions Repeat (Y) as 2 bytes
/// [1] Byte - Indicator new section start
/// [4] Bytes - Unknown
/// [6] Bytes - Null?
/// [4] Bytes - Always (01 00 00 00) Indicates Image Start
/// ~ DXT1 Image (To get Image size take actual dimension ( X * Y / 2)
///
/// *The Rest of file is currently Unknown
/// Third Section (Flora/Model)
///
/// The only offset to this section exists in Static Map Header see: [4] Bytes - Offset in Third Section table (Third // Section Offset + these bytes)
///
/// *The following entries are very sloppy and jump around quite a bit.
///
/// [4] Bytes - always zero?
/// [4] Bytes - if zero again it will need to read until it is at least reaches 1.
/// [4] Bytes - Global X coords [Float]
/// [4] Bytes - Global Y coords [Float]
/// [4] Bytes - Global Z coords [Float]
/// [4] Bytes - Unknown (Rotation?) [Float]
/// [4] Bytes - Unknown (Always 23?) [Float]
/// [4] Bytes - Unknown [Integer]
/// [4] Bytes - Offset to Chunk [Integer]
/// [4] Bytes - Chunk Size [Integer]
/// [4] Bytes - Offset in Chunk (take above Offset + This) [Integer]
/// [2] Bytes - Placement X? [Short]
/// [2] Bytes - Placement Y? [Short]
/// [2] Bytes - Size X? [Short]
/// [2] Bytes - Size Y? [Short]
/// [1] Byte - Indicates more entries. If zero reading is finished.
///
/// Here is the Tricky Part. If Offset above leads directly to anything other than zero you read below, else the entry is // dead and the loop above is redone.
///
/// [4] Bytes - Number of Sections (Array) [Integer]
/// [4] Bytes - Chunk Offset [Integer]
/// [4] Bytes - Chunk Size [Integer]
/// [4] Bytes - Package Offset (Chunk Offset + Package Offset) (This will lead to compressed data) [Integer]
/// [4] Bytes - Global X coords [Float]
/// [4] Bytes - Global Y coords [Float]
/// [4] Bytes - Global Z coords [Float]
/// [4] Bytes - Unknown (Rotation?) [Float]
/// [4] Bytes - Unknown [Float]
/// [4] Bytes - Unknown [Integer]
/// [4] Bytes - Unknown [Integer]
/// Compressed Package
///
/// [4] Bytes - Decompressed Size
/// [4] Bytes - Compressed Size
/// *Data is LZO compressed
/// Decompressed Data
///
///
///
///
/// 1bit sign, 4bits exponent, 27bits mantissa
///
/// value = (-1^s)*(2^(e-7))*(f/0x8000000)
///
/// s eeee fff ffffffff ffffffff ffffffff  value
/// 0 0111 100 00100110 11010110 11010100  0.5189644396305084228515625
/// 0 0111 011 10011100 11000011 11110010  0.45154561102390289306640625
/// 0 0111 000 10100000 11010110 01001001  0.078533716499805450439453125
/// 1 0111 100 00100110 11000100 10101001 -0.518929786980152130126953125
/// 0 0111 011 10011100 10011011 00110000  0.45146787166595458984375
/// 0 0111 001 11000100 10001101 10010110  0.22097317874431610107421875
/// 0 0111 001 10100000 11001111 10101001  0.203521080315113067626953125
/// 1 0111 001 01101111 11100110 11100001 -0.179639585316181182861328125
/// 0 0111 100 00111000 00111010 10001101  0.527455426752567291259765625
///
/// 0 0111 011 01110001 00111100 11110011  0.430292032659053802490234375
/// 1 0111 100 00000001 10000100 01011111 -0.500740759074687957763671875
/// 0 0111 010 10111000 00110110 10110101  0.339948095381259918212890625
/// 0 0111 100 00000010 00011000 01011010  0.50102300941944122314453125
/// 0 0111 011 01111001 11101000 10000110  0.43452553451061248779296875
/// 0 0111 010 00101010 11011000 00111011  0.270920239388942718505859375
/// 1 0111 010 10011011 11111011 10001000 -0.326163351535797119140625
/// 0 0111 010 10000010 00000001 01011100  0.3134791553020477294921875
/// 0 0111 100 00001110 01111000 10011110  0.50706599652767181396484375
///
///
///
/// Also the game mostly uses lzo1x compression, and dxt(textures).
/// ```
///
///
///
///
///
///
/// 
// pub struct StbLev {}

// impl Stb {
//     pub fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
//         let mut header_buf: [u8; 32] = [0; 32];
//         let mut entries_header_buf: [u8; 12] = [0; 12];
//         let mut entries_buf = Vec::new();

//         source.read_exact(&mut header_buf)?;
//         let (_, header) = all_consuming(Stb::decode_header)(&header_buf)?;

//         source.seek(SeekFrom::Start(header.entries_offset as u64))?;
//         source.read_exact(&mut entries_header_buf)?;

//         let (_rest, entries_header) =
//             all_consuming(Stb::decode_entries_header)(&entries_header_buf)?;

//         source.read_to_end(&mut entries_buf)?;
//         let (_, entries) = count(Stb::decode_entry, header.levels_count as usize)(&entries_buf)?;

//         Ok(Stb {
//             header: header,
//             entries_header: entries_header,
//             entries: entries,
//         })
//     }

//     pub fn decode_header(input: &[u8]) -> IResult<&[u8], StbHeader, Error> {
//         let (input, _magic_number) = tag("BBBB")(input)?;
//         let (input, version) = le_u32(input)?;
//         let (input, _unknown_1) = le_u32(input)?;
//         let (input, _unknown_2) = le_u32(input)?;
//         let (input, header_size) = le_u32(input)?;
//         let (input, files_count) = le_u32(input)?;
//         let (input, levels_count) = le_u32(input)?;
//         let (input, entries_offset) = le_u32(input)?;

//         Ok((
//             input,
//             StbHeader {
//                 version: version,
//                 header_size: header_size,
//                 files_count: files_count,
//                 levels_count: levels_count,
//                 entries_offset: entries_offset,
//             },
//         ))
//     }

//     pub fn decode_entries_header(input: &[u8]) -> IResult<&[u8], StbEntriesHeader, Error> {
//         let (input, start) = le_u32(input)?;
//         let (input, _null) = le_u32(input)?;
//         let (input, levels_count) = le_u32(input)?;

//         Ok((
//             input,
//             StbEntriesHeader {
//                 start: start,
//                 levels_count: levels_count,
//             },
//         ))
//     }

//     pub fn decode_entry(input: &[u8]) -> IResult<&[u8], StbEntry, Error> {
//         let (input, listing_start) = le_u32(input)?;
//         let (input, id) = le_u32(input)?;
//         let (input, _null) = le_u32(input)?;
//         let (input, length) = le_u32(input)?;
//         let (input, offset) = le_u32(input)?;
//         let (input, _null) = le_u32(input)?;
//         let (input, name_1) = decode_rle_string(input)?;
//         let (input, _null) = le_u32(input)?;
//         let (input, _unknown_1) = le_u32(input)?;
//         let (input, name_2) = decode_rle_string(input)?;
//         let (input, bytes_left) = le_u32(input)?;

//         // These aren't very useful until they can be understood.

//         let (input, extras) =
//             // TODO: Is this being misused? Maybe there can be different sized extras.
//             if bytes_left != 0 {
//                 let (input, field_1) = le_u32(input)?;
//                 let (input, field_2) = le_u32(input)?;
//                 let (input, field_3) = le_u32(input)?;
//                 let (input, field_4) = le_u32(input)?;
//                 (
//                     input,
//                     Some(
//                         StbEntryExtras {
//                             field_1: field_1,
//                             field_2: field_2,
//                             field_3: field_3,
//                             field_4: field_4,
//                         }
//                     )
//                 )
//             } else {
//                 (input, None)
//             };

//         Ok((
//             input,
//             StbEntry {
//                 listing_start: listing_start,
//                 id: id,
//                 length: length,
//                 offset: offset,
//                 name_1: name_1,
//                 name_2: name_2,
//                 extras: extras,
//             },
//         ))
//     }
// }
