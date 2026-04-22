//! Scroll / zoom configuration for [`Chart`] and [`ChartView`].
//!
//! Mirrors Swift Charts' `.chartScrollableAxes` + `.chartXVisibleDomain` +
//! `.chartScrollPosition` surface: the caller picks a narrower X window
//! than the full data extent, optionally anchors it with a scroll position,
//! and (on [`ChartView`]) uses the scroll-wheel / touchpad to advance that
//! window.
//!
//! Only numeric (`PlottableValue::Number`) bounds are honoured today. Date
//! and Category visible-domain support is a follow-up — the config type
//! keeps the full [`PlottableValue`] so the API doesn't need to widen
//! again when that lands.
//!
//! [`Chart`]: super::render::Chart
//! [`ChartView`]: super::view::ChartView
//! [`PlottableValue`]: super::types::PlottableValue

use super::types::PlottableValue;

/// Scroll / zoom configuration for a chart.
///
/// # Fields
///
/// - [`x_visible_domain`](Self::x_visible_domain) — narrow the X axis to a
///   sub-range of the full data extent. When set, only marks whose X falls
///   inside the domain render.
/// - [`x_scroll_position`](Self::x_scroll_position) — anchor the visible
///   domain so it starts at a specific X value. Combined with
///   [`ChartView`](super::view::ChartView)'s wheel handler, this is the
///   live scroll offset.
/// - [`y_scrollable`](Self::y_scrollable) — reserved flag for future
///   Y-axis scroll support. Defaults to `false`; setting it has no visual
///   effect yet.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartScrollConfig {
    /// The narrower X window to show. When `None`, the chart shows the full
    /// data extent. Bounds are inclusive.
    pub x_visible_domain: Option<(PlottableValue, PlottableValue)>,
    /// Where in the full data extent the visible window starts. When
    /// `None`, the window is anchored at the lower bound of
    /// [`x_visible_domain`](Self::x_visible_domain) (or the data minimum
    /// when that's also absent).
    pub x_scroll_position: Option<PlottableValue>,
    /// Whether the Y axis is scrollable. Reserved — has no effect yet.
    pub y_scrollable: bool,
}

impl ChartScrollConfig {
    /// Build an empty config. Equivalent to [`Default::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Narrow the visible X window to `(low, high)`.
    pub fn x_visible_domain(
        mut self,
        low: impl Into<PlottableValue>,
        high: impl Into<PlottableValue>,
    ) -> Self {
        self.x_visible_domain = Some((low.into(), high.into()));
        self
    }

    /// Anchor the visible window so it starts at `position`.
    pub fn x_scroll_position(mut self, position: impl Into<PlottableValue>) -> Self {
        self.x_scroll_position = Some(position.into());
        self
    }

    /// Flag the Y axis as scrollable. Reserved for future use.
    pub fn y_scrollable(mut self, scrollable: bool) -> Self {
        self.y_scrollable = scrollable;
        self
    }

    /// Resolve the effective numeric window as `(lo, hi)` given a data
    /// extent fallback. Returns `None` when neither the visible domain nor
    /// the scroll position is numeric (e.g. Date or Category variants that
    /// aren't supported yet).
    pub(crate) fn effective_numeric_window(&self, data_extent: (f64, f64)) -> Option<(f64, f64)> {
        let (d_lo, d_hi) = data_extent;
        // Pull the visible domain width from the config when present;
        // otherwise fall back to the full data width so scrolling alone
        // (without a zoom level) behaves like a no-op.
        let (win_lo, win_hi) = match &self.x_visible_domain {
            Some((lo, hi)) => (lo.as_number()?, hi.as_number()?),
            None => (d_lo, d_hi),
        };
        let width = (win_hi - win_lo).max(0.0);

        // Apply the scroll position (if any) by shifting the window so
        // its lower bound aligns with the requested start point. Clamp to
        // the data extent so the window never slides past the data.
        let (lo, hi) = match &self.x_scroll_position {
            Some(start) => {
                let start_num = start.as_number()?;
                let lo = start_num.clamp(d_lo, (d_hi - width).max(d_lo));
                (lo, lo + width)
            }
            None => (win_lo, win_hi),
        };
        Some((lo, hi))
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::*;

    #[test]
    fn default_config_has_no_window() {
        let c = ChartScrollConfig::default();
        assert!(c.x_visible_domain.is_none());
        assert!(c.x_scroll_position.is_none());
        assert!(!c.y_scrollable);
    }

    #[test]
    fn builders_set_each_field() {
        let c = ChartScrollConfig::new()
            .x_visible_domain(10.0, 30.0)
            .x_scroll_position(15.0)
            .y_scrollable(true);
        assert_eq!(
            c.x_visible_domain
                .as_ref()
                .and_then(|(lo, hi)| Some((lo.as_number()?, hi.as_number()?))),
            Some((10.0, 30.0))
        );
        assert_eq!(
            c.x_scroll_position.as_ref().and_then(|v| v.as_number()),
            Some(15.0)
        );
        assert!(c.y_scrollable);
    }

    #[test]
    fn effective_window_falls_back_to_data_extent() {
        let c = ChartScrollConfig::default();
        assert_eq!(c.effective_numeric_window((0.0, 100.0)), Some((0.0, 100.0)));
    }

    #[test]
    fn effective_window_honours_visible_domain() {
        let c = ChartScrollConfig::new().x_visible_domain(20.0, 40.0);
        assert_eq!(c.effective_numeric_window((0.0, 100.0)), Some((20.0, 40.0)));
    }

    #[test]
    fn effective_window_shifts_with_scroll_position() {
        let c = ChartScrollConfig::new()
            .x_visible_domain(0.0, 10.0)
            .x_scroll_position(50.0);
        assert_eq!(c.effective_numeric_window((0.0, 100.0)), Some((50.0, 60.0)));
    }

    #[test]
    fn effective_window_clamps_to_data_extent() {
        // With a 10-wide window and a scroll position at 95, the window
        // must clamp to (90, 100) rather than sliding past the data.
        let c = ChartScrollConfig::new()
            .x_visible_domain(0.0, 10.0)
            .x_scroll_position(95.0);
        assert_eq!(
            c.effective_numeric_window((0.0, 100.0)),
            Some((90.0, 100.0))
        );
    }

    #[test]
    fn effective_window_clamps_to_lower_bound() {
        let c = ChartScrollConfig::new()
            .x_visible_domain(0.0, 10.0)
            .x_scroll_position(-5.0);
        assert_eq!(c.effective_numeric_window((0.0, 100.0)), Some((0.0, 10.0)));
    }
}
