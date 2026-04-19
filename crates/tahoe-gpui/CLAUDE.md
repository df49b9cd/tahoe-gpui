# tahoe-gpui

Human Interface Guidelines components for GPUI. Standalone — no AI SDK dependency.

## Build & Test

```bash
cargo check -p tahoe-gpui
cargo nextest run -p tahoe-gpui
cargo clippy -p tahoe-gpui
```

## Run Examples

```bash
cargo run -p tahoe-gpui --example component_gallery
cargo run -p tahoe-gpui --example button_gallery
cargo run -p tahoe-gpui --example alert_gallery
cargo run -p tahoe-gpui --example icon_gallery
cargo run -p tahoe-gpui --example liquid_glass_gallery
cargo run -p tahoe-gpui --example liquid_glass_interactive
cargo run -p tahoe-gpui --example theme_gallery
cargo run -p tahoe-gpui --example streaming
cargo run -p tahoe-gpui --example code_gallery
cargo run -p tahoe-gpui --example workflow_demo
cargo run -p tahoe-gpui --example window_layouts
cargo run -p tahoe-gpui --example voice_demo --features voice
# macOS 26 (Tahoe) UI Kit screen mockups:
cargo run -p tahoe-gpui --example auth_app
cargo run -p tahoe-gpui --example music_app
cargo run -p tahoe-gpui --example list_app
cargo run -p tahoe-gpui --example dashboard_app
cargo run -p tahoe-gpui --example toolbar_app
```

## Architecture

### HIG-Aligned Module Structure

```
foundations/           # Design-system primitives
  color.rs             # SystemPalette, SystemColor, Appearance
  theme.rs             # TahoeTheme (global design tokens)
  typography.rs        # TextStyle, FontDesign, Dynamic Type
  icons/               # SF Symbols: Icon, IconName, AnimatedIcon
  materials.rs         # Liquid Glass: glass_surface(), GlassStyle
  layout.rs            # MIN_TOUCH_TARGET, margins, LayoutDirection
  motion.rs            # SpringAnimation, MotionTokens
  accessibility.rs     # AccessibilityMode, focus rings, contrast

components/            # HIG-organized UI controls (8 HIG subcategories)
  content/             # Label, Badge, Avatar, TextView (+ Chart, WebView stubs)
  menus_and_actions/   # Button, ContextMenu, MenuBar, PopupButton, PulldownButton
  navigation_and_search/ # Toolbar, TabBar, Sidebar, SearchField, PathControl
  presentation/        # Alert, ActionSheet, Sheet, Popover, Modal, Tooltip, Panel, ScrollView
  selection_and_input/ # TextField, Toggle, Slider, Stepper, Picker, DatePicker, ImageWell
  layout_and_organization/ # DisclosureGroup, SplitView, Table, Separator (+ TabView/ColumnView/Lockup stubs)
  status/              # ProgressIndicator, ActivityIndicator, Gauge, Shimmer, RatingIndicator
  system_experiences/  # Widgets, Notifications, Live Activities, … (all 9 pages stubbed)
```

### Component Patterns

- **Stateful** (`Entity<T>` where `T: Render`): For mutable state. Use `cx.notify()` to re-render. Examples: `TextField`, `StreamingMarkdown`.
- **Stateless** (`#[derive(IntoElement)]` + `RenderOnce`): Builder-pattern components. Examples: `Button`, `Alert`, `Badge`.

### Theme System

`TahoeTheme` is a GPUI global. Register before rendering:
```rust
cx.set_global(TahoeTheme::dark()); // or ::light(), ::liquid_glass()
```
Components read tokens via `cx.global::<TahoeTheme>()`.
For runtime switching: `theme.apply(cx)` calls `cx.refresh_windows()`.

### HIG Component Renames

| Old Name | New HIG-Aligned Name |
|---|---|
| `TextInput` | `TextField` |
| `Switch` | `Toggle` |
| `Tabs` | `TabBar` |
| `ProgressBar` | `ProgressIndicator` |
| `Collapsible` | `DisclosureGroup` |
| `HorizontalScroll` | `ScrollView` |

### Features

- `voice` — Audio/speech components (requires `cpal`)
- `test-support` — GPUI test harness

### Key Conventions

- Use `cx.listener()` for action handlers on entity-owned components
- Use `cx.processor()` for `uniform_list` / `list` item rendering callbacks
- Tests use `use core::prelude::v1::test;` to override gpui's test macro
