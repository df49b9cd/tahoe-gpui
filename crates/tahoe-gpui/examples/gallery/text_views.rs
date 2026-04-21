//! Text Views demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, HighlightStyle, StyledText, Window, div};

use tahoe_gpui::components::content::text_view::{LabelLevel, TextView};
use tahoe_gpui::foundations::theme::{
    FontDesign, LeadingStyle, TahoeTheme, TextStyle, TextStyledExt,
};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("text-views-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Text Views"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A text view displays read-only, styled text blocks. \
                     Unlike a label, it is designed for multi-line paragraphs.",
                ),
        )
        // ── Body (default) ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Body style (default)"),
        )
        .child(TextView::new(
            "The quick brown fox jumps over the lazy dog. \
             This text view uses the default Body text style.",
        ))
        // ── Title 1 ────────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Title 1 style"),
        )
        .child(TextView::new("Large styled heading text").text_style(TextStyle::Title1))
        // ── Caption ────────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Caption style"),
        )
        .child(
            TextView::new("Small caption text suitable for footnotes and metadata.")
                .text_style(TextStyle::Caption1),
        )
        // ── Emphasized ─────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Emphasized"),
        )
        .child(
            TextView::new(
                "This body text uses the HIG emphasized weight (Semibold). \
                 Useful for lead paragraphs or standout content blocks.",
            )
            .emphasize(true),
        )
        // ── max_lines ──────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With max_lines(2)"),
        )
        .child(
            TextView::new(
                "This text view has max_lines set to 2. Content beyond two \
                 lines is clipped via GPUI's native line-clamp support.",
            )
            .max_lines(2),
        )
        // ── Styled text ────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Styled text"),
        )
        .child(TextView::new("placeholder").styled_text(
            StyledText::new("Bold and accent-colored text within a single view.").with_highlights(
                vec![(
                    0..4,
                    HighlightStyle {
                        font_weight: Some(gpui::FontWeight::BOLD),
                        ..Default::default()
                    },
                )],
            ),
        ))
        // ── Font design ────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Font design: Serif (New York)"),
        )
        .child(
            TextView::new(
                "This text renders in the New York serif typeface, \
                 suitable for editorial and reading contexts per HIG.",
            )
            .font_design(FontDesign::Serif),
        )
        // ── Leading styles ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Leading: Tight vs Standard vs Loose"),
        )
        .child(
            div().flex().flex_col().gap(theme.spacing_xs).child(
                TextView::new(
                    "Tight leading — saves vertical space in constrained layouts \
                     like list rows and compact panels.",
                )
                .leading_style(LeadingStyle::Tight),
            ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .child(TextView::new(
                    "Standard leading — the default HIG line height for comfortable reading.",
                )),
        )
        .child(
            div().flex().flex_col().gap(theme.spacing_xs).child(
                TextView::new(
                    "Loose leading — extra spacing for wide columns and long-form passages.",
                )
                .leading_style(LeadingStyle::Loose),
            ),
        )
        // ── Label levels ───────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Label levels (HIG hierarchy)"),
        )
        .child(
            TextView::new("Primary text — the default label color.")
                .label_level(LabelLevel::Primary),
        )
        .child(
            TextView::new("Secondary text — supplemental or subheading content.")
                .label_level(LabelLevel::Secondary),
        )
        .child(
            TextView::new("Tertiary text — unavailable items or low-priority detail.")
                .label_level(LabelLevel::Tertiary),
        )
        .child(
            TextView::new("Quaternary text — watermark or placeholder-level content.")
                .label_level(LabelLevel::Quaternary),
        )
        // ── Disabled ───────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Disabled"),
        )
        .child(
            TextView::new(
                "This text view is in a disabled state, rendered at reduced \
                 opacity to signal it is inactive.",
            )
            .disabled(true),
        )
        // ── Readable width ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Readable width (544 pt)"),
        )
        .child(
            TextView::new(
                "This text view is constrained to the HIG readable-content optimal \
                 width of 544 points (~65 characters at macOS body size). This keeps \
                 long-form text comfortable to read regardless of the window width. \
                 The quick brown fox jumps over the lazy dog. Pack my box with five \
                 dozen liquor jugs.",
            )
            .readable_width(),
        )
        // ── Scrollable ─────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Scrollable"),
        )
        .child(
            div().max_h(gpui::px(80.0)).child(
                TextView::new(
                    "This is a scrollable text view. When the content exceeds the \
                         visible area, the text scrolls vertically. This matches the HIG \
                         requirement that text views support scrolling when content is \
                         taller than the view. The quick brown fox jumps over the lazy dog. \
                         Pack my box with five dozen liquor jugs. How vexingly quick daft \
                         zebras jump. The five boxing wizards jump quickly.",
                )
                .scrollable("scrollable-demo"),
            ),
        )
        .into_any_element()
}
