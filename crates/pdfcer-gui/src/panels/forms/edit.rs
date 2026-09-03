//! # `panels::forms::edit` — the eight things a Forms panel can ask for
//!
//! One enum and one function. The enum is the complete vocabulary of what
//! filling a form means in this build; the function is the only place any of
//! it reaches an [`EditSession`].
//!
//! ## Why one [`crate::app::actions::Action`] variant and not eight
//!
//! [`crate::app::actions`]' header claims four properties for the action
//! funnel, and the fourth — *"every state change is greppable"* — is the one
//! that decides the shape here. It survives a nested enum intact:
//! `FormEdit::Flatten` is exactly as greppable as `Action::FlattenForm` would
//! be, and `grep FormEdit::` answers "what can change a form?" completely.
//!
//! What a nested enum buys is that the **panel owns its own vocabulary**.
//! Eight flat variants would put eight form-shaped concepts —
//! fully-qualified names, on-state names, a recompute plan — into a module
//! whose other variants are zooms and page steps, and every one of them would
//! need an arm in `PdfcerApp::apply` that reached back into this module for the
//! verb anyway. So the seam is drawn where the knowledge is: `Action::Form`
//! carries the intent across the funnel, and [`apply`] is what knows how to
//! honour it.
//!
//! ## ★ The four-step mutation protocol, and why it is repeated here
//!
//! [`crate::app::actions::vector_edit`] is the same protocol for the vector
//! verbs, and this is deliberately **not** a call into it. Two reasons, and
//! the first is decisive:
//!
//! 1. **The signatures do not unify.** Every vector verb returns
//!    `Result<Vec<String>, EditError>` — a disclosure list. The form verbs
//!    return six different outcome types (`FillOutcome`, `ResetOutcome`,
//!    `FlattenOutcome`, `RegenOutcome`, `()`, and a `Vec` of fills), none of
//!    which is a `Vec<String>`. A shared helper would need a type parameter
//!    per verb and a closure per call, which is the same code with a generic
//!    bolted on.
//! 2. **`vector_edit` is private** to `crate::app::actions`, and this module
//!    may not edit that file this round.
//!
//! So the protocol is restated, and stated in full, because **each of the four
//! steps is a separate way to end up with an edit that is silently declined or
//! a page that silently keeps drawing what was just changed**:
//!
//! 1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so
//!    a worker can hold a clone while it rasterizes, and `Arc::get_mut` fails
//!    while any other strong reference exists. Cancelling first is what turns
//!    "sometimes refused, depending on how fast the page rasterized" into
//!    "always applied".
//! 2. **Mutate through `Arc::get_mut`.** A `None` is not a panic: it means
//!    something else still holds the session, which is a bug in the caller's
//!    ordering rather than in the operator's document. Traced and declined,
//!    because declining an edit is recoverable and corrupting one is not.
//! 3. **Bump `edit_epoch`.** Filling a field rewrites its widget's appearance
//!    stream, which is page content, so the canvas's decomposition and the
//!    Objects panel's paint-order indices are stale. It is also what
//!    `crate::panels::PanelsState::sync` keys on.
//! 4. **Drop the cached texture.** Nothing else notices an edit: the render
//!    key compares page index and raster scale, and a fill changes neither.
//!    Without this the page keeps showing the empty box until the operator
//!    zooms or pages away.
//!
//! ## ★ Almost nothing travels back — and the exception is the interesting part
//!
//! ### The rule
//!
//! The old shell wrote an operator-facing note into `doc.pending_note` after
//! every one of these verbs: *"Filled X"*, *"X changed, but this document has
//! no drawn appearance for that state"*, *"saved, but this form also carries
//! an XFA packet"*. This build has no such channel, and the panel does not ask
//! for one, because **every fact those notes carried is derivable from the
//! document the panel re-reads on the next frame**:
//!
//! | Old note | Where it is now |
//! |---|---|
//! | "Filled X" | The row shows the new value. |
//! | XFA may disagree | [`crate::text::forms::forms_xfa_note`], stated **before** anything is typed, because `AcroForm::xfa` is a property of the file. |
//! | no appearance for that state | The control is disabled up front — the state pdfcer would write does not exist, so the call would refuse (see [`crate::panels::forms::rows`]). |
//! | "Reset N fields" | The reset preview recomputes and lists nothing left to clear. |
//! | "Recomputed N fields" | The plan recomputes and reports every calculation already correct. |
//! | "Flattened N fields" | There is no longer an `/AcroForm`, and the panel says so. |
//!
//! This is `crate::panels::layers`' lesson one surface over: that panel
//! computes "how far has the operator diverged?" by **comparing sets** rather
//! than by counting clicks, and its own docs record that counting clicks was
//! subtly wrong. Deriving from the document is both simpler and more correct
//! than carrying a note, because a note can outlive the fact it describes and
//! a derivation cannot.
//!
//! ### ★ The exception: two facts a fill knows and the document does not
//!
//! The argument above has one precondition — *the fact is re-derivable from the
//! document next frame* — and exactly two things `FillOutcome` reports fail it:
//!
//! | fact | why it is not re-derivable |
//! |---|---|
//! | `applied_autosize` | the field's `/DA` asked for size 0 and **pdfcer chose a number**. What lands in the file is the chosen size; the fact that pdfcer chose it, rather than the document stating it, is gone the moment the command returns. |
//! | `unencodable_chars` | characters with no `WinAnsi` code were **silently replaced with `?`**. The saved value is the substituted one, so re-reading the field tells you what pdfcer wrote and never that it wrote something else. |
//!
//! Both are inferences pdfcer made on the operator's behalf, and rule 4's whole
//! subject is inferences: *"pdfcer inferred something, and another reader may do
//! otherwise"*. So both are captured — see [`FillDisclosure`] — and shown
//! **off-canvas**, in the panel, where rule 4 says a disclosure belongs.
//!
//! The four remaining `FillOutcome` fields stay discarded, and each for the
//! reason above: `widgets_updated` is `Field::widgets.len()`, `top_index` is
//! `/TI` and is in the file, `xfa_may_disagree` is `AcroForm::xfa` and is
//! stated **before** anything is typed, and `field_id` is the field the panel
//! is already looking at.
//!
//! **What is left over is a refusal**, and it is traced rather than surfaced —
//! the same posture, and the same acknowledged gap, as
//! `crate::app::actions::vector_edit`, whose own header names it. That is
//! defensible here for a reason it is not there: every refusal these verbs can
//! raise is **asked about before the control is drawn**
//! (`EditSession::fill_refusal`, `EditSession::deletion_refusal`, the per-row
//! block reasons), so reaching the trace at all means a precondition changed
//! between the frame that drew the control and the frame that applied it. See
//! this module's `KNOWN GAPS` section below for the two cases where that is
//! genuinely reachable.
//!
//! ## KNOWN GAPS — reported, not worked around
//!
//! Both are `pdfcer-core` boundary findings rather than shell defects, recorded
//! here because `pdfce_FeatureRequests/README.md`'s decision 058 says a
//! workaround that is not reported is a boundary defect that stays.
//!
//! 1. **`EditSession::fill_refusal` is a strict subset of what a fill
//!    enforces.** The verbs call the private `fill_guards`, which checks
//!    `/Encrypt`, the `/P`-aware certification gate **and** the suppressed-
//!    object guard; `fill_refusal` checks only the middle one. So it can
//!    answer `None` on a document where the fill then returns
//!    `DocumentEncrypted` or `ObjectCreationWouldExposeHiddenObjects`. The
//!    encryption arm is unreachable from this shell — an encrypted document is
//!    refused at open, see `crate::text::open_needs_password` — and the
//!    suppressed-object arm has no public accessor at all, so the panel cannot
//!    compensate for it. `fill_refusal` should mirror `fill_guards`.
//! 2. **RESOLVED — `EditSession::flatten_refusal` now exists** (pdfcer
//!    `fa243df`), and `panels::forms::mod` asks it. The gap was real:
//!    flatten shares deletion's strict certification gate but additionally
//!    creates page content, so it carries a suppression guard deletion does
//!    not — two checks of three, which works until it does not.
//!
//!    The half of the same report that claimed `deletion_refusal`
//!    under-reported was **rejected, and rightly**: it predicts DELETION and
//!    matches `deletion_preflight` exactly. The comparison had been against
//!    flatten. Acting on it would have disabled a Delete control that would
//!    have worked, and core now carries a test whose job is to stop a future
//!    reader "correcting" a correct function on the strength of it.

