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
  component with 11 new builders: `styled_text`, `selectable`, `max_lines`,
  `emphasize`, `color`, `label_level`, `font_design`, `leading_style`,
  `disabled`, `text_align`, `scrollable`, `readable_width`, and
  `accessibility_label`.
- `components::content::text_view::LabelLevel` — HIG semantic-tier enum
  (`Primary` / `Secondary` / `Tertiary` / `Quaternary` / `Quinary`, the
  last added in macOS Tahoe) resolving to the matching theme colour via
  [`TextView::label_level`].
- `foundations::accessibility::AccessibilityProps::disabled: bool` + a
  `.disabled(bool)` builder. `TextView::disabled(true)` now sets it so
  VoiceOver will announce the dimmed state once GPUI lands an AX tree.

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

### Behavioural change (source-compatible but visible)

- Apps that use `TahoeTheme::liquid_glass()` / `liquid_glass_light()` without
  wrapping their root in a `GlassSurfaceScope` will see icons render with
  `IconStyle::Standard` (muted text color, 1.2pt stroke). **Migration**: wrap
  your window's root in `GlassSurfaceScope::new(...)`, or rely on the
  in-crate glass-surface components (`GlassIconTile`, `Button` with glass
  variants) which declare scope themselves. See the
  `liquid_glass_gallery` and `liquid_glass_interactive` examples.
- `LeadingStyle::Tight` / `LeadingStyle::Loose` now scale leading by
  ±15% proportionally (`× 0.85` / `× 1.15`) instead of a flat ±2 pt
  offset. The proportional delta keeps tight/loose visually consistent
  across all [`TextStyle`] sizes — a 2 pt reduction on Body's 16 pt
  leading was 12.5% but only 6.25% on LargeTitle's 32 pt. Callers
  relying on the exact pt delta will see different pixel values.

### Known limitations

- The scope does **not** propagate across deferred-draw boundaries
  (`gpui::deferred()` children, and any component that uses it — popovers,
  pulldown buttons, combo boxes, tooltips, glass morph overlays) or across
  sub-windows opened via `cx.open_window(...)`. Components that render a glass
  child through a deferred boundary must re-wrap the deferred content in
  `GlassSurfaceScope`, or hold a `GlassSurfaceGuard` across the boundary. See
  the module-level documentation in `foundations/surface_scope.rs`.
- `TextView::selectable(true)` stores intent only — text selection is not
  yet implemented (GPUI has no built-in selection API; the planned path
  mirrors `markdown::selectable_text::SelectableText`'s raw mouse-event
  approach). A one-shot `tracing::warn!` fires in debug when selection is
  requested.
- `TextView::styled_text(...)` renders rich content without a VoiceOver
  label unless the caller also supplies `.accessibility_label("...")`.
  A one-shot debug warning fires when this combination is missed.
- `TextView::max_lines(...)` and `TextView::scrollable(...)` are mutually
  exclusive: clamped height short-circuits GPUI's scroll viewport. When
  both are set, `max_lines` wins and a one-shot debug warning fires.
