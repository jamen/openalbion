use crate::DefStruct;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct VillagePeopleDef {
    #[def("Enable")]
    pub enable: bool,
    #[def("DesiredMaleWeight")]
    pub desired_male_weight: f32,
    #[def("DesiredFemaleWeight")]
    pub desired_female_weight: f32,
    #[def("DesiredChildWeight")]
    pub desired_child_weight: f32,
    #[def("ProbabilityOfHomsexualityPerDialogueEnabledMan")]
    pub probability_of_homsexuality_per_dialogue_enabled_man: f32,
    #[def("MaleTeacher")]
    pub male_teacher: i32,
    #[def("MaleTeacher")]
    pub male_teacher2: i32,
    #[def("MaleShopKeeper")]
    pub male_shop_keeper: i32,
    #[def("FemaleShopKeeper")]
    pub female_shop_keeper: i32,
    #[def("MaleMarketTrader")]
    pub male_market_trader: i32,
    #[def("FemaleMarketTrader")]
    pub female_market_trader: i32,
    #[def("MaleBarman")]
    pub male_barman: i32,
    #[def("FemaleBarman")]
    pub female_barman: i32,
    #[def("MaleBarmaid")]
    pub male_barmaid: i32,
    #[def("FemaleBarmaid")]
    pub female_barmaid: i32,
    #[def("MaleWarehouseWorker")]
    pub male_warehouse_worker: i32,
    #[def("FemaleWarehouseWorker")]
    pub female_warehouse_worker: i32,
    #[def("MaleServant")]
    pub male_servant: i32,
    #[def("FemaleServant")]
    pub female_servant: i32,
    #[def("MaleGoodApprentice")]
    pub male_good_apprentice: i32,
    #[def("FemaleGoodApprentice")]
    pub female_good_apprentice: i32,
    #[def("MaleEvilApprentice")]
    pub male_evil_apprentice: i32,
    #[def("FemaleEvilApprentice")]
    pub female_evil_apprentice: i32,
    #[def("MaleUnemployed")]
    pub male_unemployed: i32,
    #[def("FemaleUnemployed")]
    pub female_unemployed: i32,
    #[def("MaleChild")]
    pub male_child: i32,
    #[def("FemaleChild")]
    pub female_child: i32,
    #[def("GamesMaster")]
    pub games_master: i32,
    #[def("Guard")]
    pub guard: i32,
    #[def("MaleGhost")]
    pub male_ghost: i32,
    #[def("FemaleGhost")]
    pub female_ghost: i32,
    #[def("NumGuards")]
    pub num_guards: i32,
    #[def("NumGhosts")]
    pub num_ghosts: i32,
    #[def("GuardRespawnType")]
    pub guard_respawn_type: Vec<i32>,
    #[def("GuardRespawnQuantity")]
    pub guard_respawn_quantity: Vec<i32>,
    #[def("GuardRespawnMaxRenownBasedOffset")]
    pub guard_respawn_max_renown_based_offset: i32,
}
