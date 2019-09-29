use nom::IResult;
use nom::character::complete::line_ending;
use nom::multi::{many0,many_till};
use crate::shared::script::{parse_instr,parse_instr_tag,Instr};

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: Instr,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: Instr,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: Instr,
    pub instrs: Vec<Instr>
    // TODO: Parse instrs more thoroughly into fields.
}

pub fn parse_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing> {
    let (maybe_input, new_thing) = parse_instr_tag("NewThing".to_string())(input)?;
    let (maybe_input, (instrs, _end)) = many_till(parse_instr, parse_instr_tag("EndThing".to_string()))(maybe_input)?;
    Ok(
        (
            maybe_input,
            TngThing {
                new_thing: new_thing,
                instrs: instrs,
            }
        )
    )
}

pub fn parse_tng_section(input: &[u8]) -> IResult<&[u8], TngSection> {
    let (maybe_input, section_start) = parse_instr_tag("XXXSectionStart".to_string())(input)?;
    let (maybe_input, (things, _end)) = many_till(parse_tng_thing, parse_instr_tag("XXXSectionEnd".to_string()))(maybe_input)?;
    Ok(
        (
            maybe_input,
            TngSection {
                section_start: section_start,
                things: things,
            }
        )
    )
}

pub fn parse_tng(input: &[u8]) -> IResult<&[u8], Tng> {
    let (maybe_input, version) = parse_instr_tag("Version".to_string())(input)?;
    let (maybe_input, sections) = many0(parse_tng_section)(maybe_input)?;
    Ok(
        (
            maybe_input,
            Tng {
                version: version,
                sections: sections,
            }
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_tng() {
        let file_path = concat!(env!("FABLE"), "/data/Levels/FinalAlbion/LookoutPoint.tng");
        let mut file = File::open(file_path).expect("Failed to open file.");

        let mut tng: Vec<u8> = Vec::new();

        file.read_to_end(&mut tng).expect("Failed to read file.");

        let (left, tng) = parse_tng(&tng).expect("Failed to parse.");

        println!("{:#?}", tng);

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