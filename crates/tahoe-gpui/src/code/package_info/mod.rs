//! Package info display component with compound sub-components.
//!
//! Provides a package information card with version changes, change type badges,
//! descriptions, and dependency listings. Supports both a convenience builder API
//! and compound composition.
//!
//! # Convenience builder
//! ```ignore
//! PackageInfoView::new("react", "18.2.0")
//!     .new_version("19.0.0")
//!     .change_type(ChangeType::Major)
//!     .description("A JavaScript library for building UIs")
//!     .dependencies(vec![
//!         Dependency::new("scheduler").version("0.23.0"),
//!     ])
//! ```
//!
//! # Compound composition
//! ```ignore
//! PackageInfoView::from_parts()
//!     .child(
//!         PackageInfoHeader::new()
//!             .padding(false).gap(true)
//!             .child(PackageInfoName::new("react"))
//!             .child(PackageInfoChangeType::new(ChangeType::Major)),
//!     )
//!     .child(
//!         PackageInfoVersion::new()
//!             .current("18.2.0")
//!             .new_ver("19.0.0"),
//!     )
//!     .child(PackageInfoDescription::new("A JavaScript library"))
//!     .child(
//!         PackageInfoContent::new()
//!             .child(
//!                 PackageInfoDependencies::new()
//!                     .child(PackageInfoDependency::new("scheduler").version("0.23.0")),
//!             ),
//!     )
//! ```

#[cfg(test)]
mod tests;
mod types;

pub use types::{ChangeType, Dependency};

use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::layout_and_organization::FlexHeader;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, FontWeight, SharedString, Window, div, px};

// -- PackageInfoDependency ----------------------------------------------------

/// An individual dependency row displaying name and optional version.
#[derive(IntoElement)]
pub struct PackageInfoDependency {
    name: SharedString,
    version: Option<SharedString>,
}

impl PackageInfoDependency {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }

    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }
}

impl RenderOnce for PackageInfoDependency {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut row = div()
            .flex()
            .items_center()
            .justify_between()
            .text_style(TextStyle::Subheadline, theme);

        row = row.child(
            div()
                .font_family(theme.font_mono.clone())
                .text_color(theme.text_muted)
                .child(self.name),
        );

        if let Some(version) = self.version {
            row = row.child(
                div()
                    .font_family(theme.font_mono.clone())
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text)
                    .child(version),
            );
        }

        row
    }
}

// -- PackageInfoDependencies --------------------------------------------------

/// A labeled dependencies section containing dependency rows.
#[derive(IntoElement)]
pub struct PackageInfoDependencies {
    label: SharedString,
    children: Vec<AnyElement>,
}

impl Default for PackageInfoDependencies {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageInfoDependencies {
    pub fn new() -> Self {
        Self {
            label: "DEPENDENCIES".into(),
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for PackageInfoDependencies {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text_muted)
                    .child(self.label),
            )
            .child(div().flex().flex_col().gap(px(4.0)).children(self.children))
    }
}

// -- PackageInfoContent -------------------------------------------------------

/// A content container with a top border separator.
#[derive(IntoElement)]
pub struct PackageInfoContent {
    children: Vec<AnyElement>,
}

impl Default for PackageInfoContent {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageInfoContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for PackageInfoContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .mt(theme.spacing_md)
            .border_t_1()
            .border_color(theme.border)
            .pt(theme.spacing_md)
            .children(self.children)
    }
}

// -- PackageInfoDescription ---------------------------------------------------

/// A description paragraph.
#[derive(IntoElement)]
pub struct PackageInfoDescription {
    text: SharedString,
}

impl PackageInfoDescription {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for PackageInfoDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .mt(theme.spacing_sm)
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text_muted)
            .child(self.text)
    }
}

// -- PackageInfoVersion -------------------------------------------------------

/// Version transition display (current -> new).
#[derive(IntoElement)]
pub struct PackageInfoVersion {
    current_version: Option<SharedString>,
    new_version: Option<SharedString>,
    custom: Option<AnyElement>,
}

