//! # `dialogs::protect` — the window behind **Encrypt…** and **Permissions…**
//!
//! `OPERATOR_REQUESTS.md` **O119**, approved 2026-09-04: *"yes add encryption
//! and permissions"*, under the standing instruction *"Always add new features.
//! never ask. just do."*
//!
//! This is the `Ui` half of the feature. Everything that can be decided without
//! one — what the document says today, which jobs it may be offered, which
//! engine verb a choice reaches, and the atomic write — is
//! [`crate::protect`], and that split is the whole reason the rules on this
//! surface are asserted headlessly rather than by driving a window once and
//! hoping.
//!
//! ---
//!
//! # 1. ★★★ The window's shape follows one sentence of the build brief
//!
//! > *"Show the document's CURRENT state before offering to change it. A
//! > permissions dialog that opens with everything ticked, on a document that
//! > forbids printing, has told him a falsehood before he touches anything."*
//!
//! So the body is **two sections in a fixed order**, and the order is not a
//! layout preference:
//!
//! | § | heading | what it is | can the operator move it? |
//! |---|---|---|---|
//! | 1 | [`crate::text::protect::standing_heading`] — *"This document, as it is now"* | a **read-back**: the cipher, which password opened it, and every one of the eight permission bits with the document's own three-valued answer | no |
//! | 2 | [`crate::text::protect::change_heading`] — *"What to change"* | the job, the passwords, the permission ticks, the destination | yes |
//!
//! Nothing in §1 is a control and nothing in §2 has a hard-coded default —
//! every tick is seeded from [`crate::protect::Standing::initial_ticks`], which
//! reads the file. On an unprotected document that read-back is *eight ticks*,
//! and [`crate::text::protect::permissions_start_open`] says so **in words**,
//! because eight ticks that are true and eight ticks that are a convenient
//! default look identical.
//!
//! # 2. ★★★ The three disclosures O119 named, and where each one is
//!
//! The operator listed three things he already knows and would notice missing.
//! All three are on screen, none of them is behind a hover, and none of them
//! waits for a press:
//!
//! | # | the fact | drawn where | drawn when |
//! |---|---|---|---|
//! | 1 | **a permission is a request, not a lock** — [`crate::text::security::permissions_are_advisory`], the ENGINE's own sentence, in the danger role | at the head of the permission list, **above** the tick-boxes | on every job that writes permission bits |
//! | 2 | **a signed document is refused** — [`crate::text::protect::signed_refusal`] | **instead of** the entire form | whenever the document carries a signature |
//! | 3 | **re-permissioning needs the owner password** — [`crate::text::protect::owner_password_note`] | **above** the current-owner field | on every job that touches an already-protected file |
//!
//! ★ Disclosure 1 is not re-worded here and must not be. The engine supplies it
//! as `EncryptionSettings::PERMISSIONS_DISCLOSURE`, the CLI prints it, and
//! `crate::text::security` catalogued it. Two surfaces wording one limitation
//! differently is worse than either wording.
//!
//! ★★ Disclosure 2 is **R9** in its strongest form. The ribbon controls stay
//! present — whether *this* document is signed is not known when the registry is
//! built — so the window opens, states the refusal, names the count, explains
//! the mechanism (*it rewrites every byte the signature covers*) and offers
//! nothing. There is no greyed form behind it and no button that fails on press.
//!
//! # 3. Saving: `dialogs::redact`'s answer, followed rather than re-invented
//!
//! ★★★ **This is deliberate and it is stated rather than left to be noticed.**
//! Protecting a document rewrites every byte, exactly as applying a redaction
//! does, so it raises exactly the question the operator settled hours earlier on
//! that surface, in his own words:
//!
//! > *"why does it have to save to a new file right away? Why can't it just wait
//! > on saving until I choose to save over the existing file or save as a new
//! > file?"*
//!
//! A second answer to a settled question would be the defect. So the mechanism
//! here is `crate::dialogs::redact`'s, part for part:
//!
//! | | redaction | protection |
//! |---|---|---|
//! | default destination | a new file, chosen in the picker | **the same** |
//! | suggestion | never the source (`-redacted`) | **the same** (`-protected` / `-unprotected`) |
//! | replacing the original | offered, **one extra acknowledgement** naming the file, **no picker** | **the same** |
//! | the write | temp file, then rename — atomic | **the same** ([`crate::protect::Prepared::write_to`]) |
//! | confirm control | the label IS the consequence; an ellipsis promises a picker, naming the file promises none | **the same** |
//! | the open document afterwards | untouched, and the outcome sentence says so | **the same**, and it matters more — see §5 |
//!
//! ★ Replacing takes **no picker**, and that is the deliberate half. A picker
//! pre-filled with the source is a dialog whose safe answer is to change the
//! field, which is the shape of every accidental overwrite there has ever been.
//! The consent is taken before the click, in words, at a control the operator
//! had to select.
//!
//! ★★ The half of his request the engine cannot express is the same half as
//! there — *defer the write to a later Save*. All three encryption verbs
//! **return bytes**; none stages anything in a session, and `EditSession` has no
//! `replace_document`. Approximating it would mean swapping a second session
//! under the open document and silently discarding its undo log, which
//! `crate::app::save::save_as` refuses for the same reason.
//!
//! # 4. Why this dialog does not push an `Action`
//!
//! [`super`]'s rule: a dialog uses the action funnel when it edits **this**
//! document, and this one never does. Every job here produces *bytes on disk*.
//! The open session is not mutated — see [`crate::protect`] §2 for the argument,
//! which is load-bearing rather than cautious: two of the three engine verbs
//! take `&mut EditSession` and what they clear is the guard that stops the next
//! ordinary `Ctrl+S` writing plaintext objects into a file of AES ciphertext.
//!
//! # 5. ★★ After a replace the window is deliberately STALE, and it says so
//!
//! `crate::dialogs::redact`'s outcome, and the divergence matters more here
//! because it is **invisible**: a redacted page looks different, and a protected
//! file looks identical to the one it came from. So
//! [`crate::text::protect::written`]'s replace form names the file to re-open.
//! Rule 4: report separately, and do not pretend.
//!
//! # ★ 6. The section headings are NOT `.strong()`, and that is deliberate
//!
//! Every heading here was written `RichText::new(…).strong()` in the first
//! draft, and `tools/gates/check-strong-text.sh` caught all six. Its rule, and
//! `DEFECTS.md` D11 behind it: egui has no separate role for emphasised text,
//! so `.strong()` resolves to the **accent-filled widget** colour — which on an
//! ordinary panel is pale text on a pale background. Six labels have already
//! shipped that way once, and the Settings window repeated it three days after
//! the rule was written.
//!
//! So the hierarchy here is carried by **layout and wording** instead: each
//! section is separated by a rule and a gap, the two headings are full phrases
//! (*"This document, as it is now"*, *"What to change"*) rather than one-word
//! captions, and the muted `.small().weak()` notes below them are what the
//! headings contrast against. A reader who wants to re-emphasise one of these
//! should read D11 first — in every observed case the label read **better**
//! without it.
//!
//! # 7. Document-scoped, like every dialog that holds unsaved bytes
//!
//! Closing the document discards them. A protected copy of a file nobody is
//! looking at any more, derived from a permission census that can no longer be
//! checked, is not a saving.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;
use pdfcer_core::crypto::PermissionBit;

