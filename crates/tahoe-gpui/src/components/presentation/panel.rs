//! Panel component (HIG `#panels` — floating auxiliary surface).
//!
//! Panels render `NSPanel`-style floating surfaces (Inspector, Fonts,
//! Colors, HUD overlays) over their parent window. Distinct from
//! [`Sidebar`](crate::components::navigation_and_search::sidebar::Sidebar)
//! (persistent navigation), [`Sheet`](super::sheet::Sheet) (bottom /
//! cardlike overlay), and [`Window`](super::window::WindowStyle)
//! (primary / auxiliary windows).
//!
//! # Styles
//!
//! - [`PanelStyle::Standard`] — Inspector-style side panel that docks
//!   to the leading or trailing edge with a backdrop dim. Rendered
//!   with `GlassSize::Large`.
//! - [`PanelStyle::Inspector`] — attached-to-parent rightmost pane.
//!   320 pt default width, always-visible title bar, no dim backdrop.
//! - [`PanelStyle::Tool`] — free-floating auxiliary palette
//!   (180 pt wide). No dim backdrop; draggable by the title bar.
//! - [`PanelStyle::Dashboard`] — dismissable informational overlay
//!   rendered with slightly translucent Liquid Glass.
//! - [`PanelStyle::TextStyle`] — narrow palette (240 pt) for text
//!   attributes (font / size / color).
//! - [`PanelStyle::HUD`] — "Heads-up display" panel that floats over
//!   its parent with a dark translucent glass surface and no
//!   backdrop dim. Used for Fonts/Colors-style overlays that should
//!   not suppress interaction with the underlying window.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/panels>

use gpui::prelude::*;
use gpui::{
    AnyElement, App, CursorStyle, ElementId, KeyDownEvent, MouseDownEvent, Pixels, SharedString,
    Window, div, px,
};

use crate::callback_types::{OnMutCallback, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::{INSPECTOR_PANEL_WIDTH, MACOS_PANEL_TITLE_BAR_HEIGHT};
use crate::foundations::materials::{backdrop_overlay, glass_surface, glass_surface_hud};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Default panel width, backed by the shared layout token
/// [`INSPECTOR_PANEL_WIDTH`] (320 pt — Apple macOS inspector convention).
const DEFAULT_PANEL_WIDTH: Pixels = px(INSPECTOR_PANEL_WIDTH);

/// Default width for [`PanelStyle::Inspector`] (320 pt, matching
/// Apple's Attributes Inspector convention).
const INSPECTOR_WIDTH_DEFAULT: Pixels = px(INSPECTOR_PANEL_WIDTH);

/// Default width for [`PanelStyle::Tool`] — free-floating auxiliary
/// palette (180 pt, matching macOS Fonts / Colors palette width).
const TOOL_WIDTH_DEFAULT: Pixels = px(180.0);

/// Default width for [`PanelStyle::Dashboard`] — informational
/// overlay (320 pt, matching the Inspector default).
const DASHBOARD_WIDTH_DEFAULT: Pixels = px(INSPECTOR_PANEL_WIDTH);

/// Default width for [`PanelStyle::TextStyle`] — narrow palette
/// (240 pt) for font / size / color attribute pickers.
const TEXT_STYLE_WIDTH_DEFAULT: Pixels = px(240.0);

/// Height of the (non-HUD) panel title bar, per HIG
/// [`MACOS_PANEL_TITLE_BAR_HEIGHT`] (22 pt).
const TITLE_BAR_HEIGHT: Pixels = px(MACOS_PANEL_TITLE_BAR_HEIGHT);

/// Position of the panel relative to the viewport edge.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PanelPosition {
    Left,
    #[default]
    Right,
}

/// Panel chrome style per HIG `#panels`.
///
/// Maps to the `NSPanel` variants exposed by AppKit. `Standard`
/// corresponds to a regular utility-window panel; `HUD` tracks
/// `NSPanel.StyleMask.HUDWindow` (dark translucent, no dim backdrop);
/// `Inspector`, `Tool`, `Dashboard`, and `TextStyle` mirror the
/// auxiliary-panel archetypes Apple ships in Xcode / Pages / the
/// Fonts + Colors palettes.
///
/// Marked `#[non_exhaustive]` so new variants can land without a
/// breaking API change.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PanelStyle {
    /// Inspector-style side panel with backdrop dim. Default.
    #[default]
    Standard,
    /// Attached-to-parent rightmost inspector pane. Always-visible
    /// title bar, no dim backdrop. Default width: 320 pt.
    Inspector,
    /// Free-floating auxiliary palette (Fonts / Colors style).
    /// Default width: 180 pt. No dim backdrop; draggable by the
    /// title bar.
    Tool,
    /// Informational overlay rendered with slightly translucent
    /// Liquid Glass. Dismissable; dims the backdrop.
    Dashboard,
    /// Narrow palette (240 pt) for text attributes (font / size /
    /// color). Dims the backdrop.
    TextStyle,
    /// HUD-style floating panel — dark translucent glass surface,
    /// no backdrop dim, suitable for tool-palette overlays (Fonts,
    /// Colors, HUD).
    HUD,
}

