//! # `dialogs::sign` — the window that puts the operator's signature on a
//! document
//!
//! The surface for [`crate::sign`]; read that module's header first, because
//! every rule this window enforces is argued there and none of it is repeated
//! here. What this file adds is the **order things are asked in**, and that
//! order is the design.
//!
//! ## ★★★ 1. THE IDENTITY IS OPENED BEFORE ANYTHING ELSE IS OFFERED
//!
//! The window has two states while it is being filled in, and they are not
//! cosmetic:
//!
//! ```text
//!   ┌ certificate not yet opened ─────────────────────────────────┐
//!   │  Choose certificate…   [ path ]                             │
//!   │  Passphrase            [ •••• ]                             │
//!   │  [ Open certificate ]                                       │
//!   └─────────────────────────────────────────────────────────────┘
//!               │  the container verified, the key came out
//!               ▼
//!   ┌ identity on screen ─────────────────────────────────────────┐
//!   │  Signed by: CN=…            ← read out of the FILE          │
//!   │  Key: RSA-2048, chain of 3                                  │
//!   │  Integrity: checked / NOT checked                           │
//!   │  Reason / Location / page / destination / [ Sign and save… ]│
//!   └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! **Nothing below the identity exists until the identity does.** That is not
//! progressive disclosure for tidiness — it is the one guard this surface can
//! offer against the mistake that matters. Signing is an act of identity, and
//! the only thing standing between "I picked a file" and "I attached my name to
//! a legal document" is the operator reading whose certificate came out of the
//! file. A form that let them fill in a reason, choose a destination and press
//! *Sign* with the certificate still unopened would put the identity check
//! **after** the decision, where it is a formality.
//!
//! ★ It is also the passphrase check, and it costs nothing extra: a wrong
//! passphrase is `Pkcs12Error::MacMismatch`, arriving at the moment the
//! operator is looking at the passphrase box rather than three fields later.
//!
//! ## ★★ 2. Why the write is an `Action` and not a call
//!
//! `EditSession::sign` takes **`&mut EditSession`** and a dialog body is handed
//! `&OpenDoc`. That is not an inconvenience to route around — it is the rule
//! that stops a window mutating a document while the frame that drew it is
//! still reading one. `Arc::get_mut` is the funnel's second step and it fails
//! outright while the render worker holds its clone, so a mutation attempted
//! from inside a draw would be *silently declined*, which is the worst of the
//! available failures.
//!
//! ⚠ [`crate::dialogs::protect`] does the opposite — it calls the engine from
//! inside `commit` — and the difference is real rather than inconsistency:
//! `set_encryption` takes `&self`. Every verb that takes `&mut` reaches the
//! session through [`crate::app::actions::Action`], and this one does too.
//!
//! ## ★★★ 3. THE PRIVATE KEY DOES NOT TRAVEL IN THE ACTION QUEUE
//!
//! [`Action`] derives `Debug`, `Clone` and `PartialEq`. Every one of those is
//! wrong for a private key:
//!
//! | trait | what it would mean |
//! |---|---|
//! | `Debug` | the key can be formatted into a trace `tools/ui-verify` keeps on disk |
//! | `Clone` | copies of the key material nobody is counting |
//! | `PartialEq` | a **non-constant-time comparison** over secret bytes |
//!
//! So the loaded [`crate::sign::Identity`] stays in this struct, and the action
//! carries the certificate's **path** and the passphrase as a
//! [`crate::secret::Secret`] — a type whose whole guarantee is that its value
//! cannot be formatted, and which this enum already carries for
//! `Action::OpenWithPassword`.
//!
//! ⇒ The consequence, stated because it looks like waste: **the `.pfx` is read
//! and parsed twice.** Once here, whose job is to show the operator whose key
//! it is, and once in the handler, whose job is to sign. That is the right
//! trade — the alternative is smuggling key material through a queue that
//! derives three traits it must not have — and the second read is not
//! redundant: it is the read that actually signs, so a file that changed under
//! the operator between the two is caught rather than assumed away.
//!
//! ## ★ The section headings are NOT `.strong()`
//!
//! `crate::dialogs::protect`'s §6, taken rather than re-argued and caught by
//! `tools/gates/check-strong-text.sh` on the first draft of this file exactly
//! as it was on that one. egui has no separate role for emphasised text, so
//! `.strong()` resolves to the **accent-filled widget** colour — pale text on a
//! pale panel (`DEFECTS.md` D11). The hierarchy here is carried by layout and
//! wording instead: a rule and a gap between sections, headings that are
//! phrases (*"Your certificate"*, *"What the signature will say"*, *"On the
//! page"*) rather than one-word captions, and the muted `.small()` notes below
//! them to contrast against.
//!
//! ## 4. What comes back
//!
//! The handler reports through [`super::DialogsState::sign_outcome`], which is
//! the same two-step every dialog here uses for anything that happens outside
//! its own closure. There is no polling and no shared cell: the app owns both
//! the dialog and the handler, so the outcome is handed over rather than
//! looked for.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::secret::Secret;
use pdfcer_core::sign::apply::MdpPermission;

