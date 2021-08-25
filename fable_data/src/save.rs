/// ## Format Description
///
/// Temporary comments from fabletlcmod.com.
///
/// ```txt
/// Game Save Specification
/// Header
///
/// 12byte File ID (Always= FableSave!..)
/// 4 Bytes - Offset to CRC (File is CRC'd from 0x0 until offset (Custom 32bit 1s Complement)
/// 4 Bytes - Chunk 1 Decompressed size
/// 4 Bytes - Chunk 2 Decompressed size
/// Chunk 1
///
/// 4 Bytes - Chunk 1 Compressed Size
/// Zlib compression of Chunk 1 (78 DA) (Best Compression Option)
/// Header
///
/// 7 Bytes = HEADER (NULL Terminated)
/// 4 Bytes = Chunk Size (minus 7 byte header)
///
/// From here on out everything is passed by CRC values of Strings (see above). Xbox is only 1 byte while PC contains 4 (PC // below!)
/// Control Bytes
///
///   CRC            Bytes        Value
/// 224FDEA8        (String)      World Name (Usually FinalAlbion) (Null Terminated)
/// A0CA0F5B          4           World Frame
/// A7E624D3          1           Boolean (Teleporting Enabled)
/// 20627570          1           Boolean (Saving Enabled)
/// D2AB286A          1           Boolean (Expirience Spending Enabled)
/// 50596E3F          1           Boolean (Creature Generators Enabled)
/// 7A109958          4           Creature Generation on Disabled Groups
/// B2DD2D45          1           Boolean (Hero Sleeping Enabled)
/// A4E20924          1           Boolean (Map Table Show Quest Cards)
/// B695FB1B          1           Boolean (Mini Map Enabled)
/// 481D7C4C          1           Boolean (Mini Map active before Disabled)
/// 500F26AD          1           Boolean (Guild Master Messages Enabled)
/// 2C8B8182          1           Boolean (Summoner Death Effects Hero)
/// 0FA53138          4           Int32   (Most Recent Save Type)
/// BD12926C          4           Int32   (Most Recent Save Type Before Manual Save)
/// DBA8729A        (String)      String  (Most Recent Manual Save Name) (Null Terminated, Unicode ends with 00 00)
/// 04649280          4           Float   (Save game marker position X) (view FinalAlbion.wld for all coordinates)
///                   4           Float   (Save game marker position Y)
///                   4           Float   (Save game marker position Z)
/// F295DFAE          4           Float   (Save Game Marker Angle X/Y)
/// 373CB734          4           Float   (Guild Seal Recall Position X)
///                   4           Float   (Guild Seal Recall Position Y)
///                   4           Float   (Guild Seal Recall Position Z)
/// C71A110A          4           Float   (Guild Seal Recall Marker Angle X/Y)
/// 0DF39CFD        (String)      String  (Current Region Name) (Null Terminated)
/// 50506DEB        (String)      String  (Current Mini Map Name) (Null Terminated)
/// 40242445          4           Int32   (Total Time Played)
///
/// Chunk 2
///
/// 4 Bytes - Chunk 2 Compressed Size
/// Decompressed Header
///
///   CRC          Bytes           Value
/// 67E8E2EE         4             Time (Float, Normalized Time of Day. 0-1.)
/// AD9E35E8         4             TimeStep (Float)
/// 344B012E         4             DayCount (Integer)
///
/// **ENTITIES Header**
/// [9] Bytes - ENTITIES (Null terminated)
/// [4] Bytes - Size of ENTITIES Chunk
/// [4] Byte  - NULL     (1 Byte Null on XBOX!)
/// [4] Bytes - Always (02 00 00 00)
/// [4] Bytes - NULL     (1 Byte Null on XBOX!)
/// [4] Bytes - Last Generated ID? (Integer)
/// [4] Bytes - NULL
///
/// **SAVED_ENTITIES Header**
/// [15] Bytes - SAVED_ENTITIES (Null terminated)
/// [4] Bytes - Offset to  SAVED_ENTITIES
/// [4] Bytes - NULL  (1 Byte Null on XBOX!)
/// [4] Bytes - Last Map ID Read (Integer)
/// [4] Bytes - NULL (1 Byte Null on XBOX!)
///
/// SAVED_ENTITIES Compressed
///
/// The saved entities for each map are compressed using a weaker form of zlib And are preceded by 8 bytes NULL (5 for Xbox)//  for each map, if the map doesn't have anything saved then the header isn't present either (it will still pass Null // Bytes per map until the next is found) (Blank Tngs create entries but there is no data present). The maps go in order // based on the FinalAlbion.wld
///
/// 4 Bytes - Total Block Size
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - MapUid (From FinalAlbion.wld)
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - Compressed Size
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - Decompressed Size
/// Zlib Begins
///
/// After all maps are cycled a Table Begins.
/// 4 Bytes - Number of Entries Each Entry is 20 bytes Long (PC) and 14 bytes on (Xbox)
///
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - Last Generated ID in file?
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - Map UID (From FinalAlbion.wld)
/// 4 Bytes - Null  (1 Byte Null on XBOX!)
/// 4 Bytes - CRC String of what Quest Section it relates to
///
/// SAVED_ENTITIES Decompressed
///
/// Header
///
/// [4] bytes - Number Quest Sections
/// [char]    - Quest Section String (Null terminated)
/// [4] bytes - Quest Section Size
///
/// Entities
///
/// [4] bytes - Entity Size
/// [char]    - Entity Type String (Null Terminated)
/// [1] bytes - NULL
/// [4] bytes - Entity Type
/// [1] bytes - NULL
/// [4] bytes - Entity UID
/// [1] bytes - NULL
/// [char]    - Entity Enum string (Null Terminated)
/// [4] bytes - Header Size (Base Properties)
/// ~Base Properties of the Entity
/// ~ Everything is passed by CRC String Values (Relative to Object, Creature similar to Bin Files)
/// ~Followed by Default CTC commands (Unless overridden in level script)
///
/// Misc Notes
///
/// Saved Npc's section (Not Compressed)
/// 16 Bytes - SAVED_NPC_NAMES (Null Terminated)
/// 4 Byte - Block Size
///
/// Then Begins NPC names:
///
/// 8 bytes (64 bit) - NPC UID
/// 4 bytes - Always 0A000000? seems byte size of "UNATTACHED"
/// 10 Bytes - "UNATTACHED"?
/// ~Unicode text string. Null Terminated Unicode (00 00)
///
/// Then Player information begins:
/// 7 Bytes - "PLAYER" (Null Terminated)
/// 4 Bytes - Size of PLAYER
/// 4 Bytes - 27C8AD96  (CRC String = PlayerCharacterUID)
/// 8 Bytes - Player UID (Quad)
/// ~Current Map String (Null Terminated)
///
/// Quest Section Begins (lzo1x_999 compressed, view as txt)
/// 7 Bytes - "QUESTS" (NULL terminated)
/// 4 Bytes - Block Size
/// 4 Bytes - Decompressed Size
/// 2 Byte - Compressed size
/// ~lzo compression
///
/// Next is REGIONS (Not Compressed)
/// 8 Bytes - "REGIONS" (NULL Terminated)
/// 4 Bytes - Block Size
/// 4 Bytes - Null
/// 4 Bytes - Number of sections (or regions)
/// ~ Regions begin Ascii text followed by a couple 00 01 patterns
///
/// Next is FACTIONS (lzo1x_999 compressed)
/// 9 Bytes - "FACTIONS" (NULL Terminated)
/// 4 Bytes - Compressed Block Size
/// 4 Bytes - C3450E4F (CRC String = NumberOfFactions)
/// 4 Bytes - Number of Files
/// 4 Bytes - D1F50398  (CRC String Unknown)
///
/// Each file is individually lzo'd compressed
/// 4 Bytes - decompressed size
/// 2 Bytes - compressed size
/// ```
pub struct Save {}
