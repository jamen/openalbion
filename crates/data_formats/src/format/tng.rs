use crate::util::{
    kv::{Kv, KvError, KvField},
    slice::TakeSliceExt,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Tng {
    sections: Vec<TngSection>,
}

#[derive(Clone, Debug, Error)]
pub enum TngError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("expected Version field at {line_num}")]
    ExpectedVersion { line_num: usize },

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected version at {line_num}")]
    UnexpectedVersion { line_num: usize },

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

        let version = fields.grab_first().ok_or_else(|| TngError::UnexpectedEnd)?;

        // Check field name
        if version.key.ident != "Version" {
            return Err(TngError::ExpectedVersion {
                line_num: version.line_num,
            });
        }

        // Check field has no path
        let mut version_path_iter = version.key.path.iter();

        if version_path_iter.next().is_some() {
            return Err(TngError::UnexpectedPath {
                line_num: version.line_num,
            });
        }

        // Check version number
        let version_num = version
            .value
            .integer()
            .map_err(|err| TngError::UnexpectedVersion {
                line_num: version.line_num,
            })?
            .to_owned();

        if version_num != 2 {
            return Err(TngError::UnexpectedVersion {
                line_num: version.line_num,
            });
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

#[derive(Copy, Clone, Debug, Error)]
pub enum TngSectionError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("expected XXXSectionStart field at {line_num}")]
    ExpectedSectionStart { line_num: usize },

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("invalid section name at {line_num}")]
    InvalidName { line_num: usize },

    #[error("expected NewThing field at {line_num}")]
    ExpectedNewThing { line_num: usize },

    #[error(transparent)]
    Thing(#[from] TngThingItemError),
}

impl TngSection {
    fn parse(mut fields: &mut &[KvField]) -> Result<Self, TngSectionError> {
        let section_start = fields
            .grab_first()
            .ok_or_else(|| TngSectionError::UnexpectedEnd)?;

        // Check field name
        if section_start.key.ident != "XXXSectionStart" {
            return Err(TngSectionError::ExpectedSectionStart {
                line_num: section_start.line_num,
            });
        }

        // Check field has no path
        let mut section_start_path_iter = section_start.key.path.iter();

        if section_start_path_iter.next().is_some() {
            return Err(TngSectionError::UnexpectedPath {
                line_num: section_start.line_num,
            });
        }

        // Grab section's name
        let name = section_start
            .value
            .ident()
            .map_err(|err| TngSectionError::InvalidName {
                line_num: section_start.line_num,
            })?
            .to_owned();

        // Begin parsing section's "things"
        let mut things = Vec::new();

        loop {
            let field = fields
                .first()
                .ok_or_else(|| TngSectionError::UnexpectedEnd)?;

            println!("{:?}", field);

            match field.key.ident {
                "NewThing" => things.push(TngThingItem::parse(&mut fields)?),
                "XXXSectionEnd" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngSectionError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngSectionError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    let _ = fields.grab_first();

                    println!("last {:?}", fields);

                    break;
                }
                _ => {
                    return Err(TngSectionError::ExpectedNewThing {
                        line_num: field.line_num,
                    })
                }
            }
        }

        Ok(Self { name, things })
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

#[derive(Copy, Clone, Debug, Error)]
pub enum TngThingItemError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("expected NewThing field at {line_num}")]
    ExpectedNewThing { line_num: usize },

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("expected identifier value at {line_num}")]
    ExpectedIdentValue { line_num: usize },

    #[error("unexpected thing variant at {line_num}")]
    UnexpectedThingVariant { line_num: usize },

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
        let new_thing = fields
            .grab_first()
            .ok_or_else(|| TngThingItemError::UnexpectedEnd)?;

        // Check field name
        if new_thing.key.ident != "NewThing" {
            return Err(TngThingItemError::ExpectedNewThing {
                line_num: new_thing.line_num,
            });
        }

        // Check field has no path
        let mut new_thing_path_iter = new_thing.key.path.iter();

        if new_thing_path_iter.next().is_some() {
            return Err(TngThingItemError::UnexpectedPath {
                line_num: new_thing.line_num,
            });
        }

        // Get NewThing ident value to determine which variant it is
        let thing_kind =
            new_thing
                .value
                .ident()
                .map_err(|_| TngThingItemError::ExpectedIdentValue {
                    line_num: new_thing.line_num,
                })?;

        let thing_item = match thing_kind {
            "Thing" => Self::Thing(TngThing::parse(&mut fields)?),
            "Marker" => Self::Marker(TngMarker::parse(&mut fields)?),
            "Object" => Self::Object(TngObject::parse(&mut fields)?),
            "Holy Site" => Self::HolySite(TngHolySite::parse(&mut fields)?),
            "Building" => Self::Building(TngBuilding::parse(&mut fields)?),
            "Village" => Self::Village(TngVillage::parse(&mut fields)?),
            "AICreature" => Self::AICreature(TngAICreature::parse(&mut fields)?),
            _ => {
                return Err(TngThingItemError::UnexpectedThingVariant {
                    line_num: new_thing.line_num,
                })
            }
        };

        Ok(thing_item)
    }
}

#[derive(Clone, Debug)]
pub struct TngThing {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngThingError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngThing {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngThingError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngThingError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngThingError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngThingError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngMarker {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngMarkerError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngMarker {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngMarkerError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngMarkerError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngMarkerError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngMarkerError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngObject {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngObjectError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngObject {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngObjectError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngObjectError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngObjectError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngObjectError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngHolySite {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngHolySiteError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngHolySite {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngHolySiteError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngHolySiteError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngHolySiteError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngHolySiteError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngBuilding {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngBuildingError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngBuilding {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngBuildingError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngBuildingError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngBuildingError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngBuildingError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngVillage {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngVillageError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngVillage {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngVillageError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngVillageError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngVillageError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ = field
                        .value
                        .empty()
                        .map_err(|_| TngVillageError::UnexpectedValue {
                            line_num: field.line_num,
                        })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Clone, Debug)]
pub struct TngAICreature {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngAICreatureError {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("unexpected path at {line_num}")]
    UnexpectedPath { line_num: usize },

    #[error("unexpected value at {line_num}")]
    UnexpectedValue { line_num: usize },

    #[error("unexpected field at {line_num}")]
    UnexpectedField { line_num: usize },
}

impl TngAICreature {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngAICreatureError> {
        loop {
            let field = fields
                .grab_first()
                .ok_or_else(|| TngAICreatureError::UnexpectedEnd)?;

            match field.key.ident {
                "EndThing" => {
                    // Check field has no path
                    let mut field_path_iter = field.key.path.iter();

                    if field_path_iter.next().is_some() {
                        return Err(TngAICreatureError::UnexpectedPath {
                            line_num: field.line_num,
                        });
                    }

                    // Check field has no value
                    let _ =
                        field
                            .value
                            .empty()
                            .map_err(|_| TngAICreatureError::UnexpectedValue {
                                line_num: field.line_num,
                            })?;

                    break;
                }
                // _ => {
                //     return Err(TngThingError::UnexpectedField {
                //         line_num: field.line_num,
                //     })
                // }
                _ => {
                    println!("{:?}", field);
                }
            }
        }

        Ok(Self {})
    }
}
