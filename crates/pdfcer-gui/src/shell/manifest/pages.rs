//! The **Pages** tab — *what am I doing to the set of pages?*
//!
//! `RIBBON_IA.md` §5.3. Three groups: Insert, Organise, Transform.
//!
//! # Why this tab exists at all
//!
//! It is the largest structural change in the new layout, and it is a
//! discoverability fix rather than a feature:
//!
//! > **Page operations are hidden.** Insert, delete, extract, reorder,
//! > split and merge exist and work, but live in the thumbnail rail's
//! > selection action bar and in a `Tools ▸ Batch` pane. Nothing on any
//! > ribbon tab says "pages". A user who wants to delete page 7 has no
//! > path that starts at the ribbon.
//!
//! Every command here already worked. What was missing was a name for the
//! place they live.
//!
//! # The organising rule, and the line it draws against Tools
//!
//! Every command on this tab operates on **the current document's page
//! set**, and every one of them respects the thumbnail rail's current
//! selection when there is one. That is what distinguishes it from Tools:
//! **Pages changes *this* document; Tools produces *new* files.**
//!
//! The rule matters because two pairs of commands would otherwise look
//! like duplicates and are not:
//!
//! | Here | On Tools | Difference |
//! |---|---|---|
//! | `pages.split` | `tools.split_files` | this document, at boundaries you choose · one or more files on disk, originals untouched |
//! | `pages.merge_into` | `tools.merge_files` | adds pages to this document · combines files into a new one |
//!
//! They are four distinct commands with four distinct ids, and their
//! tooltips point at each other so an operator who reached for the wrong
//! one is told where the other lives. This is not a P1 violation: P1 is
//! about one command with two homes, and these are two commands.
//!
//! # The thumbnail rail keeps its action bar
//!
//! Also not a P1 violation — the rail is a panel, not a tab, and a
//! selection-scoped action bar next to the selection is correct. The
//! ribbon becomes the *discoverable* path and the rail stays the *fast*
//! path. The same relationship the QAT has to the File tab.
//!
//! # What is absent
//!
//! The whole **Stamp** group — watermark, header & footer, Bates numbering
//! — is **N**, so the group is not here at all rather than here and empty.
//! `Insert blank` is **C**: `pdfcer-core` can do it and no GUI reaches it.
//! Crop, Resize, Replace and Insert scan are **N**. All are in
//! [`super::PLANNED`].

use super::{command, group, icon_only, large};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The Pages tab.
pub(super) fn tab() -> Tab {
    Tab::new("pages", ribbon::tab_pages())
        .with_question(ribbon::question_pages())
        .with_groups([
            // ---------------------------------------------------------------
            // Insert — new pages arriving from somewhere.
            //
            // One of the three specified commands exists. `Insert blank`
            // is **C** and `Insert scan` is **N**.
            // ---------------------------------------------------------------
            group(
                "insert",
                ribbon::group_pages_insert(),
                [large("pages.insert_from_file")],
            ),
            // ---------------------------------------------------------------
            // Organise — the existing rail commands, given a ribbon home.
            //
            // Order follows the specification: destructive first (delete,
            // extract), then reordering, then the two document-level
            // structural operations. It reads oddly to put Delete first
            // and it is deliberate — this is the band a user comes to
            // *because* they want to remove a page, and burying the
            // command they came for under four they did not is how a
            // discoverability fix fails to fix anything.
            //
            // `Replace…` is **N** and would sit after Extract.
            // ---------------------------------------------------------------
            // ---------------------------------------------------------------
            // ★★ Clipboard — O59 item 2, 2026-08-29.
            //
            // Its own band rather than three more entries in Organise, and the
            // reason is the same one that put Delete first in that band: an
            // operator comes to a band because of what it is called. Cut, Copy
            // and Paste under a caption reading *"Organise"* are three commands
            // nobody scanning for a clipboard would look at.
            //
            // ★ Before Organise, because a copy is the non-destructive one and
            // because Organise's own note explains that IT leads with the
            // destructive verb deliberately -- putting a second destructive band
            // in front of it would undo that argument.
            //
            // ★★★ These are the only clipboard controls in the program that are
            // NOT also a chord, and that is not an omission. `Ctrl+C` belongs to
            // the canvas: the `pages.*` operand rule always resolves -- picked
            // sheets, else the current one -- so a chord rung consulting it
            // would answer *yes* on every document and take the clipboard from
            // the canvas permanently. See `app::dispatch::pageclip`.
            // ---------------------------------------------------------------
            group(
                "page-clipboard",
                ribbon::group_pages_clipboard(),
                [
                    // ★ All three large, 2026-09-04 — the mockup's page
                    // Clipboard group is three big controls and nothing else.
                    // Whole group promoted, so the Cut / Copy / Paste triad
                    // an operator reaches for by position keeps its order.
                    large("pages.cut"),
                    large("pages.copy"),
                    large("pages.paste"),
                ],
            ),
            group(
                "organise",
                ribbon::group_pages_organise(),
                [
                    command("pages.delete"),
                    command("pages.extract"),
                    command("pages.move_up"),
                    command("pages.move_down"),
                    // ★★★ `pages.split` was HERE until 2026-08-31 — O68.
                    // Unregistered with `tools.split_files`; see
                    // `catalog::tools` for the argument. R9: nothing is drawn
                    // until the boundary chooser exists.
                    command("pages.merge_into"),
                ],
            ),
            // ---------------------------------------------------------------
            // Transform — changing a page rather than the set of them.
            //
            // Rotate left/right move here from Edit ▸ Pages. That is one
            // of the three moves `RIBBON_IA.md` §7 flags as visible to a
            // returning user, and it is the least contentious of them: a
            // group named `Pages` on a tab named `Edit` that contained
            // *only* rotate was what made the absence of every other page
            // operation loudest.
            //
            // ★★★ Crop is still **N**. **Resize is not** — 2026-09-06.
            //
            // `pages.resize` is the third control in this band and the first
            // that changes a page's *paper* rather than its orientation, which
            // is why it sits here and not in Organise: Organise is about the
            // set of sheets, Transform is about a sheet.
            //
            // ★ Labelled (`command`) where its two neighbours are `icon_only`,
            // and that asymmetry is deliberate rather than an oversight. Rotate
            // left and rotate right are a *pair* an operator finds by shape and
            // position, and a glyph is enough for them. This one opens a window
            // that will crop his drawing if he is not careful, and there is no
            // icon in the catalogue that says "sheet size" — so it says it in
            // words. `large` is not available here anyway:
            // `large_items_already_lead_their_group` forbids promoting an item
            // that does not lead its group, and this one is third.
            //
            // It reuses the `page-single` glyph, shared with
            // `view.page_single` under the catalogue's shared-key convention —
            // the two are never drawn together, because one tab's band shows at
            // a time. A key of its own was not available: `icons::catalog` is
            // at 1,498 of R2's 1,500 lines.
            // ---------------------------------------------------------------
            group(
                "transform",
                ribbon::group_pages_transform(),
                [
                    icon_only("pages.rotate_left"),
                    icon_only("pages.rotate_right"),
                    command("pages.resize"),
                ],
            ),
        ])
}
