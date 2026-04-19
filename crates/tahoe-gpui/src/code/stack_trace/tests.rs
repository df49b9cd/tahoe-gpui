//! Tests for stack trace parsing and subcomponents.

use super::{
    StackFrame, StackTraceActions, StackTraceContent, StackTraceError, StackTraceFrames,
    StackTraceHeader, StackTraceView, parse_stack_trace,
};
use core::prelude::v1::test;
use gpui::{ElementId, SharedString};
use std::rc::Rc;

#[test]
fn parse_js_error() {
    let trace = "TypeError: Cannot read property 'foo' of undefined\n    at Object.<anonymous> (/app/src/index.js:10:15)\n    at Module._compile (node:internal/modules/cjs/loader:1101:14)";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.error_type.as_deref(), Some("TypeError"));
    assert_eq!(parsed.frames.len(), 2);
    assert!(!parsed.frames[0].is_internal);
    assert!(parsed.frames[1].is_internal);
}

#[test]
fn parse_empty() {
    let parsed = parse_stack_trace("");
    assert!(parsed.frames.is_empty());
}

#[test]
fn parse_rust_panic() {
    let trace = "thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5', src/main.rs:10:5\n\
                 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace";
    let parsed = parse_stack_trace(trace);
    // The first line does not match "Error:" pattern, so error_type is None
    assert!(parsed.error_type.is_none());
    assert!(parsed.error_message.contains("panicked"));
    // No "at " prefixed frames in this format
    assert!(parsed.frames.is_empty());
}

#[test]
fn parse_python_traceback() {
    let trace = "NameError: name 'x' is not defined\n\
                 Traceback (most recent call last):\n\
                   File \"test.py\", line 1, in <module>";
    let parsed = parse_stack_trace(trace);
    // "NameError" does not end with "Error" literally... let's check
    // Actually the code checks prefix.ends_with("Error") so "NameError" should match
    assert_eq!(parsed.error_type.as_deref(), Some("NameError"));
    assert_eq!(parsed.error_message, "name 'x' is not defined");
    // Python traceback lines don't start with "at " so no frames parsed
    assert!(parsed.frames.is_empty());
}

#[test]
fn parse_js_error_with_internal_frames() {
    let trace = "TypeError: Cannot read property 'foo' of undefined\n\
                 at myFunction (/app/src/index.js:10:15)\n\
                 at Object.<anonymous> (/app/src/app.js:5:3)\n\
                 at Module._compile (node:internal/modules/cjs/loader:1101:14)\n\
                 at Module._extensions (node_modules/ts-node/dist/index.js:851:20)";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.error_type.as_deref(), Some("TypeError"));
    assert_eq!(parsed.frames.len(), 4);

    // First frame: user code
    assert_eq!(
        parsed.frames[0].function_name.as_deref(),
        Some("myFunction")
    );
    assert_eq!(
        parsed.frames[0].file_path.as_deref(),
        Some("/app/src/index.js")
    );
    assert_eq!(parsed.frames[0].line_number, Some(10));
    assert_eq!(parsed.frames[0].column_number, Some(15));
    assert!(!parsed.frames[0].is_internal);

    // Second frame: user code
    assert!(!parsed.frames[1].is_internal);

    // Third frame: node internal
    assert!(parsed.frames[2].is_internal);

    // Fourth frame: node_modules
    assert!(parsed.frames[3].is_internal);
}

#[test]
fn parse_frame_no_function_name() {
    let trace = "Error: something broke\n    at /app/src/index.js:42:7";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.frames.len(), 1);
    assert!(parsed.frames[0].function_name.is_none());
    assert_eq!(
        parsed.frames[0].file_path.as_deref(),
        Some("/app/src/index.js")
    );
    assert_eq!(parsed.frames[0].line_number, Some(42));
    assert_eq!(parsed.frames[0].column_number, Some(7));
}

#[test]
fn parse_deep_stack_trace() {
    let mut trace = "Error: deep stack\n".to_string();
    for i in 0..25 {
        trace.push_str(&format!(
            "    at func_{} (/app/src/deep.js:{}:1)\n",
            i,
            i + 1
        ));
    }
    let parsed = parse_stack_trace(&trace);
    assert_eq!(parsed.error_type.as_deref(), Some("Error"));
    assert_eq!(parsed.error_message, "deep stack");
    assert_eq!(parsed.frames.len(), 25);

    // Verify first and last frames
    assert_eq!(parsed.frames[0].function_name.as_deref(), Some("func_0"));
    assert_eq!(parsed.frames[0].line_number, Some(1));
    assert_eq!(parsed.frames[24].function_name.as_deref(), Some("func_24"));
    assert_eq!(parsed.frames[24].line_number, Some(25));
}