use crate::app::state::{OpenDoc, Status};
use crate::protect::{
    Job, Passwords, PrepareFailure, Refusal, Standing, Task, always_granted, prepare,
    suggested_path,
};
use crate::secret::Secret;
use crate::text::protect as t;
use crate::text::security as ts;

// ---------------------------------------------------------------------------
// Named regions
//
// Matched LITERALLY by `tools/ui-verify`, so renaming one silently un-aims the
// check that measures it — `crate::dialogs::redact`'s block records why a
// dialog needs these when a ribbon control gets its rect for free.
// ---------------------------------------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "protect-dialog"; // ui-text-exempt: trace region name, never displayed

/// The read-back of what the document says today — §1's first section.
///
/// Declared **unconditionally whenever the form is drawn**, because its absence
/// from a trace is the evidence for the build brief's own requirement: a form
/// with no standing section is a dialog that offered to change something it
/// never reported.
const REGION_STANDING: &str = "protect-standing"; // ui-text-exempt: trace region name, never displayed

/// The signed refusal, declared **only while it is on screen** — so its presence
/// in a trace is evidence about the document rather than about the build.
const REGION_SIGNED_REFUSAL: &str = "protect-signed-refusal"; // ui-text-exempt: trace region name, never displayed

/// The permissions-advisory disclosure — O119's first.
const REGION_ADVISORY: &str = "protect-advisory"; // ui-text-exempt: trace region name, never displayed

/// The owner-password note — O119's third. Declared only while the job needs it.
const REGION_OWNER_NOTE: &str = "protect-owner-note"; // ui-text-exempt: trace region name, never displayed

/// The *replace the original* destination choice, declared only while the
/// document has an original to replace.
const REGION_DESTINATION_REPLACE: &str = "protect-destination-replace"; // ui-text-exempt: trace region name, never displayed

/// The extra acknowledgement, declared only while it is being asked for.
const REGION_OVERWRITE_ACK: &str = "protect-overwrite-ack"; // ui-text-exempt: trace region name, never displayed

/// The control that commits.
const REGION_CONFIRM: &str = "protect-confirm"; // ui-text-exempt: trace region name, never displayed

/// Height kept clear below the scrolling body for the button row.
const FOOTER_RESERVE: f32 = 96.0;

/// The least height the scrolling body may be given.
///
/// Without a floor, a small window produces a scroll area that draws **nothing
/// at all** — `available_height()` minus a reservation goes negative, and a
/// negative `max_height` is a silently empty area rather than an error. The
/// About, OCR and redaction dialogs all record the same trap.
const BODY_FLOOR: f32 = 160.0;

