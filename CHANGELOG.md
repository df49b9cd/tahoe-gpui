# Changelog

All notable changes to `tahoe-gpui` land here. Format loosely tracks
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning will
follow SemVer once the crate reaches 1.0.

## [Unreleased]

### Changed — rendering

- `glass_blur_surface`, `glass_lens_surface`, `backdrop_overlay`, and
  `backdrop_blur_overlay` now paint real dual-Kawase backdrop blur and
  Liquid Glass lens composites via GPUI's new `Window::paint_blur_rect` /
  `paint_lens_rect` entry points. Previous releases fell back to a
  translucent tinted fill + shadows. Each blur/lens primitive forces a
  render-pass break — prefer one primitive per glass surface; do not use
  them for per-list-row backgrounds. Until the upstream Zed PR merges,
  `crates/tahoe-gpui/Cargo.toml` tracks the `df49b9cd/zed` fork's
  `tahoe-gpui/blur-primitive` branch
  ([df49b9cd/zed#1](https://github.com/df49b9cd/zed/pull/1)); cargo
  resolves it over the network, so no local fork checkout is required.

- **Breaking (signature)** — `glass_blur_surface` and `glass_lens_surface`
  no longer take an `el: Div` parameter. Each returns a `.relative()`
  wrapper whose first child is the blur/lens canvas; callers attach
  content by chaining `.child(...)` on the return, which paints on top
  of the blur. The previous shape silently blurred any pre-existing
  children of `el`. `backdrop_overlay` / `backdrop_blur_overlay` are
  unchanged.

- `LensEffect::refraction` and `LensEffect::dispersion` are now correctly
  denormalized from the documented 0.0..1.0 scale to GPUI's raw 0..100
  Figma scale at the API boundary (via `From<&LensEffect> for
  gpui::LensEffect`). Before this change, the HIG-default Liquid Glass
  lens was rendering at ~1% refraction strength. `LensEffect::light_angle`
  is similarly converted from degrees to radians at the boundary.

### Fixed

- **`IconStyle::Auto` now consults surface scope instead of always resolving to
  `LiquidGlass`** (issue
  [#13](https://github.com/df49b9cd/tahoe-gpui/issues/13)). Previously every
  `Icon::new(...)` — the default — rendered with glass pastel colors and a
  1.5pt stroke regardless of whether the icon sat on a Liquid Glass surface.
  Per HIG §Materials (`crates/tahoe-gpui/docs/hig/foundations.md:1045`), vibrancy is applied by
  the *surface*, not by a global theme mode, so the fix threads "this subtree
  is on glass" through the element tree via a new
  [`foundations::GlassSurfaceScope`] wrapper. `IconStyle::Auto` now resolves
  to `Standard` outside a glass scope and `LiquidGlass` inside one.

### Added

- `foundations::surface_scope` module exposing `GlassSurfaceScope<E>`, a
  minimal custom `Element` that marks its child subtree as a Liquid Glass
  surface, plus `GlassSurfaceGuard` (RAII, useful for custom `Element`
  implementations that need to re-establish scope across GPUI's deferred-draw
  boundary) and `is_on_glass_surface()` for direct queries.
- `ButtonVariant::is_glass_surfaced()` — predicate used by `Button` to decide
  whether to wrap itself in a `GlassSurfaceScope`. Centralises the membership
  test so future glass variants only need to update this one method.
- Reduce Transparency accessibility fallback: `Icon` and `GlassIconTile`
  now drop from `theme.glass.icon_*` to `theme.text_muted` when
  `accessibility_mode.reduce_transparency()` is set, matching the opaque
  fallback fill that `glass_surface()` already swaps in.
- `TextView` expanded from a minimal read-only display into a HIG-aligned
  component with an extensive builder surface — see the `TextView` rustdoc
  for the full list (color / typography / layout / decoration / truncation /
  accessibility / selection / focus groups).
- `TextView::styled_text(text, highlights)` — accepts a plain-text
  `SharedString` alongside `Vec<(Range<usize>, HighlightStyle)>` so
  VoiceOver has a label to announce without callers needing to restate
  the content via `.accessibility_label(...)`. Highlights that fall
  outside the text (start or end beyond `text.len()`, or a reversed
  range) panic in all builds so mismatches surface loudly rather than
  silently rendering truncated runs at runtime.
- `components::content::selectable_text::SelectionCoordinator` trait
  and `components::content::text_view::TextViewSelection` struct —
  the single-paragraph coordinator that `TextView` feeds to the shared
  `SelectableText` primitive. Exposed so host apps can programmatically
  clear the selection or query its state (e.g. to toggle Copy menu
  items).
- `TextView` keyboard scroll (scrollable views only): Up / Down move by one
  rendered line-height, Page Up / Page Down move by one viewport height,
  Home jumps to the start, End jumps to the end. Bound inside the new
  `TEXT_VIEW_CONTEXT` key scope so the raw keys only fire when a focused
  `TextView` owns the dispatch path — they do not leak to the global scope
  where Up / Down drive tab switches, menu cursors, or sliders.
- `TextView` right-click context menu: a two-item menu (Copy + Select All)
  backed by the existing [`ContextMenu`] component. Copy renders disabled
  when the selection is empty; Select All is always enabled. Activation
  dispatches the same `text_editing::{Copy, SelectAll}` actions a keyboard
  shortcut fires, so click, ⌘-key, and the (future) command palette all
  share one handler. Selectable views only — non-selectable `TextView`s
  suppress the menu since they have no Copy / Select All semantics.
- `tahoe_gpui::text_actions::{Up, Down, PageUp, PageDown}` actions in the
  `text_editing` namespace. `TextView` binds them for scroll; future
  multi-line text editors may rebind them for cursor movement.
- `foundations::theme::LabelLevel` — HIG semantic-tier enum (`Primary` /
  `Secondary` / `Tertiary` / `Quaternary` / `Quinary`, the last added in
  macOS Tahoe) resolving to the matching theme colour via
  [`TextView::label_level`]. Lives in `foundations::typography` and is
  re-exported through `foundations::theme` and `tahoe_gpui::prelude`.
- `foundations::accessibility::AccessibilityProps::disabled: bool` + a
  `.disabled(bool)` builder. Interactive components (Toggle, Checkbox,
  Stepper, SegmentedControl, workflow controls) set it so VoiceOver will
  announce the dimmed state once GPUI lands an AX tree.

### Changed

- `TahoeTheme.avatar_size` default raised from `px(28.0)` to `px(32.0)`
  so the token matches the HIG `AvatarSize::Standard` baseline (the
  canonical default most app surfaces use). Callers that set a size
  explicitly via `Avatar::size(...)` / `Avatar::canonical_size(...)` are
  unaffected; only `Avatar::new("…")` without a size override picks up
  the new default.
- `TextView::max_lines(...)` and `TextView::scrollable(...)` are mutually
  exclusive: clamped height short-circuits GPUI's scroll viewport.
  Setting both panics so the conflict is caught in all builds; release
  builds silently prefer `max_lines`.
- `GlassIconTile` now declares its own glass surface scope via
  `GlassSurfaceScope`. The redundant explicit `.style(IconStyle::LiquidGlass)`
  on its inner `Icon` was removed — the scope drives resolution now.
- `Button` with `ButtonVariant::Glass` / `GlassProminent` wraps its output in a
  `GlassSurfaceScope` so icon children automatically render with glass
  vibrancy.
- **Breaking (signature)** — `IconStyle::resolve` no longer takes a
  `&TahoeTheme` argument, and `Icon::resolved_stroke_width` no longer takes one
  either. Both derive their result from the surface scope, not the theme.
  Call sites inside this crate and the two in-repo examples have been updated.
- **Breaking (signature)** — `TextView::new(text)` → `TextView::new(cx, text)`.
  `TextView` is now a stateful `Entity<Self>: Render + Focusable` instead of
  a `RenderOnce` element, so construction goes through
  `cx.new(|cx| TextView::new(cx, "…").…)`. It owns a `FocusHandle`, a
  `TextViewSelection` coordinator (drag-select, double-click word,
  triple-click paragraph, shift-click extend, ⌘A, ⌘C), and — for scrollable
  views — a `ScrollHandle` wired to the keyboard-scroll action set. Hosts
  should register the new `textview_keybindings()` set alongside the
  existing `text_keybindings()` (or call `all_keybindings()`) during app
  startup so the raw Up / Down / Page / Home / End keys fire against the
  focused view. The gallery and all in-crate tests are migrated.

  ```rust
  // Before
  TextView::new("Hello").text_style(TextStyle::Body)

  // After
  cx.new(|cx| TextView::new(cx, "Hello").text_style(TextStyle::Body))
  ```

### Behavioural change (source-compatible but visible)

- Apps that use `TahoeTheme::liquid_glass()` / `liquid_glass_light()` without
  wrapping their root in a `GlassSurfaceScope` will see icons render with
  `IconStyle::Standard` (muted text color, 1.2pt stroke). **Migration**: wrap
  your window's root in `GlassSurfaceScope::new(...)`, or rely on the
  in-crate glass-surface components (`GlassIconTile`, `Button` with glass
  variants) which declare scope themselves. See the
  `liquid_glass_gallery` and `liquid_glass_interactive` examples.
- `LeadingStyle::Tight` / `LeadingStyle::Loose` now scale leading
  proportionally (`× 0.95` / `× 1.15`) instead of a flat ±2 pt offset.
  The proportional delta keeps tight/loose visually consistent across
  all [`TextStyle`] sizes — a 2 pt reduction on Body's 16 pt leading
  was 12.5% but only 6.25% on LargeTitle's 32 pt. Callers relying on
  the exact pt delta will see different pixel values.
- `LeadingStyle::Tight` floor is now tiered by size: body-scale styles
  (size ≤ 15 pt — Body, Callout, Subheadline, Footnote, Caption1,
  Caption2) clamp at `size × 1.5` so running paragraphs meet WCAG
  1.4.12 (*Text Spacing*); display styles (Title*, LargeTitle,
  Headline) keep the prior `size × 1.15` SF Pro ascender/descender
  floor. Apps using `LeadingStyle::Tight` on Body copy will see
  slightly taller line boxes than before; display styles are
  unchanged.

### Known limitations

- The scope does **not** propagate across deferred-draw boundaries
  (`gpui::deferred()` children, and any component that uses it — popovers,
  pulldown buttons, combo boxes, tooltips, glass morph overlays) or across
  sub-windows opened via `cx.open_window(...)`. Components that render a glass
  child through a deferred boundary must re-wrap the deferred content in
  `GlassSurfaceScope`, or hold a `GlassSurfaceGuard` across the boundary. See
  the module-level documentation in `foundations/surface_scope.rs`.
