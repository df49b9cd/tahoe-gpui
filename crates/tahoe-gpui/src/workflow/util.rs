//! Shared utilities for workflow graph rendering.

/// Handle exit direction on a node.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HandlePosition {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// Evaluate a cubic bezier curve at parameter t.
pub(super) fn cubic_bezier(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let inv = 1.0 - t;
    let inv2 = inv * inv;
    let inv3 = inv2 * inv;
    let t2 = t * t;
    let t3 = t2 * t;
    (
        inv3 * p0.0 + 3.0 * inv2 * t * p1.0 + 3.0 * inv * t2 * p2.0 + t3 * p3.0,
        inv3 * p0.1 + 3.0 * inv2 * t * p1.1 + 3.0 * inv * t2 * p2.1 + t3 * p3.1,
    )
}

/// Evaluate a quadratic bezier curve at parameter t.
pub(super) fn quadratic_bezier(
    p0: (f32, f32),
    ctrl: (f32, f32),
    p2: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let inv = 1.0 - t;
    let inv2 = inv * inv;
    let t2 = t * t;
    (
        inv2 * p0.0 + 2.0 * inv * t * ctrl.0 + t2 * p2.0,
        inv2 * p0.1 + 2.0 * inv * t * ctrl.1 + t2 * p2.1,
    )
}

/// Default curvature factor matching upstream React Flow `getBezierPath`.
const DEFAULT_CURVATURE: f32 = 0.25;

/// Compute the control-point offset for a given signed distance.
///
/// Matches upstream React Flow `calculateControlOffset`:
/// - When the target is "ahead" of the source (`distance >= 0`), offset = `0.5 * distance`.
/// - When the target is "behind" (`distance < 0`, loopback), offset grows
///   as `curvature * 25 * sqrt(-distance)` to push the curve outward.
fn calculate_control_offset(distance: f32, curvature: f32) -> f32 {
    if distance >= 0.0 {
        0.5 * distance
    } else {
        curvature * 25.0 * (-distance).sqrt()
    }
}

/// Compute a control point extending from `(x1, y1)` in the direction of
/// `pos`, using `(x2, y2)` as the opposing point.
///
/// Matches upstream React Flow `getControlWithCurvature`.
fn control_with_curvature(
    pos: HandlePosition,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    curvature: f32,
) -> (f32, f32) {
    match pos {
        HandlePosition::Left => (x1 - calculate_control_offset(x1 - x2, curvature), y1),
        HandlePosition::Right => (x1 + calculate_control_offset(x2 - x1, curvature), y1),
        HandlePosition::Top => (x1, y1 - calculate_control_offset(y1 - y2, curvature)),
        HandlePosition::Bottom => (x1, y1 + calculate_control_offset(y2 - y1, curvature)),
    }
}

/// Compute cubic bezier control points matching upstream React Flow `getBezierPath`.
///
/// Uses curvature-based offsets that handle loopback edges gracefully.
/// Returns `(ctrl1, ctrl2)` for a cubic bezier from `from` to `to`.
pub(super) fn compute_bezier_control_points(
    from: (f32, f32),
    to: (f32, f32),
    source_pos: HandlePosition,
    target_pos: HandlePosition,
) -> ((f32, f32), (f32, f32)) {
    let ctrl1 = control_with_curvature(source_pos, from.0, from.1, to.0, to.1, DEFAULT_CURVATURE);
    let ctrl2 = control_with_curvature(target_pos, to.0, to.1, from.0, from.1, DEFAULT_CURVATURE);
    (ctrl1, ctrl2)
}

/// Compute cubic bezier control points matching upstream React Flow `getSimpleBezierPath`.
///
/// Uses midpoint-based control points for a simpler curve shape.
/// Returns `(ctrl1, ctrl2)` for a cubic bezier from `from` to `to`.
pub(super) fn compute_simple_control_points(
    from: (f32, f32),
    to: (f32, f32),
    source_pos: HandlePosition,
    target_pos: HandlePosition,
) -> ((f32, f32), (f32, f32)) {
    let ctrl1 = match source_pos {
        HandlePosition::Left | HandlePosition::Right => (0.5 * (from.0 + to.0), from.1),
        HandlePosition::Top | HandlePosition::Bottom => (from.0, 0.5 * (from.1 + to.1)),
    };
    let ctrl2 = match target_pos {
        HandlePosition::Left | HandlePosition::Right => (0.5 * (from.0 + to.0), to.1),
        HandlePosition::Top | HandlePosition::Bottom => (to.0, 0.5 * (from.1 + to.1)),
    };
    (ctrl1, ctrl2)
}

