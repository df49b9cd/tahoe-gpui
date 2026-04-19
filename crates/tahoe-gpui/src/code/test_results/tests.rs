//! Tests for the test results display components.

use super::{
    Test, TestCase, TestError, TestResults, TestResultsHeader, TestResultsProgress, TestStatus,
    TestSuite, TestSummary, compute_suite_status, format_duration,
};
use core::prelude::v1::test;
use gpui::div;

// -- TestStatus -----------------------------------------------------------

#[test]
fn test_status_equality() {
    assert_eq!(TestStatus::Passed, TestStatus::Passed);
    assert_ne!(TestStatus::Passed, TestStatus::Failed);
    assert_ne!(TestStatus::Failed, TestStatus::Skipped);
    assert_ne!(TestStatus::Skipped, TestStatus::Running);
}

// -- TestSummary ----------------------------------------------------------

#[test]
fn test_summary_from_tests() {
    let tests = vec![
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Failed),
        TestCase::new("c", TestStatus::Skipped),
        TestCase::new("d", TestStatus::Passed),
        TestCase::new("e", TestStatus::Running),
    ];
    let s = TestSummary::from_tests(&tests);
    assert_eq!(s.passed, 2);
    assert_eq!(s.failed, 1);
    assert_eq!(s.skipped, 1);
    assert_eq!(s.running, 1);
    assert_eq!(s.total(), 5);
}

#[test]
fn test_summary_pass_rate() {
    let tests = vec![
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Failed),
    ];
    let s = TestSummary::from_tests(&tests);
    assert!((s.pass_rate() - 0.5).abs() < 0.01);
}

#[test]
fn test_summary_pass_rate_excludes_running() {
    let tests = vec![
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Running),
    ];
    let s = TestSummary::from_tests(&tests);
    assert!((s.pass_rate() - 1.0).abs() < 0.01);
}

#[test]
fn test_summary_empty() {
    let s = TestSummary::from_tests(&[]);
    assert_eq!(s.total(), 0);
    assert_eq!(s.pass_rate(), 0.0);
    assert_eq!(s.duration_ms, None);
}

#[test]
fn test_summary_duration() {
    let tests = vec![
        TestCase::new("a", TestStatus::Passed).duration_ms(100),
        TestCase::new("b", TestStatus::Passed).duration_ms(200),
        TestCase::new("c", TestStatus::Failed),
    ];
    let s = TestSummary::from_tests(&tests);
    assert_eq!(s.duration_ms, Some(300));
}

// -- TestCase -------------------------------------------------------------

#[test]
fn test_case_with_suite() {
    let t = TestCase::new("test_foo", TestStatus::Passed).suite("my_suite");
    assert_eq!(t.suite.unwrap().as_ref(), "my_suite");
}

#[test]
fn test_case_error_stack() {
    let t = TestCase::new("fail", TestStatus::Failed)
        .error("assertion failed")
        .error_stack("at test.js:42\n  at run()");
    assert_eq!(t.error_message.unwrap().as_ref(), "assertion failed");
    assert!(t.error_stack.unwrap().as_ref().contains("test.js:42"));
}

// -- compute_suite_status -------------------------------------------------

#[test]
fn test_suite_status_running_takes_priority() {
    let cases = [
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Running),
    ];
    let refs: Vec<&TestCase> = cases.iter().collect();
    assert_eq!(compute_suite_status(&refs), TestStatus::Running);
}

#[test]
fn test_suite_status_failed_over_passed() {
    let cases = [
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Failed),
    ];
    let refs: Vec<&TestCase> = cases.iter().collect();
    assert_eq!(compute_suite_status(&refs), TestStatus::Failed);
}

#[test]
fn test_suite_status_all_skipped() {
    let cases = [
        TestCase::new("a", TestStatus::Skipped),
        TestCase::new("b", TestStatus::Skipped),
    ];
    let refs: Vec<&TestCase> = cases.iter().collect();
    assert_eq!(compute_suite_status(&refs), TestStatus::Skipped);
}

