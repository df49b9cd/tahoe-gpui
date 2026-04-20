//! Environment variables display component with value toggle, copy, grouping, and required badges.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::callback_types::OnToggle;
use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::layout_and_organization::disclosure_group::DisclosureGroup;
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::components::selection_and_input::toggle::Toggle;
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{App, Context, ElementId, Entity, SharedString, Window, div, px};

/// Format used when copying an environment variable.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CopyFormat {
    /// Copy just the variable name.
    Name,
    /// Copy just the value (default).
    #[default]
    Value,
    /// Copy as `export KEY="value"`.
    Export,
}

/// Format an environment variable for clipboard copy.
pub fn format_env_copy(var: &EnvVar, format: CopyFormat) -> String {
    match format {
        CopyFormat::Name => var.key.to_string(),
        CopyFormat::Value => var.value.to_string(),
        CopyFormat::Export => format!("export {}=\"{}\"", var.key, var.value),
    }
}

/// A single environment variable entry.
pub struct EnvVar {
    pub key: SharedString,
    pub value: SharedString,
    pub sensitive: bool,
    pub required: bool,
    pub group: Option<SharedString>,
    pub copy_format: Option<CopyFormat>,
    pub copy_timeout: Option<Duration>,
    pub on_copy: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    pub on_error: Option<Arc<dyn Fn(String) + Send + Sync + 'static>>,
}

impl EnvVar {
    pub fn new(key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            sensitive: false,
            required: false,
            group: None,
            copy_format: None,
            copy_timeout: None,
            on_copy: None,
            on_error: None,
        }
    }

    /// Mark this variable as sensitive (will be masked when hidden).
    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    /// Mark this variable as required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Assign this variable to a named group.
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Override the copy format for this variable's copy button.
    pub fn copy_format(mut self, format: CopyFormat) -> Self {
        self.copy_format = Some(format);
        self
    }

    /// Override the copy button feedback timeout for this variable.
    pub fn copy_timeout(mut self, timeout: Duration) -> Self {
        self.copy_timeout = Some(timeout);
        self
    }

    /// Set a callback invoked after this variable is copied.
    pub fn on_copy(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_copy = Some(Arc::new(handler));
        self
    }

    /// Set a callback invoked when copying this variable fails.
    pub fn on_error(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_error = Some(Arc::new(handler));
        self
    }
}

/// A display of environment variable key-value pairs with interactive toggle,
/// copy buttons, required badges, and optional grouping.
pub struct EnvironmentVariablesView {
    vars: Vec<EnvVar>,
    show_values: bool,
    interactive: bool,
    copy_format: CopyFormat,
    copy_buttons: Vec<Entity<CopyButton>>,
    group_states: HashMap<SharedString, bool>,
    title: SharedString,
    toggle_id: ElementId,
    on_show_values_change: OnToggle,
}

impl EnvironmentVariablesView {
    pub fn new(vars: Vec<EnvVar>, cx: &mut App) -> Entity<Self> {
        let copy_buttons: Vec<Entity<CopyButton>> = vars
            .iter()
            .map(|var| {
                let btn = CopyButton::new(var.value.to_string(), cx);
                btn.update(cx, |btn, _cx| {
                    if let Some(timeout) = var.copy_timeout {
                        btn.set_timeout(timeout);
                    }
                    if let Some(ref on_copy) = var.on_copy {
                        btn.set_on_copy(on_copy.clone());
                    }
                    if let Some(ref on_error) = var.on_error {
                        btn.set_on_error(on_error.clone());
                    }
                });
                btn
            })
            .collect();

        let mut group_states = HashMap::new();
        for var in &vars {
            if let Some(group) = &var.group {
                group_states.entry(group.clone()).or_insert(true);
            }
        }

        cx.new(|_| Self {
            vars,
            show_values: false,
            interactive: false,
            copy_format: CopyFormat::default(),
            copy_buttons,
            group_states,
            title: "Environment Variables".into(),
            toggle_id: next_element_id("env-toggle"),
            on_show_values_change: None,
        })
    }

    /// Set whether to show values (default: false, which masks sensitive values).
    pub fn set_show_values(&mut self, show: bool) {
        self.show_values = show;
    }

