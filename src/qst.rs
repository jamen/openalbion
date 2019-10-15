use nom::IResult;
use nom::multi::{many1,many0};
use nom::character::complete::line_ending;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use crate::format::shared::script::{parse_instr_value_call,InstrValue};

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<InstrValue>
}

pub fn parse_qst(input: &[u8]) -> IResult<&[u8], Qst> {
    let (maybe_input, body) = many1(parse_call)(input)?;
    Ok((maybe_input, Qst { body: body }))
}

pub fn parse_call(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, _ln) = opt(many0(line_ending))(input)?;
    let (maybe_input, call) = parse_instr_value_call(maybe_input)?;
    let (maybe_input, _semi) = tag(";")(maybe_input)?;
    let (maybe_input, _ln) = many1(line_ending)(maybe_input)?;

    Ok((maybe_input, call))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_qst() {
        let file_path = concat!(env!("FABLE"), "/data/Levels/FinalAlbion.qst");
        let mut file = File::open(file_path).expect("Failed to open file.");

        let mut qst: Vec<u8> = Vec::new();

        file.read_to_end(&mut qst).expect("Failed to read file.");

        let (left, qst) = match parse_qst(&qst) {
            Ok(x) => x,
            Err(nom::Err::Error((_input, error))) => return println!("Error {:?}", error),
            Err(error) => return println!("Error {:?}", error),
        };

        println!("{:#?}", qst);

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