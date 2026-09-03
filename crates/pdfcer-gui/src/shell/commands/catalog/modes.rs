//! # `shell::commands::catalog::modes` — the mode selector — Read, Review, Edit
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
        //
        // Not ribbon commands: the three positions of the selector, bound
        // to Ctrl+1/2/3. Always available — a mode is an interface-
        // complexity control, not a permission, and there is no document
        // state in which changing your own view stance should be refused.
        //
        // ★ **No icons, and this is the one entry in the whole "which
        // commands get a glyph" question that is settled by the renderer
        // rather than by taste.** `egui_shell::ribbon::mode_selector` draws
        // the modes as **text segments** of an N-position segmented control,
        // taking each one's `Mode::label` from the manifest — it never looks
        // at a `Command`, and the module contains no icon path at all (the
        // string `icon` does not occur in the file). `MODES_AND_PANELS.md`
        // Part 1 is why: the control must render "as a real segmented control
        // with all three labels visible — not a bare track with a knob, where
        // the available positions are invisible until you drag."
        //
        // So a key here would resolve to art nothing draws. Worse, it would
        // look like a wiring bug to the next reader — a command that names a
        // glyph and never shows one — which is the failure mode the visible
        // slashed mark exists to make loud, arriving in the one place the
        // mark cannot appear.
        // ===================================================================
        command("mode.read", t::mode_read(), 900),
        command("mode.review", t::mode_review(), 901),
        command("mode.edit", t::mode_edit(), 902),
    ]
}
