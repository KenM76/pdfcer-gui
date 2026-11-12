//! # `prefs::printpresets` — saved Print settings and the folders they live in
//!
//! A preset is one text file, `<name>.pdfcerprint`, holding the Print block of
//! `preferences.txt` in the same `key = value` vocabulary
//! ([`super::printing::write_block`] writes it, [`super::printing::parse_key`]
//! reads it), so one parser owns both. The default folder, `print-presets` in
//! the settings folder, is always searched first and is where new presets are
//! saved; `print_preset_folder` (repeated, in search order) names the extra
//! folders. A name found in two folders lists once, from the first.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/printpresets.md`.

use std::path::{Path, PathBuf};

use super::PrintPrefs;
use super::printing::KeyOutcome;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const FOLDER_KEY: &str = "print_preset_folder";
/// The extension of a preset file, without its dot.
pub const EXTENSION: &str = "pdfcerprint"; // ui-text-exempt: a file extension, never displayed
/// The default folder's name inside the settings folder.
const DEFAULT_FOLDER: &str = "print-presets"; // ui-text-exempt: a folder name on disk
/// Characters no Windows file name may hold.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
/// The longest name accepted, in characters.
pub const MAX_NAME: usize = 80;

/// The extra folders searched after the default one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrintPresetFolders {
    folders: Vec<PathBuf>,
}

impl PrintPresetFolders {
    /// The most extra folders kept.
    pub const MAX_FOLDERS: usize = 16;

    /// The extra folders, in search order.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// Append `folder`; `false` when it is already listed, is the default
    /// folder, or the list is full.
    pub fn add_folder(&mut self, folder: &Path) -> bool {
        let listed = self.folders.iter().any(|f| f == folder)
            || default_folder().is_some_and(|d| d == folder);
        if listed || self.folders.len() >= Self::MAX_FOLDERS {
            return false;
        }
        self.folders.push(folder.to_path_buf());
        true
    }

    /// Remove the extra folder at `index`, if there is one.
    pub fn remove_folder(&mut self, index: usize) {
        if index < self.folders.len() {
            self.folders.remove(index);
        }
    }

    /// Every folder searched, the default first.
    #[must_use]
    pub fn search_list(&self) -> Vec<PathBuf> {
        default_folder()
            .into_iter()
            .chain(self.folders.iter().cloned())
            .collect()
    }
}

/// The default folder, `print-presets` in the settings folder; `None` when the
/// program has no settings folder.
#[must_use]
pub fn default_folder() -> Option<PathBuf> {
    pdfcer_core::settings::resolve_store()
        .directory()
        .map(|dir| dir.join(DEFAULT_FOLDER))
}

/// One preset found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetFile {
    /// The file's stem, which is the name the dropdown shows.
    pub name: String,
    pub path: PathBuf,
}

/// Every preset in `folders`, sorted by name without case; a name repeated in a
/// later folder is dropped. A folder that cannot be read contributes nothing.
#[must_use]
pub fn discover(folders: &[PathBuf]) -> Vec<PresetFile> {
    let mut found: Vec<PresetFile> = Vec::new();
    for folder in folders {
        let Ok(entries) = std::fs::read_dir(folder) else {
            continue;
        };
        let mut here: Vec<PresetFile> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .is_some_and(|x| x.eq_ignore_ascii_case(EXTENSION))
            })
            .filter_map(|path| {
                let name = path.file_stem()?.to_string_lossy().into_owned();
                Some(PresetFile { name, path })
            })
            .collect();
        here.retain(|p| !found.iter().any(|f| f.name.eq_ignore_ascii_case(&p.name)));
        found.extend(here);
    }
    found.sort_by_key(|p| p.name.to_lowercase());
    found
}

/// Why a name cannot be a preset's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRefusal {
    Empty,
    TooLong,
    /// A character a file name cannot hold.
    Forbidden(char),
    /// Ends in a dot or a space, which Windows strips.
    BadEnd,
}

/// The name, trimmed, if a preset may be saved under it.
pub fn check_name(name: &str) -> Result<&str, NameRefusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameRefusal::Empty);
    }
    if name.chars().count() > MAX_NAME {
        return Err(NameRefusal::TooLong);
    }
    if let Some(c) = name
        .chars()
        .find(|c| FORBIDDEN.contains(c) || c.is_control())
    {
        return Err(NameRefusal::Forbidden(c));
    }
    if name.ends_with('.') {
        return Err(NameRefusal::BadEnd);
    }
    Ok(name)
}

/// Where a preset named `name` is saved in `folder`.
#[must_use]
pub fn path_in(folder: &Path, name: &str) -> PathBuf {
    folder.join(format!("{name}.{EXTENSION}"))
}

