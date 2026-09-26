//! Tests for `crate::app::recent`, kept in the gui because they reach gui modules.

use crate::app::recent::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A fresh, empty directory nothing else is using.
fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    // The pid is what `tools/gates/check-test-temp-paths.py` requires;
    // `nanos` stays because it also separates repeated runs inside one
    // process, which the pid does not.
    let dir = std::env::temp_dir().join(format!(
        "pdfcer-gui-recent-{tag}-{nanos}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a temp dir");
    dir
}

/// A real file inside `dir`, so `is_file` says yes about it.
fn touch(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, b"%PDF-1.7\n").expect("writes");
    path
}

/// **The recent list sits beside the layout file and the settings
/// file.**
#[test]
fn the_recent_file_lives_beside_the_layout_file() {
    let dir = temp_dir("beside");
    let recent = RecentFiles::load_in(&dir);
    assert_eq!(recent.path(), Some(dir.join(RECENT_FILE).as_path()));
    assert_eq!(
        RecentFiles::default_path()
            .as_deref()
            .and_then(Path::parent),
        crate::app::persistence::LayoutStore::default_path()
            .as_deref()
            .and_then(Path::parent),
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A first run is an empty list, and it is not a failure.
#[test]
fn a_first_run_is_an_empty_list() {
    let dir = temp_dir("first-run");
    let recent = RecentFiles::load_in(&dir);
    assert!(recent.is_empty());
    assert!(recent.can_save());
    assert_eq!(recent.saves(), 0);
    assert_eq!(recent.save_error(), None);
    let _ = std::fs::remove_dir_all(&dir);
}

/// **Newest first, and re-opening a document moves it rather than
/// duplicating it.**
#[test]
fn the_newest_document_is_first_and_a_repeat_open_moves_it() {
    let dir = temp_dir("order");
    let mut recent = RecentFiles::load_in(&dir);
    let a = touch(&dir, "a.pdf");
    let b = touch(&dir, "b.pdf");
    let c = touch(&dir, "c.pdf");

    recent.remember(&a);
    recent.remember(&b);
    recent.remember(&c);
    assert_eq!(recent.entries(), [c.clone(), b.clone(), a.clone()]);

    recent.remember(&a);
    assert_eq!(
        recent.entries(),
        [a.clone(), c.clone(), b.clone()],
        "an entry already present moves to the front rather than appearing twice"
    );

    // …and re-opening what is already at the front costs no write at all.
    let before = recent.saves();
    recent.remember(&a);
    assert_eq!(recent.saves(), before);
    assert_eq!(recent.entries().len(), 3);

    let _ = std::fs::remove_dir_all(&dir);
}

/// The list is capped, and the cap drops the OLDEST entry.
#[test]
fn the_list_is_capped_and_the_oldest_entry_is_what_goes() {
    let dir = temp_dir("cap");
    let mut recent = RecentFiles::load_in(&dir);
    let first = touch(&dir, "0.pdf");
    recent.remember(&first);
    for n in 1..=CAP {
        let path = touch(&dir, &format!("{n}.pdf"));
        recent.remember(&path);
    }
    assert_eq!(recent.entries().len(), CAP);
    assert!(
        !recent.entries().contains(&first),
        "the oldest entry is the one the cap drops"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// **A missing file is dropped at DISPLAY time and kept in the file.**
#[test]
fn a_missing_file_leaves_the_menu_and_stays_in_the_file() {
    let dir = temp_dir("missing");
    let mut recent = RecentFiles::load_in(&dir);
    let here = touch(&dir, "here.pdf");
    let gone = touch(&dir, "gone.pdf");
    recent.remember(&here);
    recent.remember(&gone);

    let now = Instant::now();
    assert_eq!(recent.present_at(now).len(), 2);

    std::fs::remove_file(&gone).expect("removes");
    // Far enough past the throttle that the answer is re-taken.
    let later = now + PRESENCE_TTL;
    assert_eq!(
        recent.present_at(later),
        vec![here.clone()],
        "the menu must not offer a document that is not there"
    );
    assert!(
        recent.entries().contains(&gone),
        "…and the list must not forget it: the drive may come back"
    );

    let text = std::fs::read_to_string(dir.join(RECENT_FILE)).expect("reads back");
    assert!(
        text.contains("gone.pdf"),
        "the absent entry must survive in the file: {text}"
    );

    // And when it comes back, so does the entry — with no re-open needed.
    std::fs::write(&gone, b"%PDF-1.7\n").expect("writes");
    let later_still = later + PRESENCE_TTL;
    assert_eq!(recent.present_at(later_still).len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

/// **The presence check is throttled.**
#[test]
fn the_presence_answer_is_reused_until_the_window_passes() {
    let dir = temp_dir("throttle");
    let mut recent = RecentFiles::load_in(&dir);
    let path = touch(&dir, "sheet.pdf");
    recent.remember(&path);

    let now = Instant::now();
    assert_eq!(recent.present_at(now).len(), 1);
    std::fs::remove_file(&path).expect("removes");
    assert_eq!(
        recent.present_at(now + PRESENCE_TTL / 2).len(),
        1,
        "inside the window the cached answer stands, whatever the disk now says"
    );
    assert_eq!(
        recent.present_at(now + PRESENCE_TTL).len(),
        0,
        "…and at the window the answer is taken again"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The list survives a restart, through a real file.
#[test]
fn the_list_survives_a_restart() {
    let dir = temp_dir("round-trip");
    let a = touch(&dir, "a.pdf");
    let b = touch(&dir, "b.pdf");
    {
        let mut recent = RecentFiles::load_in(&dir);
        recent.remember(&a);
        recent.remember(&b);
        assert_eq!(recent.saves(), 2);
        assert_eq!(recent.save_error(), None);
    }
    let reopened = RecentFiles::load_in(&dir);
    assert_eq!(reopened.entries(), [b, a]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A relative path is stored absolute, or it means something else next
/// time.
#[test]
fn a_relative_path_is_stored_absolute() {
    let dir = temp_dir("relative");
    let mut recent = RecentFiles::load_in(&dir);
    recent.remember(Path::new("drawing.pdf"));
    let stored = &recent.entries()[0];
    assert!(
        stored.is_absolute(),
        "a relative entry names a different file from a different working \
         directory: {stored:?}"
    );
    assert_eq!(
        stored.file_name().and_then(|n| n.to_str()),
        Some("drawing.pdf")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A hand-edited file cannot produce a list this module would not itself
/// have written: blank lines go, duplicates go, the cap holds.
#[test]
fn a_hand_edited_file_is_normalised_on_load() {
    let dir = temp_dir("hand-edited");
    let mut text = String::from("\n\n");
    text.push_str(&dir.join("one.pdf").to_string_lossy());
    text.push('\n');
    text.push_str(&dir.join("one.pdf").to_string_lossy());
    text.push('\n');
    for n in 0..CAP * 2 {
        text.push_str(&dir.join(format!("{n}.pdf")).to_string_lossy());
        text.push('\n');
    }
    std::fs::write(dir.join(RECENT_FILE), text).expect("writes");

    let recent = RecentFiles::load_in(&dir);
    assert_eq!(recent.entries().len(), CAP);
    assert_eq!(recent.entries()[0], dir.join("one.pdf"));
    assert_eq!(
        recent.entries()[1],
        dir.join("0.pdf"),
        "the duplicate second line is dropped rather than shifting everything"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// **A default store points nowhere and can erase nothing.**
#[test]
fn a_default_store_points_nowhere_and_writes_nothing() {
    let mut recent = RecentFiles::default();
    assert!(!recent.can_save());
    assert_eq!(recent.path(), None);
    recent.remember(Path::new("anything.pdf"));
    assert_eq!(recent.entries().len(), 1, "the session still remembers");
    assert_eq!(recent.saves(), 0, "…and nothing was written anywhere");
    assert_eq!(recent.save_error(), None);
}

// =======================================================================
// The application path: what records an entry, and what opens one
// =======================================================================

/// **Opening a document records it; failing to open one does not.**
#[test]
fn opening_a_document_records_it_and_a_failed_open_does_not() {
    use crate::panels::objects::test_support::engine_fixture;

    let mut app = crate::app::PdfcerApp::new();
    assert!(
        app.recent.is_empty(),
        "a test build starts with a store that points nowhere — see `PdfcerApp::new`"
    );

    let fixture = engine_fixture("pageops/four-pages.pdf");
    app.open_path(fixture.clone());
    assert_eq!(
        app.recent.entries(),
        [std::path::absolute(&fixture).expect("absolute")]
    );

    app.open_path(engine_fixture("not-a-pdf.bin"));
    assert!(
        matches!(app.status, crate::app::state::Status::Failed { .. }),
        "this fixture must fail to open, or the test proves nothing"
    );
    assert_eq!(
        app.recent.entries().len(),
        1,
        "a file that would not open must not be offered as a recent DOCUMENT"
    );

    // …and closing does not forget it. Closing a document is the single
    // most likely moment to reach for the one before it.
    app.close_document();
    assert_eq!(app.recent.entries().len(), 1);
}

/// **`file.recent` opens the entry the menu parked, and falls back to
/// the newest reachable one when there is none.**
#[test]
fn the_recent_command_opens_the_parked_choice_or_the_newest_reachable() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    use crate::app::actions::Action;
    use crate::panels::objects::test_support::engine_fixture;

    let mut app = crate::app::PdfcerApp::new();
    let four = engine_fixture("pageops/four-pages.pdf");
    let layers = engine_fixture("layers/painted-layers.pdf");
    app.recent.remember(&four);
    app.recent.remember(&layers);

    // With nothing parked: the newest entry that can be seen.
    let mut actions = Vec::new();
    app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
    assert_eq!(
        actions,
        vec![Action::Open(
            std::path::absolute(&layers).expect("absolute")
        )]
    );

    // With a choice parked by the menu: that one, and the slot is emptied
    // so the next invocation cannot re-open it by accident.
    app.recent_choice = Some(four.clone());
    let mut actions = Vec::new();
    app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
    assert_eq!(actions, vec![Action::Open(four)]);
    assert!(app.recent_choice.is_none(), "the operand is consumed");
}

/// An empty list raises nothing rather than an action naming nothing.
#[test]
fn the_recent_command_with_nothing_to_open_raises_nothing() {
    // A bare context: these tests exercise the dispatcher, not a
    // frame. `dispatch_command` needs one because three navigation arms
    // write the armed tool and the zoom anchor into egui memory, which
    // is where per-frame UI state lives.
    let ctx = egui::Context::default();
    let mut app = crate::app::PdfcerApp::new();
    let mut actions = Vec::new();
    app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
    assert!(actions.is_empty());
}

/// An unreadable file is an empty list and a working session.
#[test]
fn a_directory_where_the_file_should_be_is_survived() {
    let dir = temp_dir("unreadable");
    // A directory named `recent.txt` cannot be read as a file, on every
    // platform, without needing permissions a test cannot set.
    std::fs::create_dir_all(dir.join(RECENT_FILE)).expect("a decoy directory");
    let recent = RecentFiles::load_in(&dir);
    assert!(recent.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The per-document store shares a directory with the recent list.
#[test]
fn the_remembered_store_sits_beside_the_recent_list() {
    assert_eq!(
        crate::viewer::remembered::default_path()
            .as_deref()
            .and_then(std::path::Path::parent),
        RecentFiles::default_path()
            .as_deref()
            .and_then(std::path::Path::parent),
    );
}