use std::cell::RefCell;
use std::sync::Arc;

use pdfcer_core::edit::{EditError, EditSession, FillOutcome};

use crate::app::state::OpenDoc;

/// What a fill decided that the document cannot afterwards be asked.
///
/// See this module's header, "The exception". Two facts, both **inferences
/// pdfcer made** rather than instructions the file gave, and both invisible in
/// the saved bytes: an auto-size pdfcer chose, and characters it replaced with
/// `?` because the field's font could not encode them.
///
/// # Why the field name travels with them
///
/// A fill raised from the **canvas** happens where the operator is looking and
/// a fill raised from the **panel** happens in a list of forty rows. In both
/// cases the disclosure is read somewhere other than where the value was
/// typed, so a sentence that said only *"the size was chosen for you"* would
/// leave the operator to guess which field it was about.
///
/// The **value** is deliberately absent, exactly as it is absent from
/// [`FormEdit::label`]: a `/Ff` `Password` field's contents must not travel any
/// further than the box they were typed into.
#[derive(Clone, Debug, PartialEq)]
pub struct FillDisclosure {
    /// The field's fully-qualified name.
    pub field: String,
    /// The revision this describes — [`OpenDoc::edit_epoch`] **after** the
    /// fill. A disclosure whose epoch is not the document's current one
    /// describes an edit that has since been undone or superseded, and must
    /// not be shown.
    pub epoch: u64,
    /// `Some(size)` when the field's `/DA` asked for auto-size and pdfcer
    /// picked this one.
    pub applied_autosize: Option<f64>,
    /// How many characters had no `WinAnsi` code and were replaced with `?`.
    pub unencodable_chars: usize,
}

impl FillDisclosure {
    /// Whether there is anything here worth a sentence.
    ///
    /// The overwhelmingly common fill discloses nothing, and a panel that drew
    /// an empty disclosure line under every edit would train the operator to
    /// stop reading the ones that matter — the same argument
    /// `crate::app::status::page_box`'s `Note` makes for having no `Ok`
    /// variant.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.applied_autosize.is_none() && self.unencodable_chars == 0
    }
}

