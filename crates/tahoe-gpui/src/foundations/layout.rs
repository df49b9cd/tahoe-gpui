//! Layout tokens aligned with HIG.
//!
//! Provides platform-aware layout constants, direction handling, and shape
//! type definitions. All sizing values come from the Apple Human Interface
//! Guidelines and adapt per target platform.
//!
//! Line references like `foundations.md:Lxxx` point at
//! `docs/hig/foundations.md` in this repo.

use gpui::{Div, ParentElement, Pixels, Styled, div, px};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Flex helpers — Zed-style h_flex / v_flex
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// Finding 2 in the Zed cross-reference audit. Zed
// ships free functions `h_flex()` / `v_flex()` plus the matching `StyledExt`
// methods so every component can write a one-call row or column layout
// instead of chaining `.flex().flex_row().items_center()` / `.flex_col()`.
// We mirror both forms verbatim.

/// Horizontally stacks elements.
///
/// Equivalent to `div().flex().flex_row().items_center()`.
///
/// Mirrors Zed's `ui::h_flex()` (see
/// `crates/ui/src/components/stack.rs` in zed-industries/zed).
#[track_caller]
pub fn h_flex() -> Div {
    div().h_flex()
}

/// Vertically stacks elements.
///
/// Equivalent to `div().flex().flex_col()`.
///
/// Mirrors Zed's `ui::v_flex()` (see
/// `crates/ui/src/components/stack.rs` in zed-industries/zed).
#[track_caller]
pub fn v_flex() -> Div {
    div().v_flex()
}

/// Extension methods on any GPUI [`Styled`] element for Zed-compatible
/// flex helpers.
///
/// Kept identical to Zed's `StyledExt::h_flex` / `v_flex` so code copied
/// between the two codebases keeps working. `h_flex` centers children
/// vertically (matching SwiftUI `HStack`'s default VerticalAlignment.center);
/// `v_flex` leaves cross-axis alignment to the caller since `VStack`'s
/// default is leading alignment, which `flex_col()` already produces.
pub trait FlexExt: Styled + Sized {
    /// Horizontally stacks elements. Sets `flex()`, `flex_row()`, `items_center()`.
    fn h_flex(self) -> Self {
        self.flex().flex_row().items_center()
    }

    /// Vertically stacks elements. Sets `flex()`, `flex_col()`.
    fn v_flex(self) -> Self {
        self.flex().flex_col()
    }
}

impl<E: Styled + Sized> FlexExt for E {}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Hit-region expansion (pointer target ≥ visual size)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Wrap `child` in a transparent, centered container at least `min` in
/// each dimension so pointer clicks land on the wrapper and reach the
/// same target even when the visual child is smaller.
///
/// Use this when a compact control has a visual footprint smaller than
/// comfortable pointer acquisition would expect — e.g. 14 pt traffic-light
/// dots in a title bar, `ControlSize::Mini` glyphs in a dense toolbar, or
/// split-view divider strokes. Attach the `on_click` / `on_action` handler
/// on the returned `Div` (the wrapper), not the child.
///
/// A reasonable floor for macOS is the [`ControlSize`] tier one step above
/// the visual size — e.g. a [`ControlSize::Mini`] (20 pt) glyph gets a
/// [`ControlSize::Regular`] (28 pt) hit region. Apple's public HIG does
/// not publish a macOS pointer minimum, so avoid hardcoding a fixed 44 pt
/// floor; scale relative to the tiers instead.
///
/// ```ignore
/// use tahoe_gpui::foundations::layout::{hit_region, ControlSize};
/// use gpui::px;
///
/// let tier = ControlSize::Regular.height(theme.platform);
/// hit_region(px(tier), my_tiny_icon).on_click(|_, _, _| {})
/// ```
pub fn hit_region(min: Pixels, child: impl gpui::IntoElement) -> Div {
    div()
        .min_w(min)
        .min_h(min)
        .flex()
        .items_center()
        .justify_center()
        .child(child)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Platform
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Target platform per HIG.
///
/// Different platforms have different sizing guidelines for interactive
/// controls, text, and layout. Use this to get the correct values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Platform {
    /// iOS and iPadOS — touch interaction, medium displays.
    IOS,
    /// macOS — pointer interaction, large displays.
    #[default]
    MacOS,
    /// tvOS — remote interaction, large TV displays.
    TvOS,
    /// visionOS — eye/hand interaction, spatial displays.
    VisionOS,
    /// watchOS — touch interaction, small displays.
    WatchOS,
}

impl Platform {
    /// Default interactive target size per HIG.
    ///
    /// These are the **AppKit / SwiftUI control metrics** used by each
    /// platform's default controls. On touch and spatial platforms they
    /// double as the Apple-published accessibility minimum for mobility;
    /// macOS does not publish a separate pointer-accessibility floor
    /// (see `foundations.md:L72–L81`), so the default control metric is
    /// the recommended visual size and interactive elements should extend
    /// their **hit region** past their visual size in dense layouts.
    ///
    /// | Platform | Default | Minimum |
    /// |---|---|---|
    /// | iOS, iPadOS | 44×44 pt | 28×28 pt |
    /// | macOS | 28×28 pt | 20×20 pt |
    /// | tvOS | 66×66 pt | 56×56 pt |
    /// | visionOS | 60×60 pt | 28×28 pt |
    /// | watchOS | 44×44 pt | 28×28 pt |
    pub fn default_target_size(self) -> f32 {
        ControlSize::Regular.height(self)
    }

