#![cfg(test)]
//! Tests for [`super`] — the decisions, over values.
//!
//! # ★★ What is asserted here, and what deliberately is not
//!
//! Everything in this file runs without a registry and without starting a
//! process. That is not a limitation of the tests, it is the property
//! [`super::Registrations`] and [`super::Launcher`] exist to give them: a test
//! that shelled out to `reg.exe` would pass or fail according to what Adobe's
//! installer last did on the machine running it, which makes it a report about
//! the machine rather than about the code — and it would produce a *different*
//! verdict on the operator's laptop, on a build server, and on a colleague's
//! desk with Reader instead of Pro.
//!
//! What is **not** asserted here, and cannot be: that `reg query`'s output
//! format is what this code believes. That is pinned separately, against bytes
//! captured from this machine, in [`super::windows`]'s own tests.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Edition, Launcher, Prompt, Registrations, Source, Viewer, prompt_for, resolve};

/// A machine described by a table rather than discovered.
///
/// `app_paths` is keyed by executable name — `"Acrobat.exe"` — exactly as
/// [`Registrations::app_path`] is asked. `present` is the set of paths that
/// exist; **a path not in it does not exist**, which is how "the registry
/// names an Acrobat that has been uninstalled" is expressed without creating
/// and deleting real files.
#[derive(Debug, Default)]
struct Machine {
    app_paths: BTreeMap<String, String>,
    handler: Option<String>,
    present: Vec<PathBuf>,
}

impl Machine {
    fn with_app_path(mut self, executable: &str, value: &str) -> Self {
        self.app_paths
            .insert(executable.to_owned(), value.to_owned());
        self.exists_at(value)
    }

    fn with_handler(mut self, command: &str) -> Self {
        self.handler = Some(command.to_owned());
        self
    }

    /// Say a path exists without registering it anywhere.
    fn exists_at(mut self, path: &str) -> Self {
        self.present.push(PathBuf::from(path));
        self
    }

    /// Say a registration exists but the file behind it does not — an
    /// uninstall that left its `App Paths` key behind.
    fn with_stale_app_path(mut self, executable: &str, value: &str) -> Self {
        self.app_paths
            .insert(executable.to_owned(), value.to_owned());
        self
    }
}

impl Registrations for Machine {
    fn app_path(&self, executable: &str) -> Option<String> {
        self.app_paths.get(executable).cloned()
    }

    fn pdf_handler_command(&self) -> Option<String> {
        self.handler.clone()
    }

    fn exists(&self, path: &Path) -> bool {
        self.present.iter().any(|p| p == path)
    }
}

const PRO: &str = r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe";
const READER: &str = r"C:\Program Files\Adobe\Reader\AcroRd32.exe";
const ELSEWHERE: &str = r"D:\Apps\Acrobat\Acrobat.exe";
const RIVAL: &str = r"C:\Program Files\PDF Studio\PDFStudio.exe";

/// **★★★ Nothing found and nothing configured ⇒ no button.**
///
/// R9, and the reason the whole capability is expressed as a `visible_when`
/// rather than as an `enabled_when`: *an unavailable capability renders
/// nothing; greying is reserved for **temporarily** unavailable and is always
/// explained on hover.* A machine with no Acrobat is not temporarily anything.
///
/// The decision the ribbon makes is `resolve(...).is_some()`, so this is the
/// button-visibility test in the only form it has outside a window.
#[test]
fn a_machine_with_no_acrobat_offers_no_viewer_and_therefore_no_button() {
    let bare = Machine::default();
    assert_eq!(resolve(&bare, None), None);
    assert_eq!(resolve(&bare, Some("")), None, "a cleared setting is unset");
    assert_eq!(resolve(&bare, Some("   ")), None);

    // A machine whose only PDF handler is somebody else's product is a
    // machine with no Acrobat. See `discover::edition_of`.
    let rival = Machine::default()
        .with_handler(&format!("\"{RIVAL}\" \"%1\""))
        .exists_at(RIVAL);
    assert_eq!(
        resolve(&rival, None),
        None,
        "PDF Studio is not an Acrobat, whatever it is registered to open"
    );
}

