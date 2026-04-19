//! Separator/divider component.
//!
//! Renders a 1pt hairline aligned with HIG:
//! <https://developer.apple.com/design/human-interface-guidelines/lists-and-tables>
//! and `#boxes`. The default colour is `theme.separator_color()`, which maps
//! to the semantic `NSColor.separatorColor` token (distinct from the label
//! hierarchy colours used for text).

use crate::foundations::theme::{ActiveTheme};
use gpui::prelude::*;
use gpui::{App, Hsla, Pixels, Window, div, px};

/// Orientation for a separator line.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A thin line separator.
#[derive(IntoElement)]
pub struct Separator {
    orientation: SeparatorOrientation,
    color: Option<Hsla>,
    /// Leading-edge inset (in points). Matches `NSTableView`'s default
    /// 16 pt inset for row separators, letting the separator align with
    /// cell content rather than running to the leading edge.
    inset: Pixels,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::default(),
            color: None,
            inset: px(0.0),
        }
    }

    pub fn horizontal() -> Self {
        Self::new()
    }

    pub fn vertical() -> Self {
        Self {
            orientation: SeparatorOrientation::Vertical,
            color: None,
            inset: px(0.0),
        }
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Shift the separator off the leading edge by `inset` points. Matches
    /// the `NSTableView` row-separator convention where separators align
    /// with the leading content, not the container edge.
    pub fn inset(mut self, inset: Pixels) -> Self {
        self.inset = inset;
        self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // HIG: `NSColor.separatorColor` is a dedicated semantic token,
        // distinct from the label-color hierarchy. `theme.separator_color()`
        // wraps `semantic.separator` which maps to that token.
        let color = self.color.unwrap_or_else(|| theme.separator_color());

        let thickness = theme.separator_thickness;
        let inset = self.inset;

        match self.orientation {
            SeparatorOrientation::Horizontal => {
                let mut line = div().w_full().h(thickness).bg(color).flex_shrink_0();
                if f32::from(inset) > 0.0 {
                    line = line.ml(inset);
                }
                line
            }
            SeparatorOrientation::Vertical => {
                let mut line = div().h_full().w(thickness).bg(color).flex_shrink_0();
                if f32::from(inset) > 0.0 {
                    line = line.mt(inset);
                }
                line
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::layout_and_organization::separator::{Separator, SeparatorOrientation};
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn default_orientation_horizontal() {
        let sep = Separator::new();
        assert_eq!(sep.orientation, SeparatorOrientation::Horizontal);
    }

    #[test]
    fn vertical_orientation() {
        let sep = Separator::vertical();
        assert_eq!(sep.orientation, SeparatorOrientation::Vertical);
    }

    #[test]
    fn orientations_distinct() {
        assert_ne!(
            SeparatorOrientation::Horizontal,
            SeparatorOrientation::Vertical
        );
    }

    #[test]
    fn inset_defaults_to_zero() {
        let sep = Separator::new();
        assert_eq!(f32::from(sep.inset), 0.0);
    }

    #[test]
    fn inset_builder_sets_value() {
        let sep = Separator::new().inset(px(16.0));
        assert_eq!(f32::from(sep.inset), 16.0);
    }
}
