use std::str::FromStr;

use crate::util::{
    kv::{
        missing_field,
        CommonFieldError::{self, InvalidPath, InvalidValue, UnexpectedEnd, UnexpectedField},
        Kv, KvError, KvField,
    },
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
    Common(#[from] CommonFieldError),

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

        let line_num = version_field.line;

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
    things: Vec<TngThing>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngSectionError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),

    #[error(transparent)]
    Thing(#[from] TngThingError),
}

impl TngSection {
    fn parse(mut fields: &mut &[KvField]) -> Result<Self, TngSectionError> {
        let (_section_start_field, name) = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("XXXSectionStart")?
            .with_no_path()?
            .with_identifier_value()?;

        let name = name.to_owned();

        let mut things = Vec::new();

        loop {
            let field = fields
                .first()
                .ok_or_else(|| UnexpectedEnd)?
                .with_no_path()?;

            let line_num = field.line;

            match field.key.identifier {
                "NewThing" => things.push(TngThing::parse(&mut fields)?),
                "XXXSectionEnd" => {
                    let _ = field.with_no_value()?;
                    let _ = fields.grab_first();
                    break;
                }
                _ => Err(UnexpectedField { line: line_num })?,
            }
        }

        Ok(Self { name, things })
    }
}

#[derive(Clone, Debug)]
pub enum TngThingKind {
    Thing,
    Marker,
    Object,
    HolySite,
    Building,
    Village,
    AICreature,
    TrackNode,
    Switch,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[error("unrecognized kind")]
pub struct TngThingKindError;

impl TngThingKind {}

impl FromStr for TngThingKind {
    type Err = TngThingKindError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(match source {
            "Thing" => Self::Thing,
            "Marker" => Self::Marker,
            "Object" => Self::Object,
            "Holy Site" => Self::HolySite,
            "Building" => Self::Building,
            "Village" => Self::Village,
            "AICreature" => Self::AICreature,
            "TrackNode" => Self::TrackNode,
            "Switch" => Self::Switch,
            _ => Err(TngThingKindError)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TngThing {
    kind: TngThingKind,
    player: i32,
    uid: u64,
    definition_type: String,
    script_name: String,
    script_data: String,
    thing_game_persistent: bool,
    thing_level_persistent: bool,
    ctc_editor: CTCEditor,
    extras: Option<Box<TngThingExtras>>,
}

#[derive(Clone, Debug)]
struct TngThingExtras {
    create_tc: Option<String>,
    ctc_physics_light: Option<CTCPhysicsLight>,
    ctcd_navigation_seed: Option<CTCDNavigationSeed>,
    ctc_physics_standard: Option<CTCPhysicsStandard>,
    ctc_camera_point: Option<CTCDCameraPoint>,
    ctc_camera_point_scripted: Option<CTCCameraPointScripted>,
    ctc_camera_point_scripted_spline: Option<CTCCameraPointScriptedSpline>,
    ctcd_particle_emitter: Option<CTCDParticleEmitter>,
    ctcd_region_exit: Option<CTCDRegionExit>,
    ctcd_region_entrance: Option<CTCDRegionEntrance>,
    ctc_owned_entity: Option<CTCOwnedEntity>,
    ctc_camera_point_fixed_point: Option<CTCCameraPointFixedPoint>,
    ctc_shape_manager: Option<CTCShapeManager>,
    ctc_camera_point_track: Option<CTCCameraPointTrack>,
    ctc_camera_point_general_case: Option<CTCCameraPointGeneralCase>,
    ctc_targeted: Option<CTCTargeted>,
    ctc_action_use_scripted_hook: Option<CTCActionUseScriptedHook>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngThingError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),

    #[error("thing of unrecognized kind on line {line}")]
    Unrecognized { line: usize },

    #[error(transparent)]
    CTCPhysicsLight(#[from] CTCPhysicsLightError),

    #[error(transparent)]
    CTCEditor(#[from] CTCEditorError),

    #[error(transparent)]
    CTCDNavigationSeed(#[from] CTCDNavigationSeedError),

    #[error(transparent)]
    CTCPhysicsStandard(#[from] CTCPhysicsStandardError),

    #[error(transparent)]
    CTCDCameraPoint(#[from] CTCDCameraPointError),

    #[error(transparent)]
    CTCCameraPointScripted(#[from] CTCCameraPointScriptedError),

    #[error(transparent)]
    CTCCameraPointScriptedSpline(#[from] CTCCameraPointScriptedSplineError),

    #[error(transparent)]
    CTCDParticleEmitter(#[from] CTCDParticleEmitterError),

    #[error(transparent)]
    CTCDRegionExit(#[from] CTCDRegionExitError),

    #[error(transparent)]
    CTCDRegionEntrance(#[from] CTCDRegionEntranceError),

    #[error(transparent)]
    CTCOwnedEntity(#[from] CTCOwnedEntityError),

    #[error(transparent)]
    CTCCameraPointFixedPoint(#[from] CTCCameraPointFixedPointError),

    #[error(transparent)]
    CTCShapeManager(#[from] CTCShapeManagerError),

    #[error(transparent)]
    CTCCameraPointTrack(#[from] CTCCameraPointTrackError),

    #[error(transparent)]
    CTCCameraPointGeneralCase(#[from] CTCCameraPointGeneralCaseError),

    #[error(transparent)]
    CTCTargeted(#[from] CTCTargetedError),

    #[error(transparent)]
    CTCActionUseScriptedHook(#[from] CTCActionUseScriptedHookError),
}

impl TngThing {
    fn parse_kind(field: &KvField) -> Result<TngThingKind, TngThingError> {
        let (new_thing_field, kind_source) = field
            .with_key("NewThing")?
            .with_no_path()?
            .with_identifier_value()?;

        let kind =
            kind_source
                .parse::<TngThingKind>()
                .map_err(|_| TngThingError::Unrecognized {
                    line: new_thing_field.line,
                })?;

        Ok(kind)
    }

    fn parse(fields: &mut &[KvField]) -> Result<Self, TngThingError> {
        // Required
        let mut kind = None;
        let mut player = None;
        let mut uid = None;
        let mut definition_type = None;
        let mut script_name = None;
        let mut script_data = None;
        let mut thing_game_persistent = None;
        let mut thing_level_persistent = None;
        let mut ctc_editor = None;

        // Extras
        let mut create_tc = None;
        let mut ctc_physics_light = None;
        let mut ctcd_navigation_seed = None;
        let mut ctc_physics_standard = None;
        let mut ctc_camera_point = None;
        let mut ctc_camera_point_scripted = None;
        let mut ctc_camera_point_scripted_spline = None;
        let mut ctcd_particle_emitter = None;
        let mut ctcd_region_exit = None;
        let mut ctcd_region_entrance = None;
        let mut ctc_owned_entity = None;
        let mut ctc_camera_point_fixed_point = None;
        let mut ctc_shape_manager = None;
        let mut ctc_camera_point_track = None;
        let mut ctc_camera_point_general_case = None;
        let mut ctc_targeted = None;
        let mut ctc_action_use_scripted_hook = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "NewThing" => {
                    kind = Some(Self::parse_kind(&field)?);
                }
                "Player" => {
                    player = Some(field.with_no_path()?.with_integer_value()?.1);
                }
                "UID" => {
                    uid = Some(field.with_no_path()?.with_uid_value()?.1);
                }
                "DefinitionType" => {
                    definition_type = Some(field.with_no_path()?.with_string_value()?.1.to_owned())
                }
                "ScriptName" => {
                    script_name = Some(field.with_no_path()?.with_identifier_value()?.1.to_owned())
                }
                "ScriptData" => {
                    script_data = Some(field.with_no_path()?.with_string_value()?.1.to_owned())
                }
                "ThingGamePersistent" => {
                    thing_game_persistent = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "ThingLevelPersistent" => {
                    thing_level_persistent = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "CreateTC" => {
                    create_tc = Some(field.with_no_path()?.with_string_value()?.1.to_owned());
                }
                "StartCTCPhysicsLight" => {
                    ctc_physics_light = Some(CTCPhysicsLight::parse(fields)?);
                }
                "StartCTCEditor" => {
                    ctc_editor = Some(CTCEditor::parse(fields)?);
                }
                "StartCTCDNavigationSeed" => {
                    ctcd_navigation_seed = Some(CTCDNavigationSeed::parse(fields)?);
                }
                "StartCTCPhysicsStandard" => {
                    ctc_physics_standard = Some(CTCPhysicsStandard::parse(fields)?);
                }
                "StartCTCDCameraPoint" => {
                    ctc_camera_point = Some(CTCDCameraPoint::parse(fields)?);
                }
                "StartCTCCameraPointScripted" => {
                    ctc_camera_point_scripted = Some(CTCCameraPointScripted::parse(fields)?);
                }
                "StartCTCCameraPointScriptedSpline" => {
                    ctc_camera_point_scripted_spline =
                        Some(CTCCameraPointScriptedSpline::parse(fields)?);
                }
                "StartCTCDParticleEmitter" => {
                    ctcd_particle_emitter = Some(CTCDParticleEmitter::parse(fields)?);
                }
                "StartCTCDRegionExit" => {
                    ctcd_region_exit = Some(CTCDRegionExit::parse(fields)?);
                }
                "StartCTCDRegionEntrance" => {
                    ctcd_region_entrance = Some(CTCDRegionEntrance::parse(fields)?);
                }
                "StartCTCOwnedEntity" => {
                    ctc_owned_entity = Some(CTCOwnedEntity::parse(fields)?);
                }
                "StartCTCCameraPointFixedPoint" => {
                    ctc_camera_point_fixed_point = Some(CTCCameraPointFixedPoint::parse(fields)?);
                }
                "StartCTCShapeManager" => {
                    ctc_shape_manager = Some(CTCShapeManager::parse(fields)?);
                }
                "StartCTCCameraPointTrack" => {
                    ctc_camera_point_track = Some(CTCCameraPointTrack::parse(fields)?);
                }
                "StartCTCCameraPointGeneralCase" => {
                    ctc_camera_point_general_case = Some(CTCCameraPointGeneralCase::parse(fields)?);
                }
                "StartCTCTargeted" => {
                    ctc_targeted = Some(CTCTargeted::parse(fields)?);
                }
                "StartCTCActionUseScriptedHook" => {
                    ctc_action_use_scripted_hook = Some(CTCActionUseScriptedHook::parse(fields)?);
                }
                "EndThing" => {
                    let end_thing = field.with_no_path()?.with_no_value()?;
                    let ln = end_thing.line;

                    // Required
                    let kind = kind.ok_or_else(|| TngThingError::Unrecognized { line: ln })?;
                    let player = player.ok_or_else(|| missing_field(ln, "Player"))?;
                    let uid = uid.ok_or_else(|| missing_field(ln, "UID"))?;
                    let definition_type =
                        definition_type.ok_or_else(|| missing_field(ln, "DefinitionType"))?;
                    let script_name = script_name.ok_or_else(|| missing_field(ln, "ScriptName"))?;
                    let script_data = script_data.ok_or_else(|| missing_field(ln, "ScriptData"))?;
                    let thing_game_persistent = thing_game_persistent
                        .ok_or_else(|| missing_field(ln, "ThingGamePersistent"))?;
                    let thing_level_persistent = thing_level_persistent
                        .ok_or_else(|| missing_field(ln, "ThingLevelPersistent"))?;
                    let ctc_editor =
                        ctc_editor.ok_or_else(|| missing_field(ln, "StartCTCEditor"))?;

                    // Extras
                    let extras = TngThingExtras {
                        create_tc,
                        ctc_physics_light,
                        ctcd_navigation_seed,
                        ctc_physics_standard,
                        ctc_camera_point,
                        ctc_camera_point_scripted,
                        ctc_camera_point_scripted_spline,
                        ctcd_particle_emitter,
                        ctcd_region_exit,
                        ctcd_region_entrance,
                        ctc_owned_entity,
                        ctc_camera_point_fixed_point,
                        ctc_shape_manager,
                        ctc_camera_point_track,
                        ctc_camera_point_general_case,
                        ctc_targeted,
                        ctc_action_use_scripted_hook,
                    };

                    let extras = Some(Box::new(extras));

                    return Ok(Self {
                        kind,
                        player,
                        uid,
                        definition_type,
                        script_name,
                        script_data,
                        thing_game_persistent,
                        thing_level_persistent,
                        ctc_editor,
                        extras,
                    });
                }
                // _ => Err(UnexpectedField { line: field.line })?,
                _ => {}
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsLight {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCPhysicsLightError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCPhysicsLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCPhysicsLightError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCPhysicsLight" => {
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
pub struct CTCEditor {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCEditorError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCEditor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCEditorError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCEditor" => {
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
pub struct CTCDNavigationSeed {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCDNavigationSeedError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCDNavigationSeed {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCDNavigationSeedError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDNavigationSeed" => {
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
pub struct CTCPhysicsStandard {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCPhysicsStandardError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCPhysicsStandard {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCPhysicsStandardError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCPhysicsStandard" => {
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
pub struct CTCDCameraPoint {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCDCameraPointError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCDCameraPoint {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCDCameraPointError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDCameraPoint" => {
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
pub struct CTCCameraPointScripted {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCCameraPointScriptedError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCCameraPointScripted {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCCameraPointScriptedError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointScripted" => {
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
pub struct CTCCameraPointScriptedSpline {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCCameraPointScriptedSplineError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCCameraPointScriptedSpline {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCCameraPointScriptedSplineError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointScriptedSpline" => {
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
pub struct CTCDParticleEmitter {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCDParticleEmitterError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCDParticleEmitter {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCDParticleEmitterError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDParticleEmitter" => {
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
pub struct CTCDRegionExit {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCDRegionExitError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCDRegionExit {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCDRegionExitError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDRegionExit" => {
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
pub struct CTCDRegionEntrance {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCDRegionEntranceError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCDRegionEntrance {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCDRegionEntranceError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDRegionEntrance" => {
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
pub struct CTCOwnedEntity {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCOwnedEntityError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCOwnedEntity {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCOwnedEntityError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCOwnedEntity" => {
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
pub struct CTCCameraPointFixedPoint {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCCameraPointFixedPointError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCCameraPointFixedPoint {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCCameraPointFixedPointError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointFixedPoint" => {
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
pub struct CTCShapeManager {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCShapeManagerError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCShapeManager {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCShapeManagerError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCShapeManager" => {
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
pub struct CTCCameraPointTrack {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCCameraPointTrackError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCCameraPointTrack {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCCameraPointTrackError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointTrack" => {
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
pub struct CTCCameraPointGeneralCase {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCCameraPointGeneralCaseError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCCameraPointGeneralCase {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCCameraPointGeneralCaseError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointGeneralCase" => {
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
pub struct CTCTargeted {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCTargetedError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCTargeted {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCTargetedError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCTargeted" => {
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
pub struct CTCActionUseScriptedHook {}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum CTCActionUseScriptedHookError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),
}

impl CTCActionUseScriptedHook {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CTCActionUseScriptedHookError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCActionUseScriptedHook" => {
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
