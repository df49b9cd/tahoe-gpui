//! Artifact display component with compound sub-components.
//!
//! Provides a structured container for displaying generated content (code,
//! documents, etc.) with header, title, description, actions, and content areas.
//! Supports both a convenience builder API and compound composition.
//!
//! # Convenience builder
//! ```ignore
//! Artifact::new("My Component")
//!     .description("A React component")
//!     .action(ArtifactAction::new("copy", IconName::Copy).tooltip("Copy"))
//!     .on_close(|w, cx| { /* ... */ })
//!     .content(my_code_block)
//! ```
//!
//! # Compound composition
//! ```ignore
//! Artifact::from_parts()
//!     .header(
//!         ArtifactHeader::new()
//!             .align(crate::components::layout_and_organization::FlexAlign::Start)
//!             .gap(true).border(true)
//!             .child(ArtifactTitle::new("My Component"))
//!             .child(ArtifactDescription::new("A React component"))
//!             .child(
//!                 ArtifactActions::new()
//!                     .action(ArtifactAction::new("copy", IconName::Copy)
//!                         .tooltip("Copy"))
//!             )
//!             .child(ArtifactClose::new("close").on_click(|w, cx| { /* ... */ }))
//!     )
//!     .body(ArtifactContent::new().child(my_code_block))
//! ```

use crate::callback_types::{OnMutCallback, OnToggle};
use crate::components::layout_and_organization::FlexHeader;
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::presentation::tooltip::Tooltip;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, ClickEvent, ElementId, FontWeight, SharedString, Window, div};

// -- ArtifactTitle ------------------------------------------------------------

/// Title text for an artifact header.
#[derive(IntoElement)]
pub struct ArtifactTitle {
    text: SharedString,
}

impl ArtifactTitle {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for ArtifactTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
            .text_color(theme.text)
            .child(self.text)
    }
}

// -- ArtifactDescription ------------------------------------------------------

/// Description text for an artifact header.
#[derive(IntoElement)]
pub struct ArtifactDescription {
    text: SharedString,
}

impl ArtifactDescription {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for ArtifactDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(self.text)
    }
}

// -- ArtifactAction -----------------------------------------------------------

/// An individual action button with tooltip and icon for the artifact header.
#[derive(IntoElement)]
pub struct ArtifactAction {
    id: ElementId,
    icon: IconName,
    tooltip: Option<SharedString>,
    on_click: OnMutCallback,
}

impl ArtifactAction {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            tooltip: None,
            on_click: None,
        }
    }

    /// Set tooltip text shown on hover.
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ArtifactAction {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut btn = Button::new(self.id.clone())
            .icon(
                Icon::new(self.icon)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm);

        if let Some(handler) = self.on_click {
            btn = btn.on_click(move |_, window, cx| handler(window, cx));
        }

        if let Some(tooltip_text) = self.tooltip {
            Tooltip::new(self.id, tooltip_text, btn).into_any_element()
        } else {
            btn.into_any_element()
        }
    }
}

// -- ArtifactActions ----------------------------------------------------------

/// Container for action buttons in the artifact header.
#[derive(Default, IntoElement)]
pub struct ArtifactActions {
    children: Vec<AnyElement>,
}

impl ArtifactActions {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a single action.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.children.push(action.into_any_element());
        self
    }

    /// Add multiple actions.
    pub fn actions(mut self, actions: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(actions.into_iter().map(|a| a.into_any_element()));
        self
    }
}

impl RenderOnce for ArtifactActions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .flex_shrink_0()
            .children(self.children)
    }
}

// -- ArtifactClose ------------------------------------------------------------

/// Close button for the artifact header.
#[derive(IntoElement)]
pub struct ArtifactClose {
    id: ElementId,
    on_click: OnMutCallback,
}

impl ArtifactClose {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on_click: None,
        }
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ArtifactClose {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let tooltip_id = ElementId::from(SharedString::from(format!("{}-tooltip", &self.id)));

        let mut btn = Button::new(self.id)
            .icon(
                Icon::new(IconName::X)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm);

        if let Some(handler) = self.on_click {
            btn = btn.on_click(move |_, window, cx| handler(window, cx));
        }

        Tooltip::new(tooltip_id, "Close", btn)
    }
}

// -- ArtifactHeader -----------------------------------------------------------

/// Header section for an artifact with title, description, actions, and close.
///
/// Type alias for [`FlexHeader`] configured with top-alignment, gap, and border
/// at the call site.
pub type ArtifactHeader = FlexHeader;

// -- ArtifactContent ----------------------------------------------------------

/// Content area for an artifact.
#[derive(Default, IntoElement)]
pub struct ArtifactContent {
    children: Vec<AnyElement>,
}

