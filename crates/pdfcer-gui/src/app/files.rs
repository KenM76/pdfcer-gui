//! # `app::files` — how a path gets from an operator (or a harness) to
//! [`crate::app::actions::Action::Open`]
//!
//! One question, asked in one place: **which document does the operator want
//! to open?** Everything downstream of the answer — loading, the three-way
//! failure split, forgetting the previous document's panel state, recording
//! the file in the recent list — is [`crate::app::PdfcerApp::open_path`]'s and
//! is reached through the action funnel, never from here.
//!
//! ## ★ Rule 1: substitute the dialog's ANSWER, never its interaction
//!
//! `D:\dev\rag\egui\native_file_dialog_is_a_hard_wall_substitute_the_answer_via_env_var.md`
//! records this as a **pattern in this project**, promoted after its second
//! independent instance (`diag::font_dirs`, then `PDFCER_DIAG_EXPORT_DIR`):
//!
//! > A native file/folder picker hands control to the OS shell, outside
//! > egui's own event loop. Neither `eframe::App::raw_input_hook` synthetic
//! > events nor OS-level `SendInput`/`PostMessage` automation can drive the
//! > native dialog's own widget tree — it is a separate top-level window
//! > owned by the shell, not an `egui::Window`. […] Don't try to script the
//! > dialog. Check an environment variable BEFORE opening it; if set, use its
//! > value as the dialog's result and skip opening the dialog at all.
//!
//! So [`pick_document`] checks [`DIAG_OPEN_PATH`] first, and the seam
//! replaces exactly **one** call: everything the harness is actually testing
//! — the action, the load, the failure classification, the recent list, the
//! panels forgetting the previous document — runs through the identical code
//! path a real click produces. The RAG's own instruction for anything new is
//! followed here rather than rediscovered: *"any future `rfd` call added to a
//! scripted-driven GUI should get the same `PDFCER_DIAG_<PURPOSE>` seam from
//! the start, not after the harness fails to reach it."*
//!
//! | `PDFCER_DIAG_OPEN_PATH` | [`pick_document`] returns | For |
//! |---|---|---|
//! | unset | whatever the native picker says | the operator |
//! | a path | [`Picked::Path`] — no dialog opens | a harness opening a second document |
//! | set but **empty** | [`Picked::Cancelled`] — no dialog opens | a harness exercising the *cancel* path, which is the one that must change nothing |
//!
//! ## The picker is `rfd`, and the version is not a choice
//!
//! [`rfd`](https://docs.rs/rfd) 0.17.2 — the **exact** version in
//! `D:\Dev\pdfce\crates\pdfce-gui\Cargo.toml`. Pinning to it is what makes
//! this an *adoption* of a dependency already in pdfcer's lockfile and already
//! licence-vetted (MIT, no C dependency), rather than a new one this workspace
//! introduced. The workspace root states the rule: a crate this workspace adds
//! which is not already in pdfcer's lockfile is an operator decision.
//!
//! It opens the OS's real dialog, so the operator gets the picker they already
//! know — their own places, recent folders and typing habits — and the dialog
//! is owned by the pdfcer window, so it travels with it and centres on it.
//!
//! ### What was here first, and why it is worth recording
//!
//! This module was built while the manifest was not its to edit, and the
//! obvious response to that — draw the Open button and leave it inert until
//! the dependency arrives — is defect **D1**'s exact shape, on the command
//! where it matters most: *a reader that cannot open a second file is not a
//! reader*. So the first implementation shelled out to `powershell.exe` for
//! WinForms' `OpenFileDialog`, returning the chosen path as **ASCII hex**
//! because a redirected PowerShell stdout carries the console code page and
//! `Übersicht.pdf` would otherwise arrive mangled — naming a different file,
//! or none, while looking exactly like a real answer.
//!
//! It worked, and it was verified against a live dialog. It was also
//! Windows-only, unparented, and cost a process launch before anything
//! appeared. It is gone now, and the reason to keep the paragraph is the
//! judgement rather than the code: **an honest interim that works beats a
//! placeholder that does not**, and the interim was built so that replacing it
//! touched exactly one function — the seam, the action, the command and the
//! dirty-document rule were all deliberately on this side of the call.
//!
//! ## ★ Rule 3: no test may dispatch `file.open`
//!
//! On the machine this is built on, dispatching `file.open` opens a **real
//! modal dialog** and blocks until a human dismisses it. A `cargo test` that
//! did that would hang the suite with an invisible window behind the
//! terminal. So the tests here cover [`from_env`], which is pure, and
//! [`raise`] — the translation from a [`Picked`] to an action — with all
//! three variants supplied directly. The only untested millimetre is the
//! `env::var_os` read itself, and it cannot be tested: `std::env::set_var` is
//! `unsafe` in edition 2024 and this crate is `#![forbid(unsafe_code)]`.
//!
//! ## ★ The dirty-document rule, stated where it will be needed
//!
//! [`crate::app::actions`]' header has always said an Open must not proceed
//! while a save is pending. **`file.save_copy` was wired on 2026-08-14 and this
//! paragraph still holds**, which is worth stating rather than assuming,
//! because the obvious reading of "there is a save now" is the wrong one:
//!
//! * `save_pending` asks *"is a save **in flight**?"* — is there a moment at
//!   which the bytes on disk are a partial revision and the `EditSession` being
//!   read from must not be replaced. `crate::app::save::save_copy` is
//!   **synchronous**: it is entered and finished inside one
//!   [`crate::app::PdfcerApp::apply`] call, and no frame is ever drawn while it
//!   is part-way through. So there is still no state in which the predicate can
//!   be true, and `PROJECT_PLAN.md`'s no-placeholders invariant still forbids
//!   building a confirmation dialog for a condition that cannot occur.
//! * It is emphatically **not** *"are there unsaved edits?"*. Save-a-copy does
//!   not clear that, because a copy went elsewhere and the open document is
//!   still unsaved **at its own path** — see `crate::app::save`, which carries
//!   the whole argument and the reason nothing on `OpenDoc` moves.
//!
//! What exists is one predicate, [`crate::app::PdfcerApp::save_pending`],
//! consulted by [`crate::app::actions::Action::Open`],
//! `crate::app::actions::Action::New` and
//! [`crate::app::actions::Action::Close`], returning `false` with the whole
//! rule written above it. The day an **asynchronous** save lands — the one
//! `file.save` in `crate::shell::manifest::PLANNED` is blocked on, behind
//! autosave and crash recovery — that function reads its state and the three
//! arms grow a confirmation, in one place, already wired.

