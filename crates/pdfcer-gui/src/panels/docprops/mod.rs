//! # `panels::docprops` — the **Document properties** panel: this file's own
//! title, author, subject and keywords, and the facts pdfcer read about it
//!
//! ## ★★★ Why this is a panel of its own — the operator, 2026-09-05
//!
//! > *"the document properties are still always visible in the properties tab.
//! > it needs to get out of there and be in its own document properties tab."*
//!
//! Until this commit every line below was the **last section of the selection
//! inspector** (`crate::panels::properties`), drawn under the armed tool's
//! settings, the markup restyle controls, the ce-dimension section and the
//! focused object's read-only facts. It was the only one of those with no
//! condition attached to it at all, so the *"This document"* heading was on
//! screen every frame of every session, at the bottom of a panel whose subject
//! is what you have selected.
//!
//! ★★ **The previous arrangement was DESIGNED, not accidental, and this is a
//! supersession rather than a bug fix.** `file.properties`' shipped tooltip
//! commissioned both halves in one sentence — *"The document's own title,
//! author, subject and keywords, and the properties of whatever is selected on
//! the page."* — and `RIBBON_IA.md` §5.1 put that one command in File ▸
//! Document. One command, two subjects, one panel. That reading was coherent
//! and it is the operator's to overrule; he has.
//!
//! ★★★ **And it was the last thing in the inspector that was not the detail of
//! anything.** `OPERATOR_REQUESTS.md` O123 / A7 is his: *"I never understood
//! why there is a tool dock when everything can be in object and properties."*
//! Objects and Properties became one master–detail column so that picking a row
//! shows that row's detail — `app::modes::defaults`' Edit arm builds exactly
//! that shape, two adjacent stacks in one column with a draggable split. A
//! permanent document-metadata block at the foot of the detail pane is the one
//! block in that column that answers no selection, so it made the master–detail
//! reading false for every operator who scrolled to the bottom. That is why the
//! move is worth the churn, and it is the whole of the justification.
//!
//! O75 is the same complaint arriving a fortnight earlier and being answered
//! with a **collapse** — *"the Properties section is always showing the This
//! document properties instead of just the properties of the objects I am
//! editing."* The section learned to fold itself shut whenever a
//! selection-scoped section above it had spoken. That machinery is deleted with
//! this move rather than kept: there is nothing above this section any more, so
//! `anything_above` would be permanently `false` and the edge-triggered
//! `set_open`/`store` dance would be a mechanism whose input never changes. The
//! reasoning it recorded — why neither `default_open` nor `open(Some(_))` can
//! express a collapse the operator may override — is preserved in
//! `crate::panels::properties`' header, because it is a finding about egui 0.35
//! and not about this panel.
//!
//! ## What it is, and what it is not
//!
//! | | |
//! |---|---|
//! | **subject** | the file: `/Info`, its size, its version, its sheet count and size, its encryption, and whether pdfcer had to rebuild its index to open it |
//! | **command** | `file.document_properties`, on **File ▸ Document**, beside Properties and Fonts — *"inspection, not action"*, per `shell::manifest::ladder` |
//! | **modes** | all three. Reading a document's title is **reading**, and Read is shown the `file` tab |
//! | **not here** | anything scoped to a selection. Every one of those sections stayed in `crate::panels::properties`, which is now purely the detail of what is picked |
//!
//! ## ★ R9 — what it shows with no document open
//!
//! Nothing of its own. [`crate::panels::Panel::show`] answers the empty case
//! **once**, for every panel, before any body runs: it forgets the panel state
//! and draws `crate::text::panels::panel_no_document`. So this module is never
//! called without a document and has no empty state to get wrong — which is
//! also why there is no "no document" sentence written here to drift from the
//! other eleven.
//!
//! An empty *field* is a different matter and is answered below: a PDF with no
//! `/Info` dictionary at all renders four empty boxes, because absent is a
//! value and an empty box is how absent is spelled.
//!
//! ## ★ The blocker this module's own header used to quote
//!
//! It opened, for months, by quoting `Panel::command_id`:
//!
//! > Only the second half is built here; the first needs a `/Info` accessor
//! > that `pdfcer-core` does not expose on `Document` at all.
//!
//! ## ★ That last clause was TRUE when written and false when read
//!
//! `EditSession::info_text` (`edit.rs:4663`) and `info_bytes`
//! (`edit.rs:4644`) both exist, both are `&self`, and both are documented as
//! *"reflects unsaved edits"*. `InfoField::all()` (`edit.rs:224`) exists too,
//! and its own doc comment was written **for this panel**:
//!
//! > Every editable field, in the order a properties panel should show them.
//! > Provided so a front end enumerates the real list instead of hard-coding
//! > one that drifts when a field is added.
//!
//! So the blocker had already cleared and the prose had not moved. That is the
//! **sixth** stale claim of this class found in this project, and the previous
//! five are recorded in `NO_SURFACE.md` §4 and `RIBBON_IA.md` §5.6. The
//! generalisable part is the one those notes already state: *a measurement in
//! a document is a measurement with a timestamp*, and a blocker quoted in
//! prose is a measurement. **Re-run it before believing it**, especially when
//! it names a crate somebody else is working on in parallel.
//!
//! ## ★ The disclosure this surface owes, and it is not the obvious one
//!
//! Not "these are the metadata fields". It is `InfoText::exact`:
//!
//! > `true` when every byte was decoded with certainty. When `false`,
//! > re-encoding [`InfoText::text`] would **not** reproduce the original
//! > bytes, so a front end must not write the field back unless the operator
//! > actually changed it.
//!
//! A `/Title` written in an encoding pdfcer cannot fully resolve comes back
//! with U+FFFD where the unmappable bytes were. The operator sees a plausible
//! string. If the panel then wrote it back — on a focus change, on a save, on
//! any "keep everything in sync" impulse — it would **replace the document's
//! own bytes with pdfcer's guess at them**, silently, in a field nobody looks
//! at twice.
//!
//! Two things follow, and the second is the one that is easy to skip:
//!
//! 1. **Never write a field the operator did not change.** Discharged by
//!    construction: this module commits through
//!    [`crate::panels::forms::rows::commit`], whose second condition is
//!    exactly *the draft differs from what the document already holds*. It is
//!    the same function the Forms panel and the canvas form editor use, so
//!    there is one rule and three callers rather than three rules.
//! 2. **Say so.** Rule 4's half that survives is the inference the operator
//!    *cannot see* — and a substituted character in a metadata field is
//!    exactly that. The row carries a sentence when `exact` is false. It is a
//!    fact about the **document**, not a pdfcer failure, and is worded that way.
//!
//! ## Why an empty field CLEARS rather than sets an empty string
//!
//! `set_info_field(field, None)` removes the key; `Some("")` would write an
//! empty string object. They are different documents, and the one an operator
//! means by deleting the contents of a box is the first: a document with no
//! title, not a document whose title is nothing.
//!
//! The row says so, because it is not guessable and because it is the one
//! action here that *removes* something.

