//! Focus-graph grouping primitive: [`FocusGroup`] + [`FocusGroupExt`].

use gpui::{App, FocusHandle, InteractiveElement, KeyDownEvent, Window};

/// Traversal behavior at the edges of a [`FocusGroup`].
///
/// - [`Open`](Self::Open): the default — no Tab interception; focus falls
///   through to the enclosing order at edges. Use for logical clusters where
///   Tab should still exit naturally (toolbar slot clusters, form rows).
/// - [`Cycle`](Self::Cycle): wrap around inside the group's programmatic
///   navigation (`focus_next` / `focus_previous`). Intended as the
///   *focus-movement* substrate for arrow-key clusters — segmented
///   controls, tab bars, and the focus half of an APG radio group. Tab
///   is left to GPUI's native tab-stop map so the surrounding document
///   order stays walkable — this means Tab does **not** wrap at the
///   group's edges. Only the programmatic
///   `focus_next` / `focus_previous` entry points honor Cycle's wrap
///   contract; host-bound keys (typically arrow keys) are the intended
///   driver. Selection-follows-focus (as APG's radio-group pattern
///   requires) is the host's responsibility — `FocusGroup` moves focus
///   only.
/// - [`Trap`](Self::Trap): Tab and Shift+Tab are intercepted, wrapped, and
///   consumed so focus cannot escape the group. Use for modal dialogs and
///   action sheets following the WAI-ARIA dialog pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusGroupMode {
    #[default]
    Open,
    Cycle,
    Trap,
}

struct FocusGroupInner {
    handles: Vec<FocusHandle>,
    mode: FocusGroupMode,
}

/// Ordered collection of [`FocusHandle`]s forming a logical focus cluster.
///
/// Cheap to clone — the backing storage is an [`Rc`](std::rc::Rc), shared by
/// all clones. Intended to live on the parent entity (or as a stateless
/// value passed by reference) and be populated by children during render.
///
/// # Why this exists
///
/// GPUI exposes per-element `tab_index` / `tab_stop` / `tab_group` on
/// [`InteractiveElement`] plus `Window::focus_next` / `focus_prev` for Tab
/// traversal, but has no grouping primitive that bundles those with programmatic
/// navigation (`focus_next`/`focus_previous`/`focus_first`/`focus_last`) and
/// trap/cycle semantics. Without this layer, every component that needs an
/// arrow-key cluster or modal focus trap re-implements the wrap math and
/// Tab-swallow. See [`Modal`](crate::components::presentation::Modal) for a
/// concrete Trap-mode user.
///
/// # Contracts
///
/// - **Registration order is tab order.** The first call to [`register`](Self::register)
///   (directly or via [`FocusGroupExt::focus_group`]) receives index `0`, the
///   next gets `1`, and so on. Hosts that iterate an ordered collection of
///   child handles get the expected visual tab order for free.
/// - **Disabled members must be deregistered by the host.** `FocusGroup` has
///   no enabled/disabled state; a registered handle is always a valid focus
///   target from the group's perspective. Hosts that hide or disable a row
///   must [`clear`](Self::clear) and re-register the remaining handles (or
///   avoid calling [`FocusGroupExt::focus_group`] in the first place) so
///   programmatic navigation does not land on an inert element.
///
/// # Typical usage
///
/// ```ignore
/// // On the parent entity (e.g. a segmented control):
/// struct MySegmentedControl {
///     options: Vec<FocusHandle>,
///     group: FocusGroup,
/// }
///
/// impl MySegmentedControl {
///     fn new(cx: &mut Context<Self>) -> Self {
///         Self {
///             options: (0..3).map(|_| cx.focus_handle()).collect(),
///             group: FocusGroup::cycle(),
///         }
///     }
/// }
///
/// // In render, register each option and attach keyboard nav:
/// div().on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
///     match ev.keystroke.key.as_str() {
///         "right" | "down" => this.group.focus_next(window, cx),
///         "left"  | "up"   => this.group.focus_previous(window, cx),
///         _ => {}
///     }
/// }))
/// ```
#[derive(Clone)]
pub struct FocusGroup {
    inner: std::rc::Rc<std::cell::RefCell<FocusGroupInner>>,
}

