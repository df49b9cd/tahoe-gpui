//! API endpoint schema display component.
//!
//! Visualizes REST API endpoints with HTTP method badges, path parameter
//! highlighting, and collapsible sections for parameters, request body,
//! and response body schemas.

use crate::foundations::layout::SPACING_4;
use std::collections::HashSet;

use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::layout_and_organization::disclosure_group::DisclosureGroup;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, Context, ElementId, FontWeight, Hsla, SharedString, Window, div, px};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// HTTP method for an API endpoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }

    fn color(&self, theme: &TahoeTheme) -> Hsla {
        match self {
            Self::Get => theme.success,
            Self::Post => theme.info,
            Self::Put => theme.palette.orange,
            Self::Patch => theme.palette.yellow,
            Self::Delete => theme.error,
        }
    }
}

/// Location of an API parameter.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ParameterLocation {
    #[default]
    Query,
    Path,
    Header,
}

impl ParameterLocation {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Path => "path",
            Self::Header => "header",
        }
    }
}

/// An API endpoint parameter (query, path, or header).
pub struct EndpointParameter {
    pub name: SharedString,
    pub type_name: SharedString,
    pub required: bool,
    pub description: Option<SharedString>,
    pub location: ParameterLocation,
}

impl EndpointParameter {
    pub fn new(name: impl Into<SharedString>, type_name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            required: false,
            description: None,
            location: ParameterLocation::default(),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn location(mut self, location: ParameterLocation) -> Self {
        self.location = location;
        self
    }
}

/// A schema property for request/response bodies.
/// Recursive: objects have nested `properties`, arrays have `items`.
pub struct EndpointProperty {
    pub name: SharedString,
    pub type_name: SharedString,
    pub required: bool,
    pub description: Option<SharedString>,
    pub properties: Vec<EndpointProperty>,
    pub items: Option<Box<EndpointProperty>>,
}

impl EndpointProperty {
    pub fn new(name: impl Into<SharedString>, type_name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            required: false,
            description: None,
            properties: Vec::new(),
            items: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn properties(mut self, properties: Vec<EndpointProperty>) -> Self {
        self.properties = properties;
        self
    }

    pub fn items(mut self, items: EndpointProperty) -> Self {
        self.items = Some(Box::new(items));
        self
    }

    fn has_children(&self) -> bool {
        !self.properties.is_empty() || self.items.is_some()
    }
}

// ---------------------------------------------------------------------------
// Path parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    Literal(String),
    Param(String),
}

fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_param = false;

