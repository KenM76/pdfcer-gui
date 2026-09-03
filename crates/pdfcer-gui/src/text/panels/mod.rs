//! # `text::panels` — every string the dock's panels show
//!
//! One area of the catalog described in [`crate::text`]'s header. It covers
//! the panel bodies in [`crate::panels`]. The three document-structure
//! panels are in this file; Comments, Fonts, Objects and Properties each have
//! their own module, and Forms and Pages have their own areas one level up.
//!
//! | Module | Panel |
//! |---|---|
//! | `mod.rs` (this file) | the three **document-structure** panels — Bookmarks, Layers, Signatures — plus [`byte_size`], which several areas share |
//! | [`attachments`] | the Attachments panel — the files a document carries inside itself, and the four verbs opposite them |
//! | [`comments`] | the Comments panel — every annotation on the document, what each one is, and the five disclosures a row can carry |
//! | [`face`] | the **face chooser**, drawn on two surfaces from one module, and the standard-14 disclosure it owes |
//! | [`fonts`] | the Fonts panel's inventory report |
//! | [`objects`] | the Objects panel, and the wording of every [`crate::panels::objects::summary::ObjectSummary`] fact |
//! | [`properties`] | the Properties panel |
//!
//! ## Almost every sentence here is salvaged verbatim, and that is the point
//!
//! These strings came across from the old shell's `ui_text.rs` (7,912 lines,
//! 1,193 entries) **with their doc comments**, because the doc comment is
//! usually the record of a defect the wording was changed to fix. Three
//! examples, all of which are below:
//!
//! - [`signature_leaves_tail`] is worded as *under-protection* rather than
//!   as damage, because ISO 32000-1 §12.8.1 makes whole-file coverage a
//!   `should` — the document is conforming, and an operator told "invalid"
//!   about a legal file has been misled just as surely as one told nothing.
//! - [`layers_session_only_note`] exists because a panel of tickboxes over a
//!   document is, by every other application's convention, an editor — and
//!   this one is not. Its doc comment carries the full wording history,
//!   including the two occasions the sentence was wrong.
//! - [`fonts::font_verdict_removable`] and its four siblings are **two
//!   words each**, and were full sentences until a screenshot of the running
//!   panel showed the row clipped at the dock's edge with the byte size cut
//!   to `59`.
//!
//! Rewriting any of those from scratch would re-derive a decision already
//! paid for, which is exactly what `SALVAGE.md`'s procedure forbids.
//!
//! ## The three panels in this file share a posture
//!
//! **Each says what it cannot tell you, first.** The Signatures panel opens
//! with the sentence that pdfcer performs no cryptographic verification,
//! because a panel headed "Signatures" listing byte counts is the single
//! likeliest place in this application for an operator to take away more
//! than was said. The Layers panel opens by saying that a toggle changes what
//! you see and not the document, and that nothing it does is saved — which at
//! S3 also had to say the toggle was absent, and at S4 does not, because it
//! is back. The Bookmarks panel says when its own reader gave up.
//!
//! That ordering is not stylistic. A caveat below a list arrives after the
//! operator has already drawn a conclusion.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Name the thing and what the operator can do about it.**
//! - **Never state a capability the build does not have.** Several strings
//!   below were amended at salvage for exactly this reason, and each says so
//!   in its own doc comment rather than being quietly reworded.

/// ★ The Attachments panel — the whole files a document carries inside itself
/// (§7.11.4.1), and the four verbs opposite them.
///
/// Its own header carries the three sentences that are **not optional** on this
/// feature, each one an obligation `pdfcer-core` states in its own doc comment:
/// that removing an attachment does not erase its bytes under an incremental
/// save, that an embedded file may be encrypted inside an otherwise unencrypted
/// document, and that the name pdfcer writes to disk is not always the name the
/// document shows.
pub mod attachments;
/// ★ The words for **moving** a bookmark and for **expanding or collapsing**
/// one — `pdfcer-core` `Pass 161.0`'s two verbs.
///
/// The Bookmarks panel's other strings stay in this file, with Layers and
/// Signatures; these went to a module of their own under R2 when the two verbs
/// arrived, and the subject boundary is *the words for the verbs that change a
/// bookmark's PLACE rather than its name*.
///
/// Its header carries the two rules a reader must not have to re-derive: why
/// **two** numbers describe one move (the engine's `visible_items` counts what
/// was on screen, and a collapsed branch reports `1` however large it is), and
/// why neither of its decline sentences names the bookmark it is about.
pub mod bookmarks;
pub mod comments;
/// The ce-dimension properties section — the bottom tier of the style cascade
/// made reachable, with the tier each value came from named beside it.
///
/// Its own header carries the one rule this catalog must not break: **it never
/// builds a label**. A limit tolerance suppresses the nominal rather than
/// printing beside it, and a panel previewing the two by concatenation
/// disagrees with the bytes in the page.
pub mod dimension;
/// ★ The **face chooser**, which is one control drawn on two surfaces — the
/// Properties panel's *This text* section and the ribbon's Format ▸ Font group.
///
/// Its own header carries the obligation that made it a module rather than a
/// block inside [`properties`]: since `Pass 162.0` the chooser offers faces the
/// document does **not** contain, pdfcer authors those without embedding
/// anything, and the text is then drawn with the reader's own copy of the face
/// — an inference the operator cannot see on this screen and can see on
/// somebody else's, which is exactly the case rule 4 requires a sentence for.
pub mod face;
/// The Fonts panel's inventory report.
pub mod fonts;
/// The Comments panel — every annotation on the document, listed.
pub mod formfield;
/// The Objects panel, and the wording of every object fact.
pub mod objects;
/// The Properties panel.
pub mod properties;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// Format a byte count for a listing.
///
/// Base-1024 arithmetic with the colloquial "KB"/"MB" labels rather than the
/// pedantically correct "KiB"/"MiB": this is read by operators comparing the
/// figure against what their file manager tells them, and matching that is
/// worth more here than matching IEC. The exact count is always shown beside
/// it (see [`fonts::font_size_line`]), so nothing is lost to the rounding.
///
/// Deliberately different from the byte counts in the Signatures panel, which
/// are printed raw. Those exist to be compared against a file's own length —
/// an exactness task. These exist to be ranked across up to a couple of
/// hundred rows — a magnitude task. Different purpose, different format.
#[must_use]
pub fn byte_size(bytes: usize) -> String {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a display rounding to one or two decimals; the exact count is printed alongside" // ui-text-exempt: clippy lint justification, never displayed
    )]
    let n = bytes as f64;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", n / 1024.0)
    } else {
        format!("{:.2} MB", n / (1024.0 * 1024.0))
    }
}

