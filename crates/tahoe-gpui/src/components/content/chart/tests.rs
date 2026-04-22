use core::prelude::v1::test;
use gpui::{
    AppContext, Context, FocusHandle, IntoElement, Render, TestAppContext, Window, hsla, px,
};

use crate::foundations::accessibility::{AccessibilityMode, FocusGroup};
use crate::foundations::theme::TahoeTheme;
use crate::test_helpers::helpers::setup_test_window;

use super::render::Chart;
use super::types::{
    AxisConfig, ChartDataSeries, ChartDataSet, ChartSeries, ChartType, GridlineConfig, bar_width,
    nice_ticks, point_size,
};

fn series() -> ChartDataSeries {
    ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0, 30.0, 25.0])
}

#[test]
fn chart_default_type_is_bar() {
    let chart = Chart::new(series());
    assert_eq!(chart.chart_type, ChartType::Bar);
}

#[test]
fn chart_builder_sets_type() {
    let chart = Chart::new(series()).chart_type(ChartType::Line);
    assert_eq!(chart.chart_type, ChartType::Line);
}

#[test]
fn chart_builder_sets_size() {
    let chart = Chart::new(series()).size(px(300.0), px(160.0));
    assert_eq!(chart.width, px(300.0));
    assert_eq!(chart.height, px(160.0));
}

#[test]
fn chart_builder_sets_color() {
    let c = hsla(0.3, 1.0, 0.5, 1.0);
    let chart = Chart::new(series()).color(c);
    assert_eq!(chart.color, Some(c));
}

#[test]
fn chart_builder_sets_accessibility_label() {
    let chart = Chart::new(series()).accessibility_label("Quarterly sales");
    assert_eq!(
        chart.accessibility_label.as_ref().map(|s| s.as_ref()),
        Some("Quarterly sales")
    );
}

#[test]
fn chart_voice_label_covers_all_mark_types() {
    assert_eq!(ChartType::Bar.voice_label(), "bar");
    assert_eq!(ChartType::Line.voice_label(), "line");
    assert_eq!(ChartType::Area.voice_label(), "area");
    assert_eq!(ChartType::Point.voice_label(), "point");
    assert_eq!(ChartType::Range.voice_label(), "range");
    assert_eq!(ChartType::Rule.voice_label(), "rule");
}

#[test]
fn default_accessibility_label_includes_type_name_count_range() {
    let chart = Chart::new(series()).chart_type(ChartType::Bar);
    let label = chart.default_accessibility_label();
    assert!(label.starts_with("bar chart:"), "got {label:?}");
    assert!(label.contains("Sales"));
    assert!(label.contains("5 values"));
    assert!(label.contains("10.00"));
    assert!(label.contains("30.00"));
}

#[test]
fn default_accessibility_label_handles_empty_series() {
    let chart = Chart::new(ChartDataSeries::new("Empty", vec![]));
    let label = chart.default_accessibility_label();
    assert!(label.contains("no values"));
}

#[test]
fn data_series_min_max() {
    let s = series();
    assert!((s.min_value() - 10.0).abs() < f32::EPSILON);
    assert!((s.max_value() - 30.0).abs() < f32::EPSILON);
}

#[test]
fn bar_area_range_anchor_at_zero() {
    assert!(ChartType::Bar.anchors_at_zero());
    assert!(ChartType::Area.anchors_at_zero());
    assert!(ChartType::Range.anchors_at_zero());
}

#[test]
fn line_point_rule_do_not_anchor_at_zero() {
    assert!(!ChartType::Line.anchors_at_zero());
    assert!(!ChartType::Point.anchors_at_zero());
    assert!(!ChartType::Rule.anchors_at_zero());
}

#[test]
fn bar_width_floors_at_one_pixel_even_for_tiny_slots() {
    // The helper is the single source of truth; asserting it here catches
    // any future refactor that accidentally drops the `.max(1.0)` floor.
    assert!(bar_width(1.0, 1) >= 1.0);
    assert!(bar_width(0.5, 3) >= 1.0);
}

#[test]
fn bar_width_shares_slot_fairly_across_series() {
    // Three series in a 60px slot with BAR_WIDTH_RATIO 0.7 + BAR_GAP 1.0
    // should each get roughly (60*0.7 - 2*1.0) / 3 = 13.33 px.
    let w = bar_width(60.0, 3);
    assert!((12.0..=15.0).contains(&w), "bar width was {w}");
}

#[test]
fn point_size_clamps_to_min_and_max() {
    // Tiny slots floor at MIN_POINT_SIZE (4.0); huge slots cap at
    // MAX_POINT_SIZE (10.0).
    assert_eq!(point_size(1.0), 4.0);
    assert_eq!(point_size(100.0), 10.0);
    // Mid-range slots pass through unchanged.
    assert_eq!(point_size(7.0), 7.0);
}

// ─── HIG: Full Keyboard Access ───────────────────────────────────

struct ChartFkaHarness {
    handles: Vec<FocusHandle>,
    group: FocusGroup,
    chart_type: ChartType,
}

impl ChartFkaHarness {
    fn new(cx: &mut Context<Self>, count: usize, chart_type: ChartType) -> Self {
        Self {
            handles: (0..count).map(|_| cx.focus_handle()).collect(),
            group: FocusGroup::open(),
            chart_type,
        }
    }
}

impl Render for ChartFkaHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Chart::new(series())
            .chart_type(self.chart_type)
            .point_focus_group(self.group.clone())
            .point_focus_handles(self.handles.clone())
    }
}

#[gpui::test]
async fn fka_off_does_not_register_point_handles(cx: &mut TestAppContext) {
    let (host, _cx) = setup_test_window(cx, |_window, cx| {
        ChartFkaHarness::new(cx, 5, ChartType::Bar)
    });
    host.update(cx, |host, _cx| {
        assert!(host.group.is_empty());
    });
}

#[gpui::test]
async fn fka_on_registers_one_focus_per_point(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 5, ChartType::Bar)
    });
    host.update(vcx, |host, _cx| {
        assert_eq!(host.group.len(), 5);
    });
}

#[gpui::test]
async fn fka_on_registers_one_focus_per_point_for_line(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 5, ChartType::Line)
    });
    host.update(vcx, |host, _cx| {
        assert_eq!(host.group.len(), 5);
    });
}

#[gpui::test]
async fn fka_on_preserves_registration_order(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 3, ChartType::Bar)
    });
    host.update(vcx, |host, _cx| {
        for (i, handle) in host.handles.iter().enumerate() {
            assert_eq!(host.group.register(handle), i);
        }
    });
}

#[gpui::test]
async fn fka_on_mismatched_handle_count_skips_registration(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness {
            group: FocusGroup::open(),
            handles: vec![cx.focus_handle(), cx.focus_handle()], // 2 for 5
            chart_type: ChartType::Bar,
        }
    });
    host.update(vcx, |host, _cx| {
        assert!(host.group.is_empty());
    });
}

#[gpui::test]
async fn fka_on_focus_next_advances_along_axis(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 5, ChartType::Bar)
    });
    host.update_in(vcx, |host, window, cx| {
        host.handles[0].focus(window, cx);
        host.group.focus_next(window, cx);
        assert!(host.handles[1].is_focused(window));
    });
}

#[gpui::test]
async fn fka_on_focus_previous_retreats_along_axis(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 5, ChartType::Bar)
    });
    host.update_in(vcx, |host, window, cx| {
        host.handles[2].focus(window, cx);
        host.group.focus_previous(window, cx);
        assert!(host.handles[1].is_focused(window));
    });
}

