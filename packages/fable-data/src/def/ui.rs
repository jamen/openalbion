use std::collections::BTreeMap;

use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason, list_control_byte_size, parse_bool, parse_id, parse_list,
    parse_scalar, parse_wstr, serialize_bool, serialize_id, serialize_list, serialize_scalar,
    serialize_wstr, wstr_control_byte_size,
};
use crate::bytes::{
    put_le, put_null_terminated_utf8, take_le, take_null_terminated_utf8,
};

#[derive(Debug)]
pub struct UiDef {
    pub ui_type: UiType,
    pub children: Vec<u32>,
    pub mesh_index: u32,
    pub text_value: String,
    pub font: i32,
    pub height: f32,
    pub width: f32,
    pub expansion_type: TableExpansionTypes,
    pub sprites: Sprites,
    pub horizontal_separations: Vec<u32>,
    pub vertical_separations: Vec<u32>,
    pub states: Vec<UiStateDef>,
    pub text_line_break: bool,
    pub scale_text: bool,
    pub inpdenedent: bool,
    pub mesh_type: EngineGraphicType,
    pub non_scrolling_children: Vec<u32>,
    pub text_window_tlx: f32,
    pub text_window_tly: f32,
    pub text_window_brx: f32,
    pub text_window_bry: f32,
    pub layer: i32,
    pub angle: f32,
    pub position_is_center: bool,
    pub scrolling_speed: f32,
    pub wrapping: bool,
    pub inverted: bool,
    pub position_offset_x: f32,
    pub position_offset_y: f32,
    pub alpha_offset: u32,
    pub up_x: f32,
    pub up_y: f32,
    pub up_z: f32,
    pub forward_x: f32,
    pub forward_y: f32,
    pub forward_z: f32,
    pub rotation_axis_x: f32,
    pub rotation_axis_y: f32,
    pub rotation_axis_z: f32,
    pub rotation_speed: f32,
    pub animation_index: u32,
    pub down_arrow: i32,
    pub up_arrow: i32,
    pub up_limit: i32,
    pub down_limit: i32,
    pub scrolling: bool,
    pub compute_offsets_on_activate: bool,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub step_x: f32,
    pub step_y: f32,
    pub dimensions_x: f32,
    pub dimensions_y: f32,
    pub slider_left: i32,
    pub slider_right: i32,
    pub action: ActionType,
    pub action_on_back: ActionType,
    pub action_on_selected: ActionType,
    pub action_on_unselected: ActionType,
    pub action_on_destruction: ActionType,
    pub action_on_left_clicked: ActionType,
    pub action_on_left_unclicked: ActionType,
    pub action_on_left_held: ActionType,
    pub action_on_right_clicked: ActionType,
    pub action_on_dropped: ActionType,
    pub action_on_dropped_nowhere: ActionType,
    pub pre_action: ActionType,
    pub action_on_dragged_up: ActionType,
    pub action_on_dragged_down: ActionType,
    pub action_on_left_clicked_above: ActionType,
    pub action_on_left_clicked_under: ActionType,
    pub input_delay: f32,
    pub draw_from_viewport: bool,
    pub text_bank_index: u32,
    pub action_text: i32,
    pub key_text: i32,
    pub redefiner: i32,
    pub undefined_warning: i32,
    pub action_map: ActionMap,
    pub action_map_aliases: ActionMapAliases,
    pub action_order: Vec<u32>,
    pub edit_box_parent_is_button: bool,
    pub password_box: bool,
    pub edit_box_char_limit: i32,
    pub edit_box_uses_ime: bool,
    pub movie_filename: String,
    pub disallow_space_as_first_char: bool,
    pub layer_independent: bool,
    pub swapping_states: Vec<u32>,
    pub swapping_times: Vec<f32>,
    pub bastard_child: bool,
    pub alignment: TextAlignment,
    pub random_swap: bool,
    pub use_relative_zoom: bool,
    pub use_relative_position: bool,
    pub hovered_state: i32,
    pub left_clicked_state: i32,
    pub right_clicked_state: i32,
    pub shape_children: Vec<u32>,
    pub view_area_tlx: i32,
    pub view_area_tly: i32,
    pub view_area_brx: i32,
    pub view_area_bry: i32,
    pub use_view_area: bool,
    pub part_of_list_tree: bool,
    pub pc_style: bool,
    pub sprite_2d_flag: EngineSprite2dFlag,
}