use crate::sign::{Authored, Identity, IdentityFailure, Placement, Refusal, Standing};
use crate::text::protect as tp;
use crate::text::sign as t;

// ---------------------------------------------------------------------------
// Named regions
//
// Matched LITERALLY by `tools/ui-verify`, so renaming one silently un-aims the
// check that measures it. `crate::dialogs::protect`'s block records why a
// dialog needs these when a ribbon control gets its rect for free.
// ---------------------------------------------------------------------------

/// The whole window.
pub(super) const REGION_DIALOG: &str = "sign-dialog"; // ui-text-exempt: trace region name, never displayed

/// A refusal, declared **only while it is on screen** — so its presence in a
/// trace is evidence about the operator's document rather than about the build.
pub(super) const REGION_REFUSAL: &str = "sign-refusal"; // ui-text-exempt: trace region name, never displayed

/// The identity read-back, declared only once a certificate has been opened.
pub(super) const REGION_IDENTITY: &str = "sign-identity"; // ui-text-exempt: trace region name, never displayed

/// The control that opens the file picker for a certificate.
pub(super) const REGION_CHOOSE_CERTIFICATE: &str = "sign-choose-certificate"; // ui-text-exempt: trace region name, never displayed

/// The passphrase field.
///
/// ★ Its RECTANGLE, which carries nothing about what is typed into it — a
/// region name is a position, and `crate::diag::ui_rect` publishes a rect and a
/// name and never a value. A driven check needs somewhere to click before it
/// types, and this is it.
pub(super) const REGION_PASSPHRASE: &str = "sign-passphrase"; // ui-text-exempt: trace region name, never displayed

/// The control that opens the chosen certificate.
pub(super) const REGION_OPEN_CERTIFICATE: &str = "sign-open-certificate"; // ui-text-exempt: trace region name, never displayed

/// The control that commits, declared only while it is live.
pub(super) const REGION_CONFIRM: &str = "sign-confirm"; // ui-text-exempt: trace region name, never displayed

/// The radio that chooses *sign into a box already on the document*
/// (`Pass 10.13`), declared **only while the document has one to offer** — so
/// its presence in a trace is evidence about the operator's document.
pub(super) const REGION_EXISTING: &str = "sign-existing-field"; // ui-text-exempt: trace region name, never displayed

/// **The scrolling body's own viewport**, declared every frame it is drawn.
///
/// ★★★ NOT [`REGION_DIALOG`], and the difference cost a driven run. The window
/// region is `ui.max_rect()` for the whole host and includes the separator and
/// the button row **below** the scroll area. A control scrolled to just above
/// that footer is inside the window rectangle and **clipped out of the scroll
/// area**, so egui reports its position and refuses the click — which reads to
/// a harness as *"the control is there and pressing it does nothing"*.
///
/// ⇒ A check that wants to press something in this form must compare against
/// THIS rectangle. It is `ui.clip_rect()` taken inside the scroll closure,
/// which is the viewport egui itself interacts within.
pub(super) const REGION_BODY: &str = "sign-body"; // ui-text-exempt: trace region name, never displayed

/// The radio that chooses *draw a signature box on the page*.
///
/// ★ Declared unconditionally, unlike its two neighbours: it is always an
/// option, so its presence carries no evidence and its only job is to give a
/// driven check somewhere to press. [`REGION_EXISTING`] and
/// [`REGION_BOX_WHERE`] are the ones whose presence is a measurement.
pub(super) const REGION_PLACE_BOX: &str = "sign-place-box"; // ui-text-exempt: trace region name, never displayed

/// The line stating where a box this shell places will go, declared **only
/// while that is the choice**.
///
/// ★★★ Its whole job is to make *retirement* measurable on a ONE-PAGE document.
/// [`REGION_PAGE`] is the obvious probe and it is not drawn on a single-page
/// document at all — a chooser with one possible value is a label pretending to
/// be a choice — so a check aimed at it could not tell *"the page control
/// retired because a pre-placed box was chosen"* from *"there was never a page
/// control"*. This region is declared for `Place::Box` and for nothing else, on
/// a document of any length, so its presence and its absence are both evidence.
pub(super) const REGION_BOX_WHERE: &str = "sign-box-where"; // ui-text-exempt: trace region name, never displayed

/// The page chooser, declared only while a box is being placed by the operator.
///
/// ★★★ Named so that its **absence** is measurable. `--visible`/`--page` are
/// refused by the engine alongside a field name, so this control retires when a
/// pre-placed box is chosen; a driven check can only prove *"retired"* rather
/// than *"greyed"* if the region has a name to be missing under.
pub(super) const REGION_PAGE: &str = "sign-page"; // ui-text-exempt: trace region name, never displayed

