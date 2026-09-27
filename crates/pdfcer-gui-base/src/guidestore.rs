//! # `guidestore` — where a document's guides live between sessions
//!
//! One line per document in `guides.txt` in the settings directory: the
//! encoded guide set, a tab, then the document's absolute path. The newest
//! write goes first and the file keeps [`CAP`] documents. A missing or
//! unreadable store answers "no guides", so a first run needs no special case.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/guides.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::settings;

use crate::guidemodel::Guides;

/// The file's name, inside the settings directory.
pub const GUIDES_FILE: &str = "guides.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The separator between the guide payload and the path.
///
/// A tab, because it is the one ASCII character a Windows path cannot contain
/// and therefore the one that needs no escaping. See the module header.
const SEPARATOR: char = '\t';

/// How many documents are remembered.
pub const CAP: usize = 200;

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(GUIDES_FILE))
}

/// **The guides remembered for `document`**, or none.
///
/// Never fails. A missing file, an unreadable one and a corrupt one all answer
/// an empty set, because every one of them means the same thing to the caller.
#[must_use]
pub fn recall(document: &Path) -> Guides {
    recall_at(default_path().as_deref(), document)
}

/// **Remember `guides` against `document`.**
pub fn remember(document: &Path, guides: &Guides) {
    remember_at(default_path().as_deref(), document, guides);
}

/// **What a freshly opened document starts with**: its remembered guides,
/// and the view state that shows them.
///
/// The two halves are returned together because the rule joining them is the
/// point, and it belongs here rather than in
/// [`crate::app::state::OpenDoc::new`]: **a document that has remembered
/// guides opens with `view.guides` already on.**
///
/// The presence of the work *is* the preference — see this module's header §2
/// on why the three View ▸ Display toggles are not persisted while the guides
/// are. The alternative, restoring guides and leaving them invisible until the
/// operator finds a switch, is a feature that appears not to have worked; and
/// storing a fourth flag to say "show the things I just restored" would be
/// storing something derivable.
///
/// Every other field of the returned [`crate::viewer::ViewState`] is
/// `Default`, which is the conservative one — the same division of labour
/// `ViewState::default`'s own docs describe for Read mode's continuous
/// default: the path that knows the document is the path that may know better.
#[must_use]
pub fn opening(document: &Path) -> (Guides, crate::viewer::ViewState) {
    let guides = recall(document);
    let view = crate::viewer::ViewState {
        guides: !guides.is_empty(),
        ..crate::viewer::ViewState::default()
    };
    (guides, view)
}

/// [`recall`], against an explicit file — the seam tests use.
#[must_use]
pub fn recall_at(file: Option<&Path>, document: &Path) -> Guides {
    let wanted = absolute(document);
    let Some(file) = file else {
        return Guides::default();
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        return Guides::default();
    };
    for (payload, path) in parse(&text) {
        if path == wanted {
            return Guides::decode(&payload);
        }
    }
    Guides::default()
}

/// [`remember`], against an explicit file.
pub fn remember_at(file: Option<&Path>, document: &Path, guides: &Guides) {
    let Some(file) = file else {
        return;
    };
    let wanted = absolute(document);
    let previous = std::fs::read_to_string(file).unwrap_or_default();
    let mut lines: Vec<String> = Vec::with_capacity(CAP);
    if !guides.is_empty() {
        lines.push(format!(
            "{}{SEPARATOR}{}",
            guides.encode(),
            wanted.display()
        ));
    }
    for (payload, path) in parse(&previous) {
        if path == wanted || lines.len() >= CAP {
            continue;
        }
        lines.push(format!("{payload}{SEPARATOR}{}", path.display()));
    }

    // The parent directory may not exist on a first run — the same situation
    // `recent.rs` and `remembered.rs` handle, and the same answer: create it,
    // and let the write's own failure be the one that is reported.
    if let Some(dir) = file.parent()
        && !dir.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(dir);
    }
    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    if let Err(err) = std::fs::write(file, body) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("guides-write-failed path={} err={err}", file.display())
        });
    }
}

