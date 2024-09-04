# pengu-rust
Challenge of implementing Pengu Core in Rust 🦀

> The choice of programming language for a core library is a matter of preference and practicality.
> The key is to leverage the language's strengths to create a robust, efficient, and maintainable library that seamlessly integrates into the application's ecosystem.

<br />

<p align="center">
  <img src="https://github.com/user-attachments/assets/3ef6b79a-9b27-41bc-b5f5-fc6e05718d7c" width="200" />
</p>

## Why Rust?

- UTF-8 strings
- Filesystem support
- C++ FFI & bindgen
- Mono codebase for cross-platform

## Achievements

- [x] Bootstrapper
  - [x] IFEO mode
  - [x] Symlink mode
- [x] Handmade hooks
- [ ] Plugin assets
- [ ] Custom imports
- [ ] Preload scripts
- [ ] Basic user plugins
- [ ] Cross-platform
  - [x] Windows
  - [ ] macOS 

## Development

**Your pull-requests are welcome!**

Refer to Pengu Loader source code.

[Rust Bindgen](https://github.com/rust-lang/rust-bindgen) is a good thing to generate bindings from C++ headers, see the [`build.rs`](build/build.rs).

A new Pengu docs will release soon to help you to understand the architecture and how it works.

### Building

```
cargo build
```

To build in production mode, just add `--release` to the command above.

### Installing

After your successful build, you will see the `core.dll` in the folder `target/debug` or `target/release` according to your build mode above.
This DLL is compatible with the current Pengu Loader app, so you just replace it in the app folder.

If you prefer to manual install using IFEO method, run this command in elevated `cmd`.

```bat
set "dval=rundll32 \"path\to\core.dll\", #6000"
reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\LeagueClientUx.exe" /v Debugger /t REG_SZ "%dval%"
```

### Debugging

Use `lldb` or VSCode extension to attach to the running `LeagueClientUx.exe` or `LeagueClientUxRenderer.exe`.
