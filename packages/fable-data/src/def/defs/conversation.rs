use crate::def_struct;

def_struct! {
    /// C++ `CConversation` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct Conversation {
        "Lines" => pub lines: i32,
        "Speaker" => pub speaker: Vec<String>,
        "Dialogue" => pub dialogue: Vec<String>,
        "Animation" => pub animation: Vec<String>,
        "AnimLoop" => pub anim_loop: Vec<String>,
    }
}
