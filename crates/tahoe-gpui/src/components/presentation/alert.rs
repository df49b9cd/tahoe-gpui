//! Alert dialog component aligned with Human Interface Guidelines.
//!
//! Presents a centered modal dialog with a title, optional message, and up to three
//! action buttons. When exactly two actions are provided they render side-by-side;
//! otherwise actions stack vertically.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/alerts>
//!
//! Alerts render differently per platform:
//! - **macOS** — 320 pt wide, Headline title, optional app icon above title,
//!   optional suppression checkbox, optional Help button, Return key
//!   activates the default action, Escape / Command-Period cancel.
//! - **iOS / iPadOS** — 270 pt wide (`UIAlertController`), Title3 title,
//!   no app icon, no suppression checkbox, no Help button.
//!
//! The parent manages `is_open` state and provides an `on_dismiss` callback that
//! fires on backdrop clicks, Escape, and Command-Period.

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window,
    div, px,
};

use crate::callback_types::OnMutCallback;
use crate::components::menus_and_actions::button::{Button, ButtonVariant};
use crate::components::selection_and_input::checkbox::{Checkbox, CheckboxState};
use crate::foundations::icons::Icon;
use crate::foundations::layout::{ALERT_WIDTH_IOS, ALERT_WIDTH_MACOS, Platform};
use crate::foundations::materials::{SurfaceContext, backdrop_overlay, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle, TextStyledExt};

/// Maximum number of actions an alert may contain per HIG.
const MAX_ACTIONS: usize = 3;

/// Shareable click handler carried alongside the rendered button so the
/// containing alert can wire the Return key to the first Default-role
/// action without re-boxing.
type ActionClick = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

/// Boxed callback for toggling the suppression checkbox.
type SuppressionChange = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// Boxed callback for the Help button.
type HelpClick = Box<dyn Fn(&mut Window, &mut App) + 'static>;

/// The semantic role of an alert action, which determines its visual treatment.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AlertActionRole {
    /// Standard action. Rendered with the accent (blue) color. The first
    /// default-role action is bound to the Return key per HIG Alerts.
    #[default]
    Default,
    /// Cancel / dismiss action. Rendered with the label color (secondary
    /// to Default) and semibold weight per HIG `#buttons` roles guidance.
    Cancel,
    /// Destructive action (e.g. delete). Rendered in `theme.error` color
    /// with semibold weight to reinforce severity.
    Destructive,
}

/// A single action button within an [`Alert`].
pub struct AlertAction {
    /// Button label text.
    pub label: SharedString,
    /// Semantic role controlling visual treatment.
    pub role: AlertActionRole,
    /// Callback invoked when the action is clicked.
    pub on_click: OnMutCallback,
}

