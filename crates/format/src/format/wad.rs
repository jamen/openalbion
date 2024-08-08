use crate::{put, put_bytes, take, take_bytes, Loc};

#[derive(Debug, Copy, Clone, Default)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeated: u32,
    pub first_entry_position: u32,
}

#[derive(Debug)]
pub enum WadHeaderPart {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    RepeatedEntryCount,
    FirstEntryOffset,
}

impl WadHeader {
    pub fn parse(i: &mut &[u8]) -> Result<Self, Loc<WadHeaderPart>> {
        use WadHeaderPart::*;

        let magic = take::<[u8; 4]>(i).ok_or_else(|| Loc::new(i, Magic))?;

        let version = take::<[u32; 3]>(i)
            .ok_or_else(|| Loc::new(i, Version))?
            .map(u32::to_le);

        let block_size = take::<u32>(i)
            .ok_or_else(|| Loc::new(i, BlockSize))?
            .to_le();

        let entry_count = take::<u32>(i)
            .ok_or_else(|| Loc::new(i, EntryCount))?
            .to_le();

        let entry_count_repeated = take::<u32>(i)
            .ok_or_else(|| Loc::new(i, RepeatedEntryCount))?
            .to_le();

        let first_entry_position = take::<u32>(i)
            .ok_or_else(|| Loc::new(i, FirstEntryOffset))?
            .to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entry_count,
            entry_count_repeated,
            first_entry_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderPart> {
        use WadHeaderPart::*;

        put(out, &self.magic).ok_or(Magic)?;
        put(out, &self.version.map(u32::to_le)).ok_or(Version)?;
        put(out, &self.block_size.to_le()).ok_or(BlockSize)?;
        put(out, &self.entry_count.to_le()).ok_or(EntryCount)?;
        put(out, &self.entry_count_repeated.to_le()).ok_or(RepeatedEntryCount)?;
        put(out, &self.first_entry_position.to_le()).ok_or(FirstEntryOffset)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WadEntry<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub offset: u32,
    pub length: u32,
    pub unknown_3: u32,
    pub path: &'a [u8],
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

#[derive(Debug)]
pub enum WadEntryPart {
    Unknown1,
    Id,
    Unknown2,
    Offset,
    Length,
    Unknown3,
    PathLen,
    Path,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

impl<'a> WadEntry<'a> {
    pub fn parse(i: &mut &'a [u8]) -> Result<WadEntry<'a>, Loc<WadEntryPart>> {
        use WadEntryPart::*;

        let unknown_1 = take::<[u8; 16]>(i).ok_or_else(|| Loc::new(i, Unknown1))?;
        let id = take::<u32>(i).ok_or_else(|| Loc::new(i, Id))?.to_le();
        let unknown_2 = take::<u32>(i).ok_or_else(|| Loc::new(i, Unknown2))?.to_le();
        let offset = take::<u32>(i).ok_or_else(|| Loc::new(i, Offset))?.to_le();
        let length = take::<u32>(i).ok_or_else(|| Loc::new(i, Length))?.to_le();
        let unknown_3 = take::<u32>(i).ok_or_else(|| Loc::new(i, Unknown3))?.to_le();

        let path_len = take::<u32>(i).ok_or_else(|| Loc::new(i, PathLen))?.to_le() as usize;
        let path = take_bytes(i, path_len).ok_or_else(|| Loc::new(i, Path))?;

        let unknown_4 = take::<[u8; 16]>(i).ok_or_else(|| Loc::new(i, Unknown4))?;

        let created = take::<[u32; 7]>(i)
            .ok_or_else(|| Loc::new(i, Created))?
            .map(u32::to_le);

        let accessed = take::<[u32; 7]>(i)
            .ok_or_else(|| Loc::new(i, Accessed))?
            .map(u32::to_le);

        let modified = take::<[u32; 5]>(i)
            .ok_or_else(|| Loc::new(i, Modified))?
            .map(u32::to_le);

        Ok(WadEntry {
            unknown_1,
            id,
            unknown_2,
            offset,
            length,
            unknown_3,
            path,
            unknown_4,
            created,
            accessed,
            modified,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadEntryPart> {
        use WadEntryPart::*;

        put(out, &self.unknown_1).ok_or(Unknown1)?;
        put(out, &self.id.to_le()).ok_or(Id)?;
        put(out, &self.unknown_2.to_le()).ok_or(Unknown2)?;
        put(out, &self.offset.to_le()).ok_or(Offset)?;
        put(out, &self.length.to_le()).ok_or(Length)?;
        put(out, &self.unknown_3.to_le()).ok_or(Unknown3)?;

        let path_size = u32::try_from(self.path.len()).or(Err(PathLen))?;

        put(out, &path_size.to_le()).ok_or(PathLen)?;

        put_bytes(out, &self.path).ok_or(Path)?;

        put(out, &self.unknown_4).ok_or(Unknown4)?;
        put(out, &self.created.map(u32::to_le)).ok_or(Created)?;
        put(out, &self.accessed.map(u32::to_le)).ok_or(Accessed)?;
        put(out, &self.modified.map(u32::to_le)).ok_or(Modified)?;

        Ok(())
    }
}
