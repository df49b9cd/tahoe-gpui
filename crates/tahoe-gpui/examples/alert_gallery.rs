//! Visual gallery of `Alert` variants for diffing against the macOS 26
//! Alerts reference page in Apple's UI Kit.
//!
//! Renders three canonical alert configurations: Side-by-side (2 actions),
//! Stacked (3 actions), and Single (1 action). Click any button in the
//! bottom toolbar to reopen the corresponding alert.

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Bounds, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, div,
    hsla, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::alert::{Alert, AlertAction, AlertActionRole};
use tahoe_gpui::foundations::icons::EmbeddedIconAssets;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AlertKind {
    SideBySide,
    Stacked,
    Single,
    Destructive,
}

struct AlertGallery {
    open: Option<AlertKind>,
}

impl AlertGallery {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            open: Some(AlertKind::SideBySide),
        }
    }

    fn open(&mut self, kind: AlertKind, cx: &mut Context<Self>) {
        self.open = Some(kind);
        cx.notify();
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.open = None;
        cx.notify();
    }
}

impl Render for AlertGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let theme = &theme;
        let open = self.open;

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

        // Toolbar to switch between alert variants
        let toolbar = div()
            .px(theme.spacing_xl)
            .pb(theme.spacing_lg)
            .flex()
            .gap(theme.spacing_sm)
            .child(
                Button::new("show-side-by-side")
                    .label("Side-by-side")
                    .variant(ButtonVariant::Primary)
                    .size(ButtonSize::Md)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open(AlertKind::SideBySide, cx);
                    })),
            )
            .child(
                Button::new("show-stacked")
                    .label("Stacked (3 actions)")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Md)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open(AlertKind::Stacked, cx);
                    })),
            )
            .child(
                Button::new("show-single")
                    .label("Single action")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Md)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open(AlertKind::Single, cx);
                    })),
            )
            .child(
                Button::new("show-destructive")
                    .label("Destructive")
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Md)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open(AlertKind::Destructive, cx);
                    })),
            );

        // Build the active alert as an AnyElement
        let entity = cx.entity().downgrade();
        let dismiss = {
            let entity = entity.clone();
            move |_window: &mut Window, cx: &mut App| {
                if let Some(this) = entity.upgrade() {
                    this.update(cx, |this, cx| this.dismiss(cx));
                }
            }
        };
        let alert: AnyElement = match open {
            Some(AlertKind::SideBySide) => Alert::new("alert-side", "Discard changes?")
                .message(
                    "If you discard now, you'll lose any edits you've made since the last save.",
                )
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
            .id("alert-gallery")
            .size_full()
            .relative()
            .bg(hsla(0.6, 0.30, 0.65, 1.0)) // Sample-color background so we can see the alert
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(header)
                    .child(toolbar),
            )
            .child(alert)
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            let bounds = Bounds::centered(None, size(px(900.0), px(700.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(AlertGallery::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
