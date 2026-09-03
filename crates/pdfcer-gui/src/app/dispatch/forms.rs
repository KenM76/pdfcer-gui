//! # `app::dispatch::forms` — what the five form-field commands do
//!
//! One arm of [`super::PdfcerApp::dispatch_command`], lifted into its own file.
//! The dispatcher is a routing table and this is a route; what it gained by
//! moving is that the **reasoning** below is no longer sitting in the middle of
//! ninety-eight unrelated arms, and R2 stopped being breached by four lines.
//!
//! ## The whole of what a form command does: it arms a tool
//!
//! Nothing is authored here, and — unusually — nothing is authored on release
//! either. The placing gesture raises `Action::BeginFormField`, which opens
//! `crate::dialogs::formfield`, and the field exists once the operator presses
//! Add. `crate::canvas::formfield`'s header argues why that indirection is the
//! feature rather than an extra step: a stray form field is invisible on a
//! printed page and swallows every keystroke aimed near it, so a mis-drag must
//! cost nothing.
//!
//! ## ★★★ Why the push button is refused here, in words
//!
//! This file exists to hold one finding, and it is worth the space.
//!
//! `edit.form_push_button` is `enabled_when("forms.push_button_runnable")`, a
//! condition nothing sets, so its ribbon item is greyed. That much is measured
//! rather than assumed — `egui_shell::ribbon::ctx::condition_holds` answers
//! `false` for an unset name.
//!
//! **But `egui` refusing a click on a disabled widget is the entire mechanism
//! of greying.** Every other route into the dispatcher — a keyboard chord, the
//! QAT, a context menu, the `PDFCER_DIAG_INVOKE` harness seam — never touches
//! the ribbon at all. Driving the release binary with that id armed the tool
//! and traced `form-tool-armed kind=PushButton`. Ninety-nine commands carry an
//! `enabled_when`; the greying on all of them was a drawing, not a rule.
//!
//! ### ★★ The obvious repair was written, and the test suite refused it
//!
//! One guard at the top of `dispatch_command`: refuse any command whose
//! `enable` predicate is false. Ninety-nine controls fixed in six lines. It
//! compiled, and two tests failed — one of which carries the argument in its
//! own header:
//!
//! > *"the dispatcher must not consult one. `undo.available` greys the control
//! > and the apply arm declines an empty stack **in words** — both of which are
//! > somebody else's job."*
//!
//! That is right, and it is the more important half of the rule. **Greying is a
//! hint; the worded decline is the answer.** A choke point that swallowed the
//! command would have made `Ctrl+Z` on an empty stack do nothing at all *and*
//! say nothing at all — strictly worse than the status line it produces today,
//! and the exact shape of the silent-control defect this project keeps finding.
//!
//! So enforcement lives where the words can live, which is the arm. What that
//! costs is one branch per command that needs it; what it buys is that an
//! operator who reaches a greyed capability by some other route is told why
//! rather than left pressing a key that does nothing.

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
    // ★★★ **THE INERT-KIND BRANCH IS GONE, and its deletion is dated.**
    //
    // It read: *"the two are welded by `only_the_push_button_is_inert` … on the
    // day pdfcer runs PDF actions, one predicate flips and both surfaces
    // follow."* That day was 2026-08-30, when `pdfcer-core` shipped
    // `EditSession::set_button_action`, and this shell consumed it on
    // 2026-09-01. `FormFieldKind::is_useful_once_placed` now answers `true` for
    // all five, so the branch could never fire and a branch that cannot fire is
    // a mechanism with no caller — which rots, silently, and is believed.
    //
    // ★★ The GUARD survives, and it moved to where it can still fail:
    // `canvas::formfield`'s `no_kind_is_authorable_but_inert`. A sixth kind that
    // pdfcer can author and cannot use fails that test, and its message names
    // both halves of the repair — a condition `app::conditions` does not set,
    // AND a worded decline here. Both, because of the finding this file's header
    // records at length: **greying is drawn, never enforced**, and every route
    // into the dispatcher except a ribbon click ignores it.
    let _ = crate::canvas::tool::arm_form(ctx, kind);
}

/// **Flatten every field in the document**, or decline in words.
///
/// # ★★★ This control was drawn, on the Edit tab, and inert — for the whole
/// life of the project
///
/// `edit.form_flatten` was registered with an icon and an `enabled_when`,
/// placed in Edit ▸ Forms, and had **no dispatch arm**. Its entry in
/// `shell::commands::reach::register`'s SCAFFOLDED list gave the reason:
///
/// > ~~The third of the unbuilt forms-authoring verbs, on `FEATURES.md`'s same
/// > row — and the one that is irreversible on the document, so it also needs
/// > the disclosure surface a destructive verb takes before it can honestly be
/// > offered.~~
///
/// **Both halves of that were wrong by 2026-08-27, and neither could fail a
/// test.**
///
/// * *"Unbuilt"* — `EditSession::flatten_fields` exists, and this shell has
///   been calling it since the Forms panel shipped
///   (`panels::forms::edit`'s `FormEdit::Flatten` arm). The row it cites is
///   stale in the same way: field creation shipped as O39 on 2026-08-26.
/// * *"Irreversible"* — it is one `EditSession` command and therefore one
///   `Ctrl+Z`, and `text::forms`' `forms_flatten_tooltip` had already argued
///   the point at length: flatten **appends** an overlay stream and leaves
///   existing content byte-verbatim, so under the default incremental save the
///   prior revision still holds the values. Its irreversibility is conditional
///   on the save mode, not structural. That argument is why the panel's own
///   button is *"delete-shaped weight: a rich, honest tooltip and one undo
///   step — NOT redaction's blocking modal"*, and it applies here unchanged.
///
/// ⇒ A capability the operator could reach only by opening a panel, behind a
/// ribbon control that did nothing, with a written reason that had quietly
/// stopped being true. `edit.form_flatten`'s own manifest comment already said
/// what the fix was for: *"a command buried in a panel is reachable only by
/// someone who already opened the panel."*
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
/// The panel's own comment carries the full history of that distinction,
/// including the half-wrong boundary report that produced it; this arm asks
/// the same question so the greyed panel button and the declining ribbon
/// control cannot disagree.
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