/// **★★★ Pro beats Reader when both are installed.**
///
/// `OPERATOR_REQUESTS.md` O122: *"acrobat reader or pro depending on what is
/// installed"*, decided in favour of Pro because Pro is the superset and is
/// what somebody who owns both reaches for.
#[test]
fn pro_beats_reader_when_both_are_installed() {
    let both = Machine::default()
        .with_app_path("AcroRd32.exe", READER)
        .with_app_path("Acrobat.exe", PRO);
    let chosen = resolve(&both, None).expect("one of the two");
    assert_eq!(chosen.edition, Edition::Pro);
    assert_eq!(chosen.path, PathBuf::from(PRO));

    // Reader alone is still an answer — it is the fallback, not a refusal.
    let reader_only = Machine::default().with_app_path("AcroRd32.exe", READER);
    let chosen = resolve(&reader_only, None).expect("Reader is enough");
    assert_eq!(chosen.edition, Edition::Reader);
    assert_eq!(chosen.source, Source::AppPaths);
}

/// **★★ Edition outranks source: a Pro found through the `.pdf` handler beats
/// a Reader found in `App Paths`.**
///
/// The ordering that makes [`Source::rank`] a tie-break rather than the first
/// sort key. Sorting by source first would answer *Reader*, on the reasoning
/// that `App Paths` is the better-quality registration — which is true and
/// beside the point, because the operator asked for Pro when Pro is there.
#[test]
fn pro_beats_reader_even_when_reader_is_the_registered_handler() {
    let mixed = Machine::default()
        .with_app_path("AcroRd32.exe", READER)
        .with_handler(&format!("\"{PRO}\" \"%1\""))
        .exists_at(PRO);
    let chosen = resolve(&mixed, None).expect("both are present");
    assert_eq!(chosen.edition, Edition::Pro);
    assert_eq!(chosen.source, Source::PdfHandler);
}

/// **★★★ A configured path beats discovery, and does not fall back to it.**
///
/// The escape hatch of O122 point 4, and the two halves are equally
/// load-bearing. Beating discovery is what makes the setting mean anything at
/// all. *Not falling back* is what stops a person who deliberately pointed
/// pdfcer at their second installation from being silently sent to their
/// first — which would undo the setting with nothing on screen saying so.
#[test]
fn a_configured_path_beats_discovery_and_never_falls_back_to_it() {
    let machine = Machine::default()
        .with_app_path("Acrobat.exe", PRO)
        .exists_at(ELSEWHERE);

    let chosen = resolve(&machine, Some(ELSEWHERE)).expect("the typed path exists");
    assert_eq!(
        chosen.path,
        PathBuf::from(ELSEWHERE),
        "the operator's choice"
    );
    assert_eq!(chosen.source, Source::Configured);
    assert_eq!(chosen.edition, Edition::Pro);

    // Whitespace around a typed path is the operator's, not a second path.
    let padded = resolve(&machine, Some(&format!("  {ELSEWHERE}  "))).expect("trimmed");
    assert_eq!(padded.path, PathBuf::from(ELSEWHERE));

    // A typo yields NOTHING — not the discovered Acrobat.
    assert_eq!(
        resolve(&machine, Some(r"D:\Apps\Acrobat\Acrobatt.exe")),
        None,
        "a configured path that does not exist must not silently fall back"
    );
}

/// **★★ A registration whose file is gone is not offered.**
///
/// An uninstall that leaves its `App Paths` key behind is ordinary. Offering
/// the button anyway would produce a control that is present, enabled, and
/// does nothing when pressed — R9's failure reached from the other direction.
#[test]
fn a_stale_registration_is_not_offered() {
    let stale = Machine::default().with_stale_app_path("Acrobat.exe", PRO);
    assert_eq!(resolve(&stale, None), None);

    // …and it does not shadow a Reader that IS installed.
    let stale_pro_live_reader = Machine::default()
        .with_stale_app_path("Acrobat.exe", PRO)
        .with_app_path("AcroRd32.exe", READER);
    let chosen = resolve(&stale_pro_live_reader, None).expect("Reader is really there");
    assert_eq!(chosen.edition, Edition::Reader);
}

