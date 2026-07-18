use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CShopItemDef` — C++ `CShopItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ShopItemDef {
        "Type" => pub type_: i32,
        "Quantity" => pub quantity: f32,
        "PriceMultiplier" => pub price_multiplier: f32 = 1.0,
        "MaxStock" => pub max_stock: f32,
        "RestockPeriod" => pub restock_period: i32,
        "RestockPeriodRandomness" => pub restock_period_randomness: f32,
        "SalesPerDay" => pub sales_per_day: f32,
        "SalesPerDayRandomness" => pub sales_per_day_randomness: f32,
    }
}
