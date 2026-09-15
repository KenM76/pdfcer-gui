//! # `shell::commands::catalog::edit` — the Edit tab — changing content that is already there
//!
//! One band of [`super::all`]'s catalogue: the registrations for the Edit tab,
//! and the argument behind each one.
//!
//! ## Why the band is a file
//!
//! A tab's registrations and the prose that justifies them are one subject, and
//! they are re-read together. Splitting the catalogue per tab does **not** hide
//! a handler-token collision — `super::super::tests::every_handler_token_is_unique`
//! sweeps the whole registry and `every_handler_token_is_in_its_tabs_block`
//! asserts each token sits in its own tab's hundred, so a collision is a red
//! test in any arrangement.
//!
//! ## One picture per control, and where that rule bites hardest
//!
//! [`super`]'s header permits a shared icon key across a *family* — `copy` on
//! both copy verbs, `redact` on both redaction verbs — because a family sharing
//! a glyph is how a ribbon reads as grouped. **That convention does not reach
//! controls whose difference is the thing the picture would have to show**, and
//! this band holds the two worst cases in the application:
//!
//! * **The five form-field placers.** `edit.form_text_field`,
//!   `edit.form_check_box`, `edit.form_radio_button`, `edit.form_choice` and
//!   `edit.form_push_button` sit in ONE ribbon group, drawn side by side. A
//!   shared `form-field` key gives five different words under five identical
//!   pictures — and the field TYPE that the picture is supposed to show is
//!   exactly what distinguishes them. Each therefore owns a key:
//!   `check-box`, `radio-button`, `drop-down`, `push-button`.
//! * **The three redaction commands.** `edit.redact` arms a marking tool,
//!   `edit.redact_selection` marks what is already picked, and
//!   `edit.redact_apply` destroys content irreversibly. Three verbs, not one
//!   verb over three operands, so `redact-selection` and `apply-redactions`
//!   are separate from `redact`.
//!
//! ## What a discharged refusal leaves behind
//!
//! Several registrations below record that a control **refused** a glyph and
//! now has one. Those paragraphs are rewritten in place rather than deleted,
//! and the part that outlives the refusal is the load-bearing half: **which
//! neighbouring glyph the control must never be redrawn to resemble.** That
//! constraint is exactly what a later "make this group look consistent" pass
//! would break, and the refusal is where it is written down.
//!
//! ⚠ **Who wrote a refusal is part of the refusal.** A build session's argument
//! quoted back often enough begins to read as an operator ruling, and a
//! refusal attributed to the operator is one nobody revisits. Where a
//! registration below names an author, the attribution is doing work.
//!
//! ⚠ **This band's registrations feed an arithmetic that lives elsewhere.**
//! `super::super::tests::the_icon_coverage_split_adds_up_to_the_registry` pins
//! named-versus-refused as two literals over the whole registry; adding or
//! discharging a glyph here moves them. The count is never restated in prose,
//! here or in [`super`]'s header — the test is the only copy.
//!
//! ## What is here, and what is not
//!
//! The `Command` entries and the argument for each one's label, tooltip,
//! handler token, icon and enable predicate. **The prose is the point** — most
//! of this file is the record of decisions that would otherwise be
//! re-litigated.
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
        // `doc.pages` only, like its three neighbours, and NOT a mode guard.
        // A reflow does rewrite the page's own content stream, so a reading
        // stance must not offer it — but that is settled by **visibility**: the
        // whole Edit tab is absent outside the Edit mode
        // (`mode.edit_content`, see `app::conditions`). Adding a second,
        // enable-time gate would be a duplicate rule that can disagree with the
        // first, which is the shape `conditions`' own header argues against.
        //
        // ⚠ **`reflow` is a hard glyph, and it carries two separate
        // obligations.** *"Re-wrap this paragraph"* has no glyph in the
        // applications an operator has used — Word gives it a menu line, not a
        // picture — so the two failure modes are inventing a symbol nobody has
        // been taught, and drifting into a neighbour's.
        //
        // 1. *It must not be a pilcrow-with-arrows.* [`crate::icons::Icon::Reflow`]
        //    carries the argument: `reflow.svg` is three naked rules, the third
        //    stopping short, with the wrap arrow hooking down from the right
        //    margin and back to the left — the carriage return every text
        //    editor draws for word wrap, which is the one mark for this idea an
        //    operator HAS been taught.
        // 2. *It must stay drawn by hand.* `icons/assets/PROVENANCE.md`
        //    declares the directory the operator's own art, which is what
        //    exempts it from `check-shipped-assets`; a machine-drawn
        //    replacement would make that note false.
        //
        // **A glyph of bare horizontal rules sits one small edit away from
        // two others**, and both near misses are named by the variant:
        //
        // * [`crate::icons::Icon::ManageList`] — `list`, which this very band
        //   uses two registrations below — puts a small square beside each
        //   rule, because a list is an inventory of NAMED things. These rules
        //   must keep nothing beside them: they are prose.
        // * [`crate::icons::Icon::Properties`] and [`crate::icons::Icon::Text`]
        //   wrap their rules in a page frame, because they mean "a document".
        //   This one must stay UNFRAMED: what it acts on is a paragraph, not a
        //   file.
        //
        // ⇒ Adding markers, or adding a sheet outline "so the Content group
        // matches", is precisely the edit a later consistency pass makes on
        // sight — and either one silently turns this button into a different
        // button's picture.
        command("edit.reflow_block", t::edit_reflow_block(), 406)
            .with_icon("reflow")
            .enabled_when("doc.pages"),
        // ═══════════════════════════════════════════════════════════════════
        // `edit.copy_as_vector` — the clipboard's copy-OUT.
        //
        // `OPERATOR_REQUESTS.md` **O120**, the operator: *"Also I'd
        // like to be able to copy and paste anything to other software - like
        // copy and paste vector graphics into word or inkscape for example if
        // possible."*
        //
        // **Registered HERE and not beside its three siblings**, and the
        // deviation is deliberate rather than an oversight worth quietly
        // correcting later.
        //
        // `edit.cut`, `edit.copy`, `edit.paste` and `edit.paste_duplicate` are
        // registered in `catalog::view`, which is the anomaly: this file's own
        // header states that *"the split is per TAB"*, and `super::super`'s
        // `every_handler_token_is_in_its_tabs_block` enforces the same taxonomy
        // on the token — an `edit.*` id must take a token in `400..500`, which
        // this one does. Following the four into `view.rs` would deepen a
        // deviation from the rule the tests already encode; following the rule
        // costs one cross-reference, which is this paragraph.
        //
        // ⇒ Nothing about the RIBBON changes either way: the placement is
        // `shell::manifest::edit`'s Clipboard group, beside the other four,
        // which is where the mockup draws it and where
        // `every_command_id_names_its_owning_tab` requires an `edit.*` id to be.
        //
        // **`doc.pages`, not a selection condition.** The command has two
        // operands — the selection if there is one, the whole page otherwise —
        // so there is no state in which an open document makes it meaningless.
        // Gating on `selection.any` would grey it in exactly the case an
        // operator most wants it: *"copy this whole sheet into my report"*.
        //
        // **No chord.** `Ctrl+Shift+C` was considered and not taken:
        // `check-clipboard-chords.sh` records that `egui-winit` intercepts the
        // three clipboard chords before they become key events, and a fourth
        // clipboard verb reachable by a modifier on one of them is precisely
        // the arrangement that file exists to warn about. It is a ribbon and
        // menu-reachable command, like `file.export_image` beside it, and a
        // binding can be added by the keymap editor by anyone who wants one.
        // ═══════════════════════════════════════════════════════════════════
        command("edit.copy_as_vector", t::edit_copy_as_vector(), 408)
            .with_icon("copy-as-vector")
            .enabled_when("doc.pages"),
        // ═══════════════════════════════════════════════════════════════════
        // `edit.duplicate` — Ctrl+D over a selected comment.
        //
        // **What it closes.** A markup could be duplicated only by `Ctrl+C`
        // then `Ctrl+V`, which works and **destroys whatever was on the
        // clipboard**. An operator placing a row of identical revision marks
        // pays that once per mark. Acrobat has had `Ctrl+D` on a comment for as
        // long as it has had comments, and `mockups/app.html:198` — the
        // approved canvas context menu — already draws *"Duplicate  Ctrl+D"*.
        //
        // **A sibling command rather than an extension of
        // `edit.paste_duplicate`, and that was checked before it was decided.**
        // That command already routes by selection kind: over a form field it
        // pastes as another widget of the same field, and over a markup it
        // *falls through to the ordinary paste* — `app::dispatch::clipboard`'s
        // header says so in as many words, because a markup has no second sense
        // to paste into. Making it duplicate the SELECTION instead would be a
        // paste verb that acts when the clipboard is empty and ignores the
        // clipboard when it is not: two unrelated behaviours behind one id,
        // reachable by a chord named for the one it would stop doing.
        //
        // **`selection.any`, not `doc.pages`.** Every other member of this
        // group has an operand rule that always resolves — a paste has the
        // clipboard, a copy-out falls back to the whole page — so all of them
        // are live on any open document. This one has nothing to act on with
        // nothing selected, and R83 says an affordance that cannot be honoured
        // is not offered. The chord is still pushed through blind and declined
        // in words by the dispatcher; the *button* greys.
        //
        // **It reuses the `copy` glyph**, under the header's shared-key
        // convention and with the same argument `edit.paste_duplicate` makes
        // for reusing `paste` two registrations above:
        //
        // * The mark is two overlapping sheets, and what that mark MEANS is
        //   *"there are now two of these"* — which is what a duplicate is, more
        //   exactly than it is what a copy is. Illustrator, Figma and Inkscape
        //   all draw Duplicate with overlapping shapes for that reason.
        // * A second, subtly different two-sheets glyph would be a distinction
        //   the operator has to learn in order to gain nothing, and would put
        //   this build one step nearer the icon set nobody can tell apart.
        // * `icons/assets/PROVENANCE.md` is untouched, because nothing was
        //   drawn. That directory is declared the operator's own work, and a
        //   machine-drawn substitute would make the declaration false — which
        //   is why "draw a duplicate glyph" was not the answer.
        //
        // It is placed **last** in the ribbon's Clipboard group rather than
        // beside `edit.copy`, so the two controls that share a glyph are not
        // adjacent. `edit.paste` and `edit.paste_duplicate` are adjacent and do
        // share one, which is the precedent that makes the reuse admissible at
        // all; not repeating the adjacency is the cheap half of not making the
        // band harder to read.
        // ═══════════════════════════════════════════════════════════════════
        command("edit.duplicate", t::edit_duplicate(), 409)
            .with_icon("copy")
            .enabled_when("selection.any"),
        // ⚠ **There is deliberately no `edit.objects` command.**
        // `OPERATOR_REQUESTS.md` row O69, the operator: *"We shouldn't even
        // need an Edit Objects button."*
        //
        // The one route to editing drawing objects is the tool palette,
        // View ▸ Navigate, in the order every program in this class puts it:
        // arrow, white arrow, type, hand. The convention argument and the
        // operator's own instruction agree.
        //
        // A button of that name is not merely redundant — it is a trap. Aliased
        // to `view.tool_select`, it arms the black arrow, so an operator who
        // presses the control named for editing a drawing is pressing the one
        // that ENDS node editing. **The rule that catches this is that a
        // tooltip must be written from what a command DOES, not from what it is
        // NAMED**: *"drag an anchor to move that node"* describes the Points
        // tool, and writing it under a button that arms a different one is the
        // evidence the alias was pointed at the wrong target.
        // `insert-image` is the picture glyph the icon ui-spec §8.5 reserved
        // for OCR. This command is the earlier and primary claim on it: it
        // places an actual raster on the page, where OCR only reads one.
        command("edit.insert_image", t::edit_insert_image(), 410)
            .with_icon("insert-image")
            .enabled_when("doc.pages"),
        // **The paperclip is the right picture, and it must be a drawn one.**
        // `icons/assets/PROVENANCE.md` makes the assets directory the
        // operator's own work, so a home-made paperclip beside hand-drawn
        // neighbours is both a visual mismatch and a false provenance note.
        // `attachment.svg` is drawn to that style contract: one open spiral of
        // three concentric arcs with two free ends.
        //
        // **Two neighbours this glyph must never be redrawn toward**, both
        // named by [`crate::icons::Icon::Attachment`]:
        //
        // * [`crate::icons::Icon::Combine`] — `link.svg`, the chain, and the
        //   near miss that actually matters, because both pictures mean *a
        //   thing fastened to a thing*. The chain is two CLOSED interlocking
        //   rings, symmetric, with no free ends: two files becoming one. The
        //   clip is a SINGLE open curve, and the openness IS the meaning — an
        //   attachment is carried separably and can be taken out again, which
        //   is exactly what the panel behind this button offers (attach one,
        //   save one out, remove one). Closing the curve for optical balance
        //   would make the picture say the opposite of what the command does.
        // * [`crate::icons::Icon::ShapeInk`] — the set's other single unbroken
        //   stroke, told apart by REGULARITY rather than by shape:
        //   `shape-ink.svg` is deliberately aperiodic with no baseline because
        //   it means "the path your hand took"; these arcs are concentric and
        //   evenly nested, machined rather than drawn.
        //
        // `doc.open`, not `doc.pages`, and the difference is real: a
        // document-level attachment lives in the catalogue and belongs to no
        // page, so a document with an empty page tree can still carry files and
        // still be attached to. Gating on pages would hide the panel for the one
        // document whose attachments are all it has.
        command("edit.attachments", t::edit_attachments(), 411)
            .with_icon("attachment")
            .enabled_when("doc.open"),
        // ⚠ **Tokens 420, 421 and 430 are reserved and stay unused.** They
        // belonged to `edit.copy_page_text`, `edit.copy_document_text` and
        // `edit.form_fill`, whose commands are now `file.copy_page_text` and
        // `file.copy_document_text` in File ▸ Export and `view.panel_forms` —
        // see those registrations for the argument. **A token is what a trace
        // prints**, so handing one of these to the next Edit command would make
        // an old trace of a text copy or a form fill read as whatever took its
        // number.
        //
        // The Edit ▸ Clipboard group is gone with the first two: they were its
        // only members, and an empty group must not ship.
        //
        // FIVE COMMANDS, ONE PER FIELD TYPE, rather than a single
        // `edit.form_create_field` taking a type argument.
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
        //
        // ⚠ **Four of the five must NOT name `form-field`.** They sit in one
        // ribbon group, drawn side by side, so a shared key gives five
        // controls, five different words and one identical picture.
        //
        // A shared key IS this catalogue's convention where two controls have
        // the same SUBJECT and are separated by something else the operator
        // can see — `edit.paste_duplicate` beside `paste`, the three page
        // clipboard commands, `format.select_form` beside
        // `format.unshare_form`. It is the wrong convention here, and the
        // reason is exact: those pairs are told apart by a tab or by a verb,
        // and these five are told apart by **the field type the picture was
        // supposed to be showing**. The one distinction that mattered was the
        // one the glyph erased.
        //
        // ⇒ It is the same fault the text-markup pass refused when it declined
        // to draw underline, strikeout and squiggly as three copies of
        // `shape-highlight`. That pass was right; this group was the
        // counter-example nobody went back for.
        //
        // `form-field` is NOT re-pointed. [`crate::icons::Icon::FormField`]
        // belongs to `edit.form_text_field`, which keeps it; what changed is
        // that the other four stopped naming another command's glyph.
        //
        // **Each new glyph has a named near miss, and the distinctions are
        // load-bearing.** They are repeated here rather than left in the icon
        // catalogue alone, because THIS is where somebody stands when he
        // decides the group ought to look more consistent:
        //
        // * `check-box` — a tick ENCLOSED IN A BOX, and the box is the
        //   subject: it names a field type, not an accept verb. The variant
        //   asks that the tick never be lifted out of this file to serve as a
        //   bare accept glyph, and that ask holds however the `measure.finish`
        //   and `markup.finish` refusals are eventually settled — those are
        //   other registrations' to settle, and the same batch carries a
        //   separate `check` glyph for them. Its other near miss is
        //   [`crate::icons::Icon::Signatures`], which is forbidden from being
        //   a checkmark at all (a checkmark reads as VALIDATED, and pdfcer
        //   verifies nothing); that one is the mark itself on a rule, with no
        //   frame, where this is a box first and a mark second.
        // * `radio-button` — two circles about one centre and nothing else,
        //   which is what separates it from the set's other circles
        //   ([`crate::icons::Icon::Search`] and the two zooms), each of which
        //   is a circle with a stem running off it. The inner mark is a
        //   RING, not a filled disc, even though a real radio button's
        //   selected state is filled: [`crate::icons::Icon::Redact`]'s fill is
        //   the set's one semantic exception AND the icon pipeline's only
        //   coverage of the fill path, so borrowing it here would cost both.
        // * `drop-down` — the delicate pair is
        //   [`crate::icons::Icon::ChevronDown`], which is not another
        //   command's art but the ribbon's own split-button disclosure marker,
        //   so it can appear on the chrome of a neighbouring control in this
        //   very band. The FRAME is the whole cue: a bare chevron means "this
        //   control opens something below it"; a chevron inside a field
        //   rectangle means "the control IS a list". The box must never be
        //   dropped for optical balance. Against `form-field` one item to its
        //   left: a caret means "type here", a value line with a chevron on it
        //   means "pick from these".
        // * `push-button` — rounded on all four corners and standing on a base
        //   line, against the square-cornered `form-field` and `drop-down`,
        //   because a field is a hole in the page and a button is an object on
        //   top of it. Its other collision is
        //   [`crate::icons::Icon::Stamp`], which shares a base line at the
        //   same height and separates on proportion alone: stamp is a tall
        //   stack read vertically, this is a wide slab.
        //
        // The `drop-down` key is named for the LABEL and not for the command
        // id — `edit.form_choice` ships as "Drop-down" — because an icon key
        // answers to what the operator reads.
        //
        // And the greying argument below cuts the same way rather than
        // against it: a control that spends time dimmed needs its own picture
        // MORE, not less. Five identical glyphs of which one is grey reads as
        // a rendering fault, not as an unavailable capability.
        command("edit.form_text_field", t::edit_form_text_field(), 434)
            .with_icon("form-field")
            .enabled_when("doc.pages"),
        command("edit.form_check_box", t::edit_form_check_box(), 435)
            .with_icon("check-box")
            .enabled_when("doc.pages"),
        command("edit.form_radio_button", t::edit_form_radio_button(), 436)
            .with_icon("radio-button")
            .enabled_when("doc.pages"),
        command("edit.form_choice", t::edit_form_choice(), 437)
            .with_icon("drop-down")
            .enabled_when("doc.pages"),
        // Present and greyed rather than absent — the operator's ruling:
        // *"leave push buttons on the ribbon but greyed out for now."*
        //
        // ⚠ **`forms.push_button_runnable` is a condition something SETS.**
        // `app::conditions` sets the name alongside `doc.pages`, on the
        // strength of `pdfcer-core`'s `EditSession::set_button_action`, so the
        // greying means what R9 says greying means: no document open, nothing
        // to place a button on. A condition nothing sets would grey the control
        // permanently and tell the operator nothing about why.
        //
        // The `enabled_when` STAYS. Deleting it would make the control live
        // with no document open. The catalog test that pins this string is what
        // keeps the ribbon and `app::dispatch::forms` asking the same
        // question.
        // **Select all**, and the reason it is on the EDIT tab rather than
        // being a bare chord: an operator who has lost an object off the side of
        // the sheet needs to FIND the command, and a keyboard shortcut nobody
        // can see is not findable. `Ctrl+A` is bound to it as well, because that
        // is the chord every program in the world uses.
        // ⚠ **The icon must not read as a rubber band.** Word, Acrobat and
        // Illustrator all present Select All as words, and a bare marquee says
        // *rubber band* — which is the gesture this command exists to replace
        // when the rubber band cannot reach. So the marquee is not bare: it
        // encloses the pointer. `icons/assets/select-all.svg` carries the whole
        // account.
        //
        // ⚠ **A refusal recorded here is only the operator's if he said it.**
        // The argument above is a build-time judgment about art, and a
        // well-argued refusal written by whoever happened to be building that
        // day is not an operator decision — quoting it in a report does not
        // promote it to one. Where a registration in this band attributes a
        // refusal, the attribution is what makes it re-openable.
        command("edit.select_all", t::edit_select_all(), 402)
            .with_icon("select-all")
            .enabled_when("doc.pages"),
        command("edit.form_push_button", t::edit_form_push_button(), 438)
            .with_icon("push-button")
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
        // Find — registered, bound to Ctrl+F, and on **no tab**.
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
        // ⚠ **THE REDACTION FAMILY MUST NOT BE ONE PICTURE.** The shared-key
        // convention reads *"three controls about one operation, told apart by
        // their labels"* — and it does not hold here, because **this is not one
        // verb over three operands: it is arm, mark, obliterate.**
        // `edit.redact` arms a tool and changes nothing in the file.
        // `edit.redact_selection` adds a `/Redact` annotation over what is
        // already picked, and removes nothing. `edit.redact_apply` destroys
        // content and does not come back. Three different promises, two and
        // three rows apart on one tab, is the most expensive collision this set
        // could carry, because the price of the wrong press is not a wasted
        // click.
        //
        // ⇒ The shape of that mistake recurs in this file: a borrow justified
        // by what commands are ABOUT rather than by what they DO — the same
        // slip as writing a tooltip from a button's NAME instead of its
        // behaviour.
        //
        // **What each glyph keeps, and what separates it.** Both halves are
        // load-bearing and both are the artist's, not this registration's:
        //
        // * `redact-selection` keeps the solid bar — the family mark — and
        //   separates on a DASHED marquee around it, this shell's vocabulary
        //   for "a selection" and the detail that resolves first at 16 px. It
        //   carries none of `redact`'s two text rules above and below, because
        //   a selection need not be text. The dash is not decoration:
        //   [`crate::icons::svg`] says in its own words that without it this
        //   glyph *is* `redact`, which is why `stroke-dasharray` stopped being
        //   an ignored attribute.
        // * `apply-redactions` is the bar with a tick struck against it — the
        //   mark is no longer a proposal, it has been carried out — and is
        //   deliberately UNFRAMED where [`crate::icons::Icon::Redact`] wraps
        //   its bar in a page outline. Arming puts a mark ON a page; applying
        //   is done to the whole document, so the bar floats free.
        //
        // ⇒ Against each other the difference is tick-versus-enclosure, the
        // strongest pairwise cue available at this size. A later pass that
        // "unified the family" by restoring the page frame to either one would
        // undo exactly that, and would do it to the one command in this band
        // whose mistake cannot be undone.
        //
        // The THIRD marking route — O60. Gated on
        // `selection.any`, which is exactly its operand: it marks what is
        // selected and there is nothing to mark without one.
        //
        // Not gated on `selection.delete_permitted` or any removal
        // predicate, deliberately. Marking is not applying — a `/Redact`
        // annotation removes nothing — so the question this control asks the
        // document is *may I add an annotation*, not *may I destroy content*.
        // `edit.redact_apply` is where that second question belongs and where
        // the engine already asks it.
        command("edit.redact_selection", t::edit_redact_selection(), 442)
            .with_icon("redact-selection")
            .enabled_when("selection.any"),
        command("edit.redact_apply", t::edit_redact_apply(), 441)
            .with_icon("apply-redactions")
            .enabled_when("doc.pages"),
        //
        // ⇒ A control that scanned and deleted on one press would be exactly
        // that break — the middle step skipped, on the one operation where the
        // wrong press is not a wasted click.
        //
        // Why it is in **Protect** and not in View. Off-page content is not
        // a display mode, it is a **leak class**: a cropped title block with the
        // old revision table still past the left edge, a superseded note dragged
        // off the sheet rather than deleted, a customer's name moved out of the
        // frame. None of it renders. All of it is extractable, searchable, and
        // sent. That is the same sentence the rest of this group exists for.
        //
        // `doc.pages`, not `selection.any` and not a capability predicate.
        // Its operand is the whole document and it needs nothing else — which is
        // what makes it the one redaction control that is useful before the
        // operator suspects anything. A document with no pages has nothing to
        // walk, and that is the only state where it is meaningless.
        //
        // The glyph is its own (`off-page`), not a borrow of `redact`. This
        // file's recurring mistake is borrowing on what two commands are ABOUT
        // rather than on what they DO, and this one does something none of the
        // other three do: it **looks**. Its art is a page outline with a mark
        // sitting outside it — the subject stated literally, which at 16 px is
        // the only thing that survives.
        command("edit.offpage", t::edit_offpage(), 443)
            .with_icon("off-page")
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
