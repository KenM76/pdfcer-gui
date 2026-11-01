//! The documents open when the program last ran, for a host whose system
//! ends the process while the operator is away (Android): the host calls
//! [`keep`] once at start, opens what it returns, and from then on every
//! frame's open documents are written to the settings folder when they
//! change. A host that never calls [`keep`] writes nothing.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The list's file, beside the recent-files list.
const FILE: &str = "open-documents.txt"; // ui-text-exempt: file name, never displayed

/// The folder the list lives in and what it last held, once [`keep`] ran.
static KEPT: Mutex<Option<(PathBuf, Vec<PathBuf>)>> = Mutex::new(None);

/// Keeps settings, recent files and the reopen list in `dir`, for a host
/// whose platform has no settings folder the engine can find (Android).
/// Call once, before anything loads settings; `false` when the engine
/// refused it (the store was already resolved, or another folder was set).
pub fn store_in(dir: &Path) -> bool {
    let set = pdfcer_core::settings::set_store_dir(dir);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("settings-store-dir ok={}", set.is_ok())
    });
    set.is_ok()
}

/// Starts keeping the open documents and returns those open last time that
/// still exist, in tab order. Empty when there is no settings folder.
pub fn keep() -> Vec<PathBuf> {
    let Some(dir) = pdfcer_core::settings::resolve_store()
        .directory()
        .map(Path::to_path_buf)
    else {
        return Vec::new();
    };
    keep_in(dir)
}

fn keep_in(dir: PathBuf) -> Vec<PathBuf> {
    let last = read(&dir.join(FILE));
    let found: Vec<PathBuf> = last.iter().filter(|p| p.is_file()).cloned().collect();
    *KEPT.lock().unwrap_or_else(|e| e.into_inner()) = Some((dir, last));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("reopen-kept found={}", found.len())
    });
    found
}

/// Records `open`, the documents open now in tab order, when [`keep`] ran
/// and they differ from what was last written.
pub(crate) fn note(open: &[&Path]) {
    let mut kept = KEPT.lock().unwrap_or_else(|e| e.into_inner());
    let Some((dir, last)) = kept.as_mut() else {
        return;
    };
    if last.len() == open.len() && last.iter().zip(open).all(|(a, b)| a == b) {
        return;
    }
    *last = open.iter().map(|p| p.to_path_buf()).collect();
    let text: String = last.iter().map(|p| format!("{}\n", p.display())).collect();
    let written =
        std::fs::create_dir_all(&*dir).and_then(|()| std::fs::write(dir.join(FILE), text));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "reopen-noted open={} written={}",
            open.len(),
            written.is_ok()
        )
    });
}

/// The paths in `file`, one per line; none when it is missing.
fn read(file: &Path) -> Vec<PathBuf> {
    std::fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_was_open_comes_back_without_the_files_now_gone() {
        let dir = std::env::temp_dir().join(format!("pdfcer-reopen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.pdf"), dir.join("b.pdf"));
        std::fs::write(&a, b"%PDF").unwrap();
        std::fs::write(&b, b"%PDF").unwrap();
        assert!(keep_in(dir.clone()).is_empty(), "nothing was open before");
        note(&[b.as_path(), a.as_path()]);
        std::fs::remove_file(&b).unwrap();
        assert_eq!(
            keep_in(dir.clone()),
            vec![a.clone()],
            "b is gone; a stays, in order"
        );
        note(&[]);
        assert!(
            keep_in(dir.clone()).is_empty(),
            "closing everything forgets it"
        );
        *KEPT.lock().unwrap() = None;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
