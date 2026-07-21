use crate::def_struct;

def_struct! {
    /// `CBoastingPodiumDef` — C++ `CBoastingPodiumDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BoastingPodiumDef {
        "HeroOnPodiumRadius" => pub hero_on_podium_radius: f32 = 1.0,
    }
}
