//! # `app::dispatch::forms` — what the form-field commands do
//!
//! One arm of [`super::PdfcerApp::dispatch_command`], in its own file. The
//! dispatcher is a routing table and this is a route; the **reasoning** below
//! is the reason it is not an arm, because it is about form fields rather than
//! about routing and would be unfindable among the dispatcher's other arms.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/forms.md`.

use super::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::Status;
use crate::canvas::formfield::FormFieldKind;
use crate::panels::forms::edit::FormEdit;

/// Arm the placement tool for `kind`, or decline in words.
///
/// `id` is carried only for the trace — it is recoverable from `kind`, but a
/// trace line that printed a reconstructed id would be a second opinion about
/// what the operator invoked, and the whole value of a trace is that it is a
/// record rather than an inference.
pub(super) fn arm(app: &PdfcerApp, ctx: &egui::Context, id: &str, kind: FormFieldKind) {
    if !app.capabilities().edit_content {
        // ★ `edit_content`, not `author_markup`: a form field is a change to
        // the document's own content rather than an annotation over it, so
        // Review mode places no controls. Pairing it with markup would let a
        // reviewer author interactive controls, which is not a review activity.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("command-declined id={id} reason=mode-cannot-edit-content")
        });
        return;
    }
    // ★★ **No branch here refuses an inert kind, and that is load-bearing
    // rather than an oversight.** `FormFieldKind::is_useful_once_placed`
    // answers `true` for every kind the enum currently carries, so such a
    // branch could not fire — and a branch that cannot fire is a mechanism with
    // no caller, which rots silently and is believed anyway.
    //
    // ★★ The guard lives where it can still fail:
    // `canvas::formfield`'s `no_kind_is_authorable_but_inert`. A new kind pdfcer
    // can author and cannot use fails that test, and its message names both
    // halves of the repair — a condition `app::conditions` does not set, AND a
    // worded decline here. Both, because of the finding this file's header
    // records at length: **greying is drawn, never enforced**, and every route
    // into the dispatcher except a ribbon click ignores it.
    let _ = crate::canvas::tool::arm_form(ctx, kind);
}

/// **Flatten every field in the document**, or decline in words.
///
/// # ★★ Flattening is NOT a destructive verb, and takes no blocking modal
///
/// It is one `EditSession` command and therefore one `Ctrl+Z`, and
/// `EditSession::flatten_fields` **appends** an overlay stream while leaving
/// existing content byte-verbatim — so under the default incremental save the
/// prior revision still holds the field values. Its irreversibility is
/// conditional on the save mode, not structural. That is why the panel's own
/// button carries delete-shaped weight — a rich, honest tooltip and one undo
/// step — rather than redaction's blocking modal, and the same reasoning
/// applies to this ribbon route unchanged. `text::forms`'
/// `forms_flatten_tooltip` is where the wording lives.
///
/// ★ It is on the ribbon *as well as* in the Forms panel because a command
/// buried in a panel is reachable only by someone who already opened the panel.
///
/// # ★★ Why this raises the SAME action as the panel button, and takes no
/// extra gate
///
/// It pushes `FieldAction::Edit(FormEdit::Flatten)` — the identical intent,
/// through the identical apply path — so the two routes cannot become two
/// implementations. `Action::Command` makes that argument for command-to-command
/// routes and it is the same argument here.
///
/// ★ And it deliberately adds **no mode gate**, though `arm` above has one.
/// The Edit tab is shown only in Edit, so the ribbon route is already
/// mode-scoped; a chord could reach further, and there is no chord. What
/// decided it is that the **panel** offers Flatten with no mode gate, in every
/// mode its dock is mounted in, and a second route with a stricter rule is the
/// disagreement this project refuses — two controls for one capability
/// answering differently, with the operator left to work out which one is
/// lying. If flattening should be Edit-only, that is one change in
/// `panels::forms` and this arm follows it; it is **recorded as an open
/// question rather than decided here**, because it is a scope call.
///
/// # The refusal is the strict gate, and it is asked in the same words
///
/// `flatten_refusal`, not `fill_refusal`. Flattening removes the form, which
/// is a structural change, and on the ordinary real-world shape — a certified
/// fillable form at `/P 2` — filling is permitted while flattening is refused.
/// `panels::forms` owns the statement of that distinction; this arm asks the
/// same question so the greyed panel button and the declining ribbon control
/// cannot disagree.
pub(super) fn flatten(app: &PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    let Status::Open(doc) = &app.status else {
        return;
    };
    // ★ A worded decline, never silence. `enabled_when("doc.pages")` greys this
    // control on an empty document and says nothing about certification, so an
    // operator on a certified form meets a live control that must refuse — and
    // `dispatch::forms`' own header carries the ruling: *greying is a hint; the
    // worded decline is the answer.*
    if doc.session.flatten_refusal().is_some() {
        crate::app::status::decline::record_flatten_certified();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("command-declined id={id} reason=flatten-refused-by-certification")
        });
        return;
    }
    actions.push(Action::Field(FieldAction::Edit(FormEdit::Flatten)));
}
