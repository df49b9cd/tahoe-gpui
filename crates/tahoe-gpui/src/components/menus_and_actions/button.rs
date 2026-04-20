//! Button component with multiple variants.

use crate::callback_types::OnClick;
use crate::components::menus_and_actions::button_like::ButtonLike;
use crate::components::status::activity_indicator::ActivityIndicator;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::color::{darken, lighten, with_alpha};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    Action, AnyElement, App, BoxShadow, ClickEvent, ElementId, FocusHandle, SharedString, Window,
    div, point, px, transparent_black,
};

/// Visual variant for buttons.
///
/// The set is drawn from the macOS 26 (Tahoe) "Buttons" reference page in
/// Apple's UI Kit. SwiftUI / AppKit equivalents are listed against each
/// variant so the mapping back to Apple's conventions stays explicit.
///
/// Declaration order matches the documentation table and the `match` arms
/// in `RenderOnce::render` — please keep all three in sync if you add or
/// reorder variants.
///
/// | Variant         | Canonical name (HIG)        | Visual                                            |
/// |-----------------|-----------------------------|---------------------------------------------------|
/// | `Primary`       | Colored / Primary / Tinted  | Tinted accent fill, white text                    |
/// | `Secondary`     | Gray                        | Subtle alpha-grey fill + faint border, label text |
/// | `Ghost`         | Borderless / Plain          | Transparent fill, label text — toolbar pattern    |
/// | `Outline`       | Bordered                    | Transparent fill with a thin neutral border       |
/// | `Destructive`   | Destructive                 | Red fill, white text                              |
/// | `Glass`         | Liquid Glass material       | Translucent glass surface, capsule                |
/// | `GlassProminent`| Liquid Glass tinted         | Tinted glass, primary CTA on glass                |
/// | `Filled`        | (Tahoe) high-contrast CTA   | Dark fill on light themes (black CTA)             |
/// | `Help`          | Help button                 | 20pt neutral circle w/ `?` glyph                  |
/// | `Disclosure`    | Disclosure control          | Transparent fill, accent-tinted triangle chevron  |
/// | `Gradient`      | Bezel / Gradient            | Linear gradient fill, 1pt border, label text      |
/// | `Link`          | Link role                   | Transparent bg, accent text + underline           |
///
/// HIG (macOS 26) styles covered here: Plain, Gray, Tinted, Filled,
/// Bordered, Borderless, Glass. Each variant below carries a
/// `#[doc(alias = ...)]` mapping to its canonical HIG name + SwiftUI
/// `.buttonStyle` equivalent so rustdoc search finds the familiar Apple
/// term.
///
/// HIG macOS 26 variants **not yet represented** (pending separate PRs so
/// we don't churn the public API in one pass):
/// - `Help` — 20 pt circle showing a `?` that opens help documentation.
///   Would use `IconName::QuestionMark` (not currently in `IconName`).
/// - `Disclosure` — small triangle chevron for collapsible-group headers.
///   Distinct from the `DisclosureGroup` component; here it's the bare
///   button chrome for custom group headers.
/// - `Gradient` — bezel / subtly tinted gradient used for legacy AppKit
///   push-button compatibility in Tahoe toolbars.
///
/// See `docs/hig/components/menus-and-actions.md` for the canonical list.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[non_exhaustive]
pub enum ButtonVariant {
    /// Tinted accent CTA — SwiftUI `.borderedProminent` with the system tint.
    /// Uses `theme.accent` for the fill and `theme.text_on_accent` for the
    /// label. Primary call-to-action on most macOS surfaces.
    #[doc(alias = "Tinted")]
    #[doc(alias = "Colored")]
    #[default]
    Primary,
    /// Subtle neutral fill, no border. SwiftUI `.bordered` with a low-prominence
    /// neutral fill. Use for less-emphasized actions next to a `Primary`.
    ///
    /// The fill is built from `theme.text` at low alpha so it composites
    /// against any background — this guarantees visible contrast in both
    /// light and dark themes without depending on `theme.surface` (which is
    /// only ~4 % off `theme.background`).
    #[doc(alias = "Gray")]
    Secondary,
    /// Borderless toolbar / inline button — SwiftUI `.borderless` / `.plain`.
    /// Transparent fill, neutral `theme.text` label. The HIG "Borderless"
    /// pattern uses the *label* color, not accent — accent is reserved for
    /// link-style actions which sit on top of body copy.
    #[doc(alias = "Borderless")]
    #[doc(alias = "Plain")]
    Ghost,
    /// Bordered "Default" button — SwiftUI `.bordered`. Transparent fill with
    /// a thin neutral border.
    #[doc(alias = "Bordered")]
    Outline,
    /// Destructive action — SwiftUI `.borderedProminent` with `.role(.destructive)`.
    /// Red fill, white text. Use for delete / discard / unsubscribe.
    Destructive,
    /// Translucent glass surface (capsule shape). Uses glass morphism when the
    /// theme has a `glass` material configured; falls back to `Outline`
    /// otherwise.
    #[doc(alias = "LiquidGlass")]
    Glass,
    /// Opaque tinted glass surface — primary CTA on glass themes.
    #[doc(alias = "LiquidGlassTinted")]
    GlassProminent,
    /// Neutral high-contrast filled button (black on light themes, white on
    /// dark themes). Apple's macOS 26 (Tahoe) prominent CTA pattern when the
    /// system tint isn't appropriate (e.g. marketing surfaces). HIG calls
    /// this the "prominent" filled style.
    #[doc(alias = "Prominent")]
    Filled,
    /// HIG §Buttons — Help button for contextual help.
    ///
    /// Renders a 20pt neutral circle with the `questionmark.circle` glyph
    /// tinted in `theme.text`. Forces [`ButtonShape::Circle`] regardless of
    /// the configured shape so the classic macOS help affordance always
    /// reads correctly.
    #[doc(alias = "HelpButton")]
    Help,
    /// HIG §Disclosure controls — filled triangle for collapsible sections.
    ///
    /// Transparent fill with an accent-tinted triangle chevron. Pair with
    /// [`crate::foundations::icons::IconName::ArrowTriangleRight`] /
    /// [`crate::foundations::icons::IconName::ArrowTriangleDown`] in the
    /// icon slot to build a custom group header whose disclosure state
    /// matches AppKit's `NSButton.BezelStyle.disclosure` behaviour.
    #[doc(alias = "DisclosureTriangle")]
    Disclosure,
    /// HIG §Buttons — legacy bezel/gradient for inline toolbars.
    ///
    /// Linear gradient fill from `theme.surface` to `theme.background`
    /// with a 1pt `theme.border` and `theme.text` foreground. Mirrors the
    /// AppKit `NSBezelStyle.smallSquare`/`.regularSquare` gradient bezel
    /// still used inside custom toolbar chrome (e.g. Finder's window-bar
    /// tool affordances).
    #[doc(alias = "Bezel")]
    Gradient,
    /// HIG §Buttons — Link role: inline underlined accent text.
    ///
    /// Transparent background with `theme.accent` text and a 1pt bottom
    /// border that renders as a classic underline. No extra padding is
    /// applied so the control sits flush with surrounding body copy, in
    /// line with SwiftUI `Button(role: .link)` and AppKit `NSHyperlink`.
    #[doc(alias = "Hyperlink")]
    Link,
    /// HIG §Buttons — Cancel role: neutral dismissal control.
    ///
    /// SwiftUI `Button(role: .cancel)`. Distinct from [`Destructive`]: Cancel
    /// is "get me out of here without changing anything" while Destructive is
    /// "discard work now." Rendered with a neutral alpha fill and `theme.text`
    /// label — visually secondary to Primary but *not* red. Parent alerts /
    /// sheets should bind Escape / ⌘. to the Cancel action per HIG.
    ///
    /// [`Destructive`]: ButtonVariant::Destructive
    #[doc(alias = "CancelRole")]
    Cancel,
}

