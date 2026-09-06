//! The **Edit** tab — *what am I changing about content that is already
//! there?*
//!
//! `RIBBON_IA.md` §5.4. **Four** groups: Content, Insert, Forms, Protect.
//!
//! ★ It was five until 2026-08-14, when the operator moved the two text-copy
//! commands to File ▸ Export and the **Clipboard** group — whose only members
//! they were — was deleted rather than left empty. The site of that group
//! carries the full reasoning; the one-line version is that copying is not
//! authoring, so the verb does not belong on the authoring tab, and an empty
//! band is the placeholder `RIBBON_IA.md` P3 forbids.
//!
//! # Three renames that are the point of the tab
//!
//! The salvage source's Content group carried three buttons labelled
//! `Aa`, `I⁺ Aa` and `Obj`. `Obj` is not a word, and the first two
//! returned the *same string literal* — two adjacent buttons
//! distinguishable only by icon and tooltip. These are the primary
//! content-editing tools and they were the least legible controls in the
//! application. They are now **Edit text**, **Add text** and **Edit
//! objects**, with the icons kept.
//!
//! # The `Editing on` master toggle is gone
//!
//! Operator decision, 2026-08-12: *"make it work the same way other
//! programs do."*
//!
//! No mainstream editor has a global editing switch. Acrobat, Bluebeam,
//! Word and Illustrator all work the same way: selection and Delete are
//! always live, and picking a tool arms *that tool* until Escape or
//! another tool. There is no state in which a click does nothing without
//! the application saying so.
//!
//! So there is no `Mode` group on this tab and no `edit.editing_enabled`
//! command. `RIBBON_IA.md` §7's migration map has a row for it — `Edit ▸
//! ContentTools ▸ Editing on` → `Edit ▸ Mode` — which §5.4 then
//! supersedes; §5.4 is the later and more specific statement and it is the
//! one implemented. The command is deliberately **not** in
//! [`super::PLANNED`] either, because it is not planned: it is deleted.
//!
//! This matters for how the Read/Review/Edit modes must behave. The rule
//! that makes those safe — *a mode changes what is **visible**; it never
//! makes a visible control silently inert* — is precisely the rule the
//! master toggle broke. A mode **removes** the tools it disables, so
//! there is no click that mysteriously fails. Reintroducing a global
//! enable flag under any name would undo that.
//!
//! # Redact arrives here
//!
//! From Tools ▸ Protect. One of the three moves a returning user will
//! notice, and the reasoning is the same as for the other two: a user
//! editing a document looks under Edit for the command that removes
//! content from it. Tools is for jobs that run across *other* files.
//!
//! The pair is kept together and in this order — mark, then apply —
//! because the asymmetry between them is the dangerous part: marking is
//! reversible and applying is not, and both tooltips say so.
//!
//! # What is absent
//!
//! The whole **Arrange** group (align, distribute, bring forward, send
//! backward, group, ungroup, flip) is **N**, so it is not here. So is the
//! object clipboard — cut, copy, paste, paste in place — and with the two
//! text-copy commands gone to File ▸ Export there is nothing left for a
//! Clipboard band to hold, so the band is absent rather than empty.
//! `Shape ⌄` and `Sanitise…` are **N**.