/// Split a file's text into `(payload, path)` pairs, dropping unparseable
/// lines.
fn parse(text: &str) -> Vec<(String, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let (payload, path) = line.split_once(SEPARATOR)?;
            let path = path.trim_end_matches(['\r']);
            if path.is_empty() {
                return None;
            }
            Some((payload.to_owned(), PathBuf::from(path)))
        })
        .collect()
}

/// A document's path as this store keys it.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidemodel::{Guide, GuideAxis};

    fn sample() -> Guides {
        let mut g = Guides::default();
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Horizontal,
            at: 120.5,
        }));
        assert!(g.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 64.0,
        }));
        assert!(g.add(Guide {
            page: 3,
            axis: GuideAxis::Horizontal,
            at: -12.25,
        }));
        g
    }

    /// **A document's guides come back after a reopen, and a second
    /// document's do not leak into the first.**
    #[test]
    fn guides_survive_a_reopen_and_stay_with_their_own_document() {
        let dir = std::env::temp_dir().join(format!("pdfcer-guides-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("guides-roundtrip.txt");
        let _ = std::fs::remove_file(&file);

        let drawing = PathBuf::from("D:\\Drawings\\job 4471\\sheet set.pdf");
        let report = PathBuf::from("C:\\reports\\quarterly.pdf");

        remember_at(Some(&file), &drawing, &sample());
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        // A document nobody has ruled up has no guides, and asking does not
        // hand it the other document's.
        assert!(recall_at(Some(&file), &report).is_empty());

        // A second document writes its own line without disturbing the first
        // — the read-modify-write property, and the one a naive "write my
        // line" implementation loses.
        let mut theirs = Guides::default();
        theirs.add(Guide {
            page: 0,
            axis: GuideAxis::Vertical,
            at: 306.0,
        });
        remember_at(Some(&file), &report, &theirs);
        assert_eq!(recall_at(Some(&file), &drawing), sample());
        assert_eq!(recall_at(Some(&file), &report), theirs);

        // Clearing the last guide forgets the document rather than leaving an
        // empty marker behind.
        remember_at(Some(&file), &report, &Guides::default());
        assert!(recall_at(Some(&file), &report).is_empty());
        let text = std::fs::read_to_string(&file).expect("the file is still there");
        assert!(
            !text.contains("quarterly.pdf"),
            "an emptied document must not keep a line: {text:?}"
        );
        assert!(
            text.contains("sheet set.pdf"),
            "the other document survived"
        );

        let _ = std::fs::remove_file(&file);
    }

    /// **A path containing spaces round-trips**, which is why the payload is
    /// written first and the path is the whole remainder of the line.
    #[test]
    fn a_path_with_spaces_survives_the_format() {
        let pairs = parse("0:h:10 1:v:20\tC:\\Program Files\\a b\\c d.pdf\n");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "0:h:10 1:v:20");
        assert_eq!(pairs[0].1, PathBuf::from("C:\\Program Files\\a b\\c d.pdf"));
    }

    /// A line with no tab, or with an empty path, is dropped rather than
    /// producing a guide set attached to nothing.
    #[test]
    fn a_malformed_line_is_dropped() {
        assert!(parse("no tab here\n").is_empty());
        assert!(parse("0:h:10\t\n").is_empty());
        assert!(parse("").is_empty());
        // …and a well-formed line among broken ones still reads.
        let pairs = parse("junk\n0:h:10\tC:\\a.pdf\n\t\n");
        assert_eq!(pairs.len(), 1);
    }

    /// Reading a store that does not exist answers "no guides" rather than
    /// failing — the same posture `remembered::recall` takes, and the reason
    /// a first run needs no special case.
    #[test]
    fn a_missing_store_answers_no_guides() {
        // temp-path-exempt: nothing is created here; the test needs a path
        // that is absent, and absence is not contended.
        let missing = std::env::temp_dir().join("pdfcer-guides-does-not-exist-4471.txt");
        let _ = std::fs::remove_file(&missing);
        assert!(recall_at(Some(&missing), Path::new("C:\\a.pdf")).is_empty());
        assert!(recall_at(None, Path::new("C:\\a.pdf")).is_empty());
    }
}
