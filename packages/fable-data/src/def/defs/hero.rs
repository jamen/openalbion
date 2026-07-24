use crate::DefStruct;
use crate::def::wire::DefIndex;
use crate::def::enums::HeroTrainingStatus;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroDef {
    #[def("DefaultTitle")]
    pub default_title: DefIndex,
    #[def("DefaultHeroTrainingStatus")]
    pub default_hero_training_status: HeroTrainingStatus,
}
