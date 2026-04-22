//! Chart annotations — captions, callouts, and icons anchored to a mark
//! or a `(x, y)` value on the plot.
//!
//! Mirrors Swift Charts' `.annotation()` surface: attach content to a
//! data point (by series/point index) or a raw `(x, y)` value, pick a
//! side (`Top`, `Bottom`, `Leading`, `Trailing`, `Overlay`), and let the
//! render path absolute-position the element inside the plot area.
//!
//! Overflow resolution follows Swift Charts'
//! [`AnnotationOverflowResolution`](https://developer.apple.com/documentation/charts/annotationoverflowresolution):
//! when the anchor would place the annotation outside the plot area, flip
//! to the opposite side so the caption never sits under the chart's
//! rounded clip edge.
//!
//! Content is intentionally narrow — `Text` and `Icon` — because `Chart`
//! is `RenderOnce` and cannot carry arbitrary `AnyElement` children
//! through the builder pattern without interior mutability. Callers who
//! need richer content can stack the underlying chart inside their own
//! view and compose their own overlay.

use gpui::SharedString;

use crate::foundations::icons::IconName;

use super::types::PlottableValue;

/// Where the annotation sits relative to the mark it targets.
///
/// `Top` places the annotation above the data point, `Bottom` below,
/// `Leading`/`Trailing` to the left/right, and `Overlay` centres it on
/// the mark itself. Overflow resolution flips `Top` ↔ `Bottom` and
/// `Leading` ↔ `Trailing` when the annotation would land outside the
/// plot area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnnotationPosition {
    /// Above the mark.
    #[default]
    Top,
    /// Below the mark.
    Bottom,
    /// To the left of the mark (reading order — in RTL this is the
    /// right side, but the chart's coordinate space is always
    /// left-to-right, so the annotation still lands on the low-X side).
    Leading,
    /// To the right of the mark.
    Trailing,
    /// Centred on the mark.
    Overlay,
}

/// Which mark or value the annotation is pinned to.
#[derive(Debug, Clone)]
pub enum AnnotationTarget {
    /// A specific `(series, point)` in the chart's data set.
    DataPoint {
        /// Zero-based series index.
        series_idx: usize,
        /// Zero-based point index within that series.
        point_idx: usize,
    },
    /// A raw `(x, y)` coordinate — projected through the chart's scales
    /// the same way a data point would be. Useful for annotating a
    /// value that isn't in the data set (e.g. a target threshold).
    Value {
        /// X position of the anchor.
        x: PlottableValue,
        /// Y position of the anchor.
        y: PlottableValue,
    },
}

/// The body of the annotation.
#[derive(Debug, Clone)]
pub enum AnnotationContent {
    /// A short text caption — rendered in the theme's Caption1 style.
    Text(SharedString),
    /// A symbol — rendered at the theme's default icon size.
    Icon(IconName),
}

/// A single annotation on the chart.
///
/// Use the [`ChartAnnotation::text`] and [`ChartAnnotation::icon`]
/// constructors rather than building the struct literally so additional
/// fields (tooltip body, role overrides, callout line) can be added
/// without breaking callers.
#[derive(Debug, Clone)]
pub struct ChartAnnotation {
    /// The mark or value the annotation is pinned to.
    pub target: AnnotationTarget,
    /// Where the annotation sits relative to [`target`](Self::target).
    pub position: AnnotationPosition,
    /// The annotation's visible body.
    pub content: AnnotationContent,
}

impl ChartAnnotation {
    /// Create a text annotation.
    pub fn text(
        target: AnnotationTarget,
        position: AnnotationPosition,
        text: impl Into<SharedString>,
    ) -> Self {
        Self {
            target,
            position,
            content: AnnotationContent::Text(text.into()),
        }
    }

    /// Create an icon annotation.
    pub fn icon(target: AnnotationTarget, position: AnnotationPosition, icon: IconName) -> Self {
        Self {
            target,
            position,
            content: AnnotationContent::Icon(icon),
        }
    }
}

