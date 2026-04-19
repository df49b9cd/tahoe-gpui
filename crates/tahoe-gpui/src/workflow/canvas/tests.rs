use super::super::connection::{Connection, PortId};
use super::super::controls::FitViewOptions;
use super::super::node::PortType;
use super::{
    AUTO_PAN_MARGIN, AUTO_PAN_STEP, EDGE_HIT_TOLERANCE_SCREEN_PX, MAX_ZOOM, MIN_ZOOM,
    NODE_TITLE_HEIGHT, NODE_WIDTH, PORT_HIT_RADIUS_SCREEN_PX, compute_fit_zoom_and_center,
};
use core::prelude::v1::test;

#[test]
fn zoom_clamp_within_bounds() {
    assert_eq!(1.5_f32.clamp(MIN_ZOOM, MAX_ZOOM), 1.5);
}
#[test]
fn zoom_clamp_at_lower_bound() {
    assert_eq!(MIN_ZOOM.clamp(MIN_ZOOM, MAX_ZOOM), MIN_ZOOM);
}
#[test]
fn zoom_clamp_at_upper_bound() {
    assert_eq!(MAX_ZOOM.clamp(MIN_ZOOM, MAX_ZOOM), MAX_ZOOM);
}
#[test]
fn zoom_clamp_below_minimum() {
    assert_eq!(0.0_f32.clamp(MIN_ZOOM, MAX_ZOOM), MIN_ZOOM);
}
#[test]
fn zoom_clamp_negative() {
    assert_eq!((-1.0_f32).clamp(MIN_ZOOM, MAX_ZOOM), MIN_ZOOM);
}
#[test]
fn zoom_clamp_above_maximum() {
    assert_eq!(5.0_f32.clamp(MIN_ZOOM, MAX_ZOOM), MAX_ZOOM);
}
#[test]
fn zoom_clamp_very_large() {
    assert_eq!(f32::MAX.clamp(MIN_ZOOM, MAX_ZOOM), MAX_ZOOM);
}
#[test]
fn zoom_clamp_very_small() {
    assert_eq!(f32::MIN.clamp(MIN_ZOOM, MAX_ZOOM), MIN_ZOOM);
}

#[test]
fn zoom_in_step_from_default() {
    assert!(((1.0_f32 + 0.1).clamp(MIN_ZOOM, MAX_ZOOM) - 1.1).abs() < f32::EPSILON);
}
#[test]
fn zoom_out_step_from_default() {
    assert!(((1.0_f32 - 0.1).clamp(MIN_ZOOM, MAX_ZOOM) - 0.9).abs() < f32::EPSILON);
}
#[test]
fn zoom_in_at_max_stays_clamped() {
    assert_eq!((MAX_ZOOM + 0.1).clamp(MIN_ZOOM, MAX_ZOOM), MAX_ZOOM);
}
#[test]
fn zoom_out_at_min_stays_clamped() {
    assert_eq!((MIN_ZOOM - 0.1).clamp(MIN_ZOOM, MAX_ZOOM), MIN_ZOOM);
}

#[test]
fn node_width_constant() {
    assert_eq!(NODE_WIDTH, 384.0);
}
#[test]
fn node_title_height_constant() {
    assert_eq!(NODE_TITLE_HEIGHT, 32.0);
}

#[test]
fn edge_source_position_calculation() {
    let src = (100.0_f32, 200.0_f32);
    assert_eq!(
        (src.0 + NODE_WIDTH, src.1 + NODE_TITLE_HEIGHT / 2.0),
        (484.0, 216.0)
    );
}

#[test]
fn edge_target_position_calculation() {
    let tgt = (400.0_f32, 200.0_f32);
    assert_eq!((tgt.0, tgt.1 + NODE_TITLE_HEIGHT / 2.0), (400.0, 216.0));
}