/// Shown in any panel when no document is open.
///
/// **One sentence for every panel**, deliberately. A panel is never
/// blanked — a blank region is indistinguishable from a broken one — and a
/// bespoke "open a document to…" sentence per panel would be nine chances for one of
/// them to drift into a different voice, for no gain: at the moment nothing
/// is open, which panel the operator is looking at does not change the
/// answer.
#[must_use]
pub fn panel_no_document() -> &'static str {
    "Open a document to see this panel."
}

/// Shown when the dock holds a panel this build does not have.
///
/// Reachable exactly one way: a **saved layout** — or a named workspace —
/// naming a panel whose capability is not compiled into this binary
/// (`SHELL_FRAMEWORK.md` §5b). The dock's loader drops such entries and
/// reports them, so in practice this is the belt to that loader's braces.
///
/// It says what happened rather than apologising, because the operator's
/// next question is whether their layout is broken. It is not: the rest of
/// it loaded, and re-saving will forget this entry.
///
/// **Not a placeholder.** The no-placeholders rule forbids a control that
/// looks available and is not; it does not forbid explaining a pane the
/// operator's own saved layout asked for. A blank pane here would be
/// indistinguishable from a panel that had nothing to say.
#[must_use]
pub fn panel_unknown() -> &'static str {
    "This panel is not part of this build. Your saved layout asked for it; \
     everything else in the layout loaded normally."
}

// ---------------------------------------------------------------------------
// Signatures
// ---------------------------------------------------------------------------

/// The caveat, shown ABOVE the list on every visit.
///
/// A panel headed "Signatures" listing byte counts is the likeliest place in
/// this application for an operator to take away more than was said. The
/// sentence is first, not a tooltip, because the person who most needs it is
/// the one who will not hover.
#[must_use]
pub fn signatures_not_a_validity_check() -> &'static str {
    "pdfcer does not check whether these signatures are valid — it cannot yet. What follows is what each signature COVERS: which parts of the file it would protect if it is valid."
}

/// No signature carries a byte range.
#[must_use]
pub fn signatures_none() -> &'static str {
    "This document has no signatures. (An empty signature field, waiting to be signed, is not one.)"
}

/// The file could not be measured.
///
/// Distinct from "no signatures": pdfcer could not read the file's length
/// from disk, so it has nothing to compare a byte range against. Saying
/// "no signatures" here would be a claim about the document made from an
/// inability to look.
#[must_use]
pub fn signatures_file_unreadable() -> &'static str {
    "pdfcer could not read this file's size from disk, so it cannot say what the signatures cover. Nothing here is a statement about the document."
}

/// A signature field with no name of its own.
#[must_use]
pub fn signature_unnamed() -> &'static str {
    "(unnamed signature)"
}

/// The good case.
#[must_use]
pub fn signature_covers_whole_file(covered: u64) -> String {
    format!("Covers the whole file — {covered} bytes, up to the last one.")
}

/// The case that matters: content exists beyond the signed range.
///
/// Worded as under-protection rather than as damage. ISO 32000-1 §12.8.1
/// makes whole-file coverage a `should`, so this document is CONFORMING —
/// and an operator told "invalid" about a legal file has been misled just as
/// surely as one told nothing.
#[must_use]
pub fn signature_leaves_tail(covered: u64, tail: u64) -> String {
    format!(
        "Covers {covered} bytes, but {tail} bytes come after the signed range — content this signature does not protect. That is allowed by the standard, and it means the signature guarantees less than its presence suggests."
    )
}

/// Overlapping or backwards ranges.
#[must_use]
pub fn signature_range_malformed() -> &'static str {
    "This signature's byte range is malformed — its parts overlap or run backwards, which the standard does not permit. The numbers below are what the file claims; another reader may compute something different, or refuse it."
}

/// A single range, which cannot verify.
#[must_use]
pub fn signature_single_range() -> &'static str {
    "This signature declares one continuous range, so it includes its own signature value in what it signs. A signature in that shape cannot verify anywhere."
}

/// The line naming which state of the file the coverage numbers describe.
///
/// **New at salvage, and not decoration.** The old panel's doc comment
/// carried this fact and the panel never said it out loud:
///
/// > Unsaved edits are not counted, and cannot be: they are not in the file
/// > yet. The panel says which state it is describing rather than leaving an
/// > operator to assume.
///
/// The second sentence was a promise the code did not keep — the panel
/// stated the numbers and left the reader to work out that they are about
/// bytes on disk. `/ByteRange` is a claim about bytes, so it can only be
/// checked against bytes, and the length used is the file **on disk right
/// now**. Once this shell can edit, "does the signature cover the file as it
/// currently exists" and "does it cover what I am looking at" are different
/// questions with different answers, and only one of them is being answered.
#[must_use]
pub fn signatures_measured_on_disk() -> &'static str {
    "Measured against the file as it is on disk right now. Any edits you have not saved are not part of these numbers."
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

/// Shown when the document declares no optional content at all.
///
/// Distinct from "no layers I could read": most PDFs simply have none, and
/// saying so plainly stops an operator hunting for a panel fault.
#[must_use]
pub fn layers_none() -> &'static str {
    "This document has no layers."
}

/// Count above the list.
#[must_use]
pub fn layers_count(total: usize) -> String {
    if total == 1 {
        "1 layer.".to_owned()
    } else {
        format!("{total} layers.")
    }
}

