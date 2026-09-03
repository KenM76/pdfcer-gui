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
        command("measure.linear", t::measure_linear(), 600)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        command("measure.radius_diameter", t::measure_radius_diameter(), 601)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // ★ **Perimeter** - the operator's ask of 2026-08-20.
        //
        // Shares the `measure` glyph with its three neighbours for the reason
        // the note below gives about Two-line: all four place a dimension, and
        // what differs is what they measure FROM. Four near-identical rulers
        // would make the group harder to read, not easier, and the label is
        // where the distinction belongs.
        command("measure.perimeter", t::measure_perimeter(), 604)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // ★ **Length**, 2026-08-20 - the same gesture that never closes. It is
        // a separate control rather than an option on Perimeter because
        // "Perimeter" says CLOSED, and nobody measuring a pipe run would reach
        // for it. See `MeasureKind::PathLength` for the argument.
        command("measure.length", t::measure_length(), 605)
            .with_icon("measure")
            .enabled_when("doc.pages"),
        // Registered as part of Phase 7, moving out of `manifest::PLANNED`.
        //
        // It shares the `measure` glyph with Linear and Radius/diameter rather
        // than getting a third: all three place a dimension, and what differs
        // is what they measure *from* — two clicked points, an arc, or two
        // lines already on the drawing. That distinction is what the label
        // says, and drawing three near-identical rulers would make the group
        // harder to read rather than easier.
        command("measure.two_line", t::measure_two_line(), 602)
            .with_icon("measure")
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
        // # No icon, and it is a deliberate refusal
        //
        // There is no check-mark, tick or accept glyph in the set, and no
        // existing key means "complete this gesture". Reusing `measure` — the
        // key the three tools share — would draw a fourth identical ruler in
        // the same group and undermine the very argument the two-line
        // registration above makes for sharing it: the family shares a glyph
        // because all three *place a dimension*, and this one places nothing,
        // it ends the placing. Naming a key that does not exist draws a visible
        // slashed mark, which is a placeholder arriving through the back door.
        // So it renders as its word, which for a one-word completion verb is
        // the clearest thing it could be.
        command("measure.finish", t::measure_finish(), 603).enabled_when("measure.finishable"),
        // `set-scale` is the conversion glyph the icon ui-spec §8.2 assigned
        // — two arrows chasing each other round a circle. Deliberately not a
        // third `measure`: this command measures nothing, it changes what
        // measurements are read against.
        command("measure.set_scale", t::measure_set_scale(), 610)
            .with_icon("set-scale")
            .enabled_when("doc.pages"),
        // §8.2 also assigned `icon-ring.svg` here, and that half is a
        // **recorded deviation**: two concentric circles read as a target or
        // a radio button at 16 px, not as a list of named things. The row was
        // written at reservation depth before the Measure surface existed and
        // states no reasoning to weigh against. `list` is shared with
        // `edit.form_manage_fields`; see that registration.
        command("measure.manage_groups", t::measure_manage_groups(), 611)
            .with_icon("list")
            .enabled_when("doc.open"),
    ]
}
