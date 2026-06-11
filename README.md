# danmuji-sdk-rs

Rust wrapper for the Bilibili Danmuji .NET plugin SDK.

The Danmuji host loads .NET Framework plugins that inherit `DMPlugin`, so Rust
cannot replace the .NET class directly. This repository provides:

- `crates/danmuji-sdk`: Rust types, host API, lifecycle/event trait, and export macro.
- `plugin`: the default Rust `cdylib` plugin target.
- `example`: a fuller example plugin that handles comments, gifts, SC, and admin summaries.
- `bridge/DanmujiRustBridge`: a small .NET Framework bridge that inherits `DMPlugin`
  and forwards events to the Rust DLL.

## Build

```powershell
.\build.ps1 -Configuration Release
```

The packaged plugin files are written to `dist/`.

To package the example plugin:

```powershell
.\example\build.ps1 -Configuration Release
```

The example package is written to `example\dist\`.

## Edit the Rust plugin

Change `plugin/src/lib.rs`. The key entry points are methods on `DanmujiPlugin`:

- `metadata`
- `start`, `stop`, `admin`, `inited`, `deinit`
- `connected`, `disconnected`
- `room_count`
- `danmaku`

The Rust plugin must export the bridge ABI:

```rust
danmuji_sdk::export_plugin!(ExamplePlugin::default());
```

## Deploy

Copy the files from `dist/` into Danmuji's plugin folder:

- `DanmujiRustBridge.dll`
- `danmuji_rust_plugin.dll`
- `BilibiliDM_PluginFramework.dll`
- `Newtonsoft.Json.dll`

If Danmuji is running as 32-bit, build the Rust DLL with an x86 Windows target
and make sure the .NET bridge is built for x86 as well.