impl PanelStyle {
    /// Per-variant default width.
    fn default_width(self) -> Pixels {
        match self {
            Self::Standard | Self::Inspector => INSPECTOR_WIDTH_DEFAULT,
            Self::Tool => TOOL_WIDTH_DEFAULT,
            Self::Dashboard => DASHBOARD_WIDTH_DEFAULT,
            Self::TextStyle => TEXT_STYLE_WIDTH_DEFAULT,
            Self::HUD => DEFAULT_PANEL_WIDTH,
        }
    }

    /// Whether this variant dims the backdrop behind the panel.
    ///
    /// Only `Standard`, `Inspector` (when rendered without a parent
    /// pane), `Dashboard`, and `TextStyle` dim. `Tool` and `HUD`
    /// leave the underlying window interactive.
    ///
    /// Per the component spec `Inspector` is attached-to-parent and
    /// therefore does *not* dim; callers that present it as a
    /// dismissable overlay should compose their own scrim.
    fn dims_backdrop(self) -> bool {
        match self {
            Self::Standard | Self::Dashboard | Self::TextStyle => true,
            Self::Inspector | Self::Tool | Self::HUD => false,
        }
    }

    /// Whether this variant renders the 22 pt title bar. Every
    /// non-HUD variant renders it; HUD panels are title-less.
    fn renders_title_bar(self) -> bool {
        !matches!(self, Self::HUD)
    }
}

/// An inspector-style side panel overlay following Human Interface Guidelines.
///
/// When open, renders an absolute-positioned glass surface anchored to the left
/// or right viewport edge with a semi-transparent backdrop. Supports dismiss via
/// Escape key and click-outside.
///
/// # Example
///
/// ```ignore
/// Panel::new("inspector")
///     .open(true)
///     .position(PanelPosition::Right)
///     .width(px(360.0))
///     .title("Attributes")
///     .on_dismiss(|_window, _cx| { /* toggle state */ })
///     .child(div().child("Inspector content"))
/// ```
#[derive(IntoElement)]
pub struct Panel {
    id: ElementId,
    is_open: bool,
    position: PanelPosition,
    width: Option<Pixels>,
    on_dismiss: OnMutCallback,
    children: Vec<AnyElement>,
    style: PanelStyle,
    title: Option<SharedString>,
}

