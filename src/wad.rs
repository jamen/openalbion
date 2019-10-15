use chrono::naive::NaiveDateTime;
use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{tag,take};
use nom::sequence::tuple;
use nom::multi::count;
use std::fs::{File,create_dir_all};
use std::io::{SeekFrom,Seek,Read,Write,Error,ErrorKind};
use std::collections::{HashMap,HashSet};
use std::path::Path;
use std::convert::TryInto;
use crate::format::shared::timestamp::{parse_timestamp, parse_short_timestamp};

#[derive(Debug)]
pub struct Wad<'a> {
    pub file: &'a File,
    pub header: WadHeader,
    pub entries: Vec<WadEntry>,
}

#[derive(Debug,PartialEq)]
pub struct WadHeader {
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entries_count: u32,
    pub entries_offset: u32,
}

#[derive(Debug,PartialEq)]
pub struct WadEntry {
    pub id: u32,
    pub offset: u32,
    pub length: u32,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub accessed_at: NaiveDateTime,
    pub written_at: NaiveDateTime,
}

#[derive(Debug,PartialEq)]
pub enum FileOption {
    Include,
    Exclude,
}

#[derive(Debug)]
pub enum WriteError {
    Error(std::io::Error),
    Infallible(std::convert::Infallible)
}

impl From<std::io::Error> for WriteError {
    fn from(error: std::io::Error) -> Self {
        WriteError::Error(error)
    }
}

impl From<std::convert::Infallible> for WriteError {
    fn from(error: std::convert::Infallible) -> Self {
        WriteError::Infallible(error)
    }
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], WadHeader> {
    let (input, _magic_number) = tag("BBBB")(input)?;
    let (input, version) = tuple((le_u32, le_u32, le_u32))(input)?;
    let (input, block_size) = le_u32(input)?;
    let (input, entries_count) = le_u32(input)?;
    let (input, _entries_count_again) = le_u32(input)?;
    let (input, entries_offset) = le_u32(input)?;

    Ok(
        (
            input,
            WadHeader {
                version: version,
                block_size: block_size,
                entries_count: entries_count,
                entries_offset: entries_offset,
            }
        )
    )
}

pub fn parse_entry(input: &[u8]) -> IResult<&[u8], WadEntry> {
    let (input, _unknown_1) = take(16usize)(input)?;
    let (input, id) = le_u32(input)?;
    let (input, _unknown_2) = le_u32(input)?;
    let (input, length) = le_u32(input)?;
    let (input, offset) = le_u32(input)?;
    let (input, _unknown_3) = le_u32(input)?;
    let (input, path_length) = le_u32(input)?;
    let (input, path) = take(path_length as usize)(input)?;

    let path = match std::str::from_utf8(path) {
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
        Ok(value) => value.to_owned()
    };

    // println!("id {:?}, length {:?}, offset {:?}, path {:?}", id, length, offset, path);

    let (input, _unknown_4) = take(16usize)(input)?;

    // TODO: Create `parse_timestamp() -> NaiveDateTime` parsers for these timestamps
    let (input, created_at) = parse_timestamp(input)?;
    let (input, accessed_at) = parse_timestamp(input)?;
    let (input, written_at) = parse_short_timestamp(input)?;

    Ok(
        (
            input,
            WadEntry {
                id: id,
                length: length,
                offset: offset,
                path: path,
                created_at: created_at,
                accessed_at: accessed_at,
                written_at: written_at,
            }
        )
    )
}

impl Wad<'_> {
    pub fn new(file: &mut File) -> Result<Wad, Error> {
        // Get header values
        let mut header_data: [u8; 32] = [0; 32];

        file.read_exact(&mut header_data[..])?;

        let (_, header) = match parse_header(&header_data[..]) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "header is invalid"))
        };

        // println!("header {:?}", header);

        // Get entries
        let mut entries_data: Vec<u8> = Vec::new();

        // println!("entries_offset {:?}", header.entries_offset);

        file.seek(SeekFrom::Start(header.entries_offset as u64))?;
        file.read_to_end(&mut entries_data)?;

        let (_, entries) = match count(parse_entry, header.entries_count as usize)(&entries_data) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "entries is invalid"))
        };

        // Create a map of paths -> entries
        // let mut parser_iter = iterator(&entries_data[..], parse_entry);
        // let entries = parser_iter
        //     .map(|x| (x.path.clone(), x))
        //     .collect::<HashMap<String, WadEntry>>();

        // for entry in parse_iter {
        //     entries.insert(entry.path.clone(), entry);
        // }

        // match parser_iter.finish() {
        //     Err(nom::Err::Incomplete(_needed)) =>
        //         return Err(Error::new(ErrorKind::InvalidData, "incomplete")),
        //     Err(nom::Err::Failure((_input, error))) |
        //     Err(nom::Err::Error((_input, error))) =>
        //         return Err(Error::new(ErrorKind::InvalidData, error.description())),
        //     Ok((_, ())) => {},
        // }

        // println!("entries {:?}", entries);

        Ok(
            Wad {
                file: file,
                header: header,
                entries: entries,
            }
        )
    }

    pub fn copy(&self, output: String, file_options: HashMap<String, FileOption>) -> Result<(), WriteError> {
        let mut directories_created: HashSet<String> = HashSet::new();

        let mut file = self.file;

        for entry in &self.entries {
            // println!("entry {:?}", entry);

            match file_options.get(&entry.path.clone()) {
                Some(FileOption::Exclude) => continue,
                Some(FileOption::Include) => {},
                None => {},
            }

            let file_path_buf = Path::new(&output).join(entry.path.to_string());

            let file_path = file_path_buf.as_path();
            let file_path = file_path.as_os_str();
            let file_path = file_path.to_str().unwrap();
            let file_path = file_path.to_owned();

            let file_directory = file_path_buf.parent().unwrap();
            let file_directory = file_directory.as_os_str();
            let file_directory = file_directory.to_str().unwrap();
            let file_directory = file_directory.to_owned();

            let mut file_data: Vec<u8> = Vec::new();

            file.seek(SeekFrom::Start(entry.offset.try_into()?))?;
            file.take(entry.length.try_into()?).read_to_end(&mut file_data)?;

            if !directories_created.contains(&file_directory) {
                create_dir_all(&file_directory)?;
                directories_created.insert(file_directory.to_string());
            }

            let mut output_file = File::create(file_path)?;

            output_file.write_all(file_data.as_slice())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_wad() {

    }
}