//! # `statusfitting` — what the bar can afford to show when the window is narrow
//!
//! ## The defect this exists for
//!
//!
//! ```text
//! status-group:page    457.4 .. 603.1     ok
//! status-group:zoom    326.7 .. 435.4     ok
//! status-group:fit       6.6 .. 304.7     ok, and 298 pt wide — half the bar
//! status-group:find    -54.2 ..  -15.4    off the left edge
//! status-group:filter -127.5 ..  -76.2    off the left edge
//! ```
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/statusfitting.md`.

use std::collections::BTreeMap;

/// The groups of the bar's fixed right-hand cluster, in the order
/// `status::bar` adds them — which is right to left on screen, and
/// most-essential first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// `◀ n / N ▶`. **Never shed** — see the module header.
    Page,
    /// `− 100 % +`.
    Zoom,
    /// `Actual size · Fit width · Fit height · Fit page`. The widest of the
    /// five by a factor of three, and therefore the one whose absence buys the
    /// most.
    Fit,
    /// The Find toggle.
    Find,
    /// The selection filter.
    Filter,
}

impl Group {
    /// Every group, in the order the bar adds them.
    pub const ORDER: [Self; 5] = [Self::Page, Self::Zoom, Self::Fit, Self::Find, Self::Filter];

    /// The region name this group publishes its rect under, which is also the
    /// key its width is remembered by.
    pub const fn region(self) -> &'static str {
        match self {
            // ui-text-exempt: trace region names, never displayed
            Self::Page => "status-group:page",
            Self::Zoom => "status-group:zoom",
            Self::Fit => "status-group:fit",
            Self::Find => "status-group:find",
            Self::Filter => "status-group:filter",
        }
    }
}

/// **The groups this module may drop, in the order it drops them, each
/// beside a command that still reaches it.**
pub const SHED_ORDER: &[(Group, &str)] = &[
    // ui-text-exempt: command ids, never displayed
    (Group::Fit, "view.zoom_fit_page"),
    (Group::Find, "edit.find"),
];

/// What each group occupied on the previous frame, in points.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Widths(BTreeMap<Group, f32>);

impl Widths {
    /// Remember what `group` occupied.
    pub fn record(&mut self, group: Group, width: f32) {
        if width.is_finite() && width >= 0.0 {
            self.0.insert(group, width);
        }
    }

    /// What `group` occupied last time, if it has ever been drawn.
    #[must_use]
    pub fn get(&self, group: Group) -> Option<f32> {
        self.0.get(&group).copied()
    }
}

/// How much room a separator between two groups needs, in points.
const SEPARATOR_PTS: f32 = 6.0;

/// The room the bar's left half must leave: what the groups this module never
/// sheds occupied last frame, with the separator before them. Zero until they
/// have been measured.
#[must_use]
pub fn floor_width(widths: &Widths) -> f32 {
    let kept: Vec<Group> = Group::ORDER
        .into_iter()
        .filter(|g| still_reachable_at(*g).is_none())
        .collect();
    let width = measured_width(&kept, widths);
    if width > 0.0 {
        width + SEPARATOR_PTS
    } else {
        0.0
    }
}

/// **Which of the cluster's groups fit in `available` points.**
#[must_use]
pub fn affordable(available: f32, widths: &Widths) -> Vec<Group> {
    let mut shown: Vec<Group> = Group::ORDER.to_vec();
    if !available.is_finite() {
        return shown;
    }
    for (group, _) in SHED_ORDER {
        if measured_width(&shown, widths) <= available {
            break;
        }
        shown.retain(|g| g != group);
    }
    shown
}

/// What `shown` would occupy: the groups' own widths plus a separator between
/// each adjacent pair.
#[doc(hidden)]
pub fn measured_width(shown: &[Group], widths: &Widths) -> f32 {
    let known: Vec<f32> = shown.iter().filter_map(|g| widths.get(*g)).collect();
    // Separators are counted between MEASURED groups, not between shown ones.
    //
    // Counting them per shown group broke the bootstrap and the test caught it:
    // with nothing yet measured the sum is zero but four separators are 24 pt,
    // so a bar narrower than that shed two groups on its very first frame —
    // before either had ever been drawn, and therefore before either could
    // acquire the width that would have let it back. A rule that cannot
    // bootstrap is not a rule.
    let separators = SEPARATOR_PTS * (known.len().saturating_sub(1)) as f32;
    known.iter().sum::<f32>() + separators
}

/// Where a shed group is still reachable, if this module may shed it.
#[must_use]
pub fn still_reachable_at(group: Group) -> Option<&'static str> {
    SHED_ORDER
        .iter()
        .find(|(g, _)| *g == group)
        .map(|(_, command)| *command)
}

/// **Say what the bar dropped, and where it still is.**
pub fn trace_shed(shown: &[Group]) {
    let dropped: Vec<Group> = Group::ORDER
        .into_iter()
        .filter(|g| !shown.contains(g))
        .collect();
    crate::diag::trace_changed(SHED_SLOT, move || {
        if dropped.is_empty() {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            return "status-shed groups=none".to_owned();
        }
        let named: Vec<String> = dropped
            .iter()
            .map(|g| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "{}@{}",
                    g.region(),
                    still_reachable_at(*g).unwrap_or("nowhere")
                )
            })
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("status-shed groups={}", named.join(","))
    });
}

/// The de-duplication slot for [`trace_shed`]. A line per frame at sixty hertz
/// is not a diagnostic.
const SHED_SLOT: &str = "status-shed"; // ui-text-exempt: a trace slot key, never displayed
