//! HIG TimePicker — hour/minute selector with scrollable columns.
//!
//! A stateless `RenderOnce` component that renders a trigger button and,
//! when open, an absolute-positioned dropdown with hour and minute columns.
//! All state is owned by the parent.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, FontWeight, KeyDownEvent, MouseDownEvent, SharedString, Window,
    deferred, div, px,
};

use crate::callback_types::{OnTimeChange, OnToggle, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{apply_standard_control_styling, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

// ─── Formatting helpers ─────────────────────────────────────────────────────

/// Format a time value as "HH:MM" (24-hour).
fn format_24h(hour: u8, minute: u8) -> String {
    format!("{:02}:{:02}", hour, minute)
}

/// Format a time value as "h:MM AM/PM" (12-hour).
fn format_12h(hour: u8, minute: u8) -> String {
    let period = if hour < 12 { "AM" } else { "PM" };
    let display_hour = match hour {
        0 => 12,
        1..=12 => hour,
        _ => hour - 12,
    };
    format!("{}:{:02} {}", display_hour, minute, period)
}

/// Convert a 12-hour value + AM/PM flag back to 24-hour.
fn to_24h(hour_12: u8, is_pm: bool) -> u8 {
    debug_assert!(
        (1..=12).contains(&hour_12),
        "hour_12 must be 1..=12, got {hour_12}"
    );
    match (hour_12, is_pm) {
        (12, false) => 0,
        (12, true) => 12,
        (h, false) => h,
        (h, true) => h + 12,
    }
}

/// Increment (or decrement) a (hour, minute) pair by `delta_minutes`, wrapping
/// across 00:00 / 23:59. Used by the `StepperField` style's ± buttons.
fn shift_minutes(hour: u8, minute: u8, delta_minutes: i32) -> (u8, u8) {
    let total = (hour as i32) * 60 + (minute as i32) + delta_minutes;
    let total = total.rem_euclid(24 * 60);
    ((total / 60) as u8, (total % 60) as u8)
}

/// Visual presentation style per HIG.
///
/// Marked `#[non_exhaustive]` so additional styles can be added later
/// without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum TimePickerStyle {
    /// Trigger button + scroll-column dropdown. Matches the original
    /// macOS textual style. Default.
    #[default]
    Compact,
    /// Plain HH:MM text field. The parent owns parse-on-blur; no
    /// dropdown is rendered.
    Field,
    /// Three-column wheel (hour / minute / AM-PM) rendered inline.
    /// No trigger, always visible.
    Wheel,
    /// Text field paired with a ± stepper that increments or decrements
    /// by `minute_granularity` minutes, wrapping across midnight.
    StepperField,
}

// ─── TimePicker ─────────────────────────────────────────────────────────────

/// HIG time picker with hour/minute column dropdown.
///
/// Stateless `RenderOnce` — the parent owns `hour`, `minute`, `is_open`,
/// and `use_24h`, and provides callbacks.
///
/// # Example
///
/// ```ignore
/// TimePicker::new("my-time")
///     .hour(14)
///     .minute(30)
///     .open(true)
///     .use_24h(false)
///     .on_change(|h, m, _win, _cx| { /* update state */ })
///     .on_toggle(|open, _win, _cx| { /* toggle dropdown */ })
/// ```
#[derive(IntoElement)]
#[allow(clippy::type_complexity)]
pub struct TimePicker {
    id: ElementId,
    hour: u8,
    minute: u8,
    is_open: bool,
    use_24h: bool,
    on_change: OnTimeChange,
    on_toggle: OnToggle,
    /// Whether this time picker trigger is keyboard-focused.
    focused: bool,
    /// Optional focus handle; when present, drives the focus ring
    /// reactively from GPUI's focus graph (overrides `focused`).
    focus_handle: Option<FocusHandle>,
    /// Which column is keyboard-focused: 0 = hour, 1 = minute, 2 = AM/PM.
    highlighted_column: u8,
    /// Which row within the highlighted column is highlighted.
    highlighted_row: Option<usize>,
    /// Minute step granularity. Default 5 matches the traditional wheel
    /// style; set to 1 for field-style arbitrary minute entry. Values
    /// outside 1..=30 are clamped.
    minute_granularity: u8,
    /// Fired when arrow keys move the highlight. Receives
    /// `(column, row)` so the parent can update state.
    on_highlight: Option<Box<dyn Fn(u8, usize, &mut Window, &mut App) + 'static>>,
    /// HIG presentation style. Defaults to [`TimePickerStyle::Compact`].
    style: TimePickerStyle,
}

impl TimePicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            hour: 0,
            minute: 0,
            is_open: false,
            use_24h: true,
            on_change: None,
            on_toggle: None,
            focused: false,
            focus_handle: None,
            highlighted_column: 0,
            highlighted_row: None,
            minute_granularity: 5,
            on_highlight: None,
            style: TimePickerStyle::Compact,
        }
    }

    /// Select the HIG presentation style. Defaults to
    /// [`TimePickerStyle::Compact`].
    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the minute step granularity. `1` enables every-minute entry,
    /// `5` matches the default wheel style. Clamped to 1..=30.
    pub fn minute_granularity(mut self, step: u8) -> Self {
        self.minute_granularity = step.clamp(1, 30);
        self
    }

    /// Fires when arrow keys change the highlighted (column, row).
    pub fn on_highlight(
        mut self,
        handler: impl Fn(u8, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }

    pub fn hour(mut self, hour: u8) -> Self {
        self.hour = hour.min(23);
        self
    }

    pub fn minute(mut self, minute: u8) -> Self {
        self.minute = minute.min(59);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn use_24h(mut self, use_24h: bool) -> Self {
        self.use_24h = use_24h;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(u8, u8, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Marks this time picker as keyboard-focused, showing a visible focus ring.
    ///
    /// Ignored when a [`focus_handle`](Self::focus_handle) is supplied — the
    /// handle's reactive state takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the time picker participates in GPUI's
    /// focus graph. When present, the focus ring is driven by
    /// `handle.is_focused(window)` and the Compact trigger / Wheel wrapper
    /// threads `track_focus`.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Sets the keyboard-focused column: 0 = hour, 1 = minute, 2 = AM/PM.
    pub fn highlighted_column(mut self, col: u8) -> Self {
        self.highlighted_column = col;
        self
    }

    /// Sets the keyboard-highlighted row within the current column.
    pub fn highlighted_row(mut self, row: Option<usize>) -> Self {
        self.highlighted_row = row;
        self
    }
}

/// Returns the number of items in a given column.
/// Column 0 = hours (24 or 12), column 1 = minutes (12 five-min increments),
/// column 2 = AM/PM (2 items, only in 12h mode).
#[cfg(test)]
fn column_item_count(column: u8, use_24h: bool) -> usize {
    match column {
        0 => {
            if use_24h {
                24
            } else {
                12
            }
        }
        1 => 12,
        2 => 2,
        _ => 0,
    }
}

/// Returns the default row for a given column based on the current selection.
#[cfg(test)]
fn default_row_for_column(column: u8, hour: u8, minute: u8, use_24h: bool) -> usize {
    match column {
        0 => {
            if use_24h {
                hour as usize
            } else {
                let h12 = match hour {
                    0 => 12,
                    1..=12 => hour,
                    _ => hour - 12,
                };
                // 12h column is 1..=12, index 0-based
                (h12 as usize).saturating_sub(1)
            }
        }
        1 => (minute / 5).min(11) as usize,
        2 => usize::from(hour >= 12),
        _ => 0,
    }
}

/// Resolves the hour/minute from the highlighted column + row, returning (hour_24, minute).
#[cfg(test)]
fn resolve_selection(
    column: u8,
    row: usize,
    current_hour: u8,
    current_minute: u8,
    use_24h: bool,
) -> (u8, u8) {
    // Default 5-minute granularity — callers that don't thread
    // `minute_granularity` through get the 5-minute snap.
    resolve_selection_with_granularity(column, row, current_hour, current_minute, use_24h, 5)
}

/// Resolves the hour/minute from the highlighted column + row using a
/// configurable minute granularity.
fn resolve_selection_with_granularity(
    column: u8,
    row: usize,
    current_hour: u8,
    current_minute: u8,
    use_24h: bool,
    minute_granularity: u8,
) -> (u8, u8) {
    let is_pm = current_hour >= 12;
    let current_display_hour = if use_24h {
        current_hour
    } else {
        match current_hour {
            0 => 12,
            1..=12 => current_hour,
            _ => current_hour - 12,
        }
    };

    match column {
        0 => {
            // Hour column
            let hour_range: Vec<u8> = if use_24h {
                (0..24).collect()
            } else {
                (1..=12).collect()
            };
            let h = hour_range.get(row).copied().unwrap_or(current_display_hour);
            let hour_24 = if use_24h { h } else { to_24h(h, is_pm) };
            (hour_24, current_minute)
        }
        1 => {
            // Minute column — row scales by granularity so every-minute
            // entry works alongside the 5-minute default.
            let step = minute_granularity.clamp(1, 30);
            let m = (row as u32) * step as u32;
            (current_hour, m.min(59) as u8)
        }
        2 => {
            // AM/PM column
            let pm_flag = row == 1;
            let hour_24 = to_24h(current_display_hour, pm_flag);
            (hour_24, current_minute)
        }
        _ => (current_hour, current_minute),
    }
}

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // FocusHandle (when supplied) drives the ring reactively; falls
        // back to the manual `focused: bool` flag otherwise.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        let style = self.style;

        // ── Trigger label ──────────────────────────────────────────────────
        let trigger_label: SharedString = SharedString::from(if self.use_24h {
            format_24h(self.hour, self.minute)
        } else {
            format_12h(self.hour, self.minute)
        });

        let on_toggle = rc_wrap(self.on_toggle);
        let on_change = rc_wrap(self.on_change);
        let on_highlight = self.on_highlight.map(std::rc::Rc::new);

        // ── Wheel variant: always-visible inline columns ───────────────────
        // Short-circuit before the Compact trigger is built — Wheel owns
        // its own focusable wrapper (the same element that carries
        // `on_key_down`), so the Compact trigger's focus/key wiring would
        // be dead weight here.
        if matches!(style, TimePickerStyle::Wheel) {
            return build_time_wheel(
                self.id.clone(),
                self.focus_handle.as_ref(),
                self.hour,
                self.minute,
                self.use_24h,
                self.highlighted_column,
                self.highlighted_row,
                self.minute_granularity,
                theme,
                focused,
                on_change,
                on_highlight,
            );
        }

        // ── Trigger button ─────────────────────────────────────────────────
        // Icon shown only for `Compact`; `Field` / `StepperField` look
        // like plain text fields per HIG.
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;
        let show_trigger_icon = matches!(style, TimePickerStyle::Compact);

        let mut trigger_content = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(trigger_label),
            );
        if show_trigger_icon {
            trigger_content = trigger_content.child(
                Icon::new(IconName::Clock)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            );
        }

        let mut trigger = div()
            .id(self.id.clone())
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .px(theme.spacing_md)
            .cursor_pointer()
            .focusable();
        if let Some(handle) = self.focus_handle.as_ref() {
            trigger = trigger.track_focus(handle);
        }

        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);

        trigger = trigger
            .hover(|style| style.cursor_pointer())
            .child(trigger_content);

        // Click-to-toggle applies only to `Compact`. The text-field styles
        // have their own commit semantics (parse-on-blur handled by the
        // parent).
        if matches!(style, TimePickerStyle::Compact)
            && let Some(handler) = toggle_for_trigger
        {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space/Down opens the dropdown
        // in `Compact`. In `Field` / `StepperField`, Enter fires
        // `on_toggle(false)` as a "commit / blur" signal for the parent.
        if let Some(handler) = trigger_key_toggle {
            let style_captured = style;
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let is_activation = crate::foundations::keyboard::is_activation_key(event)
                    || event.keystroke.key.as_str() == "down";
                match style_captured {
                    TimePickerStyle::Compact => {
                        if is_activation && !is_open {
                            cx.stop_propagation();
                            handler(true, window, cx);
                        }
                    }
                    TimePickerStyle::Field | TimePickerStyle::StepperField => {
                        if crate::foundations::keyboard::is_activation_key(event) {
                            cx.stop_propagation();
                            handler(false, window, cx);
                        }
                    }
                    TimePickerStyle::Wheel => {}
                }
            });
        }

        // ── StepperField variant: trigger + ± stepper buttons ──────────────
        let container_root = if matches!(style, TimePickerStyle::StepperField) {
            let step = self.minute_granularity.max(1) as i32;
            let hour_now = self.hour;
            let minute_now = self.minute;
            let on_change_dec = on_change.clone();
            let on_change_inc = on_change.clone();
            let dec_btn = tp_stepper_button(
                theme,
                ElementId::from((self.id.clone(), "tp-step-dec")),
                IconName::Minus,
                move |_event, window, cx| {
                    if let Some(handler) = on_change_dec.as_ref() {
                        let (h, m) = shift_minutes(hour_now, minute_now, -step);
                        handler(h, m, window, cx);
                    }
                },
            );
            let inc_btn = tp_stepper_button(
                theme,
                ElementId::from((self.id.clone(), "tp-step-inc")),
                IconName::Plus,
                move |_event, window, cx| {
                    if let Some(handler) = on_change_inc.as_ref() {
                        let (h, m) = shift_minutes(hour_now, minute_now, step);
                        handler(h, m, window, cx);
                    }
                },
            );
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_sm)
                .child(div().flex_grow().child(trigger))
                .child(dec_btn)
                .child(inc_btn)
        } else {
            div().relative().child(trigger)
        };

        // ── Container ──────────────────────────────────────────────────────
        let mut container = container_root;

        // Only `Compact` shows the popover; `Field` / `StepperField` are
        // text-entry surfaces and ignore `is_open`.
        let show_popover = matches!(style, TimePickerStyle::Compact) && self.is_open;
        if show_popover {
            let current_hour = self.hour;
            let current_minute = self.minute;
            let use_24h = self.use_24h;
            let is_pm = current_hour >= 12;
            let accent = theme.accent;
            let highlighted_column = self.highlighted_column;
            let highlighted_row = self.highlighted_row;
            let hover_bg = theme.hover_bg();

            // ── Hour column ────────────────────────────────────────────────
            let hour_range: Vec<u8> = if use_24h {
                (0..24).collect()
            } else {
                (1..=12).collect()
            };

            let current_display_hour = if use_24h {
                current_hour
            } else {
                match current_hour {
                    0 => 12,
                    1..=12 => current_hour,
                    _ => current_hour - 12,
                }
            };

            let mut hour_col = div()
                .id("tp-hour-col")
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .max_h(px(DROPDOWN_MAX_HEIGHT))
                .min_w(px(60.0));

            for (idx, h) in hour_range.iter().enumerate() {
                let h = *h;
                let is_selected = h == current_display_hour;
                let is_highlighted = highlighted_column == 0 && highlighted_row == Some(idx);
                let on_change_h = on_change.clone();

                let cell_id = ElementId::from(SharedString::from(format!("tp-h-{}", h)));

                let mut cell = div()
                    .id(cell_id)
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_style(TextStyle::Body, theme)
                    .cursor_pointer();

                if is_selected {
                    cell = cell
                        .bg(accent)
                        .text_color(theme.text_on_accent)
                        .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
                } else if is_highlighted {
                    cell = cell.bg(hover_bg).text_color(theme.text);
                } else {
                    cell = cell
                        .text_color(theme.text)
                        .hover(|style| style.bg(theme.hover));
                }

                cell = cell.child(SharedString::from(format!("{:02}", h)));

                // Fix: preserve current_minute instead of snapping to 5-min increments.
                let minute_val = current_minute;
                let is_pm_val = is_pm;
                cell = cell.on_click(move |_event, window, cx| {
                    let hour_24 = if use_24h { h } else { to_24h(h, is_pm_val) };
                    if let Some(handler) = &on_change_h {
                        handler(hour_24, minute_val, window, cx);
                    }
                });

                hour_col = hour_col.child(cell);
            }

            // ── Minute column (5-min increments) ───────────────────────────
            let mut minute_col = div()
                .id("tp-minute-col")
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .max_h(px(DROPDOWN_MAX_HEIGHT))
                .min_w(px(60.0));

            let granularity = self.minute_granularity.max(1);
            let step_count = (60 / granularity as u16).max(1) as u8;
            let minute_values: Vec<u8> = (0..step_count).map(|i| i * granularity).collect();

            let snapped_minute = (current_minute / granularity) * granularity;
            for (idx, m) in minute_values.iter().enumerate() {
                let m = *m;
                let is_selected = m == snapped_minute;
                let is_highlighted = highlighted_column == 1 && highlighted_row == Some(idx);
                let on_change_m = on_change.clone();
                let on_toggle_m = on_toggle.clone();

                let cell_id = ElementId::from(SharedString::from(format!("tp-m-{}", m)));

                let mut cell = div()
                    .id(cell_id)
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_style(TextStyle::Body, theme)
                    .cursor_pointer();

                if is_selected {
                    cell = cell
                        .bg(accent)
                        .text_color(theme.text_on_accent)
                        .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
                } else if is_highlighted {
                    cell = cell.bg(hover_bg).text_color(theme.text);
                } else {
                    cell = cell
                        .text_color(theme.text)
                        .hover(|style| style.bg(theme.hover));
                }

                cell = cell.child(SharedString::from(format!("{:02}", m)));

                let hour_val = current_hour;
                cell = cell.on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_change_m {
                        handler(hour_val, m, window, cx);
                    }
                    if let Some(handler) = &on_toggle_m {
                        handler(false, window, cx);
                    }
                });

                minute_col = minute_col.child(cell);
            }

            // ── AM/PM column (12-hour mode only) ───────────────────────────
            let mut columns = div()
                .flex()
                .flex_row()
                .gap(px(1.0))
                .child(hour_col)
                .child(minute_col);

            if !use_24h {
                let mut ampm_col = div().flex().flex_col().min_w(px(52.0));

                for (idx, (label, pm_flag)) in [("AM", false), ("PM", true)].iter().enumerate() {
                    let pm_flag = *pm_flag;
                    let is_selected = is_pm == pm_flag;
                    let is_highlighted = highlighted_column == 2 && highlighted_row == Some(idx);
                    let on_change_ap = on_change.clone();

                    let cell_id = ElementId::from(SharedString::from(format!("tp-{}", label)));

                    let mut cell = div()
                        .id(cell_id)
                        .min_h(px(theme.target_size()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_style(TextStyle::Subheadline, theme)
                        .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                        .cursor_pointer();

                    if is_selected {
                        cell = cell.bg(accent).text_color(theme.text_on_accent);
                    } else if is_highlighted {
                        cell = cell.bg(hover_bg).text_color(theme.text);
                    } else {
                        cell = cell
                            .text_color(theme.text)
                            .hover(|style| style.bg(theme.hover));
                    }

                    cell = cell.child(SharedString::from(*label));

                    // Fix: preserve current_minute instead of snapping.
                    let display_hour = current_display_hour;
                    let minute_val = current_minute;
                    cell = cell.on_click(move |_event, window, cx| {
                        let hour_24 = to_24h(display_hour, pm_flag);
                        if let Some(handler) = &on_change_ap {
                            handler(hour_24, minute_val, window, cx);
                        }
                    });

                    ampm_col = ampm_col.child(cell);
                }

                columns = columns.child(ampm_col);
            }

            // ── Dropdown ───────────────────────────────────────────────────
            let dropdown_content = div().flex().flex_col().p(theme.spacing_sm).child(columns);

            let mut dropdown = glass_surface(
                div()
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .w(px(if use_24h { 136.0 } else { 188.0 }))
                    .overflow_hidden(),
                theme,
                GlassSize::Medium,
            )
            .id(ElementId::from((self.id.clone(), "dropdown")))
            .focusable();

            // Keyboard nav: Up/Down/Left/Right/Enter/Escape.
            let key_on_toggle = on_toggle.clone();
            let key_on_change = on_change.clone();
            let key_on_highlight = on_highlight.clone();
            let has_ampm = !use_24h;
            let hour_rows = if use_24h { 24 } else { 12 };
            let minute_rows = step_count as usize;
            dropdown = dropdown.on_key_down(move |event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    _ if crate::foundations::keyboard::is_escape_key(event) => {
                        if let Some(ref handler) = key_on_toggle {
                            handler(false, window, cx);
                        }
                    }
                    "enter" => {
                        if let Some(row) = highlighted_row {
                            let (h, m) = resolve_selection_with_granularity(
                                highlighted_column,
                                row,
                                current_hour,
                                current_minute,
                                use_24h,
                                granularity,
                            );
                            if let Some(ref handler) = key_on_change {
                                handler(h, m, window, cx);
                            }
                            if let Some(ref handler) = key_on_toggle {
                                handler(false, window, cx);
                            }
                        }
                    }
                    // Left/Right switch columns; Up/Down move rows within
                    // the current column. Column count is 2 in 24h mode,
                    // 3 in 12h mode (hour/minute/AM-PM).
                    key @ ("up" | "down" | "left" | "right") => {
                        cx.stop_propagation();
                        let max_col: u8 = if has_ampm { 2 } else { 1 };
                        let col_rows = match highlighted_column {
                            0 => hour_rows,
                            1 => minute_rows,
                            2 => 2,
                            _ => 1,
                        };
                        let current_row = highlighted_row.unwrap_or(0);
                        let (new_col, new_row) = match key {
                            "left" => {
                                let c = if highlighted_column == 0 {
                                    max_col
                                } else {
                                    highlighted_column - 1
                                };
                                (c, 0usize)
                            }
                            "right" => {
                                let c = if highlighted_column >= max_col {
                                    0
                                } else {
                                    highlighted_column + 1
                                };
                                (c, 0usize)
                            }
                            "up" => {
                                let r = if current_row == 0 {
                                    col_rows.saturating_sub(1)
                                } else {
                                    current_row - 1
                                };
                                (highlighted_column, r)
                            }
                            "down" => {
                                let r = if current_row + 1 >= col_rows {
                                    0
                                } else {
                                    current_row + 1
                                };
                                (highlighted_column, r)
                            }
                            _ => (highlighted_column, current_row),
                        };
                        if let Some(ref handler) = key_on_highlight {
                            handler(new_col, new_row, window, cx);
                        }
                    }
                    _ => {}
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                dropdown =
                    dropdown.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                        handler(false, window, cx);
                    });
            }

            dropdown = dropdown.child(dropdown_content);

            container = container.child(deferred(dropdown).with_priority(1));
        }

        container.into_any_element()
    }
}

