//! Workflow toolbar component.
//!
//! A horizontal bar of action buttons, typically placed above the canvas.
//! Per HIG Toolbars (macOS 26 update) the bar renders on a Liquid Glass
//! material with leading / center / trailing sections so actions sort into
//! the three canonical anatomy groups instead of a single undifferentiated row.

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{App, ClickEvent, ElementId, SharedString, Window, div};

/// Where an action sits in the Apple toolbar anatomy.
///
/// F23 (#149) — HIG Toolbars: actions form three groups — leading
/// (navigation / disclosure), center (primary title or tool), trailing
/// (document-scoped actions). The groups are divided by tiny separators.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ToolbarSection {
    /// Shown at the bar's leading edge (left in LTR).
    Leading,
    /// Shown in the center. Typically title metadata or the primary tool.
    Center,
    /// Shown at the bar's trailing edge (right in LTR).
    #[default]
    Trailing,
}

/// A single action in the workflow toolbar.
pub struct ToolbarAction {
    /// Display label for the action.
    pub label: SharedString,
    /// Icon to show alongside the label.
    pub icon: IconName,
    /// Which anatomy section the action belongs to.
    pub section: ToolbarSection,
    /// Whether the action is currently disabled (e.g. Undo with empty stack).
    pub disabled: bool,
    /// Click handler.
    pub on_click: Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl ToolbarAction {
    pub fn new(
        label: impl Into<SharedString>,
        icon: IconName,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            icon,
            section: ToolbarSection::default(),
            disabled: false,
            on_click: Box::new(on_click),
        }
    }

    /// Place the action in a specific anatomy section.
    pub fn section(mut self, section: ToolbarSection) -> Self {
        self.section = section;
        self
    }

    /// Mark the action disabled. Used for Undo/Redo affordances (F7).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A horizontal toolbar with action buttons for the workflow editor.
#[derive(IntoElement)]
pub struct WorkflowToolbar {
    actions: Vec<ToolbarAction>,
}

impl Default for WorkflowToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowToolbar {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add an action to the toolbar.
    pub fn action(mut self, action: ToolbarAction) -> Self {
        self.actions.push(action);
        self
    }
}

impl RenderOnce for WorkflowToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Capture theme tokens as owned values so we can mutably borrow `cx`
        // while building the action buttons below (Button::new takes &mut App).
        let (spacing_sm, spacing_md, spacing_xs, bar_bg, shadows, theme_clone) = {
            let theme = cx.theme();
            (
                theme.spacing_sm,
                theme.spacing_md,
                theme.spacing_xs,
                // F23 (#149): Liquid Glass material, per HIG Toolbars on
                // macOS 26. `accessible_bg` respects `reduce_transparency`
                // and high-contrast preferences so the bar stays legible
                // across a11y modes.
                theme
                    .glass
                    .accessible_bg(GlassSize::Medium, theme.accessibility_mode),
                theme.glass.shadows(GlassSize::Medium).to_vec(),
                theme.clone(),
            )
        };

        let mut bar = div()
            .flex()
            .items_center()
            .gap(spacing_sm)
            .px(spacing_md)
            .py(spacing_sm)
            .bg(bar_bg)
            .shadow(shadows);
        bar = crate::foundations::materials::apply_high_contrast_border(bar, &theme_clone);

        // Sort actions into their anatomy sections. Stable partition keeps
        // the within-section order the caller supplied.
        let mut leading = Vec::new();
        let mut center = Vec::new();
        let mut trailing = Vec::new();
        for (idx, action) in self.actions.into_iter().enumerate() {
            match action.section {
                ToolbarSection::Leading => leading.push((idx, action)),
                ToolbarSection::Center => center.push((idx, action)),
                ToolbarSection::Trailing => trailing.push((idx, action)),
            }
        }

        let mut leading_group = div().flex().items_center().gap(spacing_xs);
        for (idx, action) in leading {
            leading_group = leading_group.child(render_action(idx, action, cx));
        }
        let mut center_group = div().flex().items_center().gap(spacing_xs);
        for (idx, action) in center {
            center_group = center_group.child(render_action(idx, action, cx));
        }
        let mut trailing_group = div().flex().items_center().gap(spacing_xs);
        for (idx, action) in trailing {
            trailing_group = trailing_group.child(render_action(idx, action, cx));
        }

        bar.child(leading_group)
            .child(div().flex_1()) // leading spacer
            .child(center_group)
            .child(div().flex_1()) // trailing spacer
            .child(trailing_group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;

    fn test_action(label: &'static str, section: ToolbarSection) -> ToolbarAction {
        ToolbarAction::new(label, IconName::Plus, |_, _, _| {}).section(section)
    }

    #[test]
    fn toolbar_section_default_is_trailing() {
        // HIG default placement: document-level actions sit trailing.
        assert_eq!(ToolbarSection::default(), ToolbarSection::Trailing);
    }

    #[test]
    fn action_builder_section_and_disabled() {
        let action = test_action("Undo", ToolbarSection::Leading).disabled(true);
        assert_eq!(action.section, ToolbarSection::Leading);
        assert!(action.disabled);
    }

    #[test]
    fn toolbar_accepts_multi_section_actions() {
        // Smoke test: the builder accepts actions in any section without
        // panic. Rendering goes through GPUI's window machinery; this
        // test just verifies the model-side wiring.
        let bar = WorkflowToolbar::new()
            .action(test_action("Undo", ToolbarSection::Leading))
            .action(test_action("Save", ToolbarSection::Center))
            .action(test_action("Share", ToolbarSection::Trailing));
        assert_eq!(bar.actions.len(), 3);
    }
}

/// Render a single toolbar action as a ghost button.
fn render_action(
    idx: usize,
    action: ToolbarAction,
    _cx: &mut App,
) -> gpui::AnyElement {
    let handler = action.on_click;
    let mut btn = Button::new(ElementId::NamedInteger(
        "wf-toolbar-action".into(),
        idx as u64,
    ))
    .icon(Icon::new(action.icon))
    .label(action.label)
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::Sm);
    if action.disabled {
        btn = btn.disabled(true);
    } else {
        btn = btn.on_click(move |event, window, cx| handler(event, window, cx));
    }
    btn.into_any_element()
}
