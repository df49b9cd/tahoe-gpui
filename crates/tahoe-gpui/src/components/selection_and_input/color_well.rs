//! HIG Color Well -- swatch button that opens a color picker grid.
//!
//! A stateless `RenderOnce` component that renders a color swatch trigger.
//! When open, an absolute-positioned dropdown displays a 6x3 grid of preset
//! colors from the theme's system color palette (18 colors), plus numeric
//! RGB or HSB entry rows, a hex field, and an alpha scrubber. The selected
//! color shows a white Check icon overlay.

use gpui::prelude::*;
use gpui::{
    App, ElementId, Hsla, KeyDownEvent, MouseDownEvent, Rgba, SharedString, Window, deferred, div,
    hsla, px,
};

use crate::callback_types::{OnHslaChange, OnToggle, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::materials::apply_high_contrast_border;
use crate::foundations::materials::glass_surface;
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle, TextStyledExt};

/// Swatch size inside the dropdown grid (32x32pt).
const SWATCH_SIZE: f32 = 32.0;

/// Number of columns in the color grid.
const GRID_COLUMNS: usize = 6;

/// Compact trigger width (HIG macOS Tahoe).
const COMPACT_WIDTH: f32 = 28.0;
/// Compact trigger height (HIG macOS Tahoe).
const COMPACT_HEIGHT: f32 = 14.0;

/// Number of ticks in the alpha scrubber row (8 steps: 0%, 12.5%, 25%, …, 100%).
const ALPHA_TICK_COUNT: usize = 8;

/// Visual style of the color well trigger.
///
/// Per HIG macOS Tahoe, `Compact` is the recommended default for toolbar
/// and inline contexts; `Expanded` is the pre-Tahoe 44pt square retained
/// for backcompat and for wells that need a larger tap target.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorWellStyle {
    /// 28×14pt rounded-rect swatch with a trailing chevron glyph.
    Compact,
    /// 44×44pt square swatch; pre-Tahoe default. No chevron.
    ///
    /// Retained as the library default to preserve the behaviour of
    /// `ColorWell::new(...).color(...)` callers written before
    /// `ColorWellStyle` existed.
    #[default]
    Expanded,
}

/// Which numeric entry tab is shown inside the popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverTab {
    /// R/G/B 0-255 inputs.
    #[default]
    Rgb,
    /// H (0-360) / S (0-100) / B (0-100) inputs.
    Hsb,
}

/// Returns all 18 system colors from the theme as `(name, Hsla)` pairs.
fn system_color_palette(theme: &TahoeTheme) -> Vec<(&'static str, Hsla)> {
    let p = &theme.palette;
    vec![
        ("red", p.red),
        ("orange", p.orange),
        ("yellow", p.yellow),
        ("green", p.green),
        ("mint", p.mint),
        ("teal", p.teal),
        ("cyan", p.cyan),
        ("blue", p.blue),
        ("indigo", p.indigo),
        ("purple", p.purple),
        ("pink", p.pink),
        ("brown", p.brown),
        ("gray", p.gray),
        ("gray2", p.gray2),
        ("gray3", p.gray3),
        ("gray4", p.gray4),
        ("gray5", p.gray5),
        ("gray6", p.gray6),
    ]
}

