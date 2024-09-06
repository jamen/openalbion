use crate::util::{
    kv::{InvalidPath, InvalidValue, Kv, KvError, KvField, KvValueKind, UnexpectedField},
    slice::TakeSliceExt,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Tng {
    sections: Vec<TngSection>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TngError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),

    #[error("version field on line {line_num} is an unsupported version")]
    UnsupportedVersion { line_num: usize },

    #[error(transparent)]
    Kv(#[from] KvError),

    #[error(transparent)]
    Section(#[from] TngSectionError),
}

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngError> {
        let kv = Kv::parse(source)?;
        let mut fields = &kv.fields[..];
        let mut sections = Vec::new();

        let (version_field, version) = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("Version")?
            .with_no_path()?
            .with_integer_value()?;

        let line_num = version_field.line_num;

        if version != 2 {
            Err(TngError::UnsupportedVersion { line_num })?
        }

        while !fields.is_empty() {
            sections.push(TngSection::parse(&mut fields)?);
        }

        Ok(Self { sections })
    }
}

#[derive(Clone, Debug)]
pub struct TngSection {
    name: String,
    things: Vec<TngThingItem>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngSectionError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),

    #[error(transparent)]
    Thing(#[from] TngThingItemError),
}

impl TngSection {
    fn parse(mut fields: &mut &[KvField]) -> Result<Self, TngSectionError> {
        let (_section_start_field, section_name) = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("XXXSectionStart")?
            .with_no_path()?
            .with_identifier_value()?;

        let section_name = section_name.to_owned();

        let mut things = Vec::new();

        loop {
            let field = fields
                .first()
                .ok_or_else(|| UnexpectedEnd)?
                .with_no_path()?;

            let line_num = field.line_num;

            match field.key.identifier {
                "NewThing" => things.push(TngThingItem::parse(&mut fields)?),
                "XXXSectionEnd" => {
                    let _ = field.with_no_value()?;
                    let _ = fields.grab_first();
                    break;
                }
                _ => Err(UnexpectedField { line_num })?,
            }
        }

        Ok(Self {
            name: section_name,
            things,
        })
    }
}

#[derive(Clone, Debug)]
pub enum TngThingItem {
    Thing(TngThing),
    Marker(TngMarker),
    Object(TngObject),
    HolySite(TngHolySite),
    Building(TngBuilding),
    Village(TngVillage),
    AICreature(TngAICreature),
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngThingItemError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),

    #[error("unexpected thing variant at {line_num}")]
    UnrecognizedThing { line_num: usize },

    #[error(transparent)]
    ThingItem(#[from] TngThingError),

    #[error(transparent)]
    Marker(#[from] TngMarkerError),

    #[error(transparent)]
    Object(#[from] TngObjectError),

    #[error(transparent)]
    HolySite(#[from] TngHolySiteError),

    #[error(transparent)]
    Building(#[from] TngBuildingError),

    #[error(transparent)]
    Village(#[from] TngVillageError),

    #[error(transparent)]
    AICreature(#[from] TngAICreatureError),
}

impl TngThingItem {
    fn parse(mut fields: &mut &[KvField]) -> Result<Self, TngThingItemError> {
        let (new_thing_field, thing_kind) = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("NewThing")?
            .with_no_path()?
            .with_identifier_value()?;

        let thing_item = match thing_kind {
            "Thing" => Self::Thing(TngThing::parse(&mut fields)?),
            "Marker" => Self::Marker(TngMarker::parse(&mut fields)?),
            "Object" => Self::Object(TngObject::parse(&mut fields)?),
            "Holy Site" => Self::HolySite(TngHolySite::parse(&mut fields)?),
            "Building" => Self::Building(TngBuilding::parse(&mut fields)?),
            "Village" => Self::Village(TngVillage::parse(&mut fields)?),
            "AICreature" => Self::AICreature(TngAICreature::parse(&mut fields)?),
            _ => Err(TngThingItemError::UnrecognizedThing {
                line_num: new_thing_field.line_num,
            })?,
        };

        Ok(thing_item)
    }
}

#[derive(Clone, Debug)]
pub struct TngThing {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngThingError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngThing {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngThingError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_value()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field)
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngMarker {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngMarkerError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngMarker {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngMarkerError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_value()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngObject {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngObjectError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngObject {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngObjectError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngHolySite {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngHolySiteError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngHolySite {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngHolySiteError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngBuilding {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngBuildingError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngBuilding {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngBuildingError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngVillage {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngVillageError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngVillage {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngVillageError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngAICreature {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngAICreatureError {
    #[error(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    UnexpectedField(#[from] UnexpectedField),

    #[error(transparent)]
    InvalidPath(#[from] InvalidPath),

    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
}

impl TngAICreature {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngAICreatureError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndThing" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    // println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("unexpected end of input")]
pub struct UnexpectedEnd;
