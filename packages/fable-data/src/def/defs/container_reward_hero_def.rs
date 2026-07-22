use crate::DefStruct;
use crate::def::prelude::*;

/// `CContainerRewardHeroDef` — C++ `CContainerRewardHeroDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ContainerRewardHeroDef {
    #[def("ObjectFamilies")]
    pub object_families: Vec<DefIndex>,
}