impl UiDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            ui_type: UiType::parse(cur)?,
            children: parse_list::<u32>(cur, "Children")?,
            mesh_index: parse_scalar::<u32>(cur, "MeshIndex")?,
            text_value: parse_wstr(cur, "TextValue")?,
            font: parse_scalar::<i32>(cur, "Font")?,
            height: parse_scalar::<f32>(cur, "Height")?,
            width: parse_scalar::<f32>(cur, "Width")?,
            expansion_type: TableExpansionTypes::parse(cur)?,
            sprites: Sprites::parse(cur)?,
            horizontal_separations: parse_list::<u32>(cur, "HorizontalSeparations")?,
            vertical_separations: parse_list::<u32>(cur, "VerticalSeparations")?,
            states: UiStateDef::parse_list(cur)?,
            text_line_break: parse_bool(cur, "TextLineBreak")?,
            scale_text: parse_bool(cur, "ScaleText")?,
            inpdenedent: parse_bool(cur, "Independant")?,
            mesh_type: EngineGraphicType::parse(cur)?,
            non_scrolling_children: parse_list::<u32>(cur, "NonScrollingChildren")?,
            text_window_tlx: parse_scalar::<f32>(cur, "TextWindowTLX")?,
            text_window_tly: parse_scalar::<f32>(cur, "TextWindowTLY")?,
            text_window_brx: parse_scalar::<f32>(cur, "TextWindowBRX")?,
            text_window_bry: parse_scalar::<f32>(cur, "TextWindowBRY")?,
            layer: parse_scalar::<i32>(cur, "Layer")?,
            angle: parse_scalar::<f32>(cur, "Angle")?,
            position_is_center: parse_bool(cur, "PositionIsCenter")?,
            scrolling_speed: parse_scalar::<f32>(cur, "ScrollingSpeed")?,
            wrapping: parse_bool(cur, "Wrapping")?,
            inverted: parse_bool(cur, "Inverted")?,
            position_offset_x: parse_scalar::<f32>(cur, "PositionOffsetX")?,
            position_offset_y: parse_scalar::<f32>(cur, "PositionOffsetY")?,
            alpha_offset: parse_scalar::<u32>(cur, "AlphaOffset")?,
            up_x: parse_scalar::<f32>(cur, "UpX")?,
            up_y: parse_scalar::<f32>(cur, "UpY")?,
            up_z: parse_scalar::<f32>(cur, "UpZ")?,
            forward_x: parse_scalar::<f32>(cur, "ForwardX")?,
            forward_y: parse_scalar::<f32>(cur, "ForwardY")?,
            forward_z: parse_scalar::<f32>(cur, "ForwardZ")?,
            rotation_axis_x: parse_scalar::<f32>(cur, "RotationAxisX")?,
            rotation_axis_y: parse_scalar::<f32>(cur, "RotationAxisY")?,
            rotation_axis_z: parse_scalar::<f32>(cur, "RotationAxisZ")?,
            rotation_speed: parse_scalar::<f32>(cur, "RotationSpeed")?,
            animation_index: parse_scalar::<u32>(cur, "AnimationIndex")?,
            down_arrow: parse_scalar::<i32>(cur, "DownArrow")?,
            up_arrow: parse_scalar::<i32>(cur, "UpArrow")?,
            up_limit: parse_scalar::<i32>(cur, "UpLimit")?,
            down_limit: parse_scalar::<i32>(cur, "DownLimit")?,
            scrolling: parse_bool(cur, "Scrolling")?,
            compute_offsets_on_activate: parse_bool(cur, "ComputeOffsetsOnActivate")?,
            min_x: parse_scalar::<f32>(cur, "MinX")?,
            min_y: parse_scalar::<f32>(cur, "MinY")?,
            max_x: parse_scalar::<f32>(cur, "MaxX")?,
            max_y: parse_scalar::<f32>(cur, "MaxY")?,
            step_x: parse_scalar::<f32>(cur, "StepX")?,
            step_y: parse_scalar::<f32>(cur, "StepY")?,
            dimensions_x: parse_scalar::<f32>(cur, "DimensionsX")?,
            dimensions_y: parse_scalar::<f32>(cur, "DimensionsY")?,
            slider_left: parse_scalar::<i32>(cur, "SliderLeft")?,
            slider_right: parse_scalar::<i32>(cur, "SliderRight")?,
            action: ActionType::parse(cur, "Action")?,
            action_on_back: ActionType::parse(cur, "ActionOnBack")?,
            action_on_selected: ActionType::parse(cur, "ActionOnSelected")?,
            action_on_unselected: ActionType::parse(cur, "ActionOnUnselected")?,
            action_on_destruction: ActionType::parse(cur, "ActionOnDestruction")?,
            action_on_left_clicked: ActionType::parse(cur, "ActionOnLeftClicked")?,
            action_on_left_unclicked: ActionType::parse(cur, "ActionOnLeftUnclicked")?,
            action_on_left_held: ActionType::parse(cur, "ActionOnLeftHeld")?,
            action_on_right_clicked: ActionType::parse(cur, "ActionOnRightClicked")?,
            action_on_dropped: ActionType::parse(cur, "ActionOnDropped")?,
            action_on_dropped_nowhere: ActionType::parse(cur, "ActionOnDroppedNowhere")?,
            pre_action: ActionType::parse(cur, "PreAction")?,
            action_on_dragged_up: ActionType::parse(cur, "ActionOnDraggedUp")?,
            action_on_dragged_down: ActionType::parse(cur, "ActionOnDraggedDown")?,
            action_on_left_clicked_above: ActionType::parse(cur, "ActionOnLeftClickedAbove")?,
            action_on_left_clicked_under: ActionType::parse(cur, "ActionOnLeftClickedUnder")?,
            input_delay: parse_scalar::<f32>(cur, "InputDelay")?,
            draw_from_viewport: parse_bool(cur, "DrawFromViewport")?,
            text_bank_index: parse_scalar::<u32>(cur, "TextBankIndex")?,
            action_text: parse_scalar::<i32>(cur, "ActionText")?,
            key_text: parse_scalar::<i32>(cur, "KeyText")?,
            redefiner: parse_scalar::<i32>(cur, "Redefiner")?,
            undefined_warning: parse_scalar::<i32>(cur, "UndefinedWarning")?,
            action_map: ActionMap::parse(cur)?,
            action_map_aliases: ActionMapAliases::parse(cur)?,
            action_order: parse_list::<u32>(cur, "ActionOrder")?,
            edit_box_parent_is_button: parse_bool(cur, "EditBoxParentIsButton")?,
            password_box: parse_bool(cur, "PasswordBox")?,
            edit_box_char_limit: parse_scalar::<i32>(cur, "EditBoxCharLimit")?,
            edit_box_uses_ime: parse_bool(cur, "EditBoxUsesIME")?,
            movie_filename: parse_wstr(cur, "MovieFilename")?,
            disallow_space_as_first_char: parse_bool(cur, "DisallowSpaceAsFirstChar")?,
            layer_independent: parse_bool(cur, "LayerIndependant")?,
            swapping_states: parse_list::<u32>(cur, "SwappingStates")?,
            swapping_times: parse_list::<f32>(cur, "SwappingTimes")?,
            bastard_child: parse_bool(cur, "BastardChild")?,
            alignment: TextAlignment::parse(cur)?,
            random_swap: parse_bool(cur, "RandomSwap")?,
            use_relative_zoom: parse_bool(cur, "UseRelativeZoom")?,
            use_relative_position: parse_bool(cur, "UseRelativePosition")?,
            hovered_state: parse_scalar::<i32>(cur, "HoveredState")?,
            left_clicked_state: parse_scalar::<i32>(cur, "LeftClickedState")?,
            right_clicked_state: parse_scalar::<i32>(cur, "RightClickedState")?,
            shape_children: parse_list::<u32>(cur, "ShapeChildren")?,
            view_area_tlx: parse_scalar::<i32>(cur, "ViewAreaTLX")?,
            view_area_tly: parse_scalar::<i32>(cur, "ViewAreaTLY")?,
            view_area_brx: parse_scalar::<i32>(cur, "ViewAreaBRX")?,
            view_area_bry: parse_scalar::<i32>(cur, "ViewAreaBRY")?,
            use_view_area: parse_bool(cur, "UseViewArea")?,
            part_of_list_tree: parse_bool(cur, "PartOfListTree")?,
            pc_style: parse_bool(cur, "PCStyle")?,
            sprite_2d_flag: EngineSprite2dFlag::parse(cur)?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        self.ui_type.serialize(out)?;
        serialize_list::<u32>(out, "Children", &self.children)?;
        serialize_scalar::<u32>(out, "MeshIndex", self.mesh_index)?;
        serialize_wstr(out, "TextValue", &self.text_value)?;
        serialize_scalar::<i32>(out, "Font", self.font)?;
        serialize_scalar::<f32>(out, "Height", self.height)?;
        serialize_scalar::<f32>(out, "Width", self.width)?;
        self.expansion_type.serialize(out)?;
        self.sprites.serialize(out)?;
        serialize_list::<u32>(out, "HorizontalSeparations", &self.horizontal_separations)?;
        serialize_list::<u32>(out, "VerticalSeparations", &self.vertical_separations)?;
        UiStateDef::serialize_list(&self.states, out)?;
        serialize_bool(out, "TextLineBreak", self.text_line_break)?;
        serialize_bool(out, "ScaleText", self.scale_text)?;
        serialize_bool(out, "Independant", self.inpdenedent)?;
        self.mesh_type.serialize(out)?;
        serialize_list::<u32>(out, "NonScrollingChildren", &self.non_scrolling_children)?;
        serialize_scalar::<f32>(out, "TextWindowTLX", self.text_window_tlx)?;
        serialize_scalar::<f32>(out, "TextWindowTLY", self.text_window_tly)?;
        serialize_scalar::<f32>(out, "TextWindowBRX", self.text_window_brx)?;
        serialize_scalar::<f32>(out, "TextWindowBRY", self.text_window_bry)?;
        serialize_scalar::<i32>(out, "Layer", self.layer)?;
        serialize_scalar::<f32>(out, "Angle", self.angle)?;
        serialize_bool(out, "PositionIsCenter", self.position_is_center)?;
        serialize_scalar::<f32>(out, "ScrollingSpeed", self.scrolling_speed)?;
        serialize_bool(out, "Wrapping", self.wrapping)?;
        serialize_bool(out, "Inverted", self.inverted)?;
        serialize_scalar::<f32>(out, "PositionOffsetX", self.position_offset_x)?;
        serialize_scalar::<f32>(out, "PositionOffsetY", self.position_offset_y)?;
        serialize_scalar::<u32>(out, "AlphaOffset", self.alpha_offset)?;
        serialize_scalar::<f32>(out, "UpX", self.up_x)?;
        serialize_scalar::<f32>(out, "UpY", self.up_y)?;
        serialize_scalar::<f32>(out, "UpZ", self.up_z)?;
        serialize_scalar::<f32>(out, "ForwardX", self.forward_x)?;
        serialize_scalar::<f32>(out, "ForwardY", self.forward_y)?;
        serialize_scalar::<f32>(out, "ForwardZ", self.forward_z)?;
        serialize_scalar::<f32>(out, "RotationAxisX", self.rotation_axis_x)?;
        serialize_scalar::<f32>(out, "RotationAxisY", self.rotation_axis_y)?;
        serialize_scalar::<f32>(out, "RotationAxisZ", self.rotation_axis_z)?;
        serialize_scalar::<f32>(out, "RotationSpeed", self.rotation_speed)?;
        serialize_scalar::<u32>(out, "AnimationIndex", self.animation_index)?;
        serialize_scalar::<i32>(out, "DownArrow", self.down_arrow)?;
        serialize_scalar::<i32>(out, "UpArrow", self.up_arrow)?;
        serialize_scalar::<i32>(out, "UpLimit", self.up_limit)?;
        serialize_scalar::<i32>(out, "DownLimit", self.down_limit)?;
        serialize_bool(out, "Scrolling", self.scrolling)?;
        serialize_bool(out, "ComputeOffsetsOnActivate", self.compute_offsets_on_activate)?;
        serialize_scalar::<f32>(out, "MinX", self.min_x)?;
        serialize_scalar::<f32>(out, "MinY", self.min_y)?;
        serialize_scalar::<f32>(out, "MaxX", self.max_x)?;
        serialize_scalar::<f32>(out, "MaxY", self.max_y)?;
        serialize_scalar::<f32>(out, "StepX", self.step_x)?;
        serialize_scalar::<f32>(out, "StepY", self.step_y)?;
        serialize_scalar::<f32>(out, "DimensionsX", self.dimensions_x)?;
        serialize_scalar::<f32>(out, "DimensionsY", self.dimensions_y)?;
        serialize_scalar::<i32>(out, "SliderLeft", self.slider_left)?;
        serialize_scalar::<i32>(out, "SliderRight", self.slider_right)?;
        self.action.serialize(out, "Action")?;
        self.action_on_back.serialize(out, "ActionOnBack")?;
        self.action_on_selected.serialize(out, "ActionOnSelected")?;
        self.action_on_unselected.serialize(out, "ActionOnUnselected")?;
        self.action_on_destruction.serialize(out, "ActionOnDestruction")?;
        self.action_on_left_clicked.serialize(out, "ActionOnLeftClicked")?;
        self.action_on_left_unclicked.serialize(out, "ActionOnLeftUnclicked")?;
        self.action_on_left_held.serialize(out, "ActionOnLeftHeld")?;
        self.action_on_right_clicked.serialize(out, "ActionOnRightClicked")?;
        self.action_on_dropped.serialize(out, "ActionOnDropped")?;
        self.action_on_dropped_nowhere.serialize(out, "ActionOnDroppedNowhere")?;
        self.pre_action.serialize(out, "PreAction")?;
        self.action_on_dragged_up.serialize(out, "ActionOnDraggedUp")?;
        self.action_on_dragged_down.serialize(out, "ActionOnDraggedDown")?;
        self.action_on_left_clicked_above.serialize(out, "ActionOnLeftClickedAbove")?;
        self.action_on_left_clicked_under.serialize(out, "ActionOnLeftClickedUnder")?;
        serialize_scalar::<f32>(out, "InputDelay", self.input_delay)?;
        serialize_bool(out, "DrawFromViewport", self.draw_from_viewport)?;
        serialize_scalar::<u32>(out, "TextBankIndex", self.text_bank_index)?;
        serialize_scalar::<i32>(out, "ActionText", self.action_text)?;
        serialize_scalar::<i32>(out, "KeyText", self.key_text)?;
        serialize_scalar::<i32>(out, "Redefiner", self.redefiner)?;
        serialize_scalar::<i32>(out, "UndefinedWarning", self.undefined_warning)?;
        self.action_map.serialize(out)?;
        self.action_map_aliases.serialize(out)?;
        serialize_list::<u32>(out, "ActionOrder", &self.action_order)?;
        serialize_bool(out, "EditBoxParentIsButton", self.edit_box_parent_is_button)?;
        serialize_bool(out, "PasswordBox", self.password_box)?;
        serialize_scalar::<i32>(out, "EditBoxCharLimit", self.edit_box_char_limit)?;
        serialize_bool(out, "EditBoxUsesIME", self.edit_box_uses_ime)?;
        serialize_wstr(out, "MovieFilename", &self.movie_filename)?;
        serialize_bool(out, "DisallowSpaceAsFirstChar", self.disallow_space_as_first_char)?;
        serialize_bool(out, "LayerIndependant", self.layer_independent)?;
        serialize_list::<u32>(out, "SwappingStates", &self.swapping_states)?;
        serialize_list::<f32>(out, "SwappingTimes", &self.swapping_times)?;
        serialize_bool(out, "BastardChild", self.bastard_child)?;
        self.alignment.serialize(out)?;
        serialize_bool(out, "RandomSwap", self.random_swap)?;
        serialize_bool(out, "UseRelativeZoom", self.use_relative_zoom)?;
        serialize_bool(out, "UseRelativePosition", self.use_relative_position)?;
        serialize_scalar::<i32>(out, "HoveredState", self.hovered_state)?;
        serialize_scalar::<i32>(out, "LeftClickedState", self.left_clicked_state)?;
        serialize_scalar::<i32>(out, "RightClickedState", self.right_clicked_state)?;
        serialize_list::<u32>(out, "ShapeChildren", &self.shape_children)?;
        serialize_scalar::<i32>(out, "ViewAreaTLX", self.view_area_tlx)?;
        serialize_scalar::<i32>(out, "ViewAreaTLY", self.view_area_tly)?;
        serialize_scalar::<i32>(out, "ViewAreaBRX", self.view_area_brx)?;
        serialize_scalar::<i32>(out, "ViewAreaBRY", self.view_area_bry)?;
        serialize_bool(out, "UseViewArea", self.use_view_area)?;
        serialize_bool(out, "PartOfListTree", self.part_of_list_tree)?;
        serialize_bool(out, "PCStyle", self.pc_style)?;
        self.sprite_2d_flag.serialize(out)?;
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        UiType::BYTE_SIZE
            + list_control_byte_size(&self.children)
            + (ID_BYTE_SIZE + size_of::<u32>())
            + wstr_control_byte_size(&self.text_value)
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + TableExpansionTypes::BYTE_SIZE
            + self.sprites.byte_size()
            + list_control_byte_size(&self.horizontal_separations)
            + list_control_byte_size(&self.vertical_separations)
            + UiStateDef::list_byte_size(&self.states)
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + EngineGraphicType::BYTE_SIZE
            + list_control_byte_size(&self.non_scrolling_children)
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + ActionType::BYTE_SIZE
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + self.action_map.byte_size()
            + self.action_map_aliases.byte_size()
            + list_control_byte_size(&self.action_order)
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + wstr_control_byte_size(&self.movie_filename)
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + list_control_byte_size(&self.swapping_states)
            + list_control_byte_size(&self.swapping_times)
            + (ID_BYTE_SIZE + size_of::<bool>())
            + TextAlignment::BYTE_SIZE
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + list_control_byte_size(&self.shape_children)
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<bool>())
            + EngineSprite2dFlag::BYTE_SIZE
    }
}

