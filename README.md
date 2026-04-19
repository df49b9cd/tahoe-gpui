<p align="center">
  <img src="branding/tahoe-gpui-logo.svg" alt="tahoe-gpui" width="160" />
</p>

<h1 align="center">tahoe-gpui</h1>

<p align="center">
  <strong>Human Interface Guidelines components for GPUI.</strong>
</p>

<p align="center">
  <a href="https://github.com/df49b9cd/tahoe-gpui/actions/workflows/ci.yml"><img src="https://github.com/df49b9cd/tahoe-gpui/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="Apache-2.0" /></a>
  <img src="https://img.shields.io/badge/rust-1.94.1%2B-orange" alt="rust 1.94.1+" />
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="macOS" />
</p>

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