/// The disclosure above the list, always shown.
///
/// ## ★ Wording history, because this string has been wrong twice
///
/// It has now had three lives, and the record matters more than any one of
/// them: **nothing compiles a doc comment against the behaviour it
/// describes**, so the only defence is that the sentence and the control
/// change in the same commit, and that the reason is written down here when
/// it does.
///
/// | When | What it said | Was it true? |
/// |---|---|---|
/// | old shell, pre-checkbox | *"Switching a layer changes what you see, not the document. Nothing here is saved."* | **No** — for three commits it described a checkbox the panel did not yet have, and the error was found by a person reading the file. |
/// | old shell, post-checkbox | the same sentence | yes |
/// | this build, S3 | *"…Switching a layer on or off is not available in this build…"* | yes — the panel genuinely had no control |
/// | this build, S4 (**now**) | the sentence below | yes — the control is back |
///
/// ## ★ What changed at S4, and when
///
/// S4 completed the third and last of the preconditions
/// `crate::panels::layers`' header tracks. `crate::app::actions::Action`
/// gained `SetLayerVisible`, `ResetLayers` and `ToggleAnnotations`, each
/// implemented in `PdfcerApp::apply`; the render worker's `RenderKey` had
/// already gained `layers_generation`, and `crate::app::state::OpenDoc` had
/// already gained the override and its mutators. So the panel has a control
/// again, and the S3 clause — *"is not available in this build"* — became
/// false the moment it did. It is removed **in the same commit that added
/// the control**, which is the whole of the discipline this table exists to
/// record.
///
/// ## Why the first sentence is the salvaged one, verbatim
///
/// *"Switching a layer changes what you see, not the document"* is the thing
/// an operator most needs to know and the hardest for them to discover: a
/// panel of tickboxes over a document is, by every other application's
/// convention, an editor. Restating it in fresh words would re-derive a
/// decision already paid for (`SALVAGE.md`), so it comes back exactly as it
/// was.
///
/// ## Why a clause was ADDED, and what it is for
///
/// The old sentence's second half — *"Nothing here is saved"* — is true and
/// incomplete in a way that reads as a pdfcer limitation. It is not one:
/// §8.11.2.1 puts a group's live ON/OFF state **outside the document
/// entirely**, so there is nowhere in the file for a save to put it. An
/// operator who reads "not saved" as "pdfcer cannot save this yet" will wait
/// for a version that can, and no version can. So the sentence now names the
/// consequence they will actually meet — the states come back on reopen —
/// and attributes it to the format rather than to the build.
///
/// That is also why it does **not** promise the states survive anywhere
/// else: they do not survive a reopen, a second window, or an export.
#[must_use]
pub fn layers_session_only_note() -> &'static str {
    "Switching a layer changes what you see, not the document. Nothing here is saved — a layer's on or off state lives outside the file, so this document opens with its own settings again next time."
}

/// How many layers currently differ from the document's own configuration.
///
/// Salvaged verbatim from the old shell. Shown beside the Reset control, and
/// only when the number is non-zero, so the pair reads as one statement:
/// *this many things differ, and here is the way back*.
///
/// **"Differ from the document", not "you changed"**, because the panel
/// computes it by comparing the effective hidden set against
/// `pdfcer_core::annot::optional_content_default_off` rather than by counting
/// clicks. A layer switched off and on again is back to agreeing with the
/// document and is not counted — which is the answer the operator would give
/// if asked, and the one a click-counter gets wrong.
#[must_use]
pub fn layers_overridden(n: usize) -> String {
    if n == 1 {
        "1 layer differs from the document.".to_owned()
    } else {
        format!("{n} layers differ from the document.")
    }
}

/// Label on the control that drops the operator's layer changes.
#[must_use]
pub fn layers_reset_label() -> &'static str {
    "Reset"
}

/// Tooltip on that control.
///
/// Says what it returns **to**, because "reset" in a layers panel could
/// equally be read as "turn everything on" — and those are different acts on
/// a document that declares a "Confidential" watermark off by default.
/// Revealing such a layer is a disclosure event; returning to the document's
/// own configuration is the opposite of one.
///
/// The second sentence exists to make the difference concrete rather than
/// leaving it in the word "specifies".
#[must_use]
pub fn layers_reset_tooltip() -> &'static str {
    "Go back to the layer states the document itself specifies. This shows hidden layers as hidden again."
}

/// Tooltip on a layer's visibility control.
///
/// Salvaged verbatim. Repeats the boundary that
/// [`layers_session_only_note`] states above the list, deliberately: the note
/// is read once when the panel opens and the tooltip is read at the moment of
/// the click, which is when the question *"am I editing this file?"* is
/// actually being asked.
#[must_use]
pub fn layer_toggle_tooltip() -> &'static str {
    "Show or hide this layer on screen. The document is not changed."
}

/// Tooltip on a layer whose state the operator has changed.
///
/// Names the document's own state, so the operator can always see what they
/// are diverging FROM without resetting to find out. Salvaged verbatim.
///
/// Both arms are needed and they are not symmetric in consequence: *"You have
/// shown this layer. The document hides it."* is the one that matters, since
/// a layer the document hides may be hidden for a reason.
#[must_use]
pub fn layer_overridden_tooltip(document_wanted_visible: bool) -> &'static str {
    if document_wanted_visible {
        "You have hidden this layer. The document shows it."
    } else {
        "You have shown this layer. The document hides it."
    }
}

/// Some layers' states are managed automatically (§8.11.4.4).
///
/// Says what the list IS rather than apologising for what it is not: the
/// states shown are the document's opening states, and for these layers the
/// page may legitimately disagree at the current zoom.
#[must_use]
pub fn layers_auto_managed(n: usize) -> String {
    if n == 1 {
        "1 layer switches itself on or off as you zoom. The state shown here is the one the document opens in.".to_owned()
    } else {
        format!(
            "{n} layers switch themselves on or off as you zoom. The states shown here are the ones the document opens in."
        )
    }
}

/// Tooltip on a layer whose `/Intent` excludes viewing (§8.11.2.3).
///
/// Explains why a layer the document lists as off is shown anyway —
/// otherwise the only available reading is "pdfcer got it wrong".
///
/// ## ★ A second sentence was added at S4, and it is not a stylistic one
///
/// The first sentence alone became **actively misleading** the moment the
/// visibility control returned, because it invites the reading *"so there is
/// no point switching this row"*. The opposite is true, and it is a property
/// of the engine rather than of this panel:
///
/// `pdfcer_render`'s interpreter resolves a group's state from
/// `oc_off_set()`, and when an operator override is in force that function
/// returns **the override verbatim** — no `/Intent` filtering, no `/AS`
/// usage application (`interpret.rs`, and `annot.rs` for annotation `/OC`).
/// Intent filtering happens only inside
/// `pdfcer_core::annot::optional_content_default_off`, which is what builds
/// the *document's* answer. So:
///
/// | state | is a design-intent group's `/OFF` membership honoured? |
/// |---|---|
/// | no override (the document's own configuration) | **no** — §8.11.2.3 filters it out, and the group draws |
/// | any override in force | **yes, for every group in the set**, this one included |
///
/// That asymmetry is the engine's documented replace-not-merge contract
/// (core API trap T-12.9) doing exactly what it says. It is disclosed rather
/// than papered over, per rule 4: pdfcer inferred something and the inference
/// changes the page.
#[must_use]
pub fn layer_design_intent_tooltip() -> &'static str {
    "This layer is marked for design use, not viewing, so the document's own on or off setting for it does not affect what is drawn. Switching it here does: your choice replaces the document's whole layer configuration for as long as this document is open."
}

