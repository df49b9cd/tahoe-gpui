//! JSX Preview component with streaming support and syntax highlighting.
//!
//! Since GPUI has no iframe/webview, this renders JSX as syntax-highlighted
//! source code with a preview header, streaming indicator, and "Open in Browser"
//! fallback that generates a temp HTML page with CDN-loaded React.

use std::collections::HashMap;

use crate::callback_types::OnClick;
use crate::components::content::badge::Badge;
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use crate::markdown::code_block::CodeBlockView;
use gpui::prelude::*;
use gpui::{AnyElement, App, ClickEvent, ElementId, SharedString, Window, div, px};
use serde_json::Value;

/// HTML void elements that never have closing tags.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Error information for JSX preview failures.
#[derive(Debug, Clone)]
pub struct JsxPreviewError {
    pub message: String,
}

/// A JSX preview component that displays syntax-highlighted JSX code
/// with streaming support and error handling.
///
/// Since GPUI has no webview/iframe, this renders JSX as highlighted
/// source code with a preview header, streaming indicator, and
/// "Open in Browser" fallback.
#[allow(clippy::type_complexity)]
pub struct JsxPreview {
    jsx: String,
    is_streaming: bool,
    error: Option<JsxPreviewError>,
    components: Vec<SharedString>,
    bindings: HashMap<String, Value>,
    element_id: ElementId,
    on_error: Option<Box<dyn Fn(&JsxPreviewError, &mut Window, &mut App) + 'static>>,
    render_error:
        Option<Box<dyn Fn(&JsxPreviewError, &mut Window, &mut App) -> AnyElement + 'static>>,
}

impl JsxPreview {
    pub fn new(jsx: impl Into<String>, cx: &mut Context<Self>) -> Self {
        let _ = cx;
        Self {
            jsx: jsx.into(),
            is_streaming: false,
            error: None,
            components: Vec::new(),
            bindings: HashMap::new(),
            element_id: next_element_id("jsx-preview"),
            on_error: None,
            render_error: None,
        }
    }

    /// Replace the entire JSX content.
    pub fn set_jsx(&mut self, jsx: impl Into<String>, cx: &mut Context<Self>) {
        self.jsx = jsx.into();
        cx.notify();
    }

    /// Append a streaming delta to the JSX content.
    pub fn push_delta(&mut self, delta: &str, cx: &mut Context<Self>) {
        self.jsx.push_str(delta);
        self.is_streaming = true;
        cx.notify();
    }

    /// Mark streaming as complete.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        self.is_streaming = false;
        cx.notify();
    }

    /// Set whether content is actively streaming.
    pub fn set_streaming(&mut self, streaming: bool, cx: &mut Context<Self>) {
        self.is_streaming = streaming;
        cx.notify();
    }

    /// Set an error state. Fires the on_error callback if registered.
    pub fn set_error(
        &mut self,
        error: JsxPreviewError,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.error = Some(error.clone());
        if let Some(ref cb) = self.on_error {
            cb(self.error.as_ref().unwrap(), window, cx);
        }
        cx.notify();
    }

    /// Clear the error state.
    pub fn clear_error(&mut self, cx: &mut Context<Self>) {
        self.error = None;
        cx.notify();
    }

    /// Register an error callback.
    pub fn set_on_error(
        &mut self,
        handler: impl Fn(&JsxPreviewError, &mut Window, &mut App) + 'static,
    ) {
        self.on_error = Some(Box::new(handler));
    }

    /// Register a custom error rendering function.
    pub fn set_render_error(
        &mut self,
        f: impl Fn(&JsxPreviewError, &mut Window, &mut App) -> AnyElement + 'static,
    ) {
        self.render_error = Some(Box::new(f));
    }

    /// Set custom component names available in the JSX scope.
    ///
    /// Panics if any name is not a valid JavaScript identifier.
    pub fn set_components(&mut self, components: Vec<SharedString>, cx: &mut Context<Self>) {
        assert!(
            components.iter().all(|n| is_valid_js_identifier(n)),
            "set_components: all names must be valid JavaScript identifiers, got: {:?}",
            components,
        );
        self.components = components;
        cx.notify();
    }

    /// Set variable bindings available in the JSX scope.
    pub fn set_bindings(&mut self, bindings: HashMap<String, Value>, cx: &mut Context<Self>) {
        self.bindings = bindings;
        cx.notify();
    }

    /// Get the displayable JSX. When streaming, auto-closes unclosed tags.
    fn display_jsx(&self) -> String {
        if self.is_streaming {
            close_unclosed_tags(&self.jsx)
        } else {
            self.jsx.clone()
        }
    }
}