/// The radio that makes this a **certifying** signature (`Pass 10.12`),
/// declared only while the document permits one.
pub(super) const REGION_CERTIFY: &str = "sign-certify"; // ui-text-exempt: trace region name, never displayed

/// One row in the list of pre-placed signature fields, by index.
///
/// ★ A function rather than a constant because there is one per field and a
/// check has to aim at a particular one. The index is the position in
/// [`crate::sign::Standing::empty_fields`], which is the order the engine's own
/// form projection returns — stable for a given document, which is all a check
/// needs.
pub(super) fn field_region(index: usize) -> String {
    // ui-text-exempt: trace region name, never displayed.
    format!("sign-field-{index}")
}

/// The control that opens what was just written.
const REGION_OPEN_SIGNED: &str = "sign-open-signed"; // ui-text-exempt: trace region name, never displayed

/// Height kept clear below the scrolling body for the button row.
const FOOTER_RESERVE: f32 = 96.0;

/// The least height the scrolling body may be given.
///
/// Without a floor, a small window produces a scroll area that draws **nothing
/// at all** — `available_height()` minus a reservation goes negative, and a
/// negative `max_height` is a silently empty area rather than an error. The
/// About, OCR, print, protect and redaction dialogs all record the same trap.
const BODY_FLOOR: f32 = 160.0;

/// The width of the single-line fields.
pub(super) const FIELD_WIDTH: f32 = 320.0;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Where one signing has got to.
///
/// A state machine rather than several `Option`s, for
/// `crate::dialogs::redact::Phase`'s reason: the states are mutually exclusive
/// and an `Option` quadruple has combinations that would all compile and none
/// of which means anything.
#[derive(Debug)]
enum Phase {
    /// The form is being filled in.
    Filling,
    /// The document itself is out of scope, and the window says why instead of
    /// drawing a form.
    Refused(Refusal),
    /// The action has been raised and the handler has not answered yet.
    ///
    /// ★ A state of its own rather than a flag, because it is the one moment
    /// on this surface when pressing the confirm control again would sign
    /// twice. It normally lasts one frame; it is drawn anyway, because "one
    /// frame" is an assumption about a machine and the state is cheap.
    Signing,
    /// The bytes reached the path.
    Written {
        /// Where the operator put it.
        path: PathBuf,
        /// Whether that path was the document that is open. Carried rather
        /// than re-derived, exactly as `crate::dialogs::protect` carries it:
        /// the sentence must describe **what happened**.
        replaced: bool,
        /// The engine's own account of what it wrote.
        details: String,
    },
    /// Something refused after the form was filled in. The form is still
    /// behind it — see [`SignDialog::body`].
    Failed(String),
}

/// **The placement radio group's value.**
///
/// A three-way choice on screen, matching [`Placement`]'s three arms — but a
/// `Copy` enum of its own rather than `Placement` itself, because
/// `egui::Ui::radio_value` compares and assigns its value and `Placement`'s
/// third arm owns a `String`. The two are converted once, at
/// [`SignDialog::commit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Place {
    /// Nothing is drawn on any page. The default — [`Placement`]'s header
    /// argues why.
    Nothing,
    /// A box this shell places, at [`crate::sign::default_rect`].
    Box,
    /// A box the document's author already placed. `Pass 10.13`.
    Existing,
}

/// **Where the signed document goes.**
///
/// [`crate::dialogs::protect::Destination`]'s twin; §6 of [`crate::sign`]'s
/// header is the argument, including why replacing is more defensible here
/// than for a redaction and still not the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Destination {
    /// A new file, chosen in the save picker. The default.
    NewFile,
    /// The document that is open, replaced in place.
    ReplaceOriginal,
}

