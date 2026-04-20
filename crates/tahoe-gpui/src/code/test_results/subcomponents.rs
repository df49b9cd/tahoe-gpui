//! Stateless subcomponents for the test results display.

use crate::callback_types::OnToggle;
use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::layout_and_organization::disclosure_group::DisclosureGroup;
use crate::components::status::progress_indicator::ProgressIndicator;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, SharedString, Window, div, px};

use super::types::{TestCase, TestStatus, TestSummary, format_duration, status_icon_and_color};

// ─── TestError ──────────────────────────────────────────────────────────────

/// Error display for a failed test, with optional stack trace.
#[derive(IntoElement)]
pub struct TestError {
    pub(crate) message: SharedString,
    pub(crate) stack: Option<SharedString>,
}

impl TestError {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            stack: None,
        }
    }

    pub fn stack(mut self, stack: impl Into<SharedString>) -> Self {
        self.stack = Some(stack.into());
        self
    }
}

impl RenderOnce for TestError {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut col = div().flex().flex_col().pl(theme.spacing_lg);

        col = col.child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.error)
                .font(theme.mono_font())
                .child(self.message),
        );

        if let Some(stack) = self.stack {
            col = col.child(
                div()
                    .pt(px(2.0))
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .font(theme.mono_font())
                    .overflow_x_hidden()
                    .child(stack),
            );
        }

        col
    }
}

// ─── Test ───────────────────────────────────────────────────────────────

/// An individual test entry with status icon, name, duration, and optional error.
#[derive(IntoElement)]
pub struct Test {
    pub(crate) name: SharedString,
    pub(crate) status: TestStatus,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) error: Option<TestError>,
}

impl Test {
    pub fn new(name: impl Into<SharedString>, status: TestStatus) -> Self {
        Self {
            name: name.into(),
            status,
            duration_ms: None,
            error: None,
        }
    }

    /// Create a `Test` from a `TestCase` data struct.
    pub fn from_case(case: &TestCase) -> Self {
        let mut t = Self::new(case.name.clone(), case.status);
        t.duration_ms = case.duration_ms;
        if let Some(ref msg) = case.error_message {
            let mut err = TestError::new(msg.clone());
            if let Some(ref stack) = case.error_stack {
                err = err.stack(stack.clone());
            }
            t.error = Some(err);
        }
        t
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn error(mut self, error: TestError) -> Self {
        self.error = Some(error);
        self
    }
}

impl RenderOnce for Test {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (icon_name, icon_color) = status_icon_and_color(self.status, theme);

        let mut col = div().flex().flex_col();

        // Main row: icon + name (flex_1 spacer pushes duration to the
        // trailing edge). Mirrors Xcode Test Navigator where the run
        // duration lives in a right-aligned column — keeps the test
        // name column scannable while preserving the at-a-glance timing.
        let mut row = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(Icon::new(icon_name).size(px(12.0)).color(icon_color))
            .child(
                div()
                    .flex_1()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .child(self.name),
            );

        if let Some(ms) = self.duration_ms {
            row = row.child(
                div()
                    .flex_none()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .font(theme.mono_font())
                    .child(format_duration(ms)),
            );
        }

        col = col.child(row);

        if let Some(error) = self.error {
            col = col.child(error);
        }

        col
    }
}

// ─── TestResultsHeader ──────────────────────────────────────────────────────

/// Header section with test-tube icon, summary badges, and optional duration.
#[derive(IntoElement)]
pub struct TestResultsHeader {
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) skipped: usize,
    pub(crate) running: usize,
    pub(crate) total_duration_ms: Option<u64>,
}

impl TestResultsHeader {
    pub fn new(summary: &TestSummary) -> Self {
        Self {
            passed: summary.passed,
            failed: summary.failed,
            skipped: summary.skipped,
            running: summary.running,
            total_duration_ms: summary.duration_ms,
        }
    }

    pub fn total_duration_ms(mut self, ms: u64) -> Self {
        self.total_duration_ms = Some(ms);
        self
    }
}

impl RenderOnce for TestResultsHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut header = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(
                Icon::new(IconName::TestTube)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(Badge::new(format!("{} passed", self.passed)).variant(BadgeVariant::Success))
            .child(
                Badge::new(format!("{} failed", self.failed)).variant(if self.failed > 0 {
                    BadgeVariant::Error
                } else {
                    BadgeVariant::Muted
                }),
            )
            .child(
                Badge::new(format!("{} skipped", self.skipped)).variant(if self.skipped > 0 {
                    BadgeVariant::Warning
                } else {
                    BadgeVariant::Muted
                }),
            );

        if self.running > 0 {
            header = header
                .child(Badge::new(format!("{} running", self.running)).variant(BadgeVariant::Info));
        }

        if let Some(ms) = self.total_duration_ms {
            header = header.child(
                div()
                    .ml_auto()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(format_duration(ms)),
            );
        }

        header
    }
}

// ─── TestResultsProgress ────────────────────────────────────────────────────

