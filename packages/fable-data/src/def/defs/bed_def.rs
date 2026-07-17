use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBedDef` — C++ `CBedDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BedDef {
        "GetInBedAnimName" => pub get_in_bed_anim_name: DefString,
        "SleepInBedAnimName" => pub sleep_in_bed_anim_name: DefString,
        "GetUpFromBedAnimName" => pub get_up_from_bed_anim_name: DefString,
        "GetUpFromBedFastAnimName" => pub get_up_from_bed_fast_anim_name: DefString,
        "Animating" => pub animating: bool,
    }
}
