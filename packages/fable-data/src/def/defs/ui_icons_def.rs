use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `UI_ICONS_DEF` — C++ `NUISystem::CUIIconsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct UiIconsDef {
        "IconFriendRequestReceived" => pub icon_friend_request_received: u32,
        "IconFriendRequestReceivedOn" => pub icon_friend_request_received_on: u32,
        "IconFriendRequestSent" => pub icon_friend_request_sent: u32,
        "IconFriendRequestSentOn" => pub icon_friend_request_sent_on: u32,
        "IconGameInviteReceived" => pub icon_game_invite_received: u32,
        "IconGameInviteReceivedOn" => pub icon_game_invite_received_on: u32,
        "IconGameInviteSent" => pub icon_game_invite_sent: u32,
        "IconGameInviteSentOn" => pub icon_game_invite_sent_on: u32,
        "IconMute" => pub icon_mute: u32,
        "IconMuteOn" => pub icon_mute_on: u32,
        "IconOnline" => pub icon_online: u32,
        "IconOnlineOn" => pub icon_online_on: u32,
        "IconPasscodeBlank" => pub icon_passcode_blank: u32,
        "IconPasscodeFilled" => pub icon_passcode_filled: u32,
        "IconTV" => pub icon_tv: u32,
        "IconTVOn" => pub icon_tv_on: u32,
        "IconVoice" => pub icon_voice: u32,
        "IconVoiceOn" => pub icon_voice_on: u32,
        "IconWait1" => pub icon_wait1: u32,
        "IconWait2" => pub icon_wait2: u32,
        "IconWait3" => pub icon_wait3: u32,
        "IconWait4" => pub icon_wait4: u32,
        "IconProgress" => pub icon_progress: u32,
        "IconProgressOn" => pub icon_progress_on: u32,
        "IconA" => pub icon_a: u32,
        "IconB" => pub icon_b: u32,
        "IconX" => pub icon_x: u32,
        "IconY" => pub icon_y: u32,
        "IconBlank" => pub icon_blank: u32,
        "IconUpArrow" => pub icon_up_arrow: u32,
        "IconDownArrow" => pub icon_down_arrow: u32,
        "IconListHighlight" => pub icon_list_highlight: u32,
    }
}