/// Placeholder for a layer whose `/Name` is absent.
///
/// `/Name` is Required (Table 98), so its absence is a real malformation.
/// The placeholder says so rather than inventing "Layer 3", which would
/// disguise a defect as data from the file.
#[must_use]
pub fn layer_unnamed() -> &'static str {
    "(no name in the file)"
}

/// Text marker for a layer drawn by default. TEXT, never colour alone.
#[must_use]
pub fn layer_visible_marker() -> &'static str {
    "shown"
}

/// Text marker for a layer hidden by default.
#[must_use]
pub fn layer_hidden_marker() -> &'static str {
    "hidden"
}

/// Tooltip on a locked layer.
///
/// States what the lock actually is: the specification's own table blesses
/// JavaScript and `/AS` bypass, so calling it "cannot be changed" would
/// overstate it.
///
/// At S4 this became the tooltip on a **disabled** control rather than on a
/// bare row, which is why `crate::panels::layers` attaches it with
/// `on_disabled_hover_text` as well as `on_hover_text` — egui does not show
/// the ordinary hover text of a disabled widget, and this is the one row
/// whose explanation is the whole reason it looks broken.
#[must_use]
pub fn layer_locked_tooltip() -> &'static str {
    "The document marks this layer locked, so a viewer should not offer to switch it. It is an interface lock, not a guarantee — the document's own scripts can still change it."
}

/// Tooltip on a layer that content references but the default configuration
/// never registered.
#[must_use]
pub fn layer_unregistered_tooltip() -> &'static str {
    "Page content uses this layer, but the document never listed it in its layer configuration. Some readers will not show it in their own layer panel at all."
}

/// Tooltip on a layer in a radio-button group.
///
/// Table 101's `/RBGroups` are "radio button" groups: at most one member
/// visible at a time.
///
/// **The wording did not change at S4, and that is worth recording**: it was
/// already written as a plain statement of what a switch does, so restoring
/// the control turned a description of the document into a description of the
/// panel without a word moving. The fact was worth knowing either way — a CAD
/// drawing with two mutually exclusive title blocks is a different document
/// from one with two independent ones.
///
/// Note what it does **not** say: that switching this layer *off* switches a
/// sibling on. "At most one" permits none, and choosing a replacement would
/// be pdfcer deciding which alternate the operator meant.
#[must_use]
pub fn layer_radio_tooltip() -> &'static str {
    "One of a group where switching this layer on switches the others off."
}

/// Tooltip on a radio-group member whose group also contains a locked layer.
///
/// ## ★ This is pdfcer's answer to a question the standard leaves open
///
/// `pdfcer_core::layers`' own module docs name it `DA-A8` and hand it here
/// verbatim: *"a locked group's state 'cannot be changed through the user
/// interface', while a sibling being turned ON means 'all others **shall** be
/// turned OFF'. Reported, not resolved — resolving it is the toggling
/// surface's decision to make and to disclose."*
///
/// **pdfcer lets the lock win.** Turning on a radio member leaves a locked
/// sibling exactly as it was, so the panel can end up showing two members of
/// a mutually exclusive group at once.
///
/// The reasoning, since the choice is not obvious and the losing option is
/// respectable: both rules are addressed to the *user interface*, so the
/// tie-break has to be about which failure an operator can see and act on.
/// Turning off a locked layer as a side effect of clicking a **different**
/// row is a lock bypass through a side door — invisible at the moment it
/// happens, and it is exactly the "Confidential watermark quietly switched
/// off" shape that `/Locked` exists to prevent. Two title blocks painted over
/// each other is wrong too, but it is wrong *on the screen*, where the
/// operator is already looking. Between an invisible violation and a visible
/// one, take the visible one — and then say so, which is what this string is.
#[must_use]
pub fn layer_radio_locked_sibling_tooltip() -> &'static str {
    "Another layer in this group is locked, so pdfcer will not switch it off for you. Switching this layer on can leave two of the group showing at once, which the document says should not happen."
}

// ---------------------------------------------------------------------------
// Bookmarks
// ---------------------------------------------------------------------------

/// Summary line above the tree.
#[must_use]
pub fn bookmarks_count(total: usize) -> String {
    if total == 1 {
        "1 bookmark.".to_owned()
    } else {
        format!("{total} bookmarks.")
    }
}

/// Shown when the document has an outline but no items pdfcer could read.
#[must_use]
pub fn bookmarks_empty() -> &'static str {
    "This document has no bookmarks."
}
// ---------------------------------------------------------------------------
// Bookmarks — writing one
// ---------------------------------------------------------------------------

/// The heading over the add-a-bookmark row.
#[must_use]
pub const fn bookmark_add_heading() -> &'static str {
    "Add a bookmark"
}

/// Where a new bookmark will be filed, when a row has been clicked.
#[must_use]
pub fn bookmark_add_under(parent: &str) -> String {
    format!("Under {parent}")
}

/// Where it will be filed when no row has been clicked.
#[must_use]
pub const fn bookmark_add_at_top() -> &'static str {
    "At the top level"
}

/// The control that clears the chosen parent.
#[must_use]
pub const fn bookmark_add_to_top_button() -> &'static str {
    "Move to top level"
}

/// How a parent is chosen.
///
/// ★ Says it out loud because the gesture is **overloaded on purpose**: a
/// bookmark click navigates, and it also records the row as the parent for the
/// next add. Both are true of the row the operator pointed at, which is what
/// makes the overload honest — but an operator who was not told would file a
/// bookmark under whichever heading they last used to jump somewhere.
#[must_use]
pub const fn bookmark_add_parent_hint() -> &'static str {
    "Click a bookmark above to file the new one under it. Clicking also jumps \
     there, as it always does."
}

/// The destination the new bookmark will point at.
///
/// Stated rather than chosen, and stated by **page number**, for the reason the
/// Insert-from-file dialog gives for its own destination: the panel is beside a
/// document the operator may have scrolled, and the number is what makes the
/// choice checkable.
#[must_use]
pub fn bookmark_add_destination(page_number: usize) -> String {
    format!("It will point at page {page_number}, the one on screen.")
}

