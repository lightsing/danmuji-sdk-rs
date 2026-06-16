use std::path::PathBuf;

use crate::ffi::{host_api, FfiStr, HostGetStringFn};

#[derive(Debug, Clone, Copy)]
pub struct Host;

impl Host {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, text: impl AsRef<str>) {
        if let Some(api) = host_api() {
            let text = text.as_ref();
            unsafe {
                (api.log)(api.userdata, FfiStr::borrowed(text));
            }
        }
    }

    pub fn add_dm(&self, text: impl AsRef<str>, fullscreen: bool) {
        if let Some(api) = host_api() {
            let text = text.as_ref();
            unsafe {
                (api.add_dm)(
                    api.userdata,
                    FfiStr::borrowed(text),
                    if fullscreen { 1 } else { 0 },
                );
            }
        }
    }

    pub fn send_ssp_msg(&self, text: impl AsRef<str>) {
        if let Some(api) = host_api() {
            let text = text.as_ref();
            unsafe {
                (api.send_ssp_msg)(api.userdata, FfiStr::borrowed(text));
            }
        }
    }

    pub fn room_id(&self) -> Option<i32> {
        let api = host_api()?;
        let mut room_id = 0;
        let has_room_id = unsafe { (api.get_room_id)(api.userdata, &mut room_id) };

        (has_room_id != 0).then_some(room_id)
    }

    pub fn status(&self) -> bool {
        host_api()
            .map(|api| unsafe { (api.get_status)(api.userdata) != 0 })
            .unwrap_or(false)
    }

    pub fn debug_mode(&self) -> bool {
        host_api()
            .map(|api| unsafe { (api.get_debug_mode)(api.userdata) != 0 })
            .unwrap_or(false)
    }

    pub fn plugin_path(&self) -> Option<PathBuf> {
        self.host_string(|api| api.get_plugin_path)
            .map(PathBuf::from)
    }

    pub fn plugin_dir(&self) -> Option<PathBuf> {
        self.plugin_path()
            .and_then(|path| path.parent().map(PathBuf::from))
    }

    fn host_string(
        &self,
        callback: impl FnOnce(crate::ffi::FfiHostApi) -> HostGetStringFn,
    ) -> Option<String> {
        let api = host_api()?;
        let callback = callback(api);
        let required_len = unsafe { callback(api.userdata, std::ptr::null_mut(), 0) };
        if required_len == 0 {
            return None;
        }

        let mut bytes = vec![0; required_len];
        let written_len = unsafe { callback(api.userdata, bytes.as_mut_ptr(), bytes.len()) };
        if written_len == 0 {
            return None;
        }

        if written_len > bytes.len() {
            bytes.resize(written_len, 0);
            let written_len = unsafe { callback(api.userdata, bytes.as_mut_ptr(), bytes.len()) };
            if written_len == 0 || written_len > bytes.len() {
                return None;
            }

            bytes.truncate(written_len);
        } else {
            bytes.truncate(written_len);
        }

        Some(String::from_utf8_lossy(&bytes).into_owned())
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::path::PathBuf;

    use crate::ffi::{set_host_api, FfiHostApi, FfiStr};

    use super::Host;

    unsafe extern "C" fn host_log(_userdata: *mut c_void, _text: FfiStr) {}

    unsafe extern "C" fn host_add_dm(_userdata: *mut c_void, _text: FfiStr, _fullscreen: i32) {}

    unsafe extern "C" fn host_send_ssp_msg(_userdata: *mut c_void, _text: FfiStr) {}

    unsafe extern "C" fn host_get_room_id(_userdata: *mut c_void, _room_id: *mut i32) -> i32 {
        0
    }

    unsafe extern "C" fn host_get_flag(_userdata: *mut c_void) -> i32 {
        0
    }

    unsafe extern "C" fn host_get_plugin_path(
        _userdata: *mut c_void,
        buffer: *mut u8,
        buffer_len: usize,
    ) -> usize {
        let path = b"/tmp/example_plugin.dll";
        if !buffer.is_null() && buffer_len >= path.len() {
            unsafe {
                std::ptr::copy_nonoverlapping(path.as_ptr(), buffer, path.len());
            }
        }

        path.len()
    }

    #[test]
    fn plugin_path_uses_host_callback() {
        set_host_api(FfiHostApi {
            userdata: std::ptr::null_mut(),
            log: host_log,
            add_dm: host_add_dm,
            send_ssp_msg: host_send_ssp_msg,
            get_room_id: host_get_room_id,
            get_status: host_get_flag,
            get_debug_mode: host_get_flag,
            get_plugin_path: host_get_plugin_path,
        });

        let host = Host::new();

        assert_eq!(
            host.plugin_path(),
            Some(PathBuf::from("/tmp/example_plugin.dll"))
        );
        assert_eq!(host.plugin_dir(), Some(PathBuf::from("/tmp")));
    }
}
