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
.\example\build.ps1 -Configuration Release
```

The packaged files are written to `example\dist\`.

