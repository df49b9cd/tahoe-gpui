You are an experienced, pragmatic software engineering AI agent. Do not over-engineer a solution when a simple one is possible. Keep edits minimal. If you want an exception to ANY rule, you MUST stop and get permission first.

# Project Overview

`tahoe-gpui` is a component library that brings Apple's macOS Tahoe HIG (San Francisco typography, SF Symbols, Liquid Glass materials, Dynamic Type, spring motion) to applications built on [Zed's GPUI framework](https://github.com/zed-industries/zed/tree/main/crates/gpui). Standalone — no AI SDK dependency required.

- **Language**: Rust 2024 edition, MSRV 1.94.1 (clippy lints against 1.87.0 for forward compatibility)
- **License**: Apache-2.0
- **Target platform**: macOS first-class; Linux/Windows track GPUI's support.
- **Test runner**: `cargo nextest` (never plain `cargo test`).
- **Renderer**: GPUI `SvgRenderer` + `pulldown-cmark` + `tree-sitter` + `mermaid-rs-renderer`.

# Reference

## Crate Architecture

```
crates/
├── tahoe-gpui/          # HIG UI components (the library)
│   ├── foundations/     # Design-system primitives
│   │   ├── color.rs     # SystemPalette, SystemColor, Appearance
│   │   ├── theme.rs     # TahoeTheme (global design tokens)
│   │   ├── typography.rs# TextStyle, FontDesign, Dynamic Type
│   │   ├── icons/       # SF Symbols: Icon, IconName, AnimatedIcon
│   │   ├── materials.rs # Liquid Glass: glass_surface(), GlassStyle
│   │   ├── layout.rs    # MIN_TOUCH_TARGET, margins, LayoutDirection
│   │   ├── motion.rs    # SpringAnimation, MotionTokens
│   │   └── accessibility.rs
│   ├── components/      # HIG-organized UI controls (8 subcategories)
│   │   ├── content/
│   │   ├── menus_and_actions/
│   │   ├── navigation_and_search/
│   │   ├── presentation/
│   │   ├── selection_and_input/
│   │   ├── layout_and_organization/
│   │   ├── status/
│   │   └── system_experiences/
│   ├── markdown/, code/, context/
│   ├── workflow/, voice/
│   └── citation.rs, text_actions.rs
│
└── remend/              # Streaming Markdown preprocessor
                         # Auto-completes incomplete syntax during
                         # token-by-token streaming.
```

## Optional Features

- `voice` — Audio/speech components (requires `cpal`).
- `test-support` — GPUI test harness; pulls `image` for visual regression.

## Component Patterns

- **Stateful** (`Entity<T>` where `T: Render`) — mutable state, use `cx.notify()` to re-render. Examples: `TextField`, `StreamingMarkdown`.
- **Stateless** (`#[derive(IntoElement)]` + `RenderOnce`) — builder-pattern components. Examples: `Button`, `Alert`, `Badge`.

## Theme System

`TahoeTheme` is a GPUI global. Register once before rendering:

```rust
cx.set_global(TahoeTheme::dark()); // or ::light(), ::liquid_glass()
```

Components read tokens via `cx.global::<TahoeTheme>()`. For runtime switching, `theme.apply(cx)` calls `cx.refresh_windows()`.

# Essential Commands

```bash
# Build the entire workspace
cargo build

# Build a single crate
cargo build -p tahoe-gpui

# Format
cargo fmt
cargo fmt --check

# Lint (zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
cargo nextest run

# Single-crate tests
cargo nextest run -p tahoe-gpui
cargo nextest run -p remend

# Dependency audit
cargo deny check

# Examples
cargo run -p tahoe-gpui --example component_gallery
cargo run -p tahoe-gpui --example liquid_glass_gallery
cargo run -p tahoe-gpui --example dashboard_app
cargo run -p tahoe-gpui --example voice_demo --features voice
```

# Patterns

## Adding a New HIG Component

1. Identify which HIG subcategory it belongs to (`content/`, `menus_and_actions/`, etc.) and add a file there.
2. Stateless builder (`#[derive(IntoElement)]` + `RenderOnce`) unless the component owns mutable state.
3. Read design tokens from `cx.global::<TahoeTheme>()` rather than hardcoding colors/metrics.
4. Respect `MIN_TOUCH_TARGET` for interactive elements.
5. Add SF Symbol support via `foundations::icons::Icon`.
6. Add a `gallery` entry in the nearest `*_gallery.rs` example.
7. Add unit tests (and visual-regression goldens when appropriate via `test-support`).

## Testing Patterns

- **Unit tests**: `#[cfg(test)] mod tests` at the bottom of the source file.
- **GPUI tests**: `use core::prelude::v1::test;` at the top of the test module to override gpui's `#[test]` macro.
- **Property tests**: `proptest!` macro (see `remend` and streaming-Markdown tests).
- **Visual regression**: `tahoe-gpui` `test-support` feature renders components to `RenderImage` bitmaps and diffs against goldens.
- **No glob imports in tests** — always spell out `use crate::foo::{Bar, Baz};`.

## Decomposition Pattern

Large files (>500 lines) should be decomposed into directory modules:

- `foo.rs` → `foo/` with `mod.rs` (re-exports) + per-variant files.
- `mod.rs` must `pub use` all items to preserve the public API surface.

## Unsafe Code

All `unsafe` blocks must use `// SAFETY:` comment convention with a multi-line justification:

```rust
// SAFETY: The buffer `buf` contains only:
// 1. ASCII string literals: `b"data: "` and `b"\n\n"` (always valid UTF-8)
// 2. Output of `serde_json::to_writer`, which produces valid UTF-8 by contract
// Therefore the entire buffer is valid UTF-8 and `from_utf8_unchecked` is sound.
```

# Anti-patterns

- **Don't use `cargo test`** — use `cargo nextest run`. The workspace's `.config/nextest.toml` tunes parallelism and retries.
- **Don't use glob imports in tests** — always explicit `use` statements.
- **Don't hardcode colors/metrics** — read them from `TahoeTheme` tokens so light/dark/liquid-glass variants stay consistent.
- **Don't use `unwrap()` in production code** — use `.expect("reason")` or propagate errors.
- **Don't skip `MIN_TOUCH_TARGET`** for interactive controls — HIG compliance requires 44 pt minimum.
- **Don't use `// Safety:` comment style** — use `// SAFETY:` (uppercase, colon suffix).

# Code Style

- **Formatter**: `rustfmt` with edition 2024, `max_width 100`, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`.
- **Linting**: clippy with MSRV 1.87.0 (intentionally lower than the workspace `rust-version` so clippy flags suggestions that would require newer Rust), `too-many-arguments-threshold = 8`, `type-complexity-threshold = 300`.
- **Documentation**: public items on `tahoe-gpui` should carry doc comments; crate-level docs should describe the HIG mapping.

# Commit and Pull Request Guidelines

## Before Committing

1. `cargo fmt` — format all code.
2. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
3. `cargo nextest run` — all tests pass.

## Commit Message Conventions

Conventional commits with scope:

```
feat(button): add plain variant for toolbar contexts
fix(markdown): buffer incomplete UTF-8 across stream chunks
refactor(theme): extract TahoeTheme tokens into foundations module
test(alert): cover destructive action-sheet variant
docs(hig): sync Presentation page against the Apr 2026 HIG revision
perf(code): cache tree-sitter highlight grammars across renders
```

Common scopes: `foundations`, `components`, `markdown`, `code`, `theme`, `materials`, `voice`, `workflow`, `remend`.

## PR Description Requirements

- List of changes with scope.
- Breaking API changes called out explicitly (e.g., `BREAKING: TextField::with_prompt renamed to TextField::placeholder`).
- Test coverage notes for new/changed code.
- Reference to the HIG section or issue number when relevant.
