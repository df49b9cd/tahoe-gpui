//! JSON Schema display as a collapsible tree.

use crate::components::content::badge::Badge;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, ElementId, FontWeight, SharedString, Window, div, px};
use std::collections::HashSet;

/// A node in the parsed schema tree.
#[derive(Debug, Clone)]
enum SchemaNode {
    Object {
        properties: Vec<(String, SchemaNode)>,
        required: HashSet<String>,
        description: Option<String>,
    },
    Array {
        items: Box<SchemaNode>,
        description: Option<String>,
    },
    Primitive {
        type_name: String,
        format: Option<String>,
        description: Option<String>,
        enum_values: Option<Vec<String>>,
    },
}

impl SchemaNode {
    fn description(&self) -> Option<&str> {
        match self {
            SchemaNode::Object { description, .. }
            | SchemaNode::Array { description, .. }
            | SchemaNode::Primitive { description, .. } => description.as_deref(),
        }
    }

    fn type_label(&self) -> &str {
        match self {
            SchemaNode::Object { .. } => "object",
            SchemaNode::Array { .. } => "array",
            SchemaNode::Primitive { type_name, .. } => type_name.as_str(),
        }
    }
}

/// Parse a JSON Schema value into a SchemaNode tree.
fn parse_schema(value: &serde_json::Value) -> SchemaNode {
    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            return SchemaNode::Primitive {
                type_name: "any".into(),
                format: None,
                description: None,
                enum_values: None,
            };
        }
    };

    let type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("any");
    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    match type_str {
        "object" => {
            let required: HashSet<String> = obj
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let properties = obj
                .get("properties")
                .and_then(|v| v.as_object())
                .map(|props| {
                    props
                        .iter()
                        .map(|(k, v)| (k.clone(), parse_schema(v)))
                        .collect()
                })
                .unwrap_or_default();

            SchemaNode::Object {
                properties,
                required,
                description,
            }
        }
        "array" => {
            let items = obj
                .get("items")
                .map(parse_schema)
                .unwrap_or(SchemaNode::Primitive {
                    type_name: "any".into(),
                    format: None,
                    description: None,
                    enum_values: None,
                });
            SchemaNode::Array {
                items: Box::new(items),
                description,
            }
        }
        _ => {
            let format = obj.get("format").and_then(|v| v.as_str()).map(String::from);
            let enum_values = obj.get("enum").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .map(|v| v.as_str().unwrap_or("?").to_string())
                    .collect()
            });
            SchemaNode::Primitive {
                type_name: type_str.to_string(),
                format,
                description,
                enum_values,
            }
        }
    }
}

/// A JSON Schema viewer with collapsible objects and arrays.
pub struct SchemaDisplayView {
    schema: SchemaNode,
    expanded_paths: HashSet<String>,
}

impl SchemaDisplayView {
    pub fn new(schema_value: &serde_json::Value, _cx: &mut Context<Self>) -> Self {
        Self {
            schema: parse_schema(schema_value),
            expanded_paths: HashSet::new(),
        }
    }

    pub fn expand_all(&mut self, cx: &mut Context<Self>) {
        let schema = self.schema.clone();
        let mut paths = HashSet::new();
        self.collect_paths(&schema, "".to_string(), &mut paths);
        self.expanded_paths = paths;
        cx.notify();
    }

    fn collect_paths(&self, node: &SchemaNode, prefix: String, paths: &mut HashSet<String>) {
        match node {
            SchemaNode::Object { properties, .. } => {
                paths.insert(prefix.clone());
                for (key, child) in properties {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    self.collect_paths(child, path, paths);
                }
            }
            SchemaNode::Array { items, .. } => {
                paths.insert(prefix.clone());
                self.collect_paths(items, format!("{}[]", prefix), paths);
            }
            _ => {}
        }
    }

    pub fn toggle_path(&mut self, path: String, cx: &mut Context<Self>) {
        if self.expanded_paths.contains(&path) {
            self.expanded_paths.remove(&path);
        } else {
            self.expanded_paths.insert(path);
        }
        cx.notify();
    }

