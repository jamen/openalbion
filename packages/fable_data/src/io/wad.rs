use crate::format::wad::{WadAssetInfo, WadAssetInfoSection, WadHeader, WadHeaderSection};
use derive_more::{Display, Error};
use fallible_iterator::FallibleIterator;
use std::{
    borrow::Cow,
    io::{self, Read, Seek, SeekFrom, Write},
};

pub struct WadReader<Source: Read + Seek> {
    source: Source,
}

impl<Source: Read + Seek> WadReader<Source> {
    pub fn new(source: Source) -> Self {
        Self { source }
    }

    pub fn read_header(&mut self) -> Result<WadHeader, WadReaderError> {
        use WadReaderError as E;

        let mut header_bytes = [0; WadHeader::BYTE_SIZE];

        self.source
            .seek(SeekFrom::Start(0))
            .map_err(E::SeekHeader)?;

        self.source
            .read_exact(&mut header_bytes)
            .map_err(E::ReadHeader)?;

        WadHeader::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)
    }

    /// Read the wad entry list.
    ///
    /// Returns an [fallible iterator](https://docs.rs/fallible-iterator) that parses entries
    /// one-by-one. See [`WadAssetInfoReader`] for more details.
    pub fn read_entries<'a>(&mut self) -> Result<WadAssetInfoReader<'a>, WadReaderError> {
        use WadReaderError as E;

        let header = self.read_header()?;

        let entries_position = u64::try_from(header.entries_position)
            .map_err(|_| E::ParseHeader(WadHeaderSection::FirstEntryPosition))?;

        let entry_count = usize::try_from(header.entry_count)
            .map_err(|_| E::ParseHeader(WadHeaderSection::EntryCount))?;

        let mut entries_bytes = Vec::new();

        self.source
            .seek(SeekFrom::Start(entries_position))
            .map_err(E::SeekEntries)?;

        self.source
            .read_to_end(&mut entries_bytes)
            .map_err(E::ReadEntries)?;

        Ok(WadAssetInfoReader::new(entries_bytes.into(), entry_count))
    }

    /// Read the file content of an entry.
    ///
    /// *Important:* The entry must originate from the same wad file and it shouldn't be invalidated
    /// by writing to the wad file inbetween reading entries and reading file contents. Failing to
    /// do this can return corrupted data.
    ///
    /// Entries can be obtained with [`WadReader::read_entries`].
    pub fn read_content(&mut self, entry: &WadAssetInfo) -> Result<Vec<u8>, WadReaderError> {
        use WadReaderError as E;

        let content_position = u64::try_from(entry.content_position)
            .map_err(|_| E::ParseEntry(WadAssetInfoSection::ContentPosition))?;

        let content_length = usize::try_from(entry.content_length)
            .map_err(|_| E::ParseEntry(WadAssetInfoSection::ContentLength))?;

        let mut content_bytes = vec![0; content_length];

        self.source
            .seek(SeekFrom::Start(content_position))
            .map_err(E::SeekContents)?;

        self.source
            .read_exact(&mut content_bytes)
            .map_err(E::ReadContents)?;

        Ok(content_bytes)
    }
}

#[derive(Debug, Display, Error)]
pub enum WadReaderError {
    ParseHeader(WadHeaderSection),
    SeekHeader(io::Error),
    ReadHeader(io::Error),
    ParseEntry(WadAssetInfoSection),
    SeekEntries(io::Error),
    ReadEntries(io::Error),
    SeekContents(io::Error),
    ReadContents(io::Error),
}

pub struct WadAssetInfoReader<'a> {
    bytes: Cow<'a, [u8]>,
    position: usize,
    entries_left: usize,
}

impl<'a> WadAssetInfoReader<'a> {
    fn new(bytes: Cow<'a, [u8]>, entry_count: usize) -> Self {
        Self {
            bytes,
            position: 0,
            entries_left: entry_count,
        }
    }

    pub fn into_owned(self) -> WadAssetInfoReader<'static> {
        WadAssetInfoReader {
            bytes: Cow::Owned(self.bytes.into_owned()),
            position: self.position,
            entries_left: self.entries_left,
        }
    }

    pub fn into_iterator(self) -> fallible_iterator::Iterator<Self> {
        self.iterator()
    }
}

impl<'a> FallibleIterator for WadAssetInfoReader<'a> {
    type Item = WadAssetInfo<'static>;
    type Error = WadAssetInfoSection;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.entries_left > 0 {
            let mut entry_bytes = &self.bytes[self.position..];
            let entry = WadAssetInfo::parse(&mut entry_bytes)?;
            self.position += entry.byte_size();
            self.entries_left -= 1;
            Ok(Some(entry.into_owned()))
        } else {
            Ok(None)
        }
    }
}

pub struct WadWriter<Sink: Write + Seek> {
    sink: Sink,
}

impl<Sink: Write + Seek> WadWriter<Sink> {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }

    pub fn write_header<S: Write + Seek>(
        &mut self,
        header: &WadHeader,
    ) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut header_bytes = [0; WadHeader::BYTE_SIZE];

        header
            .serialize(&mut &mut header_bytes[..])
            .map_err(E::SerializeHeader)?;

        self.sink.seek(SeekFrom::Start(0)).map_err(E::SeekHeader)?;
        self.sink.write_all(&header_bytes).map_err(E::WriteHeader)?;

        Ok(())
    }

    pub fn write_content(
        &mut self,
        content: &[u8],
        block_size: usize,
    ) -> Result<usize, WadWriterError> {
        use WadWriterError as E;

        let padding_amount = (block_size - (content.len() % block_size)) % block_size;
        let padding = vec![0; padding_amount];

        self.sink.write_all(&content).map_err(E::WriteContent)?;
        self.sink.write_all(&padding).map_err(E::WriteContent)?;

        let written = content.len() + padding_amount;

        Ok(written)
    }

    pub fn write_entries(&mut self, entries: &[WadAssetInfo]) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut entries_bytes = Vec::new();

        for entry in entries {
            let entries_bytes_end = entries_bytes.len();

            entries_bytes.resize(entries_bytes_end + entry.byte_size(), 0);

            let entry_bytes = &mut &mut entries_bytes[entries_bytes_end..];

            entry
                .serialize(&mut &mut entry_bytes[..])
                .map_err(E::SerializeEntry)?;
        }

        self.sink.seek(SeekFrom::End(0)).map_err(E::SeekEntries)?;
        self.sink
            .write_all(&entries_bytes)
            .map_err(E::WriteEntries)?;

        Ok(())
    }
}

#[derive(Error, Debug, Display)]
pub enum WadWriterError {
    SerializeHeader(WadHeaderSection),
    SeekHeader(io::Error),
    WriteHeader(io::Error),
    SerializeEntry(WadAssetInfoSection),
    SeekEntries(io::Error),
    WriteEntries(io::Error),
    WriteContent(io::Error),
}
