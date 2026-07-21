use crate::def_struct;

def_struct! {
    /// C++ `CAppearanceModifierScalingDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct AppearanceModifierScalingDef {
        "Attractiveness" => pub attractiveness: f32,
        "Scariness" => pub scariness: f32,
        "Goodstrength" => pub goodstrength: f32,
    }
}