    fn render_node(
        &self,
        node: &SchemaNode,
        name: Option<&str>,
        path: &str,
        depth: usize,
        required: bool,
        theme: &TahoeTheme,
        cx: &Context<Self>,
    ) -> AnyElement {
        let indent = px(depth as f32 * 16.0);
        // Hoist once — this function recurses and has five call sites.
        // `Font` clones only bump `Arc` refcounts, so per-site clones
        // stay allocation-free.
        let mono = theme.mono_font();

        match node {
            SchemaNode::Object {
                properties,
                required: req_set,
                ..
            } => {
                let is_expanded = self.expanded_paths.contains(path);
                let path_owned = path.to_string();

                let mut container = div().flex().flex_col();

                // Header row
                let mut row = div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .pl(indent)
                    .py(theme.spacing_xs)
                    .cursor_pointer()
                    .id(ElementId::Name(SharedString::from(format!(
                        "schema-row-{}",
                        path
                    ))))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_path(path_owned.clone(), cx);
                    }));

                // HIG §SF Symbols: use SF Symbol chevrons for disclosure
                // so scaling and accessibility traits track the rest of the
                // UI. Previously rendered Unicode `▼`/`▶` (U+25BC/U+25B6)
                // which neither scales with Dynamic Type nor respects the
                // system icon-and-widget-style preference.
                row = row.child(
                    Icon::new(if is_expanded {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .size(theme.icon_size_small)
                    .color(theme.text_muted),
                );

                if let Some(n) = name {
                    row = row.child(
                        div()
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text)
                            .child(n.to_string()),
                    );
                }
                row = row.child(
                    // HIG §Typography: code/type annotations in monospace —
                    // mirrors Swagger UI and Xcode's JSON viewer. Wrapping
                    // the `Badge` in a monospace container lets the type
                    // label inherit `font_mono` without changing Badge's
                    // own visual tokens.
                    div().font(mono.clone()).child(Badge::new("object")),
                );
                if required {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.error)
                            .child("required"),
                    );
                }
                if let Some(desc) = node.description() {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child(desc.to_string()),
                    );
                }

                container = container.child(row);

                if is_expanded {
                    for (key, child) in properties {
                        let child_path = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", path, key)
                        };
                        let child_required = req_set.contains(key);
                        container = container.child(self.render_node(
                            child,
                            Some(key),
                            &child_path,
                            depth + 1,
                            child_required,
                            theme,
                            cx,
                        ));
                    }
                }

                container.into_any_element()
            }
            SchemaNode::Array { items, .. } => {
                let is_expanded = self.expanded_paths.contains(path);
                let path_owned = path.to_string();

                let mut container = div().flex().flex_col();

                let mut row = div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .pl(indent)
                    .py(theme.spacing_xs)
                    .cursor_pointer()
                    .id(ElementId::Name(SharedString::from(format!(
                        "schema-row-{}",
                        path
                    ))))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_path(path_owned.clone(), cx);
                    }));

                row = row.child(
                    Icon::new(if is_expanded {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .size(theme.icon_size_small)
                    .color(theme.text_muted),
                );

                if let Some(n) = name {
                    row = row.child(
                        div()
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text)
                            .child(n.to_string()),
                    );
                }
                row = row
                    .child(div().font(mono.clone()).child(Badge::new("array")))
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .font(mono.clone())
                            .child(format!("of {}", items.type_label())),
                    );
                if required {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.error)
                            .child("required"),
                    );
                }

                container = container.child(row);

                if is_expanded {
                    let items_path = format!("{}[]", path);
                    container = container.child(self.render_node(
                        items,
                        Some("items"),
                        &items_path,
                        depth + 1,
                        false,
                        theme,
                        cx,
                    ));
                }

                container.into_any_element()
            }
            SchemaNode::Primitive {
                type_name,
                format,
                enum_values,
                ..
            } => {
                let mut row = div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .pl(indent)
                    .py(theme.spacing_xs);

                if let Some(n) = name {
                    row = row.child(
                        div()
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text)
                            .child(n.to_string()),
                    );
                }

                let mut type_label = type_name.clone();
                if let Some(f) = format {
                    type_label = format!("{} ({})", type_label, f);
                }
                row = row.child(
                    // Monospace wrapper so the primitive type/format label
                    // inherits `font_mono` (finding #20).
                    div().font(mono.clone()).child(Badge::new(type_label)),
                );

                if required {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.error)
                            .child("required"),
                    );
                }

                if let Some(desc) = node.description() {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child(desc.to_string()),
                    );
                }

                if let Some(values) = enum_values {
                    row = row.child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .font(mono.clone())
                            .child(format!("enum: [{}]", values.join(", "))),
                    );
                }

                row.into_any_element()
            }
        }
    }
}