impl Default for PackageInfoVersion {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageInfoVersion {
    pub fn new() -> Self {
        Self {
            current_version: None,
            new_version: None,
            custom: None,
        }
    }

    pub fn current(mut self, version: impl Into<SharedString>) -> Self {
        self.current_version = Some(version.into());
        self
    }

    pub fn new_ver(mut self, version: impl Into<SharedString>) -> Self {
        self.new_version = Some(version.into());
        self
    }

    pub fn custom(mut self, child: impl IntoElement) -> Self {
        self.custom = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for PackageInfoVersion {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let container = div()
            .mt(theme.spacing_sm)
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .font_family(theme.font_mono.clone())
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text_muted);

        if let Some(custom) = self.custom {
            return container.child(custom);
        }

        let mut c = container;

        let has_both = self.current_version.is_some() && self.new_version.is_some();

        if let Some(current) = self.current_version {
            if has_both {
                c = c.child(div().child(current));
            } else {
                c = c.child(div().child(format!("@{current}")));
            }
        }

        if has_both {
            c = c.child(
                Icon::new(IconName::ArrowRight)
                    .size(px(12.0))
                    .color(theme.text_muted),
            );
        }

        if let Some(new) = self.new_version {
            c = c.child(
                div()
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .text_color(theme.text)
                    .child(new),
            );
        }

        c
    }
}

// -- PackageInfoChangeType ----------------------------------------------------

/// A badge displaying the change type with an icon.
#[derive(IntoElement)]
pub struct PackageInfoChangeType {
    change_type: ChangeType,
    custom: Option<AnyElement>,
}

impl PackageInfoChangeType {
    pub fn new(change_type: ChangeType) -> Self {
        Self {
            change_type,
            custom: None,
        }
    }

    pub fn custom(mut self, child: impl IntoElement) -> Self {
        self.custom = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for PackageInfoChangeType {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(custom) = self.custom {
            return div().child(custom);
        }

        let theme = cx.theme();
        let ct = self.change_type;

        let (bg, text_color) = match ct.badge_variant() {
            BadgeVariant::Error => (theme.error, theme.text_on_accent),
            BadgeVariant::Warning => (theme.warning, theme.text_on_accent),
            BadgeVariant::Success => (theme.success, theme.text_on_accent),
            BadgeVariant::Info => (theme.info, theme.text_on_accent),
            BadgeVariant::Muted => (theme.border, theme.text_muted),
            BadgeVariant::Default | BadgeVariant::Notification { .. } | BadgeVariant::Dot => {
                (theme.surface, theme.text)
            }
        };

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(theme.spacing_sm)
            .py(px(2.0))
            .rounded(theme.radius_full)
            .bg(bg)
            .text_color(text_color)
            .text_style(TextStyle::Caption1, theme)
            .child(Icon::new(ct.icon_name()).size(px(10.0)).color(text_color))
            .child(ct.label())
    }
}

// -- PackageInfoName ----------------------------------------------------------

/// Package name with icon.
#[derive(IntoElement)]
pub struct PackageInfoName {
    name: SharedString,
    custom: Option<AnyElement>,
}

impl PackageInfoName {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            custom: None,
        }
    }

