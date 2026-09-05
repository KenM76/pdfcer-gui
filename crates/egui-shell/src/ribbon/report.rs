//! `ui_rect` reporting — where the ribbon says what it just drew, and
//! where.
//!
//! # The problem this solves
//!
//! A verification harness that wants to assert *"the mode selector's
//! Review segment is legible"* has to know where that segment is. There
//! are three ways it can find out, and two of them rot.
//!
//! 1. **Hard-code a fraction of the window.** *"The selector is in the
//!    right-hand 18% of the top 30 px."* This is what a screenshot-diffing
//!    harness usually ends up doing, and it is wrong the first time a
//!    fourth mode is added, a label is reworded, or the theme's padding
//!    changes. Worse, it goes wrong *silently*: the assertion still
//!    passes, it is just now measuring the wrong pixels.
//! 2. **Re-derive the layout in the harness.** Now there are two
//!    implementations of the ribbon's arithmetic and the harness is
//!    asserting that they agree with each other rather than that the
//!    application is correct.
//! 3. **Have the application publish what it drew.** The renderer already
//!    knows the rect — it just allocated it — and publishing it costs one
//!    call.
//!
//! This module is (3). Every group caption and every mode-selector
//! segment publishes its [`egui::Rect`] under a **stable name**, on the
//! frame it was drawn, through a callback the application supplies. A
//! harness then asserts against a rect that is true for the frame it is
//! looking at, rather than against a fraction that was true when someone
//! wrote it down.
//!
//! # Zero-cost when nobody is listening
//!
//! The sink is an `Option<&mut dyn FnMut>`. When it is `None`:
//!
//! - no rect is stored;
//! - **and no name is formatted**. Every name here is built inside a
//!   closure that is only called when a sink exists, because
//!   `format!("ribbon.group.{tab}.{group}.caption")` is an allocation,
//!   and one per group per frame in the paint loop is exactly the kind of
//!   cost that gets a diagnostic feature switched off.
//!
//! See [`Reporter::report`] for the shape that enforces it.
//!
//! # Why the names are a stability contract
//!
//! These strings are an API. A harness greps for
//! `ribbon.mode.review`; renaming it to `ribbon.modes.review` breaks
//! every assertion that used it, at a distance, in another repository.
//! `the_reported_names_are_a_stability_contract` pins the exact spellings
//! so that a rename has to be a deliberate act with a failing test in
//! front of it, rather than a tidy-up.

use egui::Rect;

/// The callback an application supplies to receive drawn rectangles.
///
/// Takes the name by reference so the ribbon never has to allocate a
/// `String` the sink might not keep, and the rect by value because it is
/// four `f32`.
pub type RectSink<'a> = dyn FnMut(&str, Rect) + 'a;

/// The name prefix every rect this module publishes begins with.
///
/// A harness can therefore filter the ribbon's reports out of a stream
/// that also carries a dock's or a status bar's.
pub const PREFIX: &str = "ribbon";

/// The name under which one ribbon **tab button** is published.
#[must_use]
pub fn tab(tab_id: &str) -> String {
    format!("{PREFIX}.tab.{tab_id}")
}

/// The name under which one **group** — controls and caption together —
/// is published.
#[must_use]
pub fn group(tab_id: &str, group_id: &str) -> String {
    format!("{PREFIX}.group.{tab_id}.{group_id}")
}

/// The name under which a **collapsed** group's single button is published.
///
/// ★ A distinct name from [`group`], deliberately. A collapsed group is on the
/// band and its items are not, which is a third state — the other two being
/// *expanded on the band* and *in the overflow menu* — and a driven check that
/// could not tell them apart would report a collapse as a disappearance. The
/// suffix means an existing check asserting `ribbon.group.<tab>.<id>` keeps
/// meaning exactly what it meant: **this group is drawn, expanded**.
#[must_use]
pub fn group_collapsed(tab_id: &str, group_id: &str) -> String {
    format!("{PREFIX}.group.{tab_id}.{group_id}.collapsed")
}

/// The name under which one group's **caption** is published.
///
/// This is the rect a legibility assertion wants: the caption is the
/// smallest text the ribbon draws, it is drawn `weak()` and `small()`,
/// and it is therefore the first thing to become unreadable under a
/// theme change or a scale change.
#[must_use]
pub fn group_caption(tab_id: &str, group_id: &str) -> String {
    format!("{PREFIX}.group.{tab_id}.{group_id}.caption")
}

