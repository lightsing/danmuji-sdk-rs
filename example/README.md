# Example Plugin

This is a complete Rust plugin example for `danmuji-sdk-rs`.

It demonstrates:

- plugin metadata
- lifecycle callbacks
- host calls through `log` and `add_dm`
- comment, gift, Super Chat, interact, live start, and live end events
- simple in-memory statistics shown from `Admin`

Build and package it from the repository root:

```powershell
cargo run -p cargo-danmuji -- danmuji build `
    --manifest-path Cargo.toml `
    --package danmuji-rust-example-plugin `
    --lib-name danmuji_rust_example_plugin `
    --release `
    --output example\dist\RustSampleDanmujiPlugin.dll
```

The packaged single-file plugin is written to `example\dist\RustSampleDanmujiPlugin.dll`.

This command only builds the Rust example and uses the bridge template prepared
by `cargo-danmuji`'s build script. Run `cargo clean -p cargo-danmuji` only when
you need to force a bridge template rebuild.
