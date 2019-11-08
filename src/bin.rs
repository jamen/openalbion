pub mod decode;
pub mod encode;

// Temporary comments from fabletlcmod.com.
//
// Compiled Defs (.Bin)
//
// Game.Bin, Script.Bin and Frontend.Bin use Names.Bin as a library.
//
// All Names.Bin offsets do not count 20 byte header
// Header
//
// [1] Byte - 00 (Indicates to use Names.Bin as Library)
// [4] Bytes - File Indicator
// [4] Bytes - Platform Indicator (Xbox / PC)
// [4] Bytes - Number of Entries
// Names Lookup
//
// Each Row is 12 Bytes long. Loop until Number of Entries is met.
//
// [4] Bytes - Definition Name Offset in Names.Bin
// [4] Bytes - »PC File Name offset in Names.Bin »Xbox Enumerator for specific file.
// [4] Bytes - Counter Based on Definition Used
//
// *Exceptions For File Name or Enumerator Bytes:
// PC If the entry Equals (FF FF FF FF) Name is Defined outside of Names.Bin and does not require parsing.
// Xbox If the enumeration equals (00 00 00 00) Then Enumerator is Defined in Names.Bin
//
// Second Table Header
//
//
// [4] Bytes - Number of Compressed Chunks (Actual Compressed Chunks is always one less)
// [4] Bytes - Null
//
// Second Table Lookup (Compressed)
//
//
// Each Row is 8 Bytes long, loop until number of compressed chunks is met.
//
// [4] Bytes - Offset to Compressed Chunk
// [4] Bytes - Last File Number Contained in Chunk (Running Counter)
//
// *Each Offset is based after the second table ends. (Equals (Number of Entries * 12 + 13 byte header) + (Number of Compressed Chunks * 8 + 8 byte header))
//
//
// Each Compressed Chunk is Zlib Compressed
//
// Decompressed
//
// [2] Bytes - Offset
//
// *If you divide initial Offset by 2 it equals number of files
// Each File uses indicator bytes that are defined in the NullDefs (Based on Definition From First Table)
//
// // *Additional Notes: PC .Bin Files require Names.Bin to be parsed into each file offset. Files are listed in order in the First Table.
// Bin Entries
//
// more as we get time…