use super::{command, group, icon_only, large};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Edit tab.
pub(super) fn tab() -> Tab {
    Tab::new("edit", ribbon::tab_edit())
        .with_question(ribbon::question_edit())
        .with_groups([
            // ---------------------------------------------------------------
            // Content — the three primary editing tools, relabelled.
            // ---------------------------------------------------------------
            group(
                "content",
                ribbon::group_edit_content(),
                [
                    // ★★★ FIRST in the group, and the position is the argument.
                    //
                    // An operator reaches for this when something has gone
                    // wrong — an object dragged off the sheet, invisible and
                    // unclickable — and a rescue that is findable only by
                    // knowing `Ctrl+A` is not findable. It sits where the eye
                    // lands first on the tab they are already working in.
                    command("edit.select_all"),
                    command("edit.text"),
                    command("edit.add_text"),
                    // ★ Beside the text verbs it belongs with, and after them:
                    // an operator reflows a paragraph *because* they have just
                    // retyped a sentence in it, so it follows the tool that
                    // does the retyping.
                    command("edit.reflow_block"),
                    // ★ `edit.objects` was the fourth member until 2026-08-31.
                    // Deleted, not moved — see its former registration in
                    // `shell::commands::catalog::edit` for O69's argument. The
                    // group is three again, and Content now means "the text on
                    // this page", which is what its three members actually do.
                ],
            ),
            // ---------------------------------------------------------------
            // Insert — new content onto an existing page.
            //
            // Placing an image works today only by drag and drop, which is
            // a gesture with no discoverable equivalent: there is nothing
            // on screen that tells an operator it is possible. A command
            // is the affordance.
            //
            // `Shape ⌄` is **N**.
            // ---------------------------------------------------------------
            // ★★ **`RIBBON_IA.md` names Attachments nowhere**, and this is the
            // group it lands in. Recorded here rather than in a commit message,
            // because a placement the IA did not make is the one a later reader
            // will want the argument for.
            //
            // §5.2's Panels row lists Pages, Objects, Bookmarks, Layers,
            // Signatures, Comments and Forms; §5.1's Document band holds
            // Properties and Fonts; no section of that document mentions an
            // embedded file. So the tab was chosen by the rule §5.13 states
            // after it had decided three tabs in a row — *a command refused in
            // a mode where the operator plainly needs it is evidence that the
            // command's tab is wrong* — read in the direction it also runs:
            //
            // Read is shown `file` and `view` alone. Putting this on either
            // would give a **reading stance** a control that embeds and removes
            // whole files. That is `crate::panels::Panel::Redact`'s argument
            // exactly, and Redact is the closest thing this IA has to a
            // precedent for a panel whose subject is what the document
            // permanently carries.
            //
            // ★ **Insert rather than a new group**, and the group's own caption
            // is what makes it fit: this band is *"new content onto an existing
            // page"* for the image and *"something that was not in this
            // document before"* for the attachment. The distinction — an
            // embedded file is document-level and on no page at all — is real,
            // and it is stated where an operator meets it, in the panel's own
            // rows. A one-command group of its own would be improvising a
            // ribbon band the IA does not have, which is a larger invention
            // than borrowing a caption.
            group(
                "insert",
                ribbon::group_edit_insert(),
                // ★ BOTH large, 2026-09-04 — the mockup draws `Image…` and
                // `Attachments` as the Insert group's two big controls. The
                // whole group is promoted, so nothing is hoisted past
                // anything.
                [large("edit.insert_image"), large("edit.attachments")],
            ),
            // ★★ **Clipboard is BACK, 2026-08-19** — and the note below, which
            // explains why it was deleted, is kept because its reasoning was
            // right and only its premise expired.
            //
            // It was deleted because its two members moved to File ▸ Export and
            // nothing was left. What refills it is not those two returning: it
            // is `edit.cut`, `edit.copy` and `edit.paste`, the three the old
            // note called *"a group whose eventual first four entries are
            // cut/copy/paste/paste-in-place for the object clipboard (N)"*.
            //
            // Three, not four. **Paste-in-place is not here** and its absence is
            // a decision: a same-page paste offsets so the copy is visible, and
            // a cross-page paste lands in place already — so the fourth command
            // would be a control that does what `Ctrl+V` does on the only page
            // where an operator would press it. `canvas::clipboard`'s header
            // carries the argument.
            group(
                "clipboard",
                ribbon::group_edit_clipboard(),
                [
                    icon_only("edit.cut"),
                    icon_only("edit.copy"),
                    icon_only("edit.paste"),
                    // ★★ Four now, and the fourth is NOT the paste-in-place the
                    // note above rules out. It is `edit.paste_duplicate`, and it
                    // earns its place on the one ground that note demanded and
                    // paste-in-place could not meet: it does something Ctrl+V
                    // does not. Pasting a form field as another BOX for the same
                    // field is a different act with a different result, not the
                    // same act in a different position.
                    //
                    // ★ Labelled rather than icon-only would be better on
                    // discoverability grounds and is not offered, because the
                    // group is four icons wide and a single labelled member in a
                    // row of three icons reads as a mistake. The label is in the
                    // tooltip and in the Edit menu; `RIBBON_SCALING.md`'s rule is
                    // that a group's members share a presentation.
                    icon_only("edit.paste_duplicate"),
                    // ★★★ **Five now — `edit.copy_as_vector`, 2026-09-04**, and
                    // it is the one member of this group that copies OUT of
                    // pdfcer rather than within it (`OPERATOR_REQUESTS.md`
                    // O120).
                    //
                    // ★★ **Clipboard, not File ▸ Export**, and the mockup's own
                    // caption note is the argument: *"it is a clipboard verb —
                    // chord-reachable, and its result is pasted, not saved."*
                    // Export writes a file the operator then has to find and
                    // place; this puts the same geometry one `Ctrl+V` away.
                    // Putting it beside Copy is also what makes the difference
                    // legible — it reads as a variant of Copy, which is exactly
                    // what it is.
                    //
                    // ★ **`icon_only`, and the approved mockup draws it
                    // labelled.** The deviation is this group's own rule, stated
                    // one comment up when `edit.paste_duplicate` joined: *"a
                    // single labelled member in a row of three icons reads as a
                    // mistake … `RIBBON_SCALING.md`'s rule is that a group's
                    // members share a presentation."* The mockup's Clipboard cap
                    // draws three members and this one; the shipped group has
                    // four already, all icon-only, so adopting the label here
                    // would make this the only labelled control in a band of
                    // five. The label is in the tooltip and in the Edit menu,
                    // which is where `edit.paste_duplicate` puts its own.
                    icon_only("edit.copy_as_vector"),
                    // ★★★ **Six now — `edit.duplicate`, 2026-09-06** (Ctrl+D),
                    // and it is the one member of this group that never touches
                    // the clipboard at all.
                    //
                    // ★★ **That is exactly why it is in the Clipboard group.**
                    // The band is the operator's *"make another one of this"*
                    // cluster, and until today the only way to make another
                    // comment was Copy-then-Paste — which is to say, the
                    // capability was in this group already, spread across two
                    // of its buttons and costing whatever was on the clipboard.
                    // Putting the direct verb anywhere else would separate it
                    // from the two controls it replaces.
                    //
                    // ★ **Placed LAST rather than beside `edit.copy`**, whose
                    // glyph it reuses. `edit.paste` and `edit.paste_duplicate`
                    // are adjacent and share `paste`, which is the precedent
                    // that makes a shared glyph admissible here at all; not
                    // repeating the adjacency keeps the band readable. The
                    // registration in `shell::commands::catalog::edit` carries
                    // the whole argument for the reuse.
                    //
                    // ★★ `icon_only`, like its five neighbours —
                    // `RIBBON_SCALING.md`'s rule that a group's members share a
                    // presentation, and the same rule `edit.copy_as_vector`
                    // deviated from the mockup to honour one comment up.
                    //
                    // ⚠ **The approved mockup's home for this verb is the
                    // CANVAS CONTEXT MENU**, not the ribbon —
                    // `mockups/app.html:198` draws *"Duplicate  Ctrl+D"* in the
                    // object menu. That placement is not made here because
                    // `canvas::menus` and `shell::menus` belonged to a
                    // concurrent track on the day this landed, and a context
                    // menu edited from two sessions is a merge nobody can
                    // review. The ribbon entry is a placement, not a
                    // substitute: **the context-menu row is still owed**, and
                    // this note is where whoever adds it should start.
                    icon_only("edit.duplicate"),
                ],
            ),
            // ---------------------------------------------------------------
            // ★ **Clipboard was here, and it is deleted rather than emptied.**
            //
            // It held exactly two commands — `edit.copy_page_text` and
            // `edit.copy_document_text` — and on 2026-08-14 the operator moved
            // both to File ▸ Export as `file.copy_page_text` and
            // `file.copy_document_text`. Its own note read:
            //
            //     Clipboard — the two commands that moved off File.
            //     Copying text out of a document is a content operation, not a
            //     file operation. That is the whole argument, and it is why
            //     these two are the first entries in a group whose eventual
            //     first four entries are cut/copy/paste/paste-in-place for the
            //     object clipboard (**N**).
            //
            // The premise held; the conclusion did not follow. A content
            // operation is not automatically an **authoring** operation, and
            // copying authors nothing — it reads the page and writes to the
            // clipboard. What made that visible was the chord/mode gate
            // refusing `Ctrl+Shift+C` in Read, a mode measured against Acrobat
            // Reader, which copies text. Same line as `edit.form_fill` →
            // `view.panel_forms`: *filling is not authoring*, and neither is
            // copying.
            //
            // **The group goes with them, and does not stay as a placeholder
            // for the object clipboard it was reserving space for.** P3 is the
            // rule — an unavailable capability renders nothing — and a caption
            // with no controls under it is the emptiest possible stub: a band
            // that promises cut, copy and paste and offers no way to reach any
            // of them. All four object-clipboard ids are **N** in
            // `super::PLANNED`, which is where that reservation belongs, and
            // that is where the group will be rebuilt from on the day one of
            // them ships. `super::manifest`'s documented group count goes
            // 32 → 31.
            // ---------------------------------------------------------------
            // Forms — one band where the salvage source had two, and now
            // one band holding **three** steps of a form's life rather than
            // four.
            //
            // `Forms` (fill) and `Build Form` (author) were separate groups
            // on the same tab, which asks the operator to already know which
            // side of that line they are on before they can find the
            // control. That argument put fill, create, manage and flatten
            // together, and it was right about the operator who is *in this
            // tab*.
            //
            // ★ **Fill left on 2026-08-14, and the argument that moved it is
            // stronger than the one that kept it here.** The operator's
            // answer to `crate::app::modes`' open question is that Read
            // fills forms — Acrobat Reader does, and replacing it is the
            // stated goal. Read is shown `file` and `view` alone, and P1
            // gives a command exactly one tab, so `edit.form_fill` became
            // `view.panel_forms` in View ▸ Panels.
            //
            // What is left is not a remnant. Filling a field is using the
            // document as its author designed it; creating a field, renaming
            // one and flattening the result are changes to the design
            // itself. That line is real, it is the line the mode taxonomy
            // already draws between Review and Edit, and the three verbs
            // that stayed are on the authoring side of it together.
            //
            // `Flatten` moves out of the Forms pane for the same reason
            // `Export form data` moved to File ▸ Export: a command buried
            // in a panel is reachable only by someone who already opened
            // the panel.
            // ---------------------------------------------------------------
            group(
                "forms",
                ribbon::group_edit_forms(),
                [
                    // ★★★ FIVE FIELD TYPES, replacing the single
                    // `edit.form_create_field` that was drawn and inert. Each
                    // arms a placement tool: click the page for a standard
                    // size, or drag out the exact one, and a dialog collects
                    // the details before anything is authored.
                    //
                    // ★ The order is by how often a form uses them, not
                    // alphabetically and not by engine convenience. Text boxes
                    // outnumber everything else on a real form; the button
                    // comes last because it is the one that cannot yet do
                    // anything.
                    command("edit.form_text_field"),
                    command("edit.form_check_box"),
                    command("edit.form_radio_button"),
                    command("edit.form_choice"),
                    // ★★ Greyed, never absent — the operator's ruling. R9
                    // permits greying for a TEMPORARILY unavailable capability
                    // explained on hover, and this is exactly that: pdfcer can
                    // place a button and cannot yet run what one does. See
                    // `edit_form_push_button_unavailable` for the sentence.
                    command("edit.form_push_button"),
                    command("edit.form_manage_fields"),
                    command("edit.form_flatten"),
                ],
            ),
            // ---------------------------------------------------------------
            // Protect — mark, then apply. See the module header.
            // ---------------------------------------------------------------
            group(
                "protect",
                ribbon::group_edit_protect(),
                [
                    // ★ Large — the mockup's `Redact` big, with the two
                    // qualified redaction verbs in a column beside it. First
                    // in the group already.
                    large("edit.redact"),
                    // ★ Between mark-by-search and Apply, which is the order an
                    // operator works in: find what you can find, mark what you
                    // cannot, then apply once. Putting it after Apply would put
                    // a marking verb on the far side of the destructive one.
                    command("edit.redact_selection"),
                    command("edit.redact_apply"),
                ],
            ),
        ])
}
