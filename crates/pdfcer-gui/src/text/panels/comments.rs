//! # `text::panels::comments` — every string the Comments panel shows
//!
//! The copy for [`crate::panels::comments`], which lists **every annotation
//! in the document** — the comment list a reviewer works through. One module
//! per panel surface, as [`super`]'s header lays out; `crate::panels::comments`
//! is the sole consumer.
//!
//! ## Most of this is salvaged verbatim, and the doc comments came with it
//!
//! Nine of the entries below came across from the old shell's `ui_text.rs`
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\ui_text.rs:9530-9625`) **with their
//! doc comments**, because in this project a doc comment on a string is
//! usually the record of the defect the wording was changed to fix.
//! `SALVAGE.md`'s procedure forbids re-deriving a decision already paid for.
//!
//! The two that carry the most history:
//!
//! - [`comments_all_without_notes`] exists because pdfcer's own markup
//!   authoring cannot write `/Contents` on a geometric shape — `MarkupSpec`
//!   has no contents field on any variant, deliberately
//!   (`D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:210-212`) — so a
//!   document whose annotations pdfcer drew shows a column of identical "no
//!   note" captions. `docs/core-api/03-capabilities.md:1085` makes saying so
//!   **mandatory copy**: *"A bare 'No note text' column reads as data loss."*
//! - [`comments_none`] names what is **excluded**, because a document that is
//!   nothing but form fields would otherwise show an empty comment list and
//!   look broken.
//!
//! ## What is new here, and why each one exists
//!
//! Six entries have no ancestor in the old shell. Every one of them is a
//! **disclosure** that `docs/core-api/03-capabilities.md` §3.4 ("★ what the UI
//! must disclose") or §3.5 ("Traps") asks for by name, and each says which:
//!
//! | Entry | Commissioned by |
//! |---|---|
//! | [`comments_excluded`] | this panel's own decision to state its filter in numbers rather than in the abstract — see [`crate::panels::comments`]' header |
//! | [`comment_row_hidden`] | §3.4.5 — *"A Comments panel that silently omits it is hiding document content; list it and mark it hidden."* |
//! | [`comment_row_appearance_unresolved`] | §3.4.4 — pdfcer *"displays nothing and does not guess"*, and the governing default is explicitly a reasoned guess, i.e. an inference, i.e. rule 4 applies |
//! | [`comment_row_description_caption`] | §3.5 — *"`/Contents` is dual-purpose … a UI labelling this 'comment' is right for markup and wrong for a Link"* |
//! | [`comment_row_is_group_member`] | §3.5 — the §12.5.6.2 group-attribute rule is **deliberately not applied** by core, so what this panel shows is the raw dictionary value and a conforming reader shows something else |
//! | [`comment_row_ce_dimension_heading`] / [`comment_row_ce_dimension_no_note`] | project rule 15 — a **ce dimension** is a `/Line` annotation, and a row that called it "Line" would be true about the file and useless to the operator |
//!
//! ## ★ Rule 15 is enforced by a test in this module
//!
//! *"Never write a bare 'dimension".* **ce dimensions** are the ones pdfcer
//! authors (`/Line` + `/IT /LineDimension` + a `/PieceInfo` sidecar); **pdf
//! dimensions** are CAD-exported page content. They have opposite properties
//! and the ambiguity has already sent one investigation down the wrong path,
//! so [`tests::no_string_here_says_a_bare_dimension`] sweeps every entry
//! rather than trusting review — this is a *catalog*, which is exactly the
//! kind of file where a bare noun slips in during a late reword.
//!
//! ## Conventions, restated from [`crate::text`] because they bind here
//!
//! - **Sentence case, no trailing period on labels; full sentences with
//!   punctuation for prose.**
//! - **Never state a capability the build does not have.**
//!
//! ### ★★★ CORRECTED 2026-09-05 — the Delete paragraph had outlived its reason
//!
//! This section read, from 2026-08-14 until that date:
//!
//! > Two of the old shell's Comments strings were deliberately **not**
//! > salvaged for exactly this reason: `comment_row_delete` and its five
//! > deletion siblings. This build's panel has no Delete, because `Action`
//! > carries no variant that could delete an annotation and inventing one is
//! > not this panel's to do.
//!
//! **The stated reason stopped being true.** `AnnotAction::Delete { page, id }`
//! exists, `crate::app::actions::annots::delete` calls
//! `EditSession::delete_annotation` through it, and the canvas Delete key and
//! the Format tab have both been using it — while this panel, the reviewer's
//! own work list, was the one surface that could not. The operator's report of
//! 2026-09-05 (*"the review features should look and act the same as they do
//! in Acrobat Reader"*) is what sent somebody to check, and a reviewer
//! deleting their own comment is the plainest thing on that list.
//!
//! ⇒ [`comment_row_delete`] and [`comment_row_delete_tooltip`] are below.
//! **Corrected in place and dated rather than left standing beside the
//! truth**, which is R5 — and this is the sixth time in this project a
//! limitation sentence has outlived its reason. *A sentence about what the
//! build cannot do is a dated citation with a shelf life measured in hours.*
//! Where the claim can be an assertion, it should be one; the assertion here
//! is `crate::panels::comments::tests::the_delete_control_reaches_the_engine`.

/// The count line, above the list.
///
/// Salvaged verbatim. `note(s) and markup item(s)` rather than "comments":
/// the list contains a `/Link`, a `/Stamp` and a ce dimension as readily as it
/// contains somebody's sticky note, and calling all of those "comments" is
/// wrong on the rows where the distinction matters most.
#[must_use]
pub fn comments_count(total: usize) -> String {
    format!("{total} note(s) and markup item(s).")
}

/// Shown when the document carries no listable annotation.
///
/// Salvaged verbatim. Names what is EXCLUDED, because a document full of form
/// fields would otherwise show an empty comment list and look broken. Form
/// fields and pop-up windows are deliberately not comments.
///
/// [`comments_excluded`] follows it with the actual numbers when there are
/// any, so this sentence answers *"is this panel broken?"* and that one
/// answers *"then where did my annotations go?"*.
#[must_use]
pub fn comments_none() -> &'static str {
    "No notes or markup on this document. Form fields and pop-up windows are not listed here — form fields have their own panel."
}

/// **What this panel filtered out, in numbers** — `None` when it filtered
/// nothing.
///
/// # Why the counts are stated rather than the rule
///
/// `crate::panels::comments`' filter is settled and argued in that module's
/// header, but an operator reading a list of six rows on a drawing they know
/// carries forty annotations needs the arithmetic, not the doctrine. The old
/// shell stated the rule ([`comments_none`]) and only on the empty case; this
/// states the numbers, on every case where there are any.
///
/// It is a **disclosure**, in rule 4's sense: the panel made a decision about
/// what to show, and the decision lives off-canvas in the panel that made it.
///
/// # `Option` rather than an empty string
///
/// So the caller cannot accidentally draw a blank line. A panel that renders
/// an empty label still reserves its height, which on a narrow dock reads as
/// a rendering fault. `None` means *draw nothing*, which is also the
/// no-placeholders rule applied to prose.
///
/// # The three clauses are built by hand rather than by a list joiner
///
/// Because each one has to name *where the thing went*, and the destinations
/// differ: form fields have another panel, pop-ups belong to the annotation
/// they hang off, and a `/TrapNet` is prepress output state that no reviewer
/// wrote and nobody can answer. A generic "N items were excluded" would be
/// the count without the fact that makes it actionable.
#[must_use]
pub fn comments_excluded(widgets: usize, popups: usize, trap_nets: usize) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();
    if widgets > 0 {
        clauses.push(format!(
            "{widgets} form field(s), which the Forms panel lists"
        ));
    }
    if popups > 0 {
        clauses.push(format!(
            "{popups} pop-up window(s), which belong to the annotations they hang off rather than being annotations in their own right"
        ));
    }
    if trap_nets > 0 {
        clauses.push(format!(
            "{trap_nets} trapping record(s), which are prepress output state written by a RIP rather than anything a person wrote"
        ));
    }
    if clauses.is_empty() {
        return None;
    }
    Some(format!("Not listed here: {}.", clauses.join("; ")))
}

/// Shown when EVERY listed annotation lacks `/Contents`.
///
/// Salvaged verbatim, and it is **mandatory copy** rather than a nicety —
/// `docs/core-api/03-capabilities.md:1085` requires it in these words, and
/// records why: pdfcer's own markup tools cannot attach a note to a shape
/// (`MarkupSpec` has no contents field on any variant, and that is
/// deliberate), so a document whose annotations pdfcer authored shows a column
/// of identical "no note" captions. Said once at the top rather than left to
/// be inferred from the repetition.
///
/// ★ **Note-text authoring for geometric markup is a filed request** — one of
/// the three this project owes pdfcer, per `HANDOFF.md` §1. The day it lands,
/// this sentence stops being true for newly drawn markup and stays true for
/// everything drawn before it, which is why it is worded as a fact about the
/// shapes rather than as a promise about the future.
#[must_use]
pub fn comments_all_without_notes() -> &'static str {
    "None of these carry note text. Shapes drawn in pdfcer do not have a note attached to them yet, so this is expected rather than missing data."
}

/// One row's heading — what it is and which page it is on.
///
/// Salvaged verbatim. `subtype` is
/// `pdfcer_core::annot::Annotation::subtype_label`, which is the `/Subtype`
/// name decoded lossily or `(no Subtype)` when the key is absent — a
/// malformed annotation, surfaced rather than repaired.
#[must_use]
pub fn comment_row_heading(subtype: &str, page_number: usize) -> String {
    format!("{subtype} — p. {page_number}")
}

/// A **ce dimension**'s row heading, which names it as one.
///
/// # Why this is a second function and not a substituted label
///
/// Project rule 15: a **ce dimension** is the thing pdfcer authors — a `/Line`
/// annotation carrying `/IT /LineDimension`, a baked `/AP` and a `/PieceInfo`
/// sidecar record — and a **pdf dimension** is CAD-exported page content. They
/// have opposite properties. A row that showed a ce dimension as plain "Line"
/// would be true about the file and useless to the operator, who has a Measure
/// tab full of verbs for exactly this object.
///
/// The subtype is kept **in brackets rather than replaced**, and that is the
/// whole reason this reads the way it does. The old shell's exclusion argument
/// (`main.rs:7031-7051`) turns on ce dimensions being ordinary `/Line`
/// annotations — that is why they cannot be filtered out by subtype without
/// also hiding a genuine `/Line` markup somebody drew. A heading that hid the
/// `/Line` would quietly contradict the argument that put the row here.
#[must_use]
pub fn comment_row_ce_dimension_heading(subtype: &str, page_number: usize) -> String {
    format!("ce dimension ({subtype}) — p. {page_number}")
}

/// A row's byline — its author and its modification date, when it has them.
///
/// `None` when it has neither, so the caller draws nothing rather than an
/// empty line. See [`comments_excluded`] on why absence is an `Option` here.
///
/// # Both halves are legitimately absent, and neither absence is a fault
///
/// `/T` is a **Table 170 markup key**, so it is legitimately absent on a
/// `/Link` or a `/PrinterMark`, where `None` means *"this subtype has no such
/// concept"* rather than *"anonymous"*
/// (`D:\Dev\pdfcer\crates\pdfcer-core\src\annot.rs:340-347`). Printing an
/// "(unknown author)" placeholder would turn a correct fact about a subtype
/// into a claim about a person.
///
/// `/M` is optional on everything.
///
/// # The date is passed through verbatim, and that is not laziness
///
/// §12.5.2 gives `/M`'s type as *"date **or** text string"* and requires a
/// conforming reader to *"accept and display a string in any format"*, so
/// `pdfcer-core` stores it raw and its own docs say *"do not assume it
/// parses"*. Formatting it here would mean writing a §7.9.4 parser whose
/// failure mode is either rejecting a value the standard requires be accepted
/// or silently mangling it. The word "modified" carries the meaning; the value
/// carries whatever the file said. [`comment_row_modified_tooltip`] is where
/// the operator finds that out.
#[must_use]
pub fn comment_row_byline(author: Option<&str>, modified: Option<&str>) -> Option<String> {
    match (author, modified) {
        (Some(a), Some(m)) => Some(format!("by {a} · modified {m}")),
        (Some(a), None) => Some(format!("by {a}")),
        (None, Some(m)) => Some(format!("modified {m}")),
        (None, None) => None,
    }
}

/// Why a modification date can look like machine output.
///
/// On hover rather than on the row, because it is the answer to a question
/// most operators will never ask: the ordinary case is `D:20240117093000Z`,
/// which is legible enough to compare two rows by, and the sentence below is
/// only wanted by whoever wonders why pdfcer did not tidy it.
#[must_use]
pub fn comment_row_modified_tooltip() -> &'static str {
    "Shown exactly as the file wrote it. The standard lets this be a date or any text at all, and requires a reader to accept whatever is there, so pdfcer does not reformat it."
}

/// A row's note body.
///
/// Salvaged verbatim, and it is a passthrough on purpose: the operator is
/// reading somebody else's words, and a catalog entry that decorated them
/// would be putting pdfcer's voice inside a quotation.
#[must_use]
pub fn comment_row_body(text: &str) -> String {
    text.to_owned()
}

/// A row's caption when the annotation has no `/Contents`.
///
/// Salvaged verbatim. "No note text" rather than blank space: an empty row is
/// indistinguishable from a rendering failure, and this is a real, expected
/// state — see [`comments_all_without_notes`] for the reason it is *usually*
/// this state on a document pdfcer drew on.
///
/// **Worded as a fact about the document, not as an error.** There is no
/// "missing", no "(none)", no empty-set glyph and no warning colour. The
/// annotation is exactly as its author left it; a panel that dressed that up
/// as absent data would send an operator looking for damage in an ordinary
/// file — the same defect `crate::panels::bookmarks`' three-state row
/// distinction exists to avoid.
#[must_use]
pub fn comment_row_no_note() -> &'static str {
    "No note text on this markup."
}

/// A **ce dimension**'s caption when it has no `/Contents`.
///
/// A ce dimension never has note text, and for a different reason from the
/// shapes [`comments_all_without_notes`] covers: its measurement is baked into
/// its own appearance stream by `author_dimension`, so the number the operator
/// reads is *on the page*, not in a note. "No note text on this markup" would
/// be true and would read as a shortcoming; this says where the text actually
/// is.
#[must_use]
pub fn comment_row_ce_dimension_no_note() -> &'static str {
    "No note text. A ce dimension carries its measurement in its own appearance on the page rather than as a note."
}

/// The caption under `/Contents` on a subtype that does not display text.
///
/// # The trap this closes
///
/// `docs/core-api/03-capabilities.md:1113` states it as a trap in so many
/// words: *"`/Contents` is dual-purpose — a UI labelling this 'comment' is
/// right for markup and wrong for a Link."* §12.5.2 defines the key as *"text
/// displayed for the annotation, **or** (if the type does not display text) an
/// alternate human-readable description"* for accessibility (§14.9.3), and
/// which of the two it is depends entirely on the subtype.
///
/// So on a `/Link`, a `/Movie` or a `/PrinterMark` the string above this
/// caption is not something a reviewer wrote — it is the document's
/// description of a control, addressed to a screen reader. Showing it in a
/// list headed "Comments" without saying so invites an operator to reply to
/// nobody.
#[must_use]
pub fn comment_row_description_caption() -> &'static str {
    "This is the annotation's accessibility description, not a note somebody wrote — this kind of annotation displays no text of its own."
}

/// A row whose annotation the file says not to show on screen.
///
/// # Listed and marked, never omitted
///
/// `docs/core-api/03-capabilities.md:1100` is explicit: *"an annotation the
/// file says not to show. A Comments panel that silently omits it is hiding
/// document content; list it and mark it hidden."*
///
/// The predicate is `AnnotFlags::suppressed_on_screen`, which is `/F` bit 2
/// (Hidden) or bit 6 (NoView) — §12.5.3 Table 165. The wording says *"you will
/// not find this on the page"* rather than *"this is hidden"* because the
/// operator's next act is to click Go to and look for it.
#[must_use]
pub fn comment_row_hidden() -> &'static str {
    "The document marks this one as not shown on screen, so it will not be on the page when you get there."
}

/// A row whose appearance state pdfcer could not resolve.
///
/// # Why this is here at all, and why rule 4 makes it mandatory
///
/// `docs/core-api/03-capabilities.md:1093`: `Appearance::StateUnresolved`
/// means pdfcer *"displays nothing and does not guess a first / `On` / `Off`
/// key"*, and *"a blank annotation with no explanation looks like a rendering
/// bug."*
///
/// The governing setting is `MissingAppearanceState`, whose default
/// (`PaintNothing`) is documented in core as **evidence tier (d), a reasoned
/// guess**. That makes the blank page an *inference of pdfcer's*, and rule 4
/// says an inference the operator cannot see still owes an off-canvas report:
/// *"Render normally; report separately. Both."* A panel is the right home for
/// it, and nothing may be drawn on the page to mark it.
#[must_use]
pub fn comment_row_appearance_unresolved() -> &'static str {
    "pdfcer could not work out which appearance this annotation should use, so it draws nothing for it. That is pdfcer declining to guess, not a fault in the page."
}

/// A row that is a **reply** to another annotation (`/IRT` with `/RT /R`).
///
/// Flat rather than indented under its target, and that is a scope decision
/// worth stating: threading the list would need the panel to resolve every
/// `/IRT` into a row, decide what to do about a dangling one, and pick a depth
/// for a cycle. This says the same fact in one line and leaves the ordering —
/// page order, then `/Annots` order — meaning exactly what it says.
///
/// Table 170 makes `/RT` default to `R` when absent, which is the ordinary
/// case for a threaded comment; the panel asks
/// `Annotation::effective_reply_type` rather than reading `/RT` itself, for
/// the reason core's docs give: *"a call site that treats an absent `/RT` as
/// 'not a reply' is wrong in the ordinary case."*
#[must_use]
pub fn comment_row_is_reply() -> &'static str {
    "A reply to another annotation on this document."
}

/// A row that is a `/RT /Group` **subordinate**.
///
/// # The one row in this panel where pdfcer and another reader will disagree
///
/// §12.5.6.2 says a group subordinate's own `Contents`, `M`, `C`, `T`,
/// `Popup`, `CreationDate`, `Subj` and `Open` *"shall be ignored"* in favour
/// of the group primary's. `pdfcer-core` **deliberately does not apply that
/// rule** — `Annotation::contents` is the raw dictionary value, because
/// *"silently substituting a primary's `/Contents` for a subordinate's would
/// make the model disagree with the file, which is the one thing
/// `pdfcer-core`'s read half must never do"* (`annot.rs:326-335`).
///
/// Both halves of that are right, and together they mean the note text on this
/// row is text the standard instructs a conforming reader **not** to display.
/// Saying so is rule 4's *"pdfcer inferred something, and another reader may
/// compute different values"* in its purest form: nothing here is wrong, and
/// an operator who does not know it will be surprised by another viewer.
#[must_use]
pub fn comment_row_is_group_member() -> &'static str {
    "Part of a group. The standard says a reader should show the group's main annotation's note here instead; pdfcer shows what this one actually says, so another viewer may show something different."
}

/// The go-to-page button on a row.
///
/// Salvaged verbatim.
#[must_use]
pub fn comment_row_goto() -> &'static str {
    "Go to"
}

/// Its tooltip — names the page, because the button sits in a list of many.
///
/// Salvaged verbatim.
#[must_use]
pub fn comment_row_goto_tooltip(page_number: usize) -> String {
    format!("Show page {page_number}, where this is")
}

/// **The control that opens the note editor on a row that has no note.**
///
/// Two labels rather than one, because *add* and *edit* are different acts to
/// the operator and the difference is legible from the row: a row showing
/// somebody's words offers to change them, a row saying "No note text" offers
/// to write some. A single "Note…" would make the operator read the row above
/// the button to find out what pressing it does.
///
/// # ★ Why this exists only from 2026-08-28
///
/// `pdfcer-core` had no verb that could set `/Contents` on an annotation that
/// **already exists** until `Pass 154.0`, so every shape this shell drew was
/// permanently wordless and this panel was a viewer. That is recorded in
/// [`comments_all_without_notes`]' doc comment as a *capability that does not
/// exist yet*; it exists now, and the sentence there has been corrected rather
/// than deleted.
#[must_use]
pub fn comment_row_add_note() -> &'static str {
    "Add note"
}

/// The same control on a row that already carries note text.
#[must_use]
pub fn comment_row_edit_note() -> &'static str {
    "Edit note"
}

/// The editor's Save.
///
/// **An explicit commit, not a live binding**, for the reason
/// `crate::panels::properties::geometry` states about its own Apply: one
/// keystroke per undo entry would make `Ctrl+Z` walk backwards through a
/// sentence one letter at a time. One press is one `CommandKind::SetMarkupNote`.
#[must_use]
pub fn comment_row_note_save() -> &'static str {
    "Save note"
}

/// The editor's Cancel — abandons the draft and writes nothing.
#[must_use]
pub fn comment_row_note_cancel() -> &'static str {
    "Cancel"
}

/// **Remove the note entirely**, leaving the shape on the page.
///
/// A separate control from Save-with-empty-text because `pdfcer-core` models
/// them as separate verbs and says why: *"an empty comment is a comment, and a
/// reviewer deleting their remark is not the same as leaving a blank one."*
/// `clear_markup_note` removes `/Contents`, `/T` and `/M` together.
#[must_use]
pub fn comment_row_note_remove() -> &'static str {
    "Remove note"
}

/// Its tooltip — says what survives, because the button sits next to a Delete
/// in the operator's mental model even though it is not one.
#[must_use]
pub fn comment_row_note_remove_tooltip() -> &'static str {
    "Remove the words, the author and the date. The markup itself stays on the page."
}

/// The hint under the editor while it is open.
///
/// Names the two keys that are NOT obvious in a multi-line box: Enter inserts
/// a line rather than saving, so the operator needs telling how to save, and
/// Escape is the standard abandon.
#[must_use]
pub fn comment_row_note_hint() -> &'static str {
    "Enter starts a new line. Press Save note to write it, or Escape to abandon it."
}

/// **What the editor will write into `/T`, disclosed before it is written.**
///
/// Rule 4's surviving half: the author name is invisible on the page — a
/// sticky's byline lives in a pop-up window this shell does not draw, and a
/// shape's lives nowhere at all — so an operator who has never opened Settings
/// has no way to discover what name their comments carry, or that they carry
/// none.
///
/// # ★ Why it names the setting rather than the value
///
/// A panel body is handed `&OpenDoc` and `&mut PanelsState` and **nothing
/// else** — no preferences — so this string cannot quote the configured name
/// without threading prefs through every panel signature in the crate for one
/// sentence. Naming the place answers the operator's real question (*where do
/// I change what my comments say?*), and the value itself becomes visible the
/// moment the note is saved, as the row's own byline.
///
/// Shown only when the row has **no** author, because on a row that has one
/// nothing is written to `/T` at all: `pdfcer-core` leaves an omitted key
/// untouched, which is what stops correcting a typo from un-signing somebody
/// else's comment.
#[must_use]
pub fn comment_row_note_signature() -> &'static str {
    "Saving signs this with the name in Settings > Comments, and dates it. Leave that name blank to comment anonymously."
}

/// The same disclosure on a row that **already has an author** — what is
/// preserved, rather than what is written.
///
/// The operator is about to change somebody's words, and the thing they cannot
/// see is that the byline will not move with them. Saying so is what stops the
/// panel looking as though it silently re-attributed a comment.
#[must_use]
pub fn comment_row_note_signature_kept(author: &str) -> String {
    format!("This note stays credited to {author}. Saving updates its date.")
}

/// The caption on a row whose note cannot be edited here, and where its text
/// actually lives.
///
/// A **ce dimension** is refused by `pdfcer-core` **by name** — its `/Contents`
/// is generated from the measurement by `author_dimension`, so a note written
/// over it would be silently regenerated away. Saying where the text comes from
/// is more use than a greyed button, and R9 forbids the greyed button anyway:
/// this is not *temporarily* unavailable.
#[must_use]
pub fn comment_row_note_not_editable_ce_dimension() -> &'static str {
    "A ce dimension's text comes from its measurement. Use the Measure tools to change it."
}

/// The caption on a row whose annotation is written as a **direct dictionary**
/// and therefore has no object id to name.
///
/// §12.5.2 Table 164 requires an annotation dictionary to be an indirect
/// object, so this is a malformed file rather than a shortcoming of pdfcer, and
/// the row says which. Nothing that needs a handle may be offered for it.
#[must_use]
pub fn comment_row_note_no_handle() -> &'static str {
    "This annotation is written into the page rather than as its own object, so pdfcer cannot address it."
}

/// **The heading of the row whose annotation is selected on the canvas.**
///
/// # ★★★ A word, not a colour
///
/// `DEFECTS.md` **D2** is this project's record of a theme making text
/// invisible against its own background — near-white on light grey, shipped,
/// with two theme tests sitting next to it that measured no
/// foreground/background pair. Every list in this shell that marks a row marks
/// it with a **shape or a word**: the Pages panel changes a tile's outline
/// *and* writes a count, and the Objects tree indents.
///
/// A reviewer scanning forty rows for the cloud they just drew needs that mark
/// to survive a theme nobody has measured yet.
///
/// # Why it wraps the heading rather than sitting on its own line
///
/// Because the row is already between two and seven lines tall and a separate
/// line would put the mark a variable distance from the thing it marks. The
/// arrow leads, so a column of headings scanned down the left edge shows it
/// without reading a word.
#[must_use]
pub fn comment_row_selected_heading(heading: &str) -> String {
    format!("> {heading} — selected on the page")
}

// ---------------------------------------------------------------------------
// The filter strip, and Delete — added 2026-09-05
// ---------------------------------------------------------------------------

/// ★★★ **The disclosure a filtered list owes**, above the rows.
///
/// This panel's founding discipline is that *"nothing is silently omitted"* —
/// [`comments_excluded`] already states the arithmetic for widgets, pop-ups
/// and `/TrapNet`. A filter is a fourth kind of omission and the only one the
/// operator caused, which makes it **more** important to state rather than
/// less: an exclusion is a property of the document a reviewer can learn once,
/// while a filter is a switch they set an hour ago and have since forgotten.
///
/// A reviewer who reads six rows off a drawing they know carries forty and
/// concludes the other thirty-four are gone has been misled by this surface.
#[must_use]
pub fn comments_filtered(shown: usize, total: usize) -> String {
    format!("Showing {shown} of {total}. A filter is hiding the rest.")
}

/// The control that puts every row back.
///
/// One press rather than three menus reset one at a time, because the state a
/// reviewer wants back is *"all of them"* and reaching it by undoing each
/// choice is three chances to leave one set.
#[must_use]
pub fn comment_filter_clear() -> &'static str {
    "Show all"
}

/// The author chooser's label.
///
/// *Author*, not *Reviewer*: `/T` is §12.5.6.4 Table 170's *"name of the
/// person who created the annotation"*, and this panel lists `/Link`s and
/// stamps as readily as review comments. Calling the column Reviewer would
/// name a role the file does not record.
#[must_use]
pub fn comment_filter_author() -> &'static str {
    "Author"
}

/// The type chooser's label. *Type* rather than *Subtype*, because the values
/// under it are the file's own spellings and the operator does not need the
/// dictionary key's name to use them.
#[must_use]
pub fn comment_filter_type() -> &'static str {
    "Type"
}

/// The "no filter" entry in either chooser.
#[must_use]
pub fn comment_filter_all() -> &'static str {
    "All"
}

/// The switch that hides rows carrying no note text.
///
/// ★ It exists because of a property of pdfcer rather than of PDF:
/// `MarkupSpec` has no contents field on any variant, so **every shape this
/// program draws arrives with no `/Contents`**. On a drawing marked up here
/// the list is mostly rows with nothing to read, and this is the switch that
/// leaves the remarks somebody actually wrote.
#[must_use]
pub fn comment_filter_with_note() -> &'static str {
    "With text only"
}

/// The ordering chooser's label.
#[must_use]
pub fn comment_sort_label() -> &'static str {
    "Order"
}

/// The default ordering — page order, then `/Annots` order.
///
/// Named *By page* rather than *Document order*, because the operator's
/// question is *"where is it"* and the sheet number is the answer they act on.
#[must_use]
pub fn comment_sort_document() -> &'static str {
    "By page"
}

/// Ordering by `/T`.
#[must_use]
pub fn comment_sort_author() -> &'static str {
    "By author"
}

/// Ordering by `/Subtype`.
#[must_use]
pub fn comment_sort_subtype() -> &'static str {
    "By type"
}

/// ★★★ **Delete this comment** — the control this panel spent its whole life
/// without.
///
/// # The header that forbade this string was true and stopped being true
///
/// `crate::panels::comments`' header said, until 2026-09-05: *"This build has
/// no Delete, because `crate::app::actions::Action` has no variant that could
/// carry the intent."* That was correct when written. `AnnotAction::Delete`
/// has existed since; the canvas Delete key and the Format tab have both been
/// reaching `EditSession::delete_annotation` through it, and only this panel —
/// **the reviewer's own work list** — could not. A reviewer deletes their own
/// comment, and Acrobat has it.
///
/// ⇒ The sixth time this project has found a *"we cannot do this"* sentence
/// outliving its reason. Corrected in place and dated, in that module's header
/// and in this one's, rather than left as two answers.
#[must_use]
pub fn comment_row_delete() -> &'static str {
    "Delete comment"
}

/// Its tooltip, carrying the three things `docs/core-api/03-capabilities.md`
/// §3.4 requires a delete to disclose.
///
/// What goes, that **delete is not redaction**, and — implied by the second —
/// that the words may still be in the file. The collateral (a pop-up removed,
/// replies orphaned, group members promoted) is reported *after* the call by
/// [`crate::text::markup::deleted_collateral`], because only the engine knows
/// what it actually took.
#[must_use]
pub fn comment_row_delete_tooltip() -> &'static str {
    "Remove this markup and its note from the page. This is not redaction: saving without rewriting the whole file leaves the previous revision in place."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fixed sentence in this module, for the sweeps below.
    ///
    /// Hand-written, like every enumeration of things Rust cannot enumerate
    /// for us. It is only used by tests, so an entry missed here weakens a
    /// check rather than shipping a defect — but it is listed in the order
    /// the panel draws them so a reader can diff the two.
    ///
    /// A function rather than a `const`, for the same reason
    /// `crate::app::modes::defaults`' `SideSpec` is owned rather than
    /// `&'static`: these are ordinary functions, and an ordinary call cannot
    /// be promoted into a `const` initializer.
    fn all_fixed() -> [&'static str; 11] {
        [
            comments_none(),
            comments_all_without_notes(),
            comment_row_modified_tooltip(),
            comment_row_no_note(),
            comment_row_ce_dimension_no_note(),
            comment_row_description_caption(),
            comment_row_hidden(),
            comment_row_appearance_unresolved(),
            comment_row_is_reply(),
            comment_row_is_group_member(),
            comment_row_goto(),
        ]
    }

    /// **★ Rule 15 — no string here writes a bare "dimension".**
    ///
    /// *"**ce dimensions** are the ones pdfcer authors; **pdf dimensions** are
    /// CAD-exported page content pdfcer reads and must not silently alter. They
    /// have opposite properties and the ambiguity has already sent one
    /// investigation down the wrong path."*
    ///
    /// Swept rather than reviewed because a catalog is exactly the kind of
    /// file where a bare noun arrives in a late reword — someone shortens a
    /// sentence that ran long in a screenshot, and the qualifier is the first
    /// thing to go. The two headings that legitimately contain the word are
    /// checked for their qualifier rather than exempted, so the test still
    /// bites if one of them loses it.
    #[test]
    fn no_string_here_says_a_bare_dimension() {
        let qualified = |s: &str| {
            // Every occurrence must be preceded by "ce " or "pdf ".
            let lower = s.to_lowercase();
            let mut at = 0;
            while let Some(found) = lower[at..].find("dimension") {
                let start = at + found;
                let before = &lower[..start];
                if !(before.ends_with("ce ") || before.ends_with("pdf ")) {
                    return false;
                }
                at = start + "dimension".len();
            }
            true
        };

        for s in all_fixed() {
            assert!(
                qualified(s),
                "this string writes a bare \"dimension\": {s}. Rule 15 — say \
                 \"ce dimension\" or \"pdf dimension\"."
            );
        }
        // The two formatted entries that DO name the concept, checked rather
        // than exempted.
        let heading = comment_row_ce_dimension_heading("Line", 3);
        assert!(qualified(&heading), "{heading}");
        assert!(qualified(comment_row_ce_dimension_no_note()));
        // …and the sweep can actually fail, which is the half a green run
        // never proves.
        assert!(
            !qualified("the dimension is wrong"),
            "the rule-15 check cannot detect its own violation"
        );
    }

    /// **The exclusion line is `None` when nothing was excluded.**
    ///
    /// The alternative — an empty string — still reserves a label's height,
    /// which on a narrow dock reads as a rendering fault, and it is the
    /// no-placeholders rule applied to prose.
    #[test]
    fn nothing_excluded_draws_nothing() {
        assert_eq!(comments_excluded(0, 0, 0), None);
    }

    /// …and it names every kind that was excluded, and only those.
    ///
    /// The failure this stops is the one-clause version that says "12 items
    /// were not listed": the count without the destination, which tells an
    /// operator something is missing and not where to look for it.
    #[test]
    fn the_exclusion_line_names_where_each_kind_went() {
        let only_widgets = comments_excluded(12, 0, 0).expect("something was excluded");
        assert!(only_widgets.contains("12"), "{only_widgets}");
        assert!(only_widgets.contains("Forms panel"), "{only_widgets}");
        assert!(
            !only_widgets.contains("pop-up"),
            "a kind that was not excluded must not be mentioned: {only_widgets}"
        );

        let all_three = comments_excluded(1, 2, 3).expect("something was excluded");
        for n in ["1", "2", "3"] {
            assert!(all_three.contains(n), "{all_three}");
        }
        assert!(all_three.contains("pop-up"), "{all_three}");
        assert!(all_three.contains("prepress"), "{all_three}");
    }

    /// **The byline is `None` only when the annotation has neither half.**
    ///
    /// Both halves are legitimately absent — `/T` is a Table 170 markup key
    /// and means "this subtype has no such concept" on a `/Link` — so all four
    /// combinations are reachable on real documents and each has to render
    /// correctly. The `(None, None)` case in particular must not become an
    /// empty line.
    #[test]
    fn a_byline_with_neither_half_is_not_drawn() {
        assert_eq!(comment_row_byline(None, None), None);
        assert_eq!(
            comment_row_byline(Some("Ken"), None).as_deref(),
            Some("by Ken")
        );
        let modified = comment_row_byline(None, Some("D:20240117093000Z"))
            .expect("a date alone is still a byline");
        assert!(modified.contains("D:20240117093000Z"), "{modified}");
        assert!(
            !modified.contains("by "),
            "an absent author must not produce the word \"by\": {modified}"
        );
        let both = comment_row_byline(Some("Ken"), Some("D:20240117093000Z"))
            .expect("both halves is still a byline");
        assert!(both.contains("Ken") && both.contains("D:2024"), "{both}");
    }

    /// **The date is passed through byte for byte.**
    ///
    /// §12.5.2 makes `/M` *"date or text string"* and requires a reader to
    /// accept any format, so `pdfcer-core` stores it raw. A catalog entry that
    /// tidied it would either reject a value the standard requires be accepted
    /// or silently mangle it — and the mangling would look like a document
    /// fact rather than pdfcer's own edit.
    #[test]
    fn a_modification_date_is_never_reformatted() {
        for raw in [
            "D:20240117093000Z",
            "D:19980223085612-04'00",
            "last Tuesday",
            "",
        ] {
            let line = comment_row_byline(None, Some(raw));
            match line {
                Some(line) => assert!(line.contains(raw), "{raw} was altered: {line}"),
                None => panic!("a present /M must produce a byline, even an odd one"),
            }
        }
    }

    /// **The "no note" caption reads as a fact, not as an error.**
    ///
    /// The whole point of the sentence. Note text is absent on every shape
    /// pdfcer itself drew — `MarkupSpec` carries no contents field, deliberately
    /// — so this caption is the *ordinary* case on a pdfcer-marked document, and
    /// a word like "missing" or "error" would send an operator hunting for
    /// damage in a file that has none.
    #[test]
    fn an_absent_note_is_never_described_as_missing_or_broken() {
        for caption in [comment_row_no_note(), comment_row_ce_dimension_no_note()] {
            let lower = caption.to_lowercase();
            for alarm in ["missing", "error", "fail", "invalid", "corrupt", "warning"] {
                assert!(
                    !lower.contains(alarm),
                    "`{caption}` describes an ordinary state with the word \
                     \"{alarm}\", which reads as damage"
                );
            }
        }
        // And the document-wide disclosure says the absence is expected, in
        // that word — `03-capabilities.md:1085` asks for it in these terms.
        assert!(
            comments_all_without_notes().contains("expected"),
            "{}",
            comments_all_without_notes()
        );
    }

    /// **Every fixed sentence is prose or a label, never both halves of one.**
    ///
    /// `crate::text`'s convention: a label is a name and carries no trailing
    /// period; a message is a statement and does. [`comment_row_goto`] is the
    /// only label here, and it is the only entry allowed to end without
    /// punctuation.
    #[test]
    fn prose_is_punctuated_and_the_one_label_is_not() {
        for s in all_fixed() {
            assert!(!s.is_empty(), "an empty catalog entry");
            if s == comment_row_goto() {
                assert!(!s.ends_with('.'), "`{s}` is a button label, not a sentence");
            } else {
                assert!(
                    s.ends_with('.'),
                    "`{s}` is prose and must be punctuated as a sentence"
                );
            }
        }
    }

    /// **No two of these sentences are the same.**
    ///
    /// Each one distinguishes a *different* state — an absent note, a hidden
    /// annotation, an unresolved appearance, a reply, a group member — and two
    /// that read alike would collapse two states the operator has to be able
    /// to tell apart. Same reasoning as
    /// `crate::panels::bookmarks`' three-row-state check, which exists because
    /// a heading and a broken destination rendering identically would send an
    /// operator hunting for damage in an ordinary document.
    #[test]
    fn every_row_state_says_something_different() {
        let mut seen: Vec<&str> = Vec::new();
        for s in all_fixed() {
            assert!(!seen.contains(&s), "two states share the sentence `{s}`");
            seen.push(s);
        }
    }

    /// The two page-numbered entries print the number they were given.
    ///
    /// The off-by-one guard, from the other side. `Action::GoToPage` takes a
    /// **0-based** index and these take a **1-based** human page number, so the
    /// `+ 1` happens exactly once, at the call site — see
    /// `crate::panels::comments`' own test for the half of this that pins the
    /// action.
    #[test]
    fn a_page_number_reaches_the_string_unchanged() {
        assert!(comment_row_heading("Circle", 7).contains('7'));
        assert!(comment_row_ce_dimension_heading("Line", 7).contains('7'));
        assert!(comment_row_goto_tooltip(7).contains('7'));
        // The subtype survives too, including the malformed-annotation label
        // core hands over when `/Subtype` is absent.
        assert!(comment_row_heading("(no Subtype)", 1).contains("(no Subtype)"));
    }

    /// A note body is returned byte for byte.
    ///
    /// The operator is reading somebody else's words; a catalog entry that
    /// decorated them would put pdfcer's voice inside a quotation.
    #[test]
    fn a_note_body_is_not_decorated() {
        for text in ["Check this weld", "  leading space", "", "多行\ntext"] {
            assert_eq!(comment_row_body(text), text);
        }
    }
}
