use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ContainerRewardHeroDef {
    #[def("ObjectFamilies")]
    pub object_families: Vec<DefIndex>,
}
