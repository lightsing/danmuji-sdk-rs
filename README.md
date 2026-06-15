# danmuji-sdk-rs

Rust wrapper for the Bilibili Danmuji .NET plugin SDK.

The Danmuji host loads .NET Framework plugins that inherit `DMPlugin`, so Rust
cannot replace the .NET class directly. This repository provides:

- `crates/cargo-danmuji`: Cargo subcommand that builds and packages plugins.
- `crates/danmuji-sdk`: Rust types, host API, lifecycle/event trait, and export macro.
- `example`: a complete example plugin that handles comments, gifts, SC, and admin summaries.
- `bridge/DanmujiRustBridge`: a small .NET Framework bridge that inherits `DMPlugin`
  and forwards events to the Rust DLL.

## Build

Package the example plugin:

```powershell
cargo run -p cargo-danmuji -- danmuji build `
    --manifest-path Cargo.toml `
    --package danmuji-rust-example-plugin `
    --lib-name danmuji_rust_example_plugin `
    --release `
    --output example\dist\RustSampleDanmujiPlugin.dll
```

The example package is written to `example\dist\RustSampleDanmujiPlugin.dll`.

Normal packaging builds the Rust plugin, uses the bridge template prepared by
`cargo-danmuji`'s build script, appends the Rust native DLL as a PE overlay, and
writes one plugin DLL.

The Danmuji host still requires a .NET Framework plugin DLL at runtime because
it discovers plugins by loading assemblies that inherit `DMPlugin`. Day-to-day
Rust plugin builds do not run MSBuild unless `cargo-danmuji` itself needs to be
rebuilt. After changing C# bridge code, rerun the same command; Cargo will rerun
`cargo-danmuji`'s build script when bridge sources changed. To force that path:

```powershell
cargo clean -p cargo-danmuji
```

External plugin authors should install the cargo subcommand and build from their
own plugin crate:

```powershell
cargo install cargo-danmuji
cargo danmuji new my-plugin
cd my-plugin
cargo danmuji build --release --output dist\MyPlugin.dll
```

## Edit the Rust plugin

Change `example/src/lib.rs`. The key entry points are methods on `DanmujiPlugin`:

- `metadata`
- `start`, `stop`, `admin`, `inited`, `deinit`
- `connected`, `disconnected`
- `room_count`
- `danmaku`

The Rust plugin must export the bridge ABI:

```rust
danmuji_sdk::export_plugin!(SamplePlugin::default());
```

## Deploy

Copy the generated plugin DLL into Danmuji's plugin folder. For the sample
plugin, use:

```text
example\dist\RustSampleDanmujiPlugin.dll
```

`BilibiliDM_PluginFramework.dll` and `Newtonsoft.Json.dll` are not packaged
because Danmuji already provides them in production.

If Danmuji is running as 32-bit, build the Rust DLL with an x86 Windows target
and make sure the .NET bridge is built for x86 as well.