impl Render for JsxPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let display_code = self.display_jsx();
        let is_streaming = self.is_streaming;
        let error = self.error.clone();
        let components = self.components.clone();

        // Extract theme values before any mutable borrows of cx.
        let surface = theme.surface;
        let radius_lg = theme.radius_lg;
        let border = theme.border;

        let jsx_for_browser = display_code.clone();
        let components_for_browser = self.components.clone();
        let bindings_for_browser = self.bindings.clone();

        let header = JsxPreviewHeader::new(self.element_id.clone())
            .is_streaming(is_streaming)
            .components(components)
            .on_open_browser(move |_event, _window, cx| {
                open_jsx_in_browser(
                    &jsx_for_browser,
                    &components_for_browser,
                    &bindings_for_browser,
                    cx,
                );
            });

        let mut content = JsxPreviewContent::new(display_code);
        if let Some(err) = error {
            if let Some(ref render_fn) = self.render_error {
                content = content.error_element(render_fn(&err, window, cx));
            } else {
                content = content.error(err);
            }
        }

        div()
            .flex()
            .flex_col()
            .bg(surface)
            .rounded(radius_lg)
            .border_1()
            .border_color(border)
            .overflow_hidden()
            .child(header)
            .child(content)
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/// Header bar for the JSX preview, showing title, badges, streaming indicator,
/// and "Open in Browser" button.
#[derive(IntoElement)]
pub struct JsxPreviewHeader {
    element_id: ElementId,
    is_streaming: bool,
    components: Vec<SharedString>,
    on_open_browser: OnClick,
}

impl JsxPreviewHeader {
    pub fn new(element_id: ElementId) -> Self {
        Self {
            element_id,
            is_streaming: false,
            components: Vec::new(),
            on_open_browser: None,
        }
    }

    pub fn is_streaming(mut self, streaming: bool) -> Self {
        self.is_streaming = streaming;
        self
    }

    pub fn components(mut self, components: Vec<SharedString>) -> Self {
        self.components = components;
        self
    }

    pub fn on_open_browser(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_browser = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for JsxPreviewHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = _cx.theme();

        let mut header_left = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(
                Icon::new(IconName::Code)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .child("JSX Preview"),
            )
            .child(Badge::new("JSX"));

        for name in &self.components {
            header_left = header_left.child(Badge::new(name.clone()));
        }

        if self.is_streaming {
            header_left = header_left.child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .child(
                        div()
                            .size(px(8.0))
                            .rounded_full()
                            .bg(theme.success)
                            .flex_shrink_0(),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child("Streaming"),
                    ),
            );
        }

        let mut header_right = div().flex().items_center().gap(theme.spacing_xs);

        if let Some(on_open) = self.on_open_browser {
            header_right = header_right.child(
                Button::new(self.element_id)
                    .label("Open in Browser")
                    .icon(Icon::new(IconName::Globe))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .on_click(move |event, window, cx| on_open(event, window, cx)),
            );
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .border_b_1()
            .border_color(theme.border)
            .child(header_left)
            .child(header_right)
    }
}

/// Content area for the JSX preview, showing an optional error panel
/// followed by syntax-highlighted JSX code.
#[derive(IntoElement)]
pub struct JsxPreviewContent {
    jsx: String,
    error: Option<JsxPreviewError>,
    error_element: Option<AnyElement>,
}

impl JsxPreviewContent {
    pub fn new(jsx: impl Into<String>) -> Self {
        Self {
            jsx: jsx.into(),
            error: None,
            error_element: None,
        }
    }

    pub fn error(mut self, error: JsxPreviewError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn error_element(mut self, element: AnyElement) -> Self {
        self.error_element = Some(element);
        self
    }
}

impl RenderOnce for JsxPreviewContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut container = div().flex().flex_col();

        if let Some(element) = self.error_element {
            container = container.child(element);
        } else if let Some(err) = self.error {
            container = container.child(JsxPreviewErrorDisplay::new(err));
        }

