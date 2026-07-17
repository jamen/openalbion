use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CONTROL_SCHEME` — `CControlsDef`.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ControlsDef {
        "Controls" => pub controls: Vec<ActionInputControl>,
        "ToggleZTarget" => pub toggle_z_target: bool,
        "ToggleSpells" => pub toggle_spells: bool,
        "ToggleSneak" => pub toggle_sneak: bool,
        "ToggleExpressionMenu" => pub toggle_expression_menu: bool,
        "ToggleExpressionShift" => pub toggle_expression_shift: bool,
        "FlourishNeedsAttackButtonHeld" => pub flourish_needs_attack_button_held: bool,
    }
}
