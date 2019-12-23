pub mod decode;
pub mod encode;

use chrono::naive::NaiveDateTime;
use nom::multi::count;
use std::fs::{File,create_dir_all};
use std::io::{SeekFrom,Seek,Read,Write,Error,ErrorKind};
use std::collections::{HashMap,HashSet};
use std::path::Path;
use std::convert::TryInto;

use self::decode::{decode_header,decode_entry};
// use self::encode::{encode_header,encode_entry};

#[derive(Debug)]
pub struct Wad {
    pub file: File,
    pub header: WadHeader,
    pub entries: Vec<WadEntry>,
}

#[derive(Debug,PartialEq,Default)]
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

impl Wad {
    pub fn create(file_path: &str) -> Result<Wad, Error> {
        Ok(
            Wad {
                file: File::create(file_path)?,
                header: WadHeader::default(),
                entries: Vec::new(),
            }
        )
    }

    pub fn open(file_path: &str) -> Result<Wad, Error> {
        Self::from_file(File::open(file_path)?)
    }

    pub fn from_file(mut file: File) -> Result<Wad, Error> {
        let header = Self::read_header(&mut file)?;
        let entries = Self::read_entries(&mut file, &header)?;

        Ok(
            Wad {
                file: file,
                header: header,
                entries: entries,
            }
        )
    }

    pub fn read_header(mut file: &File) -> Result<WadHeader, Error> {
        let mut header_data: [u8; 32] = [0; 32];

        file.read_exact(&mut header_data[..])?;

        let (_, header) = match decode_header(&header_data[..]) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "header is invalid"))
        };

        Ok(header)
    }

    pub fn read_entries(mut file: &File, header: &WadHeader) -> Result<Vec<WadEntry>, Error> {
        let mut entries_data = Vec::new();

        file.seek(SeekFrom::Start(header.entries_offset as u64))?;
        file.read_to_end(&mut entries_data)?;

        let (_, entries) = match count(decode_entry, header.entries_count as usize)(&entries_data) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "entries is invalid"))
        };

        Ok(entries)
    }

    pub fn unpack(&self, output: &str, file_options: HashMap<String, FileOption>) -> Result<u32, WriteError> {
        let mut directories_created: HashSet<String> = HashSet::new();

        let mut file = &self.file;
        let mut files_written = 0;

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

            // TODO: Set file metadata.

            output_file.write_all(file_data.as_slice())?;

            files_written += 1;
        }

        Ok(files_written)
    }
}