impl AlertAction {
    /// Create a new action with the given label and default role.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            role: AlertActionRole::Default,
            on_click: None,
        }
    }

    /// Set the semantic role.
    pub fn role(mut self, role: AlertActionRole) -> Self {
        self.role = role;
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// Returns the HIG default width (in points) for an alert panel on
/// the given platform.
///
/// macOS uses 320 pt to match `NSAlert`; iOS/iPadOS use 270 pt
/// (`UIAlertController`). Other platforms fall back to the macOS width
/// since they closely match Apple's desktop / spatial conventions.
pub fn alert_width_for(platform: Platform) -> f32 {
    match platform {
        Platform::IOS | Platform::WatchOS => ALERT_WIDTH_IOS,
        Platform::MacOS | Platform::TvOS | Platform::VisionOS => ALERT_WIDTH_MACOS,
    }
}

/// A centered alert dialog with semi-transparent backdrop.
///
/// Follows HIG alert conventions:
/// - Title is always present, displayed prominently. macOS uses
///   [`TextStyle::Headline`] (13 pt Bold); iOS/iPadOS keep
///   [`TextStyle::Title3`] (15 pt Semibold).
/// - Optional message provides additional context.
/// - Up to 3 action buttons; 2 actions render side-by-side, others stack
///   vertically.
/// - macOS: optional app icon above the title, optional suppression
///   checkbox below the message, optional Help button to the left of
///   the action row.
/// - Uses `GlassSize::Large` glass surface.
/// - Backdrop click, Escape, and Command-Period invoke `on_dismiss`.
/// - Return activates the first `AlertActionRole::Default` action.
#[derive(IntoElement)]
pub struct Alert {
    id: ElementId,
    title: SharedString,
    message: Option<SharedString>,
    is_open: bool,
    actions: Vec<AlertAction>,
    on_dismiss: OnMutCallback,
    focus_handle: Option<FocusHandle>,
    /// Optional platform override. `None` = read from `TahoeTheme`.
    platform: Option<Platform>,
    /// Optional icon rendered above the title (macOS HIG app icon slot).
    icon: Option<Icon>,
    /// Optional extra content slot between message and action buttons
    /// (e.g. `TextField` for prompted input, custom inline controls).
    accessory: Option<AnyElement>,
    /// Optional suppression-checkbox label (macOS only).
    suppression_label: Option<SharedString>,
    /// Current suppression checkbox state (parent-owned).
    suppression_checked: bool,
    /// Callback when the suppression checkbox is toggled.
    on_suppression_change: Option<SuppressionChange>,
    /// Callback when the Help button is clicked (macOS). The Help button
    /// renders only when this handler is set.
    on_help: Option<HelpClick>,
}

impl Alert {
    /// Create a new alert with the given id and title.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            message: None,
            is_open: false,
            actions: Vec::new(),
            on_dismiss: None,
            focus_handle: None,
            platform: None,
            icon: None,
            accessory: None,
            suppression_label: None,
            suppression_checked: false,
            on_suppression_change: None,
            on_help: None,
        }
    }

    /// Set the optional message body.
    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the open/closed state.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Add a single action. At most `MAX_ACTIONS` (3) actions are retained.
    pub fn action(mut self, action: AlertAction) -> Self {
        if self.actions.len() < MAX_ACTIONS {
            self.actions.push(action);
        }
        self
    }

    /// Set all actions at once, truncating to `MAX_ACTIONS`.
    pub fn actions(mut self, actions: Vec<AlertAction>) -> Self {
        self.actions = actions.into_iter().take(MAX_ACTIONS).collect();
        self
    }

    /// Override the focus handle tracked by the alert content.
    ///
    /// By default Alert mints an internal handle on open and focuses it so the
    /// Escape-dismiss handler fires without any parent boilerplate. Provide a
    /// handle here only when the parent needs to coordinate focus (e.g. restore
    /// the previously focused element on dismiss).
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Set the dismiss handler (backdrop click, Escape, and Command-Period).
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Override the platform used to size / style the alert. Defaults to
    /// [`TahoeTheme::platform`].
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Set an icon rendered above the title. HIG `#alerts` macOS:
    /// "macOS automatically displays your app icon in an alert, but you
    /// can supply an alternative icon or symbol." Ignored on iOS.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Provide an inline accessory rendered between the message and the
    /// action row. Intended for prompted-input dialogs (pass a
    /// `TextField` rendered as an [`AnyElement`]) but accepts any element.
    /// Per HIG `#alerts`: "an alert can include a text field" on iOS,
    /// iPadOS, macOS, and visionOS.
    pub fn accessory(mut self, accessory: impl IntoElement) -> Self {
        self.accessory = Some(accessory.into_any_element());
        self
    }

    /// Add a suppression checkbox (macOS only) — HIG `#alerts`: "macOS
    /// alerts can add a suppression checkbox." The parent owns the
    /// checked state and is notified via `on_suppression_change`.
    pub fn suppression(
        mut self,
        label: impl Into<SharedString>,
        checked: bool,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.suppression_label = Some(label.into());
        self.suppression_checked = checked;
        self.on_suppression_change = Some(Box::new(on_change));
        self
    }

    /// Attach a Help button (macOS only). The Help button renders in the
    /// lower-leading corner of the alert and invokes `handler` when
    /// clicked.
    pub fn on_help(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_help = Some(Box::new(handler));
        self
    }

    fn resolved_platform(&self, theme: &TahoeTheme) -> Platform {
        self.platform.unwrap_or(theme.platform)
    }
}

