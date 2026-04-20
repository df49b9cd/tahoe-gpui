//! Typography tokens aligned with HIG.
//!
//! Provides text styles, font design variants, tracking utilities,
//! and the `TextStyledExt` trait for applying HIG type scale to GPUI elements.

use gpui::{FontWeight, Pixels, Styled, px};

use super::theme::TahoeTheme;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TextStyle
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HIG text style — bundles point size, weight, and leading.
///
/// These correspond to the 11 built-in macOS text styles from the SF Pro type scale.
/// Values are sourced from the macOS Human Interface Guidelines.
/// macOS does not support Dynamic Type; sizes are fixed.
///
/// # Usage
///
/// ```ignore
/// let attrs = TextStyle::Body.attrs();
/// el = el.text_size(attrs.size).line_height(attrs.leading);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    LargeTitle,
    Title1,
    Title2,
    Title3,
    Headline,
    Body,
    Callout,
    Subheadline,
    Footnote,
    Caption1,
    Caption2,
}

/// iOS/iPadOS Dynamic Type size level.
///
/// These correspond to the system text size slider in Settings > Display & Brightness.
/// Each level scales all 11 text styles proportionally.
/// The `Large` variant is the iOS default.
///
/// # Reference
/// Values from HIG Typography > iOS, iPadOS Dynamic Type sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DynamicTypeSize {
    /// Smallest standard text size.
    XSmall,
    Small,
    Medium,
    /// Default iOS text size.
    #[default]
    Large,
    XLarge,
    XXLarge,
    XXXLarge,
    /// Accessibility text sizes (require Larger Accessibility Text Sizes enabled).
    AX1,
    AX2,
    AX3,
    AX4,
    AX5,
}

impl DynamicTypeSize {
    /// Returns true if this is an accessibility size (AX1-AX5).
    pub fn is_accessibility(self) -> bool {
        matches!(
            self,
            Self::AX1 | Self::AX2 | Self::AX3 | Self::AX4 | Self::AX5
        )
    }
}

/// Resolved text style attributes for a specific `TextStyle`.
#[derive(Debug, Clone, Copy)]
pub struct TextStyleAttrs {
    pub size: Pixels,
    pub weight: FontWeight,
    /// Absolute line height in points (macOS HIG value).
    pub leading: Pixels,
    /// Letter-spacing in points (macOS SF Pro tracking value).
    /// Negative = tighter, positive = looser.
    ///
    /// # Pending GPUI support
    ///
    /// GPUI currently ignores tracking when laying out text, so applying
    /// an attrs-derived style via [`TextStyledExt`] has no visible
    /// effect on tracking today. The field is retained so the canonical
    /// HIG tracking values stay documented against each style and any
    /// consumer compositing their own text runs (e.g. via a native
    /// CoreText backend) can read them. When GPUI lands letter-spacing,
    /// the existing values render with zero API churn.
    pub tracking: f32,
}