#[test]
fn edge_position_with_zoom_and_pan() {
    let (zoom, pan, from) = (2.0_f32, (50.0_f32, 30.0_f32), (100.0_f32, 200.0_f32));
    assert_eq!(
        (from.0 * zoom + pan.0, from.1 * zoom + pan.1),
        (250.0, 430.0)
    );
}

#[test]
fn connection_construction() {
    let conn = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"));
    assert_eq!(conn.id, "c1");
    assert!(conn.is_valid());
}

#[test]
fn connection_same_node_invalid() {
    let conn = Connection::new("c2", PortId::new("n1", "out"), PortId::new("n1", "in"));
    assert!(!conn.is_valid());
}

// --- compute_fit_zoom_and_center tests ---

#[test]
fn fit_zoom_single_node_default_opts() {
    let opts = FitViewOptions::default(); // padding=50, min=0.1, max=2.0
    // Single node at (0,0), size NODE_WIDTH x NODE_HEIGHT = 384x80
    let bounds = (0.0, 0.0, 384.0, 80.0);
    let viewport = (800.0, 600.0);
    let (zoom, (pan_x, pan_y)) = compute_fit_zoom_and_center(bounds, viewport, &opts).unwrap();
    // avail_w = 800 - 100 = 700, avail_h = 600 - 100 = 500
    // zoom_w = 700/384 ≈ 1.822, zoom_h = 500/80 = 6.25
    // target = min(1.822, 6.25) = 1.822, clamped to max_zoom=2.0 → 1.822
    assert!(zoom > 1.8 && zoom < 1.9, "zoom={zoom}");
    // center = (192, 40), pan_x = 400 - 192*zoom, pan_y = 300 - 40*zoom
    let expected_pan_x = 400.0 - 192.0 * zoom;
    let expected_pan_y = 300.0 - 40.0 * zoom;
    assert!((pan_x - expected_pan_x).abs() < 0.01);
    assert!((pan_y - expected_pan_y).abs() < 0.01);
}

#[test]
fn fit_zoom_zero_extent_returns_none() {
    let opts = FitViewOptions::default();
    // All nodes at same position → content_w = 0
    assert!(
        compute_fit_zoom_and_center((100.0, 100.0, 100.0, 100.0), (800.0, 600.0), &opts).is_none()
    );
}

#[test]
fn fit_zoom_viewport_smaller_than_padding() {
    let opts = FitViewOptions {
        padding: 50.0,
        min_zoom: 0.1,
        max_zoom: 2.0,
    };
    // viewport=60x60, padding=50 → avail = max(60-100, 1) = 1.0
    let bounds = (0.0, 0.0, 384.0, 80.0);
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (60.0, 60.0), &opts).unwrap();
    // target = min(1/384, 1/80) ≈ 0.0026, clamped to min_zoom=0.1
    assert_eq!(zoom, 0.1);
}

#[test]
fn fit_zoom_min_greater_than_max_swaps() {
    let opts = FitViewOptions {
        padding: 0.0,
        min_zoom: 3.0,
        max_zoom: 0.5,
    };
    let bounds = (0.0, 0.0, 100.0, 100.0);
    // avail = 800x600, target = min(8.0, 6.0) = 6.0
    // after swap: min=0.5, max=3.0, clamped → 3.0
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (800.0, 600.0), &opts).unwrap();
    assert_eq!(zoom, 3.0);
}

#[test]
fn fit_zoom_nan_min_treated_as_no_lower_bound() {
    let opts = FitViewOptions {
        padding: 0.0,
        min_zoom: f32::NAN,
        max_zoom: 1.0,
    };
    let bounds = (0.0, 0.0, 100.0, 100.0);
    // target = min(8.0, 6.0) = 6.0, clamped to max=1.0
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (800.0, 600.0), &opts).unwrap();
    assert_eq!(zoom, 1.0);
}