/// ★★ **The `/Count` trap, turned into a sentence.**
///
/// The engine called it *"not a footnote … the entire difficulty of the
/// feature"*: a bookmark added under a **collapsed** ancestor does not change
/// the document's visible total, because it is not visible — §12.3.3 defines no
/// `/Open` key, so the sign on `/Count` is the only carrier of open-or-closed.
///
/// Getting the count right is the low bar. The operator's actual problem is
/// that they will add a bookmark, look at the panel, and **not see it** — and
/// the panel will be correct. So this is said **before** the press, which is
/// the same posture the ce-dimension group window takes about re-measuring on
/// a move.
///
/// Worded as a fact about the parent, not as a warning about the action: the
/// add will work perfectly.
#[must_use]
pub const fn bookmark_add_under_collapsed() -> &'static str {
    "That bookmark is collapsed, so the new one will not appear until you \
     expand it. It will still be in the file."
}

/// The title field's placeholder.
#[must_use]
pub const fn bookmark_add_title_hint() -> &'static str {
    "What to call it"
}

/// The button that writes the bookmark.
#[must_use]
pub const fn bookmark_add_button() -> &'static str {
    "Add"
}

/// Why the button is unavailable with an empty title.
///
/// ★ Greyed **with** an explanation rather than absent, unlike the Rename
/// button in the groups window — and the difference is which control it is.
/// That one is an alternative to a field that already shows the name; this one
/// is the whole of the feature, and a row that vanished until you typed would
/// leave an operator looking for where bookmarks are added.
#[must_use]
pub const fn bookmark_add_needs_a_title() -> &'static str {
    "Type a name first. A bookmark with no title still appears in the list, as \
     a blank row nothing distinguishes."
}

// ---------------------------------------------------------------------------
// Bookmarks — renaming one, and removing one with everything under it
//
// The surface for `EditSession::set_outline_title` and
// `EditSession::delete_outline_item`, both `pdfcer-core` `Pass 156.0`. See
// `crate::panels::bookmarks::edit` for the interaction; this block is only the
// words, and two of them are load-bearing in a way the rest of this file's
// entries are not:
//
//   * `bookmark_delete_takes_subtree` is said BEFORE the press, because the
//     verb's blast radius is larger than the row the operator clicked, and
//     they cannot see how much larger when the row is collapsed.
//   * `bookmark_deleted` is said AFTER it, from the number the ENGINE
//     returned, because the shell's own count is a count of what pdfcer could
//     read and the engine's is a count of what it removed.
// ---------------------------------------------------------------------------

/// The heading over the rename-and-remove block.
///
/// Names both verbs, because the block is only drawn when a row is selected
/// and an operator who has clicked a bookmark to jump somewhere needs to know
/// why two new controls appeared under it.
#[must_use]
pub const fn bookmark_edit_heading() -> &'static str {
    "Selected bookmark"
}

/// Which bookmark the rename and remove controls act on.
///
/// ★ The selected row is named rather than merely highlighted, for the reason
/// the ce-dimension group window names its group: this block sits **above** an
/// unbounded scroll area, so the row it acts on may be scrolled out of sight by
/// the time the operator presses a button. A highlight nobody can see is not a
/// selection indicator.
#[must_use]
pub fn bookmark_edit_selected(title: &str) -> String {
    format!("Selected: {title}")
}

/// The label beside the rename field.
#[must_use]
pub const fn bookmark_rename_label() -> &'static str {
    "Name"
}

/// The button that writes the new title.
#[must_use]
pub const fn bookmark_rename_button() -> &'static str {
    "Rename"
}

/// The button that removes the bookmark.
///
/// *"Remove"*, not *"Delete"*: the operator-facing distinction this application
/// keeps is that removing a bookmark takes it out of the navigation and leaves
/// every page exactly where it was. A button reading "Delete" beside a document
/// How many bookmarks a copy will take with it.
///
/// ★ The same shape as `bookmark_delete_takes_subtree`, deliberately: an
/// operator who has read one has read the other, and a copy and a delete take
/// exactly the same set. Two wordings for one fact is how a panel comes to
/// describe two different operations that are in fact identical in scope.
#[must_use]
pub fn bookmark_copy_takes_subtree(descendants: usize) -> String {
    if descendants == 1 {
        "Copying this takes the one bookmark filed under it as well.".to_owned()
    } else {
        format!("Copying this takes the {descendants} bookmarks filed under it as well.")
    }
}

/// The Copy button.
#[must_use]
pub const fn bookmark_copy_button() -> &'static str {
    "Copy"
}

/// The Cut button.
#[must_use]
pub const fn bookmark_cut_button() -> &'static str {
    "Cut"
}

/// The engine declined to copy the bookmark, in its own words.
#[must_use]
pub fn bookmark_copy_refused(engine: &str) -> String {
    format!("That bookmark could not be copied. {engine}")
}

/// What is on the clipboard, above the Paste button.
///
/// ★ It counts the WHOLE subtree, not the roots, because that is what will
/// arrive — and an operator who copied one chapter heading and sees *"12
/// bookmarks"* has learned something true that the tree did not show them.
#[must_use]
pub fn bookmark_paste_heading(items: usize) -> String {
    if items == 1 {
        "1 bookmark copied.".to_owned()
    } else {
        format!("{items} bookmarks copied.")
    }
}

/// ★★★ **The warning that must be read BEFORE the press.**
///
/// A pasted bookmark whose destination names a page this document does not have
/// is **dropped, not clamped** — it arrives, shows, keeps its title, and does
/// nothing when clicked. Nothing on screen distinguishes it from one that
/// works, which is why this is the only pre-press warning in the panel.
///
/// # Why it names both numbers
///
/// *"needs 14 pages, this one has 6"* is a fact the operator can act on: add
/// the sheets first, or accept the loss. *"Some destinations will be dropped"*
/// is a warning they can only obey or ignore.
///
/// ★ It does **not** say how many will drop, and that is honest rather than
/// lazy: the clip knows its deepest destination, not the distribution of the
/// rest, so any count here would be a guess. The engine reports the real number
/// after the paste, which is where an exact figure belongs.
#[must_use]
pub fn bookmark_paste_destinations_dropped(needs: usize, has: usize) -> String {
    format!(
        "Some of these point at page {needs}, and this document has {has}. Those bookmarks will \
         arrive with no destination \u{2014} they will show in the list and do nothing when \
         clicked. Add the pages first if you want them to work."
    )
}

