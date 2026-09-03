//! # `shell::commands::catalog::edit` — the Edit tab — changing content that is already there
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
        command("edit.text", t::edit_text(), 400)
            .with_icon("edit-text")
            .enabled_when("doc.pages"),
        command("edit.add_text", t::edit_add_text(), 401)
            .with_icon("add-text")
            .enabled_when("doc.pages"),
        // ★★ `doc.pages` only, like its three neighbours, and NOT a mode guard.
        // A reflow does rewrite the page's own content stream, so a reading
        // stance must not offer it — but that is settled by **visibility**: the
        // whole Edit tab is absent outside the Edit mode
        // (`mode.edit_content`, see `app::conditions`). Adding a second,
        // enable-time gate would be a duplicate rule that can disagree with the
        // first, which is the shape `conditions`' own header argues against.
        command("edit.reflow_block", t::edit_reflow_block(), 406).enabled_when("doc.pages"),
        // ★★★ `edit.objects` was HERE until 2026-08-31, and it is DELETED
        // rather than repointed. `OPERATOR_REQUESTS.md` row O69, the operator:
        // *"We shouldn't even need an Edit Objects button."*
        //
        // He is right, and it was worse than redundant. It was a
        // `dispatch::routes` alias for `view.tool_select`, so pressing it after
        // arming the Points tool silently put him back on the black arrow —
        // i.e. the control he had been told to press in order to edit a drawing
        // was the one that ended node editing. Its tooltip promised *"drag an
        // anchor to move that node"*, which is the POINTS tool described
        // exactly, by a button that armed a different one.
        //
        // ★★ The route was added on 2026-08-28 on the argument that *"a second
        // route to an existing command must not become a second implementation
        // of it"*, and that argument is still sound. What was wrong is one step
        // earlier: it was routed to the wrong target, and the tooltip is the
        // evidence — it was written from what the button was NAMED, not from
        // what it DID.
        //
        // ⇒ The one route to editing drawing objects is now the tool palette,
        // View ▸ Navigate, in the order every program in this class puts it:
        // arrow, white arrow, type, hand. Which is the convention argument, and
        // the operator's own instruction, agreeing.
        // `insert-image` is the picture glyph the icon ui-spec §8.5 reserved
        // for OCR. This command is the earlier and primary claim on it: it
        // places an actual raster on the page, where OCR only reads one.
        command("edit.insert_image", t::edit_insert_image(), 410)
            .with_icon("insert-image")
            .enabled_when("doc.pages"),
        // ★ **No icon**, and the refusal is argued rather than inherited. The
        // conventional glyph for this is a paperclip; `icons/assets/PROVENANCE.md`
        // makes that directory the operator's own work, so the alternative to
        // shipping none is not "draw one" but "ask him for one" — the same
        // judgment `file.save_compacted` and `edit.reflow_block` reached. A
        // home-made paperclip beside four hand-drawn glyphs is the mismatch a
        // borrowed icon set exists to avoid, and the label says it plainly.
        //
        // ★ `doc.open`, not `doc.pages`, and the difference is real: a
        // document-level attachment lives in the catalogue and belongs to no
        // page, so a document with an empty page tree can still carry files and
        // still be attached to. Gating on pages would hide the panel for the one
        // document whose attachments are all it has.
        command("edit.attachments", t::edit_attachments(), 411).enabled_when("doc.open"),
        // `edit.copy_page_text` and `edit.copy_document_text` were here, tokens
        // 420 and 421. They are now `file.copy_page_text` and
        // `file.copy_document_text` in File ▸ Export — operator decision,
        // 2026-08-14; see those registrations for the argument. Both numbers
        // stay unused, exactly as 430 below does, and for the same reason.
        //
        // The Edit ▸ Clipboard group went with them, because those two were its
        // only members and an empty group must not ship. `super::manifest`'s
        // group count moved 32 → 31 with it.
        //
        // `edit.form_fill` was here, token 430. It is now `view.panel_forms`
        // — see that registration. Token 430 stays unused rather than being
        // handed to the next Edit command: a token is what a trace prints,
        // and reusing this one would make an old trace of a form fill read
        // as whatever took its number.
        // ★★★ FIVE COMMANDS, ONE PER FIELD TYPE — 2026-08-26, replacing the
        // single `edit.form_create_field` that was registered and never wired.
        //
        // Separate ids rather than one command with a type argument, because R8
        // makes registration the ONLY way the ribbon learns a capability
        // exists: a build without one of these simply does not register it and
        // its item disappears, with no `#[cfg]` in the manifest and no panel
        // asking what is present.
        //
        // Tokens 434-438 rather than reusing 431. A token is what a trace
        // prints, and an old trace of a "create field" must not read as
        // whichever type took its number — the same rule the comment above
        // applies to 430.
        command("edit.form_text_field", t::edit_form_text_field(), 434)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        command("edit.form_check_box", t::edit_form_check_box(), 435)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        command("edit.form_radio_button", t::edit_form_radio_button(), 436)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        command("edit.form_choice", t::edit_form_choice(), 437)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        // ★★ GREYED, ALWAYS, and deliberately not absent — the operator's
        // ruling of 2026-08-26: *"leave push buttons on the ribbon but greyed
        // out for now."*
        //
        // ★★★ **AND SOMETHING DOES SET IT, since 2026-09-01.** The paragraph
        // above is the history; this is the state.
        //
        // `forms.push_button_runnable` was a condition nothing set, because a
        // button pdfcer placed ran nothing. `pdfcer-core` shipped
        // `EditSession::set_button_action` on 2026-08-30 and `app::conditions`
        // sets the name alongside `doc.pages` — the one line this comment
        // promised. The greying now means what R9 says greying means: no
        // document open, nothing to place a button on.
        //
        // ★ The `enabled_when` STAYS. Deleting it would make the control live
        // with no document, which is the thing the condition was always also
        // doing. The catalog test that pins this string is what keeps the
        // ribbon and `app::dispatch::forms` asking the same question.
        // ★★★ **Select all**, and the reason it is on the EDIT tab rather than
        // being a bare chord: an operator who has lost an object off the side of
        // the sheet needs to FIND the command, and a keyboard shortcut nobody
        // can see is not findable. `Ctrl+A` is bound to it as well, because that
        // is the chord every program in the world uses.
        command("edit.select_all", t::edit_select_all(), 402).enabled_when("doc.pages"),
        command("edit.form_push_button", t::edit_form_push_button(), 438)
            .with_icon("form-field")
            .enabled_when("forms.push_button_runnable"),
        // `list` is shared with `measure.manage_groups`, and the family it
        // belongs to is one of ACTION rather than of subject: form fields and
        // dimension groups have nothing to do with each other, but both
        // commands answer a click by opening a list you add to, rename in and
        // remove from — which is the only thing a glyph can honestly promise
        // where "fields" and "dimension groups" are words only a label can
        // say. Different tabs, so never drawn together.
        command("edit.form_manage_fields", t::edit_form_manage_fields(), 432)
            .with_icon("list")
            .enabled_when("doc.pages"),
        // Drawn to the icon ui-spec §8.14's own construction for this exact
        // command: "a form-field rectangle with a small downward chevron
        // pressing onto it (burn-in metaphor)".
        command("edit.form_flatten", t::edit_form_flatten(), 433)
            .with_icon("form-flatten")
            .enabled_when("doc.pages"),
        // ★ Find — registered, bound to Ctrl+F, and on **no tab**.
        //
        // A third documented exception to the "every command is on its owning
        // tab" convention, alongside `edit.undo`/`edit.redo` (QAT only). Its
        // control is the **status bar's Find toggle**: `RIBBON_IA.md` §6 lists
        // the status bar's contents and puts Find first among them, in the
        // section headed "what deliberately does not go on the ribbon". The
        // `edit.` prefix says where it would go if it ever got a tab, which is
        // the same thing undo's and redo's prefixes say.
        //
        // It is not orphaned, and it needs no `CUSTOM_BACKED` exemption: the
        // manifest keymap binds `Ctrl+F` to it, and a keymap entry is a
        // reference site `Shell::command_references()` walks. So
        // `no_registered_command_is_orphaned` sees it, and a rename that lost
        // the binding would fail that test rather than silently producing a
        // command nothing can reach.
        //
        // `doc.pages`, not `doc.open`: there is no page text to search in a
        // document with no pages, and a Find bar over one is a control whose
        // every input is refused — the exact case that predicate exists to
        // separate.
        command("edit.find", t::edit_find(), 450)
            .with_icon("search")
            .enabled_when("doc.pages"),
        command("edit.redact", t::edit_redact(), 440)
            .with_icon("redact")
            .enabled_when("doc.pages"),
        // ★★ The THIRD marking route — O60, 2026-08-30. Gated on
        // `selection.any`, which is exactly its operand: it marks what is
        // selected and there is nothing to mark without one.
        //
        // ★ Not gated on `selection.delete_permitted` or any removal
        // predicate, deliberately. Marking is not applying — a `/Redact`
        // annotation removes nothing — so the question this control asks the
        // document is *may I add an annotation*, not *may I destroy content*.
        // `edit.redact_apply` is where that second question belongs and where
        // the engine already asks it.
        command("edit.redact_selection", t::edit_redact_selection(), 442)
            .with_icon("redact")
            .enabled_when("selection.any"),
        command("edit.redact_apply", t::edit_redact_apply(), 441)
            .with_icon("redact")
            .enabled_when("doc.pages"),
        // Undo and redo live on the QAT alone. Their predicates are the
        // canonical example of "greying is for temporarily unavailable":
        // an empty stack is a state that ends the moment anything happens,
        // and the tooltip is what explains it.
        command("edit.undo", t::edit_undo(), 490)
            .with_icon("undo")
            .enabled_when("undo.available"),
        command("edit.redo", t::edit_redo(), 491)
            .with_icon("redo")
            .enabled_when("redo.available"),
    ]
}
