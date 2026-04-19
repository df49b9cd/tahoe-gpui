//! Visual gallery of every `tahoe_gpui::code` component.
//!
//! Every `Entity<T>` demoed here is constructed once in
//! `CodeGallery::new` and stored as a field. Re-creating entities inside
//! `render` would mint a new `EntityId` each frame, which in turn changes
//! the `ElementId::View(...)` segment of every descendant's
//! `GlobalElementId` — and the element-state map that backs
//! `gpui::AnimationElement`, `cx.global`-style caches, and every other
//! per-element persistent state is keyed by that id. Churn the id and
//! animations never advance past delta = 0, clicks lose their "copied"
//! feedback, disclosure toggles reset, etc.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    App, Bounds, Div, Entity, FontWeight, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_platform::application;

use tahoe_gpui::code::*;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::menus_and_actions::copy_button::CopyButton;
use tahoe_gpui::foundations::accessibility::AccessibilityMode;
use tahoe_gpui::foundations::icons::EmbeddedIconAssets;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use tahoe_gpui::foundations::typography::DynamicTypeSize;
use tahoe_gpui::markdown::code_block::{CodeBlockView, LanguageVariant};

/// Holds every `Entity<T>` the gallery demos. Constructing them here
/// (once, at startup) — and wiring the matching `CopyButton`s for
/// `CodeBlockView` / `Snippet` via their `.copy_button(…)` hooks — keeps
/// every descendant `GlobalElementId` stable across frames, so animations
/// advance, disclosure state sticks, and `Copy → Check` feedback persists
/// for its full 2 s window.
struct CodeGallery {
    dark_mode: bool,

    // Persistent CopyButton entities — one per copyable surface. Pre-
    // creating them and passing them into the matching builders is the
    // documented workaround for the "CopyButton re-created each render"
    // anti-pattern noted on `Snippet::copy_button` / `CodeBlockView::copy_button`.
    code_block_rust_copy: Entity<CopyButton>,
    code_block_python_copy: Entity<CopyButton>,
    code_block_variants_copy: Entity<CopyButton>,
    code_block_collapsed_copy: Entity<CopyButton>,
    snippet_copies: [Entity<CopyButton>; 7],
    artifact_copy: Entity<CopyButton>,
    commit_copy: Entity<CopyButton>,

    // Stateful views. Same rule: construct once, hold the entity, reuse
    // its handle in `render` — never rebuild via `cx.new(…)` from render.
    env_vars: Entity<EnvironmentVariablesView>,
    jsx_static: Entity<JsxPreview>,
    jsx_streaming: Entity<JsxPreview>,
    file_tree: Entity<FileTreeView>,
    terminal: Entity<TerminalView>,
    stack_trace: Entity<StackTraceView>,
    api_endpoint: Entity<ApiEndpointView>,
    test_results: Entity<TestResultsView>,
    sandbox: Entity<SandboxView>,
    agent: Entity<AgentView>,
}