impl Default for FocusGroup {
    fn default() -> Self {
        Self::new(FocusGroupMode::default())
    }
}

impl std::fmt::Debug for FocusGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("FocusGroup")
            .field("mode", &inner.mode)
            .field("len", &inner.handles.len())
            .finish()
    }
}

impl FocusGroup {
    /// Construct an empty group with the given traversal [`mode`](FocusGroupMode).
    pub fn new(mode: FocusGroupMode) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(FocusGroupInner {
                handles: Vec::new(),
                mode,
            })),
        }
    }

    /// Construct an [`Open`](FocusGroupMode::Open) group (no edge behavior).
    pub fn open() -> Self {
        Self::new(FocusGroupMode::Open)
    }

    /// Construct a [`Cycle`](FocusGroupMode::Cycle) group (wraps at edges for
    /// programmatic `focus_next`/`focus_previous`).
    pub fn cycle() -> Self {
        Self::new(FocusGroupMode::Cycle)
    }

    /// Construct a [`Trap`](FocusGroupMode::Trap) group (intercepts Tab so
    /// focus cannot escape — typical for modal dialogs).
    pub fn trap() -> Self {
        Self::new(FocusGroupMode::Trap)
    }

    /// Current traversal [`mode`](FocusGroupMode).
    pub fn mode(&self) -> FocusGroupMode {
        self.inner.borrow().mode
    }

    /// Number of registered members.
    pub fn len(&self) -> usize {
        self.inner.borrow().handles.len()
    }

    /// `true` when no members are registered.
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().handles.is_empty()
    }

    /// Register `handle` as a member, appending to tab order.
    ///
    /// Idempotent: if a handle comparing equal (`FocusHandle: PartialEq` by
    /// FocusId) is already registered, the collection is left unchanged and the
    /// existing index is returned. Safe to call every render.
    ///
    /// The returned index is the handle's position in the group — and, by
    /// contract, its [`tab_index`](InteractiveElement::tab_index) when wired
    /// through [`FocusGroupExt::focus_group`]. Callers relying on a specific
    /// visual tab order must register in that order.
    pub fn register(&self, handle: &FocusHandle) -> usize {
        let mut inner = self.inner.borrow_mut();
        if let Some(idx) = inner.handles.iter().position(|h| h == handle) {
            return idx;
        }
        inner.handles.push(handle.clone());
        inner.handles.len() - 1
    }

    /// Remove all registered handles. Call from the parent's render when
    /// member identity changes frame-to-frame (e.g. the number of options
    /// depends on runtime state).
    pub fn clear(&self) {
        self.inner.borrow_mut().handles.clear();
    }

    /// Replace the registered handles with those yielded by `handles`,
    /// preserving iteration order as tab order.
    ///
    /// Equivalent to [`clear`](Self::clear) followed by
    /// [`register`](Self::register) for each handle, but performed under
    /// a single borrow. Use from the render path when member identity
    /// (not just count) changes frame-to-frame — avoids accidentally
    /// leaving stale handles from a prior frame.
    ///
    /// Duplicates in the input iterator are dropped (first occurrence
    /// wins), mirroring [`register`](Self::register)'s idempotency so
    /// `len()` and tab-index semantics stay consistent between the two
    /// entry points.
    pub fn set_members<'a>(&self, handles: impl IntoIterator<Item = &'a FocusHandle>) {
        let mut inner = self.inner.borrow_mut();
        inner.handles.clear();
        for handle in handles {
            if !inner.handles.iter().any(|existing| existing == handle) {
                inner.handles.push(handle.clone());
            }
        }
    }

    /// Focus the first registered member. No-op when the group is empty.
    pub fn focus_first(&self, window: &mut Window, cx: &mut App) {
        let handle = self.inner.borrow().handles.first().cloned();
        if let Some(handle) = handle {
            handle.focus(window, cx);
        }
    }

    /// Focus the last registered member. No-op when the group is empty.
    pub fn focus_last(&self, window: &mut Window, cx: &mut App) {
        let handle = self.inner.borrow().handles.last().cloned();
        if let Some(handle) = handle {
            handle.focus(window, cx);
        }
    }

    /// Advance focus to the next member. In [`FocusGroupMode::Open`] stops at
    /// the last member (no wrap); in `Cycle` / `Trap` wraps to the first.
    /// When no member currently holds focus, lands on the first.
    pub fn focus_next(&self, window: &mut Window, cx: &mut App) {
        self.advance(window, cx, /* forward */ true);
    }

    /// Retreat focus to the previous member. In [`FocusGroupMode::Open`]
    /// stops at the first member (no wrap); in `Cycle` / `Trap` wraps to the
    /// last. When no member currently holds focus, lands on the last.
    pub fn focus_previous(&self, window: &mut Window, cx: &mut App) {
        self.advance(window, cx, /* forward */ false);
    }

    fn advance(&self, window: &mut Window, cx: &mut App, forward: bool) {
        // Snapshot the member handles and mode under a short borrow, then
        // release it before calling `is_focused` / `focus`. Those helpers
        // are pure today but live in GPUI's focus machinery — a future
        // release that notifies reentrantly would otherwise panic on the
        // RefCell.
        let (handles, wrap) = {
            let inner = self.inner.borrow();
            if inner.handles.is_empty() {
                return;
            }
            let wrap = matches!(inner.mode, FocusGroupMode::Cycle | FocusGroupMode::Trap);
            (inner.handles.clone(), wrap)
        };
        let len = handles.len();
        let current = handles.iter().position(|h| h.is_focused(window));
        let next = match current {
            Some(idx) if forward => {
                if idx + 1 < len {
                    Some(idx + 1)
                } else if wrap {
                    Some(0)
                } else {
                    None
                }
            }
            Some(idx) => {
                if idx > 0 {
                    Some(idx - 1)
                } else if wrap {
                    Some(len - 1)
                } else {
                    None
                }
            }
            None if forward => Some(0),
            None => Some(len - 1),
        };
        if let Some(i) = next {
            handles[i].focus(window, cx);
        }
    }

    /// True when any registered member currently holds focus.
    pub fn contains_focused(&self, window: &Window) -> bool {
        let handles = self.inner.borrow().handles.clone();
        handles.iter().any(|h| h.is_focused(window))
    }

    /// Hook to call from the group host's `on_key_down`.
    ///
    /// In [`FocusGroupMode::Trap`], intercepts Tab / Shift+Tab, calls
    /// `cx.stop_propagation()`, and advances focus through the group with
    /// wrap-around. Returns `true` when the event was consumed. In `Open` /
    /// `Cycle` modes this is a no-op (`false`) so GPUI's native TabStopMap
    /// continues to drive Tab traversal.
    pub fn handle_key_down(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) -> bool {
        if event.keystroke.key.as_str() != "tab" {
            return false;
        }
        if self.mode() != FocusGroupMode::Trap {
            return false;
        }
        cx.stop_propagation();
        if event.keystroke.modifiers.shift {
            self.focus_previous(window, cx);
        } else {
            self.focus_next(window, cx);
        }
        true
    }
}

