//! Fable's `.tng` "Things" files: everything standing on a level's landscape.
//!
//! A `.tng` is a [`crate::text`] body of `XXXSectionStart … XXXSectionEnd`
//! sections holding `NewThing … EndThing` blocks. A thing carries top-level
//! fields and [`Components`] — the `StartCTC… … EndCTC…` blocks, one per
//! `CTCBase` subclass with per-instance state.
//!
//! # Shape
//!
//! [`TngThing`] is an enum over the nine kinds the game writes, because the kind
//! is a real closed set: the game emits `"NewThing " + CThing::GetTypeName()`
//! (`fablelib/thing_manager.cpp:5638`) from a type registry, and three of the
//! nine subclasses add fields of their own. The rest of a thing decomposes along
//! the class hierarchy:
//!
//! - **`CThing`** — [`ThingBase`]: `UID`, `Player`, `DefinitionType`,
//!   `ScriptName`, `ScriptData`, the two persistence flags, `CreateTC`. On all
//!   21,800 things in the game.
//! - **`CThingPhysical`** — [`PhysicalBase`]: adds `Health`, `ObjectScale`,
//!   `CanComeBetweenCameraAndHero` (`fablelib/thing_physical.cpp:496`). On every
//!   kind except the bare [`PlainThing`], which is why that one is the odd
//!   variant out.
//! - **the subclass** — only [`AiCreatureThing`], [`TrackNodeThing`] and
//!   [`SwitchThing`] add anything.
//!
//! Nothing is untyped and nothing is dropped: every field of every thing in
//! `FinalAlbion.wad` reads into one of these structs, and a field, component
//! class or thing kind that is not modelled is a [`TngError`] rather than a
//! silent loss.

pub mod component;
pub mod read;

pub use self::component::{
    ActionUse, ActivationReceptor, CameraPoint, CameraPointBase, ComponentError, ComponentRef,
    Components, Container, ContainerBase, Driver, KeyCamera, NavStep, Physics, PhysicsLight,
    PhysicsStandard, Shape,
};
pub use self::read::{FieldErrorKind, Redundant, Rgba, RightHandedSet};

use self::read::ReadField;
use crate::text::{self, PathSegment, Statement, TextError};
use derive_more::{Display, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct Tng {
    pub sections: Vec<TngSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TngSection {
    /// The `XXXSectionStart` name — a quest or village script, `NULL` for the
    /// unowned section. Empty for things written outside any section.
    pub name: String,
    pub things: Vec<TngThing>,
}

/// One `NewThing … EndThing`, by kind.
#[derive(Debug, Clone, PartialEq)]
pub enum TngThing {
    Object(ObjectThing),
    Marker(MarkerThing),
    /// `NewThing Thing` — a bare `CThing`.
    Plain(PlainThing),
    Building(BuildingThing),
    HolySite(HolySiteThing),
    Village(VillageThing),
    AiCreature(AiCreatureThing),
    TrackNode(TrackNodeThing),
    Switch(SwitchThing),
}

/// `CThing`'s own serialised fields, on every thing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThingBase {
    pub uid: u64,
    /// `-1` when the thing is unowned.
    pub player: i32,
    /// The `OBJECT_*` / `CREATURE_*` / … def this thing instantiates.
    pub definition_type: String,
    pub script_name: String,
    pub script_data: String,
    pub game_persistent: bool,
    pub level_persistent: bool,
    /// Components the placement adds that its def did not declare. Repeats, and
    /// is the only route by which `CTCActionUseScriptedHook` reaches a thing.
    pub create_tc: Vec<String>,
    pub components: Components,
}

/// `CThingPhysical` — `CThing` plus the fields a thing with a body has.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhysicalBase {
    pub thing: ThingBase,
    pub health: Option<f32>,
    /// A multiplier on the thing's rendered size, defaulting to 1.
    pub object_scale: Option<f32>,
    /// Written as a number, not `TRUE`/`FALSE`, on the 28 objects that set it.
    pub can_come_between_camera_and_hero: Option<i32>,
}

macro_rules! plain_kinds {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Default, PartialEq)]
            pub struct $name {
                pub base: PhysicalBase,
            }
        )*
    };
}

plain_kinds! {
    /// `NewThing Object` — 11,964 in the game. Adds no fields of its own.
    ObjectThing,
    /// `NewThing Marker` — 5,177 in the game. Adds no fields of its own.
    MarkerThing,
    /// `NewThing Building` — 291 in the game. Adds no fields of its own.
    BuildingThing,
    /// `NewThing Holy Site` — 129 in the game. Adds no fields of its own.
    ///
    /// The only kind whose written name contains a space: it is a display name
    /// from `GetTypeName()`, not an identifier.
    HolySiteThing,
    /// `NewThing Village` — 51 in the game. Adds no fields of its own.
    VillageThing,
}

