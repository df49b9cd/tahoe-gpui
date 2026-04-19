# Bundled icon assets

All SVGs in `assets/icons/` and `assets/icons-glass/` are original work
authored for `tahoe-gpui` and are licensed under Apache-2.0 (the same
licence as the crate itself; see `LICENSE` at the workspace root).

## Visual language

Glyphs in `assets/icons/symbols/` follow Apple's Human Interface
Guidelines for system symbols:

* 24×24 view-box.
* `currentColor` fill or stroke, so GPUI's `text_color` tinting flows
  through.
* Stroked variants use `stroke-width="1.75"` with rounded caps and joins,
  giving a typographic weight comparable to SF Symbols Regular.
* Filled variants (`*-fill.svg`, `play-fill`, `pause-fill`, the
  disclosure triangles, etc.) mirror SF's `.fill` modifier.

The set is original — no Apple SF Symbols path data is included or
redistributed. `IconName::system_name()` still returns the matching
SF Symbol identifier (`"checkmark"`, `"play.fill"`, …) so consumers
shipping on macOS can pass it to `NSImage(systemSymbolName:)` for
native rendering when desired.

## Domain icon sets

* `assets/icons/languages/` — programming-language marks.
* `assets/icons/providers/` — LLM provider marks (Anthropic, OpenAI,
  Google, etc.).
* `assets/icons/git/`, `assets/icons/dev-tools/` — multi-colour glyphs
  used by the AI/devops surfaces.
* `assets/icons-glass/` — Liquid Glass variants of the same domain
  icons (heavier strokes for frosted-glass tiles); the `symbols/` set
  is theme-invariant and serves both standard and glass themes.

All domain icons are original Apache-2.0 work as well.
