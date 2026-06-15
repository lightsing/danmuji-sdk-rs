# danmuji-sdk-rs

面向 [B站弹幕姬](https://github.com/copyliu/bililive_dm)（`bililive_dm`）的 Rust 插件 SDK。

上游插件系统通过 .NET Framework 加载继承 `DMPlugin` 的插件类；本项目保留一个薄的 .NET 桥接层，把弹幕姬事件转发给 Rust `cdylib`，最后打包成一个可直接放进插件目录的 `.dll`。

## 目录

- `crates/danmuji-sdk`：Rust 侧 SDK、事件模型、宿主 API 和 `export_plugin!`。
- `crates/cargo-danmuji`：Cargo 子命令，负责构建 Rust DLL 并打包单文件插件。
- `bridge/DanmujiRustBridge`：继承 `BilibiliDM_PluginFramework.DMPlugin` 的 .NET 桥接模板。
- `example`：仓库内示例插件。

## 构建示例

在仓库根目录执行：

```powershell
cargo run -p cargo-danmuji -- danmuji build `
    --manifest-path Cargo.toml `
    --package danmuji-rust-example-plugin `
    --lib-name danmuji_rust_example_plugin `
    --release `
    --output example\dist\RustSampleDanmujiPlugin.dll
```

输出文件是 `example\dist\RustSampleDanmujiPlugin.dll`，复制到 B站弹幕姬插件目录即可。

`BilibiliDM_PluginFramework.dll` 和 `Newtonsoft.Json.dll` 不会被打包；生产环境中的 B站弹幕姬已经提供它们。

## 独立插件开发

开发者不需要 clone 本仓库。安装 Cargo 子命令后，在自己的插件项目里构建：

```powershell
cargo install cargo-danmuji
cargo danmuji new my-plugin
cd my-plugin
cargo danmuji build --release --output dist\MyPlugin.dll
```

Rust 插件实现 `DanmujiPlugin`，并导出桥接入口：

```rust
danmuji_sdk::export_plugin!(MyPlugin::default());
```

## 构建说明

日常执行 `cargo danmuji build` 只会构建 Rust 插件，并把生成的 native DLL 追加到 .NET 桥接模板末尾，输出单个插件 `.dll`。

.NET 桥接模板由 `cargo-danmuji` 的 `build.rs` 管理；只有安装或重新构建 `cargo-danmuji`、或者桥接源码发生变化时，才需要重新编译 .NET 桥接层。

## 命名约定

- 上游项目：`copyliu/bililive_dm`，文档中称为“B站弹幕姬”。
- 上游插件 SDK：`BilibiliDM_PluginFramework.dll` / `DMPlugin`。
- 本项目：`danmuji-sdk-rs`，Rust 包和命令沿用 `danmuji` 作为 crate/CLI 名称。