thread_local! {
    /// The most recent fill's disclosures, waiting to be read by the panel.
    ///
    /// # ★ Why a thread-local and not a field on `OpenDoc`
    ///
    /// It should be a field on [`OpenDoc`], beside `edit_epoch`, dropped with
    /// the document — and the constraint is a **boundary rather than a design
    /// judgement**, the same one [`crate::panels::forms::FormsUi`]'s header
    /// records for the drafts: `OpenDoc` is declared in `crate::app::state`,
    /// which this work may not extend. Stated here rather than left for
    /// somebody to find, so that whoever lifts the constraint knows what the
    /// preferred shape is.
    ///
    /// Why it is nonetheless sound rather than a smuggled mutation: this is
    /// **not document state**. It is a note about an edit that has already
    /// happened through the funnel, it cannot change a pixel of the page, and
    /// nothing reads it except a panel deciding whether to draw a sentence.
    /// It is also correctly scoped — `eframe`'s update loop is one thread, so
    /// the writer and the reader are the same thread, and a test running on
    /// another one gets its own empty slot rather than another test's leftovers
    /// (which a `static Mutex` would hand it).
    ///
    /// Staleness is handled by the `epoch` rather than by clearing: a
    /// disclosure is shown only while it describes the revision on screen, so
    /// an undo silences it without anything having to remember to.
    static LAST_FILL: RefCell<Option<FillDisclosure>> = const { RefCell::new(None) };
}

/// What the last fill disclosed, if it still describes the open document.
///
/// **The panel's read** — see [`crate::panels::forms::body`]. Returns `None`
/// when the last fill was on another document, has been undone, or had nothing
/// to disclose.
#[must_use]
pub fn last_fill_disclosure(epoch: u64) -> Option<FillDisclosure> {
    LAST_FILL.with_borrow(|slot| {
        slot.as_ref()
            .filter(|d| d.epoch == epoch && !d.is_empty())
            .cloned()
    })
}

/// Record a fill's disclosures.
fn record_fill_disclosure(disclosure: Option<FillDisclosure>) {
    LAST_FILL.with_borrow_mut(|slot| *slot = disclosure);
}

/// Plant a disclosure, for tests in other modules that must draw one.
///
/// `#[cfg(test)]` so it cannot become a second way to record one — the real
/// path is [`record_fill_disclosure`], called from `apply` with the epoch the
/// fill produced, and a second entry point is how two callers come to
/// disagree about what "the last fill" means.
///
/// It exists because the status bar draws this and must prove it does not
/// grow the bar while doing so (R128), and that measurement has to happen in
/// `crate::app::status`, which cannot reach a `thread_local` here.
#[cfg(test)]
pub(crate) fn plant_fill_disclosure_for_test(disclosure: FillDisclosure) {
    record_fill_disclosure(Some(disclosure));
}

/// One thing an operator asked the Forms panel to do.
///
/// Every variant is reachable from a real control today. A variant nothing can
/// raise is dead code wearing a design pattern, and the "no placeholders"
/// invariant (`PROJECT_PLAN.md` §3) applies to enums as much as to labels.
///
/// **The operands travel with the intent.** A `String` field name rather than
/// an `ObjId`, because that is the vocabulary every one of the core verbs
/// takes and because a fully-qualified name survives the document being
/// re-parsed between the frame that raised the action and the frame that
/// applies it — an object id would too, but the verb would then have to
/// translate back, and two spellings of "which field" is one too many.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormEdit {
    /// Write `value` into the text field named `field`.
    ///
    /// Raised when a text row loses focus with a draft that differs from what
    /// the document holds — see [`crate::panels::forms::rows::commit`] for why
    /// both halves of that condition are needed.
    FillText {
        /// The field's fully-qualified name (§12.7.3.2).
        field: String,
        /// The plain text to store.
        value: String,
    },
    /// Write `value` into the rich-text field named `field`, **discarding its
    /// `/RV`**.
    ///
    /// A separate variant from [`Self::FillText`] rather than a flag, because
    /// it is a separate act: it destroys formatting, it is offered behind its
    /// own button with its own tooltip, and it calls a different verb
    /// (`fill_text_field_downgrading_rich_text`). A boolean would let a
    /// caller reach the destructive path by passing `true` to something whose
    /// name says "fill".
    ///
    /// `value` is the field's **current plain text**, unchanged — the button
    /// converts the field, it does not also retype it. The operator types
    /// afterwards, into the ordinary text row the conversion produces.
    ConvertRichTextToPlain {
        /// The field's fully-qualified name.
        field: String,
        /// The plain text to keep — the field's existing `/V`.
        value: String,
    },
    /// Select `state` on the check box or radio group named `field`.
    ///
    /// One variant for both, because it is one verb: a check box is a
    /// two-state button and a radio group is an n-state one, and
    /// `EditSession::set_button_state` takes the state name either way.
    ///
    /// `state` is `Off` to clear — the §12.7.4.2.3 name for the cleared state
    /// of every button, whatever its ON state happens to be called. Core
    /// accepts `Off` unconditionally and refuses any other name no widget
    /// defines, which is why the panel only ever offers names it read off the
    /// widgets.
    SetButtonState {
        /// The field's fully-qualified name.
        field: String,
        /// The on-state name, or `Off`.
        state: String,
    },
    /// Select `values` in the choice field named `field`.
    ///
    /// A `Vec` even for a single-select combo, because
    /// `EditSession::set_choice_value` takes a slice and a single-element
    /// slice is the honest way to say "one selection". A separate scalar
    /// variant would be a second spelling of the same command.
    ///
    /// The strings are **export** values where `/Opt` provides them
    /// (§12.7.4.4's `[export display]` pairs), because that is what `/V`
    /// stores.
    SetChoice {
        /// The field's fully-qualified name.
        field: String,
        /// The selections, in the order `/Opt` lists them.
        values: Vec<String>,
    },
    /// Write every value in a recompute plan the operator has just reviewed.
    ///
    /// # ★ The plan travels with the action rather than being recomputed
    ///
    /// [`apply`] could call `form_script::recompute::plan` itself and get the
    /// same answer — the action is applied in the same frame that raised it,
    /// against the same document. It does not, and the reason is rule 4: what
    /// the operator consented to is **the list of values that was on screen**,
    /// and an action is a complete statement of an operator's intent. Carrying
    /// the list makes "what did they agree to?" answerable from the action
    /// alone; recomputing it makes the answer depend on when the question is
    /// asked.
    ///
    /// **This is N undo entries, not one.** `pdfcer-core` has no batch verb —
    /// applying a plan is a loop the shell writes — so each pair below becomes
    /// its own `fill_text_field` command. Disclosed in
    /// [`crate::text::forms::recompute_apply_tooltip`], whose own doc comment
    /// records that the salvaged wording claimed otherwise.
    Recompute {
        /// `(fully-qualified name, proposed value)`, in evaluation order.
        changes: Vec<(String, String)>,
    },
    /// Return every eligible field to its `/DV`, or empty it (§12.7.5.3).
    ///
    /// No operand list: the panel offers only the whole-form reset, because
    /// the preview it shows above the button is the whole-form preview and a
    /// per-field reset control would need a per-field preview beside it to
    /// mean anything.
    Reset,
    /// Draw every field's current value into the document and clear
    /// `/NeedAppearances`.
    ///
    /// Not authoring, and the distinction is worth stating because this is the
    /// one variant here that does not change a **value**. It changes how the
    /// values already stored are *drawn*, which is the operator-facing answer
    /// to [`crate::text::forms::forms_need_appearances_note`] and the
    /// precondition for [`Self::Flatten`] keeping anything.
    RegenerateAppearances,
    /// Burn every field's appearance into page content and remove the form.
    ///
    /// See this module's header for why it carries a tooltip rather than a
    /// blocking confirmation.
    Flatten,
}