impl CodeGallery {
    fn new(cx: &mut Context<Self>) -> Self {
        // ── Copy buttons ────────────────────────────────────────────────
        let code_block_rust_copy = CopyButton::new(RUST_SAMPLE.to_string(), cx);
        let code_block_python_copy = CopyButton::new(PYTHON_SAMPLE.to_string(), cx);
        let code_block_variants_copy = CopyButton::new(TS_SAMPLE.to_string(), cx);
        let code_block_collapsed_copy = CopyButton::new(RUST_SAMPLE.to_string(), cx);
        let snippet_copies = [
            CopyButton::new("npm install @ai-sdk/react".to_string(), cx),
            CopyButton::new("cargo add tahoe-gpui".to_string(), cx),
            CopyButton::new("git push origin main".to_string(), cx),
            CopyButton::new("export API_KEY=sk-...".to_string(), cx),
            CopyButton::new("echo 'Disabled example'".to_string(), cx),
            CopyButton::new("world".to_string(), cx),
            CopyButton::new("sensitive-cmd".to_string(), cx),
        ];
        let artifact_copy = CopyButton::new(ARTIFACT_SAMPLE.to_string(), cx);
        let commit_copy = CopyButton::new("abc1234".to_string(), cx);

        // ── Env vars ────────────────────────────────────────────────────
        let env_vars = EnvironmentVariablesView::new(
            vec![
                EnvVar::new("OPENAI_API_KEY", "sk-proj-***").sensitive(true),
                EnvVar::new("ANTHROPIC_API_KEY", "sk-ant-***").sensitive(true),
                EnvVar::new("DATABASE_URL", "postgres://localhost:5432/mydb"),
                EnvVar::new("REDIS_URL", "").required(true),
            ],
            cx,
        );

        // ── JSX previews ────────────────────────────────────────────────
        let jsx_static = cx.new(|cx| {
            let mut preview = JsxPreview::new(JSX_SAMPLE, cx);
            preview.set_components(vec!["Button".into(), "Card".into()], cx);
            let mut bindings = HashMap::new();
            bindings.insert("count".to_string(), serde_json::Value::from(0));
            preview.set_bindings(bindings, cx);
            preview
        });
        let jsx_streaming = cx.new(|cx| {
            let mut preview = JsxPreview::new("<div><span>Loading", cx);
            preview.set_streaming(true, cx);
            preview
        });

        // ── File tree ───────────────────────────────────────────────────
        let file_tree = cx.new(|cx| {
            let mut tree = FileTreeView::new(cx);
            tree.set_children(
                vec![
                    TreeNode::folder(
                        "src",
                        "src",
                        vec![
                            TreeNode::file("main.rs", "src/main.rs"),
                            TreeNode::file("lib.rs", "src/lib.rs"),
                            TreeNode::folder(
                                "components",
                                "src/components",
                                vec![
                                    TreeNode::file(
                                        "button.rs",
                                        "src/components/button.rs",
                                    ),
                                    TreeNode::file(
                                        "modal.rs",
                                        "src/components/modal.rs",
                                    ),
                                ],
                            ),
                        ],
                    ),
                    TreeNode::file("Cargo.toml", "Cargo.toml"),
                    TreeNode::file("README.md", "README.md"),
                ],
                cx,
            );
            tree.set_default_expanded(HashSet::from(["src".to_string()]), cx);
            tree.set_on_select(|path, _, _| println!("Selected: {path}"));
            tree.set_on_expanded_change(|expanded, _, _| {
                println!("Expanded: {expanded:?}")
            });
            tree
        });

        // ── Terminal ────────────────────────────────────────────────────
        let terminal = cx.new(|cx| {
            let mut terminal = TerminalView::new(cx);
            terminal.set_title("zsh — cargo test", cx);
            terminal.push_output(
                "running 3 tests\n\
                 test tests::it_builds ... ok\n\
                 test tests::it_runs ... ok\n\
                 test tests::it_passes ... ok\n\
                 \ntest result: ok. 3 passed; 0 failed; 0 ignored\n",
                cx,
            );
            terminal
        });

        // ── Stack trace ─────────────────────────────────────────────────
        let stack_trace = cx.new(|cx| {
            StackTraceView::new(
                "TypeError: Cannot read properties of undefined\n\
                 \tat fetchUser (/app/users.js:42:12)\n\
                 \tat async handler (/app/api/users.js:17:3)\n\
                 \tat async middleware (/app/middleware.js:8:5)",
                cx,
            )
        });

        // ── API endpoint ────────────────────────────────────────────────
        let api_endpoint = cx.new(|cx| {
            let mut endpoint =
                ApiEndpointView::new(HttpMethod::Post, "/api/users", cx);
            endpoint.description("Create a new user account");
            endpoint.parameters(vec![
                EndpointParameter::new("page", "integer")
                    .location(ParameterLocation::Query)
                    .description("Zero-indexed page number"),
                EndpointParameter::new("id", "string")
                    .location(ParameterLocation::Path)
                    .required(true)
                    .description("User identifier"),
            ]);
            endpoint.request_body(vec![
                EndpointProperty::new("email", "string").required(true),
                EndpointProperty::new("name", "string").required(true),
            ]);
            endpoint
        });

        // ── Test results ────────────────────────────────────────────────
        let test_results = cx.new(|cx| {
            let mut results = TestResultsView::new(cx);
            let failing_case = {
                let mut case =
                    TestCase::new("handles_expired_session", TestStatus::Failed)
                        .duration_ms(18)
                        .suite("auth::tests");
                case.error_message = Some("expected 401, got 500".into());
                case
            };
            results.set_tests(
                vec![
                    TestCase::new("signs_in_with_valid_credentials", TestStatus::Passed)
                        .duration_ms(14)
                        .suite("auth::tests"),
                    TestCase::new("rejects_invalid_token", TestStatus::Passed)
                        .duration_ms(9)
                        .suite("auth::tests"),
                    failing_case,
                    TestCase::new("refresh_flow", TestStatus::Skipped).suite("auth::tests"),
                ],
                cx,
            );
            results
        });

        // ── Sandbox ─────────────────────────────────────────────────────
        let sandbox = cx.new(|cx| {
            let mut sandbox = SandboxView::new(
                "import requests\nprint(requests.get('https://example.com').status_code)",
                "python",
                cx,
            );
            sandbox.set_title("Python Sandbox", cx);
            sandbox.set_status(SandboxStatus::Running, cx);
            sandbox.set_logs(
                "Collecting requests\n\
                 Installing collected packages: requests\n\
                 Successfully installed requests-2.31.0\n200",
                cx,
            );
            sandbox
        });

        // ── Agent ───────────────────────────────────────────────────────
        let agent = cx.new(|cx| {
            let mut agent = AgentView::new("Code Reviewer", cx);
            agent.model("claude-opus-4-7");
            agent.instructions(
                "Review the latest commit and flag anything that doesn't match the project style guide.",
                cx,
            );
            agent.tools(
                vec![
                    AgentToolDef::new("read_file"),
                    AgentToolDef::new("comment_on_pr"),
                ],
                cx,
            );
            agent
        });

        Self {
            dark_mode: true,
            code_block_rust_copy,
            code_block_python_copy,
            code_block_variants_copy,
            code_block_collapsed_copy,
            snippet_copies,
            artifact_copy,
            commit_copy,
            env_vars,
            jsx_static,
            jsx_streaming,
            file_tree,
            terminal,
            stack_trace,
            api_endpoint,
            test_results,
            sandbox,
            agent,
        }
    }

    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.dark_mode = !self.dark_mode;
        let prior = cx.global::<TahoeTheme>().clone();
        let mut theme = if self.dark_mode {
            TahoeTheme::dark()
        } else {
            TahoeTheme::light()
        };
        theme.accessibility_mode = prior.accessibility_mode;
        theme.dynamic_type_size = prior.dynamic_type_size;
        theme.apply(cx);
    }

    fn toggle_accessibility(&mut self, flag: AccessibilityMode, cx: &mut Context<Self>) {
        let mut theme = cx.global::<TahoeTheme>().clone();
        theme.accessibility_mode = theme.accessibility_mode.toggled(flag);
        theme.apply(cx);
    }

    fn step_text_size(&mut self, step: i32, cx: &mut Context<Self>) {
        let order = [
            DynamicTypeSize::XSmall,
            DynamicTypeSize::Small,
            DynamicTypeSize::Medium,
            DynamicTypeSize::Large,
            DynamicTypeSize::XLarge,
            DynamicTypeSize::XXLarge,
            DynamicTypeSize::XXXLarge,
            DynamicTypeSize::AX1,
            DynamicTypeSize::AX2,
            DynamicTypeSize::AX3,
            DynamicTypeSize::AX4,
            DynamicTypeSize::AX5,
        ];
        let current = cx.global::<TahoeTheme>().dynamic_type_size;
        let idx = order.iter().position(|s| *s == current).unwrap_or(3) as i32;
        let clamped = (idx + step).clamp(0, order.len() as i32 - 1) as usize;
        let mut theme = cx.global::<TahoeTheme>().clone();
        theme.dynamic_type_size = order[clamped];
        theme.apply(cx);
    }
}

