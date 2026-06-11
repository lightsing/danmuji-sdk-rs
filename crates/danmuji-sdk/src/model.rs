use crate::ffi::{FfiPluginContext, FfiPluginMetadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Comment,
    GiftSend,
    GiftTop,
    Welcome,
    LiveStart,
    LiveEnd,
    Unknown,
    WelcomeGuard,
    GuardBuy,
    SuperChat,
    Interact,
    Warning,
    WatchedChange,
    OpConnectionEnd,
    Other(i32),
}

impl MsgType {
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Comment,
            1 => Self::GiftSend,
            2 => Self::GiftTop,
            3 => Self::Welcome,
            4 => Self::LiveStart,
            5 => Self::LiveEnd,
            6 => Self::Unknown,
            7 => Self::WelcomeGuard,
            8 => Self::GuardBuy,
            9 => Self::SuperChat,
            10 => Self::Interact,
            11 => Self::Warning,
            12 => Self::WatchedChange,
            13 => Self::OpConnectionEnd,
            other => Self::Other(other),
        }
    }

    pub fn as_raw(self) -> i32 {
        match self {
            Self::Comment => 0,
            Self::GiftSend => 1,
            Self::GiftTop => 2,
            Self::Welcome => 3,
            Self::LiveStart => 4,
            Self::LiveEnd => 5,
            Self::Unknown => 6,
            Self::WelcomeGuard => 7,
            Self::GuardBuy => 8,
            Self::SuperChat => 9,
            Self::Interact => 10,
            Self::Warning => 11,
            Self::WatchedChange => 12,
            Self::OpConnectionEnd => 13,
            Self::Other(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractType {
    Enter,
    Follow,
    Share,
    SpecialFollow,
    MutualFollow,
    Like,
    Other(i32),
}

impl InteractType {
    pub fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => None,
            1 => Some(Self::Enter),
            2 => Some(Self::Follow),
            3 => Some(Self::Share),
            4 => Some(Self::SpecialFollow),
            5 => Some(Self::MutualFollow),
            6 => Some(Self::Like),
            other => Some(Self::Other(other)),
        }
    }

    pub fn as_raw(self) -> i32 {
        match self {
            Self::Enter => 1,
            Self::Follow => 2,
            Self::Share => 3,
            Self::SpecialFollow => 4,
            Self::MutualFollow => 5,
            Self::Like => 6,
            Self::Other(value) => value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: &'static str,
    pub author: &'static str,
    pub contact: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

impl PluginMetadata {
    pub fn into_ffi(self) -> FfiPluginMetadata {
        FfiPluginMetadata {
            name: self.name.into(),
            author: self.author.into(),
            contact: self.contact.into(),
            version: self.version.into(),
            description: self.description.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginContext {
    pub room_id: Option<i32>,
    pub status: bool,
    pub debug_mode: bool,
}

impl From<FfiPluginContext> for PluginContext {
    fn from(value: FfiPluginContext) -> Self {
        Self {
            room_id: (value.has_room_id != 0).then_some(value.room_id),
            status: value.status != 0,
            debug_mode: value.debug_mode != 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisconnectEvent {
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GiftRank {
    pub user_name: Option<String>,
    pub coin: Option<String>,
    pub uid: i32,
    pub uid_long: i64,
    pub uid_str: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Danmaku {
    pub msg_type: MsgType,
    pub interact_type: Option<InteractType>,
    pub comment_text: Option<String>,
    pub comment_user: Option<String>,
    pub user_name: Option<String>,
    pub user_id: i32,
    pub user_id_long: i64,
    pub user_id_str: Option<String>,
    pub user_guard_level: i32,
    pub gift_user: Option<String>,
    pub gift_name: Option<String>,
    pub gift_num: Option<String>,
    pub gift_count: i32,
    pub gift_rcost: Option<String>,
    pub gift_ranking: Vec<GiftRank>,
    pub is_admin: bool,
    pub is_vip: bool,
    pub room_id: Option<String>,
    pub raw_data: Option<String>,
    pub json_version: i32,
    pub price: Option<String>,
    pub sc_keep_time: i32,
    pub watched_count: i64,
    pub raw_data_jtoken: Option<String>,
}
