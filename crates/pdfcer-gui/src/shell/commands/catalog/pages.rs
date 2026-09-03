//! # `shell::commands::catalog::pages` — the Pages tab — what happens to the set of sheets
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
        // Every one of these needs a page to act on, so `doc.pages`
        // throughout. They additionally respect the thumbnail rail's
        // selection when there is one, which is a property of the handler
        // rather than of availability: with no selection they act on the
        // current page, which is a defined answer and not a disabled state.
        // ===================================================================
        command("pages.insert_from_file", t::pages_insert_from_file(), 300)
            .with_icon("insert-pages")
            .enabled_when("doc.pages"),
        // `delete` is the waste-bin glyph, shared with `format.delete` under
        // the header's shared-key convention: the verb is the same one and
        // the two are never drawn together, because Format is contextual and
        // one tab's band shows at a time. What differs is the target, which
        // is what the label says.
        command("pages.delete", t::pages_delete(), 310)
            .with_icon("delete")
            .enabled_when("doc.pages"),
        command("pages.extract", t::pages_extract(), 311)
            .with_icon("page-extract")
            .enabled_when("doc.pages"),
        // ★ These two REUSE existing keys rather than gaining art, and the
        // reuse is the catalogue's own documented meaning rather than a
        // near-enough substitution. `crate::icons::Icon::ChevronUp`'s doc
        // comment already reads: *"'Move selection up' in the page rail and
        // the Combine-files list"* — it was authored 2026-08-03 for exactly
        // this verb, because `▲` (U+25B2) was VERIFIED tofu in the shipped
        // font stack.
        //
        // Drawing page-shaped art for reorder would have been the worse
        // answer twice over: two more assets to keep in step with the rest of
        // the Pages tab, and a departure from the up/down chevron pair, which
        // is the reorder convention in every list control an operator has
        // used. The pair sits side by side here with its labels, which is
        // what disambiguates `chevron-down` from its other role as a menu
        // disclosure marker.
        command("pages.move_up", t::pages_move_up(), 312)
            .with_icon("chevron-up")
            .enabled_when("doc.pages"),
        command("pages.move_down", t::pages_move_down(), 313)
            .with_icon("chevron-down")
            .enabled_when("doc.pages"),
        // ★★★ `pages.split` was HERE until 2026-08-31 — O68. Unregistered
        // with `tools.split_files`, which is the same dialog with a different
        // operand set; see that one's note in `catalog::tools` for the whole
        // argument. R9: nothing is drawn until the boundary chooser exists.
        command("pages.merge_into", t::pages_merge_into(), 315)
            .with_icon("combine")
            .enabled_when("doc.pages"),
        command("pages.rotate_left", t::pages_rotate_left(), 320)
            .with_icon("rotate-ccw")
            .enabled_when("doc.pages"),
        command("pages.rotate_right", t::pages_rotate_right(), 321)
            .with_icon("rotate-cw")
            .enabled_when("doc.pages"),
        // ★★ The PAGE clipboard — O59 item 2. Three commands rather than a
        // chord: `app::dispatch::pageclip`'s header carries why, and the short
        // form is that the `pages.*` operand rule always resolves, so a chord
        // rung reading it would take Ctrl+C from the canvas for ever.
        //
        // ★ Gated on `doc.pages` alone. There is no "are any picked" condition
        // and there must not be: with nothing picked these act on the current
        // sheet, which is a defined answer rather than a disabled state --
        // exactly as `pages.delete` and `pages.rotate_*` already behave.
        command("pages.copy", t::pages_copy(), 322)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("pages.cut", t::pages_cut(), 323)
            .with_icon("cut")
            .enabled_when("doc.pages"),
        command("pages.paste", t::pages_paste(), 324)
            .with_icon("paste")
            .enabled_when("doc.pages"),
    ]
}