impl FormEdit {
    /// A short, stable name for the diagnostic trace.
    ///
    /// Separate from `Debug` on purpose: `Debug` prints the operands, which on
    /// a `Recompute` is the whole plan and on a `FillText` is whatever the
    /// operator typed — including into a `/Ff` `Password` field. A trace line
    /// is written to stderr and read by whoever is diagnosing a machine they
    /// cannot see, and neither of those belongs there.
    ///
    /// **That is not a hypothetical.** `crate::text::forms::form_field_password_tooltip`
    /// exists to tell an operator that a masked field is stored as plain text
    /// in the PDF; echoing it to stderr as well would be pdfcer widening the
    /// exposure it just warned about.
    const fn label(&self) -> &'static str {
        match self {
            Self::FillText { .. } => "form-fill-text",
            Self::ConvertRichTextToPlain { .. } => "form-convert-rich-text",
            Self::SetButtonState { .. } => "form-set-button-state",
            Self::SetChoice { .. } => "form-set-choice",
            Self::Recompute { .. } => "form-recompute",
            Self::Reset => "form-reset",
            Self::RegenerateAppearances => "form-regenerate-appearances",
            Self::Flatten => "form-flatten",
        }
    }
}

/// Apply one [`FormEdit`] to `doc`.
///
/// **The one place a form verb is called.** Called from
/// `PdfcerApp::apply`'s `Action::Form` arm; see this module's header for the
/// four-step protocol and for why it is restated here rather than shared with
/// `crate::app::actions::vector_edit`.
///
/// Reports nothing to the operator and returns nothing, for the reason set out
/// in the header: everything a report would have said is re-derived by the
/// panel from the document on the next frame. A refusal is traced.
pub fn apply(doc: &mut OpenDoc, edit: &FormEdit) {
    let label = edit.label();

    // 1. Stop the render worker, so `Arc::get_mut` can succeed.
    doc.render_worker.cancel_and_wait();

    // 2. Take the session mutably, or decline.
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused reason=session-borrowed")
        });
        return;
    };

    match run(session, edit) {
        Ok(Applied {
            commands,
            disclosed,
        }) => {
            // A verb that changed nothing must not invalidate anything. This
            // is reachable and not defensive: `Recompute` with an empty plan
            // is the obvious case, and bumping the epoch for it would drop
            // the page texture, throw away the canvas's resolved selection
            // and clear the Objects tree's expansion set — all to redraw a
            // document nobody touched.
            if commands == 0 {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("{label} commands=0 (nothing to do)")
                });
                return;
            }
            // 3. The document changed: every paint-order index and every
            //    cached decomposition describing it is now stale.
            doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
            // ★★★ **…and the per-page answer beside it** —
            // `OPERATOR_REQUESTS.md` O74, and this is the operator's literal
            // case: *"even just fill out a form … it seems to really slow down
            // clicking a checkbox."*
            //
            // Measured, before it was built: on his own 36-sheet SolidWorks
            // set, twelve visible thumbnails cost **666 ms of UI-thread work
            // after every fill**, because the rail's cache is keyed on the
            // document-wide `edit_epoch` and threw all of them away.
            //
            // `scope_of` is deliberately conservative and returns `None` for
            // every verb it has not established — see its own docs. A field
            // whose widgets straddle sheets is `None`; a `/P`-less widget is
            // `None`; `Reset`, `Flatten`, `RegenerateAppearances` and
            // `Recompute` are all `None`. Getting this wrong shows the operator
            // a picture of content he has already changed, which under rule 4
            // outranks the slowness it fixes.
            match scope_of(doc, edit) {
                Some(page) => doc.page_epochs.bump(page),
                None => doc.page_epochs.bump_all(),
            }
            // 4. Nothing else notices an edit — the render key compares page
            //    index and raster scale, and a fill changes neither. The epoch
            //    bump above is what `render::settle` reads, through
            //    `OpenDoc::page_texture_epoch`.
            //
            //    ★ This used to also assign `page_texture = None`, which
            //    blanked the page between the fill and the next raster. A form
            //    fill cannot change the page SET, so the old raster is an older
            //    picture of the same sheet and belongs on screen until the new
            //    one lands. See `actions::apply::vector_edit`.
            // ★ Stamped with the epoch bumped two lines above, NOT the one the
            // fill ran against. The panel shows a disclosure only while it
            // describes the revision on screen, and the revision on screen from
            // now until the next edit is this one — so an undo silences the
            // sentence by moving the epoch past it, with nothing anywhere that
            // has to remember to clear it. A disclosure stamped with the *old*
            // epoch would be invisible from the moment it was written.
            record_fill_disclosure(disclosed.map(|d| FillDisclosure {
                epoch: doc.edit_epoch,
                ..d
            }));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("{label} commands={commands} epoch={}", doc.edit_epoch)
            });
        }
        // Traced and the document left alone. Every refusal these verbs can
        // raise is asked about before the control is drawn, so reaching here
        // means a precondition moved between the frame that offered the
        // control and the frame that honoured it — or that one of the two
        // gaps in this module's header has been hit.
        Err(error) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("{label}-refused detail={error}")
        }),
    }
}

