use crate::def_struct;

def_struct! {
    /// `CStockItemDef` — C++ `CStockItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct StockItemDef {
        "DefaultPrice" => pub default_price: i32,
        "DefaultIsStealable" => pub default_is_stealable: bool = true,
        "SellersBanter" => pub sellers_banter: u32,
        "BuyersPhrase" => pub buyers_phrase: u32,
        "CanOnlyBeSoldToPlayer" => pub can_only_be_sold_to_player: bool,
        "ChanceOfBecomingAvailable" => pub chance_of_becoming_available: f32 = 0.15,
        "CanBeDisplayedInShop" => pub can_be_displayed_in_shop: bool = true,
    }
}
