use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernGameCoinBaseDef` — C++ `CTavernGameCoinBaseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernGameCoinBaseDef {
        "CoinIDensity" => pub coin_i_density: f32,
        "CoinRadius" => pub coin_radius: f32,
        "CoinEdgeRestitution" => pub coin_edge_restitution: f32,
        "CoinEdgeFriction" => pub coin_edge_friction: f32,
        "CoinSurfaceFriction" => pub coin_surface_friction: f32,
        "CoinDefIndex" => pub coin_def_index: DefIndex,
        "ArrowDefIndex" => pub arrow_def_index: DefIndex,
    }
}
