use nom::IResult;
use nom::multi::{many1,many_till};
use crate::shared::script::{parse_instr,parse_instr_tag,Instr};

#[derive(Debug,PartialEq)]
pub struct Wld {
    start_initial_quests: Vec<Instr>,
    map_uid_count: Instr,
    thing_manager_uid_count: Instr,
    maps: Vec<WldMap>,
    regions: Vec<WldRegion>,
}

#[derive(Debug,PartialEq)]
pub struct WldMap {
    new_map: Instr,
    instrs: Vec<Instr>,
}

#[derive(Debug,PartialEq)]
pub struct WldRegion {
    new_region: Instr,
    instrs: Vec<Instr>,
}

pub fn parse_wld_map(input: &[u8]) -> IResult<&[u8], WldMap> {
    let (input, new_map) = parse_instr_tag("NewMap".to_string())(input)?;
    let (input, (instrs, _end_instr)) = many_till(parse_instr, parse_instr_tag("EndMap".to_string()))(input)?;
    Ok(
        (
            input,
            WldMap {
                new_map: new_map,
                instrs: instrs,
            }
        )
    )
}

pub fn parse_wld_region(input: &[u8]) -> IResult<&[u8], WldRegion> {
    let (input, new_region) = parse_instr_tag("NewRegion".to_string())(input)?;
    let (input, (instrs, _end)) = many_till(parse_instr, parse_instr_tag("EndRegion".to_string()))(input)?;
    Ok(
        (
            input,
            WldRegion {
                new_region: new_region,
                instrs: instrs,
            }
        )
    )
}

pub fn parse_wld_initial_quests(input: &[u8]) -> IResult<&[u8], Vec<Instr>> {
    let (input, _start) = parse_instr_tag("START_INITIAL_QUESTS".to_string())(input)?;
    let (input, (instrs, _end)) = many_till(parse_instr, parse_instr_tag("END_INITIAL_QUESTS".to_string()))(input)?;
    Ok(
        (
            input,
            instrs,
        )
    )
}

pub fn parse_wld(input: &[u8]) -> IResult<&[u8], Wld> {
    let (input, start_initial_quests) = parse_wld_initial_quests(input)?;
    let (input, map_uid_count) = parse_instr_tag("MapUIDCount".to_string())(input)?;
    let (input, thing_manager_uid_count) = parse_instr_tag("ThingManagerUIDCount".to_string())(input)?;
    let (input, maps) = many1(parse_wld_map)(input)?;
    let (input, regions) = many1(parse_wld_region)(input)?;
    Ok(
        (
            input,
            Wld {
                start_initial_quests: start_initial_quests,
                map_uid_count: map_uid_count,
                thing_manager_uid_count: thing_manager_uid_count,
                maps: maps,
                regions: regions,
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
    fn test_wld() {
        let file_path = concat!(env!("FABLE"), "/data/Levels/FinalAlbion.wld");
        let mut file = File::open(file_path).expect("Failed to open file.");

        let mut wld: Vec<u8> = Vec::new();

        file.read_to_end(&mut wld).expect("Failed to read file.");

       let (left, wld) = match parse_wld(&wld) {
            Ok(x) => x,
            Err(nom::Err::Error((_input, error))) => return println!("Error {:?}", error),
            Err(error) => return println!("Error {:?}", error),
        };

        println!("{:#?}", wld);

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