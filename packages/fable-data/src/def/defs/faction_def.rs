use crate::def_struct;

def_struct! {
    /// `FACTION` — C++ `CFactionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FactionDef {
        "EnemyFactions" => pub enemy_factions: Vec<String>,
        "AlliedFactions" => pub allied_factions: Vec<String>,
    }
}