#[test]
fn parse_windows_path() {
    let trace =
        "Error: file not found\n    at readFile (C:\\Users\\dev\\project\\src\\main.js:15:8)";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.frames.len(), 1);
    assert_eq!(parsed.frames[0].function_name.as_deref(), Some("readFile"));
    assert_eq!(
        parsed.frames[0].file_path.as_deref(),
        Some("C:\\Users\\dev\\project\\src\\main.js")
    );
    assert_eq!(parsed.frames[0].line_number, Some(15));
    assert_eq!(parsed.frames[0].column_number, Some(8));
}

#[test]
fn parse_url_location() {
    let trace = "Error: fetch failed\n    at fetchData (http://example.com/bundle.js:100:20)";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.frames.len(), 1);
    assert_eq!(
        parsed.frames[0].file_path.as_deref(),
        Some("http://example.com/bundle.js")
    );
    assert_eq!(parsed.frames[0].line_number, Some(100));
    assert_eq!(parsed.frames[0].column_number, Some(20));
}

#[test]
fn parse_only_whitespace_lines_ignored() {
    let trace = "Error: oops\n   \n    at fn1 (/app/a.js:1:1)\n   \n";
    let parsed = parse_stack_trace(trace);
    assert_eq!(parsed.frames.len(), 1);
}

#[test]
fn parse_error_type_detection() {
    // "SyntaxError" ends with "Error"
    let parsed = parse_stack_trace("SyntaxError: Unexpected token");
    assert_eq!(parsed.error_type.as_deref(), Some("SyntaxError"));

    // "ReferenceError" ends with "Error"
    let parsed = parse_stack_trace("ReferenceError: x is not defined");
    assert_eq!(parsed.error_type.as_deref(), Some("ReferenceError"));

    // Just "Error"
    let parsed = parse_stack_trace("Error: generic");
    assert_eq!(parsed.error_type.as_deref(), Some("Error"));
}

#[test]
fn parse_non_error_first_line() {
    // First line without ": " gets no error_type
    let parsed = parse_stack_trace("some random text without colon");
    assert!(parsed.error_type.is_none());
    assert_eq!(parsed.error_message, "some random text without colon");
}

#[test]
fn parse_colon_but_not_error_prefix() {
    // Has ": " but prefix doesn't end with "Error"
    let parsed = parse_stack_trace("Warning: something happened");
    assert!(parsed.error_type.is_none());
    assert_eq!(parsed.error_message, "Warning: something happened");
}

#[test]
fn stack_frame_is_internal_detection() {
    let trace = "Error: test\n\
                 at a (node:fs:1:1)\n\
                 at b (/app/node_modules/pkg/index.js:2:2)\n\
                 at c (node:internal/modules/run:3:3)\n\
                 at d (/app/src/main.js:4:4)";
    let parsed = parse_stack_trace(trace);
    assert!(parsed.frames[0].is_internal); // node:fs
    assert!(parsed.frames[1].is_internal); // node_modules
    assert!(parsed.frames[2].is_internal); // internal/
    assert!(!parsed.frames[3].is_internal); // user code
}

#[test]
fn parsed_stack_trace_debug() {
    let parsed = parse_stack_trace("Error: test");
    let dbg = format!("{:?}", parsed);
    assert!(dbg.contains("Error"));
}

#[test]
fn stack_frame_clone() {
    let frame = StackFrame {
        function_name: Some("test".into()),
        file_path: Some("/app/test.js".into()),
        line_number: Some(1),
        column_number: Some(1),
        is_internal: false,
        raw: "at test (/app/test.js:1:1)".into(),
    };
    let cloned = frame.clone();
    assert_eq!(cloned.function_name, frame.function_name);
    assert_eq!(cloned.file_path, frame.file_path);
    assert_eq!(cloned.line_number, frame.line_number);
}

