use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroSpecialMovementDef` — C++ `CHeroSpecialMovementDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroSpecialMovementDef {
        "TotalTimeToSprintSeconds" => pub total_time_to_sprint_seconds: f32 = 7.0,
        "MinimumNumberOfSecondsLeftToBeAllowedToSprint" => pub minimum_number_of_seconds_left_to_be_allowed_to_sprint: f32 = 1.5,
        "TotalTimeToFullyRestoreSprintBarSeconds" => pub total_time_to_fully_restore_sprint_bar_seconds: f32 = 10.0,
        "NoSecondsToRunBeforeSprintStarts" => pub no_seconds_to_run_before_sprint_starts: f32 = 3.0,
    }
}
