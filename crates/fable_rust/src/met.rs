pub mod decode;
pub mod encode;

// From fabletlcmod.com
//
// .Met
//
// Only documenting .Met files as they control .lug audio banks.
// .Met are usually 2 sections (Audio Information, Environment Settings) Ingame.Met contains a third section that links // audio to compiled events.
// Header
//
// [4] Bytes - Always (01 00 00 00)
// [4] Bytes - Number of Files
// [4] Bytes - Unknown
// Audio Information Listing
//
// [4] Bytes - Entry Number
// [4] Bytes - ID
// [4] Bytes - Import Directory File Name Length (String Size)
// ~ String
// [4] Bytes - Audio File Size (Does not have header like .Lut audio files)
// [4] Bytes - Offset
// [2] Bytes - Number of Channels
// [2] Bytes - Audio Codec (105 = Xb-Adpcm, 1 = PCM) Only two used.
// [4] Bytes - Frequency
// [8] Bytes - Event ID, mostly unused
// Environment Audio Listing
//
// Few pieces are unknown! (Hard to test some of these)
// Header for Environment Audio
//
// [4] Bytes - Number of Files
// Audio Listing Loop
//
// 48 Bytes per listing. Loop until file number is met.
//
// [4] Bytes - File Counter
// [4] Bytes - File ID
// [4] Bytes - String Size for Group Listing (If zero, then string does not exist)
// ~ String (If it exists)
// [2] Bytes - Event Identifier (01 = Day Sound, 02 = Night Sound, 00 = Should be anytime)
// [2] Bytes - Unknown
// [4] Bytes - Number of Loops (Usually either 1 or 9999)
// [4] Bytes - Unknown (relative to number of loops, should be memory allocation)
// [4] Bytes - Unknown (usually nothing)
// [2] Bytes - Unknown (relative to sound volume. Should be either initial gain or user gain.)
// [2] Bytes - Unknown (relative to sound volume. Should be either initial gain or user gain.)
// [2] Bytes - Unknown (3D volume or Head Relative)
// [2] Bytes - Unknown (2D volume?)
// [4] Bytes - Location to Play sound (X) {Float}
// [4] Bytes - Location to Play sound (Y) {Float}
// [4] Bytes - Priority
//
// *Upon Last Entry after this loop. If the 4 bytes has value, push to next section.
// Event Listings
//
// Only Exists in Ingame.Met
//
// [4] Bytes Number of Entries
// Event Loop
//
// [4] Bytes - String Length
// ~ String (Event Name)
// [4] Bytes - Number of Sounds to Associate with Event Name
// ~Then Based on number of sounds to associate are 4 byte ID listings

pub struct Met {
}