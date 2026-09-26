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
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/report.md`.

use egui::Rect;

/// The callback an application supplies to receive drawn rectangles.
pub type RectSink<'a> = dyn FnMut(&str, Rect) + 'a;

/// The name prefix every rect this module publishes begins with.
///
/// A harness can therefore filter the ribbon's reports out of a stream
/// that also carries a dock's or a status bar's.
pub const PREFIX: &str = "ribbon";

/// The trace event name under which a control publishes **whether it was drawn
/// pressable**.
pub const ENABLEMENT_EVENT: &str = "ribbon-item-enablement";

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
#[must_use]
pub fn group_collapsed(tab_id: &str, group_id: &str) -> String {
    format!("{PREFIX}.group.{tab_id}.{group_id}.collapsed")
}

/// The name under which one group's **caption** is published.
#[must_use]
pub fn group_caption(tab_id: &str, group_id: &str) -> String {
    format!("{PREFIX}.group.{tab_id}.{group_id}.caption")
}

/// **The auto-hide trigger** — the tab strip, taken as the rectangle whose
/// hover reveals a hidden band.
#[must_use]
pub fn auto_hide_trigger() -> String {
    format!("{PREFIX}.autohide.trigger")
}

/// **The revealed band's overlay rectangle**, on the frames auto-hide draws
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
#[must_use]
pub fn mode_segment(mode_id: &str) -> String {
    format!("{PREFIX}.mode.{mode_id}")
}

/// The name under which the **overflow affordance** is published.
pub const OVERFLOW: &str = "ribbon.overflow";

/// See [`OVERFLOW`].
#[must_use]
pub fn overflow() -> &'static str {
    "ribbon.overflow"
}

/// The name under which the **tab strip's** overflow affordance is
/// published.
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
#[must_use]
pub fn trailing_item(command_id: &str) -> String {
    format!("{PREFIX}.trailing.{command_id}")
}

/// The name under which one **band command control** — a button inside a
/// captioned group on the active tab — is published.
#[must_use]
pub fn band_item(command_id: &str) -> String {
    format!("{PREFIX}.item.{command_id}")
}

/// Holds the application's rect sink, if there is one.
pub struct Reporter<'a> {
    sink: Option<&'a mut RectSink<'a>>,
}

impl<'a> Reporter<'a> {
    /// A reporter that publishes to `sink`, or discards if it is `None`.
    pub fn new(sink: Option<&'a mut RectSink<'a>>) -> Self {
        Self { sink }
    }

    /// Whether anything is listening.
    pub fn is_listening(&self) -> bool {
        self.sink.is_some()
    }

    /// Publish `rect` under the name `name()` produces.
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

    /// **The published names are a stability contract, and this test is
    /// the tripwire on it.**
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
