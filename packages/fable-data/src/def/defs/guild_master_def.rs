use crate::def_struct;

def_struct! {
    /// `CGuildMasterDef` — C++ `CGuildMasterDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GuildMasterDef {
        "MinSecondsBetweenTeleport" => pub min_seconds_between_teleport: f32,
        "MinSecondsBetweenWillUse" => pub min_seconds_between_will_use: f32,
    }
}