use std::ffi::OsString;
use std::path::PathBuf;

/// The environment variable that answers the dialog instead of opening it.
///
/// `PDFCER_DIAG_*` is this project's established prefix for a
/// diagnostics-only seam — `PDFCER_DIAG`, `PDFCER_DIAG_VIEWPORT`,
/// `PDFCER_DIAG_EXPORT_DIR` — and the naming is part of the pattern rather
/// than decoration: a reader who finds one of them knows what kind of thing
/// the others are.
/// The environment variable `pages.insert_from_file`'s picker reads.
///
/// Deliberately NOT `DIAG_OPEN_PATH` — see [`pick_insert_source`] for why
/// sharing one seam between two verbs would make a check that drives both
/// impossible to write.
const DIAG_INSERT_PATH: &str = "PDFCER_DIAG_INSERT_PATH"; // ui-text-exempt: an environment variable name, never displayed
/// The seam that answers the **image** picker.
///
/// A third variable rather than a shared one, on the argument
/// [`pick_insert_source`] spells out for the second: one seam answering two
/// pickers makes a run that opens a PDF and inserts a picture unwritable, and
/// a run meant to test one quietly test both. Three verbs, three seams.
const DIAG_IMAGE_PATH: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name, never displayed

pub const DIAG_OPEN_PATH: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **attach a file** picker.
///
/// A fourth source variable rather than a shared one, on the argument
/// [`pick_insert_source`] spells out for the second and this module's header
/// states as a standing instruction: one seam answering two pickers makes a
/// run that opens a document and then attaches a spreadsheet to it unwritable,
/// which is precisely the run a driven check of this feature has to be.
pub const DIAG_ATTACH_PATH: &str = "PDFCER_DIAG_ATTACH_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **save an attachment out** dialog.
///
/// ★ Separate from [`DIAG_SAVE_PATH`], and this is the sharpest instance of the
/// rule rather than a routine application of it. [`pick_save_path`]'s own doc
/// records that its seam is *shared* by its two callers, so a check driving
/// both in one session gets one file. Attaching and saving out are the two
/// halves of the round trip a check of this feature exists to prove — attach a
/// known file, save it back, compare the bytes — and that check is
/// unwritable if the save seam is the one the document-save also reads.
pub const DIAG_ATTACHMENT_SAVE_PATH: &str = "PDFCER_DIAG_ATTACHMENT_SAVE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_form_data_source`].
///
/// Its own variable rather than sharing [`DIAG_OPEN_PATH`], for the reason
/// `DIAG_INSERT_PATH` gives: a driven check that imports form data into an
/// already-open document must be able to name the data file **without** also
/// answering the document picker, and one variable answering both would make
/// the two indistinguishable.
pub const DIAG_FORM_DATA_PATH: &str = "PDFCER_DIAG_FORM_DATA_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_font_folder`].
///
/// ★ Its own variable, for `DIAG_FORM_DATA_PATH`'s reason: a driven check that
/// adds a font folder must be able to name it without also answering the
/// document picker.
pub const DIAG_FONT_FOLDER_PATH: &str = "PDFCER_DIAG_FONT_FOLDER"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_acrobat`] — `OPERATOR_REQUESTS.md` O122.
///
/// ★ Its own variable, for [`DIAG_FONT_FOLDER_PATH`]'s reason and with an
/// extra one of its own: a driven check that sets the Acrobat path must be
/// able to name a **program** without also answering the document picker, and
/// the file it names is deliberately not a PDF — sharing a variable with the
/// open picker would make a check that set one accidentally answer the other.
pub const DIAG_ACROBAT_PATH: &str = "PDFCER_DIAG_ACROBAT_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_trust_store`] — the signature-trust work,
/// 2026-09-05.
///
/// ★ Its own variable, and NOT shared with [`DIAG_ACROBAT_PATH`], for the
/// reason that one gives about the document picker: the two controls sit in
/// different groups of the same window, and a driven check that set one
/// variable to answer both would silently make the Acrobat browse button
/// return an `addressbook.acrodata`. The check would still pass and the thing
/// it proved would be false.
pub const DIAG_TRUST_STORE_PATH: &str = "PDFCER_DIAG_TRUST_STORE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_certificate`] — the `.pfx`/`.p12` a driven
/// check signs with.
///
/// ⚠ **A PATH, and never a passphrase.** There is deliberately no
/// `PDFCER_DIAG_CERTIFICATE_PASSPHRASE` beside it: `tools/ui-verify` captures
/// the child's environment into the same evidence directory it captures the
/// trace into, and `crate::sign`'s §5 forbids a private key's passphrase
/// reaching any file that outlives the session. A driven check types the
/// passphrase into the field like an operator does, which is also the only way
/// to prove the field works.
#[cfg(feature = "signing")]
pub const DIAG_CERTIFICATE_PATH: &str = "PDFCER_DIAG_CERTIFICATE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The environment variable that answers the **save** dialog instead of
/// opening it.
///
/// ★ Added with [`pick_save_path`], and from the start rather than after a
/// harness failed to reach it — which is this module's own recorded
/// instruction: *"any future `rfd` call added to a scripted-driven GUI should
/// get the same `PDFCER_DIAG_<PURPOSE>` seam from the start."* A native save
/// dialog is the same hard wall as a native open dialog: it is a top-level
/// window owned by the OS shell, outside egui's event loop, and no synthetic
/// input reaches it.
///
/// It matters more here than it did for Open. `tools/ui-verify` can drive a
/// recognition and read the trace, but without this seam the one thing it
/// could never observe is **whether the recognised bytes are actually a
/// document** — the write is behind a modal no harness can dismiss. With it,
/// the check names a path, the file appears, and the assertion can be about
/// the file rather than about a button having been pressed.
///
/// | `PDFCER_DIAG_SAVE_PATH` | [`pick_save_path`] returns | For |
/// |---|---|---|
/// | unset | whatever the native dialog says | the operator |
/// | a path | [`Picked::Path`] — no dialog opens | a harness verifying what was written |
/// | set but **empty** | [`Picked::Cancelled`] — no dialog opens | a harness exercising the path where the operator declines to save, which must leave nothing behind |
pub const DIAG_SAVE_PATH: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// ★★★ **The seam for the MULTI-file picker** — `OPERATOR_REQUESTS.md` O68.
///
/// `PDFCER_DIAG_MERGE_SOURCES`, and it is the only one of these that names
/// **several** paths: they are separated by `;`, which is the Windows path-list
/// separator and therefore the one character that cannot appear in a path on
/// this platform. (`:` would have been wrong — `C:\` — and a newline is
/// unwritable in a `set` on a command line.)
///
/// Empty answers `Cancelled`, exactly as every other seam here does, so a
/// driven check can exercise the branch where the operator dismisses the
/// dialog. A single path is a legal answer and produces a one-source merge,
/// which `pageops::merge` accepts and which is worth being able to drive: it is
/// the case where the report's page count must equal the source's exactly.
pub const DIAG_MERGE_SOURCES: &str = "PDFCER_DIAG_MERGE_SOURCES"; // ui-text-exempt: an environment variable name, never displayed

