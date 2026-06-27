use super::binary::control::{ID_BYTE_SIZE, ParseControlError, SerializeControlError, parse_scalar, serialize_scalar};

#[derive(Debug)]
pub struct UiIconsDef {
    pub icon_friend_request_received: u32,
    pub icon_friend_request_received_on: u32,
    pub icon_friend_request_sent: u32,
    pub icon_friend_request_sent_on: u32,
    pub icon_game_invite_received: u32,
    pub icon_game_invite_received_on: u32,
    pub icon_game_invite_sent: u32,
    pub icon_game_invite_sent_on: u32,
    pub icon_mute: u32,
    pub icon_mute_on: u32,
    pub icon_online: u32,
    pub icon_online_on: u32,
    pub icon_passcode_blank: u32,
    pub icon_passcode_filled: u32,
    pub icon_tv: u32,
    pub icon_tv_on: u32,
    pub icon_voice: u32,
    pub icon_voice_on: u32,
    pub icon_wait_1: u32,
    pub icon_wait_2: u32,
    pub icon_wait_3: u32,
    pub icon_wait_4: u32,
    pub icon_progress: u32,
    pub icon_progress_on: u32,
    pub icon_a: u32,
    pub icon_b: u32,
    pub icon_x: u32,
    pub icon_y: u32,
    pub icon_blank: u32,
    pub icon_up_arrow: u32,
    pub icon_down_arrow: u32,
    pub icon_list_highlight: u32,
}

impl UiIconsDef {
    pub(crate) const BYTE_SIZE: usize = ID_BYTE_SIZE * 32 + size_of::<UiIconsDef>();