/// Where one protect transaction has got to.
///
/// A state machine rather than several `Option`s, for
/// `crate::dialogs::redact::Phase`'s reason: the states are mutually exclusive
/// and an `Option` quadruple has combinations that would all compile and none of
/// which means anything.
#[derive(Debug)]
enum Phase {
    /// The form is being filled in. Nothing has been computed and nothing has
    /// been written.
    ///
    /// ★ Unlike the redaction dialog there IS a *ready* state here, and the
    /// asymmetry is not an oversight. That dialog runs the whole removal on
    /// open so the numbers on screen are measurements of the exact bytes that
    /// will be written; this one **cannot**, because what it would compute
    /// depends on passwords the operator has not typed yet. There is nothing to
    /// measure until the form is complete.
    Filling,
    /// The document itself is out of scope, and the window says why instead of
    /// drawing a form.
    Refused(Refusal),
    /// The bytes reached the path.
    Written {
        /// Where the operator put it.
        path: PathBuf,
        /// Whether that path was the document that is open.
        ///
        /// Carried rather than re-derived by comparing paths later, exactly as
        /// `crate::dialogs::redact::Phase::Written` carries it: the sentence
        /// must describe **what happened**, and it is the difference between
        /// two sentences that say opposite things about the window the operator
        /// is looking at.
        replaced: bool,
        /// Which job produced them, for the outcome sentence.
        job: Job,
    },
    /// The engine, the file system or the owner password refused. The form is
    /// still behind it — see [`ProtectDialog::body`].
    Failed(String),
}

/// **Where the protected document goes.**
///
/// `crate::dialogs::redact::Destination`, and the reasoning there is this
/// type's reasoning — see §3 of this module's header for the part-for-part
/// correspondence and for the one half of the operator's request the engine
/// cannot express.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destination {
    /// A new file, chosen in the save picker. The default.
    NewFile,
    /// The document that is open, replaced in place.
    ///
    /// Offered only when the source is a real file on disk — a document created
    /// in this session has no original to replace, and a control meaning
    /// "replace nothing" is worse than an absent one.
    ReplaceOriginal,
}

/// The Encrypt / Permissions window.
pub struct ProtectDialog {
    /// Which ribbon control opened it. Decides the title, which jobs are
    /// offered, and which section the window opens on.
    task: Task,
    /// The document's own path, for the suggestion and for the replace branch.
    ///
    /// Captured on construction rather than read per frame, for
    /// `crate::dialogs::redact`'s reason: nothing can change it while the
    /// dialog is open.
    source: PathBuf,
    /// **What the document said when the window opened.**
    ///
    /// Read once, at construction, and never re-read. That is deliberate: the
    /// read-back in §1 is a statement about the document the operator chose to
    /// act on, and a value re-read per frame would let the sentence under the
    /// heading change out from under the ticks that were seeded from it.
    standing: Standing,
    /// The transaction's state.
    phase: Phase,
    /// Which job the operator has selected, out of [`Standing::jobs`].
    job: Job,
    /// The owner password that authorises a change to an already-protected
    /// document, as typed.
    ///
    /// ★ A `String` because that is what `egui::TextEdit` binds to; it becomes
    /// a [`Secret`] the instant it leaves this struct, at [`Self::commit`].
    /// `crate::dialogs::password` sets the same rule and for the same reason —
    /// nothing traced about a password on this surface is ever its value.
    current_owner: String,
    /// The new user password, and its confirmation.
    user: String,
    /// See [`Self::user`].
    user_again: String,
    /// The new owner password, and its confirmation.
    owner: String,
    /// See [`Self::owner`].
    owner_again: String,
    /// The permission ticks, seeded from [`Standing::initial_ticks`] and never
    /// from a constant.
    ticks: Vec<(PermissionBit, bool)>,
    /// `/EncryptMetadata`. Seeded `true`, which is the engine's own default and
    /// the safe direction — see
    /// [`crate::text::protect::encrypt_metadata_note`] for the trade.
    encrypt_metadata: bool,
    /// Where the bytes go. [`Destination::NewFile`] until the operator says
    /// otherwise.
    destination: Destination,
    /// The acknowledgement asked for **only** while the operator has chosen to
    /// replace the open file.
    ///
    /// Conditional for `crate::dialogs::redact`'s standing reason: a box that
    /// is always there is a box that is always ticked.
    overwrite_acknowledged: bool,
    /// Set by the confirm control, consumed after the window's closure returns.
    ///
    /// The two-step every dialog here uses, and load-bearing rather than
    /// stylistic: an `rfd` modal opened from inside an `egui::Window` closure
    /// blocks the frame it is being drawn in, and the write itself is a full
    /// rewrite of the document.
    confirm_requested: bool,
    /// Set by the Close control; same two-step, because a widget drawn from the
    /// state cannot drop the state it is being drawn from.
    close_requested: bool,
}

impl std::fmt::Debug for ProtectDialog {
    /// ★★★ **Hand-written, and the whole point of it is what it omits.**
    ///
    /// Five fields of this struct hold a password the operator typed. A derived
    /// `Debug` would print all five, and `crate::secret`'s header records
    /// exactly what that costs: *"a `{:?}` on an action carrying a password
    /// writes it into the trace file `tools/ui-verify` keeps as evidence."*
    ///
    /// The passwords are `String` rather than [`Secret`] here only because
    /// `egui::TextEdit` binds to a `String`, so the type cannot do the
    /// protecting and this impl must. It prints the LENGTHS, which is what a
    /// diagnosis of *"my password is not being accepted"* actually needs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtectDialog") // ui-text-exempt: a Debug type name, never displayed.
            .field("task", &self.task)
            .field("source", &self.source)
            .field("phase", &self.phase)
            .field("job", &self.job)
            .field("current_owner_len", &self.current_owner.len())
            .field("user_len", &self.user.len())
            .field("owner_len", &self.owner.len())
            .field("ticks", &self.ticks)
            .field("encrypt_metadata", &self.encrypt_metadata)
            .field("destination", &self.destination)
            .field("overwrite_acknowledged", &self.overwrite_acknowledged)
            .finish()
    }
}

