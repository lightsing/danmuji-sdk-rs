use std::ffi::c_void;
use std::slice;
use std::sync::Mutex;

use crate::model::{Danmaku, GiftRank, InteractType, MsgType};

pub type HostLogFn = unsafe extern "C" fn(userdata: *mut c_void, text: FfiStr);
pub type HostAddDmFn = unsafe extern "C" fn(userdata: *mut c_void, text: FfiStr, fullscreen: i32);
pub type HostSendSspMsgFn = unsafe extern "C" fn(userdata: *mut c_void, text: FfiStr);
pub type HostGetRoomIdFn = unsafe extern "C" fn(userdata: *mut c_void, room_id: *mut i32) -> i32;
pub type HostGetFlagFn = unsafe extern "C" fn(userdata: *mut c_void) -> i32;
pub type HostGetStringFn =
    unsafe extern "C" fn(userdata: *mut c_void, buffer: *mut u8, buffer_len: usize) -> usize;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiStr {
    pub ptr: *const u8,
    pub len: usize,
}

impl FfiStr {
    pub fn null() -> Self {
        Self {
            ptr: std::ptr::null(),
            len: 0,
        }
    }

    pub fn borrowed(value: &str) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }

    /// Converts a non-null FFI string into an owned Rust string.
    ///
    /// # Safety
    ///
    /// `self.ptr` must either be null or point to `self.len` readable bytes for the
    /// duration of this call.
    pub unsafe fn to_string_lossy(self) -> Option<String> {
        if self.ptr.is_null() {
            return None;
        }

        let bytes = unsafe { slice::from_raw_parts(self.ptr, self.len) };
        Some(String::from_utf8_lossy(bytes).into_owned())
    }
}

impl From<&'static str> for FfiStr {
    fn from(value: &'static str) -> Self {
        Self::borrowed(value)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiPluginMetadata {
    pub name: FfiStr,
    pub author: FfiStr,
    pub contact: FfiStr,
    pub version: FfiStr,
    pub description: FfiStr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiPluginContext {
    pub has_room_id: i32,
    pub room_id: i32,
    pub status: i32,
    pub debug_mode: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiHostApi {
    pub userdata: *mut c_void,
    pub log: HostLogFn,
    pub add_dm: HostAddDmFn,
    pub send_ssp_msg: HostSendSspMsgFn,
    pub get_room_id: HostGetRoomIdFn,
    pub get_status: HostGetFlagFn,
    pub get_debug_mode: HostGetFlagFn,
    pub get_plugin_path: HostGetStringFn,
}

unsafe impl Send for FfiHostApi {}
unsafe impl Sync for FfiHostApi {}

static HOST_API: Mutex<Option<FfiHostApi>> = Mutex::new(None);

pub fn set_host_api(api: FfiHostApi) {
    if let Ok(mut host_api) = HOST_API.lock() {
        *host_api = Some(api);
    }
}

pub fn host_api() -> Option<FfiHostApi> {
    HOST_API.lock().ok().and_then(|api| *api)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiGiftRank {
    pub user_name: FfiStr,
    pub coin: FfiStr,
    pub uid: i32,
    pub uid_long: i64,
    pub uid_str: FfiStr,
}

impl FfiGiftRank {
    /// Converts an FFI gift rank into the safe Rust model.
    ///
    /// # Safety
    ///
    /// All `FfiStr` fields must either be null or point to readable UTF-8 byte
    /// sequences for the duration of this call.
    pub unsafe fn to_model(self) -> GiftRank {
        GiftRank {
            user_name: unsafe { self.user_name.to_string_lossy() },
            coin: unsafe { self.coin.to_string_lossy() },
            uid: self.uid,
            uid_long: self.uid_long,
            uid_str: unsafe { self.uid_str.to_string_lossy() },
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiDanmaku {
    pub msg_type: i32,
    pub interact_type: i32,
    pub comment_text: FfiStr,
    pub comment_user: FfiStr,
    pub user_name: FfiStr,
    pub user_id: i32,
    pub user_id_long: i64,
    pub user_id_str: FfiStr,
    pub user_guard_level: i32,
    pub gift_user: FfiStr,
    pub gift_name: FfiStr,
    pub gift_num: FfiStr,
    pub gift_count: i32,
    pub gift_rcost: FfiStr,
    pub gift_ranking_ptr: *const FfiGiftRank,
    pub gift_ranking_len: usize,
    pub is_admin: i32,
    pub is_vip: i32,
    pub room_id: FfiStr,
    pub raw_data: FfiStr,
    pub json_version: i32,
    pub price: FfiStr,
    pub sc_keep_time: i32,
    pub watched_count: i64,
    pub raw_data_jtoken: FfiStr,
}

impl FfiDanmaku {
    /// Converts an FFI danmaku payload into the safe Rust model.
    ///
    /// # Safety
    ///
    /// All `FfiStr` fields must either be null or point to readable UTF-8 byte
    /// sequences. When `gift_ranking_ptr` is non-null, it must point to
    /// `gift_ranking_len` readable `FfiGiftRank` values for the duration of this
    /// call.
    pub unsafe fn to_model(self) -> Danmaku {
        let gift_ranking = if self.gift_ranking_ptr.is_null() || self.gift_ranking_len == 0 {
            Vec::new()
        } else {
            unsafe { slice::from_raw_parts(self.gift_ranking_ptr, self.gift_ranking_len) }
                .iter()
                .map(|rank| unsafe { (*rank).to_model() })
                .collect()
        };

        Danmaku {
            msg_type: MsgType::from_raw(self.msg_type),
            interact_type: InteractType::from_raw(self.interact_type),
            comment_text: unsafe { self.comment_text.to_string_lossy() },
            comment_user: unsafe { self.comment_user.to_string_lossy() },
            user_name: unsafe { self.user_name.to_string_lossy() },
            user_id: self.user_id,
            user_id_long: self.user_id_long,
            user_id_str: unsafe { self.user_id_str.to_string_lossy() },
            user_guard_level: self.user_guard_level,
            gift_user: unsafe { self.gift_user.to_string_lossy() },
            gift_name: unsafe { self.gift_name.to_string_lossy() },
            gift_num: unsafe { self.gift_num.to_string_lossy() },
            gift_count: self.gift_count,
            gift_rcost: unsafe { self.gift_rcost.to_string_lossy() },
            gift_ranking,
            is_admin: self.is_admin != 0,
            is_vip: self.is_vip != 0,
            room_id: unsafe { self.room_id.to_string_lossy() },
            raw_data: unsafe { self.raw_data.to_string_lossy() },
            json_version: self.json_version,
            price: unsafe { self.price.to_string_lossy() },
            sc_keep_time: self.sc_keep_time,
            watched_count: self.watched_count,
            raw_data_jtoken: unsafe { self.raw_data_jtoken.to_string_lossy() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FfiStr;

    #[test]
    fn ffi_str_round_trips_utf8() {
        let text = "hello";
        let ffi = FfiStr::borrowed(text);

        let decoded = unsafe { ffi.to_string_lossy() };

        assert_eq!(decoded.as_deref(), Some(text));
    }

    #[test]
    fn null_ffi_str_is_none() {
        let decoded = unsafe { FfiStr::null().to_string_lossy() };

        assert!(decoded.is_none());
    }
}