/// Where the paste will land, when a bookmark is selected.
#[must_use]
pub fn bookmark_paste_under(title: &str) -> String {
    format!("They will go under \u{201c}{title}\u{201d}.")
}

/// Where the paste will land, when nothing is selected.
#[must_use]
pub const fn bookmark_paste_at_top_level() -> &'static str {
    "They will go at the top level. Select a bookmark first to file them under it."
}

/// The Paste button.
#[must_use]
pub const fn bookmark_paste_button() -> &'static str {
    "Paste bookmarks"
}

/// **How many arrived without their destination** — reported after the paste.
///
/// ★ The panel predicted this before the press and this is what happened, and
/// the two are not duplicates: a prediction is a guess nobody confirmed, and a
/// report alone arrives too late to choose differently. The operator gets the
/// choice *and* the outcome.
#[must_use]
pub fn bookmark_paste_dropped(n: usize) -> String {
    if n == 1 {
        "One pasted bookmark points at a page this document does not have, so it arrived with no \
         destination and does nothing when clicked."
            .to_owned()
    } else {
        format!(
            "{n} pasted bookmarks point at pages this document does not have, so they arrived \
             with no destination and do nothing when clicked."
        )
    }
}

/// invites the reading that the pages go too.
#[must_use]
pub const fn bookmark_delete_button() -> &'static str {
    "Remove"
}

/// ★★ **The subtree warning, said before the press.**
///
/// The engine's rule, and the reason it is Acrobat's too:
///
/// > *"promoting orphaned children to the deleted item's parent silently
/// > reorganises a document's navigation, and an operator who deleted one
/// > chapter heading would find its ten sections spliced into the top level."*
///
/// So the subtree goes, which is the predictable act — and it is also an act
/// whose size the operator **cannot see** when the row is collapsed, because
/// §12.3.3 gives a closed item's ancestors a `/Count` contribution of exactly
/// one however large its subtree is.
///
/// ⇒ Stated as a fact about what the button will do, before it is pressed, in
/// the same posture `bookmark_add_under_collapsed` takes about the add. The
/// count is the shell's own read of the tree it drew; the count reported
/// afterwards is the engine's, and `bookmark_deleted` explains why they are
/// allowed to differ.
#[must_use]
pub fn bookmark_delete_takes_subtree(descendants: usize) -> String {
    if descendants == 1 {
        "Removing this also removes the 1 bookmark filed under it.".to_owned()
    } else {
        format!("Removing this also removes the {descendants} bookmarks filed under it.")
    }
}

/// The reassurance that goes with it, and it is a fact rather than comfort.
///
/// An outline is a document-level structure reached from the catalogue's
/// `/Outlines` (§12.3.3), not from any page. Removing a bookmark removes a way
/// of *reaching* a page and changes nothing drawn on one. Worth saying beside a
/// control that removes several things at once, because the operator's
/// reasonable fear at that moment is that the pages are what is being removed.
#[must_use]
pub const fn bookmark_delete_keeps_pages() -> &'static str {
    "The pages themselves are not touched — only the way of jumping to them."
}

/// ★★ **What was actually removed**, reported after the fact from the count the
/// engine returned.
///
/// `EditSession::delete_outline_item` returns how many items went, the clicked
/// one included, and that number is the answer to the question this verb raises
/// and cannot answer any other way: the subtree went too, and on a collapsed
/// parent the operator could not see how large it was.
///
/// ★ **It may disagree with the number promised before the press, and that is
/// why both are said.** `read_outline` gives up part-way on a cycle, on
/// excessive depth, or on exhausting its item budget — the panel draws a
/// truncation notice when it does — so the pre-press count is a count of *what
/// pdfcer could read* and this one is a count of *what it removed*. On any
/// ordinary document they agree.
///
/// One is not spelled as none: *"including its 0 bookmarks beneath it"* is the
/// shape of sentence that makes a program look like it is reading from a
/// template, so the leaf case gets its own words.
#[must_use]
pub fn bookmark_deleted(removed: usize) -> String {
    if removed <= 1 {
        "Bookmark removed.".to_owned()
    } else {
        format!(
            "Bookmark removed, along with the {} filed under it. Undo puts them all back.",
            removed - 1
        )
    }
}

/// Why the Rename button is not offered for a blank name.
///
/// ★ Absent rather than greyed, unlike the Add button one block up, and the
/// asymmetry is deliberate in both directions. The Add button is the **whole**
/// of its feature and a row that vanished until you typed would leave an
/// operator hunting for where bookmarks are added; the Rename button sits
/// beside a field that already shows the bookmark's current name, so the field
/// alone reads as *"this is what it is called"*, which is true. This sentence
/// is the hover text on the field for the one case worth explaining — a name
/// typed down to nothing.
#[must_use]
pub const fn bookmark_rename_needs_a_title() -> &'static str {
    "A bookmark needs a name. One with a blank title still appears in the \
     list, as a blank row nothing distinguishes."
}

/// Disclosure when pdfcer's own reader had to give up part-way.
///
/// A truncated tree looks exactly like a short one from the outside, so
/// silence here would let an operator conclude the document simply has few
/// bookmarks. Stated as what it is: pdfcer stopped, the document did not end.
#[must_use]
pub fn bookmarks_truncated() -> &'static str {
    "pdfcer stopped reading this outline early — it loops back on itself or is deeper than pdfcer follows. Some bookmarks are missing from this list."
}

/// An outline item with no title of its own.
///
/// Its row still has to exist: a bookmark's children hang off it, and
/// omitting an untitled parent would show them at the wrong depth, silently
/// misrepresenting the document's structure.
#[must_use]
pub fn bookmark_untitled() -> &'static str {
    "(untitled)"
}

/// Tooltip on a bookmark row, naming where it goes.
///
/// The destination page is stated rather than left to be discovered by
/// clicking: an operator scanning a long outline for "where is the parts
/// list" should not have to jump to find out.
#[must_use]
pub fn bookmark_row_tooltip(page_number: usize) -> String {
    format!("Go to page {page_number}.")
}

/// A heading bookmark: no destination, by design.
#[must_use]
pub fn bookmark_row_heading_tooltip() -> &'static str {
    "A heading. It groups the bookmarks beneath it and does not point at a page of its own."
}