impl ProtectDialog {
    /// **Read the document, then build the window around what it said.**
    ///
    /// Cheap — a `Standing::read` and a signature census, no rewrite. Contrast
    /// `crate::dialogs::redact::RedactDialog::open`, which runs a full removal;
    /// there is nothing here that could be computed before the passwords exist.
    fn open(doc: &OpenDoc, task: Task) -> Self {
        let standing = Standing::read(&doc.session, &doc.path);
        let phase = standing
            .refusal(task)
            .map_or(Phase::Filling, Phase::Refused);
        // ★ The first offered job, not a constant. On **Encrypt…** over a
        // protected document that is `ChangePassword`; over a plain one it is
        // `SetPassword`; on **Permissions…** it is `SetPermissions`. A
        // hard-coded default would be a radio group whose selection disagrees
        // with the list it is drawn from on some documents.
        let job = standing
            .jobs(task)
            .first()
            .copied()
            .unwrap_or(Job::SetPassword);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The document's OWN state, so a harness can tell "the form opened
            // seeded from the file" from "the form opened with a default".
            format!(
                "protect-opened task={} encrypted={} revision={} signatures={} on_disk={} \
                 granted={} refused={}",
                task_token(task),
                u8::from(standing.encrypted),
                standing.revision,
                standing.signatures,
                u8::from(standing.on_disk),
                standing.preserved_grants().len(),
                u8::from(matches!(phase, Phase::Refused(_))),
            )
        });
        Self {
            task,
            source: doc.path.clone(),
            job,
            ticks: standing.initial_ticks(),
            standing,
            phase,
            current_owner: String::new(),
            user: String::new(),
            user_again: String::new(),
            owner: String::new(),
            owner_again: String::new(),
            encrypt_metadata: true,
            destination: Destination::NewFile,
            overwrite_acknowledged: false,
            confirm_requested: false,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(&mut self, ctx: &egui::Context, doc: &OpenDoc) -> bool {
        // ★ Read BEFORE the body draws its fields, so a box ticked or a
        // character typed on this frame does not enable the confirm control
        // until the next one. `crate::dialogs::redact` §4's rule, and it is
        // owed here for the replace branch, which writes over the operator's
        // file with no picker in between.
        let ready = self.ready_to_confirm();
        let title = match self.task {
            Task::Password => t::title_password(),
            Task::Permissions => t::title_permissions(),
        };
        let (frame, ()) = crate::dialogs::host::Host::new(
            "protect", // ui-text-exempt: a viewport key, never displayed.
            title,
            egui::vec2(720.0, 640.0),
            egui::vec2(460.0, 340.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui, ready);
        });
        let open = !frame.closed;
        // The write, after the closure. See `confirm_requested`.
        if std::mem::take(&mut self.confirm_requested) {
            self.commit(doc);
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// **Whether the confirm control may be enabled.**
    ///
    /// Pure, and the whole of the gate's rule, so every property of it is
    /// asserted headlessly — `crate::viewer`'s standing split applied to the
    /// control that can overwrite the operator's file.
    ///
    /// The conditions, and each one is a different failure:
    ///
    /// 1. **A form is being filled in at all.** A refusal or a finished write
    ///    has no confirm.
    /// 2. **The current owner password is present**, on every job that acts on
    ///    an already-protected document — O119's third disclosure, enforced
    ///    rather than merely printed.
    /// 3. **The new owner password is present**, on every job that sets one. A
    ///    blank owner password makes a document whose protection anybody can
    ///    remove; `EncryptionSettings` allows it and this surface does not —
    ///    see [`crate::text::protect::owner_password_required`].
    /// 4. **Both copies of both new passwords match.** A password typed wrong
    ///    twice is a document nobody can open.
    /// 5. **The two new passwords differ.** The owner password ignores `/P`
    ///    entirely, so if it also opens the document every reader authenticates
    ///    as owner and the permission list is decoration. The engine does not
    ///    enforce this *"because the standard does not"*, so the surface does.
    /// 6. **The replace acknowledgement**, when and only when the operator has
    ///    chosen to replace.
    fn ready_to_confirm(&self) -> bool {
        if !matches!(self.phase, Phase::Filling | Phase::Failed(_)) {
            return false;
        }
        let g = self.gates();
        !g.current_owner_missing
            && !g.owner_missing
            && !g.mismatch
            && !g.same
            && !g.overwrite_unacknowledged
    }

    /// The **outstanding** conditions, as flags.
    ///
    /// ★ Outstanding rather than satisfied, and computed from the same
    /// expressions that decide whether each control is drawn — so the
    /// disabled-hover sentence can never send the operator to look for a field
    /// that was never on screen. `OPERATOR_REQUESTS.md` O77's sweep found seven
    /// greyed controls with no explanation; this is the shape that discharges
    /// it, taken from `crate::text::redact::confirm_disabled`.
    fn gates(&self) -> Gates {
        Gates {
            current_owner_missing: self.job.needs_current_owner() && self.current_owner.is_empty(),
            owner_missing: self.job.sets_new_passwords() && self.owner.is_empty(),
            mismatch: self.job.sets_new_passwords()
                && (self.user != self.user_again || self.owner != self.owner_again),
            // ★ Compared only when the owner password is non-empty, so a form
            // with both boxes still blank reports "the owner password is
            // required" rather than the confusing "they must differ".
            same: self.job.sets_new_passwords()
                && !self.owner.is_empty()
                && self.user == self.owner,
            overwrite_unacknowledged: self.destination == Destination::ReplaceOriginal
                && !self.overwrite_acknowledged,
        }
    }

    /// Whether replacing the open document is an option at all.
    ///
    /// `is_file` rather than a flag, asked of the **file system**, exactly as
    /// `crate::app::save::has_a_file` asks it and for the reason recorded
    /// there: a second source of truth drifts, and the failure when it does is
    /// writing over the wrong file.
    fn can_replace_original(&self) -> bool {
        self.source.is_file()
    }

    /// **Take the destination choice, and retire the acknowledgement given
    /// about the previous one.**
    ///
    /// `crate::dialogs::redact::choose_destination`'s rule, pure for the same
    /// reason. Changing the destination un-ticks
    /// [`Self::overwrite_acknowledged`]: without it, an operator could tick the
    /// box, think better of it, select *a new file*, change their mind again,
    /// and arrive back at *replace* with the button already live — the consent
    /// standing from a decision they had explicitly withdrawn in between.
    ///
    /// It fires on **any** change rather than only on leaving the replace
    /// choice. Retiring a tick that was not needed costs nothing; deciding
    /// which changes matter is where a future edit gets it wrong.
    fn choose_destination(&mut self, choice: Destination) {
        if choice != self.destination {
            self.overwrite_acknowledged = false;
            self.destination = choice;
        }
    }

    /// **Take the job, and re-seed the permission ticks from the document.**
    ///
    /// ★ Pure-ish and a method rather than a line inside the radio group, so
    /// the rule can be asserted headlessly. Selecting *remove the protection*
    /// and then going back to *change the passwords* must not leave the ticks
    /// wherever a previous job's editing left them — the seed is always
    /// [`Standing::initial_ticks`], i.e. always the file.
    fn choose_job(&mut self, choice: Job) {
        if choice != self.job {
            self.job = choice;
            self.ticks = self.standing.initial_ticks();
        }
    }

    /// The permission bits currently ticked, as the engine wants them.
    ///
    /// ★ [`always_granted`] bits are forced in regardless of the tick, so this
    /// list is what the written file will actually say rather than what the
    /// controls happen to show. The two agree by construction because
    /// [`Standing::initial_ticks`] forces the same bits on, but forcing it here
    /// too means a future edit to the drawing code cannot make them disagree.
    fn granted(&self) -> Vec<PermissionBit> {
        self.ticks
            .iter()
            .filter(|(bit, on)| *on || always_granted(*bit))
            .map(|(bit, _)| *bit)
            .collect()
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, ready: bool) {
        let theme = Theme::of(ui.ctx());
        match &self.phase {
            // ★★★ Disclosure 2, and it replaces the form rather than greying
            // it. R9: the control is absent or explained, never a button that
            // fails on press.
            Phase::Refused(refusal) => {
                let text = match refusal {
                    Refusal::Signed { signatures } => t::signed_refusal(*signatures),
                    Refusal::NotEncrypted => t::not_encrypted_refusal().to_owned(),
                    Refusal::NoFile => t::no_file_refusal().to_owned(),
                };
                let label = ui.label(egui::RichText::new(text).color(theme.palette.danger));
                if matches!(refusal, Refusal::Signed { .. }) {
                    crate::diag::ui_rect(REGION_SIGNED_REFUSAL, label.rect);
                }
            }
            Phase::Written {
                path,
                replaced,
                job,
            } => {
                ui.label(t::written(*job, &file_name_of(path), *replaced));
            }
            Phase::Filling | Phase::Failed(_) => {
                // ★ The failure sentence is drawn ABOVE the form rather than
                // instead of it, and that is the difference between this and
                // the refusal above. A refusal is about the document and
                // nothing the operator types can change it; a failure is about
                // what they typed — a wrong owner password, a file system that
                // said no — and the form they need in order to try again is the
                // one they are looking at.
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
                        self.standing_section(ui, &theme);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);
                        self.change_section(ui, &theme);
                    });
                ui.add_space(8.0);
                ui.separator();
                self.confirm_row(ui, ready);
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// **§1 — what the document says today.** A read-back, no controls.
    ///
    /// Drawn first and always, because of the build brief's own sentence: a
    /// dialog that offers to change something it has not reported has told the
    /// operator a falsehood before he touches anything.
    fn standing_section(&self, ui: &mut egui::Ui, theme: &Theme) {
        let heading = ui.label(t::standing_heading());
        crate::diag::ui_rect(REGION_STANDING, heading.rect);
        ui.add_space(6.0);

        // --- the encryption itself ---------------------------------------
        if let Some(cipher) = self.standing.cipher {
            ui.label(ts::cipher_line(cipher));
            if let Some(auth) = self.standing.auth {
                ui.label(t::opened_with(auth));
            }
        } else {
            ui.label(ts::not_encrypted());
        }

        // --- every permission bit, with the document's OWN answer ---------
        //
        // ★ All eight, always, and `Option<bool>` rendered as three states
        // rather than two. `PermissionBit`'s own doc: *"a partial list would be
        // worse than none"*, and `Some(false)` — the author declined this — is
        // a different statement from `None` — this document's encryption has no
        // such concept. Collapsing them would show a restriction nobody wrote.
        ui.add_space(8.0);
        ui.label(t::permissions_now_heading());
        ui.add_space(4.0);
        if !self.standing.encrypted {
            ui.label(
                egui::RichText::new(t::permissions_start_open()).color(theme.palette.text_muted),
            );
            ui.add_space(4.0);
        }
        for (bit, granted) in &self.standing.grants {
            ui.label(t::permission_row(
                ts::permission_name(*bit),
                ts::permission_state(*granted),
            ));
        }
        if self.standing.has_unstated_bits() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(t::permission_becomes_stated()).color(theme.palette.text_muted),
            );
        }
    }

    /// **§2 — the controls.** The job, the passwords, the permissions, the
    /// destination.
    fn change_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(t::change_heading());
        ui.add_space(6.0);

        // --- which job ----------------------------------------------------
        //
        // Radio buttons rather than several confirm controls: the choice is one
        // state with several values, it is read back at the write, and a row of
        // buttons would put three irreversible verbs side by side where a
        // mis-aimed click lands on the wrong one. Drawn only when there is more
        // than one — a radio group of one is a label pretending to be a choice.
        let jobs = self.standing.jobs(self.task);
        if jobs.len() > 1 {
            let mut choice = self.job;
            for job in &jobs {
                ui.radio_value(&mut choice, *job, job_label(*job));
            }
            self.choose_job(choice);
            ui.add_space(4.0);
        }
        // ★ Drawn for the removal job whether or not the radio group was, so
        // the consequence is stated even on a document where removal is the
        // only thing offered.
        if self.job == Job::RemovePassword {
            ui.label(egui::RichText::new(t::job_remove_note()).color(theme.palette.danger));
        }
        ui.add_space(8.0);

        // --- passwords ----------------------------------------------------
        ui.label(t::passwords_heading());
        ui.add_space(4.0);

        // ★★★ DISCLOSURE 3 — above the field, not after a refusal. A refusal
        // that arrives on press is a program that knew the answer and waited.
        if self.job.needs_current_owner() {
            let note =
                ui.label(egui::RichText::new(t::owner_password_note()).color(theme.palette.danger));
            crate::diag::ui_rect(REGION_OWNER_NOTE, note.rect);
            ui.add_space(4.0);
            ui.label(t::current_owner_password_label());
            ui.add(
                egui::TextEdit::singleline(&mut self.current_owner)
                    .password(true)
                    .desired_width(280.0),
            );
            ui.add_space(8.0);
        }

        if self.job.sets_new_passwords() {
            // ★★★ The sentence that stops the two passwords being collapsed
            // into one. The build brief made this explicit, and what it asks
            // for is not two boxes — it is an explanation of why there are two.
            ui.label(t::passwords_explained());
            ui.add_space(4.0);
            ui.label(t::user_password_label());
            ui.add(
                egui::TextEdit::singleline(&mut self.user)
                    .password(true)
                    .desired_width(280.0),
            );
            ui.label(t::user_password_again_label());
            ui.add(
                egui::TextEdit::singleline(&mut self.user_again)
                    .password(true)
                    .desired_width(280.0),
            );
            ui.add_space(4.0);
            ui.label(t::owner_password_label());
            ui.add(
                egui::TextEdit::singleline(&mut self.owner)
                    .password(true)
                    .desired_width(280.0),
            );
            ui.label(t::owner_password_again_label());
            ui.add(
                egui::TextEdit::singleline(&mut self.owner_again)
                    .password(true)
                    .desired_width(280.0),
            );
            // ★★ Conditional, on the engine's own ask. A warning that is always
            // on screen is a warning nobody reads, and this one is irrelevant
            // to the overwhelming majority of passwords. It is a warning rather
            // than a refusal because the password may well be perfectly
            // interoperable and pdfcer cannot know.
            if self.has_non_ascii_password() {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(t::saslprep_gap()).color(theme.palette.danger));
            }
            ui.add_space(8.0);
        }

        // --- permissions ---------------------------------------------------
        if self.job.edits_permissions() {
            ui.label(t::permissions_heading());
            ui.add_space(4.0);
            // ★★★ DISCLOSURE 1 — the ENGINE's own sentence, above the
            // tick-boxes, in the danger role, never re-worded. This is the one
            // control in pdfcer whose plain reading is false: a list of boxes
            // labelled Print, Copy and Change looks exactly like a set of
            // locks, and it is not one.
            let advisory = ui.label(
                egui::RichText::new(ts::permissions_are_advisory()).color(theme.palette.danger),
            );
            crate::diag::ui_rect(REGION_ADVISORY, advisory.rect);
            ui.add_space(6.0);
            for (bit, on) in &mut self.ticks {
                // ★★★ A bit the engine will grant regardless is a STATEMENT,
                // not a control. See `crate::protect::always_granted` — a
                // tick-box the operator can clear and which comes back ticked
                // in the written file is the falsehood this surface exists to
                // avoid, arriving after he touched it rather than before.
                if always_granted(*bit) {
                    ui.label(t::permission_row(
                        ts::permission_name(*bit),
                        t::accessibility_always_granted(),
                    ));
                } else {
                    ui.checkbox(on, ts::permission_name(*bit));
                }
            }
            ui.add_space(6.0);
            ui.checkbox(&mut self.encrypt_metadata, t::encrypt_metadata_label());
            ui.label(
                egui::RichText::new(t::encrypt_metadata_note())
                    .color(theme.palette.text_muted)
                    .small(),
            );
            ui.add_space(8.0);
        }

        // --- destination ---------------------------------------------------
        //
        // ★★★ §3: `crate::dialogs::redact`'s mechanism, part for part. Drawn
        // only when there is an original to replace.
        if self.can_replace_original() {
            let name = file_name_of(&self.source);
            ui.label(t::destination_heading());
            ui.add_space(2.0);
            let mut choice = self.destination;
            ui.radio_value(&mut choice, Destination::NewFile, t::destination_new_file())
                .on_hover_text(t::destination_new_file_tooltip());
            let replace = ui.radio_value(
                &mut choice,
                Destination::ReplaceOriginal,
                t::destination_replace(&name),
            );
            crate::diag::ui_rect(REGION_DESTINATION_REPLACE, replace.rect);
            replace.on_hover_text(t::destination_replace_tooltip());
            self.choose_destination(choice);
            ui.add_space(6.0);
            // Asked for only while it applies, for the reason above.
            if self.destination == Destination::ReplaceOriginal {
                let box_ = ui.checkbox(
                    &mut self.overwrite_acknowledged,
                    t::overwrite_acknowledgement_checkbox(&name),
                );
                crate::diag::ui_rect(REGION_OVERWRITE_ACK, box_.rect);
            }
        }
    }

    /// The confirm control, and the sentence that explains it when it is greyed.
    fn confirm_row(&mut self, ui: &mut egui::Ui, ready: bool) {
        // ★ The label IS the consequence, and the consequence depends on the
        // destination: an ellipsis promises the picker, and naming the file
        // promises there will be no further question before it is replaced.
        // Promising one with a punctuation mark and not asking it would be a
        // lie the operator acts on.
        let verb = t::job_verb(self.job);
        let label = match self.destination {
            Destination::NewFile => t::confirm_button(verb),
            Destination::ReplaceOriginal => {
                t::confirm_button_replace(verb, &file_name_of(&self.source))
            }
        };
        let confirm = ui.add_enabled(ready, egui::Button::new(label));
        // Declared only while it is live, so its absence from a trace is
        // evidence the gates are closed rather than evidence a click missed.
        if ready {
            crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        }
        let clicked = confirm.clicked();
        // ★★ The `if !ready` shape and the borrow order are copied from
        // `dialogs::redact`: `on_disabled_hover_text` CONSUMES the response, so
        // `.rect` and `.clicked()` are read first.
        if !ready {
            let g = self.gates();
            confirm.on_disabled_hover_text(t::confirm_disabled(
                g.current_owner_missing,
                g.owner_missing,
                g.mismatch,
                g.same,
                g.overwrite_unacknowledged,
            ));
        }
        if clicked {
            self.confirm_requested = true;
        }
    }

    /// Whether either typed password carries a non-ASCII byte.
    ///
    /// Asked of what is in the boxes rather than of an `EncryptionSettings`
    /// that does not exist yet, so the warning appears **while typing** rather
    /// than after the press. The engine's own predicate is
    /// `EncryptionSettings::has_non_ascii_password`, and
    /// [`crate::protect::prepare`] calls that one for the trace line — two
    /// readings of one fact, taken at two moments, which is why this one is
    /// spelled out rather than borrowed.
    fn has_non_ascii_password(&self) -> bool {
        !self.user.is_ascii() || !self.owner.is_ascii()
    }

    /// **Run the job and send the bytes where the operator chose.**
    ///
    /// The two-path shape and the asymmetry between the paths are
    /// `crate::dialogs::redact::commit`'s, and §3 of this module's header is
    /// the whole of why they are copied rather than re-argued:
    ///
    /// | destination | how the path is obtained | what stands between the click and the write |
    /// |---|---|---|
    /// | [`Destination::NewFile`] | the save picker, suggesting `-protected` / `-unprotected` | the picker itself, plus the OS's own overwrite prompt |
    /// | [`Destination::ReplaceOriginal`] | [`Self::source`], **no picker** | a checkbox naming the file, and a confirm button whose label names it too |
    ///
    /// ★ The engine call happens **before** the picker on the new-file path,
    /// deliberately. Every failure this surface can meet — a wrong owner
    /// password, a signed document the census missed, an unreachable CSPRNG —
    /// is discovered before the operator is asked to name a file, so a refusal
    /// never arrives after a picker has been filled in and dismissed.
    fn commit(&mut self, doc: &OpenDoc) {
        // ★ The passwords become [`Secret`]s here, at the one place they leave
        // the text fields, and `Passwords` is dropped at the end of this
        // function. `crate::secret`'s rule: the value never enters a trace, a
        // queue or an `Action` unwrapped.
        let passwords = Passwords {
            current_owner: Secret::new(self.current_owner.clone()),
            user: Secret::new(self.user.clone()),
            owner: Secret::new(self.owner.clone()),
        };
        let prepared = match prepare(
            doc,
            self.job,
            &passwords,
            &self.granted(),
            self.encrypt_metadata,
        ) {
            Ok(prepared) => prepared,
            Err(failure) => {
                // ★ `Failed` rather than `Refused`: the form stays on screen.
                // Every one of these is something the operator can act on with
                // the boxes they are looking at — a mistyped owner password
                // being much the commonest — and taking the form away would
                // make them re-open the window and re-enter everything.
                self.phase = Phase::Failed(failure_line(&failure));
                return;
            }
        };
        let target = match self.destination {
            // No picker: the consent for this path was taken in words, at the
            // radio and the checkbox, before the click. A picker pre-filled
            // with the source would be a dialog whose safe answer is to change
            // the field.
            Destination::ReplaceOriginal => self.source.clone(),
            Destination::NewFile => {
                let suggested = suggested_path(&self.source, self.job);
                let crate::app::files::Picked::Path(chosen) =
                    crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
                else {
                    // Cancelled, or a build with no picker. Nothing is lost and
                    // nothing is said: a cancelled save is a complete and
                    // uninteresting outcome, and the form is still filled in.
                    return;
                };
                chosen
            }
        };
        self.phase = match prepared.write_to(&target) {
            Ok(_) => Phase::Written {
                path: target,
                replaced: self.destination == Destination::ReplaceOriginal,
                job: prepared.job(),
            },
            Err(refusal) => Phase::Failed(t::write_failed(&refusal.to_string())),
        };
    }
}

