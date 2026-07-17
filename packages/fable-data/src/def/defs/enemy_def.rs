use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CEnemyDef` — C++ `CEnemyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EnemyDef {
        "Faction" => pub faction: i32,
    }
}