//! ## ★ Why `Action::SetInfoField` takes `Option<String>` and not `String`
//!
//! Moved here from that variant's doc comment on 2026-09-03, when
//! `action.rs` reached R2's 1,500-line limit and the seam turned out to be
//! that the enum was carrying the rationale for mechanisms living in other
//! modules. The documentation belongs beside the mechanism; the variant keeps
//! a summary and points here.
//!
//! `None` is a different edit, not an empty one.
//! `EditSession::set_info_field(field, None)` **removes the key** from the
//! `/Info` dictionary; `Some("")` writes an empty string object. A document
//! with no title and a document whose title is the empty string are different
//! files, and the first is what an operator means by deleting the contents of a
//! box.
//!
//! Collapsing the two — taking a `String` and treating empty as clear — would
//! work today and would make the distinction unrepresentable, which is the shape
//! `pdfcer-core`'s own `Some(Tolerance::None)` vs `None` note warns about one
//! feature along.
//!
//! ## Why it goes through the funnel when the edit is one dictionary entry
//!
//! Not for size — for **ordering and the undo log**. It is a document change
//! like any other, and the funnel is what makes it appear once in the command
//! log, bump the epoch once, and be undone by one `Ctrl+Z`.
//!
//! ★ And the epoch bump is load-bearing here in a way it is not elsewhere: this
//! panel re-seeds its text drafts whenever the epoch moves, which is what makes
//! `Ctrl+Z` visibly restore the old value in the box. Applying the edit outside
//! the funnel would leave the box holding a string the document no longer has,
//! and the next focus change would write it back — an undo the panel silently
//! reverses.

