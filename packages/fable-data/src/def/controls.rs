use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason, parse_bool, parse_id, serialize_bool, serialize_id,
};
use crate::bytes::{TakeError, UnexpectedEnd, put, put_le, take, take_le};

#[derive(Debug)]
pub struct ControlsDef {
    pub controls: Vec<ActionInputControl>,
    pub toggle_z_target: bool,
    pub toggle_spells: bool,
    pub toggle_sneak: bool,
    pub toggle_expression_menu: bool,
    pub toggle_expression_shift: bool,
    pub flourish_needs_attack_button_held: bool,
}

impl ControlsDef {
    pub(crate) fn byte_size(&self) -> usize {
        self.controls_list_byte_size() + (ID_BYTE_SIZE + size_of::<bool>()) * 6
    }

    fn controls_list_byte_size(&self) -> usize {
        ID_BYTE_SIZE + size_of::<u32>() + self.controls.iter().map(|x| x.byte_size()).sum::<usize>()
    }

    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            controls: ActionInputControl::parse_list(cur)?,
            toggle_z_target: parse_bool(cur, "ToggleZTarget")?,
            toggle_spells: parse_bool(cur, "ToggleSpells")?,
            toggle_sneak: parse_bool(cur, "ToggleSneak")?,
            toggle_expression_menu: parse_bool(cur, "ToggleExpressionMenu")?,
            toggle_expression_shift: parse_bool(cur, "ToggleExpressionShift")?,
            flourish_needs_attack_button_held: parse_bool(cur, "FlourishNeedsAttackButtonHeld")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        ActionInputControl::serialize_list(&self.controls, out)?;

        serialize_bool(out, "ToggleZTarget", self.toggle_z_target)?;

        serialize_bool(out, "ToggleSpells", self.toggle_spells)?;

        serialize_bool(out, "ToggleSneak", self.toggle_sneak)?;

        serialize_bool(out, "ToggleExpressionMenu", self.toggle_expression_menu)?;

        serialize_bool(out, "ToggleExpressionShift", self.toggle_expression_shift)?;

        serialize_bool(
            out,
            "FlourishNeedsAttackButtonHeld",
            self.flourish_needs_attack_button_held,
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ActionInputControl {
    pub game_action: GameAction,
    pub controller_type: ControllerType,
    pub keyboard_key: InputKey,
    pub xbox_button: XboxControllerButton,
    pub mouse_button: MouseButtonControl,
    pub control_direction: [f32; 2],
}

impl ActionInputControl {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        Ok(Self {
            game_action: GameAction::parse(cur)?,
            controller_type: ControllerType::parse(cur)?,
            keyboard_key: InputKey::parse(cur)?,
            xbox_button: XboxControllerButton::parse(cur)?,
            mouse_button: MouseButtonControl::parse(cur)?,
            control_direction: take::<[f32; 2]>(cur)?,
        })
    }

    fn parse_list(cur: &mut &[u8]) -> Result<Vec<Self>, ParseControlError> {
        let name = "Controls";

        let _id = parse_id(cur, name)?;

        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;

        let controls = (0..count as usize)
            .map(|i| {
                Self::parse(cur).map_err(|inner| ParseControlError {
                    name,
                    reason: ParseControlErrorReason::ListItem(i, inner),
                })
            })
            .collect::<Result<Vec<Self>, _>>()?;

        Ok(controls)
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        self.game_action.serialize(out)?;
        self.controller_type.serialize(out)?;
        self.keyboard_key.serialize(out)?;
        self.xbox_button.serialize(out)?;
        self.mouse_button.serialize(out)?;
        put(out, &self.control_direction)?;
        Ok(())
    }

    fn serialize_list(list: &[Self], out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        let name = "Controls";
        serialize_id(out, name)?;

        put(out, &(list.len() as u32).to_le()).map_err(|inner| SerializeControlError {
            name,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;

        for (i, control) in list.iter().enumerate() {
            control
                .serialize(out)
                .map_err(|inner| SerializeControlError {
                    name,
                    reason: SerializeControlErrorReason::ListItem(i, inner),
                })?;
        }

        Ok(())
    }

    pub const fn byte_size(&self) -> usize {
        size_of::<GameAction>()
            + size_of::<ControllerType>()
            + size_of::<InputKey>()
            + size_of::<XboxControllerButton>()
            + size_of::<MouseButtonControl>()
            + size_of::<[f32; 2]>()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct GameAction {
    pub inner: i32,
}

impl GameAction {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        take_le::<i32>(cur).map(|inner| Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_le(out, &self.inner.to_le())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ControllerType {
    pub inner: i32,
}

impl ControllerType {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        take_le::<i32>(cur).map(|inner| Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_le(out, &self.inner.to_le())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct InputKey {
    pub inner: i32,
}

impl InputKey {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        take_le::<i32>(cur).map(|inner| Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_le(out, &self.inner.to_le())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct XboxControllerButton {
    pub inner: i32,
}

impl XboxControllerButton {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        take_le::<i32>(cur).map(|inner| Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_le(out, &self.inner.to_le())
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct MouseButtonControl {
    pub inner: i32,
}

impl MouseButtonControl {
    fn parse(cur: &mut &[u8]) -> Result<Self, TakeError> {
        take_le::<i32>(cur).map(|inner| Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_le(out, &self.inner.to_le())
    }
}
