use crate::DefStruct;
use crate::def::{
    defs::UiStateDef,
    wire::WStr,
};
use std::collections::BTreeMap;


#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct UiDef {
    #[def("Type")]
    pub type_: i32,
    #[def("Children")]
    pub children: Vec<u32>,
    #[def("MeshIndex")]
    pub mesh_index: u32,
    #[def("TextValue")]
    pub text_value: WStr,
    #[def("Font")]
    pub font: i32,
    #[def("Height")]
    pub height: f32,
    #[def("Width")]
    pub width: f32,
    #[def("ExpansionType")]
    pub expansion_type: i32,
    #[def("Sprites")]
    pub sprites: BTreeMap<i32, i32>,
    #[def("HorizontalSeparations")]
    pub horizontal_separations: Vec<u32>,
    #[def("VerticalSeparations")]
    pub vertical_separations: Vec<u32>,
    #[def("States")]
    pub states: Vec<UiStateDef>,
    #[def("TextLineBreak")]
    pub text_line_break: bool,
    #[def("ScaleText")]
    pub scale_text: bool,
    #[def("Independant")]
    pub independant: bool,
    #[def("MeshType")]
    pub mesh_type: i32,
    #[def("NonScrollingChildren")]
    pub non_scrolling_children: Vec<u32>,
    #[def("TextWindowTLX")]
    pub text_window_tlx: f32,
    #[def("TextWindowTLY")]
    pub text_window_tly: f32,
    #[def("TextWindowBRX")]
    pub text_window_brx: f32,
    #[def("TextWindowBRY")]
    pub text_window_bry: f32,
    #[def("Layer")]
    pub layer: i32,
    #[def("Angle")]
    pub angle: f32,
    #[def("PositionIsCenter")]
    pub position_is_center: bool,
    #[def("ScrollingSpeed")]
    pub scrolling_speed: f32,
    #[def("Wrapping")]
    pub wrapping: bool,
    #[def("Inverted")]
    pub inverted: bool,
    #[def("PositionOffsetX")]
    pub position_offset_x: f32,
    #[def("PositionOffsetY")]
    pub position_offset_y: f32,
    #[def("AlphaOffset")]
    pub alpha_offset: u32,
    #[def("UpX")]
    pub up_x: f32,
    #[def("UpY")]
    pub up_y: f32,
    #[def("UpZ")]
    pub up_z: f32,
    #[def("ForwardX")]
    pub forward_x: f32,
    #[def("ForwardY")]
    pub forward_y: f32,
    #[def("ForwardZ")]
    pub forward_z: f32,
    #[def("RotationAxisX")]
    pub rotation_axis_x: f32,
    #[def("RotationAxisY")]
    pub rotation_axis_y: f32,
    #[def("RotationAxisZ")]
    pub rotation_axis_z: f32,
    #[def("RotationSpeed")]
    pub rotation_speed: f32,
    #[def("AnimationIndex")]
    pub animation_index: u32,
    #[def("DownArrow")]
    pub down_arrow: i32,
    #[def("UpArrow")]
    pub up_arrow: i32,
    #[def("UpLimit")]
    pub up_limit: i32,
    #[def("DownLimit")]
    pub down_limit: i32,
    #[def("Scrolling")]
    pub scrolling: bool,
    #[def("ComputeOffsetsOnActivate")]
    pub compute_offsets_on_activate: bool,
    #[def("MinX")]
    pub min_x: f32,
    #[def("MinY")]
    pub min_y: f32,
    #[def("MaxX")]
    pub max_x: f32,
    #[def("MaxY")]
    pub max_y: f32,
    #[def("StepX")]
    pub step_x: f32,
    #[def("StepY")]
    pub step_y: f32,
    #[def("DimensionsX")]
    pub dimensions_x: f32,
    #[def("DimensionsY")]
    pub dimensions_y: f32,
    #[def("SliderLeft")]
    pub slider_left: i32,
    #[def("SliderRight")]
    pub slider_right: i32,
    #[def("Action")]
    pub action: i32,
    #[def("ActionOnBack")]
    pub action_on_back: i32,
    #[def("ActionOnSelected")]
    pub action_on_selected: i32,
    #[def("ActionOnUnselected")]
    pub action_on_unselected: i32,
    #[def("ActionOnDestruction")]
    pub action_on_destruction: i32,
    #[def("ActionOnLeftClicked")]
    pub action_on_left_clicked: i32,
    #[def("ActionOnLeftUnclicked")]
    pub action_on_left_unclicked: i32,
    #[def("ActionOnLeftHeld")]
    pub action_on_left_held: i32,
    #[def("ActionOnRightClicked")]
    pub action_on_right_clicked: i32,
    #[def("ActionOnDropped")]
    pub action_on_dropped: i32,
    #[def("ActionOnDroppedNowhere")]
    pub action_on_dropped_nowhere: i32,
    #[def("PreAction")]
    pub pre_action: i32,
    #[def("ActionOnDraggedUp")]
    pub action_on_dragged_up: i32,
    #[def("ActionOnDraggedDown")]
    pub action_on_dragged_down: i32,
    #[def("ActionOnLeftClickedAbove")]
    pub action_on_left_clicked_above: i32,
    #[def("ActionOnLeftClickedUnder")]
    pub action_on_left_clicked_under: i32,
    #[def("InputDelay")]
    pub input_delay: f32,
    #[def("DrawFromViewport")]
    pub draw_from_viewport: bool,
    #[def("TextBankIndex")]
    pub text_bank_index: u32,
    #[def("ActionText")]
    pub action_text: i32,
    #[def("KeyText")]
    pub key_text: i32,
    #[def("Redefiner")]
    pub redefiner: i32,
    #[def("UndefinedWarning")]
    pub undefined_warning: i32,
    #[def("ActionMap")]
    pub action_map: BTreeMap<u32, String>,
    #[def("ActionMapAliases")]
    pub action_map_aliases: BTreeMap<u32, u32>,
    #[def("ActionOrder")]
    pub action_order: Vec<u32>,
    #[def("EditBoxParentIsButton")]
    pub edit_box_parent_is_button: bool,
    #[def("PasswordBox")]
    pub password_box: bool,
    #[def("EditBoxCharLimit")]
    pub edit_box_char_limit: i32,
    #[def("EditBoxUsesIME")]
    pub edit_box_uses_ime: bool,
    #[def("MovieFilename")]
    pub movie_filename: WStr,
    #[def("DisallowSpaceAsFirstChar")]
    pub disallow_space_as_first_char: bool,
    #[def("LayerIndependant")]
    pub layer_independant: bool,
    #[def("SwappingStates")]
    pub swapping_states: Vec<u32>,
    #[def("SwappingTimes")]
    pub swapping_times: Vec<f32>,
    #[def("BastardChild")]
    pub bastard_child: bool,
    #[def("Alignement")]
    pub alignement: i32,
    #[def("RandomSwap")]
    pub random_swap: bool,
    #[def("UseRelativeZoom")]
    pub use_relative_zoom: bool,
    #[def("UseRelativePosition")]
    pub use_relative_position: bool,
    #[def("HoveredState")]
    pub hovered_state: i32,
    #[def("LeftClickedState")]
    pub left_clicked_state: i32,
    #[def("RightClickedState")]
    pub right_clicked_state: i32,
    #[def("ShapeChildren")]
    pub shape_children: Vec<u32>,
    #[def("ViewAreaTLX")]
    pub view_area_tlx: i32,
    #[def("ViewAreaTLY")]
    pub view_area_tly: i32,
    #[def("ViewAreaBRX")]
    pub view_area_brx: i32,
    #[def("ViewAreaBRY")]
    pub view_area_bry: i32,
    #[def("UseViewArea")]
    pub use_view_area: bool,
    #[def("PartOfListTree")]
    pub part_of_list_tree: bool,
    #[def("PCStyle")]
    pub pc_style: bool,
    #[def("Sprite2DFlag")]
    pub sprite2_d_flag: i32,
}

