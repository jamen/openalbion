use crate::def_struct;

def_struct! {
    /// `CEnemyDef` — C++ `CEnemyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EnemyDef {
        "Faction" => pub faction: i32,
    }
}
