# Changelog

All notable changes to `tahoe-gpui` land here. Format loosely tracks
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning will
follow SemVer once the crate reaches 1.0.

## [Unreleased]

### Fixed

- **`IconStyle::Auto` now consults surface scope instead of always resolving to
  `LiquidGlass`** (issue
  [#13](https://github.com/df49b9cd/tahoe-gpui/issues/13)). Previously every
  `Icon::new(...)` — the default — rendered with glass pastel colors and a
  1.5pt stroke regardless of whether the icon sat on a Liquid Glass surface.
  Per HIG §Materials (`docs/hig/foundations.md:1045`), vibrancy is applied by
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
  component with 11 new builders: `styled_text`, `max_lines`, `emphasize`,
  `color`, `label_level`, `font_design`, `leading_style`, `text_align`,
  `scrollable`, `readable_width`, and `accessibility_label`.
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
  was 12.5% but only 6.25% on LargeTitle's 32 pt. Tight is capped at
  `× 0.95` rather than a larger reduction so SF Pro ascenders /
  descenders never collide (at Body that is 15.2 pt against a 13 pt
  size, ≈1.17×). Callers relying on the exact pt delta will see
  different pixel values.

### Known limitations

- The scope does **not** propagate across deferred-draw boundaries
  (`gpui::deferred()` children, and any component that uses it — popovers,
  pulldown buttons, combo boxes, tooltips, glass morph overlays) or across
  sub-windows opened via `cx.open_window(...)`. Components that render a glass
  child through a deferred boundary must re-wrap the deferred content in
  `GlassSurfaceScope`, or hold a `GlassSurfaceGuard` across the boundary. See
  the module-level documentation in `foundations/surface_scope.rs`.
- `TextView::styled_text(text, styled)` now takes a plain-text argument
  alongside the `StyledText` so VoiceOver has a label to announce without
  callers needing to restate the content via `.accessibility_label(...)`.
- `TextView::max_lines(...)` and `TextView::scrollable(...)` are mutually
  exclusive: clamped height short-circuits GPUI's scroll viewport. Setting
  both trips a `debug_assert!` so the conflict panics in tests; release
  builds silently prefer `max_lines`.
