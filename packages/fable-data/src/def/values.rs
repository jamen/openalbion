//! Compound wire value types (`wire_struct!` / `def_variant!`).

use crate::{wire_struct, def_variant};
use crate::def::prelude::*;

wire_struct! {
    /// C++ `CRGBColour`.
    pub struct RGBColour {
        pub int_value: u32,
    }
}

wire_struct! {
    /// C++ `CEngineGraphic` — OG retail layout (the Anniversary build reordered
    /// it to bank…alpha,type; the spec documents the newer order). Retail
    /// bytes only fit type-first: `Type` is a 4-byte enum value and
    /// `AdditiveAlpha` a 1-byte bool.
    pub struct EngineGraphic {
        pub type_: i32,
        pub bank_index: i32,
        pub anim_step: f32,
        pub render_size_x: f32,
        pub additive_alpha: u8,
    }
}

wire_struct! {
    pub struct SoundMapF0EntryValue {
        pub first_sound: u32,
        pub last_sound: u32,
    }
}

wire_struct! {
    pub struct SoundMapF0Entry {
        pub key: u32,
        pub value: SoundMapF0EntryValue,
    }
}

wire_struct! {
    pub struct SoundMapF1Entry {
        pub key: u32,
        pub value: i32,
    }
}

wire_struct! {
    /// C++ `CSoundMap`.
    pub struct SoundMap {
        pub f0: Vec<SoundMapF0Entry>,
        pub f1: Vec<SoundMapF1Entry>,
    }
}

wire_struct! {
    pub struct WoundMorphsMorphsEntry {
        pub body_location: i32,
        pub data: [u8; 40],
    }
}