impl ArtifactContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a child element to the content area.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple child elements to the content area.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for ArtifactContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().children(self.children)
    }
}

// -- Artifact -----------------------------------------------------------------

/// A structured container for displaying generated content with header and
/// content areas.
///
/// Supports two usage modes:
/// - **Convenience builder**: `Artifact::new(title).description(...).content(...)`
/// - **Compound composition**: `Artifact::from_parts().header(...).body(...)`
#[derive(IntoElement)]
pub struct Artifact {
    // Convenience builder fields
    title: Option<SharedString>,
    description: Option<SharedString>,
    actions: Vec<AnyElement>,
    on_close: OnMutCallback,
    body_content: Option<AnyElement>,
    // Compound composition fields
    artifact_header: Option<ArtifactHeader>,
    artifact_content: Option<ArtifactContent>,
    // Collapse state (finding #24). When `collapsible` is set, the header
    // renders a disclosure chevron and the body is hidden while `collapsed`.
    collapsible: bool,
    collapsed: bool,
    on_toggle: OnToggle,
}

impl Artifact {
    /// Convenience constructor with a title. Use builder methods to configure.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: Some(title.into()),
            description: None,
            actions: Vec::new(),
            on_close: None,
            body_content: None,
            artifact_header: None,
            artifact_content: None,
            collapsible: false,
            collapsed: false,
            on_toggle: None,
        }
    }

    /// Compound constructor. Use `.header()` and `.body()` to compose.
    pub fn from_parts() -> Self {
        Self {
            title: None,
            description: None,
            actions: Vec::new(),
            on_close: None,
            body_content: None,
            artifact_header: None,
            artifact_content: None,
            collapsible: false,
            collapsed: false,
            on_toggle: None,
        }
    }

    /// Make the artifact's body collapsible with the given initial state.
    /// Mirrors Zed's `agent_ui` disclosure pattern — caller owns the
    /// `collapsed` flag and receives toggle events via [`Self::on_toggle`].
    /// When `collapsed == true` the body is hidden; the header stays
    /// visible with a disclosure chevron.
    pub fn collapsible(mut self, collapsed: bool) -> Self {
        self.collapsible = true;
        self.collapsed = collapsed;
        self
    }

    /// Set a callback fired when the collapse chevron is clicked. Receives
    /// the new collapsed state (post-toggle). No-op when the artifact is
    /// not [`Self::collapsible`].
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Set the description text (convenience API).
    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add an action element to the header (convenience API).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// Add a close button to the header (convenience API).
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Set the body content (convenience API).
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.body_content = Some(content.into_any_element());
        self
    }

    /// Set the header sub-component (compound API).
    ///
    /// Clears any convenience fields (`title`, `description`, `actions`, `on_close`)
    /// to avoid mixing APIs.
    pub fn header(mut self, header: ArtifactHeader) -> Self {
        self.artifact_header = Some(header);
        self.title = None;
        self.description = None;
        self.actions.clear();
        self.on_close = None;
        self
    }

    /// Set the content sub-component (compound API).
    ///
    /// Clears the convenience `body_content` field to avoid mixing APIs.
    pub fn body(mut self, content: ArtifactContent) -> Self {
        self.artifact_content = Some(content);
        self.body_content = None;
        self
    }
}