/// **The auto-hide trigger** — the tab strip, taken as the rectangle whose
/// hover reveals a hidden band.
///
/// ★★ Published even when auto-hide is OFF, and that is deliberate. The
/// question a driven check asks of this region is *"is the way back to the
/// ribbon on screen and big enough to hit"*, and the honest answer has to be
/// available in both settings — otherwise the check can only run in the state
/// it is trying to prove safe. It is reported through the visibility-gated
/// channel like every other ribbon region, so a trigger that laid out off
/// screen publishes nothing.
#[must_use]
pub fn auto_hide_trigger() -> String {
    format!("{PREFIX}.autohide.trigger")
}

/// **The revealed band's overlay rectangle**, on the frames auto-hide draws
/// one.
///
/// Absent whenever the band is inline or hidden, which is what makes it an
/// oracle: `ribbon.band` exists in all three states and this exists in exactly
/// one.
#[must_use]
pub fn auto_hide_overlay() -> String {
    format!("{PREFIX}.autohide.overlay")
}

/// The name under which the whole **mode selector** is published.
#[must_use]
pub fn mode_selector() -> &'static str {
    "ribbon.modes"
}

/// The name under which one **mode-selector segment** is published.
///
/// `MODES_AND_PANELS.md` Part 1 requires that the selector *"render as a
/// real segmented control with all three labels visible — not a bare
/// track with a knob, where the available positions are invisible until
/// you drag."* A per-segment rect is what makes that assertable: a
/// harness can check that every mode in the manifest produced a segment
/// with a positive area, which a whole-control rect cannot distinguish
/// from a track.
#[must_use]
pub fn mode_segment(mode_id: &str) -> String {
    format!("{PREFIX}.mode.{mode_id}")
}

/// The name under which the **overflow affordance** is published.
///
/// Published so a harness can assert `MODES_AND_PANELS.md` Part 2's
/// failure mode #8 against a running window: at a width where groups are
/// hidden, this rect must exist and have a positive area.
pub const OVERFLOW: &str = "ribbon.overflow";

/// See [`OVERFLOW`].
#[must_use]
pub fn overflow() -> &'static str {
    "ribbon.overflow"
}

/// The name under which the **tab strip's** overflow affordance is
/// published.
///
/// Distinct from [`overflow`], which is the *band's*. Both exist on the
/// same ribbon at the same time and answer different questions — "which
/// groups of this tab are hidden" versus "which tabs are hidden" — so a
/// harness that could not tell them apart would assert about whichever one
/// happened to be published first.
///
/// Spelled under `ribbon.tabs.` rather than `ribbon.tab.` so it cannot be
/// mistaken for a tab whose id happens to be `overflow`: [`tab`] builds
/// `ribbon.tab.{id}`, and the two namespaces stay disjoint.
#[must_use]
pub fn tab_overflow() -> &'static str {
    "ribbon.tabs.overflow"
}

/// The name under which one **quick-access toolbar** control is
/// published.
#[must_use]
pub fn qat_item(command_id: &str) -> String {
    format!("{PREFIX}.qat.{command_id}")
}

/// The name under which one **trailing control** — a button at the far right
/// of the tab-strip row, past the mode selector — is published.
///
/// Its own name rather than sharing [`qat_item`], because the two regions are
/// at opposite ends of the row and a driven check that could not tell them
/// apart would pass on a build that had put the control in the wrong one.
#[must_use]
pub fn trailing_item(command_id: &str) -> String {
    format!("{PREFIX}.trailing.{command_id}")
}

