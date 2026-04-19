//! Sandbox display component with collapsible header, status badge, and tabbed Code/Output views.

use crate::callback_types::OnClick;
use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::components::status::activity_indicator::ActivityIndicator;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use crate::markdown::code_block::CodeBlockView;
use gpui::prelude::*;
use gpui::{ClickEvent, ElementId, Entity, FontWeight, SharedString, Window, div};

/// Execution status of a sandbox environment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SandboxStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Error,
}

/// Returns the badge variant and label for a sandbox status.
pub fn status_badge(status: SandboxStatus) -> (BadgeVariant, &'static str) {
    match status {
        SandboxStatus::Pending => (BadgeVariant::Muted, "Pending"),
        SandboxStatus::Running => (BadgeVariant::Warning, "Running"),
        SandboxStatus::Completed => (BadgeVariant::Success, "Completed"),
        SandboxStatus::Error => (BadgeVariant::Error, "Error"),
    }
}

/// Active tab in the sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SandboxTab {
    #[default]
    Code,
    Output,
}

/// A sandbox view with a collapsible header, status badge, and tabbed Code/Output panels.
pub struct SandboxView {
    element_id: ElementId,
    title: Option<SharedString>,
    code: SharedString,
    language: SharedString,
    logs: Option<SharedString>,
    status: SandboxStatus,
    active_tab: SandboxTab,
    is_open: bool,
    copy_button: Entity<CopyButton>,
    on_run: OnClick,
    on_stop: OnClick,
}

impl SandboxView {
    pub fn new(
        code: impl Into<SharedString>,
        language: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let code = code.into();
        let copy_button = CopyButton::new(code.to_string(), cx);
        Self {
            element_id: next_element_id("sandbox"),
            title: None,
            code,
            language: language.into(),
            logs: None,
            status: SandboxStatus::Pending,
            active_tab: SandboxTab::Code,
            is_open: true,
            copy_button,
            on_run: None,
            on_stop: None,
        }
    }

    /// Register a handler for the Run (Play) button in the header. When
    /// neither `on_run` nor `on_stop` are set, the execution controls are
    /// hidden entirely — so callers that only want a read-only preview
    /// keep the pre-existing compact layout. HIG §Workflows: expose the
    /// primary action as an explicit affordance rather than hiding it
    /// behind a collapse toggle.
    pub fn set_on_run(
        &mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) {
        self.on_run = Some(Box::new(handler));
    }

    /// Register a handler for the Stop button in the header. Rendered
    /// instead of Run while `status == Running`. Callers that only supply
    /// `on_run` get a disabled Stop button while running to keep the
    /// layout stable.
    pub fn set_on_stop(
        &mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    ) {
        self.on_stop = Some(Box::new(handler));
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = Some(title.into());
        cx.notify();
    }

    pub fn set_logs(&mut self, logs: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.logs = Some(logs.into());
        cx.notify();
    }

    pub fn set_status(&mut self, status: SandboxStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    pub fn set_active_tab(&mut self, tab: SandboxTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }

    pub fn set_code(&mut self, code: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.code = code.into();
        self.copy_button.update(cx, |btn, _cx| {
            btn.set_content(self.code.to_string());
        });
        cx.notify();
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }
}

impl Render for SandboxView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (badge_variant, badge_label) = status_badge(self.status);
        let is_open = self.is_open;
        let active_tab = self.active_tab;
        let status = self.status;
        let has_run_stop = self.on_run.is_some() || self.on_stop.is_some();

        let title_text = self
            .title
            .clone()
            .unwrap_or_else(|| SharedString::from("Sandbox"));

