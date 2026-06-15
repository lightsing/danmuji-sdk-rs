# danmuji-sdk-rs

面向 [B站弹幕姬](https://github.com/copyliu/bililive_dm)（`bililive_dm`）的 Rust 插件 SDK。

上游插件系统通过 .NET Framework 加载继承 `DMPlugin` 的插件类；本项目保留一个薄的 .NET 桥接层，把弹幕姬事件转发给 Rust `cdylib`，最后打包成一个可直接放进插件目录的 `.dll`。

## 目录

- `crates/danmuji-sdk`：Rust 侧 SDK、事件模型、宿主 API 和 `export_plugin!`。
- `crates/cargo-danmuji`：Cargo 子命令，负责选择上游 SDK 版本、生成桥接 DLL、构建 Rust DLL 并打包单文件插件。
- `crates/cargo-danmuji/bridge`：继承 `BilibiliDM_PluginFramework.DMPlugin` 的 C# 桥接源码模板。
- `example`：仓库内示例插件。

## 安装

从 crates.io 安装 Cargo 子命令：

```powershell
cargo install cargo-danmuji
```

Rust 插件项目依赖：

```toml
[dependencies]
danmuji-sdk = "0.1"
```

## 构建示例

构建仓库内示例插件：

```powershell
cd example
cargo danmuji build --release --output dist\RustSampleDanmujiPlugin.dll
```

输出文件是 `dist\RustSampleDanmujiPlugin.dll`，复制到 B站弹幕姬插件目录即可。

`BilibiliDM_PluginFramework.dll` 和 `Newtonsoft.Json.dll` 不会被打包；生产环境中的 B站弹幕姬已经提供它们。

示例仓库根目录的 `.danmuji-version` 固定了上游 SDK 版本。也可以在命令行覆盖：

```powershell
cargo danmuji build --sdk-version 1.1.1.132 --release --output dist\MyPlugin.dll
```

升级到上游最新版本：

```powershell
cargo danmuji upgrade
```

## 独立插件开发

开发者不需要 clone 本仓库。在自己的插件项目里构建：

```powershell
cargo danmuji new my-plugin
cd my-plugin
cargo danmuji build --release --output dist\MyPlugin.dll
```

如果项目里还没有 `.danmuji-version`，第一次 `cargo danmuji build` 会查询上游最新数字 tag，写入 `.danmuji-version`，然后用这个版本构建。后续构建会复用这个锁定版本。`cargo danmuji new --sdk-version <版本>` 也可以在创建项目时直接写入版本文件。

手写项目时，只要引入 `danmuji-sdk`，执行 `cargo danmuji build` 就能完成 Rust 构建和桥接打包。

Rust 插件实现 `DanmujiPlugin`，并导出桥接入口：

```rust
danmuji_sdk::export_plugin!(MyPlugin::default());
```

## 构建说明

日常执行 `cargo danmuji build` 会先构建 Rust 插件，再按 SDK 版本准备 .NET 桥接 DLL，并把 Rust native DLL 追加到桥接 DLL 末尾，输出单个插件 `.dll`。

桥接 DLL 由 `cargo-danmuji` 在临时缓存目录生成：

1. 优先使用 `--sdk-version`；否则读取 `.danmuji-version`；如果文件不存在，就查询上游最新数字 tag 并创建 `.danmuji-version`。
2. 把对应 tag、branch 或 commit 拉到缓存目录。
3. 在缓存目录生成 SDK-style C# 工程，编译 `BilibiliDM_PluginFramework` 引用程序集和 Rust 桥接 DLL。
4. 按 SDK 版本、上游 commit 和桥接源码 hash 复用缓存。

需要重新拉取分支或重建缓存时使用 `--refresh-sdk`。需要把 `.danmuji-version` 更新到最新 tag 时使用 `cargo danmuji upgrade`。缓存根目录默认在系统临时目录下，也可以用 `DANMUJI_CACHE_DIR` 指定。