/// The text of a preset file holding `prefs`.
#[must_use]
pub fn to_text(prefs: &PrintPrefs) -> String {
    let mut out = String::from(
        "# A pdfcer Print preset. Choose it from the Preset list at the top of\n\
         # the Print window. Same keys as the Print block of preferences.txt.\n",
    );
    super::printing::write_block(prefs, &mut out);
    out
}

/// Write `prefs` as the preset `name` in `folder`, creating the folder.
pub fn save(folder: &Path, name: &str, prefs: &PrintPrefs) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let path = path_in(folder, name);
    std::fs::write(&path, to_text(prefs)).map_err(|e| e.to_string())?;
    Ok(path)
}

/// The settings `text` holds, over the defaults, and how many lines were not
/// understood.
#[must_use]
pub fn from_text(text: &str) -> (PrintPrefs, usize) {
    let mut prefs = PrintPrefs::default();
    let mut unread = 0;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            unread += 1;
            continue;
        };
        if super::printing::parse_key(&mut prefs, key.trim(), value.trim()) != KeyOutcome::Accepted
        {
            unread += 1;
        }
    }
    (prefs, unread)
}

/// Read the preset at `path`.
pub fn load(path: &Path) -> Result<(PrintPrefs, usize), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(from_text(&text))
}

/// Read one `key = value` line into [`PrintPresetFolders`], if it belongs here.
pub(super) fn parse_key(prefs: &mut PrintPresetFolders, key: &str, value: &str) -> KeyOutcome {
    match key {
        FOLDER_KEY if value.trim().is_empty() => KeyOutcome::BadValue,
        FOLDER_KEY => {
            prefs.add_folder(Path::new(value.trim()));
            KeyOutcome::Accepted
        }
        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &PrintPresetFolders, out: &mut String) {
    out.push_str(
        "\n\
         # Folders the Print window searches for saved presets (*.pdfcerprint),\n\
         # after the print-presets folder beside this file, where new presets\n\
         # are saved. Repeat the key for more than one; searched in order.\n",
    );
    for folder in &prefs.folders {
        // ui-text-exempt: a file KEY and its separator, never displayed.
        out.push_str("print_preset_folder = ");
        out.push_str(&folder.display().to_string());
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printspooler::{Duplex, ScaleMode};

    #[test]
    fn a_preset_round_trips_every_print_setting() {
        let mut prefs = PrintPrefs {
            printer: Some("Office Laser".to_owned()),
            copies: 3,
            duplex: Duplex::LongEdge,
            scale: ScaleMode::ActualSize,
            ..PrintPrefs::default()
        };
        prefs.poster.on = true;
        prefs.colour.simulate_paper = true;
        prefs.lines.fixed = true;
        let (back, unread) = from_text(&to_text(&prefs));
        assert_eq!(unread, 0);
        assert_eq!(back, prefs);
    }

    #[test]
    fn unknown_lines_are_counted_and_the_rest_read() {
        let (back, unread) = from_text("print_copies = 4\nnonsense\nprint_wobble = 1\n");
        assert_eq!(back.copies, 4);
        assert_eq!(unread, 2);
    }

    #[test]
    fn names_a_file_cannot_hold_are_refused() {
        assert_eq!(check_name("  A3 drawings "), Ok("A3 drawings"));
        assert_eq!(check_name("   "), Err(NameRefusal::Empty));
        assert_eq!(check_name("a/b"), Err(NameRefusal::Forbidden('/')));
        assert_eq!(check_name("draft."), Err(NameRefusal::BadEnd));
        assert_eq!(
            check_name(&"x".repeat(MAX_NAME + 1)),
            Err(NameRefusal::TooLong)
        );
    }

    #[test]
    fn discovery_lists_a_repeated_name_once_from_the_first_folder() {
        let root = std::env::temp_dir().join(format!("pdfcer-presets-{}", std::process::id()));
        let (a, b) = (root.join("a"), root.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(path_in(&a, "Plans"), "").unwrap();
        std::fs::write(path_in(&b, "plans"), "").unwrap();
        std::fs::write(path_in(&b, "Booklet"), "").unwrap();
        std::fs::write(b.join("notes.txt"), "").unwrap();
        let found = discover(&[a.clone(), b]);
        let _ = std::fs::remove_dir_all(&root);
        let names: Vec<&str> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Booklet", "Plans"]);
        assert!(found[1].path.starts_with(&a));
    }

    #[test]
    fn the_folder_list_keeps_order_and_refuses_repeats() {
        let mut folders = PrintPresetFolders::default();
        assert!(folders.add_folder(Path::new("X:/one")));
        assert!(folders.add_folder(Path::new("X:/two")));
        assert!(!folders.add_folder(Path::new("X:/one")));
        let mut out = String::new();
        write_block(&folders, &mut out);
        let mut back = PrintPresetFolders::default();
        for line in out.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let (k, v) = line.split_once('=').unwrap();
            assert_eq!(parse_key(&mut back, k.trim(), v), KeyOutcome::Accepted);
        }
        assert_eq!(back, folders);
    }
}