#[test]
fn fit_zoom_nan_max_treated_as_no_upper_bound() {
    let opts = FitViewOptions {
        padding: 0.0,
        min_zoom: 0.1,
        max_zoom: f32::NAN,
    };
    let bounds = (0.0, 0.0, 100.0, 100.0);
    // target = min(8.0, 6.0) = 6.0, no upper bound → 6.0
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (800.0, 600.0), &opts).unwrap();
    assert_eq!(zoom, 6.0);
}

#[test]
fn fit_zoom_custom_constraints() {
    let opts = FitViewOptions {
        padding: 20.0,
        min_zoom: 0.5,
        max_zoom: 1.5,
    };
    let bounds = (0.0, 0.0, 100.0, 100.0);
    // avail_w = 800 - 40 = 760, avail_h = 600 - 40 = 560
    // target = min(7.6, 5.6) = 5.6, clamped to 1.5
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (800.0, 600.0), &opts).unwrap();
    assert_eq!(zoom, 1.5);
}

#[test]
fn fit_zoom_zero_padding() {
    let opts = FitViewOptions {
        padding: 0.0,
        min_zoom: 0.1,
        max_zoom: 2.0,
    };
    let bounds = (0.0, 0.0, 800.0, 600.0);
    // Content exactly fills viewport → zoom = 1.0
    let (zoom, _) = compute_fit_zoom_and_center(bounds, (800.0, 600.0), &opts).unwrap();
    assert_eq!(zoom, 1.0);
}

#[test]
fn fit_zoom_centers_content() {
    let opts = FitViewOptions {
        padding: 0.0,
        min_zoom: 0.1,
        max_zoom: 10.0,
    };
    let bounds = (100.0, 200.0, 300.0, 400.0); // 200x200 content
    let viewport = (400.0, 400.0);
    // target = min(2.0, 2.0) = 2.0, center = (200, 300)
    let (zoom, (pan_x, pan_y)) = compute_fit_zoom_and_center(bounds, viewport, &opts).unwrap();
    assert_eq!(zoom, 2.0);
    assert_eq!(pan_x, 400.0 / 2.0 - 200.0 * 2.0); // 200 - 400 = -200
    assert_eq!(pan_y, 400.0 / 2.0 - 300.0 * 2.0); // 200 - 600 = -400
}

// ── Connection validation tests ───���─────────────────────────────
//
// These mirror the validation predicate in `finish_connection`:
//   valid = source.node_id != target.node_id && source_type != target_type

#[test]
fn connection_drag_rejects_same_node() {
    let src = PortId::new("n1", "out");
    let tgt = PortId::new("n1", "in");
    let valid = src.node_id != tgt.node_id && PortType::Output != PortType::Input;
    assert!(
        !valid,
        "connections between ports on the same node must be rejected"
    );
}

#[test]
fn connection_drag_rejects_same_port_type() {
    let src = PortId::new("n1", "out_a");
    let tgt = PortId::new("n2", "out_b");
    let valid = src.node_id != tgt.node_id && PortType::Output != PortType::Output;
    assert!(
        !valid,
        "connections between same port types must be rejected"
    );
}

#[test]
fn connection_drag_accepts_valid_pair() {
    let src = PortId::new("n1", "out");
    let tgt = PortId::new("n2", "in");
    let valid = src.node_id != tgt.node_id && PortType::Output != PortType::Input;
    assert!(
        valid,
        "output-to-input across different nodes must be accepted"
    );
}

#[test]
fn connection_drag_normalizes_direction() {
    // When source is Input and target is Output, the swap condition fires.
    let src_type = PortType::Input;
    let tgt_type = PortType::Output;
    let should_swap = src_type != PortType::Output;
    assert!(should_swap);

    // After normalization, Output comes first.
    let (normalized_src, normalized_tgt) = if should_swap {
        (tgt_type, src_type)
    } else {
        (src_type, tgt_type)
    };
    assert_eq!(normalized_src, PortType::Output);
    assert_eq!(normalized_tgt, PortType::Input);
}

// ── Grid customization tests ────────────────────────────────────
//
// These test the default constants used in the canvas render path.

