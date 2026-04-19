//! Test results display component with compound sub-components.
//!
//! Provides a collapsible test view with summary badges, progress bar,
//! collapsible suites, and per-test details. Supports both a convenience
//! builder API and compound composition.
//!
//! # Convenience builder
//! ```ignore
//! TestResults::new("results-1")
//!     .summary(TestSummary { passed: 5, failed: 1, skipped: 0, running: 0, duration_ms: Some(1200) })
//!     .entry(
//!         TestSuite::new("suite-auth")
//!             .name("Authentication")
//!             .status(TestStatus::Passed)
//!             .stats(3, 0, 0)
//!             .test(Test::new("login", TestStatus::Passed).duration_ms(42))
//!     )
//! ```
//!
//! # Compound composition
//! ```ignore
//! let summary = TestSummary { passed: 1, failed: 0, skipped: 0, running: 0, duration_ms: Some(42) };
//!
//! TestResults::from_parts("results-1")
//!     .header(TestResultsHeader::new(&summary))
//!     .progress(TestResultsProgress::new(&summary))
//!     .child(
//!         TestSuite::from_parts("suite-auth")
//!             .trigger(div().child("Custom header"))
//!             .child(Test::new("login", TestStatus::Passed))
//!     )
//! ```

pub mod subcomponents;
pub mod types;

pub use subcomponents::*;
pub use types::*;

use crate::foundations::theme::ActiveTheme;
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, ElementId, SharedString, Window, div};
use std::collections::HashMap;

use types::compute_suite_status;

// ─── TestResults ────────────────────────────────────────────────────────────

/// Top-level test results container.
///
/// Supports two usage modes:
/// - **Convenience builder**: `TestResults::new(id).summary(s).entry(...)`
/// - **Compound composition**: `TestResults::from_parts(id).header(...).progress(...).child(...)`
///
/// **Note**: In debug builds, mixing convenience and compound APIs will panic.
/// In release builds, compound fields take precedence (convenience fields are ignored).
#[derive(IntoElement)]
pub struct TestResults {
    id: ElementId,
    // Convenience fields
    pub(crate) summary: Option<TestSummary>,
    pub(crate) entries: Vec<AnyElement>,
    // Compound fields
    pub(crate) header: Option<AnyElement>,
    pub(crate) progress: Option<AnyElement>,
    pub(crate) children: Vec<AnyElement>,
}

impl TestResults {
    /// Convenience constructor. Use `.summary()`, `.entry()` to add content.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            summary: None,
            entries: Vec::new(),
            header: None,
            progress: None,
            children: Vec::new(),
        }
    }

    /// Compound constructor. Use `.header()`, `.progress()`, `.child()` to compose.
    pub fn from_parts(id: impl Into<ElementId>) -> Self {
        Self::new(id)
    }

    pub fn summary(mut self, summary: TestSummary) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Add an entry (suite or flat test) to the results (convenience API).
    pub fn entry(mut self, entry: impl IntoElement) -> Self {
        self.entries.push(entry.into_any_element());
        self
    }

    /// Set the header element (compound API).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Set the progress element (compound API).
    pub fn progress(mut self, progress: impl IntoElement) -> Self {
        self.progress = Some(progress.into_any_element());
        self
    }

    /// Add a child element (compound API).
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl TestResults {
    /// Panics (debug only) if convenience and compound APIs are mixed.
    pub(crate) fn validate(&self) {
        debug_assert!(
            self.header.is_none() || self.summary.is_none(),
            "TestResults: set either `header` (compound) or `summary` (convenience), not both"
        );
        debug_assert!(
            self.children.is_empty() || self.entries.is_empty(),
            "TestResults: set either `child` (compound) or `entry` (convenience), not both"
        );
    }
}

impl RenderOnce for TestResults {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.validate();

        let theme = cx.theme();

        let mut container = crate::foundations::materials::card_surface(theme)
            .id(self.id)
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm);

        // Header
        if let Some(header) = self.header {
            container = container.child(header);
        } else if let Some(ref summary) = self.summary {
            container = container.child(TestResultsHeader::new(summary));
        }

        // Progress bar
        if let Some(progress) = self.progress {
            container = container.child(progress);
        } else if let Some(ref summary) = self.summary {
            container = container.child(TestResultsProgress::new(summary));
        }

        // Content
        let items = if !self.children.is_empty() {
            self.children
        } else {
            self.entries
        };

        if !items.is_empty() {
            container = container.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_xs)
                    .children(items),
            );
        }

        container
    }
}

// ─── TestResultsView (stateful wrapper) ─────────────────────────────────────

/// Stateful test results display that manages suite open/close state internally.
///
/// For the stateless compound alternative, see [`TestResults`].
pub struct TestResultsView {
    element_id: ElementId,
    tests: Vec<TestCase>,
    total_duration_ms: Option<u64>,
    suite_open_state: HashMap<SharedString, bool>,
    default_open: bool,
}

