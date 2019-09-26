use nom::IResult;
use nom::multi::{many1,many_till};
use crate::parser::util::script::{parse_instr,parse_instr_tag,Instr};

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