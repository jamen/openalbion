use crate::DefStruct;

/// `CWaspQueenBattleDef` — C++ `CWaspQueenBattleDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct WaspQueenBattleDef {
    #[def("NumFramesPauseBetweenAttacks")]
    pub num_frames_pause_between_attacks: i32,
}