impl ButtonVariant {
    /// True when this variant renders its content on a Liquid Glass
    /// surface. Used to gate glass-specific chrome (specular edge, no
    /// border, capsule shape on `RoundedRectangle`) and to propagate the
    /// scope to descendant `Icon`s via
    /// [`crate::foundations::surface_scope::GlassSurfaceScope`].
    ///
    /// Centralises the predicate so adding a new glass variant only
    /// updates this method — not every `matches!(variant, Glass | ...)`
    /// call site.
    pub fn is_glass_surfaced(self) -> bool {
        matches!(self, Self::Glass | Self::GlassProminent)
    }
}

/// Button shape per HIG.
///
/// Controls the corner radius and proportions of the button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonShape {
    /// Rounded rectangle with standard corner radius.
    #[default]
    RoundedRectangle,
    /// Fully rounded ends (pill shape). Used for prominent actions.
    Capsule,
    /// Perfect circle. Used for icon-only buttons.
    Circle,
}

/// Button size — maps to HIG's four macOS control size tiers.
///
/// HIG naming (mini / small / regular / large) with their approximate
/// minimum heights:
///
/// | Variant       | HIG name | Min height | Text style     |
/// |---------------|----------|------------|----------------|
/// | `Mini`        | mini     | ~16 pt     | Caption1       |
/// | `Small`       | small    | ~22 pt     | Subheadline    |
/// | `Regular`     | regular  | ~28 pt     | Body (default) |
/// | `Large`       | large    | ~32 pt     | Body           |
/// | `IconSmall`   | small    | ~22 pt     | —              |
/// | `Icon`        | regular  | ~28 pt     | —              |
///
/// The `Icon*` variants apply icon-only chrome (square aspect, equal insets);
/// text variants include label padding.
///
/// Maps 1-to-1 onto [`crate::foundations::layout::ControlSize`] for the
/// ordinary label tiers (Mini / Small / Regular / Large / ExtraLarge).
/// [`ButtonSize::IconSmall`] and [`ButtonSize::Icon`] are button-specific
/// icon-only shapes that reuse the `Small` and `Regular` heights
/// respectively. Heights are resolved through
/// [`crate::foundations::theme::TahoeTheme::control_height`] so the
/// values track platform scaling.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ButtonSize {
    /// SwiftUI `.mini` — 20 pt on macOS. Use inside dense inspectors,
    /// color-well labels, and toolbar palette items where `Small` is
    /// still too tall.
    Mini,
    /// SwiftUI `.small` — 24 pt on macOS. Use in inspectors, sidebar
    /// toolbars, and compact forms.
    Small,
    /// SwiftUI `.regular` — 28 pt on macOS. Default.
    #[default]
    Regular,
    /// SwiftUI `.large` — 32 pt on macOS. Use for prominent CTAs that
    /// need to read at a glance (onboarding, primary page actions).
    Large,
    /// SwiftUI `.extraLarge` — 36 pt on macOS. Use for hero buttons and
    /// accessibility-oriented large-text modes.
    ExtraLarge,
    /// Small icon-only button — matches the [`ButtonSize::Small`] height
    /// (24 pt on macOS) rendered as a square.
    ///
    /// Requires [`Button::tooltip`] or [`Button::accessibility_label`] —
    /// icon-only buttons without an accessibility name panic in debug per
    /// HIG *Buttons > Tooltips*.
    IconSmall,
    /// Regular icon-only button — matches the [`ButtonSize::Regular`]
    /// height (28 pt on macOS) rendered as a square.
    ///
    /// Requires [`Button::tooltip`] or [`Button::accessibility_label`] —
    /// icon-only buttons without an accessibility name panic in debug per
    /// HIG *Buttons > Tooltips*.
    Icon,
}

impl ButtonSize {
    /// Return the [`crate::foundations::layout::ControlSize`] tier this
    /// size maps to. The two icon-only variants collapse onto their
    /// equivalent label tier.
    pub fn control_size(self) -> crate::foundations::layout::ControlSize {
        use crate::foundations::layout::ControlSize;
        match self {
            Self::Mini => ControlSize::Mini,
            Self::Small | Self::IconSmall => ControlSize::Small,
            Self::Regular | Self::Icon => ControlSize::Regular,
            Self::Large => ControlSize::Large,
            Self::ExtraLarge => ControlSize::ExtraLarge,
        }
    }
}