/// The Sign window.
pub struct SignDialog {
    /// The document's own path, for the suggestion and the replace branch.
    source: PathBuf,
    /// What the document said when the window opened. Read once — see
    /// [`Standing`].
    standing: Standing,
    /// The transaction's state.
    phase: Phase,
    /// The chosen certificate file, before it has been opened.
    certificate: Option<PathBuf>,
    /// The passphrase, as typed.
    ///
    /// ★ A `String` because that is what `egui::TextEdit` binds to; it becomes
    /// a [`Secret`] the instant it leaves this struct. See this module's §3 and
    /// `crate::sign`'s §5.
    passphrase: String,
    /// The opened identity. `None` until *Open certificate* succeeds, and
    /// **nothing below it is drawn while it is `None`** — see §1.
    identity: Option<Identity>,
    /// Why the last attempt to open the certificate failed, if it did.
    identity_error: Option<String>,
    /// `/Reason`, as typed.
    reason: String,
    /// `/Location`, as typed.
    location: String,
    /// Where the signature goes: nothing drawn, a box this shell places, or a
    /// box somebody else already placed.
    ///
    /// ★ The page and the chosen field are kept **beside** this rather than
    /// inside it, so that switching to *draw nothing* and back does not lose
    /// either. A radio group that forgets its neighbour's value is one people
    /// learn not to touch.
    place: Place,
    /// The 0-based page a box this shell places goes on.
    page: usize,
    /// Which pre-placed field is chosen — an index into
    /// [`crate::sign::Standing::empty_fields`].
    ///
    /// ★★ An index HERE and a name in the request, and the asymmetry is
    /// deliberate. A radio group binds to a value it can compare, and an index
    /// is that; but an index that reached the engine would silently name the
    /// wrong field if the list it indexes had changed, whereas a name that no
    /// longer exists is refused by the engine, by name. The conversion happens
    /// once, at [`SignDialog::commit`].
    field: usize,
    /// Whether this is a **certifying** (author) signature — `Pass 10.12`.
    certify: bool,
    /// The `/DocMDP` level a certification would carry.
    ///
    /// ★ Table 254's own default, `P = 2`, which the engine's bare `--certify`
    /// also takes and *prints that it defaulted*. Form fill-in and further
    /// signatures are what a drawing sent out for approval needs to allow; `P =
    /// 1` would break the next person's signature, which is rarely what an
    /// author signing first actually wants.
    mdp: MdpPermission,
    /// `/M`, captured when the window opened and shown on screen.
    ///
    /// ★★★ Captured **once**, not read per frame, and that is what makes
    /// [`crate::text::sign::signing_time`] the source of the written value
    /// rather than a report about it. A clock read at the press would write a
    /// different moment from the one on screen, and the difference would be
    /// invisible and unfalsifiable.
    signing_time: Option<String>,
    /// Where the bytes go.
    destination: Destination,
    /// The acknowledgement asked for **only** while the operator has chosen to
    /// replace the open file. Conditional for `crate::dialogs::redact`'s
    /// standing reason: a box that is always there is a box that is always
    /// ticked.
    overwrite_acknowledged: bool,
    /// Set by the *Open certificate* control, consumed after the closure.
    open_certificate_requested: bool,
    /// Set by the file-picker control, consumed after the closure.
    pick_requested: bool,
    /// Set by the confirm control, consumed after the closure.
    ///
    /// The two-step every dialog here uses, and load-bearing rather than
    /// stylistic: an `rfd` modal opened from inside an `egui::Window` closure
    /// blocks the frame it is being drawn in.
    confirm_requested: bool,
    /// Set by *Open the signed document*, consumed after the closure.
    open_signed_requested: bool,
    /// Set by the Close control; same two-step, because a widget drawn from
    /// the state cannot drop the state it is being drawn from.
    close_requested: bool,
}

impl std::fmt::Debug for SignDialog {
    /// ★★★ **Hand-written, and the whole point of it is what it omits.**
    ///
    /// Two fields here touch a private key: [`Self::passphrase`], which is a
    /// `String` only because `egui::TextEdit` binds to one, and
    /// [`Self::identity`], which holds the key itself. A derived `Debug` would
    /// print the first, and `crate::secret`'s header records exactly what that
    /// costs — a `{:?}` anywhere on the path writes it into the trace file
    /// `tools/ui-verify` keeps as evidence.
    ///
    /// ★★ The **certificate's path is omitted too**, which goes further than
    /// `crate::dialogs::protect`'s equivalent. A path is not key material; it
    /// is a durable pointer at where somebody keeps their digital ID, and a
    /// trace file is kept and shared. What is printed is whether one was
    /// chosen.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignDialog") // ui-text-exempt: a Debug type name, never displayed.
            .field("source", &self.source)
            .field("standing", &self.standing)
            .field("phase", &self.phase)
            .field("certificate_chosen", &self.certificate.is_some())
            .field("passphrase_supplied", &!self.passphrase.is_empty())
            .field("identity_open", &self.identity.is_some())
            .field("place", &self.place)
            .field("page", &self.page)
            .field("field", &self.field)
            .field("certify", &self.certify)
            .field("destination", &self.destination)
            .field("overwrite_acknowledged", &self.overwrite_acknowledged)
            .finish_non_exhaustive()
    }
}