// ─── Style-specific helpers ─────────────────────────────────────────────────

/// Build a compact ± stepper button used by the `StepperField` style.
fn tp_stepper_button(
    theme: &crate::foundations::theme::TahoeTheme,
    id: ElementId,
    icon: IconName,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .min_w(px(theme.target_size()))
        .min_h(px(theme.target_size()))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .rounded(theme.radius_md)
        .hover(|style| style.bg(theme.hover))
        .child(
            Icon::new(icon)
                .size(theme.icon_size_inline)
                .color(theme.text),
        )
        .on_click(on_click)
}

/// Build the always-visible three-column time wheel used by the
/// `Wheel` style (hour / minute / AM-PM). Keyboard nav and the focus
/// ring live on the wrapper so a11y stays consistent with `Compact` —
/// `.focusable()` + `track_focus` are applied to the same element that
/// owns `on_key_down` so key events reach their handler (events fire on
/// the focused element and bubble UP; they never bubble DOWN into
/// children).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn build_time_wheel(
    id: ElementId,
    focus_handle: Option<&FocusHandle>,
    current_hour: u8,
    current_minute: u8,
    use_24h: bool,
    highlighted_column: u8,
    highlighted_row: Option<usize>,
    minute_granularity: u8,
    theme: &crate::foundations::theme::TahoeTheme,
    focused: bool,
    on_change: Option<std::rc::Rc<Box<dyn Fn(u8, u8, &mut Window, &mut App) + 'static>>>,
    on_highlight: Option<std::rc::Rc<Box<dyn Fn(u8, usize, &mut Window, &mut App) + 'static>>>,
) -> gpui::AnyElement {
    let is_pm = current_hour >= 12;
    let accent = theme.accent;
    let hover_bg = theme.hover_bg();

    let hour_range: Vec<u8> = if use_24h {
        (0..24).collect()
    } else {
        (1..=12).collect()
    };
    let current_display_hour = if use_24h {
        current_hour
    } else {
        match current_hour {
            0 => 12,
            1..=12 => current_hour,
            _ => current_hour - 12,
        }
    };

    // Hour column
    let mut hour_col = div()
        .id("tpw-hour-col")
        .flex()
        .flex_col()
        .overflow_y_scroll()
        .max_h(px(DROPDOWN_MAX_HEIGHT))
        .min_w(px(60.0));
    for (idx, h) in hour_range.iter().enumerate() {
        let h = *h;
        let is_selected = h == current_display_hour;
        let is_highlighted = highlighted_column == 0 && highlighted_row == Some(idx);
        let on_change_h = on_change.clone();
        let cell_id = ElementId::from(SharedString::from(format!("tpw-h-{}", h)));
        let mut cell = div()
            .id(cell_id)
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .justify_center()
            .text_style(TextStyle::Body, theme)
            .cursor_pointer();
        if is_selected {
            cell = cell
                .bg(accent)
                .text_color(theme.text_on_accent)
                .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
        } else if is_highlighted {
            cell = cell.bg(hover_bg).text_color(theme.text);
        } else {
            cell = cell
                .text_color(theme.text)
                .hover(|style| style.bg(theme.hover));
        }
        cell = cell.child(SharedString::from(format!("{:02}", h)));
        let minute_val = current_minute;
        let is_pm_val = is_pm;
        cell = cell.on_click(move |_event, window, cx| {
            let hour_24 = if use_24h { h } else { to_24h(h, is_pm_val) };
            if let Some(handler) = &on_change_h {
                handler(hour_24, minute_val, window, cx);
            }
        });
        hour_col = hour_col.child(cell);
    }

    // Minute column
    let granularity = minute_granularity.max(1);
    let step_count = (60 / granularity as u16).max(1) as u8;
    let minute_values: Vec<u8> = (0..step_count).map(|i| i * granularity).collect();
    let snapped_minute = (current_minute / granularity) * granularity;

    let mut minute_col = div()
        .id("tpw-minute-col")
        .flex()
        .flex_col()
        .overflow_y_scroll()
        .max_h(px(DROPDOWN_MAX_HEIGHT))
        .min_w(px(60.0));
    for (idx, m) in minute_values.iter().enumerate() {
        let m = *m;
        let is_selected = m == snapped_minute;
        let is_highlighted = highlighted_column == 1 && highlighted_row == Some(idx);
        let on_change_m = on_change.clone();
        let cell_id = ElementId::from(SharedString::from(format!("tpw-m-{}", m)));
        let mut cell = div()
            .id(cell_id)
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .justify_center()
            .text_style(TextStyle::Body, theme)
            .cursor_pointer();
        if is_selected {
            cell = cell
                .bg(accent)
                .text_color(theme.text_on_accent)
                .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
        } else if is_highlighted {
            cell = cell.bg(hover_bg).text_color(theme.text);
        } else {
            cell = cell
                .text_color(theme.text)
                .hover(|style| style.bg(theme.hover));
        }
        cell = cell.child(SharedString::from(format!("{:02}", m)));
        let hour_val = current_hour;
        cell = cell.on_click(move |_event, window, cx| {
            if let Some(handler) = &on_change_m {
                handler(hour_val, m, window, cx);
            }
        });
        minute_col = minute_col.child(cell);
    }

    let mut columns = div()
        .flex()
        .flex_row()
        .gap(px(1.0))
        .child(hour_col)
        .child(minute_col);

    // AM/PM column in 12-hour mode
    if !use_24h {
        let mut ampm_col = div().flex().flex_col().min_w(px(52.0));
        for (idx, (label, pm_flag)) in [("AM", false), ("PM", true)].iter().enumerate() {
            let pm_flag = *pm_flag;
            let is_selected = is_pm == pm_flag;
            let is_highlighted = highlighted_column == 2 && highlighted_row == Some(idx);
            let on_change_ap = on_change.clone();
            let cell_id = ElementId::from(SharedString::from(format!("tpw-{}", label)));
            let mut cell = div()
                .id(cell_id)
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .justify_center()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .cursor_pointer();
            if is_selected {
                cell = cell.bg(accent).text_color(theme.text_on_accent);
            } else if is_highlighted {
                cell = cell.bg(hover_bg).text_color(theme.text);
            } else {
                cell = cell
                    .text_color(theme.text)
                    .hover(|style| style.bg(theme.hover));
            }
            cell = cell.child(SharedString::from(*label));
            let display_hour = current_display_hour;
            let minute_val = current_minute;
            cell = cell.on_click(move |_event, window, cx| {
                let hour_24 = to_24h(display_hour, pm_flag);
                if let Some(handler) = &on_change_ap {
                    handler(hour_24, minute_val, window, cx);
                }
            });
            ampm_col = ampm_col.child(cell);
        }
        columns = columns.child(ampm_col);
    }

    let mut wrapper = div()
        .id(id)
        .flex()
        .flex_col()
        .p(theme.spacing_sm)
        .child(columns)
        .focusable();
    if let Some(handle) = focus_handle {
        wrapper = wrapper.track_focus(handle);
    }
    wrapper = apply_standard_control_styling(wrapper, theme, GlassSize::Small, focused);

    let has_ampm = !use_24h;
    let hour_rows = if use_24h { 24 } else { 12 };
    let minute_rows = step_count as usize;
    let key_on_change = on_change.clone();
    let key_on_highlight = on_highlight.clone();
    wrapper = wrapper.on_key_down(move |event: &KeyDownEvent, window, cx| {
        match event.keystroke.key.as_str() {
            "enter" => {
                if let Some(row) = highlighted_row {
                    let (h, m) = resolve_selection_with_granularity(
                        highlighted_column,
                        row,
                        current_hour,
                        current_minute,
                        use_24h,
                        granularity,
                    );
                    if let Some(ref handler) = key_on_change {
                        handler(h, m, window, cx);
                    }
                }
            }
            key @ ("up" | "down" | "left" | "right") => {
                cx.stop_propagation();
                let max_col: u8 = if has_ampm { 2 } else { 1 };
                let col_rows = match highlighted_column {
                    0 => hour_rows,
                    1 => minute_rows,
                    2 => 2,
                    _ => 1,
                };
                let current_row = highlighted_row.unwrap_or(0);
                let (new_col, new_row) = match key {
                    "left" => {
                        let c = if highlighted_column == 0 {
                            max_col
                        } else {
                            highlighted_column - 1
                        };
                        (c, 0usize)
                    }
                    "right" => {
                        let c = if highlighted_column >= max_col {
                            0
                        } else {
                            highlighted_column + 1
                        };
                        (c, 0usize)
                    }
                    "up" => {
                        let r = if current_row == 0 {
                            col_rows.saturating_sub(1)
                        } else {
                            current_row - 1
                        };
                        (highlighted_column, r)
                    }
                    "down" => {
                        let r = if current_row + 1 >= col_rows {
                            0
                        } else {
                            current_row + 1
                        };
                        (highlighted_column, r)
                    }
                    _ => (highlighted_column, current_row),
                };
                if let Some(ref handler) = key_on_highlight {
                    handler(new_col, new_row, window, cx);
                }
            }
            _ => {}
        }
    });

    wrapper.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        TimePicker, TimePickerStyle, column_item_count, default_row_for_column, format_12h,
        format_24h, resolve_selection, shift_minutes, to_24h,
    };
    use core::prelude::v1::test;

    // ── Formatting ─────────────────────────────────────────────────────────

    #[test]
    fn format_24h_midnight() {
        assert_eq!(format_24h(0, 0), "00:00");
    }

    #[test]
    fn format_24h_afternoon() {
        assert_eq!(format_24h(14, 30), "14:30");
    }

    #[test]
    fn format_12h_midnight() {
        assert_eq!(format_12h(0, 0), "12:00 AM");
    }

    #[test]
    fn format_12h_noon() {
        assert_eq!(format_12h(12, 0), "12:00 PM");
    }

    #[test]
    fn format_12h_afternoon() {
        assert_eq!(format_12h(14, 30), "2:30 PM");
    }

    #[test]
    fn format_12h_morning() {
        assert_eq!(format_12h(9, 5), "9:05 AM");
    }

    // ── 12h → 24h conversion ──────────────────────────────────────────────

    #[test]
    fn to_24h_midnight() {
        assert_eq!(to_24h(12, false), 0);
    }

    #[test]
    fn to_24h_noon() {
        assert_eq!(to_24h(12, true), 12);
    }

    #[test]
    fn to_24h_morning() {
        assert_eq!(to_24h(9, false), 9);
    }

    #[test]
    fn to_24h_evening() {
        assert_eq!(to_24h(9, true), 21);
    }

    // ── TimePicker builder ─────────────────────────────────────────────────

    #[test]
    fn timepicker_defaults() {
        let tp = TimePicker::new("test");
        assert_eq!(tp.hour, 0);
        assert_eq!(tp.minute, 0);
        assert!(!tp.is_open);
        assert!(tp.use_24h);
        assert!(tp.on_change.is_none());
        assert!(tp.on_toggle.is_none());
        assert!(!tp.focused);
        assert_eq!(tp.highlighted_column, 0);
        assert!(tp.highlighted_row.is_none());
    }

    #[test]
    fn timepicker_hour_builder() {
        let tp = TimePicker::new("test").hour(14);
        assert_eq!(tp.hour, 14);
    }

    #[test]
    fn timepicker_minute_builder() {
        let tp = TimePicker::new("test").minute(45);
        assert_eq!(tp.minute, 45);
    }

    #[test]
    fn timepicker_use_24h_builder() {
        let tp = TimePicker::new("test").use_24h(false);
        assert!(!tp.use_24h);
    }

    #[test]
    fn timepicker_open_builder() {
        let tp = TimePicker::new("test").open(true);
        assert!(tp.is_open);
    }

    #[test]
    fn timepicker_on_change_is_some() {
        let tp = TimePicker::new("test").on_change(|_, _, _, _| {});
        assert!(tp.on_change.is_some());
    }

    #[test]
    fn timepicker_on_toggle_is_some() {
        let tp = TimePicker::new("test").on_toggle(|_, _, _| {});
        assert!(tp.on_toggle.is_some());
    }

    // ── format_12h / to_24h boundary tests ─────────────────────────────────

    #[test]
    fn format_12h_one_am() {
        assert_eq!(format_12h(1, 0), "1:00 AM");
    }

    #[test]
    fn format_12h_eleven_pm() {
        assert_eq!(format_12h(23, 59), "11:59 PM");
    }

    #[test]
    fn to_24h_one_am() {
        assert_eq!(to_24h(1, false), 1);
    }

    // ── Keyboard nav builder tests ────────────────────────────────────────

    #[test]
    fn timepicker_focused_builder() {
        let tp = TimePicker::new("test").focused(true);
        assert!(tp.focused);
    }

    #[test]
    fn timepicker_focus_handle_none_by_default() {
        let tp = TimePicker::new("test");
        assert!(tp.focus_handle.is_none());
    }

    #[gpui::test]
    async fn timepicker_focus_handle_builder_stores_handle(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let handle = cx.focus_handle();
            let tp = TimePicker::new("test").focus_handle(&handle);
            assert!(
                tp.focus_handle.is_some(),
                "focus_handle(..) must round-trip into the field"
            );
        });
    }

    #[test]
    fn timepicker_highlighted_column_builder() {
        let tp = TimePicker::new("test").highlighted_column(1);
        assert_eq!(tp.highlighted_column, 1);
    }

    #[test]
    fn timepicker_highlighted_row_builder() {
        let tp = TimePicker::new("test").highlighted_row(Some(5));
        assert_eq!(tp.highlighted_row, Some(5));
    }

    // ── Hour/minute clamp tests ───────────────────────────────────────────

    #[test]
    fn hour_clamps_to_23() {
        let tp = TimePicker::new("test").hour(30);
        assert_eq!(tp.hour, 23);
    }

    #[test]
    fn minute_clamps_to_59() {
        let tp = TimePicker::new("test").minute(99);
        assert_eq!(tp.minute, 59);
    }

    // ── Helper function tests ─────────────────────────────────────────────

    #[test]
    fn column_item_count_24h() {
        assert_eq!(column_item_count(0, true), 24);
        assert_eq!(column_item_count(1, true), 12);
        assert_eq!(column_item_count(2, true), 2);
    }

    #[test]
    fn column_item_count_12h() {
        assert_eq!(column_item_count(0, false), 12);
        assert_eq!(column_item_count(1, false), 12);
        assert_eq!(column_item_count(2, false), 2);
    }

    #[test]
    fn default_row_hour_24h() {
        assert_eq!(default_row_for_column(0, 14, 30, true), 14);
        assert_eq!(default_row_for_column(0, 0, 0, true), 0);
    }

    #[test]
    fn default_row_hour_12h() {
        // hour=14 -> display 2 -> index 1 (1-based to 0-based)
        assert_eq!(default_row_for_column(0, 14, 30, false), 1);
        // hour=0 -> display 12 -> index 11
        assert_eq!(default_row_for_column(0, 0, 0, false), 11);
    }

    #[test]
    fn default_row_minute() {
        assert_eq!(default_row_for_column(1, 10, 35, true), 7);
        assert_eq!(default_row_for_column(1, 10, 0, true), 0);
    }

    #[test]
    fn default_row_ampm() {
        assert_eq!(default_row_for_column(2, 10, 0, false), 0); // AM
        assert_eq!(default_row_for_column(2, 14, 0, false), 1); // PM
    }

    #[test]
    fn resolve_selection_hour_24h() {
        let (h, m) = resolve_selection(0, 14, 10, 30, true);
        assert_eq!(h, 14);
        assert_eq!(m, 30);
    }

    #[test]
    fn resolve_selection_minute() {
        let (h, m) = resolve_selection(1, 6, 10, 30, true);
        assert_eq!(h, 10);
        assert_eq!(m, 30); // row 6 -> 30 min
    }

    #[test]
    fn resolve_selection_ampm() {
        // Select PM (row 1) while displaying hour 9
        let (h, m) = resolve_selection(2, 1, 9, 15, false);
        assert_eq!(h, 21); // 9 PM = 21
        assert_eq!(m, 15);
    }

    // ── TimePickerStyle smoke tests ───────────────────────────────────────

    #[test]
    fn timepicker_style_default_is_compact() {
        let tp = TimePicker::new("test");
        assert_eq!(tp.style, TimePickerStyle::Compact);
    }

    #[test]
    fn timepicker_style_compact() {
        let tp = TimePicker::new("test").style(TimePickerStyle::Compact);
        assert_eq!(tp.style, TimePickerStyle::Compact);
    }

    #[test]
    fn timepicker_style_field() {
        let tp = TimePicker::new("test").style(TimePickerStyle::Field);
        assert_eq!(tp.style, TimePickerStyle::Field);
    }

    #[test]
    fn timepicker_style_wheel() {
        let tp = TimePicker::new("test").style(TimePickerStyle::Wheel);
        assert_eq!(tp.style, TimePickerStyle::Wheel);
    }

    #[test]
    fn timepicker_style_stepper_field() {
        let tp = TimePicker::new("test").style(TimePickerStyle::StepperField);
        assert_eq!(tp.style, TimePickerStyle::StepperField);
    }

    // ── StepperField minute arithmetic ────────────────────────────────────

    #[test]
    fn shift_minutes_forward() {
        assert_eq!(shift_minutes(10, 25, 5), (10, 30));
    }

    #[test]
    fn shift_minutes_backward() {
        assert_eq!(shift_minutes(10, 0, -5), (9, 55));
    }

    #[test]
    fn shift_minutes_wraps_midnight_forward() {
        assert_eq!(shift_minutes(23, 55, 10), (0, 5));
    }

    #[test]
    fn shift_minutes_wraps_midnight_backward() {
        assert_eq!(shift_minutes(0, 0, -5), (23, 55));
    }
}