/// Progress bar derived from test summary pass rate.
#[derive(IntoElement)]
pub struct TestResultsProgress {
    pub(crate) pass_rate: f32,
    pub(crate) has_failures: bool,
}

impl TestResultsProgress {
    pub fn new(summary: &TestSummary) -> Self {
        Self {
            pass_rate: summary.pass_rate(),
            has_failures: summary.failed > 0,
        }
    }
}

impl RenderOnce for TestResultsProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        ProgressIndicator::new(self.pass_rate).color(if self.has_failures {
            theme.error
        } else {
            theme.success
        })
    }
}

// ─── TestSuite ──────────────────────────────────────────────────────────────

/// A collapsible test suite grouping.
///
/// Supports two usage modes:
/// - **Convenience builder**: `TestSuite::new(id).name("...").test(...)`
/// - **Compound composition**: `TestSuite::from_parts(id).trigger(...).child(...)`
///
/// **Note**: In debug builds, mixing convenience and compound APIs will panic.
/// In release builds, compound fields take precedence (convenience fields are ignored).
#[derive(IntoElement)]
pub struct TestSuite {
    id: ElementId,
    // Convenience fields
    pub(crate) name: Option<SharedString>,
    pub(crate) status: Option<TestStatus>,
    pub(crate) stats: Option<(usize, usize, usize)>,
    pub(crate) tests: Vec<AnyElement>,
    // Compound fields
    pub(crate) trigger: Option<AnyElement>,
    pub(crate) children: Vec<AnyElement>,
    // Shared fields
    pub(crate) is_open: bool,
    pub(crate) on_toggle: OnToggle,
}

impl TestSuite {
    /// Convenience constructor. Use `.name()`, `.test()` to add content.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: None,
            status: None,
            stats: None,
            tests: Vec::new(),
            trigger: None,
            children: Vec::new(),
            is_open: true,
            on_toggle: None,
        }
    }

    /// Compound constructor. Use `.trigger()` and `.child()` to compose.
    pub fn from_parts(id: impl Into<ElementId>) -> Self {
        Self::new(id)
    }

    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn status(mut self, status: TestStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set suite-level statistics (passed, failed, skipped).
    pub fn stats(mut self, passed: usize, failed: usize, skipped: usize) -> Self {
        self.stats = Some((passed, failed, skipped));
        self
    }

    /// Add a test entry (convenience API).
    pub fn test(mut self, test: impl IntoElement) -> Self {
        self.tests.push(test.into_any_element());
        self
    }

    /// Set the trigger element (compound API).
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// Add a child element (compound API).
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Set the open/closed state (default: `true`).
    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    /// Set a callback invoked when the suite header is clicked.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl TestSuite {
    /// Panics (debug only) if convenience and compound APIs are mixed.
    pub(crate) fn validate(&self) {
        debug_assert!(
            self.trigger.is_none() || self.name.is_none(),
            "TestSuite: set either `trigger` (compound) or `name` (convenience), not both"
        );
        debug_assert!(
            self.children.is_empty() || self.tests.is_empty(),
            "TestSuite: set either `child` (compound) or `test` (convenience), not both"
        );
    }
}

impl RenderOnce for TestSuite {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.validate();

        let theme = cx.theme();

        // Build the header
        let header_element = if let Some(trigger) = self.trigger {
            // Compound: user-provided trigger
            trigger
        } else {
            // Convenience: auto-build header from name/status/stats
            let suite_status = self.status.unwrap_or({
                if let Some((passed, failed, skipped)) = self.stats {
                    if failed > 0 {
                        TestStatus::Failed
                    } else if skipped > 0 && passed == 0 {
                        TestStatus::Skipped
                    } else {
                        TestStatus::Passed
                    }
                } else {
                    TestStatus::Passed
                }
            });
            let (suite_icon, suite_color) = status_icon_and_color(suite_status, theme);

            let mut h = div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .child(Icon::new(suite_icon).size(px(12.0)).color(suite_color));

            if let Some(ref name) = self.name {
                h = h.child(
                    div()
                        .text_style(TextStyle::Subheadline, theme)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.text)
                        .child(name.clone()),
                );
            }

            if let Some((passed, failed, skipped)) = self.stats {
                let mut stats_row = div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(format!("{} passed", passed));

                if failed > 0 {
                    stats_row = stats_row.child(
                        div()
                            .text_color(theme.error)
                            .child(format!("· {} failed", failed)),
                    );
                }

                if skipped > 0 {
                    stats_row = stats_row.child(
                        div()
                            .text_color(theme.warning)
                            .child(format!("· {} skipped", skipped)),
                    );
                }

                h = h.child(stats_row);
            }

            h.into_any_element()
        };

        // Build the body
        let body = {
            let items = if !self.children.is_empty() {
                self.children
            } else {
                self.tests
            };
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .pl(theme.spacing_sm)
                .children(items)
        };

        let mut collapsible =
            DisclosureGroup::new(self.id, header_element, body).open(self.is_open);

        if let Some(handler) = self.on_toggle {
            collapsible = collapsible.on_toggle(handler);
        }

        collapsible
    }
}
