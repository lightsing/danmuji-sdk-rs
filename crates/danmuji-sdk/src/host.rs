use crate::ffi::{host_api, FfiStr};

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
                (api.log)(api.userdata, FfiStr::from_str(text));
            }
        }
    }

    pub fn add_dm(&self, text: impl AsRef<str>, fullscreen: bool) {
        if let Some(api) = host_api() {
            let text = text.as_ref();
            unsafe {
                (api.add_dm)(
                    api.userdata,
                    FfiStr::from_str(text),
                    if fullscreen { 1 } else { 0 },
                );
            }
        }
    }

    pub fn send_ssp_msg(&self, text: impl AsRef<str>) {
        if let Some(api) = host_api() {
            let text = text.as_ref();
            unsafe {
                (api.send_ssp_msg)(api.userdata, FfiStr::from_str(text));
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
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}