        container.child(
            CodeBlockView::new(self.jsx)
                .language(Some("jsx".to_string()))
                .show_header(false),
        )
    }
}

/// Default error display panel for JSX preview errors.
#[derive(IntoElement)]
pub struct JsxPreviewErrorDisplay {
    error: JsxPreviewError,
}

impl JsxPreviewErrorDisplay {
    pub fn new(error: JsxPreviewError) -> Self {
        Self { error }
    }
}

impl RenderOnce for JsxPreviewErrorDisplay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = _cx.theme();

        div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.error.opacity(0.1))
            .border_b_1()
            .border_color(theme.border)
            .child(
                Icon::new(IconName::AlertTriangle)
                    .size(theme.icon_size_inline)
                    .color(theme.error),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.error)
                    .child(SharedString::from(self.error.message)),
            )
    }
}

/// Open JSX in the system browser by writing a temp HTML file with CDN React.
///
/// The file is written to a per-invocation UUID path under `$TMPDIR`
/// (rather than a shared `jsx-preview/preview.html`) and `chmod 0600`
/// is applied on Unix so the sandbox-escaped preview isn't world-readable.
/// Finding #17 in the HIG Code-surface audit tracks hardening the `file://`
/// path; a full `WKWebView`-equivalent loopback server is a larger
/// refactor and is deferred.
fn open_jsx_in_browser(
    jsx: &str,
    components: &[SharedString],
    bindings: &HashMap<String, Value>,
    cx: &mut App,
) {
    let mut preamble = String::new();

    // Inject stub components so the JSX can reference them without errors.
    for name in components {
        let escaped = name.replace('\'', "\\'").replace('\n', "\\n");
        preamble.push_str(&format!(
            "    const {name} = (props) => React.createElement('div', \
{{style: {{border: '1px dashed #ccc', padding: '8px'}}}}, \
props.children || '{escaped}');\n"
        ));
    }

    // Inject bindings as const declarations using safe JSON serialization.
    for (name, value) in bindings {
        let serialized = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
        preamble.push_str(&format!("    const {name} = {serialized};\n"));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <title>JSX Preview</title>
  <script src="https://unpkg.com/react@18/umd/react.production.min.js"></script>
  <script src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js"></script>
  <script src="https://unpkg.com/@babel/standalone/babel.min.js"></script>
  <style>body {{ font-family: system-ui, sans-serif; padding: 1rem; }}</style>
</head>
<body>
  <div id="root"></div>
  <script type="text/babel">
{preamble}    const root = ReactDOM.createRoot(document.getElementById('root'));
    root.render(<>{jsx_escaped}</>);
  </script>
</body>
</html>"#,
        preamble = preamble,
        jsx_escaped = jsx.replace('\\', "\\\\").replace('`', "\\`")
    );

    let Some(path) = write_preview_html(&html) else {
        return;
    };
    cx.open_url(&format!("file://{}", path.display()));
}

/// Write `html` to a per-session UUID path under `$TMPDIR/jsx-preview/`.
/// Returns the absolute path on success, `None` on any I/O failure.
///
/// On Unix the file is created with `chmod 0600` so it is not readable by
/// other local users — this is the `WKWebView`-sandbox-adjacent hardening
/// called out in finding #17. A separate directory per session would be
/// even tighter, but multiple previews sharing a parent directory is fine
/// because each file carries a distinct UUID.
fn write_preview_html(html: &str) -> Option<std::path::PathBuf> {
    use std::io::Write;

    let dir = std::env::temp_dir().join("jsx-preview");
    std::fs::create_dir_all(&dir).ok()?;

    // The filename must be unique per call so concurrent previews don't
    // stomp each other. `SystemTime`'s nanosecond counter is a cheap,
    // dependency-free substitute for a UUID and is sufficient for the
    // "don't collide with yourself" threat model.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = dir.join(format!("preview-{:x}-{}.html", nanos, std::process::id()));

    // On Unix we open the file with mode 0o600 so other local users cannot
    // read the page. On non-Unix targets `OpenOptions` ignores the mode
    // bit; those platforms fall back to the default temp-file permissions.
    let mut file = {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        opts.open(&path).ok()?
    };
    file.write_all(html.as_bytes()).ok()?;
    Some(path)
}

