use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBuyableHouseDef` — C++ `CBuyableHouseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BuyableHouseDef {
        "IsBuyable" => pub is_buyable: bool,
        "InitiallyEmpty" => pub initially_empty: bool,
        "Price" => pub price: Vec<i32>,
        "Rent" => pub rent: Vec<i32>,
        "DaysBetweenRentPayments" => pub days_between_rent_payments: i32,
        "IsScripted" => pub is_scripted: bool,
        "MaxRentBags" => pub max_rent_bags: i32,
        "IsBuyableWithoutSignpost" => pub is_buyable_without_signpost: bool,
    }
}