#[test]
fn test_suite_status_all_passed() {
    let cases = [
        TestCase::new("a", TestStatus::Passed),
        TestCase::new("b", TestStatus::Passed),
    ];
    let refs: Vec<&TestCase> = cases.iter().collect();
    assert_eq!(compute_suite_status(&refs), TestStatus::Passed);
}

// -- format_duration ------------------------------------------------------

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(42), "42ms");
    assert_eq!(format_duration(1500), "1.5s");
    assert_eq!(format_duration(65000), "1m 5s");
}

// -- TestError -------------------------------------------------------------

#[test]
fn test_error_defaults() {
    let err = TestError::new("assertion failed");
    assert_eq!(err.message.as_ref(), "assertion failed");
    assert!(err.stack.is_none());
}

#[test]
fn test_error_with_stack() {
    let err = TestError::new("expected true").stack("at test.js:42\n  at run()");
    assert_eq!(err.message.as_ref(), "expected true");
    assert!(err.stack.unwrap().as_ref().contains("test.js:42"));
}

// -- Test -----------------------------------------------------------------

#[test]
fn test_new_defaults() {
    let t = Test::new("login_test", TestStatus::Passed);
    assert_eq!(t.name.as_ref(), "login_test");
    assert_eq!(t.status, TestStatus::Passed);
    assert!(t.duration_ms.is_none());
    assert!(t.error.is_none());
}

#[test]
fn test_with_duration() {
    let t = Test::new("fast", TestStatus::Passed).duration_ms(42);
    assert_eq!(t.duration_ms, Some(42));
}

#[test]
fn test_with_error() {
    let t = Test::new("fail", TestStatus::Failed).error(TestError::new("bad").stack("line 1"));
    assert!(t.error.is_some());
    assert_eq!(t.error.as_ref().unwrap().message.as_ref(), "bad");
}

#[test]
fn test_from_case() {
    let case = TestCase::new("my_test", TestStatus::Failed)
        .duration_ms(100)
        .error("fail msg")
        .error_stack("stack here");
    let t = Test::from_case(&case);
    assert_eq!(t.name.as_ref(), "my_test");
    assert_eq!(t.status, TestStatus::Failed);
    assert_eq!(t.duration_ms, Some(100));
    assert!(t.error.is_some());
    assert_eq!(t.error.as_ref().unwrap().message.as_ref(), "fail msg");
    assert!(
        t.error
            .as_ref()
            .unwrap()
            .stack
            .as_ref()
            .unwrap()
            .as_ref()
            .contains("stack here")
    );
}

#[test]
fn test_from_case_no_error() {
    let case = TestCase::new("ok", TestStatus::Passed).duration_ms(10);
    let t = Test::from_case(&case);
    assert!(t.error.is_none());
    assert_eq!(t.duration_ms, Some(10));
}

// -- TestResultsHeader ----------------------------------------------------

#[test]
fn test_results_header_from_summary() {
    let summary = TestSummary {
        passed: 5,
        failed: 2,
        skipped: 1,
        running: 0,
        duration_ms: Some(3000),
    };
    let h = TestResultsHeader::new(&summary);
    assert_eq!(h.passed, 5);
    assert_eq!(h.failed, 2);
    assert_eq!(h.skipped, 1);
    assert_eq!(h.running, 0);
    assert_eq!(h.total_duration_ms, Some(3000));
}

#[test]
fn test_results_header_override_duration() {
    let summary = TestSummary {
        passed: 1,
        failed: 0,
        skipped: 0,
        running: 0,
        duration_ms: Some(100),
    };
    let h = TestResultsHeader::new(&summary).total_duration_ms(5000);
    assert_eq!(h.total_duration_ms, Some(5000));
}

// -- TestResultsProgress --------------------------------------------------

#[test]
fn test_results_progress_from_summary() {
    let summary = TestSummary {
        passed: 8,
        failed: 2,
        skipped: 0,
        running: 0,
        duration_ms: None,
    };
    let p = TestResultsProgress::new(&summary);
    assert!((p.pass_rate - 0.8).abs() < 0.01);
    assert!(p.has_failures);
}

