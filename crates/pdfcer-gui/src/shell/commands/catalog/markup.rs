//! # `shell::commands::catalog::markup` — the Markup tab — what is added for somebody else to read
//!
//! One band of [`super::all`]'s catalogue. Split out of [`super`] under **R2**
//! on 2026-08-28, when the Attachments command took that file to 1,495 of its
//! 1,500 lines and the next command registered would have broken the rule.
//!
//! ## ★★★ The split is per TAB, and the reason it was refused before is gone
//!
//! [`super`]'s header argued against exactly this cut:
//!
//! > a per-tab split would put the handler-token blocks in eight files where a
//! > collision between two of them is invisible.
//!
//! **That objection was already false when it was written.**
//! `super::super::tests::every_handler_token_is_unique` sweeps the whole
//! registry, and `every_handler_token_is_in_its_tabs_block` asserts each token
//! sits in its own tab's hundred. A collision is not invisible — it is a red
//! test, in either arrangement — so the argument that kept 120 commands in one
//! file rested on a property two tests had already taken over.
//!
//! ⇒ Recorded rather than quietly reversed, because it is the same shape this
//! project keeps finding: **a reason that was true when written, is checked by
//! nobody, and outlives what made it true.**
//!
//! ## What is here, and what is not
//!
//! The `Command` entries and the argument for each one's label, tooltip,
//! handler token, icon and enable predicate. **The prose is the point** — most
//! of this file is the record of decisions that would otherwise be re-litigated,
//! which is also why the byte count grew past a limit in the first place.
//!
//! Not here: the registration itself ([`super::super::register`]), the
//! command-id-to-behaviour mapping ([`super::super::mapping`]), and the
//! reachability register ([`super::super::reach`]).

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This band's commands, in ribbon order.
pub(super) fn band() -> Vec<Command> {
    vec![
        command("markup.rectangle", t::markup_rectangle(), 500)
            .with_icon("shape-rect")
            .enabled_when("doc.pages"),
        command("markup.ellipse", t::markup_ellipse(), 501)
            .with_icon("shape-ellipse")
            .enabled_when("doc.pages"),
        command("markup.arrow", t::markup_arrow(), 502)
            .with_icon("shape-arrow")
            .enabled_when("doc.pages"),
        // ★ **The three unblocked Phase 6 kinds** — Phase 6, 2026-08-14, moving
        // out of `manifest::PLANNED`.
        //
        // `FEATURES.md` carried all three as *"engine-ready, but not drag-shaped;
        // each needs its own gesture"* for the whole project, and that is exactly
        // what they got: `canvas::markup::vertex` for the two click-shaped kinds
        // and `canvas::markup::ink` for the freehand one. Nothing about the
        // engine changed — `MarkupSpec::PolyLine`, `Polygon` and `Ink` have been
        // there since Pass 6.1.
        //
        // # `doc.pages`, like the four kinds above and unlike the three below
        //
        // These **arm a tool**; they do not act. That is the line the enable
        // predicate draws: a shape command is live wherever there is a page to
        // draw on, and a *mark* command is live only when there is a selection to
        // mark (`selection.text`). Getting that backwards would grey Polygon
        // until something unrelated was selected.
        //
        // # The icons are three new glyphs, and two of them are one idea
        //
        // `shape-polyline` is `shape-polygon` with its closing segment removed,
        // which is exactly how the two annotations differ (§12.5.6.13). Drawing
        // them as a pair is what makes the band teachable: an operator who learns
        // one has learned the other. `shape-ink` is deliberately *not* a reuse of
        // `text-squiggly` — that one is a periodic wave in a band under two text
        // lines and means "mark these words"; this one is an aperiodic full-tile
        // stroke and means "the path your hand took". See
        // `crate::icons::Icon::ShapePolyline` and its two siblings.
        command("markup.polyline", t::markup_polyline(), 503)
            .with_icon("shape-polyline")
            .enabled_when("doc.pages"),
        command("markup.polygon", t::markup_polygon(), 504)
            .with_icon("shape-polygon")
            .enabled_when("doc.pages"),
        // ★ **Revision cloud**, registered 2026-08-19 — the operator's item 6,
        // raised three times in his own words: *"still no revision cloud
        // tool."*
        //
        // Token **507**, out of the band's own run, because 506 is
        // `markup.finish` and tokens are never reused. The ORDER on the ribbon
        // is the manifest's, not the token's, so this sits between Polygon and
        // Freehand where it belongs — beside the tool it is a variant of.
        //
        // It was in `crate::shell::manifest::PLANNED` with the reason *"the
        // ONLY markup kind still absent for an ENGINE reason rather than a
        // gesture one"*, and that had quietly stopped being true:
        // `MarkupSpec::Cloud` shipped in `pdfcer-core` and nothing in this shell
        // noticed for weeks. A PLANNED entry is a claim about the world and it
        // decays; this one cost three weeks of the operator asking for a tool
        // whose only blocker had already been removed.
        command("markup.cloud", t::markup_cloud(), 507)
            .with_icon("shape-cloud")
            .enabled_when("doc.pages"),
        command("markup.ink", t::markup_ink(), 505)
            .with_icon("shape-ink")
            .enabled_when("doc.pages"),
        // ★ **Finish shape** — the ribbon half of the vertex tools' ending, and
        // `measure.finish`'s twin in every respect that matters.
        //
        // Polyline and Polygon are the only markup gestures with no natural end:
        // a band drag ends when the button comes up and a freehand stroke ends
        // the same way, but a run of clicks does not end itself. The operator
        // settled that shape of problem on 2026-08-14 for the radius/diameter
        // tool — **two endings through one commit path** — and this is that
        // answer applied to the second tool with the same problem, deliberately
        // rather than inventing a third. A double-click on the canvas is the
        // ending most operators will use; this is the discoverable one, and the
        // one that works when the last corner sits somewhere awkward to
        // double-click.
        //
        // # Why `markup.finishable` and not `doc.pages`
        //
        // Because a Finish that is always enabled is a control that does nothing
        // on almost every press, and P3 reserves greying for *temporarily
        // unavailable* — which is exactly what this is. The predicate is the same
        // question the arm asks (`canvas::markup::vertex::finishable`, one
        // derivation shared with `vertex::finish`), so the control is live
        // precisely when pressing it would author an annotation.
        //
        // It is also where the polygon/polyline difference becomes visible: a
        // polygon needs three vertices where a polyline needs two, so after two
        // clicks this control is live for one tool and greyed for the other. The
        // operator is told the rule before they press, rather than refused after.
        //
        // # No icon, and it is the same deliberate refusal `measure.finish` makes
        //
        // There is no check-mark, tick or accept glyph in the set, and no
        // existing key means "complete this gesture". Reusing one of the three
        // shape glyphs would draw a fourth near-identical shape in the same band
        // for a command that draws nothing — it *ends* the drawing — and would
        // undermine the pairing argument the polyline/polygon glyphs above rest
        // on. Naming a key that does not exist draws a visible slashed mark,
        // which is a placeholder arriving through the back door. So it renders as
        // its words, which for a completion verb is the clearest thing it could
        // be.
        command("markup.finish", t::markup_finish(), 506).enabled_when("markup.finishable"),
        command("markup.highlight", t::markup_highlight(), 510)
            .with_icon("shape-highlight")
            .enabled_when("doc.pages"),
        // ★ **The three text-markup kinds** — Phase 6, 2026-08-14, moving out of
        // `manifest::PLANNED`.
        //
        // # Why `selection.text` and not `doc.pages`
        //
        // Because these three do not arm a tool: they act **at once**, on the
        // text selection the operator has already made
        // (`canvas::markup::text` §1, which records that this is Acrobat's
        // model and why it was chosen over arm-then-sweep). A control gated on
        // `doc.pages` would therefore be live on every open document and would
        // do nothing on almost every press — which is what `RIBBON_IA.md` P3
        // forbids and what `measure.finish` set the precedent for answering with
        // a condition of its own.
        //
        // The predicate is the same question the dispatch arm asks, so the
        // control cannot be enabled while pressing it would decline: `conditions`
        // publishes `selection.text` from a **live** selection on the open
        // document, and `markup::text::mark` refuses anything else.
        //
        // # ★ Where they are reachable, which is narrower than the tab suggests
        //
        // **Review, and Review alone.** Read cannot author markup (its tab list
        // is File and View, so the Markup tab is not there at all), and Edit
        // cannot make a text selection (its primary button is the content
        // marquee — `canvas::textsel::takes_the_press`), so in Edit these three
        // are drawn and permanently greyed. That is an inversion, it is
        // recorded rather than smoothed over, and it closes the day
        // `CanvasTool::Text` lands. See `canvas::markup::text` §2.
        //
        // # The icons
        //
        // Three new glyphs rather than a reuse of `shape-highlight`: the four
        // controls in the Text markup band differ *only* in the mark they draw,
        // so a shared glyph would make the band four identical buttons with
        // four different words — the exact opposite of the "family shares a
        // glyph" convention this module's header describes, which is for
        // commands whose difference is carried by the label.
        command("markup.underline", t::markup_underline(), 511)
            .with_icon("text-underline")
            .enabled_when("selection.text"),
        command("markup.strikeout", t::markup_strikeout(), 512)
            .with_icon("text-strikeout")
            .enabled_when("selection.text"),
        command("markup.squiggly", t::markup_squiggly(), 513)
            .with_icon("text-squiggly")
            .enabled_when("selection.text"),
        command("markup.text_box", t::markup_text_box(), 520)
            .with_icon("text-freetext")
            .enabled_when("doc.pages"),
        command("markup.sticky_note", t::markup_sticky_note(), 521)
            .with_icon("text-sticky")
            .enabled_when("doc.pages"),
        command("markup.stamp", t::markup_stamp(), 522)
            .with_icon("stamp")
            .enabled_when("doc.pages"),
        command("markup.comments", t::markup_comments(), 540)
            .with_icon("comment")
            .enabled_when("doc.open"),
    ]
}
