# danmuji-sdk-rs

Rust wrapper for the Bilibili Danmuji .NET plugin SDK.

The Danmuji host loads .NET Framework plugins that inherit `DMPlugin`, so Rust
cannot replace the .NET class directly. This repository provides:

- `crates/danmuji-sdk`: Rust types, host API, lifecycle/event trait, and export macro.
- `example`: a complete example plugin that handles comments, gifts, SC, and admin summaries.
- `bridge/DanmujiRustBridge`: a small .NET Framework bridge that inherits `DMPlugin`
  and forwards events to the Rust DLL.

## Build

Package the example plugin:

```powershell
.\example\build.ps1 -Configuration Release
```

The example package is written to `example\dist\`.

Normal packaging only builds the Rust plugin and uses the prebuilt .NET bridge
from `bridge/prebuilt`.

The Danmuji host still requires a .NET Framework plugin DLL at runtime because
it discovers plugins by loading assemblies that inherit `DMPlugin`. The bridge
DLL is stable and already checked in, so day-to-day Rust plugin builds do not
need the .NET SDK. Rebuild the bridge only after changing C# bridge code:

```powershell
.\example\build.ps1 -Configuration Release -RebuildBridge
```

The root build script is a generic packager. Custom plugin projects should call
it with an explicit package and native library name:

```powershell
.\build.ps1 `
    -Configuration Release `
    -RustPackage "danmuji-rust-example-plugin" `
    -RustLibraryName "danmuji_rust_example_plugin" `
    -OutputDirectory "example\dist"
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

Copy the files from the package output directory into Danmuji's plugin folder.
For the sample plugin, use `example\dist\`:

- `DanmujiRustBridge.dll`
- `danmuji_rust_plugin.dll`
- `BilibiliDM_PluginFramework.dll`
- `Newtonsoft.Json.dll`

If Danmuji is running as 32-bit, build the Rust DLL with an x86 Windows target
and make sure the .NET bridge is built for x86 as well.
