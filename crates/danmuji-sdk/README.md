# danmuji-sdk

Rust SDK abstractions for writing plugins for B站弹幕姬 (`copyliu/bililive_dm`).

This crate provides the Rust-side plugin trait, event models, host API wrapper,
and `export_plugin!` macro. Use it together with `cargo-danmuji` to build a
single `.dll` plugin that can be loaded by B站弹幕姬.
