use common::put;

use crate::{WadHeader, WadHeaderPart};

impl WadHeader {
    pub fn compile(&self) -> Result<[u8; WadHeader::TOTAL_BYTE_SIZE], WadHeaderPart> {
        let mut buf = [0; WadHeader::TOTAL_BYTE_SIZE];
        let out = &mut buf[..];
        put(out, &self.magic, WadHeaderPart::Magic)?;
        put(out, &self.version.map(u32::to_le), WadHeaderPart::Version)?;
        put(out, &self.block_size.to_le(), WadHeaderPart::BlockSize)?;
        put(out, &self.entry_count.to_le(), WadHeaderPart::EntryCount)?;
        put(
            out,
            &self.repeated_entry_count.to_le(),
            WadHeaderPart::RepeatedEntryCount,
        )?;
        put(
            out,
            &self.first_entry_offset.to_le(),
            WadHeaderPart::FirstEntryOffset,
        )?;
        Ok(buf)
    }
}

use crate::{WadEntry, WadEntryPart};

impl WadEntry<'_> {
    pub fn compile(&self) -> Result<Box<[u8]>, WadEntryPart> {
        let size = Self::PARTIAL_BYTE_SIZE + self.path.len();
        let mut buf = vec![0; size].into_boxed_slice();
        let out = &mut buf[..];
        put(out, &self.unknown_1, WadEntryPart::Unknown1)?;
        put(out, &self.id.to_le(), WadEntryPart::Id)?;
        put(out, &self.unknown_2.to_le(), WadEntryPart::Unknown2)?;
        put(out, &self.offset.to_le(), WadEntryPart::Offset)?;
        put(out, &self.length.to_le(), WadEntryPart::Length)?;
        put(out, &self.unknown_3.to_le(), WadEntryPart::Unknown3)?;
        put(
            out,
            &(self.path.len() as u32).to_le(),
            WadEntryPart::PathLen,
        )?;
        out.copy_from_slice(self.path);
        put(out, &self.unknown_4, WadEntryPart::Unknown4)?;
        put(out, &self.created.map(u32::to_le), WadEntryPart::Created)?;
        put(out, &self.accessed.map(u32::to_le), WadEntryPart::Accessed)?;
        put(out, &self.modified.map(u32::to_le), WadEntryPart::Modified)?;
        Ok(buf)
    }
}
