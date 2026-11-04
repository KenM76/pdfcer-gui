//! The **Edit** tab — *what am I changing about content that is already
//! there?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ribbontabs/edit.md`.

use super::{command, group, large};
use crate::text::ribbon;
use egui_shell::manifest::{Item, Tab};

/// The Edit tab.
pub fn tab() -> Tab {
    Tab::new("edit", ribbon::tab_edit())
        .with_icon("edit")
        .with_question(ribbon::question_edit())
        .with_groups([
            // ---------------------------------------------------------------
            // Content — the three primary editing tools, relabelled.
            // ---------------------------------------------------------------
            group(
                "content",
                ribbon::group_edit_content(),
                [
                    // FIRST in the group, and the position is the argument.
                    //
                    // An operator reaches for this when something has gone
                    // wrong — an object dragged off the sheet, invisible and
                    // unclickable — and a rescue that is findable only by
                    // knowing `Ctrl+A` is not findable. It sits where the eye
                    // lands first on the tab they are already working in.
                    command("edit.select_all"),
                    command("edit.text"),
                    command("edit.add_text"),
                    // Beside the text verbs it belongs with, and after them:
                    // an operator reflows a paragraph *because* they have just
                    // retyped a sentence in it, so it follows the tool that
                    // does the retyping.
                    command("edit.reflow_block"),
                    command("edit.paragraphs"),
                    command("edit.paragraphs_page"),
                    command("edit.paragraph_here"),
                    // The gap itself, where the text it spaces is edited
                    // (O312). Settings ▸ Text holds the same setting.
                    Item::custom(super::WORD_GAP),
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
            // **`RIBBON_IA.md` names Attachments nowhere**, and this is the
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
            // **Insert rather than a new group**, and the group's own caption
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
                [
                    large("edit.insert_image"),
                    large("edit.insert_link"),
                    large("edit.insert_3d"),
                    large("edit.attachments"),
                ],
            ),
            // Arrange: one Align and Distribute control where `RIBBON_IA.md`
            // once listed two, because Inkscape's is one panel holding both;
            // beside it the seven one-click aligns its Ctrl+Alt+keypad chords
            // run.
            group(
                "arrange",
                ribbon::group_edit_arrange(),
                [
                    large("edit.align"),
                    command("edit.align_left"),
                    command("edit.align_right"),
                    command("edit.align_top"),
                    command("edit.align_bottom"),
                    command("edit.align_centre_x"),
                    command("edit.align_centre_y"),
                    command("edit.align_centre"),
                    command("edit.bring_to_front"),
                    command("edit.bring_forward"),
                    command("edit.send_backward"),
                    command("edit.send_to_back"),
                ],
            ),
            // ---------------------------------------------------------------
            // **Clipboard was here, and it is deleted rather than emptied.**
            //
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
        ])
}
