//! # `text::commands::view` — every label and tooltip on the View tab
//!
//! Split out of [`super`] on 2026-08-20, when that file crossed rule R2's
//! 1,500-line ceiling. The View tab is the largest single section of the
//! catalogue by a wide margin — page display, zoom, the overlays, the nine
//! panel toggles and the window verbs — and it is also the most
//! self-contained: nothing here is read by any other tab's entry, and every
//! entry answers one question, which `RIBBON_IA.md` §3 gives as *"what am I
//! looking at, and how?"*
//!
//! ## It is re-exported, so nothing changed for a caller
//!
//! `super` carries `pub use view::*;`. Every call site still writes
//! `crate::text::commands::view_zoom_in()`, the catalogue's coverage test
//! still walks one list, and the split is a fact about where the source lives
//! rather than about the shape of the module. That is deliberate: a seam that
//! forces a rename at ninety call sites is a seam that will be argued about
//! instead of taken.
//!
//! ## The rules that apply here are `super`'s
//!
//! Sentence case, no trailing period on a label, an ellipsis when activating
//! opens a dialog, a full sentence with punctuation on a tooltip, and — the
//! one that has cost this project real defects — **never state a capability
//! the build does not have**. See that module's header, which is where those
//! are argued.

use super::CommandText;

// ===========================================================================
// VIEW TAB
// ===========================================================================

/// `view.page_single`
#[must_use]
pub const fn view_page_single() -> CommandText {
    CommandText::new(
        "Single page",
        "Show one page at a time. This is pdfcer's default, because paging one drawing sheet at \
         a time is the right model for reading a sheet set.",
    )
}

/// `view.page_continuous`
///
/// The operator's instruction of 2026-08-12 is what this control is, and the
/// tooltip carries its reasoning rather than a feature description: *"continuous
/// scroll should be an option under the view tab as the way I move around a
/// page is great when working with drafting drawings."* So the words say what
/// it is **for** — a document you read through — and leave single page
/// standing as the right answer for a sheet set, which it is.
///
/// The second sentence states the per-document persistence, because it is
/// behaviour the operator cannot see until it surprises them: choosing this on
/// a report and then opening a drawing set must not carry the setting across,
/// and a control that silently remembers something should say so.
#[must_use]
pub const fn view_page_continuous() -> CommandText {
    CommandText::new(
        "Continuous",
        "Scroll through every page in one run, for a document you read rather than a sheet set \
         you page through. pdfcer remembers this choice for this document, so another file keeps \
         its own.",
    )
}

/// `view.page_facing`
#[must_use]
pub const fn view_page_facing() -> CommandText {
    CommandText::new(
        "Facing",
        "Show two pages side by side, as an open book. The first page sits alone, so every \
         later spread pairs the way a bound document does.",
    )
}

/// `view.page_facing_continuous`
#[must_use]
pub const fn view_page_facing_continuous() -> CommandText {
    CommandText::new(
        "Facing continuous",
        "Scroll through every spread in one run — facing pages, without stopping at each one.",
    )
}

/// `view.panel_pages`
///
/// The one panel toggle whose tooltip has to say what the panel is *for*
/// rather than what it contains: a grid of thumbnails is self-explanatory to
/// look at and not to read about, so the sentence spends its words on the two
/// verbs — go to a page, and act on several at once — that are not obvious
/// from the picture.
#[must_use]
pub const fn view_panel_pages() -> CommandText {
    CommandText::new(
        "Pages",
        "Show or hide the panel of page thumbnails: click one to go there, and pick several to \
         act on them together.",
    )
}

/// `view.zoom_actual`
#[must_use]
pub const fn view_zoom_actual() -> CommandText {
    CommandText::new(
        "Actual size",
        "Show the page at actual size — one PDF point per screen point (Ctrl+0).",
    )
}

/// `view.zoom_selection`
#[must_use]
pub const fn view_zoom_selection() -> CommandText {
    CommandText::new(
        "Zoom to selection",
        "Scale and centre the view on what is selected.",
    )
}

/// `view.zoom_region`
///
/// The tooltip says *"drag"* because arming this command does not zoom
/// anything — it changes what the next drag on the page means. A control
/// that arms rather than acts has to say so, or its first press reads as
/// broken.
#[must_use]
pub const fn view_zoom_region() -> CommandText {
    CommandText::new(
        "Zoom to region",
        "Drag a rectangle on the page to zoom to it. The selection is left alone.",
    )
}

