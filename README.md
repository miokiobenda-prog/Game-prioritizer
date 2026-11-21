# Rust Game Booster

Created and maintained by [@miokiobenda-prog](https://github.com/miokiobenda-prog).

A small Windows game launcher built in Rust. Add game `.exe` files, then choose **Boost & launch**. The launched child process receives Windows' high-priority class; this is temporary and ends automatically when that game exits. The app does not stop services, edit the registry, or alter power plans.

## Build

```powershell
cargo build --release
```

The resulting executable is `target\release\rust-game-booster.exe`.
