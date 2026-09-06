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
//! ## ★★★ The 2026-09-04 icon pass — this band stops drawing one picture for
//! several controls
//!
//! Until this date the Edit tab carried **two glyph collisions**, and neither
//! was an economy. Both were the shared-key convention applied where it does
//! not hold.
//!
//! * **The five form-field placers were one glyph.** `form-field` belongs to
//!   `edit.form_text_field`; `edit.form_check_box`, `edit.form_radio_button`,
//!   `edit.form_choice` and `edit.form_push_button` each named it as well.
//!   Those five sit in ONE ribbon group, drawn side by side — so the result
//!   was five different words under five identical pictures, and the thing the
//!   picture was supposed to be showing is precisely the field TYPE that
//!   distinguishes them.
//! * **The three redaction commands were one glyph.** `edit.redact` arms a
//!   marking tool, `edit.redact_selection` marks what is already picked, and
//!   `edit.redact_apply` destroys content irreversibly. Three verbs, not one
//!   verb over three operands.
//!
//! Two further commands in this band had no picture at all, each by a refusal
//! argued in prose rather than inherited: `edit.reflow_block` and
//! `edit.attachments`.
//!
//! ⇒ All of it is settled by one event — the **thirty-six glyphs adopted from
//! the outside review of 2026-09-03**, landed on 2026-09-04; see
//! `crate::icons::assets`' batch note for the provenance, which
//! `icons/assets/PROVENANCE.md` is unchanged by. Six keys break a borrow
//! (`check-box`, `radio-button`, `drop-down`, `push-button`,
//! `redact-selection`, `apply-redactions`) and two discharge a refusal
//! (`reflow`, `attachment`).
//!
//! ★★ `edit.select_all`'s refusal was **discharged on 2026-09-04, and not by
//! art arriving** — by the operator pointing out that it had never been his.
//! It was a build session's argument that got quoted until it read as a
//! ruling. See the registration itself for the account; the point that
//! generalises is that **who wrote a refusal is part of the refusal**.
//!
//! ⇒ The two refusals are **discharged, not deleted**. Each paragraph below is
//! rewritten in place to say what spent it and to keep the part that outlives
//! it: which neighbouring glyph the control must never be redrawn to resemble.
//! That constraint is the whole reason those paragraphs are still here — it is
//! exactly what a later "make this group look consistent" pass would break,
//! and by then the refusal that recorded it would be gone.
//!
//! ★★ **This changes an arithmetic that lives elsewhere.** From this band
//! alone the icon-coverage split moves by **+2 named, −2 refused**, which
//! `super::super::tests::the_icon_coverage_split_adds_up_to_the_registry`
//! pins as two literals. Those literals are not this file's to move, and the
//! same batch is being wired across other bands concurrently, so the number is
//! the coordinating session's to settle once — noted here so the failure reads
//! as expected rather than as a surprise.
//!
//! ### ★ One correction this pass was asked to make and must not
//!
//! The instruction for this work stated that THIS header calls the four form
//! tools *"distinguishable only by icon and tooltip"*, and asked that the
//! sentence be made true. **It is not in this file, and the sentence that does
//! exist is about something else.** `crate::shell::manifest`'s `edit` header
//! and `crate::text::commands`' header both record that the salvage source's
//! three content buttons were labelled `Aa`, `I⁺ Aa` and `Obj`, that the first
//! two returned the *same string literal*, and that they were therefore *"two
//! adjacent buttons distinguishable only by icon and tooltip"*. That is a
//! statement about **labels**, about `edit.text` and `edit.add_text`, and the
//! renames those same headers describe — **Edit text**, **Add text**, **Edit
//! objects** — already answered it. It is not about the form group, it needs
//! no correction from this pass, and both files that carry it belong to other
//! owners. Recorded rather than silently skipped, because *"the header already
//! says so"* is otherwise a claim the next reader has to re-derive from
//! scratch.
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
        //
        // ★★★ **The no-icon refusal is DISCHARGED, 2026-09-04**, by a glyph
        // drawn for this role and adopted from the outside review of
        // 2026-09-03. It is not withdrawn and it was not wrong; it is spent,
        // and what spent it is written here so nobody has to reconstruct it
        // from a diff.
        //
        // The refusal was recorded at the registration count in
        // [`super::super::tests::the_icon_coverage_split_adds_up_to_the_registry`],
        // in the note that moved the refused literal 18 → 19 on 2026-08-28:
        //
        // > `edit.reflow_block` refuses a glyph. Its three neighbours in the
        // > Edit ▸ Content group carry one … the operator's own art is the only
        // > art this build ships, and *"re-wrap this paragraph"* has no
        // > conventional glyph to borrow — Word gives it a menu line, not a
        // > picture. A home-made pilcrow-with-arrows would be a symbol nobody
        // > has been taught.
        //
        // ★ **Two claims, and they are spent separately** — which matters,
        // because only the first is a supply objection and only the first is
        // discharged by art merely existing.
        //
        // 1. *Supply.* Ends with the batch. The glyph was drawn for THIS role,
        //    from outside, in the directory's own style contract, so
        //    `icons/assets/PROVENANCE.md` stays true and no machine's hand is
        //    in it. This is the same route `file.save_compacted`'s refusal
        //    describes, and it is the route that refusal named as the remedy.
        // 2. *Convention.* Answered by WHICH mark was chosen, not by the art
        //    existing, and [`crate::icons::Icon::Reflow`] is where that
        //    argument belongs: `reflow.svg` is **not the pilcrow the refusal
        //    rejected**. It is three naked rules, the third stopping short,
        //    with the wrap arrow hooking down from the right margin and back
        //    to the left — the carriage return every text editor draws for
        //    word wrap, which is the one mark for this idea an operator HAS
        //    been taught. The refusal said no conventional glyph existed to
        //    borrow; the claim on the record now is that this is one, and it
        //    is the artist's claim rather than this registration's invention.
        //
        // ★★ **What survives the refusal, and is why this paragraph is still
        // here.** A glyph of bare horizontal rules sits one small edit away
        // from two others, and both near misses are named by the variant:
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
        // ★★★ `edit.copy_as_vector` — the clipboard's copy-OUT, 2026-09-04.
        //
        // `OPERATOR_REQUESTS.md` **O120**, the operator, 2026-09-03: *"Also I'd
        // like to be able to copy and paste anything to other software - like
        // copy and paste vector graphics into word or inkscape for example if
        // possible."*
        //
        // ★★ **Registered HERE and not beside its three siblings**, and the
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
        // ★ **`doc.pages`, not a selection condition.** The command has two
        // operands — the selection if there is one, the whole page otherwise —
        // so there is no state in which an open document makes it meaningless.
        // Gating on `selection.any` would grey it in exactly the case an
        // operator most wants it: *"copy this whole sheet into my report"*.
        //
        // ★★ **No chord.** `Ctrl+Shift+C` was considered and not taken:
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
        // ★★★ `edit.duplicate` — Ctrl+D over a selected comment, 2026-09-06.
        //
        // **What it closes.** A markup could be duplicated only by `Ctrl+C`
        // then `Ctrl+V`, which works and **destroys whatever was on the
        // clipboard**. An operator placing a row of identical revision marks
        // pays that once per mark. Acrobat has had `Ctrl+D` on a comment for as
        // long as it has had comments, and `mockups/app.html:198` — the
        // approved canvas context menu — already draws *"Duplicate  Ctrl+D"*.
        //
        // ★★ **A sibling command rather than an extension of
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
        // ★★ **`selection.any`, not `doc.pages`.** Every other member of this
        // group has an operand rule that always resolves — a paste has the
        // clipboard, a copy-out falls back to the whole page — so all of them
        // are live on any open document. This one has nothing to act on with
        // nothing selected, and R83 says an affordance that cannot be honoured
        // is not offered. The chord is still pushed through blind and declined
        // in words by the dispatcher; the *button* greys.
        //
        // ★★★ **It reuses the `copy` glyph**, under the header's shared-key
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
        // ★ It is placed **last** in the ribbon's Clipboard group rather than
        // beside `edit.copy`, so the two controls that share a glyph are not
        // adjacent. `edit.paste` and `edit.paste_duplicate` are adjacent and do
        // share one, which is the precedent that makes the reuse admissible at
        // all; not repeating the adjacency is the cheap half of not making the
        // band harder to read.
        // ═══════════════════════════════════════════════════════════════════
        command("edit.duplicate", t::edit_duplicate(), 409)
            .with_icon("copy")
            .enabled_when("selection.any"),
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
        // ★★★ **The no-icon refusal is DISCHARGED, 2026-09-04**, by a glyph
        // drawn for this role and adopted from the outside review of
        // 2026-09-03. The refusal stood in this spot and read:
        //
        // > **No icon**, and the refusal is argued rather than inherited. The
        // > conventional glyph for this is a paperclip;
        // > `icons/assets/PROVENANCE.md` makes that directory the operator's
        // > own work, so the alternative to shipping none is not "draw one"
        // > but "ask him for one" — the same judgment `file.save_compacted`
        // > and `edit.reflow_block` reached. A home-made paperclip beside four
        // > hand-drawn glyphs is the mismatch a borrowed icon set exists to
        // > avoid, and the label says it plainly.
        //
        // ★ **Read it again and notice what it never argued: nothing in it is
        // against a paperclip.** It names the paperclip as the RIGHT glyph and
        // refuses only pdfcer's standing to invent one. That is a supply
        // objection carrying its own remedy — *"ask him for one"* — and the
        // asking has happened. So it is spent rather than overturned, and
        // `attachment.svg` is the very glyph it asked for: one open spiral of
        // three concentric arcs with two free ends, in the directory's own
        // style contract, leaving `PROVENANCE.md` true.
        //
        // ★★ **What survives it.** Two neighbours this glyph must never be
        // redrawn toward, both named by [`crate::icons::Icon::Attachment`]:
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
        // ⇒ The `doc.open` argument below is untouched by any of this, and is
        // still what decides when the control is live.
        //
        // ★ `doc.open`, not `doc.pages`, and the difference is real: a
        // document-level attachment lives in the catalogue and belongs to no
        // page, so a document with an empty page tree can still carry files and
        // still be attached to. Gating on pages would hide the panel for the one
        // document whose attachments are all it has.
        command("edit.attachments", t::edit_attachments(), 411)
            .with_icon("attachment")
            .enabled_when("doc.open"),
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
        //
        // ★★★ **AND FOUR OF THE FIVE STOP BORROWING — 2026-09-04.** Until
        // today every one of them named `form-field`. That is one ribbon
        // group, drawn side by side: five controls, five different words, one
        // identical picture.
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
        // ★ `form-field` is NOT re-pointed. [`crate::icons::Icon::FormField`]
        // belongs to `edit.form_text_field`, which keeps it; what changed is
        // that the other four stopped naming another command's glyph.
        //
        // ★★ **Each new glyph has a named near miss, and the distinctions are
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
        //   is a circle with a stem running off it. ★★ The inner mark is a
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
        // ★ The `drop-down` key is named for the LABEL and not for the command
        // id — `edit.form_choice` ships as "Drop-down" — because an icon key
        // answers to what the operator reads.
        //
        // ★ And the greying argument below cuts the same way rather than
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
        // ★★★ **THE REFUSAL HERE WAS NOT THE OPERATOR'S, AND IT WAS BEGINNING
        // TO BE REPORTED AS IF IT WERE — corrected 2026-09-04.**
        //
        // From 2026-09-01 this registration carried a written refusal: *"there
        // is no conventional icon for Select All — Word, Acrobat and
        // Illustrator all present it as words … a marquee glyph would say
        // 'rubber band', which is the gesture this command exists to replace
        // when the rubber band cannot reach."*
        //
        // It was a build session's argument. It was then quoted in the icon
        // coverage count, in this file's header, in `GLYPH_ADOPTION.md`, and
        // **twice in reports to the operator as a settled position** — at which
        // point it had acquired an authority nobody granted it. He answered in
        // four words: *"I didn't refuse that."*
        //
        // ⇒ The distinction is worth stating because it will recur: **a
        // well-argued refusal written by whoever happened to be building that
        // day is not an operator decision.** Quoting it does not promote it.
        // The two are told apart by asking who said it, and the answer belongs
        // in the sentence that records it.
        //
        // ★ The half of the argument that was right survives in the art rather
        // than being discarded: a bare marquee really would read as a rubber
        // band, so the marquee is not bare — it encloses the pointer. See
        // `icons/assets/select-all.svg`, which carries the whole account.
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
        // ★★★ **THE REDACTION FAMILY STOPS BEING ONE PICTURE — 2026-09-04.**
        // All three of these named `redact` until today, and the note that
        // recorded the last of those borrows — the 109 → 110 entry of
        // 2026-08-30 in
        // [`super::super::tests::the_icon_coverage_split_adds_up_to_the_registry`]
        // — defended it in one sentence: *"Three controls about one operation,
        // told apart by their labels."*
        //
        // ⇒ That is the sentence this pass falsifies, and the reason is one
        // line: **the redaction family is not one verb over three operands —
        // it is arm, mark, obliterate.** `edit.redact` arms a tool and changes
        // nothing in the file. `edit.redact_selection` adds a `/Redact`
        // annotation over what is already picked, and removes nothing.
        // `edit.redact_apply` destroys content and does not come back. Three
        // different promises, two and three rows apart on one tab, is the most
        // expensive collision this set could carry, because the price of the
        // wrong press is not a wasted click.
        //
        // ★ Note the shape of the mistake, because it is this file's recurring
        // one: the borrow was justified by what the three commands are ABOUT
        // rather than by what they DO — the same slip the deleted
        // `edit.objects` tooltip made when it was written from the button's
        // NAME instead of its behaviour.
        //
        // ★★ **What each new glyph keeps, and what separates it.** Both halves
        // are load-bearing and both are the artist's, not this registration's:
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
            .with_icon("redact-selection")
            .enabled_when("selection.any"),
        command("edit.redact_apply", t::edit_redact_apply(), 441)
            .with_icon("apply-redactions")
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
