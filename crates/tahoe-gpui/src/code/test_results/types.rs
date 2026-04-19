//! Data types for test results display.

use crate::foundations::icons::IconName;
use crate::foundations::theme::TahoeTheme;
use gpui::SharedString;

// -- Data types ---------------------------------------------------------------

/// Status of a single test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Running,
}

/// A single test case (data).
pub struct TestCase {
    pub name: SharedString,
    pub status: TestStatus,
    pub error_message: Option<SharedString>,
    pub error_stack: Option<SharedString>,
    pub duration_ms: Option<u64>,
    /// Optional suite name for grouping.
    pub suite: Option<SharedString>,
}

impl TestCase {
    pub fn new(name: impl Into<SharedString>, status: TestStatus) -> Self {
        Self {
            name: name.into(),
            status,
            error_message: None,
            error_stack: None,
            duration_ms: None,
            suite: None,
        }
    }

    pub fn suite(mut self, suite: impl Into<SharedString>) -> Self {
        self.suite = Some(suite.into());
        self
    }

    pub fn error(mut self, msg: impl Into<SharedString>) -> Self {
        self.error_message = Some(msg.into());
        self
    }

    pub fn error_stack(mut self, stack: impl Into<SharedString>) -> Self {
        self.error_stack = Some(stack.into());
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }
}

/// Test suite summary.
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub running: usize,
    pub duration_ms: Option<u64>,
}

impl TestSummary {
    pub fn from_tests(tests: &[TestCase]) -> Self {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut running = 0;
        let mut total_ms: u64 = 0;
        let mut has_duration = false;
        for t in tests {
            match t.status {
                TestStatus::Passed => passed += 1,
                TestStatus::Failed => failed += 1,
                TestStatus::Skipped => skipped += 1,
                TestStatus::Running => running += 1,
            }
            if let Some(ms) = t.duration_ms {
                total_ms += ms;
                has_duration = true;
            }
        }
        Self {
            passed,
            failed,
            skipped,
            running,
            duration_ms: if has_duration { Some(total_ms) } else { None },
        }
    }

    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped + self.running
    }

    /// Pass rate over completed tests only (excludes running).
    pub(crate) fn pass_rate(&self) -> f32 {
        let completed = self.passed + self.failed + self.skipped;
        if completed == 0 {
            0.0
        } else {
            self.passed as f32 / completed as f32
        }
    }
}

// -- Helpers ------------------------------------------------------------------

/// Compute overall status for a group of tests.
pub(crate) fn compute_suite_status(tests: &[&TestCase]) -> TestStatus {
    if tests.iter().any(|t| t.status == TestStatus::Running) {
        TestStatus::Running
    } else if tests.iter().any(|t| t.status == TestStatus::Failed) {
        TestStatus::Failed
    } else if tests.iter().all(|t| t.status == TestStatus::Skipped) {
        TestStatus::Skipped
    } else {
        TestStatus::Passed
    }
}

pub(crate) fn status_icon_and_color(
    status: TestStatus,
    theme: &TahoeTheme,
) -> (IconName, gpui::Hsla) {
    match status {
        TestStatus::Passed => (IconName::Check, theme.success),
        TestStatus::Failed => (IconName::X, theme.error),
        TestStatus::Skipped => (IconName::Minus, theme.warning),
        TestStatus::Running => (IconName::Loader, theme.info),
    }
}

/// Format a duration in milliseconds for display.
pub(crate) fn format_duration(ms: u64) -> String {
    if ms >= 60_000 {
        format!("{}m {}s", ms / 60_000, (ms % 60_000) / 1000)
    } else if ms >= 1_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