impl SignDialog {
    /// **Read the document, then build the window around what it said.**
    ///
    /// Cheap — a census and a `metadata` call. Nothing is computed that could
    /// be wrong later: the identity does not exist yet and the bytes are not
    /// produced until the press.
    fn open(doc: &OpenDoc) -> Self {
        let standing = Standing::read(&doc.session, &doc.path, &doc.pages);
        let signing_time = crate::app::clock::pdf_date_utc();
        // ★ The clock failure is a REFUSAL, not a warning. PAdES requires `/M`
        // and the engine will not invent one, so a machine whose clock is
        // before the epoch cannot sign — and finding that out after filling in
        // the form is exactly the R9 failure this window is shaped to avoid.
        let phase = match (standing.refusal(), signing_time.is_some()) {
            (Some(refusal), _) => Phase::Refused(refusal),
            (None, false) => Phase::Failed(t::clock_unusable().to_owned()),
            (None, true) => Phase::Filling,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The DOCUMENT's own state, so a harness can tell "the window
            // opened onto a form" from "it opened onto a refusal" and, when it
            // is a refusal, which one — without which a check asserting a
            // refusal cannot tell the right refusal from any refusal.
            //
            // ★★ `empty_fields=` and `signable_fields=` are two numbers rather
            // than one, and the difference is the whole of `Pass 10.13`'s
            // list-with-a-reason rule: a field that is present and cannot be
            // signed into is listed WITH its reason rather than filtered out,
            // so a check must be able to tell *"the document has no box"* from
            // *"the document has a box this build will not offer"*. One number
            // makes those indistinguishable.
            //
            // ⚠ No field NAMES here. They are text out of the operator's own
            // document and this trace is kept as evidence.
            format!(
                "sign-opened refusal={} encrypted={} redaction_pending={} recovered={} \
                 certification={} prior={} pages={} on_disk={} empty_fields={} \
                 signable_fields={} may_certify={}",
                refusal_token(standing.refusal()),
                u8::from(standing.encrypted),
                u8::from(standing.redaction_pending),
                u8::from(standing.recovered),
                standing
                    .certification_permission
                    .map_or_else(|| "none".to_owned(), |p| p.to_string()),
                standing.prior_signatures,
                standing.pages,
                u8::from(standing.on_disk),
                standing.empty_fields.len(),
                standing
                    .empty_fields
                    .iter()
                    .filter(|f| f.selectable())
                    .count(),
                u8::from(standing.may_certify().is_ok()),
            )
        });
        Self {
            source: doc.path.clone(),
            standing,
            phase,
            certificate: None,
            passphrase: String::new(),
            identity: None,
            identity_error: None,
            reason: String::new(),
            location: String::new(),
            // ★★★ The DEFAULT is *nothing drawn*, even on a document that
            // carries a pre-placed box — deliberately, and it is the one place
            // this design could reasonably have gone the other way. Pre-selecting
            // the sender's box would be helpful and would also mean the first
            // press of *Sign* writes into a field the operator never chose,
            // possibly triggering a `/Lock` that freezes fields he still has to
            // fill. Every arm of this group is one click; consent is not.
            place: Place::Nothing,
            page: 0,
            field: 0,
            certify: false,
            mdp: MdpPermission::FormFillAndSign,
            signing_time,
            destination: Destination::NewFile,
            overwrite_acknowledged: false,
            open_certificate_requested: false,
            pick_requested: false,
            confirm_requested: false,
            open_signed_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        actions: &mut Vec<Action>,
    ) -> bool {
        // ★ Read BEFORE the body draws its fields, so a box ticked or a
        // character typed on this frame does not enable the confirm control
        // until the next one. `crate::dialogs::redact` §4's rule, and it is
        // owed here for the replace branch, which writes over the operator's
        // file with no picker in between.
        let ready = self.ready_to_confirm();
        let (frame, ()) = crate::dialogs::host::Host::new(
            "sign", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(720.0, 660.0),
            egui::vec2(480.0, 360.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;
        // Everything with a side effect, after the closure. See the field docs.
        if std::mem::take(&mut self.pick_requested) {
            self.pick_certificate();
        }
        if std::mem::take(&mut self.open_certificate_requested) {
            self.open_identity();
        }
        if std::mem::take(&mut self.confirm_requested) && ready {
            self.commit(doc, actions);
        }
        if std::mem::take(&mut self.open_signed_requested)
            && let Phase::Written { path, .. } = &self.phase
        {
            actions.push(Action::Open(path.clone()));
            self.close_requested = true;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// **Take the outcome the handler produced.**
    ///
    /// Called by [`super::DialogsState::sign_outcome`]. A method rather than a
    /// public field so the only transition out of [`Phase::Signing`] is this
    /// one.
    pub(super) fn outcome(&mut self, outcome: crate::sign::Outcome) {
        self.phase = match outcome {
            crate::sign::Outcome::Written {
                path,
                replaced,
                details,
            } => Phase::Written {
                path,
                replaced,
                details,
            },
            crate::sign::Outcome::Failed(detail) => Phase::Failed(detail),
        };
    }

    /// **Whether the confirm control may be enabled.**
    ///
    /// Pure, and the whole of the gate's rule, so every property of it is
    /// asserted headlessly — `crate::viewer`'s standing split applied to the
    /// control that attaches somebody's legal identity to a file.
    ///
    /// The conditions, and each one is a different failure:
    ///
    /// 1. **A form is being filled in at all.** A refusal, a signing in flight
    ///    and a finished write all have no confirm.
    /// 2. **An identity is OPEN** — not merely chosen, and not merely a
    ///    passphrase typed. §1: the operator has seen whose certificate this is.
    /// 3. **The replace acknowledgement**, when and only when the operator has
    ///    chosen to replace.
    ///
    /// ★ There is deliberately **no** condition on `/Reason` or `/Location`.
    /// Both are optional in the standard, both are omitted when empty, and a
    /// surface that required a reason would be inventing an obligation the
    /// format does not impose — on a control where inventing obligations is how
    /// people learn to type anything at all into the box.
    fn ready_to_confirm(&self) -> bool {
        if !matches!(self.phase, Phase::Filling | Phase::Failed(_)) {
            return false;
        }
        if self.identity.is_none() {
            return false;
        }
        !(self.destination == Destination::ReplaceOriginal && !self.overwrite_acknowledged)
    }

    /// **Turn the radio group's value into the request's placement.**
    ///
    /// Pure, so every arm is asserted headlessly — including the one that
    /// matters most: *Existing* falling back to [`Placement::Invisible`] when
    /// there is no field at that index.
    ///
    /// ★★★ The fallback is `Invisible`, never `Visible`, and the choice is a
    /// safety one rather than an arbitrary default. `Invisible` writes
    /// `/Rect [0 0 0 0]` and draws nothing; `Visible` would stamp a box with the
    /// operator's name on a page he did not ask to have marked. When a surface
    /// has to guess on an unreachable branch, it should guess toward *writes
    /// less into the operator's file*.
    ///
    /// ⚠ Unreachable from the window — `Place::Existing` is only offered when
    /// there is a selectable field and the index comes from the list that was
    /// drawn — so this is the standing preference against panicking on a branch
    /// a guard has already excluded, not a live path.
    fn placement(&self) -> Placement {
        match self.place {
            Place::Nothing => Placement::Invisible,
            Place::Box => Placement::Visible { page: self.page },
            Place::Existing => {
                self.standing
                    .empty_fields
                    .get(self.field)
                    .map_or(Placement::Invisible, |f| Placement::ExistingField {
                        name: f.name.clone(),
                    })
            }
        }
    }

    /// Whether replacing the open document is an option at all.
    ///
    /// `is_file` rather than a flag, asked of the **file system**, exactly as
    /// `crate::app::save::has_a_file` asks it: a second source of truth drifts,
    /// and the failure when it does is writing over the wrong file.
    fn can_replace_original(&self) -> bool {
        self.source.is_file()
    }

    /// **Take the destination choice, and retire the acknowledgement given
    /// about the previous one.**
    ///
    /// `crate::dialogs::redact::choose_destination`'s rule, pure for the same
    /// reason. Without it, an operator could tick the box, think better of it,
    /// select *a new file*, change their mind again, and arrive back at
    /// *replace* with the button already live — the consent standing from a
    /// decision they had explicitly withdrawn in between.
    fn choose_destination(&mut self, choice: Destination) {
        if choice != self.destination {
            self.overwrite_acknowledged = false;
            self.destination = choice;
        }
    }

    /// Open the file picker and take its answer.
    fn pick_certificate(&mut self) {
        if let crate::app::files::Picked::Path(path) = crate::app::files::pick_certificate() {
            self.certificate = Some(path);
            // ★ A new file retires the old identity AND the old error. Leaving
            // either would show the operator a read-back of the certificate
            // they just replaced, which is the one sentence on this window that
            // must never describe a different file from the one that will sign.
            self.identity = None;
            self.identity_error = None;
        }
    }

    /// **Open the chosen certificate, so the operator can see whose it is.**
    ///
    /// §1 — and note what it does on failure: it clears [`Self::identity`] as
    /// well as setting the error. A second attempt with a wrong passphrase must
    /// not leave a previously opened identity standing behind an error message
    /// that says the opposite.
    fn open_identity(&mut self) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ Emitted BEFORE the attempt, so "the operator pressed Open
            // certificate" and "the container opened" are two facts a reader
            // can tell apart. Without it, a press that produced nothing is
            // indistinguishable from a press that never happened — which cost
            // the first driven run of this feature.
            format!(
                "sign-open-certificate chosen={} passphrase={}",
                u8::from(self.certificate.is_some()),
                if self.passphrase.is_empty() {
                    // ui-text-exempt: trace tokens, never displayed.
                    "empty"
                } else {
                    "set"
                },
            )
        });
        let Some(path) = self.certificate.clone() else {
            return;
        };
        let passphrase = Secret::new(self.passphrase.clone());
        match Identity::open(&path, &passphrase) {
            Ok(identity) => {
                self.identity = Some(identity);
                self.identity_error = None;
            }
            Err(failure) => {
                self.identity = None;
                // ★★★ TRACED, and this line was MISSING on the first driven
                // run — which is worth recording because of what the silence
                // cost. `Identity::open` traces its SUCCESS and said nothing at
                // all about a failure, so the harness saw
                // `certificate-picked chosen=1` and then no `sign-identity`,
                // and had no way to tell "the picker did not answer" from "the
                // container refused" from "the button was never pressed". That
                // is this project's standing cross-cutting defect — *every
                // engine refusal reaches the operator as silence* — arriving in
                // a diagnostic channel instead of on screen.
                //
                // ⚠ `kind=` and nothing else. The engine's message is on
                // screen, where the operator can act on it; a `Pkcs12Error`'s
                // rendering can name the container's algorithms, and a trace is
                // kept as evidence. `wrong-passphrase` is called out by name
                // because it is the failure a driven check must be able to tell
                // apart from every other one.
                let kind = match &failure {
                    // ui-text-exempt: trace tokens, never displayed.
                    IdentityFailure::Unreadable(_) => "unreadable",
                    IdentityFailure::Import(
                        pdfcer_core::sign::pkcs12::Pkcs12Error::MacMismatch { .. },
                    ) => "wrong-passphrase",
                    IdentityFailure::Import(_) => "refused",
                };
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("sign-identity-declined kind={kind}")
                });
                self.identity_error = Some(match failure {
                    IdentityFailure::Unreadable(detail) => t::identity_unreadable(&detail),
                    IdentityFailure::Import(error) => t::identity_refused(&error.to_string()),
                });
            }
        }
    }

    /// **Raise the signing action.**
    ///
    /// The destination is settled here — before the action — for
    /// `crate::dialogs::protect::commit`'s reason inverted: there, the engine
    /// runs before the picker so a refusal never arrives after a picker has
    /// been filled in and dismissed. Here the engine call is in the handler and
    /// cannot run first, so the picker comes first and the two known refusals
    /// were already answered when the window opened. What is left that can fail
    /// after the picker is a wrong passphrase — and the passphrase was already
    /// proved correct by *Open certificate*.
    fn commit(&mut self, doc: &OpenDoc, actions: &mut Vec<Action>) {
        let Some(identity) = &self.identity else {
            return;
        };
        let Some(signing_time) = self.signing_time.clone() else {
            return;
        };
        let target = match self.destination {
            // No picker: the consent for this path was taken in words, at the
            // radio and the checkbox, before the click. A picker pre-filled
            // with the source would be a dialog whose safe answer is to change
            // the field.
            Destination::ReplaceOriginal => self.source.clone(),
            Destination::NewFile => {
                let suggested = crate::sign::suggested_path(&self.source);
                let crate::app::files::Picked::Path(chosen) =
                    crate::app::files::pick_save_path(&suggested, tp::save_dialog_title())
                else {
                    // Cancelled, or a build with no picker. Nothing is lost and
                    // nothing is said: a cancelled save is a complete and
                    // uninteresting outcome, and the form is still filled in.
                    return;
                };
                chosen
            }
        };
        let _ = doc;
        actions.push(Action::SignDocument {
            certificate: identity.source().to_path_buf(),
            passphrase: Secret::new(self.passphrase.clone()),
            authored: Authored {
                reason: self.reason.clone(),
                location: self.location.clone(),
                placement: self.placement(),
                signing_time,
                // ★ `certify` is `None` unless the operator both chose it AND
                // the document permits it. The second half is asked again here
                // rather than trusted from the draw: the flag survives a
                // document that changed under the window, and sending a
                // certification the engine will refuse would replace a sentence
                // the operator read when the window opened with one he meets
                // after the picker.
                certify: (self.certify && self.standing.may_certify().is_ok()).then_some(self.mdp),
            },
            target,
            replace: self.destination == Destination::ReplaceOriginal,
        });
        self.phase = Phase::Signing;
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::of(ui.ctx());
        match &self.phase {
            // ★★★ R9's *explained* branch: the refusal replaces the form
            // rather than greying it. Whether THIS document can be signed is
            // not knowable when the ribbon is built, so the control is present
            // and the window states the reason.
            Phase::Refused(refusal) => {
                ui.label(t::refusal_heading());
                ui.add_space(6.0);
                let label = ui.label(
                    egui::RichText::new(t::refusal_line(*refusal)).color(theme.palette.danger),
                );
                crate::diag::ui_rect(REGION_REFUSAL, label.rect);
            }
            Phase::Signing => {
                ui.label(t::intro());
            }
            Phase::Written {
                path,
                replaced,
                details,
            } => {
                ui.label(t::written_heading());
                ui.add_space(6.0);
                ui.label(t::written(&file_name_of(path), *replaced));
                ui.add_space(6.0);
                // The rule-4 disclosure: what pdfcer actually wrote.
                ui.label(details.clone());
                ui.add_space(10.0);
                // ★★ What the OPEN document now is — `crate::sign` §3. Not a
                // footnote: an operator who pressed Ctrl+S after this without
                // being told would append a revision onto a stale base.
                ui.label(
                    egui::RichText::new(t::open_document_unchanged())
                        .color(theme.palette.text_muted),
                );
                ui.add_space(8.0);
                let open = ui.button(t::open_the_signed_document());
                crate::diag::ui_rect(REGION_OPEN_SIGNED, open.rect);
                if open.clicked() {
                    self.open_signed_requested = true;
                }
            }
            Phase::Filling | Phase::Failed(_) => {
                // ★ The failure sentence is drawn ABOVE the form rather than
                // instead of it, and that is the difference between this and
                // the refusal above. A refusal is about the document and
                // nothing the operator does here can change it; a failure is
                // about what they supplied, and the form they need in order to
                // try again is the one they are looking at.
                if let Phase::Failed(detail) = &self.phase {
                    ui.label(egui::RichText::new(detail.clone()).color(theme.palette.danger));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                }
                egui::ScrollArea::vertical()
                    .id_salt(REGION_DIALOG)
                    .auto_shrink([false, true])
                    .max_height((ui.available_height() - FOOTER_RESERVE).max(BODY_FLOOR))
                    .show(ui, |ui| {
                        // ★ The viewport, not the content — see `REGION_BODY`.
                        crate::diag::ui_rect(REGION_BODY, ui.clip_rect());
                        ui.label(t::intro());
                        if self.standing.prior_signatures > 0 {
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(t::already_signed(
                                    self.standing.prior_signatures,
                                ))
                                .color(theme.palette.text_muted),
                            );
                        }
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);
                        self.certificate_section(ui, &theme);
                        // ★★★ §1: NOTHING below the identity is drawn while
                        // there is no identity. Not greyed — absent. R9's other
                        // half: greying is for temporarily unavailable, and
                        // these controls are not unavailable, they are
                        // premature.
                        if self.identity.is_some() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);
                            self.details_section(ui, &theme);
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);
                            // ★ BEFORE placement, deliberately. *What kind of
                            // signature this is* is the bigger of the two
                            // decisions — a certification states what anybody
                            // may change afterwards — and *where the box goes*
                            // is a decision about a picture. The order on a form
                            // is a claim about which question matters.
                            self.kind_section(ui, &theme);
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);
                            self.placement_section(ui, &theme);
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);
                            self.destination_section(ui);
                        }
                    });
                ui.add_space(8.0);
                ui.separator();
                self.confirm_row(ui);
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(tp::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// **Why the confirm control is greyed**, in the order the conditions are
    /// met.
    ///
    /// R9: greying is only ever for temporarily unavailable, and it is
    /// **always explained on hover**. `OPERATOR_REQUESTS.md` O77's sweep found
    /// seven greyed controls with no explanation; this is the shape that
    /// discharges it.
    ///
    /// ★ It stays HERE rather than moving to [`sections`] with the row that
    /// draws it, because it is a statement about the window's gate — the same
    /// gate [`Self::ready_to_confirm`] enforces two functions above — and the
    /// two must be read together or they drift into disagreeing about which
    /// condition is being explained.
    fn disabled_reason(&self) -> &'static str {
        if self.identity.is_none() {
            t::confirm_disabled_no_certificate()
        } else {
            t::confirm_disabled_overwrite()
        }
    }
}