use egui::Ui;
use pdfcer_core::edit::{InfoField, InfoText};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;

/// The region this panel publishes.
///
/// ★★ **Unchanged by the move to a panel of its own, deliberately.** The name
/// is a harness interface: `ui-verify`'s `properties_metadata_round_trips`
/// finds the section by this string and finds each editor by
/// [`REGION_FIELD_PREFIX`], and that check **cannot be run by the session that
/// made this move** — the machine's pointer belongs to another track. Renaming
/// the regions would put an unverifiable change into the one instrument that
/// could confirm the panel still works, on top of a verifiable one (which tab
/// the check must bring to the front). One change at a time is what keeps the
/// next run's verdict readable.
pub const REGION: &str = "properties.info"; // ui-text-exempt: trace region name, never displayed
/// The prefix of the per-field editor regions; the field's index in
/// `InfoField::all()` is appended.
///
/// Indexed by **position in the engine's own list**, not by a name spelled
/// here, so a field the engine adds is addressable by a check without this
/// constant changing.
pub const REGION_FIELD_PREFIX: &str = "properties.info."; // ui-text-exempt: trace region name, never displayed

/// How many fields `InfoField::all()` returns.
///
/// ★ Derived from the engine's array rather than written as `4`, because
/// [`InfoDrafts`] holds a fixed-size array of drafts and the two must be the
/// same length. A fifth field added upstream changes `all()`'s return type,
/// which changes this, which changes `[String; FIELDS]` — so the drafts follow
/// automatically instead of the fifth field being dropped off the end.
///
/// **This is the protection the LABEL function cannot have.** `InfoField` is
/// `#[non_exhaustive]`, so a `match` on it in this crate needs a `_` arm and
/// compiles for ever whatever is added — see
/// `crate::text::panels::docprops::info_label`. Here the
/// dependency is on the array's *length*, which is a type-level fact
/// `#[non_exhaustive]` does not weaken.
const FIELDS: usize = InfoField::all().len();

/// The operator's half-typed metadata, between frames.
///
/// ## Why a draft exists at all
///
/// Because `TextEdit` needs a `&mut String` that survives the frame, and
/// because committing on every keystroke would make one typed word a dozen
/// undo entries. The draft is what the operator has typed; the document is
/// what it will be compared against when focus leaves.
///
/// ## ★ Why it reloads on the edit epoch
///
/// The drafts are re-seeded whenever `doc.edit_epoch` moves, and that is what
/// makes **undo work in this panel**. `Ctrl+Z` after setting a title runs the
/// engine command backwards and bumps the epoch; without the reload the box
/// would still show the title the document no longer has, and the next focus
/// change would write it straight back — an undo the panel silently reverses.
///
/// It also covers the case nobody thinks of: a field changed by some *other*
/// surface. There is only one today (this panel), and the reload means there
/// does not have to be a rule about it.
#[derive(Default)]
pub struct InfoDrafts {
    /// One draft per `InfoField::all()` position.
    drafts: [String; FIELDS],
    /// The `edit_epoch` the drafts were seeded from, or `None` before the
    /// first seed.
    ///
    /// `Option` rather than a sentinel: epoch 0 is a real, common value — it
    /// is what every freshly opened document has — and a `0` sentinel would
    /// make "never seeded" and "seeded from an unedited document"
    /// indistinguishable, which is the state this panel spends most of its
    /// life in.
    seeded_at: Option<u64>,
}

impl std::fmt::Debug for InfoDrafts {
    /// Lengths, not contents.
    ///
    /// A document's `/Title` and `/Author` are the operator's own data and
    /// this type's `Debug` reaches the trace, which is written to a file a
    /// harness keeps. `PagesUi` makes the same choice for its selection.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfoDrafts")
            .field("lengths", &self.drafts.each_ref().map(String::len))
            .field("seeded_at", &self.seeded_at)
            .finish()
    }
}

