use crate::DefStruct;

/// `CInterestingToVillagersDef` — C++ `CInterestingToVillagersDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct InterestingToVillagersDef {
    #[def("Interest")]
    pub interest: f32,
}
