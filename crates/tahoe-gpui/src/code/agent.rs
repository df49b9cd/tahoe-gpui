//! Agent configuration display component.
//!
//! Provides both a stateful `AgentView` (Entity-based, for simple all-in-one usage)
//! and composable stateless subcomponents (`AgentCard`, `AgentHeader`, `AgentContent`,
//! `AgentInstructions`, `AgentTool`, `AgentOutput`) plus a stateful `AgentTools`
//! accordion that can be used independently.

use super::schema_display::SchemaDisplayView;
use crate::callback_types::OnToggle;
use crate::components::content::badge::Badge;
use crate::components::layout_and_organization::disclosure_group::DisclosureGroup;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use crate::markdown::code_block::CodeBlockView;
use crate::markdown::renderer::StreamingMarkdown;
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, Entity, FontWeight, SharedString, Window, div};

/// A tool definition for the agent display.
pub struct AgentToolDef {
    pub name: SharedString,
    pub description: Option<SharedString>,
    pub schema_json: Option<SharedString>,
}

impl AgentToolDef {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            description: None,
            schema_json: None,
        }
    }

    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn schema(mut self, json: impl Into<SharedString>) -> Self {
        self.schema_json = Some(json.into());
        self
    }
}

/// Parsed tool schema — either a structured tree or a raw code fallback.
enum ToolSchema {
    /// Successfully parsed JSON schema displayed as an interactive tree.
    Tree(Entity<SchemaDisplayView>),
    /// Fallback: raw JSON string displayed as a code block.
    Raw(SharedString),
}

// ─── Composable stateless subcomponents ─────────────────────────────────────

/// Root wrapper card for the agent display.
///
/// Named `AgentCard` to avoid conflicting with the SDK `Agent` trait.
///
/// # Example
/// ```ignore
/// AgentCard::new()
///     .header(AgentHeader::new("Claude").model("claude-sonnet-4-5"))
///     .content(AgentContent::new().child(AgentOutput::new(schema)))
/// ```
#[derive(IntoElement)]
pub struct AgentCard {
    header: Option<AnyElement>,
    content: Option<AnyElement>,
}

impl Default for AgentCard {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentCard {
    pub fn new() -> Self {
        Self {
            header: None,
            content: None,
        }
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for AgentCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut card = crate::foundations::materials::card_surface(theme);

        if let Some(header) = self.header {
            card = card.child(header);
        }
        if let Some(content) = self.content {
            card = card.child(content);
        }

        card
    }
}

/// Stateless header displaying agent name and optional model badge.
///
/// # Example
/// ```ignore
/// AgentHeader::new("Sentiment Analyzer")
///     .model("anthropic/claude-sonnet-4-5")
/// ```
#[derive(IntoElement)]
pub struct AgentHeader {
    name: SharedString,
    model: Option<SharedString>,
}

impl AgentHeader {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            model: None,
        }
    }

    /// Optional model identifier displayed as a `Badge`.
    pub fn model(mut self, model: impl Into<SharedString>) -> Self {
        self.model = Some(model.into());
        self
    }
}

impl RenderOnce for AgentHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut header = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .child(
                Icon::new(IconName::Bot)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text)
                    .child(self.name),
            );

        if let Some(model) = self.model {
            header = header.child(Badge::new(model));
        }

        header
    }
}

/// Stateless container for agent content sections.
///
/// # Example
/// ```ignore
/// AgentContent::new()
///     .child(AgentInstructions::new(md_entity))
///     .child(tools_entity)
///     .child(AgentOutput::new(schema))
/// ```
#[derive(IntoElement)]
pub struct AgentContent {
    children: Vec<AnyElement>,
}

impl Default for AgentContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for AgentContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .children(self.children)
    }
}

/// Stateless display for agent instruction text as markdown.
///
/// Takes a pre-built `Entity<StreamingMarkdown>` and wraps it with a
/// labelled container.
///
/// # Example
/// ```ignore
/// let md = cx.new(|cx| {
///     let mut md = StreamingMarkdown::new(cx);
///     md.push_delta("Analyze sentiment…", cx);
///     md.finish(cx);
///     md
/// });
/// AgentInstructions::new(md)
/// ```
#[derive(IntoElement)]
pub struct AgentInstructions {
    markdown: Entity<StreamingMarkdown>,
}

