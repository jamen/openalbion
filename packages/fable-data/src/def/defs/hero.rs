use crate::DefStruct;
use crate::def::enums::HeroTrainingStatus;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroDef {
    #[def("DefaultTitle")]
    pub default_title: i32,
    #[def("DefaultHeroTrainingStatus")]
    pub default_hero_training_status: HeroTrainingStatus,
}