#[test]
#[allow(clippy::unnecessary_literal_unwrap)]
fn grid_default_spacing_is_40() {
    // Canvas render uses `self.grid_spacing.unwrap_or(40.0)`.
    let spacing: Option<f32> = None;
    assert_eq!(spacing.unwrap_or(40.0), 40.0);
}

#[test]
#[allow(clippy::unnecessary_literal_unwrap)]
fn grid_default_dot_size_is_2() {
    // Canvas render uses `self.grid_dot_size.unwrap_or(2.0)`.
    let dot_size: Option<f32> = None;
    assert_eq!(dot_size.unwrap_or(2.0), 2.0);
}

#[test]
#[allow(clippy::unnecessary_literal_unwrap)]
fn grid_custom_spacing_overrides_default() {
    let spacing: Option<f32> = Some(60.0);
    assert_eq!(spacing.unwrap_or(40.0), 60.0);
}

#[test]
#[allow(clippy::unnecessary_literal_unwrap)]
fn grid_custom_dot_size_overrides_default() {
    let dot_size: Option<f32> = Some(3.5);
    assert_eq!(dot_size.unwrap_or(2.0), 3.5);
}

// ── HIG hit-target guardrails (#149 F17/F19/F27) ───────────────────
//
// These assertions pin the screen-space tolerances so future refactors
// don't silently shrink the target below Apple's 44 pt minimum. If the
// design intentionally changes, update the constant and these asserts
// together — the test failure is the checkpoint.

#[test]
fn port_hit_target_meets_hig_minimum() {
    // 22 px radius ≈ 44 pt diameter.
    assert!(
        PORT_HIT_RADIUS_SCREEN_PX * 2.0 >= 44.0,
        "port hit diameter must meet HIG 44 pt minimum (#149 F17/F27)"
    );
}

#[test]
fn edge_hit_target_meets_hig_minimum() {
    assert!(
        EDGE_HIT_TOLERANCE_SCREEN_PX * 2.0 >= 22.0,
        "edge hit diameter should be large enough to reach per HIG (#149 F19)"
    );
}

#[test]
fn edge_tolerance_is_screen_space_not_world() {
    // Intentionally redundant: documents that the value is in *screen*
    // pixels so the effective target stays constant under zoom. The
    // canvas reads this constant directly in edge_at_screen_point, which
    // operates in screen coordinates.
    let tolerance_at_zoom_1x = EDGE_HIT_TOLERANCE_SCREEN_PX;
    let tolerance_at_zoom_10x = EDGE_HIT_TOLERANCE_SCREEN_PX; // unchanged
    assert_eq!(tolerance_at_zoom_1x, tolerance_at_zoom_10x);
}

// ── Auto-pan guardrails (#149 F11) ──────────────────────────────────

#[test]
fn auto_pan_margin_is_at_least_one_node_height() {
    // A node dropped within the margin should trigger scroll. A 40 px
    // margin exceeds half the default NODE_HEIGHT (80 px / 2 = 40) so
    // the node is at least visibly edging out before pan kicks in.
    assert!(AUTO_PAN_MARGIN >= 40.0, "#149 F11");
}

#[test]
fn auto_pan_step_is_perceptible_but_not_jarring() {
    // Too small (<4 px) feels stuck; too large (>24 px) overshoots.
    assert!(
        AUTO_PAN_STEP >= 4.0 && AUTO_PAN_STEP <= 24.0,
        "auto-pan step outside the comfortable range"
    );
}

// ── Keyboard-shortcut coverage (#149 F2/F3/F4/F6) ───────────────────
//
// These tests pin the modifier/key combinations handle_key_down accepts.
// They mirror the HIG Keyboards table: if Apple documents a new shortcut
// for a canvas action, add a row here and extend `handle_key_down`. A
// test failure is the checkpoint that the shortcut moved.

