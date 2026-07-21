use crate::def_struct;

def_struct! {
    /// `CDrunkennessDef` — C++ `CDrunkennessDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DrunkennessDef {
        "DrunkennessThresholdMult" => pub drunkenness_threshold_mult: f32,
    }
}