/// A configured path is labelled by its file name, and an unrecognisable one
/// is still honoured.
///
/// ★ The deliberate asymmetry with discovery: [`super::discover::edition_of`]
/// is a **filter** on what the registry offers and a **label** on what the
/// operator typed. Somebody who points this setting at a renamed executable,
/// a launcher script wrapper or a portable install has answered the question
/// the filter exists to ask, and refusing them would make the escape hatch
/// narrower than the thing it is an escape from.
#[test]
fn a_configured_path_is_honoured_even_if_its_name_is_not_one_we_know() {
    let odd = r"D:\Apps\acrobat-portable\launch.exe";
    let machine = Machine::default().exists_at(odd);
    let chosen = resolve(&machine, Some(odd)).expect("the operator said so");
    assert_eq!(chosen.path, PathBuf::from(odd));
    assert_eq!(chosen.source, Source::Configured);
    assert_eq!(
        chosen.edition,
        Edition::Pro,
        "an unrecognised name is assumed to be the fuller product, not the reader"
    );
}

/// **★★★ The three prompts, and the never-saved case refusing distinctly.**
///
/// O122 points 5 and 6, plus the case the operator did not name and the code
/// must still answer. The order of the two questions is the content of
/// [`prompt_for`]: a never-saved document is *also* dirty, so testing
/// dirtiness first would offer *"Save and open"* over a document with nowhere
/// to save to.
#[test]
fn the_document_state_chooses_the_dialog_and_never_saved_refuses_distinctly() {
    assert_eq!(
        prompt_for(true, true),
        Prompt::SaveFirst,
        "a file on disk with unsaved edits: save first, or cancel"
    );
    assert_eq!(
        prompt_for(true, false),
        Prompt::ConfirmClose,
        "clean: say the file will be closed, OK or Cancel"
    );
    assert_eq!(
        prompt_for(false, true),
        Prompt::NoFileOnDisk,
        "never saved: a refusal in its own words, not `save first`"
    );
    assert_eq!(
        prompt_for(false, false),
        Prompt::NoFileOnDisk,
        "no file is no file, however the edit counter reads"
    );

    // The three are genuinely three. A test that only checked each input
    // produced *an* answer would pass on an implementation that returned the
    // same one every time.
    assert_ne!(prompt_for(true, true), prompt_for(true, false));
    assert_ne!(prompt_for(true, true), prompt_for(false, true));
    assert_ne!(prompt_for(true, false), prompt_for(false, false));
}

/// A launcher that records rather than launches.
#[derive(Debug, Default)]
struct Recorder {
    calls: std::cell::RefCell<Vec<(PathBuf, PathBuf)>>,
}

impl Launcher for Recorder {
    fn launch(&self, viewer: &Viewer, file: &Path) -> std::io::Result<()> {
        self.calls
            .borrow_mut()
            .push((viewer.path.clone(), file.to_path_buf()));
        Ok(())
    }
}

/// **★ The launch hands the viewer the document, and exactly that.**
///
/// Thin, and worth having anyway: it is the assertion that the two paths are
/// not transposed. A `launch(file, viewer)` compiles, runs, and fails only on
/// a real machine, where it would try to start the operator's PDF as if it
/// were a program.
#[test]
fn launching_passes_the_document_to_the_viewer() {
    let recorder = Recorder::default();
    let viewer = Viewer {
        path: PathBuf::from(PRO),
        edition: Edition::Pro,
        source: Source::AppPaths,
    };
    let document = PathBuf::from(r"D:\Drawings\sheet 1.pdf");
    super::launch(&recorder, &viewer, &document).expect("the recorder never fails");

    let calls = recorder.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, PathBuf::from(PRO), "the program to start");
    assert_eq!(calls[0].1, document, "the file it is given");
}
