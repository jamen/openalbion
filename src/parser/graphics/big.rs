use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{tag,take,is_not};
use nom::multi::count;
use crate::parser::util::parse_rle_string;

#[derive(Debug,PartialEq)]
pub struct BigHeader {
    version: u32,
    bank_address: u32,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], BigHeader> {
    let (input, _magic_number) = tag("BIGB")(input)?;
    let (input, version) = le_u32(input)?;
    let (input, bank_address) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?;

    Ok(
        (
            input,
            BigHeader {
                version: version,
                bank_address: bank_address,
            }
        )
    )
}

#[derive(Debug,PartialEq)]
pub struct BigBankIndex {
    name: String,
    bank_id: u32,
    bank_entries_count: u32,
    index_start: u32,
    index_size: u32,
    block_size: u32,
}

pub fn parse_bank_index(input: &[u8]) -> IResult<&[u8], BigBankIndex> {
    let (input, _banks_count) = le_u32(input)?;
    let (input, name) = is_not("\0")(input)?;
    let (input, _zero) = tag("\0")(input)?;
    let (input, bank_id) = le_u32(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    };

    let (input, bank_entries_count) = le_u32(input)?;
    let (input, index_start) = le_u32(input)?;
    let (input, index_size) = le_u32(input)?;
    let (input, block_size) = le_u32(input)?;

    Ok(
        (
            input,
            BigBankIndex {
                name: name,
                bank_id: bank_id,
                bank_entries_count: bank_entries_count,
                index_start: index_start,
                index_size: index_size,
                block_size: block_size,
            }
        )
    )
}

#[derive(Debug,PartialEq)]
pub struct BigFileIndex {
    // file_types_count: u32,
    // file_type: u32,
    // entries_count: u32,
    entries: Vec<BigFileEntry>
}

pub fn parse_file_index(input: &[u8]) -> IResult<&[u8], BigFileIndex> {
    let (input, file_types_count) = le_u32(input)?;
    let (input, _file_type) = le_u32(input)?;
    let (input, entries_count) = le_u32(input)?;

    println!("file_types_count {:?}", file_types_count);
    // println!("file_type {:?}", file_type);
    println!("entries_count {:?}", entries_count);
    // let entries_count = 10;

    // Lots of integers not documented in fabletlcmod.com
    let (input, _unknown_1) = take(56usize)(input)?;

    let (input, entries) = count(parse_file_index_entry, entries_count as usize)(input)?;

    Ok(
        (
            input,
            BigFileIndex {
                // file_types_count: file_types_count,
                // file_type: file_type,
                entries: entries,
                // entries_count: entries_count,
            }
        )
    )
}

#[derive(Debug,PartialEq)]
pub struct BigFileEntry {
    magic_number: u32,
    id: u32,
    file_type: u32,
    size: u32,
    start: u32,
    type_dev: u32,
    symbol_name: String,
    crc: u32,
    files: Vec<String>,
    // sub_header: BigSubHeader,
    sub_header: Vec<u8>,

}

pub fn parse_file_index_entry(input: &[u8]) -> IResult<&[u8], BigFileEntry> {
    let (input, magic_number) = le_u32(input)?;
    let (input, id) = le_u32(input)?;
    let (input, file_type) = le_u32(input)?;
    let (input, size) = le_u32(input)?;
    let (input, start) = le_u32(input)?;
    let (input, type_dev) = le_u32(input)?;

    let (input, symbol_name_length) = le_u32(input)?;
    let (input, symbol_name) = take(symbol_name_length as usize)(input)?;

    let symbol_name = match String::from_utf8(symbol_name.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    };

    let (input, crc) = le_u32(input)?;

    let (input, files_count) = le_u32(input)?;
    let (input, files) = count(parse_rle_string, files_count as usize)(input)?;

    let (input, sub_header_size) = le_u32(input)?;
    let (input, sub_header) = take(sub_header_size as usize)(input)?;

    let sub_header = sub_header.to_vec();

    Ok(
        (
            input,
            BigFileEntry {
                magic_number: magic_number,
                id: id,
                file_type: file_type,
                size: size,
                start: start,
                files: files,
                type_dev: type_dev,
                symbol_name: symbol_name,
                crc: crc,
                sub_header: sub_header,
            }
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_big() {
        let mut file = File::open(concat!(env!("FABLE"), "/data/graphics/graphics.big")).expect("failed to open file.");

        let mut header: [u8; 16] = [0; 16];

        file.read(&mut header).expect("Failed to read file.");

        let (_, big_header) = parse_header(&header[..]).expect("Failed to parse header.");

        println!("{:?}", big_header);

        let mut bank_index: Vec<u8> = Vec::new();
        file.seek(SeekFrom::Start(big_header.bank_address as u64)).expect("Failed to seek file.");
        file.read_to_end(&mut bank_index).expect("Failed to read file.");

        let (_, big_bank_index) = parse_bank_index(&bank_index).expect("Failed to parse bank index.");

        println!("{:?}", big_bank_index);

        let mut file_index: Vec<u8> = Vec::new();
        file.seek(SeekFrom::Start(big_bank_index.index_start as u64)).expect("Failed to seek file.");
        file.take(big_bank_index.index_size as u64).read_to_end(&mut file_index).expect("Failed to read file.");
        // file.read_to_end(&mut file_index).expect("Failed to read file.");

        let (_, big_file_index) = match parse_file_index(&file_index) {
            Ok(value) => value,
            Err(nom::Err::Error((_, error))) => return println!("Error {:?}", error),
            Err(nom::Err::Failure((_, error))) => return println!("Error {:?}", error),
            Err(_) => return println!("Error"),
        };

        // println!("{:#?}", big_file_index);
    }
}