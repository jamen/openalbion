use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCoinGameObstacleDef` — C++ `CCoinGameObstacleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CoinGameObstacleDef {
        "Radius" => pub radius: f32,
    }
}
