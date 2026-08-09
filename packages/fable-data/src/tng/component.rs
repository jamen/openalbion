//! The `StartCTC… … EndCTC…` component blocks, one type per `CTCBase` subclass.
//!
//! A thing's behaviour is assembled from components. `CThing` stores them in a
//! `CVectorMap<ETCInterfaceType, CTCBase*>` — a map keyed by *interface*, not a
//! list — and `CTCBase::BeginSerialising` writes each one as
//! `"Start" + GetClassTypeName()` … `"End" + …`, gated on
//! `IsComponentSerialisable()`. That gate is why only **59** of the 224 classes
//! the defs declare ever reach a `.tng`: the rest carry no per-instance state.
//!
//! # Why this is a struct of slots and not a list
//!
//! Three properties of the engine's storage, each verified over all 21,800
//! things in `FinalAlbion.wad`:
//!
//! - **At most one component per class** — no thing writes a class twice.
//! - **At most one per interface.** Several classes are one interface because
//!   they share a base: `CTCPhysicsNavigator` *is a* `CTCPhysicsStandard` *is a*
//!   `CTCPhysicsBase`, and only the base declares `GetInterfaceType`. No thing
//!   writes two physics components, two drivers, two camera points, or two
//!   containers. Those families are [`Physics`], [`Driver`], [`CameraPoint`],
//!   [`Container`], [`ActionUse`] and [`ActivationReceptor`].
//! - **Order carries no information** — it is a total order over the classes
//!   (0 of 257 ordered pairs ever vary), so nothing is lost by storing slots and
//!   the declaration order below reproduces the file.
//!
//! So [`Components`] is 45 named `Option` slots, and a duplicate or a conflicting
//! interface is a parse error rather than a silent overwrite.
//!
//! # What is *not* encoded
//!
//! Which components a given *kind* of thing may carry is deliberately not a type
//! constraint. It looks like one — no `Marker` in the game serialises a
//! `CTCShop` — but the defs disagree: `MARKER` defs declare `CTCShop`, `CTCBed`
//! and `CTCThingOwner`, and `Components.Add` places no restriction on the thing
//! class. Encoding the observed sets would invent a rule the engine does not
//! have and reject data it accepts.

use super::read::{FieldErrorKind, ReadField, Redundant, Rgba, RightHandedSet};
use crate::text::{Body, PathSegment, Statement, Value};

// ── Array element structs ─────────────────────────────────────────────────────

/// One entry of `CTCCameraPointScriptedSpline`'s `KeyCameras[i]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyCamera {
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

impl ComponentBody for KeyCamera {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Position" => self.position.read_field(rest, value),
            "LookDirection" => self.look_direction.read_field(rest, value),
            "FOV" => self.fov.read_field(rest, value),
            "ShuttleSpeed" => self.shuttle_speed.read_field(rest, value),
            "Duration" => self.duration.read_field(rest, value),
            "PauseTime" => self.pause_time.read_field(rest, value),
            "Event" => self.event.read_field(rest, value),
            "AnimationSpeed" => self.animation_speed.read_field(rest, value),
            "RollAngle" => self.roll_angle.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

impl ReadField for KeyCamera {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // An element of an indexed array: the index has been consumed, so what is
        // left names one of this struct's own fields.
        let (name, tail) = match rest {
            [PathSegment::Field(name), tail @ ..] | [PathSegment::Call(name), tail @ ..] => {
                (name, tail)
            }
            _ => return Err(FieldErrorKind::UnexpectedPath("value without a field name")),
        };
        self.read_named(name, tail, value)
    }
}
/// One entry of `CTCShapeManager`'s `Shape[i]` — a polyline or closed area.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shape {
    pub kind: String,
    /// `Shape[i].pos[j]`, written a component at a time (`.X` / `.Y` / `.Z`).
    pub positions: Vec<[f32; 3]>,
    /// Written `Shape[i].size()`. Equals `positions.len()` on all 47 shapes
    /// in the game.
    pub size: Redundant,
}

impl ComponentBody for Shape {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Type" => self.kind.read_field(rest, value),
            "pos" => self.positions.read_field(rest, value),
            "size" => self.size.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

impl ReadField for Shape {
    fn read_field(
        &mut self,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // An element of an indexed array: the index has been consumed, so what is
        // left names one of this struct's own fields.
        let (name, tail) = match rest {
            [PathSegment::Field(name), tail @ ..] | [PathSegment::Call(name), tail @ ..] => {
                (name, tail)
            }
            _ => return Err(FieldErrorKind::UnexpectedPath("value without a field name")),
        };
        self.read_named(name, tail, value)
    }
}

/// One step of a [`PreCalculatedNavigationRoute`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavStep {
    pub position: [f32; 2],
    pub layer: i32,
}

// ── Families ──────────────────────────────────────────────────────────────────

/// A thing's placement, from whichever `CTCPhysicsBase` subclass it carries.
///
/// `CThing::PhysicsTC` is one pointer, so these are mutually exclusive.
/// `CTCPhysicsBase` declares `Position`; `CTCPhysicsStandard` adds the
/// `CRightHandedSet`; `CTCPhysicsNavigator` and `CTCPhysicsLight` add nothing
/// that is serialised — which is exactly the 9 / 9 / 3 fields the game writes.
#[derive(Clone, Debug, PartialEq)]
pub enum Physics {
    /// `CTCPhysicsLight` — 113 in the game. Position only, no orientation.
    Light(PhysicsLight),
    /// `CTCPhysicsStandard` — 20,868 in the game. **A thing's placement.**
    Standard(PhysicsStandard),
    /// `CTCPhysicsNavigator` — 819, all `AICreature`s. A `CTCPhysicsStandard`
    /// that navigates; it serialises the same fields.
    Navigator(PhysicsStandard),
}

impl Physics {
    pub fn position(&self) -> [f32; 3] {
        match self {
            Physics::Light(p) => p.position,
            Physics::Standard(p) | Physics::Navigator(p) => p.position,
        }
    }

    /// The forward/up pair, for the two subclasses that write one.
    pub fn right_handed_set(&self) -> Option<RightHandedSet> {
        match self {
            Physics::Light(_) => None,
            Physics::Standard(p) | Physics::Navigator(p) => Some(p.right_handed_set),
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            Physics::Light(_) => PhysicsLight::CLASS,
            Physics::Standard(_) => PhysicsStandard::CLASS,
            Physics::Navigator(_) => "CTCPhysicsNavigator",
        }
    }
}

/// `CTCPhysicsLight` — `CTCPhysicsBase::Position` and nothing else.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicsLight {
    /// Z-up, in world units — one landscape cell is 1.0 (AGENTS.md §3.6).
    pub position: [f32; 3],
}

impl PhysicsLight {
    pub const CLASS: &'static str = "CTCPhysicsLight";
}

impl ComponentBody for PhysicsLight {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        read_position(&mut self.position, name, rest, value)
    }
}

/// `CTCPhysicsStandard` — position plus `CRightHandedSet RHSet`
/// (`fablelib/tc_physics_standard.cpp:471`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicsStandard {
    /// Z-up, in world units.
    pub position: [f32; 3],
    /// Written as `RHSetForwardX` … `RHSetUpZ`. Already orthonormal in every
    /// `.tng` the game ships, which is what lets the object matrix be built from
    /// it directly (`CalcObjectMatrix`).
    pub right_handed_set: RightHandedSet,
}

impl PhysicsStandard {
    pub const CLASS: &'static str = "CTCPhysicsStandard";
}

impl ComponentBody for PhysicsStandard {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "RHSetForwardX" => self.right_handed_set.forward[0].read_field(rest, value),
            "RHSetForwardY" => self.right_handed_set.forward[1].read_field(rest, value),
            "RHSetForwardZ" => self.right_handed_set.forward[2].read_field(rest, value),
            "RHSetUpX" => self.right_handed_set.up[0].read_field(rest, value),
            "RHSetUpY" => self.right_handed_set.up[1].read_field(rest, value),
            "RHSetUpZ" => self.right_handed_set.up[2].read_field(rest, value),
            _ => read_position(&mut self.position, name, rest, value),
        }
    }
}

/// `CTCPhysicsBase::Position`, written as three sibling fields.
fn read_position(
    position: &mut [f32; 3],
    name: &str,
    rest: &[PathSegment<'_>],
    value: &Value<'_>,
) -> Result<(), FieldErrorKind> {
    match name {
        "PositionX" => position[0].read_field(rest, value),
        "PositionY" => position[1].read_field(rest, value),
        "PositionZ" => position[2].read_field(rest, value),
        _ => Err(FieldErrorKind::UnknownField),
    }
}

/// The fields every camera point shares, from `CTCCameraPointDefinitionBase`
/// (`fablelib/tc_camera_point_definition_base.hpp`). Its `CTCCameraPointShapeActivationBase`
/// subclass adds behaviour but no data, so this is the whole shared set.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraPointBase {
    pub cut_into: bool,
    pub cut_out_of: bool,
    pub test_angle_before_activation: bool,
    pub fov: f32,
    pub self_terminate: bool,
    pub hero_is_subject: bool,
    pub coord_base: [f32; 3],
    pub using_relative_coords: bool,
    pub using_relative_orientation: bool,
    pub is_coord_base_relative_to_parent: bool,
    /// `CoordAxisFwd` / `CoordAxisUp` — declared as one `CRightHandedSet CoordAxis`,
    /// the same pair `CTCPhysicsStandard` uses.
    pub coord_axis: RightHandedSet,
}

impl ComponentBody for CameraPointBase {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "CutInto" => self.cut_into.read_field(rest, value),
            "CutOutOf" => self.cut_out_of.read_field(rest, value),
            "TestAngleBeforeActivation" => {
                self.test_angle_before_activation.read_field(rest, value)
            }
            "FOV" => self.fov.read_field(rest, value),
            "SelfTerminate" => self.self_terminate.read_field(rest, value),
            "HeroIsSubject" => self.hero_is_subject.read_field(rest, value),
            "CoordBase" => self.coord_base.read_field(rest, value),
            "UsingRelativeCoords" => self.using_relative_coords.read_field(rest, value),
            "UsingRelativeOrientation" => self.using_relative_orientation.read_field(rest, value),
            "IsCoordBaseRelativeToParent" => self
                .is_coord_base_relative_to_parent
                .read_field(rest, value),
            "CoordAxisFwd" => self.coord_axis.forward.read_field(rest, value),
            "CoordAxisUp" => self.coord_axis.up.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

/// A camera point. `CTCCameraPointDefinitionBase` holds one interface slot, and
/// each subclass adds its own leaf data through `OnSerialseLeaf` — the game's
/// spelling — which is what makes this an enum over a shared base.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraPoint {
    /// `CTCCameraPointFixedPoint` — 24 in the game.
    FixedPoint(CameraPointFixedPoint),
    /// `CTCCameraPointGeneralCase` — 5 in the game.
    GeneralCase(CameraPointGeneralCase),
    /// `CTCCameraPointScripted` — 1,325 in the game.
    Scripted(CameraPointScripted),
    /// `CTCCameraPointScriptedSpline` — 1,000 in the game.
    ScriptedSpline(CameraPointScriptedSpline),
    /// `CTCCameraPointTrack` — 4 in the game.
    Track(CameraPointTrack),
}