/// **Which single page this form edit could have changed, if exactly one.**
///
/// `OPERATOR_REQUESTS.md` O74. `None` means *"not established"* and is the
/// answer for everything this function has not proved, which is most of it.
///
/// # ★★★ Why every branch below defaults to `None`
///
/// A `Some(page)` is a promise that no rasteriser drawing any **other** sheet
/// would produce a different picture. If that promise is ever false the page
/// rail shows the operator content he has already changed — rule 4's *sneaky*,
/// which outranks the slowness this exists to fix. So the shape of every arm
/// is "prove it or say `None`", and the four whole-form verbs never even ask.
///
/// | verb | answer | why |
/// |---|---|---|
/// | `FillText`, `ConvertRichTextToPlain`, `SetButtonState`, `SetChoice` | the field's page, if its widgets share one | one `/T`, and §12.7.3.1 lets one field have widgets on several sheets — so this is checked, not assumed |
/// | `Recompute` | `None` | N fields, N undo entries, no reason to think they share a page |
/// | `Reset` | `None` | every eligible field in the document |
/// | `RegenerateAppearances` | `None` | every field's `/AP`, and `/NeedAppearances` on the catalog |
/// | `Flatten` | `None` | burns every widget into page content and removes the form |
///
/// # Why it reads the form AFTER the edit rather than before
///
/// Because the pages that need invalidating are the ones the widgets are on
/// **now**. None of these verbs moves a widget between sheets, so the two
/// readings agree today — but "they agree today" is the kind of premise that
/// stops being true silently, and reading after costs the same.
///
/// # What it costs
///
/// One `parse_acroform` walk per fill. On the operator's own set that is a
/// small fraction of the single thumbnail it saves, and it replaces twelve.
fn scope_of(doc: &OpenDoc, edit: &FormEdit) -> Option<usize> {
    let field_name = match edit {
        FormEdit::FillText { field, .. }
        | FormEdit::ConvertRichTextToPlain { field, .. }
        | FormEdit::SetButtonState { field, .. }
        | FormEdit::SetChoice { field, .. } => field.as_str(),
        // The four whole-form verbs. Listed rather than wildcarded so a fifth
        // variant added later fails to compile here and has to be ruled on,
        // instead of silently inheriting whichever answer a `_` arm gave.
        FormEdit::Recompute { .. }
        | FormEdit::Reset
        | FormEdit::RegenerateAppearances
        | FormEdit::Flatten => return None,
    };

    // Page object id -> page index, so a widget's `/P` can be resolved to the
    // position the caches are keyed on.
    let index_of = |id: pdfcer_core::object::ObjId| doc.pages.iter().position(|p| p.id == id);

    let view = doc.session.view();
    let form = pdfcer_core::forms::parse_acroform(&view)?;
    let field = form
        .fields
        .iter()
        .find(|f| f.fully_qualified_name == field_name)?;

    // Every widget must name a page, and they must all name the SAME one.
    // A widget with no `/P` is not an error — §12.5.6.19 makes the key
    // optional — it is simply a widget whose page this cannot establish, and
    // an unestablished page is `None`.
    let mut page: Option<usize> = None;
    for widget in &field.widgets {
        let here = index_of(widget.page?)?;
        match page {
            None => page = Some(here),
            Some(seen) if seen == here => {}
            Some(_) => return None,
        }
    }
    page
}