impl Render for CodeGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let theme = &theme;
        let dark = self.dark_mode;
        let access = theme.accessibility_mode;
        let text_size = theme.dynamic_type_size;

        let access_chip =
            |id: &'static str, label: &'static str, flag: AccessibilityMode, active: bool| {
                Button::new(id)
                    .label(label)
                    .variant(if active {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .size(ButtonSize::Sm)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_accessibility(flag, cx);
                    }))
            };

        let header = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .pb(theme.spacing_md)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_style(TextStyle::Title1, theme)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Code Gallery"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(theme.spacing_sm)
                    .items_center()
                    .child(
                        Button::new("toggle-theme")
                            .label(if dark {
                                "\u{2600} Toggle to Light"
                            } else {
                                "\u{263E} Toggle to Dark"
                            })
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_theme(cx))),
                    )
                    .child(
                        Button::new("text-size-smaller")
                            .label("A\u{2212}")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .on_click(cx.listener(|this, _, _, cx| this.step_text_size(-1, cx))),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child(format!("{text_size:?}")),
                    )
                    .child(
                        Button::new("text-size-larger")
                            .label("A+")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .on_click(cx.listener(|this, _, _, cx| this.step_text_size(1, cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(theme.spacing_xs)
                    .child(access_chip(
                        "ax-reduce-transparency",
                        "Reduce Transparency",
                        AccessibilityMode::REDUCE_TRANSPARENCY,
                        access.reduce_transparency(),
                    ))
                    .child(access_chip(
                        "ax-increase-contrast",
                        "Increase Contrast",
                        AccessibilityMode::INCREASE_CONTRAST,
                        access.increase_contrast(),
                    ))
                    .child(access_chip(
                        "ax-reduce-motion",
                        "Reduce Motion",
                        AccessibilityMode::REDUCE_MOTION,
                        access.reduce_motion(),
                    ))
                    .child(access_chip(
                        "ax-bold-text",
                        "Bold Text",
                        AccessibilityMode::BOLD_TEXT,
                        access.bold_text(),
                    )),
            );

        div()
            .id("code-gallery-scroll")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .p(px(24.0))
            .gap(px(24.0))
            .overflow_y_scroll()
            .child(header)
            .child(section("Code Block (Rust)", theme).child(
                CodeBlockView::new(RUST_SAMPLE)
                    .language(Some("rust".to_string()))
                    .show_line_numbers(true)
                    .filename("fibonacci.rs")
                    .copy_button(self.code_block_rust_copy.clone()),
            ))
            .child(section("Code Block (Python)", theme).child(
                CodeBlockView::new(PYTHON_SAMPLE)
                    .language(Some("python".to_string()))
                    .show_line_numbers(true)
                    .copy_button(self.code_block_python_copy.clone()),
            ))
            .child(section("Code Block (language selector)", theme).child(
                CodeBlockView::new("")
                    .show_line_numbers(true)
                    .language_variants(vec![
                        LanguageVariant {
                            label: "TypeScript".into(),
                            language: "typescript".into(),
                            code: TS_SAMPLE.to_string(),
                        },
                        LanguageVariant {
                            label: "Rust".into(),
                            language: "rust".into(),
                            code: RUST_SAMPLE.to_string(),
                        },
                    ])
                    .active_variant_index(0)
                    .copy_button(self.code_block_variants_copy.clone()),
            ))
            .child(section("Code Block (collapsed)", theme).child(
                CodeBlockView::new(RUST_SAMPLE)
                    .language(Some("rust".to_string()))
                    .show_line_numbers(true)
                    .max_lines(5)
                    .copy_button(self.code_block_collapsed_copy.clone()),
            ))
            .child(section("Snippet", theme).child(
                Snippet::new("npm install @ai-sdk/react")
                    .copy_button(self.snippet_copies[0].clone()),
            ))
            .child(section("Snippet (with prefix)", theme).child(
                Snippet::new("cargo add tahoe-gpui")
                    .prefix("$")
                    .copy_button(self.snippet_copies[1].clone()),
            ))
            .child(section("Snippet (custom timeout)", theme).child(
                Snippet::new("git push origin main")
                    .prefix("$")
                    .timeout(Duration::from_secs(5))
                    .copy_button(self.snippet_copies[2].clone()),
            ))
            .child(section("Snippet (on_copy)", theme).child(
                Snippet::new("export API_KEY=sk-...")
                    .on_copy(Arc::new(|| println!("Copied API key command!")))
                    .copy_button(self.snippet_copies[3].clone()),
            ))
            .child(section("Snippet (disabled)", theme).child(
                Snippet::new("echo 'Disabled example'")
                    .prefix("$")
                    .disabled(true)
                    .copy_button(self.snippet_copies[4].clone()),
            ))
            .child(section("Snippet (multiple addons)", theme).child(
                Snippet::new("world")
                    .addon(div().child("hello "))
                    .addon(div().child("beautiful "))
                    .copy_button(self.snippet_copies[5].clone()),
            ))
            .child(section("Snippet (on_error callback)", theme).child(
                Snippet::new("sensitive-cmd")
                    .on_error(Arc::new(|err| eprintln!("Copy failed: {err}")))
                    .copy_button(self.snippet_copies[6].clone()),
            ))
            .child(section("Commit", theme).child(
                Commit::new("abc1234", "Fix streaming animation for markdown renderer")
                    .author("dev@example.com")
                    .date("2026-04-04")
                    .open(true)
                    .copy_button(self.commit_copy.clone())
                    .file_changes(vec![
                        CommitFileData::new("src/markdown/renderer.rs", FileStatus::Modified)
                            .additions(42)
                            .deletions(15),
                        CommitFileData::new("src/markdown/animation.rs", FileStatus::Added)
                            .additions(78),
                        CommitFileData::new("tests/markdown_test.rs", FileStatus::Modified)
                            .additions(23)
                            .deletions(5),
                    ]),
            ))
            .child(section("Package Info (convenience API)", theme).child(
                PackageInfoView::new("tahoe-gpui", "0.1.0")
                    .new_version("0.2.0")
                    .change_type(ChangeType::Minor)
                    .description("Tahoe HIG-aligned UI components for GPUI")
                    .license("Apache-2.0")
                    .dependencies(vec![
                        Dependency::new("gpui").version("0.1.0"),
                        Dependency::new("serde"),
                    ]),
            ))
            .child(section("Package Info (compound API)", theme).child(
                PackageInfoView::from_parts()
                    .child(
                        PackageInfoHeader::new()
                            .child(PackageInfoName::new("react"))
                            .child(PackageInfoChangeType::new(ChangeType::Major)),
                    )
                    .child(
                        PackageInfoVersion::new()
                            .current("18.2.0")
                            .new_ver("19.0.0"),
                    )
                    .child(PackageInfoDescription::new(
                        "The library for web and native user interfaces",
                    ))
                    .child(
                        PackageInfoContent::new().child(
                            PackageInfoDependencies::new()
                                .child(
                                    PackageInfoDependency::new("scheduler").version("0.23.0"),
                                )
                                .child(PackageInfoDependency::new("loose-envify")),
                        ),
                    ),
            ))
            .child(section("Environment Variables", theme).child(self.env_vars.clone()))
            .child(section("Artifact", theme).child(
                Artifact::new("Generated Component")
                    .content(
                        CodeBlockView::new(ARTIFACT_SAMPLE.to_string())
                            .copy_button(self.artifact_copy.clone()),
                    ),
            ))
            .child(section("JSX Preview (static)", theme).child(self.jsx_static.clone()))
            .child(section("JSX Preview (streaming)", theme).child(self.jsx_streaming.clone()))
            // FileTreeView uses `uniform_list` which needs a bounded
            // height to allocate rows; wrap in a fixed-height container so
            // the vertical scroll doesn't collapse it to zero height.
            .child(section("File Tree", theme).child(
                div().h(px(220.0)).child(self.file_tree.clone()),
            ))
            .child(section("Terminal", theme).child(self.terminal.clone()))
            .child(section("Stack Trace", theme).child(self.stack_trace.clone()))
            .child(section("API Endpoint", theme).child(self.api_endpoint.clone()))
            .child(section("Test Results", theme).child(self.test_results.clone()))
            .child(section("Sandbox", theme).child(self.sandbox.clone()))
            .child(section("Web Preview", theme).child(
                WebPreview::new("https://app.example.com/dashboard"),
            ))
            .child(section("Agent", theme).child(self.agent.clone()))
    }
}

fn section(title: &str, theme: &TahoeTheme) -> Div {
    div().flex().flex_col().gap(px(8.0)).child(
        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_muted)
            .child(title.to_string()),
    )
}

const RUST_SAMPLE: &str = r#"use std::collections::HashMap;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let result = fibonacci(10);
    println!("fib(10) = {result}");
}"#;

const PYTHON_SAMPLE: &str = r#"import asyncio

async def fetch_data(url: str) -> dict:
    """Fetch data from an API endpoint."""
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            return await response.json()

# Main entry point
if __name__ == "__main__":
    data = asyncio.run(fetch_data("https://api.example.com"))
    print(f"Got {len(data)} items")"#;

const TS_SAMPLE: &str = r#"export async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  if (!response.ok) {
    throw new Error(`Failed to load user ${id}`);
  }
  return response.json();
}"#;

const JSX_SAMPLE: &str = r#"<div className="card">
  <h1>Hello World</h1>
  <Button onClick={() => setCount(c => c + 1)}>
    Click me: {count}
  </Button>
</div>"#;

const ARTIFACT_SAMPLE: &str =
    "export function Button({ label }: { label: string }) {\n  return <button>{label}</button>;\n}";

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            cx.set_global(TahoeTheme::dark());

            let bounds = Bounds::centered(None, size(px(900.), px(1100.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(CodeGallery::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
