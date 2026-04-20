//! Interactive slider primitive with click-to-seek and drag support.

use crate::foundations::layout::SPACING_8;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, Bounds, CursorStyle, ElementId, Entity, FocusHandle, GlobalElementId, Hsla,
    InspectorElementId, KeyDownEvent, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, SharedString, Style, TextAlign, TextRun, Window, div, fill, point, px,
    relative,
};

use crate::callback_types::OnF32Change;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use crate::ids::next_element_id;

/// Slider axis orientation. HIG `NSSlider.sliderType = .linear` with
/// `isVertical` toggled: vertical sliders place the minimum at the bottom
/// and increase upward.
///
/// # TODO: circular slider
///
/// `NSSlider.sliderType = .circular` (circular/rotary) is intentionally
/// deferred. It requires a dedicated hit-test / painting pipeline for the
/// circular track + rotary thumb, plus its own keyboard semantics, and
/// there is no first-party circular-slider consumer in tree to anchor
/// the design against. Revisit once a concrete use-case lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SliderOrientation {
    /// Track runs horizontally (left → right in LTR). HIG default.
    #[default]
    Horizontal,
    /// Track runs vertically; minimum sits at the bottom. HIG macOS
    /// vertical `NSSlider` orientation.
    Vertical,
}

/// Which thumb is being dragged in range mode. HIG-consistent
/// double-ended slider (`NSSliderCell` with `allowsTickMarkValuesOnly = false`
/// and two cells): the user drags the nearer of the two thumbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveThumb {
    Low,
    High,
}

/// Option alias for range-mode change callbacks — fires with `(low, high)`
/// after snapping. Matches the `OnF32Change` shape but pairs the two
/// endpoints so consumers get the full interval in one hop.
#[allow(clippy::type_complexity)]
pub type OnRangeChange = Option<Box<dyn Fn(f32, f32, &mut Window, &mut App) + 'static>>;

/// An interactive slider component for values in the 0.0..1.0 range.
///
/// Supports click-to-set and drag interactions. Used internally by
/// `AudioPlayerView` for seek and volume controls.
///
/// # Stepped mode
///
/// Set `step_count` to enable discrete/stepped behavior. The slider will snap
/// to evenly-spaced positions (e.g. `step_count = 5` gives 0.0, 0.25, 0.5, 0.75, 1.0).
///
/// # Min/Max icons
///
/// Per HIG, sliders can display icons at the left and right ends to
/// illustrate the meaning of the minimum and maximum values.
/// Keyboard step increment for continuous mode (1% of range).
const KEYBOARD_STEP: f32 = 0.01;

/// Boxed factory that builds an `AnyElement` (used for slider end-cap icons).
type ElementFactory = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

pub struct Slider {
    element_id: ElementId,
    focus_handle: FocusHandle,
    value: f32,
    is_dragging: bool,
    last_bounds: Option<Bounds<Pixels>>,
    height: Pixels,
    thumb_size: Pixels,
    color: Option<Hsla>,
    track_color: Option<Hsla>,
    on_change: OnF32Change,
    /// Number of discrete steps. `None` = continuous.
    step_count: Option<usize>,
    /// Factory for the minimum (left) icon, called on each render.
    min_icon: Option<ElementFactory>,
    /// Factory for the maximum (right) icon, called on each render.
    max_icon: Option<ElementFactory>,
    /// Accessibility label for screen readers.
    accessibility_label: Option<gpui::SharedString>,
    /// When true, tick marks are rendered below the track at each step
    /// boundary. Requires `step_count` to be set.
    show_ticks: bool,
    /// Optional formatter for the value tooltip shown while dragging.
    /// When `None`, no tooltip is rendered.
    value_formatter: Option<Box<dyn Fn(f32) -> String + 'static>>,
    /// Track axis. Vertical sliders flip `value_from_position` onto the
    /// y-axis and swap keyboard semantics so Up/Right increase and
    /// Down/Left decrease regardless of orientation.
    orientation: SliderOrientation,
    /// When true, the slider exposes two thumbs bracketing an interval
    /// `[range_low, range_high]`. `value` is treated as the alias for
    /// `range_high` so existing read-only consumers keep working.
    range_mode: bool,
    /// Low endpoint of the range. Ignored unless `range_mode` is set.
    range_low: f32,
    /// High endpoint of the range — kept in sync with `value` so existing
    /// single-thumb consumers that read `value` still see the upper bound.
    /// Ignored unless `range_mode` is set.
    range_high: f32,
    /// Which thumb (low/high) last received input. Used so keyboard
    /// nudges target the same thumb the user most recently clicked.
    active_thumb: ActiveThumb,
    /// Range-mode change callback — fires after each snapped update.
    on_change_range: OnRangeChange,
}

