//! HIG DatePicker — desktop-style date selector with calendar grid.
//!
//! A stateless `RenderOnce` component that renders a trigger button and,
//! when open, an absolute-positioned calendar dropdown. All state (selected
//! date, viewing month/year, open/closed) is owned by the parent.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FontWeight, KeyDownEvent, MouseDownEvent, SharedString, Window, div, px,
};

use crate::callback_types::{OnDateNavigate, OnToggle, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::CONTENT_MARGIN;
use crate::foundations::materials::{
    LensEffect, apply_standard_control_styling, glass_lens_surface,
};
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Calendar day-cell size for the `Compact` popover style (in points).
/// The popover calendar is visually denser than the inline `Graphical`
/// variant, so its cells shrink to match HIG's compact density.
const DATE_CELL_SIZE_COMPACT: f32 = 28.0;

/// Calendar day-cell size for the `Graphical` inline style (in points).
/// Matches macOS Tahoe's always-visible inline calendar sizing.
const DATE_CELL_SIZE_GRAPHICAL: f32 = 36.0;

/// Calendar day-of-week header row height (in points).
const DATE_HEADER_HEIGHT: f32 = 28.0;

/// Columns in the calendar grid (7 days of the week).
const DATE_GRID_COLUMNS: f32 = 7.0;

// ─── SimpleDate ─────────────────────────────────────────────────────────────

/// A lightweight date representation without external dependencies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl SimpleDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let month = month.clamp(1, 12);
        let day = day.clamp(1, Self::days_in_month(year, month));
        Self { year, month, day }
    }

    /// Format as "YYYY-MM-DD".
    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Returns the number of days in the given month, handling leap years.
    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30, // fallback
        }
    }

    /// Returns the day of the week: 0 = Sunday .. 6 = Saturday.
    ///
    /// Uses Tomohiko Sakamoto's algorithm.
    pub fn day_of_week(year: i32, month: u8, day: u8) -> u8 {
        if month == 0 || month > 12 {
            return 0; // Invalid month — fallback to Sunday
        }
        // Tomohiko Sakamoto's lookup table
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = year;
        if month < 3 {
            y -= 1;
        }
        let m = month as i32;
        let d = day as i32;
        ((y + y / 4 - y / 100 + y / 400 + T[(m - 1) as usize] + d).rem_euclid(7)) as u8
    }
}

// ─── Month name helper ──────────────────────────────────────────────────────

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn short_month_name(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Unk",
    }
}

/// Display format for the trigger button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateDisplayFormat {
    /// Locale-style short form, e.g. "Jun 15, 2025". Default per HIG
    /// macOS textual style.
    Locale,
    /// ISO 8601 short form "YYYY-MM-DD". Useful for developer-facing UIs.
    Iso,
}

/// Visual presentation style per HIG.
///
/// Marked `#[non_exhaustive]` so additional styles (e.g. a future
/// `Automatic`) can be added without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DatePickerStyle {
    /// Trigger button that pops a calendar dropdown. Day cells are 28pt
    /// to match the denser popover form. Default.
    #[default]
    Compact,
    /// Plain text field. The user types a date in the locale format;
    /// callers validate on blur. No dropdown, no icon trigger.
    Field,
    /// Always-visible inline calendar. The trigger button is omitted
    /// and `is_open` is ignored — the grid renders directly. Day cells
    /// are 36pt to match macOS Tahoe's inline sizing.
    Graphical,
    /// Text field paired with a ± stepper that increments or decrements
    /// the selected date by one day.
    StepperField,
}

// ─── DatePicker ─────────────────────────────────────────────────────────────

/// HIG date picker with a calendar grid dropdown.
///
/// Stateless `RenderOnce` — the parent owns all state (`selected`, `viewing_year`,
/// `viewing_month`, `is_open`) and provides callbacks.
///
/// # Example
///
/// ```ignore
/// DatePicker::new("my-date")
///     .selected(SimpleDate::new(2025, 6, 15))
///     .viewing(2025, 6)
///     .open(true)
///     .on_change(|date, _win, _cx| { /* update state */ })
///     .on_toggle(|open, _win, _cx| { /* toggle dropdown */ })
/// ```
#[derive(IntoElement)]
#[allow(clippy::type_complexity)]
pub struct DatePicker {
    id: ElementId,
    selected: Option<SimpleDate>,
    viewing_year: i32,
    viewing_month: u8,
    is_open: bool,
    on_change: Option<Box<dyn Fn(SimpleDate, &mut Window, &mut App) + 'static>>,
    on_toggle: OnToggle,
    on_navigate: OnDateNavigate,
    /// Fired when Left/Right/Up/Down arrow keys move the highlight.
    /// Parent should update its tracked `highlighted_day` to the emitted
    /// value. Crossing a month boundary fires `on_navigate` first, so
    /// the callback receives a day valid for the new viewing month.
    on_highlight: Option<Box<dyn Fn(u8, &mut Window, &mut App) + 'static>>,
    /// Whether this date picker trigger is keyboard-focused.
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in
    /// the Zed cross-reference audit. When set, the focus-ring visibility
    /// is derived from `handle.is_focused(window)` and the trigger
    /// threads `track_focus(&handle)`; otherwise uses the explicit
    /// [`focused`](Self::focused) bool.
    focus_handle: Option<gpui::FocusHandle>,
    /// The keyboard-highlighted day (1-based) in the current viewing month.
    highlighted_day: Option<u8>,
    /// Per-component override for the first day of the week. When `None`
    /// the date picker falls back to `TahoeTheme::first_weekday`, so
    /// locale-aware apps configure the weekday once on the theme instead
    /// of every picker site.
    first_weekday: Option<u8>,
    /// Earliest selectable date. Days before this render dimmed and ignore
    /// clicks. `None` means no lower bound.
    min_date: Option<SimpleDate>,
    /// Latest selectable date. Days after this render dimmed and ignore
    /// clicks. `None` means no upper bound.
    max_date: Option<SimpleDate>,
    /// Trigger-label format. Defaults to locale-style short form per HIG.
    display_format: DateDisplayFormat,
    /// HIG presentation style. Defaults to [`DatePickerStyle::Compact`].
    style: DatePickerStyle,
}