    for ch in path.chars() {
        match ch {
            '{' if !in_param => {
                if !current.is_empty() {
                    segments.push(PathSegment::Literal(std::mem::take(&mut current)));
                }
                in_param = true;
            }
            '}' if in_param => {
                if !current.is_empty() {
                    segments.push(PathSegment::Param(std::mem::take(&mut current)));
                }
                in_param = false;
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        if in_param {
            // Unclosed brace — treat remainder as literal
            segments.push(PathSegment::Literal(format!("{{{}", current)));
        } else {
            segments.push(PathSegment::Literal(current));
        }
    }

    segments
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// An API endpoint display showing method, path, parameters, and body schemas.
pub struct ApiEndpointView {
    method: HttpMethod,
    path: SharedString,
    description: Option<SharedString>,
    parameters: Vec<EndpointParameter>,
    request_body: Vec<EndpointProperty>,
    response_body: Vec<EndpointProperty>,
    // Expand/collapse state
    params_open: bool,
    request_open: bool,
    response_open: bool,
    expanded_properties: HashSet<String>,
}

impl ApiEndpointView {
    pub fn new(method: HttpMethod, path: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self {
        Self {
            method,
            path: path.into(),
            description: None,
            parameters: Vec::new(),
            request_body: Vec::new(),
            response_body: Vec::new(),
            params_open: false,
            request_open: false,
            response_open: false,
            expanded_properties: HashSet::new(),
        }
    }

    pub fn description(&mut self, desc: impl Into<SharedString>) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    pub fn parameters(&mut self, params: Vec<EndpointParameter>) -> &mut Self {
        self.parameters = params;
        self
    }

    pub fn request_body(&mut self, props: Vec<EndpointProperty>) -> &mut Self {
        self.request_body = props;
        self
    }

    pub fn response_body(&mut self, props: Vec<EndpointProperty>) -> &mut Self {
        self.response_body = props;
        self
    }

    fn toggle_params(&mut self, open: bool, cx: &mut Context<Self>) {
        self.params_open = open;
        cx.notify();
    }

    fn toggle_request(&mut self, open: bool, cx: &mut Context<Self>) {
        self.request_open = open;
        cx.notify();
    }

    fn toggle_response(&mut self, open: bool, cx: &mut Context<Self>) {
        self.response_open = open;
        cx.notify();
    }

    fn toggle_property(&mut self, path: String, cx: &mut Context<Self>) {
        if self.expanded_properties.contains(&path) {
            self.expanded_properties.remove(&path);
        } else {
            self.expanded_properties.insert(path);
        }
        cx.notify();
    }

    // -- Render helpers -----------------------------------------------------

    fn render_method_badge(&self, theme: &TahoeTheme) -> impl IntoElement {
        let color = self.method.color(theme);
        div()
            .px(theme.spacing_sm)
            .py(px(2.0))
            .rounded(theme.radius_full)
            .bg(color)
            .text_color(theme.text_on_accent)
            .text_style(TextStyle::Caption1, theme)
            .font_weight(theme.effective_weight(FontWeight::BOLD))
            .child(self.method.label())
    }

    fn render_path(&self, theme: &TahoeTheme) -> impl IntoElement {
        let segments = parse_path_segments(&self.path);
        let mut row = div()
            .flex()
            .items_center()
            .font(theme.font_mono())
            .text_style(TextStyle::Subheadline, theme);

        for seg in segments {
            match seg {
                PathSegment::Literal(text) => {
                    row = row.child(div().text_color(theme.text).child(text));
                }
                PathSegment::Param(name) => {
                    row = row.child(
                        div()
                            .px(px(SPACING_4))
                            .rounded(theme.radius_sm)
                            .bg(theme.code_bg)
                            .text_color(theme.accent)
                            .child(format!("{{{}}}", name)),
                    );
                }
            }
        }

        row
    }

    fn render_parameter_row(
        &self,
        param: &EndpointParameter,
        theme: &TahoeTheme,
    ) -> impl IntoElement {
        let mut row = div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .py(theme.spacing_xs);

        row = row.child(
            div()
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text)
                .child(param.name.clone()),
        );

        row = row.child(Badge::new(param.type_name.clone()));
        row = row.child(Badge::new(param.location.label()).variant(BadgeVariant::Muted));

        if param.required {
            row = row.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.error)
                    .child("required"),
            );
        }

        if let Some(desc) = &param.description {
            row = row.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(desc.clone()),
            );
        }

        row
    }

    fn render_property(
        &self,
        prop: &EndpointProperty,
        path: &str,
        depth: usize,
        theme: &TahoeTheme,
        cx: &Context<Self>,
    ) -> AnyElement {
        let indent = px(depth as f32 * 16.0);
        let has_children = prop.has_children();

        let mut container = div().flex().flex_col();

        // Build the inline children shared by both branches
        let name_el = div()
            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text)
            .child(prop.name.clone());
        let badge_el = Badge::new(prop.type_name.clone());
        let required_el = if prop.required {
            Some(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.error)
                    .child("required"),
            )
        } else {
            None
        };
        let desc_el = prop.description.as_ref().map(|desc| {
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(desc.clone())
        });

        // Header row: stateful (interactive) when expandable, plain div otherwise
        if has_children {
            let is_expanded = self.expanded_properties.contains(path);
            let path_owned = path.to_string();

            let mut row = div()
                .flex()
                .items_center()
                .gap(theme.spacing_xs)
                .pl(indent)
                .py(theme.spacing_xs)
                .cursor_pointer()
                .id(ElementId::Name(SharedString::from(format!(
                    "api-prop-{}",
                    path
                ))))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.toggle_property(path_owned.clone(), cx);
                }))
                .child(
                    Icon::new(if is_expanded {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .size(px(12.0)),
                )
                .child(name_el)
                .child(badge_el);

            if let Some(el) = required_el {
                row = row.child(el);
            }
            if let Some(el) = desc_el {
                row = row.child(el);
            }
            container = container.child(row);
        } else {
            let mut row = div()
                .flex()
                .items_center()
                .gap(theme.spacing_xs)
                .pl(indent)
                .py(theme.spacing_xs)
                .child(name_el)
                .child(badge_el);

            if let Some(el) = required_el {
                row = row.child(el);
            }
            if let Some(el) = desc_el {
                row = row.child(el);
            }
            container = container.child(row);
        }

        // Children (when expanded)
        if has_children && self.expanded_properties.contains(path) {
            for child_prop in &prop.properties {
                let child_path = format!("{}.{}", path, child_prop.name);
                container = container.child(self.render_property(
                    child_prop,
                    &child_path,
                    depth + 1,
                    theme,
                    cx,
                ));
            }
            if let Some(items) = &prop.items {
                let items_path = format!("{}[]", path);
                container =
                    container.child(self.render_property(items, &items_path, depth + 1, theme, cx));
            }
        }

        container.into_any_element()
    }

    fn render_properties_list(
        &self,
        props: &[EndpointProperty],
        prefix: &str,
        theme: &TahoeTheme,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let mut list = div().flex().flex_col();
        for prop in props {
            let path = format!("{}.{}", prefix, prop.name);
            list = list.child(self.render_property(prop, &path, 0, theme, cx));
        }
        list
    }
}