impl InfoDrafts {
    /// Re-seed the drafts from the document when the revision has moved.
    ///
    /// Returns the current values as the document holds them, so the caller
    /// compares against **the document** rather than against the draft it just
    /// wrote — two different questions, and only the first is what
    /// [`crate::panels::forms::rows::commit`] wants.
    fn sync(&mut self, doc: &OpenDoc) -> [Option<InfoText>; FIELDS] {
        let stored = InfoField::all().map(|field| doc.session.info_text(field));
        if self.seeded_at != Some(doc.edit_epoch) {
            self.seeded_at = Some(doc.edit_epoch);
            for (draft, value) in self.drafts.iter_mut().zip(stored.iter()) {
                draft.clear();
                if let Some(info) = value {
                    draft.push_str(&info.text);
                }
            }
        }
        stored
    }
}

/// **Draw the Document properties panel.**
///
/// # Why there is no "nothing to show" state
///
/// Every PDF has these four fields, in the sense that matters: absent is a
/// value, and an empty box is how absent is spelled. A document with no
/// `/Info` dictionary at all renders four empty boxes, which is the truth and
/// is also exactly what the operator needs in order to add one — `set_info_field`
/// **creates `/Info` if it is absent**.
///
/// The *no document at all* case never reaches here; see the module header's R9
/// section.
///
/// # ★ One scroll area, round everything, and it is here rather than in the
/// dock
///
/// The four editors, the seven facts and the recovery note together are taller
/// than a 320 pt inspector at most window sizes, and a dock pane does not
/// scroll its body for a panel. `crate::panels::properties::body` carries the
/// same wrapper and the measurement behind it: before it existed, the one
/// control that commits an edit was laid out below the window with no scrollbar
/// and no gesture that would reach it, and it was reported as a dead button.
///
/// ★ No nested scroll area anywhere inside: a scroll area inside a scroll area
/// steals the wheel from its parent depending on where the pointer happens to
/// sit, which is a worse surface than the one being fixed.
///
/// # ★ No collapsing header any more
///
/// The section used to be a `CollapsingState` that folded itself shut whenever
/// a selection-scoped section above it had spoken (O75). Nothing is above it
/// now. A header that can only ever be open is a control that does nothing,
/// and the panel's own tab is where an operator closes this surface.
pub fn body(ui: &mut Ui, doc: &OpenDoc, drafts: &mut InfoDrafts, actions: &mut Vec<Action>) {
    egui::ScrollArea::vertical()
        .id_salt("docprops-body")
        .auto_shrink([false, false])
        .show(ui, |ui| info_body(ui, doc, drafts, actions));
}

/// The panel's contents, inside the scroll area [`body`] wraps them in.
///
/// Split out so the scroll area is impossible to forget: content added to this
/// function is inside it by construction, where content appended to [`body`]
/// after the `.show(..)` call would silently be outside it again.
fn info_body(ui: &mut Ui, doc: &OpenDoc, drafts: &mut InfoDrafts, actions: &mut Vec<Action>) {
    // No `.strong()` — R84 / DEFECTS.md D11: egui resolves it to the
    // accent-filled widget foreground, which on an ordinary panel is pale text
    // on pale ground.
    ui.label(t::heading());
    ui.label(egui::RichText::new(t::note()).small().weak());
    recovery_note(ui, doc);
    ui.add_space(4.0);

    facts(ui, doc);
    ui.add_space(6.0);

    let stored = drafts.sync(doc);

    for (index, field) in InfoField::all().into_iter().enumerate() {
        // `get_mut`/`get` rather than indexing: both arrays are `[_; FIELDS]`
        // and the loop is over a `[_; FIELDS]`, so this cannot fail — and
        // `clippy::indexing_slicing` is denied crate-wide precisely so that a
        // future change which *could* fail is written as a decision.
        let (Some(draft), Some(current)) = (drafts.drafts.get_mut(index), stored.get(index)) else {
            continue;
        };
        row(ui, index, field, draft, current.as_ref(), actions);
    }

    // ★ `ui_rect_visible` rather than `ui_rect`, and published LAST — the
    // reason survives the move even though the collapse that prompted it did
    // not. This body is inside a `ScrollArea`, and a rect published for a
    // scrolled-out control gets CLICKED by the harness at a coordinate the
    // operator can never reach. `geometry::section` paid for that once; its
    // answer is copied rather than re-derived.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
}

