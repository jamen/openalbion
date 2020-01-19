use std::io::Read;

use nom::IResult;
use nom::multi::{many1,many_till};
use nom::combinator::all_consuming;

use crate::shared::{Decode,Error};
use crate::shared::script::Instr;
use crate::shared::script::decode::{decode_instr,decode_instr_tag};

use super::{WldMap, WldRegion, Wld};

static MAPUICOUNT: &'static str = "MapUIDCount";
static THINGMANAGERUIDCOUNT: &'static str = "ThingManagerUIDCount";
static START_INITIAL_QUESTS: &'static str = "START_INITIAL_QUESTS";
static END_INITIAL_QUESTS: &'static str = "END_INITIAL_QUESTS";
static NEWMAP: &'static str = "NewMap";
static ENDMAP: &'static str = "EndMap";
static NEWREGION: &'static str = "NewRegion";
static ENDREGION: &'static str = "EndRegion";

impl Decode for Wld {
    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, wld) = all_consuming(Self::decode_wld)(&input)?;
        Ok(wld)
    }
}

impl Wld {
    pub fn decode_wld(input: &[u8]) -> IResult<&[u8], Wld, Error> {
        let (input, start_initial_quests) = Self::decode_wld_initial_quests(input)?;
        let (input, map_uid_count) = decode_instr_tag(MAPUICOUNT)(input)?;
        let (input, thing_manager_uid_count) = decode_instr_tag(THINGMANAGERUIDCOUNT)(input)?;
        let (input, maps) = many1(Self::decode_wld_map)(input)?;
        let (input, regions) = many1(Self::decode_wld_region)(input)?;

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

    pub fn decode_wld_initial_quests(input: &[u8]) -> IResult<&[u8], Vec<Instr>, Error> {
        let (input, _start) = decode_instr_tag(START_INITIAL_QUESTS)(input)?;
        let (input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag(END_INITIAL_QUESTS))(input)?;

        Ok(
            (
                input,
                instrs,
            )
        )
    }

    pub fn decode_wld_map(input: &[u8]) -> IResult<&[u8], WldMap, Error> {
        let (input, new_map) = decode_instr_tag(NEWMAP)(input)?;
        let (input, (instrs, _end_instr)) = many_till(decode_instr, decode_instr_tag(ENDMAP))(input)?;

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

    pub fn decode_wld_region(input: &[u8]) -> IResult<&[u8], WldRegion, Error> {
        let (input, new_region) = decode_instr_tag(NEWREGION)(input)?;
        let (input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag(ENDREGION))(input)?;

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
}
