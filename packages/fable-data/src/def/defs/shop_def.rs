use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CShopDef` — C++ `CShopDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ShopDef {
        "Name" => pub name: DefIndex,
        "InitialHawkingPhrase" => pub initial_hawking_phrase: DefIndex,
        "ReplyToRequestPhrase" => pub reply_to_request_phrase: DefIndex,
        "CustomerFollowPhrase" => pub customer_follow_phrase: DefIndex,
        "ThankingPhrase" => pub thanking_phrase: DefIndex,
        "ShopkeeperMeshType" => pub shopkeeper_mesh_type: String,
        "DefaultStock" => pub default_stock: Vec<ShopItemDef>,
        "StockCategoryDefaults" => pub stock_category_defaults: Vec<ShopItemDef>,
        "BuyPriceMultiplier" => pub buy_price_multiplier: f32,
        "SellPriceMultiplier" => pub sell_price_multiplier: f32,
        "AttitudeBuyPriceMultiplierDefault" => pub attitude_buy_price_multiplier_default: f32,
        "AttitudeBuyPriceMultiplier" => pub attitude_buy_price_multiplier: VecMap<Opinion, f32>,
        "AttitudeSellPriceMultiplierDefault" => pub attitude_sell_price_multiplier_default: f32,
        "AttitudeSellPriceMultiplier" => pub attitude_sell_price_multiplier: VecMap<Opinion, f32>,
        "MaxStockChangePerSell" => pub max_stock_change_per_sell: f32,
        "MaxStockChangePerBuy" => pub max_stock_change_per_buy: f32,
        "MaxStockRevertPerDay" => pub max_stock_revert_per_day: f32,
        "PriceReductionFractionPerItem" => pub price_reduction_fraction_per_item: f32,
        "MaxPriceReductionFractionDueToStockCount" => pub max_price_reduction_fraction_due_to_stock_count: f32,
        "IsStockDisplayPermanent" => pub is_stock_display_permanent: bool,
        "PreferredShopkeeper" => pub preferred_shopkeeper: DefIndex,
        "IsTattooShop" => pub is_tattoo_shop: bool,
        "IsBarberShop" => pub is_barber_shop: bool,
        "IsTitleShop" => pub is_title_shop: bool,
        "MinPriceMultiplierForItemToBeWanted" => pub min_price_multiplier_for_item_to_be_wanted: f32,
        "AlwaysOpen" => pub always_open: bool,
        "MaxPriceToDisplay" => pub max_price_to_display: f32,
        "MinTimeToSteal" => pub min_time_to_steal: f32,
        "MaxTimeToSteal" => pub max_time_to_steal: f32,
        "MinGoldForDonation" => pub min_gold_for_donation: DefIndex,
        "MaxGoldForDonation" => pub max_gold_for_donation: DefIndex,
        "MaxMoralityForDonation" => pub max_morality_for_donation: f32,
        "LogarithmicPower" => pub logarithmic_power: f32,
        "NumDaysToKeepSpecialItem" => pub num_days_to_keep_special_item: DefIndex,
        "MinAmountBeforeSpecialItem" => pub min_amount_before_special_item: DefIndex,
        "ProbabilityOfSpecialItemPerDay" => pub probability_of_special_item_per_day: f32,
        "SpecialItemPriceMult" => pub special_item_price_mult: f32,
    }
}