/// The read-only facts about the file itself.
///
/// # Why these five, and not the twenty Acrobat's Description tab shows
///
/// Each one below is answerable **from data already loaded**, and each answers
/// a question an operator actually opens this panel with: *which file is this,
/// how big is it, what will read it, how many sheets, what size are they, and
/// is it locked.* Nothing here costs a parse, a walk or an allocation beyond a
/// string.
///
/// What is deliberately absent is anything pdfcer would have to **infer** —
/// producer and creator strings are not in `InfoField`, so they are neither
/// read nor written here; permissions are discussed at
/// [`t::encryption_note`]; and page-level facts beyond the sheet
/// size belong to the object half of this panel.
///
/// # ★ Every value is read through `session.document()`, and one of them lies
/// if you do not qualify it
///
/// `EditSession::document()` is documented as *"the base revision, not the
/// edited state"*. For the version, the encryption and the page geometry that
/// is exactly right — none of them is something this build can edit. For
/// `bytes().len()` it is **the file as it was opened**, which is a different
/// number from what a save would write the moment anything is edited. So the
/// size row carries a sentence while `is_modified()` is true, and does not
/// otherwise.
fn facts(ui: &mut Ui, doc: &OpenDoc) {
    let base = doc.session.document();

    fact(ui, t::file_label(), &file_name(doc));
    fact(
        ui,
        t::size_label(),
        &crate::text::panels::byte_size(base.bytes().len()),
    );
    if doc.session.is_modified() {
        ui.weak(t::size_is_base());
    }
    fact(ui, t::version_label(), &base.version().to_string());
    fact(
        ui,
        t::pages_label(),
        &crate::text::pages::pages_count(doc.pages.len()),
    );
    if let Some(size) = sheet_size(doc) {
        fact(ui, t::page_size_label(), &size);
    }

    let encrypted = base.encryption().is_some();
    fact(
        ui,
        t::encryption_label(),
        if encrypted {
            t::encrypted()
        } else {
            t::not_encrypted()
        },
    );
    if encrypted {
        ui.weak(t::encryption_note());
    }
}

/// One label-and-value row, in the same shape the object half uses.
fn fact(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        ui.label(value);
    });
}

