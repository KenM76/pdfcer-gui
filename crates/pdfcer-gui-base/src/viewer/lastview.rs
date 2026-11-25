//! # `viewer::lastview` — where each document was left, per document, on disk
//!
//! Opening a file again returns to the page, position and zoom it was closed
//! at (`OPERATOR_REQUESTS.md` O464), under Settings > Display.
//!
//! One line per document, newest first, capped at [`CAP`]:
//! `page<TAB>fx<TAB>fy<TAB>zoom<TAB>fit<TAB>path`.
//! - `page` is zero-based.
//! - `fx`/`fy` place the viewport's centre as a fraction of that page's drawn
//!   size, so the position survives a different window size.
//! - `zoom` is the view's zoom factor; `fit` is `none`, `page`, `width` or
//!   `height`, and a fit is restored as the fit rather than as its zoom.
//! - Paths are absolute, as in [`super::remembered`]. A line that does not
//!   parse is dropped.

use std::path::{Path, PathBuf};

use pdfcer_core::settings;

use super::fit::FitMode;
use super::remembered::absolute;

/// The file's name, inside the settings directory.
pub const LAST_VIEW_FILE: &str = "last-view.txt"; // ui-text-exempt: a file name, never displayed as copy

/// How many documents are remembered.
pub const CAP: usize = 200;

/// Where one document was left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LastView {
    /// The zero-based page the viewport's centre was on.
    pub page: usize,
    /// The viewport's centre as a fraction of that page's drawn size.
    pub frac: (f32, f32),
    /// The view's zoom factor.
    pub zoom: f32,
    /// The fit the view was in.
    pub fit: FitMode,
}

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(LAST_VIEW_FILE))
}

/// Where `document` was left, if it is remembered.
#[must_use]
pub fn recall(document: &Path) -> Option<LastView> {
    recall_at(default_path().as_deref(), document)
}

/// Remember that `document` was left at `view`.
pub fn remember(document: &Path, view: LastView) {
    remember_at(default_path().as_deref(), document, view);
}

/// [`recall`], against an explicit file.
#[must_use]
pub fn recall_at(file: Option<&Path>, document: &Path) -> Option<LastView> {
    let wanted = absolute(document);
    let text = std::fs::read_to_string(file?).ok()?;
    parse(&text)
        .into_iter()
        .find(|(_, path)| *path == wanted)
        .map(|(view, _)| view)
}

/// [`remember`], against an explicit file. Writes nothing when `document` is
/// already first with this view.
pub fn remember_at(file: Option<&Path>, document: &Path, view: LastView) {
    let Some(file) = file else {
        return;
    };
    let path = absolute(document);
    let mut entries = std::fs::read_to_string(file)
        .ok()
        .map(|text| parse(&text))
        .unwrap_or_default();
    if entries
        .first()
        .is_some_and(|(v, p)| *v == view && *p == path)
    {
        return;
    }
    entries.retain(|(_, existing)| *existing != path);
    entries.insert(0, (view, path.clone()));
    entries.truncate(CAP);
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(file, render(&entries)) {
        Ok(()) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "last-view-remembered page={} at={:.4},{:.4} zoom={:.4} fit={} path={path:?}",
                view.page,
                view.frac.0,
                view.frac.1,
                view.zoom,
                fit_key(view.fit)
            )
        }),
        Err(error) => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("last-view-remember-failed file={file:?} error={error}")
        }),
    }
}

fn fit_key(fit: FitMode) -> &'static str {
    match fit {
        FitMode::None => "none",
        FitMode::Page => "page",
        FitMode::Width => "width",
        FitMode::Height => "height",
    }
}

fn fit_from_key(key: &str) -> Option<FitMode> {
    Some(match key {
        "none" => FitMode::None,
        "page" => FitMode::Page,
        "width" => FitMode::Width,
        "height" => FitMode::Height,
        _ => return None,
    })
}

fn parse_line(line: &str) -> Option<(LastView, PathBuf)> {
    let mut fields = line.trim_end_matches(['\r', '\n']).splitn(6, '\t');
    let page = fields.next()?.parse().ok()?;
    let fx: f32 = fields.next()?.parse().ok()?;
    let fy: f32 = fields.next()?.parse().ok()?;
    let zoom: f32 = fields.next()?.parse().ok()?;
    let fit = fit_from_key(fields.next()?)?;
    let path = fields.next().filter(|p| !p.is_empty())?;
    let sane = fx.is_finite() && fy.is_finite() && zoom.is_finite() && zoom > 0.0;
    sane.then(|| {
        let view = LastView {
            page,
            frac: (fx, fy),
            zoom,
            fit,
        };
        (view, PathBuf::from(path))
    })
}

fn parse(text: &str) -> Vec<(LastView, PathBuf)> {
    text.lines().filter_map(parse_line).collect()
}

fn render(entries: &[(LastView, PathBuf)]) -> String {
    let mut text = String::new();
    for (v, path) in entries {
        // A path that is not valid Unicode cannot be spelled in this format,
        // as in `remembered`.
        if let Some(spelled) = path.to_str() {
            text.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{spelled}\n",
                v.page,
                v.frac.0,
                v.frac.1,
                v.zoom,
                fit_key(v.fit)
            ));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir()
            .join(format!(
                "pdfcer-lastview-{tag}-{}-{nanos}",
                std::process::id()
            ))
            .join(LAST_VIEW_FILE)
    }

    fn view(page: usize) -> LastView {
        LastView {
            page,
            frac: (0.25, 0.75),
            zoom: 2.5,
            fit: FitMode::None,
        }
    }

    #[test]
    fn a_remembered_view_is_recalled_exactly() {
        let file = temp_file("round");
        let one = Path::new("C:/a b/one.pdf");
        let two = Path::new("C:/two.pdf");
        remember_at(Some(&file), one, view(3));
        let width = LastView {
            fit: FitMode::Width,
            ..view(0)
        };
        remember_at(Some(&file), two, width);
        assert_eq!(recall_at(Some(&file), one), Some(view(3)));
        assert_eq!(recall_at(Some(&file), two), Some(width));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn remembering_again_replaces_the_entry_and_moves_it_first() {
        let file = temp_file("again");
        remember_at(Some(&file), Path::new("C:/a.pdf"), view(1));
        remember_at(Some(&file), Path::new("C:/b.pdf"), view(2));
        remember_at(Some(&file), Path::new("C:/a.pdf"), view(5));
        let entries = parse(&std::fs::read_to_string(&file).unwrap());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], (view(5), absolute(Path::new("C:/a.pdf"))));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn a_damaged_line_is_dropped_and_its_neighbours_kept() {
        let text =
            "1\t0.5\t0.5\t1\tpage\tC:/good.pdf\nnot a line\n2\tx\t0.5\t1\tpage\tC:/bad.pdf\n";
        let parsed = parse(text);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].1, PathBuf::from("C:/good.pdf"));
    }

    #[test]
    fn no_writable_location_remembers_nothing() {
        remember_at(None, Path::new("C:/a.pdf"), view(1));
        assert_eq!(recall_at(None, Path::new("C:/a.pdf")), None);
    }
}