    /// Enable interactive toggle button in the header.
    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    /// Set the copy format for all copy buttons.
    pub fn set_copy_format(&mut self, format: CopyFormat) {
        self.copy_format = format;
    }

    /// Set the header title (default: "Environment Variables").
    pub fn set_title(&mut self, title: impl Into<SharedString>) {
        self.title = title.into();
    }

    /// Set a callback invoked when the visibility toggle changes.
    pub fn set_on_show_values_change(
        &mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) {
        self.on_show_values_change = Some(Box::new(handler));
    }

    fn toggle_group(&mut self, group: SharedString, cx: &mut Context<Self>) {
        let current = self.group_states.get(&group).copied().unwrap_or(true);
        self.group_states.insert(group, !current);
        cx.notify();
    }
}

impl Render for EnvironmentVariablesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let show = self.show_values;

        // Update copy button content based on current format.
        for (i, var) in self.vars.iter().enumerate() {
            if let Some(btn) = self.copy_buttons.get(i) {
                let effective_format = var.copy_format.unwrap_or(self.copy_format);
                let content = format_env_copy(var, effective_format);
                btn.update(cx, |btn, _cx| {
                    btn.set_content(content);
                });
            }
        }

        let mut container = div()
            .flex()
            .flex_col()
            .bg(theme.code_bg)
            .rounded(theme.radius_md)
            .overflow_hidden();

        // Header with optional interactive toggle
        if self.interactive {
            let entity = cx.entity().clone();
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(theme.spacing_md)
                    .py(theme.spacing_xs)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, &theme)
                            .text_color(theme.text_muted)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(theme.spacing_xs)
                            .child(
                                div()
                                    .text_style(TextStyle::Caption1, &theme)
                                    .text_color(theme.text_muted)
                                    .child(if show { "Visible" } else { "Hidden" }),
                            )
                            .child(Toggle::new(self.toggle_id.clone()).checked(show).on_change(
                                move |new_val, window, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.show_values = new_val;
                                        if let Some(ref handler) = this.on_show_values_change {
                                            handler(new_val, window, cx);
                                        }
                                        cx.notify();
                                    });
                                },
                            )),
                    ),
            );
        }

        // Partition variables into ungrouped and grouped.
        let mut ungrouped_indices: Vec<usize> = Vec::new();
        let mut grouped: Vec<(SharedString, Vec<usize>)> = Vec::new();
        let mut group_order: Vec<SharedString> = Vec::new();

        for (i, var) in self.vars.iter().enumerate() {
            if let Some(group) = &var.group {
                if let Some(pos) = group_order.iter().position(|g| g == group) {
                    grouped[pos].1.push(i);
                } else {
                    group_order.push(group.clone());
                    grouped.push((group.clone(), vec![i]));
                }
            } else {
                ungrouped_indices.push(i);
            }
        }

        let mut vars_list = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .font(theme.mono_font())
            .text_style(TextStyle::Subheadline, &theme)
            .px(theme.spacing_md)
            .py(theme.spacing_sm);

        // Render ungrouped variables flat.
        for &idx in &ungrouped_indices {
            vars_list = vars_list.child(self.render_var_row(idx, &theme));
        }

        // Render grouped variables in collapsibles.
        for (group_name, indices) in &grouped {
            let is_open = self.group_states.get(group_name).copied().unwrap_or(true);
            let count = indices.len();

            let header = div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .child(
                    div()
                        .text_color(theme.text_muted)
                        .text_style(TextStyle::Caption1, &theme)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(group_name.clone()),
                )
                .child(
                    Badge::new(SharedString::from(count.to_string())).variant(BadgeVariant::Muted),
                );

            let mut body = div().flex().flex_col().gap(px(2.0));
            for &idx in indices {
                body = body.child(self.render_var_row(idx, &theme));
            }

            let entity = cx.entity().clone();
            let group_key = group_name.clone();
            vars_list = vars_list.child(
                DisclosureGroup::new(
                    SharedString::from(format!("env-group-{}", group_name)),
                    header,
                    body,
                )
                .open(is_open)
                .on_toggle(move |_new_state, _window, cx| {
                    let key = group_key.clone();
                    entity.update(cx, |this, cx| this.toggle_group(key, cx));
                }),
            );
        }

        container = container.child(vars_list);
        container
    }
}