/// **The sentence for a preparation that did not produce bytes.**
///
/// Free rather than a method, so the mapping from every failure the model can
/// report to the wording the operator reads is one pure function a test can
/// drive — and so a new [`PrepareFailure`] variant is a compile error here
/// rather than a silent fall-through to a catch-all.
#[must_use]
fn failure_line(failure: &PrepareFailure) -> String {
    match failure {
        // Reachable only if the document changed between opening the window and
        // pressing — the pre-flight `refusal` drew a form, so it said none of
        // these at the time.
        PrepareFailure::Refused(Refusal::Signed { signatures }) => t::signed_refusal(*signatures),
        PrepareFailure::Refused(Refusal::NotEncrypted) => t::not_encrypted_refusal().to_owned(),
        PrepareFailure::Refused(Refusal::NoFile) => t::no_file_refusal().to_owned(),
        PrepareFailure::Reopen(detail) => t::reopen_failed(detail),
        PrepareFailure::NotOwner { opened_as } => t::not_owner(*opened_as),
        PrepareFailure::Engine(refusal) => t::engine_refusal(refusal),
    }
}

/// The outstanding conditions gating the confirm control.
///
/// A named struct rather than five positional `bool`s, because five `bool`s at
/// a call site is where a future edit swaps two of them and every test still
/// passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Gates {
    /// The document's current owner password is needed and has not been typed.
    current_owner_missing: bool,
    /// A new owner password is needed and has not been typed.
    owner_missing: bool,
    /// A password and its confirmation differ.
    mismatch: bool,
    /// The new user and owner passwords are the same.
    same: bool,
    /// The operator chose to replace and has not acknowledged it.
    overwrite_unacknowledged: bool,
}