/// The single-token name of a refusal, for a trace line.
///
/// ★ `const`, and spelled here rather than derived, because
/// **`{:?}` on a domain type is what produced two false failure reports on
/// 2026-09-05**: a check parses these tokens, and a `Debug` rendering is a
/// spelling nobody chose and any refactor may change.
#[must_use]
const fn refusal_token(refusal: Option<Refusal>) -> &'static str {
    match refusal {
        None => "none", // ui-text-exempt: trace token, never displayed
        Some(Refusal::RedactionPending) => "redaction-pending", // ui-text-exempt: trace token, never displayed
        Some(Refusal::Encrypted) => "encrypted", // ui-text-exempt: trace token, never displayed
        Some(Refusal::CertificationForbids { .. }) => "certification-forbids", // ui-text-exempt: trace token, never displayed
        Some(Refusal::RecoveredBase) => "recovered-base", // ui-text-exempt: trace token, never displayed
        Some(Refusal::NotOnDisk) => "not-on-disk", // ui-text-exempt: trace token, never displayed
    }
}

/// A path's file name, for a sentence that names a file.
pub(super) fn file_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Build the window for `doc`, or nothing when there is no document.
///
/// The already-open and no-document guards live in
/// [`super::DialogsState::open_sign`], so a chord and a ribbon click are gated
/// by one expression.
pub(super) fn open_for(status: &crate::app::state::Status) -> Option<SignDialog> {
    let crate::app::state::Status::Open(doc) = status else {
        return None;
    };
    Some(SignDialog::open(doc))
}

mod sections;

#[cfg(test)]
mod tests;
