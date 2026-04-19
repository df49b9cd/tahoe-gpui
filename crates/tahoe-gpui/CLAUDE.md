# tahoe-gpui

Crate-local notes. See the workspace [AGENTS.md](../../AGENTS.md) for the full architecture, essential commands, and code-style rules.

## Run Examples

```bash
# Component galleries
cargo run -p tahoe-gpui --example component_gallery
cargo run -p tahoe-gpui --example button_gallery
cargo run -p tahoe-gpui --example alert_gallery
cargo run -p tahoe-gpui --example icon_gallery
cargo run -p tahoe-gpui --example liquid_glass_gallery
cargo run -p tahoe-gpui --example liquid_glass_interactive
cargo run -p tahoe-gpui --example theme_gallery
cargo run -p tahoe-gpui --example code_gallery

# Streaming + workflow
cargo run -p tahoe-gpui --example streaming
cargo run -p tahoe-gpui --example workflow_demo
cargo run -p tahoe-gpui --example window_layouts
cargo run -p tahoe-gpui --example context

# Voice (requires `voice` feature + `cpal`)
cargo run -p tahoe-gpui --example voice_demo --features voice

# macOS Tahoe UI-kit screen mockups
cargo run -p tahoe-gpui --example auth_app
cargo run -p tahoe-gpui --example music_app
cargo run -p tahoe-gpui --example list_app
cargo run -p tahoe-gpui --example dashboard_app
cargo run -p tahoe-gpui --example toolbar_app
```

## Crate-local conventions

- Use `cx.listener()` for action handlers on entity-owned components.
- Use `cx.processor()` for `uniform_list` / `list` item rendering callbacks.
- Tests override gpui's `#[test]` macro with `use core::prelude::v1::test;` at the top of the test module.
