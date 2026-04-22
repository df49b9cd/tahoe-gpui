//! Text Views demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, HighlightStyle, StrikethroughStyle, UnderlineStyle, Window, div};

use tahoe_gpui::components::content::text_view::{TextRuns, TextView};
use tahoe_gpui::foundations::accessibility::{HeadingLevel, TextContentType};
use tahoe_gpui::foundations::theme::{
    FontDesign, LabelLevel, LeadingStyle, TahoeTheme, TextCase, TextStyle, TextStyledExt,
    TruncationMode,
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
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "The quick brown fox jumps over the lazy dog. \
                 This text view uses the default Body text style.",
            )
        }))
        // ── Title 1 ────────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Title 1 style"),
        )
        .child(
            cx.new(|cx| {
                TextView::new(cx, "Large styled heading text").text_style(TextStyle::Title1)
            }),
        )
        // ── Caption ────────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Caption style"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "Small caption text suitable for footnotes and metadata.",
            )
            .text_style(TextStyle::Caption1)
        }))
        // ── Emphasized ─────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Emphasized"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "This body text uses the HIG emphasized weight (Semibold). \
                 Useful for lead paragraphs or standout content blocks.",
            )
            .emphasize(true)
        }))
        // ── Bold / weight override ─────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Bold (SwiftUI parity)"),
        )
        .child(cx.new(|cx| {
            // `.bold()` is an alias for `.weight(FontWeight::BOLD)` —
            // matches SwiftUI's `Text.bold()` modifier. Prefer
            // `.emphasize(true)` when the surrounding text style already
            // carries its own emphasized weight (e.g. Headline → BLACK).
            TextView::new(cx, "Bold body text via `.bold()`.").bold()
        }))
        // ── Italic ─────────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Italic"),
        )
        .child(cx.new(|cx| TextView::new(cx, "Italic body text via `.italic(true)`.").italic(true)))
        // ── Underline ──────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Underline"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "Underlined text via `.underline(UnderlineStyle::default())`.",
            )
            .underline(UnderlineStyle::default())
        }))
        // ── Strikethrough ──────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Strikethrough"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "Struck-through text via `.strikethrough(StrikethroughStyle::default())`.",
            )
            .strikethrough(StrikethroughStyle::default())
        }))
        // ── Monospaced digits ──────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Monospaced digits (tnum)"),
        )
        .child(cx.new(|cx| {
            // Constant-width digits — counters and timestamps don't
            // jitter between frames when a proportional "1" widens into
            // a "0". OpenType feature `tnum` is enabled.
            TextView::new(cx, "00:12:34  00:12:35  00:12:36").monospaced_digit(true)
        }))
        // ── Ligatures disabled ─────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Ligatures disabled (calt=0)"),
        )
        .child(cx.new(|cx| {
            // Disable contextual alternates so programming ligatures
            // like `=>` and `!=` render as individual glyphs. Useful
            // inside code-style snippets where the compiler token
            // matters more than prose typography.
            TextView::new(cx, "fn compare() { a != b && c => d }")
                .font_design(FontDesign::Monospaced)
                .ligatures(false)
        }))
        // ── Text case ──────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Text case transform"),
        )
        .child(
            cx.new(|cx| {
                TextView::new(cx, "Uppercase via text_case").text_case(TextCase::Uppercase)
            }),
        )
        .child(
            cx.new(|cx| {
                TextView::new(cx, "Lowercase VIA text_case").text_case(TextCase::Lowercase)
            }),
        )
        .child(cx.new(|cx| {
            TextView::new(cx, "sentence case via text_case").text_case(TextCase::SentenceCase)
        }))
        // ── max_lines ──────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With max_lines(2)"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "This text view has max_lines set to 2. Content beyond two \
                 lines is clipped via GPUI's native line-clamp support.",
            )
            .max_lines(2)
        }))
        // ── Styled text ────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Styled text"),
        )
        .child(cx.new(|cx| {
            TextView::new(cx, "placeholder").styled_text(
                "Bold and accent-colored text within a single view.",
                vec![(
                    0..4,
                    HighlightStyle {
                        font_weight: Some(gpui::FontWeight::BOLD),
                        ..Default::default()
                    },
                )],
            )
        }))
        // ── TextRuns composition (SwiftUI `Text + Text`) ───────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("TextRuns composition"),
        )
        .child(cx.new(|cx| {
            // Build a single view from an ordered sequence of runs.
            // Matches SwiftUI's `Text("a") + Text("b").bold()` idiom and
            // keeps selection / copy / VoiceOver seeing one merged string.
            TextView::from_runs(
                cx,
                TextRuns::new()
                    .plain("Visit the ")
                    .bold("Human Interface Guidelines")
                    .plain(" to read more about ")
                    .italic("typography")
                    .plain(" and ")
                    .underline("accessibility")
                    .plain("."),
            )
        }))
        // ── Formatted content (SwiftUI `Text(_ value, format:)`) ───────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Formatted content"),
        )
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "TextView::formatted delegates to the value's Display impl, \
                     making the intent (a formatted value, not verbatim copy) \
                     visible at the call site.",
                ),
        )
        .child(cx.new(|cx| TextView::formatted(cx, 42_u32)))
        .child(cx.new(|cx| TextView::formatted(cx, 99.5_f64)))
        .child(cx.new(|cx| TextView::formatted(cx, true)))
        // ── Font design ────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Font design: Serif (New York)"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "This text renders in the New York serif typeface, \
                 suitable for editorial and reading contexts per HIG.",
            )
            .font_design(FontDesign::Serif)
        }))
        // ── Leading styles ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Leading: Tight vs Standard vs Loose"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .child(cx.new(|cx| {
                    TextView::new(
                        cx,
                        "Tight leading — saves vertical space in constrained layouts \
                     like list rows and compact panels.",
                    )
                    .leading_style(LeadingStyle::Tight)
                })),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .child(cx.new(|cx| {
                    TextView::new(
                        cx,
                        "Standard leading — the default HIG line height for comfortable reading.",
                    )
                })),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .child(cx.new(|cx| {
                    TextView::new(
                        cx,
                        "Loose leading — extra spacing for wide columns and long-form passages.",
                    )
                    .leading_style(LeadingStyle::Loose)
                })),
        )
        // ── Label levels ───────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Label levels (HIG hierarchy)"),
        )
        .child(cx.new(|cx| {
            TextView::new(cx, "Primary text — the default label color.")
                .label_level(LabelLevel::Primary)
        }))
        .child(cx.new(|cx| {
            TextView::new(cx, "Secondary text — supplemental or subheading content.")
                .label_level(LabelLevel::Secondary)
        }))
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "Tertiary text — unavailable items or low-priority detail.",
            )
            .label_level(LabelLevel::Tertiary)
        }))
        .child(cx.new(|cx| {
            // Quaternary is HIG's watermark / empty-state tier — use it for
            // background-style hints that should recede into the surface,
            // not for running prose.
            TextView::new(cx, "Drop a file here to start")
                .label_level(LabelLevel::Quaternary)
                .text_align(gpui::TextAlign::Center)
        }))
        .child(cx.new(|cx| {
            // Quinary (macOS Tahoe) is the lightest tier — reserved for
            // decorative separators or timestamps that should not compete
            // with nearby primary content. Pair it with Caption1 (not
            // Caption2) so the combination of the low-contrast colour and
            // the small size still clears WCAG AA.
            TextView::new(cx, "Last updated 2 min ago")
                .label_level(LabelLevel::Quinary)
                .text_style(TextStyle::Caption1)
        }))
        // ── Disabled look via explicit color ───────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Disabled look"),
        )
        .child(cx.new(|cx| {
            // TextView has no .disabled() — it's read-only with no
            // interactive state. For a disabled appearance, pass the
            // disabled color directly.
            TextView::new(
                cx,
                "This text view uses theme.text_disabled() to signal that \
                 the surrounding control is inactive.",
            )
            .color(theme.text_disabled())
        }))
        // ── Readable width ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Readable width (544 pt)"),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "This text view is constrained to the HIG readable-content optimal \
                 width of 544 points (~65 characters at macOS body size). This keeps \
                 long-form text comfortable to read regardless of the window width. \
                 The quick brown fox jumps over the lazy dog. Pack my box with five \
                 dozen liquor jugs.",
            )
            .readable_width(true)
        }))
        // ── Text alignment ─────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Text alignment"),
        )
        .child(cx.new(|cx| {
            TextView::new(cx, "Leading alignment — the HIG default for body text.")
                .text_align(gpui::TextAlign::Left)
        }))
        .child(cx.new(|cx| {
            TextView::new(cx, "Centered alignment — short headlines, empty states.")
                .text_align(gpui::TextAlign::Center)
        }))
        .child(cx.new(|cx| {
            TextView::new(cx, "Trailing alignment — numeric columns, timestamps.")
                .text_align(gpui::TextAlign::Right)
        }))
        // ── Truncation mode ────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Truncation mode (Tail / Head / Middle)"),
        )
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "Head / Tail map to GPUI's native text-overflow. \
                     Middle emulates NSLineBreakMode.byTruncatingMiddle via \
                     a character-budget helper — useful for file paths and \
                     breadcrumbs where both ends matter.",
                ),
        )
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "crates/tahoe-gpui/src/components/content/text_view.rs",
            )
            .truncation_mode(TruncationMode::Middle)
            .truncation_char_budget(32)
        }))
        .child(cx.new(|cx| {
            TextView::new(
                cx,
                "/Users/example/projects/tahoe/crates/tahoe-gpui/src/components/content/text_view.rs",
            )
            .truncation_mode(TruncationMode::Middle)
            .truncation_char_budget(40)
        }))
        // ── Non-selectable ────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Non-selectable"),
        )
        .child(cx.new(|cx| {
            // `selectable(false)` opts out of drag-select, shift-click,
            // ⌘A / ⌘C, and the context menu. Use it for labels that
            // decorate a control (tick-marks on a slider, axis labels on
            // a chart) where selection would interfere with the
            // surrounding gesture.
            TextView::new(cx, "Try to drag-select this — the gesture is suppressed.")
                .selectable(false)
        }))
        // ── Accessibility label ───────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Accessibility label"),
        )
        .child(cx.new(|cx| {
            // When the visual text is ambiguous or abbreviated, provide
            // a separate VoiceOver label so screen-reader users hear the
            // full meaning without cluttering the on-screen copy.
            TextView::new(cx, "⌘⇧N").accessibility_label("Keyboard shortcut: Command Shift N")
        }))
        // ── Accessibility heading + content type ──────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Accessibility heading & content type"),
        )
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "Promoting the AX role to Heading lets VoiceOver's \
                     next-heading rotor gesture skip to this element. \
                     Classifying the content type tunes reading cadence \
                     for file paths, source code, console output, etc. \
                     No-op today; wired as soon as GPUI exposes an AX tree.",
                ),
        )
        .child(cx.new(|cx| {
            TextView::new(cx, "Release notes")
                .text_style(TextStyle::Title2)
                .accessibility_heading(HeadingLevel::new_clamped(2))
        }))
        .child(cx.new(|cx| {
            TextView::new(cx, "/Users/example/Documents/report.pdf")
                .accessibility_text_content_type(TextContentType::FileSystemPath)
        }))
        .child(cx.new(|cx| {
            TextView::new(cx, "error: cannot find function `foo` in this scope")
                .font_design(FontDesign::Monospaced)
                .accessibility_text_content_type(TextContentType::ConsoleOutput)
        }))
        // ── Scrollable ─────────────────────────────────────────────────────
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Scrollable"),
        )
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(
                    "Focus the view, then Up / Down / Page / Home / End \
                     move the viewport. Shift+F10 opens Copy / Select All \
                     from the keyboard.",
                ),
        )
        .child(div().max_h(gpui::px(120.0)).child(cx.new(|cx| {
            TextView::new(
                cx,
                "This is a scrollable text view. When the content exceeds the \
                     visible area, the text scrolls vertically. This matches the HIG \
                     requirement that text views support scrolling when content is \
                     taller than the view. The quick brown fox jumps over the lazy dog. \
                     Pack my box with five dozen liquor jugs. How vexingly quick daft \
                     zebras jump. The five boxing wizards jump quickly.",
            )
            .scrollable("scrollable-demo")
        })))
        .into_any_element()
}