impl AgentInstructions {
    pub fn new(markdown: Entity<StreamingMarkdown>) -> Self {
        Self { markdown }
    }
}

impl RenderOnce for AgentInstructions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text_muted)
                    .child("Instructions"),
            )
            .child(
                div()
                    .px(theme.spacing_sm)
                    .py(theme.spacing_sm)
                    .bg(theme.hover)
                    .rounded(theme.radius_md)
                    .child(self.markdown),
            )
    }
}

/// Stateless individual tool with expandable input schema.
///
/// For standalone use — accepts a pre-built schema body element and
/// delegates expand/collapse to `DisclosureGroup`.
///
/// # Example
/// ```ignore
/// AgentTool::new("search-tool", "Search the web")
///     .schema_body(schema_view)
///     .is_open(true)
///     .on_toggle(|new_state, window, cx| { /* … */ })
/// ```
#[derive(IntoElement)]
pub struct AgentTool {
    id: ElementId,
    description: SharedString,
    schema_body: Option<AnyElement>,
    is_open: bool,
    on_toggle: OnToggle,
}

impl AgentTool {
    pub fn new(id: impl Into<ElementId>, description: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            schema_body: None,
            is_open: false,
            on_toggle: None,
        }
    }

    /// Pre-built schema element to show when expanded.
    pub fn schema_body(mut self, body: impl IntoElement) -> Self {
        self.schema_body = Some(body.into_any_element());
        self
    }

    /// Controls whether the tool is currently expanded.
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Callback invoked when the header is clicked. Receives the new open state.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for AgentTool {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let tool_header = div()
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text)
            .child(self.description);

        let tool_body: AnyElement = self.schema_body.unwrap_or_else(|| {
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child("No schema")
                .into_any_element()
        });

        let mut collapsible =
            DisclosureGroup::new(self.id, tool_header, tool_body).open(self.is_open);

        if let Some(handler) = self.on_toggle {
            collapsible = collapsible.on_toggle(handler);
        }

        collapsible
    }
}

/// Stateless display for agent output schema with syntax highlighting.
///
/// # Example
/// ```ignore
/// AgentOutput::new("z.object({ sentiment: z.string() })")
///     .language("typescript")
/// ```
#[derive(IntoElement)]
pub struct AgentOutput {
    schema: SharedString,
    language: Option<String>,
}

impl AgentOutput {
    pub fn new(schema: impl Into<SharedString>) -> Self {
        Self {
            schema: schema.into(),
            language: Some("typescript".to_string()),
        }
    }

    /// Override the syntax-highlighting language (default: `"typescript"`).
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }
}

impl RenderOnce for AgentOutput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .child(
                // HIG §Accessibility: section headers below 18 pt need a
                // 4.5:1 contrast ratio. `text_muted` maps to
                // `secondary_label` which drops below that threshold
                // against `background` under some appearance modes.
                // Promote to `text` and lean on SEMIBOLD for the "section
                // header" visual weight — matches Xcode and Zed inspector
                // panes.
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                    .text_color(theme.text)
                    .child("Output Schema"),
            )
            .child(CodeBlockView::new(self.schema.to_string()).language(self.language))
    }
}

// ─── Stateful AgentTools accordion ──────────────────────────────────────────

/// Agent tools accordion container.
///
/// Stateful component that manages which tool is currently expanded.
/// Only one tool can be open at a time (standard accordion behavior).
///
/// # Example
/// ```ignore
/// let tools = cx.new(|cx| {
///     let mut t = AgentTools::new(cx);
///     t.tools(vec![AgentToolDef::new("search").description("Search")], cx);
///     t
/// });
/// ```
pub struct AgentTools {
    tools: Vec<AgentToolDef>,
    tool_schemas: Vec<Option<ToolSchema>>,
    open_tool_index: Option<usize>,
}