impl TextStyle {
    /// Returns the macOS HIG-defined attributes for this text style.
    /// Default platform size is 13pt (Body); minimum is 10pt.
    pub fn attrs(self) -> TextStyleAttrs {
        match self {
            // macOS Tahoe aligned `LargeTitle` with iOS's bold treatment:
            // the default weight is Bold, not Regular. Sheet and Modal
            // titles now render with the correct emphasis out of the box.
            Self::LargeTitle => TextStyleAttrs {
                size: px(26.0),
                weight: FontWeight::BOLD,
                leading: px(32.0),
                tracking: 0.22,
            },
            Self::Title1 => TextStyleAttrs {
                size: px(22.0),
                weight: FontWeight::NORMAL,
                leading: px(26.0),
                tracking: -0.26,
            },
            Self::Title2 => TextStyleAttrs {
                size: px(17.0),
                weight: FontWeight::NORMAL,
                leading: px(22.0),
                tracking: -0.43,
            },
            Self::Title3 => TextStyleAttrs {
                size: px(15.0),
                weight: FontWeight::NORMAL,
                leading: px(20.0),
                tracking: -0.23,
            },
            Self::Headline => TextStyleAttrs {
                size: px(13.0),
                weight: FontWeight::BOLD,
                leading: px(16.0),
                tracking: -0.08,
            },
            Self::Body => TextStyleAttrs {
                size: px(13.0),
                weight: FontWeight::NORMAL,
                leading: px(16.0),
                tracking: -0.08,
            },
            Self::Callout => TextStyleAttrs {
                size: px(12.0),
                weight: FontWeight::NORMAL,
                leading: px(15.0),
                tracking: 0.0,
            },
            Self::Subheadline => TextStyleAttrs {
                size: px(11.0),
                weight: FontWeight::NORMAL,
                leading: px(14.0),
                tracking: 0.06,
            },
            // On macOS `Footnote`, `Caption1`, and `Caption2` all share the same
            // 10pt size, 13pt leading, and 0.12 tracking. The only metric that
            // differs is `Caption2`'s base weight (Medium vs Regular for the
            // others); per the macOS HIG table this is not a leading difference.
            // On iOS all three styles are Regular and their sizes/leadings
            // diverge — consumers targeting iOS should call `ios_attrs` instead.
            Self::Footnote => TextStyleAttrs {
                size: px(10.0),
                weight: FontWeight::NORMAL,
                leading: px(13.0),
                tracking: 0.12,
            },
            Self::Caption1 => TextStyleAttrs {
                size: px(10.0),
                weight: FontWeight::NORMAL,
                leading: px(13.0),
                tracking: 0.12,
            },
            Self::Caption2 => TextStyleAttrs {
                size: px(10.0),
                weight: FontWeight::MEDIUM,
                leading: px(13.0),
                tracking: 0.12,
            },
        }
    }

    /// Returns attributes with `BoldText` accessibility adjustment applied.
    /// Bumps each weight up one step: Normal -> Medium, Medium -> Semibold, etc.
    pub fn attrs_bold(self) -> TextStyleAttrs {
        let mut attrs = self.attrs();
        attrs.weight = bold_step(attrs.weight);
        attrs
    }

    /// Returns attributes with the macOS HIG "Emphasized" weight applied.
    /// Each style has a specific emphasized weight defined by the HIG,
    /// which differs from the uniform one-step bump of `bold_step()`.
    ///
    /// Values are sourced from the macOS built-in text styles table in the
    /// HIG ("Emphasized weight" column).
    pub fn emphasized(self) -> TextStyleAttrs {
        let mut attrs = self.attrs();
        attrs.weight = match self {
            Self::LargeTitle | Self::Title1 | Self::Title2 => FontWeight::BOLD,
            Self::Title3
            | Self::Body
            | Self::Callout
            | Self::Subheadline
            | Self::Footnote
            | Self::Caption2 => FontWeight::SEMIBOLD,
            // Apple "Heavy" ≈ GPUI `FontWeight::BLACK` (CSS weight 900). The
            // HIG macOS table lists Headline's emphasized weight as Heavy.
            Self::Headline => FontWeight::BLACK,
            Self::Caption1 => FontWeight::MEDIUM,
        };
        attrs
    }

