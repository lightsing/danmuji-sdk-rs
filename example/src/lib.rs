use std::time::Instant;

use danmuji_sdk::{
    Danmaku, DanmujiPlugin, DisconnectEvent, Host, InteractType, MsgType, PluginContext,
    PluginMetadata,
};

#[derive(Default)]
struct SamplePlugin {
    started_at: Option<Instant>,
    comments: u64,
    gifts: u64,
    super_chats: u64,
    latest_room_count: Option<u32>,
}

impl DanmujiPlugin for SamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Rust Sample Danmuji Plugin",
            author: "danmuji-sdk-rs",
            contact: "",
            version: "v0.1.0",
            description: "Sample Rust plugin for Bilibili Danmuji",
        }
    }

    fn start(&mut self, host: Host, ctx: PluginContext) {
        self.started_at = Some(Instant::now());
        self.comments = 0;
        self.gifts = 0;
        self.super_chats = 0;
        self.latest_room_count = None;

        host.log("Sample Rust plugin started");
        host.add_dm("Sample Rust plugin started", true);

        if let Some(room_id) = ctx.room_id {
            host.log(format!("Current room: {room_id}"));
        }
    }

    fn stop(&mut self, host: Host, _ctx: PluginContext) {
        host.log(format!("Sample Rust plugin stopped. {}", self.summary()));
    }

    fn admin(&mut self, host: Host, _ctx: PluginContext) {
        let summary = self.summary();
        host.log(format!("Admin summary: {summary}"));
        host.add_dm(summary, true);
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
        self.latest_room_count = Some(user_count);
        host.log(format!("Room count changed: {user_count}"));
    }

    fn danmaku(&mut self, host: Host, danmaku: Danmaku) {
        match danmaku.msg_type {
            MsgType::Comment => self.handle_comment(host, &danmaku),
            MsgType::GiftSend | MsgType::GuardBuy => self.handle_gift(host, &danmaku),
            MsgType::SuperChat => self.handle_super_chat(host, &danmaku),
            MsgType::Interact => self.handle_interact(host, &danmaku),
            MsgType::LiveStart => host.log("Live started"),
            MsgType::LiveEnd => host.log("Live ended"),
            _ => {}
        }
    }
}

impl SamplePlugin {
    fn handle_comment(&mut self, host: Host, danmaku: &Danmaku) {
        self.comments += 1;

        let user = danmaku.user_name.as_deref().unwrap_or("unknown");
        let text = danmaku.comment_text.as_deref().unwrap_or("");

        host.log(format!("[comment] {user}: {text}"));

        if text.trim().eq_ignore_ascii_case("!ping") {
            host.add_dm(format!("pong, {user}"), false);
        }
    }

    fn handle_gift(&mut self, host: Host, danmaku: &Danmaku) {
        let count = u64::try_from(danmaku.gift_count).unwrap_or(0).max(1);
        self.gifts += count;

        let user = danmaku.user_name.as_deref().unwrap_or("unknown");
        let gift_name = danmaku.gift_name.as_deref().unwrap_or("gift");

        host.log(format!("[gift] {user} sent {gift_name} x{count}"));
    }

    fn handle_super_chat(&mut self, host: Host, danmaku: &Danmaku) {
        self.super_chats += 1;

        let user = danmaku.user_name.as_deref().unwrap_or("unknown");
        let price = danmaku.price.as_deref().unwrap_or("?");
        let text = danmaku.comment_text.as_deref().unwrap_or("");

        host.log(format!("[super chat] {user} paid {price}: {text}"));
        host.add_dm(format!("SC from {user}: {text}"), true);
    }

    fn handle_interact(&mut self, host: Host, danmaku: &Danmaku) {
        let user = danmaku.user_name.as_deref().unwrap_or("unknown");

        match danmaku.interact_type {
            Some(InteractType::Follow) => host.log(format!("[follow] {user} followed")),
            Some(InteractType::Share) => host.log(format!("[share] {user} shared the room")),
            Some(InteractType::Like) => host.log(format!("[like] {user} liked")),
            _ => {}
        }
    }

    fn summary(&self) -> String {
        let uptime = self
            .started_at
            .map(|started_at| format!("uptime={}s", started_at.elapsed().as_secs()))
            .unwrap_or_else(|| "uptime=not-started".to_string());
        let room_count = self
            .latest_room_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "{uptime}, comments={}, gifts={}, super_chats={}, latest_room_count={room_count}",
            self.comments, self.gifts, self.super_chats
        )
    }
}

danmuji_sdk::export_plugin!(SamplePlugin::default());
