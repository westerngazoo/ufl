//! Unified recursion depth contract (SPEC-0003 §7).
//!
//! Exposes a thread-safe, configurable maximum recursion depth for parsing,
//! rendering, lowering, and evaluation to prevent DoS via stack overflow.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Default recursion depth limit, chosen to be safe for the default thread
/// stack size while accommodating typical nested expressions.
pub const DEFAULT_MAX_DEPTH: usize = 128;

static MAX_DEPTH: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_DEPTH);

/// Get the currently configured maximum recursion depth.
pub fn get_max_depth() -> usize {
    MAX_DEPTH.load(Ordering::Relaxed)
}

/// Set a new maximum recursion depth across the process.
pub fn set_max_depth(depth: usize) {
    MAX_DEPTH.store(depth, Ordering::Relaxed);
}
