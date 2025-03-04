use super::{
    kv::{
        missing,
        CommonFieldError::{self, InvalidPath, InvalidValue, UnexpectedEnd, UnexpectedField},
        Kv, KvError, KvField, KvPathItem, KvValueKind,
    },
    slice::TakeSliceExt,
};
use derive_more::{Display, From};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone, Debug)]
pub struct Tng {
    pub sections: Vec<TngSection>,
}

#[derive(Clone, Debug, Display, From, PartialEq, Eq)]
pub enum TngError {
    Common(CommonFieldError),

    #[display("version field on line {line} is an unsupported version")]
    UnsupportedVersion {
        line: usize,
    },

    Kv(KvError),

    Section(TngSectionError),
}

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngError> {
        let kv = Kv::parse(source)?;
        let mut fields = &kv.fields[..];
        let mut sections = Vec::new();

        let version_field = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("Version")?;

        let line = version_field.line;

        let version = version_field.integer_value()?;

        if version != 2 {
            Err(TngError::UnsupportedVersion { line })?
        }

        while !fields.is_empty() {
            sections.push(TngSection::parse(&mut fields)?);
        }

        Ok(Self { sections })
    }
}

#[derive(Clone, Debug)]
pub struct TngSection {
    pub name: String,
    pub things: Vec<TngThing>,
}

#[derive(Copy, Clone, Debug, Display, From, PartialEq, Eq)]
pub enum TngSectionError {
    Common(CommonFieldError),
    Thing(TngThingError),
}

