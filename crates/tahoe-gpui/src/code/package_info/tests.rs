//! Tests for package info components.

use super::{
    ChangeType, Dependency, PackageInfoChangeType, PackageInfoContent, PackageInfoDependencies,
    PackageInfoDependency, PackageInfoDescription, PackageInfoHeader, PackageInfoName,
    PackageInfoVersion, PackageInfoView,
};
use crate::components::content::badge::BadgeVariant;
use crate::foundations::icons::IconName;
use core::prelude::v1::test;
use gpui::div;

// -- ChangeType -----------------------------------------------------------

#[test]
fn change_type_labels() {
    assert_eq!(ChangeType::Major.label(), "major");
    assert_eq!(ChangeType::Minor.label(), "minor");
    assert_eq!(ChangeType::Patch.label(), "patch");
    assert_eq!(ChangeType::Added.label(), "added");
    assert_eq!(ChangeType::Removed.label(), "removed");
}

#[test]
fn change_type_badge_variants() {
    assert_eq!(ChangeType::Major.badge_variant(), BadgeVariant::Error);
    assert_eq!(ChangeType::Minor.badge_variant(), BadgeVariant::Warning);
    assert_eq!(ChangeType::Patch.badge_variant(), BadgeVariant::Success);
    assert_eq!(ChangeType::Added.badge_variant(), BadgeVariant::Info);
    assert_eq!(ChangeType::Removed.badge_variant(), BadgeVariant::Muted);
}

#[test]
fn change_type_icon_names() {
    assert_eq!(ChangeType::Added.icon_name(), IconName::Plus);
    assert_eq!(ChangeType::Major.icon_name(), IconName::ArrowRight);
    assert_eq!(ChangeType::Minor.icon_name(), IconName::ArrowRight);
    assert_eq!(ChangeType::Patch.icon_name(), IconName::ArrowRight);
    assert_eq!(ChangeType::Removed.icon_name(), IconName::Minus);
}

#[test]
fn change_type_all_distinct() {
    let types = [
        ChangeType::Major,
        ChangeType::Minor,
        ChangeType::Patch,
        ChangeType::Added,
        ChangeType::Removed,
    ];
    for i in 0..types.len() {
        for j in 0..types.len() {
            if i == j {
                assert_eq!(types[i], types[j]);
            } else {
                assert_ne!(types[i], types[j]);
            }
        }
    }
}

// -- Dependency -----------------------------------------------------------

#[test]
fn dependency_builder() {
    let dep = Dependency::new("serde");
    assert_eq!(dep.name.as_ref(), "serde");
    assert!(dep.version.is_none());

    let dep = Dependency::new("gpui").version("0.1.0");
    assert_eq!(dep.name.as_ref(), "gpui");
    assert_eq!(dep.version.as_ref().map(|v| v.as_ref()), Some("0.1.0"));
}

// -- PackageInfoDependency ------------------------------------------------

#[test]
fn package_info_dependency_defaults() {
    let d = PackageInfoDependency::new("serde");
    assert_eq!(d.name.as_ref(), "serde");
    assert!(d.version.is_none());
}

#[test]
fn package_info_dependency_with_version() {
    let d = PackageInfoDependency::new("gpui").version("0.1.0");
    assert_eq!(d.name.as_ref(), "gpui");
    assert_eq!(d.version.as_ref().map(|v| v.as_ref()), Some("0.1.0"));
}

// -- PackageInfoDependencies ----------------------------------------------

#[test]
fn package_info_dependencies_defaults() {
    let d = PackageInfoDependencies::new();
    assert_eq!(d.label.as_ref(), "DEPENDENCIES");
    assert!(d.children.is_empty());
}

#[test]
fn package_info_dependencies_custom_label() {
    let d = PackageInfoDependencies::new().label("PEER DEPS");
    assert_eq!(d.label.as_ref(), "PEER DEPS");
}

// -- PackageInfoContent ---------------------------------------------------

#[test]
fn package_info_content_defaults() {
    let c = PackageInfoContent::new();
    assert!(c.children.is_empty());
}

// -- PackageInfoDescription -----------------------------------------------

