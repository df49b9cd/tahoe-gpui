//! ANSI escape code parser for terminal output rendering.
//!
//! Parses the SGR (Select Graphic Rendition) subset of ANSI escape codes
//! plus OSC 8 hyperlinks, and produces styled spans for GPUI rendering.
//!
//! Supported sequences:
//!
//! - CSI SGR (`ESC[…m`): standard and bright 8-colour palette,
//!   256-colour (`38;5;N`), 24-bit RGB (`38;2;R;G;B`), dim/faint (2/22),
//!   bold (1/22), italic (3/23), underline (4/24), blink (5/6/25),
//!   reverse/invert (7/27), conceal (8/28), strikethrough (9/29),
//!   and overline (53/55).
//! - OSC 8 hyperlinks (`ESC]8;params;URL BEL text ESC]8;; BEL`): attach
//!   a link URL to enclosed text. Terminal.app on macOS 26, iTerm2, Zed,
//!   and Windows Terminal all render these clickable.

use crate::foundations::theme::AnsiColors;
use gpui::{Hsla, SharedString, hsla};

/// Style attributes for a span of text.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnsiStyle {
    pub fg: Option<Hsla>,
    pub bg: Option<Hsla>,
    /// Bright-palette version of `fg` when `fg` was set from a standard
    /// 8-colour code (SGR 30–37). Used at render time by bold-is-bright
    /// to substitute the bright variant without re-parsing.
    pub fg_bright: Option<Hsla>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// SGR 2 — reduced intensity (faint). Rendered at render time as a
    /// lower-opacity variant of the resolved foreground.
    pub dim: bool,
    /// SGR 5/6 — blinking text. Stored so renderers that support blink
    /// (and respect Reduce Motion) can opt in; the default GPUI renderer
    /// does not blink since HIG and WCAG both discourage it.
    pub blink: bool,
    /// SGR 7 — reverse video. Swap foreground and background at render
    /// time so the terminal's default fg/bg are used when either side is
    /// unset.
    pub reverse: bool,
    /// SGR 8 — conceal. Text is still present in the model but rendered
    /// with foreground == background so it is visually hidden (matches
    /// xterm semantics for password prompts).
    pub conceal: bool,
    /// SGR 9 — strikethrough.
    pub strikethrough: bool,
    /// SGR 53 — overline (line above baseline). Rarely used but emitted
    /// by a handful of tools (e.g. `lesspipe` highlighting).
    pub overline: bool,
    /// OSC 8 hyperlink URL currently in scope. `None` means the span is
    /// not clickable.
    pub link: Option<SharedString>,
}

/// A span of text with ANSI styling applied.
#[derive(Debug, Clone)]
pub struct AnsiSpan {
    pub text: String,
    pub style: AnsiStyle,
}

/// Parse a string containing ANSI escape codes into styled spans.
pub fn parse_ansi(input: &str, colors: &AnsiColors) -> Vec<AnsiSpan> {
    parse_ansi_with_style(input, colors, AnsiStyle::default()).0
}

/// Parse ANSI codes with an initial style, returning spans and the final style state.
pub fn parse_ansi_with_style(
    input: &str,
    colors: &AnsiColors,
    initial_style: AnsiStyle,
) -> (Vec<AnsiSpan>, AnsiStyle) {
    let mut spans = Vec::new();
    let mut current_style = initial_style;
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                // CSI: ESC [
                Some('[') => {
                    chars.next();

                    if !current_text.is_empty() {
                        spans.push(AnsiSpan {
                            text: std::mem::take(&mut current_text),
                            style: current_style.clone(),
                        });
                    }

                    let mut params = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == ';' {
                            params.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    let final_char = chars.next();
                    if final_char == Some('m') {
                        apply_sgr(&params, &mut current_style, colors);
                    }
                    // Other CSI finals (H, J, K, …) are silently dropped —
                    // this is an output renderer, not a terminal emulator.
                }
                // OSC: ESC ]
                Some(']') => {
                    chars.next();

                    if !current_text.is_empty() {
                        spans.push(AnsiSpan {
                            text: std::mem::take(&mut current_text),
                            style: current_style.clone(),
                        });
                    }

                    // Collect OSC payload until BEL (\x07) or ST (ESC \).
                    let mut payload = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '\x07' {
                            chars.next();
                            break;
                        }
                        if c == '\x1b' {
                            chars.next();
                            // Expect '\\'
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        payload.push(c);
                        chars.next();
                    }

                    apply_osc(&payload, &mut current_style);
                }
                _ => {
                    current_text.push(ch);
                }
            }
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        spans.push(AnsiSpan {
            text: current_text,
            style: current_style.clone(),
        });
    }

    (spans, current_style)
}