/// Total ordering helper for `SimpleDate`. We need it for range checks but
/// don't want to require `Ord` downstream.
fn date_cmp(a: SimpleDate, b: SimpleDate) -> std::cmp::Ordering {
    (a.year, a.month, a.day).cmp(&(b.year, b.month, b.day))
}

impl DatePicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            viewing_year: 2025,
            viewing_month: 1,
            is_open: false,
            on_change: None,
            on_toggle: None,
            on_navigate: None,
            on_highlight: None,
            focused: false,
            focus_handle: None,
            highlighted_day: None,
            first_weekday: None,
            min_date: None,
            max_date: None,
            display_format: DateDisplayFormat::Locale,
            style: DatePickerStyle::Compact,
        }
    }

    /// Select the HIG presentation style. Defaults to
    /// [`DatePickerStyle::Compact`].
    pub fn style(mut self, style: DatePickerStyle) -> Self {
        self.style = style;
        self
    }

    /// Per-component override for the first day of the week. When unset,
    /// the theme's `first_weekday` is used. `0` = Sunday (US), `1` =
    /// Monday (ISO 8601 / Europe). Values outside 0..=6 are clamped.
    pub fn first_weekday(mut self, day: u8) -> Self {
        self.first_weekday = Some(day.min(6));
        self
    }

    /// Constrain the earliest selectable date. Days before this are
    /// dimmed and non-interactive.
    pub fn min_date(mut self, date: SimpleDate) -> Self {
        self.min_date = Some(date);
        self
    }

    /// Constrain the latest selectable date. Days after this are dimmed
    /// and non-interactive.
    pub fn max_date(mut self, date: SimpleDate) -> Self {
        self.max_date = Some(date);
        self
    }

    /// Override the trigger label format.
    pub fn display_format(mut self, format: DateDisplayFormat) -> Self {
        self.display_format = format;
        self
    }

    pub fn selected(mut self, date: SimpleDate) -> Self {
        self.selected = Some(date);
        self
    }

    pub fn viewing(mut self, year: i32, month: u8) -> Self {
        self.viewing_year = year;
        self.viewing_month = month.clamp(1, 12);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SimpleDate, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Set the callback fired when the user navigates to a different month.
    pub fn on_navigate(
        mut self,
        handler: impl Fn(i32, u8, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(Box::new(handler));
        self
    }

    /// Set the callback fired when arrow keys move the highlighted day.
    /// Parents that already render `highlighted_day` from their own state
    /// should update it with the emitted value.
    pub fn on_highlight(mut self, handler: impl Fn(u8, &mut Window, &mut App) + 'static) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }

    /// Marks this date picker as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`gpui::FocusHandle`] so the date picker participates in the
    /// host's focus graph. When set, the focus-ring is derived from
    /// `handle.is_focused(window)` and the trigger threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &gpui::FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Sets the keyboard-highlighted day in the calendar grid (1-based).
    pub fn highlighted_day(mut self, day: Option<u8>) -> Self {
        self.highlighted_day = day;
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        let style = self.style;
        // Day-cell size switches by style: the Compact popover is denser
        // at 28pt while the Graphical inline calendar uses 36pt to match
        // macOS Tahoe.
        let cell_size = match style {
            DatePickerStyle::Graphical => DATE_CELL_SIZE_GRAPHICAL,
            _ => DATE_CELL_SIZE_COMPACT,
        };

        // ── Trigger label ──────────────────────────────────────────────────
        let display_format = self.display_format;
        let trigger_label: SharedString = self
            .selected
            .map(|d| {
                let s = match display_format {
                    DateDisplayFormat::Iso => d.format(),
                    DateDisplayFormat::Locale => {
                        format!("{} {}, {:04}", short_month_name(d.month), d.day, d.year)
                    }
                };
                SharedString::from(s)
            })
            .unwrap_or_else(|| SharedString::from("Select date"));

        let trigger_text_color = if self.selected.is_some() {
            theme.text
        } else {
            theme.text_muted
        };

        let on_toggle = rc_wrap(self.on_toggle);
        let on_change = rc_wrap(self.on_change);
        let on_navigate = rc_wrap(self.on_navigate);
        let on_highlight = self.on_highlight.map(std::rc::Rc::new);

        // ── Trigger button ─────────────────────────────────────────────────
        // Trigger variants differ by style:
        //   Compact      -> button with list icon, popover on open
        //   Field        -> text-field visual, no icon, no popover
        //   Graphical    -> no trigger (inline calendar)
        //   StepperField -> text-field visual + ±1 day stepper buttons
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;

        // Icon is only shown for Compact; Field/StepperField look like
        // plain text fields.
        let show_trigger_icon = matches!(style, DatePickerStyle::Compact);

        let mut trigger_content = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(trigger_text_color)
                    .child(trigger_label),
            );
        if show_trigger_icon {
            trigger_content = trigger_content.child(
                Icon::new(IconName::ListTodo)
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
            .cursor_pointer();

        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);

        if let Some(handle) = self.focus_handle.as_ref() {
            trigger = trigger.track_focus(handle);
        }

        trigger = trigger
            .hover(|style| style.cursor_pointer())
            .child(trigger_content);

        // Click-to-toggle applies to Compact only; Field/StepperField are
        // text entry surfaces. (StepperField's ± buttons handle their own
        // clicks further below.)
        if matches!(style, DatePickerStyle::Compact)
            && let Some(handler) = toggle_for_trigger
        {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space opens the dropdown
        // for Compact. For Field/StepperField it commits the (parent-
        // validated) value by firing on_toggle(false); callers treat
        // that as a "blur".
        if let Some(handler) = trigger_key_toggle {
            let style_captured = style;
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event) {
                    match style_captured {
                        DatePickerStyle::Compact => {
                            if !is_open {
                                cx.stop_propagation();
                                handler(true, window, cx);
                            }
                        }
                        DatePickerStyle::Field | DatePickerStyle::StepperField => {
                            cx.stop_propagation();
                            handler(false, window, cx);
                        }
                        DatePickerStyle::Graphical => {}
                    }
                }
            });
        }

        // Graphical variant: render the calendar inline and skip the
        // trigger + popover entirely. All interactivity, keyboard
        // handling, and the focus ring live on the inline calendar.
        if matches!(style, DatePickerStyle::Graphical) {
            let calendar = build_inline_calendar(
                self.viewing_year,
                self.viewing_month,
                self.selected,
                self.highlighted_day,
                self.first_weekday,
                self.min_date,
                self.max_date,
                self.focus_handle.clone(),
                cell_size,
                theme,
                focused,
                on_change.clone(),
                on_navigate.clone(),
                on_highlight.clone(),
            );
            return div().child(calendar).into_any_element();
        }

        // Compact style defers the trigger into an `AnchoredOverlay` at
        // the end (so the calendar popover escapes parent overflow-hidden).
        // Other styles consume the trigger into their `container_root` as
        // before. `trigger_opt.take()` lets the branches below steal
        // ownership exactly once; Compact leaves it behind for the final
        // assembly to pick up.
        let mut trigger_opt = Some(trigger);

        // StepperField variant: wrap the text field in a row with ± buttons.
        let container_root = if matches!(style, DatePickerStyle::StepperField) {
            let t = trigger_opt
                .take()
                .expect("trigger available for StepperField");
            let selected = self.selected;
            let on_change_dec = on_change.clone();
            let on_change_inc = on_change.clone();
            let dec_btn = stepper_button(
                theme,
                ElementId::from((self.id.clone(), "dp-step-dec")),
                IconName::Minus,
                move |_event, window, cx| {
                    if let (Some(date), Some(handler)) = (selected, on_change_dec.as_ref()) {
                        let prev = shift_days(date, -1);
                        handler(prev, window, cx);
                    }
                },
            );
            let inc_btn = stepper_button(
                theme,
                ElementId::from((self.id.clone(), "dp-step-inc")),
                IconName::Plus,
                move |_event, window, cx| {
                    if let (Some(date), Some(handler)) = (selected, on_change_inc.as_ref()) {
                        let next = shift_days(date, 1);
                        handler(next, window, cx);
                    }
                },
            );
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_sm)
                .child(div().flex_grow().child(t))
                .child(dec_btn)
                .child(inc_btn)
        } else if matches!(style, DatePickerStyle::Compact) {
            // Trigger stays in `trigger_opt` for the AnchoredOverlay
            // assembly at the end; this placeholder is a `Div` to keep
            // the `if/else if/else` chain's types matching.
            div()
        } else {
            // Field
            let t = trigger_opt.take().expect("trigger available for Field");
            div().relative().child(t)
        };

        let container = container_root;
        // Dropdown is only built for Compact + is_open; holding it in an
        // Option lets us pass it into `AnchoredOverlay::content` at the
        // end without mutating a partial container during build.
        let mut dropdown_el: Option<gpui::AnyElement> = None;

        // Field / StepperField don't have a popover — the field itself
        // is the value-entry surface. Callers supply an on_change hook
        // for parse-on-blur handling (HIG guidance).
        let show_popover = matches!(style, DatePickerStyle::Compact) && self.is_open;
        if show_popover {
            let year = self.viewing_year;
            let month = self.viewing_month;
            let days_in = SimpleDate::days_in_month(year, month);
            // Offset for the first-of-month relative to the locale's first
            // weekday. Example: first_weekday=1 (Monday-first) and a month
            // that starts on Sunday (dow=0) leaves 6 leading cells, not 0.
            // Per-component override wins over the theme-wide default.
            let first_weekday = self.first_weekday.unwrap_or(theme.first_weekday).min(6);
            let raw_dow = SimpleDate::day_of_week(year, month, 1);
            let first_dow = ((raw_dow as i16 - first_weekday as i16).rem_euclid(7)) as u8;
            let highlighted_day = self.highlighted_day;
            let min_date = self.min_date;
            let max_date = self.max_date;

            // Previous month overflow days
            let prev_month = if month == 1 { 12 } else { month - 1 };
            let prev_year = if month == 1 { year - 1 } else { year };
            let prev_days = SimpleDate::days_in_month(prev_year, prev_month);

            // ── Header: prev/month-year/next ───────────────────────────────
            let on_nav_prev = on_navigate.clone();
            let on_nav_next = on_navigate.clone();

            let header_label = SharedString::from(format!("{} {}", month_name(month), year));

            let mut prev_btn = div()
                .id("datepicker-prev")
                .min_w(px(theme.target_size()))
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .rounded(theme.radius_md)
                .hover(|style| style.bg(theme.hover))
                .child(
                    Icon::new(IconName::ChevronLeft)
                        .size(theme.icon_size_inline)
                        .color(theme.text),
                );

            if let Some(handler) = on_nav_prev {
                prev_btn = prev_btn.on_click(move |_event, window, cx| {
                    handler(prev_year, prev_month, window, cx);
                });
            }

            let next_month_val = if month == 12 { 1 } else { month + 1 };
            let next_year_val = if month == 12 { year + 1 } else { year };

            let mut next_btn = div()
                .id("datepicker-next")
                .min_w(px(theme.target_size()))
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .rounded(theme.radius_md)
                .hover(|style| style.bg(theme.hover))
                .child(
                    Icon::new(IconName::ChevronRight)
                        .size(theme.icon_size_inline)
                        .color(theme.text),
                );

            if let Some(handler) = on_nav_next {
                next_btn = next_btn.on_click(move |_event, window, cx| {
                    handler(next_year_val, next_month_val, window, cx);
                });
            }

            let header = div()
                .flex()
                .items_center()
                .justify_between()
                .px(theme.spacing_sm)
                .py(theme.spacing_sm)
                .child(prev_btn)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                        .text_color(theme.text)
                        .child(header_label),
                )
                .child(next_btn);

            // ── Day-of-week row ────────────────────────────────────────────
            // Base labels start on Sunday. Rotate left by `first_weekday`
            // so the row leads with the locale's first day of the week.
            // Two-letter abbreviations starting Sunday. HIG macOS `NSCalendarView`
            // renders "Sun/Mon/..." at wide cells and the iOS compact calendar
            // uses single letters ("S M T W T F S") which collide on Tuesday
            // vs Thursday. Two letters resolve the collision while keeping a
            // uniform cell width across all seven columns.
            let base_labels = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
            let mut dow_row = div().flex().flex_row();
            for i in 0..7 {
                let idx = (i + first_weekday as usize) % 7;
                let label = base_labels[idx];
                dow_row = dow_row.child(
                    div()
                        .w(px(cell_size))
                        .h(px(DATE_HEADER_HEIGHT))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_style(TextStyle::Subheadline, theme)
                        .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                        .text_color(theme.text_muted)
                        .child(SharedString::from(label)),
                );
            }

            // ── Calendar grid (6 rows x 7 cols) ───────────────────────────
            let today = self.selected; // highlight reference
            let accent = theme.accent;
            let text_color = theme.text;
            let muted_color = theme.text_muted;
            let hover_bg = theme.hover_bg();

            let mut grid = div().flex().flex_col();

            let mut day_cursor: i32 = 1;
            let mut next_month_day: i32 = 1;

            for row_idx in 0u8..6 {
                let mut row = div().flex().flex_row();

                for col in 0u8..7 {
                    let cell_index = (row_idx as i32) * 7 + (col as i32);
                    let offset = cell_index - first_dow as i32;

                    let (display_day, is_current_month) = if offset < 0 {
                        // Previous month
                        let d = prev_days as i32 + offset + 1;
                        (d as u8, false)
                    } else if day_cursor <= days_in as i32 {
                        let d = day_cursor;
                        day_cursor += 1;
                        (d as u8, true)
                    } else {
                        // Next month
                        let d = next_month_day;
                        next_month_day += 1;
                        (d as u8, false)
                    };

                    let is_selected = is_current_month
                        && today.is_some_and(|t| {
                            t.year == year && t.month == month && t.day == display_day
                        });
                    let is_highlighted = is_current_month && highlighted_day == Some(display_day);
                    let cell_date = if is_current_month {
                        SimpleDate::new(year, month, display_day)
                    } else {
                        // Approximate prev/next month dates for range checks.
                        let (y, m) = if offset < 0 {
                            (prev_year, prev_month)
                        } else {
                            (next_year_val, next_month_val)
                        };
                        SimpleDate::new(y, m, display_day)
                    };
                    let in_range = min_date
                        .is_none_or(|d| date_cmp(cell_date, d) != std::cmp::Ordering::Less)
                        && max_date
                            .is_none_or(|d| date_cmp(cell_date, d) != std::cmp::Ordering::Greater);
                    let is_interactable = is_current_month && in_range;

                    let on_change_cell = on_change.clone();
                    let on_toggle_cell = on_toggle.clone();

                    let cell_id = ElementId::from(SharedString::from(format!(
                        "dp-{}-{}-{}",
                        row_idx, col, display_day
                    )));

                    let mut cell = div()
                        .id(cell_id)
                        .w(px(cell_size))
                        .h(px(cell_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_style(TextStyle::Subheadline, theme)
                        .rounded(px(cell_size / 2.0)); // circle

                    if is_selected && in_range {
                        cell = cell
                            .bg(accent)
                            .text_color(theme.text_on_accent)
                            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
                    } else if is_highlighted && in_range {
                        cell = cell
                            .bg(hover_bg)
                            .text_color(text_color)
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM));
                    } else if is_interactable {
                        cell = cell
                            .text_color(text_color)
                            .hover(|style| style.bg(theme.hover).cursor_pointer());
                    } else if is_current_month && !in_range {
                        // Out-of-range day inside the viewing month: dim
                        // to signal non-interactability.
                        cell = cell.text_color(muted_color).opacity(0.35);
                    } else {
                        cell = cell.text_color(muted_color).opacity(0.5);
                    }

                    cell = cell.child(SharedString::from(format!("{}", display_day)));

                    if is_interactable {
                        let date = SimpleDate::new(year, month, display_day);
                        cell = cell.cursor_pointer().on_click(move |_event, window, cx| {
                            if let Some(handler) = &on_change_cell {
                                handler(date, window, cx);
                            }
                            if let Some(handler) = &on_toggle_cell {
                                handler(false, window, cx);
                            }
                        });
                    }

                    row = row.child(cell);
                }

                grid = grid.child(row);
            }

            // ── Dropdown ───────────────────────────────────────────────────
            let dropdown_content = div()
                .flex()
                .flex_col()
                .p(theme.spacing_sm)
                .child(header)
                .child(dow_row)
                .child(grid);

            let dropdown_effect = LensEffect::liquid_glass(GlassSize::Medium, theme);
            let mut dropdown = glass_lens_surface(theme, &dropdown_effect, GlassSize::Medium)
                .w(px(cell_size * DATE_GRID_COLUMNS + CONTENT_MARGIN))
                .overflow_hidden()
                .id(ElementId::from((self.id.clone(), "dropdown")))
                .debug_selector(|| "date-picker-dropdown".into())
                .focusable();

            // Keyboard nav: Arrow keys + Enter + Escape on calendar.
            let key_on_toggle = on_toggle.clone();
            let key_on_change = on_change.clone();
            let key_on_navigate = on_navigate.clone();
            let key_on_highlight = on_highlight.clone();
            dropdown = dropdown.on_key_down(move |event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    _ if crate::foundations::keyboard::is_escape_key(event) => {
                        if let Some(ref handler) = key_on_toggle {
                            handler(false, window, cx);
                        }
                    }
                    "enter" => {
                        if let Some(day) = highlighted_day
                            && day >= 1
                            && day <= days_in
                        {
                            let date = SimpleDate::new(year, month, day);
                            if let Some(ref handler) = key_on_change {
                                handler(date, window, cx);
                            }
                            if let Some(ref handler) = key_on_toggle {
                                handler(false, window, cx);
                            }
                        }
                    }
                    // Left/Right ±1 day, Up/Down ±7 days (one week). Crossing
                    // a month boundary fires `on_navigate` with the new
                    // (year, month), then `on_highlight` with the target day.
                    key @ ("left" | "right" | "up" | "down") => {
                        cx.stop_propagation();
                        let start = highlighted_day.unwrap_or(1) as i32;
                        let delta = match key {
                            "left" => -1,
                            "right" => 1,
                            "up" => -7,
                            "down" => 7,
                            _ => 0,
                        };
                        let target = start + delta;
                        let (target_year, target_month, target_day) = if target < 1 {
                            let (py, pm) = if month == 1 {
                                (year - 1, 12)
                            } else {
                                (year, month - 1)
                            };
                            let pd = SimpleDate::days_in_month(py, pm) as i32 + target;
                            (py, pm, pd.max(1) as u8)
                        } else if target > days_in as i32 {
                            let (ny, nm) = if month == 12 {
                                (year + 1, 1)
                            } else {
                                (year, month + 1)
                            };
                            let nd = target - days_in as i32;
                            let nd = nd.min(SimpleDate::days_in_month(ny, nm) as i32).max(1);
                            (ny, nm, nd as u8)
                        } else {
                            (year, month, target as u8)
                        };
                        if (target_year, target_month) != (year, month)
                            && let Some(ref handler) = key_on_navigate
                        {
                            handler(target_year, target_month, window, cx);
                        }
                        if let Some(ref handler) = key_on_highlight {
                            handler(target_day, window, cx);
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
            dropdown_el = Some(dropdown.into_any_element());
        }

        // Assemble: Compact uses `AnchoredOverlay` so the calendar popover
        // escapes parent `overflow_hidden()`; Field and StepperField just
        // render the container_root (no popover surface).
        if matches!(style, DatePickerStyle::Compact) {
            let trigger = trigger_opt
                .take()
                .expect("trigger reserved for Compact AnchoredOverlay");
            let overlay_id = ElementId::from((self.id.clone(), "overlay"));
            let mut overlay = AnchoredOverlay::new(overlay_id, trigger)
                .anchor(OverlayAnchor::BelowLeft)
                .gap(theme.dropdown_offset);
            if let Some(dropdown) = dropdown_el {
                overlay = overlay.content(dropdown);
            }
            overlay.into_any_element()
        } else {
            container.into_any_element()
        }
    }
}

// ─── Style-specific helpers ─────────────────────────────────────────────────

/// Shift a [`SimpleDate`] by a signed day delta, rolling over months and
/// years. Used by the `StepperField` style's ± buttons.
fn shift_days(date: SimpleDate, delta: i32) -> SimpleDate {
    let mut year = date.year;
    let mut month = date.month as i32;
    let mut day = date.day as i32 + delta;
    // Walk backward/forward month by month rather than computing ordinal
    // dates, which keeps leap-year handling inside `days_in_month`.
    loop {
        if day < 1 {
            month -= 1;
            if month < 1 {
                month = 12;
                year -= 1;
            }
            day += SimpleDate::days_in_month(year, month as u8) as i32;
        } else {
            let max = SimpleDate::days_in_month(year, month as u8) as i32;
            if day > max {
                day -= max;
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
            } else {
                break;
            }
        }
    }
    SimpleDate::new(year, month as u8, day as u8)
}

/// Build a compact ± stepper button used by the `StepperField` style.
fn stepper_button(
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

/// Build the inline calendar used by the `Graphical` style.
///
/// Mirrors the popover body produced for `Compact` but with no
/// absolute positioning and no mouse-down-out dismissal — the calendar
/// is always visible. Keyboard navigation and the focus ring are
/// preserved so the a11y surface stays consistent across styles.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn build_inline_calendar(
    year: i32,
    month: u8,
    selected: Option<SimpleDate>,
    highlighted_day: Option<u8>,
    first_weekday_override: Option<u8>,
    min_date: Option<SimpleDate>,
    max_date: Option<SimpleDate>,
    focus_handle: Option<gpui::FocusHandle>,
    cell_size: f32,
    theme: &crate::foundations::theme::TahoeTheme,
    focused: bool,
    on_change: Option<std::rc::Rc<Box<dyn Fn(SimpleDate, &mut Window, &mut App) + 'static>>>,
    on_navigate: Option<std::rc::Rc<Box<dyn Fn(i32, u8, &mut Window, &mut App) + 'static>>>,
    on_highlight: Option<std::rc::Rc<Box<dyn Fn(u8, &mut Window, &mut App) + 'static>>>,
) -> gpui::AnyElement {
    let days_in = SimpleDate::days_in_month(year, month);
    let first_weekday = first_weekday_override.unwrap_or(theme.first_weekday).min(6);
    let raw_dow = SimpleDate::day_of_week(year, month, 1);
    let first_dow = ((raw_dow as i16 - first_weekday as i16).rem_euclid(7)) as u8;

    let prev_month = if month == 1 { 12 } else { month - 1 };
    let prev_year = if month == 1 { year - 1 } else { year };
    let prev_days = SimpleDate::days_in_month(prev_year, prev_month);

    let next_month_val = if month == 12 { 1 } else { month + 1 };
    let next_year_val = if month == 12 { year + 1 } else { year };

    // ── Header ────────────────────────────────────────────────────────────
    let header_label = SharedString::from(format!("{} {}", month_name(month), year));

    let on_nav_prev = on_navigate.clone();
    let mut prev_btn = div()
        .id("datepicker-graphical-prev")
        .min_w(px(theme.target_size()))
        .min_h(px(theme.target_size()))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .rounded(theme.radius_md)
        .hover(|style| style.bg(theme.hover))
        .child(
            Icon::new(IconName::ChevronLeft)
                .size(theme.icon_size_inline)
                .color(theme.text),
        );
    if let Some(handler) = on_nav_prev {
        prev_btn = prev_btn.on_click(move |_event, window, cx| {
            handler(prev_year, prev_month, window, cx);
        });
    }

    let on_nav_next = on_navigate.clone();
    let mut next_btn = div()
        .id("datepicker-graphical-next")
        .min_w(px(theme.target_size()))
        .min_h(px(theme.target_size()))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .rounded(theme.radius_md)
        .hover(|style| style.bg(theme.hover))
        .child(
            Icon::new(IconName::ChevronRight)
                .size(theme.icon_size_inline)
                .color(theme.text),
        );
    if let Some(handler) = on_nav_next {
        next_btn = next_btn.on_click(move |_event, window, cx| {
            handler(next_year_val, next_month_val, window, cx);
        });
    }

    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .px(theme.spacing_sm)
        .py(theme.spacing_sm)
        .child(prev_btn)
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                .text_color(theme.text)
                .child(header_label),
        )
        .child(next_btn);

    // ── Day-of-week row ──────────────────────────────────────────────────
    // Two-letter abbreviations disambiguate Tuesday/Thursday (both "T" in
    // 1-letter form). Matches the Compact popover at date_picker.rs:649.
    let base_labels = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    let mut dow_row = div().flex().flex_row();
    for i in 0..7 {
        let idx = (i + first_weekday as usize) % 7;
        let label = base_labels[idx];
        dow_row = dow_row.child(
            div()
                .w(px(cell_size))
                .h(px(DATE_HEADER_HEIGHT))
                .flex()
                .items_center()
                .justify_center()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .text_color(theme.text_muted)
                .child(SharedString::from(label)),
        );
    }

    // ── Calendar grid ────────────────────────────────────────────────────
    let today = selected;
    let accent = theme.accent;
    let text_color = theme.text;
    let muted_color = theme.text_muted;
    let hover_bg = theme.hover_bg();

    let mut grid = div().flex().flex_col();
    let mut day_cursor: i32 = 1;
    let mut next_month_day: i32 = 1;

    for row_idx in 0u8..6 {
        let mut row = div().flex().flex_row();
        for col in 0u8..7 {
            let cell_index = (row_idx as i32) * 7 + (col as i32);
            let offset = cell_index - first_dow as i32;
            let (display_day, is_current_month) = if offset < 0 {
                let d = prev_days as i32 + offset + 1;
                (d as u8, false)
            } else if day_cursor <= days_in as i32 {
                let d = day_cursor;
                day_cursor += 1;
                (d as u8, true)
            } else {
                let d = next_month_day;
                next_month_day += 1;
                (d as u8, false)
            };

            let is_selected = is_current_month
                && today
                    .is_some_and(|t| t.year == year && t.month == month && t.day == display_day);
            let is_highlighted = is_current_month && highlighted_day == Some(display_day);
            let cell_date = if is_current_month {
                SimpleDate::new(year, month, display_day)
            } else {
                let (y, m) = if offset < 0 {
                    (prev_year, prev_month)
                } else {
                    (next_year_val, next_month_val)
                };
                SimpleDate::new(y, m, display_day)
            };
            let in_range = min_date
                .is_none_or(|d| date_cmp(cell_date, d) != std::cmp::Ordering::Less)
                && max_date.is_none_or(|d| date_cmp(cell_date, d) != std::cmp::Ordering::Greater);
            let is_interactable = is_current_month && in_range;

            let on_change_cell = on_change.clone();

            let cell_id = ElementId::from(SharedString::from(format!(
                "dpg-{}-{}-{}",
                row_idx, col, display_day
            )));

            let mut cell = div()
                .id(cell_id)
                .w(px(cell_size))
                .h(px(cell_size))
                .flex()
                .items_center()
                .justify_center()
                .text_style(TextStyle::Subheadline, theme)
                .rounded(px(cell_size / 2.0));

            if is_selected && in_range {
                cell = cell
                    .bg(accent)
                    .text_color(theme.text_on_accent)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD));
            } else if is_highlighted && in_range {
                cell = cell
                    .bg(hover_bg)
                    .text_color(text_color)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM));
            } else if is_interactable {
                cell = cell
                    .text_color(text_color)
                    .hover(|style| style.bg(theme.hover).cursor_pointer());
            } else if is_current_month && !in_range {
                cell = cell.text_color(muted_color).opacity(0.35);
            } else {
                cell = cell.text_color(muted_color).opacity(0.5);
            }

            cell = cell.child(SharedString::from(format!("{}", display_day)));

            if is_interactable {
                let date = SimpleDate::new(year, month, display_day);
                cell = cell.cursor_pointer().on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_change_cell {
                        handler(date, window, cx);
                    }
                });
            }

            row = row.child(cell);
        }
        grid = grid.child(row);
    }

    // ── Wrap with focus ring + keyboard nav ──────────────────────────────
    let mut wrapper = div()
        .flex()
        .flex_col()
        .p(theme.spacing_sm)
        .child(header)
        .child(dow_row)
        .child(grid);

    wrapper = apply_standard_control_styling(wrapper, theme, GlassSize::Small, focused);
    if let Some(handle) = focus_handle.as_ref() {
        wrapper = wrapper.track_focus(handle);
    }

    let key_on_change = on_change.clone();
    let key_on_navigate = on_navigate.clone();
    let key_on_highlight = on_highlight.clone();
    let year_c = year;
    let month_c = month;
    let days_in_c = days_in;
    wrapper = wrapper.on_key_down(move |event: &KeyDownEvent, window, cx| {
        match event.keystroke.key.as_str() {
            "enter" => {
                if let Some(day) = highlighted_day
                    && day >= 1
                    && day <= days_in_c
                {
                    let date = SimpleDate::new(year_c, month_c, day);
                    if let Some(ref handler) = key_on_change {
                        handler(date, window, cx);
                    }
                }
            }
            key @ ("left" | "right" | "up" | "down") => {
                cx.stop_propagation();
                let start = highlighted_day.unwrap_or(1) as i32;
                let delta = match key {
                    "left" => -1,
                    "right" => 1,
                    "up" => -7,
                    "down" => 7,
                    _ => 0,
                };
                let target = start + delta;
                let (target_year, target_month, target_day) = if target < 1 {
                    let (py, pm) = if month_c == 1 {
                        (year_c - 1, 12)
                    } else {
                        (year_c, month_c - 1)
                    };
                    let pd = SimpleDate::days_in_month(py, pm) as i32 + target;
                    (py, pm, pd.max(1) as u8)
                } else if target > days_in_c as i32 {
                    let (ny, nm) = if month_c == 12 {
                        (year_c + 1, 1)
                    } else {
                        (year_c, month_c + 1)
                    };
                    let nd = target - days_in_c as i32;
                    let nd = nd.min(SimpleDate::days_in_month(ny, nm) as i32).max(1);
                    (ny, nm, nd as u8)
                } else {
                    (year_c, month_c, target as u8)
                };
                if (target_year, target_month) != (year_c, month_c)
                    && let Some(ref handler) = key_on_navigate
                {
                    handler(target_year, target_month, window, cx);
                }
                if let Some(ref handler) = key_on_highlight {
                    handler(target_day, window, cx);
                }
            }
            _ => {}
        }
    });

    wrapper.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{DatePicker, DatePickerStyle, SimpleDate, shift_days};
    use core::prelude::v1::test;

    // ── SimpleDate::days_in_month ──────────────────────────────────────────

    #[test]
    fn days_in_month_january() {
        assert_eq!(SimpleDate::days_in_month(2025, 1), 31);
    }

    #[test]
    fn days_in_month_april() {
        assert_eq!(SimpleDate::days_in_month(2025, 4), 30);
    }

    #[test]
    fn days_in_month_february_normal() {
        assert_eq!(SimpleDate::days_in_month(2023, 2), 28);
    }

    #[test]
    fn days_in_month_february_leap() {
        assert_eq!(SimpleDate::days_in_month(2024, 2), 29);
    }

    #[test]
    fn days_in_month_century_non_leap() {
        assert_eq!(SimpleDate::days_in_month(1900, 2), 28);
    }

    #[test]
    fn days_in_month_quad_century_leap() {
        assert_eq!(SimpleDate::days_in_month(2000, 2), 29);
    }

    // ── SimpleDate::day_of_week ────────────────────────────────────────────

    #[test]
    fn day_of_week_known_date() {
        // 2025-01-01 is a Wednesday (3)
        assert_eq!(SimpleDate::day_of_week(2025, 1, 1), 3);
    }

    #[test]
    fn day_of_week_sunday() {
        // 2025-06-15 is a Sunday (0)
        assert_eq!(SimpleDate::day_of_week(2025, 6, 15), 0);
    }

    // ── SimpleDate::format ─────────────────────────────────────────────────

    #[test]
    fn format_pads_correctly() {
        let d = SimpleDate::new(2025, 3, 7);
        assert_eq!(d.format(), "2025-03-07");
    }

    // ── DatePicker builder ─────────────────────────────────────────────────

    #[test]
    fn datepicker_defaults() {
        let dp = DatePicker::new("test");
        assert!(dp.selected.is_none());
        assert_eq!(dp.viewing_year, 2025);
        assert_eq!(dp.viewing_month, 1);
        assert!(!dp.is_open);
        assert!(dp.on_change.is_none());
        assert!(dp.on_toggle.is_none());
        assert!(dp.on_navigate.is_none());
        assert!(!dp.focused);
        assert!(dp.highlighted_day.is_none());
    }

    #[test]
    fn datepicker_selected_builder() {
        let date = SimpleDate::new(2025, 6, 15);
        let dp = DatePicker::new("test").selected(date);
        assert_eq!(dp.selected, Some(date));
    }

    #[test]
    fn datepicker_viewing_builder() {
        let dp = DatePicker::new("test").viewing(2024, 12);
        assert_eq!(dp.viewing_year, 2024);
        assert_eq!(dp.viewing_month, 12);
    }

    #[test]
    fn datepicker_open_builder() {
        let dp = DatePicker::new("test").open(true);
        assert!(dp.is_open);
    }

    #[test]
    fn datepicker_on_change_is_some() {
        let dp = DatePicker::new("test").on_change(|_, _, _| {});
        assert!(dp.on_change.is_some());
    }

    #[test]
    fn datepicker_on_toggle_is_some() {
        let dp = DatePicker::new("test").on_toggle(|_, _, _| {});
        assert!(dp.on_toggle.is_some());
    }

    #[test]
    fn datepicker_on_navigate_is_some() {
        let dp = DatePicker::new("test").on_navigate(|_, _, _, _| {});
        assert!(dp.on_navigate.is_some());
    }

    // ── day_of_week edge cases ─────────────────────────────────────────────

    #[test]
    fn day_of_week_y2k() {
        // 2000-01-01 = Saturday (6)
        assert_eq!(SimpleDate::day_of_week(2000, 1, 1), 6);
    }

    #[test]
    fn day_of_week_leap_day() {
        // 2024-02-29 = Thursday (4)
        assert_eq!(SimpleDate::day_of_week(2024, 2, 29), 4);
    }

    #[test]
    fn day_of_week_unix_epoch() {
        // 1970-01-01 = Thursday (4)
        assert_eq!(SimpleDate::day_of_week(1970, 1, 1), 4);
    }

    #[test]
    fn day_of_week_year_end() {
        // 2025-12-31 = Wednesday (3)
        assert_eq!(SimpleDate::day_of_week(2025, 12, 31), 3);
    }

    #[test]
    fn day_of_week_century_non_leap() {
        // 1900-03-01 = Thursday (4)
        assert_eq!(SimpleDate::day_of_week(1900, 3, 1), 4);
    }

    // ── days_in_month out-of-range fallback ────────────────────────────────

    #[test]
    fn days_in_month_zero_fallback() {
        assert_eq!(SimpleDate::days_in_month(2025, 0), 30);
    }

    #[test]
    fn days_in_month_thirteen_fallback() {
        assert_eq!(SimpleDate::days_in_month(2025, 13), 30);
    }

    // ── viewing builder clamps month ───────────────────────────────────────

    #[test]
    fn viewing_clamps_month_zero() {
        let dp = DatePicker::new("test").viewing(2025, 0);
        assert_eq!(dp.viewing_month, 1);
    }

    #[test]
    fn viewing_clamps_month_overflow() {
        let dp = DatePicker::new("test").viewing(2025, 15);
        assert_eq!(dp.viewing_month, 12);
    }

    // ── Keyboard nav builder tests ────────────────────────────────────────

    #[test]
    fn datepicker_focused_builder() {
        let dp = DatePicker::new("test").focused(true);
        assert!(dp.focused);
    }

    #[test]
    fn datepicker_highlighted_day_builder() {
        let dp = DatePicker::new("test").highlighted_day(Some(15));
        assert_eq!(dp.highlighted_day, Some(15));
    }

    #[test]
    fn datepicker_highlighted_day_none() {
        let dp = DatePicker::new("test").highlighted_day(None);
        assert_eq!(dp.highlighted_day, None);
    }

    #[test]
    fn day_of_week_rem_euclid_consistency() {
        // Verify day_of_week returns 0..6 for a range of dates.
        for year in [1900, 2000, 2024, 2025] {
            for month in 1u8..=12 {
                let dow = SimpleDate::day_of_week(year, month, 1);
                assert!(dow < 7, "day_of_week({}, {}, 1) = {}", year, month, dow);
            }
        }
    }

    // ── DatePickerStyle smoke tests ───────────────────────────────────────
    //
    // One smoke test per style: verifies the builder threads through and
    // that the value is retained. Rendering correctness is covered by
    // existing keyboard / grid tests plus the gallery examples.

    #[test]
    fn datepicker_style_default_is_compact() {
        let dp = DatePicker::new("test");
        assert_eq!(dp.style, DatePickerStyle::Compact);
    }

    #[test]
    fn datepicker_style_compact() {
        let dp = DatePicker::new("test").style(DatePickerStyle::Compact);
        assert_eq!(dp.style, DatePickerStyle::Compact);
    }

    #[test]
    fn datepicker_style_field() {
        let dp = DatePicker::new("test").style(DatePickerStyle::Field);
        assert_eq!(dp.style, DatePickerStyle::Field);
    }

    #[test]
    fn datepicker_style_graphical() {
        let dp = DatePicker::new("test").style(DatePickerStyle::Graphical);
        assert_eq!(dp.style, DatePickerStyle::Graphical);
    }

    #[test]
    fn datepicker_style_stepper_field() {
        let dp = DatePicker::new("test").style(DatePickerStyle::StepperField);
        assert_eq!(dp.style, DatePickerStyle::StepperField);
    }

    // ── StepperField day-arithmetic ───────────────────────────────────────

    #[test]
    fn shift_days_forward_within_month() {
        let d = shift_days(SimpleDate::new(2025, 6, 15), 1);
        assert_eq!((d.year, d.month, d.day), (2025, 6, 16));
    }

    #[test]
    fn shift_days_backward_across_month() {
        let d = shift_days(SimpleDate::new(2025, 3, 1), -1);
        assert_eq!((d.year, d.month, d.day), (2025, 2, 28));
    }

    #[test]
    fn shift_days_forward_across_year() {
        let d = shift_days(SimpleDate::new(2024, 12, 31), 1);
        assert_eq!((d.year, d.month, d.day), (2025, 1, 1));
    }

    #[test]
    fn shift_days_leap_year() {
        let d = shift_days(SimpleDate::new(2024, 2, 28), 1);
        assert_eq!((d.year, d.month, d.day), (2024, 2, 29));
    }
}