        // -- Header (clickable, toggles collapse) --
        let mut header = div()
            .id(self.element_id.clone())
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .cursor_pointer()
            .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                this.toggle(cx);
            }))
            .child(
                Icon::new(if is_open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(theme.icon_size_inline)
                .color(theme.text_muted),
            )
            .child(
                Icon::new(IconName::Code)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text)
                    .child(title_text),
            );

        // Status indicator: ActivityIndicator while Running (HIG §Progress
        // Indicators — animated glyph communicates ongoing work), Badge
        // for the other terminal states.
        if status == SandboxStatus::Running {
            let spinner_id =
                SharedString::from(format!("{}-activity", self.element_id.clone()));
            header = header.child(
                ActivityIndicator::new(spinner_id)
                    .label("Running")
                    .size(theme.icon_size_inline)
                    .color(theme.accent),
            );
        } else {
            header = header.child(Badge::new(badge_label).variant(badge_variant));
        }

        // Run / Stop button. Play when idle; Stop while running. Absent
        // handlers hide the affordance so read-only callers keep the
        // compact header. Using `cx.listener` avoids moving the boxed
        // handler out of `self` on each render — the listener dispatches
        // back into `self.on_run` / `self.on_stop` at click time.
        if has_run_stop {
            let run_id = SharedString::from(format!("{}-run", self.element_id.clone()));
            if status == SandboxStatus::Running {
                let mut btn = Button::new(run_id)
                    .icon(Icon::new(IconName::Square).size(theme.icon_size_inline))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm);
                if self.on_stop.is_some() {
                    btn = btn.on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                        cx.stop_propagation();
                        if let Some(handler) = this.on_stop.as_ref() {
                            handler(event, window, &mut *cx);
                        }
                    }));
                } else {
                    btn = btn.disabled(true);
                }
                header = header.child(btn);
            } else {
                let mut btn = Button::new(run_id)
                    .icon(Icon::new(IconName::Play).size(theme.icon_size_inline))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm);
                if self.on_run.is_some() {
                    btn = btn.on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                        cx.stop_propagation();
                        if let Some(handler) = this.on_run.as_ref() {
                            handler(event, window, &mut *cx);
                        }
                    }));
                } else {
                    btn = btn.disabled(true);
                }
                header = header.child(btn);
            }
        }

        // -- Container --
        let mut container = crate::foundations::materials::card_surface(theme).child(header);

        if !is_open {
            return container;
        }

        // -- Tab bar --
        let code_tab_active = active_tab == SandboxTab::Code;
        let output_tab_active = active_tab == SandboxTab::Output;

        let tab_bar = div()
            .flex()
            .items_center()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .id("sandbox-tab-code")
                    .px(theme.spacing_md)
                    .py(theme.spacing_xs)
                    .cursor_pointer()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(if code_tab_active {
                        theme.text
                    } else {
                        theme.text_muted
                    })
                    .when(code_tab_active, |el| {
                        el.border_b_2().border_color(theme.accent)
                    })
                    .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                        this.set_active_tab(SandboxTab::Code, cx);
                    }))
                    .child("Code"),
            )
            .child(
                div()
                    .id("sandbox-tab-output")
                    .px(theme.spacing_md)
                    .py(theme.spacing_xs)
                    .cursor_pointer()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(if output_tab_active {
                        theme.text
                    } else {
                        theme.text_muted
                    })
                    .when(output_tab_active, |el| {
                        el.border_b_2().border_color(theme.accent)
                    })
                    .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                        this.set_active_tab(SandboxTab::Output, cx);
                    }))
                    .child("Output"),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_end()
                    .px(theme.spacing_sm)
                    .child(self.copy_button.clone()),
            );

        container = container.child(tab_bar);

        // -- Tab content --
        match active_tab {
            SandboxTab::Code => {
                container = container.child(
                    CodeBlockView::new(self.code.to_string())
                        .language(Some(self.language.to_string()))
                        .show_header(false),
                );
            }
            SandboxTab::Output => {
                let content = if let Some(ref logs) = self.logs {
                    div()
                        .px(theme.spacing_md)
                        .py(theme.spacing_sm)
                        .font_family(theme.font_mono.clone())
                        .text_style(TextStyle::Subheadline, theme)
                        .text_color(theme.text)
                        .child(logs.clone())
                } else {
                    div()
                        .px(theme.spacing_md)
                        .py(theme.spacing_sm)
                        .text_style(TextStyle::Subheadline, theme)
                        .text_color(theme.text_muted)
                        .child("No output yet")
                };
                container = container.child(content);
            }
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{SandboxStatus, SandboxTab, status_badge};
    use crate::components::content::badge::BadgeVariant;
    use core::prelude::v1::test;

    #[test]
    fn sandbox_status_variants() {
        assert_ne!(SandboxStatus::Pending, SandboxStatus::Running);
        assert_ne!(SandboxStatus::Running, SandboxStatus::Error);
        assert_ne!(SandboxStatus::Completed, SandboxStatus::Error);
        assert_eq!(SandboxStatus::default(), SandboxStatus::Pending);
    }

    #[test]
    fn sandbox_tab_default() {
        assert_eq!(SandboxTab::default(), SandboxTab::Code);
    }

    #[test]
    fn sandbox_tab_variants() {
        assert_ne!(SandboxTab::Code, SandboxTab::Output);
    }

    #[test]
    fn status_badge_mapping() {
        assert_eq!(
            status_badge(SandboxStatus::Pending),
            (BadgeVariant::Muted, "Pending")
        );
        assert_eq!(
            status_badge(SandboxStatus::Running),
            (BadgeVariant::Warning, "Running")
        );
        assert_eq!(
            status_badge(SandboxStatus::Completed),
            (BadgeVariant::Success, "Completed")
        );
        assert_eq!(
            status_badge(SandboxStatus::Error),
            (BadgeVariant::Error, "Error")
        );
    }
}
