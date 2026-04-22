# Contributing to tahoe-gpui

Thanks for your interest in contributing! This document covers the development workflow, style, and conventions.

## Prerequisites

- **Rust** ≥ 1.95.0 (2024 edition). Install via [rustup][rustup].
- **cargo-nextest** for running the test suite — `cargo install cargo-nextest`.
- **cargo-deny** (optional, for dependency audits) — `cargo install cargo-deny`.
- macOS 14+ is recommended for running the examples (GPUI's platform support is most mature on Apple Silicon).

## Repository layout

```
tahoe-gpui/
├── crates/
│   ├── tahoe-gpui/   # Main UI component library
│   └── mdstitch/     # Streaming Markdown preprocessor (internal)
├── rustfmt.toml      # Formatter config (edition 2024, max_width 100)
├── clippy.toml       # Linter config (MSRV 1.95.0 — see AGENTS.md)
├── deny.toml         # Dependency audit policy
├── .config/
│   └── nextest.toml  # Test runner profiles (default, ci)
└── .cargo/
    └── config.toml   # Build config (-jobs=-1)
```

## Build and test

```bash
# Build everything
cargo build

# Run tests (use nextest — not `cargo test`)
cargo nextest run

# Specific crate
cargo nextest run -p tahoe-gpui

# Lint (zero warnings required)
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt
cargo fmt --check

# Dependency audit
cargo deny check
```

## Running examples

```bash
cargo run -p tahoe-gpui --example component_gallery
cargo run -p tahoe-gpui --example liquid_glass_gallery
cargo run -p tahoe-gpui --example voice_demo --features voice
```

See [`crates/tahoe-gpui/CLAUDE.md`](crates/tahoe-gpui/CLAUDE.md) for the full example list.

## Code style

- **Formatter**: `rustfmt` with edition 2024, `max_width = 100`, `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`.
- **Linting**: clippy with MSRV 1.95.0 (matches the workspace `rust-version`). No warnings allowed on CI.
- **No glob imports in tests** — always spell out `use crate::foo::{Bar, Baz};` rather than `use super::*;`.
- **Comments**: default to none. Only add one when the *why* is non-obvious (hidden constraint, workaround, surprising behavior).
- **Unsafe code**: every `unsafe` block must carry a multi-line `// SAFETY:` justification (uppercase, colon suffix).

## Testing conventions

- **Test runner**: always use `cargo nextest run`, never plain `cargo test`.
- **Unit tests** live at the bottom of the source file inside `#[cfg(test)] mod tests`.
- **Property tests** use the `proptest` crate.
- **GPUI tests** need `use core::prelude::v1::test;` at the top of the test module to override gpui's `#[test]` macro.
- **Visual regression** helpers (`tahoe-gpui` `test-support` feature) compare rendered bitmaps against golden images via the `image` crate.

## Commit messages

[Conventional Commits][conv] with a scope:

```
feat(button): add plain variant for toolbar contexts
fix(markdown): buffer incomplete UTF-8 across stream chunks
refactor(theme): extract TahoeTheme tokens into foundations module
test(alert): cover destructive action-sheet variant
docs(hig): sync Presentation page against the Apr 2026 HIG revision
```

Common scopes: `foundations`, `components`, `markdown`, `code`, `theme`, `materials`, `voice`, `workflow`, `mdstitch`.

## Pull requests

- Describe *why*, not just *what* — link to the HIG section or issue when relevant.
- Call out breaking API changes explicitly (`BREAKING:` prefix in the PR body).
- Include test coverage for new or changed behavior.
- Ensure `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo nextest run` all pass locally before requesting review.

## License

By contributing, you agree that your contributions will be licensed under the [Apache-2.0 License](LICENSE).

[rustup]: https://rustup.rs
[conv]: https://www.conventionalcommits.org
