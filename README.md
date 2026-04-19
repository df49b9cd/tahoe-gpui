# tahoe-gpui

**Human Interface Guidelines components for GPUI.**

A component library that brings Apple's macOS Tahoe HIG (San Francisco typography, SF Symbols, Liquid Glass materials, Dynamic Type, spring motion) to applications built on [Zed's GPUI framework][gpui]. Standalone — no AI SDK dependency required.

## Crates

| Crate | Description |
| --- | --- |
| [`tahoe-gpui`](crates/tahoe-gpui) | HIG-aligned UI components (buttons, alerts, toolbars, streaming markdown, …). |
| [`remend`](crates/remend) | Streaming Markdown preprocessor that auto-completes incomplete syntax during token-by-token rendering. Used internally by `tahoe-gpui`. |

## Quick start

```toml
[dependencies]
tahoe-gpui = { git = "https://github.com/df49b9cd/tahoe-gpui" }
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.231.1-pre" }
```

```rust
use gpui::App;
use tahoe_gpui::TahoeTheme;

fn main() {
    App::new().run(|cx| {
        cx.set_global(TahoeTheme::dark());
        // … open a window, compose components
    });
}
```

To integrate with the [rust-ai-sdk](https://github.com/df49b9cd/ai-sdk-rust), use the companion [`rust-ai-elements`](https://github.com/df49b9cd/ai-sdk-rust/tree/main/crates/rust-ai-elements) crate — it re-exports `tahoe-gpui` and adds streaming chatbot UI that binds to `rust-ai-sdk-provider` stream parts.

See [`crates/tahoe-gpui/examples`](crates/tahoe-gpui/examples) for runnable demos:

```bash
cargo run -p tahoe-gpui --example component_gallery
cargo run -p tahoe-gpui --example liquid_glass_gallery
cargo run -p tahoe-gpui --example dashboard_app
```

## Features

- `voice` — Audio/speech components (requires `cpal`).
- `test-support` — GPUI test harness with `image`-based visual regression helpers.

## Building

```bash
cargo build -p tahoe-gpui
cargo nextest run -p tahoe-gpui
cargo clippy -p tahoe-gpui -- -D warnings
cargo fmt --check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow.

## License

Apache-2.0. See [LICENSE](LICENSE).

[gpui]: https://github.com/zed-industries/zed/tree/main/crates/gpui
