// From fabletlcmod.com:
//
// .Lut
//
// All Offsets in .Lut files do not count 44 byte header.
// Header
//
// [27] Bytes - Header (LiOnHeAdLHAudioBankCompData)
// [13] Bytes - Null
// [4] Bytes - Offset to Audio Bank Lookup Table
// Audio Bank Lookup Table
//
// [22] Bytes - Header (LHAudioBankLookupTable)
// [10] Bytes - Null
// [4] Bytes - Number of Bytes left in Table (size)
// [4] Bytes - File Type (1000 for main, 500 for secondary)
// [4] Bytes - Number of Files
// Table loop
//
// Each Entry takes up 12 Bytes. Loop until number of files is met.
//
// [4] Bytes - File ID
// [4] Bytes - Chunk Size (All .Lut entries contain 36 Byte Header)
// [4] Bytes - Offset (Remember initial 44 byte header is not counted)