// ─── Render-harness smoke tests ─────────────────────────────────────
//
// The `#[gpui::test]` suites from here to the end of the file force a
// render pass for a given Chart/ChartView configuration inside a test
// window and assert only that the pass completes without panic.
//
// They catch panics, borrow failures, accessibility-mode transitions,
// and missing child-element wiring. They do NOT assert rendered
// behaviour (legend visibility, tick positions, tooltip contents, axis
// layout). Visual-regression goldens via the `test-support` feature
// are the right tool for that and should be added when a UI bug slips
// past these smoke tests.

// ─── Differentiate Without Color ────────────────────────────────────

struct ChartDwcHarness {
    chart_type: ChartType,
}

impl ChartDwcHarness {
    fn new(chart_type: ChartType) -> Self {
        Self { chart_type }
    }
}

impl Render for ChartDwcHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Chart::new(series())
            .id("dwc-test")
            .chart_type(self.chart_type)
            .size(px(200.0), px(100.0))
    }
}

#[gpui::test]
async fn dwc_renders_bar_without_panic(cx: &mut TestAppContext) {
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        ChartDwcHarness::new(ChartType::Bar)
    });
    host.update(_vcx, |_h, _cx| {
        // Render completed without panic — DwC border styling applied.
    });
}

#[gpui::test]
async fn dwc_renders_point_without_panic(cx: &mut TestAppContext) {
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        ChartDwcHarness::new(ChartType::Point)
    });
    host.update(_vcx, |_h, _cx| {
        // Render completed without panic — DwC border styling applied.
    });
}

// Canvas marks (Line/Area/Range/Rule) paint via the GPUI canvas path and
// share a separate DwC code branch from the div-based Bar/Point marks.
// Each render smoke-tests that branch so a regression in shape rotation
// or ring overlay surfaces as a panic instead of going unnoticed.

#[gpui::test]
async fn dwc_renders_line_without_panic(cx: &mut TestAppContext) {
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        ChartDwcHarness::new(ChartType::Line)
    });
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn dwc_renders_area_without_panic(cx: &mut TestAppContext) {
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        ChartDwcHarness::new(ChartType::Area)
    });
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn dwc_renders_range_without_panic(cx: &mut TestAppContext) {
    struct RangeDwcHarness;
    impl Render for RangeDwcHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::range(
                "Band",
                vec![5.0, 8.0, 10.0],
                vec![15.0, 20.0, 18.0],
            ))
            .id("dwc-range")
            .chart_type(ChartType::Range)
            .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        RangeDwcHarness
    });
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn dwc_renders_rule_without_panic(cx: &mut TestAppContext) {
    struct RuleDwcHarness;
    impl Render for RuleDwcHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new("Target", vec![50.0]))
                .id("dwc-rule")
                .chart_type(ChartType::Rule)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        cx.set_global(theme);
        RuleDwcHarness
    });
    host.update(_vcx, |_h, _cx| {});
}

// ─── Empty series ──────────────────────────────────────────────────

struct ChartEmptyHarness;

impl Render for ChartEmptyHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Chart::new(ChartDataSeries::new("Empty", vec![]))
            .id("empty-test")
            .chart_type(ChartType::Bar)
            .size(px(200.0), px(100.0))
    }
}

#[gpui::test]
async fn empty_series_renders_without_panic(cx: &mut TestAppContext) {
    let (host, _vcx) = setup_test_window(cx, |_, _| ChartEmptyHarness);
    host.update(_vcx, |_h, _cx| {
        // Render completed without panic — "No data" placeholder rendered.
    });
}

#[gpui::test]
async fn fka_on_focus_first_and_last_jump_to_edges(cx: &mut TestAppContext) {
    let (host, vcx) = cx.add_window_view(|_window, cx| {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
        cx.set_global(theme);
        ChartFkaHarness::new(cx, 5, ChartType::Bar)
    });
    host.update_in(vcx, |host, window, cx| {
        host.handles[2].focus(window, cx);
        host.group.focus_last(window, cx);
        assert!(
            host.handles[4].is_focused(window),
            "focus_last lands on final registered handle (End key)"
        );
        host.group.focus_first(window, cx);
        assert!(
            host.handles[0].is_focused(window),
            "focus_first lands on first registered handle (Home key)"
        );
    });
}

// ─── Multi-series ──────────────────────────────────────────────────

#[test]
fn data_set_from_single_series() {
    let ds = ChartDataSet::single(series());
    assert_eq!(ds.series.len(), 1);
    assert!(!ds.is_multi());
}

#[test]
fn data_set_multi_is_multi() {
    let ds = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 2.0])),
        ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 4.0])),
    ]);
    assert_eq!(ds.series.len(), 2);
    assert!(ds.is_multi());
}

#[test]
fn data_set_global_min_max() {
    let ds = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 5.0])),
        ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 10.0])),
    ]);
    assert!((ds.global_min() - 1.0).abs() < f32::EPSILON);
    assert!((ds.global_max() - 10.0).abs() < f32::EPSILON);
}

#[test]
fn chart_accepts_data_set() {
    let ds = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("A", vec![1.0])),
        ChartSeries::new(ChartDataSeries::new("B", vec![2.0])),
    ]);
    let chart = Chart::new(ds);
    assert_eq!(chart.data_set.series.len(), 2);
}

#[test]
fn multi_series_accessibility_label() {
    let ds = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("Revenue", vec![10.0])),
        ChartSeries::new(ChartDataSeries::new("Costs", vec![5.0])),
    ]);
    let chart = Chart::new(ds).chart_type(ChartType::Line);
    let label = chart.default_accessibility_label();
    assert!(label.starts_with("line chart:"), "got {label:?}");
    assert!(label.contains("2 series"));
    assert!(label.contains("Revenue"));
    assert!(label.contains("Costs"));
}

#[gpui::test]
async fn multi_series_bar_renders_without_panic(cx: &mut TestAppContext) {
    struct MultiBarHarness;
    impl Render for MultiBarHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 2.0, 3.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 1.0, 2.0])),
            ]);
            Chart::new(ds)
                .id("multi-bar")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| MultiBarHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn multi_series_line_renders_without_panic(cx: &mut TestAppContext) {
    struct MultiLineHarness;
    impl Render for MultiLineHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 3.0, 2.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![2.0, 1.0, 3.0])),
            ]);
            Chart::new(ds)
                .id("multi-line")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| MultiLineHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn multi_series_area_renders_without_panic(cx: &mut TestAppContext) {
    struct MultiAreaHarness;
    impl Render for MultiAreaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 3.0, 2.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![2.0, 1.0, 3.0])),
            ]);
            Chart::new(ds)
                .id("multi-area")
                .chart_type(ChartType::Area)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| MultiAreaHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Axes ──────────────────────────────────────────────────────────

#[test]
fn nice_ticks_produces_round_values() {
    let ticks = nice_ticks(0.0, 100.0, 5);
    assert!(ticks.contains(&0.0), "ticks: {ticks:?}");
    assert!(ticks.contains(&100.0), "ticks: {ticks:?}");
    // All ticks should be multiples of 20.
    for t in &ticks {
        let rem = *t % 20.0;
        assert!(rem.abs() < 0.01, "tick {t} is not a nice multiple");
    }
}

#[test]
fn nice_ticks_single_value_range() {
    let ticks = nice_ticks(5.0, 5.0, 5);
    assert_eq!(ticks, vec![5.0]);
}

#[test]
fn nice_ticks_handles_inverted_range() {
    // Swapping min/max must not change the output beyond tolerance.
    let forward = nice_ticks(0.0, 100.0, 5);
    let inverted = nice_ticks(100.0, 0.0, 5);
    assert_eq!(forward, inverted, "inverted ranges should normalise");
}