wire_struct! {
    /// C++ `CWoundMorphs`.
    pub struct WoundMorphs {
        pub morphs: Vec<WoundMorphsMorphsEntry>,
        pub trailing_u32: u32,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts0MeshesEntryTextureMorphsEntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts0MeshesEntryTextureMorphs {
        pub entries: Vec<RandomAppearanceMorphBodyParts0MeshesEntryTextureMorphsEntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts0MeshesEntry {
        pub mesh_id: i32,
        pub texture_id: i32,
        pub texture_morphs: RandomAppearanceMorphBodyParts0MeshesEntryTextureMorphs,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts0 {
        pub meshes: Vec<RandomAppearanceMorphBodyParts0MeshesEntry>,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts1MeshesEntryTextureMorphsEntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts1MeshesEntryTextureMorphs {
        pub entries: Vec<RandomAppearanceMorphBodyParts1MeshesEntryTextureMorphsEntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts1MeshesEntry {
        pub mesh_id: i32,
        pub texture_id: i32,
        pub texture_morphs: RandomAppearanceMorphBodyParts1MeshesEntryTextureMorphs,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts1 {
        pub meshes: Vec<RandomAppearanceMorphBodyParts1MeshesEntry>,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts2MeshesEntryTextureMorphsEntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts2MeshesEntryTextureMorphs {
        pub entries: Vec<RandomAppearanceMorphBodyParts2MeshesEntryTextureMorphsEntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts2MeshesEntry {
        pub mesh_id: i32,
        pub texture_id: i32,
        pub texture_morphs: RandomAppearanceMorphBodyParts2MeshesEntryTextureMorphs,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyParts2 {
        pub meshes: Vec<RandomAppearanceMorphBodyParts2MeshesEntry>,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs0EntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs0 {
        pub entries: Vec<RandomAppearanceMorphBodyPartTextureMorphs0EntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs1EntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs1 {
        pub entries: Vec<RandomAppearanceMorphBodyPartTextureMorphs1EntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs2EntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphBodyPartTextureMorphs2 {
        pub entries: Vec<RandomAppearanceMorphBodyPartTextureMorphs2EntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphSkeletalMorphsEntry {
        pub key: i32,
        pub morph_id: i32,
        pub bone_or_morph_index: i32,
        pub allow_variation: bool,
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphGeneralTextureMorphsEntriesEntry {
        pub key: u32,
        pub morph: [u8; 8],
    }
}

wire_struct! {
    pub struct RandomAppearanceMorphGeneralTextureMorphs {
        pub entries: Vec<RandomAppearanceMorphGeneralTextureMorphsEntriesEntry>,
        pub trailing: u32,
        pub dirty: u8,
    }
}

wire_struct! {
    /// C++ `CRandomAppearanceMorph`.
    pub struct RandomAppearanceMorph {
        pub num_body_parts: u32,
        pub body_parts0: RandomAppearanceMorphBodyParts0,
        pub body_parts1: RandomAppearanceMorphBodyParts1,
        pub body_parts2: RandomAppearanceMorphBodyParts2,
        pub body_part_texture_morphs0: RandomAppearanceMorphBodyPartTextureMorphs0,
        pub body_part_texture_morphs1: RandomAppearanceMorphBodyPartTextureMorphs1,
        pub body_part_texture_morphs2: RandomAppearanceMorphBodyPartTextureMorphs2,
        pub skeletal_morphs: Vec<RandomAppearanceMorphSkeletalMorphsEntry>,
        pub general_texture_morphs: RandomAppearanceMorphGeneralTextureMorphs,
        pub final_trailing: u32,
    }
}

wire_struct! {
    pub struct ExpressionSetExpressionsEntry {
        pub type_: i32,
        pub data1: u32,
        pub data2: u32,
    }
}

wire_struct! {
    /// C++ `CExpressionSet`.
    pub struct ExpressionSet {
        pub expressions: Vec<ExpressionSetExpressionsEntry>,
        pub trailing_u32: u32,
    }
}

wire_struct! {
    /// C++ `CFloatRange`.
    pub struct FloatRange {
        pub low_value: f32,
        pub high_value: f32,
    }
}

wire_struct! {
    pub struct ThingComponentSetEntriesEntry {
        pub data: [u8; 9],
    }
}

wire_struct! {
    /// C++ `CThingComponentSet`.
    pub struct ThingComponentSet {
        pub entries: Vec<ThingComponentSetEntriesEntry>,
        pub trailing_u32: u32,
    }
}

wire_struct! {
    /// C++ `COpinionTransientOffset` — 6 consecutive u32 values (24 bytes).
    /// The text form supplies 5 positional args: `(opinion_axis, peak, run_in,
    /// run_out, persist)`; the 6th field defaults to zero.
    ///
    /// KNOWN DIVERGENCE (OPINION_DEED_EFFECTS/SOURCE/PERSONALITY, ~99 entries):
    /// the retail compiled values are NOT a straight copy of the text args.
    /// `opinion_axis` and `peak` match, but the trailing fields are
    /// engine-*computed* at compile time (e.g. text
    /// `Add(OPINION_MORALITY,-0.1,0.0,100.0,-0.025)` → retail
    /// `{axis:0, peak:-0.1, run_in:<raw int 1>, run_out:0.0, persist:5e-5,
    /// f5:1500}`), converting human-readable seconds/persist into internal
    /// units. That transform is not present in the current decomp bodies, so
    /// these entries are deferred to the retail body by the linker until it is
    /// reverse-engineered. This is a value transform, NOT a field layout/type
    /// bug (layout is confirmed 24 bytes / 6 fields) or source drift.
    pub struct OpinionTransientOffset {
        pub opinion_axis: i32,
        pub peak: f32,
        pub run_in: f32,
        pub run_out: f32,
        pub persist: f32,
        pub f5: i32,
    }
}

wire_struct! {
    /// C++ `std::list<COpinionTransientOffset>`.
    pub struct OpinionTransientOffsetList {
        pub f0: Vec<OpinionTransientOffset>,
    }
}

wire_struct! {
    /// C++ `CCombatAbilityData`.
    pub struct CombatAbilityData {
        pub percentage_chance: i32,
        pub seconds_duration: f32,
        pub seconds_to_wait_before_repeat: f32,
        pub anim: DefString,
    }
}

wire_struct! {
    /// C++ `C3DVector`.
    pub struct Vector3D {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
}

wire_struct! {
    /// C++ `C2DVector`.
    pub struct Vector2D {
        pub x: f32,
        pub y: f32,
    }
}

wire_struct! {
    /// C++ `CMiniMapGraphics`.
    pub struct MiniMapGraphics {
        pub f0: VecMap<String, i32>,
    }
}

wire_struct! {
    /// C++ `NParticleAttachment::CParticleAttachmentInfo`.
    pub struct ParticleAttachmentInfo {
        pub particle_effect: DefString,
        pub offset: DefString,
        pub weight: f32,
        pub dummy_object: bool,
    }
}

wire_struct! {
    /// C++ `COpinionPersonalityTraits*`.
    pub struct OpinionPersonalityTraitsPtr {
        pub f0: [u8; 180],
    }
}

wire_struct! {
    /// C++ `CTextureReplacementDef::CEntry`.
    pub struct TextureReplacementEntry {
        pub original_texture: i32,
        pub new_texture: i32,
    }
}

wire_struct! {
    pub struct MapPathsF0Entry {
        pub id: u32,
        pub graphic: i32,
    }
}

wire_struct! {
    /// C++ `CMapPaths`.
    pub struct MapPaths {
        pub f0: Vec<MapPathsF0Entry>,
    }
}

wire_struct! {
    /// C++ `CComboMultiplierData`.
    pub struct ComboMultiplierData {
        pub experience_multiplier_num_seconds_for_attenuation: f32,
        pub experience_multiplier_param_decay_delta_per_second: f32,
        pub combo_multiplier: VecMap<DamageAttribute, f32>,
    }
}

wire_struct! {
    /// C++ `CHeroStatIncreaseData`.
    pub struct HeroStatIncreaseData {
        pub stat_increases: VecMap<HeroExperienceStatCategory, f32>,
    }
}

wire_struct! {
    pub struct AttackHistoryComboAttackComboEntry {
        pub type_: u32,
        pub attribute: u32,
    }
}

wire_struct! {
    /// C++ `CAttackHistoryCombo`.
    pub struct AttackHistoryCombo {
        pub attack_combo: Vec<AttackHistoryComboAttackComboEntry>,
        pub multiplier: f32,
    }
}

wire_struct! {
    /// C++ `CObjectFamilyEntry`.
    pub struct ObjectFamilyEntry {
        pub object: i32,
        pub probability: f32,
    }
}

wire_struct! {
    /// C++ `CObjectAugmentationParticleSet`.
    pub struct ObjectAugmentationParticleSet {
        pub period_secs: f32,
        pub particle_effect: i32,
        pub particle_effect_to_blend_to: i32,
        pub attack_particle_effect: i32,
        pub attack_particle_effect_to_blend_to: i32,
        pub start_point: Vec<DefString>,
        pub end_point: Vec<DefString>,
    }
}

def_variant! {
    pub enum PhysicalPrimitiveInitType: u32 {
        0 => Tag0 {},
        1 => Tag1 { base_name: i32, radius: f32 },
        2 => Tag2 { base_name: i32, radius: f32, height: f32 },
        3 => Tag3 {},
    }
}

wire_struct! {
    /// C++ `CPhysicalPrimitiveInit`.
    pub struct PhysicalPrimitiveInit {
        pub type_: PhysicalPrimitiveInitType,
    }
}

wire_struct! {
    /// C++ `CWeaponTrailGraphicSet`.
    pub struct WeaponTrailGraphicSet {
        pub attack: i32,
        pub knockdown: i32,
    }
}

def_variant! {
    pub enum AnimationEntryComponentsEntry: u32 {
        0 => Tag0 { flags: i32 },
        1 => Tag1 { transition_in_time: i32 },
        2 => Tag2 { delay: i32 },
        3 => Tag3 { start_handedness: i32, end_handedness: i32 },
        4 => Tag4 { combo_stage: i32, combo_id: i32 },
        5 => Tag5 { recoil_anim_index: i32 },
        6 => Tag6 { melee_flourish: bool, melee_knockdown: bool },
        7 => Tag7 { target_offset: bool, target_offset_vector_x: f32, target_offset_vector_y: f32 },
        8 => Tag8 { next_anim_name: i32 },
        9 => Tag9 { animation_speed_multiplier: f32 },
        10 => Tag10 { mode: i32 },
        11 => Tag11 { next_filter: i32 },
        12 => Tag12 { response_anim_name: PString },
    }
}

wire_struct! {
    /// C++ `CAnimationEntry`.
    pub struct AnimationEntry {
        pub bank_index: i32,
        pub anim_name: i32,
        pub group_name: i32,
        pub components: Vec<AnimationEntryComponentsEntry>,
    }
}

wire_struct! {
    pub struct AnimationSetAnimsEntry {
        pub key: u32,
        pub entry: AnimationEntry,
    }
}

wire_struct! {
    /// C++ `CAnimationSet`.
    pub struct AnimationSet {
        pub anims: Vec<AnimationSetAnimsEntry>,
        pub default_flags: u32,
        pub default_transition_time: u32,
        pub default_delay: u32,
        pub default_group: i32,
    }
}

wire_struct! {
    /// C++ `CSpecialEffectsStringMap`.
    pub struct SpecialEffectsStringMap {
        pub f0: VecMap<u32, i32>,
    }
}

wire_struct! {
    /// C++ `CFireEffectCreationSet`.
    pub struct FireEffectCreationSet {
        pub containment_map: VecMap<DefString, i32>,
    }
}

def_variant! {
    pub enum ReactionMatchListElementsEntry: u32 {
        0 => Tag0 { reaction_type: i32, axis: i32, lower_bound_on_axis: f32, inv_interval_on_axis: f32, m_shift_zero: f32, m_shift_weight: f32, r_shift_zero: f32, r_shift_weight: f32 },
        1 => Tag1 { reaction_type: i32, axis: i32, centre_on_axis: f32, radius_on_axis: f32, m_shift_zero: f32, m_shift_weight: f32, r_shift_zero: f32, r_shift_weight: f32 },
        2 => Tag2 { reaction_type: i32, x_axis_opinion_type: i32, y_axis_opinion_type: i32, x_centre: f32, neg_inv_x_radius: f32, pos_inv_x_radius: f32, y_centre: f32, neg_inv_y_radius: f32, pos_inv_y_radius: f32 },
        3 => Tag3 { reaction_type: i32, x_axis_opinion_type: i32, y_axis_opinion_type: i32, x_centre: f32, neg_inv_x_radius: f32, pos_inv_x_radius: f32, y_centre: f32, neg_inv_y_radius: f32, pos_inv_y_radius: f32, scariness_shift: f32, agreeableness_shift: f32, attractiveness_shift: f32 },
    }
}

wire_struct! {
    /// C++ `std::list<CReactionMatch *,std::allocator<CReactionMatch *>>`.
    pub struct ReactionMatchList {
        pub elements: Vec<ReactionMatchListElementsEntry>,
    }
}

def_variant! {
    pub enum ReactionFrequencyTraitsArrayTraitsEntry: u32 {
        0 => Tag0 {},
        1 => Tag1 { min_wait: f32, wait_range: f32 },
        2 => Tag2 { low_freq_per_min: f32, high_low_freq_interval: f32, frames_per_minute: u32, ramp_up_time_frames: u32, cutoff_interval_frames: u32 },
        3 => Tag3 { allow_individual_repeats: bool, min_gap_frames: u32, count_recharge_per_frame: f32, current_available_count: f32, max_count: f32, inv_max_count: f32 },
    }
}

wire_struct! {
    /// C++ `CReactionFrequencyTraitsArray`.
    pub struct ReactionFrequencyTraitsArray {
        pub traits: [ReactionFrequencyTraitsArrayTraitsEntry; 158],
    }
}

wire_struct! {
    pub struct SimVoiceVoicesEntry {
        pub speech_type: ReactionSpeechType,
        pub voice_id: i32,
    }
}

wire_struct! {
    /// C++ `CSimVoice`.
    pub struct SimVoice {
        pub voices: Vec<SimVoiceVoicesEntry>,
    }
}

wire_struct! {
    pub struct AppearanceModifierGraphicsGraphicsEntry {
        pub data: [u8; 24],
    }
}

wire_struct! {
    /// C++ `CAppearanceModifierGraphics`.
    pub struct AppearanceModifierGraphics {
        pub graphics: Vec<AppearanceModifierGraphicsGraphicsEntry>,
    }
}

wire_struct! {
    /// C++ `CBrainUpdateZone`.
    pub struct BrainUpdateZone {
        pub distance: i32,
        pub update_frequency: i32,
        pub interruption_frequency: i32,
    }
}

wire_struct! {
    /// C++ `CBrainBehaviour`.
    pub struct BrainBehaviour {
        pub name: DefString,
        pub priority: i32,
        pub script_ai_priority: i32,
    }
}

wire_struct! {
    /// C++ `CBlendedParticleEffectSet`.
    pub struct BlendedParticleEffectSet {
        pub particle1_index: i32,
        pub particle2_index: i32,
        pub particle1_value: f32,
        pub particle2_value: f32,
    }
}

wire_struct! {
    pub struct TextureMorphsMorphsEntry {
        pub morph_type: i32,
        pub data: [u8; 20],
    }
}

wire_struct! {
    /// C++ `CTextureMorphs`.
    pub struct TextureMorphs {
        pub morphs: Vec<TextureMorphsMorphsEntry>,
        pub trailing_u32: u32,
    }
}

wire_struct! {
    pub struct SkeletalMorphsMorphsEntry {
        pub crc: u32,
        pub morph_id: u32,
        pub bone_or_morph_index: u16,
        pub mirror_over: bool,
    }
}

wire_struct! {
    /// C++ `CSkeletalMorphs`.
    pub struct SkeletalMorphs {
        pub morphs: Vec<SkeletalMorphsMorphsEntry>,
    }
}

wire_struct! {
    pub struct ParticleMorphsMorphsEntry {
        pub morph_type: i32,
        pub data: [u8; 36],
    }
}

wire_struct! {
    /// C++ `CParticleMorphs`.
    pub struct ParticleMorphs {
        pub morphs: Vec<ParticleMorphsMorphsEntry>,
        pub trailing_u32: u32,
    }
}

wire_struct! {
    /// C++ `CExplosionRing`.
    pub struct ExplosionRing {
        pub num_explosions: i32,
        pub ring_radius: f32,
        pub angle_offset: f32,
        pub seconds_before_spawn_new_ring: f32,
    }
}

wire_struct! {
    /// C++ `CSoundBankEntry`.
    pub struct SoundBankEntry {
        pub id: u32,
        pub filename: DefString,
        pub symbol_filename: DefString,
        pub lip_sync_bank_handle: DefString,
        pub dvd: bool,
    }
}

wire_struct! {
    /// C++ `CAtmosBankEntry`.
    pub struct AtmosBankEntry {
        pub id: u32,
        pub filename: DefString,
    }
}

wire_struct! {
    /// C++ `CMusicEntry`.
    pub struct MusicEntry {
        pub id: u32,
        pub filename: DefString,
        pub gain: f32,
        pub cache_to_hd: bool,
    }
}

wire_struct! {
    /// C++ `CMusicSetEntry`.
    pub struct MusicSetEntry {
        pub normal_music_id: i32,
        pub danger_music_id: i32,
        pub loop_count: i32,
    }
}

wire_struct! {
    /// C++ `CReverbEnvironmentEntry`.
    pub struct ReverbEnvironmentEntry {
        pub param0: f32,
        pub param1: f32,
        pub param2: f32,
        pub param3: f32,
        pub param4: f32,
        pub param5: f32,
        pub param6: f32,
        pub param7: f32,
        pub param8: f32,
        pub param9: f32,
        pub param10: f32,
        pub param11: f32,
        pub param12: f32,
        pub param13: f32,
        pub param14: f32,
        pub param15: f32,
        pub param16: f32,
        pub param17: f32,
        pub param18: f32,
        pub param19: f32,
        pub param20: f32,
    }
}

wire_struct! {
    pub struct ActionInputControl {
        pub game_action: GameAction,
        pub controller_type: ControllerType,
        pub keyboard_key: InputKey,
        pub xbox_button: XboxControllerButton,
        pub mouse_button: MouseButtonControl,
        pub control_direction: [f32; 2],
    }
}

wire_struct! {
    pub struct MapPathEntry {
        pub id: u32,
        pub graphic: i32,
    }
}