/// The document's file name, or the sentence for one that has none.
///
/// ★ `Origin` rather than a path check. A document `file.new` created has a
/// `path` that is a **name** — `text::files::untitled` — and nothing is at it,
/// which `Origin::Created`'s own doc comment states. Showing it as a file would
/// tell the operator their work is somewhere it is not, and the one panel
/// headed *"This document"* is the worst place available to be wrong about
/// that.
fn file_name(doc: &OpenDoc) -> String {
    if doc.origin == crate::app::state::Origin::Created {
        return t::file_unsaved().to_owned();
    }
    doc.path.file_name().map_or_else(
        || doc.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The sheet size in millimetres, saying so when the sheets differ.
///
/// ★ **Mixed is the common case for this operator, not an edge case.** A
/// drawing set is an A1 general arrangement with A3 details behind it, and
/// reporting page one's size alone would be a true number that reads as a
/// claim about the whole document.
///
/// Compared at **millimetre** resolution rather than exactly, because a CAD
/// exporter's A3 and a scanner's A3 differ in the sixth decimal of a point and
/// an operator does not have two sheet sizes because of that.
fn sheet_size(doc: &OpenDoc) -> Option<String> {
    let first = doc.pages.first()?;
    let (w, h) = crate::viewer::page_extent_pts(first);
    let (w_mm, h_mm) = (w / PTS_PER_MM, h / PTS_PER_MM);
    let mixed = doc.pages.iter().skip(1).any(|page| {
        let (ow, oh) = crate::viewer::page_extent_pts(page);
        (ow / PTS_PER_MM - w_mm).abs() >= 1.0 || (oh / PTS_PER_MM - h_mm).abs() >= 1.0
    });
    Some(if mixed {
        t::page_size_mixed(w_mm, h_mm)
    } else {
        t::page_size(w_mm, h_mm)
    })
}

/// Points per millimetre.
///
/// A PDF user-space unit is 1/72 inch by definition (§8.3.2.3), and an inch is
/// 25.4 mm. The same constant `panels::pages` carries for its tile tooltip;
/// duplicated rather than shared because a two-term definition restated is
/// cheaper to read than an import that sends the reader to another module for
/// a number they already know.
const PTS_PER_MM: f32 = 72.0 / 25.4;

/// One field: its label, its editor, and whatever the document owes about it.
fn row(
    ui: &mut Ui,
    index: usize,
    field: InfoField,
    draft: &mut String,
    current: Option<&InfoText>,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        ui.label(t::info_label(field));
        let response = ui.add(egui::TextEdit::singleline(draft).desired_width(200.0));
        crate::diag::ui_rect(
            // ui-text-exempt: trace region name, never displayed
            &format!("{REGION_FIELD_PREFIX}{index}"),
            response.rect,
        );

        // ★ The commit rule is the FORMS panel's, called rather than restated.
        //
        // Its two conditions bind here for the same two reasons they bind
        // there — `lost_focus`, because `TextEdit::changed()` fires per
        // keystroke and one typed word must not be a dozen undo entries; and
        // draft-differs-from-stored, because tabbing THROUGH a field the
        // operator did not touch must not write a command.
        //
        // The second condition is also what discharges `InfoText::exact`'s
        // obligation: a field pdfcer could not decode exactly is never written
        // back unless the operator changed it, which is the engine's rule
        // verbatim and is true here by construction rather than by a guard
        // somebody has to remember.
        let stored_text = current.map_or("", |info| info.text.as_str());
        let ended = response.lost_focus();
        if crate::panels::forms::rows::commit(ended, draft.as_str(), stored_text).is_some() {
            let value = if draft.trim().is_empty() {
                // ★ Empty CLEARS the key rather than writing an empty string.
                // See the module header: a document with no title and a
                // document whose title is nothing are different documents, and
                // the first is what deleting the contents of a box means.
                //
                // `trim` on the test but not on the value: a title of "  " is
                // not a title, and a title of " Site Plan " is one the operator
                // may have spaced deliberately.
                None
            } else {
                Some(draft.clone())
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed. LENGTH,
                // not the text: this is the operator's own document metadata
                // and the trace is written to a file a harness keeps.
                format!(
                    "info-field-commit field={field:?} clearing={} chars={}",
                    u8::from(value.is_none()),
                    draft.chars().count()
                )
            });
            actions.push(Action::SetInfoField { field, value });
        }
    });

    // ★ The disclosure, and it is under the row rather than beside it because
    // it is a sentence. Drawn only when the decode was lossy, which on an
    // ordinary document is never — a note on every row would be noise that
    // trains the operator to skip the one that matters.
    if current.is_some_and(|info| !info.exact) {
        ui.weak(t::info_not_exact());
    }
}

