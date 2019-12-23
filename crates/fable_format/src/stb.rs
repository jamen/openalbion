pub mod decode;
pub mod encode;

// Temporary comments from fabletlcmod.com.
//
// ==== Stb Archive Header ====
//
// [4] Bytes - File ID (BBBB)
// [4] Bytes - Build Verion
// [4] Bytes - Unknown
// [4] Bytes - Always (0100 0000)
// [4] Bytes - Header Size
// [4] Bytes - Number of Files
// [4] Bytes - Number of Levels
// [4] Bytes - Offset to Developer Listings
//
// === Developer Header ===
//
// [4] Bytes - Start (0100 0000)
// [4] Bytes - Null
// [4] Bytes - Number of Levels
//
// ==== Developer Listings ====
//
// [4] Bytes - Listing Start (2A00 0000)
// [4] Bytes - File ID
// [4] Bytes - Null
// [4] Bytes - File Size
// [4] Bytes - Offset
// [4] Bytes - Null
// [4] Bytes - Length of String Name
// ~ String
// [4] Bytes - Null
// [4] Bytes - Always 01
// [4] Bytes - Length of Second String Name
// ~ String
// [4] Bytes - Bytes Left in Listing *
//
// *This is Where regular Entries end. Only Engine Listings have data past here.
// [4] Bytes - Always 0C
// [4] Bytes - Always 1600 0000
// [4] Bytes - Null
// [4] Bytes - Unknown (Enumerator or a CRC?)
//
// Last Entry: STATIC_MAP_COMMON_HEADER Needs to be accessed for easier editing on the individual .Lev Files

#[derive(Debug,PartialEq)]
pub struct StbHeader {
    version: u32,
    header_size: u32,
    files_count: u32,
    levels_count: u32,
    developer_listings: u32,
}

#[derive(Debug,PartialEq)]
pub struct StbDevHeader {
    listing_start: u32,
    file_id: u32,
    file_size: u32,
    offset: u32,
    file_name: String,
    file_name_2: String,
    bytes_left: u32,
}