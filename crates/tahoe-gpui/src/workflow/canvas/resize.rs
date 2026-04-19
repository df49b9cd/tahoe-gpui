//! Resize-handle hit-testing and geometry (F28 from #149).
//!
//! Responsibilities kept strictly to "given a node rect and a pointer,
//! which of the 8 HIG handles is under it, and what does dragging it
//! produce?" — the canvas itself owns mutation and the undo stack. Keeping
//! this math pure makes the behaviour unit-testable without a `Context`.

use gpui::CursorStyle;

use super::super::node::{NODE_MIN_HEIGHT, NODE_MIN_WIDTH};

/// Screen-space radius around a handle's painted square that still counts
/// as a hit. The visual handle is 10 px across, so 9 px radius gives a
/// ~22 px target — the same 44 pt HIG minimum we apply to ports (F17).
pub(super) const HANDLE_HIT_RADIUS: f32 = 9.0;
/// Painted side-length of the handle square.
pub(super) const HANDLE_VISUAL_SIZE: f32 = 10.0;

/// Which of the eight resize handles is being manipulated.
///
/// Laid out clockwise starting at the top-left — rotating the slice into
/// any of these names gives the same handle the user sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl ResizeHandle {
    pub(super) const ALL: [ResizeHandle; 8] = [
        ResizeHandle::TopLeft,
        ResizeHandle::Top,
        ResizeHandle::TopRight,
        ResizeHandle::Right,
        ResizeHandle::BottomRight,
        ResizeHandle::Bottom,
        ResizeHandle::BottomLeft,
        ResizeHandle::Left,
    ];

    /// HIG-aligned pointer shape for the handle. TopLeft/BottomRight share
    /// a diagonal; TopRight/BottomLeft share the opposite diagonal; the
    /// axis-aligned pairs share their respective horizontal / vertical
    /// resize pointers.
    pub(super) fn cursor(self) -> CursorStyle {
        match self {
            ResizeHandle::TopLeft | ResizeHandle::BottomRight => {
                CursorStyle::ResizeUpLeftDownRight
            }
            ResizeHandle::TopRight | ResizeHandle::BottomLeft => {
                CursorStyle::ResizeUpRightDownLeft
            }
            ResizeHandle::Top | ResizeHandle::Bottom => CursorStyle::ResizeUpDown,
            ResizeHandle::Left | ResizeHandle::Right => CursorStyle::ResizeLeftRight,
        }
    }

    /// Screen-space centre of the handle given the node's screen rect.
    /// `(x, y, w, h)` is the node's top-left origin plus its dimensions.
    pub(super) fn centre(self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
        let mx = x + w / 2.0;
        let my = y + h / 2.0;
        let right = x + w;
        let bottom = y + h;
        match self {
            ResizeHandle::TopLeft => (x, y),
            ResizeHandle::Top => (mx, y),
            ResizeHandle::TopRight => (right, y),
            ResizeHandle::Right => (right, my),
            ResizeHandle::BottomRight => (right, bottom),
            ResizeHandle::Bottom => (mx, bottom),
            ResizeHandle::BottomLeft => (x, bottom),
            ResizeHandle::Left => (x, my),
        }
    }
}

