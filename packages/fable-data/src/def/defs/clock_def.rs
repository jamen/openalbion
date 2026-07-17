use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CClockDef` — C++ `CClockDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ClockDef {
        "Sound" => pub sound: VecMap<String, i32>,
        "HandType" => pub hand_type: ClockHandType,
    }
}