// ── Fixed-name i32 newtypes ───────────────────────────────────────────────────

macro_rules! i32_control {
    ($name:ident, $control:literal) => {
        #[derive(Debug)]
        pub struct $name {
            pub inner: i32,
        }

        impl $name {
            fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
                Ok(Self {
                    inner: parse_scalar::<i32>(cur, $control)?,
                })
            }

            fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
                serialize_scalar::<i32>(out, $control, self.inner)
            }

            pub const BYTE_SIZE: usize = ID_BYTE_SIZE + size_of::<i32>();
        }
    };
}

i32_control!(UiType, "Type");
i32_control!(TableExpansionTypes, "ExpansionType");
i32_control!(EngineGraphicType, "MeshType");
i32_control!(TextAlignment, "Alignement");
i32_control!(EngineSprite2dFlag, "Sprite2DFlag");
i32_control!(StateChangeType, "StateChangeType");

/// Like the fixed-name newtypes, but the control name varies per field (the same
/// `ActionType` is written under many different names in [`UiDef`]).
#[derive(Debug)]
pub struct ActionType {
    pub inner: i32,
}

impl ActionType {
    fn parse(cur: &mut &[u8], name: &'static str) -> Result<Self, ParseControlError> {
        Ok(Self {
            inner: parse_scalar::<i32>(cur, name)?,
        })
    }