/// `view.tool_hand`
#[must_use]
pub const fn edit_cut() -> CommandText {
    CommandText::new(
        "Cut",
        "Copy what is selected and remove it \u{2014} a comment, a shape on the page, or a \
         form field. One Ctrl+Z brings it back.",
    )
}

/// `edit.copy` — the object clipboard's copy.
///
/// ★ The tooltip names **what it copies**, not what it does, because the honest
/// scope is narrower than the word "copy" promises: `EditSession` has no verb
/// that puts page content back, so a copied path could never be pasted. Saying
/// *"comment or markup"* is the difference between a control that under-promises
/// and one an operator discovers is a lie the first time they try it on a line.
#[must_use]
pub const fn edit_copy() -> CommandText {
    CommandText::new(
        "Copy",
        "Copy what is selected \u{2014} a comment, a shape on the page, or a form field. \
         Ctrl+C over selected TEXT copies the text instead.",
    )
}

/// `edit.paste` — the object clipboard's paste.
///
/// ★ The tooltip was rewritten on 2026-08-29 and the old wording is worth
/// recording, because it had quietly become false twice: it said *"the copied
/// comment or markup"* after `Pass 120.0` made page content pasteable and again
/// after form fields joined the clipboard. A tooltip that names a NARROWER scope
/// than the command has is the same defect class as one that names a wider one —
/// the operator never tries the thing that would have worked.
#[must_use]
pub const fn edit_paste() -> CommandText {
    CommandText::new(
        "Paste",
        "Put what you copied on this page. A copied form field arrives as a NEW field \
         with its own value. On the page it came from it lands slightly offset so you \
         can see it; on any other page it lands where it was.",
    )
}

/// `edit.paste_duplicate` — the second sense of a form-field paste.
///
/// **Ken, 2026-08-29:** *"ctrl shift v for paste as duplicate."*
///
/// ★★★ The tooltip leads with the CONSEQUENCE — *"typing in one fills both"* —
/// rather than with the mechanism (*"a second widget of the same field"*), which
/// is a sentence about PDF structure. The consequence is the thing that will
/// surprise him at the printer, and it is the ONLY disclosure there is: two
/// linked boxes and two independent boxes are pixel-identical on the page.
///
/// ★ It says what happens over a non-field clipboard too, because the command is
/// not withheld there — it falls through to an ordinary paste, and a control that
/// silently means something else is worse than one that says so.
#[must_use]
pub const fn edit_paste_duplicate() -> CommandText {
    CommandText::new(
        "Paste as duplicate",
        "Paste a copied form field as ANOTHER BOX FOR THE SAME FIELD \u{2014} typing in one \
         fills both, and it keeps the original's font, colour and any calculation. For \
         anything else on the clipboard this is an ordinary paste.",
    )
}

/// `edit.copy_as_vector` — the clipboard's copy-OUT.
///
/// **Ken, 2026-09-03** (`OPERATOR_REQUESTS.md` **O120**): *"Also I'd like to be
/// able to copy and paste anything to other software - like copy and paste
/// vector graphics into word or inkscape for example if possible."*
///
/// ★★★ The tooltip leads with **what arrives at the other end**, because that is
/// the only thing that distinguishes this from the Copy beside it. Ordinary Copy
/// puts an internal clip on the clipboard plus a picture; this puts the
/// *geometry*, so what lands in the next drawing is still line-work rather than
/// a photograph of line-work. An operator cannot tell those apart by looking at
/// the paste — they find out when they try to recolour it — so the difference
/// has to be stated before the press rather than discovered after it.
///
/// ★ It names Word and Inkscape by name, which this catalogue does sparingly,
/// because they are the two applications the operator named and because the
/// promise is *specifically* about them: the format order was measured against a
/// real Word paste, and Inkscape's own preference list is where the SVG's
/// position came from. A vaguer "other programs" would be a weaker claim than
/// the one that was actually tested.
///
/// ★★ It says what the operand is — selection if there is one, page otherwise —
/// because that is the one thing about this command that is not visible on the
/// button, and getting a whole sheet when three parts were selected is the
/// surprise worth spending a clause on.
#[must_use]
pub const fn edit_copy_as_vector() -> CommandText {
    CommandText::new(
        "Copy as vector",
        "Copy the selection \u{2014} or this whole page, if nothing is selected \u{2014} as \
         EDITABLE GEOMETRY rather than as a picture of it. Paste into Word, PowerPoint or \
         Inkscape and the line-work can still be scaled, recoloured and taken apart.",
    )
}

