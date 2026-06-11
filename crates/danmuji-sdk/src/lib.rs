//! Rust side SDK for writing Bilibili Danmuji plugins through the .NET bridge.

pub mod ffi;
pub mod host;
pub mod model;

pub use host::Host;
pub use model::{
    Danmaku, DisconnectEvent, GiftRank, InteractType, MsgType, PluginContext, PluginMetadata,
};

pub trait DanmujiPlugin: Send + 'static {
    fn metadata(&self) -> PluginMetadata;

    fn inited(&mut self, _host: Host, _ctx: PluginContext) {}

    fn start(&mut self, _host: Host, _ctx: PluginContext) {}

    fn stop(&mut self, _host: Host, _ctx: PluginContext) {}

    fn admin(&mut self, _host: Host, _ctx: PluginContext) {}

    fn deinit(&mut self, _host: Host, _ctx: PluginContext) {}

    fn connected(&mut self, _host: Host, _room_id: i32) {}

    fn disconnected(&mut self, _host: Host, _event: DisconnectEvent) {}

    fn room_count(&mut self, _host: Host, _user_count: u32) {}

    fn danmaku(&mut self, _host: Host, _danmaku: Danmaku) {}
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin:expr) => {
        static DANMUJI_RS_PLUGIN: ::std::sync::OnceLock<
            ::std::sync::Mutex<Box<dyn $crate::DanmujiPlugin>>,
        > = ::std::sync::OnceLock::new();

        fn danmuji_rs_plugin_instance(
        ) -> &'static ::std::sync::Mutex<Box<dyn $crate::DanmujiPlugin>> {
            DANMUJI_RS_PLUGIN.get_or_init(|| ::std::sync::Mutex::new(Box::new($plugin)))
        }

        fn danmuji_rs_with_plugin<F>(f: F)
        where
            F: FnOnce(&mut dyn $crate::DanmujiPlugin),
        {
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let instance = danmuji_rs_plugin_instance();
                match instance.lock() {
                    Ok(mut plugin) => f(plugin.as_mut()),
                    Err(poisoned) => {
                        let mut plugin = poisoned.into_inner();
                        f(plugin.as_mut());
                    }
                }
            }));

            if result.is_err() {
                $crate::Host::new().log("Rust plugin panic was caught by danmuji-sdk");
            }
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_metadata(out: *mut $crate::ffi::FfiPluginMetadata) {
            if out.is_null() {
                return;
            }

            let metadata = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let instance = danmuji_rs_plugin_instance();
                match instance.lock() {
                    Ok(plugin) => plugin.metadata(),
                    Err(poisoned) => poisoned.into_inner().metadata(),
                }
            }))
            .unwrap_or($crate::PluginMetadata {
                name: "Rust Plugin",
                author: "",
                contact: "",
                version: "",
                description: "Rust plugin metadata failed",
            });

            unsafe {
                *out = metadata.into_ffi();
            }
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_set_host(api: $crate::ffi::FfiHostApi) {
            $crate::ffi::set_host_api(api);
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_inited(ctx: $crate::ffi::FfiPluginContext) {
            danmuji_rs_with_plugin(|plugin| plugin.inited($crate::Host::new(), ctx.into()));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_start(ctx: $crate::ffi::FfiPluginContext) {
            danmuji_rs_with_plugin(|plugin| plugin.start($crate::Host::new(), ctx.into()));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_stop(ctx: $crate::ffi::FfiPluginContext) {
            danmuji_rs_with_plugin(|plugin| plugin.stop($crate::Host::new(), ctx.into()));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_admin(ctx: $crate::ffi::FfiPluginContext) {
            danmuji_rs_with_plugin(|plugin| plugin.admin($crate::Host::new(), ctx.into()));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_deinit(ctx: $crate::ffi::FfiPluginContext) {
            danmuji_rs_with_plugin(|plugin| plugin.deinit($crate::Host::new(), ctx.into()));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_on_connected(room_id: i32) {
            danmuji_rs_with_plugin(|plugin| plugin.connected($crate::Host::new(), room_id));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_on_disconnected(error: $crate::ffi::FfiStr) {
            let event = $crate::DisconnectEvent {
                error: unsafe { error.to_string_lossy() },
            };
            danmuji_rs_with_plugin(|plugin| plugin.disconnected($crate::Host::new(), event));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_on_room_count(user_count: u32) {
            danmuji_rs_with_plugin(|plugin| plugin.room_count($crate::Host::new(), user_count));
        }

        #[no_mangle]
        pub extern "C" fn danmuji_rs_plugin_on_danmaku(danmaku: *const $crate::ffi::FfiDanmaku) {
            if danmaku.is_null() {
                return;
            }

            let danmaku = unsafe { (*danmaku).to_model() };
            danmuji_rs_with_plugin(|plugin| plugin.danmaku($crate::Host::new(), danmaku));
        }
    };
}
