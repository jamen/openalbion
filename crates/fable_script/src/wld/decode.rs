use fable_base::nom::IResult;
use fable_base::nom::multi::{many1,many_till};

use crate::shared::Instr;
use crate::shared::decode::{decode_instr,decode_instr_tag};

use crate::wld::{
    WldMap,
    WldRegion,
    Wld,
};

pub fn decode_wld(input: &[u8]) -> IResult<&[u8], Wld> {
    let (input, start_initial_quests) = decode_wld_initial_quests(input)?;
    let (input, map_uid_count) = decode_instr_tag("MapUIDCount")(input)?;
    let (input, thing_manager_uid_count) = decode_instr_tag("ThingManagerUIDCount")(input)?;
    let (input, maps) = many1(decode_wld_map)(input)?;
    let (input, regions) = many1(decode_wld_region)(input)?;

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

pub fn decode_wld_initial_quests(input: &[u8]) -> IResult<&[u8], Vec<Instr>> {
    let (input, _start) = decode_instr_tag("START_INITIAL_QUESTS")(input)?;
    let (input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag("END_INITIAL_QUESTS"))(input)?;

    Ok(
        (
            input,
            instrs,
        )
    )
}

pub fn decode_wld_map(input: &[u8]) -> IResult<&[u8], WldMap> {
    let (input, new_map) = decode_instr_tag("NewMap")(input)?;
    let (input, (instrs, _end_instr)) = many_till(decode_instr, decode_instr_tag("EndMap"))(input)?;

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

pub fn decode_wld_region(input: &[u8]) -> IResult<&[u8], WldRegion> {
    let (input, new_region) = decode_instr_tag("NewRegion")(input)?;
    let (input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag("EndRegion"))(input)?;

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