impl Render for SchemaDisplayView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let content = self.render_node(&self.schema.clone(), None, "", 0, false, theme, cx);

        div()
            .flex()
            .flex_col()
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .overflow_hidden()
            .text_style(TextStyle::Subheadline, theme)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::{SchemaNode, parse_schema};
    use core::prelude::v1::test;
    use serde_json::json;

    #[test]
    fn parse_basic_object_with_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        });
        let node = parse_schema(&schema);
        match node {
            SchemaNode::Object {
                properties,
                required,
                description,
            } => {
                assert_eq!(properties.len(), 2);
                assert!(required.contains("name"));
                assert!(!required.contains("age"));
                assert!(description.is_none());
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn parse_nested_object() {
        let schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    },
                    "required": ["city"]
                }
            }
        });
        let node = parse_schema(&schema);
        match &node {
            SchemaNode::Object { properties, .. } => {
                let (_, inner) = properties.iter().find(|(k, _)| k == "address").unwrap();
                match inner {
                    SchemaNode::Object {
                        properties: inner_props,
                        required: inner_req,
                        ..
                    } => {
                        assert_eq!(inner_props.len(), 1);
                        assert_eq!(inner_props[0].0, "city");
                        assert!(inner_req.contains("city"));
                    }
                    _ => panic!("expected nested Object"),
                }
            }
            _ => panic!("expected Object"),
        }
    }

    #[test]
    fn parse_array_with_typed_items() {
        let schema = json!({
            "type": "array",
            "items": { "type": "number", "description": "a score" },
            "description": "list of scores"
        });
        let node = parse_schema(&schema);
        match &node {
            SchemaNode::Array { items, description } => {
                assert_eq!(description.as_deref(), Some("list of scores"));
                assert_eq!(items.type_label(), "number");
                assert_eq!(items.description(), Some("a score"));
            }
            _ => panic!("expected Array"),
        }
    }

    #[test]
    fn parse_primitive_with_format_and_enum() {
        let schema = json!({
            "type": "string",
            "format": "date-time",
            "enum": ["a", "b", "c"],
            "description": "pick one"
        });
        let node = parse_schema(&schema);
        match &node {
            SchemaNode::Primitive {
                type_name,
                format,
                enum_values,
                description,
            } => {
                assert_eq!(type_name, "string");
                assert_eq!(format.as_deref(), Some("date-time"));
                assert_eq!(description.as_deref(), Some("pick one"));
                assert_eq!(enum_values.as_ref().unwrap(), &["a", "b", "c"]);
            }
            _ => panic!("expected Primitive"),
        }
    }

    #[test]
    fn parse_missing_type_defaults_to_any() {
        let schema = json!({ "description": "mystery" });
        let node = parse_schema(&schema);
        assert_eq!(node.type_label(), "any");
        assert_eq!(node.description(), Some("mystery"));
    }

    #[test]
    fn parse_bare_null() {
        let node = parse_schema(&serde_json::Value::Null);
        assert_eq!(node.type_label(), "any");
        assert!(node.description().is_none());
    }

    #[test]
    fn parse_empty_object() {
        let schema = json!({});
        let node = parse_schema(&schema);
        // Empty object has no "type" field, defaults to "any"
        assert_eq!(node.type_label(), "any");
    }
}