    /// Returns the iOS/iPadOS HIG-defined attributes for this text style
    /// at the given Dynamic Type size level.
    ///
    /// Values are from the HIG Typography > iOS, iPadOS Dynamic Type sizes tables.
    /// The `Large` size is the iOS default.
    pub fn ios_attrs(self, size_level: DynamicTypeSize) -> TextStyleAttrs {
        use DynamicTypeSize::*;
        let (size, weight, leading) = match (self, size_level) {
            // -- Large (default) ------------------------------------------
            (Self::LargeTitle, Large) => (34.0, FontWeight::NORMAL, 41.0),
            (Self::Title1, Large) => (28.0, FontWeight::NORMAL, 34.0),
            (Self::Title2, Large) => (22.0, FontWeight::NORMAL, 28.0),
            (Self::Title3, Large) => (20.0, FontWeight::NORMAL, 25.0),
            (Self::Headline, Large) => (17.0, FontWeight::SEMIBOLD, 22.0),
            (Self::Body, Large) => (17.0, FontWeight::NORMAL, 22.0),
            (Self::Callout, Large) => (16.0, FontWeight::NORMAL, 21.0),
            (Self::Subheadline, Large) => (15.0, FontWeight::NORMAL, 20.0),
            (Self::Footnote, Large) => (13.0, FontWeight::NORMAL, 18.0),
            (Self::Caption1, Large) => (12.0, FontWeight::NORMAL, 16.0),
            (Self::Caption2, Large) => (11.0, FontWeight::NORMAL, 13.0),

            // -- xSmall ---------------------------------------------------
            (Self::LargeTitle, XSmall) => (31.0, FontWeight::NORMAL, 38.0),
            (Self::Title1, XSmall) => (25.0, FontWeight::NORMAL, 31.0),
            (Self::Title2, XSmall) => (19.0, FontWeight::NORMAL, 24.0),
            (Self::Title3, XSmall) => (17.0, FontWeight::NORMAL, 22.0),
            (Self::Headline, XSmall) => (14.0, FontWeight::SEMIBOLD, 19.0),
            (Self::Body, XSmall) => (14.0, FontWeight::NORMAL, 19.0),
            (Self::Callout, XSmall) => (13.0, FontWeight::NORMAL, 18.0),
            (Self::Subheadline, XSmall) => (12.0, FontWeight::NORMAL, 16.0),
            (Self::Footnote, XSmall) => (12.0, FontWeight::NORMAL, 16.0),
            (Self::Caption1, XSmall) => (11.0, FontWeight::NORMAL, 13.0),
            (Self::Caption2, XSmall) => (11.0, FontWeight::NORMAL, 13.0),

            // -- Small ----------------------------------------------------
            (Self::LargeTitle, Small) => (32.0, FontWeight::NORMAL, 39.0),
            (Self::Title1, Small) => (26.0, FontWeight::NORMAL, 32.0),
            (Self::Title2, Small) => (20.0, FontWeight::NORMAL, 25.0),
            (Self::Title3, Small) => (18.0, FontWeight::NORMAL, 23.0),
            (Self::Headline, Small) => (15.0, FontWeight::SEMIBOLD, 20.0),
            (Self::Body, Small) => (15.0, FontWeight::NORMAL, 20.0),
            (Self::Callout, Small) => (14.0, FontWeight::NORMAL, 19.0),
            (Self::Subheadline, Small) => (13.0, FontWeight::NORMAL, 18.0),
            (Self::Footnote, Small) => (12.0, FontWeight::NORMAL, 16.0),
            (Self::Caption1, Small) => (11.0, FontWeight::NORMAL, 13.0),
            (Self::Caption2, Small) => (11.0, FontWeight::NORMAL, 13.0),

            // -- Medium ---------------------------------------------------
            (Self::LargeTitle, Medium) => (33.0, FontWeight::NORMAL, 40.0),
            (Self::Title1, Medium) => (27.0, FontWeight::NORMAL, 33.0),
            (Self::Title2, Medium) => (21.0, FontWeight::NORMAL, 26.0),
            (Self::Title3, Medium) => (19.0, FontWeight::NORMAL, 24.0),
            (Self::Headline, Medium) => (16.0, FontWeight::SEMIBOLD, 21.0),
            (Self::Body, Medium) => (16.0, FontWeight::NORMAL, 21.0),
            (Self::Callout, Medium) => (15.0, FontWeight::NORMAL, 20.0),
            (Self::Subheadline, Medium) => (14.0, FontWeight::NORMAL, 19.0),
            (Self::Footnote, Medium) => (12.0, FontWeight::NORMAL, 16.0),
            (Self::Caption1, Medium) => (11.0, FontWeight::NORMAL, 13.0),
            (Self::Caption2, Medium) => (11.0, FontWeight::NORMAL, 13.0),

            // -- xLarge ---------------------------------------------------
            (Self::LargeTitle, XLarge) => (36.0, FontWeight::NORMAL, 43.0),
            (Self::Title1, XLarge) => (30.0, FontWeight::NORMAL, 37.0),
            (Self::Title2, XLarge) => (24.0, FontWeight::NORMAL, 30.0),
            (Self::Title3, XLarge) => (22.0, FontWeight::NORMAL, 28.0),
            (Self::Headline, XLarge) => (19.0, FontWeight::SEMIBOLD, 24.0),
            (Self::Body, XLarge) => (19.0, FontWeight::NORMAL, 24.0),
            (Self::Callout, XLarge) => (18.0, FontWeight::NORMAL, 23.0),
            (Self::Subheadline, XLarge) => (17.0, FontWeight::NORMAL, 22.0),
            (Self::Footnote, XLarge) => (15.0, FontWeight::NORMAL, 20.0),
            (Self::Caption1, XLarge) => (14.0, FontWeight::NORMAL, 19.0),
            (Self::Caption2, XLarge) => (13.0, FontWeight::NORMAL, 18.0),

            // -- xxLarge --------------------------------------------------
            (Self::LargeTitle, XXLarge) => (38.0, FontWeight::NORMAL, 46.0),
            (Self::Title1, XXLarge) => (32.0, FontWeight::NORMAL, 39.0),
            (Self::Title2, XXLarge) => (26.0, FontWeight::NORMAL, 32.0),
            (Self::Title3, XXLarge) => (24.0, FontWeight::NORMAL, 30.0),
            (Self::Headline, XXLarge) => (21.0, FontWeight::SEMIBOLD, 26.0),
            (Self::Body, XXLarge) => (21.0, FontWeight::NORMAL, 26.0),
            (Self::Callout, XXLarge) => (20.0, FontWeight::NORMAL, 25.0),
            (Self::Subheadline, XXLarge) => (19.0, FontWeight::NORMAL, 24.0),
            (Self::Footnote, XXLarge) => (17.0, FontWeight::NORMAL, 22.0),
            (Self::Caption1, XXLarge) => (16.0, FontWeight::NORMAL, 21.0),
            (Self::Caption2, XXLarge) => (15.0, FontWeight::NORMAL, 20.0),

            // -- xxxLarge -------------------------------------------------
            (Self::LargeTitle, XXXLarge) => (40.0, FontWeight::NORMAL, 48.0),
            (Self::Title1, XXXLarge) => (34.0, FontWeight::NORMAL, 41.0),
            (Self::Title2, XXXLarge) => (28.0, FontWeight::NORMAL, 34.0),
            (Self::Title3, XXXLarge) => (26.0, FontWeight::NORMAL, 32.0),
            (Self::Headline, XXXLarge) => (23.0, FontWeight::SEMIBOLD, 29.0),
            (Self::Body, XXXLarge) => (23.0, FontWeight::NORMAL, 29.0),
            (Self::Callout, XXXLarge) => (22.0, FontWeight::NORMAL, 28.0),
            (Self::Subheadline, XXXLarge) => (21.0, FontWeight::NORMAL, 28.0),
            (Self::Footnote, XXXLarge) => (19.0, FontWeight::NORMAL, 24.0),
            (Self::Caption1, XXXLarge) => (18.0, FontWeight::NORMAL, 23.0),
            (Self::Caption2, XXXLarge) => (17.0, FontWeight::NORMAL, 22.0),

            // -- AX1 ------------------------------------------------------
            (Self::LargeTitle, AX1) => (44.0, FontWeight::NORMAL, 52.0),
            (Self::Title1, AX1) => (38.0, FontWeight::NORMAL, 46.0),
            (Self::Title2, AX1) => (34.0, FontWeight::NORMAL, 41.0),
            (Self::Title3, AX1) => (31.0, FontWeight::NORMAL, 38.0),
            (Self::Headline, AX1) => (28.0, FontWeight::SEMIBOLD, 34.0),
            (Self::Body, AX1) => (28.0, FontWeight::NORMAL, 34.0),
            (Self::Callout, AX1) => (26.0, FontWeight::NORMAL, 32.0),
            (Self::Subheadline, AX1) => (25.0, FontWeight::NORMAL, 31.0),
            (Self::Footnote, AX1) => (23.0, FontWeight::NORMAL, 29.0),
            (Self::Caption1, AX1) => (22.0, FontWeight::NORMAL, 28.0),
            (Self::Caption2, AX1) => (20.0, FontWeight::NORMAL, 25.0),

            // -- AX2 ------------------------------------------------------
            (Self::LargeTitle, AX2) => (48.0, FontWeight::NORMAL, 57.0),
            (Self::Title1, AX2) => (43.0, FontWeight::NORMAL, 51.0),
            (Self::Title2, AX2) => (39.0, FontWeight::NORMAL, 47.0),
            (Self::Title3, AX2) => (37.0, FontWeight::NORMAL, 44.0),
            (Self::Headline, AX2) => (33.0, FontWeight::SEMIBOLD, 40.0),
            (Self::Body, AX2) => (33.0, FontWeight::NORMAL, 40.0),
            (Self::Callout, AX2) => (32.0, FontWeight::NORMAL, 39.0),
            (Self::Subheadline, AX2) => (30.0, FontWeight::NORMAL, 37.0),
            (Self::Footnote, AX2) => (27.0, FontWeight::NORMAL, 33.0),
            (Self::Caption1, AX2) => (26.0, FontWeight::NORMAL, 32.0),
            (Self::Caption2, AX2) => (24.0, FontWeight::NORMAL, 30.0),

            // -- AX3 ------------------------------------------------------
            (Self::LargeTitle, AX3) => (52.0, FontWeight::NORMAL, 61.0),
            (Self::Title1, AX3) => (48.0, FontWeight::NORMAL, 57.0),
            (Self::Title2, AX3) => (44.0, FontWeight::NORMAL, 52.0),
            (Self::Title3, AX3) => (43.0, FontWeight::NORMAL, 51.0),
            (Self::Headline, AX3) => (40.0, FontWeight::SEMIBOLD, 48.0),
            (Self::Body, AX3) => (40.0, FontWeight::NORMAL, 48.0),
            (Self::Callout, AX3) => (38.0, FontWeight::NORMAL, 46.0),
            (Self::Subheadline, AX3) => (36.0, FontWeight::NORMAL, 43.0),
            (Self::Footnote, AX3) => (33.0, FontWeight::NORMAL, 40.0),
            (Self::Caption1, AX3) => (32.0, FontWeight::NORMAL, 39.0),
            (Self::Caption2, AX3) => (29.0, FontWeight::NORMAL, 35.0),

            // -- AX4 ------------------------------------------------------
            (Self::LargeTitle, AX4) => (56.0, FontWeight::NORMAL, 66.0),
            (Self::Title1, AX4) => (53.0, FontWeight::NORMAL, 62.0),
            (Self::Title2, AX4) => (50.0, FontWeight::NORMAL, 59.0),
            (Self::Title3, AX4) => (49.0, FontWeight::NORMAL, 58.0),
            (Self::Headline, AX4) => (47.0, FontWeight::SEMIBOLD, 56.0),
            (Self::Body, AX4) => (47.0, FontWeight::NORMAL, 56.0),
            (Self::Callout, AX4) => (44.0, FontWeight::NORMAL, 52.0),
            (Self::Subheadline, AX4) => (42.0, FontWeight::NORMAL, 50.0),
            (Self::Footnote, AX4) => (38.0, FontWeight::NORMAL, 46.0),
            (Self::Caption1, AX4) => (37.0, FontWeight::NORMAL, 44.0),
            (Self::Caption2, AX4) => (34.0, FontWeight::NORMAL, 41.0),

            // -- AX5 ------------------------------------------------------
            (Self::LargeTitle, AX5) => (60.0, FontWeight::NORMAL, 70.0),
            (Self::Title1, AX5) => (58.0, FontWeight::NORMAL, 68.0),
            (Self::Title2, AX5) => (56.0, FontWeight::NORMAL, 66.0),
            (Self::Title3, AX5) => (55.0, FontWeight::NORMAL, 65.0),
            (Self::Headline, AX5) => (53.0, FontWeight::SEMIBOLD, 62.0),
            (Self::Body, AX5) => (53.0, FontWeight::NORMAL, 62.0),
            (Self::Callout, AX5) => (51.0, FontWeight::NORMAL, 60.0),
            (Self::Subheadline, AX5) => (49.0, FontWeight::NORMAL, 58.0),
            (Self::Footnote, AX5) => (44.0, FontWeight::NORMAL, 52.0),
            (Self::Caption1, AX5) => (43.0, FontWeight::NORMAL, 51.0),
            (Self::Caption2, AX5) => (40.0, FontWeight::NORMAL, 48.0),
        };
        TextStyleAttrs {
            size: px(size),
            weight,
            leading: px(leading),
            // The HIG's SF Pro tracking table (the source of `macos_tracking`)
            // is identical to the per-platform macOS tracking table — so the
            // same values apply on iOS and iPadOS when SF Pro is in use.
            // GPUI does not currently render tracking.
            tracking: crate::foundations::typography::macos_tracking(size),
        }
    }