impl Slider {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("slider"),
            focus_handle: cx.focus_handle(),
            value: 0.0,
            is_dragging: false,
            last_bounds: None,
            // HIG / NSSlider: 4 pt track, ~20 pt lozenge thumb. The
            // prior 6 pt / 14 pt defaults diverged visibly from the system
            // control — see the HIG Selection & Input audit finding 12.
            height: px(4.0),
            thumb_size: px(20.0),
            color: None,
            track_color: None,
            on_change: None,
            step_count: None,
            min_icon: None,
            max_icon: None,
            accessibility_label: None,
            show_ticks: false,
            value_formatter: None,
            orientation: SliderOrientation::Horizontal,
            range_mode: false,
            range_low: 0.0,
            range_high: 0.0,
            active_thumb: ActiveThumb::High,
            on_change_range: None,
        }
    }

    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        self.value = value.clamp(0.0, 1.0);
        if self.range_mode {
            // Keep the high thumb in sync — `value` aliases `range_high`
            // so single-thumb callers that still read `value` see the
            // upper endpoint without needing the range API.
            self.range_high = self.value.max(self.range_low);
        }
        cx.notify();
    }

    /// Configure the slider's axis. Vertical orientation flips the track
    /// so the minimum sits at the bottom and keyboard Up/Right increase.
    pub fn set_orientation(&mut self, orientation: SliderOrientation) {
        self.orientation = orientation;
    }

    /// Enable range (double-thumb) mode. Two thumbs expose the interval
    /// `[range_low, range_high]`; `value` remains aliased to
    /// `range_high`. Set `on_change_range` to observe both endpoints.
    pub fn set_range_mode(&mut self, enabled: bool) {
        self.range_mode = enabled;
        if enabled {
            // Seed a sensible default range if the caller hasn't set one
            // yet: low stays at 0.0 and high mirrors `value`.
            self.range_high = self.range_high.max(self.value);
            if self.range_low > self.range_high {
                self.range_low = self.range_high;
            }
        }
    }

    /// Set both range endpoints at once. Values are clamped to `[0, 1]`
    /// and reordered so `low <= high`.
    pub fn set_range(&mut self, low: f32, high: f32, cx: &mut Context<Self>) {
        let (l, h) = (low.clamp(0.0, 1.0), high.clamp(0.0, 1.0));
        let (l, h) = if l <= h { (l, h) } else { (h, l) };
        self.range_low = l;
        self.range_high = h;
        self.value = h;
        cx.notify();
    }

    /// Observe range-mode changes. The callback receives `(low, high)`
    /// post-snap.
    pub fn set_on_change_range(
        &mut self,
        handler: impl Fn(f32, f32, &mut Window, &mut App) + 'static,
    ) {
        self.on_change_range = Some(Box::new(handler));
    }

    /// Read the current range endpoints. Returns `(range_low, range_high)`
    /// whether or not `range_mode` is enabled — useful for tests.
    pub fn range(&self) -> (f32, f32) {
        (self.range_low, self.range_high)
    }

    pub fn set_on_change(&mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) {
        self.on_change = Some(Box::new(handler));
    }

    pub fn set_height(&mut self, height: Pixels) {
        self.height = height;
    }

    pub fn set_thumb_size(&mut self, size: Pixels) {
        self.thumb_size = size;
    }

    pub fn set_color(&mut self, color: Hsla) {
        self.color = Some(color);
    }

    pub fn set_track_color(&mut self, color: Hsla) {
        self.track_color = Some(color);
    }

    /// Enables discrete/stepped mode. The slider snaps to `count` evenly-spaced positions.
    /// For example, `count = 5` gives values [0.0, 0.25, 0.5, 0.75, 1.0].
    /// `count` must be >= 2 (otherwise ignored).
    pub fn set_step_count(&mut self, count: usize) {
        if count >= 2 {
            self.step_count = Some(count);
        }
    }

    /// Sets a factory for the minimum (left) icon of the slider.
    pub fn set_min_icon(
        &mut self,
        factory: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) {
        self.min_icon = Some(Box::new(factory));
    }

    /// Sets a factory for the maximum (right) icon of the slider.
    pub fn set_max_icon(
        &mut self,
        factory: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) {
        self.max_icon = Some(Box::new(factory));
    }

    /// Sets an accessibility label for screen readers.
    pub fn set_accessibility_label(&mut self, label: impl Into<gpui::SharedString>) {
        self.accessibility_label = Some(label.into());
    }

    /// Enable rendering of tick marks under the track.
    ///
    /// Ticks are drawn at each step boundary when `step_count` is set and
    /// this flag is true. HIG macOS: "Use tick marks to increase clarity
    /// and accuracy." Ignored in continuous mode.
    pub fn set_show_ticks(&mut self, show: bool) {
        self.show_ticks = show;
    }

    /// Set a formatter invoked while the user drags the thumb to build a
    /// value tooltip over the control.
    ///
    /// `None` — the default — disables the tooltip. HIG macOS: "provide a
    /// tooltip that displays the value of the thumb when people hold
    /// their pointer over it."
    pub fn set_value_formatter(&mut self, formatter: impl Fn(f32) -> String + 'static) {
        self.value_formatter = Some(Box::new(formatter));
    }

    /// Snaps a continuous value to the nearest step if stepped mode is active.
    fn snap_value(&self, raw: f32) -> f32 {
        match self.step_count {
            Some(count) if count >= 2 => {
                let steps = (count - 1) as f32;
                (raw * steps).round() / steps
            }
            _ => raw,
        }
    }

    /// Unified event-to-fraction map that accounts for orientation. In
    /// vertical mode the event's y is projected onto the track height
    /// and inverted so bottom = 0.0 / top = 1.0, matching `NSSlider`
    /// vertical semantics.
    fn fraction_from_event(&self, x: Pixels, y: Pixels, rtl: bool) -> f32 {
        let Some(bounds) = self.last_bounds else {
            return self.value;
        };
        match self.orientation {
            SliderOrientation::Horizontal => {
                let width = bounds.size.width;
                if f32::from(width) <= 0.0 {
                    return 0.0;
                }
                let relative_x = x - bounds.left();
                let fraction = (f32::from(relative_x) / f32::from(width)).clamp(0.0, 1.0);
                if rtl { 1.0 - fraction } else { fraction }
            }
            SliderOrientation::Vertical => {
                let height = bounds.size.height;
                if f32::from(height) <= 0.0 {
                    return 0.0;
                }
                let relative_y = y - bounds.top();
                let raw = (f32::from(relative_y) / f32::from(height)).clamp(0.0, 1.0);
                // Bottom-anchored: invert so y=bottom → 1.0.
                1.0 - raw
            }
        }
    }

    /// Pick the nearer of the two range thumbs to `fraction`. Used on
    /// mouse-down to decide which endpoint this drag targets.
    fn nearest_thumb(&self, fraction: f32) -> ActiveThumb {
        let to_low = (fraction - self.range_low).abs();
        let to_high = (fraction - self.range_high).abs();
        if to_low <= to_high {
            ActiveThumb::Low
        } else {
            ActiveThumb::High
        }
    }

    /// Commit a new fraction to the active thumb in range mode, clamping
    /// it so the two thumbs can't cross.
    fn apply_range_fraction(&mut self, fraction: f32) {
        match self.active_thumb {
            ActiveThumb::Low => {
                self.range_low = fraction.min(self.range_high);
            }
            ActiveThumb::High => {
                self.range_high = fraction.max(self.range_low);
            }
        }
        // `value` aliases the high thumb so existing single-thumb readers
        // continue to see a meaningful number.
        self.value = self.range_high;
    }

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        self.is_dragging = true;
        // On the very first click the prepaint hasn't run yet, so
        // `last_bounds` is `None`. Defer the seek: it will fire on the next
        // mouse-move once bounds are cached, or we compute it from the
        // event's relative position when possible.
        if self.last_bounds.is_some() {
            let rtl = cx.theme().is_rtl();
            let raw = self.fraction_from_event(event.position.x, event.position.y, rtl);
            let new_value = self.snap_value(raw);
            if new_value.is_finite() {
                if self.range_mode {
                    self.active_thumb = self.nearest_thumb(new_value);
                    self.apply_range_fraction(new_value);
                } else {
                    self.value = new_value;
                }
            }
            if self.range_mode {
                if let Some(on_change) = &self.on_change_range {
                    on_change(self.range_low, self.range_high, window, cx);
                }
            } else if let Some(on_change) = &self.on_change {
                on_change(self.value, window, cx);
            }
        }
        cx.notify();
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_dragging = false;
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_dragging {
            let rtl = cx.theme().is_rtl();
            let raw = self.fraction_from_event(event.position.x, event.position.y, rtl);
            let new_value = self.snap_value(raw);
            if new_value.is_finite() {
                if self.range_mode {
                    self.apply_range_fraction(new_value);
                } else {
                    self.value = new_value;
                }
            }
            if self.range_mode {
                if let Some(on_change) = &self.on_change_range {
                    on_change(self.range_low, self.range_high, window, cx);
                }
            } else if let Some(on_change) = &self.on_change {
                on_change(self.value, window, cx);
            }
            cx.notify();
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base_step = match self.step_count {
            Some(count) if count >= 2 => 1.0 / (count - 1) as f32,
            _ => KEYBOARD_STEP,
        };
        // HIG Slider accessibility: arrow keys move 1 unit; Shift+arrow moves
        // 10 units (a "big step"). For stepped sliders this still jumps by
        // `base_step * 10`, rounded into the step grid by `snap_value` below.
        let step = if event.keystroke.modifiers.shift {
            base_step * 10.0
        } else {
            base_step
        };

        let rtl = cx.theme().is_rtl();
        // Horizontal: left/right follow RTL; up always increases. Vertical:
        // up/right increase, down/left decrease — matches NSSlider
        // vertical orientation where "forward" is upward.
        let (plus_keys, minus_keys) = match self.orientation {
            SliderOrientation::Horizontal => {
                if rtl {
                    (vec!["left", "up"], vec!["right", "down"])
                } else {
                    (vec!["right", "up"], vec!["left", "down"])
                }
            }
            SliderOrientation::Vertical => (vec!["up", "right"], vec!["down", "left"]),
        };
        let key = event.keystroke.key.as_str();

        // In range mode, keyboard drives the thumb that was most-recently
        // clicked; default is the high thumb so single-thumb keyboard-only
        // consumers keep the old behaviour.
        let current = if self.range_mode {
            match self.active_thumb {
                ActiveThumb::Low => self.range_low,
                ActiveThumb::High => self.range_high,
            }
        } else {
            self.value
        };

        let new_value = if plus_keys.contains(&key) {
            Some(((current + step) * 1000.0).round() / 1000.0).map(|v: f32| v.min(1.0))
        } else if minus_keys.contains(&key) {
            Some(((current - step) * 1000.0).round() / 1000.0).map(|v: f32| v.max(0.0))
        } else if key == "home" {
            Some(0.0)
        } else if key == "end" {
            Some(1.0)
        } else {
            None
        };

        if let Some(v) = new_value {
            let snapped = self.snap_value(v);
            if self.range_mode {
                self.apply_range_fraction(snapped);
                if let Some(on_change) = &self.on_change_range {
                    on_change(self.range_low, self.range_high, window, cx);
                }
            } else {
                self.value = snapped;
                if let Some(on_change) = &self.on_change {
                    on_change(self.value, window, cx);
                }
            }
            cx.notify();
        }
    }
}

impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Build icon elements before borrowing theme (factory closures need &mut App)
        let min_icon_el = self.min_icon.as_ref().map(|f| f(window, cx));
        let max_icon_el = self.max_icon.as_ref().map(|f| f(window, cx));

        let theme = cx.theme();
        let focused = self.focus_handle.is_focused(window);
        let color = self.color.unwrap_or(theme.accent);
        let track_color = self.track_color.unwrap_or_else(|| {
            theme
                .glass
                .accessible_bg(GlassSize::Small, theme.accessibility_mode)
        });
        let natural_radius = px(f32::from(self.height) / 2.0);
        let radius = natural_radius.min(theme.glass.radius(GlassSize::Small));
        let thumb_radius = px(f32::from(self.thumb_size) / 2.0);

        // HIG: 44pt minimum hit area for touch targets
        let hit_area_height = self.thumb_size.max(px(theme.target_size()));

        let rtl = theme.is_rtl();
        let tick_count = if self.show_ticks {
            self.step_count
        } else {
            None
        };
        let is_dragging = self.is_dragging;
        let tooltip_text = self
            .value_formatter
            .as_ref()
            .filter(|_| is_dragging)
            .map(|f| SharedString::from(f(self.value)));
        let tooltip_bg = theme.surface;
        let tooltip_fg = theme.text;
        let range = if self.range_mode {
            Some((self.range_low, self.range_high))
        } else {
            None
        };
        let track_element = SliderTrackElement {
            slider: cx.entity().clone(),
            value: self.value,
            height: self.height,
            thumb_size: self.thumb_size,
            color,
            track_color,
            radius,
            thumb_radius,
            focused,
            rtl,
            tick_count,
            tick_color: theme.text_muted,
            tooltip_text,
            tooltip_bg,
            tooltip_fg,
            orientation: self.orientation,
            range,
        };

        // Main slider track: its layout flips based on orientation so
        // vertical sliders stretch along the y-axis instead of the x-axis.
        // HIG vertical `NSSlider` sizes to a fixed width similar to the
        // hit-area height.
        let mut track_div = div()
            .id(self.element_id.clone())
            .debug_selector(|| "slider-track".into())
            .track_focus(&self.focus_handle)
            .flex()
            .items_center()
            .justify_center()
            .cursor(CursorStyle::PointingHand)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(track_element);

        // VoiceOver: per HIG Sliders the current value must be spoken on
        // every change. Use the caller's value_formatter when supplied;
        // otherwise fall back to a percent representation of the [0, 1]
        // range (sliders whose domain is outside [0, 1] should always
        // supply a formatter for meaningful readout).
        let ax_value_string = self
            .value_formatter
            .as_ref()
            .map(|f| f(self.value))
            .unwrap_or_else(|| format!("{:.0} percent", self.value * 100.0));
        let mut props = AccessibilityProps::new()
            .role(AccessibilityRole::Slider)
            .value(SharedString::from(ax_value_string));
        if let Some(label) = self.accessibility_label.clone() {
            props = props.label(label);
        }
        track_div = track_div.with_accessibility(&props);

        match self.orientation {
            SliderOrientation::Horizontal => {
                let mut slider_row = div().w_full().flex().items_center().gap(px(SPACING_8));
                if let Some(icon) = min_icon_el {
                    slider_row = slider_row.child(div().flex_shrink_0().child(icon));
                }
                slider_row = slider_row.child(track_div.flex_1().h(hit_area_height));
                if let Some(icon) = max_icon_el {
                    slider_row = slider_row.child(div().flex_shrink_0().child(icon));
                }
                slider_row.into_any_element()
            }
            SliderOrientation::Vertical => {
                // Vertical stack: max icon sits above, min icon below, so
                // the icon order visually encodes "up = more".
                let mut slider_col = div()
                    .h_full()
                    .min_h(px(120.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(SPACING_8));
                if let Some(icon) = max_icon_el {
                    slider_col = slider_col.child(div().flex_shrink_0().child(icon));
                }
                slider_col = slider_col.child(track_div.flex_1().w(hit_area_height));
                if let Some(icon) = min_icon_el {
                    slider_col = slider_col.child(div().flex_shrink_0().child(icon));
                }
                slider_col.into_any_element()
            }
        }
    }
}

