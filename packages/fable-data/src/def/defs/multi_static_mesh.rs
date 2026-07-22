use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct MultiStaticMeshDef {
    #[def("Meshes")]
    pub meshes: Vec<MultiStaticMeshEntryDef>,
}