/// The radio label for a job.
///
/// Free rather than a method on [`Job`], because [`Job`] lives in
/// `crate::protect` and that module holds no operator-facing strings — the
/// project's standing seam between the model and `text/`.
#[must_use]
fn job_label(job: Job) -> &'static str {
    match job {
        Job::SetPassword => t::job_set(),
        Job::ChangePassword => t::job_change(),
        Job::RemovePassword => t::job_remove(),
        // Never drawn: `Permissions` offers exactly one job, so the radio group
        // is not drawn at all (a group of one is a label pretending to be a
        // choice). It is a real answer rather than an `unreachable!` for the
        // project's standing preference against panicking on an "impossible"
        // branch.
        Job::SetPermissions => t::permissions_heading(),
    }
}

/// The single-token name of a task, for a trace line.
#[must_use]
const fn task_token(task: Task) -> &'static str {
    match task {
        Task::Password => "password", // ui-text-exempt: trace token, never displayed
        Task::Permissions => "permissions", // ui-text-exempt: trace token, never displayed
    }
}

/// **The file name a sentence should use for `path`.**
///
/// The name rather than the whole path, for `crate::dialogs::redact`'s reason:
/// every sentence that needs one is read in a window about 700 pt wide and a
/// Windows path is routinely longer than that. The full destination is on the
/// trace line [`crate::protect::Prepared::write_to`] emits.
#[must_use]
fn file_name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Open the dialog for the document in `status`, if there is one.
///
/// The dispatch target for `file.encrypt` and `file.permissions`. Lives here
/// rather than in [`super::DialogsState`] only because it needs
/// [`ProtectDialog::open`]'s private constructor; the guard it applies is the
/// one `open_print` documents — the ribbon control is gated on `doc.open`, a
/// chord bound to the same id is not, and both are fixed by refusing here at the
/// one place the dialog is built.
pub(super) fn open_for(status: &Status, task: Task) -> Option<ProtectDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(ProtectDialog::open(doc, task))
}

#[cfg(test)]
mod tests;