impl CameraPoint {
    /// The shared `CTCCameraPointDefinitionBase` fields.
    pub fn base(&self) -> &CameraPointBase {
        match self {
            CameraPoint::FixedPoint(c) => &c.base,
            CameraPoint::GeneralCase(c) => &c.base,
            CameraPoint::Scripted(c) => &c.base,
            CameraPoint::ScriptedSpline(c) => &c.base,
            CameraPoint::Track(c) => &c.base,
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            CameraPoint::FixedPoint(_) => CameraPointFixedPoint::CLASS,
            CameraPoint::GeneralCase(_) => CameraPointGeneralCase::CLASS,
            CameraPoint::Scripted(_) => CameraPointScripted::CLASS,
            CameraPoint::ScriptedSpline(_) => CameraPointScriptedSpline::CLASS,
            CameraPoint::Track(_) => CameraPointTrack::CLASS,
        }
    }
}
/// `CTCCameraPointFixedPoint` — 24 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraPointFixedPoint {
    /// The fields every CameraPointBase shares.
    pub base: CameraPointBase,
    /// Written one axis at a time (`LookVector.X`), unlike `CoordBase`.
    pub look_vector: [f32; 3],
    pub track_thing: bool,
}

impl CameraPointFixedPoint {
    /// The block name, as written: `StartCTCCameraPointFixedPoint` … `EndCTCCameraPointFixedPoint`.
    pub const CLASS: &'static str = "CTCCameraPointFixedPoint";
}

impl ComponentBody for CameraPointFixedPoint {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "LookVector" => self.look_vector.read_field(rest, value),
            "TrackThing" => self.track_thing.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}
/// `CTCCameraPointGeneralCase` — 5 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraPointGeneralCase {
    /// The fields every CameraPointBase shares.
    pub base: CameraPointBase,
    pub allow_right_stick_rotation: bool,
    pub allow_right_stick_zoom: bool,
    pub allow_z_target: bool,
    pub auto_go_behind: bool,
    pub auto_go_behind_time: f32,
    pub cage_radius: f32,
    pub height_offset: f32,
    pub string_length: f32,
}

impl CameraPointGeneralCase {
    /// The block name, as written: `StartCTCCameraPointGeneralCase` … `EndCTCCameraPointGeneralCase`.
    pub const CLASS: &'static str = "CTCCameraPointGeneralCase";
}

impl ComponentBody for CameraPointGeneralCase {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "AllowRightStickRotation" => self.allow_right_stick_rotation.read_field(rest, value),
            "AllowRightStickZoom" => self.allow_right_stick_zoom.read_field(rest, value),
            "AllowZTarget" => self.allow_z_target.read_field(rest, value),
            "AutoGoBehind" => self.auto_go_behind.read_field(rest, value),
            "AutoGoBehindTime" => self.auto_go_behind_time.read_field(rest, value),
            "CageRadius" => self.cage_radius.read_field(rest, value),
            "HeightOffset" => self.height_offset.read_field(rest, value),
            "StringLength" => self.string_length.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}
/// `CTCCameraPointScripted` — 1,325 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraPointScripted {
    /// The fields every CameraPointBase shares.
    pub base: CameraPointBase,
    pub end_pos: [f32; 3],
    pub look_direction: [f32; 3],
    pub look_direction_end: [f32; 3],
    pub start_pos: [f32; 3],
    pub thing_uid: Option<u64>,
    pub transition_time: f32,
}

impl CameraPointScripted {
    /// The block name, as written: `StartCTCCameraPointScripted` … `EndCTCCameraPointScripted`.
    pub const CLASS: &'static str = "CTCCameraPointScripted";
}

impl ComponentBody for CameraPointScripted {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "EndPos" => self.end_pos.read_field(rest, value),
            "LookDirection" => self.look_direction.read_field(rest, value),
            "LookDirectionEnd" => self.look_direction_end.read_field(rest, value),
            "StartPos" => self.start_pos.read_field(rest, value),
            "ThingUID" => self.thing_uid.read_field(rest, value),
            "TransitionTime" => self.transition_time.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}
/// `CTCCameraPointScriptedSpline` — 1,000 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraPointScriptedSpline {
    /// The fields every CameraPointBase shares.
    pub base: CameraPointBase,
    /// The spline's control cameras.
    pub key_cameras: Vec<KeyCamera>,
    /// Equals `key_cameras.len()` on every spline in the game.
    pub num_key_cameras: Redundant,
    pub tension: f32,
    pub time_to_play: f32,
    pub valid_anims: Vec<String>,
}

impl CameraPointScriptedSpline {
    /// The block name, as written: `StartCTCCameraPointScriptedSpline` … `EndCTCCameraPointScriptedSpline`.
    pub const CLASS: &'static str = "CTCCameraPointScriptedSpline";
}

impl ComponentBody for CameraPointScriptedSpline {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "KeyCameras" => self.key_cameras.read_field(rest, value),
            "NumKeyCameras" => self.num_key_cameras.read_field(rest, value),
            "Tension" => self.tension.read_field(rest, value),
            "TimeToPlay" => self.time_to_play.read_field(rest, value),
            "ValidAnims" => self.valid_anims.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}
/// `CTCCameraPointTrack` — 4 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraPointTrack {
    /// The fields every CameraPointBase shares.
    pub base: CameraPointBase,
    pub cage_radius: f32,
    pub string_length: f32,
}

impl CameraPointTrack {
    /// The block name, as written: `StartCTCCameraPointTrack` … `EndCTCCameraPointTrack`.
    pub const CLASS: &'static str = "CTCCameraPointTrack";
}

impl ComponentBody for CameraPointTrack {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "CageRadius" => self.cage_radius.read_field(rest, value),
            "StringLength" => self.string_length.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}

/// A container's contents, from `CTCContainer::ContainerItems`
/// (`fablelib/defs/carrying_def.hpp:1163`) — the base that holds the interface,
/// which is why all three subclasses write `ContainerContents[]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContainerBase {
    /// The `OBJECT_*` defs the container holds.
    pub contents: Vec<String>,
}

impl ComponentBody for ContainerBase {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ContainerContents" => self.contents.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

/// One `CTCContainer` interface slot: `CTCChest` and `CTCSearchableContainer`
/// both derive from `CTCContainerRewardHero`.
#[derive(Clone, Debug, PartialEq)]
pub enum Container {
    /// `CTCContainerRewardHero` — 1,101 in the game.
    RewardHero(ContainerRewardHero),
    /// `CTCChest` — 118 in the game.
    Chest(Chest),
    /// `CTCSearchableContainer` — 218 in the game.
    Searchable(SearchableContainer),
}

impl Container {
    pub fn base(&self) -> &ContainerBase {
        match self {
            Container::RewardHero(c) => &c.base,
            Container::Chest(c) => &c.base,
            Container::Searchable(c) => &c.base,
        }
    }

    pub fn class(&self) -> &'static str {
        match self {
            Container::RewardHero(_) => ContainerRewardHero::CLASS,
            Container::Chest(_) => Chest::CLASS,
            Container::Searchable(_) => SearchableContainer::CLASS,
        }
    }
}
/// `CTCContainerRewardHero` — 1,101 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContainerRewardHero {
    /// The fields every ContainerBase shares.
    pub base: ContainerBase,
}

impl ContainerRewardHero {
    /// The block name, as written: `StartCTCContainerRewardHero` … `EndCTCContainerRewardHero`.
    pub const CLASS: &'static str = "CTCContainerRewardHero";
}

impl ComponentBody for ContainerRewardHero {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        self.base.read_named(name, rest, value)
    }
}
/// `CTCChest` — 118 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chest {
    /// The fields every ContainerBase shares.
    pub base: ContainerBase,
    pub chest_open: bool,
}

impl Chest {
    /// The block name, as written: `StartCTCChest` … `EndCTCChest`.
    pub const CLASS: &'static str = "CTCChest";
}

impl ComponentBody for Chest {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ChestOpen" => self.chest_open.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}
/// `CTCSearchableContainer` — 218 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchableContainer {
    /// The fields every ContainerBase shares.
    pub base: ContainerBase,
    pub number_of_times_to_search: i32,
}

impl SearchableContainer {
    /// The block name, as written: `StartCTCSearchableContainer` … `EndCTCSearchableContainer`.
    pub const CLASS: &'static str = "CTCSearchableContainer";
}

impl ComponentBody for SearchableContainer {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "NumberOfTimesToSearch" => self.number_of_times_to_search.read_field(rest, value),
            _ => self.base.read_named(name, rest, value),
        }
    }
}

/// One `CTCActionUseBase` interface slot.
#[derive(Clone, Debug, PartialEq)]
pub enum ActionUse {
    /// `CTCActionUseBed` — 230 in the game.
    Bed(ActionUseBed),
    /// `CTCActionUseReadable` — 57 in the game. Signs and books.
    Readable(ActionUseReadable),
}

impl ActionUse {
    pub fn class(&self) -> &'static str {
        match self {
            ActionUse::Bed(_) => ActionUseBed::CLASS,
            ActionUse::Readable(_) => ActionUseReadable::CLASS,
        }
    }
}

/// One `CTCActivationReceptorBase` interface slot.
#[derive(Clone, Debug, PartialEq)]
pub enum ActivationReceptor {
    /// `CTCActivationReceptorCreatureGenerator` — 73 in the game.
    CreatureGenerator(ActivationReceptorCreatureGenerator),
    /// `CTCActivationReceptorDoor` — 6 in the game.
    Door(ActivationReceptorDoor),
}

impl ActivationReceptor {
    pub fn class(&self) -> &'static str {
        match self {
            ActivationReceptor::CreatureGenerator(_) => ActivationReceptorCreatureGenerator::CLASS,
            ActivationReceptor::Door(_) => ActivationReceptorDoor::CLASS,
        }
    }
}

/// One `CTCDriverBase` interface slot — `CThing::PTCDriver` is a single pointer.
///
/// The `CTCD*` prefix is what the `D` stands for: a driver is the component that
/// gives a bare `Thing` its purpose.
#[derive(Clone, Debug, PartialEq)]
pub enum Driver {
    /// `CTCDNavigationSeed` — 113 in the game. No serialised state.
    NavigationSeed(DNavigationSeed),
    /// `CTCDRegionEntrance` — 187 in the game.
    RegionEntrance(DRegionEntrance),
    /// `CTCDRegionExit` — 139 in the game.
    RegionExit(DRegionExit),
    /// `CTCDCameraPoint` — 2,358 in the game. No serialised state.
    CameraPoint(DCameraPoint),
    /// `CTCDParticleEmitter` — 252 in the game.
    ParticleEmitter(DParticleEmitter),
}