    fn serialize(&self, out: &mut &mut [u8], name: &'static str) -> Result<(), SerializeControlError> {
        serialize_scalar::<i32>(out, name, self.inner)
    }

    pub const BYTE_SIZE: usize = ID_BYTE_SIZE + size_of::<i32>();
}

// ── States ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct UiStateDef {
    pub graphic_index: u32,
    pub position_x: f32,
    pub position_y: f32,
    pub zoom_x: f32,
    pub zoom_y: f32,
    pub colour_r: f32,
    pub colour_g: f32,
    pub colour_b: f32,
    pub colour_a: f32,
    pub update_time: f32,
    pub state_change_type: StateChangeType,
    pub linear_change: bool,
    pub state_change_flag: u32,
    pub children_not_affected: Vec<i32>,
}

impl UiStateDef {
    const STATES: &'static str = "States";
    const CHILDREN: &'static str = "ChildrenNotAffected";

    fn parse_list(cur: &mut &[u8]) -> Result<Vec<Self>, ParseControlError> {
        let _id = parse_id(cur, Self::STATES)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::STATES,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut list = Vec::with_capacity(count as usize);
        for _ in 0..count {
            list.push(Self::parse(cur)?);
        }
        Ok(list)
    }

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let graphic_index = parse_scalar::<u32>(cur, "GraphicIndex")?;
        let position_x = parse_scalar::<f32>(cur, "PositionX")?;
        let position_y = parse_scalar::<f32>(cur, "PositionY")?;
        let zoom_x = parse_scalar::<f32>(cur, "ZoomX")?;
        let zoom_y = parse_scalar::<f32>(cur, "ZoomY")?;
        let colour_r = parse_scalar::<f32>(cur, "ColourR")?;
        let colour_g = parse_scalar::<f32>(cur, "ColourG")?;
        let colour_b = parse_scalar::<f32>(cur, "ColourB")?;
        let colour_a = parse_scalar::<f32>(cur, "ColourA")?;
        let update_time = parse_scalar::<f32>(cur, "UpdateTime")?;
        let state_change_type = StateChangeType::parse(cur)?;
        let linear_change = parse_bool(cur, "LinearChange")?;
        let state_change_flag = parse_scalar::<u32>(cur, "StateChangeFlag")?;

