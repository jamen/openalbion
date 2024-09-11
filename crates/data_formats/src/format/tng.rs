use std::str::FromStr;

use crate::util::{
    kv::{
        missing_field,
        CommonFieldError::{self, UnexpectedEnd, UnexpectedField},
        Kv, KvError, KvField, KvPathItem,
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
    extras: Box<TngThingExtras>,
}

#[derive(Clone, Debug, Default)]
struct TngThingExtras {
    create_tc: Option<String>,
    health: Option<f32>,
    object_scale: Option<f32>,
    linked_to_uid_1: Option<u64>,
    linked_to_uid_2: Option<u64>,
    start: Option<bool>,
    end: Option<bool>,
    has_information: Option<bool>,
    wander_with_information: Option<bool>,
    wave_with_information: Option<bool>,
    continue_ai_with_information: Option<bool>,
    enable_creature_auto_placing: Option<bool>,
    allowed_to_follow_hero: Option<bool>,
    region_following_overridden_from_script: Option<bool>,
    responding_to_follow_and_wait: Option<bool>,
    can_be_courted: Option<bool>,
    can_be_married: Option<bool>,
    initial_pos_x: Option<f32>,
    initial_pos_y: Option<f32>,
    initial_pos_z: Option<f32>,
    overriding_brain_name: Option<String>,
    can_come_between_camera_and_hero: Option<i32>,
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
    ctc_door: Option<CTCDoor>,
    ctc_village_member: Option<CTCVillageMember>,
    ctc_shop: Option<CTCShop>,
    ctc_buyable_house: Option<CTCBuyableHouse>,
    ctc_village: Option<CTCVillage>,
    ctc_enemy: Option<CTCEnemy>,
    ctc_creature_opinion_of_hero: Option<CTCCreatureOpinionOfHero>,
    ctc_teleporter: Option<CTCTeleporter>,
    ctc_chest: Option<CTCChest>,
    ctc_searchable_container: Option<CTCSearchableContainer>,
    ctc_light: Option<CTCLight>,
    ctc_atmos_player: Option<CTCAtmosPlayer>,
    ctc_physics_navigator: Option<CTCPhysicsNavigator>,
    ctc_talk: Option<CTCTalk>,
    ctc_action_use_bed: Option<CTCActionUseBed>,
    ctc_hero_centre_door_marker: Option<CTCHeroCentreDoorMarker>,
    ctc_hero: Option<CTCHero>,
}

#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
pub enum TngThingError {
    #[error(transparent)]
    Common(#[from] CommonFieldError),

    #[error("thing of unrecognized kind on line {line}")]
    Unrecognized { line: usize },
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
        let mut health = None;
        let mut object_scale = None;
        let mut linked_to_uid_1 = None;
        let mut linked_to_uid_2 = None;
        let mut start = None;
        let mut end = None;
        let mut has_information = None;
        let mut wander_with_information = None;
        let mut wave_with_information = None;
        let mut continue_ai_with_information = None;
        let mut enable_creature_auto_placing = None;
        let mut allowed_to_follow_hero = None;
        let mut region_following_overridden_from_script = None;
        let mut responding_to_follow_and_wait = None;
        let mut can_be_courted = None;
        let mut can_be_married = None;
        let mut initial_pos_x = None;
        let mut initial_pos_y = None;
        let mut initial_pos_z = None;
        let mut can_come_between_camera_and_hero = None;

        // Extras (structured)
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
        let mut ctc_door = None;
        let mut ctc_village_member = None;
        let mut ctc_shop = None;
        let mut ctc_buyable_house = None;
        let mut ctc_village = None;
        let mut ctc_enemy = None;
        let mut ctc_creature_opinion_of_hero = None;
        let mut ctc_teleporter = None;
        let mut ctc_chest = None;
        let mut ctc_searchable_container = None;
        let mut ctc_light = None;
        let mut ctc_atmos_player = None;
        let mut ctc_physics_navigator = None;
        let mut ctc_talk = None;
        let mut overriding_brain_name = None;
        let mut ctc_action_use_bed = None;
        let mut ctc_hero_centre_door_marker = None;
        let mut ctc_hero = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                // Required field
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
                "StartCTCEditor" => {
                    ctc_editor = Some(CTCEditor::parse(fields)?);
                }

                // Extra
                "CreateTC" => {
                    create_tc = Some(field.with_no_path()?.with_string_value()?.1.to_owned());
                }
                "Health" => health = Some(field.with_no_path()?.with_float_value()?.1),
                "ObjectScale" => object_scale = Some(field.with_no_path()?.with_float_value()?.1),
                "LinkedToUID1" => linked_to_uid_1 = Some(field.with_no_path()?.with_uid_value()?.1),
                "LinkedToUID2" => linked_to_uid_2 = Some(field.with_no_path()?.with_uid_value()?.1),
                "Start" => start = Some(field.with_no_path()?.with_bool_value()?.1),
                "End" => end = Some(field.with_no_path()?.with_bool_value()?.1),
                "HasInformation" => {
                    has_information = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "WanderWithInformation" => {
                    wander_with_information = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "WaveWithInformation" => {
                    wave_with_information = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "ContinueAIWithInformation" => {
                    continue_ai_with_information = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "EnableCreatureAutoPlacing" => {
                    enable_creature_auto_placing = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "AllowedToFollowHero" => {
                    allowed_to_follow_hero = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "RegionFollowingOverriddenFromScript" => {
                    region_following_overridden_from_script =
                        Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "RespondingToFollowAndWait" => {
                    responding_to_follow_and_wait =
                        Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "CanBeCourted" => {
                    can_be_courted = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "CanBeMarried" => {
                    can_be_married = Some(field.with_no_path()?.with_bool_value()?.1);
                }
                "InitialPosX" => {
                    initial_pos_x = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "InitialPosY" => {
                    initial_pos_y = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "InitialPosZ" => {
                    initial_pos_z = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "OverridingBrainName" => {
                    overriding_brain_name =
                        Some(field.with_no_path()?.with_identifier_value()?.1.to_owned())
                }
                "CanComeBetweenCameraAndHero" => {
                    can_come_between_camera_and_hero =
                        Some(field.with_no_path()?.with_integer_value()?.1.to_owned());
                }
                "StartCTCPhysicsLight" => {
                    ctc_physics_light = Some(CTCPhysicsLight::parse(fields)?);
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
                "StartCTCDoor" => {
                    ctc_door = Some(CTCDoor::parse(fields)?);
                }
                "StartCTCVillageMember" => {
                    ctc_village_member = Some(CTCVillageMember::parse(fields)?);
                }
                "StartCTCShop" => {
                    ctc_shop = Some(CTCShop::parse(fields)?);
                }
                "StartCTCBuyableHouse" => {
                    ctc_buyable_house = Some(CTCBuyableHouse::parse(fields)?);
                }
                "StartCTCVillage" => {
                    ctc_village = Some(CTCVillage::parse(fields)?);
                }
                "StartCTCEnemy" => {
                    ctc_enemy = Some(CTCEnemy::parse(fields)?);
                }
                "StartCTCCreatureOpinionOfHero" => {
                    ctc_creature_opinion_of_hero = Some(CTCCreatureOpinionOfHero::parse(fields)?);
                }
                "StartCTCTeleporter" => {
                    ctc_teleporter = Some(CTCTeleporter::parse(fields)?);
                }
                "StartCTCChest" => {
                    ctc_chest = Some(CTCChest::parse(fields)?);
                }
                "StartCTCSearchableContainer" => {
                    ctc_searchable_container = Some(CTCSearchableContainer::parse(fields)?);
                }
                "StartCTCLight" => {
                    ctc_light = Some(CTCLight::parse(fields)?);
                }
                "StartCTCAtmosPlayer" => {
                    ctc_atmos_player = Some(CTCAtmosPlayer::parse(fields)?);
                }
                "StartCTCPhysicsNavigator" => {
                    ctc_physics_navigator = Some(CTCPhysicsNavigator::parse(fields)?);
                }
                "StartCTCTalk" => {
                    ctc_talk = Some(CTCTalk::parse(fields)?);
                }
                "StartCTCActionUseBed" => {
                    ctc_action_use_bed = Some(CTCActionUseBed::parse(fields)?);
                }
                "StartCTCHeroCentreDoorMarker" => {
                    ctc_hero_centre_door_marker = Some(CTCHeroCentreDoorMarker::parse(fields)?);
                }
                "StartCTCHero" => {
                    ctc_hero = Some(CTCHero::parse(fields)?);
                }

                // Final field
                "EndThing" => {
                    let end_thing = field.with_no_path()?.with_no_value()?;
                    let line = end_thing.line;

                    let kind = kind.ok_or_else(|| TngThingError::Unrecognized { line })?;
                    let player = player.ok_or_else(|| missing_field(line, "Player"))?;
                    let uid = uid.ok_or_else(|| missing_field(line, "UID"))?;
                    let definition_type =
                        definition_type.ok_or_else(|| missing_field(line, "DefinitionType"))?;
                    let script_name =
                        script_name.ok_or_else(|| missing_field(line, "ScriptName"))?;
                    let script_data =
                        script_data.ok_or_else(|| missing_field(line, "ScriptData"))?;
                    let thing_game_persistent = thing_game_persistent
                        .ok_or_else(|| missing_field(line, "ThingGamePersistent"))?;
                    let thing_level_persistent = thing_level_persistent
                        .ok_or_else(|| missing_field(line, "ThingLevelPersistent"))?;
                    let ctc_editor =
                        ctc_editor.ok_or_else(|| missing_field(line, "StartCTCEditor"))?;

                    let extras = Box::new(TngThingExtras {
                        create_tc,
                        health,
                        object_scale,
                        linked_to_uid_1,
                        linked_to_uid_2,
                        start,
                        end,
                        overriding_brain_name,
                        has_information,
                        wander_with_information,
                        wave_with_information,
                        continue_ai_with_information,
                        enable_creature_auto_placing,
                        allowed_to_follow_hero,
                        region_following_overridden_from_script,
                        responding_to_follow_and_wait,
                        can_be_courted,
                        can_be_married,
                        initial_pos_x,
                        initial_pos_y,
                        initial_pos_z,
                        can_come_between_camera_and_hero,
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
                        ctc_door,
                        ctc_village_member,
                        ctc_shop,
                        ctc_buyable_house,
                        ctc_village,
                        ctc_enemy,
                        ctc_creature_opinion_of_hero,
                        ctc_teleporter,
                        ctc_chest,
                        ctc_searchable_container,
                        ctc_light,
                        ctc_atmos_player,
                        ctc_physics_navigator,
                        ctc_talk,
                        ctc_action_use_bed,
                        ctc_hero_centre_door_marker,
                        ctc_hero,
                    });

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
                _ => Err(UnexpectedField { line: field.line })?,
                // _ => {}
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsLight {}

impl CTCPhysicsLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCPhysicsLight" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCEditor {}

impl CTCEditor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCEditor" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDoor {}

impl CTCDoor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDoor" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDNavigationSeed {}

impl CTCDNavigationSeed {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDNavigationSeed" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsStandard {
    position_x: f32,
    position_y: f32,
    position_z: f32,
    rh_set_forward_x: f32,
    rh_set_forward_y: f32,
    rh_set_forward_z: f32,
    rh_set_up_x: f32,
    rh_set_up_y: f32,
    rh_set_up_z: f32,
}

impl CTCPhysicsStandard {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut position_x = None;
        let mut position_y = None;
        let mut position_z = None;
        let mut rh_set_forward_x = None;
        let mut rh_set_forward_y = None;
        let mut rh_set_forward_z = None;
        let mut rh_set_up_x = None;
        let mut rh_set_up_y = None;
        let mut rh_set_up_z = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCPhysicsStandard" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    let line = field.line;

                    let position_x = position_x.ok_or_else(|| missing_field(line, "PositionX"))?;
                    let position_y = position_y.ok_or_else(|| missing_field(line, "PositionY"))?;
                    let position_z = position_z.ok_or_else(|| missing_field(line, "PositionZ"))?;
                    let rh_set_forward_x =
                        rh_set_forward_x.ok_or_else(|| missing_field(line, "RHSetForwardX"))?;
                    let rh_set_forward_y =
                        rh_set_forward_y.ok_or_else(|| missing_field(line, "RHSetForwardY"))?;
                    let rh_set_forward_z =
                        rh_set_forward_z.ok_or_else(|| missing_field(line, "RHSetForwardZ"))?;
                    let rh_set_up_x = rh_set_up_x.ok_or_else(|| missing_field(line, "RHSetUpX"))?;
                    let rh_set_up_y = rh_set_up_y.ok_or_else(|| missing_field(line, "RHSetUpY"))?;
                    let rh_set_up_z = rh_set_up_z.ok_or_else(|| missing_field(line, "RHSetUpZ"))?;

                    return Ok(Self {
                        position_x,
                        position_y,
                        position_z,
                        rh_set_forward_x,
                        rh_set_forward_y,
                        rh_set_forward_z,
                        rh_set_up_x,
                        rh_set_up_y,
                        rh_set_up_z,
                    });
                }
                "PositionX" => {
                    position_x = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "PositionY" => {
                    position_y = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "PositionZ" => {
                    position_z = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetForwardX" => {
                    rh_set_forward_x = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetForwardY" => {
                    rh_set_forward_y = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetForwardZ" => {
                    rh_set_forward_z = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetUpX" => {
                    rh_set_up_x = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetUpY" => {
                    rh_set_up_y = Some(field.with_no_path()?.with_float_value()?.1);
                }
                "RHSetUpZ" => {
                    rh_set_up_z = Some(field.with_no_path()?.with_float_value()?.1);
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDCameraPoint {}

impl CTCDCameraPoint {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDCameraPoint" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointScripted {
    cut_into: bool,
    cut_out_of: bool,
    test_angle_before_activation: bool,
    self_terminate: bool,
    hero_is_subject: bool,
    fov: f32,
    is_coord_base_relative_to_parent: bool,
    coord_base: [f32; 3],
    coord_axis_up: [f32; 3],
    coord_axis_fwd: [f32; 3],
    using_relative_coords: bool,
    using_relative_orientation: bool,
    look_direction: [f32; 3],
    look_direction_end: [f32; 3],
    start_pos: [f32; 3],
    end_pos: [f32; 3],
    transition_time: f32,
}

impl CTCCameraPointScripted {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut cut_into = None;
        let mut cut_out_of = None;
        let mut test_angle_before_activation = None;
        let mut self_terminate = None;
        let mut hero_is_subject = None;
        let mut fov = None;
        let mut is_coord_base_relative_to_parent = None;
        let mut coord_base = None;
        let mut coord_axis_up = None;
        let mut coord_axis_fwd = None;
        let mut using_relative_coords = None;
        let mut using_relative_orientation = None;
        let mut look_direction = [None, None, None];
        let mut look_direction_end = [None, None, None];
        let mut start_pos = [None, None, None];
        let mut end_pos = [None, None, None];
        let mut transition_time = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.with_no_path()?.with_bool_value()?.1),
                "CutOutOf" => cut_out_of = Some(field.with_no_path()?.with_bool_value()?.1),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "SelfTerminate" => {
                    self_terminate = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "HeroIsSubject" => {
                    hero_is_subject = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "FOV" => fov = Some(field.with_no_path()?.with_float_value()?.1),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent =
                        Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "CoordBase" => coord_base = Some(field.with_no_path()?.with_c3dcoordf_value()?.1),
                "CoordAxisUp" => {
                    coord_axis_up = Some(field.with_no_path()?.with_c3dcoordf_value()?.1)
                }
                "CoordAxisFwd" => {
                    coord_axis_fwd = Some(field.with_no_path()?.with_c3dcoordf_value()?.1)
                }
                "UsingRelativeCoords" => {
                    using_relative_coords = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.with_no_path()?.with_bool_value()?.1)
                }
                "LookDirection" => {
                    let (_, path) = field.with_path()?;

                    if path.len() != 1 {
                        Err(CommonFieldError::InvalidPath { line: field.line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(CommonFieldError::InvalidPath { line: field.line })?,
                    };

                    let (_, float) = field.with_float_value()?;

                    look_direction[index] = Some(float);
                }
                "LookDirectionEnd" => {
                    let (_, path) = field.with_path()?;

                    if path.len() != 1 {
                        Err(CommonFieldError::InvalidPath { line: field.line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(CommonFieldError::InvalidPath { line: field.line })?,
                    };

                    let (_, float) = field.with_float_value()?;

                    look_direction_end[index] = Some(float);
                }
                "StartPos" => {
                    let (_, path) = field.with_path()?;

                    if path.len() != 1 {
                        Err(CommonFieldError::InvalidPath { line: field.line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(CommonFieldError::InvalidPath { line: field.line })?,
                    };

                    let (_, float) = field.with_float_value()?;

                    start_pos[index] = Some(float);
                }
                "EndPos" => {
                    let (_, path) = field.with_path()?;

                    if path.len() != 1 {
                        Err(CommonFieldError::InvalidPath { line: field.line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(CommonFieldError::InvalidPath { line: field.line })?,
                    };

                    let (_, float) = field.with_float_value()?;

                    end_pos[index] = Some(float);
                }
                "TransitionTime" => {
                    transition_time = Some(field.with_no_path()?.with_float_value()?.1)
                }
                "EndCTCCameraPointScripted" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    let line = field.line;

                    let cut_into = cut_into.ok_or_else(|| missing_field(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing_field(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing_field(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing_field(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing_field(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing_field(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing_field(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing_field(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing_field(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing_field(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing_field(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing_field(line, "UsingRelativeOrientation"))?;
                    let look_direction = [
                        look_direction[0].ok_or_else(|| missing_field(line, "LookDirection.X"))?,
                        look_direction[1].ok_or_else(|| missing_field(line, "LookDirection.Y"))?,
                        look_direction[2].ok_or_else(|| missing_field(line, "LookDirection.Z"))?,
                    ];
                    let look_direction_end = [
                        look_direction_end[0]
                            .ok_or_else(|| missing_field(line, "LookDirectionEnd.X"))?,
                        look_direction_end[1]
                            .ok_or_else(|| missing_field(line, "LookDirectionEnd.Y"))?,
                        look_direction_end[2]
                            .ok_or_else(|| missing_field(line, "LookDirectionEnd.Z"))?,
                    ];
                    let start_pos = [
                        start_pos[0].ok_or_else(|| missing_field(line, "StartPos.X"))?,
                        start_pos[1].ok_or_else(|| missing_field(line, "StartPos.Y"))?,
                        start_pos[2].ok_or_else(|| missing_field(line, "StartPos.Z"))?,
                    ];
                    let end_pos = [
                        end_pos[0].ok_or_else(|| missing_field(line, "EndPos.X"))?,
                        end_pos[1].ok_or_else(|| missing_field(line, "EndPos.Y"))?,
                        end_pos[2].ok_or_else(|| missing_field(line, "EndPos.Z"))?,
                    ];
                    let transition_time =
                        transition_time.ok_or_else(|| missing_field(line, "TransitionTime"))?;

                    return Ok(Self {
                        cut_into,
                        cut_out_of,
                        test_angle_before_activation,
                        self_terminate,
                        hero_is_subject,
                        fov,
                        is_coord_base_relative_to_parent,
                        coord_base,
                        coord_axis_up,
                        coord_axis_fwd,
                        using_relative_coords,
                        using_relative_orientation,
                        look_direction,
                        look_direction_end,
                        start_pos,
                        end_pos,
                        transition_time,
                    });
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointScriptedSpline {}

impl CTCCameraPointScriptedSpline {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointScriptedSpline" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDParticleEmitter {}

impl CTCDParticleEmitter {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDParticleEmitter" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDRegionExit {}

impl CTCDRegionExit {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDRegionExit" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDRegionEntrance {}

impl CTCDRegionEntrance {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCDRegionEntrance" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCOwnedEntity {}

impl CTCOwnedEntity {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCOwnedEntity" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointFixedPoint {}

impl CTCCameraPointFixedPoint {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointFixedPoint" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCShapeManager {}

impl CTCShapeManager {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCShapeManager" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointTrack {}

impl CTCCameraPointTrack {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointTrack" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointGeneralCase {}

impl CTCCameraPointGeneralCase {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCameraPointGeneralCase" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCTargeted {}

impl CTCTargeted {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCTargeted" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActionUseScriptedHook {}

impl CTCActionUseScriptedHook {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCActionUseScriptedHook" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCVillageMember {}

impl CTCVillageMember {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCVillageMember" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCShop {}

impl CTCShop {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCShop" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCBuyableHouse {}

impl CTCBuyableHouse {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCBuyableHouse" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCVillage {}

impl CTCVillage {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCVillage" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCEnemy {}

impl CTCEnemy {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCEnemy" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCreatureOpinionOfHero {}

impl CTCCreatureOpinionOfHero {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCCreatureOpinionOfHero" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCTeleporter {}

impl CTCTeleporter {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCTeleporter" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCChest {}

impl CTCChest {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCChest" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCSearchableContainer {}

impl CTCSearchableContainer {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCSearchableContainer" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCLight {}

impl CTCLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCLight" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCAtmosPlayer {}

impl CTCAtmosPlayer {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCAtmosPlayer" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsNavigator {}

impl CTCPhysicsNavigator {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCPhysicsNavigator" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCTalk {}

impl CTCTalk {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCTalk" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActionUseBed {}

impl CTCActionUseBed {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCActionUseBed" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCHeroCentreDoorMarker {}

impl CTCHeroCentreDoorMarker {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCHeroCentreDoorMarker" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCHero {}

impl CTCHero {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

            match field.key.identifier {
                "EndCTCHero" => {
                    let _ = field.with_no_path()?.with_no_path()?;
                    return Ok(Self {});
                }
                _ => Err(CommonFieldError::UnexpectedField { line: field.line })?,
            }
        }
    }
}
