//! Share button component (HIG "Activity views").
//!
//! HIG: <https://developer.apple.com/design/human-interface-guidelines/activity-views>
//!
//! The Share button is the canonical trigger for the system share sheet.
//! HIG: "People expect the Share button to trigger an activity view."
//!
//! # Platform notes
//!
//! - **iOS / iPadOS**: the share button should call
//!   `UIActivityViewController`, which GPUI does not yet expose. The
//!   component here is a declarative preview; callers must still wire the
//!   share action to the system API.
//! - **macOS**: AppKit's equivalent is `NSSharingServicePicker`. GPUI
//!   does not expose a picker API yet either. The rendered behaviour is
//!   a [`PulldownButton`] populated with the app-supplied
//!   [`ShareService`] entries.
//!
//! When GPUI grows an AppKit bridge for sharing services, this component
//! will gain an `.use_system_picker(true)` flag and deprecate the in-app
//! fallback menu.

use gpui::prelude::*;
use gpui::{App, ElementId, SharedString, Window, div};

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menus_and_actions::pulldown_button::{
    PulldownButton, PulldownItem, PulldownItemStyle,
};
use crate::foundations::icons::{Icon, IconName};

/// A single share-target entry in the in-app share fallback menu.
pub struct ShareService {
    /// Display label (e.g. "Messages", "Mail", "Copy Link").
    pub label: SharedString,
    /// Leading icon.
    pub icon: Option<IconName>,
    /// Invoked when the user selects the target.
    pub on_invoke: crate::callback_types::OnMutCallback,
}

impl ShareService {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_invoke: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn on_invoke(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_invoke = Some(Box::new(handler));
        self
    }
}

/// Share button — displays the standard `square.and.arrow.up` SF Symbol
/// and, when open, reveals an in-app activity-view fallback populated
/// with the caller's [`ShareService`] entries.
///
/// Stateless `RenderOnce`. The open/closed state is owned by the parent
/// (mirroring [`PulldownButton`]'s contract).
#[derive(IntoElement)]
pub struct ShareButton {
    id: ElementId,
    services: Vec<ShareService>,
    is_open: bool,
    disabled: bool,
    focused: bool,
    on_toggle: crate::callback_types::OnToggle,
    /// When `true`, render as an icon-only toolbar button (no "Share"
    /// label next to the glyph). HIG toolbar share-button pattern.
    icon_only: bool,
}

impl ShareButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            services: Vec::new(),
            is_open: false,
            disabled: false,
            focused: false,
            on_toggle: None,
            icon_only: false,
        }
    }

    pub fn service(mut self, service: ShareService) -> Self {
        self.services.push(service);
        self
    }

    pub fn services(mut self, services: Vec<ShareService>) -> Self {
        self.services = services;
        self
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Render as an icon-only toolbar button (no "Share" text label).
    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = icon_only;
        self
    }
}

impl RenderOnce for ShareButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // If there are no configured services, render a plain button —
        // callers that intend to bridge to NSSharingServicePicker pass an
        // empty `services` list and install their own `on_toggle`.
        if self.services.is_empty() {
            let mut btn = Button::new(self.id)
                .icon(
                    Icon::new(IconName::Share)
                        .size(gpui::px(16.0))
                        .color(cx.global::<crate::foundations::theme::TahoeTheme>().text),
                )
                .variant(ButtonVariant::Ghost)
                .size(if self.icon_only {
                    ButtonSize::Icon
                } else {
                    ButtonSize::Md
                })
                .disabled(self.disabled)
                .focused(self.focused);
            if !self.icon_only {
                btn = btn.label("Share");
            }
            if let Some(handler) = self.on_toggle {
                let is_open = self.is_open;
                btn = btn.on_click(move |_ev, window, cx| handler(!is_open, window, cx));
            }
            return div().child(btn).into_any_element();
        }

        // Convert services into pulldown items.
        let items: Vec<PulldownItem> = self
            .services
            .into_iter()
            .map(|svc| {
                let mut item = PulldownItem::new(svc.label).style(PulldownItemStyle::Default);
                if let Some(icon) = svc.icon {
                    item = item.icon(icon);
                }
                if let Some(cb) = svc.on_invoke {
                    item = item.on_click(move |window, cx| cb(window, cx));
                }
                item
            })
            .collect();

        let mut pb = PulldownButton::new(self.id, if self.icon_only { "" } else { "Share" })
            .icon(Icon::new(IconName::Share).size(gpui::px(16.0)))
            .open(self.is_open)
            .disabled(self.disabled)
            .focused(self.focused);
        for item in items {
            pb = pb.item(item);
        }
        if let Some(handler) = self.on_toggle {
            pb = pb.on_toggle(handler);
        }
        div().child(pb).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{ShareButton, ShareService};
    use core::prelude::v1::test;

    #[test]
    fn share_button_defaults() {
        let btn = ShareButton::new("share");
        assert!(btn.services.is_empty());
        assert!(!btn.is_open);
        assert!(!btn.disabled);
        assert!(!btn.focused);
        assert!(!btn.icon_only);
    }

    #[test]
    fn service_builder_chains() {
        use crate::foundations::icons::IconName;
        let svc = ShareService::new("Messages")
            .icon(IconName::Send)
            .on_invoke(|_, _| {});
        assert_eq!(svc.label.as_ref(), "Messages");
        assert!(svc.icon.is_some());
        assert!(svc.on_invoke.is_some());
    }

    #[test]
    fn services_accumulate() {
        let btn = ShareButton::new("share")
            .service(ShareService::new("Messages"))
            .service(ShareService::new("Mail"));
        assert_eq!(btn.services.len(), 2);
    }

    #[test]
    fn icon_only_builder_sets_flag() {
        let btn = ShareButton::new("share").icon_only(true);
        assert!(btn.icon_only);
    }
}