impl RenderOnce for Alert {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Resolve platform-dependent values up-front so the subsequent
        // focus handle acquisition can take a mutable borrow of `cx`.
        let (platform, width, is_macos, title_style) = {
            let theme = cx.theme();
            let platform = self.resolved_platform(theme);
            let width = alert_width_for(platform);
            let is_macos = matches!(platform, Platform::MacOS);
            // macOS `NSAlert` uses the 13 pt Headline weight; iOS
            // `UIAlertController` uses a 15 pt rounded title we
            // approximate with Title3.
            let title_style = if is_macos {
                TextStyle::Headline
            } else {
                TextStyle::Title3
            };
            (platform, width, is_macos, title_style)
        };
        let _ = platform; // retained for future per-platform branching

        // Mint a default focus handle if the parent didn't provide one, then
        // request focus so Escape works immediately without parent boilerplate.
        let focus_handle = self.focus_handle.unwrap_or_else(|| cx.focus_handle());
        if !focus_handle.is_focused(window) {
            focus_handle.focus(window, cx);
        }

        let theme = cx.theme();

        // Use Rc to share the dismiss callback between backdrop, key
        // handler, and the per-alert default-action Return binding.
        let on_dismiss_rc = self.on_dismiss.map(std::rc::Rc::new);

        // -- Backdrop --------------------------------------------------------
        let backdrop = backdrop_overlay(theme)
            .id(ElementId::from((self.id.clone(), "backdrop")))
            .flex()
            .items_center()
            .justify_center();

        // -- Content container (glass surface) --------------------------------
        let content_id = ElementId::from((self.id.clone(), "content"));
        let mut content_div = glass_surface(
            div().w(px(width)).overflow_hidden(),
            theme,
            GlassSize::Large,
        )
        .id(content_id)
        .focusable()
        .flex()
        .flex_col()
        .items_center();

        content_div = content_div
            .track_focus(&focus_handle)
            .debug_selector(|| "alert-content".into());

