//! # `shell::commands::catalog::measure` — the Measure tab — ce dimensions and the scale they are read at
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
        // ★★★ **THE FIVE-WAY SHARE OF `measure` ENDED 2026-09-04**, with art
        // adopted from the outside review of 2026-09-03. Read this before the
        // per-command notes below, because each of those notes used to argue
        // FOR the share and each has been rewritten.
        //
        // Until now `measure.linear`, `measure.radius_diameter`,
        // `measure.perimeter`, `measure.length` and `measure.two_line` all drew
        // [`crate::icons::Icon::Measure`]'s ruler. Five commands, one ribbon
        // group, drawn side by side, all one picture: an operator scanning the
        // Measure tab saw a row of identical tiles and had to READ five labels
        // to tell a radius from a pipe run. That is precisely the failure the
        // text-markup pass refused when it declined to reuse `shape-highlight`
        // for underline, strikeout and squiggly, and precisely the failure the
        // form-field group is having fixed in `catalog::edit` in the same pass.
        // The Measure tab was the worst instance in the build, because five is
        // more than four and because these are the tab's whole purpose.
        //
        // ★★ **The argument that held the share together was sound and is now
        // spent.** It ran: *all of these place a dimension, what differs is what
        // they measure FROM, and four near-identical rulers would make the group
        // harder to read rather than easier.* Both halves were true of the
        // catalogue as it stood. Nothing then in the set said "arc", "closed
        // route" or "angle" without redrawing a ruler, so the choice really was
        // between one shared ruler and several confusable ones, and one shared
        // ruler was the better of two bad answers.
        //
        // What changed is the premise, not the reasoning: the four new assets
        // are not rulers at all. Each draws the GEOMETRY it measures — a circle
        // with a spoke, an open curve between terminator ticks, a dashed closed
        // route, two lines with an arc swept across the corner — so the group is
        // now four visibly different figures rather than four attempts at the
        // same one. The old note's fear was of near-identical art, and the way
        // to honour it is to check the art is not near-identical, which the
        // variant docs in `crate::icons::catalog` do pairwise and in writing.
        //
        // ⇒ Same shape this project keeps finding, recorded rather than quietly
        // reversed: **a reason that was true when written, is checked by nobody,
        // and outlives what made it true.** The refusal was correct on the day;
        // leaving it standing after the assets arrived would have been the
        // defect.
        //
        // `measure` itself stays HERE, with Linear, and keeps its owner: a ruler
        // with a graduated ladder inside a band is a straight measured span
        // between two picked points, which is exactly what this command does.
        // The four below moved away from it; it did not move.
        command("measure.linear", t::measure_linear(), 600)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // ★ **Radius / diameter** — a closed circle, a spoke to the rim, a dot
        // on the centre: the drafting convention for a radius dimension, and one
        // glyph for both readings because the two are one stored geometry at two
        // scales (decision 011, `diameter = 2 x radius`) rather than two
        // measurements.
        //
        // The variant doc names the neighbour it must survive and it is not on
        // this tab: [`crate::icons::Icon::ShapeEllipse`], whose bare circle is
        // this one's outer ring to within a unit. The ring therefore carries no
        // difference at all and both cues are interior — the centre dot, which a
        // markup ellipse has no reason to draw, and the spoke, which is the
        // radius itself and also stops dot-inside-ring reading as a radio
        // button. Do not "clean up" either one: the tile is a radio button
        // without them.
        command("measure.radius_diameter", t::measure_radius_diameter(), 601)
            .with_icon("measure-radius")
            .enabled_when("doc.pages"),
        // ★ **Perimeter** - the operator's ask of 2026-08-20.
        //
        // Its glyph is an irregular CLOSED quadrilateral drawn DASHED, and both
        // halves are load-bearing. Closed is the word on the label — this is a
        // separate control from Length precisely because "Perimeter" promises a
        // ring — so closure is what separates it from Length's open run, which
        // sits directly beside it and is meant to be read as the contrast.
        //
        // Dashed separates it from [`crate::icons::Icon::ShapePolygon`], the
        // solid irregular closed outline on the Markup tab. Solid means "an
        // annotation you author by clicking corners"; dashed means "a route
        // traced round something already on the page". A measurement path is not
        // ink the document keeps. At 16 px the broken line is the ONLY cue that
        // says so — the corner count, four against five, is not legible at that
        // size — so the dash may not be dropped for optical weight. It is also
        // the only asset in the set using `stroke-dasharray`; `icons::svg`
        // parses that attribute for this pair and the font pair, and nothing
        // else.
        command("measure.perimeter", t::measure_perimeter(), 604)
            .with_icon("measure-perimeter")
            .enabled_when("doc.pages"),
        // ★ **Length**, 2026-08-20 - the same gesture that never closes. It is
        // a separate control rather than an option on Perimeter because
        // "Perimeter" says CLOSED, and nobody measuring a pipe run would reach
        // for it. See `MeasureKind::PathLength` for the argument.
        //
        // Now a meandering OPEN run with a short upright tick standing off each
        // end: *this thing, from here to here, is how long* — the sentence the
        // tooltip makes about a pipe, a cable or a kerb line. Open where
        // Perimeter closes, and the two sit adjacent so that contrast is the
        // first thing an operator sees; curved where
        // [`crate::icons::Icon::ShapePolyline`] is angular, because vertices are
        // what that glyph is about and a cable run has none.
        //
        // ★ Its dangerous neighbour is [`crate::icons::Icon::ShapeInk`] — one
        // irregular flowing stroke spanning the tile, no baseline, no
        // periodicity, which describes both glyphs exactly. **The entire
        // difference is the two terminator ticks.** Freehand ink has no ends
        // worth marking; a measured run is bounded, and the ticks sit outside
        // the curve's own endpoints the way real extension lines do, so they
        // read as measurement furniture rather than stray marks. Removing them
        // does not simplify this icon, it turns it into a different one.
        command("measure.length", t::measure_length(), 605)
            .with_icon("measure-length")
            .enabled_when("doc.pages"),
        // Registered as part of Phase 7, moving out of `manifest::PLANNED`.
        //
        // ★ Two straight lines meeting at a vertex with a small arc swept across
        // the corner: the drafting convention for an angular dimension, and a
        // picture of the gesture — pick one line, pick a second, and
        // `pdfcer_core::dimension::author_from_two_lines` places whichever
        // dimension the geometry calls for.
        //
        // ★★ **The arc draws only half of what this tool does, and the variant
        // doc says so deliberately.** The tool is linear between parallels and
        // angular between lines that meet. Drawing the parallel case would mean
        // two parallels with a dimension across them, which is
        // [`crate::icons::Icon::Measure`]'s job and would put a near-identical
        // band back into this group — the exact outcome the share above was
        // dissolved to escape. The angled case is the one with a shape of its
        // own and the one nothing else on the tab can express, so it is the one
        // drawn. Agree with that reading rather than "fixing" the icon to cover
        // both: covering both is how the group became one picture in the first
        // place.
        //
        // Apart from [`crate::icons::Icon::ShapePolyline`], the other
        // bare-strokes-at-vertices glyph, by the arc (nothing in the shapes
        // family draws one) and by having ONE vertex where that one needs three
        // to read as a chain. Apart from [`crate::icons::Icon::ShapeArrow`], the
        // other single-vertex glyph, because its second mark is a chevron head
        // and this one's is a curve.
        command("measure.two_line", t::measure_two_line(), 602)
            .with_icon("measure-angle")
            .enabled_when("doc.pages"),
        // ★ **Finish** — the ribbon half of the radius/diameter tool's ending.
        //
        // The radius/diameter gesture is the only one on this tab with no
        // natural end: Linear finishes at three clicks and Two-line at two,
        // because both are picks of a known arity, and a best-fit circle is
        // finished when the operator says it is. A double-click on the canvas
        // is the other half of the answer and is the one most operators will
        // use; this is the discoverable one, and the one that works when the
        // last picked arc is somewhere awkward to double-click.
        //
        // # Why `measure.finishable` and not `doc.pages`
        //
        // Because a Finish that is always enabled is a control that does
        // nothing on almost every press, and P3 reserves greying for
        // *temporarily unavailable* — which is exactly what this is. The
        // predicate is the same question the arm asks
        // (`canvas::measure::finishable`, one derivation shared with
        // `canvas::measure::finish`), so the control is live precisely when
        // pressing it would author a dimension: the circular tool armed, a pick
        // set on the page, and a fit that is not degenerate. Two picked arcs on
        // a straight line leave it greyed, correctly — there is no circle in
        // them to commit.
        //
        // # The icon refusal that stood here is DISCHARGED — 2026-09-04
        //
        // This registration and `catalog::markup`'s `markup.finish` carried the
        // same sentence, word for word, in two files: *"There is no check-mark,
        // tick or accept glyph in the set, and no existing key means 'complete
        // this gesture'."* That was a true statement about the catalogue and it
        // is no longer true — `check` was adopted from the outside review of
        // 2026-09-03 — so the refusal is **spent rather than overturned**, the
        // same way [`crate::icons::Icon::Pages`] and the Attachments paperclip
        // record. The note is rewritten rather than deleted because the reader
        // who finds a bare `.with_icon("check")` here with no history will
        // eventually re-derive the wrong lesson from it.
        //
        // What in it was load-bearing and still is:
        //
        // * **Reusing `measure` would have been wrong, and still would be.** The
        //   family's glyphs mean "this places a dimension"; this command places
        //   nothing, it ends the placing. That argument survived the share above
        //   being dissolved and is why Finish did not simply inherit whichever
        //   glyph its tool wears.
        // * **Naming a key that does not exist draws a visible slashed mark**, a
        //   placeholder arriving through the back door. Every key added in this
        //   pass was checked against `Icon::name` in `icons::catalog::mapping`
        //   before it was written down.
        // * A completion verb rendering as its word was an honest fallback, not
        //   a defect — which is why the button was shippable in the meantime.
        //
        // ★ **Why `check` here and `finish-shape` on `markup.finish`.** The
        // review supplied two candidate glyphs for these two near-identical
        // commands: `check`, a bare asymmetric tick, and `finish-shape`, a
        // vertex run with a tick appended. They must not share — that is the
        // whole lesson of the five-way `measure` share above, and two commands
        // whose labels both read "Finish" are the last pair that could afford
        // one picture. The split is decided and recorded so nobody re-litigates
        // it:
        //
        // > **A measurement's finish accepts a RESULT; a markup's finish closes
        // > a FIGURE.** What this command commits is a readout — a best-fit
        // > circle's radius, a number that goes on the page — and there is no
        // > shape being closed at all, so a glyph depicting a vertex run would
        // > describe the wrong operation. `finish-shape` shows a run of vertices
        // > with the tick appended, which is literally what Polyline and Polygon
        // > are doing and literally not what this is. The bare tick means "the
        // > pick set is accepted, author the dimension from it", which is the
        // > whole of it.
        //
        // The set's one-asset-per-role rule therefore holds with both commands
        // iconed and neither borrowing.
        //
        // The tick's two limbs are deliberately UNEQUAL — a short down-stroke
        // and a long up-stroke — because evened out it reads as
        // [`crate::icons::Icon::ChevronRight`] or a bare `>` at 16 px. The
        // asymmetry is the cue, not styling. And it could not have been a text
        // glyph in any case: `icons::glyphs` measures `✓` U+2713 **absent** from
        // the shipped font stack, so this concept had no character fallback —
        // which is the deeper reason the refusal could not be worked around and
        // had to wait for an asset.
        command("measure.finish", t::measure_finish(), 603)
            .with_icon("check")
            .enabled_when("measure.finishable"),
        // `set-scale` is the conversion glyph the icon ui-spec §8.2 assigned
        // — two arrows chasing each other round a circle. Deliberately not a
        // third `measure`: this command measures nothing, it changes what
        // measurements are read against.
        command("measure.set_scale", t::measure_set_scale(), 610)
            .with_icon("set-scale")
            .enabled_when("doc.pages"),
        // §8.2 also assigned `icon-ring.svg` here, and that half remains a
        // **recorded deviation**: two concentric circles read as a target or
        // a radio button at 16 px, not as a list of named things. The row was
        // written at reservation depth before the Measure surface existed and
        // states no reasoning to weigh against. That deviation is unchanged and
        // still stands — what changed on 2026-09-04 is the substitute.
        //
        // ★ **The borrow of `list` ended**, and what it was costing is worth
        // stating because the borrow was defensible right up until it was not.
        // `list` — [`crate::icons::Icon::ManageList`], three equal rules — is a
        // glyph of ACTION rather than of SUBJECT: it says "here is a set of
        // named things to manage", and it was shared with
        // `edit.form_manage_fields` on exactly that footing. The cost was that
        // the button on the Measure tab and the button on the Edit tab were the
        // same picture, so the tile said "manage a list" and only the label said
        // WHICH list — on a tab whose entire subject is dimensions. The reason
        // given for the share was that "dimension groups" was a phrase only a
        // label could say. `dimension-groups` says it.
        //
        // Two stacked dimension lines, each a rule capped at both ends by an
        // upright extension tick, the lower shorter and resuming past its
        // right-hand tick as a detached stub. A rule terminated by an extension
        // tick at each end is a dimension and nothing else; two stacked is a set
        // of them; and a set carrying one scale, one number format and one
        // drafting standard is exactly what a dimension group is.
        //
        // Three separations the variant doc names, all worth keeping:
        // from [`crate::icons::Icon::Measure`] by having NO enclosing band and
        // NO graduated ladder (a ruler is subdivided where a dimension is
        // terminated); from `list`'s three equal rules by the UNEQUAL row
        // lengths, with the stub past the lower tick doing the "and more" work
        // that makes two rows read as a list rather than as exactly two; and
        // from `measure-length` two rows above — whose ticks also flank a run —
        // because that run is single and curved, a measurement being TAKEN
        // rather than measurements LISTED. That last one is the near pair to
        // watch, since both now live on this tab.
        //
        // The panel this opens (`crate::panels::Panel::DimensionGroups`) wears
        // the same key, so the tab and the button that raises it agree.
        command("measure.manage_groups", t::measure_manage_groups(), 611)
            .with_icon("dimension-groups")
            .enabled_when("doc.open"),
    ]
}