    pub(crate) const fn byte_size(&self) -> usize {
        Self::BYTE_SIZE
    }

    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            icon_friend_request_received: parse_scalar::<u32>(cur, "IconFriendRequestReceived")?,
            icon_friend_request_received_on: parse_scalar::<u32>(cur, "IconFriendRequestReceivedOn")?,
            icon_friend_request_sent: parse_scalar::<u32>(cur, "IconFriendRequestSent")?,
            icon_friend_request_sent_on: parse_scalar::<u32>(cur, "IconFriendRequestSentOn")?,
            icon_game_invite_received: parse_scalar::<u32>(cur, "IconGameInviteReceived")?,
            icon_game_invite_received_on: parse_scalar::<u32>(cur, "IconGameInviteReceivedOn")?,
            icon_game_invite_sent: parse_scalar::<u32>(cur, "IconGameInviteSent")?,
            icon_game_invite_sent_on: parse_scalar::<u32>(cur, "IconGameInviteSentOn")?,
            icon_mute: parse_scalar::<u32>(cur, "IconMute")?,
            icon_mute_on: parse_scalar::<u32>(cur, "IconMuteOn")?,
            icon_online: parse_scalar::<u32>(cur, "IconOnline")?,
            icon_online_on: parse_scalar::<u32>(cur, "IconOnlineOn")?,
            icon_passcode_blank: parse_scalar::<u32>(cur, "IconPasscodeBlank")?,
            icon_passcode_filled: parse_scalar::<u32>(cur, "IconPasscodeFilled")?,
            icon_tv: parse_scalar::<u32>(cur, "IconTV")?,
            icon_tv_on: parse_scalar::<u32>(cur, "IconTVOn")?,
            icon_voice: parse_scalar::<u32>(cur, "IconVoice")?,
            icon_voice_on: parse_scalar::<u32>(cur, "IconVoiceOn")?,
            icon_wait_1: parse_scalar::<u32>(cur, "IconWait1")?,
            icon_wait_2: parse_scalar::<u32>(cur, "IconWait2")?,
            icon_wait_3: parse_scalar::<u32>(cur, "IconWait3")?,
            icon_wait_4: parse_scalar::<u32>(cur, "IconWait4")?,
            icon_progress: parse_scalar::<u32>(cur, "IconProgress")?,
            icon_progress_on: parse_scalar::<u32>(cur, "IconProgressOn")?,
            icon_a: parse_scalar::<u32>(cur, "IconA")?,
            icon_b: parse_scalar::<u32>(cur, "IconB")?,
            icon_x: parse_scalar::<u32>(cur, "IconX")?,
            icon_y: parse_scalar::<u32>(cur, "IconY")?,
            icon_blank: parse_scalar::<u32>(cur, "IconBlank")?,
            icon_up_arrow: parse_scalar::<u32>(cur, "IconUpArrow")?,
            icon_down_arrow: parse_scalar::<u32>(cur, "IconDownArrow")?,
            icon_list_highlight: parse_scalar::<u32>(cur, "IconListHighlight")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<u32>(out, "IconFriendRequestReceived", self.icon_friend_request_received)?;
        serialize_scalar::<u32>(out, "IconFriendRequestReceivedOn", self.icon_friend_request_received_on)?;
        serialize_scalar::<u32>(out, "IconFriendRequestSent", self.icon_friend_request_sent)?;
        serialize_scalar::<u32>(out, "IconFriendRequestSentOn", self.icon_friend_request_sent_on)?;
        serialize_scalar::<u32>(out, "IconGameInviteReceived", self.icon_game_invite_received)?;
        serialize_scalar::<u32>(out, "IconGameInviteReceivedOn", self.icon_game_invite_received_on)?;
        serialize_scalar::<u32>(out, "IconGameInviteSent", self.icon_game_invite_sent)?;
        serialize_scalar::<u32>(out, "IconGameInviteSentOn", self.icon_game_invite_sent_on)?;
        serialize_scalar::<u32>(out, "IconMute", self.icon_mute)?;
        serialize_scalar::<u32>(out, "IconMuteOn", self.icon_mute_on)?;
        serialize_scalar::<u32>(out, "IconOnline", self.icon_online)?;
        serialize_scalar::<u32>(out, "IconOnlineOn", self.icon_online_on)?;
        serialize_scalar::<u32>(out, "IconPasscodeBlank", self.icon_passcode_blank)?;
        serialize_scalar::<u32>(out, "IconPasscodeFilled", self.icon_passcode_filled)?;
        serialize_scalar::<u32>(out, "IconTV", self.icon_tv)?;
        serialize_scalar::<u32>(out, "IconTVOn", self.icon_tv_on)?;
        serialize_scalar::<u32>(out, "IconVoice", self.icon_voice)?;
        serialize_scalar::<u32>(out, "IconVoiceOn", self.icon_voice_on)?;
        serialize_scalar::<u32>(out, "IconWait1", self.icon_wait_1)?;
        serialize_scalar::<u32>(out, "IconWait2", self.icon_wait_2)?;
        serialize_scalar::<u32>(out, "IconWait3", self.icon_wait_3)?;
        serialize_scalar::<u32>(out, "IconWait4", self.icon_wait_4)?;
        serialize_scalar::<u32>(out, "IconProgress", self.icon_progress)?;
        serialize_scalar::<u32>(out, "IconProgressOn", self.icon_progress_on)?;
        serialize_scalar::<u32>(out, "IconA", self.icon_a)?;
        serialize_scalar::<u32>(out, "IconB", self.icon_b)?;
        serialize_scalar::<u32>(out, "IconX", self.icon_x)?;
        serialize_scalar::<u32>(out, "IconY", self.icon_y)?;
        serialize_scalar::<u32>(out, "IconBlank", self.icon_blank)?;
        serialize_scalar::<u32>(out, "IconUpArrow", self.icon_up_arrow)?;
        serialize_scalar::<u32>(out, "IconDownArrow", self.icon_down_arrow)?;
        serialize_scalar::<u32>(out, "IconListHighlight", self.icon_list_highlight)?;
        Ok(())
    }

}