impl Driver {
    pub fn class(&self) -> &'static str {
        match self {
            Driver::NavigationSeed(_) => DNavigationSeed::CLASS,
            Driver::RegionEntrance(_) => DRegionEntrance::CLASS,
            Driver::RegionExit(_) => DRegionExit::CLASS,
            Driver::CameraPoint(_) => DCameraPoint::CLASS,
            Driver::ParticleEmitter(_) => DParticleEmitter::CLASS,
        }
    }
}

// ── Components ────────────────────────────────────────────────────────────────
/// `CTCAIScratchpad` — 61 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AIScratchpad {}

impl AIScratchpad {
    /// The block name, as written: `StartCTCAIScratchpad` … `EndCTCAIScratchpad`.
    pub const CLASS: &'static str = "CTCAIScratchpad";
}

impl ComponentBody for AIScratchpad {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCActionUseBed` — 230 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionUseBed {
    pub owned_by_hero: bool,
    pub useable_by_hero: bool,
}

impl ActionUseBed {
    /// The block name, as written: `StartCTCActionUseBed` … `EndCTCActionUseBed`.
    pub const CLASS: &'static str = "CTCActionUseBed";
}

impl ComponentBody for ActionUseBed {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "OwnedByHero" => self.owned_by_hero.read_field(rest, value),
            "UseableByHero" => self.useable_by_hero.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCActionUseReadable` — 57 in the game. Signs and books.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionUseReadable {
    pub game_text_def_name: String,
}

impl ActionUseReadable {
    /// The block name, as written: `StartCTCActionUseReadable` … `EndCTCActionUseReadable`.
    pub const CLASS: &'static str = "CTCActionUseReadable";
}

impl ComponentBody for ActionUseReadable {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "GameTextDefName" => self.game_text_def_name.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCActionUseScriptedHook` — 221 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionUseScriptedHook {
    pub animation_name: String,
    pub camera_track_uid: Option<u64>,
    pub entrance_connected_to_uid: Option<u64>,
    pub force_confirmation: bool,
    pub hidden_on_mini_map: Option<bool>,
    pub replacement_object: i32,
    pub reversed_on_mini_map: Option<bool>,
    pub sound_name: String,
    pub teleport_to_region_entrance: bool,
    pub usable: bool,
    pub version_number: i32,
}

impl ActionUseScriptedHook {
    /// The block name, as written: `StartCTCActionUseScriptedHook` … `EndCTCActionUseScriptedHook`.
    pub const CLASS: &'static str = "CTCActionUseScriptedHook";
}

impl ComponentBody for ActionUseScriptedHook {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "AnimationName" => self.animation_name.read_field(rest, value),
            "CameraTrackUID" => self.camera_track_uid.read_field(rest, value),
            "EntranceConnectedToUID" => self.entrance_connected_to_uid.read_field(rest, value),
            "ForceConfirmation" => self.force_confirmation.read_field(rest, value),
            "HiddenOnMiniMap" => self.hidden_on_mini_map.read_field(rest, value),
            "ReplacementObject" => self.replacement_object.read_field(rest, value),
            "ReversedOnMiniMap" => self.reversed_on_mini_map.read_field(rest, value),
            "SoundName" => self.sound_name.read_field(rest, value),
            "TeleportToRegionEntrance" => self.teleport_to_region_entrance.read_field(rest, value),
            "Usable" => self.usable.read_field(rest, value),
            "VersionNumber" => self.version_number.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCActivationReceptorCreatureGenerator` — 73 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivationReceptorCreatureGenerator {
    pub activate_on_activate: bool,
    pub deactivate_after_set_time: bool,
    pub frames_after_activation_to_deactivate: i32,
    pub trigger_on_activate: bool,
}

impl ActivationReceptorCreatureGenerator {
    /// The block name, as written: `StartCTCActivationReceptorCreatureGenerator` … `EndCTCActivationReceptorCreatureGenerator`.
    pub const CLASS: &'static str = "CTCActivationReceptorCreatureGenerator";
}

impl ComponentBody for ActivationReceptorCreatureGenerator {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ActivateOnActivate" => self.activate_on_activate.read_field(rest, value),
            "DeactivateAfterSetTime" => self.deactivate_after_set_time.read_field(rest, value),
            "FramesAfterActivationToDeactivate" => self
                .frames_after_activation_to_deactivate
                .read_field(rest, value),
            "TriggerOnActivate" => self.trigger_on_activate.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCActivationReceptorDoor` — 6 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivationReceptorDoor {
    pub deactivate_after_set_time: bool,
    pub frames_after_activation_to_deactivate: i32,
}

impl ActivationReceptorDoor {
    /// The block name, as written: `StartCTCActivationReceptorDoor` … `EndCTCActivationReceptorDoor`.
    pub const CLASS: &'static str = "CTCActivationReceptorDoor";
}

impl ComponentBody for ActivationReceptorDoor {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "DeactivateAfterSetTime" => self.deactivate_after_set_time.read_field(rest, value),
            "FramesAfterActivationToDeactivate" => self
                .frames_after_activation_to_deactivate
                .read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCActivationTrigger` — 73 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivationTrigger {
    pub receptor_uid: u64,
}

impl ActivationTrigger {
    /// The block name, as written: `StartCTCActivationTrigger` … `EndCTCActivationTrigger`.
    pub const CLASS: &'static str = "CTCActivationTrigger";
}

impl ComponentBody for ActivationTrigger {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ReceptorUID" => self.receptor_uid.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCAtmosPlayer` — 28 in the game. An ambient sound theme.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtmosPlayer {
    pub atmos_name: String,
}

impl AtmosPlayer {
    /// The block name, as written: `StartCTCAtmosPlayer` … `EndCTCAtmosPlayer`.
    pub const CLASS: &'static str = "CTCAtmosPlayer";
}

impl ComponentBody for AtmosPlayer {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "AtmosName" => self.atmos_name.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCBoastingArea` — 1 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoastingArea {
    pub radius: f32,
}

impl BoastingArea {
    /// The block name, as written: `StartCTCBoastingArea` … `EndCTCBoastingArea`.
    pub const CLASS: &'static str = "CTCBoastingArea";
}

impl ComponentBody for BoastingArea {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Radius" => self.radius.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCBuyableHouse` — 291 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuyableHouse {
    pub current_dress_level: i32,
    pub day_next_rent_is_due: i32,
    pub is_residential: Option<bool>,
    pub is_scripted: bool,
    pub owned_by_player: bool,
    pub rented: bool,
    pub virtual_money_bags: i32,
    pub wife_living_here: i32,
}

impl BuyableHouse {
    /// The block name, as written: `StartCTCBuyableHouse` … `EndCTCBuyableHouse`.
    pub const CLASS: &'static str = "CTCBuyableHouse";
}

impl ComponentBody for BuyableHouse {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "CurrentDressLevel" => self.current_dress_level.read_field(rest, value),
            "DayNextRentIsDue" => self.day_next_rent_is_due.read_field(rest, value),
            "IsResidential" => self.is_residential.read_field(rest, value),
            "IsScripted" => self.is_scripted.read_field(rest, value),
            "OwnedByPlayer" => self.owned_by_player.read_field(rest, value),
            "Rented" => self.rented.read_field(rest, value),
            "VirtualMoneyBags" => self.virtual_money_bags.read_field(rest, value),
            "WifeLivingHere" => self.wife_living_here.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCCarriedActionUseRead` — 2 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CarriedActionUseRead {
    pub already_read: bool,
}

impl CarriedActionUseRead {
    /// The block name, as written: `StartCTCCarriedActionUseRead` … `EndCTCCarriedActionUseRead`.
    pub const CLASS: &'static str = "CTCCarriedActionUseRead";
}

impl ComponentBody for CarriedActionUseRead {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "AlreadyRead" => self.already_read.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCCreatureGenerator` — 73 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreatureGenerator {
    pub active_creature_limit: i32,
    pub creature_families: Vec<String>,
    pub generation_radius: f32,
    pub num_triggers: i32,
    pub script_name_of_all_generated_creatures: String,
    pub self_trigger: bool,
    pub self_trigger_radius: f32,
    pub self_trigger_reset_interval: i32,
    pub total_generation_limit: i32,
    pub trigger_on_activate: bool,
}

impl CreatureGenerator {
    /// The block name, as written: `StartCTCCreatureGenerator` … `EndCTCCreatureGenerator`.
    pub const CLASS: &'static str = "CTCCreatureGenerator";
}

impl ComponentBody for CreatureGenerator {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ActiveCreatureLimit" => self.active_creature_limit.read_field(rest, value),
            "CreatureFamilies" => self.creature_families.read_field(rest, value),
            "GenerationRadius" => self.generation_radius.read_field(rest, value),
            "NumTriggers" => self.num_triggers.read_field(rest, value),
            "ScriptNameOfAllGeneratedCreatures" => self
                .script_name_of_all_generated_creatures
                .read_field(rest, value),
            "SelfTrigger" => self.self_trigger.read_field(rest, value),
            "SelfTriggerRadius" => self.self_trigger_radius.read_field(rest, value),
            "SelfTriggerResetInterval" => self.self_trigger_reset_interval.read_field(rest, value),
            "TotalGenerationLimit" => self.total_generation_limit.read_field(rest, value),
            "TriggerOnActivate" => self.trigger_on_activate.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCCreatureGeneratorCreator` — 67 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreatureGeneratorCreator {}

impl CreatureGeneratorCreator {
    /// The block name, as written: `StartCTCCreatureGeneratorCreator` … `EndCTCCreatureGeneratorCreator`.
    pub const CLASS: &'static str = "CTCCreatureGeneratorCreator";
}

impl ComponentBody for CreatureGeneratorCreator {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCCreatureOpinionOfHero` — 603 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreatureOpinionOfHero {
    pub forced_attitude: i32,
    pub frame_to_decay_number_of_times_hit: i32,
    pub greeted_flag: bool,
    pub hero_opinion_enemy: Option<bool>,
    pub interacted_flag: Option<bool>,
    pub last_opinion_reaction_frame: i32,
    pub number_of_times_hit: f32,
    pub tolerance_to_being_hit_override: f32,
}

impl CreatureOpinionOfHero {
    /// The block name, as written: `StartCTCCreatureOpinionOfHero` … `EndCTCCreatureOpinionOfHero`.
    pub const CLASS: &'static str = "CTCCreatureOpinionOfHero";
}

impl ComponentBody for CreatureOpinionOfHero {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ForcedAttitude" => self.forced_attitude.read_field(rest, value),
            "FrameToDecayNumberOfTimesHit" => self
                .frame_to_decay_number_of_times_hit
                .read_field(rest, value),
            "GreetedFlag" => self.greeted_flag.read_field(rest, value),
            "HeroOpinionEnemy" => self.hero_opinion_enemy.read_field(rest, value),
            "InteractedFlag" => self.interacted_flag.read_field(rest, value),
            "LastOpinionReactionFrame" => self.last_opinion_reaction_frame.read_field(rest, value),
            "NumberOfTimesHit" => self.number_of_times_hit.read_field(rest, value),
            "ToleranceToBeingHitOverride" => {
                self.tolerance_to_being_hit_override.read_field(rest, value)
            }
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCDCameraPoint` — 2,358 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DCameraPoint {}

impl DCameraPoint {
    /// The block name, as written: `StartCTCDCameraPoint` … `EndCTCDCameraPoint`.
    pub const CLASS: &'static str = "CTCDCameraPoint";
}

impl ComponentBody for DCameraPoint {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCDNavigationSeed` — 113 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DNavigationSeed {}

impl DNavigationSeed {
    /// The block name, as written: `StartCTCDNavigationSeed` … `EndCTCDNavigationSeed`.
    pub const CLASS: &'static str = "CTCDNavigationSeed";
}

impl ComponentBody for DNavigationSeed {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCDParticleEmitter` — 252 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DParticleEmitter {
    pub independant_object: bool,
    pub particle_type_name: String,
}

impl DParticleEmitter {
    /// The block name, as written: `StartCTCDParticleEmitter` … `EndCTCDParticleEmitter`.
    pub const CLASS: &'static str = "CTCDParticleEmitter";
}

impl ComponentBody for DParticleEmitter {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "IndependantObject" => self.independant_object.read_field(rest, value),
            "ParticleTypeName" => self.particle_type_name.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCDRegionEntrance` — 187 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DRegionEntrance {
    pub active: Option<bool>,
}

impl DRegionEntrance {
    /// The block name, as written: `StartCTCDRegionEntrance` … `EndCTCDRegionEntrance`.
    pub const CLASS: &'static str = "CTCDRegionEntrance";
}

impl ComponentBody for DRegionEntrance {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Active" => self.active.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCDRegionExit` — 139 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DRegionExit {
    pub active: bool,
    pub entrance_connected_to_uid: u64,
    pub hidden_on_mini_map: Option<bool>,
    pub message_radius: f32,
    pub radius: f32,
    pub reversed_on_mini_map: Option<bool>,
}

impl DRegionExit {
    /// The block name, as written: `StartCTCDRegionExit` … `EndCTCDRegionExit`.
    pub const CLASS: &'static str = "CTCDRegionExit";
}

impl ComponentBody for DRegionExit {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Active" => self.active.read_field(rest, value),
            "EntranceConnectedToUID" => self.entrance_connected_to_uid.read_field(rest, value),
            "HiddenOnMiniMap" => self.hidden_on_mini_map.read_field(rest, value),
            "MessageRadius" => self.message_radius.read_field(rest, value),
            "Radius" => self.radius.read_field(rest, value),
            "ReversedOnMiniMap" => self.reversed_on_mini_map.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCDiggingSpot` — 60 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiggingSpot {
    pub hidden: bool,
}

impl DiggingSpot {
    /// The block name, as written: `StartCTCDiggingSpot` … `EndCTCDiggingSpot`.
    pub const CLASS: &'static str = "CTCDiggingSpot";
}

impl ComponentBody for DiggingSpot {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Hidden" => self.hidden.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCDoor` — 173 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Door {
    pub door_trigger_type: Option<i32>,
    pub open: bool,
}

impl Door {
    /// The block name, as written: `StartCTCDoor` … `EndCTCDoor`.
    pub const CLASS: &'static str = "CTCDoor";
}

impl ComponentBody for Door {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "DoorTriggerType" => self.door_trigger_type.read_field(rest, value),
            "Open" => self.open.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCEditor` — on all 21,800 things. The world editor's own state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Editor {
    pub locked_in_place: Option<bool>,
}

impl Editor {
    /// The block name, as written: `StartCTCEditor` … `EndCTCEditor`.
    pub const CLASS: &'static str = "CTCEditor";
}

impl ComponentBody for Editor {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "LockedInPlace" => self.locked_in_place.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCEnemy` — 861 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Enemy {
    pub enable_followers_enemy_proxy: Option<bool>,
    pub faction_name: Option<String>,
    pub friends_with_everything_flag: bool,
}

impl Enemy {
    /// The block name, as written: `StartCTCEnemy` … `EndCTCEnemy`.
    pub const CLASS: &'static str = "CTCEnemy";
}

impl ComponentBody for Enemy {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "EnableFollowersEnemyProxy" => {
                self.enable_followers_enemy_proxy.read_field(rest, value)
            }
            "FactionName" => self.faction_name.read_field(rest, value),
            "FriendsWithEverythingFlag" => {
                self.friends_with_everything_flag.read_field(rest, value)
            }
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCExplodingObject` — 56 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExplodingObject {
    pub fire_damage: i32,
    pub max_damage: f32,
    pub radius: f32,
    pub trigger_radius: f32,
    pub triggered_on_creature_proximity: bool,
}

impl ExplodingObject {
    /// The block name, as written: `StartCTCExplodingObject` … `EndCTCExplodingObject`.
    pub const CLASS: &'static str = "CTCExplodingObject";
}

impl ComponentBody for ExplodingObject {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "FireDamage" => self.fire_damage.read_field(rest, value),
            "MaxDamage" => self.max_damage.read_field(rest, value),
            "Radius" => self.radius.read_field(rest, value),
            "TriggerRadius" => self.trigger_radius.read_field(rest, value),
            "TriggeredOnCreatureProximity" => {
                self.triggered_on_creature_proximity.read_field(rest, value)
            }
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCFishingSpot` — 32 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FishingSpot {}

impl FishingSpot {
    /// The block name, as written: `StartCTCFishingSpot` … `EndCTCFishingSpot`.
    pub const CLASS: &'static str = "CTCFishingSpot";
}

impl ComponentBody for FishingSpot {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCGuard` — 154 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Guard {
    pub bribe_pool: i32,
    pub frame_last_bribe_added: i32,
    pub frame_last_crime_seen: i32,
    pub frame_last_received_apology: i32,
    pub frame_pending_crimes_added: i32,
    pub last_crime_seen_severity: i32,
}

impl Guard {
    /// The block name, as written: `StartCTCGuard` … `EndCTCGuard`.
    pub const CLASS: &'static str = "CTCGuard";
}

impl ComponentBody for Guard {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "BribePool" => self.bribe_pool.read_field(rest, value),
            "FrameLastBribeAdded" => self.frame_last_bribe_added.read_field(rest, value),
            "FrameLastCrimeSeen" => self.frame_last_crime_seen.read_field(rest, value),
            "FrameLastReceivedApology" => self.frame_last_received_apology.read_field(rest, value),
            "FramePendingCrimesAdded" => self.frame_pending_crimes_added.read_field(rest, value),
            "LastCrimeSeenSeverity" => self.last_crime_seen_severity.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCHero` — 27 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hero {
    pub last_weapon_equipped_id: i32,
    /// Written in snake case by the game, unlike every other field.
    pub hero_title_object_def_name: String,
}

impl Hero {
    /// The block name, as written: `StartCTCHero` … `EndCTCHero`.
    pub const CLASS: &'static str = "CTCHero";
}

impl ComponentBody for Hero {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "LastWeaponEquippedID" => self.last_weapon_equipped_id.read_field(rest, value),
            "hero_title_object_def_name" => self.hero_title_object_def_name.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCHeroCentreDoorMarker` — 1 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeroCentreDoorMarker {
    pub door_type2: i32,
    pub radius: f32,
}

impl HeroCentreDoorMarker {
    /// The block name, as written: `StartCTCHeroCentreDoorMarker` … `EndCTCHeroCentreDoorMarker`.
    pub const CLASS: &'static str = "CTCHeroCentreDoorMarker";
}

impl ComponentBody for HeroCentreDoorMarker {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "DoorType2" => self.door_type2.read_field(rest, value),
            "Radius" => self.radius.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCInfoDisplay` — 66 in the game. Signposts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InfoDisplay {
    pub display_time: f32,
    pub radius: f32,
    pub text_tag: String,
    pub text_tag_back: String,
}

impl InfoDisplay {
    /// The block name, as written: `StartCTCInfoDisplay` … `EndCTCInfoDisplay`.
    pub const CLASS: &'static str = "CTCInfoDisplay";
}

impl ComponentBody for InfoDisplay {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "DisplayTime" => self.display_time.read_field(rest, value),
            "Radius" => self.radius.read_field(rest, value),
            "TextTag" => self.text_tag.read_field(rest, value),
            "TextTagBack" => self.text_tag_back.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCInventoryItem` — 79 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InventoryItem {
    pub inventory_uid: u64,
}

impl InventoryItem {
    /// The block name, as written: `StartCTCInventoryItem` … `EndCTCInventoryItem`.
    pub const CLASS: &'static str = "CTCInventoryItem";
}

impl ComponentBody for InventoryItem {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "InventoryUID" => self.inventory_uid.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCLight` — 989 in the game. A point light.
///
/// Everything but `Active` and `Overridden` is absent unless the placement
/// overrides the def's own values, which is what `Overridden` records.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Light {
    pub active: bool,
    pub colour: Option<Rgba>,
    pub flicker: Option<f32>,
    pub inner_radius: Option<f32>,
    pub inverted: Option<bool>,
    pub outer_radius: Option<f32>,
    pub overridden: bool,
}

impl Light {
    /// The block name, as written: `StartCTCLight` … `EndCTCLight`.
    pub const CLASS: &'static str = "CTCLight";
}

impl ComponentBody for Light {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Active" => self.active.read_field(rest, value),
            "Colour" => self.colour.read_field(rest, value),
            "Flicker" => self.flicker.read_field(rest, value),
            "InnerRadius" => self.inner_radius.read_field(rest, value),
            "Inverted" => self.inverted.read_field(rest, value),
            "OuterRadius" => self.outer_radius.read_field(rest, value),
            "Overridden" => self.overridden.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCObjectAugmentations` — 5 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectAugmentations {
    pub augmentation_def_names: Vec<String>,
    pub saved_in_game: bool,
}

impl ObjectAugmentations {
    /// The block name, as written: `StartCTCObjectAugmentations` … `EndCTCObjectAugmentations`.
    pub const CLASS: &'static str = "CTCObjectAugmentations";
}

impl ComponentBody for ObjectAugmentations {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "AugmentationDefNames" => self.augmentation_def_names.read_field(rest, value),
            "SavedInGame" => self.saved_in_game.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCOwnedEntity` — 2,424 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OwnedEntity {
    pub owner_uid: u64,
    pub switchable_navigation_tc_added: bool,
    pub version_number: i32,
}

impl OwnedEntity {
    /// The block name, as written: `StartCTCOwnedEntity` … `EndCTCOwnedEntity`.
    pub const CLASS: &'static str = "CTCOwnedEntity";
}

impl ComponentBody for OwnedEntity {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "OwnerUID" => self.owner_uid.read_field(rest, value),
            "SwitchableNavigationTCAdded" => {
                self.switchable_navigation_tc_added.read_field(rest, value)
            }
            "VersionNumber" => self.version_number.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCRandomAppearanceMorph` — 553 in the game, all `AICreature`s. The seed
/// that picks a villager's texture set.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RandomAppearanceMorph {
    pub seed: i32,
}

impl RandomAppearanceMorph {
    /// The block name, as written: `StartCTCRandomAppearanceMorph` … `EndCTCRandomAppearanceMorph`.
    pub const CLASS: &'static str = "CTCRandomAppearanceMorph";
}

impl ComponentBody for RandomAppearanceMorph {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Seed" => self.seed.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCShapeManager` — 38 in the game. Named regions drawn as polylines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeManager {
    pub is_coords_relative_to_map: bool,
    pub shapes: Vec<Shape>,
    /// Equals `shapes.len()` on every shape manager in the game.
    pub num_shapes: Redundant,
}

impl ShapeManager {
    /// The block name, as written: `StartCTCShapeManager` … `EndCTCShapeManager`.
    pub const CLASS: &'static str = "CTCShapeManager";
}

impl ComponentBody for ShapeManager {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "IsCoordsRelativeToMap" => self.is_coords_relative_to_map.read_field(rest, value),
            "Shape" => self.shapes.read_field(rest, value),
            "NumShapes" => self.num_shapes.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCShop` — 40 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shop {}

impl Shop {
    /// The block name, as written: `StartCTCShop` … `EndCTCShop`.
    pub const CLASS: &'static str = "CTCShop";
}

impl ComponentBody for Shop {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCSpotLight` — 30 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpotLight {
    pub angle: f32,
    pub colour: Rgba,
    pub flicker: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub overridden: bool,
    pub width: f32,
}

impl SpotLight {
    /// The block name, as written: `StartCTCSpotLight` … `EndCTCSpotLight`.
    pub const CLASS: &'static str = "CTCSpotLight";
}

impl ComponentBody for SpotLight {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Angle" => self.angle.read_field(rest, value),
            "Colour" => self.colour.read_field(rest, value),
            "Flicker" => self.flicker.read_field(rest, value),
            "InnerRadius" => self.inner_radius.read_field(rest, value),
            "OuterRadius" => self.outer_radius.read_field(rest, value),
            "Overridden" => self.overridden.read_field(rest, value),
            "Width" => self.width.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCStealableItemLocation` — 1 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StealableItemLocation {
    pub radius_to_be_within: f32,
    pub radius_to_take_items_back_to: f32,
}

impl StealableItemLocation {
    /// The block name, as written: `StartCTCStealableItemLocation` … `EndCTCStealableItemLocation`.
    pub const CLASS: &'static str = "CTCStealableItemLocation";
}

impl ComponentBody for StealableItemLocation {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "RadiusToBeWithin" => self.radius_to_be_within.read_field(rest, value),
            "RadiusToTakeItemsBackTo" => self.radius_to_take_items_back_to.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCStockItem` — 58 in the game. A shop's wares.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StockItem {
    pub for_sale: bool,
    pub price: i32,
    pub stealable: bool,
}

impl StockItem {
    /// The block name, as written: `StartCTCStockItem` … `EndCTCStockItem`.
    pub const CLASS: &'static str = "CTCStockItem";
}

impl ComponentBody for StockItem {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "ForSale" => self.for_sale.read_field(rest, value),
            "Price" => self.price.read_field(rest, value),
            "Stealable" => self.stealable.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCTalk` — 772 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Talk {}

impl Talk {
    /// The block name, as written: `StartCTCTalk` … `EndCTCTalk`.
    pub const CLASS: &'static str = "CTCTalk";
}

impl ComponentBody for Talk {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCTargeted` — 2,289 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Targeted {
    pub targetable: bool,
}

impl Targeted {
    /// The block name, as written: `StartCTCTargeted` … `EndCTCTargeted`.
    pub const CLASS: &'static str = "CTCTargeted";
}

impl ComponentBody for Targeted {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "Targetable" => self.targetable.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCTeleporter` — 13 in the game. No serialised state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Teleporter {}

impl Teleporter {
    /// The block name, as written: `StartCTCTeleporter` … `EndCTCTeleporter`.
    pub const CLASS: &'static str = "CTCTeleporter";
}

impl ComponentBody for Teleporter {
    fn read_named(
        &mut self,
        _name: &str,
        _rest: &[PathSegment<'_>],
        _value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        // No serialised state: the block is a presence marker.
        Err(FieldErrorKind::UnknownField)
    }
}
/// `CTCTrophy` — 2 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Trophy {
    pub best_witnesses_ahead_to_date: i32,
    pub mountable: bool,
}

impl Trophy {
    /// The block name, as written: `StartCTCTrophy` … `EndCTCTrophy`.
    pub const CLASS: &'static str = "CTCTrophy";
}

impl ComponentBody for Trophy {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "BestWitnessesAheadToDate" => self.best_witnesses_ahead_to_date.read_field(rest, value),
            "Mountable" => self.mountable.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCVillage` — 51 in the game, one per `Village` thing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Village {
    pub current_is_hero_criminal: Option<bool>,
    pub frame_player_last_seen_by_guard: i32,
    pub has_been_initially_populated: bool,
    pub is_enemy_because_of_crime: bool,
    pub limbo: bool,
}

impl Village {
    /// The block name, as written: `StartCTCVillage` … `EndCTCVillage`.
    pub const CLASS: &'static str = "CTCVillage";
}

impl ComponentBody for Village {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "CurrentIsHeroCriminal" => self.current_is_hero_criminal.read_field(rest, value),
            "FramePlayerLastSeenByGuard" => {
                self.frame_player_last_seen_by_guard.read_field(rest, value)
            }
            "HasBeenInitiallyPopulated" => {
                self.has_been_initially_populated.read_field(rest, value)
            }
            "IsEnemyBecauseOfCrime" => self.is_enemy_because_of_crime.read_field(rest, value),
            "Limbo" => self.limbo.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCVillageMember` — 2,229 in the game. Links a thing to its village.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VillageMember {
    pub village_uid: u64,
}

impl VillageMember {
    /// The block name, as written: `StartCTCVillageMember` … `EndCTCVillageMember`.
    pub const CLASS: &'static str = "CTCVillageMember";
}

impl ComponentBody for VillageMember {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "VillageUID" => self.village_uid.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCWallMount` — 9 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WallMount {
    pub bought_for_amount: i32,
    pub trophy_id: i32,
}

impl WallMount {
    /// The block name, as written: `StartCTCWallMount` … `EndCTCWallMount`.
    pub const CLASS: &'static str = "CTCWallMount";
}

impl ComponentBody for WallMount {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "BoughtForAmount" => self.bought_for_amount.read_field(rest, value),
            "TrophyID" => self.trophy_id.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}
/// `CTCWife` — 66 in the game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wife {
    pub boolean_husband_appearances: Vec<bool>,
    pub courting_blocked: bool,
    pub divorced_hero: Option<bool>,
    pub frame_entered_attitude_hate: i32,
    pub frame_entered_love_with_husband_present_at_home: i32,
    pub frame_got_married_to_the_player: i32,
    pub frame_last_aware_of_husband: i32,
    pub frame_last_considered_giving_gift: i32,
    pub frame_last_culled_gifts_received: i32,
    pub frame_last_evaluated_gift_opinion: i32,
    pub frame_last_evaluated_love_attitude: i32,
    pub frame_last_gave_divorce_warning: i32,
    pub frame_last_gave_sex_offer: i32,
    pub frame_last_received_nice_gift: i32,
    pub frame_last_reduced_opinion: i32,
    pub frame_to_check_appearance_changes: i32,
    pub gift_giving_opinion_distance_from_max: f32,
    pub gift_giving_price_value: i32,
    pub gift_to_give_def: i32,
    pub has_been_in_love_with_player: bool,
    pub house_dressing_level_last_commented_on: i32,
    pub just_married: bool,
    pub last_fatness_change_point: f32,
    pub love_attitude_value: f32,
    pub needs_to_change_brain: bool,
    pub permitted_to_region_follow: bool,
    pub received_wedding_ring: bool,
}

impl Wife {
    /// The block name, as written: `StartCTCWife` … `EndCTCWife`.
    pub const CLASS: &'static str = "CTCWife";
}

impl ComponentBody for Wife {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        match name {
            "BooleanHusbandAppearances" => self.boolean_husband_appearances.read_field(rest, value),
            "CourtingBlocked" => self.courting_blocked.read_field(rest, value),
            "DivorcedHero" => self.divorced_hero.read_field(rest, value),
            "FrameEnteredAttitudeHate" => self.frame_entered_attitude_hate.read_field(rest, value),
            "FrameEnteredLoveWithHusbandPresentAtHome" => self
                .frame_entered_love_with_husband_present_at_home
                .read_field(rest, value),
            "FrameGotMarriedToThePlayer" => {
                self.frame_got_married_to_the_player.read_field(rest, value)
            }
            "FrameLastAwareOfHusband" => self.frame_last_aware_of_husband.read_field(rest, value),
            "FrameLastConsideredGivingGift" => self
                .frame_last_considered_giving_gift
                .read_field(rest, value),
            "FrameLastCulledGiftsReceived" => self
                .frame_last_culled_gifts_received
                .read_field(rest, value),
            "FrameLastEvaluatedGiftOpinion" => self
                .frame_last_evaluated_gift_opinion
                .read_field(rest, value),
            "FrameLastEvaluatedLoveAttitude" => self
                .frame_last_evaluated_love_attitude
                .read_field(rest, value),
            "FrameLastGaveDivorceWarning" => {
                self.frame_last_gave_divorce_warning.read_field(rest, value)
            }
            "FrameLastGaveSexOffer" => self.frame_last_gave_sex_offer.read_field(rest, value),
            "FrameLastReceivedNiceGift" => {
                self.frame_last_received_nice_gift.read_field(rest, value)
            }
            "FrameLastReducedOpinion" => self.frame_last_reduced_opinion.read_field(rest, value),
            "FrameToCheckAppearanceChanges" => self
                .frame_to_check_appearance_changes
                .read_field(rest, value),
            "GiftGivingOpinionDistanceFromMax" => self
                .gift_giving_opinion_distance_from_max
                .read_field(rest, value),
            "GiftGivingPriceValue" => self.gift_giving_price_value.read_field(rest, value),
            "GiftToGiveDef" => self.gift_to_give_def.read_field(rest, value),
            "HasBeenInLoveWithPlayer" => self.has_been_in_love_with_player.read_field(rest, value),
            "HouseDressingLevelLastCommentedOn" => self
                .house_dressing_level_last_commented_on
                .read_field(rest, value),
            "JustMarried" => self.just_married.read_field(rest, value),
            "LastFatnessChangePoint" => self.last_fatness_change_point.read_field(rest, value),
            "LoveAttitudeValue" => self.love_attitude_value.read_field(rest, value),
            "NeedsToChangeBrain" => self.needs_to_change_brain.read_field(rest, value),
            "PermittedToRegionFollow" => self.permitted_to_region_follow.read_field(rest, value),
            "ReceivedWeddingRing" => self.received_wedding_ring.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

/// `CTCPreCalculatedNavigationRoute` — 35 in the game.
///
/// Its route is written as eight *numbered* field pairs (`NavPosition0`…`7`,
/// `NavLayer0`…`7`) rather than an indexed array, so the steps are gathered by
/// hand. All 35 routes fill the first `NumberOfStepsOnRoute` slots with no gaps.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreCalculatedNavigationRoute {
    pub steps: Vec<NavStep>,
    pub thing_to_calculate_route_to_uid: u64,
    /// The game's own spelling.
    pub prec_calculated_navigation_route_version: i32,
    /// `NumberOfStepsOnRoute`. Equals `steps.len()` on all 35 routes.
    pub number_of_steps_on_route: Redundant,
}

/// The number of numbered `NavPosition`/`NavLayer` slots the format allows.
const NAV_STEPS: usize = 8;

impl PreCalculatedNavigationRoute {
    pub const CLASS: &'static str = "CTCPreCalculatedNavigationRoute";
}

impl ComponentBody for PreCalculatedNavigationRoute {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind> {
        /// `NavPosition3` → `Some(3)`, for a slot within range.
        fn slot(name: &str, prefix: &str) -> Option<usize> {
            let index: usize = name.strip_prefix(prefix)?.parse().ok()?;
            (index < NAV_STEPS).then_some(index)
        }