/// Return the handle under `(mx, my)` for a node whose screen rect is
/// `(x, y, w, h)`, or `None` if the pointer is not within `HANDLE_HIT_RADIUS`
/// of any handle centre.
pub(super) fn handle_at(
    mx: f32,
    my: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Option<ResizeHandle> {
    // Check in reverse order so corner hits win over edge hits when the
    // pointer falls near an intersection — corners give better resize
    // control, so they're the friendlier default.
    for &handle in ResizeHandle::ALL.iter().rev() {
        let (cx, cy) = handle.centre(x, y, w, h);
        let dx = mx - cx;
        let dy = my - cy;
        if dx.hypot(dy) <= HANDLE_HIT_RADIUS {
            return Some(handle);
        }
    }
    None
}

/// Compute the new `(position, size)` produced by dragging `handle` from
/// `start_pos` / `start_size` by `(delta_x, delta_y)` world-space pixels.
///
/// Enforces `NODE_MIN_WIDTH` / `NODE_MIN_HEIGHT` — the handle effectively
/// "pins" once the minimum is reached, which matches Keynote / Freeform.
pub(super) fn apply_handle_delta(
    handle: ResizeHandle,
    start_pos: (f32, f32),
    start_size: (f32, f32),
    delta: (f32, f32),
) -> ((f32, f32), (f32, f32)) {
    let (mut x, mut y) = start_pos;
    let (mut w, mut h) = start_size;
    let (dx, dy) = delta;

    // Drives left-edge resizes: moving the edge right (positive dx) shrinks
    // the node and slides the origin. Mirror for right-edge, top, bottom.
    match handle {
        ResizeHandle::TopLeft => {
            let new_w = (w - dx).max(NODE_MIN_WIDTH);
            let new_h = (h - dy).max(NODE_MIN_HEIGHT);
            x += w - new_w;
            y += h - new_h;
            w = new_w;
            h = new_h;
        }
        ResizeHandle::Top => {
            let new_h = (h - dy).max(NODE_MIN_HEIGHT);
            y += h - new_h;
            h = new_h;
        }
        ResizeHandle::TopRight => {
            let new_h = (h - dy).max(NODE_MIN_HEIGHT);
            y += h - new_h;
            h = new_h;
            w = (w + dx).max(NODE_MIN_WIDTH);
        }
        ResizeHandle::Right => {
            w = (w + dx).max(NODE_MIN_WIDTH);
        }
        ResizeHandle::BottomRight => {
            w = (w + dx).max(NODE_MIN_WIDTH);
            h = (h + dy).max(NODE_MIN_HEIGHT);
        }
        ResizeHandle::Bottom => {
            h = (h + dy).max(NODE_MIN_HEIGHT);
        }
        ResizeHandle::BottomLeft => {
            let new_w = (w - dx).max(NODE_MIN_WIDTH);
            x += w - new_w;
            w = new_w;
            h = (h + dy).max(NODE_MIN_HEIGHT);
        }
        ResizeHandle::Left => {
            let new_w = (w - dx).max(NODE_MIN_WIDTH);
            x += w - new_w;
            w = new_w;
        }
    }

    ((x, y), (w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn bottom_right_grows_without_moving_origin() {
        let ((x, y), (w, h)) = apply_handle_delta(
            ResizeHandle::BottomRight,
            (100.0, 100.0),
            (400.0, 100.0),
            (50.0, 40.0),
        );
        assert_eq!((x, y), (100.0, 100.0));
        assert_eq!((w, h), (450.0, 140.0));
    }

    #[test]
    fn top_left_shrinks_and_moves_origin() {
        let ((x, y), (w, h)) = apply_handle_delta(
            ResizeHandle::TopLeft,
            (100.0, 100.0),
            (500.0, 200.0),
            (30.0, 20.0),
        );
        // Width goes 500 → 470, height 200 → 180, origin slides by
        // (500-470, 200-180) = (30, 20).
        assert_eq!((x, y), (130.0, 120.0));
        assert_eq!((w, h), (470.0, 180.0));
    }

    #[test]
    fn left_handle_does_not_change_height() {
        let ((x, y), (w, h)) = apply_handle_delta(
            ResizeHandle::Left,
            (100.0, 100.0),
            (500.0, 200.0),
            (10.0, 50.0),
        );
        assert_eq!(y, 100.0);
        assert_eq!(h, 200.0);
        assert_eq!(w, 490.0);
        assert_eq!(x, 110.0);
    }

    #[test]
    fn min_size_clamps_prevent_collapse() {
        let ((_, _), (w, h)) = apply_handle_delta(
            ResizeHandle::BottomRight,
            (0.0, 0.0),
            (400.0, 100.0),
            (-9999.0, -9999.0),
        );
        // Clamped at the minimums, not crushed to zero.
        assert_eq!(w, NODE_MIN_WIDTH);
        assert_eq!(h, NODE_MIN_HEIGHT);
    }

    #[test]
    fn top_left_clamp_does_not_run_origin_past_opposite_corner() {
        // Starting 400×100, moving TopLeft handle far down-right. The
        // origin should only slide by the actual reduction (i.e. stop
        // once the minimum size is hit).
        let ((x, y), (w, h)) = apply_handle_delta(
            ResizeHandle::TopLeft,
            (0.0, 0.0),
            (500.0, 200.0),
            (9999.0, 9999.0),
        );
        // Reduction = start - min = 500-384 = 116 for width; 200-80 = 120 for height.
        assert_eq!(w, NODE_MIN_WIDTH);
        assert_eq!(h, NODE_MIN_HEIGHT);
        assert_eq!(x, 500.0 - NODE_MIN_WIDTH);
        assert_eq!(y, 200.0 - NODE_MIN_HEIGHT);
    }

    #[test]
    fn handle_centre_matches_expected_geometry() {
        let x = 10.0;
        let y = 20.0;
        let w = 400.0;
        let h = 100.0;
        assert_eq!(ResizeHandle::TopLeft.centre(x, y, w, h), (10.0, 20.0));
        assert_eq!(ResizeHandle::Top.centre(x, y, w, h), (210.0, 20.0));
        assert_eq!(ResizeHandle::TopRight.centre(x, y, w, h), (410.0, 20.0));
        assert_eq!(ResizeHandle::Right.centre(x, y, w, h), (410.0, 70.0));
        assert_eq!(ResizeHandle::BottomRight.centre(x, y, w, h), (410.0, 120.0));
        assert_eq!(ResizeHandle::Bottom.centre(x, y, w, h), (210.0, 120.0));
        assert_eq!(ResizeHandle::BottomLeft.centre(x, y, w, h), (10.0, 120.0));
        assert_eq!(ResizeHandle::Left.centre(x, y, w, h), (10.0, 70.0));
    }

    #[test]
    fn handle_at_prefers_corner_over_edge() {
        // At the top-left corner the pointer is equally close to Top and
        // Left edges, but the TopLeft corner handle should win because
        // `handle_at` iterates the variant list in reverse so corners
        // (earlier in the list) take priority.
        let hit = handle_at(0.0, 0.0, 0.0, 0.0, 400.0, 100.0);
        assert_eq!(hit, Some(ResizeHandle::TopLeft));
    }

    #[test]
    fn handle_at_returns_none_when_far_from_any_handle() {
        assert!(handle_at(200.0, 50.0, 0.0, 0.0, 400.0, 100.0).is_none());
    }
}
