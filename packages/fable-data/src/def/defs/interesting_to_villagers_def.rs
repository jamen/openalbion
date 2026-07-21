use crate::def_struct;

def_struct! {
    /// `CInterestingToVillagersDef` — C++ `CInterestingToVillagersDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct InterestingToVillagersDef {
        "Interest" => pub interest: f32,
    }
}
