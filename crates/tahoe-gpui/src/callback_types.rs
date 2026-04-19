//! Type aliases for GPUI callback types to reduce type complexity.
//!
//! GPUI callback signatures involving `&mut Window, &mut App` are complex
//! enough to trigger `clippy::type_complexity` warnings. This module provides
//! short aliases for the recurring patterns.

use gpui::AnyElement;
use gpui::App;
use gpui::ClickEvent;
use gpui::SharedString;
use gpui::Window;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

// ---- Generic element builders (no extra args) ----

/// `Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>`
pub type OnMutCallback = Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(&mut Window, &mut App) -> AnyElement>>`
pub type ElementBuilder = Option<Box<dyn Fn(&mut Window, &mut App) -> AnyElement>>;

/// `Option<Box<dyn Fn(&App) -> AnyElement + 'static>>`
pub type AppElementBuilder = Option<Box<dyn Fn(&App) -> AnyElement + 'static>>;

// ---- Click callbacks ----

/// `Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>`
pub type OnClick = Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>;

// ---- Toggle / bool change ----

/// `Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>`
pub type OnToggle = Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>`
pub type OnBoolRefChange = Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>;

// ---- String/str change ----

/// `Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>`
pub type OnStrChange = Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>`
pub type OnSharedStringChange = Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(String, &mut Window, &mut App) + 'static>>`
pub type OnStringChange = Option<Box<dyn Fn(String, &mut Window, &mut App) + 'static>>;

// ---- Numeric change ----

/// `Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>`
pub type OnF32Change = Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(f64, &mut Window, &mut App) + 'static>>`
pub type OnF64Change = Option<Box<dyn Fn(f64, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>`
pub type OnUsizeChange = Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>;

// ---- Arc-based (Send + Sync) ----

/// `Option<Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>>`
pub type OnMutCallbackArc = Option<Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>>;

// ---- Rc-based element/action renderers ----

/// `Option<Rc<dyn Fn(&str, &mut Window, &mut App) -> Option<AnyElement>>>`
pub type RenderActionsRc = Option<Rc<dyn Fn(&str, &mut Window, &mut App) -> Option<AnyElement>>>;

/// `Option<Box<dyn Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App) + 'static>>`
pub type OnFileClick =
    Option<Box<dyn Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App) + 'static>>;

/// `Option<Rc<dyn Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App)>>`
pub type OnFileClickRc = Option<Rc<dyn Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App)>>;

// ---- Typed select/change callbacks ----

/// `Option<Box<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>>`
pub type OnSharedStringRefChange =
    Option<Box<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>>;

// ---- Unique per-component types ----

/// `Option<Box<dyn Fn(HashSet<String>, &mut Window, &mut App) + 'static>>`
pub type OnExpandedChange = Option<Box<dyn Fn(HashSet<String>, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(gpui::Hsla, &mut Window, &mut App) + 'static>>`
/// Used for color picker / color well changes.
pub type OnHslaChange = Option<Box<dyn Fn(gpui::Hsla, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(u8, u8, &mut Window, &mut App) + 'static>>`
/// Used for time picker changes (hour, minute).
pub type OnTimeChange = Option<Box<dyn Fn(u8, u8, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(i32, u8, &mut Window, &mut App) + 'static>>`
/// Used for date picker month navigation (year, month).
pub type OnDateNavigate = Option<Box<dyn Fn(i32, u8, &mut Window, &mut App) + 'static>>;

// ---- Types shared with the rust-ai-elements chatbot module ----
// These types are *not* consumed inside tahoe-gpui itself, but they are
// actively referenced from `rust-ai-elements` (conversation.rs,
// prompt_input.rs, queue.rs, message.rs, streaming.rs). They live here
// rather than in the chatbot crate so both crates can name the same
// callback signature without one depending on the other. Removing any of
// them is a breaking change for the binding crate — keep them in sync.

/// `Option<Box<dyn Fn(&App) -> Vec<AnyElement> + 'static>>`
/// Used by `rust-ai-elements::chatbot::prompt_input` for dynamic
/// attachment element rendering.
pub type AppElementsBuilder = Option<Box<dyn Fn(&App) -> Vec<AnyElement> + 'static>>;

/// `Option<Box<dyn Fn(&str) -> String>>`
/// Used by `rust-ai-elements::chatbot::conversation` to transform
/// message text before display (e.g. markdown preprocessing).
pub type FormatMessageFn = Option<Box<dyn Fn(&str) -> String>>;

/// `Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>`
/// Shared-ref click callback used by `rust-ai-elements::chatbot::queue`
/// where the same handler is cloned across queue items.
pub type OnClickRc = Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>;

/// `Option<Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>`
/// Shared-ref index-change callback used by
/// `rust-ai-elements::chatbot::message`.
pub type OnUsizeChangeRc = Option<Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(String, bool, Option<String>, &mut Window, &mut App) + 'static>>`
/// Used by `rust-ai-elements::chatbot::streaming` for tool approval
/// responses (tool_id, approved, feedback).
pub type OnApprovalResponse =
    Option<Box<dyn Fn(String, bool, Option<String>, &mut Window, &mut App) + 'static>>;

/// `Option<Box<dyn Fn(gpui::Point<gpui::Pixels>, &mut Window, &mut App) + 'static>>`
pub type OnPointClick =
    Option<Box<dyn Fn(gpui::Point<gpui::Pixels>, &mut Window, &mut App) + 'static>>;

/// Wrap an optional callback in `Rc` for sharing across multiple closures.
///
/// Replaces the recurring `.map(std::rc::Rc::new)` pattern.
pub fn rc_wrap<T>(opt: Option<T>) -> Option<Rc<T>> {
    opt.map(Rc::new)
}

// Note: `Option<Box<dyn Fn(&mut Window, &mut Context<Self>) + 'static>>` can't
// be a generic alias due to `Context<Self>`. Use per-file alias in that case.