/// The name under which one **band command control** — a button inside a
/// captioned group on the active tab — is published.
///
/// # Why this exists at all
///
/// Until this landed, the ribbon published a rect for every *group
/// caption*, every *tab*, every *mode segment* and every *QAT control*,
/// and **nothing for the forty controls an operator actually clicks**. So
/// no process outside the application could say where the `Rectangle`
/// button was, and therefore nothing outside the application could click
/// it and observe what happened.
///
/// That gap has a precise cost, and this crate has already paid it once.
/// The icon painter existed, was tested, and was never handed to the
/// ribbon — a defect invisible to every unit test in two crates, found
/// only by reading the width of a rect the *running* window declared (see
/// [`qat_item`]'s consumer, `tools/ui-verify`'s `qat_icons` check). A
/// control whose rect is unpublished is a control no such check can ever
/// be written for.
///
/// # The name: `ribbon.item.<command_id>`
///
/// Two decisions, both deliberate.
///
/// **`item`** is [`crate::manifest::Item`]'s own word. The band draws a
/// group's `Item`s, and `Item::Command` is the variant this reports; a
/// reader who greps the manifest for what a band contains finds the same
/// noun. It also stays clear of `ribbon.group.` — a *group* rect is the
/// block, a *caption* rect is its label, and an *item* rect is one control
/// inside it, so the three namespaces answer three different questions and
/// a filter for one cannot catch another.
///
/// **The command id alone**, with no tab or group segment, exactly as
/// [`qat_item`] spells a QAT control. A command id is unique in a
/// [`crate::commands::Registry`] — that is what a registry *is* — so the
/// name is already unambiguous, and it survives the one event that would
/// otherwise break every selector built on it: `SHELL_FRAMEWORK.md` §5
/// permits an operator to **move a command between groups**, and a name
/// carrying `.<tab>.<group>.` would change out from under a harness the
/// first time somebody reorganised their ribbon. The identity being
/// reported is the control's, not its current address.
///
/// # The one ambiguity this leaves, stated
///
/// A manifest that places the *same command id in two groups of one tab*
/// publishes two rects under one name, and a consumer keeping the last
/// occurrence per name would see them alternate. Nothing here forbids
/// that, because nothing here can: the manifest is the application's. It
/// is a manifest defect rather than a naming defect — the same command
/// twice on one tab is two controls the operator cannot tell apart — and
/// the alternative spelling would trade a visible ambiguity in a
/// diagnostic name for an invisible one in the ribbon itself.
///
/// # Zero cost when nobody is listening
///
/// Called once per drawn command per frame, which is the busiest reporting
/// site in this module — hence the closure discipline [`Reporter::report`]
/// documents. With no sink installed, no name is built.
#[must_use]
pub fn band_item(command_id: &str) -> String {
    format!("{PREFIX}.item.{command_id}")
}

/// Holds the application's rect sink, if there is one.
///
/// A struct rather than a bare `Option` so that [`Self::report`] can own
/// the "do not format the name unless someone is listening" rule in one
/// place. Every call site in the ribbon goes through it.
pub struct Reporter<'a> {
    sink: Option<&'a mut RectSink<'a>>,
}

impl<'a> Reporter<'a> {
    /// A reporter that publishes to `sink`, or discards if it is `None`.
    pub fn new(sink: Option<&'a mut RectSink<'a>>) -> Self {
        Self { sink }
    }

    /// Whether anything is listening.
    ///
    /// Callers use this only to skip work that is expensive *beyond* the
    /// name — computing a rect that is not otherwise needed, say.
    /// Formatting the name is already deferred by [`Self::report`].
    pub fn is_listening(&self) -> bool {
        self.sink.is_some()
    }

    /// Publish `rect` under the name `name()` produces.
    ///
    /// # Why the name is a closure
    ///
    /// Because it is an allocation, and this is called once per caption,
    /// per segment, per tab, per QAT item, **per frame**. At 60 fps with
    /// a seven-tab ribbon that is thousands of `String`s a second built
    /// to be dropped unread, which is how a diagnostic hook becomes
    /// something a profiler blames and someone deletes.
    ///
    /// With no sink installed this function is one `Option` test and a
    /// return; the closure is never called and no name is ever built.
    pub fn report(&mut self, rect: Rect, name: impl FnOnce() -> String) {
        if let Some(sink) = self.sink.as_deref_mut() {
            sink(&name(), rect);
        }
    }

    /// [`Self::report`] for a name that is already `'static`, so no
    /// closure is needed at the call site.
    pub fn report_static(&mut self, rect: Rect, name: &'static str) {
        if let Some(sink) = self.sink.as_deref_mut() {
            sink(name, rect);
        }
    }
}