#[test]
fn nice_ticks_handles_nan_and_infinity() {
    // All degenerate inputs must terminate and return a finite (possibly
    // singleton) tick list. Prior to the guard, NaN step sizes drove an
    // unbounded loop on certain inputs.
    let cases = [
        (f32::NAN, 10.0, 5),
        (0.0, f32::NAN, 5),
        (f32::INFINITY, 10.0, 5),
        (0.0, f32::INFINITY, 5),
        (f32::NEG_INFINITY, f32::INFINITY, 5),
        (0.0, 10.0, 0),
    ];
    for (lo, hi, n) in cases {
        let ticks = nice_ticks(lo, hi, n);
        assert!(
            ticks.iter().all(|t| t.is_finite()),
            "non-finite tick from ({lo}, {hi}, {n}): {ticks:?}"
        );
    }
}

proptest::proptest! {
    // Guards against the infinite-loop / NaN-propagation classes from
    // P1.1. The strategy mixes the ordinary numeric range with the
    // exact degenerate values the guard at types.rs was written to
    // defend against (NaN, ±Infinity, f32::MAX, f32::MIN_POSITIVE),
    // so a regression in the log10/powf round-trip will surface as a
    // non-finite tick rather than being hidden by a narrow domain.
    #[test]
    fn nice_ticks_terminates_for_arbitrary_ranges(
        min in proptest::prop_oneof![
            -1e9f32..1e9f32,
            proptest::strategy::Just(f32::NAN),
            proptest::strategy::Just(f32::INFINITY),
            proptest::strategy::Just(f32::NEG_INFINITY),
            proptest::strategy::Just(f32::MAX),
            proptest::strategy::Just(f32::MIN_POSITIVE),
        ],
        max in proptest::prop_oneof![
            -1e9f32..1e9f32,
            proptest::strategy::Just(f32::NAN),
            proptest::strategy::Just(f32::INFINITY),
            proptest::strategy::Just(f32::NEG_INFINITY),
            proptest::strategy::Just(f32::MAX),
            proptest::strategy::Just(f32::MIN_POSITIVE),
        ],
        count in 0usize..10,
    ) {
        let ticks = nice_ticks(min, max, count);
        proptest::prop_assert!(ticks.iter().all(|t| t.is_finite()));
    }

    // Axis-renderer contract: ticks must be strictly increasing so the
    // Y-label column paints top-to-bottom without duplicates. Restrict to
    // a clean numeric strategy (nice_min/nice_max are only well-defined on
    // finite inputs; the degenerate-inputs property above already covers
    // NaN/Inf/overflow).
    #[test]
    fn nice_ticks_are_monotonically_increasing(
        min in -1e6f32..1e6f32,
        delta in 1e-3f32..1e6f32,
        count in 2usize..10,
    ) {
        let max = min + delta;
        let ticks = nice_ticks(min, max, count);
        proptest::prop_assert!(
            ticks.windows(2).all(|w| w[0] < w[1]),
            "ticks not strictly increasing: {ticks:?}"
        );
    }
}

#[test]
fn axis_config_default_is_not_active_for_zero_ticks() {
    let cfg = AxisConfig::new().y_tick_count(0);
    // y_tick_count 0 but no other axis features — still not very useful.
    // But is_active checks if any feature is enabled.
    assert!(cfg.x_labels.is_none());
}

#[test]
fn axis_config_compute_ticks_uses_nice_algorithm() {
    let cfg = AxisConfig::new().y_tick_count(5);
    let ticks = cfg.compute_y_ticks(0.0, 50.0);
    assert!(ticks.len() >= 2);
    assert!(ticks.first().copied().unwrap_or(f32::NAN) <= 0.0);
}

#[test]
fn axis_config_explicit_ticks_override() {
    let cfg = AxisConfig::new().y_ticks(vec![0.0, 10.0, 20.0]);
    let ticks = cfg.compute_y_ticks(0.0, 100.0);
    assert_eq!(ticks, vec![0.0, 10.0, 20.0]);
}

#[gpui::test]
async fn chart_with_axis_renders_without_panic(cx: &mut TestAppContext) {
    struct AxisHarness;
    impl Render for AxisHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new(
                "Sales",
                vec![10.0, 20.0, 15.0, 30.0, 25.0],
            ))
            .id("axis-test")
            .chart_type(ChartType::Bar)
            .size(px(300.0), px(160.0))
            .axis(
                AxisConfig::new()
                    .y_tick_count(5)
                    .x_labels(vec!["Jan", "Feb", "Mar", "Apr", "May"]),
            )
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| AxisHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_with_axis_line_renders_without_panic(cx: &mut TestAppContext) {
    struct AxisLineHarness;
    impl Render for AxisLineHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new("Temp", vec![5.0, 15.0, 25.0, 10.0]))
                .id("axis-line")
                .chart_type(ChartType::Line)
                .size(px(300.0), px(160.0))
                .axis(
                    AxisConfig::new()
                        .y_tick_count(4)
                        .x_labels(vec!["Q1", "Q2", "Q3", "Q4"]),
                )
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| AxisLineHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Title/Subtitle ──────────────────────────────────────────────────

#[test]
fn chart_builder_sets_title_and_subtitle() {
    let chart = Chart::new(series()).title("Revenue").subtitle("Q1 2026");
    assert_eq!(chart.title.as_ref().map(|s| s.as_ref()), Some("Revenue"));
    assert_eq!(chart.subtitle.as_ref().map(|s| s.as_ref()), Some("Q1 2026"));
}

#[gpui::test]
async fn chart_with_title_renders_without_panic(cx: &mut TestAppContext) {
    struct TitleHarness;
    impl Render for TitleHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new(
                "Sales",
                vec![10.0, 20.0, 15.0, 30.0, 25.0],
            ))
            .id("title-test")
            .chart_type(ChartType::Bar)
            .size(px(300.0), px(160.0))
            .title("Monthly Sales")
            .subtitle("Last 5 months")
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| TitleHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_with_title_only_renders_without_panic(cx: &mut TestAppContext) {
    struct TitleOnlyHarness;
    impl Render for TitleOnlyHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0]))
                .id("title-only")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
                .title("Quarterly Sales")
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| TitleOnlyHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Y-axis line ─────────────────────────────────────────────────────

#[gpui::test]
async fn chart_with_show_y_line_renders_without_panic(cx: &mut TestAppContext) {
    struct YLineHarness;
    impl Render for YLineHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new(
                "Sales",
                vec![10.0, 20.0, 15.0, 30.0, 25.0],
            ))
            .id("y-line-test")
            .chart_type(ChartType::Line)
            .size(px(300.0), px(160.0))
            .axis(AxisConfig::new().y_tick_count(5).show_y_line())
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| YLineHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Gridlines ─────────────────────────────────────────────────────

#[test]
fn gridline_config_default_is_inactive() {
    let cfg = GridlineConfig::default();
    assert!(!cfg.is_active());
}

#[test]
fn gridline_config_horizontal_is_active() {
    let cfg = GridlineConfig::horizontal();
    assert!(cfg.is_active());
    assert!(cfg.horizontal);
    assert!(!cfg.vertical);
}

#[test]
fn gridline_config_vertical_is_active() {
    let cfg = GridlineConfig::vertical();
    assert!(cfg.is_active());
    assert!(!cfg.horizontal);
    assert!(cfg.vertical);
}

