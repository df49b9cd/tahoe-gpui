//! Example: a centered sign-up screen with email and Google sign-in.
//!
//! Demonstrates Button, TextInput, Separator, Label, and centered layout
//! composition. Mirrors the macOS 26 "Auth" screen pattern from the
//! Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{
    App, Bounds, FontWeight, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, div,
    px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::layout_and_organization::separator::Separator;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── App state ────────────────────────────────────────────────────────────────

struct AuthApp;

impl AuthApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for AuthApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();

        // ── Card content ─────────────────────────────────────────────────
        // Width matches the Figma frame's centered card column (~340pt).
        let card = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(0.0))
            .w(px(340.0))
            // App name (large title at top of card)
            .child(
                div()
                    .text_style_emphasized(TextStyle::LargeTitle, theme)
                    .text_color(theme.text)
                    .child("App Name"),
            )
            // Spacer between title and form
            .child(div().h(px(96.0)))
            // "Create an account" headline
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title3, theme)
                    .text_color(theme.text)
                    .child("Create an account"),
            )
            // Subheadline
            .child(
                div()
                    .mt(theme.spacing_xs)
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child("Enter your email to sign up for this app"),
            )
            // Email input (rendered as a styled placeholder div for now)
            .child(
                div()
                    .id("email-input")
                    .mt(theme.spacing_md)
                    .w_full()
                    .h(px(36.0))
                    .px(theme.spacing_md)
                    .flex()
                    .items_center()
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md)
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child("email@domain.com"),
            )
            // Sign up button (filled, full-width, dark)
            .child(
                div().mt(theme.spacing_sm).w_full().child(
                    Button::new("sign-up")
                        .label("Sign up with email")
                        .variant(ButtonVariant::Filled)
                        .size(ButtonSize::Regular)
                        .full_width(true),
                ),
            )
            // "or continue with" divider
            .child(
                div()
                    .mt(theme.spacing_md)
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .child(div().flex_1().child(Separator::horizontal()))
                    .child(
                        div()
                            .text_style(TextStyle::Footnote, theme)
                            .text_color(theme.text_muted)
                            .child("or continue with"),
                    )
                    .child(div().flex_1().child(Separator::horizontal())),
            )
            // Google button (outlined, with G logo)
            .child(
                div().mt(theme.spacing_md).w_full().child(
                    Button::new("google")
                        .label("Google")
                        .icon(Icon::new(IconName::Globe).size(px(16.0)))
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Regular)
                        .full_width(true),
                ),
            )
            // Footer (Terms / Privacy)
            .child(
                div()
                    .mt(theme.spacing_md)
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .text_style(TextStyle::Footnote, theme)
                    .text_color(theme.text_muted)
                    .child(div().child("By clicking continue, you agree to our"))
                    .child(
                        div()
                            .flex()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_color(theme.text)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Terms of Service"),
                            )
                            .child(div().child("and"))
                            .child(
                                div()
                                    .text_color(theme.text)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Privacy Policy"),
                            ),
                    ),
            );

        // ── Root layout ──────────────────────────────────────────────────
        // Use the theme background so dark mode picks up the right surface;
        // hardcoded white here was a holdover from when this demo only ran
        // in light mode.
        div()
            .size_full()
            .bg(theme.background)
            .flex()
            .items_center()
            .justify_center()
            .child(card)
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            // Match Figma frame size (1440 x 960)
            let bounds = Bounds::centered(None, size(px(1440.0), px(960.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(AuthApp::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
