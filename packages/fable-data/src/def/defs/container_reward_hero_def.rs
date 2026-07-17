use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CContainerRewardHeroDef` — C++ `CContainerRewardHeroDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ContainerRewardHeroDef {
        "ObjectFamilies" => pub object_families: Vec<DefIndex>,
    }
}
