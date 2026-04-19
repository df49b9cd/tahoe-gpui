//! HIG DatePicker — desktop-style date selector with calendar grid.
//!
//! A stateless `RenderOnce` component that renders a trigger button and,
//! when open, an absolute-positioned calendar dropdown. All state (selected
//! date, viewing month/year, open/closed) is owned by the parent.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FontWeight, KeyDownEvent, MouseDownEvent, SharedString, Window, deferred, div,
    px,
};

use crate::callback_types::{OnDateNavigate, OnToggle, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::CONTENT_MARGIN;
use crate::foundations::materials::{apply_standard_control_styling, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Calendar day-cell square size (width and height, in points).
const DATE_CELL_SIZE: f32 = 36.0;

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
        }
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

    /// Attach a [`FocusHandle`] so the date picker participates in the
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
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;

        let trigger_content = div()
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
            )
            .child(
                Icon::new(IconName::ListTodo)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            );

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

        // ── Container ──────────────────────────────────────────────────────
        let mut container = div().relative().child(trigger);

        if self.is_open {
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
            let base_labels = ["S", "M", "T", "W", "T", "F", "S"];
            let mut dow_row = div().flex().flex_row();
            for i in 0..7 {
                let idx = (i + first_weekday as usize) % 7;
                let label = base_labels[idx];
                dow_row = dow_row.child(
                    div()
                        .w(px(DATE_CELL_SIZE))
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
                        .w(px(DATE_CELL_SIZE))
                        .h(px(DATE_CELL_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_style(TextStyle::Subheadline, theme)
                        .rounded(px(DATE_CELL_SIZE / 2.0)); // circle

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

            let mut dropdown = glass_surface(
                div()
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .w(px(DATE_CELL_SIZE * DATE_GRID_COLUMNS + CONTENT_MARGIN)) // 7 cells + padding
                    .overflow_hidden(),
                theme,
                GlassSize::Medium,
            )
            .id(ElementId::from((self.id.clone(), "dropdown")))
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

            container = container.child(deferred(dropdown).with_priority(1));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{DatePicker, SimpleDate};
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

    // u2500u2500 day_of_week edge cases u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

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

    // u2500u2500 days_in_month out-of-range fallback u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

    #[test]
    fn days_in_month_zero_fallback() {
        assert_eq!(SimpleDate::days_in_month(2025, 0), 30);
    }

    #[test]
    fn days_in_month_thirteen_fallback() {
        assert_eq!(SimpleDate::days_in_month(2025, 13), 30);
    }

    // u2500u2500 viewing builder clamps month u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

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
}
