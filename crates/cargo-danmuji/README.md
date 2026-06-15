# cargo-danmuji

Cargo subcommand for building Rust plugins for B站弹幕姬 (`copyliu/bililive_dm`).

`cargo danmuji build` builds the Rust `cdylib`, resolves the upstream
`BilibiliDM_PluginFramework` SDK version from `.danmuji-version` or
`--sdk-version`, generates a cached .NET bridge DLL, and packages everything
into a single plugin `.dll`.

Install:

```powershell
cargo install cargo-danmuji
```