fn apply_sgr(params: &str, style: &mut AnsiStyle, colors: &AnsiColors) {
    if params.is_empty() {
        let link = style.link.clone();
        *style = AnsiStyle {
            link,
            ..AnsiStyle::default()
        };
        return;
    }

    let codes: Vec<u32> = params
        .split(';')
        .map(|s| {
            if s.is_empty() {
                0
            } else {
                s.parse().unwrap_or(0)
            }
        })
        .collect();

    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => {
                // Reset preserves active OSC 8 link — per the de-facto
                // terminal standard, `\e[m` does not close a hyperlink.
                let link = style.link.clone();
                *style = AnsiStyle {
                    link,
                    ..AnsiStyle::default()
                };
            }
            1 => style.bold = true,
            2 => style.dim = true,
            3 => style.italic = true,
            4 => style.underline = true,
            5 | 6 => style.blink = true,
            7 => style.reverse = true,
            8 => style.conceal = true,
            9 => style.strikethrough = true,
            22 => {
                style.bold = false;
                style.dim = false;
            }
            23 => style.italic = false,
            24 => style.underline = false,
            25 => style.blink = false,
            27 => style.reverse = false,
            28 => style.conceal = false,
            29 => style.strikethrough = false,

            // Standard foreground colors (30-37) — store the bright
            // counterpart so bold-is-bright rendering can substitute
            // it without re-parsing.
            30 => set_basic_fg(style, colors.black, colors.bright_black),
            31 => set_basic_fg(style, colors.red, colors.bright_red),
            32 => set_basic_fg(style, colors.green, colors.bright_green),
            33 => set_basic_fg(style, colors.yellow, colors.bright_yellow),
            34 => set_basic_fg(style, colors.blue, colors.bright_blue),
            35 => set_basic_fg(style, colors.magenta, colors.bright_magenta),
            36 => set_basic_fg(style, colors.cyan, colors.bright_cyan),
            37 => set_basic_fg(style, colors.white, colors.bright_white),
            39 => {
                style.fg = None;
                style.fg_bright = None;
            }

            // Standard background colors (40-47)
            40 => style.bg = Some(colors.black),
            41 => style.bg = Some(colors.red),
            42 => style.bg = Some(colors.green),
            43 => style.bg = Some(colors.yellow),
            44 => style.bg = Some(colors.blue),
            45 => style.bg = Some(colors.magenta),
            46 => style.bg = Some(colors.cyan),
            47 => style.bg = Some(colors.white),
            49 => style.bg = None,

            // SGR 53/55: overline on/off.
            53 => style.overline = true,
            55 => style.overline = false,

            // Bright foreground colors (90-97). These bypass bold-is-bright.
            90 => {
                style.fg = Some(colors.bright_black);
                style.fg_bright = None;
            }
            91 => {
                style.fg = Some(colors.bright_red);
                style.fg_bright = None;
            }
            92 => {
                style.fg = Some(colors.bright_green);
                style.fg_bright = None;
            }
            93 => {
                style.fg = Some(colors.bright_yellow);
                style.fg_bright = None;
            }
            94 => {
                style.fg = Some(colors.bright_blue);
                style.fg_bright = None;
            }
            95 => {
                style.fg = Some(colors.bright_magenta);
                style.fg_bright = None;
            }
            96 => {
                style.fg = Some(colors.bright_cyan);
                style.fg_bright = None;
            }
            97 => {
                style.fg = Some(colors.bright_white);
                style.fg_bright = None;
            }

            // Bright background colors (100-107)
            100 => style.bg = Some(colors.bright_black),
            101 => style.bg = Some(colors.bright_red),
            102 => style.bg = Some(colors.bright_green),
            103 => style.bg = Some(colors.bright_yellow),
            104 => style.bg = Some(colors.bright_blue),
            105 => style.bg = Some(colors.bright_magenta),
            106 => style.bg = Some(colors.bright_cyan),
            107 => style.bg = Some(colors.bright_white),

            // 256-color and 24-bit RGB
            38 | 48 => {
                let is_fg = codes[i] == 38;
                if i + 1 < codes.len() && codes[i + 1] == 5 && i + 2 < codes.len() {
                    let color = color_from_256(codes[i + 2], colors);
                    if is_fg {
                        style.fg = Some(color);
                        style.fg_bright = None;
                    } else {
                        style.bg = Some(color);
                    }
                    i += 2;
                } else if i + 1 < codes.len() && codes[i + 1] == 2 && i + 4 < codes.len() {
                    let color = color_from_rgb(codes[i + 2], codes[i + 3], codes[i + 4]);
                    if is_fg {
                        style.fg = Some(color);
                        style.fg_bright = None;
                    } else {
                        style.bg = Some(color);
                    }
                    i += 4;
                }
            }

            _ => {}
        }
        i += 1;
    }
}

