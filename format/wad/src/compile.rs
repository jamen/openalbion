use common::{bytemuck::PodCastError, WritePod};

use crate::{WadEntry, WadHeader};

pub struct WadHeaderCompileError;

impl From<PodCastError> for WadHeaderCompileError {
    fn from(_x: PodCastError) -> Self {
        Self
    }
}

impl WadHeader {
    pub fn compile(&self) -> Result<[u8; WadHeader::TOTAL_BYTE_SIZE], WadHeaderCompileError> {
        let mut buf = [0; WadHeader::TOTAL_BYTE_SIZE];
        let mut out = &mut buf[..];
        out.write_pod(&self.magic)?;
        out.write_pod(&self.version.map(u32::to_le))?;
        out.write_pod(&self.block_size.to_le())?;
        out.write_pod(&self.entry_count.to_le())?;
        out.write_pod(&self.repeated_entry_count.to_le())?;
        out.write_pod(&self.first_entry_offset.to_le())?;
        Ok(buf)
    }
}

pub struct WadEntryCompileError;

impl From<PodCastError> for WadEntryCompileError {
    fn from(_x: PodCastError) -> Self {
        Self
    }
}

impl WadEntry<'_> {
    pub fn compile(&self) -> Result<Box<[u8]>, WadHeaderCompileError> {
        let size = Self::PARTIAL_BYTE_SIZE + self.path.len();
        let mut buf = vec![0; size].into_boxed_slice();
        let mut out = &mut buf[..];
        out.write_pod(&self.unknown_1)?;
        out.write_pod(&self.id.to_le())?;
        out.write_pod(&self.unknown_2.to_le())?;
        out.write_pod(&self.offset.to_le())?;
        out.write_pod(&self.length.to_le())?;
        out.write_pod(&self.unknown_3.to_le())?;
        out.write_pod(&(self.path.len() as u32).to_le())?;
        out.copy_from_slice(self.path);
        out.write_pod(&self.unknown_4)?;
        out.write_pod(&self.created.map(u32::to_le))?;
        out.write_pod(&self.accessed.map(u32::to_le))?;
        out.write_pod(&self.modified.map(u32::to_le))?;
        Ok(buf)
    }
}
