//! The canvas object menu's **Redact selection** row, and the bounds it asks for.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/redact_menu.md`.

/// The shell's line for the verb, carrying `quads=` and `bbox=`.
pub(super) const REQUESTED: &str = "redact-mark-selection-requested"; // ui-text-exempt: a trace event name

/// How far outside another rectangle a rectangle may fall and still be inside
/// it, in points.
const CONTAINMENT_SLACK_PT: f64 = 1.0;

/// A rectangle read off a `bbox=` field, in PDF user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bounds {
    llx: f64,
    lly: f64,
    urx: f64,
    ury: f64,
}

impl Bounds {
    /// Parse `llx,lly,urx,ury`. `None` for `bbox=none`, a short field or any
    /// component that is not a number — all of which mean the same thing to
    /// every caller: **this line cannot say what was marked.**
    pub(super) fn parse(field: &str) -> Option<Self> {
        let mut parts = field.split(',');
        let mut next = || parts.next()?.parse::<f64>().ok();
        let (llx, lly, urx, ury) = (next()?, next()?, next()?, next()?);
        if parts.next().is_some() {
            return None;
        }
        Some(Self { llx, lly, urx, ury })
    }

    pub(super) fn height(self) -> f64 {
        self.ury - self.lly
    }

    /// Whether `self` lies inside `outer`, allowing [`CONTAINMENT_SLACK_PT`].
    pub(super) fn inside(self, outer: Self) -> bool {
        self.llx >= outer.llx - CONTAINMENT_SLACK_PT
            && self.lly >= outer.lly - CONTAINMENT_SLACK_PT
            && self.urx <= outer.urx + CONTAINMENT_SLACK_PT
            && self.ury <= outer.ury + CONTAINMENT_SLACK_PT
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.1},{:.1},{:.1},{:.1}",
            self.llx, self.lly, self.urx, self.ury
        )
    }
}

/// What one press of the redact row asked the document for.
#[derive(Debug, Clone, Copy)]
pub(super) struct Marked {
    /// How many separate regions the verb built — `quads=` on [`REQUESTED`].
    ///
    /// **The design's unit of disclosure.** One gesture over a set of lines is
    /// one mark holding one region per line, so this number is what separates
    /// a build that honours the set from one that unioned it into a single
    /// rectangle over text the operator never selected.
    pub(super) quads: usize,
    /// The union of those regions — `bbox=` on the same line.
    pub(super) bbox: Bounds,
}
