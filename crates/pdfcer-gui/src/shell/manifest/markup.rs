//! The **Markup** tab — *what am I adding for someone else to read?*
//!
//! `RIBBON_IA.md` §5.5. Five groups: Shapes, Text markup, Notes, Style,
//! Comments.
//!
//! # Why it is not called Review
//!
//! What lives here is markup *authoring* — shapes, notes, stamps.
//! "Review" promises a review *workflow*: compare revisions, resolve
//! comments, track changes. pdfcer does not have that yet, and when it does
//! it will want the name. `Markup` is also the term this project's
//! audience uses; Bluebeam and every drafting office call it that.
//!
//! Note that the *mode* called Review is a different thing and the
//! collision is deliberate rather than accidental: Review mode is the
//! stance in which a reviewer works, and this tab is one of the five it
//! contains.
//!
//! # The Style group sets the style of the *next* markup
//!
//! Not of the selected one. Changing an existing markup's style happens on
//! the contextual Format tab, and `RIBBON_IA.md` §5.5 is explicit that
//! both surfaces must exist — *"today only the first does, which is why a
//! placed markup feels final."*
//!
//! Colour is the only style property with a control today, and it is not a
//! button: it is a swatch that opens a colour picker. That is what
//! `egui_shell::manifest::Item::Custom` is for. The shell reserves the
//! space and hands the `kind` back; it draws nothing and interprets
//! nothing. Modelling the swatch as a `Command` would have meant either
//! lying about what the control is or growing the framework a
//! `ColourSwatch` item variant, which is the road by which a reusable
//! shell stops being reusable.
//!
//! Line width, fill and opacity are **N** and join the swatch when they
//! exist.
//!
//! # ★ ONE of ten markup kinds is missing, and it is the one that matters most
//!
//! **Cloud** — revision clouds. It is the one this audience will name first: it
//! is AEC table stakes, and it is the only markup kind blocked on the **engine**
//! (`MarkupSpec::Cloud`, accepted 2026-08-14, not started). It is in
//! [`super::PLANNED`], alongside `markup.line`, which is a *style* of the
//! existing Arrow rather than a kind.
//!
//! ## The count, and what each correction to it taught
//!
//! This paragraph said **six** until 2026-08-14, then **three**, and now one.
//! Both corrections are kept rather than deleted, because the shape of the
//! unblocking is the useful part and it was the same shape twice:
//!
//! > **the count that mattered was never the count of kinds, it was the count of
//! > gestures.**
//!
//! * **Underline, StrikeOut and Squiggly** left first. Their blocker was never
//!   the engine — all three have authored appearance streams since Pass 6.1 —
//!   but the *gesture*: they mark text, and this shell had no way to select any.
//!   `canvas::textsel` closed that and `canvas::markup::text` spent it.
//! * **Ink, Polygon and PolyLine** left second, later the same day, and their
//!   blocker was the same word for a different reason: they were *"engine-ready
//!   but not drag-shaped"*, so the two-point rubber band could not express them.
//!   `canvas::markup::vertex` gave the two click-shaped kinds a gesture with two
//!   endings, and `canvas::markup::ink` gave the freehand one a trail. Again no
//!   engine change; again a gesture.
//!
//! What is left after both is the one kind whose blocker really *is* the
//! engine — which is the distinction this section existed to make and had been
//! obscuring by counting all four together.
//!
//! `RIBBON_IA.md` §5.5 lists `Line` among the four **G** shapes. The
//! shipped build has four markup kinds — Rectangle, Ellipse, `Arrow line`
//! and `Highlight band` — and no plain line: the parenthetical *"(Arrow is
//! `Arrow line`)"* resolves which of the two the existing control is, and
//! the answer is the arrow. A plain line is therefore **N** and is in
//! PLANNED rather than emitted, which is the conservative reading and the
//! one P3 requires: a button that arms a tool that does not exist is
//! exactly the placeholder the rule forbids.