impl std::fmt::Debug for Reporter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reporter")
            .field("listening", &self.sink.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ The published names are a stability contract, and this test is
    /// the tripwire on it.**
    ///
    /// These strings are consumed by a harness in another tool, possibly
    /// in another repository, by literal comparison. A rename is
    /// therefore a breaking change with no compiler to catch it: the
    /// harness keeps building, its assertions simply stop matching
    /// anything, and a test that matches nothing passes.
    ///
    /// Pinning the exact spellings makes a rename a deliberate act with a
    /// failing test in front of it. If this test is ever updated, the
    /// harness's selectors have to be updated in the same change.
    #[test]
    fn the_reported_names_are_a_stability_contract() {
        assert_eq!(tab("view"), "ribbon.tab.view");
        assert_eq!(
            group("view", "page_display"),
            "ribbon.group.view.page_display"
        );
        assert_eq!(
            group_caption("view", "page_display"),
            "ribbon.group.view.page_display.caption"
        );
        assert_eq!(mode_selector(), "ribbon.modes");
        assert_eq!(mode_segment("review"), "ribbon.mode.review");
        assert_eq!(overflow(), "ribbon.overflow");
        assert_eq!(tab_overflow(), "ribbon.tabs.overflow");
        assert_eq!(qat_item("file.open"), "ribbon.qat.file.open");
        assert_eq!(
            band_item("markup.rectangle"),
            "ribbon.item.markup.rectangle"
        );

        // A band control, a QAT control and a group are three different
        // things about the same command, and a harness filtering for one
        // must never catch another. `ribbon.item.` is disjoint from both
        // `ribbon.qat.` and `ribbon.group.`, in both directions.
        assert!(!band_item("file.open").starts_with("ribbon.qat."));
        assert!(!qat_item("file.open").starts_with("ribbon.item."));
        assert!(!band_item("file.open").starts_with("ribbon.group."));
        assert!(!band_item("view.zoom").starts_with("ribbon.tab."));

        // The two overflow affordances are different controls on the same
        // ribbon and must never be confused for one another, in either
        // direction: a harness filtering `ribbon.tab.` for tabs must not
        // catch the strip's affordance either.
        assert_ne!(overflow(), tab_overflow());
        assert!(!tab_overflow().starts_with("ribbon.tab."));
    }

    /// Every name begins with [`PREFIX`], so a harness can separate the
    /// ribbon's reports from any other surface's in one filter.
    #[test]
    fn every_name_carries_the_ribbon_prefix() {
        for name in [
            tab("t"),
            group("t", "g"),
            group_caption("t", "g"),
            mode_selector().to_owned(),
            mode_segment("m"),
            overflow().to_owned(),
            tab_overflow().to_owned(),
            qat_item("c"),
            band_item("c"),
        ] {
            assert!(
                name.starts_with(PREFIX),
                "`{name}` is not filterable as a ribbon report"
            );
        }
    }

    /// **A reporter with no sink never builds a name.**
    ///
    /// This is the zero-cost claim, and the only way to observe it is a
    /// side effect inside the closure that is supposed not to run. If
    /// this fails, every rect call site in the paint loop is allocating a
    /// `String` per frame to throw it away.
    #[test]
    fn a_reporter_with_no_sink_never_builds_a_name() {
        let mut built = 0_usize;
        let mut reporter = Reporter::new(None);
        assert!(!reporter.is_listening());
        reporter.report(Rect::ZERO, || {
            built += 1;
            "expensive".to_owned()
        });
        assert_eq!(
            built, 0,
            "the name closure ran with no sink installed, so every reporting \
             call site is paying for a string nobody reads"
        );
    }

    /// A reporter with a sink delivers the name and the rect.
    #[test]
    fn a_reporter_with_a_sink_delivers_what_was_drawn() {
        let mut seen: Vec<(String, Rect)> = Vec::new();
        {
            let mut sink = |name: &str, rect: Rect| seen.push((name.to_owned(), rect));
            let mut reporter = Reporter::new(Some(&mut sink));
            assert!(reporter.is_listening());
            let r = Rect::from_min_size(egui::pos2(3.0, 4.0), egui::vec2(10.0, 2.0));
            reporter.report(r, || group_caption("view", "window"));
            reporter.report_static(r, overflow());
        }
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].0, "ribbon.group.view.window.caption");
        assert_eq!(seen[1].0, "ribbon.overflow");
        assert_eq!(seen[0].1.width(), 10.0);
    }
}