    /// Minimum interactive target size per HIG.
    ///
    /// Absolute floor for compressed ("mini" / "small") control variants.
    /// On touch and spatial platforms these match the Apple-published
    /// accessibility minimum. On macOS this is the AppKit / SwiftUI
    /// mini-control metric; Apple does not document a separate pointer
    /// accessibility floor, so extend the **hit region** past the visual
    /// size when neighbouring targets are tight.
    pub fn min_target_size(self) -> f32 {
        ControlSize::Mini.height(self)
    }

    /// Default body text size per HIG Typography.
    ///
    /// | Platform | Default | Minimum |
    /// |---|---|---|
    /// | iOS, iPadOS | 17 pt | 11 pt |
    /// | macOS | 13 pt | 10 pt |
    /// | tvOS | 29 pt | 23 pt |
    /// | visionOS | 17 pt | 12 pt |
    /// | watchOS | 16 pt | 12 pt |
    pub fn default_text_size(self) -> f32 {
        match self {
            Self::IOS => 17.0,
            Self::MacOS => 13.0,
            Self::TvOS => 29.0,
            Self::VisionOS => 17.0,
            Self::WatchOS => 16.0,
        }
    }

    /// Minimum text size per HIG Typography.
    pub fn min_text_size(self) -> f32 {
        match self {
            Self::IOS => 11.0,
            Self::MacOS => 10.0,
            Self::TvOS => 23.0,
            Self::VisionOS => 12.0,
            Self::WatchOS => 12.0,
        }
    }

    /// Standard list/menu row height for the platform.
    pub fn row_height(self) -> f32 {
        match self {
            Self::IOS => 44.0,
            Self::MacOS => 28.0,
            Self::TvOS => 66.0,
            Self::VisionOS => 60.0,
            Self::WatchOS => 44.0,
        }
    }

    /// Standard navigation bar height for the platform.
    ///
    /// iOS / iPadOS use a 44 pt `UINavigationBar`. macOS Tahoe toolbars sit
    /// at 36 pt (44 pt only when the title bar + toolbar is in the unified
    /// large style — see [`MACOS_TOOLBAR_UNIFIED_HEIGHT`] for that variant).
    /// tvOS and visionOS scale up; watchOS has no top navigation bar and
    /// falls back to 44 pt.
    pub fn navigation_bar_height(self) -> f32 {
        match self {
            Self::IOS => 44.0,
            Self::MacOS => 36.0,
            Self::TvOS => 88.0,
            Self::VisionOS => 60.0,
            Self::WatchOS => 44.0,
        }
    }