        fn grow(steps: &mut Vec<NavStep>, index: usize) {
            if index >= steps.len() {
                steps.resize(index + 1, NavStep::default());
            }
        }

        if let Some(index) = slot(name, "NavPosition") {
            grow(&mut self.steps, index);
            return self.steps[index].position.read_field(rest, value);
        }
        if let Some(index) = slot(name, "NavLayer") {
            grow(&mut self.steps, index);
            return self.steps[index].layer.read_field(rest, value);
        }
        match name {
            "ThingToCalculateRouteToUID" => {
                self.thing_to_calculate_route_to_uid.read_field(rest, value)
            }
            "PrecCalculatedNavigationRouteVersion" => self
                .prec_calculated_navigation_route_version
                .read_field(rest, value),
            "NumberOfStepsOnRoute" => self.number_of_steps_on_route.read_field(rest, value),
            _ => Err(FieldErrorKind::UnknownField),
        }
    }
}

// ── The component set ─────────────────────────────────────────────────────────

/// Every component a thing carries, one slot per interface.
///
/// This mirrors `CThing`'s `CVectorMap<ETCInterfaceType, CTCBase*>`: a component
/// is present or absent, never repeated, and the six families collapse into one
/// slot each. Slots are declared in the order the game writes them, which is a
/// total order over the classes — so iterating them reproduces the file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Components {
    pub physics: Option<Physics>,
    pub random_appearance_morph: Option<RandomAppearanceMorph>,
    pub targeted: Option<Targeted>,
    pub carried_action_use_read: Option<CarriedActionUseRead>,
    pub inventory_item: Option<InventoryItem>,
    pub talk: Option<Talk>,
    pub action_use: Option<ActionUse>,
    pub editor: Option<Editor>,
    pub atmos_player: Option<AtmosPlayer>,
    pub boasting_area: Option<BoastingArea>,
    pub creature_generator: Option<CreatureGenerator>,
    pub info_display: Option<InfoDisplay>,
    pub spot_light: Option<SpotLight>,
    pub stealable_item_location: Option<StealableItemLocation>,
    pub teleporter: Option<Teleporter>,
    pub trophy: Option<Trophy>,
    pub village: Option<Village>,
    pub village_member: Option<VillageMember>,
    pub door: Option<Door>,
    pub hero: Option<Hero>,
    pub light: Option<Light>,
    pub owned_entity: Option<OwnedEntity>,
    pub hero_centre_door_marker: Option<HeroCentreDoorMarker>,
    pub pre_calculated_navigation_route: Option<PreCalculatedNavigationRoute>,
    pub shop: Option<Shop>,
    pub buyable_house: Option<BuyableHouse>,
    pub stock_item: Option<StockItem>,
    pub object_augmentations: Option<ObjectAugmentations>,
    pub wall_mount: Option<WallMount>,
    pub activation_receptor: Option<ActivationReceptor>,
    pub activation_trigger: Option<ActivationTrigger>,
    pub creature_generator_creator: Option<CreatureGeneratorCreator>,
    pub container: Option<Container>,
    pub digging_spot: Option<DiggingSpot>,
    pub enemy: Option<Enemy>,
    pub creature_opinion_of_hero: Option<CreatureOpinionOfHero>,
    pub aiscratchpad: Option<AIScratchpad>,
    pub exploding_object: Option<ExplodingObject>,
    pub fishing_spot: Option<FishingSpot>,
    pub guard: Option<Guard>,
    pub wife: Option<Wife>,
    pub driver: Option<Driver>,
    pub camera_point: Option<CameraPoint>,
    pub action_use_scripted_hook: Option<ActionUseScriptedHook>,
    pub shape_manager: Option<ShapeManager>,
}

impl Components {
    /// Read one `StartCTC… … EndCTC…` block into its slot.
    ///
    /// Two things are refused rather than silently absorbed, because the engine's
    /// storage cannot express either: the same class twice, and two members of
    /// one interface family.
    pub(super) fn read_block(
        &mut self,
        class: &str,
        body: &Body<'_>,
    ) -> Result<(), ComponentError> {
        match class {
            "CTCPhysicsLight" => slot(
                &mut self.physics,
                class,
                Physics::Light(read::<PhysicsLight>(class, body)?),
            ),
            "CTCPhysicsStandard" => slot(
                &mut self.physics,
                class,
                Physics::Standard(read::<PhysicsStandard>(class, body)?),
            ),
            "CTCPhysicsNavigator" => slot(
                &mut self.physics,
                class,
                Physics::Navigator(read::<PhysicsStandard>(class, body)?),
            ),
            RandomAppearanceMorph::CLASS => slot(
                &mut self.random_appearance_morph,
                class,
                read::<RandomAppearanceMorph>(class, body)?,
            ),
            Targeted::CLASS => slot(&mut self.targeted, class, read::<Targeted>(class, body)?),
            CarriedActionUseRead::CLASS => slot(
                &mut self.carried_action_use_read,
                class,
                read::<CarriedActionUseRead>(class, body)?,
            ),
            InventoryItem::CLASS => slot(
                &mut self.inventory_item,
                class,
                read::<InventoryItem>(class, body)?,
            ),
            Talk::CLASS => slot(&mut self.talk, class, read::<Talk>(class, body)?),
            ActionUseBed::CLASS => slot(
                &mut self.action_use,
                class,
                ActionUse::Bed(read::<ActionUseBed>(class, body)?),
            ),
            ActionUseReadable::CLASS => slot(
                &mut self.action_use,
                class,
                ActionUse::Readable(read::<ActionUseReadable>(class, body)?),
            ),
            Editor::CLASS => slot(&mut self.editor, class, read::<Editor>(class, body)?),
            AtmosPlayer::CLASS => slot(
                &mut self.atmos_player,
                class,
                read::<AtmosPlayer>(class, body)?,
            ),
            BoastingArea::CLASS => slot(
                &mut self.boasting_area,
                class,
                read::<BoastingArea>(class, body)?,
            ),
            CreatureGenerator::CLASS => slot(
                &mut self.creature_generator,
                class,
                read::<CreatureGenerator>(class, body)?,
            ),
            InfoDisplay::CLASS => slot(
                &mut self.info_display,
                class,
                read::<InfoDisplay>(class, body)?,
            ),
            SpotLight::CLASS => slot(&mut self.spot_light, class, read::<SpotLight>(class, body)?),
            StealableItemLocation::CLASS => slot(
                &mut self.stealable_item_location,
                class,
                read::<StealableItemLocation>(class, body)?,
            ),
            Teleporter::CLASS => slot(
                &mut self.teleporter,
                class,
                read::<Teleporter>(class, body)?,
            ),
            Trophy::CLASS => slot(&mut self.trophy, class, read::<Trophy>(class, body)?),
            Village::CLASS => slot(&mut self.village, class, read::<Village>(class, body)?),
            VillageMember::CLASS => slot(
                &mut self.village_member,
                class,
                read::<VillageMember>(class, body)?,
            ),
            Door::CLASS => slot(&mut self.door, class, read::<Door>(class, body)?),
            Hero::CLASS => slot(&mut self.hero, class, read::<Hero>(class, body)?),
            Light::CLASS => slot(&mut self.light, class, read::<Light>(class, body)?),
            OwnedEntity::CLASS => slot(
                &mut self.owned_entity,
                class,
                read::<OwnedEntity>(class, body)?,
            ),
            HeroCentreDoorMarker::CLASS => slot(
                &mut self.hero_centre_door_marker,
                class,
                read::<HeroCentreDoorMarker>(class, body)?,
            ),
            PreCalculatedNavigationRoute::CLASS => slot(
                &mut self.pre_calculated_navigation_route,
                class,
                read::<PreCalculatedNavigationRoute>(class, body)?,
            ),
            Shop::CLASS => slot(&mut self.shop, class, read::<Shop>(class, body)?),
            BuyableHouse::CLASS => slot(
                &mut self.buyable_house,
                class,
                read::<BuyableHouse>(class, body)?,
            ),
            StockItem::CLASS => slot(&mut self.stock_item, class, read::<StockItem>(class, body)?),
            ObjectAugmentations::CLASS => slot(
                &mut self.object_augmentations,
                class,
                read::<ObjectAugmentations>(class, body)?,
            ),
            WallMount::CLASS => slot(&mut self.wall_mount, class, read::<WallMount>(class, body)?),
            ActivationReceptorCreatureGenerator::CLASS => slot(
                &mut self.activation_receptor,
                class,
                ActivationReceptor::CreatureGenerator(read::<ActivationReceptorCreatureGenerator>(
                    class, body,
                )?),
            ),
            ActivationReceptorDoor::CLASS => slot(
                &mut self.activation_receptor,
                class,
                ActivationReceptor::Door(read::<ActivationReceptorDoor>(class, body)?),
            ),
            ActivationTrigger::CLASS => slot(
                &mut self.activation_trigger,
                class,
                read::<ActivationTrigger>(class, body)?,
            ),
            CreatureGeneratorCreator::CLASS => slot(
                &mut self.creature_generator_creator,
                class,
                read::<CreatureGeneratorCreator>(class, body)?,
            ),
            ContainerRewardHero::CLASS => slot(
                &mut self.container,
                class,
                Container::RewardHero(read::<ContainerRewardHero>(class, body)?),
            ),
            Chest::CLASS => slot(
                &mut self.container,
                class,
                Container::Chest(read::<Chest>(class, body)?),
            ),
            SearchableContainer::CLASS => slot(
                &mut self.container,
                class,
                Container::Searchable(read::<SearchableContainer>(class, body)?),
            ),
            DiggingSpot::CLASS => slot(
                &mut self.digging_spot,
                class,
                read::<DiggingSpot>(class, body)?,
            ),
            Enemy::CLASS => slot(&mut self.enemy, class, read::<Enemy>(class, body)?),
            CreatureOpinionOfHero::CLASS => slot(
                &mut self.creature_opinion_of_hero,
                class,
                read::<CreatureOpinionOfHero>(class, body)?,
            ),
            AIScratchpad::CLASS => slot(
                &mut self.aiscratchpad,
                class,
                read::<AIScratchpad>(class, body)?,
            ),
            ExplodingObject::CLASS => slot(
                &mut self.exploding_object,
                class,
                read::<ExplodingObject>(class, body)?,
            ),
            FishingSpot::CLASS => slot(
                &mut self.fishing_spot,
                class,
                read::<FishingSpot>(class, body)?,
            ),
            Guard::CLASS => slot(&mut self.guard, class, read::<Guard>(class, body)?),
            Wife::CLASS => slot(&mut self.wife, class, read::<Wife>(class, body)?),
            DNavigationSeed::CLASS => slot(
                &mut self.driver,
                class,
                Driver::NavigationSeed(read::<DNavigationSeed>(class, body)?),
            ),
            DRegionEntrance::CLASS => slot(
                &mut self.driver,
                class,
                Driver::RegionEntrance(read::<DRegionEntrance>(class, body)?),
            ),
            DRegionExit::CLASS => slot(
                &mut self.driver,
                class,
                Driver::RegionExit(read::<DRegionExit>(class, body)?),
            ),
            DCameraPoint::CLASS => slot(
                &mut self.driver,
                class,
                Driver::CameraPoint(read::<DCameraPoint>(class, body)?),
            ),
            DParticleEmitter::CLASS => slot(
                &mut self.driver,
                class,
                Driver::ParticleEmitter(read::<DParticleEmitter>(class, body)?),
            ),
            CameraPointFixedPoint::CLASS => slot(
                &mut self.camera_point,
                class,
                CameraPoint::FixedPoint(read::<CameraPointFixedPoint>(class, body)?),
            ),
            CameraPointGeneralCase::CLASS => slot(
                &mut self.camera_point,
                class,
                CameraPoint::GeneralCase(read::<CameraPointGeneralCase>(class, body)?),
            ),
            CameraPointScripted::CLASS => slot(
                &mut self.camera_point,
                class,
                CameraPoint::Scripted(read::<CameraPointScripted>(class, body)?),
            ),
            CameraPointScriptedSpline::CLASS => slot(
                &mut self.camera_point,
                class,
                CameraPoint::ScriptedSpline(read::<CameraPointScriptedSpline>(class, body)?),
            ),
            CameraPointTrack::CLASS => slot(
                &mut self.camera_point,
                class,
                CameraPoint::Track(read::<CameraPointTrack>(class, body)?),
            ),
            ActionUseScriptedHook::CLASS => slot(
                &mut self.action_use_scripted_hook,
                class,
                read::<ActionUseScriptedHook>(class, body)?,
            ),
            ShapeManager::CLASS => slot(
                &mut self.shape_manager,
                class,
                read::<ShapeManager>(class, body)?,
            ),
            _ => Err(ComponentError::UnknownClass),
        }
    }