/// What one [`FormEdit`] did: how many undo commands it pushed, and whatever
/// it disclosed that the document cannot afterwards be asked.
///
/// A struct rather than a tuple because the two travel for different reasons —
/// the count decides whether to invalidate the page, the disclosure decides
/// whether the panel says a sentence — and a `(usize, Option<_>)` at four call
/// sites is four chances to read the pair in the wrong order.
#[derive(Debug, PartialEq)]
struct Applied {
    /// How many undo commands were pushed. See [`run`] on why a count.
    commands: usize,
    /// The inferences this fill made, if it made any.
    ///
    /// `None` for every verb that is not a fill: a reset, a flatten and an
    /// appearance regeneration all change the document without pdfcer choosing
    /// anything on the operator's behalf.
    disclosed: Option<FillDisclosure>,
}

impl Applied {
    /// A verb that changed `commands` things and inferred nothing.
    const fn plain(commands: usize) -> Self {
        Self {
            commands,
            disclosed: None,
        }
    }
}

/// Build a [`FillDisclosure`] from a fill's outcome.
///
/// `epoch` is filled in by [`apply`], which is the only place that knows the
/// revision the disclosure will be read against — see the ★ comment there.
fn disclosure_of(field: &str, out: &FillOutcome) -> FillDisclosure {
    FillDisclosure {
        field: field.to_owned(),
        epoch: 0,
        applied_autosize: out.applied_autosize,
        unencodable_chars: out.unencodable_chars,
    }
}

