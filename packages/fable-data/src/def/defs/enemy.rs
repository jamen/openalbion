use crate::DefStruct;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct EnemyDef {
    #[def("Faction")]
    pub faction: i32,
}