impl RenderOnce for Artifact {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            self.artifact_header.is_none() || self.title.is_none(),
            "Artifact: set either `header` (compound) or `title` (convenience), not both"
        );
        debug_assert!(
            self.artifact_content.is_none() || self.body_content.is_none(),
            "Artifact: set either `body` (compound) or `content` (convenience), not both"
        );

        let theme = cx.theme();

        // Build header: compound takes priority, then convenience, then nothing.
        let header_element = if let Some(header) = self.artifact_header {
            Some(header.into_any_element())
        } else if self.title.is_some()
            || self.description.is_some()
            || !self.actions.is_empty()
            || self.on_close.is_some()
        {
            // Convenience path: build header from individual fields.
            let mut title_col = div().flex().flex_col().gap(theme.spacing_xs).flex_1();

            if let Some(title) = self.title {
                title_col = title_col.child(ArtifactTitle::new(title));
            }

            if let Some(desc) = self.description {
                title_col = title_col.child(ArtifactDescription::new(desc));
            }

            let mut header_right = div()
                .flex()
                .items_center()
                .gap(theme.spacing_xs)
                .flex_shrink_0();

            for action in self.actions {
                header_right = header_right.child(action);
            }

            if self.collapsible {
                let collapsed = self.collapsed;
                let on_toggle = self.on_toggle;
                let new_state = !collapsed;
                let mut btn = Button::new("artifact-collapse")
                    .icon(
                        Icon::new(if collapsed {
                            IconName::ChevronRight
                        } else {
                            IconName::ChevronDown
                        })
                        .size(theme.icon_size_inline)
                        .color(theme.text_muted),
                    )
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm);
                if let Some(handler) = on_toggle {
                    btn = btn.on_click(move |_: &ClickEvent, window, cx| {
                        handler(new_state, window, cx);
                    });
                }
                header_right = header_right.child(btn);
            }

            if let Some(on_close) = self.on_close {
                header_right =
                    header_right.child(ArtifactClose::new("artifact-close").on_click(on_close));
            }

            Some(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap(theme.spacing_sm)
                    .px(theme.spacing_md)
                    .py(theme.spacing_sm)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(title_col)
                    .child(header_right)
                    .into_any_element(),
            )
        } else {
            None
        };

        // Build body: compound takes priority, then convenience.
        let body_element = if let Some(content) = self.artifact_content {
            Some(content.into_any_element())
        } else {
            self.body_content
        };

        let mut container = crate::foundations::materials::card_surface(theme);

        if let Some(header) = header_element {
            container = container.child(header);
        }

        // Hide body while collapsed — matches HIG §Disclosure Controls:
        // collapsing a disclosure hides the contained content but preserves
        // the header/trigger, mirroring Zed's thread_view expanded-tool-call
        // pattern (`expanded_tool_calls: HashSet`).
        if !self.collapsed
            && let Some(body) = body_element
        {
            container = container.child(body);
        }

        container
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        Artifact, ArtifactAction, ArtifactActions, ArtifactClose, ArtifactContent,
        ArtifactDescription, ArtifactHeader, ArtifactTitle,
    };
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;
    use gpui::div;

    // -- ArtifactTitle --------------------------------------------------------

    #[test]
    fn title_new() {
        let title = ArtifactTitle::new("Hello");
        assert_eq!(title.text.as_ref(), "Hello");
    }

    // -- ArtifactDescription --------------------------------------------------

    #[test]
    fn description_new() {
        let desc = ArtifactDescription::new("A description");
        assert_eq!(desc.text.as_ref(), "A description");
    }

    // -- ArtifactAction -------------------------------------------------------

    #[test]
    fn action_defaults() {
        let action = ArtifactAction::new("copy", IconName::Copy);
        assert_eq!(action.icon, IconName::Copy);
        assert!(action.tooltip.is_none());
        assert!(action.on_click.is_none());
    }

    #[test]
    fn action_with_tooltip() {
        let action = ArtifactAction::new("copy", IconName::Copy).tooltip("Copy code");
        assert_eq!(
            action.tooltip.as_ref().map(|s| s.as_ref()),
            Some("Copy code")
        );
    }

    #[test]
    fn action_with_on_click() {
        let action = ArtifactAction::new("copy", IconName::Copy).on_click(|_w, _cx| {});
        assert!(action.on_click.is_some());
    }

    #[test]
    fn action_full_chain() {
        let action = ArtifactAction::new("dl", IconName::Download)
            .tooltip("Download")
            .on_click(|_w, _cx| {});
        assert_eq!(action.icon, IconName::Download);
        assert!(action.tooltip.is_some());
        assert!(action.on_click.is_some());
    }

    // -- ArtifactActions ------------------------------------------------------

    #[test]
    fn actions_empty() {
        let actions = ArtifactActions::new();
        assert!(actions.children.is_empty());
    }

    #[test]
    fn actions_single() {
        let actions = ArtifactActions::new().action(div());
        assert_eq!(actions.children.len(), 1);
    }

    #[test]
    fn actions_multiple() {
        let actions = ArtifactActions::new().action(div()).action(div());
        assert_eq!(actions.children.len(), 2);
    }

    #[test]
    fn actions_bulk() {
        let actions = ArtifactActions::new().actions(vec![div(), div(), div()]);
        assert_eq!(actions.children.len(), 3);
    }

    // -- ArtifactClose --------------------------------------------------------

    #[test]
    fn close_defaults() {
        let close = ArtifactClose::new("close");
        assert!(close.on_click.is_none());
    }

    #[test]
    fn close_with_handler() {
        let close = ArtifactClose::new("close").on_click(|_w, _cx| {});
        assert!(close.on_click.is_some());
    }

    // -- ArtifactHeader -------------------------------------------------------

    #[test]
    fn header_empty() {
        let header = ArtifactHeader::new();
        assert!(header.children.is_empty());
    }

    #[test]
    fn header_child_appends() {
        let header = ArtifactHeader::new().child(div()).child(div());
        assert_eq!(header.children.len(), 2);
    }

    #[test]
    fn header_children_bulk() {
        let header = ArtifactHeader::new().children(vec![div(), div(), div()]);
        assert_eq!(header.children.len(), 3);
    }

    // -- ArtifactContent ------------------------------------------------------

    #[test]
    fn content_empty() {
        let content = ArtifactContent::new();
        assert!(content.children.is_empty());
    }

    #[test]
    fn content_child_appends() {
        let content = ArtifactContent::new().child(div()).child(div());
        assert_eq!(content.children.len(), 2);
    }

    #[test]
    fn content_children_bulk() {
        let content = ArtifactContent::new().children(vec![div(), div(), div()]);
        assert_eq!(content.children.len(), 3);
    }

    // -- Artifact (convenience) -----------------------------------------------

    #[test]
    fn artifact_new_defaults() {
        let a = Artifact::new("Title");
        assert_eq!(a.title.as_ref().map(|s| s.as_ref()), Some("Title"));
        assert!(a.description.is_none());
        assert!(a.actions.is_empty());
        assert!(a.on_close.is_none());
        assert!(a.body_content.is_none());
        assert!(a.artifact_header.is_none());
        assert!(a.artifact_content.is_none());
    }

    #[test]
    fn artifact_with_description() {
        let a = Artifact::new("Title").description("Desc");
        assert_eq!(a.description.as_ref().map(|s| s.as_ref()), Some("Desc"));
    }

    #[test]
    fn artifact_with_actions() {
        let a = Artifact::new("Title").action(div()).action(div());
        assert_eq!(a.actions.len(), 2);
    }

    #[test]
    fn artifact_with_on_close() {
        let a = Artifact::new("Title").on_close(|_w, _cx| {});
        assert!(a.on_close.is_some());
    }

    #[test]
    fn artifact_with_content() {
        let a = Artifact::new("Title").content(div());
        assert!(a.body_content.is_some());
    }

    // -- Artifact (compound) --------------------------------------------------

    #[test]
    fn artifact_from_parts_defaults() {
        let a = Artifact::from_parts();
        assert!(a.title.is_none());
        assert!(a.description.is_none());
        assert!(a.actions.is_empty());
        assert!(a.on_close.is_none());
        assert!(a.body_content.is_none());
        assert!(a.artifact_header.is_none());
        assert!(a.artifact_content.is_none());
    }

    #[test]
    fn artifact_from_parts_with_header() {
        let a = Artifact::from_parts().header(ArtifactHeader::new().child(div()));
        assert!(a.artifact_header.is_some());
        assert_eq!(a.artifact_header.as_ref().unwrap().children.len(), 1);
    }

    #[test]
    fn artifact_from_parts_with_body() {
        let a = Artifact::from_parts().body(ArtifactContent::new().child(div()));
        assert!(a.artifact_content.is_some());
        assert_eq!(a.artifact_content.as_ref().unwrap().children.len(), 1);
    }

    #[test]
    fn artifact_compound_full() {
        let a = Artifact::from_parts()
            .header(ArtifactHeader::new().child(div()).child(div()))
            .body(ArtifactContent::new().child(div()));
        assert!(a.artifact_header.is_some());
        assert!(a.artifact_content.is_some());
        assert_eq!(a.artifact_header.as_ref().unwrap().children.len(), 2);
    }

    // -- API mixing -----------------------------------------------------------

    #[test]
    fn artifact_header_clears_convenience_fields() {
        let a = Artifact::new("Title")
            .description("Desc")
            .action(div())
            .on_close(|_w, _cx| {})
            .header(ArtifactHeader::new().child(div()));
        assert!(a.artifact_header.is_some());
        assert!(a.title.is_none());
        assert!(a.description.is_none());
        assert!(a.actions.is_empty());
        assert!(a.on_close.is_none());
    }

    #[test]
    fn artifact_body_clears_convenience_content() {
        let a = Artifact::from_parts()
            .content(div())
            .body(ArtifactContent::new().child(div()));
        assert!(a.artifact_content.is_some());
        assert!(a.body_content.is_none());
    }

    // -- Collapse (finding #24) -----------------------------------------------

    #[test]
    fn artifact_defaults_not_collapsible() {
        let a = Artifact::new("Title");
        assert!(!a.collapsible);
        assert!(!a.collapsed);
        assert!(a.on_toggle.is_none());
    }

    #[test]
    fn artifact_collapsible_stores_state() {
        let a = Artifact::new("T").collapsible(true);
        assert!(a.collapsible);
        assert!(a.collapsed);
        let b = Artifact::new("T").collapsible(false);
        assert!(b.collapsible);
        assert!(!b.collapsed);
    }

    #[test]
    fn artifact_on_toggle_stored() {
        let a = Artifact::new("T")
            .collapsible(false)
            .on_toggle(|_new, _w, _cx| {});
        assert!(a.on_toggle.is_some());
    }
}