#[cfg(test)]
mod clip_escape_tests {
    use gpui::prelude::*;
    use gpui::{Context, IntoElement, Render, TestAppContext, div, px};

    use super::{DatePicker, DatePickerStyle};
    use crate::test_helpers::helpers::{LocatorExt, setup_test_window};

    /// Compact date picker with an open calendar must anchor the popover
    /// outside its parent's clip region.
    struct ClipEscapeHarness;

    impl Render for ClipEscapeHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div().pt(px(120.0)).pl(px(40.0)).child(
                div()
                    .debug_selector(|| "clip-region".into())
                    .w(px(120.0))
                    .h(px(32.0))
                    .overflow_hidden()
                    .child(
                        DatePicker::new("date-picker")
                            .style(DatePickerStyle::Compact)
                            .open(true),
                    ),
            )
        }
    }

    #[gpui::test]
    async fn compact_calendar_anchors_outside_parent_clip(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, _cx| ClipEscapeHarness);

        let clip = cx.get_element("clip-region");
        let dropdown = cx.get_element("date-picker-dropdown");

        assert!(
            dropdown.bounds.top() >= clip.bounds.bottom(),
            "dropdown.top() {:?} should be at or below clip.bottom() {:?}",
            dropdown.bounds.top(),
            clip.bounds.bottom(),
        );
    }
}
