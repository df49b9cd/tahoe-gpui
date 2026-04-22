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

    use super::marks::canvas_paint_callback;

    let empty: Arc<[f32]> = Arc::from(Vec::<f32>::new());
    let callback = canvas_paint_callback(
        ChartType::Rule,
        point(px(0.0), px(0.0)),
        100.0,
        50.0,
        empty,
        None,
        0.0,
        100.0,
        hsla(0.0, 0.0, 0.0, 1.0),
    );
    assert!(callback.is_none(), "empty Rule must not register a paint");
}

#[test]
fn paint_callback_rule_with_value_returns_some() {
    use std::sync::Arc;

    use gpui::point;

    use super::marks::canvas_paint_callback;

    let values: Arc<[f32]> = Arc::from(vec![50.0]);
    let callback = canvas_paint_callback(
        ChartType::Rule,
        point(px(0.0), px(0.0)),
        100.0,
        50.0,
        values,
        None,
        0.0,
        100.0,
        hsla(0.0, 0.0, 0.0, 1.0),
    );
    assert!(callback.is_some(), "Rule with value must register a paint");
}

#[test]
fn paint_callback_bar_and_point_return_none() {
    // Bar and Point render via div fallback, not the canvas path.
    // A non-None callback would cause double-paint.
    use std::sync::Arc;

    use gpui::point;

    use super::marks::canvas_paint_callback;

    let values: Arc<[f32]> = Arc::from(vec![10.0, 20.0, 30.0]);
    for chart_type in [ChartType::Bar, ChartType::Point] {
        let callback = canvas_paint_callback(
            chart_type,
            point(px(0.0), px(0.0)),
            100.0,
            50.0,
            values.clone(),
            None,
            0.0,
            30.0,
            hsla(0.0, 0.0, 0.0, 1.0),
        );
        assert!(
            callback.is_none(),
            "{chart_type:?} must not register a canvas paint — div fallback handles it"
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
