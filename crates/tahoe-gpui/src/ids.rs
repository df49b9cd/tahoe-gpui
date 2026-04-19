//! Shared element ID generation.

use gpui::{ElementId, SharedString};
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique `ElementId` with the given static-string prefix.
///
/// Uses `ElementId::NamedInteger` so no `String` is allocated at the call
/// site — the prefix lives in `SharedString`'s interned path and the
/// counter is stored inline.
///
/// # Ordering: why `Relaxed` is sufficient
///
/// The ID counter only needs *atomicity* per increment, not happens-before
/// ordering with any other memory location. GPUI renders on a single
/// thread, so every call site observes the counter in program order.
/// `Ordering::Relaxed` gives us the atomic increment without a memory
/// fence, which is the minimum the C++ / Rust memory model permits.
///
/// **Test isolation.** Each `cargo nextest` process runs in a fresh
/// address space, so the static counter starts from 0 per process —
/// parallel nextest workers cannot collide on IDs because each worker
/// has its own `GLOBAL_ID_COUNTER`. The `Relaxed` ordering is safe
/// within any single process regardless of thread count.
pub fn next_element_id(prefix: &'static str) -> ElementId {
    // Relaxed: GPUI is single-threaded and each nextest process has an
    // independent static. See module-level safety note.
    let id = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    ElementId::NamedInteger(SharedString::new_static(prefix), id)
}

/// Generate a unique opaque string id with the given prefix — used by
/// non-rendering correlation handles (feedback callbacks, analytics
/// trace ids) that need a stable identifier across renders but not the
/// `ElementId` shape.
///
/// Uses the same global counter as [`next_element_id`], so ids are
/// monotonically unique within a process regardless of which helper
/// produced them.
pub fn next_element_id_string(prefix: &str) -> String {
    let id = GLOBAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{id}")
}
