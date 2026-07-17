use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SOUND_SETUP` — C++ `CSoundDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SoundDef {
        "SoundBankEntries" => pub sound_bank_entries: Vec<SoundBankEntry>,
        "AtmosBankEntries" => pub atmos_bank_entries: Vec<AtmosBankEntry>,
        "MusicEntries" => pub music_entries: Vec<MusicEntry>,
        "MusicSetEntries" => pub music_set_entries: Vec<MusicSetEntry>,
        "ReverbEnvironmentEntries" => pub reverb_environment_entries: Vec<ReverbEnvironmentEntry>,
        "AtmosListenerDampingMin" => pub atmos_listener_damping_min: f32,
        "AtmosListenerDampingRange" => pub atmos_listener_damping_range: f32,
        "DistanceModelCurve" => pub distance_model_curve: f32,
    }
}