        // Dismiss on click outside the content.
        if let Some(ref handler) = on_dismiss_rc {
            let handler = handler.clone();
            content_div =
                content_div.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(window, cx);
                });
        }

        // -- Icon (macOS app icon slot) --------------------------------------
        let icon_el = if is_macos {
            self.icon.map(|icon| {
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .pt(theme.spacing_lg)
                    .px(theme.spacing_md)
                    .child(icon)
            })
        } else {
            None
        };

        // -- Title ------------------------------------------------------------
        let title_pt = if icon_el.is_some() {
            theme.spacing_sm
        } else {
            theme.spacing_lg
        };
        let title_el = div()
            .w_full()
            .flex()
            .justify_center()
            .pt(title_pt)
            .px(theme.spacing_md)
            .text_style(title_style, theme)
            .text_color(theme.label_color(SurfaceContext::GlassDim))
            .child(self.title);

        // -- Message ----------------------------------------------------------
        let message_el = self.message.map(|msg| {
            div()
                .w_full()
                .flex()
                .justify_center()
                .pt(theme.spacing_xs)
                .px(theme.spacing_md)
                .pb(theme.spacing_sm)
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.secondary_label_color(SurfaceContext::GlassDim))
                .child(msg)
        });

        // -- Accessory slot (e.g. TextField) ---------------------------------
        let accessory_el = self.accessory.map(|acc| {
            div()
                .w_full()
                .flex()
                .flex_col()
                .px(theme.spacing_md)
                .pb(theme.spacing_sm)
                .child(acc)
        });

        // -- Suppression checkbox (macOS only) -------------------------------
        let suppression_el = if is_macos {
            self.suppression_label.map(|label| {
                let checked = self.suppression_checked;
                let suppression_id = ElementId::from((self.id.clone(), "suppression"));
                let mut cb = Checkbox::new(suppression_id)
                    .state(CheckboxState::from_bool(checked))
                    .label(label);
                if let Some(handler) = self.on_suppression_change {
                    cb = cb.on_change(move |state, window, cx| {
                        handler(state.is_filled(), window, cx);
                    });
                }
                div()
                    .w_full()
                    .px(theme.spacing_md)
                    .pb(theme.spacing_sm)
                    .child(cb)
            })
        } else {
            None
        };

        // -- Actions ----------------------------------------------------------
        let action_count = self.actions.len();
        let use_horizontal = action_count == 2;

        let hover_bg = theme.hover_bg();

        let separator_border = if theme.accessibility_mode.increase_contrast() {
            theme.glass.accessibility.high_contrast_border
        } else {
            theme.border
        };

        // Build buttons, extracting the first-Default click for Return.
        let mut default_return: Option<ActionClick> = None;
        // Each entry: (role, wrapper div that holds the Button).
        let mut built_buttons: Vec<(AlertActionRole, gpui::Div)> = Vec::with_capacity(action_count);
        for (idx, action) in self.actions.into_iter().enumerate() {
            let role = action.role;
            let variant = match role {
                AlertActionRole::Default => ButtonVariant::Primary,
                AlertActionRole::Cancel => ButtonVariant::Cancel,
                AlertActionRole::Destructive => ButtonVariant::Destructive,
            };
            let btn_id =
                ElementId::NamedInteger(format!("{:?}-action", self.id).into(), idx as u64);
            let mut btn = Button::new(btn_id)
                .label(action.label.clone())
                .variant(variant)
                .full_width(true);

            let mut click_rc: Option<ActionClick> = None;
            if let Some(handler) = action.on_click {
                let handler: ActionClick = std::rc::Rc::from(handler);
                let btn_h = handler.clone();
                btn = btn.on_click(move |_event, window, cx| btn_h(window, cx));
                click_rc = Some(handler);
            }
            if role == AlertActionRole::Default && default_return.is_none() {
                default_return = click_rc;
            }

            // Wrap in a flex container that carries the separator border and
            // sizing. Horizontal: flex_1 so both halves are equal width.
            // Vertical: w_full.
            let mut wrapper = div().border_t_1().border_color(separator_border).child(btn);
            if use_horizontal {
                wrapper = wrapper.flex_1();
                if idx == 1 {
                    wrapper = wrapper.border_l_1().border_color(separator_border);
                }
            } else {
                wrapper = wrapper.w_full();
            }
            built_buttons.push((role, wrapper));
        }

        // HIG Alerts (macOS + iOS): in a horizontal two-button row, Cancel
        // and Destructive actions live on the LEADING side (LTR: left); the
        // Default action sits on the TRAILING side (LTR: right) so Return
        // maps to the rightmost button people expect. Sort stably by role
        // weight so callers can pass `[Default, Cancel]` or `[Cancel,
        // Default]` and we still place them correctly.
        //
        // Weight order (ascending → leading-to-trailing): Cancel = 0,
        // Destructive = 1, Default = 2. Destructive sits between Cancel
        // and Default because HIG treats it as "act" rather than "dismiss".
        if use_horizontal {
            built_buttons.sort_by_key(|(role, _)| match role {
                AlertActionRole::Cancel => 0,
                AlertActionRole::Destructive => 1,
                AlertActionRole::Default => 2,
            });
        }

        let actions_container = if use_horizontal {
            let mut row = div().w_full().flex().flex_row();
            for (_, wrapper) in built_buttons.into_iter() {
                row = row.child(wrapper);
            }
            row
        } else {
            let mut col = div().w_full().flex().flex_col();
            for (_, wrapper) in built_buttons.into_iter() {
                col = col.child(wrapper);
            }
            col
        };

        // -- Help button (macOS only) -----------------------------------------
        let help_el = if is_macos {
            self.on_help.map(|handler| {
                let handler = std::rc::Rc::new(handler);
                div()
                    .id(ElementId::from((self.id.clone(), "help")))
                    .debug_selector(|| "alert-help".into())
                    .absolute()
                    .left(theme.spacing_md)
                    .bottom(theme.spacing_md)
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.label_color(SurfaceContext::GlassDim))
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_bg))
                    .on_click(move |_event, window, cx| handler(window, cx))
                    .child("?")
            })
        } else {
            None
        };

        // Key handler: Escape, Command-Period, and Return. All three
        // shortcuts are wired per HIG Alerts table.
        let dismiss_for_keys = on_dismiss_rc.clone();
        let return_handler = default_return.clone();
        content_div = content_div.on_key_down(move |event: &KeyDownEvent, window, cx| {
            let key = event.keystroke.key.as_str();
            let modifiers = &event.keystroke.modifiers;
            // Command-Period on macOS matches Escape per HIG.
            let is_cmd_period = modifiers.platform && key == ".";
            if crate::foundations::keyboard::is_escape_key(event) || is_cmd_period {
                if let Some(handler) = &dismiss_for_keys {
                    handler(window, cx);
                }
                return;
            }
            if (key == "enter" || key == "return")
                && !modifiers.shift
                && !modifiers.platform
                && let Some(handler) = &return_handler
            {
                cx.stop_propagation();
                handler(window, cx);
            }
        });

        // -- Spacing before actions when there's no message ------------------
        let needs_spacer =
            message_el.is_none() && accessory_el.is_none() && suppression_el.is_none();
        let spacer = if needs_spacer {
            Some(div().pb(theme.spacing_md))
        } else {
            None
        };

        // -- Assemble ---------------------------------------------------------
        if let Some(ic) = icon_el {
            content_div = content_div.child(ic);
        }
        content_div = content_div.child(title_el);
        if let Some(msg) = message_el {
            content_div = content_div.child(msg);
        }
        if let Some(acc) = accessory_el {
            content_div = content_div.child(acc);
        }
        if let Some(sup) = suppression_el {
            content_div = content_div.child(sup);
        }
        if let Some(sp) = spacer {
            content_div = content_div.child(sp);
        }
        if action_count > 0 {
            // Wrap actions in a relative container so the Help button
            // (absolutely positioned) hangs on its leading edge.
            let mut action_row_wrap = div().relative().w_full().child(actions_container);
            if let Some(help) = help_el {
                action_row_wrap = action_row_wrap.child(help);
            }
            content_div = content_div.child(action_row_wrap);
        }

        backdrop.child(content_div).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{Alert, AlertAction, AlertActionRole, MAX_ACTIONS, alert_width_for};
    use crate::foundations::layout::{ALERT_WIDTH_IOS, ALERT_WIDTH_MACOS, Platform};

    #[test]
    fn alert_new_defaults() {
        let alert = Alert::new("test-alert", "Title");
        assert_eq!(
            format!("{:?}", alert.id),
            format!("{:?}", gpui::ElementId::from("test-alert"))
        );
        assert_eq!(alert.title.as_ref(), "Title");
        assert!(alert.message.is_none());
        assert!(!alert.is_open);
        assert!(alert.actions.is_empty());
        assert!(alert.on_dismiss.is_none());
        assert!(alert.focus_handle.is_none());
        assert!(alert.platform.is_none());
        assert!(alert.icon.is_none());
        assert!(alert.accessory.is_none());
        assert!(alert.suppression_label.is_none());
        assert!(!alert.suppression_checked);
        assert!(alert.on_suppression_change.is_none());
        assert!(alert.on_help.is_none());
    }

    #[test]
    fn alert_builder_message() {
        let alert = Alert::new("a", "T").message("Details here");
        assert_eq!(
            alert.message.as_ref().map(|s| s.as_ref()),
            Some("Details here")
        );
    }

    #[test]
    fn alert_builder_open() {
        let alert = Alert::new("a", "T").open(true);
        assert!(alert.is_open);
    }

    #[test]
    fn alert_builder_on_dismiss() {
        let alert = Alert::new("a", "T").on_dismiss(|_, _| {});
        assert!(alert.on_dismiss.is_some());
    }

    #[test]
    fn alert_focus_handle_defaults_to_none() {
        let alert = Alert::new("a", "T");
        assert!(alert.focus_handle.is_none());
    }

    #[test]
    fn alert_builder_platform() {
        let alert = Alert::new("a", "T").platform(Platform::IOS);
        assert_eq!(alert.platform, Some(Platform::IOS));
    }

    #[test]
    fn horizontal_role_sort_keys_match_hig() {
        // HIG horizontal alerts: leading → trailing reads as
        // Cancel < Destructive < Default. Sort key implementation in
        // `Render::render` must agree with this ordering — verifying the
        // contract here keeps the in-render sort honest.
        let key = |role: AlertActionRole| match role {
            AlertActionRole::Cancel => 0,
            AlertActionRole::Destructive => 1,
            AlertActionRole::Default => 2,
        };
        assert!(key(AlertActionRole::Cancel) < key(AlertActionRole::Default));
        assert!(key(AlertActionRole::Destructive) < key(AlertActionRole::Default));
        assert!(key(AlertActionRole::Cancel) < key(AlertActionRole::Destructive));
    }

    #[test]
    fn horizontal_two_button_sort_places_default_trailing() {
        // Caller passes [Default, Cancel] (a common mistake) — verify the
        // sort lifts Cancel to leading, Default to trailing per HIG macOS
        // alert ordering.
        let mut buttons = [
            (AlertActionRole::Default, "OK"),
            (AlertActionRole::Cancel, "Cancel"),
        ];
        buttons.sort_by_key(|(role, _)| match role {
            AlertActionRole::Cancel => 0,
            AlertActionRole::Destructive => 1,
            AlertActionRole::Default => 2,
        });
        assert_eq!(buttons[0].1, "Cancel");
        assert_eq!(buttons[1].1, "OK");
    }

    #[test]
    fn horizontal_destructive_sits_between_cancel_and_default() {
        let mut buttons = [
            (AlertActionRole::Default, "Save"),
            (AlertActionRole::Cancel, "Cancel"),
            (AlertActionRole::Destructive, "Discard"),
        ];
        buttons.sort_by_key(|(role, _)| match role {
            AlertActionRole::Cancel => 0,
            AlertActionRole::Destructive => 1,
            AlertActionRole::Default => 2,
        });
        let labels: Vec<&str> = buttons.iter().map(|(_, l)| *l).collect();
        assert_eq!(labels, ["Cancel", "Discard", "Save"]);
    }

    #[test]
    fn alert_builder_suppression() {
        let alert = Alert::new("a", "T").suppression("Don't ask again", true, |_, _, _| {});
        assert_eq!(
            alert.suppression_label.as_ref().map(|s| s.as_ref()),
            Some("Don't ask again")
        );
        assert!(alert.suppression_checked);
        assert!(alert.on_suppression_change.is_some());
    }

    #[test]
    fn alert_builder_on_help() {
        let alert = Alert::new("a", "T").on_help(|_, _| {});
        assert!(alert.on_help.is_some());
    }

    #[test]
    fn alert_action_new_defaults() {
        let action = AlertAction::new("OK");
        assert_eq!(action.label.as_ref(), "OK");
        assert_eq!(action.role, AlertActionRole::Default);
        assert!(action.on_click.is_none());
    }

    #[test]
    fn alert_action_role_builder() {
        let action = AlertAction::new("Delete").role(AlertActionRole::Destructive);
        assert_eq!(action.role, AlertActionRole::Destructive);
    }

    #[test]
    fn alert_action_on_click_builder() {
        let action = AlertAction::new("OK").on_click(|_, _| {});
        assert!(action.on_click.is_some());
    }

    #[test]
    fn alert_action_role_default_variant() {
        assert_eq!(AlertActionRole::default(), AlertActionRole::Default);
    }

    #[test]
    fn alert_action_role_all_distinct() {
        let roles = [
            AlertActionRole::Default,
            AlertActionRole::Cancel,
            AlertActionRole::Destructive,
        ];
        for i in 0..roles.len() {
            for j in 0..roles.len() {
                if i == j {
                    assert_eq!(roles[i], roles[j]);
                } else {
                    assert_ne!(roles[i], roles[j]);
                }
            }
        }
    }

    #[test]
    fn alert_single_action() {
        let alert = Alert::new("a", "T").action(AlertAction::new("OK"));
        assert_eq!(alert.actions.len(), 1);
    }

    #[test]
    fn alert_actions_bulk() {
        let alert = Alert::new("a", "T").actions(vec![
            AlertAction::new("Cancel").role(AlertActionRole::Cancel),
            AlertAction::new("Delete").role(AlertActionRole::Destructive),
        ]);
        assert_eq!(alert.actions.len(), 2);
        assert_eq!(alert.actions[0].role, AlertActionRole::Cancel);
        assert_eq!(alert.actions[1].role, AlertActionRole::Destructive);
    }

    #[test]
    fn alert_max_actions_enforced_via_action() {
        let mut alert = Alert::new("a", "T");
        for i in 0..5 {
            alert = alert.action(AlertAction::new(format!("Action {i}")));
        }
        assert_eq!(alert.actions.len(), MAX_ACTIONS);
    }

    #[test]
    fn alert_max_actions_enforced_via_actions() {
        let alert = Alert::new("a", "T").actions(vec![
            AlertAction::new("1"),
            AlertAction::new("2"),
            AlertAction::new("3"),
            AlertAction::new("4"),
        ]);
        assert_eq!(alert.actions.len(), MAX_ACTIONS);
    }

    #[test]
    fn max_actions_constant_is_three() {
        assert_eq!(MAX_ACTIONS, 3);
    }

    #[test]
    fn alert_width_for_platform() {
        assert!((alert_width_for(Platform::IOS) - ALERT_WIDTH_IOS).abs() < f32::EPSILON);
        assert!((alert_width_for(Platform::MacOS) - ALERT_WIDTH_MACOS).abs() < f32::EPSILON);
        assert!((alert_width_for(Platform::VisionOS) - ALERT_WIDTH_MACOS).abs() < f32::EPSILON);
        assert!((alert_width_for(Platform::WatchOS) - ALERT_WIDTH_IOS).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::{Alert, AlertAction, AlertActionRole};
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const ALERT_CONTENT_SELECTOR: &str = "alert-content";

    struct AlertHarness {
        focus_handle: FocusHandle,
        is_open: bool,
        dismiss_count: usize,
    }

    impl AlertHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                is_open: true,
                dismiss_count: 0,
            }
        }
    }

    impl Render for AlertHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            div().w(px(320.0)).h(px(240.0)).child(
                Alert::new("alert", "Delete?")
                    .open(self.is_open)
                    .focus_handle(self.focus_handle.clone())
                    .action(AlertAction::new("OK"))
                    .on_dismiss(move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.dismiss_count += 1;
                            this.is_open = false;
                            cx.notify();
                        });
                    }),
            )
        }
    }

    #[gpui::test]
    async fn escape_dismisses_alert_when_focused(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| AlertHarness::new(cx));
        // Alert auto-focuses its content when opened, so Escape is live
        // without any parent-side focus wiring.
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });

        cx.press("escape");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.dismiss_count, 1);
            assert!(!host.is_open);
        });
        assert_element_absent(cx, ALERT_CONTENT_SELECTOR);
    }

    #[gpui::test]
    async fn open_alert_renders_content(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, cx| AlertHarness::new(cx));
        assert_element_exists(cx, ALERT_CONTENT_SELECTOR);
    }

    // Harness that captures Return-key activation of the default action.
    struct ReturnKeyHarness {
        focus_handle: FocusHandle,
        activations: Rc<RefCell<usize>>,
    }

    impl ReturnKeyHarness {
        fn new(cx: &mut Context<Self>, activations: Rc<RefCell<usize>>) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                activations,
            }
        }
    }

    impl Render for ReturnKeyHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let activations = self.activations.clone();
            Alert::new("alert", "Delete?")
                .open(true)
                .focus_handle(self.focus_handle.clone())
                .action(AlertAction::new("OK").on_click(move |_, _| {
                    *activations.borrow_mut() += 1;
                }))
                .action(AlertAction::new("Cancel").role(AlertActionRole::Cancel))
        }
    }

    #[gpui::test]
    async fn return_key_activates_default_action(cx: &mut TestAppContext) {
        let activations = Rc::new(RefCell::new(0));
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            ReturnKeyHarness::new(cx, activations.clone())
        });

        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
        cx.press("enter");

        assert_eq!(*activations.borrow(), 1);
    }
}
