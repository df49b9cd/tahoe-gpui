//! Alerts demo for the primitive gallery — mirrors the macOS 26
//! "Alerts" page from the Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::alert::{Alert, AlertAction, AlertActionRole};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertKind {
    SideBySide,
    Stacked,
    Single,
    Destructive,
}

#[derive(Default)]
pub struct AlertsState {
    pub open: Option<AlertKind>,
}

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let open = state.alerts_state.open;

    let header = div()
        .px(theme.spacing_xl)
        .pt(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Alerts"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("An alert gives people critical information they need right away."),
        );

    let toolbar = div()
        .px(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .gap(theme.spacing_sm)
        .child(
            Button::new("alerts-show-side")
                .label("Side-by-side")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.alerts_state.open = Some(AlertKind::SideBySide);
                    cx.notify();
                })),
        )
        .child(
            Button::new("alerts-show-stacked")
                .label("Stacked (3 actions)")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.alerts_state.open = Some(AlertKind::Stacked);
                    cx.notify();
                })),
        )
        .child(
            Button::new("alerts-show-single")
                .label("Single action")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.alerts_state.open = Some(AlertKind::Single);
                    cx.notify();
                })),
        )
        .child(
            Button::new("alerts-show-destructive")
                .label("Destructive")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.alerts_state.open = Some(AlertKind::Destructive);
                    cx.notify();
                })),
        );

    let entity = cx.entity().downgrade();
    let dismiss = move |_window: &mut Window, cx: &mut App| {
        if let Some(this) = entity.upgrade() {
            this.update(cx, |this, cx| {
                this.alerts_state.open = None;
                cx.notify();
            });
        }
    };

    let alert: AnyElement = match open {
        Some(AlertKind::SideBySide) => Alert::new("alert-side", "Discard changes?")
            .message("If you discard now, you'll lose any edits you've made since the last save.")
            .open(true)
            .actions(vec![
                AlertAction::new("Cancel")
                    .role(AlertActionRole::Cancel)
                    .on_click(dismiss.clone()),
                AlertAction::new("Discard").on_click(dismiss.clone()),
            ])
            .on_dismiss(dismiss.clone())
            .into_any_element(),
        Some(AlertKind::Stacked) => {
            Alert::new("alert-stacked", "Save changes to \u{201c}Untitled\u{201d}?")
                .message("Your changes will be lost if you don't save them.")
                .open(true)
                .actions(vec![
                    AlertAction::new("Save")
                        .role(AlertActionRole::Default)
                        .on_click(dismiss.clone()),
                    AlertAction::new("Don\u{2019}t Save")
                        .role(AlertActionRole::Destructive)
                        .on_click(dismiss.clone()),
                    AlertAction::new("Cancel")
                        .role(AlertActionRole::Cancel)
                        .on_click(dismiss.clone()),
                ])
                .on_dismiss(dismiss.clone())
                .into_any_element()
        }
        Some(AlertKind::Single) => Alert::new("alert-single", "Update available")
            .message("A new version is ready to install. Restart to apply the update.")
            .open(true)
            .actions(vec![
                AlertAction::new("OK")
                    .role(AlertActionRole::Cancel)
                    .on_click(dismiss.clone()),
            ])
            .on_dismiss(dismiss.clone())
            .into_any_element(),
        Some(AlertKind::Destructive) => Alert::new("alert-destructive", "Delete this file?")
            .message("This action cannot be undone.")
            .open(true)
            .actions(vec![
                AlertAction::new("Cancel")
                    .role(AlertActionRole::Cancel)
                    .on_click(dismiss.clone()),
                AlertAction::new("Delete")
                    .role(AlertActionRole::Destructive)
                    .on_click(dismiss.clone()),
            ])
            .on_dismiss(dismiss.clone())
            .into_any_element(),
        None => div().into_any_element(),
    };

    div()
        .id("alerts-pane")
        .relative()
        .size_full()
        .bg(theme.glass.root_bg)
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(header)
                .child(toolbar),
        )
        .child(alert)
        .into_any_element()
}
