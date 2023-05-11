use std::io::Read;

use crate::Error;

use crate::{
    Field,
    IResult,
    Index,
    Value,
    all_consuming,
    decode_field,
    decode_field_tagged,
    many0,
    many_till,
    multispace0,
};

#[derive(Debug,PartialEq)]
pub struct Wld {
    pub start_initial_quests: Vec<String>,
    pub map_uid_count: isize,
    pub thing_manager_uid_count: isize,
    pub maps: Vec<WldMap>,
    pub regions: Vec<WldRegion>,
}

#[derive(Debug,PartialEq,Default)]
pub struct WldMap {
    pub new_map: isize,
    pub map_x: isize,
    pub map_y: isize,
    pub level_name: String,
    pub level_script_name: String,
    pub map_uid: isize,
    pub is_sea: bool,
    pub loaded_on_player_proximity: bool,
}

#[derive(Debug,PartialEq,Default)]
pub struct WldRegion {
    pub new_region: isize,
    pub region_name: String,
    pub new_display_name: String,
    pub region_def: String,
    pub appear_on_world_map: bool,
    pub mini_map_graphic: Option<String>,
    pub mini_map_scale: f32,
    pub mini_map_offset_x: f32,
    pub mini_map_offset_y: f32,
    pub world_map_offset_x: f32,
    pub world_map_offset_y: f32,
    pub name_graphic_offset_x: f32,
    pub name_graphic_offset_y: f32,
    pub mini_map_region_exit_text_offset_x: Vec<(String, f32)>,
    pub mini_map_region_exit_text_offset_y: Vec<(String, f32)>,
    pub contains_map: Vec<String>,
    pub sees_map: Vec<String>,
}

