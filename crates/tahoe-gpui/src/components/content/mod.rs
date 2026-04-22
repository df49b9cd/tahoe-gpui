//! Content display components (HIG: Components > Content).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/content>
//!
//! HIG pages in this subcategory: Charts, Image views, Text views, Web views.
//! `Avatar` and `Badge` are crate extensions beyond the HIG inventory.

pub mod avatar;
pub mod badge;
pub mod chart;
pub mod image_view;
pub mod label;
pub mod text_view;
pub mod web_view;

pub use avatar::{Avatar, AvatarSize, AvatarStatus};
pub use badge::{Badge, BadgeVariant};
pub use chart::{
    AnnotationContent, AnnotationPosition, AnnotationTarget, AxisConfig, AxisDescriptor, AxisMarks,
    AxisPosition, AxisTickStyle, AxisValueFormatter, BarOrientation, CategoryScale, Chart,
    ChartAnnotation, ChartDataSeries, ChartDataSet, ChartDescriptor, ChartPoint, ChartScrollConfig,
    ChartSeries, ChartType, ChartView, DateScale, GridLineStyle, GridlineConfig,
    InterpolationMethod, LegendPosition, LinearScale, LogScale, MarkStackingMethod, PlottableValue,
    Scale, SelectedPoint, SelectionBinding, SeriesDescriptor,
};
pub use image_view::{ContentMode, ImageView};
pub use label::{Label, LabelVariant};
pub use text_view::TextView;
pub use web_view::WebView;
