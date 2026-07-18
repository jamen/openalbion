use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CActionUseDef` — C++ `CActionUseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ActionUseDef {
        "AnimationGroup" => pub animation_group: DefString,
        "SnapToActionPoint" => pub snap_to_action_point: bool,
        "IgnoreActionPoints" => pub ignore_action_points: bool = true,
        "TutorialCategory" => pub tutorial_category: TutorialCategory,
    }
}