impl Wld {
    pub fn decode<Source: Read>(source: &mut Source) -> Result<Wld, Error> {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, wld) = all_consuming(Self::decode_wld)(&input)?;
        Ok(wld)
    }

    fn decode_wld(input: &[u8]) -> IResult<&[u8], Wld, Error> {
        let (input, start_initial_quests) = Self::decode_start_initial_quests(&input)?;
        let (input, _) = multispace0(input)?;
        let (input, map_uid_count) = Self::decode_map_uid_count(&input)?;
        let (input, _) = multispace0(input)?;
        let (input, thing_manager_uid_count) = Self::decode_thing_manager_uid_count(&input)?;
        let (input, _) = multispace0(input)?;
        let (input, maps) = Self::decode_maps(&input)?;
        let (input, _) = multispace0(input)?;
        let (input, regions) = Self::decode_regions(&input)?;
        let (input, _) = multispace0(input)?;

        Ok(
            (
                input,
                Wld {
                    start_initial_quests,
                    map_uid_count,
                    thing_manager_uid_count,
                    maps,
                    regions,
                }
            )
        )
    }

    fn decode_raw_section<'a>(start: &'a str, end: &'a str) -> impl Fn(&'a [u8]) -> IResult<&[u8], (Field, Vec<Field>, Field), Error> {
        move |input: &[u8]| {
            let (input, start_field) = decode_field_tagged(start)(input)?;
            let (input, (fields, end_field)) = many_till(decode_field, decode_field_tagged(end))(input)?;
            Ok((input, (start_field, fields, end_field)))
        }
    }

    fn decode_start_initial_quests(input: &[u8]) -> IResult<&[u8], Vec<String>, Error> {
        let (input, (start_initial_quests, fields, end_initial_quests)) =
            Self::decode_raw_section("START_INITIAL_QUESTS", "END_INITIAL_QUESTS")(input)?;

        if let Value::Empty = start_initial_quests.value {
            return Err(nom::Err::Error(Error::InvalidValue))
        }

        if let Value::Empty = start_initial_quests.value {
            return Err(nom::Err::Error(Error::InvalidValue))
        }

        let start_initial_quests: Vec<String> = fields.into_iter().map(|x| x.name).collect();

        let _ = match end_initial_quests.value {
            Value::Empty => (),
            _ => return Err(nom::Err::Error(Error::InvalidValue)),
        };

        Ok((input, start_initial_quests))
    }

    fn decode_map_uid_count(input: &[u8]) -> IResult<&[u8], isize, Error> {
        let (input, field) = decode_field_tagged("MapUIDCount")(input)?;

        match field.value {
            Value::Integer(n) => Ok((input, n)),
            _ => Err(nom::Err::Error(Error::InvalidValue))
        }
    }

    fn decode_thing_manager_uid_count(input: &[u8]) -> IResult<&[u8], isize, Error> {
        let (input, field) = decode_field_tagged("ThingManagerUIDCount")(input)?;

        match field.value {
            Value::Integer(n) => Ok((input, n)),
            _ => Err(nom::Err::Error(Error::InvalidValue))
        }
    }

    fn decode_maps(input: &[u8]) -> IResult<&[u8], Vec<WldMap>, Error> {
        many0(Self::decode_map)(input)
    }

    fn decode_map(input: &[u8]) -> IResult<&[u8], WldMap, Error> {
        let (input, (new_map, fields, end_map)) = Self::decode_raw_section("NewMap", "EndMap")(input)?;

        let new_map = match new_map.value {
            Value::Integer(x) => x,
            _ => return Err(nom::Err::Error(Error::InvalidValue)),
        };

        let map_x = match fields.iter().find(|x| x.name == "MapX") {
            Some(Field { value: Value::Integer(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let map_y = match fields.iter().find(|x| x.name == "MapY") {
            Some(Field { value: Value::Integer(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let level_name = match fields.iter().find(|x| x.name == "LevelName") {
            Some(Field { value: Value::String(x), .. }) => x.clone(),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let level_script_name = match fields.iter().find(|x| x.name == "LevelScriptName") {
            Some(Field { value: Value::String(x), .. }) => x.clone(),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let map_uid = match fields.iter().find(|x| x.name == "MapUID") {
            Some(Field { value: Value::Integer(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let is_sea = match fields.iter().find(|x| x.name == "IsSea") {
            Some(Field { value: Value::Bool(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let loaded_on_player_proximity = match fields.iter().find(|x| x.name == "LoadedOnPlayerProximity") {
            Some(Field { value: Value::Bool(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let _ = match end_map.value {
            Value::Empty => (),
            _ => return Err(nom::Err::Error(Error::InvalidValue)),
        };

        Ok(
            (
                input,
                WldMap {
                    new_map,
                    map_x,
                    map_y,
                    level_name,
                    level_script_name,
                    map_uid,
                    is_sea,
                    loaded_on_player_proximity,
                }
            )
        )
    }

    fn decode_regions(input: &[u8]) -> IResult<&[u8], Vec<WldRegion>, Error> {
        many0(Self::decode_region)(input)
    }

    fn decode_region(input: &[u8]) -> IResult<&[u8], WldRegion, Error> {
        let (input, (new_region, fields, end_region)) = Self::decode_raw_section("NewRegion", "EndRegion")(input)?;

        let new_region = match new_region.value {
            Value::Integer(x) => x,
            _ => return Err(nom::Err::Error(Error::InvalidValue)),
        };

        let region_name = match fields.iter().find(|f| f.name == "RegionName") {
            Some(Field { value: Value::String(x), .. }) => x.clone(),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField)),
        };

        let new_display_name = match fields.iter().find(|f| f.name == "NewDisplayName") {
            Some(Field { value: Value::String(x), .. }) => x.clone(),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField)),
        };

        let region_def = match fields.iter().find(|f| f.name == "RegionDef") {
            Some(Field { value: Value::String(x), .. }) => x.clone(),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField)),
        };

        let appear_on_world_map = match fields.iter().find(|f| f.name == "AppearOnWorldMap") {
            Some(Field { value: Value::Empty, .. }) => true,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => false,
        };

        let mini_map_graphic = match fields.iter().find(|f| f.name == "MiniMapGraphic") {
            Some(Field { value: Value::String(x), .. }) => Some(x.clone()),
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => None,
        };

        let mini_map_scale = match fields.iter().find(|f| f.name == "MiniMapScale") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let mini_map_offset_x = match fields.iter().find(|f| f.name == "MiniMapOffsetX") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let mini_map_offset_y = match fields.iter().find(|f| f.name == "MiniMapOffsetY") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let world_map_offset_x = match fields.iter().find(|f| f.name == "WorldMapOffsetX") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let world_map_offset_y = match fields.iter().find(|f| f.name == "WorldMapOffsetY") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let name_graphic_offset_x = match fields.iter().find(|f| f.name == "NameGraphicOffsetX") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let name_graphic_offset_y = match fields.iter().find(|f| f.name == "NameGraphicOffsetY") {
            Some(Field { value: Value::Float(x), .. }) => *x,
            Some(_) => return Err(nom::Err::Error(Error::InvalidValue)),
            None => return Err(nom::Err::Error(Error::MissingRequiredField))
        };

        let mini_map_region_exit_text_offset_x: Vec<(String, f32)> = fields
            .iter()
            .filter_map(|field| {
                match (field.name.as_str(), field.indices.as_slice(), &field.value) {
                    ("MiniMapRegionExitTextOffsetX", [ Index::Box(Value::Name(name)) ], Value::Float(value)) => {
                        Some((name.clone(), value.clone()))
                    },
                    _ => None,
                }
            })
            .collect();

        let mini_map_region_exit_text_offset_y: Vec<(String, f32)> = fields
            .iter()
            .filter_map(|field| {
                match (field.name.as_str(), field.indices.as_slice(), &field.value) {
                    ("MiniMapRegionExitTextOffsetX", [ Index::Box(Value::Name(name)) ], Value::Float(value)) => {
                        Some((name.clone(), value.clone()))
                    },
                    _ => None,
                }
            })
            .collect();

        let contains_map = fields
            .iter()
            .filter_map(|field| {
                match (field.name.as_str(), field.indices.as_slice(), &field.value) {
                    ("ContainsMap", [], Value::String(value)) => Some(value.clone()),
                    _ => None,
                }
            })
            .collect();

        let sees_map = fields
            .iter()
            .filter_map(|field| {
                match (field.name.as_str(), field.indices.as_slice(), &field.value) {
                    ("SeesMap", [], Value::String(value)) => Some(value.clone()),
                    _ => None,
                }
            })
            .collect();

        let _ = match end_region.value {
            Value::Empty => (),
            _ => return Err(nom::Err::Error(Error::InvalidValue)),
        };

        Ok(
            (
                input,
                WldRegion {
                    new_region,
                    region_name,
                    new_display_name,
                    region_def,
                    appear_on_world_map,
                    mini_map_graphic,
                    mini_map_scale,
                    mini_map_offset_x,
                    mini_map_offset_y,
                    world_map_offset_x,
                    world_map_offset_y,
                    name_graphic_offset_x,
                    name_graphic_offset_y,
                    mini_map_region_exit_text_offset_x,
                    mini_map_region_exit_text_offset_y,
                    contains_map,
                    sees_map,
                }
            )
        )
    }
}