/// `pages.copy` — copy the picked sheets.
///
/// ★ The tooltip names the OPERAND RULE, because it is the one thing an
/// operator cannot see: with sheets picked it copies those, with none picked it
/// copies the one they are looking at. Every `pages.*` verb works that way and
/// none of them says so anywhere else.
#[must_use]
pub const fn pages_copy() -> CommandText {
    CommandText::new(
        "Copy pages",
        "Copy the sheets picked in the Pages panel, or the current sheet if none are picked. \
         What pdfcer holds is a complete PDF, so you can paste it here or into another drawing.",
    )
}

/// `pages.cut` — copy the picked sheets and remove them.
#[must_use]
pub const fn pages_cut() -> CommandText {
    CommandText::new(
        "Cut pages",
        "Copy the picked sheets and remove them from this drawing. One Ctrl+Z brings them back.",
    )
}

/// `pages.paste` — put copied sheets in after the current one.
///
/// ★★ It says WHERE, because a page paste changes the document dramatically and
/// an operator who cannot predict where the sheets land will not use it twice.
#[must_use]
pub const fn pages_paste() -> CommandText {
    CommandText::new(
        "Paste pages",
        "Put the copied sheets in after the one you are looking at. Copied form fields may \
         arrive as boxes nothing can fill \u{2014} pdfcer says so if they do.",
    )
}

/// `edit.redact_selection` — mark what is selected, without searching for it.
///
/// ★★★ The tooltip's second sentence is the whole control. *"Nothing is removed
/// until you apply"* is the fact that decides whether an operator trusts this
/// button or is frightened of it, and it is also the fact that stops them
/// believing the job is done. A redaction that was marked and never applied is
/// a document that still contains every word.
///
/// ★ The first sentence names what the search box cannot reach, because that is
/// why this exists: on a CAD drawing a title-block value is often vector
/// strokes and a stamp is often an image, and neither is findable by typing.
#[must_use]
pub const fn edit_redact_selection() -> CommandText {
    CommandText::new(
        "Redact selection",
        "Mark whatever you have selected \u{2014} a shape, an image, a piece of text \u{2014} to be \
         removed. Use this for anything the search box cannot find, like a drawn title block or \
         a scanned stamp. Nothing is removed until you apply the redactions.",
    )
}

/// See the module header.
#[must_use]
pub const fn view_tool_select() -> CommandText {
    CommandText::new(
        "Select",
        "Click a shape to select it, drag to move it, drag on empty paper to select several. \
         The tool everything returns to.",
    )
}

/// The **Node tool** — the white arrow.
///
/// ★ Named *Points*, not *Node*, and not *Direct selection*. "Node" is this
/// program's internal word (`SelectionLevel::Node`); a draughtsman says
/// *point*, and `text::commands`' standing rule is that a label is the
/// operator's vocabulary and an id is the format's. Illustrator's own name for
/// it — "Direct Selection" — describes the mechanism rather than the subject
/// and has confused people for thirty years.
#[must_use]
pub const fn view_tool_node() -> CommandText {
    CommandText::new(
        "Points",
        "Click a shape to show its points, then click one and drag to move it. Shift-click to \
         take several. A point on a curve also shows its handles.",
    )
}

/// See the module header.
#[must_use]
pub const fn view_tool_hand() -> CommandText {
    CommandText::new(
        "Hand",
        "Drag to pan the page instead of selecting. Hold Space to pan without switching tools.",
    )
}