/// `NewThing Thing` — 3,049 in the game: camera points, region entrances and
/// exits, particle emitters.
///
/// The only kind that is a bare `CThing` rather than a `CThingPhysical`, and so
/// the only one with no `Health`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlainThing {
    pub base: ThingBase,
}

/// `NewThing AICreature` — 819 in the game, plus what `CThingAICreature` adds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AiCreatureThing {
    pub base: PhysicalBase,
    pub allowed_to_follow_hero: bool,
    pub can_be_courted: bool,
    pub can_be_married: bool,
    pub continue_ai_with_information: Option<bool>,
    pub enable_creature_auto_placing: bool,
    pub has_information: bool,
    pub home_building_uid: Option<u64>,
    /// Where the creature was first placed, in **world** coordinates rather than
    /// the level-local ones `CTCPhysicsNavigator` uses.
    pub initial_pos: Option<[f32; 3]>,
    pub overriding_brain_name: String,
    pub region_following_overridden_from_script: Option<bool>,
    pub responding_to_follow_and_wait: Option<bool>,
    pub wander_with_information: bool,
    pub wave_with_information: bool,
    pub work_building_uid: Option<u64>,
}

/// `NewThing TrackNode` — 319 in the game. A node in a patrol route, doubly
/// linked to its neighbours.
///
/// `Start` and `End` are ordinary fields despite reading like block keywords;
/// the grammar only treats a *bare* `Start…` as opening a block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackNodeThing {
    pub base: PhysicalBase,
    pub start: bool,
    pub end: bool,
    pub linked_to_uid1: u64,
    pub linked_to_uid2: u64,
}

/// `NewThing Switch` — 1 in the game, an environment-changing area trigger.
///
/// `CThingSwitch` (`fablelib/thing_player_creature.hpp:1052-1060`) declares
/// `TriggerRadius`, `EnvironmentDef`, `TimeToChangeEnvironmentDef` and
/// `TriggerableObjectScriptName` — the last of which is what `TriggeredByThing`
/// writes, so it is a script name and not a UID.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchThing {
    pub base: PhysicalBase,
    pub environment_def: String,
    pub time_to_change_environment_def: f32,
    pub trigger_radius: f32,
    pub triggered_by_thing: String,
}

/// A thing's placement: where it is and which way it faces.
///
/// From whichever `CTCPhysicsBase` subclass the thing carries. Z-up, in world
/// units — one landscape cell is 1.0 (AGENTS.md §3.6).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Placement {
    pub position: [f32; 3],
    /// `None` for `CTCPhysicsLight`, which writes a position and no orientation.
    pub orientation: Option<RightHandedSet>,
}

// ── Accessors ─────────────────────────────────────────────────────────────────

impl TngThing {
    /// The name written after `NewThing`.
    pub fn type_name(&self) -> &'static str {
        match self {
            TngThing::Object(_) => "Object",
            TngThing::Marker(_) => "Marker",
            TngThing::Plain(_) => "Thing",
            TngThing::Building(_) => "Building",
            TngThing::HolySite(_) => "Holy Site",
            TngThing::Village(_) => "Village",
            TngThing::AiCreature(_) => "AICreature",
            TngThing::TrackNode(_) => "TrackNode",
            TngThing::Switch(_) => "Switch",
        }
    }

    /// The `CThing` fields, which every kind has.
    pub fn base(&self) -> &ThingBase {
        match self {
            TngThing::Plain(t) => &t.base,
            _ => &self.physical().expect("every other kind is physical").thing,
        }
    }

    /// The `CThingPhysical` fields, for the eight kinds that have them.
    pub fn physical(&self) -> Option<&PhysicalBase> {
        Some(match self {
            TngThing::Object(t) => &t.base,
            TngThing::Marker(t) => &t.base,
            TngThing::Building(t) => &t.base,
            TngThing::HolySite(t) => &t.base,
            TngThing::Village(t) => &t.base,
            TngThing::AiCreature(t) => &t.base,
            TngThing::TrackNode(t) => &t.base,
            TngThing::Switch(t) => &t.base,
            TngThing::Plain(_) => return None,
        })
    }

    pub fn components(&self) -> &Components {
        &self.base().components
    }

    /// Where the thing stands, from its physics component.
    ///
    /// `None` for the things with no physics at all.
    pub fn placement(&self) -> Option<Placement> {
        let physics = self.components().physics.as_ref()?;
        Some(Placement {
            position: physics.position(),
            orientation: physics.right_handed_set(),
        })
    }
}

impl std::ops::Deref for PhysicalBase {
    type Target = ThingBase;

    fn deref(&self) -> &ThingBase {
        &self.thing
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Display, Error, PartialEq)]
pub enum TngError {
    /// The file does not parse as level text at all.
    Text(TextError),
    /// A statement parsed but has nowhere to go in the typed model.
    Thing(ThingError),
}

