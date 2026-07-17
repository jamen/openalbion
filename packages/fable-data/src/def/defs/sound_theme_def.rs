use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SOUND_THEME` — C++ `CSoundThemeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SoundThemeDef {
        "AtmosIndex" => pub atmos_index: i32,
        "MusicSet" => pub music_set: i32,
        "EditColour" => pub edit_colour: RGBColour,
        "Gain" => pub gain: f32,
        "Range" => pub range: f32,
        "Granularity" => pub granularity: f32,
        "AtmosGroupMap" => pub atmos_group_map: VecMap<String, i32>,
        "PlayOnlyWhenInside" => pub play_only_when_inside: bool,
        "PlayOnlyWhenOutside" => pub play_only_when_outside: bool,
        "PlayQuietlyWhenInside" => pub play_quietly_when_inside: bool,
        "PlayQuietlyWhenOutside" => pub play_quietly_when_outside: bool,
    }
}
