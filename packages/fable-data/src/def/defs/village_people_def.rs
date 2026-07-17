use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CVillagePeopleDef` — C++ `CVillagePeopleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct VillagePeopleDef {
        "Enable" => pub enable: bool,
        "DesiredMaleWeight" => pub desired_male_weight: f32,
        "DesiredFemaleWeight" => pub desired_female_weight: f32,
        "DesiredChildWeight" => pub desired_child_weight: f32,
        "ProbabilityOfHomsexualityPerDialogueEnabledMan" => pub probability_of_homsexuality_per_dialogue_enabled_man: f32,
        "MaleTeacher" => pub male_teacher: i32,
        "MaleTeacher" => pub male_teacher2: i32,
        "MaleShopKeeper" => pub male_shop_keeper: i32,
        "FemaleShopKeeper" => pub female_shop_keeper: i32,
        "MaleMarketTrader" => pub male_market_trader: i32,
        "FemaleMarketTrader" => pub female_market_trader: i32,
        "MaleBarman" => pub male_barman: i32,
        "FemaleBarman" => pub female_barman: i32,
        "MaleBarmaid" => pub male_barmaid: i32,
        "FemaleBarmaid" => pub female_barmaid: i32,
        "MaleWarehouseWorker" => pub male_warehouse_worker: i32,
        "FemaleWarehouseWorker" => pub female_warehouse_worker: i32,
        "MaleServant" => pub male_servant: i32,
        "FemaleServant" => pub female_servant: i32,
        "MaleGoodApprentice" => pub male_good_apprentice: i32,
        "FemaleGoodApprentice" => pub female_good_apprentice: i32,
        "MaleEvilApprentice" => pub male_evil_apprentice: i32,
        "FemaleEvilApprentice" => pub female_evil_apprentice: i32,
        "MaleUnemployed" => pub male_unemployed: i32,
        "FemaleUnemployed" => pub female_unemployed: i32,
        "MaleChild" => pub male_child: i32,
        "FemaleChild" => pub female_child: i32,
        "GamesMaster" => pub games_master: i32,
        "Guard" => pub guard: i32,
        "MaleGhost" => pub male_ghost: i32,
        "FemaleGhost" => pub female_ghost: i32,
        "NumGuards" => pub num_guards: i32,
        "NumGhosts" => pub num_ghosts: i32,
        "GuardRespawnType" => pub guard_respawn_type: Vec<i32>,
        "GuardRespawnQuantity" => pub guard_respawn_quantity: Vec<i32>,
        "GuardRespawnMaxRenownBasedOffset" => pub guard_respawn_max_renown_based_offset: i32,
    }
}