impl EnvironmentVariablesView {
    fn render_var_row(&self, idx: usize, theme: &TahoeTheme) -> impl IntoElement {
        let var = &self.vars[idx];
        let show = self.show_values;
        let display_value: SharedString = if !show && var.sensitive {
            "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".into()
        } else {
            var.value.clone()
        };

        let mut row = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .py(px(2.0))
            .child(div().text_color(theme.accent).child(var.key.clone()))
            .child(div().text_color(theme.text_muted).child("="))
            .child(
                div()
                    .text_color(if !show && var.sensitive {
                        theme.text_muted
                    } else {
                        theme.text
                    })
                    .child(display_value),
            );

        if var.required {
            row = row.child(Badge::new("Required").variant(BadgeVariant::Warning));
        }

        if let Some(copy_btn) = self.copy_buttons.get(idx) {
            row = row.child(div().ml_auto().child(copy_btn.clone()));
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::{CopyFormat, EnvVar, format_env_copy};
    use core::prelude::v1::test;
    use std::time::Duration;

    #[test]
    fn env_var_new() {
        let v = EnvVar::new("KEY", "value");
        assert_eq!(v.key.as_ref(), "KEY");
        assert_eq!(v.value.as_ref(), "value");
        assert!(!v.sensitive);
        assert!(!v.required);
        assert!(v.group.is_none());
        assert!(v.copy_format.is_none());
        assert!(v.copy_timeout.is_none());
        assert!(v.on_copy.is_none());
        assert!(v.on_error.is_none());
    }

    #[test]
    fn env_var_sensitive() {
        let v = EnvVar::new("SECRET", "hidden").sensitive(true);
        assert!(v.sensitive);
    }

    #[test]
    fn env_var_required() {
        let v = EnvVar::new("DB_URL", "").required(true);
        assert!(v.required);
    }

    #[test]
    fn env_var_group() {
        let v = EnvVar::new("KEY", "val").group("Database");
        assert_eq!(v.group.as_ref().map(|s| s.as_ref()), Some("Database"));
    }

    #[test]
    fn env_var_full_builder_chain() {
        let v = EnvVar::new("SECRET", "s3cret")
            .sensitive(true)
            .required(true)
            .group("Auth");
        assert!(v.sensitive);
        assert!(v.required);
        assert_eq!(v.group.as_ref().map(|s| s.as_ref()), Some("Auth"));
    }

    #[test]
    fn copy_format_default_is_value() {
        assert_eq!(CopyFormat::default(), CopyFormat::Value);
    }

    #[test]
    fn format_env_copy_name() {
        let v = EnvVar::new("API_KEY", "sk-123");
        assert_eq!(format_env_copy(&v, CopyFormat::Name), "API_KEY");
    }

    #[test]
    fn format_env_copy_value() {
        let v = EnvVar::new("API_KEY", "sk-123");
        assert_eq!(format_env_copy(&v, CopyFormat::Value), "sk-123");
    }

    #[test]
    fn format_env_copy_export() {
        let v = EnvVar::new("API_KEY", "sk-123");
        assert_eq!(
            format_env_copy(&v, CopyFormat::Export),
            r#"export API_KEY="sk-123""#
        );
    }

    #[test]
    fn format_env_copy_export_with_quotes() {
        let v = EnvVar::new("MSG", r#"hello "world""#);
        assert_eq!(
            format_env_copy(&v, CopyFormat::Export),
            r#"export MSG="hello "world"""#
        );
    }

    #[test]
    fn env_var_copy_format_override() {
        let v = EnvVar::new("KEY", "val").copy_format(CopyFormat::Export);
        assert_eq!(v.copy_format, Some(CopyFormat::Export));
    }

    #[test]
    fn env_var_copy_timeout() {
        let v = EnvVar::new("KEY", "val").copy_timeout(Duration::from_millis(5000));
        assert_eq!(v.copy_timeout, Some(Duration::from_millis(5000)));
    }

    #[test]
    fn env_var_on_copy_is_some() {
        let v = EnvVar::new("KEY", "val").on_copy(|| {});
        assert!(v.on_copy.is_some());
    }

    #[test]
    fn env_var_on_error_is_some() {
        let v = EnvVar::new("KEY", "val").on_error(|_| {});
        assert!(v.on_error.is_some());
    }
}