impl AgentTools {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            tools: Vec::new(),
            tool_schemas: Vec::new(),
            open_tool_index: None,
        }
    }

    pub fn tools(&mut self, tools: Vec<AgentToolDef>, cx: &mut Context<Self>) -> &mut Self {
        self.open_tool_index = None;
        self.tool_schemas = tools
            .iter()
            .map(|tool| {
                tool.schema_json.as_ref().map(|json_str| {
                    match serde_json::from_str::<serde_json::Value>(json_str.as_ref()) {
                        Ok(val) => ToolSchema::Tree(cx.new(|cx| SchemaDisplayView::new(&val, cx))),
                        Err(_) => ToolSchema::Raw(json_str.clone()),
                    }
                })
            })
            .collect();
        self.tools = tools;
        self
    }

    fn toggle_tool(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.open_tool_index == Some(index) {
            self.open_tool_index = None;
        } else {
            self.open_tool_index = Some(index);
        }
        cx.notify();
    }

    fn render_tool(
        &self,
        i: usize,
        tool: &AgentToolDef,
        theme: &TahoeTheme,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let desc = tool
            .description
            .clone()
            .unwrap_or_else(|| "No description".into());

        let tool_header = div()
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text)
            .child(desc);

        let tool_body: AnyElement = match self.tool_schemas.get(i) {
            Some(Some(ToolSchema::Tree(schema_entity))) => schema_entity.clone().into_any_element(),
            Some(Some(ToolSchema::Raw(raw))) => CodeBlockView::new(raw.to_string())
                .language(Some("json".to_string()))
                .into_any_element(),
            _ => div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child("No schema")
                .into_any_element(),
        };

        let entity = cx.entity().clone();
        DisclosureGroup::new(
            ElementId::NamedInteger("agent-tools-tool".into(), i as u64),
            tool_header,
            tool_body,
        )
        .open(self.open_tool_index == Some(i))
        .on_toggle(move |_new_state, _window, cx| {
            entity.update(cx, |this, cx| this.toggle_tool(i, cx));
        })
    }
}

impl Render for AgentTools {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        if self.tools.is_empty() {
            return div();
        }

        let mut section = div().flex().flex_col().gap(theme.spacing_xs).child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .text_color(theme.text_muted)
                .child("Tools"),
        );

        for i in 0..self.tools.len() {
            let tool = &self.tools[i];
            section = section.child(self.render_tool(i, tool, theme, cx));
        }

        section
    }
}

// ─── Existing stateful AgentView ────────────────────────────────────────────

/// An agent configuration display card.
///
/// Stateful component that renders agent name, model badge, markdown
/// instructions, accordion-style tools with schema display, and an
/// output schema code block.
pub struct AgentView {
    name: SharedString,
    model: Option<SharedString>,
    instructions_text: Option<SharedString>,
    instructions_md: Option<Entity<StreamingMarkdown>>,
    tools_entity: Option<Entity<AgentTools>>,
    output_schema: Option<SharedString>,
}

impl AgentView {
    pub fn new(name: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let _ = cx;
        Self {
            name: name.into(),
            model: None,
            instructions_text: None,
            instructions_md: None,
            tools_entity: None,
            output_schema: None,
        }
    }

    pub fn model(&mut self, model: impl Into<SharedString>) -> &mut Self {
        self.model = Some(model.into());
        self
    }

    pub fn instructions(
        &mut self,
        instructions: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> &mut Self {
        let text: SharedString = instructions.into();
        self.instructions_text = Some(text.clone());
        let md = cx.new(|cx| {
            let mut md = StreamingMarkdown::new(cx);
            md.push_delta(&text, cx);
            md.finish(cx);
            md
        });
        self.instructions_md = Some(md);
        self
    }

    pub fn tools(&mut self, tools: Vec<AgentToolDef>, cx: &mut Context<Self>) -> &mut Self {
        let entity = self
            .tools_entity
            .get_or_insert_with(|| cx.new(AgentTools::new))
            .clone();
        entity.update(cx, |this, cx| {
            this.tools(tools, cx);
        });
        self
    }

    pub fn output_schema(&mut self, schema: impl Into<SharedString>) -> &mut Self {
        self.output_schema = Some(schema.into());
        self
    }
}

impl Render for AgentView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut card = crate::foundations::materials::card_surface(theme);