impl Panel {
    /// Create a new panel with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            position: PanelPosition::default(),
            width: None,
            on_dismiss: None,
            children: Vec::new(),
            style: PanelStyle::default(),
            title: None,
        }
    }

    /// Control visibility. When `false` the panel renders an empty div.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the panel position (left or right edge). Defaults to `Right`.
    pub fn position(mut self, position: PanelPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the panel width. Defaults to the selected [`PanelStyle`]'s
    /// per-variant default (320 pt Standard/Inspector/Dashboard,
    /// 240 pt TextStyle, 180 pt Tool).
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Select a [`PanelStyle`] variant. Defaults to
    /// [`PanelStyle::Standard`].
    pub fn style(mut self, style: PanelStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the title rendered in the 22 pt title bar. Title bars are
    /// drawn for every non-HUD variant regardless of whether a title
    /// is supplied; the title text is only shown when set.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Called when the user clicks outside the panel or presses Escape.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Append a child element.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Append multiple children from an iterator.
    pub fn children(mut self, iter: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(iter.into_iter().map(|el| el.into_any_element()));
        self
    }
}

impl RenderOnce for Panel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        let theme = cx.theme();
        let width = self.width.unwrap_or_else(|| self.style.default_width());
        let is_hud = matches!(self.style, PanelStyle::HUD);
        let dims_backdrop = self.style.dims_backdrop();
        let renders_title_bar = self.style.renders_title_bar();

        // Share dismiss callback via Rc for multiple handler sites.
        let on_dismiss_rc = rc_wrap(self.on_dismiss);

        // ── Backdrop ────────────────────────────────────────────────────────
        // HIG `#panels`: inspector / dashboard / text-style panels dim the
        // window behind them; Tool and HUD panels do not (the underlying
        // content stays interactive). We route the dimmed path through the
        // shared `backdrop_overlay` helper, which today tints with
        // `theme.overlay_bg` and, once GPUI ships `paint_blur_rect()`,
        // automatically applies Liquid Glass backdrop blur. Non-dimming
        // variants render a transparent positioned div so click-outside
        // dismiss still lands without a visible scrim.
        let backdrop = if dims_backdrop {
            backdrop_overlay(theme)
        } else {
            div().absolute().top_0().left_0().size_full()
        };

        // ── Title bar (22 pt, non-HUD only) ─────────────────────────────────
        // Close button (leading) + centered title + drag region (trailing).
        // Draggable for `Tool`; the cursor hint is applied on every non-HUD
        // variant so the drag region is discoverable. GPUI does not currently
        // expose a window-drag API, so the hit region here is a cursor hint
        // only (see TODO below).
        //
        // TODO(gpui): When GPUI exposes a window-drag API, wire it up on the
        // trailing drag-region div so users can reposition `PanelStyle::Tool`
        // panels by dragging the title bar.
        let title_bar = if renders_title_bar {
            let close_button_id = ElementId::from((self.id.clone(), "panel-close"));

            // Leading close button — fires `on_dismiss`.
            let mut close_button = div()
                .id(close_button_id)
                .flex()
                .items_center()
                .justify_center()
                .w(TITLE_BAR_HEIGHT)
                .h(TITLE_BAR_HEIGHT)
                .cursor_pointer()
                .child(
                    Icon::new(IconName::XmarkCircleFill)
                        .size(theme.icon_size_inline)
                        .color(if is_hud {
                            // Light glyph on HUD's dark glass for contrast.
                            theme.background
                        } else {
                            theme.text_muted
                        }),
                );
            if let Some(ref handler) = on_dismiss_rc {
                let h = handler.clone();
                close_button = close_button.on_click(move |_event, window, cx| {
                    h(window, cx);
                });
            }

            // Centered title (Caption1). Absolutely positioned so it stays
            // centred regardless of the close-button / drag-region widths.
            let title_label = self.title.clone().map(|title| {
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(if is_hud { theme.background } else { theme.text })
                            .child(title),
                    )
            });

            // Trailing drag region — fills remaining width; cursor hint
            // only (no window-drag API in GPUI today).
            let drag_region = div().flex_1().h_full().cursor(CursorStyle::OpenHand);

            let mut bar = div()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .h(TITLE_BAR_HEIGHT)
                .child(close_button)
                .child(drag_region);

            if let Some(label) = title_label {
                bar = bar.child(label);
            }

            Some(bar)
        } else {
            None
        };

        // ── Scrollable content area ─────────────────────────────────────────
        let scroll_id = ElementId::from((self.id.clone(), "scroll"));
        let mut scroll_body = div()
            .id(scroll_id)
            .flex_1()
            .overflow_y_scroll()
            .p(theme.spacing_md);

        for child in self.children {
            scroll_body = scroll_body.child(child);
        }

        // ── Panel surface (glass) ───────────────────────────────────────────
        // HUD panels route through `glass_surface_hud` so the dark tint +
        // light-text recipe matches every other HUD surface in the crate.
        let panel_id = ElementId::from((self.id.clone(), "panel"));
        let panel_body = div().w(width).h_full().flex().flex_col();
        let mut panel = if is_hud {
            glass_surface_hud(panel_body, theme, GlassSize::Large).id(panel_id)
        } else {
            glass_surface(panel_body, theme, GlassSize::Large).id(panel_id)
        };

        if let Some(bar) = title_bar {
            panel = panel.child(bar);
        }
        panel = panel.child(scroll_body);

        // Dismiss on click outside the panel.
        if let Some(ref handler) = on_dismiss_rc {
            let h = handler.clone();
            panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                h(window, cx);
            });
        }

        // Dismiss on Escape key.
        if let Some(ref handler) = on_dismiss_rc {
            let h = handler.clone();
            panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    h(window, cx);
                }
            });
        }

        // ── Position the panel at the correct edge ──────────────────────────
        let mut container = backdrop.flex().flex_row().h_full();

        match self.position {
            PanelPosition::Left => {
                container = container.justify_start();
            }
            PanelPosition::Right => {
                container = container.justify_end();
            }
        }

        container.child(panel).into_any_element()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::prelude::*;
    use gpui::{div, px};

    use crate::components::presentation::panel::{
        DASHBOARD_WIDTH_DEFAULT, DEFAULT_PANEL_WIDTH, INSPECTOR_WIDTH_DEFAULT, Panel,
        PanelPosition, PanelStyle, TEXT_STYLE_WIDTH_DEFAULT, TITLE_BAR_HEIGHT, TOOL_WIDTH_DEFAULT,
    };

    #[test]
    fn panel_position_default_is_right() {
        assert_eq!(PanelPosition::default(), PanelPosition::Right);
    }

    #[test]
    fn panel_position_equality() {
        assert_eq!(PanelPosition::Left, PanelPosition::Left);
        assert_eq!(PanelPosition::Right, PanelPosition::Right);
        assert_ne!(PanelPosition::Left, PanelPosition::Right);
    }

    #[test]
    fn panel_new_has_defaults() {
        let panel = Panel::new("test-panel");
        assert!(!panel.is_open);
        assert_eq!(panel.position, PanelPosition::Right);
        assert!(panel.width.is_none());
        assert!(panel.on_dismiss.is_none());
        assert!(panel.children.is_empty());
        assert!(panel.title.is_none());
    }

    #[test]
    fn panel_builder_open() {
        let panel = Panel::new("test-panel").open(true);
        assert!(panel.is_open);
    }

    #[test]
    fn panel_builder_position() {
        let panel = Panel::new("test-panel").position(PanelPosition::Left);
        assert_eq!(panel.position, PanelPosition::Left);
    }

    #[test]
    fn panel_builder_width() {
        let panel = Panel::new("test-panel").width(px(400.0));
        assert_eq!(panel.width, Some(px(400.0)));
    }

    #[test]
    fn panel_builder_on_dismiss() {
        let panel = Panel::new("test-panel").on_dismiss(|_w, _cx| {});
        assert!(panel.on_dismiss.is_some());
    }

    #[test]
    fn panel_builder_child_adds() {
        let panel = Panel::new("test-panel");
        assert!(panel.children.is_empty());

        let panel = panel.child(div().id("c1"));
        assert_eq!(panel.children.len(), 1);

        let panel = panel.child(div().id("c2"));
        assert_eq!(panel.children.len(), 2);
    }

    #[test]
    fn panel_builder_children_from_iter() {
        let items: Vec<_> = vec![div().id("a"), div().id("b"), div().id("c")];
        let panel = Panel::new("test-panel").children(items);
        assert_eq!(panel.children.len(), 3);
    }

    #[test]
    fn panel_builder_title_sets_value() {
        let panel = Panel::new("test-panel").title("Attributes");
        assert_eq!(panel.title.as_ref().map(|s| s.as_ref()), Some("Attributes"));
    }

    #[test]
    fn panel_full_builder_chain() {
        let _panel = Panel::new("test-panel")
            .open(true)
            .position(PanelPosition::Left)
            .width(px(360.0))
            .title("Attributes")
            .style(PanelStyle::Inspector)
            .on_dismiss(|_w, _cx| {})
            .child(div().id("content"))
            .children(vec![div().id("a"), div().id("b")]);
    }

    #[test]
    fn default_panel_width_is_positive() {
        assert!(f32::from(DEFAULT_PANEL_WIDTH) > 0.0);
    }

    #[test]
    fn panel_style_default_is_standard() {
        assert_eq!(PanelStyle::default(), PanelStyle::Standard);
    }

    // ── Per-variant smoke tests ─────────────────────────────────────────────
    //
    // These check the builder accepts every variant and the derived defaults
    // (width, backdrop-dim, title-bar rendering) match the brief. They don't
    // run the renderer — that requires a GPUI test harness — but cover the
    // variant-specific plumbing that can regress silently.

    #[test]
    fn panel_variant_standard_smoke() {
        let panel = Panel::new("p").style(PanelStyle::Standard);
        assert_eq!(panel.style, PanelStyle::Standard);
        assert_eq!(
            PanelStyle::Standard.default_width(),
            INSPECTOR_WIDTH_DEFAULT
        );
        assert!(PanelStyle::Standard.dims_backdrop());
        assert!(PanelStyle::Standard.renders_title_bar());
    }

    #[test]
    fn panel_variant_inspector_smoke() {
        let panel = Panel::new("p").style(PanelStyle::Inspector);
        assert_eq!(panel.style, PanelStyle::Inspector);
        assert_eq!(
            PanelStyle::Inspector.default_width(),
            INSPECTOR_WIDTH_DEFAULT
        );
        assert_eq!(
            f32::from(PanelStyle::Inspector.default_width()),
            320.0,
            "Inspector default width matches Xcode's Attributes Inspector"
        );
        // Inspector is attached to its parent and does not dim its own
        // backdrop; callers that present it as an overlay compose a scrim.
        assert!(!PanelStyle::Inspector.dims_backdrop());
        assert!(PanelStyle::Inspector.renders_title_bar());
    }

    #[test]
    fn panel_variant_tool_smoke() {
        let panel = Panel::new("p").style(PanelStyle::Tool);
        assert_eq!(panel.style, PanelStyle::Tool);
        assert_eq!(PanelStyle::Tool.default_width(), TOOL_WIDTH_DEFAULT);
        assert_eq!(
            f32::from(PanelStyle::Tool.default_width()),
            180.0,
            "Tool palette default width per HIG Fonts/Colors palette"
        );
        assert!(!PanelStyle::Tool.dims_backdrop());
        assert!(PanelStyle::Tool.renders_title_bar());
    }

    #[test]
    fn panel_variant_dashboard_smoke() {
        let panel = Panel::new("p").style(PanelStyle::Dashboard);
        assert_eq!(panel.style, PanelStyle::Dashboard);
        assert_eq!(
            PanelStyle::Dashboard.default_width(),
            DASHBOARD_WIDTH_DEFAULT
        );
        assert!(PanelStyle::Dashboard.dims_backdrop());
        assert!(PanelStyle::Dashboard.renders_title_bar());
    }

    #[test]
    fn panel_variant_text_style_smoke() {
        let panel = Panel::new("p").style(PanelStyle::TextStyle);
        assert_eq!(panel.style, PanelStyle::TextStyle);
        assert_eq!(
            PanelStyle::TextStyle.default_width(),
            TEXT_STYLE_WIDTH_DEFAULT
        );
        assert_eq!(
            f32::from(PanelStyle::TextStyle.default_width()),
            240.0,
            "TextStyle palette narrow default width"
        );
        assert!(PanelStyle::TextStyle.dims_backdrop());
        assert!(PanelStyle::TextStyle.renders_title_bar());
    }

    #[test]
    fn panel_variant_hud_smoke() {
        let panel = Panel::new("p").style(PanelStyle::HUD);
        assert_eq!(panel.style, PanelStyle::HUD);
        // HUD does not dim and does not render a title bar.
        assert!(!PanelStyle::HUD.dims_backdrop());
        assert!(!PanelStyle::HUD.renders_title_bar());
    }

    // ── Backdrop-dim matrix ─────────────────────────────────────────────────

    #[test]
    fn no_backdrop_for_tool_and_hud() {
        assert!(!PanelStyle::Tool.dims_backdrop());
        assert!(!PanelStyle::HUD.dims_backdrop());
    }

    #[test]
    fn inspector_dashboard_text_style_dim_per_spec() {
        // Only Standard / Dashboard / TextStyle dim. Inspector is
        // attached-to-parent and does not dim its own backdrop.
        assert!(PanelStyle::Standard.dims_backdrop());
        assert!(PanelStyle::Dashboard.dims_backdrop());
        assert!(PanelStyle::TextStyle.dims_backdrop());
        assert!(!PanelStyle::Inspector.dims_backdrop());
    }

    // ── Title bar ───────────────────────────────────────────────────────────

    #[test]
    fn title_bar_renders_when_title_set() {
        // The title bar is rendered for every non-HUD variant regardless
        // of whether a title string is supplied — setting `.title()` just
        // adds the label. This test captures the spec invariant that
        // `renders_title_bar()` is driven by the variant, not by the
        // presence of a title.
        let with_title = Panel::new("p")
            .style(PanelStyle::Inspector)
            .title("Attributes");
        assert_eq!(
            with_title.title.as_ref().map(|s| s.as_ref()),
            Some("Attributes")
        );
        assert!(PanelStyle::Inspector.renders_title_bar());

        let no_title = Panel::new("p").style(PanelStyle::Inspector);
        assert!(no_title.title.is_none());
        assert!(PanelStyle::Inspector.renders_title_bar());

        // HUD stays title-less even if a title is set.
        let hud_with_title = Panel::new("p").style(PanelStyle::HUD).title("ignored");
        assert_eq!(
            hud_with_title.title.as_ref().map(|s| s.as_ref()),
            Some("ignored")
        );
        assert!(!PanelStyle::HUD.renders_title_bar());
    }

    #[test]
    fn title_bar_height_matches_macos_panel_constant() {
        assert!((f32::from(TITLE_BAR_HEIGHT) - 22.0).abs() < f32::EPSILON);
    }
}
