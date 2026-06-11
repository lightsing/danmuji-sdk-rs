use danmuji_sdk::{
    Danmaku, DanmujiPlugin, DisconnectEvent, Host, MsgType, PluginContext, PluginMetadata,
};

#[derive(Default)]
struct ExamplePlugin;

impl DanmujiPlugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Rust Example Plugin",
            author: "Rust",
            contact: "",
            version: "v0.1.0",
            description: "Example plugin implemented in Rust",
        }
    }

    fn start(&mut self, host: Host, ctx: PluginContext) {
        host.log("Rust plugin started");
        host.add_dm("Rust plugin started", true);

        if let Some(room_id) = ctx.room_id {
            host.log(format!("Current room: {room_id}"));
        }
    }

    fn stop(&mut self, host: Host, _ctx: PluginContext) {
        host.log("Rust plugin stopped");
    }

    fn admin(&mut self, host: Host, _ctx: PluginContext) {
        host.log("Rust plugin admin opened");
        host.add_dm("Hello from Rust", true);
    }

    fn connected(&mut self, host: Host, room_id: i32) {
        host.log(format!("Connected to room {room_id}"));
    }

    fn disconnected(&mut self, host: Host, event: DisconnectEvent) {
        match event.error {
            Some(error) => host.log(format!("Disconnected: {error}")),
            None => host.log("Disconnected"),
        }
    }

    fn room_count(&mut self, host: Host, user_count: u32) {
        host.log(format!("Room count: {user_count}"));
    }

    fn danmaku(&mut self, host: Host, danmaku: Danmaku) {
        if danmaku.msg_type == MsgType::Comment {
            let user = danmaku.user_name.as_deref().unwrap_or("unknown");
            let text = danmaku.comment_text.as_deref().unwrap_or("");
            host.log(format!("{user}: {text}"));
        }
    }
}

danmuji_sdk::export_plugin!(ExamplePlugin::default());
