//! # `dialogs::compact` — the window before a full rewrite
//!
//! `file.save_compacted`, wired 2026-08-28. `OPERATOR_REQUESTS.md` **O48**,
//! answered *"yes to all three"*.
//!
//! ## ★★★ It writes the file BEFORE it opens, and that is the design
//!
//! The window's headline number is *"a compacted copy would be 1.0 MB instead of
//! 4.2 MB"*, and it is a **measurement of this document**, obtained by actually
//! serialising it — not an estimate from a heuristic.
//!
//! That is a real cost: `to_full_bytes` walks and re-emits every object, which
//! on a dense CAD sheet is not free. It is paid because the alternative is
//! worse. The operator is being asked to accept three losses — a revision
//! history, possibly every signature, and the original file's role as the
//! canonical one — **in exchange for a saving**. A predicted saving that turned
//! out wrong would mean they accepted the losses for nothing, and there is no
//! way to give it back.
//!
//! ⇒ **When a window asks somebody to trade something irreversible for a
//! benefit, the benefit must be measured rather than predicted.**
//!
//! ★★ The bytes are then **kept** and written to whatever the picker names, so
//! the file the operator receives is byte-for-byte the one the window measured.
//! Re-serialising after the picker would be a second computation of the same
//! answer, and the two could differ — the session is not editable behind this
//! window, but that is a property of today's shell rather than of the code.
//!
//! ## ★★ Why the picker opens AFTER this window and not instead of it
//!
//! `app::save`'s `save_copy` opens a picker straight away, correctly: a copy
//! costs nothing and the only question is where. This costs three things, and a
//! native file dialog is the wrong surface to state them on — it has nowhere to
//! put a sentence, and an operator halfway through choosing a folder has already
//! decided.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. The document is not changed at all — this is a
//! **save**, and the open session is untouched by it, which is why it needs none
//! of `app::actions`' four-step protocol and why `app::save`'s header applies
//! unchanged.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::compact as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "compact.body";
/// The button that goes on to the picker.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SAVE: &str = "compact.commit";

/// The window, and the bytes it measured.
pub struct CompactDialog {
    /// The compacted file, already serialised. See the header.
    bytes: Vec<u8>,
    /// What the document occupies on disk now.
    ///
    /// ★ Read from the **file**, not from the session's own byte count, because
    /// the number the operator compares against is the one Explorer shows them.
    ///
    /// ★★ `0` for a document that has never been saved — a blank one — and that
    /// falls out correctly rather than by accident: `size_change` compares
    /// `after >= before`, so a zero `before` takes the *"no smaller"* branch and
    /// says the honest thing. A file that does not exist yet cannot be shrunk,
    /// and the alternative — an `Option` and a fourth sentence — would be a
    /// branch for a case the arithmetic already answers.
    /// ★ Read from the **file** each time the window opens, never cached: the
    /// operator may have saved since, and a stale `before` would quote a saving
    /// against a file that is no longer there.
    before: u64,
    /// How many digital signatures the copy will not keep.
    signatures: usize,
    save_requested: bool,
    close_requested: bool,
}

impl CompactDialog {
    /// Serialise the compacted copy and open on the result.
    ///
    /// `None` when the engine refuses the rewrite — see [`open_for`], which
    /// turns that into a sentence rather than into silence.
    pub fn open(doc: &OpenDoc) -> Result<Self, String> {
        use crate::app::settings::SettingsExt;
        // ★ The SAME `SaveOptions` the ordinary save uses. Two settings ride on
        // it — the cross-reference entry line ending and the trailing newline —
        // and both change the bytes of the file the operator receives. A
        // `::default()` here would produce a compacted copy that differed from
        // an ordinary one in ways nobody chose, on a command whose whole subject
        // is the bytes.
        let (bytes, _report) = doc
            .session
            .to_full_bytes(&doc.settings.save_options())
            .map_err(|error| t::refused(&error.to_string()))?;
        let before = std::fs::metadata(&doc.path).map(|m| m.len()).unwrap_or(0);
        Ok(Self {
            bytes,
            before,
            signatures: doc.session.signature_census().signatures,
            save_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "compact-copy", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 340.0),
            egui::vec2(380.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.save_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "compact-requested before={} after={} signatures={}",
                    self.before,
                    self.bytes.len(),
                    self.signatures
                )
            });
            // ★★ The measured bytes travel with the action. They are the operand
            // — see the header — and rebuilding them in the apply arm would put
            // a second serialisation between what the window promised and what
            // the operator receives.
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Compacted {
                    bytes: std::mem::take(&mut self.bytes),
                    before: self.before,
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        ui.label(t::size_change(self.before, self.bytes.len() as u64));
        ui.add_space(8.0);
        ui.small(t::revisions_line());
        // ★★★ The signature warning LAST of the three and drawn full-size, not
        // `small`, when it applies. It is the only irreversible loss in the
        // window and the only one most documents do not have — so it is
        // conditional, and where it appears it is the thing the eye lands on.
        if self.signatures > 0 {
            ui.add_space(8.0);
            ui.label(t::signature_line(self.signatures));
        }

        ui.add_space(12.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Never greyed. Every state this window can be in is one an
            // operator may legitimately proceed from — including "no smaller",
            // which is an accurate answer about a tidy file and not a reason to
            // refuse them a copy they asked for.
            let save = ui.button(t::save_button());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            if save.clicked() {
                self.save_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// Build it for the current document, or answer why not.
///
/// ★★ `Err` carries a sentence rather than a flag, because the one thing that
/// can go wrong here is the engine refusing by name — a hybrid-reference file,
/// or one whose object numbering is too sparse for §7.5.4's single-section
/// table. Both are facts about the operator's file that they can act on, and
/// collapsing them to *"could not"* would waste the only useful thing the
/// refusal carries.
pub fn open_for(status: &Status) -> Option<Result<CompactDialog, String>> {
    match status {
        Status::Open(doc) => Some(CompactDialog::open(doc)),
        _ => None,
    }
}