fn set_basic_fg(style: &mut AnsiStyle, normal: Hsla, bright: Hsla) {
    style.fg = Some(normal);
    style.fg_bright = Some(bright);
}

/// Apply an OSC payload to `style`. Currently handles OSC 8 (hyperlinks)
/// and silently drops other OSC commands (window title, clipboard, …).
///
/// OSC 8 format: `8;params;URL` — empty URL closes the hyperlink. Params
/// are key=value pairs separated by `:`; they are currently ignored.
fn apply_osc(payload: &str, style: &mut AnsiStyle) {
    let mut parts = payload.splitn(3, ';');
    let command = parts.next().unwrap_or("");
    if command != "8" {
        return;
    }
    let _params = parts.next().unwrap_or("");
    let url = parts.next().unwrap_or("");
    if url.is_empty() {
        style.link = None;
    } else {
        style.link = Some(SharedString::from(url.to_string()));
    }
}

/// Convert a 256-color index to an HSLA color.
/// 0-7: standard colors, 8-15: bright colors, 16-231: 6x6x6 color cube, 232-255: grayscale ramp.
fn color_from_256(index: u32, colors: &AnsiColors) -> Hsla {
    match index {
        0 => colors.black,
        1 => colors.red,
        2 => colors.green,
        3 => colors.yellow,
        4 => colors.blue,
        5 => colors.magenta,
        6 => colors.cyan,
        7 => colors.white,
        8 => colors.bright_black,
        9 => colors.bright_red,
        10 => colors.bright_green,
        11 => colors.bright_yellow,
        12 => colors.bright_blue,
        13 => colors.bright_magenta,
        14 => colors.bright_cyan,
        15 => colors.bright_white,
        16..=231 => {
            // 6x6x6 color cube (xterm standard: v==0 ? 0 : 55+40*v)
            let idx = index - 16;
            let to_val = |v: u32| -> u32 { if v == 0 { 0 } else { 55 + 40 * v } };
            let r = to_val(idx / 36);
            let g = to_val((idx % 36) / 6);
            let b = to_val(idx % 6);
            color_from_rgb(r, g, b)
        }
        232..=255 => {
            // Grayscale ramp (24 shades)
            let l = ((index - 232) as f32 * 10.0 + 8.0) / 255.0;
            hsla(0.0, 0.0, l, 1.0)
        }
        _ => colors.white,
    }
}