        // Header
        let mut header = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .child(
                Icon::new(IconName::Bot)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text)
                    .child(self.name.clone()),
            );

        if let Some(ref model) = self.model {
            header = header.child(Badge::new(model.clone()));
        }

        card = card.child(header);

        // Content area
        let mut content = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .px(theme.spacing_md)
            .py(theme.spacing_sm);

        // Instructions (rendered as markdown)
        if let Some(ref md_entity) = self.instructions_md {
            content = content.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_xs)
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .text_color(theme.text_muted)
                            .child("Instructions"),
                    )
                    .child(
                        div()
                            .px(theme.spacing_sm)
                            .py(theme.spacing_sm)
                            .bg(theme.hover)
                            .rounded(theme.radius_md)
                            .child(md_entity.clone()),
                    ),
            );
        }

        // Tools (accordion)
        if let Some(ref tools_entity) = self.tools_entity {
            content = content.child(tools_entity.clone());
        }

        // Output schema
        if let Some(ref schema) = self.output_schema {
            content = content.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_xs)
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .text_color(theme.text_muted)
                            .child("Output Schema"),
                    )
                    .child(
                        CodeBlockView::new(schema.to_string())
                            .language(Some("typescript".to_string())),
                    ),
            );
        }

        card = card.child(content);
        card
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentCard, AgentContent, AgentHeader, AgentOutput, AgentTool, AgentToolDef};
    use core::prelude::v1::test;

    // ── Existing tests ──────────────────────────────────────────────────

    #[test]
    fn toggle_tool_opens_and_closes() {
        let mut open = None::<usize>;

        open = if open == Some(0) { None } else { Some(0) };
        assert_eq!(open, Some(0));

        open = if open == Some(1) { None } else { Some(1) };
        assert_eq!(open, Some(1));

        open = if open == Some(1) { None } else { Some(1) };
        assert_eq!(open, None);
    }

    #[test]
    fn tool_schema_parsing_valid_json() {
        let json = r#"{"type": "object", "properties": {"query": {"type": "string"}}}"#;
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["type"], "object");
    }

    #[test]
    fn tool_schema_parsing_invalid_json() {
        let json = "not valid json {{{";
        let result = serde_json::from_str::<serde_json::Value>(json);
        assert!(result.is_err());
    }

    #[test]
    fn agent_tool_def_builder() {
        let tool = AgentToolDef::new("search")
            .description("Search the web")
            .schema(r#"{"type": "object"}"#);
        assert_eq!(tool.name.as_ref(), "search");
        assert_eq!(
            tool.description.as_ref().map(|s| s.as_ref()),
            Some("Search the web")
        );
        assert!(tool.schema_json.is_some());
    }

    // ── New subcomponent tests ───────────────────────────────────────────

    #[test]
    fn agent_card_empty() {
        let card = AgentCard::new();
        assert!(card.header.is_none());
        assert!(card.content.is_none());
    }

    #[test]
    fn agent_header_builder() {
        let header = AgentHeader::new("Claude");
        assert_eq!(header.name.as_ref(), "Claude");
        assert!(header.model.is_none());
    }

    #[test]
    fn agent_header_with_model() {
        let header = AgentHeader::new("Claude").model("claude-sonnet-4-5");
        assert_eq!(header.name.as_ref(), "Claude");
        assert_eq!(
            header.model.as_ref().map(|s| s.as_ref()),
            Some("claude-sonnet-4-5")
        );
    }

    #[test]
    fn agent_content_empty() {
        let content = AgentContent::new();
        assert!(content.children.is_empty());
    }

    #[test]
    fn agent_tool_default_closed() {
        let tool = AgentTool::new("t1", "Search the web");
        assert!(!tool.is_open);
        assert!(tool.schema_body.is_none());
        assert!(tool.on_toggle.is_none());
    }

    #[test]
    fn agent_tool_open() {
        let tool = AgentTool::new("t1", "Search").is_open(true);
        assert!(tool.is_open);
    }

    #[test]
    fn agent_output_default_language() {
        let output = AgentOutput::new("z.object({})");
        assert_eq!(output.schema.as_ref(), "z.object({})");
        assert_eq!(output.language.as_deref(), Some("typescript"));
    }

    #[test]
    fn agent_output_custom_language() {
        let output = AgentOutput::new(r#"{"type":"object"}"#).language("json");
        assert_eq!(output.language.as_deref(), Some("json"));
    }
}