/// What asking for a document produced.
///
/// Three answers rather than `Option<PathBuf>`, because the third one is not
/// a refinement of "no path": **cancelled** is the operator saying no, and
/// **unavailable** is this build having no way to ask. They call for
/// different behaviour (silence versus a trace naming a build gap) and
/// conflating them is how "the button does nothing" becomes indistinguishable
/// from "the operator changed their mind".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    /// The operator (or the diagnostic seam) named this file.
    Path(PathBuf),
    /// The operator dismissed the dialog. Nothing happens, and nothing is
    /// traced beyond the fact — a cancelled Open is a complete, correct,
    /// uninteresting outcome.
    Cancelled,
    /// This build has no way to ask.
    ///
    /// **[`native_pick`] can no longer produce this** — `rfd::FileDialog`
    /// answers `Some` or `None` and has no third case. It survives because
    /// [`from_env`] can still answer it, and because the distinction is worth
    /// more than the variant costs: the moment a build appears that cannot
    /// open a picker (a headless target, a feature-stripped build under the
    /// capability-modularity rule), the alternative is silence that looks
    /// exactly like a cancelled dialog.
    ///
    /// Deleting it would be the sort of tidying that removes the only
    /// difference between "the button does nothing" and "the operator changed
    /// their mind" — which is the distinction this whole enum exists for.
    Unavailable,
}

/// **Ask for a document to open.**
///
/// The diagnostic seam first, the platform picker second. See the module
/// header for why that order is the whole point.
///
/// Blocks while a dialog is open, exactly as `rfd::FileDialog::pick_file`
/// does: the caller is the command dispatcher, which runs between frames, and
/// a picker that returned asynchronously would need a state machine to hold
/// the half-finished intent across frames — machinery worth building for a
/// dialog pdfcer draws itself, not for one the OS owns.
#[must_use]
pub fn pick_document() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_OPEN_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "open-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = native_pick();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "open-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// Read the diagnostic seam, if it is set. Pure, so it can be tested.
///
/// `None` means "the variable is not set, go and ask properly". `Some` is a
/// complete answer that the dialog is then never opened for — which is the
/// property the harness depends on, because a dialog that opened *as well*
/// would still block it.
#[must_use]
pub fn from_env(value: Option<OsString>) -> Option<Picked> {
    let value = value?;
    if value.is_empty() {
        // A deliberate, reachable answer rather than an oversight: it is how
        // a harness drives the branch in which the operator says no, without
        // which "Open changed nothing" cannot be distinguished from "Open was
        // never reached".
        return Some(Picked::Cancelled);
    }
    Some(Picked::Path(PathBuf::from(value)))
}