impl Render for ApiEndpointView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut content = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .p(theme.spacing_md)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .overflow_hidden()
            .bg(theme.surface);

        // -- Header: method badge + path -----------------------------------
        let header = div().flex().flex_col().gap(theme.spacing_sm).child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .child(self.render_method_badge(theme))
                .child(self.render_path(theme)),
        );

        content = content.child(header);

        // -- Description ---------------------------------------------------
        if let Some(desc) = &self.description {
            content = content.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(desc.clone()),
            );
        }

        // -- Parameters section --------------------------------------------
        if !self.parameters.is_empty() {
            let params_open = self.params_open;
            let params_header = div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .child(
                    div()
                        .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                        .text_style(TextStyle::Subheadline, theme)
                        .child("Parameters"),
                )
                .child(
                    Badge::new(format!("{}", self.parameters.len())).variant(BadgeVariant::Muted),
                );

            let mut params_body = div().flex().flex_col();
            for param in &self.parameters {
                params_body = params_body.child(self.render_parameter_row(param, theme));
            }

            let entity = cx.entity().clone();
            content = content.child(
                DisclosureGroup::new("api-params", params_header, params_body)
                    .open(params_open)
                    .on_toggle(move |new_state, _window, cx| {
                        entity.update(cx, |this, cx| this.toggle_params(new_state, cx));
                    }),
            );
        }

        // -- Request body section ------------------------------------------
        if !self.request_body.is_empty() {
            let request_open = self.request_open;
            let request_header = div()
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .text_style(TextStyle::Subheadline, theme)
                .child("Request Body");

            let request_body_el =
                self.render_properties_list(&self.request_body, "request", theme, cx);

            let entity = cx.entity().clone();
            content = content.child(
                DisclosureGroup::new("api-request", request_header, request_body_el)
                    .open(request_open)
                    .on_toggle(move |new_state, _window, cx| {
                        entity.update(cx, |this, cx| this.toggle_request(new_state, cx));
                    }),
            );
        }

        // -- Response body section -----------------------------------------
        if !self.response_body.is_empty() {
            let response_open = self.response_open;
            let response_header = div()
                .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                .text_style(TextStyle::Subheadline, theme)
                .child("Response Body");

            let response_body_el =
                self.render_properties_list(&self.response_body, "response", theme, cx);

            let entity = cx.entity().clone();
            content = content.child(
                DisclosureGroup::new("api-response", response_header, response_body_el)
                    .open(response_open)
                    .on_toggle(move |new_state, _window, cx| {
                        entity.update(cx, |this, cx| this.toggle_response(new_state, cx));
                    }),
            );
        }

        content
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        EndpointParameter, EndpointProperty, HttpMethod, ParameterLocation, PathSegment,
        parse_path_segments,
    };
    use core::prelude::v1::test;

    // -- HttpMethod ---------------------------------------------------------

    #[test]
    fn http_method_labels() {
        assert_eq!(HttpMethod::Get.label(), "GET");
        assert_eq!(HttpMethod::Post.label(), "POST");
        assert_eq!(HttpMethod::Put.label(), "PUT");
        assert_eq!(HttpMethod::Patch.label(), "PATCH");
        assert_eq!(HttpMethod::Delete.label(), "DELETE");
    }

    // -- ParameterLocation --------------------------------------------------

    #[test]
    fn parameter_location_default_is_query() {
        assert_eq!(ParameterLocation::default(), ParameterLocation::Query);
    }

    #[test]
    fn parameter_location_labels() {
        assert_eq!(ParameterLocation::Query.label(), "query");
        assert_eq!(ParameterLocation::Path.label(), "path");
        assert_eq!(ParameterLocation::Header.label(), "header");
    }

    // -- EndpointParameter builder ------------------------------------------

    #[test]
    fn endpoint_parameter_defaults() {
        let param = EndpointParameter::new("id", "string");
        assert_eq!(param.name.as_ref(), "id");
        assert_eq!(param.type_name.as_ref(), "string");
        assert!(!param.required);
        assert!(param.description.is_none());
        assert_eq!(param.location, ParameterLocation::Query);
    }

    #[test]
    fn endpoint_parameter_builder() {
        let param = EndpointParameter::new("userId", "integer")
            .required(true)
            .description("The user identifier")
            .location(ParameterLocation::Path);
        assert!(param.required);
        assert_eq!(param.description.unwrap().as_ref(), "The user identifier");
        assert_eq!(param.location, ParameterLocation::Path);
    }

    // -- EndpointProperty builder -------------------------------------------

    #[test]
    fn endpoint_property_defaults() {
        let prop = EndpointProperty::new("name", "string");
        assert_eq!(prop.name.as_ref(), "name");
        assert_eq!(prop.type_name.as_ref(), "string");
        assert!(!prop.required);
        assert!(prop.description.is_none());
        assert!(prop.properties.is_empty());
        assert!(prop.items.is_none());
        assert!(!prop.has_children());
    }

    #[test]
    fn endpoint_property_with_nested() {
        let prop = EndpointProperty::new("address", "object")
            .required(true)
            .properties(vec![
                EndpointProperty::new("city", "string").required(true),
                EndpointProperty::new("zip", "string"),
            ]);
        assert!(prop.has_children());
        assert_eq!(prop.properties.len(), 2);
        assert!(prop.properties[0].required);
    }

    #[test]
    fn endpoint_property_with_items() {
        let prop =
            EndpointProperty::new("tags", "array").items(EndpointProperty::new("item", "string"));
        assert!(prop.has_children());
        assert!(prop.items.is_some());
    }

    // -- Path parsing -------------------------------------------------------

    #[test]
    fn parse_simple_path() {
        assert_eq!(
            parse_path_segments("/users"),
            vec![PathSegment::Literal("/users".into())],
        );
    }

    #[test]
    fn parse_path_with_single_param() {
        assert_eq!(
            parse_path_segments("/users/{id}"),
            vec![
                PathSegment::Literal("/users/".into()),
                PathSegment::Param("id".into()),
            ],
        );
    }

    #[test]
    fn parse_path_with_multiple_params() {
        assert_eq!(
            parse_path_segments("/users/{userId}/posts/{postId}"),
            vec![
                PathSegment::Literal("/users/".into()),
                PathSegment::Param("userId".into()),
                PathSegment::Literal("/posts/".into()),
                PathSegment::Param("postId".into()),
            ],
        );
    }

    #[test]
    fn parse_param_at_start() {
        assert_eq!(
            parse_path_segments("{version}/users"),
            vec![
                PathSegment::Param("version".into()),
                PathSegment::Literal("/users".into()),
            ],
        );
    }

    #[test]
    fn parse_empty_path() {
        assert_eq!(parse_path_segments(""), Vec::<PathSegment>::new());
    }

    #[test]
    fn parse_unclosed_brace_is_literal() {
        assert_eq!(
            parse_path_segments("/users/{id"),
            vec![
                PathSegment::Literal("/users/".into()),
                PathSegment::Literal("{id".into()),
            ],
        );
    }
}