/// Run `edit` against `session`, returning how many **undo commands** it
/// pushed.
///
/// Split out from [`apply`] so the borrow of `doc.session` ends before the
/// epoch bump touches `doc`'s other fields, and so the verb dispatch is one
/// readable `match` uncluttered by the protocol around it.
///
/// # Why a command COUNT rather than `()`
///
/// Because two of the eight can legitimately do nothing, and "nothing
/// happened" must not look like "something happened":
///
/// - [`FormEdit::Recompute`] with an empty plan writes no field.
/// - [`FormEdit::Reset`] on a form that already holds its defaults commits a
///   command, but `ResetOutcome::fields_reset` is 0.
///
/// Returning the count lets [`apply`] skip the invalidation, which is the
/// difference between a no-op and a no-op that discards the page raster, the
/// canvas selection and the Objects panel's expansion state.
///
/// **It is a count of commands, not of fields**, and the two differ on exactly
/// one variant: `Recompute` pushes one per change. That is the distinction
/// `pdfce_FeatureRequests/README.md` warns about in general terms — a number a
/// verb hands back is not automatically the number the caller wanted — so the
/// unit is named in the return type's doc rather than left to the reader.
fn run(session: &mut EditSession, edit: &FormEdit) -> Result<Applied, EditError> {
    match edit {
        FormEdit::FillText { field, value } => {
            let out = session.fill_text_field(field, value)?;
            Ok(Applied {
                commands: 1,
                disclosed: Some(disclosure_of(field, &out)),
            })
        }
        FormEdit::ConvertRichTextToPlain { field, value } => {
            let out = session.fill_text_field_downgrading_rich_text(field, value)?;
            Ok(Applied {
                commands: 1,
                disclosed: Some(disclosure_of(field, &out)),
            })
        }
        FormEdit::SetButtonState { field, state } => {
            session.set_button_state(field, state)?;
            Ok(Applied::plain(1))
        }
        FormEdit::SetChoice { field, values } => {
            // `&[&str]` is what the verb takes; the borrow has to be
            // materialised because a `Vec<String>` cannot coerce to it.
            let refs: Vec<&str> = values.iter().map(String::as_str).collect();
            session.set_choice_value(field, &refs)?;
            Ok(Applied::plain(1))
        }
        FormEdit::Recompute { changes } => {
            // ★ STOPS AT THE FIRST REFUSAL, and leaves what landed.
            //
            // The alternative — carry on and report the failures at the end —
            // would be worse in the one case that matters. These values are
            // computed from each other: a plan is evaluated in dependency
            // order, so a field that refuses is one whose value the fields
            // after it were computed FROM. Writing those anyway would leave
            // the form internally inconsistent, with totals derived from an
            // operand that was never written.
            //
            // The partial write is not rolled back, because the shell has no
            // transaction spanning several commands (core's undo is per-verb)
            // and inventing one out of N undos would be a second, weaker
            // implementation of the undo stack. The operator's route back is
            // Ctrl+Z, once per field written — which is exactly what
            // `crate::text::forms::recompute_apply_tooltip` tells them.
            //
            // ★ The disclosure carried out of a recompute is the LAST fill's,
            // and that is a deliberate narrowing rather than an oversight: the
            // slot holds one, a plan can write forty, and a panel line naming
            // one of forty fields would be worse than one naming none. What
            // saves it from being a silent loss is that a recompute writes
            // *numbers pdfcer computed* — every one of them ASCII digits, a
            // separator and a sign — so `unencodable_chars` cannot fire, and
            // the only reachable disclosure is an auto-size, which is a
            // property of the field rather than of the value. Whoever gives
            // this a real home should make the slot a `Vec`.
            let mut written = 0usize;
            let mut last = None;
            for (field, value) in changes {
                let out = session.fill_text_field(field, value)?;
                last = Some(disclosure_of(field, &out));
                written += 1;
            }
            Ok(Applied {
                commands: written,
                disclosed: last.filter(|d| !d.is_empty()),
            })
        }
        FormEdit::Reset => {
            let out = session.reset_form(None)?;
            // `fields_reset` and not 1: a reset on a form that already holds
            // its defaults commits a command that writes nothing, and the
            // caller uses this to decide whether to invalidate the page.
            Ok(Applied::plain(out.fields_reset))
        }
        FormEdit::RegenerateAppearances => {
            let out = session.regenerate_appearances()?;
            // ★ NOT `out.regenerated`. Clearing `/NeedAppearances` is itself a
            // change to the document even when no field's appearance moved —
            // it is the whole point of the control on a form whose values were
            // already drawn — so a run that regenerated nothing and cleared
            // the flag must still count as having done something.
            Ok(Applied::plain(
                out.regenerated + usize::from(out.need_appearances_cleared),
            ))
        }
        FormEdit::Flatten => {
            let out = session.flatten_fields(None)?;
            Ok(Applied::plain(out.fields_flattened))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every variant has a distinct trace label.**
    ///
    /// The labels are how a refusal is identified in a log from a machine
    /// nobody can reach, and two verbs sharing one would make the log say
    /// which pair of things might have failed.
    #[test]
    fn every_form_edit_traces_under_its_own_name() {
        let all = [
            FormEdit::FillText {
                field: String::new(),
                value: String::new(),
            },
            FormEdit::ConvertRichTextToPlain {
                field: String::new(),
                value: String::new(),
            },
            FormEdit::SetButtonState {
                field: String::new(),
                state: String::new(),
            },
            FormEdit::SetChoice {
                field: String::new(),
                values: Vec::new(),
            },
            FormEdit::Recompute {
                changes: Vec::new(),
            },
            FormEdit::Reset,
            FormEdit::RegenerateAppearances,
            FormEdit::Flatten,
        ];
        let mut seen: Vec<&str> = Vec::new();
        for edit in &all {
            let label = edit.label();
            assert!(
                !label.is_empty() && !seen.contains(&label),
                "{label} is empty or already claimed by another verb"
            );
            seen.push(label);
        }
        assert_eq!(seen.len(), 8, "a variant was added without a label");
    }

    /// **★ A trace label never carries an operand.**
    ///
    /// The label is what reaches stderr, and `FormEdit::FillText` carries
    /// whatever the operator typed — which, on a `/Ff` `Password` field, is a
    /// value pdfcer has just warned them is stored in the clear. Widening that
    /// exposure into a log would be pdfcer doing the thing it cautioned
    /// against.
    ///
    /// Asserted by construction rather than by inspection: a `const fn`
    /// returning `&'static str` **cannot** interpolate a field, so the only
    /// way this test fails is if someone changes the signature to build a
    /// `String` — which is exactly the change that would need reviewing.
    #[test]
    fn a_trace_label_cannot_contain_a_typed_value() {
        let secret = "hunter2";
        let edit = FormEdit::FillText {
            field: "Personal.Password".to_owned(),
            value: secret.to_owned(),
        };
        assert!(
            !edit.label().contains(secret) && !edit.label().contains("Personal"),
            "the trace label leaked an operand: {}",
            edit.label()
        );
    }

    /// **An empty recompute plan is a no-op, and reports itself as one.**
    ///
    /// Pins the reason [`run`] returns a count at all. A plan with nothing in
    /// it is reachable from a real click — the section recomputes its plan
    /// every frame it is open, and a form whose calculations are already
    /// correct produces an empty one — and treating it as a change would drop
    /// the page texture and clear the canvas's resolved selection for nothing.
    ///
    /// Driven through a real `EditSession` so it is the actual code path
    /// rather than a restatement of the match arm.
    #[test]
    fn an_empty_recompute_plan_changes_nothing() {
        use crate::panels::objects::test_support::engine_fixture;

        let path = engine_fixture("pageops/four-pages.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);

        let before = session.undo_depth();
        let applied = run(
            &mut session,
            &FormEdit::Recompute {
                changes: Vec::new(),
            },
        )
        .expect("an empty plan cannot fail");

        assert_eq!(applied.commands, 0, "an empty plan must report no commands");
        assert_eq!(
            applied.disclosed, None,
            "a plan that wrote nothing cannot have inferred anything"
        );
        assert_eq!(
            session.undo_depth(),
            before,
            "an empty plan must not push an undo entry the operator did not \
             earn"
        );
    }

    /// **A form verb on a document with no form refuses rather than panicking.**
    ///
    /// The reachable case this guards: the panel draws Flatten, the operator
    /// clicks it, and between the two frames an undo removed the form. Every
    /// one of these verbs answers `EditError` for that, and the whole of
    /// [`apply`]'s error arm is built on their doing so.
    #[test]
    fn a_form_verb_on_a_formless_document_is_an_error_not_a_panic() {
        use crate::panels::objects::test_support::engine_fixture;

        let path = engine_fixture("pageops/four-pages.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);
        assert!(
            pdfcer_core::forms::parse_acroform(&session.graph()).is_none(),
            "this fixture must carry no /AcroForm, or the test is vacuous"
        );

        for edit in [
            FormEdit::Flatten,
            FormEdit::RegenerateAppearances,
            FormEdit::Reset,
            FormEdit::SetButtonState {
                field: "Nope".to_owned(),
                state: "Yes".to_owned(),
            },
            FormEdit::FillText {
                field: "Nope".to_owned(),
                value: "x".to_owned(),
            },
        ] {
            let result = run(&mut session, &edit);
            assert!(
                result.is_err(),
                "{:?} succeeded on a document with no form",
                edit.label()
            );
        }
        assert_eq!(
            session.undo_depth(),
            0,
            "a refused verb must leave the undo stack alone"
        );
    }

    /// ★ **A fill carries back exactly the two facts the document cannot be
    /// asked again — and only when there is something to say.**
    ///
    /// Driven through a real `EditSession` and a real form, because the whole
    /// claim is about what `FillOutcome` reports rather than about what this
    /// module remembers.
    ///
    /// Both halves matter:
    ///
    /// * an **ordinary** fill discloses nothing, so the panel draws no
    ///   sentence. A disclosure line under every edit would train the operator
    ///   to stop reading the ones that matter — the same argument
    ///   `crate::app::status::page_box`'s `Note` makes for having no `Ok`
    ///   variant;
    /// * a fill of text the field's font **cannot encode** discloses the
    ///   substitution. That is the one this exists for: the saved value IS the
    ///   substituted one, so re-reading the field afterwards reports what pdfcer
    ///   wrote and never that it wrote something else.
    #[test]
    fn a_fill_discloses_a_substitution_and_an_ordinary_fill_says_nothing() {
        use crate::panels::objects::test_support::engine_fixture;

        let path = engine_fixture("forms/demo-form.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let mut session = EditSession::new(doc);

        // The first fillable text field, found rather than named — the
        // fixture's fully-qualified names are not the `/TU` labels the panel
        // shows, and a test that hard-coded one would break the day the
        // fixture is regenerated.
        let target = {
            let view = session.view();
            let form = pdfcer_core::forms::parse_acroform(&view).expect("the fixture has a form");
            form.fields
                .iter()
                .find(|f| {
                    f.field_type == Some(pdfcer_core::forms::FieldType::Text)
                        && !f.flags.read_only()
                })
                .map(|f| f.fully_qualified_name.clone())
                .expect("the fixture has a text field")
        };

        // ★ Text every WinAnsi font can carry: nothing was substituted.
        let plain = run(
            &mut session,
            &FormEdit::FillText {
                field: target.clone(),
                value: "Anna".to_owned(),
            },
        )
        .expect("a plain fill succeeds");
        assert_eq!(plain.commands, 1);
        let plain = plain.disclosed.expect("a fill always reports an outcome");
        assert_eq!(
            plain.unencodable_chars, 0,
            "ASCII must not be reported as unencodable"
        );
        assert_eq!(plain.field, target);

        // ★ Text it cannot: two CJK characters with no WinAnsi code, written
        // as `?` and counted.
        let substituted = run(
            &mut session,
            &FormEdit::FillText {
                field: target.clone(),
                value: "\u{5341}\u{4e00}".to_owned(),
            },
        )
        .expect("an unencodable fill still succeeds — it substitutes")
        .disclosed
        .expect("a fill always reports an outcome");
        assert_eq!(
            substituted.unencodable_chars, 2,
            "both characters had to be replaced, and the operator has to be \
             told: the SAVED value is the substituted one, so nothing about \
             the document afterwards says this happened"
        );
        assert!(
            !substituted.is_empty(),
            "a substitution must reach the panel"
        );
        assert_eq!(substituted.field, target);

        // …and a verb that infers nothing carries nothing back.
        let regen = run(&mut session, &FormEdit::RegenerateAppearances)
            .expect("a regeneration succeeds on this form");
        assert_eq!(
            regen.disclosed, None,
            "only a FILL makes an inference on the operator's behalf"
        );
    }

    /// ★ **A disclosure is shown only while it describes the revision on
    /// screen.**
    ///
    /// The staleness rule, which is what lets an undo silence the sentence with
    /// nothing anywhere having to remember to clear it. Verified by driving as
    /// well — an unrelated check-box toggle made a live auto-size line
    /// disappear — and pinned here because the epoch comparison is the whole
    /// mechanism.
    #[test]
    fn a_disclosure_is_hidden_once_the_document_moves_past_it() {
        record_fill_disclosure(Some(FillDisclosure {
            field: "Name".to_owned(),
            epoch: 7,
            applied_autosize: Some(12.0),
            unencodable_chars: 0,
        }));
        assert!(last_fill_disclosure(7).is_some());
        assert!(
            last_fill_disclosure(8).is_none(),
            "a later revision must not show a note about an earlier one"
        );
        assert!(last_fill_disclosure(6).is_none());

        // A fill with nothing to say is never shown, at any epoch.
        record_fill_disclosure(Some(FillDisclosure {
            field: "Name".to_owned(),
            epoch: 7,
            applied_autosize: None,
            unencodable_chars: 0,
        }));
        assert!(
            last_fill_disclosure(7).is_none(),
            "an empty disclosure must draw no sentence"
        );
        record_fill_disclosure(None);
    }
}