    /// Classify a container width in points into an iOS/iPadOS size class.
    ///
    /// Per `foundations.md:L917–L981`, iOS size-class thresholds split
    /// roughly at the 600 pt mark — iPhones in portrait are Compact width,
    /// iPads are Regular width. macOS and tvOS are always Regular; watchOS
    /// is always Compact.
    pub fn size_class_for_width(self, points: f32) -> SizeClass {
        match self {
            // iPads and larger iPhone landscapes cross 600 pt into Regular.
            Self::IOS | Self::VisionOS => {
                if points >= 600.0 {
                    SizeClass::Regular
                } else {
                    SizeClass::Compact
                }
            }
            Self::MacOS | Self::TvOS => SizeClass::Regular,
            Self::WatchOS => SizeClass::Compact,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ControlSize — canonical height tiers (SwiftUI `ControlSize`)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Canonical interactive-control height tier.
///
/// Mirrors Apple's [`SwiftUI.ControlSize`][1] so new components can pick a
/// size without inventing a per-component enum. On **macOS** the tiers map
/// to the AppKit / SwiftUI control heights:
///
/// | Tier | macOS | iOS / iPadOS | tvOS | visionOS | watchOS |
/// |------|-------|--------------|------|----------|---------|
/// | `Mini` | 20 pt | 28 pt | 56 pt | 28 pt | 28 pt |
/// | `Small` | 24 pt | 36 pt | 60 pt | 44 pt | 36 pt |
/// | `Regular` (default) | 28 pt | 44 pt | 66 pt | 60 pt | 44 pt |
/// | `Large` | 32 pt | 50 pt | 72 pt | 66 pt | 48 pt |
/// | `ExtraLarge` | 36 pt | 56 pt | 80 pt | 72 pt | 52 pt |
///
/// `Mini` on touch platforms intentionally sits below Apple's 44 pt touch
/// minimum — it matches [`Platform::min_target_size`] for dense forms and
/// should be combined with [`hit_region`] expansion when used there.
///
/// Apple does **not** publish a separate pointer-accessibility floor for
/// macOS; compact controls in dense layouts should expand their **hit
/// region** past the visual size using [`hit_region`] rather than inflate
/// the control itself. `Regular` is the size a caller should reach for
/// unless there is a concrete reason to use another tier — it matches
/// [`Platform::default_target_size`].
///
/// [1]: https://developer.apple.com/documentation/swiftui/controlsize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ControlSize {
    /// SwiftUI `.mini`. Use in dense inspectors, color wells, palette rows.
    Mini,
    /// SwiftUI `.small`. Use in inspectors, sidebar toolbars, compact forms.
    Small,
    /// SwiftUI `.regular`. Default. Use for ordinary buttons and text fields.
    #[default]
    Regular,
    /// SwiftUI `.large`. Use for prominent CTAs (onboarding, primary actions).
    Large,
    /// SwiftUI `.extraLarge` (macOS 14+). Use for hero buttons and
    /// accessibility-oriented large-text modes.
    ExtraLarge,
}

impl ControlSize {
    /// Visual height in points for this tier on the given platform.
    pub fn height(self, platform: Platform) -> f32 {
        match (platform, self) {
            (Platform::MacOS, Self::Mini) => 20.0,
            (Platform::MacOS, Self::Small) => 24.0,
            (Platform::MacOS, Self::Regular) => 28.0,
            (Platform::MacOS, Self::Large) => 32.0,
            (Platform::MacOS, Self::ExtraLarge) => 36.0,

            (Platform::IOS, Self::Mini) => 28.0,
            (Platform::IOS, Self::Small) => 36.0,
            (Platform::IOS, Self::Regular) => 44.0,
            (Platform::IOS, Self::Large) => 50.0,
            (Platform::IOS, Self::ExtraLarge) => 56.0,

            (Platform::TvOS, Self::Mini) => 56.0,
            (Platform::TvOS, Self::Small) => 60.0,
            (Platform::TvOS, Self::Regular) => 66.0,
            (Platform::TvOS, Self::Large) => 72.0,
            (Platform::TvOS, Self::ExtraLarge) => 80.0,

            (Platform::VisionOS, Self::Mini) => 28.0,
            (Platform::VisionOS, Self::Small) => 44.0,
            (Platform::VisionOS, Self::Regular) => 60.0,
            (Platform::VisionOS, Self::Large) => 66.0,
            (Platform::VisionOS, Self::ExtraLarge) => 72.0,

            (Platform::WatchOS, Self::Mini) => 28.0,
            (Platform::WatchOS, Self::Small) => 36.0,
            (Platform::WatchOS, Self::Regular) => 44.0,
            (Platform::WatchOS, Self::Large) => 48.0,
            (Platform::WatchOS, Self::ExtraLarge) => 52.0,
        }
    }

    /// The tier one step smaller, or `self` if already at [`Self::Mini`].
    ///
    /// Useful for [`hit_region`] callers that want to derive a hit expansion
    /// floor from "one tier up" relative to the visual size.
    pub fn smaller(self) -> Self {
        match self {
            Self::Mini | Self::Small => Self::Mini,
            Self::Regular => Self::Small,
            Self::Large => Self::Regular,
            Self::ExtraLarge => Self::Large,
        }
    }

    /// The tier one step larger, or `self` if already at [`Self::ExtraLarge`].
    pub fn larger(self) -> Self {
        match self {
            Self::Mini => Self::Small,
            Self::Small => Self::Regular,
            Self::Regular => Self::Large,
            Self::Large | Self::ExtraLarge => Self::ExtraLarge,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Control metrics (macOS — AppKit / SwiftUI ControlSize)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// macOS default interactive control size (28×28 pt).
///
/// AppKit / SwiftUI `.regular` control metric for buttons, text fields,
/// menu rows, and toolbar items. Matches [`ControlSize::Regular`] on
/// [`Platform::MacOS`] and the value returned by
/// [`Platform::default_target_size`].
///
/// This is the _visual_ control size. Apple's public HIG does not publish
/// a separate macOS pointer-accessibility floor, so controls placed in
/// dense layouts should extend their **hit region** past this dimension
/// via [`hit_region`] rather than inflate the visual size.
pub const MACOS_DEFAULT_TOUCH_TARGET: f32 = 28.0;

/// macOS minimum interactive control size (20×20 pt).
///
/// AppKit / SwiftUI `.mini` control metric. Matches [`ControlSize::Mini`]
/// on [`Platform::MacOS`] and [`Platform::min_target_size`].
pub const MACOS_MIN_TOUCH_TARGET: f32 = 20.0;

/// Diameter of a presence/unread dot in points (8 pt).
///
/// Used by [`Badge::dot`](crate::components::content::badge::Badge::dot) and
/// matching avatar status indicators.
pub const BADGE_DOT_SIZE: f32 = 8.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Window Chrome (macOS AppKit) — foundations.md:L767
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// macOS title bar height without a toolbar (28 pt).
///
/// AppKit `NSWindowStyleMask::Titled` reserves 28 pt for the standalone
/// title bar region. HIG says "respect the title bar and toolbar
/// areas" (`foundations.md:L767`); the 28 pt measurement is the AppKit
/// system value exposed via `NSWindow.titlebarSeparatorStyle`.
pub const MACOS_TITLE_BAR_HEIGHT: f32 = 28.0;

/// macOS panel (`NSPanel`) title bar height (22 pt).
///
/// Floating panels — inspectors, Fonts, Colors, HUD overlays — render
/// with a narrower title bar than regular document windows. HIG
/// `#panels` describes panels as "auxiliary" surfaces; AppKit's
/// `NSWindowStyleMask::UtilityWindow` shortens the title region to 22 pt.
pub const MACOS_PANEL_TITLE_BAR_HEIGHT: f32 = 22.0;

/// macOS unified title bar + toolbar height (52 pt).
///
/// When `NSWindowToolbarStyle::Unified` is in use, title bar and toolbar
/// occupy 52 pt combined. macOS 26 Tahoe reserves this region for Liquid
/// Glass chrome — see [`SurfaceRole::WindowChrome`].
pub const MACOS_TOOLBAR_UNIFIED_HEIGHT: f32 = 52.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Content Spacing Ladder
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// HIG 4-pt baseline. The full ladder is exposed here so components can
// reach for `SPACING_4/8/12/16/20/24/32/40/48` instead of scattering raw
// `px(N.0)` literals — the latter defeat design-token overrides and drift
// from the published scale. The semantic names (`CONTENT_MARGIN`,
// `GROUP_SPACING`, `SECTION_SPACING`) stay around for intent-revealing
// call sites; they remain equal to the corresponding numeric rung.

/// HIG 4-pt spacing rung — used for tight inner padding inside controls
/// (alert suppression tick, caption spacing).
pub const SPACING_4: f32 = 4.0;

/// HIG 8-pt spacing rung — the lowest "safe" gap between separate
/// interactive elements. Equal to [`GROUP_SPACING`].
pub const SPACING_8: f32 = 8.0;

/// HIG 12-pt spacing rung — midpoint between tight (`8`) and standard (`16`).
/// Used for row-level gaps inside tables and lists.
pub const SPACING_12: f32 = 12.0;

/// HIG 16-pt spacing rung — the default content margin. Equal to
/// [`CONTENT_MARGIN`].
pub const SPACING_16: f32 = 16.0;

/// HIG 20-pt spacing rung — wide content margin. Equal to
/// [`CONTENT_MARGIN_WIDE`].
pub const SPACING_20: f32 = 20.0;

/// HIG 24-pt spacing rung — section-break spacing. Equal to
/// [`SECTION_SPACING`].
pub const SPACING_24: f32 = 24.0;

/// HIG 32-pt spacing rung — generous section breaks (e.g. between
/// unrelated forms in a preferences pane).
pub const SPACING_32: f32 = 32.0;

/// HIG 40-pt spacing rung — large breathing room (hero layouts, empty
/// states, onboarding).
pub const SPACING_40: f32 = 40.0;

/// HIG 48-pt spacing rung — the top of the common ladder. Beyond this
/// switch to a percentage-of-container measure rather than a fixed rung.
pub const SPACING_48: f32 = 48.0;

/// HIG default content margin (16 pt).
pub const CONTENT_MARGIN: f32 = SPACING_16;

/// HIG wide content margin (20 pt).
pub const CONTENT_MARGIN_WIDE: f32 = SPACING_20;

/// Horizontal inset from the window edge to primary content (20 pt).
///
/// Same value as [`CONTENT_MARGIN_WIDE`]; the distinct name documents the
/// semantic of a _window-edge inset_ vs. an interior margin.
pub const CONTENT_HORIZONTAL_PADDING: f32 = SPACING_20;

/// Vertical spacing between related controls within a group (8 pt).
///
/// Use between labels and their controls, or between tightly-related rows
/// in a Form. For unrelated groups use [`SECTION_SPACING`].
pub const GROUP_SPACING: f32 = SPACING_8;

/// Vertical spacing between major content sections (24 pt).
///
/// The section-break spacing on macOS; used to visually separate groups of
/// related controls within a single pane.
pub const SECTION_SPACING: f32 = SPACING_24;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Readable Content Widths
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Optimal readable line width (~65 characters at 13 pt macOS body).
///
/// `foundations.md:L772` recommends "readable content guides" for long-form
/// text; 544 pt is the empirically-preferred column width at the macOS
/// default body size.
pub const READABLE_OPTIMAL_WIDTH: f32 = 544.0;

/// Maximum readable line width (~70 characters at body size).
pub const READABLE_MAX_WIDTH: f32 = 672.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sidebar / Panel sizing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// macOS sidebar minimum floor (180 pt).
///
/// HIG macOS Tahoe: sidebar / source-list panes should not shrink below
/// 180 pt — the point at which row labels begin to truncate on the
/// default body-size Dynamic Type settings. Matches the `NSSplitView`
/// auto-collapse threshold observed in Mail, Finder, and System
/// Settings. Callers extending this floor (e.g. rich media browsers)
/// should pass a larger value to `Sidebar::min_width` in
/// `components::navigation_and_search`.
pub const SIDEBAR_MIN_WIDTH: f32 = 180.0;

/// macOS sidebar default width (220 pt).
///
/// HIG macOS Tahoe: the stock `NSSplitViewController` primary column
/// opens at 220 pt — the midpoint of the 180–320 pt typical range.
/// Exposed as a free constant so layout-layer code (split views,
/// three-column shells) can pick the same default without reaching
/// into `TahoeTheme`.
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 220.0;

/// Inspector panel default width (250 pt).
///
/// HIG macOS Tahoe: Notes / Mail-style inspector columns open at
/// 250 pt. Separate from [`INSPECTOR_PANEL_WIDTH`] (320 pt), which
/// covers Xcode's Attributes Inspector — the wider Pro-app variant.
pub const INSPECTOR_DEFAULT_WIDTH: f32 = 250.0;

/// Inspector panel default width (320 pt).
///
/// Apple macOS inspector panels conventionally range 256–320 pt. 320 pt is
/// the default wide inspector, matching Xcode's Attributes Inspector.
pub const INSPECTOR_PANEL_WIDTH: f32 = 320.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Split-view divider
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Visual width of a split-view divider (4 pt).
pub const SPLIT_DIVIDER_WIDTH: f32 = 4.0;

/// Hit-area width of a split-view divider (20 pt).
///
/// Matches [`MACOS_MIN_TOUCH_TARGET`]. The visual stroke is
/// [`SPLIT_DIVIDER_WIDTH`]; an invisible region extends the activation
/// region so pointer users can grab the divider comfortably.
pub const SPLIT_DIVIDER_HIT_AREA: f32 = 20.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Menu / dropdown sizing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Maximum dropdown menu height per HIG.
pub const DROPDOWN_MAX_HEIGHT: f32 = 264.0;

/// Inset kept between a dropdown / popover and the window edges when the
/// rendered surface would otherwise clip.
///
/// Mirrors Zed's convention (see `crates/ui/src/components/context_menu.rs`
/// — `anchored().snap_to_window_with_margin(px(8.0))`). Components that
/// present a floating dropdown should either wrap their content element in
/// [`gpui::anchored`] with this margin or thread the value through
/// [`snap_to_window_margin`] so window-edge clipping is avoided.
///
/// Finding 12 in the Zed cross-reference audit tracks the migration of
/// the crate's 8 dropdown components to this pattern.
pub const DROPDOWN_SNAP_MARGIN: Pixels = px(8.0);

/// Returns the [`DROPDOWN_SNAP_MARGIN`] token.
///
/// Wrapper exists so callers can import a single symbol from the prelude
/// and pass it directly into
/// `gpui::anchored().snap_to_window_with_margin(snap_to_window_margin())`
/// without reaching into the `foundations::layout` module path.
#[inline]
pub fn snap_to_window_margin() -> Pixels {
    DROPDOWN_SNAP_MARGIN
}

/// Minimum context menu width per HIG.
pub const MENU_MIN_WIDTH: f32 = 180.0;

/// Maximum context menu width per HIG.
pub const MENU_MAX_WIDTH: f32 = 280.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Modal sizing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Standard alert dialog width per the Figma Tahoe UI Kit (260 pt).
///
/// GPUI targets desktop hosts (macOS, Linux, Windows) only, so there is no
/// iOS vs macOS sizing to branch on — the kit's canonical width is 260 pt
/// regardless of the renderer. Callers may still override via
/// [`Alert::width`](crate::components::presentation::alert::Alert::width)
/// for wider content.
pub const ALERT_WIDTH: f32 = 260.0;

/// Standard modal dialog width per HIG.
///
/// Roughly `READABLE_MAX_WIDTH / 2`. Consumers can still override via
/// `Modal::width` for wider content.
pub const MODAL_WIDTH: f32 = 400.0;

/// Maximum modal dialog height per HIG.
pub const MODAL_MAX_HEIGHT: f32 = 500.0;

/// Maximum popover width per HIG `#popovers` sizing guidance (320 pt).
///
/// HIG: "avoid making a popover too big." Matches the upper bound of the
/// canonical macOS popover (typically 200–320 pt wide for navigation and
/// selection content).
pub const POPOVER_MAX_WIDTH: f32 = 320.0;

/// Hover card maximum width — shares [`POPOVER_MAX_WIDTH`] (320 pt).
///
/// HoverCards share popover layering and should respect the same upper
/// bound so rich hover surfaces don't dominate the viewport.
pub const HOVER_CARD_MAX_WIDTH: f32 = POPOVER_MAX_WIDTH;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// tvOS grid — foundations.md:L793–L858
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Uniform horizontal spacing between tvOS grid cells (40 pt).
pub const TVOS_GRID_H_SPACING: f32 = 40.0;

/// Minimum vertical spacing between tvOS grid rows (100 pt).
pub const TVOS_GRID_MIN_V_SPACING: f32 = 100.0;

/// tvOS focus-grid column counts with their per-HIG unfocused content widths.
///
/// Per `foundations.md:L793–L858`, tvOS grids have a fixed unfocused
/// content width for each supported column count. Horizontal spacing
/// ([`TVOS_GRID_H_SPACING`]) and minimum vertical spacing
/// ([`TVOS_GRID_MIN_V_SPACING`]) are uniform across all counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TvOsGridColumns {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl TvOsGridColumns {
    /// The unfocused content width in points for this column count.
    pub fn unfocused_width(self) -> f32 {
        match self {
            Self::Two => 860.0,
            Self::Three => 560.0,
            Self::Four => 410.0,
            Self::Five => 320.0,
            Self::Six => 260.0,
            Self::Seven => 217.0,
            Self::Eight => 184.0,
            Self::Nine => 160.0,
        }
    }

    /// The integer column count.
    pub fn count(self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// watchOS screen sizes — foundations.md:L985–L996
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// watchOS physical case / screen classes covered by the library.
///
/// Values are the pixel dimensions listed in `foundations.md:L985–L996`
/// for the current Apple Watch lineup (Series 11, Ultra 3). Older variants
/// can be added as needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WatchOsSize {
    /// 42 mm case — Apple Watch Series 10/11 (374 × 446 px).
    Mm42,
    /// 46 mm case — Apple Watch Series 10/11 (416 × 496 px).
    Mm46,
    /// 49 mm case — Apple Watch Ultra 3 (422 × 514 px).
    Mm49,
}

impl WatchOsSize {
    /// Screen width in pixels.
    pub fn width_px(self) -> f32 {
        match self {
            Self::Mm42 => 374.0,
            Self::Mm46 => 416.0,
            Self::Mm49 => 422.0,
        }
    }

    /// Screen height in pixels.
    pub fn height_px(self) -> f32 {
        match self {
            Self::Mm42 => 446.0,
            Self::Mm46 => 496.0,
            Self::Mm49 => 514.0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SizeClass — foundations.md:L917–L981
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// iOS/iPadOS horizontal size class.
///
/// `foundations.md:L917–L981` lists every device's portrait/landscape size
/// class. Use `Platform::size_class_for_width(width)` to classify a
/// container width. Compact width == iPhone-style layout, Regular width ==
/// iPad-style layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SizeClass {
    #[default]
    Compact,
    Regular,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SurfaceRole — foundations.md:L1049 (Liquid Glass)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Semantic role of a surface, paired with the Liquid Glass material tier.
///
/// Per `foundations.md:L1049` (macOS 26 Tahoe), Liquid Glass is the required
/// material for window chrome, sidebars, and floating panels. Regular
/// content surfaces remain opaque. Components that render a background
/// accept a [`SurfaceRole`] so the theme can pick the correct glass tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SurfaceRole {
    /// Title bar / toolbar region. Uses Liquid Glass on macOS 26 Tahoe.
    WindowChrome,
    /// Sidebar / primary navigation pane. Uses Liquid Glass on macOS 26.
    Sidebar,
    /// Floating panel (popover, inspector, toolbar overlay). Liquid Glass.
    FloatingPanel,
    /// Regular content area. Opaque material per HIG.
    #[default]
    Content,
}

impl SurfaceRole {
    /// Whether this role should render with Liquid Glass on macOS 26 Tahoe.
    pub fn uses_liquid_glass(self) -> bool {
        matches!(
            self,
            Self::WindowChrome | Self::Sidebar | Self::FloatingPanel
        )
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LayoutDirection
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Layout direction for right-to-left language support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

impl LayoutDirection {
    /// Returns true if this is a right-to-left layout.
    pub fn is_rtl(&self) -> bool {
        matches!(self, LayoutDirection::RightToLeft)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ShapeType
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Per-surface geometry — mirrors SwiftUI's `Shape` parameter on
/// [`glassEffect(_:in:)`][apple-glassEffect]. Callers pick a shape
/// independently from the glass material, so a `Glass::Regular` surface
/// can be rendered as a rounded rectangle or as a capsule without
/// changing the material recipe.
///
/// Re-exported as `Shape` from [`foundations::materials`](crate::foundations::materials)
/// for the Apple-named surface.
///
/// [apple-glassEffect]: https://developer.apple.com/documentation/SwiftUI/View/glassEffect(_:in:)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    /// Constant corner radius. Equivalent to
    /// [`ShapeType::RoundedRectangle`]; kept for back-compat with the
    /// concentricity helpers.
    Fixed(Pixels),
    /// Rounded rectangle with an explicit corner radius. Sugar that reads
    /// closer to SwiftUI's `RoundedRectangle(cornerRadius:)` at call
    /// sites — resolves identically to [`ShapeType::Fixed`] via
    /// `compute_shape_radius`.
    RoundedRectangle(Pixels),
    /// Radius equals half the container height (pill/capsule).
    Capsule,
    /// Radius = parent_radius - padding, aligned to shared center.
    Concentric {
        parent_radius: Pixels,
        padding: Pixels,
    },
    /// HIG default shape. Mirrors SwiftUI's `DefaultGlassEffectShape`.
    /// Resolves to `theme.radius_md` when no `container_height` is
    /// supplied, or to a capsule when a height is supplied — the same
    /// rule the SwiftUI default follows for interactive controls.
    Default,
}

#[cfg(test)]
mod tests {
    use super::{
        ALERT_WIDTH, ControlSize, HOVER_CARD_MAX_WIDTH, MACOS_DEFAULT_TOUCH_TARGET,
        MACOS_MIN_TOUCH_TARGET, MACOS_PANEL_TITLE_BAR_HEIGHT, MACOS_TITLE_BAR_HEIGHT,
        MACOS_TOOLBAR_UNIFIED_HEIGHT, POPOVER_MAX_WIDTH, Platform, SIDEBAR_MIN_WIDTH, SizeClass,
        SurfaceRole, TvOsGridColumns, WatchOsSize,
    };
    use core::prelude::v1::test;

    #[test]
    fn macos_touch_target_constants_match_platform() {
        assert!(
            (Platform::MacOS.default_target_size() - MACOS_DEFAULT_TOUCH_TARGET).abs()
                < f32::EPSILON
        );
        assert!((Platform::MacOS.min_target_size() - MACOS_MIN_TOUCH_TARGET).abs() < f32::EPSILON);
    }

    #[test]
    fn control_size_macos_heights_match_swiftui() {
        let p = Platform::MacOS;
        assert_eq!(ControlSize::Mini.height(p), 20.0);
        assert_eq!(ControlSize::Small.height(p), 24.0);
        assert_eq!(ControlSize::Regular.height(p), 28.0);
        assert_eq!(ControlSize::Large.height(p), 32.0);
        assert_eq!(ControlSize::ExtraLarge.height(p), 36.0);
    }

    #[test]
    fn control_size_regular_matches_default_target_size() {
        for p in [
            Platform::MacOS,
            Platform::IOS,
            Platform::TvOS,
            Platform::VisionOS,
            Platform::WatchOS,
        ] {
            assert_eq!(
                ControlSize::Regular.height(p),
                p.default_target_size(),
                "Regular tier should match Platform::{p:?}::default_target_size"
            );
            assert_eq!(
                ControlSize::Mini.height(p),
                p.min_target_size(),
                "Mini tier should match Platform::{p:?}::min_target_size"
            );
        }
    }

    #[test]
    fn control_size_neighbour_helpers_are_bounded() {
        assert_eq!(ControlSize::Mini.smaller(), ControlSize::Mini);
        assert_eq!(ControlSize::ExtraLarge.larger(), ControlSize::ExtraLarge);
        assert_eq!(ControlSize::Regular.larger(), ControlSize::Large);
        assert_eq!(ControlSize::Regular.smaller(), ControlSize::Small);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn window_chrome_heights_match_appkit() {
        assert!((MACOS_TITLE_BAR_HEIGHT - 28.0).abs() < f32::EPSILON);
        assert!((MACOS_PANEL_TITLE_BAR_HEIGHT - 22.0).abs() < f32::EPSILON);
        assert!((MACOS_TOOLBAR_UNIFIED_HEIGHT - 52.0).abs() < f32::EPSILON);
        // Documents the invariant that NSPanel title bars are shorter than
        // regular NSWindow title bars — tripping this means someone
        // retuned one of the constants without updating the other.
        assert!(MACOS_PANEL_TITLE_BAR_HEIGHT < MACOS_TITLE_BAR_HEIGHT);
    }

    #[test]
    fn alert_width_matches_figma_tahoe_kit() {
        // Figma Tahoe UI Kit — stacked alert width is 260pt on desktop.
        assert!((ALERT_WIDTH - 260.0).abs() < f32::EPSILON);
    }

    #[test]
    fn popover_and_hover_card_share_max_width() {
        assert!((POPOVER_MAX_WIDTH - 320.0).abs() < f32::EPSILON);
        assert!((HOVER_CARD_MAX_WIDTH - POPOVER_MAX_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn sidebar_min_width_matches_hig() {
        // HIG macOS Tahoe — sidebar floor is 180pt (truncation limit at
        // default Dynamic Type). Lowered from 200pt once Tahoe shipped
        // Liquid Glass source-list styling.
        assert!((SIDEBAR_MIN_WIDTH - 180.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sidebar_default_width_matches_hig() {
        // HIG macOS Tahoe — primary column default is 220pt (midpoint of
        // the 180–320pt typical range).
        assert!((super::SIDEBAR_DEFAULT_WIDTH - 220.0).abs() < f32::EPSILON);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn inspector_default_width_matches_hig() {
        // HIG macOS Tahoe — standard inspector column default is 250pt;
        // Pro-app variants (Xcode) use the wider INSPECTOR_PANEL_WIDTH (320pt).
        assert!((super::INSPECTOR_DEFAULT_WIDTH - 250.0).abs() < f32::EPSILON);
        assert!(super::INSPECTOR_DEFAULT_WIDTH < super::INSPECTOR_PANEL_WIDTH);
    }

    #[test]
    fn size_class_classification() {
        assert_eq!(
            Platform::IOS.size_class_for_width(375.0),
            SizeClass::Compact
        );
        assert_eq!(
            Platform::IOS.size_class_for_width(820.0),
            SizeClass::Regular
        );
        assert_eq!(
            Platform::MacOS.size_class_for_width(100.0),
            SizeClass::Regular
        );
        assert_eq!(
            Platform::WatchOS.size_class_for_width(500.0),
            SizeClass::Compact
        );
    }

    #[test]
    fn tvos_grid_columns_widths() {
        assert!((TvOsGridColumns::Two.unfocused_width() - 860.0).abs() < f32::EPSILON);
        assert!((TvOsGridColumns::Three.unfocused_width() - 560.0).abs() < f32::EPSILON);
        assert!((TvOsGridColumns::Four.unfocused_width() - 410.0).abs() < f32::EPSILON);
        assert!((TvOsGridColumns::Nine.unfocused_width() - 160.0).abs() < f32::EPSILON);
        assert_eq!(TvOsGridColumns::Seven.count(), 7);
    }

    #[test]
    fn watchos_sizes_match_hig() {
        assert!((WatchOsSize::Mm42.width_px() - 374.0).abs() < f32::EPSILON);
        assert!((WatchOsSize::Mm42.height_px() - 446.0).abs() < f32::EPSILON);
        assert!((WatchOsSize::Mm46.width_px() - 416.0).abs() < f32::EPSILON);
        assert!((WatchOsSize::Mm49.width_px() - 422.0).abs() < f32::EPSILON);
    }

    #[test]
    fn surface_roles_classify_liquid_glass() {
        assert!(SurfaceRole::WindowChrome.uses_liquid_glass());
        assert!(SurfaceRole::Sidebar.uses_liquid_glass());
        assert!(SurfaceRole::FloatingPanel.uses_liquid_glass());
        assert!(!SurfaceRole::Content.uses_liquid_glass());
    }

    #[test]
    fn flex_helpers_compile_as_free_functions() {
        // Invocation coverage. Real layout is validated by the example
        // galleries and visual snapshots — here we only check that the
        // free-function form Zed ships compiles the same way.
        let _row = super::h_flex();
        let _col = super::v_flex();
    }

    #[test]
    fn flex_ext_is_available_on_div() {
        use super::FlexExt;
        use gpui::div;
        // FlexExt is blanket-impl'd for any Styled + Sized, so `div()`
        // should pick up both methods. Calling them returns the same
        // type so chaining with other modifiers stays ergonomic.
        let _row = div().h_flex();
        let _col = div().v_flex();
    }

    #[test]
    fn dropdown_snap_margin_matches_zed_convention() {
        use super::{DROPDOWN_SNAP_MARGIN, snap_to_window_margin};
        let m: f32 = DROPDOWN_SNAP_MARGIN.into();
        // Zed's context menu uses 8pt — Finding 12 requires us to match.
        assert!((m - 8.0).abs() < f32::EPSILON);
        // The wrapper helper returns the identical value so callers can
        // rely on either form interchangeably.
        let wrapped: f32 = snap_to_window_margin().into();
        assert_eq!(wrapped, m);
    }

    #[test]
    fn spacing_ladder_is_strictly_increasing_and_4pt_multiples() {
        use super::{
            SPACING_4, SPACING_8, SPACING_12, SPACING_16, SPACING_20, SPACING_24, SPACING_32,
            SPACING_40, SPACING_48,
        };
        let ladder = [
            SPACING_4, SPACING_8, SPACING_12, SPACING_16, SPACING_20, SPACING_24, SPACING_32,
            SPACING_40, SPACING_48,
        ];
        for v in ladder.iter() {
            assert!((v % 4.0).abs() < f32::EPSILON, "{v} is not a 4pt multiple");
        }
        for pair in ladder.windows(2) {
            assert!(
                pair[0] < pair[1],
                "ladder not strictly increasing: {} vs {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn semantic_spacings_map_to_ladder_rungs() {
        use super::{
            CONTENT_MARGIN, CONTENT_MARGIN_WIDE, GROUP_SPACING, SECTION_SPACING, SPACING_8,
            SPACING_16, SPACING_20, SPACING_24,
        };
        assert!((CONTENT_MARGIN - SPACING_16).abs() < f32::EPSILON);
        assert!((CONTENT_MARGIN_WIDE - SPACING_20).abs() < f32::EPSILON);
        assert!((GROUP_SPACING - SPACING_8).abs() < f32::EPSILON);
        assert!((SECTION_SPACING - SPACING_24).abs() < f32::EPSILON);
    }
}