/// **Say when pdfcer had to rebuild this file's index to open it at all.**
///
/// # ★★★ The last silence of the family, and the quietest one
///
/// A PDF carries a cross-reference table: an index saying where every object
/// lives. When it is wrong — truncated download, a writer that crashed, a disc
/// error, a tool that appended badly — `pdfcer-core` does not refuse the file.
/// It **scans the whole thing and rebuilds the index from what it finds**, and
/// the document then opens and looks completely normal.
///
/// `Document::recovery()` has carried the report of that since the engine
/// landed and **this shell never called it**. `NO_SURFACE.md` §3b recorded it as
/// unreachable on 2026-08-17 and it was still unreachable today.
///
/// It is the same shape as the two silences `pdfcer-core` broke this week — a
/// search that found nothing, and a redaction that marked nothing, over text
/// that was never readable. In every case the screen looks right and the
/// operator has no way to know. Rule 4's less-remembered half: **an inference
/// the operator cannot see still owes them a report.**
///
/// ★ Why it matters more for a CAD drawing than for a letter. A rebuilt index
/// is a *best reading of damaged bytes*. `last_wins_collisions` counts objects
/// that were defined more than once, where pdfcer had to pick one — and on a
/// drawing, a wrong pick is a line in the wrong place, on a page that renders
/// perfectly. Nobody proofreads a titleblock against a file they believe is
/// intact.
///
/// ★★ Off-canvas, in Properties, and **not** a banner over the page. The
/// document is not in doubt as *drawn*; what is in doubt is how it was
/// *assembled*. A badge on the page would be a second rendering path for
/// content that is fine — the bug class decision 059 narrows rule 4 to prevent
/// — and it would nag on every document that had ever been touched by a bad
/// writer, which is a great many of them.
fn recovery_note(ui: &mut Ui, doc: &OpenDoc) {
    let Some(report) = doc.session.document().recovery() else {
        return;
    };
    ui.label(egui::RichText::new(t::recovered_heading()).color(ui.visuals().warn_fg_color));
    ui.label(
        egui::RichText::new(t::recovered_detail(
            report.file_level_objects + report.objstm_objects,
            report.last_wins_collisions,
            report.stream_lengths_recovered + report.missing_endobj_recovered,
        ))
        .small()
        .weak(),
    )
    .on_hover_text(t::recovered_tooltip());
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The draft array is exactly as long as the engine's field list.
    ///
    /// ★ The point of the assertion is the **direction it fails in**. A fifth
    /// `InfoField` added to `pdfcer-core` changes `InfoField::all()`'s return
    /// type, `FIELDS` follows it, and `[String; FIELDS]` follows that — so the
    /// build breaks at the array rather than the fifth field being silently
    /// dropped off the end of a hard-coded four. This test states the property
    /// so a reader does not have to derive it from three `const` definitions.
    #[test]
    fn the_draft_array_tracks_the_engines_field_list() {
        assert_eq!(FIELDS, InfoField::all().len());
        let drafts = InfoDrafts::default();
        assert_eq!(drafts.drafts.len(), InfoField::all().len());
        assert_eq!(
            drafts.seeded_at, None,
            "a fresh panel has never been seeded, and epoch 0 is a real value \
             that must not be confused with that"
        );
    }

    /// Every field has a label, and no two share one.
    ///
    /// A duplicated label would be two boxes an operator cannot tell apart,
    /// which is worse than a missing one because it looks like it works.
    #[test]
    fn every_info_field_is_labelled_and_no_label_repeats() {
        let labels: Vec<&str> = InfoField::all().into_iter().map(t::info_label).collect();
        let unique: std::collections::BTreeSet<&str> = labels.iter().copied().collect();
        assert_eq!(
            unique.len(),
            labels.len(),
            "two fields share a label: {labels:?}"
        );
        assert!(labels.iter().all(|l| !l.is_empty()));
    }

    /// ★ Clearing is expressed as `None`, and only a genuinely empty draft
    /// clears.
    ///
    /// The rule is stated here as data rather than exercised through a frame,
    /// because the branch it protects is one line and the consequence is a
    /// removed dictionary key. A draft of `"  "` clears; a draft of
    /// `" Site Plan "` does not, and keeps its spaces.
    #[test]
    fn only_a_blank_draft_removes_the_key() {
        for blank in ["", " ", "\t", "\n  "] {
            assert!(blank.trim().is_empty(), "{blank:?} must clear the field");
        }
        assert!(
            !" Site Plan ".trim().is_empty(),
            "a spaced title is a title the operator may have meant"
        );
    }

    /// The commit rule is the Forms panel's, not a second copy.
    ///
    /// Asserted by calling it: tabbing through an untouched field writes
    /// nothing, and a changed field with focus still held writes nothing
    /// either. If this module ever grows its own predicate, one of these two
    /// is what will fail.
    #[test]
    fn tabbing_through_a_field_writes_nothing() {
        use crate::panels::forms::rows::commit;
        assert_eq!(commit(true, "Site Plan", "Site Plan"), None);
        assert_eq!(commit(false, "Site Plan", ""), None);
        assert_eq!(commit(true, "Site Plan", ""), Some("Site Plan".to_owned()));
    }
}