use super::{command, group, icon_only, large};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The Markup tab.
pub(super) fn tab() -> Tab {
    Tab::new("markup", ribbon::tab_markup())
        .with_question(ribbon::question_markup())
        .with_groups([
            // ---------------------------------------------------------------
            // Shapes.
            //
            // ★ **Six kinds and an ending as of 2026-08-14**, where this band
            // held three and the module header explained why. Polyline, Polygon
            // and Ink moved out of `PLANNED` when the canvas grew the two
            // gestures they were waiting for; **Cloud** is the only shape still
            // absent, and it is the only one blocked on the engine.
            //
            // `markup.finish` sits **with the tools rather than in a group of
            // its own**, and the argument is `manifest::measure`'s for
            // `measure.finish`, applied to the second tab with the same problem:
            // it is not a seventh tool — it arms nothing — but a group of its own
            // for one command would spend a caption and a divider on a control
            // that is greyed whenever the operator is not mid-run. It reads
            // correctly in place, too: the group becomes "which shape, and when
            // this one is done", with the one entry that is not a tool last in
            // the row rather than lost among them.
            //
            // The order is the order an operator meets them: the three shapes
            // whose gesture is one drag, then the two whose gesture is a run of
            // clicks, then the one that follows the pointer, then the ending that
            // belongs to the middle pair.
            // ---------------------------------------------------------------
            group(
                "shapes",
                ribbon::group_markup_shapes(),
                [
                    icon_only("markup.rectangle"),
                    icon_only("markup.ellipse"),
                    icon_only("markup.arrow"),
                    icon_only("markup.polyline"),
                    icon_only("markup.polygon"),
                    // ★ Directly after Polygon, and that placement is the
                    // teaching. A revision cloud IS a polygon with a cloudy
                    // border — `/Subtype /Polygon` plus `/BE`, Table 181 — and
                    // an operator who has just learned that Polygon is "click
                    // each corner, double-click the last" needs to learn
                    // nothing else to use this. Putting it at the end of the
                    // band, after Freehand, would separate the pair that
                    // explains each other.
                    icon_only("markup.cloud"),
                    icon_only("markup.ink"),
                    command("markup.finish"),
                ],
            ),
            // ---------------------------------------------------------------
            // Text markup — markup that attaches to words already on the
            // page.
            //
            // ★ **Four controls as of 2026-08-14**, where this band held one
            // and its comment explained why. Underline, Strikeout and
            // Squiggly moved out of `PLANNED` when the canvas gained a
            // text-selection gesture, which was their only blocker.
            //
            // Splitting Highlight out of Shapes was right then and is right
            // now, for a reason the four controls together make plain: a
            // highlight is dragged across text and a rectangle is dragged
            // across space. **Highlight is still the odd one here** — it is
            // the only member of this band that is a drag rather than a mark
            // on the selection — and it stays because what it marks is text,
            // which is what the caption says. `canvas::markup::text` §3
            // records what a selection-highlight would take and why that is
            // the operator's taxonomy call rather than this file's.
            // ---------------------------------------------------------------
            group(
                "text_markup",
                ribbon::group_markup_text(),
                [
                    icon_only("markup.highlight"),
                    icon_only("markup.underline"),
                    icon_only("markup.strikeout"),
                    icon_only("markup.squiggly"),
                ],
            ),
            // ---------------------------------------------------------------
            // Notes. `Callout` is **N**; the stamp control exists and
            // needs a gallery, which is a change to the control rather
            // than a new command.
            // ---------------------------------------------------------------
            group(
                "notes",
                ribbon::group_markup_notes(),
                [
                    // ★ All three large, 2026-09-04 — the mockup's Notes
                    // group is three big controls and nothing else, so the
                    // whole group is promoted and no order changes.
                    large("markup.text_box"),
                    large("markup.sticky_note"),
                    large("markup.stamp"),
                ],
            ),
            // ---------------------------------------------------------------
            // Style — see the module header on why this is a Custom item
            // rather than a command.
            // ---------------------------------------------------------------
            group(
                "style",
                ribbon::group_markup_style(),
                [Item::custom(super::COLOUR_SWATCH)],
            ),
            // ---------------------------------------------------------------
            // Comments.
            //
            // `RIBBON_IA.md` §5.2 also lists a `Comments` entry under
            // View ▸ Panels. It cannot be in both places — one command,
            // one tab — and §7's migration map settles it explicitly:
            // `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`. Here.
            //
            // `Clear page` and `Clear all` are **N**.
            // ---------------------------------------------------------------
            group(
                "comments",
                ribbon::group_markup_comments(),
                [large("markup.comments")],
            ),
        ])
}