// ── Custom Element to capture bounds and render track + thumb ──

struct SliderTrackElement {
    slider: Entity<Slider>,
    value: f32,
    height: Pixels,
    thumb_size: Pixels,
    color: Hsla,
    track_color: Hsla,
    radius: Pixels,
    thumb_radius: Pixels,
    focused: bool,
    /// Right-to-left layout: the fill grows from the *right* edge and the
    /// thumb moves right→left as value increases (HIG Right-to-Left:
    /// Controls). Mirrors the direction used in
    /// `Slider::value_from_position`.
    rtl: bool,
    /// When `Some(n)`, paint n evenly-spaced tick marks below the track.
    tick_count: Option<usize>,
    /// Colour used for tick marks and tooltip border.
    tick_color: Hsla,
    /// When `Some`, paint a tooltip above the thumb while dragging.
    tooltip_text: Option<SharedString>,
    /// Background for the value tooltip.
    tooltip_bg: Hsla,
    /// Text color for the value tooltip.
    tooltip_fg: Hsla,
    /// Track axis — forwarded so the paint path can swap x/y layout.
    orientation: SliderOrientation,
    /// When set, paint a second (low) thumb and fill the segment
    /// `[range_low, range_high]` instead of `[0, value]`.
    range: Option<(f32, f32)>,
}