/// A statement that could not be read into the typed model.
#[derive(Debug, Clone, Display, Error, PartialEq)]
#[display("{line}:{column}: {kind}")]
pub struct ThingError {
    pub line: usize,
    pub column: usize,
    pub kind: ThingErrorKind,
}

#[derive(Debug, Clone, Display, PartialEq)]
pub enum ThingErrorKind {
    #[display("unknown thing kind `{_0}`")]
    UnknownKind(String),
    #[display("unknown component class `{class}` in thing `{thing}`")]
    UnknownComponent { thing: String, class: String },
    #[display("`{class}` appears twice on this thing")]
    DuplicateComponent { class: String },
    #[display("`{class}` conflicts with `{existing}` — both are one interface")]
    ConflictingComponent {
        class: String,
        existing: &'static str,
    },
    #[display("{kind}: `{path}` in {owner}")]
    Field {
        owner: String,
        path: String,
        kind: FieldErrorKind,
    },
}

// ── Parsing ───────────────────────────────────────────────────────────────────

impl Tng {
    pub fn parse(input: &str) -> Result<Tng, TngError> {
        let body = text::parse(input).map_err(TngError::Text)?;

        let mut sections = Vec::new();
        // Things written outside any section (v1 files) collect into one unnamed
        // section, so a caller can iterate sections without special-casing.
        let mut unsectioned: Option<TngSection> = None;

        for statement in &body.statements {
            let Statement::Block(block) = &statement.value else {
                continue;
            };
            match block.keyword {
                "XXXSectionStart" => {
                    let mut things = Vec::new();
                    for thing in block.body.blocks_named("NewThing") {
                        things.push(read_thing(thing, input)?);
                    }
                    sections.push(TngSection {
                        name: block.kind.map_or(String::new(), |k| k.value.to_string()),
                        things,
                    });
                }
                "NewThing" => unsectioned
                    .get_or_insert_with(|| TngSection {
                        name: String::new(),
                        things: Vec::new(),
                    })
                    .things
                    .push(read_thing(block, input)?),
                _ => {}
            }
        }

        sections.extend(unsectioned);

        Ok(Tng { sections })
    }

    /// Every thing in the file, in order, ignoring section boundaries.
    pub fn things(&self) -> impl Iterator<Item = &TngThing> {
        self.sections.iter().flat_map(|s| s.things.iter())
    }
}

/// A thing under construction: the shared bases plus whichever kind it is.
///
/// Reading is one pass over the statements, and each field is offered to the
/// most derived level first — subclass, then `CThingPhysical`, then `CThing` —
/// mirroring how the game's serialiser layers `OnSerialise` overrides.
enum ThingBuilder {
    Object(ObjectThing),
    Marker(MarkerThing),
    Plain(PlainThing),
    Building(BuildingThing),
    HolySite(HolySiteThing),
    Village(VillageThing),
    AiCreature(AiCreatureThing),
    TrackNode(TrackNodeThing),
    Switch(SwitchThing),
}

impl ThingBuilder {
    fn new(type_name: &str) -> Option<ThingBuilder> {
        Some(match type_name {
            "Object" => ThingBuilder::Object(ObjectThing::default()),
            "Marker" => ThingBuilder::Marker(MarkerThing::default()),
            "Thing" => ThingBuilder::Plain(PlainThing::default()),
            "Building" => ThingBuilder::Building(BuildingThing::default()),
            "Holy Site" => ThingBuilder::HolySite(HolySiteThing::default()),
            "Village" => ThingBuilder::Village(VillageThing::default()),
            "AICreature" => ThingBuilder::AiCreature(AiCreatureThing::default()),
            "TrackNode" => ThingBuilder::TrackNode(TrackNodeThing::default()),
            "Switch" => ThingBuilder::Switch(SwitchThing::default()),
            _ => return None,
        })
    }

    fn finish(self) -> TngThing {
        match self {
            ThingBuilder::Object(t) => TngThing::Object(t),
            ThingBuilder::Marker(t) => TngThing::Marker(t),
            ThingBuilder::Plain(t) => TngThing::Plain(t),
            ThingBuilder::Building(t) => TngThing::Building(t),
            ThingBuilder::HolySite(t) => TngThing::HolySite(t),
            ThingBuilder::Village(t) => TngThing::Village(t),
            ThingBuilder::AiCreature(t) => TngThing::AiCreature(t),
            ThingBuilder::TrackNode(t) => TngThing::TrackNode(t),
            ThingBuilder::Switch(t) => TngThing::Switch(t),
        }
    }

    fn thing_base(&mut self) -> &mut ThingBase {
        match self {
            ThingBuilder::Plain(t) => &mut t.base,
            _ => {
                &mut self
                    .physical_base()
                    .expect("every other kind is physical")
                    .thing
            }
        }
    }

