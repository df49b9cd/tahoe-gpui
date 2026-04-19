//! Data types for package info display.

use crate::components::content::badge::BadgeVariant;
use crate::foundations::icons::IconName;
use gpui::SharedString;

/// Type of version change for a package.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChangeType {
    /// Breaking changes (red).
    Major,
    /// New features (yellow).
    Minor,
    /// Bug fixes (green).
    Patch,
    /// New dependency (blue).
    Added,
    /// Removed dependency (gray).
    Removed,
}

impl ChangeType {
    /// Human-readable label for this change type.
    pub fn label(&self) -> &'static str {
        match self {
            ChangeType::Major => "major",
            ChangeType::Minor => "minor",
            ChangeType::Patch => "patch",
            ChangeType::Added => "added",
            ChangeType::Removed => "removed",
        }
    }

    /// Corresponding badge variant for this change type.
    pub fn badge_variant(&self) -> BadgeVariant {
        match self {
            ChangeType::Major => BadgeVariant::Error,
            ChangeType::Minor => BadgeVariant::Warning,
            ChangeType::Patch => BadgeVariant::Success,
            ChangeType::Added => BadgeVariant::Info,
            ChangeType::Removed => BadgeVariant::Muted,
        }
    }

    /// Icon associated with this change type.
    pub fn icon_name(&self) -> IconName {
        match self {
            ChangeType::Added => IconName::Plus,
            ChangeType::Major | ChangeType::Minor | ChangeType::Patch => IconName::ArrowRight,
            ChangeType::Removed => IconName::Minus,
        }
    }
}

/// A single package dependency with optional version.
#[derive(Clone)]
pub struct Dependency {
    pub name: SharedString,
    pub version: Option<SharedString>,
}

impl Dependency {
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