/// **Turn what the picker said into what the application does about it.**
///
/// The whole of the `file.open` dispatch arm, and it lives here rather than
/// in [`crate::app::PdfcerApp`] for one reason: it is the only part of that arm
/// a test may run. Dispatching `file.open` itself opens a **real modal
/// dialog** and blocks until a human dismisses it, so a test that did it would
/// hang `cargo test` behind an invisible window (rule 3 above). Everything
/// downstream of the answer is therefore reachable from a test with the answer
/// supplied directly, and only the `env::var_os` read itself is not — and
/// cannot be, because `std::env::set_var` is `unsafe` in edition 2024 and this
/// crate forbids unsafe code.
///
/// A free function rather than a method because it touches no application
/// state, which is itself the point: a picker's answer becomes an action and
/// nothing else. The deciding, the loading and the three-way failure
/// classification all happen after the frame, in
/// [`crate::app::PdfcerApp::apply_actions`].
///
/// The three answers get three different treatments, and the differences are
/// the whole reason [`Picked`] is not an `Option<PathBuf>`:
///
/// | answer | what happens | why |
/// |---|---|---|
/// | a path | [`crate::app::actions::Action::Open`] | the ordinary case |
/// | cancelled | nothing at all, not even a trace line | the operator changed their mind; that is a complete and correct outcome, and reporting it would put a line in the trace on every dismissed dialog |
/// | unavailable | a trace naming the gap | a **build** limitation rather than an operator choice, and the one a reader of a trace from a machine they cannot see most needs told apart from "the click never arrived" |
pub fn raise(picked: Picked, actions: &mut Vec<crate::app::actions::Action>) {
    match picked {
        Picked::Path(path) => actions.push(crate::app::actions::Action::Open(path)),
        Picked::Cancelled => {}
        Picked::Unavailable => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "open-unavailable reason=no-picker-in-this-build".to_owned()
        }),
    }
}

/// Ask the platform for its own file picker.
///
/// `rfd` opens the OS's real dialog — Windows' `IFileOpenDialog`, GTK/portal
/// on Linux, `NSOpenPanel` on macOS — so the operator gets the picker they
/// already know, with their own places, recent folders and typing habits.
///
/// # Errors have exactly two shapes, and only one of them is an error
///
/// `pick_file` returns `Option<PathBuf>`: `Some` is a chosen file, `None` is
/// a dismissed dialog. There is no third case, so [`Picked::Unavailable`]
/// cannot arise here at all — it survives as a variant because
/// [`Picked::from_env`] can still answer it, and because collapsing "the
/// operator said no" into "this build cannot ask" is the distinction the
/// type exists to keep.
///
/// # It blocks
///
/// The UI thread stops while the dialog is open. That is what a modal file
/// dialog is, and it is what the previous implementation did too; nothing
/// repaints behind it. `pick_file` is the blocking call deliberately rather
/// than `pick_file().await` — an async picker would need the frame loop to
/// keep running with an open document half-replaced, which is a larger
/// change than opening a file should be.
fn native_pick() -> Picked {
    rfd::FileDialog::new()
        .set_title(crate::text::files::open_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path)
}

