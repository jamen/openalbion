use crate::def_struct;

def_struct! {
    /// `CFireheartMinigameDef` — C++ `CFireheartMinigameDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FireheartMinigameDef {
        "MoonFX" => pub moon_fx: String,
        "SunFX" => pub sun_fx: String,
        "QuitText" => pub quit_text: u32,
        "YesText" => pub yes_text: u32,
        "NoText" => pub no_text: u32,
    }
}
