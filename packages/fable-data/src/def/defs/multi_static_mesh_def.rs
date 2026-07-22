use crate::DefStruct;
use crate::def::prelude::*;

/// `CMultiStaticMeshDef` — C++ `CMultiStaticMeshDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct MultiStaticMeshDef {
    #[def("Meshes")]
    pub meshes: Vec<MultiStaticMeshEntryDef>,
}