/// Convert RGB values (0-255 each) to HSLA.
fn color_from_rgb(r: u32, g: u32, b: u32) -> Hsla {
    let rf = r.min(255) as f32 / 255.0;
    let gf = g.min(255) as f32 / 255.0;
    let bf = b.min(255) as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return hsla(0.0, 0.0, l, 1.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - rf).abs() < f32::EPSILON {
        ((gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }) / 6.0
    } else if (max - gf).abs() < f32::EPSILON {
        ((bf - rf) / d + 2.0) / 6.0
    } else {
        ((rf - gf) / d + 4.0) / 6.0
    };

    hsla(h, s, l, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{AnsiStyle, color_from_256, color_from_rgb, parse_ansi};
    use crate::foundations::theme::AnsiColors;
    use crate::foundations::theme::TahoeTheme;
    use core::prelude::v1::test;

    fn test_colors() -> AnsiColors {
        TahoeTheme::dark().ansi
    }

    #[test]
    fn plain_text() {
        let spans = parse_ansi("hello world", &test_colors());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "hello world");
        assert!(spans[0].style.fg.is_none());
    }

    #[test]
    fn bold_text() {
        let spans = parse_ansi("\x1b[1mhello\x1b[0m world", &test_colors());
        assert_eq!(spans.len(), 2);
        assert!(spans[0].style.bold);
        assert_eq!(spans[0].text, "hello");
        assert!(!spans[1].style.bold);
        assert_eq!(spans[1].text, " world");
    }

    #[test]
    fn colored_text() {
        let spans = parse_ansi("\x1b[31mred\x1b[32mgreen\x1b[0mnormal", &test_colors());
        assert_eq!(spans.len(), 3);
        assert!(spans[0].style.fg.is_some());
        assert_eq!(spans[0].text, "red");
        assert!(spans[1].style.fg.is_some());
        assert_eq!(spans[1].text, "green");
        assert!(spans[2].style.fg.is_none());
        assert_eq!(spans[2].text, "normal");
    }

    #[test]
    fn empty_input() {
        let spans = parse_ansi("", &test_colors());
        assert!(spans.is_empty());
    }

    #[test]
    fn incomplete_escape() {
        let spans = parse_ansi("\x1bhello", &test_colors());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "\x1bhello");
    }

    #[test]
    fn combined_bold_and_color() {
        let spans = parse_ansi("\x1b[1;31mbold red\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.bold);
        assert!(spans[0].style.fg.is_some());
    }

    #[test]
    fn multiple_resets() {
        let spans = parse_ansi("\x1b[31mred\x1b[0m\x1b[0mnormal", &test_colors());
        let last = spans.last().unwrap();
        assert!(last.style.fg.is_none());
        assert!(!last.style.bold);
    }

    #[test]
    fn plain_text_no_escapes() {
        let spans = parse_ansi("just plain text", &test_colors());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "just plain text");
        assert!(spans[0].style.fg.is_none());
        assert!(!spans[0].style.bold);
    }

    #[test]
    fn bold_with_reset() {
        let spans = parse_ansi("\x1b[1mbold\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.bold);
        assert!(!spans.last().unwrap().style.bold);
    }

    #[test]
    fn underline_text() {
        let spans = parse_ansi("\x1b[4munderlined\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.underline);
    }

    #[test]
    fn color_256_foreground() {
        let spans = parse_ansi("\x1b[38;5;196mred\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.fg.is_some());
        assert_eq!(spans[0].text, "red");
    }

    #[test]
    fn color_256_background() {
        let spans = parse_ansi("\x1b[48;5;21mblue bg\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.bg.is_some());
    }

    #[test]
    fn color_256_grayscale() {
        let spans = parse_ansi("\x1b[38;5;240mgray\x1b[0m", &test_colors());
        assert!(spans[0].style.fg.is_some());
    }

    #[test]
    fn color_24bit_rgb_foreground() {
        let spans = parse_ansi("\x1b[38;2;255;128;0morange\x1b[0mnormal", &test_colors());
        assert!(spans.len() >= 2);
        assert!(spans[0].style.fg.is_some());
        assert_eq!(spans[0].text, "orange");
    }

    #[test]
    fn color_24bit_rgb_background() {
        let spans = parse_ansi("\x1b[48;2;0;0;128mnavy bg\x1b[0m", &test_colors());
        assert!(spans[0].style.bg.is_some());
    }

    #[test]
    fn color_from_rgb_pure_red() {
        let c = color_from_rgb(255, 0, 0);
        assert!(c.l > 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn color_from_rgb_pure_white() {
        let c = color_from_rgb(255, 255, 255);
        assert!(c.l > 0.9);
        assert_eq!(c.s, 0.0);
    }

    #[test]
    fn color_from_256_standard_maps_to_theme() {
        let colors = test_colors();
        assert_eq!(color_from_256(0, &colors), colors.black);
        assert_eq!(color_from_256(1, &colors), colors.red);
        assert_eq!(color_from_256(15, &colors), colors.bright_white);
    }

    // ─── New SGR coverage (findings #5/#8) ─────────────────────────────

    #[test]
    fn dim_sets_and_clears() {
        // SGR 2 enables dim; SGR 22 clears both bold and dim.
        let spans = parse_ansi("\x1b[2mfaint\x1b[22mnormal", &test_colors());
        assert!(spans[0].style.dim);
        assert!(!spans.last().unwrap().style.dim);
    }

    #[test]
    fn reverse_video() {
        let spans = parse_ansi("\x1b[7minv\x1b[27mnormal", &test_colors());
        assert!(spans[0].style.reverse);
        assert!(!spans.last().unwrap().style.reverse);
    }

    #[test]
    fn strikethrough() {
        let spans = parse_ansi("\x1b[9mstruck\x1b[29mnormal", &test_colors());
        assert!(spans[0].style.strikethrough);
        assert!(!spans.last().unwrap().style.strikethrough);
    }

    #[test]
    fn blink_modes() {
        let spans = parse_ansi("\x1b[5ms\x1b[6mf\x1b[25mplain", &test_colors());
        assert!(spans[0].style.blink);
        assert!(spans[1].style.blink);
        assert!(!spans.last().unwrap().style.blink);
    }

    #[test]
    fn overline_sets_and_clears() {
        let spans = parse_ansi("\x1b[53mover\x1b[55mnormal", &test_colors());
        assert!(spans[0].style.overline);
        assert!(!spans.last().unwrap().style.overline);
    }

    #[test]
    fn conceal_sets_and_clears() {
        let spans = parse_ansi("\x1b[8mhidden\x1b[28mshown", &test_colors());
        assert!(spans[0].style.conceal);
        assert!(!spans.last().unwrap().style.conceal);
    }

    // ─── bold-is-bright (finding #5) ───────────────────────────────────

    #[test]
    fn standard_fg_carries_bright_variant() {
        // SGR 31 is red; parser should populate both fg and fg_bright so
        // the renderer can substitute bright_red when bold is active.
        let spans = parse_ansi("\x1b[31mred", &test_colors());
        let colors = test_colors();
        assert_eq!(spans[0].style.fg, Some(colors.red));
        assert_eq!(spans[0].style.fg_bright, Some(colors.bright_red));
    }

    #[test]
    fn bright_fg_has_no_bright_variant() {
        // SGR 91 is already bright; fg_bright stays None so bold doesn't
        // double-promote.
        let spans = parse_ansi("\x1b[91mred", &test_colors());
        assert!(spans[0].style.fg.is_some());
        assert!(spans[0].style.fg_bright.is_none());
    }

    #[test]
    fn rgb_fg_has_no_bright_variant() {
        // 24-bit RGB is an explicit choice; bold-is-bright should not
        // override it.
        let spans = parse_ansi("\x1b[38;2;10;20;30mrgb", &test_colors());
        assert!(spans[0].style.fg.is_some());
        assert!(spans[0].style.fg_bright.is_none());
    }

    // ─── OSC 8 hyperlinks (finding #9) ─────────────────────────────────

    #[test]
    fn osc8_sets_and_closes_link() {
        // Opening OSC 8 attaches a URL; closing OSC 8 (empty URL) removes it.
        let spans = parse_ansi(
            "\x1b]8;;https://apple.com\x07text\x1b]8;;\x07after",
            &test_colors(),
        );
        assert_eq!(
            spans[0].style.link.as_ref().map(|s| s.as_ref()),
            Some("https://apple.com")
        );
        assert!(spans.last().unwrap().style.link.is_none());
    }

    #[test]
    fn osc8_st_terminator() {
        // ESC \ (ST) also terminates OSC.
        let spans = parse_ansi(
            "\x1b]8;;https://example.com\x1b\\text\x1b]8;;\x1b\\",
            &test_colors(),
        );
        assert_eq!(
            spans[0].style.link.as_ref().map(|s| s.as_ref()),
            Some("https://example.com")
        );
    }

    #[test]
    fn sgr_reset_preserves_osc8_link() {
        // Real-world: tools emit `\e]8;;url\e\\ \e[31mred\e[0m label \e]8;;\e\\`.
        // The reset in the middle must not close the hyperlink.
        let spans = parse_ansi(
            "\x1b]8;;https://apple.com\x07\x1b[31mred\x1b[0m after\x1b]8;;\x07end",
            &test_colors(),
        );
        // Every span inside the hyperlink brackets has a link set.
        assert_eq!(
            spans[0].style.link.as_ref().map(|s| s.as_ref()),
            Some("https://apple.com")
        );
        assert_eq!(
            spans[1].style.link.as_ref().map(|s| s.as_ref()),
            Some("https://apple.com")
        );
        assert!(spans.last().unwrap().style.link.is_none());
    }

    #[test]
    fn non_osc8_sequences_ignored() {
        // OSC 0 (set window title) is dropped — should not leak into text.
        let spans = parse_ansi("\x1b]0;Build\x07hello", &test_colors());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "hello");
    }

    // ─── Style equality for span-merging optimisations ─────────────────

    #[test]
    fn default_style_equality() {
        assert_eq!(AnsiStyle::default(), AnsiStyle::default());
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_ansi_never_panics(input in "[\\x1b\\[0-9;m a-zA-Z]*") {
            let colors = test_colors();
            let _ = parse_ansi(&input, &colors);
        }
    }
}