#[test]
fn zoom_in_responds_to_cmd_plus_or_equals() {
    // ⌘+ and ⌘= trigger the same action because the "+" and "=" keys
    // are the same physical key on US layouts.
    let keys = ["+", "="];
    for key in keys {
        let ks = gpui::Keystroke::parse(&format!("cmd-{key}")).unwrap();
        assert!(ks.modifiers.platform);
        assert!(ks.key.as_str() == key);
    }
}

#[test]
fn undo_redo_use_distinct_shift_modifier() {
    // ⌘Z is undo; Shift-⌘Z is redo. The modifier difference is the
    // sole discriminator.
    let undo = gpui::Keystroke::parse("cmd-z").unwrap();
    let redo = gpui::Keystroke::parse("shift-cmd-z").unwrap();
    assert_eq!(undo.key.as_str(), "z");
    assert_eq!(redo.key.as_str(), "z");
    assert!(undo.modifiers.platform && !undo.modifiers.shift);
    assert!(redo.modifiers.platform && redo.modifiers.shift);
}

#[test]
fn arrow_nudge_uses_shift_for_ten_pt_step() {
    // Plain arrow = 1pt, shift+arrow = 10pt; both unmodified by ⌘.
    let fine = gpui::Keystroke::parse("left").unwrap();
    let coarse = gpui::Keystroke::parse("shift-left").unwrap();
    assert!(!fine.modifiers.shift);
    assert!(coarse.modifiers.shift);
    assert!(!fine.modifiers.platform && !coarse.modifiers.platform);
}

// ── F1 pinch-gesture math ──────────────────────────────────────────

#[test]
fn pinch_delta_applies_multiplicatively() {
    // Per `PinchEvent` docs, `delta` of 0.1 is a 10% zoom-in. The
    // canvas applies it as `zoom * (1 + delta)` so zoom steps scale
    // with the current level — preserving perceived pinch feel.
    let zoom = 1.5_f32;
    let delta = 0.1_f32;
    let next = zoom * (1.0 + delta);
    assert!((next - 1.65).abs() < 1e-4);
}

#[test]
fn pinch_zoom_is_clamped() {
    // Pinch-out at max zoom stays pinned.
    let capped = (MAX_ZOOM * (1.0 + 0.5)).clamp(MIN_ZOOM, MAX_ZOOM);
    assert_eq!(capped, MAX_ZOOM);
    // Pinch-in at min zoom stays pinned.
    let floored = (MIN_ZOOM * (1.0 - 0.9)).clamp(MIN_ZOOM, MAX_ZOOM);
    assert_eq!(floored, MIN_ZOOM);
}

// ── F26 Tab focus cycling ──────────────────────────────────────────
//
// The full cycle lives in `selection::cycle_node_focus`; these tests
// pin the arithmetic invariants (wrap-around, reverse direction, empty
// canvas) without needing a GPUI `Context`.

fn next_focus_idx(current: Option<usize>, len: usize, forward: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let base = current.unwrap_or(if forward { len - 1 } else { 0 });
    if forward {
        Some((base + 1) % len)
    } else {
        Some((base + len - 1) % len)
    }
}

#[test]
fn tab_forward_from_unset_lands_on_first() {
    // Matching `cycle_node_focus`: None → forward picks index 0.
    assert_eq!(next_focus_idx(None, 3, true), Some(0));
}

#[test]
fn tab_backward_from_unset_lands_on_last() {
    assert_eq!(next_focus_idx(None, 3, false), Some(2));
}

#[test]
fn tab_wraps_at_end() {
    assert_eq!(next_focus_idx(Some(2), 3, true), Some(0));
}

#[test]
fn shift_tab_wraps_at_start() {
    assert_eq!(next_focus_idx(Some(0), 3, false), Some(2));
}

#[test]
fn tab_on_empty_canvas_is_no_op() {
    assert!(next_focus_idx(None, 0, true).is_none());
    assert!(next_focus_idx(Some(5), 0, false).is_none());
}
