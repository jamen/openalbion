use crate::def_struct;

def_struct! {
    /// `CWaspQueenBattleDef` — C++ `CWaspQueenBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WaspQueenBattleDef {
        "NumFramesPauseBetweenAttacks" => pub num_frames_pause_between_attacks: i32,
    }
}