/// Tooltip on a bookmark that points nowhere pdfcer can resolve.
///
/// Distinct from a bookmark with no destination at all, which is a heading
/// and perfectly normal. This one MEANT to point somewhere and pdfcer could
/// not work out where — the operator should know the difference before
/// concluding the document is broken.
#[must_use]
pub fn bookmark_row_unresolved_tooltip() -> &'static str {
    "This bookmark points somewhere pdfcer could not resolve — it may use a destination form pdfcer does not read yet, or name a page that is not in this document."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three "cannot tell you" openers are genuinely different
    /// sentences.**
    ///
    /// Each of the three structure panels leads with a limitation, and the
    /// value of doing so is entirely in the limitation being specific. Three
    /// near-identical hedges would satisfy the convention and teach an
    /// operator to skip the first line of every panel, which is worse than
    /// having none.
    #[test]
    fn each_structure_panel_leads_with_its_own_limitation() {
        let openers = [
            signatures_not_a_validity_check(),
            layers_session_only_note(),
            bookmarks_truncated(),
        ];
        for (i, a) in openers.iter().enumerate() {
            for b in openers.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
            assert!(a.len() > 40, "an opener too short to be specific: {a}");
        }
    }

    /// **The Layers note says a toggle changes the VIEW, and not the
    /// document.**
    ///
    /// This test replaces `the_layers_note_states_that_switching_is_unavailable`,
    /// which asserted the S3 truth — that the panel had no visibility control
    /// — by pinning the words "not available". S4 gave the panel its control
    /// back (`crate::app::actions::Action::SetLayerVisible`), so that clause
    /// became a lie and came out **in the same commit as the checkbox**.
    ///
    /// The test is rewritten rather than deleted because the *thing it
    /// guards* did not go away, it inverted. Two directions now:
    ///
    /// 1. **The claim that must be present.** A panel of tickboxes over a
    ///    document reads as an editor. "not the document" is the clause that
    ///    says it is not, and it is the single most load-bearing phrase in
    ///    this panel.
    /// 2. **The claim that must be absent.** If a later change reverts to the
    ///    S3 wording *without* removing the control, the panel is back to
    ///    describing a program that does not exist — the failure the old
    ///    shell's module header records happening twice, to two different doc
    ///    comments, in this exact file. A control that says of itself that it
    ///    is unavailable is not a copy-edit defect; it is the operator
    ///    concluding the application is broken.
    ///
    /// Asserted on clauses rather than on the whole string: a copy edit
    /// should be free, and a capability claim should not.
    #[test]
    fn the_layers_note_says_a_toggle_changes_the_view_and_not_the_document() {
        let note = layers_session_only_note();
        assert!(
            note.contains("not the document"),
            "the panel now HAS a visibility control, so the note's whole job is \
             to say that using it does not edit the file: {note}"
        );
        assert!(
            note.contains("Nothing here is saved"),
            "an operator who ticks a layer and closes the document must have \
             been told the tick does not travel with the file: {note}"
        );
        assert!(
            !note.contains("not available"),
            "the S3 clause is back and the control is here — the panel is now \
             denying a capability it ships: {note}"
        );
    }

    /// **Reset says what it returns TO, not merely that it resets.**
    ///
    /// `Action::ResetLayers` restores *the document's own default*, which on
    /// a file that declares a "Confidential" watermark off by default is
    /// emphatically not "show everything". Those are two different acts and
    /// only one of them is a disclosure event, so the control that performs
    /// the safe one must not be readable as the other.
    ///
    /// The label alone cannot carry that — "Reset" in a layers panel is
    /// genuinely ambiguous — so the tooltip is where the distinction lives
    /// and this is what keeps it there.
    #[test]
    fn the_reset_control_names_what_it_returns_to() {
        let label = layers_reset_label();
        assert!(!label.trim().is_empty());
        assert!(
            !label.ends_with('.'),
            "a label is a name and takes no trailing period: {label}"
        );

        let tip = layers_reset_tooltip();
        assert!(
            tip.contains("the document"),
            "reset must name the state it returns to, or it reads as 'turn \
             everything on': {tip}"
        );
        assert!(
            tip.contains("hidden"),
            "the tooltip has to say that hidden layers go back to hidden — that \
             is the half an operator would otherwise get wrong: {tip}"
        );
    }

    /// **The overridden count is a count of layers, and it is singular at
    /// one.**
    ///
    /// Same reasoning as [`layers_count`]: cheap to get wrong, immediately
    /// visible, and it sits directly beside the Reset control where an
    /// operator is deciding whether to click.
    #[test]
    fn the_overridden_count_agrees_with_itself_about_number() {
        assert!(
            layers_overridden(1).starts_with("1 layer "),
            "{}",
            layers_overridden(1)
        );
        assert!(
            layers_overridden(3).starts_with("3 layers "),
            "{}",
            layers_overridden(3)
        );
        for n in [1_usize, 3] {
            assert!(
                layers_overridden(n).contains("the document"),
                "the count is only meaningful against what it differs FROM: {}",
                layers_overridden(n)
            );
        }
    }

    /// **The two override tooltips name the document's own state, and say
    /// opposite things.**
    ///
    /// The point of [`layer_overridden_tooltip`] is that an operator can see
    /// what they are diverging from without resetting to find out. If both
    /// arms read alike, the row that says "you are showing content this
    /// document hides" — the one with a disclosure consequence — is
    /// indistinguishable from its harmless twin.
    #[test]
    fn an_overridden_layer_says_which_way_the_document_asked() {
        let doc_shows = layer_overridden_tooltip(true);
        let doc_hides = layer_overridden_tooltip(false);
        assert_ne!(doc_shows, doc_hides);
        assert!(doc_shows.contains("hidden"), "{doc_shows}");
        assert!(doc_hides.contains("shown"), "{doc_hides}");
    }

    /// ★★ **The delete disclosure speaks about the SAME quantity, before and
    /// after the press — and the two sentences count differently to do it.**
    ///
    /// This is the one piece of arithmetic in the bookmark wording and it has a
    /// genuine off-by-one waiting in it:
    ///
    /// * `bookmark_delete_takes_subtree` is handed the shell's **exclusive**
    ///   count (how many are filed *under* the row), because that is what the
    ///   panel can see and what `tree::descendants` returns;
    /// * `bookmark_deleted` is handed the engine's **inclusive** count, because
    ///   `EditSession::delete_outline_item` returns how many items went
    ///   *including* the one that was clicked.
    ///
    /// So the second must subtract one to name the same set the first named. If
    /// it did not, an operator promised *"also removes the 11"* would be told
    /// *"along with the 12"*, and the only conclusion available to them is that
    /// pdfcer removed a bookmark it was not asked to.
    ///
    /// The fixture is deliberately chosen so the right and wrong answers are
    /// different strings — the discipline the engine's `Pass 156.0` note asks
    /// for after its own delete test survived every sabotage: *"when you assert
    /// that A and B differ, check your fixture can tell them apart."*
    #[test]
    fn the_delete_promise_and_the_delete_report_name_the_same_quantity() {
        let under = 11;
        let promised = bookmark_delete_takes_subtree(under);
        assert!(promised.contains("11"), "{promised}");

        // What the engine returns for that same removal: the subtree plus the
        // clicked item.
        let reported = bookmark_deleted(under + 1);
        assert!(
            reported.contains("11"),
            "the report must name the same 11, not the 12 the engine counted: {reported}"
        );
        assert!(
            !reported.contains("12"),
            "the inclusive count must not leak into the sentence: {reported}"
        );
    }

    /// **A leaf reports a removal without a subtree clause.**
    ///
    /// `delete_outline_item` returns `1` for a childless bookmark, and
    /// *"along with the 0 filed under it"* is the shape of sentence that makes
    /// a program look like it is filling in a template. The two branches must
    /// therefore be genuinely different sentences rather than the same one with
    /// a zero in it.
    #[test]
    fn a_leaf_removal_says_nothing_about_a_subtree() {
        let leaf = bookmark_deleted(1);
        assert!(!leaf.contains('0'), "{leaf}");
        assert!(!leaf.contains("along with"), "{leaf}");
        assert_ne!(leaf, bookmark_deleted(2));
        // Defensive: a refusal cannot reach this function, but a future verb
        // that reports zero removals must not produce "the -1 filed under it".
        assert_eq!(bookmark_deleted(0), leaf);
    }

    /// ★ **The removal disclosure names the undo**, because that is what
    /// stands in for the confirmation dialog this surface deliberately does
    /// not show.
    ///
    /// `panels::bookmarks::edit`'s header carries the choice: delete is
    /// *undoable* rather than *confirmed*, on the grounds that one press is one
    /// engine command so `Ctrl+Z` restores the whole subtree. A disclosure that
    /// reported the loss without naming the remedy would be the worst of both —
    /// no question beforehand and no way out afterwards that the operator has
    /// been told about.
    #[test]
    fn the_removal_disclosure_names_the_way_back() {
        let said = bookmark_deleted(12);
        assert!(said.to_lowercase().contains("undo"), "{said}");
    }

    /// **The subtree warning is singular for one and plural for the rest.**
    ///
    /// It is read beside a button the operator is deciding whether to press, so
    /// *"the 1 bookmarks filed under it"* is exactly the sort of seam that
    /// makes a warning read as boilerplate and stop being read at all.
    #[test]
    fn the_subtree_warning_agrees_in_number() {
        let one = bookmark_delete_takes_subtree(1);
        assert!(one.contains("1 bookmark "), "{one}");
        assert!(!one.contains("bookmarks"), "{one}");
        assert!(bookmark_delete_takes_subtree(2).contains("2 bookmarks"));
    }

    /// **The pages line says what is NOT being removed.**
    ///
    /// The operator's reasonable fear beside a control that takes several
    /// things at once is that the pages are what is going. An outline is a
    /// document-level structure (§12.3.3) and removing a bookmark removes a way
    /// of *reaching* a page, so the sentence is a fact rather than reassurance —
    /// and it must actually mention the pages to do its job.
    #[test]
    fn the_pages_line_names_the_pages() {
        let said = bookmark_delete_keeps_pages();
        assert!(said.contains("pages"), "{said}");
    }

    /// A bookmark's three destination states read as three different things.
    ///
    /// "Points at a page", "is a heading" and "pdfcer could not follow it"
    /// are one good outcome, one normal outcome and one problem. Collapsing
    /// any two would send an operator looking for a fault in a document that
    /// has none, or reassure them about one that does.
    #[test]
    fn the_three_bookmark_states_are_distinguishable() {
        let go = bookmark_row_tooltip(7);
        let heading = bookmark_row_heading_tooltip();
        let unresolved = bookmark_row_unresolved_tooltip();
        assert!(go.contains('7'), "the destination page must be named: {go}");
        assert_ne!(go, heading);
        assert_ne!(heading, unresolved);
        assert_ne!(go, unresolved);
    }

    /// Counted lines say "1 layer", not "1 layers".
    ///
    /// Cheap to get wrong, immediately visible, and the reason both
    /// functions branch rather than appending an `s`.
    #[test]
    fn counted_lines_are_singular_at_one() {
        assert!(layers_count(1).starts_with("1 layer."));
        assert!(layers_count(2).starts_with("2 layers"));
        assert!(bookmarks_count(1).starts_with("1 bookmark."));
        assert!(bookmarks_count(0).starts_with("0 bookmarks"));
        assert!(layers_auto_managed(1).starts_with("1 layer switches"));
        assert!(layers_auto_managed(3).starts_with("3 layers switch"));
    }

    /// Byte sizes cross their unit boundaries where they should.
    ///
    /// Base 1024, and the boundary cases are where an off-by-one shows up as
    /// `1024 B` sitting above `1.0 KB` in a sorted list.
    #[test]
    fn byte_sizes_use_base_1024_and_switch_units_at_the_boundary() {
        assert_eq!(byte_size(0), "0 B");
        assert_eq!(byte_size(1023), "1023 B");
        assert_eq!(byte_size(1024), "1.0 KB");
        assert_eq!(byte_size(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(byte_size(1024 * 1024), "1.00 MB");
    }

    /// The two signature-coverage sentences state opposite facts and must
    /// not read alike.
    ///
    /// One is reassurance and one is a warning, and both are about a
    /// conforming file. An operator who cannot tell them apart at a glance
    /// gets no value from the panel at all.
    #[test]
    fn full_coverage_and_a_tail_read_as_different_answers() {
        let full = signature_covers_whole_file(4096);
        let tail = signature_leaves_tail(4096, 512);
        assert_ne!(full, tail);
        assert!(full.contains("4096"));
        assert!(tail.contains("4096") && tail.contains("512"));
        // The warning has to name the consequence, not just the numbers.
        assert!(tail.contains("does not protect"));
    }
}
