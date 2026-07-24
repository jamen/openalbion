use crate::DefStruct;
use crate::def::wire::DefIndex;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct EnemyDef {
    #[def("Faction")]
    pub faction: DefIndex,
}