    /// Every component present, in the order the game writes them.
    pub fn iter(&self) -> impl Iterator<Item = ComponentRef<'_>> {
        [
            self.physics.as_ref().map(ComponentRef::Physics),
            self.random_appearance_morph
                .as_ref()
                .map(ComponentRef::RandomAppearanceMorph),
            self.targeted.as_ref().map(ComponentRef::Targeted),
            self.carried_action_use_read
                .as_ref()
                .map(ComponentRef::CarriedActionUseRead),
            self.inventory_item
                .as_ref()
                .map(ComponentRef::InventoryItem),
            self.talk.as_ref().map(ComponentRef::Talk),
            self.action_use.as_ref().map(ComponentRef::ActionUse),
            self.editor.as_ref().map(ComponentRef::Editor),
            self.atmos_player.as_ref().map(ComponentRef::AtmosPlayer),
            self.boasting_area.as_ref().map(ComponentRef::BoastingArea),
            self.creature_generator
                .as_ref()
                .map(ComponentRef::CreatureGenerator),
            self.info_display.as_ref().map(ComponentRef::InfoDisplay),
            self.spot_light.as_ref().map(ComponentRef::SpotLight),
            self.stealable_item_location
                .as_ref()
                .map(ComponentRef::StealableItemLocation),
            self.teleporter.as_ref().map(ComponentRef::Teleporter),
            self.trophy.as_ref().map(ComponentRef::Trophy),
            self.village.as_ref().map(ComponentRef::Village),
            self.village_member
                .as_ref()
                .map(ComponentRef::VillageMember),
            self.door.as_ref().map(ComponentRef::Door),
            self.hero.as_ref().map(ComponentRef::Hero),
            self.light.as_ref().map(ComponentRef::Light),
            self.owned_entity.as_ref().map(ComponentRef::OwnedEntity),
            self.hero_centre_door_marker
                .as_ref()
                .map(ComponentRef::HeroCentreDoorMarker),
            self.pre_calculated_navigation_route
                .as_ref()
                .map(ComponentRef::PreCalculatedNavigationRoute),
            self.shop.as_ref().map(ComponentRef::Shop),
            self.buyable_house.as_ref().map(ComponentRef::BuyableHouse),
            self.stock_item.as_ref().map(ComponentRef::StockItem),
            self.object_augmentations
                .as_ref()
                .map(ComponentRef::ObjectAugmentations),
            self.wall_mount.as_ref().map(ComponentRef::WallMount),
            self.activation_receptor
                .as_ref()
                .map(ComponentRef::ActivationReceptor),
            self.activation_trigger
                .as_ref()
                .map(ComponentRef::ActivationTrigger),
            self.creature_generator_creator
                .as_ref()
                .map(ComponentRef::CreatureGeneratorCreator),
            self.container.as_ref().map(ComponentRef::Container),
            self.digging_spot.as_ref().map(ComponentRef::DiggingSpot),
            self.enemy.as_ref().map(ComponentRef::Enemy),
            self.creature_opinion_of_hero
                .as_ref()
                .map(ComponentRef::CreatureOpinionOfHero),
            self.aiscratchpad.as_ref().map(ComponentRef::AIScratchpad),
            self.exploding_object
                .as_ref()
                .map(ComponentRef::ExplodingObject),
            self.fishing_spot.as_ref().map(ComponentRef::FishingSpot),
            self.guard.as_ref().map(ComponentRef::Guard),
            self.wife.as_ref().map(ComponentRef::Wife),
            self.driver.as_ref().map(ComponentRef::Driver),
            self.camera_point.as_ref().map(ComponentRef::CameraPoint),
            self.action_use_scripted_hook
                .as_ref()
                .map(ComponentRef::ActionUseScriptedHook),
            self.shape_manager.as_ref().map(ComponentRef::ShapeManager),
        ]
        .into_iter()
        .flatten()
    }
}

/// A borrowed component, for iterating a [`Components`] generically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentRef<'a> {
    Physics(&'a Physics),
    RandomAppearanceMorph(&'a RandomAppearanceMorph),
    Targeted(&'a Targeted),
    ActionUse(&'a ActionUse),
    CarriedActionUseRead(&'a CarriedActionUseRead),
    InventoryItem(&'a InventoryItem),
    Talk(&'a Talk),
    Editor(&'a Editor),
    AtmosPlayer(&'a AtmosPlayer),
    BoastingArea(&'a BoastingArea),
    CreatureGenerator(&'a CreatureGenerator),
    ActivationReceptor(&'a ActivationReceptor),
    ActivationTrigger(&'a ActivationTrigger),
    CreatureGeneratorCreator(&'a CreatureGeneratorCreator),
    Driver(&'a Driver),
    InfoDisplay(&'a InfoDisplay),
    SpotLight(&'a SpotLight),
    StealableItemLocation(&'a StealableItemLocation),
    Teleporter(&'a Teleporter),
    Trophy(&'a Trophy),
    Village(&'a Village),
    VillageMember(&'a VillageMember),
    Door(&'a Door),
    Hero(&'a Hero),
    Light(&'a Light),
    OwnedEntity(&'a OwnedEntity),
    Container(&'a Container),
    CameraPoint(&'a CameraPoint),
    DiggingSpot(&'a DiggingSpot),
    Enemy(&'a Enemy),
    CreatureOpinionOfHero(&'a CreatureOpinionOfHero),
    AIScratchpad(&'a AIScratchpad),
    ExplodingObject(&'a ExplodingObject),
    FishingSpot(&'a FishingSpot),
    Guard(&'a Guard),
    HeroCentreDoorMarker(&'a HeroCentreDoorMarker),
    PreCalculatedNavigationRoute(&'a PreCalculatedNavigationRoute),
    ShapeManager(&'a ShapeManager),
    Shop(&'a Shop),
    BuyableHouse(&'a BuyableHouse),
    StockItem(&'a StockItem),
    ObjectAugmentations(&'a ObjectAugmentations),
    ActionUseScriptedHook(&'a ActionUseScriptedHook),
    WallMount(&'a WallMount),
    Wife(&'a Wife),
}

impl ComponentRef<'_> {
    /// The `CTC…` class this block names.
    pub fn class(&self) -> &'static str {
        match self {
            ComponentRef::Physics(c) => c.class(),
            ComponentRef::RandomAppearanceMorph(_) => RandomAppearanceMorph::CLASS,
            ComponentRef::Targeted(_) => Targeted::CLASS,
            ComponentRef::ActionUse(c) => c.class(),
            ComponentRef::CarriedActionUseRead(_) => CarriedActionUseRead::CLASS,
            ComponentRef::InventoryItem(_) => InventoryItem::CLASS,
            ComponentRef::Talk(_) => Talk::CLASS,
            ComponentRef::Editor(_) => Editor::CLASS,
            ComponentRef::AtmosPlayer(_) => AtmosPlayer::CLASS,
            ComponentRef::BoastingArea(_) => BoastingArea::CLASS,
            ComponentRef::CreatureGenerator(_) => CreatureGenerator::CLASS,
            ComponentRef::ActivationReceptor(c) => c.class(),
            ComponentRef::ActivationTrigger(_) => ActivationTrigger::CLASS,
            ComponentRef::CreatureGeneratorCreator(_) => CreatureGeneratorCreator::CLASS,
            ComponentRef::Driver(c) => c.class(),
            ComponentRef::InfoDisplay(_) => InfoDisplay::CLASS,
            ComponentRef::SpotLight(_) => SpotLight::CLASS,
            ComponentRef::StealableItemLocation(_) => StealableItemLocation::CLASS,
            ComponentRef::Teleporter(_) => Teleporter::CLASS,
            ComponentRef::Trophy(_) => Trophy::CLASS,
            ComponentRef::Village(_) => Village::CLASS,
            ComponentRef::VillageMember(_) => VillageMember::CLASS,
            ComponentRef::Door(_) => Door::CLASS,
            ComponentRef::Hero(_) => Hero::CLASS,
            ComponentRef::Light(_) => Light::CLASS,
            ComponentRef::OwnedEntity(_) => OwnedEntity::CLASS,
            ComponentRef::Container(c) => c.class(),
            ComponentRef::CameraPoint(c) => c.class(),
            ComponentRef::DiggingSpot(_) => DiggingSpot::CLASS,
            ComponentRef::Enemy(_) => Enemy::CLASS,
            ComponentRef::CreatureOpinionOfHero(_) => CreatureOpinionOfHero::CLASS,
            ComponentRef::AIScratchpad(_) => AIScratchpad::CLASS,
            ComponentRef::ExplodingObject(_) => ExplodingObject::CLASS,
            ComponentRef::FishingSpot(_) => FishingSpot::CLASS,
            ComponentRef::Guard(_) => Guard::CLASS,
            ComponentRef::HeroCentreDoorMarker(_) => HeroCentreDoorMarker::CLASS,
            ComponentRef::PreCalculatedNavigationRoute(_) => PreCalculatedNavigationRoute::CLASS,
            ComponentRef::ShapeManager(_) => ShapeManager::CLASS,
            ComponentRef::Shop(_) => Shop::CLASS,
            ComponentRef::BuyableHouse(_) => BuyableHouse::CLASS,
            ComponentRef::StockItem(_) => StockItem::CLASS,
            ComponentRef::ObjectAugmentations(_) => ObjectAugmentations::CLASS,
            ComponentRef::ActionUseScriptedHook(_) => ActionUseScriptedHook::CLASS,
            ComponentRef::WallMount(_) => WallMount::CLASS,
            ComponentRef::Wife(_) => Wife::CLASS,
        }
    }
}

/// Why a component block could not be stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentError {
    /// A `CTC…` class this model does not know.
    UnknownClass,
    /// The same class appeared twice on one thing.
    Duplicate,
    /// Two members of one interface family — `CThing` has one slot for both.
    Conflict { existing: &'static str },
    /// A field inside the block. `offset` is the failing statement's own byte
    /// offset, so the error points at the field and not at the block that holds
    /// it — components run to 27 fields.
    Field {
        offset: usize,
        path: String,
        kind: FieldErrorKind,
    },
}

/// Fill an empty slot, refusing a duplicate or a family conflict.
fn slot<T: SlotClass>(slot: &mut Option<T>, class: &str, value: T) -> Result<(), ComponentError> {
    match slot {
        Some(existing) if existing.class() == class => Err(ComponentError::Duplicate),
        Some(existing) => Err(ComponentError::Conflict {
            existing: existing.class(),
        }),
        None => {
            *slot = Some(value);
            Ok(())
        }
    }
}

/// The class name of whatever is in a slot — a leaf's own, or the family
/// member's.
pub trait SlotClass {
    fn class(&self) -> &'static str;
}

/// A component body: the fields between `StartCTC…` and `EndCTC…`.
///
/// One impl per class, so the field list of each is a plain `match` a reader can
/// follow without expanding a macro.
pub trait ComponentBody: Default {
    fn read_named(
        &mut self,
        name: &str,
        rest: &[PathSegment<'_>],
        value: &Value<'_>,
    ) -> Result<(), FieldErrorKind>;
}

/// Read a whole block into a fresh `T`.
fn read<T: ComponentBody>(class: &str, body: &Body<'_>) -> Result<T, ComponentError> {
    let _ = class;
    let mut out = T::default();
    for statement in &body.statements {
        let offset = statement.span.start;
        let (path, kind) = match &statement.value {
            Statement::Field(field) => {
                let result = match field.path.segments.as_slice() {
                    [PathSegment::Field(name), rest @ ..] => {
                        out.read_named(name, rest, &field.value.value)
                    }
                    _ => Err(FieldErrorKind::UnknownField),
                };
                match result {
                    Ok(()) => continue,
                    Err(kind) => (field.path.to_string(), kind),
                }
            }
            Statement::Flag(name) => ((*name).to_string(), FieldErrorKind::MissingValue),
            Statement::Block(block) => (
                block.keyword.to_string(),
                FieldErrorKind::UnexpectedNestedBlock,
            ),
        };
        return Err(ComponentError::Field { offset, path, kind });
    }
    Ok(out)
}

impl SlotClass for Physics {
    fn class(&self) -> &'static str {
        Physics::class(self)
    }
}

impl SlotClass for RandomAppearanceMorph {
    fn class(&self) -> &'static str {
        RandomAppearanceMorph::CLASS
    }
}

impl SlotClass for Targeted {
    fn class(&self) -> &'static str {
        Targeted::CLASS
    }
}

impl SlotClass for ActionUse {
    fn class(&self) -> &'static str {
        ActionUse::class(self)
    }
}

impl SlotClass for CarriedActionUseRead {
    fn class(&self) -> &'static str {
        CarriedActionUseRead::CLASS
    }
}

impl SlotClass for InventoryItem {
    fn class(&self) -> &'static str {
        InventoryItem::CLASS
    }
}

impl SlotClass for Talk {
    fn class(&self) -> &'static str {
        Talk::CLASS
    }
}

impl SlotClass for Editor {
    fn class(&self) -> &'static str {
        Editor::CLASS
    }
}

impl SlotClass for AtmosPlayer {
    fn class(&self) -> &'static str {
        AtmosPlayer::CLASS
    }
}

impl SlotClass for BoastingArea {
    fn class(&self) -> &'static str {
        BoastingArea::CLASS
    }
}

impl SlotClass for CreatureGenerator {
    fn class(&self) -> &'static str {
        CreatureGenerator::CLASS
    }
}

impl SlotClass for ActivationReceptor {
    fn class(&self) -> &'static str {
        ActivationReceptor::class(self)
    }
}

impl SlotClass for ActivationTrigger {
    fn class(&self) -> &'static str {
        ActivationTrigger::CLASS
    }
}

impl SlotClass for CreatureGeneratorCreator {
    fn class(&self) -> &'static str {
        CreatureGeneratorCreator::CLASS
    }
}

impl SlotClass for Driver {
    fn class(&self) -> &'static str {
        Driver::class(self)
    }
}

impl SlotClass for InfoDisplay {
    fn class(&self) -> &'static str {
        InfoDisplay::CLASS
    }
}

impl SlotClass for SpotLight {
    fn class(&self) -> &'static str {
        SpotLight::CLASS
    }
}

impl SlotClass for StealableItemLocation {
    fn class(&self) -> &'static str {
        StealableItemLocation::CLASS
    }
}

impl SlotClass for Teleporter {
    fn class(&self) -> &'static str {
        Teleporter::CLASS
    }
}

impl SlotClass for Trophy {
    fn class(&self) -> &'static str {
        Trophy::CLASS
    }
}

impl SlotClass for Village {
    fn class(&self) -> &'static str {
        Village::CLASS
    }
}

impl SlotClass for VillageMember {
    fn class(&self) -> &'static str {
        VillageMember::CLASS
    }
}

impl SlotClass for Door {
    fn class(&self) -> &'static str {
        Door::CLASS
    }
}

impl SlotClass for Hero {
    fn class(&self) -> &'static str {
        Hero::CLASS
    }
}

impl SlotClass for Light {
    fn class(&self) -> &'static str {
        Light::CLASS
    }
}

impl SlotClass for OwnedEntity {
    fn class(&self) -> &'static str {
        OwnedEntity::CLASS
    }
}

impl SlotClass for Container {
    fn class(&self) -> &'static str {
        Container::class(self)
    }
}

impl SlotClass for CameraPoint {
    fn class(&self) -> &'static str {
        CameraPoint::class(self)
    }
}

impl SlotClass for DiggingSpot {
    fn class(&self) -> &'static str {
        DiggingSpot::CLASS
    }
}

impl SlotClass for Enemy {
    fn class(&self) -> &'static str {
        Enemy::CLASS
    }
}

impl SlotClass for CreatureOpinionOfHero {
    fn class(&self) -> &'static str {
        CreatureOpinionOfHero::CLASS
    }
}

impl SlotClass for AIScratchpad {
    fn class(&self) -> &'static str {
        AIScratchpad::CLASS
    }
}

impl SlotClass for ExplodingObject {
    fn class(&self) -> &'static str {
        ExplodingObject::CLASS
    }
}

impl SlotClass for FishingSpot {
    fn class(&self) -> &'static str {
        FishingSpot::CLASS
    }
}

impl SlotClass for Guard {
    fn class(&self) -> &'static str {
        Guard::CLASS
    }
}

impl SlotClass for HeroCentreDoorMarker {
    fn class(&self) -> &'static str {
        HeroCentreDoorMarker::CLASS
    }
}

impl SlotClass for PreCalculatedNavigationRoute {
    fn class(&self) -> &'static str {
        PreCalculatedNavigationRoute::CLASS
    }
}

impl SlotClass for ShapeManager {
    fn class(&self) -> &'static str {
        ShapeManager::CLASS
    }
}

impl SlotClass for Shop {
    fn class(&self) -> &'static str {
        Shop::CLASS
    }
}

impl SlotClass for BuyableHouse {
    fn class(&self) -> &'static str {
        BuyableHouse::CLASS
    }
}

impl SlotClass for StockItem {
    fn class(&self) -> &'static str {
        StockItem::CLASS
    }
}

impl SlotClass for ObjectAugmentations {
    fn class(&self) -> &'static str {
        ObjectAugmentations::CLASS
    }
}

impl SlotClass for ActionUseScriptedHook {
    fn class(&self) -> &'static str {
        ActionUseScriptedHook::CLASS
    }
}

impl SlotClass for WallMount {
    fn class(&self) -> &'static str {
        WallMount::CLASS
    }
}

impl SlotClass for Wife {
    fn class(&self) -> &'static str {
        Wife::CLASS
    }
}