/// Extension trait that opts a GPUI [`InteractiveElement`] into a
/// [`FocusGroup`].
///
/// Equivalent to calling `.track_focus(handle).tab_index(group_index)` with
/// the group managing the index. Safe to call every render — [`FocusGroup::register`]
/// is idempotent by FocusId.
pub trait FocusGroupExt: InteractiveElement + Sized {
    /// Register `handle` with `group` (appending to tab order if it is
    /// not already a member) and wire the element as its focus target.
    ///
    /// The index assigned by [`FocusGroup::register`] is used as the
    /// element's [`tab_index`](InteractiveElement::tab_index), so
    /// registration order becomes the visual tab order. Idempotent:
    /// re-registering the same handle on a later render returns the
    /// existing index and leaves the group unchanged.
    fn focus_group(self, group: &FocusGroup, handle: &FocusHandle) -> Self {
        let index = group.register(handle) as isize;
        self.track_focus(handle).tab_index(index)
    }
}

impl<E: InteractiveElement + Sized> FocusGroupExt for E {}

#[cfg(test)]
mod tests {
    use super::{FocusGroup, FocusGroupMode};
    use core::prelude::v1::test;

    #[test]
    fn focus_group_mode_default_is_open() {
        assert_eq!(FocusGroupMode::default(), FocusGroupMode::Open);
    }