    pub fn custom(mut self, child: impl IntoElement) -> Self {
        self.custom = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for PackageInfoName {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let container = div().flex().items_center().gap(theme.spacing_sm);

        if let Some(custom) = self.custom {
            return container
                .child(
                    Icon::new(IconName::Package)
                        .size(theme.icon_size_inline)
                        .color(theme.text_muted),
                )
                .child(custom);
        }

        container
            .child(
                Icon::new(IconName::Package)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .font_family(theme.font_mono.clone())
                    .text_color(theme.text)
                    .child(self.name),
            )
    }
}

// -- PackageInfoHeader --------------------------------------------------------

/// Header row for the package info card.
///
/// Type alias for [`FlexHeader`] configured with gap and no padding at the call site.
pub type PackageInfoHeader = FlexHeader;

// -- PackageInfoView ----------------------------------------------------------

/// A package information display card.
///
/// Supports both a convenience builder API and compound composition.
#[derive(IntoElement)]
pub struct PackageInfoView {
    // Convenience builder fields.
    name: Option<SharedString>,
    current_version: Option<SharedString>,
    new_version: Option<SharedString>,
    description: Option<SharedString>,
    license: Option<SharedString>,
    change_type: Option<ChangeType>,
    dependencies: Vec<Dependency>,
    // Compound composition fields.
    compound_children: Vec<AnyElement>,
}

impl PackageInfoView {
    /// Create a package info card using the convenience builder API.
    pub fn new(name: impl Into<SharedString>, current_version: impl Into<SharedString>) -> Self {
        Self {
            name: Some(name.into()),
            current_version: Some(current_version.into()),
            new_version: None,
            description: None,
            license: None,
            change_type: None,
            dependencies: Vec::new(),
            compound_children: Vec::new(),
        }
    }

    /// Create a package info card for compound composition.
    pub fn from_parts() -> Self {
        Self {
            name: None,
            current_version: None,
            new_version: None,
            description: None,
            license: None,
            change_type: None,
            dependencies: Vec::new(),
            compound_children: Vec::new(),
        }
    }

    pub fn new_version(mut self, version: impl Into<SharedString>) -> Self {
        self.new_version = Some(version.into());
        self
    }

    pub fn change_type(mut self, ct: ChangeType) -> Self {
        self.change_type = Some(ct);
        self
    }

    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn license(mut self, license: impl Into<SharedString>) -> Self {
        self.license = Some(license.into());
        self
    }

    pub fn dependencies(mut self, deps: Vec<Dependency>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Add a child element for compound composition.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.compound_children.push(child.into_any_element());
        self
    }

    /// Add multiple children for compound composition.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.compound_children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for PackageInfoView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let card = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border);

        // Compound path: render children directly.
        if !self.compound_children.is_empty() {
            assert!(
                self.name.is_none()
                    && self.current_version.is_none()
                    && self.description.is_none()
                    && self.license.is_none()
                    && self.change_type.is_none()
                    && self.dependencies.is_empty(),
                "PackageInfoView: do not mix convenience fields with compound children",
            );
            return card.children(self.compound_children);
        }

        // Convenience path: auto-build sub-components.
        let mut card = card;

        // Header: name + change type badge + license badge.
        let Some(name) = self.name else {
            // Degenerate state: no compound children and no name. Render empty card.
            return card;
        };
        let mut header = PackageInfoHeader::new()
            .padding(false)
            .gap(true)
            .child(PackageInfoName::new(name));

        // Right side of header: change type + license badges.
        let has_right_badges = self.change_type.is_some() || self.license.is_some();
        if has_right_badges {
            let mut right = div().flex().items_center().gap(theme.spacing_xs);
            if let Some(ct) = self.change_type {
                right = right.child(PackageInfoChangeType::new(ct));
            }
            if let Some(license) = self.license {
                right = right.child(Badge::new(license));
            }
            header = header.child(right);
        }

        card = card.child(header);

        // Version display.
        let has_version = self.current_version.is_some() || self.new_version.is_some();
        if has_version {
            let mut version = PackageInfoVersion::new();
            if let Some(cv) = self.current_version {
                version = version.current(cv);
            }
            if let Some(nv) = self.new_version {
                version = version.new_ver(nv);
            }
            card = card.child(version);
        }

        // Description.
        if let Some(desc) = self.description {
            card = card.child(PackageInfoDescription::new(desc));
        }

        // Dependencies in a content section.
        if !self.dependencies.is_empty() {
            let mut deps = PackageInfoDependencies::new();
            for dep in self.dependencies {
                let mut d = PackageInfoDependency::new(dep.name);
                if let Some(v) = dep.version {
                    d = d.version(v);
                }
                deps = deps.child(d);
            }
            card = card.child(PackageInfoContent::new().child(deps));
        }

        card
    }
}