fn make_view(trace: &str) -> StackTraceView {
    let parsed = parse_stack_trace(trace);
    let frames_rc = Rc::new(parsed.frames.clone());
    StackTraceView {
        element_id: ElementId::from(SharedString::from("test-stack")),
        trace: parsed,
        raw: trace.to_string(),
        is_expanded: false,
        controlled_open: None,
        on_open_change: None,
        on_file_click: None,
        frames_rc,
        copy_button: None,
        show_internal_frames: true,
        max_height: 400.0,
    }
}

#[test]
fn default_open_sets_expanded() {
    let mut view = make_view("Error: test");
    assert!(!view.is_expanded);
    view.set_default_open(true);
    assert!(view.is_expanded);
}

#[test]
fn show_internal_frames_default_true() {
    let view = make_view("Error: test");
    assert!(view.show_internal_frames);
}

#[test]
fn show_internal_frames_filtering() {
    let trace = "Error: test\n\
                 at userFn (/app/src/main.js:1:1)\n\
                 at internal (node:fs:2:2)\n\
                 at dep (node_modules/pkg/index.js:3:3)";
    let mut view = make_view(trace);
    assert_eq!(view.trace.frames.len(), 3);

    // With show_internal_frames = true, all frames are visible
    let visible: Vec<_> = view
        .trace
        .frames
        .iter()
        .filter(|f| view.show_internal_frames || !f.is_internal)
        .collect();
    assert_eq!(visible.len(), 3);

    // With show_internal_frames = false, only user frames are visible
    view.set_show_internal_frames(false);
    let visible: Vec<_> = view
        .trace
        .frames
        .iter()
        .filter(|f| view.show_internal_frames || !f.is_internal)
        .collect();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].function_name.as_deref(), Some("userFn"));
}

#[test]
fn max_height_default() {
    let view = make_view("Error: test");
    assert_eq!(view.max_height, 400.0);
}

#[test]
fn max_height_custom() {
    let mut view = make_view("Error: test");
    view.set_max_height(200.0);
    assert_eq!(view.max_height, 200.0);
}

#[test]
fn controlled_open_overrides_expanded() {
    let mut view = make_view("Error: test");
    view.is_expanded = false;
    view.controlled_open = Some(true);
    let effective = view.controlled_open.unwrap_or(view.is_expanded);
    assert!(effective);
}

#[test]
fn controlled_open_none_falls_back() {
    let mut view = make_view("Error: test");
    view.is_expanded = true;
    view.controlled_open = None;
    let effective = view.controlled_open.unwrap_or(view.is_expanded);
    assert!(effective);

    view.is_expanded = false;
    let effective = view.controlled_open.unwrap_or(view.is_expanded);
    assert!(!effective);
}

// --- Subcomponent tests ---

#[test]
fn stack_trace_header_default_is_open_false() {
    let header = StackTraceHeader::new("test message");
    assert!(!header.is_open);
    assert!(header.error_type.is_none());
}

#[test]
fn stack_trace_header_builder_chain() {
    let header = StackTraceHeader::new("msg")
        .error_type("TypeError")
        .is_open(true);
    assert!(header.is_open);
    assert_eq!(
        header.error_type.as_ref().map(|s| s.as_ref()),
        Some("TypeError")
    );
    assert_eq!(header.message.as_ref(), "msg");
}

#[test]
fn stack_trace_error_stores_type() {
    let error = StackTraceError::new("something broke").error_type("ReferenceError");
    assert_eq!(
        error.error_type.as_ref().map(|s| s.as_ref()),
        Some("ReferenceError")
    );
    assert_eq!(error.message.as_ref(), "something broke");
}

#[test]
fn stack_trace_error_none_type() {
    let error = StackTraceError::new("just a message");
    assert!(error.error_type.is_none());
}

#[test]
fn stack_trace_frames_defaults() {
    let frames = StackTraceFrames::new(vec![]);
    assert!(frames.show_internal_frames);
    assert!(frames.on_file_click.is_none());
}

#[test]
fn stack_trace_frames_hide_internal() {
    let frames = StackTraceFrames::new(vec![]).show_internal_frames(false);
    assert!(!frames.show_internal_frames);
}

#[test]
fn stack_trace_content_default_max_height() {
    let content = StackTraceContent::new();
    assert_eq!(content.max_height, 400.0);
}

#[test]
fn stack_trace_content_custom_max_height() {
    let content = StackTraceContent::new().max_height(250.0);
    assert_eq!(content.max_height, 250.0);
}

#[test]
fn stack_trace_actions_empty() {
    let actions = StackTraceActions::new();
    assert!(actions.children.is_empty());
}