    #[test]
    fn focus_group_constructors_set_mode() {
        assert_eq!(FocusGroup::open().mode(), FocusGroupMode::Open);
        assert_eq!(FocusGroup::cycle().mode(), FocusGroupMode::Cycle);
        assert_eq!(FocusGroup::trap().mode(), FocusGroupMode::Trap);
        assert_eq!(FocusGroup::default().mode(), FocusGroupMode::Open);
    }

    #[test]
    fn focus_group_starts_empty() {
        let group = FocusGroup::trap();
        assert!(group.is_empty());
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn focus_group_clones_share_inner() {
        let group = FocusGroup::cycle();
        let clone = group.clone();
        // Shared Rc<RefCell<_>>: clearing via the clone is observable
        // through the original handle. Writing to one must not create a
        // snapshot.
        clone.clear();
        assert_eq!(group.len(), 0);
        assert_eq!(clone.len(), 0);
    }

    #[test]
    fn focus_group_debug_contains_mode_and_len() {
        let group = FocusGroup::trap();
        let repr = format!("{:?}", group);
        assert!(
            repr.contains("Trap"),
            "Debug output should mention mode: {repr}"
        );
        assert!(
            repr.contains("len"),
            "Debug output should include member count: {repr}"
        );
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::{FocusGroup, FocusGroupExt, FocusGroupMode};
    use crate::test_helpers::helpers::setup_test_window;
    use core::prelude::v1::test;
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    /// Harness that mints three focus handles and tracks them inside a
    /// `FocusGroup`. Tests drive the group API directly via `update_in` on
    /// this entity — the render path exists so GPUI's window has live focus
    /// wiring (focus requests and `is_focused` queries need a rendered frame).
    struct GroupHarness {
        handles: [FocusHandle; 3],
        group: FocusGroup,
    }

    impl GroupHarness {
        fn new(mode: FocusGroupMode, cx: &mut Context<Self>) -> Self {
            let handles = [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()];
            let group = FocusGroup::new(mode);
            for handle in &handles {
                group.register(handle);
            }
            Self { handles, group }
        }
    }

    impl Render for GroupHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .w(px(200.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .id("member-0")
                        .track_focus(&self.handles[0])
                        .child("0"),
                )
                .child(
                    div()
                        .id("member-1")
                        .track_focus(&self.handles[1])
                        .child("1"),
                )
                .child(
                    div()
                        .id("member-2")
                        .track_focus(&self.handles[2])
                        .child("2"),
                )
        }
    }

    #[gpui::test]
    async fn register_is_idempotent(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update(cx, |host, _cx| {
            assert_eq!(host.group.len(), 3);
            // Re-registering the same handles must be a no-op.
            let idx = host.group.register(&host.handles[1]);
            assert_eq!(idx, 1, "re-register returns the existing index");
            assert_eq!(host.group.len(), 3, "len unchanged after re-register");
        });
    }

    #[gpui::test]
    async fn register_appends_new_handle_and_returns_index(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update(cx, |host, cx| {
            let fresh = cx.focus_handle();
            let idx = host.group.register(&fresh);
            assert_eq!(idx, 3, "append path returns the newly assigned index");
            assert_eq!(host.group.len(), 4, "len reflects the appended handle");
            // A second call with the same fresh handle returns the same index
            // without growing the collection.
            let idx_again = host.group.register(&fresh);
            assert_eq!(idx_again, 3);
            assert_eq!(host.group.len(), 4);
        });
    }

    #[gpui::test]
    async fn focus_first_on_empty_is_noop(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.clear();
            // Must not panic — empty group is a valid state.
            host.group.focus_first(window, cx);
            host.group.focus_last(window, cx);
            host.group.focus_next(window, cx);
            host.group.focus_previous(window, cx);
        });
    }

    #[gpui::test]
    async fn focus_first_lands_on_first_member(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_first(window, cx);
            assert!(host.handles[0].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_last_lands_on_last_member(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_last(window, cx);
            assert!(host.handles[2].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_next_wraps_in_cycle_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            assert!(host.handles[2].is_focused(window));
            host.group.focus_next(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Cycle: focus_next past last wraps to first"
            );
        });
    }

    #[gpui::test]
    async fn focus_previous_wraps_in_trap_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(
                host.handles[2].is_focused(window),
                "Trap: focus_previous past first wraps to last"
            );
        });
    }

    #[gpui::test]
    async fn focus_previous_wraps_in_cycle_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(
                host.handles[2].is_focused(window),
                "Cycle: focus_previous past first wraps to last"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_wraps_in_trap_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Trap: focus_next past last wraps to first"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_stops_at_edge_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(
                host.handles[2].is_focused(window),
                "Open: focus_next past last stays on last"
            );
        });
    }

    #[gpui::test]
    async fn focus_previous_stops_at_edge_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Open: focus_previous past first stays on first"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_with_no_current_focus_lands_on_first(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_next(window, cx);
            assert!(host.handles[0].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_previous_with_no_current_focus_lands_on_last(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_previous(window, cx);
            assert!(host.handles[2].is_focused(window));
        });
    }

    #[gpui::test]
    async fn contains_focused_tracks_membership(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, _cx| {
            assert!(!host.group.contains_focused(window));
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        host.update_in(cx, |host, window, _cx| {
            assert!(host.group.contains_focused(window));
        });
    }

    #[gpui::test]
    async fn handle_key_down_ignores_non_tab_keys(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "enter".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(!host.group.handle_key_down(&event, window, cx));
        });
    }

    #[gpui::test]
    async fn handle_key_down_swallows_tab_in_trap_mode(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(
                host.group.handle_key_down(&event, window, cx),
                "Trap mode consumes Tab"
            );
            assert!(
                host.handles[1].is_focused(window),
                "Trap Tab advances to next member"
            );
        });
    }

    #[gpui::test]
    async fn handle_key_down_passes_through_tab_in_cycle_mode(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(
                !host.group.handle_key_down(&event, window, cx),
                "Cycle mode leaves Tab to GPUI's native tab-stop map"
            );
        });
    }

    #[gpui::test]
    async fn handle_key_down_shift_tab_trap_retreats(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        host.update_in(cx, |host, window, cx| {
            let modifiers = Modifiers {
                shift: true,
                ..Default::default()
            };
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers,
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(host.group.handle_key_down(&event, window, cx));
            assert!(
                host.handles[2].is_focused(window),
                "Trap Shift+Tab wraps to last from first"
            );
        });
    }

    #[gpui::test]
    async fn handle_key_down_passes_through_tab_in_open_mode(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(
                !host.group.handle_key_down(&event, window, cx),
                "Open mode leaves Tab to GPUI's native tab-stop map"
            );
        });
    }

    // Harness used to stress boundary sizes (1 and 2 members) and the render-path
    // idempotency contract of `FocusGroupExt`. Three-member tests all use
    // `GroupHarness`; the per-test configs kept here are small enough to inline.
    struct SmallGroupHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
    }

    impl SmallGroupHarness {
        fn new(count: usize, mode: FocusGroupMode, cx: &mut Context<Self>) -> Self {
            let handles: Vec<FocusHandle> = (0..count).map(|_| cx.focus_handle()).collect();
            let group = FocusGroup::new(mode);
            for handle in &handles {
                group.register(handle);
            }
            Self { handles, group }
        }
    }

    impl Render for SmallGroupHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let mut root = div().w(px(200.0)).h(px(80.0)).flex().flex_col();
            for (i, handle) in self.handles.iter().enumerate() {
                root = root.child(
                    div()
                        .id(("small-member", i))
                        .track_focus(handle)
                        .child(format!("{i}")),
                );
            }
            root
        }
    }

    #[gpui::test]
    async fn set_members_replaces_existing_registrations(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update(cx, |host, cx| {
            assert_eq!(host.group.len(), 3, "precondition: three pre-registered");

            // Replace with two fresh handles; pre-existing handles must drop.
            let fresh_a = cx.focus_handle();
            let fresh_b = cx.focus_handle();
            host.group.set_members([&fresh_a, &fresh_b]);

            assert_eq!(host.group.len(), 2, "set_members replaces, not appends");
            assert_eq!(
                host.group.register(&fresh_a),
                0,
                "first supplied handle is at index 0"
            );
            assert_eq!(
                host.group.register(&fresh_b),
                1,
                "second supplied handle is at index 1"
            );
            // Original handles are no longer members.
            assert_eq!(
                host.group.register(&host.handles[0]),
                2,
                "previously-registered handle is gone and re-registers fresh"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_wraps_with_two_members(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SmallGroupHarness::new(2, FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Cycle with two members wraps last→first"
            );
            host.group.focus_next(window, cx);
            assert!(
                host.handles[1].is_focused(window),
                "Cycle with two members advances first→last"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_with_single_member_stays_put(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            SmallGroupHarness::new(1, FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Cycle with one member wraps to itself"
            );
            host.group.focus_previous(window, cx);
            assert!(host.handles[0].is_focused(window));
        });
    }

    // Mirrors the render-path call pattern: every frame a host calls
    // `.focus_group(&group, &handle)` on each child element. The registration
    // must not grow across renders, or tab indices would drift.
    struct ExtIdempotencyHarness {
        handles: [FocusHandle; 3],
        group: FocusGroup,
    }

    impl ExtIdempotencyHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
                group: FocusGroup::cycle(),
            }
        }
    }

    impl Render for ExtIdempotencyHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .w(px(200.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .id("ext-0")
                        .focus_group(&self.group, &self.handles[0])
                        .child("0"),
                )
                .child(
                    div()
                        .id("ext-1")
                        .focus_group(&self.group, &self.handles[1])
                        .child("1"),
                )
                .child(
                    div()
                        .id("ext-2")
                        .focus_group(&self.group, &self.handles[2])
                        .child("2"),
                )
        }
    }

    #[gpui::test]
    async fn focus_group_ext_is_idempotent_across_renders(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ExtIdempotencyHarness::new(cx));
        // The first render registered all three handles.
        host.update(cx, |host, _cx| {
            assert_eq!(host.group.len(), 3);
        });
        // Force several more render passes and confirm the group stays put.
        for _ in 0..3 {
            host.update(cx, |host, cx| {
                cx.notify();
                assert_eq!(host.group.len(), 3);
            });
            cx.run_until_parked();
        }
        host.update(cx, |host, _cx| {
            assert_eq!(host.group.len(), 3, "render loop must not grow the group");
        });
    }
}