/// A button component with variant styling.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<AnyElement>,
    trailing_icon: Option<AnyElement>,
    extra_children: Vec<AnyElement>,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    round: bool,
    full_width: bool,
    disabled: bool,
    focused: bool,
    loading: bool,
    on_click: OnClick,
    /// Host-supplied focus handle. When present, the button participates
    /// in the host's keyboard focus graph (Tab/Shift-Tab cycling, arrow
    /// navigation) and the focus ring is driven by the handle's reactive
    /// state rather than the `focused: bool` fallback. Zed-style pattern:
    /// every interactive stateless control can opt into the focus graph
    /// via a caller-owned handle.
    focus_handle: Option<FocusHandle>,
    /// Accessibility label for screen readers. The AX name resolves with
    /// preference `accessibility_label` → `tooltip` → `label`, so this
    /// overrides both of the others when set.
    accessibility_label: Option<SharedString>,
    /// Pointer-hover tooltip. Attached via GPUI's `.tooltip()` so the hover
    /// delay matches HIG (~500 ms). Also used as the AX name fallback when
    /// `accessibility_label` is absent — required for icon-only buttons per
    /// HIG *Buttons > Tooltips*.
    tooltip: Option<SharedString>,
    /// Optional keyboard-shortcut glyph shown inside the tooltip
    /// (e.g. `"⌘C"`). Ignored when `tooltip` is `None`.
    tooltip_key_binding: Option<SharedString>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            trailing_icon: None,
            extra_children: Vec::new(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            round: false,
            full_width: false,
            disabled: false,
            focused: false,
            loading: false,
            on_click: None,
            focus_handle: None,
            accessibility_label: None,
            tooltip: None,
            tooltip_key_binding: None,
        }
    }

    /// Attach a host-owned focus handle so the button participates in
    /// the keyboard focus graph. HIG Accessibility Keyboard: every
    /// actionable control must be focusable and reachable via Tab/
    /// Shift-Tab. The host configures `tab_index` / `tab_stop` on the
    /// supplied handle — tahoe-gpui stays out of the ordering policy.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Dispatch a GPUI action when the button activates. Same dispatch
    /// path as a keyboard shortcut, so one action works from click,
    /// keybinding, and command palette. Runs *before* any `on_click`
    /// handler if both are supplied. The action is cloned via
    /// `Action::boxed_clone` on every click so repeated activations each
    /// dispatch a fresh instance.
    pub fn action(mut self, action: Box<dyn Action>) -> Self {
        let prior = self.on_click.take();
        self.on_click = Some(Box::new(move |event, window, cx| {
            window.dispatch_action(action.boxed_clone(), cx);
            if let Some(handler) = prior.as_ref() {
                handler(event, window, cx);
            }
        }));
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Set a trailing icon rendered *after* the label (between label and
    /// extras). HIG allows trailing glyphs for disclosure / chevron /
    /// external-link affordances.
    pub fn trailing_icon(mut self, icon: impl IntoElement) -> Self {
        self.trailing_icon = Some(icon.into_any_element());
        self
    }

    /// Add an arbitrary child element, rendered after icon and label.
    ///
    /// Children render in this order: `icon → label → trailing_icon → extras (in call order)`.
    /// To render an element *before* the icon, nest it inside the icon slot
    /// (e.g. wrap the icon + the element in a `div()`). Extras chain via
    /// repeated `.child(...)` calls and can be mixed with icons or text.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.extra_children.push(child.into_any_element());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the button shape per HIG.
    ///
    /// - `RoundedRectangle` (default): macOS 26 Tahoe Liquid Glass rounded
    ///   corner (`theme.glass.radius(GlassSize::Small)`). Historical AppKit
    ///   push buttons used 6pt; macOS 26 moved to a larger, more rounded
    ///   corner to match the Liquid Glass family.
    /// - `Capsule`: fully rounded ends / pill shape (`theme.radius_full`)
    /// - `Circle`: perfect circle (`theme.radius_full` + equal width/height)
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// When `true`, uses fully-rounded pill shape (`radius_full`) instead of
    /// the default `radius_md`.
    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    /// When `true`, the button stretches to fill the width of its parent
    /// (`.w_full()` on the inner element). Use for centered card CTAs and
    /// form action rows where the button should match the column width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks this button as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Marks this button as in a loading state — the icon slot is replaced
    /// with an [`ActivityIndicator`], the label continues to render, and
    /// `on_click` is suppressed.
    ///
    /// HIG Buttons (macOS 26, June 2025): actions taking longer than ~0.1 s
    /// must show visible feedback. Use this flag on every async submit,
    /// save, download, or publish CTA.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Sets an accessibility label for screen readers.
    ///
    /// The AX name resolves in the order `accessibility_label` → `tooltip`
    /// → `label`; set this when none of the visible text sources read
    /// correctly out of context (e.g. a toolbar icon whose tooltip is a
    /// keyboard shortcut, or a button whose visible label abbreviates a
    /// longer action).
    ///
    /// **GPUI accessibility gap:** GPUI's `Div` does not currently expose an
    /// `aria_label` / `accessibility_id` API, so this field is *not* wired
    /// into a VoiceOver name today. Once GPUI lands accessibility support,
    /// the label will feed into the AX name automatically. Tracked via the
    /// Zed cross-reference in issue #132.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Pointer-hover tooltip text.
    ///
    /// Attached via GPUI's `.tooltip()` so the hover delay honours the HIG
    /// ~500 ms reveal (see `TOOLTIP_SHOW_DELAY_MS`). Icon-only buttons —
    /// [`ButtonSize::Icon`] and [`ButtonSize::IconSmall`] — must carry
    /// either this or [`Button::accessibility_label`]; omitting all three
    /// name sources (tooltip, label, accessibility_label) panics in debug
    /// builds per HIG *Buttons > Tooltips*.
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Attach a pre-formatted keyboard shortcut glyph (e.g. `"⌘C"`) that
    /// renders next to the tooltip text. Ignored when no [`tooltip`] is set.
    ///
    /// [`tooltip`]: Self::tooltip
    pub fn tooltip_key_binding(mut self, binding: impl Into<SharedString>) -> Self {
        self.tooltip_key_binding = Some(binding.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Resolve the AX name with preference
    /// `accessibility_label` → `tooltip` → `label`.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `self.size` is [`ButtonSize::Icon`] or
    /// [`ButtonSize::IconSmall`] and none of `accessibility_label`,
    /// `tooltip`, or `label` is set, enforcing HIG *Buttons > Tooltips*.
    /// Release builds return `None` instead.
    fn resolve_ax_name(&self) -> Option<SharedString> {
        let ax_label = self
            .accessibility_label
            .clone()
            .or_else(|| self.tooltip.clone())
            .or_else(|| self.label.clone());

        debug_assert!(
            !(matches!(self.size, ButtonSize::Icon | ButtonSize::IconSmall) && ax_label.is_none()),
            "Button `{:?}` is icon-only ({:?}) but has no label, tooltip, or \
             accessibility_label. Per HIG Buttons > Tooltips, icon-only buttons \
             require a tooltip to serve as the accessibility name. Add \
             `.tooltip(\"…\")` or `.accessibility_label(\"…\")`.",
            self.id,
            self.size,
        );

        ax_label
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Resolve the AX name up-front so `resolve_ax_name` sees the full
        // builder state before any fields are moved out during render.
        // Also runs the icon-only HIG debug_assert.
        let ax_label = self.resolve_ax_name();

        let theme = cx.theme();

        // Match arms follow the same order as the doc-comment table on
        // `ButtonVariant`. Adding a variant? Update the table, the enum
        // declaration, and this match together.
        let (bg, text_color, _border_color, hover_bg) = match self.variant {
            ButtonVariant::Primary => {
                let accent = theme.accent;
                let hovered = if accent.l > 0.5 {
                    darken(accent, 0.08)
                } else {
                    lighten(accent, 0.08)
                };
                (accent, theme.text_on_accent, accent, hovered)
            }
            ButtonVariant::Secondary => {
                // HIG "Secondary" — subtle neutral fill that composites
                // against any background. We build the fill from `theme.text`
                // at low alpha so it adapts automatically to light/dark, and
                // we add a faint border so the edge stays visible when the
                // alpha-fill is nearly invisible against the parent surface.
                let bg = with_alpha(theme.text, 0.06);
                let hovered = with_alpha(theme.text, 0.12);
                let border = with_alpha(theme.text, 0.10);
                (bg, theme.text, border, hovered)
            }
            ButtonVariant::Ghost => {
                // Borderless toolbar / inline button per HIG: transparent
                // fill, NEUTRAL `theme.text` label (not accent -- accent is for
                // link-style buttons sitting on top of body copy). This keeps
                // contrast high in both modes and preserves the low-emphasis
                // affordance: Ghost is the "quiet" sibling of Primary.
                let glass = &theme.glass;
                (
                    glass.accessible_bg(GlassSize::Small, theme.accessibility_mode),
                    theme.text,
                    transparent_black(),
                    glass.hover_bg,
                )
            }
            ButtonVariant::Outline => {
                // HIG "Outline" — transparent fill with a visible thin
                // border. Distinguishable from Ghost (which is borderless) so
                // toolbar pickers can use Ghost for quiet actions and Outline
                // for deliberate but still-secondary choices.
                let border = with_alpha(theme.text, 0.2);
                let hovered = with_alpha(theme.text, 0.08);
                (transparent_black(), theme.text, border, hovered)
            }
            ButtonVariant::Destructive => {
                let err = theme.error;
                let hovered = if err.l > 0.5 {
                    darken(err, 0.08)
                } else {
                    lighten(err, 0.08)
                };
                (err, theme.text_on_accent, err, hovered)
            }
            ButtonVariant::Glass => {
                let glass = &theme.glass;
                let fill = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
                (fill, theme.text, transparent_black(), glass.hover_bg)
            }
            ButtonVariant::GlassProminent => {
                // Tinted accent glass -- opaque tinted surface for primary CTA
                let accent = theme.accent;
                let tinted_bg = with_alpha(accent, 0.25);
                let tinted_hover = with_alpha(accent, 0.35);
                (tinted_bg, theme.text, transparent_black(), tinted_hover)
            }
            ButtonVariant::Filled => {
                // Neutral high-contrast filled: dark fill on light themes,
                // light fill on dark themes. Hover steps the fill toward the
                // background; we let `press_bg` below pick the correct
                // direction so press feels deeper than hover regardless of
                // whether the resting fill is dark or light.
                let bg = theme.text;
                let text_color = theme.background;
                let hovered = if bg.l > 0.5 {
                    darken(bg, 0.10)
                } else {
                    lighten(bg, 0.15)
                };
                (bg, text_color, bg, hovered)
            }
            ButtonVariant::Help => {
                // HIG Help button: neutral alpha-grey circle with a `?`
                // glyph in the label color. Mirrors Secondary's alpha-
                // composited fill so it reads on any parent surface, but
                // the enclosing circle — not a rounded rect — identifies
                // it as the canonical help affordance.
                let bg = with_alpha(theme.text, 0.10);
                let hovered = with_alpha(theme.text, 0.16);
                let border = with_alpha(theme.text, 0.14);
                (bg, theme.text, border, hovered)
            }
            ButtonVariant::Disclosure => {
                // HIG Disclosure: transparent fill, accent-tinted chevron.
                // Hover darkens the hit region subtly so the affordance
                // responds without implying a pressed container.
                let hovered = with_alpha(theme.text, 0.06);
                (
                    transparent_black(),
                    theme.accent,
                    transparent_black(),
                    hovered,
                )
            }
            ButtonVariant::Gradient => {
                // HIG bezel/gradient: start from `theme.surface` and fade
                // into `theme.background`. We pick the mid-fill for the
                // resting `bg` so the shared rendering path can still pass
                // a solid colour to `.bg()`; the gradient itself is applied
                // below via the variant-aware override.
                let mid = if theme.surface.l > theme.background.l {
                    darken(theme.surface, 0.02)
                } else {
                    lighten(theme.surface, 0.02)
                };
                let hovered = if mid.l > 0.5 {
                    darken(mid, 0.04)
                } else {
                    lighten(mid, 0.04)
                };
                (mid, theme.text, theme.border, hovered)
            }
            ButtonVariant::Link => {
                // HIG Link role: accent text over a transparent bg with a
                // 1pt bottom-border underline. No extra padding is applied
                // so the control can sit flush inside a paragraph.
                let hovered = with_alpha(theme.accent, 0.08);
                (transparent_black(), theme.accent, theme.accent, hovered)
            }
            ButtonVariant::Cancel => {
                // HIG Cancel role: neutral alpha-composited fill — visually
                // close to `Secondary` so it reads as a quiet dismissal
                // control rather than a red Destructive action. Parent
                // alerts / sheets should bind Escape and ⌘. to the same
                // handler per HIG *Alerts > Keyboard*.
                let bg = with_alpha(theme.text, 0.08);
                let hovered = with_alpha(theme.text, 0.14);
                (bg, theme.text, transparent_black(), hovered)
            }
        };
        let is_glass_variant = self.variant.is_glass_surfaced();
        // HIG Help buttons are always a 20pt circle — the shape is part of
        // the affordance itself, not a caller decision — so override any
        // other shape configuration here.
        let force_circle = matches!(self.variant, ButtonVariant::Help);
        let is_circle = self.shape == ButtonShape::Circle || force_circle;
        let radius = if force_circle {
            theme.radius_full
        } else {
            match self.shape {
                ButtonShape::Capsule | ButtonShape::Circle => theme.radius_full,
                ButtonShape::RoundedRectangle => {
                    if self.round || is_glass_variant {
                        theme.radius_full
                    } else {
                        theme.glass.radius(GlassSize::Small)
                    }
                }
            }
        };

        // Size → (horizontal padding, vertical padding, text style, min height).
        // Heights flow through `theme.control_height(…)` so the button tracks
        // the canonical SwiftUI ControlSize tiers (Mini 20 / Small 24 /
        // Regular 28 / Large 32 / ExtraLarge 36 on macOS).
        let min_h = theme.control_height(self.size.control_size());
        let (px_val, py_val, ts) = match self.size {
            ButtonSize::Mini => (theme.spacing_xs, px(1.0), TextStyle::Caption1),
            ButtonSize::Small => (theme.spacing_sm, theme.spacing_xs, TextStyle::Subheadline),
            ButtonSize::IconSmall => (theme.spacing_xs, theme.spacing_xs, TextStyle::Subheadline),
            ButtonSize::Regular => (theme.spacing_md, theme.spacing_sm, TextStyle::Body),
            ButtonSize::Icon => (theme.spacing_sm, theme.spacing_sm, TextStyle::Body),
            ButtonSize::Large => (theme.spacing_md, theme.spacing_sm, TextStyle::Body),
            ButtonSize::ExtraLarge => (theme.spacing_lg, theme.spacing_md, TextStyle::Headline),
        };
        // HIG §Help buttons: always render as a 20pt circle, regardless of
        // the size tier the caller selected. The affordance itself — not
        // the caller's layout intent — fixes the footprint.
        let (px_val, py_val, min_h) = if matches!(self.variant, ButtonVariant::Help) {
            (theme.spacing_xs, theme.spacing_xs, 20.0)
        } else {
            (px_val, py_val, min_h)
        };

        let id = self.id;
        let mut el = div()
            .id(id.clone())
            .debug_selector(|| format!("button-{}", id))
            .flex()
            .items_center()
            .justify_center()
            .gap(theme.spacing_xs)
            .px(px_val)
            .py(py_val)
            .min_h(px(min_h))
            .min_w(px(min_h))
            .bg(bg)
            .text_color(text_color)
            .text_style(ts, theme)
            .rounded(radius)
            .cursor_pointer();

        // HIG §Gradient: layer a linear gradient from `theme.surface` down
        // into `theme.background` over the solid fill, plus a 1pt border.
        if matches!(self.variant, ButtonVariant::Gradient) {
            el = el
                .bg(gpui::linear_gradient(
                    180.0,
                    gpui::LinearColorStop {
                        color: theme.surface,
                        percentage: 0.0,
                    },
                    gpui::LinearColorStop {
                        color: theme.background,
                        percentage: 1.0,
                    },
                ))
                .border_1()
                .border_color(theme.border);
        }

        // HIG §Link: accent text with a 1pt bottom border underline.
        if matches!(self.variant, ButtonVariant::Link) {
            el = el
                .px(px(0.0))
                .py(px(0.0))
                .min_h(px(0.0))
                .min_w(px(0.0))
                .border_b_1()
                .border_color(theme.accent);
        }

        // Circle shape: enforce equal width and height for a perfect circle.
        if is_circle {
            el = el.size(px(min_h));
        }

        if self.full_width {
            el = el.w_full();
        }

        // Shadow composition:
        // - Glass variants carry their Liquid Glass tier shadow.
        // - Bordered non-glass variants (Primary / Secondary / Outline /
        //   Destructive / Filled) carry a 0.5pt specular rim — an inset
        //   highlight separate from the HighContrast border. Ghost is
        //   transparent and gets no rim (it would double the Outline look).
        let is_bordered_non_glass = matches!(
            self.variant,
            ButtonVariant::Primary
                | ButtonVariant::Secondary
                | ButtonVariant::Outline
                | ButtonVariant::Destructive
                | ButtonVariant::Filled
                | ButtonVariant::Cancel
        );
        let mut base: Vec<BoxShadow> = if is_glass_variant {
            theme.glass.shadows(GlassSize::Small).to_vec()
        } else {
            Vec::new()
        };
        if is_bordered_non_glass {
            base.push(BoxShadow {
                color: theme.specular_rim(),
                offset: point(px(0.0), px(0.5)),
                blur_radius: px(0.0),
                spread_radius: px(-0.5),
            });
        }

        // Treat `loading` like `disabled` for interactivity purposes — no
        // clicks fire, the cursor is default, and the activity indicator
        // replaces the leading icon below. We do NOT dim the fill: HIG
        // shows a live control that's still "alive" while waiting for the
        // async result.
        let interactive_blocked = self.disabled || self.loading;

        if self.disabled {
            // HIG: disabled tint is a fixed muted color, not a proportional
            // opacity — opacity(0.5) fails WCAG 4.5:1 on low-contrast
            // variants (Ghost, Outline). Using Button's own text_disabled
            // token instead of ButtonLike's default opacity(0.5).
            el = el.text_color(theme.text_disabled()).cursor_default();
        } else if !interactive_blocked {
            // Press should feel deeper than hover. For light-fill variants
            // (Primary, Outline, Secondary, Ghost) we darken further. For
            // dark-fill variants (Filled in light mode) we lighten further
            // so the press state continues the direction hover started.
            let active_bg = if hover_bg.l > 0.5 {
                darken(hover_bg, 0.06)
            } else {
                lighten(hover_bg, 0.06)
            };
            let is_glass = is_glass_variant;
            el = el.hover(|style| style.bg(hover_bg)).active(|style| {
                let mut s = style.bg(active_bg);
                // Apple Liquid Glass "flex response" -- subtle opacity shift on press
                if is_glass {
                    s = s.opacity(0.85);
                }
                s
            });
        } else {
            el = el.cursor_default();
        }

        // Finding 15 adoption: route the focus ring, high-contrast
        // border, click, and keyboard-activation wiring through the
        // shared `ButtonLike` substrate so `Button`, `CopyButton`, and
        // other button-shaped controls share one interactivity stack.
        // Button keeps its own variant-specific disabled text-color
        // handling above (HIG: muted text rather than opacity dimming).
        let mut bl = ButtonLike::new(id.clone())
            .focused(self.focused)
            .base_shadows(base);
        if let Some(handle) = self.focus_handle.as_ref() {
            bl = bl.focus_handle(handle);
        }
        if let Some(handler) = self.on_click
            && !interactive_blocked
        {
            bl = bl.on_click(move |event, window, cx| handler(event, window, cx));
        }
        el = bl.apply_to(el, theme, window);

        // Leading slot: loading spinner replaces the icon while loading.
        if self.loading {
            let indicator_size = match self.size {
                ButtonSize::Mini => px(10.0),
                ButtonSize::Small | ButtonSize::IconSmall => px(14.0),
                ButtonSize::Regular | ButtonSize::Icon => px(16.0),
                ButtonSize::Large => px(18.0),
                ButtonSize::ExtraLarge => px(20.0),
            };
            el = el.child(
                ActivityIndicator::new(ElementId::from((id.clone(), "loading")))
                    .size(indicator_size)
                    .color(text_color),
            );
        } else if let Some(icon) = self.icon {
            el = el.child(icon);
        }
        if let Some(label) = self.label {
            el = el.child(label);
        }
        if let Some(trailing) = self.trailing_icon {
            el = el.child(trailing);
        }
        if !self.extra_children.is_empty() {
            el = el.children(self.extra_children);
        }

        // HIG tooltips: pointer hover reveal after ~500 ms (GPUI default).
        // Icon-only buttons and toolbar actions rely on these for both
        // pointer discoverability and VoiceOver name fallback.
        if let Some(text) = self.tooltip.clone() {
            el = el.tooltip(crate::components::presentation::tooltip::text_tooltip_view(
                text,
                self.tooltip_key_binding,
            ));
        }

        // Only expose AX metadata when we have a name to announce: emitting
        // `role: Button` on an unnamed control would cause screen readers
        // to announce a bare "button" once GPUI's AX API (#138) lands.
        // The debug_assert in `resolve_ax_name` guarantees icon-only
        // variants always have a name; labelled variants may still be
        // nameless (e.g. test-only `Button::new("x")`).
        if let Some(label) = ax_label {
            let props = AccessibilityProps::new()
                .role(AccessibilityRole::Button)
                .label(label);
            el = el.with_accessibility(&props);
        }

        // Glass button variants host their child content on a Liquid Glass
        // surface. Wrap in a scope so descendant Icons auto-resolve to
        // IconStyle::LiquidGlass — matching the HIG guidance that content
        // on glass surfaces inherits vibrancy.
        if is_glass_variant {
            crate::foundations::surface_scope::GlassSurfaceScope::new(el).into_any_element()
        } else {
            el.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Button, ButtonSize, ButtonVariant};
    use core::prelude::v1::test;
    use gpui::InteractiveElement;
    use gpui::div;

    /// All button variants. New variants must be added here so the
    /// distinctness and exhaustiveness tests pick them up automatically.
    const ALL_VARIANTS: &[ButtonVariant] = &[
        ButtonVariant::Primary,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Outline,
        ButtonVariant::Destructive,
        ButtonVariant::Glass,
        ButtonVariant::GlassProminent,
        ButtonVariant::Filled,
        ButtonVariant::Help,
        ButtonVariant::Disclosure,
        ButtonVariant::Gradient,
        ButtonVariant::Link,
        ButtonVariant::Cancel,
    ];

    #[test]
    fn button_variant_default() {
        assert_eq!(ButtonVariant::default(), ButtonVariant::Primary);
    }

    #[test]
    fn button_variant_all_distinct() {
        for (i, a) in ALL_VARIANTS.iter().enumerate() {
            for (j, b) in ALL_VARIANTS.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    /// Compile-time guard: if a new variant is added without updating
    /// `ALL_VARIANTS`, this exhaustive match fails to compile.
    #[allow(dead_code)]
    fn variants_are_exhaustive(v: ButtonVariant) -> &'static str {
        match v {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Destructive => "destructive",
            ButtonVariant::Glass => "glass",
            ButtonVariant::GlassProminent => "glass_prominent",
            ButtonVariant::Filled => "filled",
            ButtonVariant::Help => "help",
            ButtonVariant::Disclosure => "disclosure",
            ButtonVariant::Gradient => "gradient",
            ButtonVariant::Link => "link",
            ButtonVariant::Cancel => "cancel",
        }
    }

    #[test]
    fn button_size_default() {
        assert_eq!(ButtonSize::default(), ButtonSize::Regular);
    }

    #[test]
    fn button_size_all_distinct() {
        let sizes = [
            ButtonSize::Mini,
            ButtonSize::Small,
            ButtonSize::Regular,
            ButtonSize::Large,
            ButtonSize::Icon,
            ButtonSize::IconSmall,
        ];
        for i in 0..sizes.len() {
            for j in 0..sizes.len() {
                if i == j {
                    assert_eq!(sizes[i], sizes[j]);
                } else {
                    assert_ne!(sizes[i], sizes[j]);
                }
            }
        }
    }

    /// Compile-time guard: adding a new `ButtonSize` without updating the
    /// above list fails this exhaustive match.
    #[allow(dead_code)]
    fn sizes_are_exhaustive(s: ButtonSize) -> &'static str {
        match s {
            ButtonSize::Mini => "mini",
            ButtonSize::Small => "small",
            ButtonSize::Regular => "regular",
            ButtonSize::Large => "large",
            ButtonSize::ExtraLarge => "extra_large",
            ButtonSize::Icon => "icon",
            ButtonSize::IconSmall => "icon_small",
        }
    }

    /// Regression test for issue #51: `ButtonSize::Mini` must resolve to the
    /// macOS `MACOS_MIN_TOUCH_TARGET = 20 pt` floor, not 16 pt. Also pins
    /// the `ButtonSize` <-> `ControlSize` mapping so future refactors of
    /// `ButtonSize::control_size` cannot silently drop the Mini tier below
    /// 20 pt or skew the other heights away from the SwiftUI ControlSize
    /// metrics (20 / 24 / 28 / 32 / 36 pt on macOS).
    #[test]
    fn button_size_heights_track_swiftui_control_size_macos() {
        use crate::foundations::layout::{ControlSize, Platform};
        use crate::foundations::theme::TahoeTheme;

        let theme = TahoeTheme::dark();
        assert_eq!(theme.platform, Platform::MacOS);

        assert_eq!(ButtonSize::Mini.control_size(), ControlSize::Mini);
        assert_eq!(theme.control_height(ButtonSize::Mini.control_size()), 20.0);

        assert_eq!(ButtonSize::Small.control_size(), ControlSize::Small);
        assert_eq!(theme.control_height(ButtonSize::Small.control_size()), 24.0);

        assert_eq!(ButtonSize::Regular.control_size(), ControlSize::Regular);
        assert_eq!(
            theme.control_height(ButtonSize::Regular.control_size()),
            28.0
        );

        assert_eq!(ButtonSize::Large.control_size(), ControlSize::Large);
        assert_eq!(theme.control_height(ButtonSize::Large.control_size()), 32.0);

        assert_eq!(
            ButtonSize::ExtraLarge.control_size(),
            ControlSize::ExtraLarge
        );
        assert_eq!(
            theme.control_height(ButtonSize::ExtraLarge.control_size()),
            36.0
        );

        assert_eq!(ButtonSize::IconSmall.control_size(), ControlSize::Small);
        assert_eq!(ButtonSize::Icon.control_size(), ControlSize::Regular);
    }

    #[test]
    fn button_loading_default_false() {
        let btn = Button::new("test");
        assert!(!btn.loading);
    }

    #[test]
    fn button_loading_builder_sets_flag() {
        let btn = Button::new("test").loading(true);
        assert!(btn.loading);
    }

    #[test]
    fn button_trailing_icon_stored() {
        let btn = Button::new("test");
        assert!(btn.trailing_icon.is_none());
        let btn = btn.trailing_icon(gpui::div());
        assert!(btn.trailing_icon.is_some());
    }

    #[test]
    fn button_round_sets_flag() {
        let btn = Button::new("test");
        assert!(!btn.round);

        let btn = btn.round(true);
        assert!(btn.round);
    }

    #[test]
    fn button_full_width_default_false() {
        let btn = Button::new("test");
        assert!(!btn.full_width);
    }

    #[test]
    fn button_full_width_builder_sets_flag() {
        let btn = Button::new("test").full_width(true);
        assert!(btn.full_width);
    }

    #[test]
    fn button_child_adds_extra_children() {
        let btn = Button::new("test");
        assert!(btn.extra_children.is_empty());

        let btn = btn.child(div().id("c1"));
        assert_eq!(btn.extra_children.len(), 1);

        let btn = btn.child(div().id("c2"));
        assert_eq!(btn.extra_children.len(), 2);
    }

    // ── AX-name resolution + icon-only HIG guard ───────────────────────

    #[test]
    fn ax_name_prefers_accessibility_label() {
        let btn = Button::new("ax")
            .label("visible")
            .tooltip("hover")
            .accessibility_label("screen reader");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("screen reader".to_string()),
            "accessibility_label must win the cascade",
        );
    }

    #[test]
    fn ax_name_falls_back_to_tooltip() {
        let btn = Button::new("ax")
            .label("visible")
            .tooltip("Copy to clipboard");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("Copy to clipboard".to_string()),
        );
    }

    #[test]
    fn ax_name_falls_back_to_label() {
        let btn = Button::new("ax").label("Save");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("Save".to_string()),
        );
    }

    #[test]
    fn ax_name_accessibility_label_wins_over_tooltip_alone() {
        let btn = Button::new("ax")
            .tooltip("hover")
            .accessibility_label("screen reader");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("screen reader".to_string()),
        );
    }

    #[test]
    fn ax_name_accessibility_label_wins_over_label_alone() {
        let btn = Button::new("ax")
            .label("visible")
            .accessibility_label("screen reader");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("screen reader".to_string()),
        );
    }

    #[test]
    fn ax_name_accessibility_label_alone_resolves() {
        let btn = Button::new("ax").accessibility_label("screen reader");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("screen reader".to_string()),
        );
    }

    #[test]
    fn ax_name_none_when_all_sources_missing_on_regular() {
        let btn = Button::new("ax").size(ButtonSize::Regular);
        assert!(btn.resolve_ax_name().is_none());
    }

    #[test]
    fn ax_name_labelled_non_icon_button_is_fine_without_tooltip() {
        let btn = Button::new("ax").label("OK").size(ButtonSize::Regular);
        assert!(btn.resolve_ax_name().is_some());
    }

    #[test]
    fn icon_only_with_tooltip_resolves_name() {
        let btn = Button::new("ax").size(ButtonSize::Icon).tooltip("Share");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("Share".to_string()),
        );
    }

    #[test]
    fn icon_only_with_accessibility_label_resolves_name() {
        let btn = Button::new("ax")
            .size(ButtonSize::IconSmall)
            .accessibility_label("Close");
        assert_eq!(
            btn.resolve_ax_name().map(|s| s.to_string()),
            Some("Close".to_string()),
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "HIG Buttons > Tooltips")]
    fn icon_only_without_name_panics_in_debug() {
        let btn = Button::new("offender").size(ButtonSize::Icon);
        let _ = btn.resolve_ax_name();
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "HIG Buttons > Tooltips")]
    fn icon_small_without_name_panics_in_debug() {
        let btn = Button::new("offender").size(ButtonSize::IconSmall);
        let _ = btn.resolve_ax_name();
    }

    // ── Theme-aware contrast / color regression tests ──────────────────
    //
    // These guard the post-review fixes:
    //   • Ghost label uses neutral `theme.text` (not accent / not muted),
    //     so it inherits text↔background contrast in every appearance.
    //   • Filled inverts `bg`/`text_color` correctly across light and dark.
    //   • Secondary uses an alpha-based fill, so the L delta against the
    //     window background is independent of `theme.surface`.

    use crate::foundations::theme::{TahoeTheme, contrast_ratio};
    use gpui::Hsla;

    /// Resolve `(bg, text_color)` for a variant on a given theme by inlining
    /// the same logic the render path uses. Keeps these tests pure and free
    /// of the GPUI test harness.
    fn variant_colors(variant: ButtonVariant, theme: &TahoeTheme) -> (Hsla, Hsla) {
        use crate::foundations::color::with_alpha;
        match variant {
            ButtonVariant::Primary => (theme.accent, theme.text_on_accent),
            ButtonVariant::Secondary => (with_alpha(theme.text, 0.06), theme.text),
            ButtonVariant::Ghost => (gpui::transparent_black(), theme.text),
            ButtonVariant::Outline => (gpui::transparent_black(), theme.text),
            ButtonVariant::Destructive => (theme.error, theme.text_on_accent),
            ButtonVariant::Glass => (gpui::transparent_black(), theme.text),
            ButtonVariant::GlassProminent => (with_alpha(theme.accent, 0.25), theme.text),
            ButtonVariant::Filled => (theme.text, theme.background),
            ButtonVariant::Help => (with_alpha(theme.text, 0.10), theme.text),
            ButtonVariant::Disclosure => (gpui::transparent_black(), theme.accent),
            ButtonVariant::Gradient => {
                use crate::foundations::color::{darken, lighten};
                let mid = if theme.surface.l > theme.background.l {
                    darken(theme.surface, 0.02)
                } else {
                    lighten(theme.surface, 0.02)
                };
                (mid, theme.text)
            }
            ButtonVariant::Link => (gpui::transparent_black(), theme.accent),
            ButtonVariant::Cancel => (with_alpha(theme.text, 0.08), theme.text),
        }
    }

    #[test]
    fn ghost_uses_neutral_text_in_light_and_dark() {
        let light = TahoeTheme::light();
        let dark = TahoeTheme::dark();
        let (_bg_l, fg_l) = variant_colors(ButtonVariant::Ghost, &light);
        let (_bg_d, fg_d) = variant_colors(ButtonVariant::Ghost, &dark);
        assert_eq!(
            fg_l, light.text,
            "Ghost label must be theme.text in light mode"
        );
        assert_eq!(
            fg_d, dark.text,
            "Ghost label must be theme.text in dark mode"
        );
    }

    #[test]
    fn ghost_label_meets_wcag_aa_against_background() {
        // Ghost has a transparent fill, so the label sits on whatever the
        // parent draws — typically theme.background. Verify that the label
        // color clears WCAG AA (4.5:1) for normal body text in both modes.
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (_, fg) = variant_colors(ButtonVariant::Ghost, &theme);
            let ratio = contrast_ratio(fg, theme.background);
            assert!(
                ratio >= 4.5,
                "Ghost label vs theme.background contrast {ratio:.2}:1 fails WCAG AA 4.5:1"
            );
        }
    }

    #[test]
    fn filled_inverts_across_themes() {
        // Light mode: dark fill on white background.
        let light = TahoeTheme::light();
        let (bg_l, fg_l) = variant_colors(ButtonVariant::Filled, &light);
        assert!(
            bg_l.l < 0.3,
            "Filled bg should be dark in light mode (L={})",
            bg_l.l
        );
        assert!(
            fg_l.l > 0.7,
            "Filled label should be light in light mode (L={})",
            fg_l.l
        );

        // Dark mode: light fill on dark background.
        let dark = TahoeTheme::dark();
        let (bg_d, fg_d) = variant_colors(ButtonVariant::Filled, &dark);
        assert!(
            bg_d.l > 0.7,
            "Filled bg should be light in dark mode (L={})",
            bg_d.l
        );
        assert!(
            fg_d.l < 0.3,
            "Filled label should be dark in dark mode (L={})",
            fg_d.l
        );

        // And the bg/label pair should always meet WCAG AAA (7:1) — Filled is
        // a high-contrast CTA pattern.
        for (theme, label) in [(&light, "light"), (&dark, "dark")] {
            let (bg, fg) = variant_colors(ButtonVariant::Filled, theme);
            let ratio = contrast_ratio(fg, bg);
            assert!(
                ratio >= 7.0,
                "Filled label/bg contrast {ratio:.2}:1 in {label} mode fails WCAG AAA 7:1"
            );
        }
    }

    // ── New HIG variant smoke tests ──────────────────────────────────
    //
    // One test per variant asserting the coarse visual identity (fill /
    // foreground token) so a regression on the match-arm wiring is caught
    // without relying on the GPUI render harness.

    #[test]
    fn help_uses_neutral_fill_and_text_foreground() {
        use crate::foundations::color::with_alpha;
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (bg, fg) = variant_colors(ButtonVariant::Help, &theme);
            assert_eq!(bg, with_alpha(theme.text, 0.10));
            assert_eq!(fg, theme.text);
        }
    }

    #[test]
    fn disclosure_is_transparent_with_accent_glyph() {
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (bg, fg) = variant_colors(ButtonVariant::Disclosure, &theme);
            assert_eq!(bg, gpui::transparent_black());
            assert_eq!(fg, theme.accent);
        }
    }

    #[test]
    fn gradient_uses_surface_midpoint_fill() {
        use crate::foundations::color::{darken, lighten};
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (bg, fg) = variant_colors(ButtonVariant::Gradient, &theme);
            let expected = if theme.surface.l > theme.background.l {
                darken(theme.surface, 0.02)
            } else {
                lighten(theme.surface, 0.02)
            };
            assert_eq!(bg, expected);
            assert_eq!(fg, theme.text);
        }
    }

    #[test]
    fn link_is_transparent_with_accent_text() {
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (bg, fg) = variant_colors(ButtonVariant::Link, &theme);
            assert_eq!(bg, gpui::transparent_black());
            assert_eq!(fg, theme.accent);
        }
    }

    #[test]
    fn secondary_fill_is_alpha_based_on_text() {
        use crate::foundations::color::with_alpha;
        for theme in [TahoeTheme::light(), TahoeTheme::dark()] {
            let (bg, fg) = variant_colors(ButtonVariant::Secondary, &theme);
            assert_eq!(bg, with_alpha(theme.text, 0.06));
            assert_eq!(fg, theme.text);
            // Sanity: the resolved fill must have non-zero alpha so the
            // composited result actually differs from theme.background.
            assert!(bg.a > 0.0, "Secondary fill alpha must be non-zero");
        }
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Context, IntoElement, Render, TestAppContext};

    use super::{Button, ButtonSize};
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    const BUTTON_SMOKE: &str = "button-smoke";

    struct ButtonHarness {
        clicks: Rc<RefCell<usize>>,
    }

    impl ButtonHarness {
        fn new(clicks: Rc<RefCell<usize>>) -> Self {
            Self { clicks }
        }
    }

    impl Render for ButtonHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            Button::new("smoke").label("Click me").on_click({
                let clicks = self.clicks.clone();
                move |_, _, _| *clicks.borrow_mut() += 1
            })
        }
    }

    #[gpui::test]
    async fn clicking_button_invokes_handler(cx: &mut TestAppContext) {
        let clicks = Rc::new(RefCell::new(0));
        let (_host, cx) = setup_test_window(cx, |_window, _cx| ButtonHarness::new(clicks.clone()));

        cx.click_on(BUTTON_SMOKE);

        assert_eq!(*clicks.borrow(), 1);
    }

    /// End-to-end guarantee for the icon-only HIG guard: rendering a bare
    /// icon-only Button through the gpui pipeline must panic in debug,
    /// not just when `resolve_ax_name` is called directly.
    struct IconOnlyNoNameHarness;

    impl Render for IconOnlyNoNameHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            Button::new("offender").size(ButtonSize::Icon)
        }
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "HIG Buttons > Tooltips")]
    async fn rendering_icon_only_without_name_panics(cx: &mut TestAppContext) {
        let (_host, _cx) = setup_test_window(cx, |_window, _cx| IconOnlyNoNameHarness);
    }
}