#[gpui::test]
async fn chart_with_gridlines_renders_without_panic(cx: &mut TestAppContext) {
    struct GridHarness;
    impl Render for GridHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new(
                "Sales",
                vec![10.0, 20.0, 15.0, 30.0, 25.0],
            ))
            .id("grid-test")
            .chart_type(ChartType::Line)
            .size(px(300.0), px(160.0))
            .axis(AxisConfig::new().y_tick_count(5))
            .gridlines(GridlineConfig::horizontal())
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| GridHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_with_both_gridlines_renders_without_panic(cx: &mut TestAppContext) {
    struct BothGridHarness;
    impl Render for BothGridHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let mut gl = GridlineConfig::horizontal();
            gl.vertical = true;
            Chart::new(ChartDataSeries::new(
                "Sales",
                vec![10.0, 20.0, 15.0, 30.0, 25.0],
            ))
            .id("both-grid")
            .chart_type(ChartType::Bar)
            .size(px(300.0), px(160.0))
            .axis(
                AxisConfig::new()
                    .y_tick_count(5)
                    .x_labels(vec!["Jan", "Feb", "Mar", "Apr", "May"]),
            )
            .gridlines(gl)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| BothGridHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Legends ───────────────────────────────────────────────────────

#[gpui::test]
async fn multi_series_auto_shows_legend(cx: &mut TestAppContext) {
    struct LegendHarness;
    impl Render for LegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("Revenue", vec![10.0, 20.0])),
                ChartSeries::new(ChartDataSeries::new("Costs", vec![5.0, 15.0])),
            ]);
            Chart::new(ds)
                .id("legend-test")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| LegendHarness);
    host.update(_vcx, |_h, _cx| {
        // Legend is auto-shown for multi-series; render completes without panic.
    });
}

#[gpui::test]
async fn single_series_legend_hidden(cx: &mut TestAppContext) {
    struct NoLegendHarness;
    impl Render for NoLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("no-legend")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| NoLegendHarness);
    host.update(_vcx, |_h, _cx| {
        // Single series — no legend; render completes without panic.
    });
}

#[gpui::test]
async fn legend_with_three_series_renders(cx: &mut TestAppContext) {
    struct ThreeLegendHarness;
    impl Render for ThreeLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 2.0, 3.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 1.0, 2.0])),
                ChartSeries::new(ChartDataSeries::new("C", vec![2.0, 3.0, 1.0])),
            ]);
            Chart::new(ds)
                .id("three-legend")
                .chart_type(ChartType::Bar)
                .size(px(300.0), px(160.0))
                .axis(AxisConfig::new().y_tick_count(4))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ThreeLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn show_legend_defaults_to_auto() {
    // No explicit `show_legend` call — field remains `None` so the render
    // path can apply the "multi → on, single → off" auto rule.
    let chart = Chart::new(series());
    assert_eq!(chart.show_legend, None);
}

#[test]
fn show_legend_true_overrides_single_series_auto_hide() {
    let chart = Chart::new(series()).show_legend(true);
    assert_eq!(chart.show_legend, Some(true));
}

#[test]
fn show_legend_false_overrides_multi_series_auto_show() {
    let ds = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 2.0])),
        ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 4.0])),
    ]);
    let chart = Chart::new(ds).show_legend(false);
    assert_eq!(chart.show_legend, Some(false));
}

#[gpui::test]
async fn show_legend_true_on_single_series_renders_without_panic(cx: &mut TestAppContext) {
    struct ForcedLegendHarness;
    impl Render for ForcedLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("forced-legend")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
                .show_legend(true)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ForcedLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn show_legend_false_on_multi_series_renders_without_panic(cx: &mut TestAppContext) {
    struct SuppressedLegendHarness;
    impl Render for SuppressedLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 2.0])),
                ChartSeries::new(ChartDataSeries::new("B", vec![3.0, 4.0])),
            ]);
            Chart::new(ds)
                .id("suppressed-legend")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
                .show_legend(false)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| SuppressedLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Range / Rule ─────────────────────────────────────────────────

#[gpui::test]
async fn chart_range_renders_without_panic(cx: &mut TestAppContext) {
    struct RangeHarness;
    impl Render for RangeHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::range(
                "Confidence",
                vec![5.0, 12.0, 8.0, 22.0, 18.0],
                vec![15.0, 28.0, 22.0, 38.0, 32.0],
            ))
            .id("range-test")
            .chart_type(ChartType::Range)
            .size(px(300.0), px(160.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| RangeHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_rule_renders_without_panic(cx: &mut TestAppContext) {
    struct RuleHarness;
    impl Render for RuleHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(ChartDataSeries::new("Target", vec![50.0]))
                .id("rule-test")
                .chart_type(ChartType::Rule)
                .size(px(300.0), px(160.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| RuleHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Paint callbacks ───────────────────────────────────────────────

#[test]
fn paint_callback_empty_rule_returns_none() {
    // An empty Rule used to paint a phantom reference line at the zero
    // baseline. Contract now: no values → no paint callback, so the
    // canvas stays untouched when callers render an empty Rule.
    use std::sync::Arc;

    use gpui::point;

    use super::interpolation::InterpolationMethod;
    use super::marks::canvas_paint_callback;
    use super::scales::{LinearScale, Scale};
    use super::types::ChartPoint;

    let empty: Arc<[ChartPoint]> = Arc::from(Vec::<ChartPoint>::new());
    let scale: Arc<dyn Scale> = Arc::new(LinearScale::new(0.0, 100.0));
    let callback = canvas_paint_callback(
        ChartType::Rule,
        point(px(0.0), px(0.0)),
        100.0,
        50.0,
        empty,
        scale,
        hsla(0.0, 0.0, 0.0, 1.0),
        InterpolationMethod::default(),
    );
    assert!(callback.is_none(), "empty Rule must not register a paint");
}

#[test]
fn paint_callback_rule_with_value_returns_some() {
    use std::sync::Arc;

    use gpui::point;

    use super::interpolation::InterpolationMethod;
    use super::marks::canvas_paint_callback;
    use super::scales::{LinearScale, Scale};
    use super::types::ChartPoint;

    let points: Arc<[ChartPoint]> = Arc::from(vec![ChartPoint::new(0, 50.0f32)]);
    let scale: Arc<dyn Scale> = Arc::new(LinearScale::new(0.0, 100.0));
    let callback = canvas_paint_callback(
        ChartType::Rule,
        point(px(0.0), px(0.0)),
        100.0,
        50.0,
        points,
        scale,
        hsla(0.0, 0.0, 0.0, 1.0),
        InterpolationMethod::default(),
    );
    assert!(callback.is_some(), "Rule with value must register a paint");
}

#[test]
fn paint_callback_bar_and_point_return_none() {
    // Bar and Point render via div fallback, not the canvas path.
    // A non-None callback would cause double-paint.
    use std::sync::Arc;

    use gpui::point;

    use super::interpolation::InterpolationMethod;
    use super::marks::canvas_paint_callback;
    use super::scales::{LinearScale, Scale};
    use super::types::ChartPoint;

    let points: Arc<[ChartPoint]> = Arc::from(vec![
        ChartPoint::new(0, 10.0f32),
        ChartPoint::new(1, 20.0f32),
        ChartPoint::new(2, 30.0f32),
    ]);
    let scale: Arc<dyn Scale> = Arc::new(LinearScale::new(0.0, 30.0));
    for chart_type in [
        ChartType::Bar,
        ChartType::Point,
        ChartType::Sector,
        ChartType::Rectangle,
    ] {
        let callback = canvas_paint_callback(
            chart_type,
            point(px(0.0), px(0.0)),
            100.0,
            50.0,
            points.clone(),
            scale.clone(),
            hsla(0.0, 0.0, 0.0, 1.0),
            InterpolationMethod::default(),
        );
        assert!(
            callback.is_none(),
            "{chart_type:?} must not register a canvas paint — div fallback or custom geometry handles it"
        );
    }
}

// ─── Ragged multi-series ───────────────────────────────────────────

// Apple Charts tolerates series with different lengths in the same dataset
// (e.g. a 7-day "Sales" series alongside a 5-day "Target"). The render
// path must not panic when indexing beyond the shortest series.

#[gpui::test]
async fn ragged_multi_series_bar_renders_without_panic(cx: &mut TestAppContext) {
    struct RaggedBarHarness;
    impl Render for RaggedBarHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("Long", vec![1.0, 2.0, 3.0, 4.0, 5.0])),
                ChartSeries::new(ChartDataSeries::new("Short", vec![3.0, 1.0])),
            ]);
            Chart::new(ds)
                .id("ragged-bar")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| RaggedBarHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn ragged_multi_series_line_renders_without_panic(cx: &mut TestAppContext) {
    struct RaggedLineHarness;
    impl Render for RaggedLineHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("Long", vec![1.0, 2.0, 3.0, 4.0, 5.0])),
                ChartSeries::new(ChartDataSeries::new("Short", vec![3.0, 1.0])),
            ]);
            Chart::new(ds)
                .id("ragged-line")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| RaggedLineHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn ragged_multi_series_with_empty_series_renders(cx: &mut TestAppContext) {
    struct RaggedEmptyHarness;
    impl Render for RaggedEmptyHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let ds = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("Data", vec![1.0, 2.0, 3.0])),
                ChartSeries::new(ChartDataSeries::new("Empty", vec![])),
            ]);
            Chart::new(ds)
                .id("ragged-empty")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| RaggedEmptyHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── ChartView ─────────────────────────────────────────────────────

