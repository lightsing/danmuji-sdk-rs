# 示例插件

`example` 是 `danmuji-sdk-rs` 的仓库内示例插件，演示插件元信息、生命周期回调、`log` / `add_dm` 宿主调用，以及弹幕、礼物、SC、互动、开播和下播事件处理。

先安装 `cargo-danmuji`：

```powershell
cargo install cargo-danmuji
```

在 `example` 目录构建：

```powershell
cargo danmuji build --release --output dist\RustSampleDanmujiPlugin.dll
```

输出文件是 `dist\RustSampleDanmujiPlugin.dll`。它已经是单文件 B站弹幕姬插件，不需要额外携带 `BilibiliDM_PluginFramework.dll` 或 `Newtonsoft.Json.dll`。

这个命令会读取仓库根目录的 `.danmuji-version`，由 `cargo-danmuji` 在临时缓存目录里拉取对应上游版本并生成 .NET 桥接 DLL。如果项目没有 `.danmuji-version`，首次构建会自动查询上游最新 tag 并创建它。首次构建会编译桥接层，之后会复用缓存。