    fn physical_base(&mut self) -> Option<&mut PhysicalBase> {
        Some(match self {
            ThingBuilder::Object(t) => &mut t.base,
            ThingBuilder::Marker(t) => &mut t.base,
            ThingBuilder::Building(t) => &mut t.base,
            ThingBuilder::HolySite(t) => &mut t.base,
            ThingBuilder::Village(t) => &mut t.base,
            ThingBuilder::AiCreature(t) => &mut t.base,
            ThingBuilder::TrackNode(t) => &mut t.base,
            ThingBuilder::Switch(t) => &mut t.base,
            ThingBuilder::Plain(_) => return None,
        })
    }

    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &text::Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // The subclass gets first refusal on the name.
        let subclass = match self {
            ThingBuilder::AiCreature(t) => read_ai_creature(t, name, rest, value),
            ThingBuilder::TrackNode(t) => read_track_node(t, name, rest, value),
            ThingBuilder::Switch(t) => read_switch(t, name, rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        };
        if !matches!(subclass, Err(FieldErrorKind::UnknownField)) {
            return subclass;
        }

        // Then `CThingPhysical`.
        if let Some(physical) = self.physical_base() {
            match name {
                "Health" => return physical.health.read_field(rest, value),
                "ObjectScale" => return physical.object_scale.read_field(rest, value),
                "CanComeBetweenCameraAndHero" => {
                    return physical
                        .can_come_between_camera_and_hero
                        .read_field(rest, value);
                }
                _ => {}
            }
        }

        // Then `CThing`.
        let base = self.thing_base();
        match name {
            "UID" => base.uid.read_field(rest, value),
            "Player" => base.player.read_field(rest, value),
            "DefinitionType" => base.definition_type.read_field(rest, value),
            "ScriptName" => base.script_name.read_field(rest, value),
            "ScriptData" => base.script_data.read_field(rest, value),
            "ThingGamePersistent" => base.game_persistent.read_field(rest, value),
            "ThingLevelPersistent" => base.level_persistent.read_field(rest, value),
            // Repeats and is written without an index, so it appends.
            "CreateTC" => {
                let mut name = String::new();
                name.read_field(rest, value)?;
                base.create_tc.push(name);
                Ok(())
            }
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

fn read_ai_creature(
    t: &mut AiCreatureThing,
    name: &str,
    rest: &[PathSegment<'_>],
    value: &text::Value<'_>,
) -> Result<(), FieldErrorKind> {
    match name {
        "AllowedToFollowHero" => t.allowed_to_follow_hero.read_field(rest, value),
        "CanBeCourted" => t.can_be_courted.read_field(rest, value),
        "CanBeMarried" => t.can_be_married.read_field(rest, value),
        "ContinueAIWithInformation" => t.continue_ai_with_information.read_field(rest, value),
        "EnableCreatureAutoPlacing" => t.enable_creature_auto_placing.read_field(rest, value),
        "HasInformation" => t.has_information.read_field(rest, value),
        "HomeBuildingUID" => t.home_building_uid.read_field(rest, value),
        "InitialPosX" => t.initial_pos.get_or_insert_default()[0].read_field(rest, value),
        "InitialPosY" => t.initial_pos.get_or_insert_default()[1].read_field(rest, value),
        "InitialPosZ" => t.initial_pos.get_or_insert_default()[2].read_field(rest, value),
        "OverridingBrainName" => t.overriding_brain_name.read_field(rest, value),
        "RegionFollowingOverriddenFromScript" => t
            .region_following_overridden_from_script
            .read_field(rest, value),
        "RespondingToFollowAndWait" => t.responding_to_follow_and_wait.read_field(rest, value),
        "WanderWithInformation" => t.wander_with_information.read_field(rest, value),
        "WaveWithInformation" => t.wave_with_information.read_field(rest, value),
        "WorkBuildingUID" => t.work_building_uid.read_field(rest, value),
        _ => Err(FieldErrorKind::UnknownField),
    }
}

fn read_track_node(
    t: &mut TrackNodeThing,
    name: &str,
    rest: &[PathSegment<'_>],
    value: &text::Value<'_>,
) -> Result<(), FieldErrorKind> {
    match name {
        "Start" => t.start.read_field(rest, value),
        "End" => t.end.read_field(rest, value),
        "LinkedToUID1" => t.linked_to_uid1.read_field(rest, value),
        "LinkedToUID2" => t.linked_to_uid2.read_field(rest, value),
        _ => Err(FieldErrorKind::UnknownField),
    }
}

fn read_switch(
    t: &mut SwitchThing,
    name: &str,
    rest: &[PathSegment<'_>],
    value: &text::Value<'_>,
) -> Result<(), FieldErrorKind> {
    match name {
        "EnvironmentDef" => t.environment_def.read_field(rest, value),
        "TimeToChangeEnvironmentDef" => t.time_to_change_environment_def.read_field(rest, value),
        "TriggerRadius" => t.trigger_radius.read_field(rest, value),
        "TriggeredByThing" => t.triggered_by_thing.read_field(rest, value),
        _ => Err(FieldErrorKind::UnknownField),
    }
}

fn read_thing(block: &text::Block<'_>, source: &str) -> Result<TngThing, TngError> {
    let type_name = block.kind.map_or("", |k| k.value);
    let at = |offset: usize, kind: ThingErrorKind| -> TngError {
        let (line, column) = line_column(source, offset);
        TngError::Thing(ThingError { line, column, kind })
    };

    let mut building = ThingBuilder::new(type_name).ok_or_else(|| {
        at(
            block.kind.map_or(0, |k| k.span.start),
            ThingErrorKind::UnknownKind(type_name.to_string()),
        )
    })?;
    // A thing with no `Player` is unowned, which the format writes as -1.
    building.thing_base().player = -1;

    for statement in &block.body.statements {
        let start = statement.span.start;
        match &statement.value {
            Statement::Field(field) => {
                let result = match field.path.segments.as_slice() {
                    [PathSegment::Field(name), rest @ ..] => {
                        building.read_named(name, rest, &field.value.value)
                    }
                    _ => Err(FieldErrorKind::UnknownField),
                };
                result.map_err(|kind| {
                    at(
                        start,
                        ThingErrorKind::Field {
                            owner: format!("thing `{type_name}`"),
                            path: field.path.to_string(),
                            kind,
                        },
                    )
                })?;
            }
            Statement::Flag(name) => {
                return Err(at(
                    start,
                    ThingErrorKind::Field {
                        owner: format!("thing `{type_name}`"),
                        path: (*name).to_string(),
                        kind: FieldErrorKind::MissingValue,
                    },
                ));
            }
            Statement::Block(component) => {
                let class = component
                    .keyword
                    .strip_prefix("Start")
                    .unwrap_or(component.keyword);
                building
                    .thing_base()
                    .components
                    .read_block(class, &component.body)
                    .map_err(|error| {
                        let start = match &error {
                            ComponentError::Field { offset, .. } => *offset,
                            _ => start,
                        };
                        at(
                            start,
                            match error {
                                ComponentError::UnknownClass => ThingErrorKind::UnknownComponent {
                                    thing: type_name.to_string(),
                                    class: class.to_string(),
                                },
                                ComponentError::Duplicate => ThingErrorKind::DuplicateComponent {
                                    class: class.to_string(),
                                },
                                ComponentError::Conflict { existing } => {
                                    ThingErrorKind::ConflictingComponent {
                                        class: class.to_string(),
                                        existing,
                                    }
                                }
                                ComponentError::Field { path, kind, .. } => ThingErrorKind::Field {
                                    owner: format!("`{class}`"),
                                    path,
                                    kind,
                                },
                            },
                        )
                    })?;
            }
        }
    }

    Ok(building.finish())
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
    let column = before.rfind('\n').map_or(offset + 1, |nl| offset - nl);
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKER: &str = "Version 2;\r\n\
        XXXSectionStart NULL;\r\n\
        NewThing Marker;\r\n\
        Player 4;\r\n\
        UID 18446741874686301490;\r\n\
        DefinitionType \"MARKER_BASIC\";\r\n\
        ScriptName VC;\r\n\
        ScriptData \"NULL\";\r\n\
        ThingGamePersistent FALSE;\r\n\
        ThingLevelPersistent FALSE;\r\n\
        StartCTCPhysicsStandard;\r\n\
        PositionX 10.0;\r\n\
        PositionY 20.0;\r\n\
        PositionZ 30.0;\r\n\
        RHSetForwardX 1.0;\r\n\
        RHSetForwardY 0.0;\r\n\
        RHSetForwardZ 0.0;\r\n\
        RHSetUpX 0.0;\r\n\
        RHSetUpY 0.0;\r\n\
        RHSetUpZ 1.0;\r\n\
        EndCTCPhysicsStandard;\r\n\
        StartCTCEditor;\r\n\
        EndCTCEditor;\r\n\
        Health 1.0;\r\n\
        EndThing;\r\n\
        XXXSectionEnd;\r\n";

    fn only_thing(input: &str) -> TngThing {
        Tng::parse(input)
            .expect("parse")
            .things()
            .next()
            .expect("a thing")
            .clone()
    }

    #[test]
    fn parses_a_marker() {
        let tng = Tng::parse(MARKER).unwrap();
        assert_eq!(tng.sections.len(), 1);
        assert_eq!(tng.sections[0].name, "NULL");

        let thing = &tng.sections[0].things[0];
        assert_eq!(thing.type_name(), "Marker");
        assert!(matches!(thing, TngThing::Marker(_)));

        let base = thing.base();
        assert_eq!(base.player, 4);
        assert_eq!(base.uid, 18446741874686301490);
        assert_eq!(base.definition_type, "MARKER_BASIC");
        assert_eq!(base.script_name, "VC");
        assert_eq!(base.script_data, "NULL");
        assert_eq!(thing.physical().unwrap().health, Some(1.0));
        assert_eq!(thing.physical().unwrap().object_scale, None);

        assert_eq!(
            thing.placement(),
            Some(Placement {
                position: [10.0, 20.0, 30.0],
                orientation: Some(RightHandedSet {
                    forward: [1.0, 0.0, 0.0],
                    up: [0.0, 0.0, 1.0],
                }),
            })
        );
    }

    #[test]
    fn components_land_in_named_slots() {
        let thing = only_thing(MARKER);
        let components = thing.components();
        assert!(matches!(components.physics, Some(Physics::Standard(_))));
        assert_eq!(components.editor.as_ref().unwrap().locked_in_place, None);
        assert!(components.light.is_none());
        // Iterating yields them in the order the game writes them.
        assert_eq!(
            components.iter().map(|c| c.class()).collect::<Vec<_>>(),
            vec!["CTCPhysicsStandard", "CTCEditor"]
        );
    }

    #[test]
    fn a_physical_base_derefs_to_the_thing_base() {
        let thing = only_thing(MARKER);
        let physical = thing.physical().unwrap();
        assert_eq!(physical.definition_type, "MARKER_BASIC");
    }

    #[test]
    fn the_bare_thing_kind_has_no_physical_fields() {
        let thing =
            only_thing("NewThing Thing;\nDefinitionType \"CAMERA_POINT_SCRIPTED\";\nEndThing;\n");
        assert!(matches!(thing, TngThing::Plain(_)));
        assert!(thing.physical().is_none());
        assert_eq!(thing.base().definition_type, "CAMERA_POINT_SCRIPTED");
    }

    #[test]
    fn track_node_subclass_fields() {
        let thing = only_thing(
            "NewThing TrackNode;\n\
             DefinitionType \"TRACK_NODE_BASIC\";\n\
             ScriptName GuardTrack;\n\
             LinkedToUID1 18446741874686301885;\n\
             Start FALSE;\n\
             End FALSE;\n\
             ScriptName GuardTrack;\n\
             EndThing;\n",
        );
        let TngThing::TrackNode(node) = &thing else {
            panic!("expected a TrackNode, got {}", thing.type_name())
        };
        assert!(!node.start);
        assert!(!node.end);
        assert_eq!(node.linked_to_uid1, 18446741874686301885);
        // Written twice; the game's reader assigns as it goes, so the last wins.
        assert_eq!(node.base.script_name, "GuardTrack");
    }

    #[test]
    fn ai_creature_subclass_fields() {
        let thing = only_thing(
            "NewThing AICreature;\n\
             DefinitionType \"CREATURE_BS_GUARD\";\n\
             OverridingBrainName BRAIN_SHEEP;\n\
             CanBeMarried TRUE;\n\
             InitialPosX 1234.5;\n\
             InitialPosZ 12.0;\n\
             EndThing;\n",
        );
        let TngThing::AiCreature(creature) = &thing else {
            panic!("expected an AICreature")
        };
        assert_eq!(creature.overriding_brain_name, "BRAIN_SHEEP");
        assert!(creature.can_be_married);
        assert_eq!(creature.initial_pos, Some([1234.5, 0.0, 12.0]));
        assert_eq!(creature.work_building_uid, None);
    }

    #[test]
    fn create_tc_accumulates() {
        let thing = only_thing(
            "NewThing Object;\n\
             CreateTC \"CTCOwnedEntity\";\n\
             CreateTC \"CTCActionUseScriptedHook\";\n\
             EndThing;\n",
        );
        assert_eq!(
            thing.base().create_tc,
            vec![
                "CTCOwnedEntity".to_string(),
                "CTCActionUseScriptedHook".into()
            ]
        );
    }

    // ── families ──────────────────────────────────────────────────────────────

    #[test]
    fn a_camera_point_shares_its_base_with_the_family() {
        let thing = only_thing(
            "NewThing Thing;\n\
             StartCTCCameraPointScriptedSpline;\n\
             CoordBase C3DCoordF(-3359.4,-3569.1,-29.3);\n\
             CoordAxisUp C3DCoordF(0.0,0.0,1.0);\n\
             CoordAxisFwd C3DCoordF(1.0,0.0,0.0);\n\
             FOV 0.111111;\n\
             NumKeyCameras 2;\n\
             KeyCameras[0].Position C3DCoordF(2.0,-2.8,0.5);\n\
             KeyCameras[0].Duration 5.7;\n\
             KeyCameras[1].Event \"WOMAN\";\n\
             EndCTCCameraPointScriptedSpline;\n\
             EndThing;\n",
        );
        let Some(CameraPoint::ScriptedSpline(spline)) = &thing.components().camera_point else {
            panic!("expected a spline")
        };
        // `CoordAxisFwd`/`CoordAxisUp` are one `CRightHandedSet CoordAxis`.
        assert_eq!(spline.base.coord_axis.forward, [1.0, 0.0, 0.0]);
        assert_eq!(spline.base.coord_axis.up, [0.0, 0.0, 1.0]);
        assert_eq!(spline.base.fov, 0.111111);
        assert_eq!(spline.key_cameras.len(), 2);
        assert_eq!(spline.key_cameras[0].duration, 5.7);
        assert_eq!(spline.key_cameras[1].event, "WOMAN");
        // The family accessor reaches the shared base without naming the leaf.
        let point = thing.components().camera_point.as_ref().unwrap();
        assert_eq!(point.class(), "CTCCameraPointScriptedSpline");
        assert_eq!(point.base().fov, 0.111111);
    }

    #[test]
    fn a_chest_is_a_container() {
        let thing = only_thing(
            "NewThing Object;\n\
             StartCTCChest;\n\
             ChestOpen FALSE;\n\
             ContainerContents[0] \"OBJECT_HEALTH_POTION\";\n\
             ContainerContents[1] \"OBJECT_MANA_POTION\";\n\
             EndCTCChest;\n\
             EndThing;\n",
        );
        let Some(Container::Chest(chest)) = &thing.components().container else {
            panic!("expected a chest")
        };
        assert!(!chest.chest_open);
        // `ContainerContents` belongs to `CTCContainer`, the base all three share.
        assert_eq!(chest.base.contents.len(), 2);
        assert_eq!(chest.base.contents[0], "OBJECT_HEALTH_POTION");
    }

    #[test]
    fn a_navigator_reads_the_same_fields_as_a_standard() {
        let thing = only_thing(
            "NewThing AICreature;\n\
             StartCTCPhysicsNavigator;\n\
             PositionX 1.0;\nPositionY 2.0;\nPositionZ 3.0;\n\
             RHSetForwardY 1.0;\nRHSetUpZ 1.0;\n\
             EndCTCPhysicsNavigator;\n\
             EndThing;\n",
        );
        let Some(Physics::Navigator(physics)) = &thing.components().physics else {
            panic!("expected a navigator")
        };
        assert_eq!(physics.position, [1.0, 2.0, 3.0]);
        assert_eq!(physics.right_handed_set.forward, [0.0, 1.0, 0.0]);
        assert_eq!(
            thing.components().physics.as_ref().unwrap().class(),
            "CTCPhysicsNavigator"
        );
        assert_eq!(thing.placement().unwrap().position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_physics_light_has_no_orientation() {
        let thing = only_thing(
            "NewThing Thing;\n\
             StartCTCPhysicsLight;\n\
             PositionX 1.0;\nPositionY 2.0;\nPositionZ 3.0;\n\
             EndCTCPhysicsLight;\n\
             EndThing;\n",
        );
        let placement = thing.placement().unwrap();
        assert_eq!(placement.position, [1.0, 2.0, 3.0]);
        assert_eq!(placement.orientation, None);
    }

    #[test]
    fn shapes_nest_arrays() {
        let thing = only_thing(
            "NewThing Marker;\n\
             StartCTCShapeManager;\n\
             IsCoordsRelativeToMap TRUE;\n\
             NumShapes 1;\n\
             Shape[0].Type \"SHAPE_TYPE_CLOSED\";\n\
             Shape[0].size() 2;\n\
             Shape[0].pos[0].X 57.0;\n\
             Shape[0].pos[0].Y 101.0;\n\
             Shape[0].pos[1].X 54.0;\n\
             EndCTCShapeManager;\n\
             EndThing;\n",
        );
        let shapes = thing.components().shape_manager.as_ref().unwrap();
        assert!(shapes.is_coords_relative_to_map);
        assert_eq!(shapes.shapes.len(), 1);
        assert_eq!(shapes.shapes[0].kind, "SHAPE_TYPE_CLOSED");
        assert_eq!(shapes.shapes[0].positions.len(), 2);
        assert_eq!(shapes.shapes[0].positions[0], [57.0, 101.0, 0.0]);
    }

    #[test]
    fn navigation_route_numbered_slots_become_steps() {
        let thing = only_thing(
            "NewThing Marker;\n\
             StartCTCPreCalculatedNavigationRoute;\n\
             NumberOfStepsOnRoute 2;\n\
             NavPosition0 C2DCoordF(1.0,2.0);\n\
             NavLayer0 0;\n\
             NavPosition1 C2DCoordF(3.0,4.0);\n\
             NavLayer1 3;\n\
             PrecCalculatedNavigationRouteVersion 2;\n\
             EndCTCPreCalculatedNavigationRoute;\n\
             EndThing;\n",
        );
        let route = thing
            .components()
            .pre_calculated_navigation_route
            .as_ref()
            .unwrap();
        assert_eq!(route.steps.len(), 2);
        assert_eq!(route.steps[0].position, [1.0, 2.0]);
        assert_eq!(route.steps[1].layer, 3);
    }

    #[test]
    fn colour_is_read_as_rgba() {
        let thing = only_thing(
            "NewThing Object;\n\
             StartCTCLight;\n\
             Active TRUE;\nOverridden TRUE;\n\
             Colour CRGBColour(100,50,10,255);\n\
             OuterRadius 12.5;\n\
             EndCTCLight;\n\
             EndThing;\n",
        );
        let light = thing.components().light.as_ref().unwrap();
        assert!(light.active);
        assert_eq!(
            light.colour,
            Some(Rgba {
                r: 100,
                g: 50,
                b: 10,
                a: 255
            })
        );
        assert_eq!(light.outer_radius, Some(12.5));
        assert_eq!(light.inner_radius, None);
    }

    #[test]
    fn things_without_sections() {
        let tng = Tng::parse(
            "NewThing Object;\nDefinitionType \"OBJECT_FISHING_ROD\";\nEndThing;\n\
             NewThing Marker;\nDefinitionType \"MARKER_BASIC\";\nEndThing;\n",
        )
        .unwrap();
        assert_eq!(tng.sections.len(), 1);
        assert_eq!(tng.sections[0].name, "");
        assert_eq!(
            tng.things()
                .map(|t| t.base().definition_type.as_str())
                .collect::<Vec<_>>(),
            vec!["OBJECT_FISHING_ROD", "MARKER_BASIC"]
        );
    }

    #[test]
    fn holy_site_type_name() {
        let thing =
            only_thing("NewThing Holy Site;\nDefinitionType \"HOLY_SITE_01\";\nEndThing;\n");
        assert_eq!(thing.type_name(), "Holy Site");
        assert!(matches!(thing, TngThing::HolySite(_)));
    }

    // ── errors ────────────────────────────────────────────────────────────────

    #[test]
    fn an_unmodelled_field_is_an_error_not_a_silent_drop() {
        let err = Tng::parse("NewThing Object;\nWhatIsThis 1;\nEndThing;\n").unwrap_err();
        assert_eq!(
            err.to_string(),
            "2:1: unknown field: `WhatIsThis` in thing `Object`"
        );
    }

    #[test]
    fn an_unmodelled_component_is_an_error() {
        let err = Tng::parse("NewThing Object;\nStartCTCMystery;\nEndCTCMystery;\nEndThing;\n")
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "2:1: unknown component class `CTCMystery` in thing `Object`"
        );
    }

    #[test]
    fn an_unmodelled_thing_kind_is_an_error() {
        let err = Tng::parse("NewThing Wombat;\nEndThing;\n").unwrap_err();
        assert_eq!(err.to_string(), "1:10: unknown thing kind `Wombat`");
    }

    /// `CThing`'s component map is keyed by interface, so neither of these can be
    /// stored — and neither should be silently absorbed.
    #[test]
    fn duplicate_and_conflicting_components_are_refused() {
        let err = Tng::parse(
            "NewThing Object;\nStartCTCEditor;\nEndCTCEditor;\n\
             StartCTCEditor;\nEndCTCEditor;\nEndThing;\n",
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "4:1: `CTCEditor` appears twice on this thing"
        );

        let err = Tng::parse(
            "NewThing AICreature;\n\
             StartCTCPhysicsStandard;\nEndCTCPhysicsStandard;\n\
             StartCTCPhysicsNavigator;\nEndCTCPhysicsNavigator;\nEndThing;\n",
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "4:1: `CTCPhysicsNavigator` conflicts with `CTCPhysicsStandard` — both are one interface"
        );
    }

    #[test]
    fn a_field_error_names_the_component_it_is_in() {
        let err =
            Tng::parse("NewThing Object;\nStartCTCLight;\nActive 3.5;\nEndCTCLight;\nEndThing;\n")
                .unwrap_err();
        assert_eq!(
            err.to_string(),
            "3:1: expected TRUE or FALSE: `Active` in `CTCLight`"
        );
    }

    #[test]
    fn a_lex_error_is_reported_as_text() {
        let err = Tng::parse("NewThing Object;\nHealth @;\n").unwrap_err();
        assert!(matches!(err, TngError::Text(_)), "got {err}");
    }
}