#[test]
fn test_results_progress_no_failures() {
    let summary = TestSummary {
        passed: 5,
        failed: 0,
        skipped: 0,
        running: 0,
        duration_ms: None,
    };
    let p = TestResultsProgress::new(&summary);
    assert!((p.pass_rate - 1.0).abs() < 0.01);
    assert!(!p.has_failures);
}

// -- TestSuite ------------------------------------------------------------

#[test]
fn test_suite_defaults() {
    let suite = TestSuite::new("s1");
    assert!(suite.name.is_none());
    assert!(suite.status.is_none());
    assert!(suite.stats.is_none());
    assert!(suite.tests.is_empty());
    assert!(suite.trigger.is_none());
    assert!(suite.children.is_empty());
    assert!(suite.is_open);
    assert!(suite.on_toggle.is_none());
}

#[test]
fn test_suite_convenience_api() {
    let suite = TestSuite::new("s1")
        .name("Auth Tests")
        .status(TestStatus::Passed)
        .stats(3, 0, 0)
        .test(div())
        .test(div())
        .open(false);
    assert_eq!(suite.name.as_ref().map(|s| s.as_ref()), Some("Auth Tests"));
    assert_eq!(suite.status, Some(TestStatus::Passed));
    assert_eq!(suite.stats, Some((3, 0, 0)));
    assert_eq!(suite.tests.len(), 2);
    assert!(!suite.is_open);
}

#[test]
fn test_suite_compound_api() {
    let suite = TestSuite::from_parts("s1")
        .trigger(div())
        .child(div())
        .child(div())
        .open(true);
    assert!(suite.trigger.is_some());
    assert_eq!(suite.children.len(), 2);
    assert!(suite.is_open);
}

#[test]
fn test_suite_on_toggle() {
    let suite = TestSuite::new("s1").on_toggle(|_open, _w, _cx| {});
    assert!(suite.on_toggle.is_some());
}

// -- TestResults ----------------------------------------------------------

#[test]
fn test_results_defaults() {
    let r = TestResults::new("r1");
    assert!(r.summary.is_none());
    assert!(r.entries.is_empty());
    assert!(r.header.is_none());
    assert!(r.progress.is_none());
    assert!(r.children.is_empty());
}

#[test]
fn test_results_convenience_api() {
    let r = TestResults::new("r1")
        .summary(TestSummary {
            passed: 5,
            failed: 1,
            skipped: 0,
            running: 0,
            duration_ms: Some(1000),
        })
        .entry(div())
        .entry(div());
    assert!(r.summary.is_some());
    assert_eq!(r.entries.len(), 2);
}

#[test]
fn test_results_compound_api() {
    let r = TestResults::from_parts("r1")
        .header(div())
        .progress(div())
        .child(div())
        .child(div());
    assert!(r.header.is_some());
    assert!(r.progress.is_some());
    assert_eq!(r.children.len(), 2);
}

// -- Validation (should_panic) --------------------------------------------

#[test]
#[should_panic(expected = "TestSuite: set either `trigger` (compound) or `name` (convenience)")]
fn test_suite_panics_on_mixed_trigger_and_name() {
    TestSuite::new("s1").name("Auth").trigger(div()).validate();
}

#[test]
#[should_panic(expected = "TestSuite: set either `child` (compound) or `test` (convenience)")]
fn test_suite_panics_on_mixed_child_and_test() {
    TestSuite::new("s1").test(div()).child(div()).validate();
}

#[test]
#[should_panic(expected = "TestResults: set either `header` (compound) or `summary` (convenience)")]
fn test_results_panics_on_mixed_header_and_summary() {
    TestResults::new("r1")
        .summary(TestSummary {
            passed: 1,
            failed: 0,
            skipped: 0,
            running: 0,
            duration_ms: None,
        })
        .header(div())
        .validate();
}

#[test]
#[should_panic(expected = "TestResults: set either `child` (compound) or `entry` (convenience)")]
fn test_results_panics_on_mixed_child_and_entry() {
    TestResults::new("r1").entry(div()).child(div()).validate();
}