/// Check if a string is a valid JavaScript identifier.
fn is_valid_js_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Close unclosed HTML/JSX tags in a partial JSX string.
///
/// Uses a stack-based approach: scans for opening tags, closing tags,
/// and self-closing tags, then appends closing tags for any remaining
/// open elements. Handles string attributes, JSX expressions, void
/// elements, and fragments.
pub fn close_unclosed_tags(partial_jsx: &str) -> String {
    let chars: Vec<char> = partial_jsx.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut stack: Vec<String> = Vec::new();
    let mut brace_depth = 0;

    while i < len {
        let ch = chars[i];

        // Track JSX expression depth
        if brace_depth > 0 {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                '"' | '\'' | '`' => {
                    let quote = ch;
                    i += 1;
                    while i < len && chars[i] != quote {
                        if chars[i] == '\\' {
                            i += 1; // skip escaped char
                        }
                        i += 1;
                    }
                }
                _ => {}
            }
            i += 1;
            continue;
        }

        if ch == '{' {
            brace_depth += 1;
            i += 1;
            continue;
        }

        if ch == '<' {
            if i + 1 >= len {
                // Ends with bare '<' — nothing to close from this
                break;
            }

            let next = chars[i + 1];

            if next == '/' {
                // Closing tag: </TagName>
                i += 2;
                let tag_start = i;
                // Fragment closing </>
                if i < len && chars[i] == '>' {
                    // Close fragment
                    if let Some(pos) = stack.iter().rposition(|t| t.is_empty()) {
                        stack.remove(pos);
                    }
                    i += 1;
                    continue;
                }
                while i < len
                    && (chars[i].is_alphanumeric()
                        || chars[i] == '-'
                        || chars[i] == '_'
                        || chars[i] == '.')
                {
                    i += 1;
                }
                let tag_name: String = chars[tag_start..i].iter().collect();
                // Skip to '>'
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i < len {
                    i += 1; // skip '>'
                }
                // Pop matching tag from stack
                if let Some(pos) = stack
                    .iter()
                    .rposition(|t| t.eq_ignore_ascii_case(&tag_name))
                {
                    stack.remove(pos);
                }
            } else if next == '>' {
                // Fragment opening <>
                stack.push(String::new());
                i += 2;
            } else if next == '!' || next == '?' {
                // Comment or processing instruction — skip to '>'
                i += 2;
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
            } else if next.is_alphabetic() || next == '_' {
                // Opening tag: <TagName ...>
                i += 1;
                let tag_start = i;
                while i < len
                    && (chars[i].is_alphanumeric()
                        || chars[i] == '-'
                        || chars[i] == '_'
                        || chars[i] == '.')
                {
                    i += 1;
                }
                let tag_name: String = chars[tag_start..i].iter().collect();

                // Scan through attributes to find > or />
                let mut is_self_closing = false;
                while i < len {
                    match chars[i] {
                        '>' => {
                            i += 1;
                            break;
                        }
                        '/' if i + 1 < len && chars[i + 1] == '>' => {
                            is_self_closing = true;
                            i += 2;
                            break;
                        }
                        '"' | '\'' => {
                            let quote = chars[i];
                            i += 1;
                            while i < len && chars[i] != quote {
                                if chars[i] == '\\' {
                                    i += 1;
                                }
                                i += 1;
                            }
                            if i < len {
                                i += 1;
                            }
                        }
                        '{' => {
                            // JSX expression in attribute
                            let mut depth = 1;
                            i += 1;
                            while i < len && depth > 0 {
                                match chars[i] {
                                    '{' => depth += 1,
                                    '}' => depth -= 1,
                                    '"' | '\'' | '`' => {
                                        let q = chars[i];
                                        i += 1;
                                        while i < len && chars[i] != q {
                                            if chars[i] == '\\' {
                                                i += 1;
                                            }
                                            i += 1;
                                        }
                                    }
                                    _ => {}
                                }
                                i += 1;
                            }
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }

                // If we ran out of input inside the tag, it's a partial tag — don't push
                if i >= len && !is_self_closing {
                    // Ended mid-tag: don't add to stack (tag never opened)
                    break;
                }

                if !is_self_closing {
                    let lower = tag_name.to_ascii_lowercase();
                    if !VOID_ELEMENTS.contains(&lower.as_str()) {
                        stack.push(tag_name);
                    }
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // Append closing tags in reverse order
    if stack.is_empty() {
        return partial_jsx.to_string();
    }

    let mut result = partial_jsx.to_string();
    for tag in stack.into_iter().rev() {
        if tag.is_empty() {
            result.push_str("</>");
        } else {
            result.push_str(&format!("</{}>", tag));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{JsxPreview, JsxPreviewError, close_unclosed_tags};
    use core::prelude::v1::test;
    use std::collections::HashMap;

    // --- close_unclosed_tags tests ---

    #[test]
    fn empty_string() {
        assert_eq!(close_unclosed_tags(""), "");
    }

    #[test]
    fn complete_jsx() {
        assert_eq!(close_unclosed_tags("<div>hello</div>"), "<div>hello</div>");
    }

    #[test]
    fn single_unclosed() {
        assert_eq!(close_unclosed_tags("<div>hello"), "<div>hello</div>");
    }

    #[test]
    fn nested_unclosed() {
        assert_eq!(
            close_unclosed_tags("<div><span>text"),
            "<div><span>text</span></div>"
        );
    }

    #[test]
    fn self_closing_tags() {
        assert_eq!(close_unclosed_tags("<img /><div>x"), "<img /><div>x</div>");
    }

    #[test]
    fn void_elements() {
        assert_eq!(close_unclosed_tags("<br><div>x"), "<br><div>x</div>");
    }

    #[test]
    fn void_element_hr() {
        assert_eq!(close_unclosed_tags("<hr><p>text"), "<hr><p>text</p>");
    }

    #[test]
    fn partial_tag_mid_attribute() {
        // Ends inside an opening tag — don't push incomplete tag
        let result = close_unclosed_tags("<div><span className=\"foo");
        // div was opened, span was started but never closed its tag
        // The raw partial content is preserved, only div gets closed
        assert_eq!(result, "<div><span className=\"foo</div>");
    }

    #[test]
    fn fragment_unclosed() {
        assert_eq!(close_unclosed_tags("<><div>hi"), "<><div>hi</div></>");
    }

    #[test]
    fn fragment_complete() {
        assert_eq!(
            close_unclosed_tags("<><div>hi</div></>"),
            "<><div>hi</div></>"
        );
    }

    #[test]
    fn with_string_attributes() {
        assert_eq!(
            close_unclosed_tags(r#"<div className="foo"><span id='bar'>x"#),
            r#"<div className="foo"><span id='bar'>x</span></div>"#
        );
    }

    #[test]
    fn jsx_expression_with_gt() {
        assert_eq!(
            close_unclosed_tags("<div>{a > b ? 'yes' : 'no'}"),
            "<div>{a > b ? 'yes' : 'no'}</div>"
        );
    }

    #[test]
    fn mixed_closed_and_unclosed() {
        assert_eq!(
            close_unclosed_tags("<div><span>a</span><p>b"),
            "<div><span>a</span><p>b</p></div>"
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            close_unclosed_tags("<div><ul><li>item"),
            "<div><ul><li>item</li></ul></div>"
        );
    }

    #[test]
    fn already_complete_nested() {
        let input = "<div><span>a</span></div>";
        assert_eq!(close_unclosed_tags(input), input);
    }

    #[test]
    fn bare_text_no_tags() {
        assert_eq!(close_unclosed_tags("hello world"), "hello world");
    }

    #[test]
    fn jsx_expression_attribute() {
        assert_eq!(
            close_unclosed_tags("<Button onClick={() => {}}>Click"),
            "<Button onClick={() => {}}>Click</Button>"
        );
    }

    #[test]
    fn component_name_with_dot() {
        assert_eq!(
            close_unclosed_tags("<Modal.Header>title"),
            "<Modal.Header>title</Modal.Header>"
        );
    }

    // --- JsxPreviewError tests ---

    #[test]
    fn error_creation_and_clone() {
        let err = JsxPreviewError {
            message: "parse failed".into(),
        };
        let cloned = err.clone();
        assert_eq!(cloned.message, "parse failed");
    }

    // --- State logic tests ---

    #[test]
    fn display_jsx_closes_when_streaming() {
        // Simulate streaming state without gpui context
        let preview = JsxPreview {
            jsx: "<div><span>hi".into(),
            is_streaming: true,
            error: None,
            components: Vec::new(),
            bindings: HashMap::new(),
            element_id: gpui::ElementId::from(gpui::SharedString::from("test")),
            on_error: None,
            render_error: None,
        };
        assert_eq!(preview.display_jsx(), "<div><span>hi</span></div>");
    }

    #[test]
    fn display_jsx_raw_when_not_streaming() {
        let preview = JsxPreview {
            jsx: "<div><span>hi".into(),
            is_streaming: false,
            error: None,
            components: Vec::new(),
            bindings: HashMap::new(),
            element_id: gpui::ElementId::from(gpui::SharedString::from("test")),
            on_error: None,
            render_error: None,
        };
        assert_eq!(preview.display_jsx(), "<div><span>hi");
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::{JsxPreview, JsxPreviewError};
    use crate::test_helpers::helpers::setup_test_window;
    use gpui::SharedString;
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[gpui::test]
    async fn set_jsx_updates_content(cx: &mut gpui::TestAppContext) {
        let (handle, cx) =
            setup_test_window(cx, |_window, cx| JsxPreview::new("<div>initial</div>", cx));
        handle.update_in(cx, |preview, _window, cx| {
            preview.set_jsx("<span>updated</span>", cx);
            assert_eq!(preview.jsx, "<span>updated</span>");
        });
    }

    #[gpui::test]
    async fn push_delta_sets_streaming(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| JsxPreview::new(String::new(), cx));
        handle.update_in(cx, |preview, _window, cx| {
            preview.push_delta("<div>", cx);
            assert!(preview.is_streaming);
            assert_eq!(preview.jsx, "<div>");
        });
    }

    #[gpui::test]
    async fn finish_clears_streaming(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| JsxPreview::new(String::new(), cx));
        handle.update_in(cx, |preview, _window, cx| {
            preview.push_delta("<div>", cx);
            assert!(preview.is_streaming);
            preview.finish(cx);
            assert!(!preview.is_streaming);
        });
    }

    #[gpui::test]
    async fn set_error_fires_on_error_callback(cx: &mut gpui::TestAppContext) {
        let counter = Rc::new(Cell::new(0u32));
        let counter_clone = counter.clone();
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            let mut preview = JsxPreview::new(String::new(), cx);
            preview.set_on_error(move |_err, _window, _cx| {
                counter_clone.set(counter_clone.get() + 1);
            });
            preview
        });
        handle.update_in(cx, |preview, window, cx| {
            preview.set_error(
                JsxPreviewError {
                    message: "oops".into(),
                },
                window,
                cx,
            );
        });
        assert_eq!(counter.get(), 1);
    }

    #[gpui::test]
    async fn clear_error_removes_error(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| JsxPreview::new(String::new(), cx));
        handle.update_in(cx, |preview, window, cx| {
            preview.set_error(
                JsxPreviewError {
                    message: "oops".into(),
                },
                window,
                cx,
            );
            assert!(preview.error.is_some());
            preview.clear_error(cx);
            assert!(preview.error.is_none());
        });
    }

    #[gpui::test]
    async fn set_components_stores_names(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| JsxPreview::new(String::new(), cx));
        handle.update_in(cx, |preview, _window, cx| {
            let names: Vec<SharedString> = vec!["Button".into(), "Card".into()];
            preview.set_components(names.clone(), cx);
            assert_eq!(preview.components, names);
        });
    }

    #[gpui::test]
    async fn set_bindings_stores_values(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| JsxPreview::new(String::new(), cx));
        handle.update_in(cx, |preview, _window, cx| {
            let mut bindings = HashMap::new();
            bindings.insert("count".to_string(), serde_json::Value::from(42));
            preview.set_bindings(bindings.clone(), cx);
            assert_eq!(preview.bindings, bindings);
        });
    }

    #[gpui::test]
    async fn display_jsx_closes_tags_when_streaming(cx: &mut gpui::TestAppContext) {
        let (handle, cx) =
            setup_test_window(cx, |_window, cx| JsxPreview::new("<div><span>hi", cx));
        handle.update_in(cx, |preview, _window, _cx| {
            preview.is_streaming = true;
            assert_eq!(preview.display_jsx(), "<div><span>hi</span></div>");
        });
    }
}