/// HIG Color Well -- swatch button that opens a color picker grid.
///
/// Closed state shows either a 28×14pt compact pill with chevron
/// (`ColorWellStyle::Compact`) or a 44×44pt rounded square filled with
/// the current color (`ColorWellStyle::Expanded`). Open state adds an
/// absolute-positioned dropdown with a glass surface containing a 6x3
/// grid of preset swatches, RGB/HSB numeric tabs, a hex input, and an
/// 8-tick alpha scrubber.
#[derive(IntoElement)]
#[allow(clippy::type_complexity)]
pub struct ColorWell {
    id: ElementId,
    color: Hsla,
    is_open: bool,
    style: ColorWellStyle,
    popover_tab: PopoverTab,
    on_change: OnHslaChange,
    on_toggle: OnToggle,
    /// Fired when arrow keys move the grid highlight. Stateless — parent
    /// owns the `highlighted_index` state.
    on_highlight: Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>,
    /// Fires when the user clicks the RGB/HSB tab header inside the
    /// popover. Stateless — parent owns the currently selected tab.
    on_tab_change: Option<Box<dyn Fn(PopoverTab, &mut Window, &mut App) + 'static>>,
    /// When `Some`, render a hex text entry row below the grid with the
    /// supplied draft as its current value. Parents manage the draft by
    /// listening to `on_hex_input` and commit a parsed color on Enter
    /// via `on_hex_commit`. Matches NSColorPanel's hex field.
    hex_draft: Option<SharedString>,
    /// Fires on every hex-entry keystroke while the dropdown is open,
    /// with the updated draft string. Paired with `hex_draft`.
    on_hex_input: Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
    /// Fires when the user presses Enter inside the hex entry row, with
    /// the parsed `Hsla` color. Hosts that supply this *without*
    /// `hex_draft` see an error on the key handler — draft ownership
    /// stays with the parent.
    on_hex_commit: Option<Box<dyn Fn(Hsla, &mut Window, &mut App) + 'static>>,
    /// Index of the keyboard-highlighted color in the grid.
    highlighted_index: Option<usize>,
    /// Whether this color well is keyboard-focused.
    focused: bool,
}

