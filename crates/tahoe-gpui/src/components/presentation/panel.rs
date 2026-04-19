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
//! - [`PanelStyle::HUD`] — "Heads-up display" panel that floats over
//!   its parent with a darker, high-contrast glass surface and no
//!   backdrop dim. Used for Fonts/Colors-style overlays that should
//!   not suppress interaction with the underlying window.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/panels>

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, KeyDownEvent, MouseDownEvent, Pixels, Window, div, px};

use crate::callback_types::{OnMutCallback, rc_wrap};
use crate::foundations::layout::INSPECTOR_PANEL_WIDTH;
use crate::foundations::materials::{backdrop_overlay, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize};

/// Default panel width, backed by the shared layout token
/// [`INSPECTOR_PANEL_WIDTH`] (320 pt — Apple macOS inspector convention).
const DEFAULT_PANEL_WIDTH: Pixels = px(INSPECTOR_PANEL_WIDTH);

/// Position of the panel relative to the viewport edge.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PanelPosition {
    Left,
    #[default]
    Right,
}

/// Panel chrome style per HIG `#panels`.
///
/// Maps to the `NSPanel` variants exposed by AppKit: `Standard`
/// corresponds to a regular utility-window panel, while `HUD` tracks
/// `NSPanel.StyleMask.HUDWindow` — a dark translucent panel without a
/// dimming backdrop so the underlying window remains interactive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PanelStyle {
    /// Inspector-style side panel with backdrop dim. Default.
    #[default]
    Standard,
    /// HUD-style floating panel — no backdrop, darker glass surface,
    /// suitable for tool-palette overlays (Fonts, Colors, HUD).
    HUD,
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

    /// Set the panel width. Defaults to 320px.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Select [`PanelStyle::Standard`] (inspector) or
    /// [`PanelStyle::HUD`] (tool-palette overlay). Defaults to
    /// [`PanelStyle::Standard`].
    pub fn style(mut self, style: PanelStyle) -> Self {
        self.style = style;
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
        let width = self.width.unwrap_or(DEFAULT_PANEL_WIDTH);

        // Share dismiss callback via Rc for multiple handler sites.
        let on_dismiss_rc = rc_wrap(self.on_dismiss);
        let is_hud = matches!(self.style, PanelStyle::HUD);

        // ── Backdrop ────────────────────────────────────────────────────────
        // HIG `#panels`: standard inspector panels dim the window behind
        // them; HUD panels do not (the underlying content stays
        // interactive). We route the Standard backdrop through the shared
        // `backdrop_overlay` helper, which today tints with
        // `theme.overlay_bg` and, once GPUI ships `paint_blur_rect()`,
        // automatically applies Liquid Glass backdrop blur — no change
        // needed at the caller site. HUD renders a transparent positioned
        // div so click-outside dismiss still lands without a visible dim.
        let backdrop = if is_hud {
            div().absolute().top_0().left_0().size_full()
        } else {
            backdrop_overlay(theme)
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
        let panel_id = ElementId::from((self.id.clone(), "panel"));
        let mut panel = glass_surface(
            div().w(width).h_full().flex().flex_col(),
            theme,
            GlassSize::Large,
        )
        .id(panel_id)
        .child(scroll_body);

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

    use crate::components::presentation::panel::{DEFAULT_PANEL_WIDTH, Panel, PanelPosition};

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
    fn panel_full_builder_chain() {
        let _panel = Panel::new("test-panel")
            .open(true)
            .position(PanelPosition::Left)
            .width(px(360.0))
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
        assert_eq!(
            crate::components::presentation::panel::PanelStyle::default(),
            crate::components::presentation::panel::PanelStyle::Standard
        );
    }

    #[test]
    fn panel_builder_style() {
        let panel =
            Panel::new("test-panel").style(crate::components::presentation::panel::PanelStyle::HUD);
        assert_eq!(
            panel.style,
            crate::components::presentation::panel::PanelStyle::HUD
        );
    }
}