/// Compute S-curve control points for a connection line between two positions.
///
/// Returns `(ctrl1, ctrl2)` where `ctrl1 = (mid_x, from.1)` and
/// `ctrl2 = (mid_x, to.1)`, matching the upstream AI SDK Elements formula:
/// `C (fromX + (toX-fromX)*0.5, fromY) (fromX + (toX-fromX)*0.5, toY) (toX, toY)`
#[cfg(test)]
pub(super) fn connection_control_points(
    from: (f32, f32),
    to: (f32, f32),
) -> ((f32, f32), (f32, f32)) {
    let ctrl_x = (from.0 + to.0) / 2.0;
    ((ctrl_x, from.1), (ctrl_x, to.1))
}

/// Minimum distance from a point to a cubic bezier curve, sampled at `segments` points.
pub(super) fn point_to_cubic_bezier_distance(
    point: (f32, f32),
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    segments: u32,
) -> f32 {
    let segments = segments.max(1);
    let mut min_dist_sq = f32::MAX;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let bp = cubic_bezier(p0, p1, p2, p3, t);
        let dx = point.0 - bp.0;
        let dy = point.1 - bp.1;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
        }
    }
    min_dist_sq.sqrt()
}

/// Minimum distance from a point to a quadratic bezier curve, sampled at `segments` points.
pub(super) fn point_to_quadratic_bezier_distance(
    point: (f32, f32),
    p0: (f32, f32),
    ctrl: (f32, f32),
    p2: (f32, f32),
    segments: u32,
) -> f32 {
    let segments = segments.max(1);
    let mut min_dist_sq = f32::MAX;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let bp = quadratic_bezier(p0, ctrl, p2, t);
        let dx = point.0 - bp.0;
        let dy = point.1 - bp.1;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
        }
    }
    min_dist_sq.sqrt()
}

#[cfg(test)]
mod tests {
    use super::{
        HandlePosition, compute_bezier_control_points, compute_simple_control_points, cubic_bezier,
        point_to_cubic_bezier_distance, point_to_quadratic_bezier_distance,
    };
    use core::prelude::v1::test;

    #[test]
    fn cubic_bezier_at_t0_returns_start() {
        let start = (0.0, 0.0);
        let result = cubic_bezier(start, (1.0, 2.0), (3.0, 4.0), (5.0, 6.0), 0.0);
        assert!((result.0 - start.0).abs() < f32::EPSILON);
        assert!((result.1 - start.1).abs() < f32::EPSILON);
    }

    #[test]
    fn cubic_bezier_at_t1_returns_end() {
        let end = (5.0, 6.0);
        let result = cubic_bezier((0.0, 0.0), (1.0, 2.0), (3.0, 4.0), end, 1.0);
        assert!((result.0 - end.0).abs() < f32::EPSILON);
        assert!((result.1 - end.1).abs() < f32::EPSILON);
    }

    #[test]
    fn cubic_bezier_midpoint_on_straight_line() {
        // For a straight line (all control points collinear), midpoint should be center
        let result = cubic_bezier((0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0), 0.5);
        assert!((result.0 - 15.0).abs() < 0.01);
        assert!((result.1 - 0.0).abs() < 0.01);
    }

    // ── compute_bezier_control_points tests ──────────────────────────