        let children_not_affected_count = parse_scalar::<i32>(cur, Self::CHILDREN)?;
        let mut children_not_affected = Vec::with_capacity(children_not_affected_count.max(0) as usize);
        for i in 0..children_not_affected_count {
            let child = take_le::<i32>(cur).map_err(|inner| ParseControlError {
                name: Self::CHILDREN,
                reason: ParseControlErrorReason::ListItem(i as usize, inner),
            })?;
            children_not_affected.push(child);
        }

        Ok(Self {
            graphic_index,
            position_x,
            position_y,
            zoom_x,
            zoom_y,
            colour_r,
            colour_g,
            colour_b,
            colour_a,
            update_time,
            state_change_type,
            linear_change,
            state_change_flag,
            children_not_affected,
        })
    }

    fn serialize_list(list: &[Self], out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::STATES)?;
        put_le(out, &(list.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::STATES,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for item in list {
            item.serialize(out)?;
        }
        Ok(())
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<u32>(out, "GraphicIndex", self.graphic_index)?;
        serialize_scalar::<f32>(out, "PositionX", self.position_x)?;
        serialize_scalar::<f32>(out, "PositionY", self.position_y)?;
        serialize_scalar::<f32>(out, "ZoomX", self.zoom_x)?;
        serialize_scalar::<f32>(out, "ZoomY", self.zoom_y)?;
        serialize_scalar::<f32>(out, "ColourR", self.colour_r)?;
        serialize_scalar::<f32>(out, "ColourG", self.colour_g)?;
        serialize_scalar::<f32>(out, "ColourB", self.colour_b)?;
        serialize_scalar::<f32>(out, "ColourA", self.colour_a)?;
        serialize_scalar::<f32>(out, "UpdateTime", self.update_time)?;
        self.state_change_type.serialize(out)?;
        serialize_bool(out, "LinearChange", self.linear_change)?;
        serialize_scalar::<u32>(out, "StateChangeFlag", self.state_change_flag)?;

        serialize_scalar::<i32>(out, Self::CHILDREN, self.children_not_affected.len() as i32)?;
        for (i, child) in self.children_not_affected.iter().enumerate() {
            put_le(out, child).map_err(|inner| SerializeControlError {
                name: Self::CHILDREN,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<f32>()) * 9
            + StateChangeType::BYTE_SIZE
            + (ID_BYTE_SIZE + size_of::<bool>())
            + (ID_BYTE_SIZE + size_of::<u32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + self.children_not_affected.len() * size_of::<i32>()
    }

    fn list_byte_size(list: &[Self]) -> usize {
        ID_BYTE_SIZE + size_of::<u32>() + list.iter().map(|x| x.byte_size()).sum::<usize>()
    }
}

// ── Sprites (TableSprites key -> i32) ─────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableSprites {
    pub inner: i32,
}

impl TableSprites {
    const NAME: &'static str = "Sprites";
    const BYTE_SIZE: usize = size_of::<i32>();

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let inner = take_le::<i32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::Value(inner),
        })?;
        Ok(Self { inner })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        put_le(out, &self.inner).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::Value(inner),
        })
    }
}

