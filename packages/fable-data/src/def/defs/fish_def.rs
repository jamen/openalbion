use crate::def_struct;

def_struct! {
    /// `CFishDef` — C++ `CFishDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FishDef {
        "MinLength" => pub min_length: f32,
        "MaxLength" => pub max_length: f32,
        "MinWeight" => pub min_weight: f32,
        "MaxWeight" => pub max_weight: f32,
        "MinSpeed" => pub min_speed: f32,
        "MaxSpeed" => pub max_speed: f32,
        "BasePullSeconds" => pub base_pull_seconds: f32,
        "MaxRandomPullSeconds" => pub max_random_pull_seconds: f32,
        "BasePullWaitSeconds" => pub base_pull_wait_seconds: f32,
        "MaxRandomPullWaitSeconds" => pub max_random_pull_wait_seconds: f32,
        "Level" => pub level: i32,
        "InfoText" => pub info_text: i32,
    }
}
