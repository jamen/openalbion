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

impl Wad {
    pub fn from_file(file_path: &str) -> Result<Wad, Error> {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidInput, "file doesn't exist"))
        };

        // Get header values
        let mut header_data: [u8; 32] = [0; 32];

        file.read_exact(&mut header_data[..])?;

        let (_, header) = match decode_header(&header_data[..]) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "header is invalid"))
        };

        // println!("header {:?}", header);

        // Get entries
        let mut entries_data: Vec<u8> = Vec::new();

        // println!("entries_offset {:?}", header.entries_offset);

        file.seek(SeekFrom::Start(header.entries_offset as u64))?;
        file.read_to_end(&mut entries_data)?;

        let (_, entries) = match count(decode_entry, header.entries_count as usize)(&entries_data) {
            Ok(value) => value,
            Err(_error) => return Err(Error::new(ErrorKind::InvalidData, "entries is invalid"))
        };

        // Create a map of paths -> entries
        // let mut parser_iter = iterator(&entries_data[..], decode_entry);
        // let entries = parser_iter
        //     .map(|x| (x.path.clone(), x))
        //     .collect::<HashMap<String, WadEntry>>();

        // for entry in decode_iter {
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

    pub fn extract(&self, output: &str, file_options: HashMap<String, FileOption>) -> Result<(), WriteError> {
        let mut directories_created: HashSet<String> = HashSet::new();

        let mut file = &self.file;

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