    #[test]
    fn bezier_control_right_to_left_forward() {
        // Target is to the right of source (forward case)
        let from = (0.0, 100.0);
        let to = (200.0, 100.0);
        let (c1, c2) =
            compute_bezier_control_points(from, to, HandlePosition::Right, HandlePosition::Left);
        // ctrl1 extends right: offset = 0.5 * (200 - 0) = 100
        assert!((c1.0 - 100.0).abs() < f32::EPSILON);
        assert!((c1.1 - 100.0).abs() < f32::EPSILON);
        // ctrl2 extends left: offset = 0.5 * (200 - 0) = 100
        assert!((c2.0 - 100.0).abs() < f32::EPSILON);
        assert!((c2.1 - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bezier_control_right_to_left_loopback() {
        // Target is to the LEFT of source (loopback case)
        let from = (200.0, 100.0);
        let to = (0.0, 100.0);
        let (c1, c2) =
            compute_bezier_control_points(from, to, HandlePosition::Right, HandlePosition::Left);
        // distance for source Right = to.x - from.x = -200, negative → loopback
        // offset = 0.25 * 25 * sqrt(200) ≈ 88.39
        assert!(c1.0 > from.0, "ctrl1 extends right of source");
        assert!((c1.1 - from.1).abs() < f32::EPSILON, "ctrl1 Y unchanged");
        // distance for target Left = to.x - from.x = 0 - 200 = -200, but
        // target uses swapped args: x1=to.x, x2=from.x → distance = to.x - from.x = -200 → loopback
        assert!(c2.0 < to.0, "ctrl2 extends left of target");
        assert!((c2.1 - to.1).abs() < f32::EPSILON, "ctrl2 Y unchanged");
    }

    #[test]
    fn bezier_control_top_to_bottom() {
        let from = (100.0, 0.0);
        let to = (100.0, 200.0);
        let (c1, c2) =
            compute_bezier_control_points(from, to, HandlePosition::Top, HandlePosition::Bottom);
        assert!(c1.1 < from.1, "ctrl1 extends above source");
        assert!((c1.0 - from.0).abs() < f32::EPSILON, "ctrl1 X unchanged");
        assert!(c2.1 > to.1, "ctrl2 extends below target");
        assert!((c2.0 - to.0).abs() < f32::EPSILON, "ctrl2 X unchanged");
    }

    #[test]
    fn bezier_control_mixed_directions() {
        let from = (0.0, 0.0);
        let to = (200.0, 200.0);
        let (c1, c2) =
            compute_bezier_control_points(from, to, HandlePosition::Right, HandlePosition::Top);
        assert!(c1.0 > from.0, "ctrl1 extends right");
        assert!((c1.1 - from.1).abs() < f32::EPSILON, "ctrl1 Y unchanged");
        assert!(c2.1 < to.1, "ctrl2 extends upward");
        assert!((c2.0 - to.0).abs() < f32::EPSILON, "ctrl2 X unchanged");
    }

    #[test]
    fn bezier_control_exact_forward_values() {
        // Verify exact offset calculation: distance=200, offset = 0.5 * 200 = 100
        let from = (50.0, 50.0);
        let to = (250.0, 50.0);
        let (c1, c2) =
            compute_bezier_control_points(from, to, HandlePosition::Right, HandlePosition::Left);
        assert!(
            (c1.0 - 150.0).abs() < f32::EPSILON,
            "source ctrl x = 50 + 100 = 150"
        );
        assert!((c1.1 - 50.0).abs() < f32::EPSILON);
        assert!(
            (c2.0 - 150.0).abs() < f32::EPSILON,
            "target ctrl x = 250 - 100 = 150"
        );
        assert!((c2.1 - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bezier_control_loopback_uses_sqrt_formula() {
        // Loopback: source Right, target Left, but target is behind source
        let from = (100.0, 0.0);
        let to = (0.0, 0.0);
        let (c1, _c2) =
            compute_bezier_control_points(from, to, HandlePosition::Right, HandlePosition::Left);
        // distance for Right = to.x - from.x = -100 → offset = 0.25 * 25 * sqrt(100) = 62.5
        let expected = 100.0 + 62.5;
        assert!(
            (c1.0 - expected).abs() < 0.01,
            "loopback offset should use sqrt formula"
        );
    }

    // ── compute_simple_control_points tests ──────────────────────────

    #[test]
    fn simple_control_right_to_left() {
        let from = (0.0, 100.0);
        let to = (200.0, 300.0);
        let (c1, c2) =
            compute_simple_control_points(from, to, HandlePosition::Right, HandlePosition::Left);
        // Left/Right → ctrl.x = midpoint of x, ctrl.y = own y
        assert!((c1.0 - 100.0).abs() < f32::EPSILON, "ctrl1.x = midpoint x");
        assert!((c1.1 - 100.0).abs() < f32::EPSILON, "ctrl1.y = from.y");
        assert!((c2.0 - 100.0).abs() < f32::EPSILON, "ctrl2.x = midpoint x");
        assert!((c2.1 - 300.0).abs() < f32::EPSILON, "ctrl2.y = to.y");
    }

    #[test]
    fn simple_control_top_to_bottom() {
        let from = (100.0, 0.0);
        let to = (300.0, 200.0);
        let (c1, c2) =
            compute_simple_control_points(from, to, HandlePosition::Top, HandlePosition::Bottom);
        // Top/Bottom → ctrl.x = own x, ctrl.y = midpoint of y
        assert!((c1.0 - 100.0).abs() < f32::EPSILON, "ctrl1.x = from.x");
        assert!((c1.1 - 100.0).abs() < f32::EPSILON, "ctrl1.y = midpoint y");
        assert!((c2.0 - 300.0).abs() < f32::EPSILON, "ctrl2.x = to.x");
        assert!((c2.1 - 100.0).abs() < f32::EPSILON, "ctrl2.y = midpoint y");
    }

    #[test]
    fn simple_control_mixed_directions() {
        let from = (0.0, 0.0);
        let to = (200.0, 200.0);
        let (c1, c2) =
            compute_simple_control_points(from, to, HandlePosition::Right, HandlePosition::Top);
        // Right → ctrl1 = (midpoint x, from.y)
        assert!((c1.0 - 100.0).abs() < f32::EPSILON);
        assert!((c1.1 - 0.0).abs() < f32::EPSILON);
        // Top → ctrl2 = (to.x, midpoint y)
        assert!((c2.0 - 200.0).abs() < f32::EPSILON);
        assert!((c2.1 - 100.0).abs() < f32::EPSILON);
    }

    // ── Distance function tests ──────────────────────────────────────

    #[test]
    fn cubic_distance_on_curve_is_zero() {
        // Point exactly on the start of the curve should have ~0 distance.
        let dist = point_to_cubic_bezier_distance(
            (0.0, 0.0),
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (30.0, 0.0),
            48,
        );
        assert!(dist < 0.01);
    }

    #[test]
    fn cubic_distance_far_point() {
        let dist = point_to_cubic_bezier_distance(
            (0.0, 100.0),
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (30.0, 0.0),
            48,
        );
        assert!(dist > 90.0);
    }

    #[test]
    fn cubic_distance_midpoint_near_curve() {
        // Straight-line bezier from (0,0) to (100,0). Point at (50, 3) should be ~3px away.
        let dist = point_to_cubic_bezier_distance(
            (50.0, 3.0),
            (0.0, 0.0),
            (33.0, 0.0),
            (66.0, 0.0),
            (100.0, 0.0),
            48,
        );
        assert!((dist - 3.0).abs() < 0.5);
    }

    #[test]
    fn quadratic_distance_on_curve_is_zero() {
        let dist =
            point_to_quadratic_bezier_distance((0.0, 0.0), (0.0, 0.0), (5.0, 0.0), (10.0, 0.0), 48);
        assert!(dist < 0.01);
    }

    #[test]
    fn quadratic_distance_far_point() {
        let dist = point_to_quadratic_bezier_distance(
            (5.0, 50.0),
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            48,
        );
        assert!(dist > 40.0);
    }

    #[test]
    fn cubic_distance_segments_zero() {
        // segments=0 should not panic or produce NaN/infinity.
        let dist = point_to_cubic_bezier_distance(
            (5.0, 5.0),
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (30.0, 0.0),
            0,
        );
        assert!(dist.is_finite());
        assert!(dist >= 0.0);
    }

    #[test]
    fn quadratic_distance_segments_zero() {
        // segments=0 should not panic or produce NaN/infinity.
        let dist =
            point_to_quadratic_bezier_distance((5.0, 5.0), (0.0, 0.0), (5.0, 0.0), (10.0, 0.0), 0);
        assert!(dist.is_finite());
        assert!(dist >= 0.0);
    }
}