#[derive(Debug)]
pub struct Sprites {
    pub map: BTreeMap<TableSprites, i32>,
}

impl Sprites {
    const NAME: &'static str = "Sprites";

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let _id = parse_id(cur, Self::NAME)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut map = BTreeMap::new();
        for i in 0..count as usize {
            let key = TableSprites::parse(cur)?;
            let value = take_le::<i32>(cur).map_err(|inner| ParseControlError {
                name: Self::NAME,
                reason: ParseControlErrorReason::ListItem(i, inner),
            })?;
            map.insert(key, value);
        }
        Ok(Self { map })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::NAME)?;
        put_le(out, &(self.map.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for (i, (key, value)) in self.map.iter().enumerate() {
            key.serialize(out)?;
            put_le(out, value).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        ID_BYTE_SIZE
            + size_of::<u32>()
            + self.map.len() * (TableSprites::BYTE_SIZE + size_of::<i32>())
    }
}

// ── ActionMap (u32 -> String) ─────────────────────────────────────────────────

#[derive(Debug)]
pub struct ActionMap {
    pub map: BTreeMap<u32, String>,
}

impl ActionMap {
    const NAME: &'static str = "ActionMap";

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let _id = parse_id(cur, Self::NAME)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut map = BTreeMap::new();
        for i in 0..count as usize {
            let key = take_le::<u32>(cur).map_err(|inner| ParseControlError {
                name: Self::NAME,
                reason: ParseControlErrorReason::ListItem(i, inner),
            })?;
            let value = take_null_terminated_utf8(cur)
                .map_err(|inner| ParseControlError {
                    name: Self::NAME,
                    reason: ParseControlErrorReason::Utf8(inner),
                })?
                .to_owned();
            map.insert(key, value);
        }
        Ok(Self { map })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::NAME)?;
        put_le(out, &(self.map.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for (i, (key, value)) in self.map.iter().enumerate() {
            put_le(out, key).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
            put_null_terminated_utf8(out, value).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        ID_BYTE_SIZE
            + size_of::<u32>()
            + self
                .map
                .iter()
                .map(|(_, v)| size_of::<u32>() + v.len() + size_of::<u8>())
                .sum::<usize>()
    }
}

// ── ActionMapAliases (u32 -> u32) ─────────────────────────────────────────────

#[derive(Debug)]
pub struct ActionMapAliases {
    pub map: BTreeMap<u32, u32>,
}

impl ActionMapAliases {
    const NAME: &'static str = "ActionMapAliases";

    fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let _id = parse_id(cur, Self::NAME)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name: Self::NAME,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut map = BTreeMap::new();
        for i in 0..count as usize {
            let key = take_le::<u32>(cur).map_err(|inner| ParseControlError {
                name: Self::NAME,
                reason: ParseControlErrorReason::ListItem(i, inner),
            })?;
            let value = take_le::<u32>(cur).map_err(|inner| ParseControlError {
                name: Self::NAME,
                reason: ParseControlErrorReason::ListItem(i, inner),
            })?;
            map.insert(key, value);
        }
        Ok(Self { map })
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_id(out, Self::NAME)?;
        put_le(out, &(self.map.len() as u32)).map_err(|inner| SerializeControlError {
            name: Self::NAME,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for (i, (key, value)) in self.map.iter().enumerate() {
            put_le(out, key).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
            put_le(out, value).map_err(|inner| SerializeControlError {
                name: Self::NAME,
                reason: SerializeControlErrorReason::ListItem(i, inner),
            })?;
        }
        Ok(())
    }

    fn byte_size(&self) -> usize {
        ID_BYTE_SIZE + size_of::<u32>() + self.map.len() * size_of::<u32>() * 2
    }
}