impl TngSection {
    fn parse(mut fields: &mut &[KvField]) -> Result<Self, TngSectionError> {
        let name = fields
            .grab_first()
            .ok_or_else(|| UnexpectedEnd)?
            .with_key("XXXSectionStart")?
            .identifier_value()?
            .to_owned();

        let mut things = Vec::new();

        loop {
            let field = fields.first().ok_or_else(|| UnexpectedEnd)?;

            let line_num = field.line;

            match field.key.identifier {
                "NewThing" => things.push(TngThing::parse(&mut fields)?),
                "XXXSectionEnd" => {
                    let _ = field.empty_value()?;
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

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq)]
#[display("unrecognized kind")]
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
    pub kind: TngThingKind,
    pub player: i32,
    pub uid: u64,
    pub definition_type: String,
    pub script_name: String,
    pub script_data: String,
    pub thing_game_persistent: bool,
    pub thing_level_persistent: bool,
    pub ctc_editor: CTCEditor,
    pub extras: Box<TngThingExtras>,
}

#[derive(Clone, Debug, Default)]
pub struct TngThingExtras {
    pub create_tc: Option<String>,
    pub health: Option<f32>,
    pub object_scale: Option<f32>,
    pub linked_to_uid_1: Option<u64>,
    pub linked_to_uid_2: Option<u64>,
    pub start: Option<bool>,
    pub end: Option<bool>,
    pub has_information: Option<bool>,
    pub wander_with_information: Option<bool>,
    pub wave_with_information: Option<bool>,
    pub continue_ai_with_information: Option<bool>,
    pub enable_creature_auto_placing: Option<bool>,
    pub allowed_to_follow_hero: Option<bool>,
    pub region_following_overridden_from_script: Option<bool>,
    pub responding_to_follow_and_wait: Option<bool>,
    pub can_be_courted: Option<bool>,
    pub can_be_married: Option<bool>,
    pub initial_pos_x: Option<f32>,
    pub initial_pos_y: Option<f32>,
    pub initial_pos_z: Option<f32>,
    pub overriding_brain_name: Option<String>,
    pub can_come_between_camera_and_hero: Option<i32>,
    pub work_building_uid: Option<u64>,
    pub trigger_radius: Option<f32>,
    pub triggered_by_thing: Option<String>,
    pub environment_def: Option<String>,
    pub time_to_change_environment_def: Option<f32>,
    pub home_building_uid: Option<u64>,

    pub ctc_physics_light: Option<CTCPhysicsLight>,
    pub ctcd_navigation_seed: Option<CTCDNavigationSeed>,
    pub ctc_physics_standard: Option<CTCPhysicsStandard>,
    pub ctc_camera_point: Option<CTCDCameraPoint>,
    pub ctc_camera_point_scripted: Option<CTCCameraPointScripted>,
    pub ctc_camera_point_scripted_spline: Option<CTCCameraPointScriptedSpline>,
    pub ctcd_particle_emitter: Option<CTCDParticleEmitter>,
    pub ctcd_region_exit: Option<CTCDRegionExit>,
    pub ctcd_region_entrance: Option<CTCDRegionEntrance>,
    pub ctc_owned_entity: Option<CTCOwnedEntity>,
    pub ctc_camera_point_fixed_point: Option<CTCCameraPointFixedPoint>,
    pub ctc_shape_manager: Option<CTCShapeManager>,
    pub ctc_camera_point_track: Option<CTCCameraPointTrack>,
    pub ctc_camera_point_general_case: Option<CTCCameraPointGeneralCase>,
    pub ctc_targeted: Option<CTCTargeted>,
    pub ctc_action_use_scripted_hook: Option<CTCActionUseScriptedHook>,
    pub ctc_door: Option<CTCDoor>,
    pub ctc_village_member: Option<CTCVillageMember>,
    pub ctc_shop: Option<CTCShop>,
    pub ctc_buyable_house: Option<CTCBuyableHouse>,
    pub ctc_village: Option<CTCVillage>,
    pub ctc_enemy: Option<CTCEnemy>,
    pub ctc_creature_opinion_of_hero: Option<CTCCreatureOpinionOfHero>,
    pub ctc_teleporter: Option<CTCTeleporter>,
    pub ctc_chest: Option<CTCChest>,
    pub ctc_searchable_container: Option<CTCSearchableContainer>,
    pub ctc_light: Option<CTCLight>,
    pub ctc_atmos_player: Option<CTCAtmosPlayer>,
    pub ctc_physics_navigator: Option<CTCPhysicsNavigator>,
    pub ctc_talk: Option<CTCTalk>,
    pub ctc_action_use_bed: Option<CTCActionUseBed>,
    pub ctc_hero_centre_door_marker: Option<CTCHeroCentreDoorMarker>,
    pub ctc_hero: Option<CTCHero>,
    pub ctc_container_reward_hero: Option<CTCContainerRewardHero>,
    pub ctc_random_appearance_morph: Option<CTCRandomAppearanceMorph>,
    pub ctc_wife: Option<CTCWife>,
    pub ctc_inventory_item: Option<CTCInventoryItem>,
    pub ctc_stock_item: Option<CTCStockItem>,
    pub ctc_guard: Option<CTCGuard>,
    pub ctc_object_augmentations: Option<CTCObjectAugmentations>,
    pub ctc_fishing_spot: Option<CTCFishingSpot>,
    pub ctc_info_display: Option<CTCInfoDisplay>,
    pub ctc_creature_generator: Option<CTCCreatureGenerator>,
    pub ctc_activation_receptor_creature_generator: Option<CTCActivationReceptorCreatureGenerator>,
    pub ctc_activation_trigger: Option<CTCActivationTrigger>,
    pub ctc_creature_generator_creator: Option<CTCCreatureGeneratorCreator>,
    pub ctc_spot_light: Option<CTCSpotLight>,
    pub ctc_carried_action_use_read: Option<CTCCarriedActionUseRead>,
    pub ctc_action_use_readable: Option<CTCActionUseReadable>,
    pub ctc_digging_spot: Option<CTCDiggingSpot>,
    pub ctc_wall_mount: Option<CTCWallMount>,
    pub ctc_ai_scratchpad: Option<CTCAIScratchpad>,
    pub ctc_pre_calculated_navigation_route: Option<CTCPreCalculatedNavigationRoute>,
    pub ctc_exploding_object: Option<CTCExplodingObject>,
    pub ctc_stealable_item_location: Option<CTCStealableItemLocation>,
    pub ctc_activation_receptor_door: Option<CTCActivationReceptorDoor>,
    pub ctc_boasting_area: Option<CTCBoastingArea>,
    pub ctc_trophy: Option<CTCTrophy>,
}

#[derive(Copy, Clone, Debug, Display, From, PartialEq, Eq)]
pub enum TngThingError {
    Common(CommonFieldError),

    #[display("thing of unrecognized kind on line {line}")]
    Unrecognized {
        line: usize,
    },
}

impl TngThing {
    fn parse_kind(field: &KvField) -> Result<TngThingKind, TngThingError> {
        let new_thing_field = field.with_key("NewThing")?;
        let line = new_thing_field.line;
        let kind = new_thing_field.identifier_value()?;

        let kind = kind
            .parse::<TngThingKind>()
            .map_err(|_| TngThingError::Unrecognized { line })?;

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
        let mut work_building_uid = None;
        let mut trigger_radius = None;
        let mut triggered_by_thing = None;
        let mut environment_def = None;
        let mut time_to_change_environment_def = None;
        let mut home_building_uid = None;

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
        let mut ctc_container_reward_hero = None;
        let mut ctc_random_appearance_morph = None;
        let mut ctc_wife = None;
        let mut ctc_inventory_item = None;
        let mut ctc_stock_item = None;
        let mut ctc_guard = None;
        let mut ctc_object_augmentations = None;
        let mut ctc_fishing_spot = None;
        let mut ctc_info_display = None;
        let mut ctc_creature_generator = None;
        let mut ctc_activation_receptor_creature_generator = None;
        let mut ctc_activation_trigger = None;
        let mut ctc_creature_generator_creator = None;
        let mut ctc_spot_light = None;
        let mut ctc_carried_action_use_read = None;
        let mut ctc_action_use_readable = None;
        let mut ctc_digging_spot = None;
        let mut ctc_wall_mount = None;
        let mut ctc_ai_scratchpad = None;
        let mut ctc_pre_calculated_navigation_route = None;
        let mut ctc_exploding_object = None;
        let mut ctc_stealable_item_location = None;
        let mut ctc_activation_receptor_door = None;
        let mut ctc_boasting_area = None;
        let mut ctc_trophy = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                // Required
                "NewThing" => kind = Some(Self::parse_kind(&field)?),
                "Player" => player = Some(field.integer_value()?),
                "UID" => uid = Some(field.uid_value()?),
                "DefinitionType" => definition_type = Some(field.string_value()?.to_owned()),
                "ScriptName" => script_name = Some(field.identifier_value()?.to_owned()),
                "ScriptData" => script_data = Some(field.string_value()?.to_owned()),
                "ThingGamePersistent" => thing_game_persistent = Some(field.bool_value()?),
                "ThingLevelPersistent" => thing_level_persistent = Some(field.bool_value()?),
                "StartCTCEditor" => ctc_editor = Some(CTCEditor::parse(fields)?),

                // Extra
                "CreateTC" => create_tc = Some(field.string_value()?.to_owned()),
                "Health" => health = Some(field.float_value()?),
                "ObjectScale" => object_scale = Some(field.float_value()?),
                "LinkedToUID1" => linked_to_uid_1 = Some(field.uid_value()?),
                "LinkedToUID2" => linked_to_uid_2 = Some(field.uid_value()?),
                "Start" => start = Some(field.bool_value()?),
                "End" => end = Some(field.bool_value()?),
                "HasInformation" => has_information = Some(field.bool_value()?),
                "WanderWithInformation" => wander_with_information = Some(field.bool_value()?),
                "WaveWithInformation" => wave_with_information = Some(field.bool_value()?),
                "ContinueAIWithInformation" => {
                    continue_ai_with_information = Some(field.bool_value()?)
                }
                "EnableCreatureAutoPlacing" => {
                    enable_creature_auto_placing = Some(field.bool_value()?)
                }
                "AllowedToFollowHero" => allowed_to_follow_hero = Some(field.bool_value()?),
                "RegionFollowingOverriddenFromScript" => {
                    region_following_overridden_from_script = Some(field.bool_value()?)
                }
                "RespondingToFollowAndWait" => {
                    responding_to_follow_and_wait = Some(field.bool_value()?);
                }
                "CanBeCourted" => {
                    can_be_courted = Some(field.bool_value()?);
                }
                "CanBeMarried" => {
                    can_be_married = Some(field.bool_value()?);
                }
                "InitialPosX" => {
                    initial_pos_x = Some(field.float_value()?);
                }
                "InitialPosY" => {
                    initial_pos_y = Some(field.float_value()?);
                }
                "InitialPosZ" => {
                    initial_pos_z = Some(field.float_value()?);
                }
                "OverridingBrainName" => {
                    overriding_brain_name = Some(field.identifier_value()?.to_owned())
                }
                "CanComeBetweenCameraAndHero" => {
                    can_come_between_camera_and_hero = Some(field.integer_value()?.to_owned());
                }
                "WorkBuildingUID" => {
                    work_building_uid = Some(field.uid_value()?);
                }
                "TriggerRadius" => {
                    trigger_radius = Some(field.float_value()?);
                }
                "TriggeredByThing" => {
                    triggered_by_thing = Some(field.identifier_value()?.to_owned());
                }
                "EnvironmentDef" => {
                    environment_def = Some(field.identifier_value()?.to_owned());
                }
                "TimeToChangeEnvironmentDef" => {
                    time_to_change_environment_def = Some(field.float_value()?);
                }
                "HomeBuildingUID" => {
                    home_building_uid = Some(field.uid_value()?);
                }

                // Extra (structured)
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
                "StartCTCContainerRewardHero" => {
                    ctc_container_reward_hero = Some(CTCContainerRewardHero::parse(fields)?);
                }
                "StartCTCRandomAppearanceMorph" => {
                    ctc_random_appearance_morph = Some(CTCRandomAppearanceMorph::parse(fields)?);
                }
                "StartCTCWife" => {
                    ctc_wife = Some(CTCWife::parse(fields)?);
                }
                "StartCTCInventoryItem" => {
                    ctc_inventory_item = Some(CTCInventoryItem::parse(fields)?);
                }
                "StartCTCStockItem" => {
                    ctc_stock_item = Some(CTCStockItem::parse(fields)?);
                }
                "StartCTCGuard" => {
                    ctc_guard = Some(CTCGuard::parse(fields)?);
                }
                "StartCTCObjectAugmentations" => {
                    ctc_object_augmentations = Some(CTCObjectAugmentations::parse(fields)?);
                }
                "StartCTCFishingSpot" => {
                    ctc_fishing_spot = Some(CTCFishingSpot::parse(fields)?);
                }
                "StartCTCInfoDisplay" => {
                    ctc_info_display = Some(CTCInfoDisplay::parse(fields)?);
                }
                "StartCTCCreatureGenerator" => {
                    ctc_creature_generator = Some(CTCCreatureGenerator::parse(fields)?);
                }
                "StartCTCActivationReceptorCreatureGenerator" => {
                    ctc_activation_receptor_creature_generator =
                        Some(CTCActivationReceptorCreatureGenerator::parse(fields)?)
                }
                "StartCTCActivationTrigger" => {
                    ctc_activation_trigger = Some(CTCActivationTrigger::parse(fields)?);
                }
                "StartCTCCreatureGeneratorCreator" => {
                    ctc_creature_generator_creator =
                        Some(CTCCreatureGeneratorCreator::parse(fields)?);
                }
                "StartCTCSpotLight" => ctc_spot_light = Some(CTCSpotLight::parse(fields)?),
                "StartCTCCarriedActionUseRead" => {
                    ctc_carried_action_use_read = Some(CTCCarriedActionUseRead::parse(fields)?)
                }
                "StartCTCActionUseReadable" => {
                    ctc_action_use_readable = Some(CTCActionUseReadable::parse(fields)?)
                }
                "StartCTCDiggingSpot" => ctc_digging_spot = Some(CTCDiggingSpot::parse(fields)?),
                "StartCTCWallMount" => ctc_wall_mount = Some(CTCWallMount::parse(fields)?),
                "StartCTCAIScratchpad" => ctc_ai_scratchpad = Some(CTCAIScratchpad::parse(fields)?),
                "StartCTCPreCalculatedNavigationRoute" => {
                    ctc_pre_calculated_navigation_route =
                        Some(CTCPreCalculatedNavigationRoute::parse(fields)?)
                }
                "StartCTCExplodingObject" => {
                    ctc_exploding_object = Some(CTCExplodingObject::parse(fields)?)
                }
                "StartCTCStealableItemLocation" => {
                    ctc_stealable_item_location = Some(CTCStealableItemLocation::parse(fields)?)
                }
                "StartCTCActivationReceptorDoor" => {
                    ctc_activation_receptor_door = Some(CTCActivationReceptorDoor::parse(fields)?)
                }
                "StartCTCBoastingArea" => ctc_boasting_area = Some(CTCBoastingArea::parse(fields)?),
                "StartCTCTrophy" => ctc_trophy = Some(CTCTrophy::parse(fields)?),

                "EndThing" => {
                    let end_thing = field;
                    let _ = end_thing.empty_value()?;
                    let line = end_thing.line;

                    let kind = kind.ok_or_else(|| TngThingError::Unrecognized { line })?;
                    let player = player.ok_or_else(|| missing(line, "Player"))?;
                    let uid = uid.ok_or_else(|| missing(line, "UID"))?;
                    let definition_type =
                        definition_type.ok_or_else(|| missing(line, "DefinitionType"))?;
                    let script_name = script_name.ok_or_else(|| missing(line, "ScriptName"))?;
                    let script_data = script_data.ok_or_else(|| missing(line, "ScriptData"))?;
                    let thing_game_persistent = thing_game_persistent
                        .ok_or_else(|| missing(line, "ThingGamePersistent"))?;
                    let thing_level_persistent = thing_level_persistent
                        .ok_or_else(|| missing(line, "ThingLevelPersistent"))?;
                    let ctc_editor = ctc_editor.ok_or_else(|| missing(line, "StartCTCEditor"))?;

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
                        work_building_uid,
                        trigger_radius,
                        triggered_by_thing,
                        environment_def,
                        time_to_change_environment_def,
                        home_building_uid,

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
                        ctc_container_reward_hero,
                        ctc_random_appearance_morph,
                        ctc_wife,
                        ctc_inventory_item,
                        ctc_stock_item,
                        ctc_guard,
                        ctc_object_augmentations,
                        ctc_fishing_spot,
                        ctc_info_display,
                        ctc_creature_generator,
                        ctc_activation_receptor_creature_generator,
                        ctc_activation_trigger,
                        ctc_creature_generator_creator,
                        ctc_spot_light,
                        ctc_carried_action_use_read,
                        ctc_action_use_readable,
                        ctc_digging_spot,
                        ctc_wall_mount,
                        ctc_ai_scratchpad,
                        ctc_pre_calculated_navigation_route,
                        ctc_exploding_object,
                        ctc_stealable_item_location,
                        ctc_activation_receptor_door,
                        ctc_boasting_area,
                        ctc_trophy,
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
                _ => Err(UnexpectedField { line })?,
                // _ => {}
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsLight {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

impl CTCPhysicsLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut position_x = None;
        let mut position_y = None;
        let mut position_z = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "PositionX" => position_x = Some(field.float_value()?),
                "PositionY" => position_y = Some(field.float_value()?),
                "PositionZ" => position_z = Some(field.float_value()?),
                "EndCTCPhysicsLight" => {
                    let position_x = position_x.ok_or_else(|| missing(line, "PositionX"))?;
                    let position_y = position_y.ok_or_else(|| missing(line, "PositionY"))?;
                    let position_z = position_z.ok_or_else(|| missing(line, "PositionZ"))?;

                    return Ok(Self {
                        position_x,
                        position_y,
                        position_z,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCEditor {
    pub locked_in_place: Option<bool>,
}

impl CTCEditor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut locked_in_place = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "LockedInPlace" => locked_in_place = Some(field.bool_value()?),
                "EndCTCEditor" => {
                    return Ok(Self { locked_in_place });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDoor {
    pub open: bool,
    pub door_trigger_type: Option<i32>,
}

impl CTCDoor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut open = None;
        let mut door_trigger_type = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Open" => open = Some(field.bool_value()?),
                "DoorTriggerType" => door_trigger_type = Some(field.integer_value()?),
                "EndCTCDoor" => {
                    let open = open.ok_or_else(|| missing(line, "Open"))?;

                    return Ok(Self {
                        open,
                        door_trigger_type,
                    });
                }
                _ => Err(UnexpectedField { line })?,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCDNavigationSeed" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsStandard {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rh_set_forward_x: f32,
    pub rh_set_forward_y: f32,
    pub rh_set_forward_z: f32,
    pub rh_set_up_x: f32,
    pub rh_set_up_y: f32,
    pub rh_set_up_z: f32,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCPhysicsStandard" => {
                    let position_x = position_x.ok_or_else(|| missing(line, "PositionX"))?;
                    let position_y = position_y.ok_or_else(|| missing(line, "PositionY"))?;
                    let position_z = position_z.ok_or_else(|| missing(line, "PositionZ"))?;
                    let rh_set_forward_x =
                        rh_set_forward_x.ok_or_else(|| missing(line, "RHSetForwardX"))?;
                    let rh_set_forward_y =
                        rh_set_forward_y.ok_or_else(|| missing(line, "RHSetForwardY"))?;
                    let rh_set_forward_z =
                        rh_set_forward_z.ok_or_else(|| missing(line, "RHSetForwardZ"))?;
                    let rh_set_up_x = rh_set_up_x.ok_or_else(|| missing(line, "RHSetUpX"))?;
                    let rh_set_up_y = rh_set_up_y.ok_or_else(|| missing(line, "RHSetUpY"))?;
                    let rh_set_up_z = rh_set_up_z.ok_or_else(|| missing(line, "RHSetUpZ"))?;

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
                    position_x = Some(field.float_value()?);
                }
                "PositionY" => {
                    position_y = Some(field.float_value()?);
                }
                "PositionZ" => {
                    position_z = Some(field.float_value()?);
                }
                "RHSetForwardX" => {
                    rh_set_forward_x = Some(field.float_value()?);
                }
                "RHSetForwardY" => {
                    rh_set_forward_y = Some(field.float_value()?);
                }
                "RHSetForwardZ" => {
                    rh_set_forward_z = Some(field.float_value()?);
                }
                "RHSetUpX" => {
                    rh_set_up_x = Some(field.float_value()?);
                }
                "RHSetUpY" => {
                    rh_set_up_y = Some(field.float_value()?);
                }
                "RHSetUpZ" => {
                    rh_set_up_z = Some(field.float_value()?);
                }
                _ => Err(UnexpectedField { line })?,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCDCameraPoint" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointScripted {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub fov: f32,
    pub is_coord_base_relative_to_parent: bool,
    pub coord_base: [f32; 3],
    pub coord_axis_up: [f32; 3],
    pub coord_axis_fwd: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub look_direction: [f32; 3],
    pub look_direction_end: [f32; 3],
    pub start_pos: [f32; 3],
    pub end_pos: [f32; 3],
    pub transition_time: f32,
    pub thing_uid: Option<u64>,
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
        let mut thing_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.bool_value()?),
                "CutOutOf" => cut_out_of = Some(field.bool_value()?),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.bool_value()?)
                }
                "SelfTerminate" => self_terminate = Some(field.bool_value()?),
                "HeroIsSubject" => hero_is_subject = Some(field.bool_value()?),
                "FOV" => fov = Some(field.float_value()?),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent = Some(field.bool_value()?)
                }
                "CoordBase" => coord_base = Some(field.c3dcoordf_value()?),
                "CoordAxisUp" => coord_axis_up = Some(field.c3dcoordf_value()?),
                "CoordAxisFwd" => coord_axis_fwd = Some(field.c3dcoordf_value()?),
                "UsingRelativeCoords" => using_relative_coords = Some(field.bool_value()?),
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.bool_value()?)
                }
                "LookDirection" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(InvalidPath { line })?,
                    };

                    look_direction[index] = Some(field.float_value()?);
                }
                "LookDirectionEnd" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(InvalidPath { line })?,
                    };

                    look_direction_end[index] = Some(field.float_value()?);
                }
                "StartPos" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(InvalidPath { line })?,
                    };

                    start_pos[index] = Some(field.float_value()?);
                }
                "EndPos" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?;
                    }

                    let index = match path[0] {
                        KvPathItem::Property("X") => 0,
                        KvPathItem::Property("Y") => 1,
                        KvPathItem::Property("Z") => 2,
                        _ => Err(InvalidPath { line })?,
                    };

                    end_pos[index] = Some(field.float_value()?);
                }
                "TransitionTime" => transition_time = Some(field.float_value()?),
                "ThingUID" => thing_uid = Some(field.uid_value()?),
                "EndCTCCameraPointScripted" => {
                    let line = field.line;

                    let cut_into = cut_into.ok_or_else(|| missing(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing(line, "UsingRelativeOrientation"))?;
                    let look_direction = [
                        look_direction[0].ok_or_else(|| missing(line, "LookDirection.X"))?,
                        look_direction[1].ok_or_else(|| missing(line, "LookDirection.Y"))?,
                        look_direction[2].ok_or_else(|| missing(line, "LookDirection.Z"))?,
                    ];
                    let look_direction_end = [
                        look_direction_end[0].ok_or_else(|| missing(line, "LookDirectionEnd.X"))?,
                        look_direction_end[1].ok_or_else(|| missing(line, "LookDirectionEnd.Y"))?,
                        look_direction_end[2].ok_or_else(|| missing(line, "LookDirectionEnd.Z"))?,
                    ];
                    let start_pos = [
                        start_pos[0].ok_or_else(|| missing(line, "StartPos.X"))?,
                        start_pos[1].ok_or_else(|| missing(line, "StartPos.Y"))?,
                        start_pos[2].ok_or_else(|| missing(line, "StartPos.Z"))?,
                    ];
                    let end_pos = [
                        end_pos[0].ok_or_else(|| missing(line, "EndPos.X"))?,
                        end_pos[1].ok_or_else(|| missing(line, "EndPos.Y"))?,
                        end_pos[2].ok_or_else(|| missing(line, "EndPos.Z"))?,
                    ];
                    let transition_time =
                        transition_time.ok_or_else(|| missing(line, "TransitionTime"))?;

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
                        thing_uid,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointScriptedSpline {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub fov: f32,
    pub is_coord_base_relative_to_parent: bool,
    pub coord_base: [f32; 3],
    pub coord_axis_up: [f32; 3],
    pub coord_axis_fwd: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub time_to_play: f32,
    pub tension: f32,
    pub num_key_cameras: i32,
    pub key_cameras: Vec<TngKeyCamera>,
    pub valid_anims: BTreeMap<usize, String>,
}

impl CTCCameraPointScriptedSpline {
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
        let mut time_to_play = None;
        let mut tension = None;
        let mut num_key_cameras = None;
        let mut key_cameras = Vec::new();
        let mut valid_anims = BTreeMap::new();

        loop {
            let peek_field = fields.first().ok_or_else(|| UnexpectedEnd)?;

            if peek_field.key.identifier == "KeyCameras" {
                let num_key_cameras =
                    num_key_cameras.ok_or_else(|| missing(peek_field.line, "NumKeyCameras"))?;
                key_cameras = TngKeyCamera::parse_list(fields, num_key_cameras)?;
                continue;
            }

            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.bool_value()?),
                "CutOutOf" => cut_out_of = Some(field.bool_value()?),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.bool_value()?)
                }
                "SelfTerminate" => self_terminate = Some(field.bool_value()?),
                "HeroIsSubject" => hero_is_subject = Some(field.bool_value()?),
                "FOV" => fov = Some(field.float_value()?),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent = Some(field.bool_value()?)
                }
                "CoordBase" => coord_base = Some(field.c3dcoordf_value()?),
                "CoordAxisUp" => coord_axis_up = Some(field.c3dcoordf_value()?),
                "CoordAxisFwd" => coord_axis_fwd = Some(field.c3dcoordf_value()?),
                "UsingRelativeCoords" => using_relative_coords = Some(field.bool_value()?),
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.bool_value()?)
                }
                "TimeToPlay" => time_to_play = Some(field.float_value()?),
                "Tension" => tension = Some(field.float_value()?),
                "NumKeyCameras" => num_key_cameras = Some(field.integer_value()?),
                "ValidAnims" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let valid_anim_index = if let KvPathItem::Index(valid_anim_index) = path[0] {
                        if valid_anim_index < 0 {
                            Err(InvalidPath { line })?
                        }

                        valid_anim_index as usize
                    } else {
                        Err(InvalidPath { line })?
                    };

                    let value = field.string_value()?.to_owned();

                    if valid_anims.insert(valid_anim_index, value).is_some() {
                        Err(UnexpectedField { line })?
                    };
                }
                "EndCTCCameraPointScriptedSpline" => {
                    let cut_into = cut_into.ok_or_else(|| missing(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing(line, "UsingRelativeOrientation"))?;
                    let time_to_play = time_to_play.ok_or_else(|| missing(line, "TimeToPlay"))?;
                    let tension = tension.ok_or_else(|| missing(line, "Tension"))?;
                    let num_key_cameras =
                        num_key_cameras.ok_or_else(|| missing(line, "NumKeyCameras"))?;

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
                        time_to_play,
                        tension,
                        num_key_cameras,
                        key_cameras,
                        valid_anims,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDParticleEmitter {
    pub independant_object: bool,
    pub particle_type_name: String,
}

impl CTCDParticleEmitter {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut independant_object = None;
        let mut particle_type_name = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "IndependantObject" => independant_object = Some(field.bool_value()?),
                "ParticleTypeName" => particle_type_name = Some(field.string_value()?.to_owned()),
                "EndCTCDParticleEmitter" => {
                    let independant_object =
                        independant_object.ok_or_else(|| missing(line, "IndependantObject"))?;
                    let particle_type_name =
                        particle_type_name.ok_or_else(|| missing(line, "ParticleTypeName"))?;

                    return Ok(Self {
                        independant_object,
                        particle_type_name,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDRegionExit {
    pub active: bool,
    pub radius: f32,
    pub message_radius: f32,
    pub reversed_on_mini_map: Option<bool>,
    pub hidden_on_mini_map: Option<bool>,
    pub entrance_connected_to_uid: u64,
}

impl CTCDRegionExit {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut active = None;
        let mut radius = None;
        let mut message_radius = None;
        let mut reversed_on_mini_map = None;
        let mut hidden_on_mini_map = None;
        let mut entrance_connected_to_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Active" => active = Some(field.bool_value()?),
                "Radius" => radius = Some(field.float_value()?),
                "MessageRadius" => message_radius = Some(field.float_value()?),
                "ReversedOnMiniMap" => reversed_on_mini_map = Some(field.bool_value()?),
                "HiddenOnMiniMap" => hidden_on_mini_map = Some(field.bool_value()?),
                "EntranceConnectedToUID" => entrance_connected_to_uid = Some(field.uid_value()?),
                "EndCTCDRegionExit" => {
                    let active = active.ok_or_else(|| missing(line, "Active"))?;
                    let radius = radius.ok_or_else(|| missing(line, "Radius"))?;
                    let message_radius =
                        message_radius.ok_or_else(|| missing(line, "MessageRadius"))?;
                    let entrance_connected_to_uid = entrance_connected_to_uid
                        .ok_or_else(|| missing(line, "EntranceConnectedToUID"))?;

                    return Ok(Self {
                        active,
                        radius,
                        message_radius,
                        reversed_on_mini_map,
                        hidden_on_mini_map,
                        entrance_connected_to_uid,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDRegionEntrance {
    pub active: Option<bool>,
}

impl CTCDRegionEntrance {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut active = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Active" => active = Some(field.bool_value()?),
                "EndCTCDRegionEntrance" => {
                    return Ok(Self { active });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCOwnedEntity {
    pub switchable_navigation_tc_added: bool,
    pub version_number: i32,
    pub owner_uid: u64,
}

impl CTCOwnedEntity {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut switchable_navigation_tc_added = None;
        let mut version_number = None;
        let mut owner_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "SwitchableNavigationTCAdded" => {
                    switchable_navigation_tc_added = Some(field.bool_value()?)
                }
                "VersionNumber" => version_number = Some(field.integer_value()?),
                "OwnerUID" => owner_uid = Some(field.uid_value()?),
                "EndCTCOwnedEntity" => {
                    let switchable_navigation_tc_added = switchable_navigation_tc_added
                        .ok_or_else(|| missing(line, "SwitchableNavigationTCAdded"))?;
                    let version_number =
                        version_number.ok_or_else(|| missing(line, "VersionNumber"))?;
                    let owner_uid = owner_uid.ok_or_else(|| missing(line, "OwnerUID"))?;

                    return Ok(Self {
                        switchable_navigation_tc_added,
                        version_number,
                        owner_uid,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointFixedPoint {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub fov: f32,
    pub is_coord_base_relative_to_parent: bool,
    pub coord_base: [f32; 3],
    pub coord_axis_up: [f32; 3],
    pub coord_axis_fwd: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub track_thing: bool,
    pub look_vector: [f32; 3],
}

impl CTCCameraPointFixedPoint {
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
        let mut track_thing = None;
        let mut look_vector = [None, None, None];

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.bool_value()?),
                "CutOutOf" => cut_out_of = Some(field.bool_value()?),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.bool_value()?)
                }
                "SelfTerminate" => self_terminate = Some(field.bool_value()?),
                "HeroIsSubject" => hero_is_subject = Some(field.bool_value()?),
                "FOV" => fov = Some(field.float_value()?),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent = Some(field.bool_value()?)
                }
                "CoordBase" => coord_base = Some(field.c3dcoordf_value()?),
                "CoordAxisUp" => coord_axis_up = Some(field.c3dcoordf_value()?),
                "CoordAxisFwd" => coord_axis_fwd = Some(field.c3dcoordf_value()?),
                "UsingRelativeCoords" => using_relative_coords = Some(field.bool_value()?),
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.bool_value()?)
                }
                "TrackThing" => track_thing = Some(field.bool_value()?),
                "LookVector" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let coord = if let KvPathItem::Property(coord) = path[0] {
                        coord
                    } else {
                        Err(InvalidPath { line })?
                    };

                    match coord {
                        "X" => look_vector[0] = Some(field.float_value()?),
                        "Y" => look_vector[1] = Some(field.float_value()?),
                        "Z" => look_vector[2] = Some(field.float_value()?),
                        _ => Err(InvalidPath { line })?,
                    }
                }
                "EndCTCCameraPointFixedPoint" => {
                    let cut_into = cut_into.ok_or_else(|| missing(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing(line, "UsingRelativeOrientation"))?;
                    let track_thing = track_thing.ok_or_else(|| missing(line, "TrackThing"))?;

                    let look_vector = [
                        look_vector[0].ok_or_else(|| missing(line, "LookVector.X"))?,
                        look_vector[1].ok_or_else(|| missing(line, "LookVector.Y"))?,
                        look_vector[2].ok_or_else(|| missing(line, "LookVector.Z"))?,
                    ];

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
                        track_thing,
                        look_vector,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCShapeManager {
    pub is_coords_relative_to_map: bool,
    pub num_shapes: i32,
    pub shape_info: BTreeMap<usize, TngShapeInfo>,
    pub shape_positions: BTreeMap<TngShapeIndex, f32>,
}

impl CTCShapeManager {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut is_coords_relative_to_map = None;
        let mut num_shapes = None;

        // Instead of parsing these `Shape` fields into a tree of structs,
        // I think it'll be simpler to parse it into a couple BTreeMaps.
        let mut shape_info = BTreeMap::new();
        let mut shape_positions = BTreeMap::new();

        let mut shape_type = None;
        let mut shape_pos_size = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "IsCoordsRelativeToMap" => is_coords_relative_to_map = Some(field.bool_value()?),
                "NumShapes" => num_shapes = Some(field.integer_value()?),
                "Shape" => {
                    let path = field.path()?;

                    let shape_index: usize =
                        if let Some(&KvPathItem::Index(shape_index)) = path.get(0) {
                            shape_index.try_into().map_err(|_| InvalidPath { line })?
                        } else {
                            Err(InvalidPath { line })?
                        };

                    let shape_prop = if let Some(&KvPathItem::Property(name)) = path.get(1) {
                        name
                    } else {
                        Err(InvalidPath { line })?
                    };

                    match shape_prop {
                        "Type" => shape_type = Some(TngShapeType::parse(field)?),
                        "size" => {
                            // NOTE: Unusual path item. Not sure what its for, but check for it.
                            if let Some(KvPathItem::Call) = path.get(2) {
                                shape_pos_size = Some(field.integer_value()?)
                            } else {
                                Err(InvalidPath { line })?
                            }

                            let r#type =
                                shape_type.clone().ok_or_else(|| UnexpectedField { line })?;

                            let position_size =
                                shape_pos_size.ok_or_else(|| UnexpectedField { line })?;

                            let info = TngShapeInfo {
                                r#type,
                                position_size,
                            };

                            if shape_info.insert(shape_index, info).is_some() {
                                Err(InvalidPath { line })?
                            }
                        }
                        "pos" => {
                            let position_size =
                                shape_pos_size.ok_or_else(|| UnexpectedField { line })?;

                            let position_index =
                                if let Some(&KvPathItem::Index(position_index)) = path.get(2) {
                                    if position_index < 0 {
                                        Err(InvalidPath { line })?
                                    }

                                    if position_index >= position_size {
                                        Err(InvalidPath { line })?
                                    }

                                    position_index as usize
                                } else {
                                    Err(InvalidPath { line })?
                                };

                            let position_coord =
                                if let Some(&KvPathItem::Property(position_coord)) = path.get(3) {
                                    TngShapeCoord::parse(position_coord)
                                        .ok_or_else(|| InvalidPath { line })?
                                } else {
                                    Err(InvalidPath { line })?
                                };

                            let complete_shape_index = TngShapeIndex {
                                shape_index,
                                position_index,
                                position_coord,
                            };

                            let value = field.float_value()?;

                            if shape_positions
                                .insert(complete_shape_index, value)
                                .is_some()
                            {
                                Err(UnexpectedField { line })?
                            }
                        }
                        _ => Err(InvalidPath { line })?,
                    }
                }
                "EndCTCShapeManager" => {
                    let is_coords_relative_to_map = is_coords_relative_to_map
                        .ok_or_else(|| missing(line, "IsCoordsRelativeToMap"))?;
                    let num_shapes = num_shapes.ok_or_else(|| missing(line, "NumShapes"))?;

                    return Ok(Self {
                        is_coords_relative_to_map,
                        num_shapes,
                        shape_info,
                        shape_positions,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TngShapeIndex {
    pub shape_index: usize,
    pub position_index: usize,
    pub position_coord: TngShapeCoord,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TngShapeCoord {
    X,
    Y,
    Z,
}

impl TngShapeCoord {
    fn parse(coord: &str) -> Option<Self> {
        match coord {
            "X" => Some(Self::X),
            "Y" => Some(Self::Y),
            "Z" => Some(Self::Z),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TngShapeInfo {
    pub position_size: i32,
    pub r#type: TngShapeType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TngShapeType {
    Line,
    Closed,
}

impl TngShapeType {
    pub fn parse(field: &KvField) -> Result<Self, CommonFieldError> {
        let line = field.line;

        match field.string_value()? {
            "SHAPE_TYPE_LINE" => Ok(Self::Line),
            "SHAPE_TYPE_CLOSED" => Ok(Self::Closed),
            _ => Err(InvalidValue {
                line,
                expected: KvValueKind::String,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointTrack {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub fov: f32,
    pub is_coord_base_relative_to_parent: bool,
    pub coord_base: [f32; 3],
    pub coord_axis_up: [f32; 3],
    pub coord_axis_fwd: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub string_length: f32,
    pub cage_radius: f32,
}

impl CTCCameraPointTrack {
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
        let mut string_length = None;
        let mut cage_radius = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.bool_value()?),
                "CutOutOf" => cut_out_of = Some(field.bool_value()?),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.bool_value()?)
                }
                "SelfTerminate" => self_terminate = Some(field.bool_value()?),
                "HeroIsSubject" => hero_is_subject = Some(field.bool_value()?),
                "FOV" => fov = Some(field.float_value()?),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent = Some(field.bool_value()?)
                }
                "CoordBase" => coord_base = Some(field.c3dcoordf_value()?),
                "CoordAxisUp" => coord_axis_up = Some(field.c3dcoordf_value()?),
                "CoordAxisFwd" => coord_axis_fwd = Some(field.c3dcoordf_value()?),
                "UsingRelativeCoords" => using_relative_coords = Some(field.bool_value()?),
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.bool_value()?)
                }
                "StringLength" => string_length = Some(field.float_value()?),
                "CageRadius" => cage_radius = Some(field.float_value()?),
                "EndCTCCameraPointTrack" => {
                    let cut_into = cut_into.ok_or_else(|| missing(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing(line, "UsingRelativeOrientation"))?;
                    let string_length =
                        string_length.ok_or_else(|| missing(line, "StringLength"))?;
                    let cage_radius = cage_radius.ok_or_else(|| missing(line, "CageRadius"))?;

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
                        string_length,
                        cage_radius,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCameraPointGeneralCase {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub fov: f32,
    pub is_coord_base_relative_to_parent: bool,
    pub coord_base: [f32; 3],
    pub coord_axis_up: [f32; 3],
    pub coord_axis_fwd: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub string_length: f32,
    pub cage_radius: f32,
    pub height_offset: f32,
    pub allow_right_stick_zoom: bool,
    pub allow_right_stick_rotation: bool,
    pub allow_z_target: bool,
    pub auto_go_behind: bool,
    pub auto_go_behind_time: f32,
}

impl CTCCameraPointGeneralCase {
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
        let mut string_length = None;
        let mut cage_radius = None;
        let mut height_offset = None;
        let mut allow_right_stick_zoom = None;
        let mut allow_right_stick_rotation = None;
        let mut allow_z_target = None;
        let mut auto_go_behind = None;
        let mut auto_go_behind_time = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CutInto" => cut_into = Some(field.bool_value()?),
                "CutOutOf" => cut_out_of = Some(field.bool_value()?),
                "TestAngleBeforeActivation" => {
                    test_angle_before_activation = Some(field.bool_value()?)
                }
                "SelfTerminate" => self_terminate = Some(field.bool_value()?),
                "HeroIsSubject" => hero_is_subject = Some(field.bool_value()?),
                "FOV" => fov = Some(field.float_value()?),
                "IsCoordBaseRelativeToParent" => {
                    is_coord_base_relative_to_parent = Some(field.bool_value()?)
                }
                "CoordBase" => coord_base = Some(field.c3dcoordf_value()?),
                "CoordAxisUp" => coord_axis_up = Some(field.c3dcoordf_value()?),
                "CoordAxisFwd" => coord_axis_fwd = Some(field.c3dcoordf_value()?),
                "UsingRelativeCoords" => using_relative_coords = Some(field.bool_value()?),
                "UsingRelativeOrientation" => {
                    using_relative_orientation = Some(field.bool_value()?)
                }
                "StringLength" => string_length = Some(field.float_value()?),
                "CageRadius" => cage_radius = Some(field.float_value()?),
                "HeightOffset" => height_offset = Some(field.float_value()?),
                "AllowRightStickZoom" => allow_right_stick_zoom = Some(field.bool_value()?),
                "AllowRightStickRotation" => allow_right_stick_rotation = Some(field.bool_value()?),
                "AllowZTarget" => allow_z_target = Some(field.bool_value()?),
                "AutoGoBehind" => auto_go_behind = Some(field.bool_value()?),
                "AutoGoBehindTime" => auto_go_behind_time = Some(field.float_value()?),
                "EndCTCCameraPointGeneralCase" => {
                    let cut_into = cut_into.ok_or_else(|| missing(line, "CutInto"))?;
                    let cut_out_of = cut_out_of.ok_or_else(|| missing(line, "CutOutOf"))?;
                    let test_angle_before_activation = test_angle_before_activation
                        .ok_or_else(|| missing(line, "TestAngleBeforeActivation"))?;
                    let self_terminate =
                        self_terminate.ok_or_else(|| missing(line, "SelfTerminate"))?;
                    let hero_is_subject =
                        hero_is_subject.ok_or_else(|| missing(line, "HeroIsSubject"))?;
                    let fov = fov.ok_or_else(|| missing(line, "FOV"))?;
                    let is_coord_base_relative_to_parent = is_coord_base_relative_to_parent
                        .ok_or_else(|| missing(line, "IsCoordBaseRelativeToParent"))?;
                    let coord_base = coord_base.ok_or_else(|| missing(line, "CoordBase"))?;
                    let coord_axis_up =
                        coord_axis_up.ok_or_else(|| missing(line, "CoordAxisUp"))?;
                    let coord_axis_fwd =
                        coord_axis_fwd.ok_or_else(|| missing(line, "CoordAxisFwd"))?;
                    let using_relative_coords = using_relative_coords
                        .ok_or_else(|| missing(line, "UsingRelativeCoords"))?;
                    let using_relative_orientation = using_relative_orientation
                        .ok_or_else(|| missing(line, "UsingRelativeOrientation"))?;
                    let string_length =
                        string_length.ok_or_else(|| missing(line, "StringLength"))?;
                    let cage_radius = cage_radius.ok_or_else(|| missing(line, "CageRadius"))?;
                    let height_offset =
                        height_offset.ok_or_else(|| missing(line, "HeightOffset"))?;
                    let allow_right_stick_zoom = allow_right_stick_zoom
                        .ok_or_else(|| missing(line, "AllowRightStickZoom"))?;
                    let allow_right_stick_rotation = allow_right_stick_rotation
                        .ok_or_else(|| missing(line, "AllowRightStickRotation"))?;
                    let allow_z_target =
                        allow_z_target.ok_or_else(|| missing(line, "AllowZTarget"))?;
                    let auto_go_behind =
                        auto_go_behind.ok_or_else(|| missing(line, "AutoGoBehind"))?;
                    let auto_go_behind_time =
                        auto_go_behind_time.ok_or_else(|| missing(line, "AutoGoBehindTime"))?;

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
                        string_length,
                        cage_radius,
                        height_offset,
                        allow_right_stick_zoom,
                        allow_right_stick_rotation,
                        allow_z_target,
                        auto_go_behind,
                        auto_go_behind_time,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCTargeted {
    pub targetable: bool,
}

impl CTCTargeted {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut targetable = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "EndCTCTargeted" => {
                    let targetable = targetable.ok_or_else(|| missing(line, "Targetable"))?;

                    return Ok(Self { targetable });
                }
                "Targetable" => targetable = Some(field.bool_value()?),
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActionUseScriptedHook {
    pub usable: bool,
    pub reversed_on_mini_map: Option<bool>,
    pub hidden_on_mini_map: Option<bool>,
    pub version_number: i32,
    pub force_confirmation: bool,
    pub teleport_to_region_entrance: bool,
    pub entrance_connected_to_uid: Option<u64>,
    pub camera_track_uid: Option<u64>,
    pub sound_name: String,
    pub animation_name: String,
    pub replacement_object: i32,
}

impl CTCActionUseScriptedHook {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut usable = None;
        let mut reversed_on_mini_map = None;
        let mut hidden_on_mini_map = None;
        let mut version_number = None;
        let mut force_confirmation = None;
        let mut teleport_to_region_entrance = None;
        let mut camera_track_uid = None;
        let mut entrance_connected_to_uid = None;
        let mut sound_name = None;
        let mut animation_name = None;
        let mut replacement_object = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Usable" => usable = Some(field.bool_value()?),
                "ReversedOnMiniMap" => reversed_on_mini_map = Some(field.bool_value()?),
                "HiddenOnMiniMap" => hidden_on_mini_map = Some(field.bool_value()?),
                "VersionNumber" => version_number = Some(field.integer_value()?),
                "ForceConfirmation" => force_confirmation = Some(field.bool_value()?),
                "TeleportToRegionEntrance" => {
                    teleport_to_region_entrance = Some(field.bool_value()?)
                }
                "CameraTrackUID" => camera_track_uid = Some(field.uid_value()?),
                "EntranceConnectedToUID" => entrance_connected_to_uid = Some(field.uid_value()?),
                "SoundName" => sound_name = Some(field.string_value()?.to_owned()),
                "AnimationName" => animation_name = Some(field.string_value()?.to_owned()),
                "ReplacementObject" => replacement_object = Some(field.integer_value()?),
                "EndCTCActionUseScriptedHook" => {
                    let usable = usable.ok_or_else(|| missing(line, "Usable"))?;
                    let version_number =
                        version_number.ok_or_else(|| missing(line, "VersionNumber"))?;
                    let force_confirmation =
                        force_confirmation.ok_or_else(|| missing(line, "ForceConfirmation"))?;
                    let teleport_to_region_entrance = teleport_to_region_entrance
                        .ok_or_else(|| missing(line, "TeleportToRegionEntrance"))?;
                    let sound_name = sound_name.ok_or_else(|| missing(line, "SoundName"))?;
                    let animation_name =
                        animation_name.ok_or_else(|| missing(line, "AnimationName"))?;
                    let replacement_object =
                        replacement_object.ok_or_else(|| missing(line, "ReplacementObject"))?;

                    return Ok(Self {
                        usable,
                        reversed_on_mini_map,
                        hidden_on_mini_map,
                        version_number,
                        force_confirmation,
                        teleport_to_region_entrance,
                        camera_track_uid,
                        entrance_connected_to_uid,
                        sound_name,
                        animation_name,
                        replacement_object,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCVillageMember {
    pub village_uid: u64,
}

impl CTCVillageMember {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut village_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "VillageUID" => village_uid = Some(field.uid_value()?),
                "EndCTCVillageMember" => {
                    let village_uid = village_uid.ok_or_else(|| missing(line, "VillageUID"))?;

                    return Ok(Self { village_uid });
                }
                _ => Err(UnexpectedField { line })?,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCShop" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCBuyableHouse {
    pub wife_living_here: i32,
    pub owned_by_player: bool,
    pub is_scripted: bool,
    pub rented: bool,
    pub day_next_rent_is_due: i32,
    pub current_dress_level: i32,
    pub virtual_money_bags: i32,
    pub is_residential: Option<bool>,
}

impl CTCBuyableHouse {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut wife_living_here = None;
        let mut owned_by_player = None;
        let mut is_scripted = None;
        let mut rented = None;
        let mut day_next_rent_is_due = None;
        let mut current_dress_level = None;
        let mut virtual_money_bags = None;
        let mut is_residential = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "WifeLivingHere" => wife_living_here = Some(field.integer_value()?),
                "OwnedByPlayer" => owned_by_player = Some(field.bool_value()?),
                "IsScripted" => is_scripted = Some(field.bool_value()?),
                "Rented" => rented = Some(field.bool_value()?),
                "DayNextRentIsDue" => day_next_rent_is_due = Some(field.integer_value()?),
                "CurrentDressLevel" => current_dress_level = Some(field.integer_value()?),
                "VirtualMoneyBags" => virtual_money_bags = Some(field.integer_value()?),
                "IsResidential" => is_residential = Some(field.bool_value()?),
                "EndCTCBuyableHouse" => {
                    let wife_living_here =
                        wife_living_here.ok_or_else(|| missing(line, "WifeLivingHere"))?;
                    let owned_by_player =
                        owned_by_player.ok_or_else(|| missing(line, "OwnedByPlayer"))?;
                    let is_scripted = is_scripted.ok_or_else(|| missing(line, "IsScripted"))?;
                    let rented = rented.ok_or_else(|| missing(line, "Rented"))?;
                    let day_next_rent_is_due =
                        day_next_rent_is_due.ok_or_else(|| missing(line, "DayNextRentIsDue"))?;
                    let current_dress_level =
                        current_dress_level.ok_or_else(|| missing(line, "CurrentDressLevel"))?;
                    let virtual_money_bags =
                        virtual_money_bags.ok_or_else(|| missing(line, "VirtualMoneyBags"))?;

                    return Ok(Self {
                        wife_living_here,
                        owned_by_player,
                        is_scripted,
                        rented,
                        day_next_rent_is_due,
                        current_dress_level,
                        virtual_money_bags,
                        is_residential,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCVillage {
    pub has_been_initially_populated: bool,
    pub frame_player_last_seen_by_guard: i32,
    pub limbo: bool,
    pub is_enemy_because_of_crime: bool,
    pub current_is_hero_criminal: Option<bool>,
}

impl CTCVillage {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut has_been_initially_populated = None;
        let mut frame_player_last_seen_by_guard = None;
        let mut limbo = None;
        let mut is_enemy_because_of_crime = None;
        let mut current_is_hero_criminal = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "HasBeenInitiallyPopulated" => {
                    has_been_initially_populated = Some(field.bool_value()?)
                }
                "FramePlayerLastSeenByGuard" => {
                    frame_player_last_seen_by_guard = Some(field.integer_value()?)
                }
                "Limbo" => limbo = Some(field.bool_value()?),
                "IsEnemyBecauseOfCrime" => is_enemy_because_of_crime = Some(field.bool_value()?),
                "CurrentIsHeroCriminal" => current_is_hero_criminal = Some(field.bool_value()?),
                "EndCTCVillage" => {
                    let has_been_initially_populated = has_been_initially_populated
                        .ok_or_else(|| missing(line, "HasBeenInitiallyPopulated"))?;
                    let frame_player_last_seen_by_guard = frame_player_last_seen_by_guard
                        .ok_or_else(|| missing(line, "FramePlayerLastSeenByGuard"))?;
                    let limbo = limbo.ok_or_else(|| missing(line, "Limbo"))?;
                    let is_enemy_because_of_crime = is_enemy_because_of_crime
                        .ok_or_else(|| missing(line, "IsEnemyBecauseOfCrime"))?;

                    return Ok(Self {
                        has_been_initially_populated,
                        frame_player_last_seen_by_guard,
                        limbo,
                        is_enemy_because_of_crime,
                        current_is_hero_criminal,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCEnemy {
    pub friends_with_everything_flag: bool,
    pub enable_followers_enemy_proxy: Option<bool>,
    pub faction_name: Option<String>,
}

impl CTCEnemy {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut friends_with_everything_flag = None;
        let mut enable_followers_enemy_proxy = None;
        let mut faction_name = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "FriendsWithEverythingFlag" => {
                    friends_with_everything_flag = Some(field.bool_value()?)
                }
                "EnableFollowersEnemyProxy" => {
                    enable_followers_enemy_proxy = Some(field.bool_value()?)
                }
                "FactionName" => faction_name = Some(field.string_value()?.to_owned()),
                "EndCTCEnemy" => {
                    let friends_with_everything_flag = friends_with_everything_flag
                        .ok_or_else(|| missing(line, "FriendsWithEverythingFlag"))?;

                    return Ok(Self {
                        friends_with_everything_flag,
                        enable_followers_enemy_proxy,
                        faction_name,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCreatureOpinionOfHero {
    pub interacted_flag: Option<bool>,
    pub greeted_flag: bool,
    pub last_opinion_reaction_frame: i32,
    pub number_of_times_hit: f32,
    pub tolerance_to_being_hit_override: f32,
    pub frame_to_decay_number_of_times_hit: i32,
    pub forced_attitude: i32,
    pub hero_opinion_enemy: Option<bool>,
}

impl CTCCreatureOpinionOfHero {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut interacted_flag = None;
        let mut greeted_flag = None;
        let mut last_opinion_reaction_frame = None;
        let mut number_of_times_hit = None;
        let mut tolerance_to_being_hit_override = None;
        let mut frame_to_decay_number_of_times_hit = None;
        let mut forced_attitude = None;
        let mut hero_opinion_enemy = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "InteractedFlag" => interacted_flag = Some(field.bool_value()?),
                "GreetedFlag" => greeted_flag = Some(field.bool_value()?),
                "LastOpinionReactionFrame" => {
                    last_opinion_reaction_frame = Some(field.integer_value()?)
                }
                "NumberOfTimesHit" => number_of_times_hit = Some(field.float_value()?),
                "ToleranceToBeingHitOverride" => {
                    tolerance_to_being_hit_override = Some(field.float_value()?)
                }
                "FrameToDecayNumberOfTimesHit" => {
                    frame_to_decay_number_of_times_hit = Some(field.integer_value()?)
                }
                "ForcedAttitude" => forced_attitude = Some(field.integer_value()?),
                "HeroOpinionEnemy" => hero_opinion_enemy = Some(field.bool_value()?),
                "EndCTCCreatureOpinionOfHero" => {
                    let greeted_flag = greeted_flag.ok_or_else(|| missing(line, "GreetedFlag"))?;
                    let last_opinion_reaction_frame = last_opinion_reaction_frame
                        .ok_or_else(|| missing(line, "LastOpinionReactionFrame"))?;
                    let number_of_times_hit =
                        number_of_times_hit.ok_or_else(|| missing(line, "NumberOfTimesHit"))?;
                    let tolerance_to_being_hit_override = tolerance_to_being_hit_override
                        .ok_or_else(|| missing(line, "ToleranceToBeingHitOverride"))?;
                    let frame_to_decay_number_of_times_hit = frame_to_decay_number_of_times_hit
                        .ok_or_else(|| missing(line, "FrameToDecayNumberOfTimesHit"))?;
                    let forced_attitude =
                        forced_attitude.ok_or_else(|| missing(line, "ForcedAttitude"))?;

                    return Ok(Self {
                        interacted_flag,
                        greeted_flag,
                        last_opinion_reaction_frame,
                        number_of_times_hit,
                        tolerance_to_being_hit_override,
                        frame_to_decay_number_of_times_hit,
                        forced_attitude,
                        hero_opinion_enemy,
                    });
                }
                _ => Err(UnexpectedField { line })?,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCTeleporter" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCChest {
    pub container_contents: BTreeMap<usize, String>,
    pub chest_open: bool,
}

impl CTCChest {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut container_contents = BTreeMap::new();
        let mut chest_open = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "ChestOpen" => chest_open = Some(field.bool_value()?),
                "ContainerContents" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    container_contents.insert(index, field.string_value()?.to_owned());
                }
                "EndCTCChest" => {
                    let chest_open = chest_open.ok_or_else(|| missing(line, "ChestOpen"))?;

                    return Ok(Self {
                        container_contents,
                        chest_open,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCSearchableContainer {
    pub container_contents: BTreeMap<usize, String>,
    pub number_of_times_to_search: i32,
}

impl CTCSearchableContainer {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut container_contents = BTreeMap::new();
        let mut number_of_times_to_search = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "NumberOfTimesToSearch" => {
                    number_of_times_to_search = Some(field.integer_value()?);
                }
                "ContainerContents" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    container_contents.insert(index, field.string_value()?.to_owned());
                }
                "EndCTCSearchableContainer" => {
                    let number_of_times_to_search = number_of_times_to_search
                        .ok_or_else(|| missing(line, "NumberOfTimesToSearch"))?;

                    return Ok(Self {
                        container_contents,
                        number_of_times_to_search,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCLight {
    pub active: bool,
    pub overridden: bool,
    pub colour: Option<[u8; 4]>,
    pub inner_radius: Option<f32>,
    pub outer_radius: Option<f32>,
    pub flicker: Option<f32>,
    pub inverted: Option<bool>,
}

impl CTCLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut active = None;
        let mut overridden = None;
        let mut colour = None;
        let mut inner_radius = None;
        let mut outer_radius = None;
        let mut flicker = None;
        let mut inverted = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Active" => active = Some(field.bool_value()?),
                "Overridden" => overridden = Some(field.bool_value()?),
                "Colour" => colour = Some(field.crgbcolour_value()?),
                "InnerRadius" => inner_radius = Some(field.float_value()?),
                "OuterRadius" => outer_radius = Some(field.float_value()?),
                "Flicker" => flicker = Some(field.float_value()?),
                "Inverted" => inverted = Some(field.bool_value()?),
                "EndCTCLight" => {
                    let active = active.ok_or_else(|| missing(line, "Active"))?;
                    let overridden = overridden.ok_or_else(|| missing(line, "Overridden"))?;

                    return Ok(Self {
                        active,
                        overridden,
                        colour,
                        inner_radius,
                        outer_radius,
                        flicker,
                        inverted,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCAtmosPlayer {
    pub atmos_name: String,
}

impl CTCAtmosPlayer {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut atmos_name = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "AtmosName" => atmos_name = Some(field.string_value()?.to_owned()),
                "EndCTCAtmosPlayer" => {
                    let atmos_name = atmos_name.ok_or_else(|| missing(line, "AtmosName"))?;

                    return Ok(Self { atmos_name });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPhysicsNavigator {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rh_set_forward_x: f32,
    pub rh_set_forward_y: f32,
    pub rh_set_forward_z: f32,
    pub rh_set_up_x: f32,
    pub rh_set_up_y: f32,
    pub rh_set_up_z: f32,
}

impl CTCPhysicsNavigator {
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
            let line = field.line;

            match field.key.identifier {
                "PositionX" => position_x = Some(field.float_value()?),
                "PositionY" => position_y = Some(field.float_value()?),
                "PositionZ" => position_z = Some(field.float_value()?),
                "RHSetForwardX" => rh_set_forward_x = Some(field.float_value()?),
                "RHSetForwardY" => rh_set_forward_y = Some(field.float_value()?),
                "RHSetForwardZ" => rh_set_forward_z = Some(field.float_value()?),
                "RHSetUpX" => rh_set_up_x = Some(field.float_value()?),
                "RHSetUpY" => rh_set_up_y = Some(field.float_value()?),
                "RHSetUpZ" => rh_set_up_z = Some(field.float_value()?),
                "EndCTCPhysicsNavigator" => {
                    let position_x = position_x.ok_or_else(|| missing(line, "PositionX"))?;
                    let position_y = position_y.ok_or_else(|| missing(line, "PositionY"))?;
                    let position_z = position_z.ok_or_else(|| missing(line, "PositionZ"))?;
                    let rh_set_forward_x =
                        rh_set_forward_x.ok_or_else(|| missing(line, "RHSetForwardX"))?;
                    let rh_set_forward_y =
                        rh_set_forward_y.ok_or_else(|| missing(line, "RHSetForwardY"))?;
                    let rh_set_forward_z =
                        rh_set_forward_z.ok_or_else(|| missing(line, "RHSetForwardZ"))?;
                    let rh_set_up_x = rh_set_up_x.ok_or_else(|| missing(line, "RHSetUpX"))?;
                    let rh_set_up_y = rh_set_up_y.ok_or_else(|| missing(line, "RHSetUpY"))?;
                    let rh_set_up_z = rh_set_up_z.ok_or_else(|| missing(line, "RHSetUpZ"))?;

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
                _ => Err(UnexpectedField { line })?,
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
            let line = field.line;

            match field.key.identifier {
                "EndCTCTalk" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActionUseBed {
    pub useable_by_hero: bool,
    pub owned_by_hero: bool,
}

impl CTCActionUseBed {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut useable_by_hero = None;
        let mut owned_by_hero = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "UseableByHero" => useable_by_hero = Some(field.bool_value()?),
                "OwnedByHero" => owned_by_hero = Some(field.bool_value()?),
                "EndCTCActionUseBed" => {
                    let useable_by_hero =
                        useable_by_hero.ok_or_else(|| missing(line, "UseableByHero"))?;
                    let owned_by_hero =
                        owned_by_hero.ok_or_else(|| missing(line, "OwnedByHero"))?;

                    return Ok(Self {
                        useable_by_hero,
                        owned_by_hero,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCHeroCentreDoorMarker {
    pub radius: f32,
    pub door_type_2: i32,
}

impl CTCHeroCentreDoorMarker {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut radius = None;
        let mut door_type_2 = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Radius" => radius = Some(field.float_value()?),
                "DoorType2" => door_type_2 = Some(field.integer_value()?),
                "EndCTCHeroCentreDoorMarker" => {
                    let radius = radius.ok_or_else(|| missing(line, "Radius"))?;
                    let door_type_2 = door_type_2.ok_or_else(|| missing(line, "DoorType2"))?;

                    return Ok(Self {
                        radius,
                        door_type_2,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCHero {
    pub last_weapon_equipped_id: i32,
    pub hero_title_object_def_name: String,
}

impl CTCHero {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut last_weapon_equipped_id = None;
        let mut hero_title_object_def_name = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "LastWeaponEquippedID" => last_weapon_equipped_id = Some(field.integer_value()?),
                "hero_title_object_def_name" => {
                    hero_title_object_def_name = Some(field.string_value()?.to_owned())
                }
                "EndCTCHero" => {
                    let last_weapon_equipped_id = last_weapon_equipped_id
                        .ok_or_else(|| missing(line, "LastWeaponEquippedID"))?;

                    let hero_title_object_def_name = hero_title_object_def_name
                        .ok_or_else(|| missing(line, "hero_title_object_def_name"))?;

                    return Ok(Self {
                        last_weapon_equipped_id,
                        hero_title_object_def_name,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCContainerRewardHero {
    pub container_contents: BTreeMap<usize, String>,
}

impl CTCContainerRewardHero {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut container_contents = BTreeMap::new();

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "ContainerContents" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    container_contents.insert(index, field.string_value()?.to_owned());
                }
                "EndCTCContainerRewardHero" => {
                    return Ok(Self { container_contents });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TngKeyCamera {
    pub position: [f32; 3],
    pub look_direction: [f32; 3],
    pub fov: f32,
    pub shuttle_speed: f32,
    pub duration: f32,
    pub pause_time: f32,
    pub event: String,
    pub animation_speed: f32,
    pub roll_angle: f32,
}

impl TngKeyCamera {
    fn parse_by_index(fields: &mut &[KvField], index: i32) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut position = None;
        let mut look_direction = None;
        let mut fov = None;
        let mut shuttle_speed = None;
        let mut duration = None;
        let mut pause_time = None;
        let mut event = None;
        let mut animation_speed = None;
        let mut roll_angle = None;

        let mut line = 0;

        loop {
            let peek_field = fields.first().ok_or_else(|| UnexpectedEnd)?;

            match peek_field.key.identifier {
                "KeyCameras" => {
                    let path = peek_field.path()?;
                    line = peek_field.line;

                    if path.len() != 2 {
                        Err(InvalidPath { line })?;
                    }

                    let field_index = if let Some(KvPathItem::Index(field_index)) = path.get(0) {
                        *field_index
                    } else {
                        Err(InvalidPath { line })?
                    };

                    if field_index != index {
                        break;
                    }

                    let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;

                    let property = if let Some(KvPathItem::Property(prop)) = path.get(1) {
                        *prop
                    } else {
                        Err(InvalidPath { line })?
                    };

                    match property {
                        "Position" => position = Some(field.c3dcoordf_value()?),
                        "LookDirection" => look_direction = Some(field.c3dcoordf_value()?),
                        "FOV" => fov = Some(field.float_value()?),
                        "ShuttleSpeed" => shuttle_speed = Some(field.float_value()?),
                        "Duration" => duration = Some(field.float_value()?),
                        "PauseTime" => pause_time = Some(field.float_value()?),
                        "Event" => event = Some(field.string_value()?.to_owned()),
                        "AnimationSpeed" => animation_speed = Some(field.float_value()?),
                        "RollAngle" => roll_angle = Some(field.float_value()?),
                        _ => {}
                    }
                }
                _ => break,
            }
        }

        let position = position.ok_or_else(|| missing(line, "KeyCamera[I].Position"))?;
        let look_direction =
            look_direction.ok_or_else(|| missing(line, "KeyCamera[I].LookDirection"))?;
        let fov = fov.ok_or_else(|| missing(line, "KeyCamera[I].FOV"))?;
        let shuttle_speed =
            shuttle_speed.ok_or_else(|| missing(line, "KeyCamera[I].ShuttleSpeed"))?;
        let duration = duration.ok_or_else(|| missing(line, "KeyCamera[I].Duration"))?;
        let pause_time = pause_time.ok_or_else(|| missing(line, "KeyCamera[I].PauseTime"))?;
        let event = event.ok_or_else(|| missing(line, "KeyCamera[I].Event"))?;
        let animation_speed =
            animation_speed.ok_or_else(|| missing(line, "KeyCamera[I].AnimationSpeed"))?;
        let roll_angle = roll_angle.ok_or_else(|| missing(line, "KeyCamera[I].RollAngle"))?;

        Ok(Self {
            position,
            look_direction,
            shuttle_speed,
            duration,
            pause_time,
            fov,
            event,
            animation_speed,
            roll_angle,
        })
    }

    fn parse_list(
        fields: &mut &[KvField],
        num_key_cameras: i32,
    ) -> Result<Vec<Self>, CommonFieldError> {
        let mut list = Vec::new();

        for index in 0..num_key_cameras {
            list.push(Self::parse_by_index(fields, index)?);
        }

        Ok(list)
    }
}

#[derive(Clone, Debug)]
pub struct CTCRandomAppearanceMorph {
    pub seed: i32,
}

impl CTCRandomAppearanceMorph {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut seed = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Seed" => seed = Some(field.integer_value()?),
                "EndCTCRandomAppearanceMorph" => {
                    let seed = seed.ok_or_else(|| missing(line, "Seed"))?;
                    return Ok(Self { seed });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCWife {
    pub courting_blocked: bool,
    pub permitted_to_region_follow: bool,
    pub frame_got_married_to_the_player: i32,
    pub divorced_hero: Option<bool>,
    pub just_married: bool,
    pub needs_to_change_brain: bool,
    pub frame_to_check_appearance_changes: i32,
    pub frame_last_aware_of_husband: i32,
    pub frame_last_reduced_opinion: i32,
    pub frame_last_evaluated_gift_opinion: i32,
    pub frame_last_considered_giving_gift: i32,
    pub frame_last_evaluated_love_attitude: i32,
    pub frame_entered_attitude_hate: i32,
    pub frame_last_gave_divorce_warning: i32,
    pub frame_entered_love_with_husband_present_at_home: i32,
    pub frame_last_gave_sex_offer: i32,
    pub gift_giving_opinion_distance_from_max: f32,
    pub gift_giving_price_value: i32,
    pub gift_to_give_def: i32,
    pub last_fatness_change_point: f32,
    pub house_dressing_level_last_commented_on: i32,
    pub boolean_husband_appearances: BTreeMap<usize, bool>,
    pub frame_last_received_nice_gift: i32,
    pub frame_last_culled_gifts_received: i32,
    pub love_attitude_value: f32,
    pub has_been_in_love_with_player: bool,
    pub received_wedding_ring: bool,
}

impl CTCWife {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut courting_blocked = None;
        let mut permitted_to_region_follow = None;
        let mut frame_got_married_to_the_player = None;
        let mut divorced_hero = None;
        let mut just_married = None;
        let mut needs_to_change_brain = None;
        let mut frame_to_check_appearance_changes = None;
        let mut frame_last_aware_of_husband = None;
        let mut frame_last_reduced_opinion = None;
        let mut frame_last_evaluated_gift_opinion = None;
        let mut frame_last_considered_giving_gift = None;
        let mut frame_last_evaluated_love_attitude = None;
        let mut frame_entered_attitude_hate = None;
        let mut frame_last_gave_divorce_warning = None;
        let mut frame_entered_love_with_husband_present_at_home = None;
        let mut frame_last_gave_sex_offer = None;
        let mut gift_giving_opinion_distance_from_max = None;
        let mut gift_giving_price_value = None;
        let mut gift_to_give_def = None;
        let mut last_fatness_change_point = None;
        let mut house_dressing_level_last_commented_on = None;
        let mut boolean_husband_appearances = BTreeMap::new();
        let mut frame_last_received_nice_gift = None;
        let mut frame_last_culled_gifts_received = None;
        let mut love_attitude_value = None;
        let mut has_been_in_love_with_player = None;
        let mut received_wedding_ring = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CourtingBlocked" => courting_blocked = Some(field.bool_value()?),
                "PermittedToRegionFollow" => permitted_to_region_follow = Some(field.bool_value()?),
                "FrameGotMarriedToThePlayer" => {
                    frame_got_married_to_the_player = Some(field.integer_value()?)
                }
                "DivorcedHero" => divorced_hero = Some(field.bool_value()?),
                "JustMarried" => just_married = Some(field.bool_value()?),
                "NeedsToChangeBrain" => needs_to_change_brain = Some(field.bool_value()?),
                "FrameToCheckAppearanceChanges" => {
                    frame_to_check_appearance_changes = Some(field.integer_value()?)
                }
                "FrameLastAwareOfHusband" => {
                    frame_last_aware_of_husband = Some(field.integer_value()?)
                }
                "FrameLastReducedOpinion" => {
                    frame_last_reduced_opinion = Some(field.integer_value()?)
                }
                "FrameLastEvaluatedGiftOpinion" => {
                    frame_last_evaluated_gift_opinion = Some(field.integer_value()?)
                }
                "FrameLastConsideredGivingGift" => {
                    frame_last_considered_giving_gift = Some(field.integer_value()?)
                }
                "FrameLastEvaluatedLoveAttitude" => {
                    frame_last_evaluated_love_attitude = Some(field.integer_value()?)
                }
                "FrameEnteredAttitudeHate" => {
                    frame_entered_attitude_hate = Some(field.integer_value()?)
                }
                "FrameLastGaveDivorceWarning" => {
                    frame_last_gave_divorce_warning = Some(field.integer_value()?)
                }
                "FrameEnteredLoveWithHusbandPresentAtHome" => {
                    frame_entered_love_with_husband_present_at_home = Some(field.integer_value()?)
                }
                "FrameLastGaveSexOffer" => frame_last_gave_sex_offer = Some(field.integer_value()?),
                "GiftGivingOpinionDistanceFromMax" => {
                    gift_giving_opinion_distance_from_max = Some(field.float_value()?)
                }
                "GiftGivingPriceValue" => gift_giving_price_value = Some(field.integer_value()?),
                "GiftToGiveDef" => gift_to_give_def = Some(field.integer_value()?),
                "LastFatnessChangePoint" => last_fatness_change_point = Some(field.float_value()?),
                "HouseDressingLevelLastCommentedOn" => {
                    house_dressing_level_last_commented_on = Some(field.integer_value()?)
                }
                "BooleanHusbandAppearances" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    boolean_husband_appearances.insert(index, field.bool_value()?);
                }
                "FrameLastReceivedNiceGift" => {
                    frame_last_received_nice_gift = Some(field.integer_value()?)
                }
                "FrameLastCulledGiftsReceived" => {
                    frame_last_culled_gifts_received = Some(field.integer_value()?)
                }
                "LoveAttitudeValue" => love_attitude_value = Some(field.float_value()?),
                "HasBeenInLoveWithPlayer" => {
                    has_been_in_love_with_player = Some(field.bool_value()?)
                }
                "ReceivedWeddingRing" => received_wedding_ring = Some(field.bool_value()?),
                "EndCTCWife" => {
                    let courting_blocked =
                        courting_blocked.ok_or_else(|| missing(line, "CourtingBlocked"))?;
                    let permitted_to_region_follow = permitted_to_region_follow
                        .ok_or_else(|| missing(line, "PermittedToRegionFollow"))?;
                    let frame_got_married_to_the_player = frame_got_married_to_the_player
                        .ok_or_else(|| missing(line, "FrameGotMarriedToThePlayer"))?;
                    let just_married = just_married.ok_or_else(|| missing(line, "JustMarried"))?;
                    let needs_to_change_brain =
                        needs_to_change_brain.ok_or_else(|| missing(line, "NeedsToChangeBrain"))?;
                    let frame_to_check_appearance_changes = frame_to_check_appearance_changes
                        .ok_or_else(|| missing(line, "FrameToCheckAppearanceChanges"))?;
                    let frame_last_aware_of_husband = frame_last_aware_of_husband
                        .ok_or_else(|| missing(line, "FrameLastAwareOfHusband"))?;
                    let frame_last_reduced_opinion = frame_last_reduced_opinion
                        .ok_or_else(|| missing(line, "FrameLastReducedOpinion"))?;
                    let frame_last_evaluated_gift_opinion = frame_last_evaluated_gift_opinion
                        .ok_or_else(|| missing(line, "FrameLastEvaluatedGiftOpinion"))?;
                    let frame_last_considered_giving_gift = frame_last_considered_giving_gift
                        .ok_or_else(|| missing(line, "FrameLastConsideredGivingGift"))?;
                    let frame_last_evaluated_love_attitude = frame_last_evaluated_love_attitude
                        .ok_or_else(|| missing(line, "FrameLastEvaluatedLoveAttitude"))?;
                    let frame_entered_attitude_hate = frame_entered_attitude_hate
                        .ok_or_else(|| missing(line, "FrameEnteredAttitudeHate"))?;
                    let frame_last_gave_divorce_warning = frame_last_gave_divorce_warning
                        .ok_or_else(|| missing(line, "FrameLastGaveDivorceWarning"))?;
                    let frame_entered_love_with_husband_present_at_home =
                        frame_entered_love_with_husband_present_at_home.ok_or_else(|| {
                            missing(line, "FrameEnteredLoveWithHusbandPresentAtHome")
                        })?;
                    let frame_last_gave_sex_offer = frame_last_gave_sex_offer
                        .ok_or_else(|| missing(line, "FrameLastGaveSexOffer"))?;
                    let gift_giving_opinion_distance_from_max =
                        gift_giving_opinion_distance_from_max
                            .ok_or_else(|| missing(line, "GiftGivingOpinionDistanceFromMax"))?;
                    let gift_giving_price_value = gift_giving_price_value
                        .ok_or_else(|| missing(line, "GiftGivingPriceValue"))?;
                    let gift_to_give_def =
                        gift_to_give_def.ok_or_else(|| missing(line, "GiftToGiveDef"))?;
                    let last_fatness_change_point = last_fatness_change_point
                        .ok_or_else(|| missing(line, "LastFatnessChangePoint"))?;
                    let house_dressing_level_last_commented_on =
                        house_dressing_level_last_commented_on
                            .ok_or_else(|| missing(line, "HouseDressingLevelLastCommentedOn"))?;
                    let frame_last_received_nice_gift = frame_last_received_nice_gift
                        .ok_or_else(|| missing(line, "FrameLastReceivedNiceGift"))?;
                    let frame_last_culled_gifts_received = frame_last_culled_gifts_received
                        .ok_or_else(|| missing(line, "FrameLastCulledGiftsReceived"))?;
                    let love_attitude_value =
                        love_attitude_value.ok_or_else(|| missing(line, "LoveAttitudeValue"))?;
                    let has_been_in_love_with_player = has_been_in_love_with_player
                        .ok_or_else(|| missing(line, "HasBeenInLoveWithPlayer"))?;
                    let received_wedding_ring = received_wedding_ring
                        .ok_or_else(|| missing(line, "ReceivedWeddingRing"))?;

                    return Ok(Self {
                        courting_blocked,
                        permitted_to_region_follow,
                        frame_got_married_to_the_player,
                        divorced_hero,
                        just_married,
                        needs_to_change_brain,
                        frame_to_check_appearance_changes,
                        frame_last_aware_of_husband,
                        frame_last_reduced_opinion,
                        frame_last_evaluated_gift_opinion,
                        frame_last_considered_giving_gift,
                        frame_last_evaluated_love_attitude,
                        frame_entered_attitude_hate,
                        frame_last_gave_divorce_warning,
                        frame_entered_love_with_husband_present_at_home,
                        frame_last_gave_sex_offer,
                        gift_giving_opinion_distance_from_max,
                        gift_giving_price_value,
                        gift_to_give_def,
                        last_fatness_change_point,
                        house_dressing_level_last_commented_on,
                        boolean_husband_appearances,
                        frame_last_received_nice_gift,
                        frame_last_culled_gifts_received,
                        love_attitude_value,
                        has_been_in_love_with_player,
                        received_wedding_ring,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCInventoryItem {
    pub inventory_uid: u64,
}

impl CTCInventoryItem {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut inventory_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "InventoryUID" => inventory_uid = Some(field.uid_value()?),
                "EndCTCInventoryItem" => {
                    let inventory_uid =
                        inventory_uid.ok_or_else(|| missing(line, "InventoryUID"))?;

                    return Ok(Self { inventory_uid });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCStockItem {
    pub for_sale: bool,
    pub stealable: bool,
    pub price: i32,
}

impl CTCStockItem {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut for_sale = None;
        let mut stealable = None;
        let mut price = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "ForSale" => {
                    for_sale = Some(field.bool_value()?);
                }
                "Stealable" => {
                    stealable = Some(field.bool_value()?);
                }
                "Price" => {
                    price = Some(field.integer_value()?);
                }
                "EndCTCStockItem" => {
                    let for_sale = for_sale.ok_or_else(|| missing(line, "ForSale"))?;
                    let stealable = stealable.ok_or_else(|| missing(line, "Stealable"))?;
                    let price = price.ok_or_else(|| missing(line, "Price"))?;

                    return Ok(Self {
                        for_sale,
                        stealable,
                        price,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCGuard {
    pub frame_pending_crimes_added: i32,
    pub frame_last_bribe_added: i32,
    pub frame_last_crime_seen: i32,
    pub frame_last_received_apology: i32,
    pub bribe_pool: i32,
    pub last_crime_seen_severity: i32,
}

impl CTCGuard {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut frame_pending_crimes_added = None;
        let mut frame_last_bribe_added = None;
        let mut frame_last_crime_seen = None;
        let mut frame_last_received_apology = None;
        let mut bribe_pool = None;
        let mut last_crime_seen_severity = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "FramePendingCrimesAdded" => {
                    frame_pending_crimes_added = Some(field.integer_value()?)
                }
                "FrameLastBribeAdded" => frame_last_bribe_added = Some(field.integer_value()?),
                "FrameLastCrimeSeen" => frame_last_crime_seen = Some(field.integer_value()?),
                "FrameLastReceivedApology" => {
                    frame_last_received_apology = Some(field.integer_value()?)
                }
                "BribePool" => bribe_pool = Some(field.integer_value()?),
                "LastCrimeSeenSeverity" => last_crime_seen_severity = Some(field.integer_value()?),
                "EndCTCGuard" => {
                    let frame_pending_crimes_added = frame_pending_crimes_added
                        .ok_or_else(|| missing(line, "FramePendingCrimesAdded"))?;
                    let frame_last_bribe_added = frame_last_bribe_added
                        .ok_or_else(|| missing(line, "FrameLastBribeAdded"))?;
                    let frame_last_crime_seen =
                        frame_last_crime_seen.ok_or_else(|| missing(line, "FrameLastCrimeSeen"))?;
                    let frame_last_received_apology = frame_last_received_apology
                        .ok_or_else(|| missing(line, "FrameLastReceivedApology"))?;
                    let bribe_pool = bribe_pool.ok_or_else(|| missing(line, "BribePool"))?;
                    let last_crime_seen_severity = last_crime_seen_severity
                        .ok_or_else(|| missing(line, "LastCrimeSeenSeverity"))?;

                    return Ok(Self {
                        frame_pending_crimes_added,
                        frame_last_bribe_added,
                        frame_last_crime_seen,
                        frame_last_received_apology,
                        bribe_pool,
                        last_crime_seen_severity,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCObjectAugmentations {
    pub saved_in_game: bool,
    pub augmentation_def_names: BTreeMap<usize, String>,
}

impl CTCObjectAugmentations {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut saved_in_game = None;
        let mut augmentation_def_names = BTreeMap::new();

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "SavedInGame" => {
                    saved_in_game = Some(field.bool_value()?);
                }
                "AugmentationDefNames" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    augmentation_def_names.insert(index, field.string_value()?.to_owned());
                }
                "EndCTCObjectAugmentations" => {
                    let saved_in_game =
                        saved_in_game.ok_or_else(|| missing(line, "SavedInGame"))?;

                    if augmentation_def_names.is_empty() {
                        Err(missing(line, "AugmentationDefNames"))?;
                    }

                    return Ok(Self {
                        saved_in_game,
                        augmentation_def_names,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCFishingSpot;

impl CTCFishingSpot {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "EndCTCFishingSpot" => {
                    return Ok(Self);
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCInfoDisplay {
    pub text_tag: String,
    pub text_tag_back: String,
    pub radius: f32,
    pub display_time: f32,
}

impl CTCInfoDisplay {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut text_tag = None;
        let mut text_tag_back = None;
        let mut radius = None;
        let mut display_time = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "TextTag" => text_tag = Some(field.string_value()?.to_owned()),
                "TextTagBack" => text_tag_back = Some(field.string_value()?.to_owned()),
                "Radius" => radius = Some(field.float_value()?),
                "DisplayTime" => display_time = Some(field.float_value()?),
                "EndCTCInfoDisplay" => {
                    let text_tag = text_tag.ok_or_else(|| missing(line, "TextTag"))?;
                    let text_tag_back =
                        text_tag_back.ok_or_else(|| missing(line, "TextTagBack"))?;
                    let radius = radius.ok_or_else(|| missing(line, "Radius"))?;
                    let display_time = display_time.ok_or_else(|| missing(line, "DisplayTime"))?;

                    return Ok(Self {
                        text_tag,
                        text_tag_back,
                        radius,
                        display_time,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCreatureGenerator {
    pub creature_families: BTreeMap<usize, String>,
    pub generation_radius: f32,
    pub self_trigger_radius: f32,
    pub self_trigger: bool,
    pub self_trigger_reset_interval: i32,
    pub trigger_on_activate: bool,
    pub active_creature_limit: i32,
    pub total_generation_limit: i32,
    pub num_triggers: i32,
    pub script_name_of_all_generated_creatures: String,
}

impl CTCCreatureGenerator {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        use InvalidPath;

        let mut creature_families = BTreeMap::new();
        let mut generation_radius = None;
        let mut self_trigger_radius = None;
        let mut self_trigger = None;
        let mut self_trigger_reset_interval = None;
        let mut trigger_on_activate = None;
        let mut active_creature_limit = None;
        let mut total_generation_limit = None;
        let mut num_triggers = None;
        let mut script_name_of_all_generated_creatures = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "CreatureFamilies" => {
                    let path = field.path()?;

                    if path.len() != 1 {
                        Err(InvalidPath { line })?
                    }

                    let index = if let Some(KvPathItem::Index(index)) = path.get(0) {
                        let index = *index;

                        if index < 0 {
                            Err(InvalidPath { line })?
                        } else {
                            index as usize
                        }
                    } else {
                        Err(InvalidPath { line })?
                    };

                    creature_families.insert(index, field.string_value()?.to_owned());
                }
                "GenerationRadius" => generation_radius = Some(field.float_value()?),
                "SelfTriggerRadius" => self_trigger_radius = Some(field.float_value()?),
                "SelfTrigger" => self_trigger = Some(field.bool_value()?),
                "SelfTriggerResetInterval" => {
                    self_trigger_reset_interval = Some(field.integer_value()?)
                }
                "TriggerOnActivate" => trigger_on_activate = Some(field.bool_value()?),
                "ActiveCreatureLimit" => active_creature_limit = Some(field.integer_value()?),
                "TotalGenerationLimit" => total_generation_limit = Some(field.integer_value()?),
                "NumTriggers" => num_triggers = Some(field.integer_value()?),
                "ScriptNameOfAllGeneratedCreatures" => {
                    script_name_of_all_generated_creatures = Some(field.string_value()?.to_owned())
                }
                "EndCTCCreatureGenerator" => {
                    if creature_families.is_empty() {
                        Err(missing(line, "CreatureFamilies"))?
                    }
                    let generation_radius =
                        generation_radius.ok_or_else(|| missing(line, "GenerationRadius"))?;
                    let self_trigger_radius =
                        self_trigger_radius.ok_or_else(|| missing(line, "SelfTriggerRadius"))?;
                    let self_trigger = self_trigger.ok_or_else(|| missing(line, "SelfTrigger"))?;
                    let self_trigger_reset_interval = self_trigger_reset_interval
                        .ok_or_else(|| missing(line, "SelfTriggerResetInterval"))?;
                    let trigger_on_activate =
                        trigger_on_activate.ok_or_else(|| missing(line, "TriggerOnActivate"))?;
                    let active_creature_limit = active_creature_limit
                        .ok_or_else(|| missing(line, "ActiveCreatureLimit"))?;
                    let total_generation_limit = total_generation_limit
                        .ok_or_else(|| missing(line, "TotalGenerationLimit"))?;
                    let num_triggers = num_triggers.ok_or_else(|| missing(line, "NumTriggers"))?;
                    let script_name_of_all_generated_creatures =
                        script_name_of_all_generated_creatures
                            .ok_or_else(|| missing(line, "ScriptNameOfAllGeneratedCreatures"))?;

                    return Ok(Self {
                        creature_families,
                        generation_radius,
                        self_trigger_radius,
                        self_trigger,
                        self_trigger_reset_interval,
                        trigger_on_activate,
                        active_creature_limit,
                        total_generation_limit,
                        num_triggers,
                        script_name_of_all_generated_creatures,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActivationReceptorCreatureGenerator {
    pub deactivate_after_set_time: bool,
    pub frames_after_activation_to_deactivate: i32,
    pub activate_on_activate: bool,
    pub trigger_on_activate: bool,
}

impl CTCActivationReceptorCreatureGenerator {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut deactivate_after_set_time = None;
        let mut frames_after_activation_to_deactivate = None;
        let mut activate_on_activate = None;
        let mut trigger_on_activate = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "DeactivateAfterSetTime" => deactivate_after_set_time = Some(field.bool_value()?),
                "FramesAfterActivationToDeactivate" => {
                    frames_after_activation_to_deactivate = Some(field.integer_value()?)
                }
                "ActivateOnActivate" => activate_on_activate = Some(field.bool_value()?),
                "TriggerOnActivate" => trigger_on_activate = Some(field.bool_value()?),
                "EndCTCActivationReceptorCreatureGenerator" => {
                    let deactivate_after_set_time = deactivate_after_set_time
                        .ok_or_else(|| missing(line, "DeactivateAfterSetTime"))?;
                    let frames_after_activation_to_deactivate =
                        frames_after_activation_to_deactivate
                            .ok_or_else(|| missing(line, "FramesAfterActivationToDeactivate"))?;
                    let activate_on_activate =
                        activate_on_activate.ok_or_else(|| missing(line, "ActivateOnActivate"))?;
                    let trigger_on_activate =
                        trigger_on_activate.ok_or_else(|| missing(line, "TriggerOnActivate"))?;

                    return Ok(Self {
                        deactivate_after_set_time,
                        frames_after_activation_to_deactivate,
                        activate_on_activate,
                        trigger_on_activate,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActivationTrigger {
    pub receptor_uid: u64,
}

impl CTCActivationTrigger {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut receptor_uid = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "ReceptorUID" => receptor_uid = Some(field.uid_value()?),
                "EndCTCActivationTrigger" => {
                    let receptor_uid = receptor_uid.ok_or_else(|| missing(line, "ReceptorUID"))?;

                    return Ok(Self { receptor_uid });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCreatureGeneratorCreator;

impl CTCCreatureGeneratorCreator {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "EndCTCCreatureGeneratorCreator" => {
                    return Ok(Self {});
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCSpotLight {
    pub overridden: bool,
    pub colour: [u8; 4],
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub angle: f32,
    pub width: f32,
    pub flicker: f32,
}

impl CTCSpotLight {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut overridden = None;
        let mut colour = None;
        let mut inner_radius = None;
        let mut outer_radius = None;
        let mut angle = None;
        let mut width = None;
        let mut flicker = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Overridden" => overridden = Some(field.bool_value()?),
                "Colour" => colour = Some(field.crgbcolour_value()?),
                "InnerRadius" => inner_radius = Some(field.float_value()?),
                "OuterRadius" => outer_radius = Some(field.float_value()?),
                "Angle" => angle = Some(field.float_value()?),
                "Width" => width = Some(field.float_value()?),
                "Flicker" => flicker = Some(field.float_value()?),
                "EndCTCSpotLight" => {
                    let overridden = overridden.ok_or_else(|| missing(line, "Overridden"))?;
                    let colour = colour.ok_or_else(|| missing(line, "Colour"))?;
                    let inner_radius = inner_radius.ok_or_else(|| missing(line, "InnerRadius"))?;
                    let outer_radius = outer_radius.ok_or_else(|| missing(line, "OuterRadius"))?;
                    let angle = angle.ok_or_else(|| missing(line, "Angle"))?;
                    let width = width.ok_or_else(|| missing(line, "Width"))?;
                    let flicker = flicker.ok_or_else(|| missing(line, "Flicker"))?;

                    return Ok(Self {
                        overridden,
                        colour,
                        inner_radius,
                        outer_radius,
                        angle,
                        width,
                        flicker,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCCarriedActionUseRead {
    pub already_read: bool,
}

impl CTCCarriedActionUseRead {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut already_read = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "AlreadyRead" => already_read = Some(field.bool_value()?),
                "EndCTCCarriedActionUseRead" => {
                    let already_read = already_read.ok_or_else(|| missing(line, "AlreadyRead"))?;

                    return Ok(Self { already_read });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActionUseReadable {
    pub game_text_def_name: String,
}

impl CTCActionUseReadable {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut game_text_def_name = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "GameTextDefName" => game_text_def_name = Some(field.string_value()?.to_owned()),
                "EndCTCActionUseReadable" => {
                    let game_text_def_name =
                        game_text_def_name.ok_or_else(|| missing(line, "GameTextDefName"))?;

                    return Ok(Self { game_text_def_name });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCDiggingSpot {
    pub hidden: bool,
}

impl CTCDiggingSpot {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut hidden = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Hidden" => hidden = Some(field.bool_value()?),
                "EndCTCDiggingSpot" => {
                    let hidden = hidden.ok_or_else(|| missing(line, "Hidden"))?;

                    return Ok(Self { hidden });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCWallMount {
    pub bought_for_amount: i32,
    pub trophy_id: i32,
}

impl CTCWallMount {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut bought_for_amount = None;
        let mut trophy_id = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "BoughtForAmount" => bought_for_amount = Some(field.integer_value()?),
                "TrophyID" => trophy_id = Some(field.integer_value()?),
                "EndCTCWallMount" => {
                    let bought_for_amount =
                        bought_for_amount.ok_or_else(|| missing(line, "BoughtForAmount"))?;
                    let trophy_id = trophy_id.ok_or_else(|| missing(line, "TrophyID"))?;

                    return Ok(Self {
                        bought_for_amount,
                        trophy_id,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCAIScratchpad;

impl CTCAIScratchpad {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "EndCTCAIScratchpad" => {
                    return Ok(Self);
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCPreCalculatedNavigationRoute {
    pub prec_calculated_navigation_route_version: i32,
    pub thing_to_calculate_route_to_uid: u64,
    pub number_of_steps_on_route: i32,
    pub nav_position_0: Option<[f32; 2]>,
    pub nav_layer_0: Option<i32>,
    pub nav_position_1: Option<[f32; 2]>,
    pub nav_layer_1: Option<i32>,
    pub nav_position_2: Option<[f32; 2]>,
    pub nav_layer_2: Option<i32>,
    pub nav_position_3: Option<[f32; 2]>,
    pub nav_layer_3: Option<i32>,
    pub nav_position_4: Option<[f32; 2]>,
    pub nav_layer_4: Option<i32>,
    pub nav_position_5: Option<[f32; 2]>,
    pub nav_layer_5: Option<i32>,
    pub nav_position_6: Option<[f32; 2]>,
    pub nav_layer_6: Option<i32>,
    pub nav_position_7: Option<[f32; 2]>,
    pub nav_layer_7: Option<i32>,
}

impl CTCPreCalculatedNavigationRoute {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut prec_calculated_navigation_route_version = None;
        let mut thing_to_calculate_route_to_uid = None;
        let mut number_of_steps_on_route = None;
        let mut nav_position_0 = None;
        let mut nav_layer_0 = None;
        let mut nav_position_1 = None;
        let mut nav_layer_1 = None;
        let mut nav_position_2 = None;
        let mut nav_layer_2 = None;
        let mut nav_position_3 = None;
        let mut nav_layer_3 = None;
        let mut nav_position_4 = None;
        let mut nav_layer_4 = None;
        let mut nav_position_5 = None;
        let mut nav_layer_5 = None;
        let mut nav_position_6 = None;
        let mut nav_layer_6 = None;
        let mut nav_position_7 = None;
        let mut nav_layer_7 = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "PrecCalculatedNavigationRouteVersion" => {
                    prec_calculated_navigation_route_version = Some(field.integer_value()?)
                }
                "ThingToCalculateRouteToUID" => {
                    thing_to_calculate_route_to_uid = Some(field.uid_value()?)
                }
                "NumberOfStepsOnRoute" => number_of_steps_on_route = Some(field.integer_value()?),
                "NavPosition0" => nav_position_0 = Some(field.c2dcoordf_value()?),
                "NavLayer0" => nav_layer_0 = Some(field.integer_value()?),
                "NavPosition1" => nav_position_1 = Some(field.c2dcoordf_value()?),
                "NavLayer1" => nav_layer_1 = Some(field.integer_value()?),
                "NavPosition2" => nav_position_2 = Some(field.c2dcoordf_value()?),
                "NavLayer2" => nav_layer_2 = Some(field.integer_value()?),
                "NavPosition3" => nav_position_3 = Some(field.c2dcoordf_value()?),
                "NavLayer3" => nav_layer_3 = Some(field.integer_value()?),
                "NavPosition4" => nav_position_4 = Some(field.c2dcoordf_value()?),
                "NavLayer4" => nav_layer_4 = Some(field.integer_value()?),
                "NavPosition5" => nav_position_5 = Some(field.c2dcoordf_value()?),
                "NavLayer5" => nav_layer_5 = Some(field.integer_value()?),
                "NavPosition6" => nav_position_6 = Some(field.c2dcoordf_value()?),
                "NavLayer6" => nav_layer_6 = Some(field.integer_value()?),
                "NavPosition7" => nav_position_7 = Some(field.c2dcoordf_value()?),
                "NavLayer7" => nav_layer_7 = Some(field.integer_value()?),
                "EndCTCPreCalculatedNavigationRoute" => {
                    let prec_calculated_navigation_route_version =
                        prec_calculated_navigation_route_version
                            .ok_or_else(|| missing(line, "PrecCalculatedNavigationRouteVersion"))?;
                    let thing_to_calculate_route_to_uid = thing_to_calculate_route_to_uid
                        .ok_or_else(|| missing(line, "ThingToCalculateRouteToUID"))?;
                    let number_of_steps_on_route = number_of_steps_on_route
                        .ok_or_else(|| missing(line, "NumberOfStepsOnRoute"))?;

                    return Ok(Self {
                        prec_calculated_navigation_route_version,
                        thing_to_calculate_route_to_uid,
                        number_of_steps_on_route,
                        nav_position_0,
                        nav_layer_0,
                        nav_position_1,
                        nav_layer_1,
                        nav_position_2,
                        nav_layer_2,
                        nav_position_3,
                        nav_layer_3,
                        nav_position_4,
                        nav_layer_4,
                        nav_position_5,
                        nav_layer_5,
                        nav_position_6,
                        nav_layer_6,
                        nav_position_7,
                        nav_layer_7,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCExplodingObject {
    pub max_damage: f32,
    pub radius: f32,
    pub fire_damage: i32,
    pub triggered_on_creature_proximity: bool,
    pub trigger_radius: f32,
}

impl CTCExplodingObject {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut max_damage = None;
        let mut radius = None;
        let mut fire_damage = None;
        let mut triggered_on_creature_proximity = None;
        let mut trigger_radius = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "MaxDamage" => max_damage = Some(field.float_value()?),
                "Radius" => radius = Some(field.float_value()?),
                "FireDamage" => fire_damage = Some(field.integer_value()?),
                "TriggeredOnCreatureProximity" => {
                    triggered_on_creature_proximity = Some(field.bool_value()?)
                }
                "TriggerRadius" => trigger_radius = Some(field.float_value()?),
                "EndCTCExplodingObject" => {
                    let max_damage = max_damage.ok_or_else(|| missing(line, "MaxDamage"))?;
                    let radius = radius.ok_or_else(|| missing(line, "Radius"))?;
                    let fire_damage = fire_damage.ok_or_else(|| missing(line, "FireDamage"))?;
                    let triggered_on_creature_proximity = triggered_on_creature_proximity
                        .ok_or_else(|| missing(line, "TriggeredOnCreatureProximity"))?;
                    let trigger_radius =
                        trigger_radius.ok_or_else(|| missing(line, "TriggerRadius"))?;

                    return Ok(Self {
                        max_damage,
                        radius,
                        fire_damage,
                        triggered_on_creature_proximity,
                        trigger_radius,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCStealableItemLocation {
    pub radius_to_be_within: f32,
    pub radius_to_take_items_back_to: f32,
}

impl CTCStealableItemLocation {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut radius_to_be_within = None;
        let mut radius_to_take_items_back_to = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "RadiusToBeWithin" => radius_to_be_within = Some(field.float_value()?),
                "RadiusToTakeItemsBackTo" => {
                    radius_to_take_items_back_to = Some(field.float_value()?)
                }
                "EndCTCStealableItemLocation" => {
                    let radius_to_be_within =
                        radius_to_be_within.ok_or_else(|| missing(line, "RadiusToBeWithin"))?;
                    let radius_to_take_items_back_to = radius_to_take_items_back_to
                        .ok_or_else(|| missing(line, "RadiusToTakeItemsBackTo"))?;

                    return Ok(Self {
                        radius_to_be_within,
                        radius_to_take_items_back_to,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCActivationReceptorDoor {
    pub deactivate_after_set_time: bool,
    pub frames_after_activation_to_deactivate: i32,
}

impl CTCActivationReceptorDoor {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut deactivate_after_set_time = None;
        let mut frames_after_activation_to_deactivate = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "DeactivateAfterSetTime" => deactivate_after_set_time = Some(field.bool_value()?),
                "FramesAfterActivationToDeactivate" => {
                    frames_after_activation_to_deactivate = Some(field.integer_value()?)
                }
                "EndCTCActivationReceptorDoor" => {
                    let deactivate_after_set_time = deactivate_after_set_time
                        .ok_or_else(|| missing(line, "DeactivateAfterSetTime"))?;
                    let frames_after_activation_to_deactivate =
                        frames_after_activation_to_deactivate
                            .ok_or_else(|| missing(line, "FramesAfterActivationToDeactivate"))?;

                    return Ok(Self {
                        deactivate_after_set_time,
                        frames_after_activation_to_deactivate,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCBoastingArea {
    pub radius: f32,
}

impl CTCBoastingArea {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut radius = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "Radius" => radius = Some(field.float_value()?),
                "EndCTCBoastingArea" => {
                    let radius = radius.ok_or_else(|| missing(line, "Radius"))?;

                    return Ok(Self { radius });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CTCTrophy {
    pub best_witnesses_ahead_to_date: i32,
    pub mountable: bool,
}

impl CTCTrophy {
    fn parse(fields: &mut &[KvField]) -> Result<Self, CommonFieldError> {
        let mut best_witnesses_ahead_to_date = None;
        let mut mountable = None;

        loop {
            let field = fields.grab_first().ok_or_else(|| UnexpectedEnd)?;
            let line = field.line;

            match field.key.identifier {
                "BestWitnessesAheadToDate" => {
                    best_witnesses_ahead_to_date = Some(field.integer_value()?)
                }
                "Mountable" => mountable = Some(field.bool_value()?),
                "EndCTCTrophy" => {
                    let best_witnesses_ahead_to_date = best_witnesses_ahead_to_date
                        .ok_or_else(|| missing(line, "BestWitnessesAheadToDate"))?;
                    let mountable = mountable.ok_or_else(|| missing(line, "Mountable"))?;

                    return Ok(Self {
                        best_witnesses_ahead_to_date,
                        mountable,
                    });
                }
                _ => Err(UnexpectedField { line })?,
            }
        }
    }
}