#[gpui::test]
async fn chart_view_renders_without_panic(cx: &mut TestAppContext) {
    struct ChartViewHarness;
    impl Render for ChartViewHarness {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use crate::components::content::chart::view::ChartView;
            cx.new(|cx| {
                ChartView::new(
                    cx,
                    ChartDataSet::multi(vec![
                        ChartSeries::new(ChartDataSeries::new("A", vec![1.0, 3.0, 2.0])),
                        ChartSeries::new(ChartDataSeries::new("B", vec![2.0, 1.0, 3.0])),
                    ]),
                )
                .id("view-test")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
            })
            .into_any_element()
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ChartViewHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── AxisConfig scale hookup ───────────────────────────────────────
//
// Phase 1 adds `x_scale` / `y_scale` overrides on `AxisConfig` and uses the
// scale's `project` / `ticks` instead of the legacy `(v - min) / range`
// projection. Without these tests the builder + render path could silently
// fall back to the default `LinearScale` and nobody would notice.

#[test]
fn axis_config_stores_y_scale_override() {
    use super::scales::LogScale;

    let axis = AxisConfig::new().y_scale(LogScale::new(1.0, 1000.0));
    assert!(
        axis.y_scale.is_some(),
        "y_scale builder must populate the field"
    );
}

#[test]
fn axis_config_stores_x_scale_override() {
    use super::scales::CategoryScale;

    let axis = AxisConfig::new().x_scale(CategoryScale::new(vec!["a", "b", "c"]));
    assert!(
        axis.x_scale.is_some(),
        "x_scale builder must populate the field"
    );
}

#[test]
fn axis_config_ticks_come_from_scale_when_provided() {
    use std::sync::Arc;

    use super::scales::{LogScale, Scale};
    use super::types::PlottableValue;

    // A LogScale over 1..=1e6 produces ticks at powers of 10, not the
    // evenly-spaced ticks `nice_ticks` would emit for the same domain.
    let scale: Arc<dyn Scale> = Arc::new(LogScale::new(1.0, 1.0e6));
    let ticks = scale.ticks(6);
    assert!(!ticks.is_empty());
    for (v, _label) in &ticks {
        let n = match v {
            PlottableValue::Number(n) => *n,
            _ => panic!("LogScale must emit numeric ticks"),
        };
        let log = n.log10();
        assert!(
            (log.round() - log).abs() < 1e-6,
            "LogScale tick {n} is not a power of 10"
        );
    }
}

#[gpui::test]
async fn chart_with_log_y_scale_renders_without_panic(cx: &mut TestAppContext) {
    struct LogHarness;
    impl Render for LogHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            use super::scales::LogScale;

            let series = ChartDataSeries::new("exp", vec![1.0, 10.0, 100.0, 1_000.0, 10_000.0]);
            Chart::new(series)
                .id("log-chart")
                .chart_type(ChartType::Line)
                .size(px(200.0), px(100.0))
                .axis(AxisConfig::new().y_scale(LogScale::new(1.0, 10_000.0)))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| LogHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_with_linear_y_scale_override_renders_without_panic(cx: &mut TestAppContext) {
    struct LinearHarness;
    impl Render for LinearHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            use super::scales::LinearScale;

            // User-forced domain larger than the data so the chart leaves
            // headroom above the data instead of tight-fitting.
            Chart::new(series())
                .id("linear-override")
                .chart_type(ChartType::Bar)
                .size(px(200.0), px(100.0))
                .axis(AxisConfig::new().y_scale(LinearScale::new(0.0, 100.0)))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| LinearHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn pie_chart_renders_without_panic(cx: &mut TestAppContext) {
    struct PieHarness;
    impl Render for PieHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let set = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("a", vec![10.0])),
                ChartSeries::new(ChartDataSeries::new("b", vec![25.0])),
                ChartSeries::new(ChartDataSeries::new("c", vec![15.0])),
                ChartSeries::new(ChartDataSeries::new("d", vec![50.0])),
            ]);
            Chart::new(set)
                .id("pie-chart")
                .chart_type(ChartType::Sector)
                .size(px(200.0), px(200.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| PieHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn donut_chart_renders_without_panic(cx: &mut TestAppContext) {
    struct DonutHarness;
    impl Render for DonutHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let set = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("a", vec![30.0])),
                ChartSeries::new(ChartDataSeries::new("b", vec![70.0])),
            ]);
            Chart::new(set)
                .id("donut-chart")
                .chart_type(ChartType::Sector)
                .inner_radius_ratio(0.6)
                .size(px(200.0), px(200.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| DonutHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn rectangle_chart_renders_without_panic(cx: &mut TestAppContext) {
    struct HeatHarness;
    impl Render for HeatHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            use super::types::ChartPoint;

            // 3×3 heatmap with z magnitudes.
            let mut points = Vec::new();
            for x in 0..3 {
                for y in 0..3 {
                    points.push(ChartPoint::new(x as f32, y as f32).with_z((x + y) as f32));
                }
            }
            let series = ChartDataSeries::from_points("heat", points);
            Chart::new(series)
                .id("heat-chart")
                .chart_type(ChartType::Rectangle)
                .size(px(200.0), px(200.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| HeatHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn sector_with_no_positive_weights_does_not_render() {
    use super::sector::sector_weights;
    let set = ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new("zero", vec![0.0])),
        ChartSeries::new(ChartDataSeries::new("neg", vec![-1.0])),
    ]);
    let pairs = sector_weights(&set, |_| hsla(0.0, 0.0, 0.0, 1.0));
    assert!(
        pairs.is_empty(),
        "sectors should reject non-positive weights so the slice ratio stays meaningful"
    );
}

#[test]
fn chart_type_uses_custom_plot_geometry_for_sector_and_rectangle() {
    assert!(ChartType::Sector.uses_custom_plot_geometry());
    assert!(ChartType::Rectangle.uses_custom_plot_geometry());
    assert!(!ChartType::Bar.uses_custom_plot_geometry());
    assert!(!ChartType::Line.uses_custom_plot_geometry());
}

#[test]
fn voice_label_for_sector_and_rectangle() {
    assert_eq!(ChartType::Sector.voice_label(), "sector");
    assert_eq!(ChartType::Rectangle.voice_label(), "heatmap");
}

#[gpui::test]
async fn stacked_bar_chart_renders_without_panic(cx: &mut TestAppContext) {
    use super::MarkStackingMethod;

    struct StackHarness;
    impl Render for StackHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let set = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("a", vec![10.0, 20.0, 15.0])),
                ChartSeries::new(ChartDataSeries::new("b", vec![30.0, 40.0, 25.0])),
                ChartSeries::new(ChartDataSeries::new("c", vec![5.0, 10.0, 20.0])),
            ]);
            Chart::new(set)
                .id("stacked-bar-chart")
                .chart_type(ChartType::Bar)
                .stacking(MarkStackingMethod::Standard)
                .size(px(300.0), px(180.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| StackHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn stacked_area_chart_renders_without_panic(cx: &mut TestAppContext) {
    use super::MarkStackingMethod;

    struct StackAreaHarness;
    impl Render for StackAreaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let set = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("a", vec![10.0, 20.0, 15.0, 30.0])),
                ChartSeries::new(ChartDataSeries::new("b", vec![30.0, 40.0, 25.0, 20.0])),
            ]);
            Chart::new(set)
                .id("stacked-area-chart")
                .chart_type(ChartType::Area)
                .stacking(MarkStackingMethod::Normalized)
                .size(px(300.0), px(180.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| StackAreaHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn stacking_is_active_reflects_method() {
    use super::MarkStackingMethod;

    assert!(!MarkStackingMethod::Unstacked.is_active());
    assert!(MarkStackingMethod::Standard.is_active());
    assert!(MarkStackingMethod::Normalized.is_active());
    assert!(MarkStackingMethod::Center.is_active());
}

#[test]
fn bar_orientation_defaults_to_vertical() {
    use super::types::BarOrientation;

    assert_eq!(BarOrientation::default(), BarOrientation::Vertical);
}

#[test]
fn chart_builder_sets_bar_orientation() {
    use super::types::BarOrientation;

    let chart = Chart::new(series()).bar_orientation(BarOrientation::Horizontal);
    assert_eq!(chart.bar_orientation, BarOrientation::Horizontal);
}

#[gpui::test]
async fn horizontal_bar_chart_renders_without_panic(cx: &mut TestAppContext) {
    use super::types::BarOrientation;

    struct HorizontalHarness;
    impl Render for HorizontalHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("horizontal-bar")
                .chart_type(ChartType::Bar)
                .bar_orientation(BarOrientation::Horizontal)
                .size(px(300.0), px(180.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| HorizontalHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn stacked_horizontal_bar_renders_without_panic(cx: &mut TestAppContext) {
    use super::MarkStackingMethod;
    use super::types::BarOrientation;

    struct StackedHorizontalHarness;
    impl Render for StackedHorizontalHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let set = ChartDataSet::multi(vec![
                ChartSeries::new(ChartDataSeries::new("a", vec![10.0, 20.0, 15.0])),
                ChartSeries::new(ChartDataSeries::new("b", vec![30.0, 40.0, 25.0])),
                ChartSeries::new(ChartDataSeries::new("c", vec![5.0, 10.0, 20.0])),
            ]);
            Chart::new(set)
                .id("stacked-horizontal")
                .chart_type(ChartType::Bar)
                .bar_orientation(BarOrientation::Horizontal)
                .stacking(MarkStackingMethod::Standard)
                .size(px(300.0), px(180.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| StackedHorizontalHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn scatter_with_x_scale_renders_without_panic(cx: &mut TestAppContext) {
    struct ScatterHarness;
    impl Render for ScatterHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            use super::scales::LinearScale;
            use super::types::ChartPoint;

            // 2D scatter: arbitrary (x, y) points, not index-based.
            let points = vec![
                ChartPoint::new(0.5f32, 2.3f32),
                ChartPoint::new(3.8f32, 7.1f32),
                ChartPoint::new(9.2f32, 4.6f32),
            ];
            let series = ChartDataSeries::from_points("scatter", points);
            Chart::new(series)
                .id("scatter-2d")
                .chart_type(ChartType::Point)
                .size(px(300.0), px(180.0))
                .axis(
                    AxisConfig::new()
                        .x_scale(LinearScale::new(0.0, 10.0))
                        .y_scale(LinearScale::new(0.0, 10.0)),
                )
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ScatterHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn chart_builder_sets_annotations() {
    use super::annotation::{
        AnnotationContent, AnnotationPosition, AnnotationTarget, ChartAnnotation,
    };

    let chart = Chart::new(series()).annotations(vec![ChartAnnotation::text(
        AnnotationTarget::DataPoint {
            series_idx: 0,
            point_idx: 3,
        },
        AnnotationPosition::Top,
        "peak",
    )]);
    assert_eq!(chart.annotations.len(), 1);
    assert!(matches!(
        chart.annotations[0].content,
        AnnotationContent::Text(_)
    ));
}

#[gpui::test]
async fn line_chart_with_annotation_renders_without_panic(cx: &mut TestAppContext) {
    use super::annotation::{AnnotationPosition, AnnotationTarget, ChartAnnotation};

    struct AnnotatedHarness;
    impl Render for AnnotatedHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("annotated-line")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(180.0))
                .annotations(vec![ChartAnnotation::text(
                    AnnotationTarget::DataPoint {
                        series_idx: 0,
                        point_idx: 3,
                    },
                    AnnotationPosition::Top,
                    "Record high",
                )])
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| AnnotatedHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn annotation_at_upper_edge_flips_below_without_panic(cx: &mut TestAppContext) {
    use super::annotation::{AnnotationPosition, AnnotationTarget, ChartAnnotation};

    struct EdgeAnnotationHarness;
    impl Render for EdgeAnnotationHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // The third point sits at the top of the plot, so a Top
            // annotation should overflow-flip to Bottom. We don't
            // introspect the resolution here (unit tests already cover
            // that); this just guarantees the render path handles the
            // flip without panicking.
            let points = vec![5.0, 10.0, 99.0, 30.0];
            Chart::new(ChartDataSeries::new("edge", points))
                .id("edge-annotation")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(180.0))
                .annotations(vec![ChartAnnotation::text(
                    AnnotationTarget::DataPoint {
                        series_idx: 0,
                        point_idx: 2,
                    },
                    AnnotationPosition::Top,
                    "Peak",
                )])
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| EdgeAnnotationHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn chart_builder_defaults_to_catmull_rom_interpolation() {
    use super::interpolation::InterpolationMethod;

    let chart = Chart::new(series());
    assert_eq!(chart.interpolation, InterpolationMethod::CatmullRom);
}

#[test]
fn chart_builder_sets_interpolation() {
    use super::interpolation::InterpolationMethod;

    let chart = Chart::new(series()).interpolation(InterpolationMethod::Monotone);
    assert_eq!(chart.interpolation, InterpolationMethod::Monotone);

    let stepped = Chart::new(series()).interpolation(InterpolationMethod::StepEnd);
    assert_eq!(stepped.interpolation, InterpolationMethod::StepEnd);

    let cardinal = Chart::new(series()).interpolation(InterpolationMethod::Cardinal(0.5));
    assert_eq!(cardinal.interpolation, InterpolationMethod::Cardinal(0.5));
}

#[gpui::test]
async fn line_chart_cycles_through_every_interpolation(cx: &mut TestAppContext) {
    use super::interpolation::InterpolationMethod;

    // Render a Line chart once per method to confirm the dispatch path
    // keeps all variants alive (no dead arms, no panics from control
    // point overflow on a representative data set).
    struct InterpHarness(InterpolationMethod);
    impl Render for InterpHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("interp-line")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(160.0))
                .interpolation(self.0)
        }
    }
    for method in [
        InterpolationMethod::Linear,
        InterpolationMethod::CatmullRom,
        InterpolationMethod::Cardinal(0.5),
        InterpolationMethod::Cardinal(1.0),
        InterpolationMethod::Monotone,
        InterpolationMethod::StepStart,
        InterpolationMethod::StepEnd,
        InterpolationMethod::StepCenter,
    ] {
        let (host, _vcx) = setup_test_window(cx, move |_, _| InterpHarness(method));
        host.update(_vcx, |_h, _cx| {});
    }
}

#[gpui::test]
async fn area_chart_with_monotone_renders_without_panic(cx: &mut TestAppContext) {
    use super::interpolation::InterpolationMethod;

    struct MonotoneAreaHarness;
    impl Render for MonotoneAreaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("monotone-area")
                .chart_type(ChartType::Area)
                .size(px(320.0), px(180.0))
                .interpolation(InterpolationMethod::Monotone)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| MonotoneAreaHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Phase 7: Axis customisation ────────────────────────────────────

#[test]
fn axis_marks_default_values() {
    use super::types::{AxisMarks, AxisPosition, AxisTickStyle, GridLineStyle};

    let marks = AxisMarks::default();
    assert_eq!(marks.position, AxisPosition::Automatic);
    assert!(matches!(marks.tick_style, AxisTickStyle::Automatic));
    assert_eq!(marks.grid_line_style, GridLineStyle::Solid);
    assert!(marks.value_label_formatter.is_none());
}

#[test]
fn axis_config_effective_positions_resolve_automatic() {
    use super::types::{AxisPosition, GridLineStyle};

    let cfg = AxisConfig::new();
    assert_eq!(cfg.effective_y_position(), AxisPosition::Leading);
    assert_eq!(cfg.effective_x_position(), AxisPosition::Bottom);
    assert_eq!(cfg.y_marks.grid_line_style, GridLineStyle::Solid);

    let trailing = AxisConfig::new().y_position(AxisPosition::Trailing);
    assert_eq!(trailing.effective_y_position(), AxisPosition::Trailing);

    // Mismatched values (e.g. Top on a Y axis) collapse to the HIG default
    // so callers that pass the wrong enum by mistake still render.
    let weird = AxisConfig::new().y_position(AxisPosition::Top);
    assert_eq!(weird.effective_y_position(), AxisPosition::Leading);
}

#[test]
fn axis_config_builders_write_into_marks() {
    use std::sync::Arc;

    use super::types::{AxisPosition, AxisTickStyle, GridLineStyle, PlottableValue};
    use gpui::SharedString;

    let cfg = AxisConfig::new()
        .y_position(AxisPosition::Trailing)
        .y_tick_style(AxisTickStyle::Manual(vec![0.0, 10.0, 20.0]))
        .y_grid_line_style(GridLineStyle::Dashed)
        .y_value_label_formatter(|v: &PlottableValue| {
            SharedString::from(format!("${}", v.as_number_f32().unwrap_or(0.0) as i64))
        });

    assert_eq!(cfg.y_marks.position, AxisPosition::Trailing);
    assert!(matches!(cfg.y_marks.tick_style, AxisTickStyle::Manual(_)));
    assert_eq!(cfg.y_marks.grid_line_style, GridLineStyle::Dashed);
    // Formatter round-trips through the marks struct.
    let fmt = cfg.y_marks.value_label_formatter.as_ref().expect("fmt");
    let label = fmt(&PlottableValue::Number(42.0));
    assert_eq!(label.as_ref(), "$42");

    // y_tick_style(Manual(...)) also mirrors into the legacy `y_ticks`
    // field so direct readers keep seeing the manual override.
    assert_eq!(cfg.y_ticks, Some(vec![0.0, 10.0, 20.0]));

    // Formatter stays Arc-cloneable (no &self capture).
    let _clone: Arc<_> = fmt.clone();
}

#[test]
fn axis_tick_style_hidden_yields_no_ticks() {
    use super::types::AxisTickStyle;

    let cfg = AxisConfig::new().y_tick_style(AxisTickStyle::Hidden);
    assert!(cfg.compute_y_ticks(0.0, 100.0).is_empty());
}

#[test]
fn axis_tick_style_manual_overrides_nice_ticks() {
    use super::types::AxisTickStyle;

    let cfg = AxisConfig::new()
        .y_tick_count(5)
        .y_tick_style(AxisTickStyle::Manual(vec![0.0, 25.0, 50.0, 75.0, 100.0]));
    let ticks = cfg.compute_y_ticks(10.0, 90.0);
    assert_eq!(ticks, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
}

#[test]
fn gridline_config_style_builder() {
    use super::types::{GridLineStyle, GridlineConfig};

    let cfg = GridlineConfig::horizontal().style(GridLineStyle::Dashed);
    assert!(cfg.horizontal);
    assert_eq!(cfg.style, GridLineStyle::Dashed);
    assert!(cfg.is_active());

    // Hidden style collapses is_active even when h/v flags are set so
    // render.rs doesn't allocate a canvas layer for invisible gridlines.
    let hidden = GridlineConfig::horizontal().style(GridLineStyle::Hidden);
    assert!(!hidden.is_active());
}

#[gpui::test]
async fn chart_renders_with_trailing_y_axis(cx: &mut TestAppContext) {
    use super::types::AxisPosition;

    struct TrailingAxisHarness;
    impl Render for TrailingAxisHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("trailing-y")
                .chart_type(ChartType::Bar)
                .size(px(280.0), px(140.0))
                .axis(AxisConfig::new().y_position(AxisPosition::Trailing))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| TrailingAxisHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_dashed_gridlines_and_formatter(cx: &mut TestAppContext) {
    use super::types::{GridLineStyle, PlottableValue};
    use gpui::SharedString;

    struct DashedFormatterHarness;
    impl Render for DashedFormatterHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("dashed-fmt")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(180.0))
                .axis(
                    AxisConfig::new().y_value_label_formatter(|v: &PlottableValue| {
                        SharedString::from(format!("${}", v.as_number_f32().unwrap_or(0.0) as i64))
                    }),
                )
                .gridlines(GridlineConfig::horizontal().style(GridLineStyle::Dashed))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| DashedFormatterHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_hidden_ticks(cx: &mut TestAppContext) {
    use super::types::AxisTickStyle;

    struct HiddenTicksHarness;
    impl Render for HiddenTicksHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(series())
                .id("hidden-ticks")
                .chart_type(ChartType::Bar)
                .size(px(260.0), px(140.0))
                .axis(AxisConfig::new().y_tick_style(AxisTickStyle::Hidden))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| HiddenTicksHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[test]
fn legend_position_default_is_automatic() {
    use super::types::LegendPosition;

    let chart = Chart::new(series());
    assert_eq!(chart.legend_position, LegendPosition::Automatic);
}

#[test]
fn legend_position_builder_sets_position() {
    use super::types::LegendPosition;

    let chart = Chart::new(series()).legend_position(LegendPosition::Top);
    assert_eq!(chart.legend_position, LegendPosition::Top);
}

fn two_series() -> ChartDataSet {
    ChartDataSet::multi(vec![
        ChartSeries::new(ChartDataSeries::new(
            "Sales",
            vec![10.0, 20.0, 15.0, 30.0, 25.0],
        )),
        ChartSeries::new(ChartDataSeries::new(
            "Target",
            vec![12.0, 18.0, 22.0, 25.0, 22.0],
        )),
    ])
}

#[gpui::test]
async fn chart_renders_with_top_legend(cx: &mut TestAppContext) {
    use super::types::LegendPosition;

    struct TopLegendHarness;
    impl Render for TopLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(two_series())
                .id("top-legend")
                .chart_type(ChartType::Bar)
                .size(px(320.0), px(160.0))
                .legend_position(LegendPosition::Top)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| TopLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_trailing_legend(cx: &mut TestAppContext) {
    use super::types::LegendPosition;

    struct TrailingLegendHarness;
    impl Render for TrailingLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(two_series())
                .id("trailing-legend")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(160.0))
                .legend_position(LegendPosition::Trailing)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| TrailingLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_leading_legend(cx: &mut TestAppContext) {
    use super::types::LegendPosition;

    struct LeadingLegendHarness;
    impl Render for LeadingLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(two_series())
                .id("leading-legend")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(160.0))
                .legend_position(LegendPosition::Leading)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| LeadingLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_hidden_legend(cx: &mut TestAppContext) {
    use super::types::LegendPosition;

    // `LegendPosition::Hidden` must win even if the caller also forces
    // `show_legend(true)` on a multi-series chart.
    struct HiddenLegendHarness;
    impl Render for HiddenLegendHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Chart::new(two_series())
                .id("hidden-legend")
                .chart_type(ChartType::Bar)
                .size(px(320.0), px(160.0))
                .show_legend(true)
                .legend_position(LegendPosition::Hidden)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| HiddenLegendHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Phase 10: scroll / zoom ─────────────────────────────────────────────

#[test]
fn chart_scroll_builder_stores_config() {
    use super::scroll::ChartScrollConfig;

    let chart = Chart::new(series()).scroll(
        ChartScrollConfig::new()
            .x_visible_domain(0.0, 2.0)
            .x_scroll_position(1.0),
    );
    assert!(chart.scroll.is_some());
    let scroll = chart.scroll.as_ref().unwrap();
    assert_eq!(
        scroll
            .x_visible_domain
            .as_ref()
            .and_then(|(lo, hi)| Some((lo.as_number()?, hi.as_number()?))),
        Some((0.0, 2.0))
    );
    assert_eq!(
        scroll
            .x_scroll_position
            .as_ref()
            .and_then(|v| v.as_number()),
        Some(1.0)
    );
}

#[gpui::test]
async fn chart_renders_with_visible_domain_subset(cx: &mut TestAppContext) {
    use super::scroll::ChartScrollConfig;

    // 100-point dataset with a 10-point visible window — the render path
    // should filter the data to the first 10 points and lay them out across
    // the full plot width.
    struct ScrollHarness;
    impl Render for ScrollHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let values: Vec<f32> = (0..100).map(|i| (i as f32 * 0.37).sin()).collect();
            Chart::new(ChartDataSeries::new("Signal", values))
                .id("scroll-domain")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(160.0))
                .scroll(ChartScrollConfig::new().x_visible_domain(0.0, 9.0))
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ScrollHarness);
    host.update(_vcx, |_h, _cx| {});
}

#[gpui::test]
async fn chart_renders_with_scroll_position(cx: &mut TestAppContext) {
    use super::scroll::ChartScrollConfig;

    // Same dataset but anchored partway through — the visible slice should
    // be points 40..49 rather than 0..9.
    struct ScrollPosHarness;
    impl Render for ScrollPosHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let values: Vec<f32> = (0..100).map(|i| (i as f32 * 0.37).sin()).collect();
            Chart::new(ChartDataSeries::new("Signal", values))
                .id("scroll-pos")
                .chart_type(ChartType::Line)
                .size(px(320.0), px(160.0))
                .scroll(
                    ChartScrollConfig::new()
                        .x_visible_domain(0.0, 9.0)
                        .x_scroll_position(40.0),
                )
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| ScrollPosHarness);
    host.update(_vcx, |_h, _cx| {});
}

// ─── Phase 11: Audio Graphs accessibility ───────────────────────────────

#[test]
fn chart_audio_graph_builder_stores_descriptor() {
    use super::audio_graph::{AxisDescriptor, ChartDescriptor, SeriesDescriptor};

    let desc = ChartDescriptor::new(
        "Quarterly sales",
        "Sales peak in Q4 around $30",
        AxisDescriptor::new("Quarter", (0.0, 4.0)),
        AxisDescriptor::new("USD", (0.0, 30.0)),
    )
    .series(vec![SeriesDescriptor::new(
        "Sales",
        [(0.0, 10.0), (1.0, 20.0), (2.0, 15.0), (3.0, 30.0)],
    )]);

    let chart = Chart::new(series()).audio_graph(desc);
    let stored = chart.audio_graph.as_ref().expect("descriptor stored");
    assert_eq!(stored.title.as_ref(), "Quarterly sales");
    assert_eq!(stored.summary.as_ref(), "Sales peak in Q4 around $30");
    assert_eq!(stored.series.len(), 1);
    assert_eq!(stored.y_axis.range, (0.0, 30.0));
}

#[test]
fn chart_audio_graph_summary_replaces_default_label() {
    use super::audio_graph::{AxisDescriptor, ChartDescriptor};

    let summary = "Weekly temperature trends upward from 18 to 27 degrees";
    let desc = ChartDescriptor::new(
        "Weekly temperature",
        summary,
        AxisDescriptor::new("Day", (0.0, 6.0)),
        AxisDescriptor::new("°C", (18.0, 27.0)),
    );
    let chart = Chart::new(series()).audio_graph(desc);
    // accessibility_label precedence chain: no explicit label set → summary
    // wins over the auto-generated default.
    assert!(chart.accessibility_label.is_none());
    assert_eq!(
        chart
            .audio_graph
            .as_ref()
            .map(|d| d.summary.as_ref())
            .unwrap_or(""),
        summary
    );
    // Confirm the default generator is different from the summary so the
    // replace-with-summary test is actually meaningful.
    let default_label = chart.default_accessibility_label();
    assert_ne!(default_label, summary);
}

#[test]
fn chart_audio_graph_explicit_label_still_wins() {
    use super::audio_graph::{AxisDescriptor, ChartDescriptor};

    let desc = ChartDescriptor::new(
        "T",
        "Summary from descriptor",
        AxisDescriptor::new("X", (0.0, 1.0)),
        AxisDescriptor::new("Y", (0.0, 1.0)),
    );
    let chart = Chart::new(series())
        .audio_graph(desc)
        .accessibility_label("Explicit override");
    // Explicit accessibility_label wins over the descriptor's summary.
    assert_eq!(
        chart.accessibility_label.as_ref().map(|s| s.as_ref()),
        Some("Explicit override")
    );
}

#[gpui::test]
async fn chart_renders_with_audio_graph_descriptor(cx: &mut TestAppContext) {
    use super::audio_graph::{AxisDescriptor, ChartDescriptor, SeriesDescriptor};

    struct AudioHarness;
    impl Render for AudioHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let desc = ChartDescriptor::new(
                "Quarterly sales",
                "Sales rise through Q4",
                AxisDescriptor::new("Quarter", (0.0, 4.0)),
                AxisDescriptor::new("USD", (0.0, 30.0)),
            )
            .series(vec![SeriesDescriptor::new(
                "Sales",
                [(0.0, 10.0), (1.0, 20.0), (2.0, 15.0), (3.0, 30.0)],
            )]);
            Chart::new(ChartDataSeries::new("Sales", vec![10.0, 20.0, 15.0, 30.0]))
                .id("audio-chart")
                .chart_type(ChartType::Bar)
                .size(px(320.0), px(160.0))
                .audio_graph(desc)
        }
    }
    let (host, _vcx) = setup_test_window(cx, |_, _| AudioHarness);
    host.update(_vcx, |_h, _cx| {});
}
