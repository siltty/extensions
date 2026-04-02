# Siltty Extensions

Official plugin repository for the [Siltty](https://github.com/siltty/siltty) terminal emulator.

## Available Plugins

| Plugin | Description |
|--------|-------------|
| [git-status](plugins/git-status) | Show git branch and status in the status bar |
| [notifications](plugins/notifications) | Command exit status notifications |
| [auto-theme](plugins/auto-theme) | Automatic theme switching |

## Installation

```bash
siltty plugin install git-status
siltty plugin install notifications
```

## Creating a Plugin

See the [Plugin SDK documentation](https://github.com/siltty/siltty/tree/main/crates/siltty-plugin-sdk).

1. Create a new Rust crate with `crate-type = ["cdylib"]`
2. Add `siltty-plugin-sdk` as dependency
3. Implement exported functions (`on_init`, `on_prompt`, `on_timer`, etc.)
4. Build: `cargo build --target wasm32-wasip1 --release`
5. Submit a PR to this repo

## License

Apache-2.0
