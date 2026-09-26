//! # `pageselection` — which pages the operator has picked
//!
//! A page selection, and the three-modifier click rule that builds it. Pure:
//! no `egui`, no document, no rendering. That is deliberate and it is what
//! makes the rule testable — the interesting part of a multi-select is the
//! *policy* (what does Shift extend from? what does a plain click discard?),
//! and the policy is the part that can be wrong in a way an operator would
//! notice.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/pageselection.md`.

use std::collections::BTreeSet;

/// Which pages are picked, and where a range would extend from.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PageSelection {
    /// The picked pages, 0-based, ascending by construction.
    pages: BTreeSet<usize>,
    /// The page a Shift+click would extend *from*, or `None` before the
    /// operator has named one.
    ///
    /// See the module header: moved by a plain click and by a Ctrl+click,
    /// never by a Shift+click.
    anchor: Option<usize>,
}

/// What a click on a tile asked for, beyond changing the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickOutcome {
    /// Whether the canvas should navigate to the clicked page.
    pub navigate: bool,
}

impl PageSelection {
    /// The picked pages, in document order.
    #[must_use]
    pub fn pages(&self) -> &BTreeSet<usize> {
        &self.pages
    }

    /// How many pages are picked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether nothing is picked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Whether `page` is picked.
    #[must_use]
    pub fn contains(&self, page: usize) -> bool {
        self.pages.contains(&page)
    }

    /// Pick nothing.
    pub fn clear(&mut self) {
        self.pages.clear();
        self.anchor = None;
    }

    /// **Apply a click on `page`, with the modifiers that came with it.**
    pub fn click(&mut self, page: usize, ctrl: bool, shift: bool) -> ClickOutcome {
        if ctrl {
            if !self.pages.remove(&page) {
                self.pages.insert(page);
            }
            // The anchor moves even when the click REMOVED the page: the
            // operator named this page deliberately, and a following
            // Shift+click means "from here". An anchor left on a page that
            // was deselected three clicks ago is the case that makes range
            // extension feel random.
            self.anchor = Some(page);
            return ClickOutcome { navigate: false };
        }
        if shift && let Some(anchor) = self.anchor {
            let (lo, hi) = if anchor <= page {
                (anchor, page)
            } else {
                (page, anchor)
            };
            // Replaces rather than unions, which is what makes an overshoot
            // correctable: Shift+click too far, Shift+click back, and the
            // range is the one you meant. A union would leave the overshoot
            // permanently selected with no gesture that removes it.
            self.pages = (lo..=hi).collect();
            // Deliberately NOT moved — see the module header.
            return ClickOutcome { navigate: false };
        }
        // Plain click, and Shift+click with no anchor to extend from: the
        // second is the first click of a session, and treating it as a plain
        // click is the only defined answer that leaves the operator somewhere
        // sensible.
        self.pages.clear();
        self.pages.insert(page);
        self.anchor = Some(page);
        ClickOutcome { navigate: true }
    }

    /// **Make a right-click's operand list agree with what was pointed at.**
    pub fn right_click(&mut self, page: usize) -> bool {
        if self.pages.contains(&page) {
            return false;
        }
        self.pages.clear();
        self.pages.insert(page);
        self.anchor = Some(page);
        true
    }

    /// Drop any picked page at or beyond `page_count`, and the anchor with
    /// it.
    pub fn retain_below(&mut self, page_count: usize) -> bool {
        let before = self.pages.len();
        self.pages.retain(|p| *p < page_count);
        if self.anchor.is_some_and(|a| a >= page_count) {
            self.anchor = None;
        }
        self.pages.len() != before
    }

    /// **Follow the picked pages across a reorder.**
    pub fn remap(&mut self, landed: &[usize]) {
        self.pages = self
            .pages
            .iter()
            .filter_map(|p| landed.get(*p).copied())
            .collect();
        self.anchor = self.anchor.and_then(|a| landed.get(a).copied());
    }
}
/// Parse `3`, `1-4`, `5,1-2` into zero-based indices.
pub fn parse_page_range(spec: &str, count: usize) -> Option<Vec<usize>> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((a, b)) => {
                let a: usize = a.trim().parse().ok()?;
                let b: usize = b.trim().parse().ok()?;
                if a == 0 || b == 0 || a > b || b > count {
                    return None;
                }
                out.extend((a - 1)..b);
            }
            None => {
                let n: usize = part.parse().ok()?;
                if n == 0 || n > count {
                    return None;
                }
                out.push(n - 1);
            }
        }
    }
    (!out.is_empty()).then_some(out)
}
