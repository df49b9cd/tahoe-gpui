//! Viewport state and zoom/pan calculations for the workflow canvas.

/// Compute zoom level and pan offset to fit content within a viewport.
///
/// Returns `None` if the content bounds have zero or negative extent.
pub(super) fn compute_fit_zoom_and_center(
    bounds: (f32, f32, f32, f32), // (min_x, min_y, max_x, max_y)
    viewport: (f32, f32),         // (width, height)
    opts: &super::super::controls::FitViewOptions,
) -> Option<(f32, (f32, f32))> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let (viewport_w, viewport_h) = viewport;

    let content_w = max_x - min_x;
    let content_h = max_y - min_y;
    if content_w <= 0.0 || content_h <= 0.0 {
        return None;
    }

    let avail_w = (viewport_w - 2.0 * opts.padding).max(1.0);
    let avail_h = (viewport_h - 2.0 * opts.padding).max(1.0);

    let target_zoom = (avail_w / content_w).min(avail_h / content_h);

    // Sanitize zoom bounds: handle NaN and inverted min/max.
    let min_zoom = if opts.min_zoom.is_nan() {
        f32::NEG_INFINITY
    } else {
        opts.min_zoom
    };
    let max_zoom = if opts.max_zoom.is_nan() {
        f32::INFINITY
    } else {
        opts.max_zoom
    };
    let (min_zoom, max_zoom) = if min_zoom > max_zoom {
        (max_zoom, min_zoom)
    } else {
        (min_zoom, max_zoom)
    };
    let new_zoom = target_zoom.max(min_zoom).min(max_zoom);

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let pan = (
        viewport_w / 2.0 - center_x * new_zoom,
        viewport_h / 2.0 - center_y * new_zoom,
    );

    Some((new_zoom, pan))
}
