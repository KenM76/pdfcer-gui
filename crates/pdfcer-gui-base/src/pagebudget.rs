//! # `pagebudget` — how long one page thumbnail may take to render, and its
//! spelling in the preferences file (milliseconds, `0` for no limit).

use std::time::Duration;

/// **The default per-page time limit: none** — what `ThumbnailCache::budget`
/// holds until the operator types a number.
pub const PAGE_BUDGET_DEFAULT: Option<Duration> = None;

/// The smallest budget the operator may set.
pub const MIN_PAGE_BUDGET: Duration = Duration::from_millis(100);

/// The largest budget the operator may set.
pub const MAX_PAGE_BUDGET: Duration = Duration::from_secs(60);

/// **The only place in this crate that decides what a budget number
/// MEANS** — `OPERATOR_REQUESTS.md` **O187**, 2026-09-12: *“setting it to 0
/// should set it to infinity (never time out)”*.
#[must_use]
pub fn budget_from_millis(millis: u64) -> Option<Duration> {
    if millis == 0 {
        return None;
    }
    Some(Duration::from_millis(millis).clamp(MIN_PAGE_BUDGET, MAX_PAGE_BUDGET))
}

/// The inverse, for the write-back that persists what the operator chose.
#[must_use]
pub fn millis_from_budget(budget: Option<Duration>) -> u64 {
    // `as u64` is lossless here: the value is clamped to MAX_PAGE_BUDGET
    // (60 000) before it is ever stored, and `u128` -> `u64` of a number
    // under 2^16 cannot truncate.
    budget.map_or(0, |d| d.as_millis() as u64)
}
