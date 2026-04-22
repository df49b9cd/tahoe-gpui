//! Flat prelude ordered by HIG taxonomy.
//!
//! Importing this module (`use tahoe_gpui::prelude::*;`) pulls in the
//! common public surface across all 8 HIG component subcategories plus the
//! foundations types most host apps rely on. Organized by HIG taxonomy so
//! coverage gaps are easy to spot.
//!
//! Stub modules (WebView, TabView, ColumnView, Lockup,
//! VirtualKeyboard, Ornament, and the `system_experiences` surface) do not
//! re-export anything — they only carry HIG documentation URLs today.

// ── Foundations ────────────────────────────────────────────────────────────

pub use crate::foundations::blink_manager::{
    BlinkManager, BlinkPhaseChanged, CURSOR_BLINK_INTERVAL_MS,
};
pub use crate::foundations::color::HslaAlphaExt;
pub use crate::foundations::icons::{
    AnimatedIcon, EmbeddedIconAssets, Icon, IconAnimation, IconLayoutBehavior, IconName,
    IconRenderMode, IconScale, IconStyle,
};
pub use crate::foundations::layout::{
    ControlSize, DROPDOWN_SNAP_MARGIN, FlexExt, Platform, h_flex, hit_region,
    snap_to_window_margin, v_flex,
};
pub use crate::foundations::right_to_left::{IconDirection, icon_direction};
pub use crate::foundations::theme::{
    ActiveTheme, GlassSize, SurfaceContext, TahoeTheme, TextStyle, TextStyledExt,
};

// ── Components > Content ───────────────────────────────────────────────────

pub use crate::components::content::{
    AnnotationContent, AnnotationPosition, AnnotationTarget, Avatar, AvatarSize, AvatarStatus,
    AxisConfig, AxisDescriptor, AxisMarks, AxisPosition, AxisTickStyle, AxisValueFormatter, Badge,
    BadgeVariant, BarOrientation, CategoryScale, Chart, ChartAnnotation, ChartDataSeries,
    ChartDataSet, ChartDescriptor, ChartPoint, ChartScrollConfig, ChartSeries, ChartType,
    ChartView, ContentMode, DateScale, GridLineStyle, GridlineConfig, InterpolationMethod, Label,
    LabelVariant, LegendPosition, LinearScale, LogScale, MarkStackingMethod, PlottableValue, Scale,
    SelectedPoint, SelectionBinding, SeriesDescriptor, TextView,
};

// ── Components > Layout and organization ───────────────────────────────────

pub use crate::components::layout_and_organization::{
    BoxStyle, BoxView, CollectionLayout, CollectionView, Disclosure, DisclosureGroup, FlexActions,
    FlexAlign, FlexContent, FlexHeader, List, ListRow, ListSection, ListStyle, OutlineNode,
    OutlineView, SelectionMode, Separator, SeparatorOrientation, SortDirection, SplitOrientation,
    SplitView, Table, TableColumn, TableRow,
};

// ── Components > Menus and actions ─────────────────────────────────────────

pub use crate::components::menus_and_actions::{
    Button, ButtonGroup, ButtonShape, ButtonSize, ButtonVariant, ContextMenu, ContextMenuEntry,
    ContextMenuItem, ContextMenuItemStyle, CopyButton, Menu, MenuBar, PopupButton, PopupItem,
    PulldownButton, PulldownItem, PulldownItemStyle,
};

// ── Components > Navigation and search ─────────────────────────────────────

pub use crate::components::navigation_and_search::{
    NavigationBarIOS, NavigationSplitView, PathControl, PathControlStyle, PathSegment, SearchBar,
    SearchField, Sidebar, SidebarItem, SidebarPosition, SidebarSection, TabBar, TabBarStyle,
    TabItem, TokenField, TokenItem, Toolbar, ToolbarStyle,
};

// ── Components > Presentation ──────────────────────────────────────────────

pub use crate::components::presentation::{
    ActionSheet, ActionSheetItem, ActionSheetPresentation, ActionSheetStyle, Alert, AlertAction,
    AlertActionRole, HoverCard, HoverCardPlacement, Modal, ModalLevel, PageControls, Panel,
    PanelPosition, PanelStyle, Popover, PopoverPlacement, ScrollView, Sheet, SheetDetent,
    SheetPresentation, Tooltip, WindowStyle,
};

// ── Components > Selection and input ───────────────────────────────────────

pub use crate::components::selection_and_input::{
    Checkbox, CheckboxState, ColorWell, ColorWellStyle, ComboBox, DateDisplayFormat, DatePicker,
    DatePickerStyle, DigitEntry, ImageWell, Picker, PickerItem, PickerSection, PickerStyle,
    SegmentItem, SegmentedControl, SimpleDate, Slider, SliderOrientation, Stepper, SubmitLabel,
    TextContentType, TextField, TextFieldStyle, TextFieldValidation, TimePicker, TimePickerStyle,
    Toggle, ToggleSize,
};

// ── Components > Status ────────────────────────────────────────────────────

/// `RatingIndicator` is re-exported only on macOS — HIG restricts the
/// control to that platform ("Not supported in iOS, iPadOS, tvOS,
/// visionOS, or watchOS").
#[cfg(target_os = "macos")]
pub use crate::components::status::RatingIndicator;
pub use crate::components::status::{
    ACTIVITY_RING_EXERCISE, ACTIVITY_RING_MOVE, ACTIVITY_RING_STAND, ActivityIndicator,
    ActivityIndicatorSize, ActivityIndicatorStyle, ActivityRing, ActivityRingSet, Gauge,
    GaugeDirection, GaugeStyle, ProgressIndicator, ProgressIndicatorSize, ProgressIndicatorValue,
    Shimmer, ShimmerEasing, SweepDirection, TextShimmer, TextShimmerState,
};

// ── Components > System experiences ────────────────────────────────────────
// Stub-only today: Widgets, Notifications, Live Activities, App Shortcuts,
// Controls, Status bars, Complications, Top Shelf, Watch faces.

// ── Markdown ───────────────────────────────────────────────────────────────

pub use crate::markdown::{
    IncrementalMarkdownParser, InlineContent, MarkdownBlock, StreamSettings, TableAlignment,
};

// ── Citations ──────────────────────────────────────────────────────────────

pub use crate::citation::{CitationSource, InlineCitation};

// ── Keybindings ────────────────────────────────────────────────────────────

pub use crate::{all_keybindings, text_keybindings, textfield_keybindings};