impl AnnotationPosition {
    /// Flip top↔bottom / leading↔trailing (overflow-aware placement).
    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Leading => Self::Trailing,
            Self::Trailing => Self::Leading,
            Self::Overlay => Self::Overlay,
        }
    }
}

/// Resolve the annotation's anchor point and effective position.
///
/// Returns `(anchor_x_px, anchor_y_px, effective_position)` in
/// plot-area pixel coordinates, or `None` when the target cannot be
/// located (missing series, out-of-range index).
///
/// Overflow resolution is deliberately coarse: if the requested side
/// would place the annotation within `flip_threshold` pixels of the
/// matching edge, flip to the opposite side. This does not know the
/// annotation's rendered size — it's a heuristic equivalent to Swift
/// Charts' "if the mark is near the top edge, flip `Top` to
/// `Bottom`". Callers who need pixel-perfect overflow handling can
/// pre-compute the correct side themselves.
pub(crate) fn resolve_annotation(
    annotation: &ChartAnnotation,
    data_set: &super::types::ChartDataSet,
    x_scale: &dyn super::scales::Scale,
    y_scale: &dyn super::scales::Scale,
    plot_width: f32,
    plot_height: f32,
    flip_threshold: f32,
) -> Option<(f32, f32, AnnotationPosition)> {
    let (x_val, y_val) = match &annotation.target {
        AnnotationTarget::DataPoint {
            series_idx,
            point_idx,
        } => {
            let series = data_set.series.get(*series_idx)?;
            let point = series.inner.points.get(*point_idx)?;
            (point.x.clone(), point.y.clone())
        }
        AnnotationTarget::Value { x, y } => (x.clone(), y.clone()),
    };
    let anchor_x = plot_width * x_scale.project(&x_val);
    // `project` returns 0..1 where 0 = domain_lo (bottom of plot) and 1
    // = domain_hi (top), so flip for pixel Y.
    let anchor_y = plot_height * (1.0 - y_scale.project(&y_val));

    let position = match annotation.position {
        AnnotationPosition::Top if anchor_y < flip_threshold => AnnotationPosition::Top.flipped(),
        AnnotationPosition::Bottom if anchor_y > plot_height - flip_threshold => {
            AnnotationPosition::Bottom.flipped()
        }
        AnnotationPosition::Leading if anchor_x < flip_threshold => {
            AnnotationPosition::Leading.flipped()
        }
        AnnotationPosition::Trailing if anchor_x > plot_width - flip_threshold => {
            AnnotationPosition::Trailing.flipped()
        }
        other => other,
    };

    Some((anchor_x, anchor_y, position))
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use std::sync::Arc;

    use super::super::scales::LinearScale;
    use super::super::types::{ChartDataSeries, ChartDataSet, ChartSeries};
    use super::*;

    fn unit_scale() -> Arc<dyn super::super::scales::Scale> {
        Arc::new(LinearScale::new(0.0, 1.0))
    }

    fn one_series_set() -> ChartDataSet {
        ChartDataSet::single(ChartDataSeries::new("s", vec![0.25, 0.5, 0.75]))
    }

    #[test]
    fn position_flip_is_involutive() {
        assert_eq!(
            AnnotationPosition::Top.flipped(),
            AnnotationPosition::Bottom
        );
        assert_eq!(
            AnnotationPosition::Bottom.flipped(),
            AnnotationPosition::Top
        );
        assert_eq!(
            AnnotationPosition::Leading.flipped(),
            AnnotationPosition::Trailing
        );
        assert_eq!(
            AnnotationPosition::Trailing.flipped(),
            AnnotationPosition::Leading
        );
        // Overlay is fixed — no opposite side.
        assert_eq!(
            AnnotationPosition::Overlay.flipped(),
            AnnotationPosition::Overlay
        );
    }

    #[test]
    fn resolve_data_point_anchors_inside_plot() {
        // X scale runs 0..2 (point index), Y scale runs 0..1. Point 1
        // (y = 0.5) should land at half the plot height with the Top
        // position unchanged (well away from the edge).
        let set = one_series_set();
        let x_scale: Arc<dyn super::super::scales::Scale> = Arc::new(LinearScale::new(0.0, 2.0));
        let y_scale = unit_scale();
        let annotation = ChartAnnotation::text(
            AnnotationTarget::DataPoint {
                series_idx: 0,
                point_idx: 1,
            },
            AnnotationPosition::Top,
            "peak",
        );
        let (ax, ay, pos) = resolve_annotation(
            &annotation,
            &set,
            x_scale.as_ref(),
            y_scale.as_ref(),
            200.0,
            100.0,
            10.0,
        )
        .expect("point 1 exists");
        assert!((ax - 100.0).abs() < 1e-3, "ax = {ax}");
        assert!((ay - 50.0).abs() < 1e-3, "ay = {ay}");
        assert_eq!(pos, AnnotationPosition::Top);
    }

    #[test]
    fn resolve_flips_top_to_bottom_at_upper_edge() {
        // y = 0.99 lands within the 10 px flip threshold of the top.
        let set = ChartDataSet::single(ChartDataSeries::new("s", vec![0.99]));
        let x_scale: Arc<dyn super::super::scales::Scale> = Arc::new(LinearScale::new(0.0, 1.0));
        let y_scale = unit_scale();
        let annotation = ChartAnnotation::text(
            AnnotationTarget::DataPoint {
                series_idx: 0,
                point_idx: 0,
            },
            AnnotationPosition::Top,
            "tight",
        );
        let (_, _, pos) = resolve_annotation(
            &annotation,
            &set,
            x_scale.as_ref(),
            y_scale.as_ref(),
            100.0,
            100.0,
            5.0,
        )
        .unwrap();
        assert_eq!(pos, AnnotationPosition::Bottom);
    }

    #[test]
    fn resolve_value_target_bypasses_data_lookup() {
        let set = ChartDataSet::single(ChartDataSeries::new("s", vec![]));
        let x_scale: Arc<dyn super::super::scales::Scale> = Arc::new(LinearScale::new(0.0, 1.0));
        let y_scale = unit_scale();
        let annotation = ChartAnnotation::text(
            AnnotationTarget::Value {
                x: PlottableValue::Number(0.5),
                y: PlottableValue::Number(0.5),
            },
            AnnotationPosition::Overlay,
            "midpoint",
        );
        let (ax, ay, pos) = resolve_annotation(
            &annotation,
            &set,
            x_scale.as_ref(),
            y_scale.as_ref(),
            100.0,
            100.0,
            5.0,
        )
        .expect("value targets ignore data-set emptiness");
        assert!((ax - 50.0).abs() < 1e-3);
        assert!((ay - 50.0).abs() < 1e-3);
        assert_eq!(pos, AnnotationPosition::Overlay);
    }

    #[test]
    fn resolve_missing_data_point_returns_none() {
        let set = one_series_set();
        let x_scale: Arc<dyn super::super::scales::Scale> = Arc::new(LinearScale::new(0.0, 2.0));
        let y_scale = unit_scale();
        let annotation = ChartAnnotation::text(
            AnnotationTarget::DataPoint {
                series_idx: 0,
                point_idx: 99,
            },
            AnnotationPosition::Top,
            "missing",
        );
        let result = resolve_annotation(
            &annotation,
            &set,
            x_scale.as_ref(),
            y_scale.as_ref(),
            100.0,
            100.0,
            10.0,
        );
        assert!(result.is_none());

        let annotation_series_oob = ChartAnnotation::text(
            AnnotationTarget::DataPoint {
                series_idx: 5,
                point_idx: 0,
            },
            AnnotationPosition::Top,
            "no series",
        );
        let result = resolve_annotation(
            &annotation_series_oob,
            &set,
            x_scale.as_ref(),
            y_scale.as_ref(),
            100.0,
            100.0,
            10.0,
        );
        assert!(result.is_none());

        // Guardrail — make sure `ChartSeries::new` path isn't panicking
        // when callers assemble the data set through the richer builder.
        let _probe =
            ChartDataSet::multi(vec![ChartSeries::new(ChartDataSeries::new("a", vec![1.0]))]);
    }
}