impl TestResultsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let _ = cx;
        Self {
            element_id: next_element_id("test-results"),
            tests: Vec::new(),
            total_duration_ms: None,
            suite_open_state: HashMap::new(),
            default_open: true,
        }
    }

    /// Replace the test cases and refresh the view.
    pub fn set_tests(&mut self, tests: Vec<TestCase>, cx: &mut Context<Self>) {
        for test in &tests {
            if let Some(suite) = &test.suite {
                self.suite_open_state
                    .entry(suite.clone())
                    .or_insert(self.default_open);
            }
        }
        self.tests = tests;
        cx.notify();
    }

    /// Set an explicit total duration (e.g. for parallel suites where sum != wall time).
    pub fn set_total_duration(&mut self, ms: u64, cx: &mut Context<Self>) {
        self.total_duration_ms = Some(ms);
        cx.notify();
    }

    /// Set whether new suites default to open or closed.
    pub fn set_default_open(&mut self, default_open: bool) {
        self.default_open = default_open;
    }

    /// Group tests by suite, preserving insertion order.
    fn grouped_tests(&self) -> Vec<(Option<SharedString>, Vec<&TestCase>)> {
        let mut suites: Vec<(Option<SharedString>, Vec<&TestCase>)> = Vec::new();
        for test in &self.tests {
            let suite_name = test.suite.clone();
            if let Some(group) = suites.iter_mut().find(|(s, _)| *s == suite_name) {
                group.1.push(test);
            } else {
                suites.push((suite_name, vec![test]));
            }
        }
        suites
    }
}

impl Render for TestResultsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut summary = TestSummary::from_tests(&self.tests);
        if let Some(ms) = self.total_duration_ms {
            summary.duration_ms = Some(ms);
        }

        let mut results = TestResults::new(self.element_id.clone()).summary(summary);

        for (suite_name, suite_tests) in self.grouped_tests() {
            if let Some(name) = suite_name {
                let is_open = *self
                    .suite_open_state
                    .get(&name)
                    .unwrap_or(&self.default_open);

                let suite_passed = suite_tests
                    .iter()
                    .filter(|t| t.status == TestStatus::Passed)
                    .count();
                let suite_failed = suite_tests
                    .iter()
                    .filter(|t| t.status == TestStatus::Failed)
                    .count();
                let suite_skipped = suite_tests
                    .iter()
                    .filter(|t| t.status == TestStatus::Skipped)
                    .count();
                let suite_status = compute_suite_status(&suite_tests);

                let entity = cx.entity().clone();
                let suite_key = name.clone();

                let mut suite = TestSuite::new(SharedString::from(format!("suite-{}", name)))
                    .name(name)
                    .status(suite_status)
                    .stats(suite_passed, suite_failed, suite_skipped)
                    .open(is_open)
                    .on_toggle(move |new_state, _window, cx| {
                        entity.update(cx, |this, cx| {
                            this.suite_open_state.insert(suite_key.clone(), new_state);
                            cx.notify();
                        });
                    });

                for t in suite_tests {
                    suite = suite.test(Test::from_case(t));
                }

                results = results.entry(suite);
            } else {
                // Unnamed tests rendered flat
                for t in suite_tests {
                    results = results.entry(Test::from_case(t));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod gpui_tests {
    use super::{TestCase, TestResultsView, TestStatus};
    use crate::test_helpers::helpers::setup_test_window;
    use gpui::SharedString;

    #[gpui::test]
    async fn test_results_view_empty(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TestResultsView::new(cx));
        handle.update_in(cx, |view, _window, _cx| {
            let groups = view.grouped_tests();
            assert!(groups.is_empty());
        });
    }

    #[gpui::test]
    async fn test_results_view_grouped_by_suite(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TestResultsView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_tests(
                vec![
                    TestCase::new("a", TestStatus::Passed).suite("s1"),
                    TestCase::new("b", TestStatus::Failed).suite("s1"),
                    TestCase::new("c", TestStatus::Passed).suite("s2"),
                ],
                cx,
            );
            let groups = view.grouped_tests();
            assert_eq!(groups.len(), 2);
            assert_eq!(groups[0].0.as_ref().map(|s| s.as_ref()), Some("s1"));
            assert_eq!(groups[0].1.len(), 2);
            assert_eq!(groups[1].0.as_ref().map(|s| s.as_ref()), Some("s2"));
            assert_eq!(groups[1].1.len(), 1);
        });
    }

    #[gpui::test]
    async fn test_results_view_unnamed_tests(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TestResultsView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_tests(
                vec![
                    TestCase::new("a", TestStatus::Passed),
                    TestCase::new("b", TestStatus::Failed),
                ],
                cx,
            );
            let groups = view.grouped_tests();
            assert_eq!(groups.len(), 1);
            assert!(groups[0].0.is_none());
            assert_eq!(groups[0].1.len(), 2);
        });
    }

    #[gpui::test]
    async fn test_results_view_mixed_named_unnamed(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TestResultsView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_tests(
                vec![
                    TestCase::new("a", TestStatus::Passed),
                    TestCase::new("b", TestStatus::Passed).suite("s1"),
                    TestCase::new("c", TestStatus::Failed),
                ],
                cx,
            );
            let groups = view.grouped_tests();
            // "a" and "c" both have suite=None, so they group together
            assert_eq!(groups.len(), 2);
            assert!(groups[0].0.is_none());
            assert_eq!(groups[0].1.len(), 2);
            assert_eq!(groups[1].0.as_ref().map(|s| s.as_ref()), Some("s1"));
        });
    }

    #[gpui::test]
    async fn test_results_view_default_open(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TestResultsView::new(cx));
        handle.update_in(cx, |view, _window, cx| {
            view.set_default_open(false);
            view.set_tests(vec![TestCase::new("a", TestStatus::Passed).suite("s1")], cx);
            assert_eq!(
                view.suite_open_state.get(&SharedString::from("s1")),
                Some(&false)
            );
        });
    }
}