/// **Ask which PDF to take pages from** — `pages.insert_from_file`.
///
/// # ★ Why this is not [`pick_document`] with a different title
///
/// Two reasons, and the second is the one that matters.
///
/// **The title.** Open replaces what is on screen; insert adds to it. A picker
/// headed *"Open a PDF"* over a document the operator is part-way through
/// editing says the wrong thing at the moment they are most likely to read it.
///
/// **★ The diagnostic seam.** [`pick_document`] reads `PDFCER_DIAG_OPEN_PATH`,
/// which is how `ui-verify` drives Open without a modal dialog blocking the
/// harness. If insert shared it, a check that set the variable to drive Open
/// would ALSO silently answer every insert picker — so a run that opened one
/// file and inserted another could not be written at all, and a run that meant
/// to test one would quietly be testing both.
///
/// `PDFCER_DIAG_INSERT_PATH` is its own seam for its own verb. Same shape, same
/// `from_env` parser, separate variable.
/// Ask for a raster image to place on the page.
///
/// The third picker in this module and the first that is not asking for a PDF.
/// The filter lists the four formats `pdfcer-core` actually places — its own
/// `SUPPORTED_FORMATS` constant — because a picker that offers every file and
/// then refuses most of them has moved the refusal from a dialog the operator
/// can dismiss to one they have to read.
///
/// The *all files* filter stays beneath it, as it does for the other two: a
/// `.jpeg` that somebody saved as `.dat` is still a JPEG, `sniff` reads the
/// bytes rather than the extension, and an operator who knows what their file
/// is should not be blocked by its name.
#[must_use]
pub fn pick_image_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_IMAGE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("image-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::images::window_title())
        .add_filter(
            crate::text::files::filter_image(),
            &["png", "jpg", "jpeg", "bmp", "tif", "tiff"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("image-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which form-data file to read.**
///
/// The mirror of `actions::export::form_data`'s save picker, and the last of
/// the form verbs to be wired (2026-08-27).
///
/// # ★ Three filters, and the format is decided by CONTENT rather than by which
/// one the operator picked
///
/// The filters are a convenience for finding the file. What decides how it is
/// parsed is the **extension of the file actually chosen**, exactly as it
/// decides the format on the way out — so the two halves of the round trip use
/// one rule, and an operator who exported `.csv` and imports `.csv` cannot land
/// in a branch they did not choose.
///
/// ★ The *all files* filter stays beneath them, for [`pick_image_source`]'s
/// stated reason: a file somebody saved under a different name is still that
/// file, and an operator who knows what theirs is should not be blocked by its
/// name.
#[must_use]
pub fn pick_form_data_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_FORM_DATA_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("form-data-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::export_form::import_dialog_title())
        .add_filter(
            crate::text::files::filter_form_data(),
            &["fdf", "xfdf", "csv"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("form-data-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which file to embed in the document** (ISO 32000-1 §7.11.4.1).
///
/// # ★★ Why this picker offers no format filter at all
///
/// Every other file picker in this module narrows what it shows, and each of
/// them is right to: an image picker that offered `.dll` would have moved a
/// refusal from a dialog the operator can dismiss to one they have to read.
///
/// **This verb refuses nothing.** `EditSession::attach_file` takes
/// `bytes: &[u8]` and writes them into an embedded file stream without
/// interpreting them — a PDF may legitimately carry a spreadsheet, a CAD
/// model, a zip, a photograph or a text file, and the whole point of the
/// feature is that the document is a container. A filter here would be this
/// shell inventing a restriction the engine does not have, and the operator
/// would have to know to select *All files* to defeat it.
///
/// So [`crate::text::files::filter_all`] alone, and it is the only entry rather
/// than the last one — a single-entry filter list is what tells the operator
/// the dialog is not hiding anything.
#[must_use]
pub fn pick_attachment_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ATTACH_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("attach-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::panels::attachments::attach_dialog_title())
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("attach-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask where to write one attachment out.**
///
/// # ★ Why this is not [`pick_save_path`] with a different title
///
/// Two differences, and both would be defects if this reused that function:
///
/// 1. **The filter.** [`native_save`] adds a hard-coded PDF filter, because its
///    two callers are both writing PDFs. An attachment is whatever the document
///    put in it, and a save dialog that offered to append `.pdf` to somebody's
///    spreadsheet would be actively wrong.
/// 2. **The seam.** See [`DIAG_ATTACHMENT_SAVE_PATH`]: the round-trip check
///    this feature needs — attach a known file, save it back, compare — cannot
///    be written if this dialog answers to the same variable the document save
///    does.
///
/// `suggested` is the **sanitised** name joined to a directory, never the raw
/// name from the document. The caller owns that, and
/// `crate::app::actions::attachments` carries the argument for why: a name in a
/// PDF is attacker-controlled and unconstrained, and handing one to a save
/// dialog is handing it to the filesystem.
///
/// Honours [`pick_save_path`]'s frame-timing requirement — the caller is
/// `PdfcerApp::apply`, which is step 3, after every panel and dialog has closed.
#[must_use]
pub fn pick_attachment_target(suggested: &std::path::Path) -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ATTACHMENT_SAVE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("attachment-save-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let mut dialog = rfd::FileDialog::new()
        .set_title(crate::text::panels::attachments::save_dialog_title())
        .add_filter(crate::text::files::filter_all(), &["*"]);
    if let Some(dir) = suggested.parent().filter(|d| !d.as_os_str().is_empty()) {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    let answer = dialog.save_file().map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("attachment-save-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which folder pdfcer may take fonts from.**
///
/// ★ A *directory* picker, not a file one. `--font-dir`'s own name says the
/// unit is a folder, and asking for a font FILE would make an operator add
/// twenty-six entries to embed a family — while the engine searches a folder
/// for whatever face it needs.
#[must_use]
pub fn pick_font_folder() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_FONT_FOLDER_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("font-folder-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::settings::font_folder_dialog_title())
        .pick_folder()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("font-folder-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which program is Acrobat** — `OPERATOR_REQUESTS.md` O122's Browse
/// button.
///
/// ★ A *file* picker, not a folder one, and not the document picker: the value
/// is a full path to an executable. It is offered beside the text field rather
/// than instead of it, because typing a path from memory is how a letter goes
/// missing and because somebody who already knows the path should not have to
/// navigate to it.
///
/// ★★ The filter offers programs first and everything second. First, because a
/// person browsing for Acrobat is looking for an `.exe` and a picker showing
/// every file in `Program Files` is a picker they have to fight. Second,
/// because pdfcer does not actually require an `.exe` — a launcher script or a
/// shim is a legitimate answer, and `crate::acrobat::resolve` honours a
/// configured path whatever its name — so a filter that could not be widened
/// would be this shell overruling the operator about their own machine.
#[must_use]
pub fn pick_acrobat() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ACROBAT_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("acrobat-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::acrobat::path_dialog_title())
        .add_filter(crate::text::acrobat::path_filter_name(), &["exe"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("acrobat-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask where Acrobat's downloaded trust list is** — the Settings ▸ Digital
/// signatures Browse button.
///
/// ★ Offered beside the text field rather than instead of it, exactly as
/// [`pick_acrobat`] is and for the same reason: the value is a full path buried
/// four directories inside `%APPDATA%`, which is a path nobody types correctly
/// from memory, and somebody who already knows it should not have to navigate.
///
/// ★★ The filter names `.acrodata` first and everything second. First because
/// that is what the file is called and a picker showing every file in a
/// `Security` directory is one the operator has to fight. Second because pdfcer
/// does **not** require the extension — `pdfcer_core::trust_store` sniffs the
/// `%PPKLITE-` header rather than the name, and an administrator who handed
/// somebody a copy called `trust.dat` has given them a perfectly readable store
/// — so a filter that could not be widened would be this shell overruling the
/// operator about their own machine.
#[must_use]
pub fn pick_trust_store() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_TRUST_STORE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("trust-store-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::trust::store_path_browse())
        .add_filter(crate::text::trust::store_path_filter(), &["acrodata"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("trust-store-picked source=native answer={answer:?}")
    });
    answer
}

/// **The certificate file to sign with** — a PKCS#12 `.pfx` or `.p12`.
///
/// [`pick_trust_store`]'s shape, and its filter argument applies here twice
/// over: `.pfx` and `.p12` are the two names one format goes by (Windows
/// exports the first, OpenSSL the second), and neither is required —
/// `Pkcs12Signer::from_der` reads DER and never looks at the name. So the
/// specific filter is offered first, because a picker showing every file in a
/// directory is one the operator has to fight, and *all files* is offered
/// second, because a certificate somebody was handed as `identity.bin` is a
/// perfectly readable one and a filter that could not be widened would be this
/// shell overruling the operator about their own machine.
///
/// ⚠ **Nothing here remembers where the operator keeps their digital ID.**
/// `set_directory` is not called, no preference is written, and the path is not
/// traced. A picker that reopened in the right folder would be a convenience
/// paid for with a durable pointer at somebody's private key, written by a file
/// nobody thinks of as sensitive.
///
/// ★ `#[cfg]` for `crate::sign`'s reason: without the capability there is no
/// window that could open this picker and no verb that could use its answer,
/// and the copy it names (`crate::text::sign`) is compiled out with it. The
/// module boundary is where a capability is present or absent;
/// `SHELL_FRAMEWORK.md` §5b's rule governs the ribbon, and is satisfied by
/// `file.sign` simply not being registered.
#[cfg(feature = "signing")]
#[must_use]
pub fn pick_certificate() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_CERTIFICATE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ `answer` is NOT interpolated, unlike every sibling in this
            // file, and the asymmetry is deliberate: this path names the file
            // holding the operator's private key, and a trace is kept as
            // evidence. Whether a path was supplied is the whole diagnostic
            // question; which path it was is not ours to publish.
            format!(
                "certificate-picked source=env chosen={}",
                u8::from(matches!(answer, Picked::Path(_)))
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::sign::certificate_picker_title())
        .add_filter(crate::text::sign::certificate_filter(), &["pfx", "p12"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. See above for why
        // the path itself is absent.
        format!(
            "certificate-picked source=native chosen={}",
            u8::from(matches!(answer, Picked::Path(_)))
        )
    });
    answer
}

#[must_use]
pub fn pick_insert_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_INSERT_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "insert-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::pages::insert_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "insert-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// **Ask for several PDFs to combine into a new one** —
/// `OPERATOR_REQUESTS.md` O68.
///
/// The only picker in this shell that answers with more than one path, and it
/// is a separate function rather than a flag on [`pick_document`] for the
/// reason that file's other seven pickers are separate: each one has a title,
/// a filter set and a diagnostic seam of its own, and a harness that could
/// answer "the Open dialog" and "the Combine dialog" with the same variable
/// could not drive a check that used both.
///
/// # ★ Why `Vec<PathBuf>` and not `Picked`
///
/// Because [`Picked`] carries exactly one path and widening it would touch
/// eight call sites to serve one. The three states are expressed instead as:
/// a non-empty vector (the operator chose), an empty vector (cancelled, or the
/// build cannot ask), and — deliberately **not** distinguished — which of those
/// two the empty case was. That collapse is safe here and is not elsewhere: a
/// merge writes a NEW file and changes nothing, so "the operator changed their
/// mind" and "there was no dialog" have the same correct consequence, which is
/// to do nothing quietly. `Picked::Unavailable` exists for verbs where the two
/// must be told apart because one of them is a silent failure.
///
/// Blocks while the dialog is open, exactly as its siblings do, and carries the
/// same frame-timing requirement: the caller runs it after its frame's layout
/// closure has returned.
#[must_use]
pub fn pick_merge_sources() -> Vec<PathBuf> {
    if let Some(raw) = std::env::var_os(DIAG_MERGE_SOURCES) {
        let text = raw.to_string_lossy().into_owned();
        let answer: Vec<PathBuf> = text
            .split(';')
            .filter(|part| !part.is_empty())
            .map(PathBuf::from)
            .collect();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "merge-picked source=env n={}",
                answer.len()
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::files::merge_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_files()
        .unwrap_or_default();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "merge-picked source=native n={}",
            answer.len()
        )
    });
    answer
}

/// **Ask where to write a new document.**
///
/// `suggested` is pre-filled into the dialog as the file name and the starting
/// directory. It is a *suggestion*: the operator may type anything, including
/// — if they insist — the file they opened. What the caller guarantees is that
/// the suggestion itself is never that file, which is
/// `crate::dialogs::ocr::suggested_path`'s job and is asserted there.
///
/// ★ **This is the only write-destination question pdfcer-gui asks**, and it is
/// now asked by **two** surfaces: `dialogs::ocr`, which wrote the first file
/// this shell ever produced, and — since 2026-08-14 — `crate::app::save`, the
/// body of `file.save_copy`. The operator's standing rule — *Read may produce a
/// new document; it may not modify this one* — is enforced here, by asking,
/// rather than by a mode check: a path the operator names cannot silently be
/// the one they opened.
///
/// One function for both, deliberately, and `title` is what the second caller
/// cost. The alternative was a second `native_save` beside this one, which
/// would have been a second place for the seam, the trace line and the
/// directory/file-name split to be written — and the seam is exactly the thing
/// that must not exist twice, because a harness that can answer one dialog and
/// not the other cannot tell a save that was declined from a save that never
/// asked.
///
/// The diagnostic seam is read first, for the reason in the module header, and
/// [`DIAG_SAVE_PATH`]'s own documentation says what each of its three states
/// buys a harness. Note that the seam is **shared** by both callers: a harness
/// that sets `PDFCER_DIAG_SAVE_PATH` answers whichever of the two runs next, so
/// a check that drives both in one session names one path and gets one file.
///
/// Blocks while the dialog is open, exactly as [`pick_document`] does.
///
/// # ★ The frame-timing requirement, and it is a requirement
///
/// **The caller runs it after its frame's layout closure has returned.** Not a
/// convention — see `dialogs::ocr`'s `save_requested` field, which exists for
/// nothing else: an `rfd` modal opened from inside an `egui::Window` closure
/// blocks the frame it is being drawn in, so the window the operator clicked is
/// left half-painted underneath a dialog they cannot dismiss to finish it.
///
/// The two callers honour it differently and both are honest about which:
///
/// * `dialogs::ocr` sets a flag in its button arm and calls this **after**
///   `egui::Window::show` returns, still inside step 2b of the frame;
/// * `crate::app::save` is reached from `PdfcerApp::apply`, which is **step 3**
///   — after every panel, the canvas, the docks, the find bar and the dialogs
///   have all closed. That is the strongest position available and it is why
///   `file.save_copy` raises an `Action` rather than picking during dispatch.
///
/// The distinction matters because *dispatch is not always outside a layout
/// closure*: `PdfcerApp::central` dispatches the canvas's context-menu tokens
/// from **inside** `egui::CentralPanel::show`. See [`pick_document`], which is
/// called straight from a dispatch arm and therefore does not honour this.
#[must_use]
pub fn pick_save_path(suggested: &std::path::Path, title: &str) -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_SAVE_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = native_save(suggested, title);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "save-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// The platform save dialog, pre-filled from `suggested` and headed `title`.
///
/// The directory and the file name are set separately because `rfd` treats
/// them as separate: handing the whole path as a name would produce a dialog
/// offering to create a file called `D:\scans\survey-recognised.pdf` inside
/// whatever folder it happened to open in.
///
/// `title` is a parameter rather than a constant because the two callers are
/// asking about different things and the window's heading is the only place
/// the OS lets pdfcer say which — see [`pick_save_path`]. It is still catalog
/// copy: both call sites pass a `crate::text::*` function, and neither builds
/// a sentence.
fn native_save(suggested: &std::path::Path, title: &str) -> Picked {
    let mut dialog = rfd::FileDialog::new()
        .set_title(title)
        .add_filter(crate::text::files::filter_pdf(), &["pdf"]);
    if let Some(dir) = suggested.parent().filter(|d| !d.as_os_str().is_empty()) {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    dialog.save_file().map_or(Picked::Cancelled, Picked::Path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PdfcerApp;
    use crate::app::actions::Action;
    use crate::app::state::Status;
    use crate::panels::objects::test_support::engine_fixture;

    /// A four-page fixture that really opens.
    fn fixture() -> PathBuf {
        engine_fixture("pageops/four-pages.pdf")
    }

    /// The handler token the ribbon would raise for `id`.
    fn token_for(app: &PdfcerApp, id: &str) -> egui_shell::commands::HandlerToken {
        app.commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
            .handler
    }

    /// ★ **The picker's answer becomes an action, and only a path does.**
    ///
    /// The `file.open` arm reduced to the part a test may run — see rule 3 in
    /// this module's header for why dispatching the command itself is
    /// forbidden here. All three answers are checked, because the interesting
    /// failure is not "a path did nothing" but "a *cancel* opened something":
    /// `Picked` exists as three variants precisely so a dismissed dialog
    /// cannot be mistaken for a path, and an `Option<PathBuf>` collapsed with
    /// `unwrap_or_default` would open `""`.
    #[test]
    fn only_a_picked_path_becomes_an_action() {
        let mut actions = Vec::new();
        raise(Picked::Path(PathBuf::from("D:\\sheet.pdf")), &mut actions);
        assert_eq!(actions, vec![Action::Open(PathBuf::from("D:\\sheet.pdf"))]);

        let mut actions = Vec::new();
        raise(Picked::Cancelled, &mut actions);
        assert!(actions.is_empty(), "a dismissed dialog opens nothing");

        let mut actions = Vec::new();
        raise(Picked::Unavailable, &mut actions);
        assert!(actions.is_empty(), "a build with no picker opens nothing");
    }

    /// ★ **`file.close` raises the Close action, and applying it empties the
    /// shell.**
    ///
    /// `file.close` was registered, drawn on the File tab, gated on
    /// `doc.open` — and had no dispatch arm, exactly as `file.open` had none.
    /// Driven through the real token lookup rather than by calling the arm, so
    /// a command that stopped being registered fails here rather than silently
    /// taking the `command-unimplemented` path.
    #[test]
    fn the_close_command_empties_the_shell() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        app.panels.set_focus(3);

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, "file.close"), &mut actions);
        assert_eq!(actions, vec![Action::Close]);

        app.apply_actions(actions, 1.0);
        assert!(matches!(app.status, Status::Empty));
        assert_eq!(
            app.panels.focus(),
            None,
            "closing must forget the paint-order indices the panels held, exactly as \
             opening does — they name positions in a document that is no longer open"
        );
    }

    /// ★ **The Open action opens, from every starting state.**
    ///
    /// Including the one an operator meets most: nothing open at all.
    /// [`crate::app::PdfcerApp::apply`] refuses every other action when
    /// `Status` is not `Open`, which is right for actions about the open
    /// document and would be fatal here — so Open and Close are matched
    /// *before* that guard, and this is the assertion that says so.
    #[test]
    fn the_open_action_opens_whether_or_not_something_is_already_open() {
        let mut app = PdfcerApp::new();
        assert!(matches!(app.status, Status::Empty));

        app.apply_actions(vec![Action::Open(fixture())], 1.0);
        assert!(
            matches!(app.status, Status::Open(_)),
            "an Open with nothing open is the ordinary case, not a refused one"
        );

        // …and again over a document that is already open, which is the
        // second-file case the whole command exists for.
        app.apply_actions(vec![Action::Open(fixture())], 1.0);
        assert!(matches!(app.status, Status::Open(_)));

        // A Close with nothing open is a no-op rather than a panic: a
        // customized keymap can reach any command from any state.
        app.apply_actions(vec![Action::Close, Action::Close], 1.0);
        assert!(matches!(app.status, Status::Empty));
    }

    /// ★ **`file.save_copy` raises the SaveCopy action, through the real token
    /// lookup.**
    ///
    /// The regression guard for the defect this command shipped with for the
    /// whole life of the project: it was registered, drawn on the File tab,
    /// drawn on the quick-access toolbar, bound to `Ctrl+S`, printed "(Ctrl+S)"
    /// in its own tooltip — and had **no dispatch arm**, so every press traced
    /// `command-unimplemented` and nothing this shell could author could be
    /// written to disk.
    ///
    /// Driven through `commands.get(id).handler` rather than by calling the arm,
    /// exactly as `the_close_command_empties_the_shell` and
    /// `the_new_command_makes_a_blank_document_from_nothing` are, and for the
    /// reason those two record: a test that called the function directly would
    /// pass against a build in which the command was never registered, or in
    /// which the token-to-id lookup had stopped resolving — which is precisely
    /// the state that produced the fall-through in the first place.
    ///
    /// # ★ Why it stops at the action, and must
    ///
    /// It raises and does **not** apply. Applying `Action::SaveCopy` reaches
    /// `crate::app::save::save_copy`, which opens a **real modal save dialog**
    /// unless `PDFCER_DIAG_SAVE_PATH` is set — and this crate is
    /// `#![forbid(unsafe_code)]` while `std::env::set_var` is `unsafe` in
    /// edition 2024, so a test cannot set it. That is rule 3 in this module's
    /// header, moved one phase along with the picker: the *dispatch* of
    /// `file.save_copy` is safe to drive and its *apply* is not.
    ///
    /// What is therefore untested here and tested elsewhere, stated rather than
    /// implied by a green run: the write itself is covered by
    /// `crate::app::save`'s own tests, which call the picker-free half directly
    /// and re-open the file that comes out; and the whole chain — ribbon click,
    /// dispatch, apply, picker, write, re-open — is covered by
    /// `tools/ui-verify`'s `save_copy_round_trip`, which answers the dialog
    /// through [`DIAG_SAVE_PATH`] because that is the only way anything can.
    #[test]
    fn the_save_copy_command_raises_the_save_action() {
        let ctx = egui::Context::default();
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, "file.save_copy"), &mut actions);
        assert_eq!(
            actions,
            vec![Action::SaveCopy],
            "`file.save_copy` must raise an action rather than falling through to \
             `command-unimplemented`, and it must raise it rather than opening the picker here — \
             see `crate::app::save` section 4 on the frame-timing requirement"
        );
    }

    /// ★ **Nothing is pending, so nothing is blocked — and the gate is real.**
    ///
    /// The dirty-document rule has one home,
    /// [`crate::app::PdfcerApp::save_pending`], consulted by both arms. This
    /// build has no save, so it answers `false`, and the assertion is that the
    /// two arms therefore proceed. It is not a tautology: it pins the
    /// direction of the gate, so a future save subsystem that wired it
    /// backwards — blocking an Open whenever a document is merely *dirty*,
    /// which is not what the rule says — fails here rather than in an
    /// operator's hands.
    #[test]
    fn the_dirty_document_gate_blocks_nothing_in_a_build_with_no_save() {
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(!app.save_pending(), "there is no save path in this build");

        app.apply_actions(vec![Action::Close], 1.0);
        assert!(matches!(app.status, Status::Empty));
    }

    /// ★ **The diagnostic seam answers the dialog, in all three shapes.**
    ///
    /// This is the whole harness contract, and every row of the table in the
    /// module header is asserted: unset defers to the picker, a value is a
    /// path, and an *empty* value is a cancel — the third being the one a
    /// reader would otherwise assume was an accident.
    #[test]
    fn the_diagnostic_seam_answers_the_dialog() {
        assert_eq!(from_env(None), None, "unset must not answer at all");
        assert_eq!(
            from_env(Some(OsString::from("D:\\drawings\\sheet.pdf"))),
            Some(Picked::Path(PathBuf::from("D:\\drawings\\sheet.pdf")))
        );
        assert_eq!(
            from_env(Some(OsString::new())),
            Some(Picked::Cancelled),
            "an empty value is how a harness drives the cancel path without a dialog"
        );
    }

    /// A path with a space, and one that is not ASCII, both survive the seam.
    ///
    /// `OsString` rather than `String` throughout for the same reason
    /// `main.rs` reads `args_os`: a path is not required to be valid Unicode,
    /// and a non-Unicode path is the operator's business rather than ours to
    /// reject.
    #[test]
    fn the_seam_does_not_mangle_a_real_path() {
        for raw in [
            "C:\\Program Files\\a drawing.pdf",
            "D:\\Zeichnungen\\Übersicht.pdf",
        ] {
            assert_eq!(
                from_env(Some(OsString::from(raw))),
                Some(Picked::Path(PathBuf::from(raw)))
            );
        }
    }
}