/// `view.tool_text`
///
/// The tooltip says *"drag"* and names what the tool **replaces**, for the
/// reason `view.zoom_region`'s does: this command arms rather than acts, so its
/// first press changes nothing an operator can see except the pointer, and a
/// control that arms has to say so or it reads as broken.
///
/// It names the marquee explicitly because that is the trade an editor is
/// making — the primary drag stops selecting objects for as long as this is
/// pressed — and because in Read and Review the tool is redundant (the select
/// tool already sweeps text there), so the sentence has to be true in a mode
/// where it changes nothing as well as in the one where it changes the most.
#[must_use]
pub const fn view_tool_text() -> CommandText {
    CommandText::new(
        "Text",
        // ★ Rewritten 2026-08-19. It used to say only "drag to select text",
        // which was the whole of what the tool did and was exactly the trap the
        // operator fell into: he armed it, got an I-beam, clicked, and could not
        // type. The click now edits, so the tooltip leads with that — the
        // sentence a control shows is a promise, and this one was keeping a
        // smaller promise than the operator was reading into it.
        "Click text to edit it, or click empty space to start new text. Drag to select text \
         for copying. Press again to return to the select tool.",
    )
}

/// `view.zoom_fit_page`
#[must_use]
pub const fn view_zoom_fit_page() -> CommandText {
    CommandText::new(
        "Fit page",
        "Scale the page so all of it is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.zoom_fit_width`
#[must_use]
pub const fn view_zoom_fit_width() -> CommandText {
    CommandText::new(
        "Fit width",
        "Scale the page so its full width is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.zoom_fit_height`
///
/// ★ Deliberately the same sentence as its two siblings with one word
/// changed. The three fit modes differ in exactly one respect and the copy
/// says so; three separately-worded tooltips would invite the reader to hunt
/// for a distinction that is not there.
#[must_use]
pub const fn view_zoom_fit_height() -> CommandText {
    CommandText::new(
        "Fit height",
        "Scale the page so its full height is visible, and keep it fitted as the window resizes.",
    )
}

/// `view.show_annotations`
#[must_use]
pub const fn view_show_annotations() -> CommandText {
    CommandText::new(
        "Annotations",
        "Show or hide the markup, stamps and form-field appearances stored in this document, \
         so the page content can be seen alone.",
    )
}

/// `view.show_points`
#[must_use]
pub const fn view_show_points() -> CommandText {
    CommandText::new(
        "Points",
        "Show the editable points of every part of the object you are working inside, not just \
         the part you have selected. Points always appear for the selected part.",
    )
}

/// `view.smart_select`
///
/// ★ The tooltip states what a **click** does and what a **double-click** does,
/// because those are two halves of one rule and an operator told only the first
/// concludes the second is not offered — the failure
/// `panels::tool::armed`'s form-field instruction records.
#[must_use]
pub const fn view_smart_select() -> CommandText {
    CommandText::new(
        "Smart select",
        "Click selects a whole thing — a title block, a stamped drawing, a symbol — instead of one line inside it. Double-click goes inside, as many times as it takes to reach the part you want. Escape steps back out.",
    )
}

/// `view.rulers`
///
/// The tooltip states the unit, because the unit is the thing an operator
/// cannot see until they have already trusted a number. `crate::canvas::rulers`'
/// header §1 carries the decision in full: points is what the file itself
/// says, and a document that has been given a measurement scale reads in that
/// scale's units instead — through the same formatter a dimension label uses,
/// so the ruler and a dimension across the same span agree to the digit.
///
/// The second sentence also does the discoverability work for the guides:
/// dragging out of a ruler is how a guide is made, and that is not something
/// an operator finds by looking at the guides button.
#[must_use]
pub const fn view_rulers() -> CommandText {
    CommandText::new(
        "Rulers",
        "Show rulers along the top and left of the page, reading in points — or in this \
         document's own units if a measurement scale has been set for it. Drag out of a ruler \
         to place a guide.",
    )
}

/// `view.grid`
///
/// "Over the page" rather than "over the canvas", deliberately: the grid is
/// anchored to each sheet and scrolls with it, which is the whole of the
/// decision in `crate::canvas::rulers`' header §2, and the wording is what
/// tells an operator that before they scroll and find out.
#[must_use]
pub const fn view_grid() -> CommandText {
    CommandText::new(
        "Grid",
        "Draw a grid over each page, spaced to match the rulers so its heavier lines fall on \
         the numbered marks. The grid belongs to the sheet and scrolls with it.",
    )
}

/// `view.guides`
///
/// Names all three verbs — place, move, remove — because a guide is the one
/// control in this group that the operator *does something to*, and a toggle
/// whose tooltip only says "show or hide" would leave them with lines they
/// cannot get rid of. The persistence is stated for the same reason
/// `view.page_continuous`' tooltip states its own: it is behaviour the
/// operator cannot see until it surprises them.
#[must_use]
pub const fn view_guides() -> CommandText {
    CommandText::new(
        "Guides",
        "Show the guide lines placed on this document's pages, and let them be moved. Drag one \
         out of a ruler to place it, drag it off the page to remove it, or double-click it. \
         pdfcer remembers a document's guides for the next time you open it.",
    )
}

/// `view.line_weights`
///
/// ★★★ **`OPERATOR_REQUESTS.md` O137, asked for by name** — *"the button to
/// show all lines without their thickness — thin lines or something like cad
/// has. The button never worked but I do want that display option!"*
///
/// # ★★★ Every clause of this tooltip is doing a job, and three of them are
/// defences against a specific misreading
///
/// **The label is "Line weights", not "Thin lines" or "Hairlines".** It names
/// the thing the switch governs, so its two positions read as *on* and *off* —
/// which is Acrobat's spelling (**View ▸ Line Weights**, checked by default)
/// and AutoCAD's (`LWDISPLAY`). A label naming the *off* state would render
/// pressed when line weights were being shown, which is backwards. It is also
/// the phrase a draughtsman already has: he asked for "lines without their
/// thickness", and thickness on a drawing is a *weight*.
///
/// **"Turn this off"** leads, because turning it off is the whole feature. On
/// is what pdfcer already did.
///
/// **"one pixel wide"**, not "thin" or "hairline". Thin is relative and invites
/// the opposite reading — Acrobat's *enhance thin lines*, which makes thin
/// things THICKER — and this is the other convention entirely. One device
/// pixel is what it actually does, and it is checkable.
///
/// **"however wide the file says they are"** says the ceiling applies to
/// everything, so an operator does not expect only the fat ones to change.
///
/// **"Filled shapes and hatching are not affected"** is not padding. Only
/// stroked paths reach the engine's stroke width; a region built out of thin
/// *fills* keeps every pixel. Without the sentence, an operator whose hatch is
/// fill-based would turn this on, see the hatch unchanged, and conclude the
/// control is half-broken.
///
/// **"Printing, exporting and the print preview always use the real
/// widths"** is the constraint the whole feature was built around, and it is
/// stated *here* rather than only in the status bar because this is where he
/// decides whether to press it. The status line says the same thing while it is
/// on; a disclosure the operator meets only after acting is half a disclosure.
///
/// # ★ Why there is no Settings entry for it (a decision, not an omission)
///
/// The 2026-08-17 sweep moved the two surviving `view.*` settings into
/// Settings ▸ Drawing the page on the argument that *"a value set once and
/// forgotten is not an activity"*. This is the other case: it is a **reading
/// aid he flips several times while reading one sheet** — turn it off to see
/// whether two lines are coincident, turn it back on to check a drafting
/// weight — so it is an activity, and P2 says a ribbon tab picks those. It is
/// also per **document** ([`crate::viewer::ViewState`]), so two open drawings
/// can disagree, which is what comparing a sheet against its neighbour needs.
///
/// A *persisted* default would additionally mean pdfcer opening every drawing
/// for the rest of time showing something the file does not say, because of a
/// switch set weeks earlier — one step removed from the failure O137 forbids
/// outright. If he asks for it to stick, it belongs in
/// `crate::app::prefs::opening::PageChrome` beside the other three view
/// toggles, seeded by `Prefs::seed_view`, and that is the whole of the work.
#[must_use]
pub const fn view_line_weights() -> CommandText {
    CommandText::new(
        "Line weights",
        "Turn this off to draw every line one pixel wide, however wide the file says they are \
         — the CAD way of reading a dense drawing, so lines that sit close together stop \
         merging into one black bar when you zoom in. Filled shapes and hatching are not \
         affected. Printing, exporting and the print preview always use the real widths.",
    )
}

/// `view.sidebar`
#[must_use]
pub const fn view_sidebar() -> CommandText {
    CommandText::new(
        "Sidebar",
        "Show or hide the left panel — page thumbnails and the active tool's options.",
    )
}

// ★★★ `view_panel_tool` was retired on 2026-09-04 — `OPERATOR_REQUESTS.md`
// O123 dissolved the panel it labelled. Its argument was that the caption had
// to be *Tool* and not *Tool options*, because the two text tools that
// produced the panel have no options at all; that argument is now settled the
// other way round, since the options the panel DID hold turned out to be
// properties and moved to the Properties panel.

/// `view.panel_bookmarks`
#[must_use]
pub const fn view_panel_bookmarks() -> CommandText {
    CommandText::new(
        "Bookmarks",
        "Show the document's bookmarks. Click one to jump to its page.",
    )
}

/// `view.panel_layers`
///
/// ## ★ Reworded at S4, because the old tooltip undersold a capability
///
/// It read:
///
/// > Show the document's layers and which of them a reader draws by default.
///
/// which was a claim about the document with **no verb in it** — accurate for
/// the S3 panel, which was a report. S4 restored the visibility control
/// (`crate::app::actions::Action::SetLayerVisible`), and a tooltip that
/// describes a panel as read-only when it is not costs the operator the
/// capability: they read it, conclude there is nothing to click, and never
/// open the panel.
///
/// The new wording follows [`view_panel_bookmarks`]'s shape — *what it shows,
/// then what you can do in it* — because that is the shape of the only other
/// panel in this build with a verb, and two panels that answer the same
/// question should answer it the same way.
///
/// The third clause is not padding. It is the same boundary
/// `crate::text::panels::layers_session_only_note` states inside the panel,
/// and it is repeated here because the ribbon tooltip is read **before** the
/// panel opens — which is the moment an operator decides whether clicking
/// this is a safe thing to do to someone else's file.
#[must_use]
pub const fn view_panel_layers() -> CommandText {
    CommandText::new(
        "Layers",
        "Show the document's layers, and switch any of them on or off while you look at it. \
         The document is not changed.",
    )
}

/// `view.panel_signatures`
///
/// ★★★ **CORRECTED 2026-09-05.** The second sentence read *"pdfcer does not
/// check whether they are valid."* It became false when
/// `signature::verify_all_with_trust` was wired (`crate::trust::examine`,
/// engine `pdfcer-core` v0.38.0 at `b01964f`), and it was **missed by the
/// sweep that corrected the panel's own explainer** — `crate::text::panels`
/// deleted the near-identical sentence the same day and left this one
/// standing, because the feature work opened that file and never opened this
/// one. The tooltip is the surface read *before* the panel opens, so it was
/// the worse of the two to leave wrong.
///
/// The replacement states the three facts the panel actually draws rather
/// than a verdict, because the panel never folds them into one.
#[must_use]
pub const fn view_panel_signatures() -> CommandText {
    CommandText::new(
        "Signatures",
        "Show each digital signature: what it covers, whether the bytes it covers were \
         altered, and whether the signer is one you have chosen to trust.",
    )
}

/// `view.panel_objects`
#[must_use]
pub const fn view_panel_objects() -> CommandText {
    CommandText::new(
        "Objects",
        "Show or hide the right-hand panel listing everything on the page, nested into parts \
         and points.",
    )
}

/// `view.panel_forms`
///
/// ## ★ Was `edit.form_fill`, moved on the operator's answer (2026-08-14)
///
/// The taxonomy's argument for keeping form-filling out of Read is recorded
/// in [`crate::app::modes`]; the operator's answer is that Acrobat Reader
/// fills forms in its default view and replacing it is the stated goal. A
/// command lives on exactly one tab (P1), and Read is shown `file` and
/// `view` alone — so the fill verb had to move to a tab Read has, and this
/// is that move.
///
/// **The label stays a verb while the id became a panel toggle**, which is
/// the one thing about this that looks inconsistent and is not. The id
/// names what the command *does to the shell* — it shows the Forms panel,
/// exactly as its five siblings in View ▸ Panels do, and it inherits their
/// mechanism rather than inventing a second one. The label names what the
/// operator *came to do*, and nobody opens this panel to look at a list of
/// field names. Renaming it "Forms" to match its neighbours would file the
/// capability under a noun the person looking for it is not searching for.
#[must_use]
pub const fn view_panel_forms() -> CommandText {
    CommandText::new(
        "Fill form",
        "List this document's fillable fields and type into them. Nothing is written to disk \
         until you save.",
    )
}

/// `view.read_mode`
#[must_use]
pub const fn view_read_mode() -> CommandText {
    CommandText::new(
        "Read mode",
        "Hide the ribbon and the panels and give the whole window to the page (Ctrl+H).",
    )
}

/// `view.fullscreen`
#[must_use]
pub const fn view_fullscreen() -> CommandText {
    CommandText::new("Full screen", "Fill the whole display with pdfcer (F11).")
}

/// `view.next_document`
///
/// ★ The tooltip names the chord because that is how this command will
/// actually be used. Ctrl+Tab is the gesture; the button is the thing that
/// tells an operator the gesture exists, which is why the button is on the
/// ribbon at all when nobody will click it twice.
#[must_use]
pub const fn view_next_document() -> CommandText {
    CommandText::new(
        "Next document",
        "Show the next open document (Ctrl+Tab). Wraps round at the end.",
    )
}

/// `view.previous_document`
#[must_use]
pub const fn view_previous_document() -> CommandText {
    CommandText::new(
        "Previous document",
        "Show the previous open document (Ctrl+Shift+Tab). Wraps round at the start.",
    )
}

/// `view.close_other_documents`
///
/// ★ The label says which one **survives**, not how many go. *"Close others"*
/// is what every browser and every editor calls it, and an operator reading it
/// in a menu opened on a specific tab already knows which that is — which is
/// what makes the short wording safe rather than merely terse.
///
/// ★★ The tooltip has to say **which** *this one* is, because the command has
/// two routes with two operands: from a tab's context menu it keeps the tab
/// that was right-clicked, and from the ribbon it keeps the one on screen. So
/// it says *"the one you opened this on"* rather than naming either, which is
/// true from both and misleading from neither.
#[must_use]
pub const fn view_close_other_documents() -> CommandText {
    CommandText::new(
        "Close others",
        "Close every open document except the one you opened this on. Any with unsaved \
         edits are asked about one at a time (Ctrl+W closes just the one you are \
         looking at).",
    )
}

/// `view.reset_layout`
///
/// ★ **This entry lost an ellipsis and a promise, and both losses are the
/// same correction.** It used to read *"Reset layout…"* / *"Put the panels
/// back where they started. You choose which ones — the left panel, the
/// right panel, or just whether they are open."*
///
/// The choice is real and specified — `RIBBON_IA.md`: *"an operator who only
/// wanted the right dock back must not lose their left one"* — and
/// `egui_shell::layout::ResetScope` implements all three scopes. What does
/// not exist is anywhere to **ask**: this build has no modal, no popup and no
/// split-button item kind, so the command was wired to `ResetScope::All` (see
/// `crate::app::PdfcerApp::dispatch_command`, whose arm records what a chooser
/// would take). A tooltip offering a choice the operator is never given is
/// exactly the "never state a capability the build does not have" failure
/// this catalog's header forbids, and the trailing `…` made the same promise
/// in punctuation.
///
/// **Restore both the moment the chooser lands** — the original wording is
/// quoted above so that is a copy rather than a rewrite.
/// **Float this panel** — tear it out of the dock into a window of its own.
///
/// # The words, and the two that were rejected
///
/// *"Float"* rather than *"Undock"* or *"Tear out"*.
///
/// **"Undock"** names the thing that stops happening rather than the thing
/// that starts. An operator reading a menu is looking for what they will
/// get, and what they get is a window — the fact that it left the dock is
/// how, not what.
///
/// **"Tear out"** is the gesture's name in every product that implements
/// this as a drag, and this one is not a drag: `MODES_AND_PANELS.md`
/// specifies *"a stationary Float this panel… command rather than
/// drag-to-tear"*, and a menu row named after a gesture that does not
/// exist here would teach the wrong thing about the interface.
///
/// ★ **No ellipsis.** It acts; it does not ask. The convention this
/// catalog follows is that an ellipsis means a dialog is coming.
#[must_use]
pub const fn view_panel_float() -> CommandText {
    CommandText::new(
        // ★ "Float panel" and not "Float". The label has to be unique across
        // the whole registry (`no_two_commands_share_a_label`), and the noun
        // earns its place beyond that test: this row sits in a menu beside
        // "Reset layout", which acts on the whole dock, so saying which
        // subject each row has is what stops the two reading as a pair of
        // options on one thing.
        "Float panel",
        "Move this panel into a window of its own, which you can put anywhere - including on \
         another monitor. Dock puts it back where it came from.",
    )
}

/// **Dock this panel** — put a floating panel back where it came from.
///
/// The tooltip promises *where it came from* rather than *back in the
/// dock*, because that is the property the implementation actually holds
/// and the one an operator would otherwise have to test to find out. See
/// `egui_shell::dock::float`'s header for why putting it back "somewhere
/// sensible" was rejected.
#[must_use]
pub const fn view_panel_dock() -> CommandText {
    CommandText::new(
        "Dock panel",
        "Put this panel back in the dock, in the same place it was when you floated it.",
    )
}

/// **Close this panel** — take it off screen entirely.
///
/// ★★ The tooltip names the way back, and that is not padding. Closing is
/// the only one of the three verbs that leaves no visible trace of the
/// panel anywhere, so it is the only one where an operator can be left
/// wondering whether they have lost something. Naming the View tab costs
/// a clause and removes the whole question.
#[must_use]
pub const fn view_panel_close() -> CommandText {
    CommandText::new(
        // ★★ "Close panel", not "Close" — and here the noun is load-bearing
        // rather than merely tidy. `file.close` is already labelled "Close"
        // and closes the DOCUMENT. Two rows reading "Close", one of which
        // discards a panel and the other of which can discard unsaved work,
        // is a collision the operator pays for and not one the registry does.
        "Close panel",
        "Take this panel off screen. You can bring it back from the View tab.",
    )
}

/// **Dock all floating panels** — the way back to a window you cannot
/// reach.
///
/// # ★★★ Why this exists as a command of its own
///
/// A floating panel lives in an OS window at a remembered desktop
/// position. Unplug the monitor that position was on and the window is
/// still open, still in the layout, and **unreachable**: it cannot be
/// dragged, it cannot be closed, and it cannot be floated again because it
/// already is.
///
/// Every other route out of that state needs the operator to act *on the
/// window*. This one acts on all of them at once, from the application
/// window, which is the one surface guaranteed to be on a monitor that
/// exists.
///
/// ★★ Reset layout is the other route and it is stronger — it also
/// restores the arrangement. This one is the *cheap* route: it costs the
/// operator nothing they arranged. Offering both is the two-tier shape
/// `MODES_AND_PANELS.md` singles out as the thing the best product in its
/// benchmark table got right.
///
/// ★ Greyed rather than hidden when nothing is floating, and that is R9
/// applied rather than R9 broken: this is *temporarily* unavailable —
/// there is simply nothing to dock this second — and the hover says so.
/// Hiding it would make the remedy invisible exactly until the operator
/// needs it, which is the wrong half of the cycle to be visible in.
#[must_use]
pub const fn view_dock_all_panels() -> CommandText {
    CommandText::new(
        "Dock all panels",
        "Bring every floating panel back into the dock. Use this if a panel window has ended up \
         on a monitor you no longer have.",
    )
}

/// **Auto-hide the ribbon** — his instruction of 2026-09-05.
///
/// The wording says what he will SEE, not what the setting is called: the tab
/// names stay, the buttons go until the pointer arrives, and the drawing does
/// not move. The last clause is there because it is the property that makes the
/// setting usable rather than nauseating, and an operator deciding whether to
/// turn it on cannot know it otherwise.
#[must_use]
pub const fn view_ribbon_auto_hide() -> CommandText {
    CommandText::new(
        "Auto-hide ribbon",
        "Keep the row of tab names and hide the buttons under it until you move the pointer \
         onto that row. The buttons then appear OVER the drawing, so nothing you were about to \
         click moves. Press this again to keep them showing.",
    )
}

/// **Auto-hide the left strip** — the same instruction, the other surface.
#[must_use]
pub const fn view_rail_auto_hide() -> CommandText {
    CommandText::new(
        "Auto-hide left strip",
        "Shrink the strip of panel and tool buttons down the left edge to a narrow marked band, \
         and bring it back when you move the pointer onto that band. It appears OVER the panel \
         beside it, so the panel does not change width. Press this again to keep it showing.",
    )
}

#[must_use]
pub const fn view_reset_layout() -> CommandText {
    CommandText::new(
        "Reset layout",
        "Put both panel docks back where this mode started them. Your other modes keep the \
         arrangements you gave them.",
    )
}
