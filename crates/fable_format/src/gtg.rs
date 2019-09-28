use nom::IResult;
use nom::character::complete::line_ending;
use nom::combinator::opt;
use nom::bytes::complete::tag;
use nom::multi::{many0,many1};
use crate::tng::{parse_tng,Tng};
use crate::shared::script::{parse_instr_tag,parse_instr_value,parse_instr_key,Instr,InstrKey};

#[derive(Debug,PartialEq)]
pub struct Gtg {
    maps: Vec<Tng>
}

pub fn parse_gtg_map(input: &[u8]) -> IResult<&[u8], Tng> {
    let (maybe_input, _start) = parse_gtg_map_instr("NEWMAP".to_string())(input)?;
    let (maybe_input, tng) = parse_tng(maybe_input)?;
    let (maybe_input, _end) = parse_gtg_map_instr("ENDMAP".to_string())(maybe_input)?;
    Ok((maybe_input, tng))
}

// "NEWMAP" and "ENDMAP" don't use semicolons, so this is an alternative of parser::util::parse_instr_tag
pub fn parse_gtg_map_instr(name: String) -> impl Fn(&[u8]) -> IResult<&[u8], Instr> {
    move |input: &[u8]| {
        let (maybe_input, _line_ending) = many0(line_ending)(input)?;
        let (maybe_input, key) = parse_instr_key(maybe_input)?;
        let (maybe_input, _space) = opt(tag(" "))(maybe_input)?;
        let (maybe_input, value) = parse_instr_value(maybe_input)?;
        let (maybe_input, _line_ending) = many1(line_ending)(maybe_input)?;

       let key_string = match key {
            InstrKey::Name(x) => x,
            InstrKey::Index(_) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
            InstrKey::Property(_) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
        };

        // println!("{:?} == {:?}", name, key);

        if key_string != name {
            return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)));
        }

        Ok((maybe_input, (InstrKey::Name(key_string), value)))
    }
}

pub fn parse_gtg(input: &[u8]) -> IResult<&[u8], Gtg> {
    let (maybe_input, maps) = many1(parse_gtg_map)(input)?;
    Ok((maybe_input, Gtg { maps: maps }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_gtg() {
        let file_path = concat!(env!("FABLE"), "/data/Levels/FinalAlbion.gtg");
        let mut file = File::open(file_path).expect("Failed to open file.");

        let mut gtg: Vec<u8> = Vec::new();

        file.read_to_end(&mut gtg).expect("Failed to read file.");

        let (left, gtg) = match parse_gtg(&gtg) {
            Ok(x) => x,
            Err(nom::Err::Error((_input, error))) => return println!("Error {:?}", error),
            Err(error) => return println!("Error {:?}", error),
        };

        println!("{:#?}", gtg);

        // let mut bank_index: Vec<u8> = Vec::new();
        // file.seek(SeekFrom::Start(big_header.bank_address as u64)).expect("Failed to seek file.");
        // file.read_to_end(&mut bank_index).expect("Failed to read file.");

        // let (_, big_bank_index) = parse_bank_index(&bank_index).expect("Failed to parse bank index.");

        // println!("{:?}", big_bank_index);

        // let mut file_index: Vec<u8> = Vec::new();
        // file.seek(SeekFrom::Start(big_bank_index.index_start as u64)).expect("Failed to seek file.");
        // file.take(big_bank_index.index_size as u64).read_to_end(&mut file_index).expect("Failed to read file.");
        // file.read_to_end(&mut file_index).expect("Failed to read file.");

        // let (_, big_file_index) = match parse_file_index(&file_index) {
        //     Ok(value) => value,
        //     Err(nom::Err::Error((_, error))) => return println!("Error {:?}", error),
        //     Err(nom::Err::Failure((_, error))) => return println!("Error {:?}", error),
        //     Err(_) => return println!("Error"),
        // };

        // println!("{:#?}", big_file_index);
    }
}