    /// Returns the iOS/iPadOS HIG "Emphasized" attributes for this text style
    /// at the given Dynamic Type size level.
    ///
    /// Values are from the "Emphasized weight" column of the HIG iOS
    /// Dynamic Type tables. On iOS the emphasized weight is uniform across
    /// all size classes (xSmall through AX5): `LargeTitle`, `Title1`, and
    /// `Title2` emphasize to Bold; every other style emphasizes to Semibold.
    /// The emphasized size and leading are identical to `ios_attrs`.
    pub fn ios_attrs_emphasized(self, size_level: DynamicTypeSize) -> TextStyleAttrs {
        let mut attrs = self.ios_attrs(size_level);
        attrs.weight = match self {
            Self::LargeTitle | Self::Title1 | Self::Title2 => FontWeight::BOLD,
            _ => FontWeight::SEMIBOLD,
        };
        attrs
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TextStyleAttrs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

impl TextStyleAttrs {
    /// Returns a copy with the leading adjusted per the given style.
    pub fn with_leading(mut self, style: LeadingStyle) -> Self {
        match style {
            LeadingStyle::Tight => self.leading = px((f32::from(self.leading) - 2.0).max(0.0)),
            LeadingStyle::Standard => {}
            LeadingStyle::Loose => self.leading = px(f32::from(self.leading) + 2.0),
        }
        self
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LeadingStyle
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Leading style per HIG symbolic traits.
///
/// The HIG says: "You can also use symbolic traits to adjust leading if you need
/// to improve readability or conserve space." Loose leading suits wide columns
/// and long passages; tight leading suits height-constrained areas like list rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LeadingStyle {
    /// Tighter line spacing for constrained areas (list rows, compact UI).
    /// Reduces leading by ~2pt.
    Tight,
    /// Default HIG leading for the text style.
    #[default]
    Standard,
    /// Looser line spacing for wide columns and long passages.
    /// Increases leading by ~2pt.
    Loose,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FontDesign
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HIG font design variants.
///
/// Corresponds to `Font.Design` in SwiftUI. Use `.Default` for the system font
/// (SF Pro), `.Serif` for New York, `.Rounded` for SF Pro Rounded,
/// and `.Monospaced` for SF Mono.
///
/// Apply to an element via [`TextStyledExt::text_style_with_design`] or
/// [`TextStyledExt::text_style_emphasized_with_design`]. The plain
/// [`TextStyledExt::text_style`] / [`TextStyledExt::text_style_emphasized`]
/// methods apply [`FontDesign::Default`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontDesign {
    /// SF Pro — the default system sans-serif.
    #[default]
    Default,
    /// New York — the system serif typeface.
    Serif,
    /// SF Pro Rounded — rounded variant for soft UI elements.
    Rounded,
    /// SF Mono — the system monospaced typeface.
    Monospaced,
}

impl FontDesign {
    /// Returns the GPUI font family name for this design variant.
    pub fn font_family(self) -> &'static str {
        match self {
            Self::Default => ".AppleSystemUIFont",
            Self::Serif => "New York",
            Self::Rounded => ".AppleSystemUIFontRounded",
            Self::Monospaced => "SF Mono",
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TextStyledExt
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Extension trait for applying `TextStyle` attributes to any GPUI `Styled` element.
///
/// This makes it easy for components to adopt the HIG type scale:
/// ```ignore
/// use crate::foundations::theme::{FontDesign, TextStyledExt};
/// div().text_style(TextStyle::Body, theme); // size, weight, line_height
/// div().text_style_with_design(TextStyle::Body, FontDesign::Monospaced, theme); // + SF Mono
/// ```
///
/// The `theme` parameter is required so that `effective_weight()` is applied,
/// ensuring `AccessibilityMode::BOLD_TEXT` is respected. The text size and
/// line height are also multiplied by `theme.font_scale_factor`, which the
/// host can drive from macOS System Settings → Accessibility → Display →
/// Text Size so user preferences flow through the type scale.
///
/// [`text_style`](Self::text_style) and [`text_style_emphasized`](Self::text_style_emphasized)
/// leave the element's `font_family` untouched so the parent's cascade wins. Use the
/// `_with_design` variants to set it explicitly.
pub trait TextStyledExt: Styled {
    /// Applies the text style's size, weight, and line height. Leaves `font_family`
    /// alone so the caller's cascade (parent element or an explicit chained
    /// `.font_family(...)`) wins.
    /// Weight is routed through `theme.effective_weight()` for BoldText accessibility.
    /// Size and leading are multiplied by `theme.font_scale_factor`.
    fn text_style(self, style: TextStyle, theme: &TahoeTheme) -> Self {
        apply_text_style_attrs(self, style.attrs(), theme)
    }

    /// Applies the text style with the emphasized (HIG) weight. Leaves `font_family`
    /// alone — see [`text_style`](Self::text_style) for cascade semantics.
    /// Weight is routed through `theme.effective_weight()` for BoldText accessibility.
    /// Size and leading are multiplied by `theme.font_scale_factor`.
    fn text_style_emphasized(self, style: TextStyle, theme: &TahoeTheme) -> Self {
        apply_text_style_attrs(self, style.emphasized(), theme)
    }

    /// Like [`text_style`](Self::text_style) but also sets the font family from
    /// the given [`FontDesign`] (e.g. `FontDesign::Monospaced` for SF Mono).
    fn text_style_with_design(
        self,
        style: TextStyle,
        design: FontDesign,
        theme: &TahoeTheme,
    ) -> Self {
        apply_text_style_attrs_with_design(self, style.attrs(), design, theme)
    }

    /// Like [`text_style_emphasized`](Self::text_style_emphasized) but also sets
    /// the font family from the given [`FontDesign`].
    fn text_style_emphasized_with_design(
        self,
        style: TextStyle,
        design: FontDesign,
        theme: &TahoeTheme,
    ) -> Self {
        apply_text_style_attrs_with_design(self, style.emphasized(), design, theme)
    }
}

fn apply_text_style_attrs<E: Styled>(el: E, attrs: TextStyleAttrs, theme: &TahoeTheme) -> E {
    let scale = theme.font_scale_factor.max(0.0);
    el.text_size(px(f32::from(attrs.size) * scale))
        .font_weight(theme.effective_weight(attrs.weight))
        .line_height(px(f32::from(attrs.leading) * scale))
}

fn apply_text_style_attrs_with_design<E: Styled>(
    el: E,
    attrs: TextStyleAttrs,
    design: FontDesign,
    theme: &TahoeTheme,
) -> E {
    apply_text_style_attrs(el, attrs, theme).font_family(design.font_family())
}

impl<E: Styled> TextStyledExt for E {}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Utility functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Returns the SF Pro tracking value (in points) for a given font size.
/// Based on the HIG tracking table for SF Pro, which is numerically
/// identical to the macOS per-platform tracking table and applies on iOS
/// and iPadOS where SF Pro is the system typeface.
/// NOTE: GPUI does not currently support letter-spacing; this function is
/// provided for documentation and potential future use.
pub fn macos_tracking(size_pt: f32) -> f32 {
    if !size_pt.is_finite() {
        return 0.0;
    }
    let size_pt = size_pt.round().clamp(0.0, u32::MAX as f32) as u32;
    match size_pt {
        0..=5 => 0.0,
        6 => 0.24,
        7 => 0.23,
        8 => 0.21,
        9 => 0.17,
        10 => 0.12,
        11 => 0.06,
        12 => 0.0,
        13 => -0.08,
        14 => -0.15,
        15 => -0.23,
        16 => -0.31,
        17 => -0.43,
        18 => -0.44,
        19 => -0.45,
        20 => -0.45,
        21 => -0.36,
        22 => -0.26,
        23 => -0.10,
        24 => 0.07,
        25 => 0.15,
        26 => 0.22,
        27 => 0.29,
        28 => 0.38,
        29 => 0.40,
        30 => 0.40,
        31 => 0.39,
        32 => 0.41,
        33 => 0.40,
        34 => 0.40,
        35 => 0.38,
        36 => 0.37,
        37 => 0.36,
        38 => 0.37,
        39 => 0.38,
        40 => 0.37,
        41 => 0.36,
        42 => 0.37,
        43 => 0.38,
        44 => 0.37,
        45 => 0.35,
        46 => 0.36,
        47 => 0.37,
        48 => 0.35,
        49 => 0.33,
        50 => 0.34,
        51 => 0.35,
        52 => 0.31,
        53 => 0.33,
        54 => 0.32,
        // The HIG tracking table lists entries for 54 and 56 but skips 55;
        // we use the 56 pt value for 55 pt via linear interpolation (54 → 0.32,
        // 56 → 0.30 interpolates to 0.31 at 55 pt, rounded to 0.30 to match
        // the next published step).
        55..=56 => 0.30,
        57..=58 => 0.28,
        59..=60 => 0.26,
        61..=62 => 0.24,
        63..=64 => 0.22,
        65..=66 => 0.19,
        67..=68 => 0.17,
        69..=70 => 0.14,
        71..=72 => 0.14,
        73..=76 => 0.07,
        77..=79 => 0.0,
        _ => 0.0,
    }
}

/// Returns the (default_size, minimum_size) for the given platform per HIG.
///
/// | Platform | Default | Minimum |
/// |---|---|---|
/// | iOS, iPadOS | 17 pt | 11 pt |
/// | macOS | 13 pt | 10 pt |
/// | tvOS | 29 pt | 23 pt |
/// | visionOS | 17 pt | 12 pt |
/// | watchOS | 16 pt | 12 pt |
pub fn platform_text_size(platform: &str) -> (f32, f32) {
    match platform {
        "ios" | "ipados" => (17.0, 11.0),
        "macos" => (13.0, 10.0),
        "tvos" => (29.0, 23.0),
        "visionos" => (17.0, 12.0),
        "watchos" => (16.0, 12.0),
        _ => (13.0, 10.0), // default to macOS
    }
}

/// Bumps a font weight up one step (for BoldText accessibility mode).
/// Normal -> Medium, Medium -> Semibold, Semibold -> Bold, Bold -> ExtraBold, etc.
/// `EXTRA_BOLD` and `BLACK` both saturate at `BLACK` — no weight exists beyond
/// it so the mapping is a no-op for those inputs.
///
/// Uses range-based dispatch so that custom `FontWeight` values (e.g., `FontWeight(150.0)`)
/// map to the correct next step instead of falling through to BLACK.
pub fn bold_step(w: FontWeight) -> FontWeight {
    match w.0 as u32 {
        // THIN (100) -> EXTRA_LIGHT
        0..150 => FontWeight::EXTRA_LIGHT,
        // EXTRA_LIGHT (200) -> LIGHT
        150..250 => FontWeight::LIGHT,
        // LIGHT (300) -> NORMAL
        250..350 => FontWeight::NORMAL,
        // NORMAL (400) -> MEDIUM
        350..450 => FontWeight::MEDIUM,
        // MEDIUM (500) -> SEMIBOLD
        450..550 => FontWeight::SEMIBOLD,
        // SEMIBOLD (600) -> BOLD
        550..650 => FontWeight::BOLD,
        // BOLD (700) -> EXTRA_BOLD
        650..750 => FontWeight::EXTRA_BOLD,
        // EXTRA_BOLD (800) and BLACK (900) both saturate at BLACK.
        _ => FontWeight::BLACK,
    }
}