#[test]
fn package_info_description_defaults() {
    let d = PackageInfoDescription::new("A cool library");
    assert_eq!(d.text.as_ref(), "A cool library");
}

// -- PackageInfoVersion ---------------------------------------------------

#[test]
fn package_info_version_defaults() {
    let v = PackageInfoVersion::new();
    assert!(v.current_version.is_none());
    assert!(v.new_version.is_none());
    assert!(v.custom.is_none());
}

#[test]
fn package_info_version_with_current_and_new() {
    let v = PackageInfoVersion::new().current("1.0.0").new_ver("2.0.0");
    assert_eq!(
        v.current_version.as_ref().map(|v| v.as_ref()),
        Some("1.0.0")
    );
    assert_eq!(v.new_version.as_ref().map(|v| v.as_ref()), Some("2.0.0"));
}

// -- PackageInfoChangeType ------------------------------------------------

#[test]
fn package_info_change_type_defaults() {
    let ct = PackageInfoChangeType::new(ChangeType::Major);
    assert_eq!(ct.change_type, ChangeType::Major);
    assert!(ct.custom.is_none());
}

// -- PackageInfoName ------------------------------------------------------

#[test]
fn package_info_name_defaults() {
    let n = PackageInfoName::new("react");
    assert_eq!(n.name.as_ref(), "react");
    assert!(n.custom.is_none());
}

// -- PackageInfoHeader ----------------------------------------------------

#[test]
fn package_info_header_defaults() {
    let h = PackageInfoHeader::new();
    assert!(h.children.is_empty());
}

// -- PackageInfoView ------------------------------------------------------

#[test]
fn package_info_convenience_api() {
    let view = PackageInfoView::new("react", "18.2.0")
        .new_version("19.0.0")
        .change_type(ChangeType::Major)
        .description("A library")
        .license("MIT")
        .dependencies(vec![Dependency::new("scheduler").version("0.23.0")]);

    assert_eq!(view.name.as_ref().map(|n| n.as_ref()), Some("react"));
    assert_eq!(
        view.current_version.as_ref().map(|v| v.as_ref()),
        Some("18.2.0")
    );
    assert_eq!(
        view.new_version.as_ref().map(|v| v.as_ref()),
        Some("19.0.0")
    );
    assert_eq!(view.change_type, Some(ChangeType::Major));
    assert_eq!(
        view.description.as_ref().map(|d| d.as_ref()),
        Some("A library")
    );
    assert_eq!(view.license.as_ref().map(|l| l.as_ref()), Some("MIT"));
    assert_eq!(view.dependencies.len(), 1);
    assert!(view.compound_children.is_empty());
}

#[test]
fn package_info_from_parts_defaults() {
    let view = PackageInfoView::from_parts();
    assert!(view.name.is_none());
    assert!(view.compound_children.is_empty());
}

// -- child()/children() tests ---------------------------------------------

#[test]
fn package_info_header_child() {
    let h = PackageInfoHeader::new().child(div()).child(div());
    assert_eq!(h.children.len(), 2);
}

#[test]
fn package_info_dependencies_child() {
    let d = PackageInfoDependencies::new()
        .child(PackageInfoDependency::new("a"))
        .child(PackageInfoDependency::new("b"));
    assert_eq!(d.children.len(), 2);
}

#[test]
fn package_info_content_child() {
    let c = PackageInfoContent::new()
        .child(div())
        .children(vec![div(), div()]);
    assert_eq!(c.children.len(), 3);
}

#[test]
fn package_info_view_compound_child() {
    let v = PackageInfoView::from_parts().child(div()).child(div());
    assert_eq!(v.compound_children.len(), 2);
}

// -- custom() override tests ----------------------------------------------

#[test]
fn package_info_change_type_custom() {
    let ct = PackageInfoChangeType::new(ChangeType::Major).custom(div());
    assert!(ct.custom.is_some());
}

#[test]
fn package_info_name_custom() {
    let n = PackageInfoName::new("react").custom(div());
    assert!(n.custom.is_some());
}

#[test]
fn package_info_version_custom() {
    let v = PackageInfoVersion::new().current("1.0").custom(div());
    assert!(v.custom.is_some());
}
