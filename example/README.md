# 示例插件

`example` 是 `danmuji-sdk-rs` 的仓库内示例插件，演示插件元信息、生命周期回调、`log` / `add_dm` 宿主调用，以及弹幕、礼物、SC、互动、开播和下播事件处理。

在仓库根目录构建：

```powershell
cargo run -p cargo-danmuji -- danmuji build `
    --manifest-path Cargo.toml `
    --package danmuji-rust-example-plugin `
    --lib-name danmuji_rust_example_plugin `
    --release `
    --output example\dist\RustSampleDanmujiPlugin.dll
```

输出文件是 `example\dist\RustSampleDanmujiPlugin.dll`。它已经是单文件 B站弹幕姬插件，不需要额外携带 `BilibiliDM_PluginFramework.dll` 或 `Newtonsoft.Json.dll`。

这个命令只构建 Rust 示例插件，并复用 `cargo-danmuji` 的 .NET 桥接模板；通常不需要手动编译桥接层。
