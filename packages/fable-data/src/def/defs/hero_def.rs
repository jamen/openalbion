use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroDef` — C++ `CHeroDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroDef {
        "DefaultTitle" => pub default_title: i32,
        "DefaultHeroTrainingStatus" => pub default_hero_training_status: HeroTrainingStatus,
    }
}