impl IntoElement for SliderTrackElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SliderTrackElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        match self.orientation {
            SliderOrientation::Horizontal => {
                style.size.width = relative(1.).into();
                style.size.height = self.height.into();
            }
            SliderOrientation::Vertical => {
                // Vertical tracks keep `self.height` as their *width* — HIG
                // vertical `NSSlider` treats the track as a rotated bar.
                style.size.width = self.height.into();
                style.size.height = relative(1.).into();
            }
        }
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Cache bounds for mouse hit-testing
        self.slider.update(cx, |slider, _cx| {
            slider.last_bounds = Some(bounds);
        });

        // Paint the two layouts separately. Vertical sharing a single
        // code path with horizontal would require a torrent of
        // conditional swaps; better to keep each branch linear.
        if self.orientation == SliderOrientation::Vertical {
            self.paint_vertical(bounds, window);
            return;
        }

        let track_height = self.height;
        let track_y = bounds.top() + (bounds.size.height - track_height) / 2.0;

        // Draw track background
        window.paint_quad(
            fill(
                Bounds::new(
                    gpui::point(bounds.left(), track_y),
                    gpui::size(bounds.size.width, track_height),
                ),
                self.track_color,
            )
            .corner_radii(self.radius),
        );

        // In range mode the fill is the *interval* between the low and
        // high thumbs; otherwise it's the single-thumb `[0, value]` bar.
        let (fill_start_frac, fill_end_frac) = if let Some((lo, hi)) = self.range {
            (lo, hi)
        } else {
            (0.0, self.value)
        };
        let fill_width = bounds.size.width * (fill_end_frac - fill_start_frac).max(0.0);
        let fill_start_x = if self.rtl {
            bounds.right() - bounds.size.width * fill_end_frac
        } else {
            bounds.left() + bounds.size.width * fill_start_frac
        };
        if f32::from(fill_width) > 0.0 {
            window.paint_quad(
                fill(
                    Bounds::new(
                        gpui::point(fill_start_x, track_y),
                        gpui::size(fill_width, track_height),
                    ),
                    self.color,
                )
                .corner_radii(self.radius),
            );
        }

        // Draw thumb circle — white with drop shadow per HIG. In RTL
        // the thumb sits on the *inner* edge of the fill (toward the centre
        // from the right), so subtract from the right instead of adding to
        // the left.
        let high_x = if self.rtl {
            bounds.right() - bounds.size.width * fill_end_frac - self.thumb_radius
        } else {
            bounds.left() + bounds.size.width * fill_end_frac - self.thumb_radius
        };
        let thumb_x = high_x;
        let thumb_y = bounds.top() + (bounds.size.height - self.thumb_size) / 2.0;
        let thumb_bounds = Bounds::new(
            gpui::point(thumb_x, thumb_y),
            gpui::size(self.thumb_size, self.thumb_size),
        );

        // Range mode: paint the low thumb too. Both endpoints share the
        // same visual treatment — HIG NSSlider double-cells are identical
        // pills mirrored across the fill interval.
        if self.range.is_some() {
            let low_x = if self.rtl {
                bounds.right() - bounds.size.width * fill_start_frac - self.thumb_radius
            } else {
                bounds.left() + bounds.size.width * fill_start_frac - self.thumb_radius
            };
            let low_bounds = Bounds::new(
                gpui::point(low_x, thumb_y),
                gpui::size(self.thumb_size, self.thumb_size),
            );
            window.paint_shadows(
                low_bounds,
                self.thumb_radius.into(),
                &[gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, 0.25),
                    offset: gpui::point(px(0.0), px(1.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                }],
            );
            window.paint_quad(
                fill(low_bounds, gpui::hsla(0.0, 0.0, 1.0, 1.0)).corner_radii(self.thumb_radius),
            );
        }

        // Shadow behind thumb
        window.paint_shadows(
            thumb_bounds,
            self.thumb_radius.into(),
            &[gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, 0.25),
                offset: gpui::point(px(0.0), px(1.0)),
                blur_radius: px(3.0),
                spread_radius: px(0.0),
            }],
        );

        // Focus ring on thumb (subtle accent outline when focused)
        if self.focused {
            let focus_expand = px(3.0);
            let focus_bounds = Bounds::new(
                gpui::point(thumb_x - focus_expand, thumb_y - focus_expand),
                gpui::size(
                    self.thumb_size + focus_expand * 2.0,
                    self.thumb_size + focus_expand * 2.0,
                ),
            );
            let mut accent = self.color;
            accent.a = 0.5;
            window.paint_quad(
                fill(focus_bounds, accent).corner_radii(self.thumb_radius + focus_expand),
            );
        }

        // White thumb circle
        window.paint_quad(
            fill(thumb_bounds, gpui::hsla(0.0, 0.0, 1.0, 1.0)).corner_radii(self.thumb_radius),
        );

        // Tick marks — painted below the track at each step boundary when
        // `show_ticks` is set and the slider is in stepped mode. Ticks are
        // 1 pt wide, ~6 pt tall, and start 3 pt below the track bottom so
        // they don't collide with the thumb.
        if let Some(count) = self.tick_count
            && count >= 2
        {
            let tick_w = px(1.0);
            let tick_h = px(6.0);
            let tick_y = track_y + track_height + px(3.0);
            let segment_w = bounds.size.width;
            for i in 0..count {
                let frac = i as f32 / (count - 1) as f32;
                let raw_x = bounds.left() + segment_w * frac;
                let tick_x = if self.rtl {
                    bounds.right() - (raw_x - bounds.left()) - tick_w
                } else {
                    raw_x - tick_w / 2.0
                };
                window.paint_quad(fill(
                    Bounds::new(gpui::point(tick_x, tick_y), gpui::size(tick_w, tick_h)),
                    self.tick_color,
                ));
            }
        }

        // Value tooltip — painted above the thumb while dragging. Uses
        // `shape_text` for the label and a fill quad for the backing so
        // the tooltip participates in layer-level painting without needing
        // a separate element.
        if let Some(ref text) = self.tooltip_text {
            let text_style = window.text_style();
            let font_size = text_style.font_size.to_pixels(window.rem_size());
            let run = TextRun {
                len: text.len(),
                font: text_style.font(),
                color: self.tooltip_fg,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            if let Ok(shaped) =
                window
                    .text_system()
                    .shape_text(text.clone(), font_size, &[run], None, None)
                && let Some(line) = shaped.into_vec().into_iter().next()
            {
                let pad_x = px(6.0);
                let pad_y = px(3.0);
                let line_w = line.unwrapped_layout.x_for_index(text.len());
                let line_h = window.line_height();
                let tooltip_w = line_w + pad_x * 2.0;
                let tooltip_h = line_h + pad_y * 2.0;
                let tooltip_x = (thumb_x + self.thumb_radius - tooltip_w / 2.0)
                    .max(bounds.left())
                    .min(bounds.right() - tooltip_w);
                let tooltip_y = thumb_y - tooltip_h - px(4.0);
                let tooltip_bounds = Bounds::new(
                    gpui::point(tooltip_x, tooltip_y),
                    gpui::size(tooltip_w, tooltip_h),
                );
                window.paint_quad(fill(tooltip_bounds, self.tooltip_bg).corner_radii(px(4.0)));
                let _ = line.paint(
                    point(tooltip_x + pad_x, tooltip_y + pad_y),
                    line_h,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
        }
    }
}

impl SliderTrackElement {
    /// Paint the vertical variant. Mirrors the horizontal path but
    /// operates on the y-axis: `value = 1.0` is at the top, `value = 0.0`
    /// at the bottom. Ticks and the drag tooltip are omitted for now —
    /// they're only used in horizontal galleries today and painting them
    /// sideways requires a rotated text path.
    fn paint_vertical(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        let track_width = self.height; // Width of the vertical track bar.
        let track_x = bounds.left() + (bounds.size.width - track_width) / 2.0;

        // Track background (full height).
        window.paint_quad(
            fill(
                Bounds::new(
                    gpui::point(track_x, bounds.top()),
                    gpui::size(track_width, bounds.size.height),
                ),
                self.track_color,
            )
            .corner_radii(self.radius),
        );

        let (fill_start_frac, fill_end_frac) = if let Some((lo, hi)) = self.range {
            (lo, hi)
        } else {
            (0.0, self.value)
        };
        // Bottom-anchored fill: y=bottom → fraction 0; y=top → fraction 1.
        let fill_h = bounds.size.height * (fill_end_frac - fill_start_frac).max(0.0);
        let fill_y = bounds.bottom() - bounds.size.height * fill_end_frac;
        if f32::from(fill_h) > 0.0 {
            window.paint_quad(
                fill(
                    Bounds::new(
                        gpui::point(track_x, fill_y),
                        gpui::size(track_width, fill_h),
                    ),
                    self.color,
                )
                .corner_radii(self.radius),
            );
        }

        let thumb_x = track_x + (track_width - self.thumb_size) / 2.0;
        let high_y = bounds.bottom() - bounds.size.height * fill_end_frac - self.thumb_radius;
        let thumb_bounds = Bounds::new(
            gpui::point(thumb_x, high_y),
            gpui::size(self.thumb_size, self.thumb_size),
        );
        window.paint_shadows(
            thumb_bounds,
            self.thumb_radius.into(),
            &[gpui::BoxShadow {
                color: gpui::hsla(0.0, 0.0, 0.0, 0.25),
                offset: gpui::point(px(0.0), px(1.0)),
                blur_radius: px(3.0),
                spread_radius: px(0.0),
            }],
        );
        if self.focused {
            let focus_expand = px(3.0);
            let focus_bounds = Bounds::new(
                gpui::point(thumb_x - focus_expand, high_y - focus_expand),
                gpui::size(
                    self.thumb_size + focus_expand * 2.0,
                    self.thumb_size + focus_expand * 2.0,
                ),
            );
            let mut accent = self.color;
            accent.a = 0.5;
            window.paint_quad(
                fill(focus_bounds, accent).corner_radii(self.thumb_radius + focus_expand),
            );
        }
        window.paint_quad(
            fill(thumb_bounds, gpui::hsla(0.0, 0.0, 1.0, 1.0)).corner_radii(self.thumb_radius),
        );

        if self.range.is_some() {
            let low_y = bounds.bottom() - bounds.size.height * fill_start_frac - self.thumb_radius;
            let low_bounds = Bounds::new(
                gpui::point(thumb_x, low_y),
                gpui::size(self.thumb_size, self.thumb_size),
            );
            window.paint_shadows(
                low_bounds,
                self.thumb_radius.into(),
                &[gpui::BoxShadow {
                    color: gpui::hsla(0.0, 0.0, 0.0, 0.25),
                    offset: gpui::point(px(0.0), px(1.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                }],
            );
            window.paint_quad(
                fill(low_bounds, gpui::hsla(0.0, 0.0, 1.0, 1.0)).corner_radii(self.thumb_radius),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KEYBOARD_STEP;
    use core::prelude::v1::test;
    use gpui::{Bounds, px};

    /// Helper struct for pure logic tests that don't require GPUI context.
    struct TestSlider {
        value: f32,
        step_count: Option<usize>,
        last_bounds: Option<Bounds<gpui::Pixels>>,
    }

    impl TestSlider {
        fn new() -> Self {
            Self {
                value: 0.0,
                step_count: None,
                last_bounds: None,
            }
        }

        fn snap_value(&self, raw: f32) -> f32 {
            match self.step_count {
                Some(count) if count >= 2 => {
                    let steps = (count - 1) as f32;
                    (raw * steps).round() / steps
                }
                _ => raw,
            }
        }

        fn value_from_position(&self, x: gpui::Pixels) -> f32 {
            let Some(bounds) = self.last_bounds else {
                return self.value;
            };
            let relative_x = x - bounds.left();
            let width = bounds.size.width;
            if f32::from(width) <= 0.0 {
                return 0.0;
            }
            (f32::from(relative_x) / f32::from(width)).clamp(0.0, 1.0)
        }
    }

    #[test]
    fn test_slider_value_clamp() {
        let slider = TestSlider::new();
        assert_eq!(slider.value_from_position(px(100.0)), 0.0);
    }

    #[test]
    fn snap_continuous_no_change() {
        let slider = TestSlider::new();
        assert!((slider.snap_value(0.33) - 0.33).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_stepped_5_steps() {
        let mut slider = TestSlider::new();
        slider.step_count = Some(5);
        assert!((slider.snap_value(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.12) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.13) - 0.25).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.5) - 0.5).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.88) - 1.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_stepped_2_steps() {
        let mut slider = TestSlider::new();
        slider.step_count = Some(2);
        assert!((slider.snap_value(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.49) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.51) - 1.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_stepped_3_steps() {
        let mut slider = TestSlider::new();
        slider.step_count = Some(3);
        assert!((slider.snap_value(0.24) - 0.0).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.26) - 0.5).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.74) - 0.5).abs() < f32::EPSILON);
        assert!((slider.snap_value(0.76) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_count_less_than_2_is_continuous() {
        let mut slider = TestSlider::new();
        slider.step_count = Some(1);
        assert!((slider.snap_value(0.33) - 0.33).abs() < f32::EPSILON);
        slider.step_count = Some(0);
        assert!((slider.snap_value(0.33) - 0.33).abs() < f32::EPSILON);
    }

    #[test]
    fn keyboard_step_is_one_percent() {
        assert!((KEYBOARD_STEP - 0.01).abs() < f32::EPSILON);
    }

    #[test]
    fn keyboard_step_for_stepped_slider() {
        // 5 steps -> step of 0.25
        let step = 1.0 / (5 - 1) as f32;
        assert!((step - 0.25).abs() < f32::EPSILON);
    }

    /// Vertical orientation defaults to Horizontal when not set.
    #[test]
    fn default_orientation_is_horizontal() {
        use super::SliderOrientation;
        assert_eq!(SliderOrientation::default(), SliderOrientation::Horizontal);
    }

    /// Circular slider TODO note must stay present — if someone lands
    /// circular support they'll also need to drop or update this
    /// reminder, keeping the two in lockstep.
    #[test]
    fn circular_slider_todo_present() {
        const SELF_SRC: &str = include_str!("slider.rs");
        assert!(
            SELF_SRC.contains("TODO: circular slider"),
            "circular-slider TODO missing from slider.rs"
        );
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::TestAppContext;

    use super::Slider;
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    const SLIDER_TRACK: &str = "slider-track";

    fn focus_slider(slider: &gpui::Entity<Slider>, cx: &mut gpui::VisualTestContext) {
        slider.update_in(cx, |slider, window, cx| {
            slider.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn clicking_track_updates_slider_value(cx: &mut TestAppContext) {
        let (slider, cx) = setup_test_window(cx, |_window, cx| Slider::new(cx));

        cx.click_within(SLIDER_TRACK, 0.75, 0.5);

        slider.update_in(cx, |slider, _window, _cx| {
            assert!(
                (slider.value - 0.75).abs() < 0.05,
                "value was {}",
                slider.value
            );
        });
    }

    #[gpui::test]
    async fn dragging_track_updates_value_and_clears_drag_state(cx: &mut TestAppContext) {
        let (slider, cx) = setup_test_window(cx, |_window, cx| Slider::new(cx));

        cx.drag_within_x(SLIDER_TRACK, 0.2, 0.8);

        slider.update_in(cx, |slider, _window, _cx| {
            assert!(
                (slider.value - 0.8).abs() < 0.05,
                "value was {}",
                slider.value
            );
            assert!(!slider.is_dragging);
        });
    }

    #[gpui::test]
    async fn stepped_click_and_keyboard_controls_snap(cx: &mut TestAppContext) {
        let (slider, cx) = setup_test_window(cx, |_window, cx| Slider::new(cx));

        slider.update_in(cx, |slider, _window, _cx| {
            slider.set_step_count(5);
        });
        cx.click_within(SLIDER_TRACK, 0.14, 0.5);
        slider.update_in(cx, |slider, _window, _cx| {
            assert!((slider.value - 0.25).abs() < f32::EPSILON);
        });

        focus_slider(&slider, cx);
        cx.press("right");
        cx.press("end");
        slider.update_in(cx, |slider, _window, _cx| {
            assert!((slider.value - 1.0).abs() < f32::EPSILON);
        });

        cx.press("home");
        slider.update_in(cx, |slider, _window, _cx| {
            assert!((slider.value - 0.0).abs() < f32::EPSILON);
        });
    }

    /// Range mode: clicking closer to the low thumb moves it; clicking
    /// closer to the high thumb moves it. HIG NSSlider double-cell
    /// behaviour.
    #[gpui::test]
    async fn range_mode_click_moves_nearest_thumb(cx: &mut TestAppContext) {
        let (slider, cx) = setup_test_window(cx, |_window, cx| Slider::new(cx));

        slider.update_in(cx, |slider, _window, cx| {
            slider.set_range_mode(true);
            slider.set_range(0.25, 0.75, cx);
        });

        // Click at 0.1 — nearer to the low thumb (0.25).
        cx.click_within(SLIDER_TRACK, 0.1, 0.5);
        slider.update_in(cx, |slider, _window, _cx| {
            let (lo, hi) = slider.range();
            assert!(lo < 0.25, "low should have moved down; got {lo}");
            assert!(
                (hi - 0.75).abs() < 0.05,
                "high thumb should be untouched; got {hi}"
            );
        });

        // Click at 0.9 — nearer to the high thumb.
        cx.click_within(SLIDER_TRACK, 0.9, 0.5);
        slider.update_in(cx, |slider, _window, _cx| {
            let (_lo, hi) = slider.range();
            assert!(hi > 0.8, "high thumb should have moved up; got {hi}");
        });
    }

    /// Range-mode callback is invoked with the full `(low, high)` pair
    /// on every committed change.
    #[gpui::test]
    async fn range_mode_callback_receives_both_endpoints(cx: &mut TestAppContext) {
        use std::cell::RefCell;
        use std::rc::Rc;

        let (slider, cx) = setup_test_window(cx, |_window, cx| Slider::new(cx));
        let seen: Rc<RefCell<Vec<(f32, f32)>>> = Rc::new(RefCell::new(Vec::new()));

        slider.update_in(cx, |slider, _window, cx| {
            slider.set_range_mode(true);
            slider.set_range(0.2, 0.8, cx);
            let sink = seen.clone();
            slider.set_on_change_range(move |lo, hi, _, _| sink.borrow_mut().push((lo, hi)));
        });

        cx.click_within(SLIDER_TRACK, 0.95, 0.5);
        assert!(!seen.borrow().is_empty(), "no range callback fired");
        let last = *seen.borrow().last().unwrap();
        assert!(last.1 > last.0, "high must stay >= low, got {last:?}");
    }
}
