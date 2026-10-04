//! The **Markup** tab — *what am I adding for someone else to read?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/markup.md`.

use super::{command, group, icon_only, large};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The Markup tab.
pub fn tab() -> Tab {
    Tab::new("markup", ribbon::tab_markup())
        .with_question(ribbon::question_markup())
        .with_groups([
            // ---------------------------------------------------------------
            // Shapes.
            //
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
                    // Directly after Polygon, and that placement is the
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
            // than a new command. The three most-used are large; the rest
            // stack in two columns so the group still fits a 1,400-wide
            // window without collapsing.
            // ---------------------------------------------------------------
            group(
                "notes",
                ribbon::group_markup_notes(),
                [
                    large("markup.text_box"),
                    large("markup.sticky_note"),
                    large("markup.stamp"),
                    command("markup.attach_file"),
                    command("markup.sound"),
                    command("markup.paste_image_stamp"),
                    command("markup.insert_text"),
                    command("markup.replace_text"),
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
            //
            // ## An amendment to `RIBBON_IA.md` §5.5, to be reflected back
            //
            // §5.5 names five groups — Shapes, Text markup, Notes, Style,
            // Comments — and this is a **sixth**. `RIBBON_IA.md` is a settled
            // spec and the standing rule is to propose rather than improvise;
            // the operator's directive of 2026-09-06 suspends that for this work
            // by name (*"never ask — placement, wording and scope are yours"*),
            // so the group is placed and the reasoning is recorded HERE for the
            // doc edit to be made from. **Do not read the absence of a §5.5 row
            // as this group being unplanned.**
            //
            // ## Why a group of its own rather than four rows in Style
            //
            // Because Style and Arrange answer opposite questions about tense.
            // This module's header states §5.5's own rule: *"the Style group sets
            // the style of the NEXT markup. Not of the selected one."* Style is a
            // pen. These four act on a mark that is **already placed**, are drawn
            // only when one is selected, and reach a different engine verb
            // (`reorder_annotations`, which permutes the page's `/Annots`, not
            // `set_markup_style`). Folding them into Style would put a live
            // control and a greyed one under one caption whose word describes
            // neither.
            //
            // ## Why HERE — after Style, before Comments
            //
            // The tab reads left to right as a sequence of tenses, and this is
            // the seam between the two halves:
            //
            //   Shapes · Text markup · Notes   what I am about to add
            //   Style                          how the next one will look
            //   ARRANGE                        what I have already added
            //   Comments                       what everyone has added
            //
            // Placing it before Style would split the three authoring groups from
            // the pen that governs them; placing it after Comments would put a
            // selection-scoped group after a document-scoped panel toggle, which
            // is the widening the order otherwise never reverses.
            //
            // ## Why the four are labelled and not icon-only
            //
            // `catalog::arrange`'s header carries the icon refusal in full — there
            // is no front/back glyph in the set and the two near-misses mean
            // *move this up the page list*. Given no glyphs, `icon_only` would
            // fall back to labels anyway (`sizing::resolved`), so asking for it
            // would be a request the shell silently declines: a line of code that
            // states an intention the build does not honour. `command` says what
            // actually happens.
            //
            // ## Two rows, not four
            //
            // `group_two_rows` — O97's shape (*"our display buttons should be on
            // two rows to save space"*). Four labelled controls in one row is the
            // widest group on the tab by some margin; asking for two lets the
            // packer find a narrower shape.
            //
            // A **hint**, not a layout: `plan::rows_for` reads it as *"skip the
            // fits-already short-circuit and search"*, then returns the narrowest
            // packing within the band's row ceiling — so this asks for a block
            // and does not dictate one. The packing is greedy in item order, so
            // the ordering below is what decides which controls share a row: the
            // two that move a mark **forward** come first and the two that move
            // it **back** follow, which is the reading that survives whichever
            // shape the packer picks.
            // ---------------------------------------------------------------
            super::group_two_rows(
                "arrange",
                crate::text::arrange::group_arrange(),
                [
                    command("markup.bring_to_front"),
                    command("markup.bring_forward"),
                    command("markup.send_backward"),
                    command("markup.send_to_back"),
                ],
            ),
            // ---------------------------------------------------------------
            // Comments.
            //
            // `RIBBON_IA.md` §5.2 also lists a `Comments` entry under
            // View ▸ Panels. It cannot be in both places — one command,
            // one tab — and §7's migration map settles it explicitly:
            // `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`. Here.
            //
            // `Make all part of the page` burns every markup on the page in
            // view: "what everyone has added", acted on at once. `Clear page`
            // and `Clear all` are **N**.
            // ---------------------------------------------------------------
            group(
                "comments",
                ribbon::group_markup_comments(),
                [large("markup.comments"), command("markup.flatten_page")],
            ),
        ])
}