impl ColorWell {
    /// Create a new color well with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            // Intentional default: blue-ish demo swatch
            color: hsla(0.6, 0.7, 0.5, 1.0),
            is_open: false,
            style: ColorWellStyle::default(),
            popover_tab: PopoverTab::default(),
            on_change: None,
            on_toggle: None,
            on_highlight: None,
            on_tab_change: None,
            hex_draft: None,
            on_hex_input: None,
            on_hex_commit: None,
            highlighted_index: None,
            focused: false,
        }
    }

    /// Current hex-entry draft string. When set, the dropdown shows an
    /// editable "#" hex row below the grid.
    pub fn hex_draft(mut self, draft: impl Into<SharedString>) -> Self {
        self.hex_draft = Some(draft.into());
        self
    }

    /// Fires on every keystroke in the hex-entry row with the updated
    /// draft. Parents typically route this into their local state.
    pub fn on_hex_input(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hex_input = Some(Box::new(handler));
        self
    }

    /// Fires when the user presses Enter in the hex-entry row and the
    /// draft parses to a valid `#RRGGBB` or `#RRGGBBAA` colour.
    pub fn on_hex_commit(
        mut self,
        handler: impl Fn(Hsla, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_hex_commit = Some(Box::new(handler));
        self
    }

    /// Fire a callback when arrow keys move the grid highlight.
    pub fn on_highlight(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }

    /// Fire a callback when the RGB/HSB tab header changes.
    pub fn on_tab_change(
        mut self,
        handler: impl Fn(PopoverTab, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_tab_change = Some(Box::new(handler));
        self
    }

    /// Set the visual style of the trigger.
    pub fn style(mut self, style: ColorWellStyle) -> Self {
        self.style = style;
        self
    }

    /// Set which numeric entry tab is shown inside the popover.
    pub fn popover_tab(mut self, tab: PopoverTab) -> Self {
        self.popover_tab = tab;
        self
    }

    /// Set the currently selected color.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    /// Set the open/closed state of the dropdown grid.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the handler called when the user picks a color from the grid.
    pub fn on_change(mut self, handler: impl Fn(Hsla, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the handler called when the dropdown opens or closes.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Marks this color well as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the keyboard-highlighted swatch index in the grid.
    pub fn highlighted_index(mut self, index: Option<usize>) -> Self {
        self.highlighted_index = index;
        self
    }
}

/// Returns `true` when two HSLA colors are visually identical (within epsilon).
fn hsla_eq(a: Hsla, b: Hsla) -> bool {
    (a.h - b.h).abs() < 0.01
        && (a.s - b.s).abs() < 0.01
        && (a.l - b.l).abs() < 0.01
        && (a.a - b.a).abs() < 0.01
}

/// Render an `Hsla` colour as `#RRGGBB` (or `#RRGGBBAA` when alpha < 1).
fn hsla_to_hex(color: Hsla) -> String {
    let rgba = Rgba::from(color);
    let r = (rgba.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (rgba.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (rgba.b * 255.0).round().clamp(0.0, 255.0) as u8;
    let a = (rgba.a * 255.0).round().clamp(0.0, 255.0) as u8;
    if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }
}

/// Parse `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` into `Hsla`. Whitespace
/// around the leading `#` is tolerated; the leading `#` itself is optional.
fn parse_hex(input: &str) -> Option<Hsla> {
    let trimmed = input.trim().trim_start_matches('#');
    if !trimmed.is_ascii() {
        return None;
    }
    let (r, g, b, a) = match trimmed.len() {
        3 => {
            let r = u8::from_str_radix(&trimmed[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&trimmed[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&trimmed[2..3].repeat(2), 16).ok()?;
            (r, g, b, 255)
        }
        4 => {
            let r = u8::from_str_radix(&trimmed[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&trimmed[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&trimmed[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&trimmed[3..4].repeat(2), 16).ok()?;
            (r, g, b, a)
        }
        6 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
            let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
            let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
            let a = u8::from_str_radix(&trimmed[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    let rgba = Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    };
    Some(rgba.into())
}

/// Return the R, G, B (each 0-255) components of an Hsla colour.
fn hsla_to_rgb_bytes(color: Hsla) -> (u8, u8, u8) {
    let rgba = Rgba::from(color);
    let r = (rgba.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (rgba.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (rgba.b * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

/// Convert HSL (0..=1 hue/sat/light) to HSB (hue in degrees 0-360,
/// saturation 0-100, brightness 0-100). Matches NSColorPanel's HSB tab.
fn hsla_to_hsb(color: Hsla) -> (u32, u32, u32) {
    let h = (color.h * 360.0).round().clamp(0.0, 360.0) as u32;
    let l = color.l.clamp(0.0, 1.0);
    let s = color.s.clamp(0.0, 1.0);
    // HSL -> HSV conversion. v = l + s * min(l, 1 - l)
    let v = l + s * l.min(1.0 - l);
    let sv = if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) };
    let s_pct = (sv * 100.0).round().clamp(0.0, 100.0) as u32;
    let b_pct = (v * 100.0).round().clamp(0.0, 100.0) as u32;
    (h, s_pct, b_pct)
}

impl RenderOnce for ColorWell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let on_toggle = rc_wrap(self.on_toggle);
        let on_change = rc_wrap(self.on_change);
        let on_highlight = self.on_highlight.map(std::rc::Rc::new);
        let on_hex_input = self.on_hex_input.map(std::rc::Rc::new);
        let on_hex_commit = self.on_hex_commit.map(std::rc::Rc::new);
        let on_tab_change = self.on_tab_change.map(std::rc::Rc::new);
        let hex_draft = self.hex_draft.clone();
        let style = self.style;
        let popover_tab = self.popover_tab;

        // -- Trigger swatch --
        //
        // Compact: 28x14pt pill with trailing chevron (Tahoe default).
        // Expanded: 44x44pt square (pre-Tahoe; retained for backcompat).
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;

        let mut trigger = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .cursor_pointer();

        trigger = match style {
            ColorWellStyle::Compact => trigger
                .w(px(COMPACT_WIDTH))
                .h(px(COMPACT_HEIGHT))
                .rounded(theme.radius_sm)
                .bg(self.color)
                .pr(theme.spacing_xs)
                .justify_end()
                .child(
                    Icon::new(IconName::ChevronDown)
                        .size(px(10.0))
                        .color(crate::foundations::color::text_on_background(self.color)),
                ),
            ColorWellStyle::Expanded => trigger
                .size(px(theme.target_size()))
                .rounded(theme.radius_md)
                .bg(self.color),
        };

        trigger = apply_focus_ring(
            trigger,
            theme,
            self.focused,
            theme.glass.shadows(GlassSize::Small),
        );
        trigger = apply_high_contrast_border(trigger, theme);

        if let Some(handler) = toggle_for_trigger {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space opens the dropdown.
        if let Some(handler) = trigger_key_toggle {
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event) && !is_open {
                    cx.stop_propagation();
                    handler(true, window, cx);
                }
            });
        }

        // -- Container (trigger + optional dropdown) --
        let mut container = div().relative();
        container = container.child(trigger);

        if self.is_open {
            let palette = system_color_palette(theme);
            let current_color = self.color;
            let highlighted_index = self.highlighted_index;
            let palette_len = palette.len();

            // Collect palette colors for keyboard enter selection.
            let palette_colors: Vec<Hsla> = palette.iter().map(|(_, c)| *c).collect();

            // Build a 6x3 grid of swatches inside a glass dropdown.
            let grid_gap = theme.spacing_xs;
            let grid_padding = theme.spacing_sm;

            let mut grid = glass_surface(
                div()
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .flex()
                    .flex_wrap()
                    .gap(grid_gap)
                    .p(grid_padding)
                    .overflow_hidden(),
                theme,
                GlassSize::Small,
            )
            .id(ElementId::from((self.id.clone(), "palette")))
            .focusable();

            // Set a fixed width: 6 swatches + 5 gaps + 2*padding
            let grid_width = (SWATCH_SIZE * GRID_COLUMNS as f32)
                + (f32::from(grid_gap) * (GRID_COLUMNS as f32 - 1.0))
                + (f32::from(grid_padding) * 2.0);
            grid = grid.w(px(grid_width));

            // -- Tab header (RGB / HSB) -------------------------------------
            //
            // Clicking either pill fires `on_tab_change`. The parent owns
            // the selected tab and re-renders with `popover_tab(...)`.
            let tab_row = {
                let tab_handler_rgb = on_tab_change.clone();
                let tab_handler_hsb = on_tab_change.clone();
                let rgb_active = matches!(popover_tab, PopoverTab::Rgb);
                let hsb_active = matches!(popover_tab, PopoverTab::Hsb);

                let mut row = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .pb(theme.spacing_xs)
                    .w_full();

                row = row.child(
                    div()
                        .id(ElementId::from((self.id.clone(), "tab-rgb")))
                        .px(theme.spacing_sm)
                        .py(px(2.0))
                        .rounded(theme.radius_sm)
                        .bg(if rgb_active {
                            theme.selected_bg
                        } else {
                            theme.hover
                        })
                        .text_style(TextStyle::Footnote, theme)
                        .text_color(if rgb_active {
                            theme.text
                        } else {
                            theme.text_muted
                        })
                        .cursor_pointer()
                        .child(SharedString::from("RGB"))
                        .on_click(move |_event, window, cx| {
                            if let Some(h) = &tab_handler_rgb {
                                h(PopoverTab::Rgb, window, cx);
                            }
                        }),
                );

                row = row.child(
                    div()
                        .id(ElementId::from((self.id.clone(), "tab-hsb")))
                        .px(theme.spacing_sm)
                        .py(px(2.0))
                        .rounded(theme.radius_sm)
                        .bg(if hsb_active {
                            theme.selected_bg
                        } else {
                            theme.hover
                        })
                        .text_style(TextStyle::Footnote, theme)
                        .text_color(if hsb_active {
                            theme.text
                        } else {
                            theme.text_muted
                        })
                        .cursor_pointer()
                        .child(SharedString::from("HSB"))
                        .on_click(move |_event, window, cx| {
                            if let Some(h) = &tab_handler_hsb {
                                h(PopoverTab::Hsb, window, cx);
                            }
                        }),
                );

                row
            };
            grid = grid.child(tab_row);

            // Keyboard nav: Arrow keys + Enter + Escape + hex entry.
            let key_on_toggle = on_toggle.clone();
            let key_on_change = on_change.clone();
            let key_on_highlight = on_highlight.clone();
            let key_on_hex_input = on_hex_input.clone();
            let key_on_hex_commit = on_hex_commit.clone();
            let key_hex_draft = hex_draft.clone();
            grid = grid.on_key_down(move |event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        if let Some(ref handler) = key_on_toggle {
                            handler(false, window, cx);
                        }
                    }
                    "enter" => {
                        // Enter commits the hex draft when present, else
                        // applies the currently highlighted swatch.
                        if let (Some(draft), Some(commit)) = (&key_hex_draft, &key_on_hex_commit)
                            && let Some(color) = parse_hex(draft.as_ref())
                        {
                            cx.stop_propagation();
                            commit(color, window, cx);
                            if let Some(ref handler) = key_on_toggle {
                                handler(false, window, cx);
                            }
                            return;
                        }
                        if let Some(idx) = highlighted_index
                            && idx < palette_len
                        {
                            if let Some(ref handler) = key_on_change {
                                handler(palette_colors[idx], window, cx);
                            }
                            if let Some(ref handler) = key_on_toggle {
                                handler(false, window, cx);
                            }
                        }
                    }
                    "backspace" => {
                        // Drop the last char from the hex draft, leaving
                        // arrow/enter behaviour for the grid unchanged.
                        if let (Some(draft), Some(handler)) =
                            (key_hex_draft.clone(), &key_on_hex_input)
                        {
                            cx.stop_propagation();
                            let mut text = draft.to_string();
                            text.pop();
                            handler(SharedString::from(text), window, cx);
                        }
                    }
                    // Left/Right by 1, Up/Down by GRID_COLUMNS (6). Emit
                    // the new index via `on_highlight` so the parent can
                    // track the focused swatch.
                    key @ ("left" | "right" | "up" | "down") => {
                        cx.stop_propagation();
                        if palette_len == 0 {
                            return;
                        }
                        let current = highlighted_index.unwrap_or(0) as isize;
                        let step = match key {
                            "left" => -1,
                            "right" => 1,
                            "up" => -(GRID_COLUMNS as isize),
                            "down" => GRID_COLUMNS as isize,
                            _ => 0,
                        };
                        let next = (current + step).rem_euclid(palette_len as isize) as usize;
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(next), window, cx);
                        }
                    }
                    _ => {
                        // Treat any printable character as hex-entry input
                        // when a draft is active. Non-hex characters are
                        // filtered downstream by `parse_hex` at commit
                        // time; we accept them in the draft so the user
                        // can paste `#FF00AA` and edit mid-string.
                        //
                        // NOTE: Tab-cycling between swatches, the hex
                        // field, and the RGB/HSB numeric inputs requires
                        // multiple independent `FocusHandle`s, which only
                        // stateful (`Entity<T>`) components can own.
                        // Within this `RenderOnce` we expose click-based
                        // selection for tabs and arrow-key navigation for
                        // swatches; callers that need full Tab cycling
                        // should promote the field to a stateful picker.
                        if let (Some(draft), Some(handler)) =
                            (key_hex_draft.clone(), &key_on_hex_input)
                        {
                            let typed =
                                event.keystroke.key_char.as_deref().filter(|s| {
                                    !s.is_empty() && !s.chars().any(|c| c.is_control())
                                });
                            if let Some(text) = typed {
                                cx.stop_propagation();
                                let mut buf = draft.to_string();
                                buf.push_str(text);
                                handler(SharedString::from(buf), window, cx);
                            }
                        }
                    }
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                grid = grid.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            for (idx, (name, swatch_color)) in palette.into_iter().enumerate() {
                let is_selected = hsla_eq(current_color, swatch_color);
                let is_highlighted = highlighted_index == Some(idx);
                let on_change = on_change.clone();
                let on_toggle = on_toggle.clone();

                let mut swatch = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "color-swatch-{}",
                        name
                    ))))
                    .size(px(SWATCH_SIZE))
                    .rounded(theme.radius_md)
                    .bg(swatch_color)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .flex_shrink_0();

                // Highlighted swatch shows a focus ring.
                swatch = apply_focus_ring(swatch, theme, is_highlighted, &[]);

                // Selected swatch shows a contrast-aware Check icon overlay.
                if is_selected {
                    swatch = swatch.child(
                        Icon::new(IconName::Check)
                            .size(px(16.0))
                            .color(crate::foundations::color::text_on_background(swatch_color)),
                    );
                }

                swatch = swatch.on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_change {
                        handler(swatch_color, window, cx);
                    }
                    if let Some(handler) = &on_toggle {
                        handler(false, window, cx);
                    }
                });

                grid = grid.child(swatch);
            }

            // -- RGB / HSB numeric display row ------------------------------
            //
            // Shows current channel values as read-only chips. The parent
            // wires up an external text-field widget if full editing is
            // needed; exposing three independent inputs with live draft
            // state from a `RenderOnce` would require six more callbacks
            // and drafts per tab. The chips reflect the *committed*
            // colour so external changes are visible here.
            let numeric_row = {
                let label_a;
                let label_b;
                let label_c;
                let val_a;
                let val_b;
                let val_c;
                match popover_tab {
                    PopoverTab::Rgb => {
                        let (r, g, b) = hsla_to_rgb_bytes(current_color);
                        label_a = "R";
                        label_b = "G";
                        label_c = "B";
                        val_a = format!("{}", r);
                        val_b = format!("{}", g);
                        val_c = format!("{}", b);
                    }
                    PopoverTab::Hsb => {
                        let (h, s, b) = hsla_to_hsb(current_color);
                        label_a = "H";
                        label_b = "S";
                        label_c = "B";
                        val_a = format!("{}", h);
                        val_b = format!("{}", s);
                        val_c = format!("{}", b);
                    }
                }
                let make_cell = |label: &'static str, value: String| {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(1.0))
                        .flex_1()
                        .child(
                            div()
                                .text_style(TextStyle::Footnote, theme)
                                .text_color(theme.text_muted)
                                .child(SharedString::from(label)),
                        )
                        .child(
                            div()
                                .w_full()
                                .px(theme.spacing_xs)
                                .py(px(2.0))
                                .rounded(theme.radius_sm)
                                .bg(theme.hover)
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child(SharedString::from(value)),
                        )
                };
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .w_full()
                    .pb(theme.spacing_xs)
                    .child(make_cell(label_a, val_a))
                    .child(make_cell(label_b, val_b))
                    .child(make_cell(label_c, val_c))
            };
            grid = grid.child(numeric_row);

            // Hex entry row. Shown when the parent supplies `hex_draft`.
            // The dropdown's on_key_down fires `on_hex_input` on every
            // keystroke and `on_hex_commit` on Enter (if the draft
            // parses). The displayed value is whatever the parent last
            // committed — it stays a pure reflection of parent state.
            if let Some(ref draft) = hex_draft {
                let placeholder = hsla_to_hex(current_color);
                let label = if draft.is_empty() {
                    SharedString::from(placeholder)
                } else {
                    draft.clone()
                };
                let draft_color = if draft.is_empty() {
                    theme.text_muted
                } else {
                    theme.text
                };
                grid = grid.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .px(theme.spacing_sm)
                        .pb(theme.spacing_sm)
                        .child(
                            div()
                                .text_style(TextStyle::Footnote, theme)
                                .text_color(theme.text_muted)
                                .child(SharedString::from("Hex")),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px(theme.spacing_xs)
                                .py(px(2.0))
                                .rounded(theme.radius_md)
                                .bg(theme.hover)
                                .text_style(TextStyle::Body, theme)
                                .text_color(draft_color)
                                .child(label),
                        ),
                );
            }

            // -- Alpha scrubber ---------------------------------------------
            //
            // HIG NSColorPanel exposes a continuous opacity slider. The
            // reusable `Slider` primitive is a stateful `Entity<T>` and
            // cannot be embedded inside this `RenderOnce` (the entity
            // must outlive the render and own a `FocusHandle`). As a
            // substitute we render an 8-tick clickable row at 0.125
            // alpha steps. Each tick commits a new colour with the
            // updated `a` channel; hue/saturation/lightness are
            // preserved. The filled portion of the track visualises the
            // current alpha so external changes are reflected back.
            let alpha_row = {
                let bar_alpha = current_color.a.clamp(0.0, 1.0);
                let bar_bg = theme.border;
                let bar_fg = theme.accent;
                let alpha_handler = on_change.clone();

                let mut track = div()
                    .h(px(6.0))
                    .w_full()
                    .rounded(px(3.0))
                    .bg(bar_bg)
                    .relative();
                track = track.child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(gpui::relative(bar_alpha))
                        .rounded(px(3.0))
                        .bg(bar_fg),
                );

                let mut ticks = div().flex().flex_row().items_center().w_full().gap(px(2.0));
                for i in 0..ALPHA_TICK_COUNT {
                    let frac = i as f32 / (ALPHA_TICK_COUNT - 1) as f32;
                    let active = (bar_alpha - frac).abs() <= 0.5 / (ALPHA_TICK_COUNT as f32 - 1.0);
                    let handler = alpha_handler.clone();
                    let base_color = current_color;
                    ticks = ticks.child(
                        div()
                            .id(ElementId::from((
                                self.id.clone(),
                                SharedString::from(format!("alpha-tick-{}", i)),
                            )))
                            .h(px(12.0))
                            .flex_1()
                            .rounded(px(2.0))
                            .bg(if active { theme.accent } else { theme.hover })
                            .cursor_pointer()
                            .on_click(move |_event, window, cx| {
                                let mut next = base_color;
                                next.a = frac.clamp(0.0, 1.0);
                                if let Some(h) = &handler {
                                    h(next, window, cx);
                                }
                            }),
                    );
                }

                div()
                    .id(ElementId::from((self.id.clone(), "alpha-row")))
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_xs)
                    .px(theme.spacing_sm)
                    .pb(theme.spacing_sm)
                    .child(
                        div()
                            .text_style(TextStyle::Footnote, theme)
                            .text_color(theme.text_muted)
                            .child(SharedString::from(format!(
                                "Opacity {}%",
                                (bar_alpha * 100.0).round() as u32
                            ))),
                    )
                    .child(track)
                    .child(ticks)
            };
            grid = grid.child(alpha_row);

            container = container.child(deferred(grid).with_priority(1));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::hsla;

    use crate::components::selection_and_input::color_well::{
        ALPHA_TICK_COUNT, ColorWell, ColorWellStyle, GRID_COLUMNS, PopoverTab, hsla_eq,
        hsla_to_hsb, hsla_to_rgb_bytes, parse_hex, system_color_palette,
    };
    use crate::foundations::theme::TahoeTheme;

    #[test]
    fn color_well_defaults() {
        let cw = ColorWell::new("test");
        assert!(!cw.is_open);
        assert!(cw.on_change.is_none());
        assert!(cw.on_toggle.is_none());
        assert!(!cw.focused);
        assert!(cw.highlighted_index.is_none());
        // Backcompat: default style must remain Expanded so callers
        // of ColorWell::new(...).color(...) keep their 44pt square.
        assert_eq!(cw.style, ColorWellStyle::Expanded);
        assert_eq!(cw.popover_tab, PopoverTab::Rgb);
    }

    #[test]
    fn color_well_color_builder() {
        let red = hsla(0.0, 1.0, 0.5, 1.0);
        let cw = ColorWell::new("test").color(red);
        assert!(hsla_eq(cw.color, red));
    }

    #[test]
    fn color_well_open_builder() {
        let cw = ColorWell::new("test").open(true);
        assert!(cw.is_open);
    }

    #[test]
    fn color_well_on_change_is_some() {
        let cw = ColorWell::new("test").on_change(|_, _, _| {});
        assert!(cw.on_change.is_some());
    }

    #[test]
    fn color_well_on_toggle_is_some() {
        let cw = ColorWell::new("test").on_toggle(|_, _, _| {});
        assert!(cw.on_toggle.is_some());
    }

    #[test]
    fn system_palette_has_18_colors() {
        let theme = TahoeTheme::dark();
        let palette = system_color_palette(&theme);
        assert_eq!(palette.len(), 18);
        // 18 colors = 6 columns x 3 rows
        assert_eq!(palette.len() % GRID_COLUMNS, 0);
    }

    #[test]
    fn hsla_eq_matches_same_color() {
        let a = hsla(0.5, 0.8, 0.6, 1.0);
        assert!(hsla_eq(a, a));
    }

    #[test]
    fn hsla_eq_rejects_different_color() {
        let a = hsla(0.5, 0.8, 0.6, 1.0);
        let b = hsla(0.0, 1.0, 0.5, 1.0);
        assert!(!hsla_eq(a, b));
    }

    // ── Keyboard nav builder tests ────────────────────────────────────────

    #[test]
    fn color_well_focused_builder() {
        let cw = ColorWell::new("test").focused(true);
        assert!(cw.focused);
    }

    #[test]
    fn color_well_highlighted_index_builder() {
        let cw = ColorWell::new("test").highlighted_index(Some(5));
        assert_eq!(cw.highlighted_index, Some(5));
    }

    #[test]
    fn color_well_highlighted_index_none() {
        let cw = ColorWell::new("test").highlighted_index(None);
        assert_eq!(cw.highlighted_index, None);
    }

    #[test]
    fn color_well_grid_nav_bounds() {
        // Verify GRID_COLUMNS divides palette evenly for arrow navigation.
        let theme = TahoeTheme::dark();
        let palette = system_color_palette(&theme);
        assert_eq!(palette.len() % GRID_COLUMNS, 0);
        // Up/Down navigates by GRID_COLUMNS (6).
        let rows = palette.len() / GRID_COLUMNS;
        assert_eq!(rows, 3);
    }

    // ── ColorWellStyle smoke test ─────────────────────────────────────────

    #[test]
    fn color_well_style_builder() {
        let cw = ColorWell::new("test").style(ColorWellStyle::Compact);
        assert_eq!(cw.style, ColorWellStyle::Compact);
        let cw = ColorWell::new("test").style(ColorWellStyle::Expanded);
        assert_eq!(cw.style, ColorWellStyle::Expanded);
    }

    #[test]
    fn color_well_style_default_is_expanded() {
        // Preserves the pre-Tahoe 44pt behaviour for
        // `ColorWell::new(...).color(...)` callers.
        assert_eq!(ColorWellStyle::default(), ColorWellStyle::Expanded);
    }

    // ── RGB / HSB entry tests ─────────────────────────────────────────────

    #[test]
    fn rgb_bytes_roundtrip_through_hex() {
        // Yellow #FFFF00 must parse to (255, 255, 0).
        let yellow = parse_hex("#FFFF00").expect("valid hex");
        let (r, g, b) = hsla_to_rgb_bytes(yellow);
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn hsb_conversion_pure_red() {
        // Pure red = HSB(0, 100, 100).
        let red = parse_hex("#FF0000").expect("valid hex");
        let (h, s, b) = hsla_to_hsb(red);
        assert_eq!(h, 0);
        assert_eq!(s, 100);
        assert_eq!(b, 100);
    }

    #[test]
    fn hsb_conversion_pure_white() {
        // Pure white = HSB(_, 0, 100). Hue is undefined for achromatic
        // colours, so we only assert on saturation and brightness.
        let white = parse_hex("#FFFFFF").expect("valid hex");
        let (_h, s, b) = hsla_to_hsb(white);
        assert_eq!(s, 0);
        assert_eq!(b, 100);
    }

    // ── parse_hex UTF-8 safety (issue #60) ────────────────────────────────
    // Multi-byte UTF-8 input with a byte length that matches one of the
    // hex arms must return None instead of panicking mid-codepoint.

    #[test]
    fn parse_hex_rejects_multibyte_len_3() {
        assert!(parse_hex("#0é").is_none());
    }

    #[test]
    fn parse_hex_rejects_multibyte_len_4() {
        assert!(parse_hex("#0é0").is_none());
    }

    #[test]
    fn parse_hex_rejects_multibyte_len_6() {
        assert!(parse_hex("#0é000").is_none());
    }

    #[test]
    fn parse_hex_rejects_emoji_len_8() {
        assert!(parse_hex("#🎨0000").is_none());
    }

    #[test]
    fn parse_hex_accepts_lowercase_ascii_unchanged() {
        assert!(parse_hex("#ff00aa").is_some());
    }

    #[test]
    fn popover_tab_builder() {
        let cw = ColorWell::new("test").popover_tab(PopoverTab::Hsb);
        assert_eq!(cw.popover_tab, PopoverTab::Hsb);
    }

    #[test]
    fn on_tab_change_is_some() {
        let cw = ColorWell::new("test").on_tab_change(|_, _, _| {});
        assert!(cw.on_tab_change.is_some());
    }

    // ── Alpha scrubber tests ──────────────────────────────────────────────

    #[test]
    fn alpha_ticks_count_is_eight() {
        // Compromise: `Slider` is a stateful `Entity<T>` and cannot be
        // embedded inside `RenderOnce`, so we expose 8 discrete ticks
        // at 0.125 alpha increments instead of a continuous slider.
        assert_eq!(ALPHA_TICK_COUNT, 8);
    }

    #[test]
    fn alpha_tick_fractions_cover_full_range() {
        // The first tick maps to alpha 0.0 and the last tick maps to 1.0.
        let first = 0_f32 / (ALPHA_TICK_COUNT as f32 - 1.0);
        let last = (ALPHA_TICK_COUNT as f32 - 1.0) / (ALPHA_TICK_COUNT as f32 - 1.0);
        assert!((first - 0.0).abs() < f32::EPSILON);
        assert!((last - 1.0).abs() < f32::EPSILON);
    }
}
