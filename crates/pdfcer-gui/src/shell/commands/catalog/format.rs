//! # `shell::commands::catalog::format` — the Format contextual tab — what changes about the selection
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
        // The tab is visible when `selection.any` and the command inside it
        // is enabled by the same condition. That is not redundant: the tab
        // and its contents are evaluated independently, and a Format tab
        // that appeared with a greyed Delete would be the placeholder P3
        // forbids, arriving through a mismatch rather than a decision.
        // ===================================================================
        // ★★ `selection.actionable`, not `selection.any`, since 2026-08-28.
        // Both commands can act on a selected FORM FIELD, which is not in
        // `SelectionState` — see `app::conditions` for why that is a second
        // condition rather than a widening of the first.
        command("format.delete", t::format_delete(), 800)
            .with_icon("delete")
            .enabled_when("selection.actionable"),
        // ★ A second ROUTE to `file.properties`, not a second command that
        // opens the panel. Its arm raises `Action::Command("file.properties")`,
        // which is the mechanism that keeps one command's guards in one place.
        //
        // Registered as its own id because the shell enforces one command, one
        // tab — and the two placements answer different questions: File ▸
        // Document is "tell me about this file", Format is "tell me about the
        // thing I just clicked".
        //
        // The icon is `properties`, shared with `file.properties` under the
        // header's shared-key convention: same panel, same glyph, and the two
        // are never drawn together because Format is contextual.
        command("format.properties", t::format_properties(), 801)
            .with_icon("properties")
            .enabled_when("selection.actionable"),
        // ★★ **Greyed, not absent, when the selection is not inside a form.**
        //
        // R9 draws the line by *why* a thing is unavailable: a capability this
        // build does not have renders nothing at all, and a capability that is
        // **temporarily** unavailable is greyed with the reason on hover. Which
        // one this is turns on a fact about the operator's current selection,
        // not about the build — the command works, on the very next click, on
        // any document with a form. That is the greying case, and it reads the
        // same way `format.delete`'s `selection.any` does one line above.
        //
        // ★ And a greyed control is a hint, never an enforcement. `enabled_when`
        // greys a ribbon item and stops nothing: every non-ribbon route — the
        // context menu, a chord, a future script — reaches the dispatcher
        // without consulting it. The arm in `app::dispatch` therefore asks the
        // same question again and *says why* when the answer is no, which is
        // the ruling this project made after a blanket dispatcher guard was
        // written and two tests refused it for making `Ctrl+Z` on an empty
        // stack do nothing and say nothing.
        //
        // ★ The glyph is `pick-form-xobject`, **reused** rather than new, under
        // the same shared-key convention `format.properties` uses one line
        // above. It is not a near-miss reuse of the kind the header's refusal
        // table is full of: that glyph's entire subject *is* a form XObject —
        // it is the pick filter's form class — and this command's entire
        // subject is a form XObject. Two controls about one thing, drawn the
        // same, is the convention working rather than an economy.
        //
        // Drawing new art was never the alternative. `icons/assets/PROVENANCE.md`
        // declares that directory the **operator's own art**, which is what
        // exempts it from `check-shipped-assets`, and a machine-drawn SVG would
        // make that provenance note false — `file.ocr`'s refusal argues it at
        // length and the argument is unchanged here.
        command("format.select_form", t::format_select_form(), 802)
            .with_icon("pick-form-xobject")
            .enabled_when("selection.in_form"),
        // ★★★ **The "option" half of decision 076**, registered 2026-08-28
        // after `EDITABLE_SURFACES.md` found `EditSession::unshare_form`
        // implemented in the engine and named nowhere in this crate.
        //
        // `RIBBON_IA.md` §5.8 is what puts it here. That section's table gives
        // the **Vector object** row as `Stroke · Fill · Winding rule · Node
        // tools · Delete` and describes the tab's job in one line: it *"carries
        // what a user changes **while working**"*, as against the Properties
        // panel's complete property set. Giving a page its own copy of a shared
        // drawing is exactly a mid-gesture act — it is what an operator does in
        // the second between noticing a typo in the title block and typing over
        // it — so it belongs on the tab rather than in the panel, and it
        // belongs in the **selection** group beside the two commands that are
        // also about *the thing you just clicked and what encloses it*.
        //
        // It is not in §5.8's table, because that table was written on
        // 2026-08-12 and this verb did not exist in the engine until this
        // month. §5.8's amendment convention is followed: the placement is
        // argued from the section's stated principle rather than from a row.
        //
        // ★★ **`selection.in_form`, the same predicate as `select_form` one
        // line above, and that is the correct answer rather than a convenient
        // one.** `app::conditions` publishes it as *"something selected on this
        // page lives inside a form XObject"*, and this verb's operand is
        // derived from a **leaf** — the outermost form enclosing it. The
        // condition is therefore literally the question "is there an operand?",
        // asked in the one place both controls read it from.
        //
        // ⇒ It is also why the two commands must not be collapsed into one that
        // selects-and-unshares: after `format.select_form` there is no leaf any
        // more, so the predicate is false and this control correctly greys.
        // Two acts, two conditions, both honest about what they need.
        //
        // ★ Greyed rather than absent, on `format.select_form`'s R9 reading
        // exactly: the capability is present in this build and on this
        // document, and what is missing is the operand, which the next click
        // supplies. The tooltip explains it on hover, which is the half of R9
        // that makes greying legitimate rather than lazy.
        //
        // ★ The glyph is `pick-form-xobject`, shared with `format.select_form`
        // under the header's shared-key convention. The two commands are about
        // one structure — a form XObject — and a family sharing a glyph is how
        // a ribbon reads as grouped. Drawing new art was never the alternative:
        // `icons/assets/PROVENANCE.md` declares that directory the operator's
        // own art, and a machine-drawn SVG would make that note false.
        command("format.unshare_form", t::format_unshare_form(), 808)
            .with_icon("pick-form-xobject")
            .enabled_when("selection.in_form"),
        // -------------------------------------------------------------------
        // The Font group — `RIBBON_IA.md` §5.8's "Text run" row.
        //
        // ★★★ **All five are `enabled_when("selection.text")` and NOT
        // `selection.any`**, and getting that backwards would grey them in
        // exactly the state where they work.
        //
        // `EditSession::format_text` locates its operand by a pinned byte span
        // into a decoded content buffer, keyed on a **run** of the page's text
        // extraction. `selection.any` is the *object* selection — a paint-order
        // index — and nothing in either crate maps between the two index
        // spaces. So the swept range is the operand, and the swept range is
        // what `selection.text` reports.
        //
        // ★★ **Greyed rather than absent when there is no sweep**, which is R9
        // read carefully. The capability is present — this build has
        // `format_text`, this mode may edit content, this document is open —
        // and what is missing is the *operand*, which the next gesture
        // supplies. That is the textbook temporarily-unavailable case, it is
        // greyed, and it is explained on hover. **The explanation is the whole
        // point**: `text::commands`' own note above these five records why
        // each tooltip has to name the route to an operand, and it is the
        // surface that answers O37's *"nothing on screen tells you to press
        // T"*.
        //
        // Their **absence** is a different rule and lives in the manifest:
        // every item of the group carries `visible_when: "mode.edit_content"`,
        // so Read and Review — which cannot change page content at all — draw
        // no Font group rather than five permanently greyed controls.
        //
        // ★ Three of the five are drawn by an `Item::Custom` and have no
        // button of their own: a face chooser, a size field and a colour
        // swatch are not buttons. They are registered anyway, because a
        // registered command is how this shell learns a capability exists
        // (R8), because the a11y name and the reachability check both read the
        // registry, and because the custom renderer draws the registered label
        // rather than a second copy of it. See `manifest::CUSTOM_BACKED`.
        //
        // ★ **No icons on any of the five.** Word draws `B` and `I` as glyphs
        // and this build has no such art; `icons/assets/PROVENANCE.md` declares
        // that directory the operator's own work, which is what exempts it from
        // `check-shipped-assets`, and a machine-drawn substitute would make that
        // note false. Without an icon a `Small` item resolves to `Medium`
        // (`egui_shell::ribbon::sizing::resolved`), so the labels are what
        // render — "Bold" and "Italic", which are unambiguous where a
        // home-made glyph would not be.
        command("format.font", t::format_font(), 803).enabled_when("selection.text"),
        command("format.font_size", t::format_font_size(), 804).enabled_when("selection.text"),
        command("format.bold", t::format_bold(), 805).enabled_when("selection.text"),
        command("format.italic", t::format_italic(), 806).enabled_when("selection.text"),
        command("format.font_colour", t::format_font_colour(), 807).enabled_when("selection.text"),
    ]
}
