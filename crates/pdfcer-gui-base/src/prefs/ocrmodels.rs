//! # `prefs::ocrmodels` — where Recognise text looks for models, and which it used
//!
//! Three keys. `ocr_folder` (repeated, in search order, the engine's own
//! spelling) names a folder searched after the bundled `models` folder;
//! `ocr_model` names the model last run, by its discovery name;
//! `ocr_program_addons = refuse` stops model folders from running a program
//! (Tesseract), and is written only when set, since allowing is the default.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/ocrmodels.md`.

use std::path::{Path, PathBuf};

use super::printing::KeyOutcome;

// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const FOLDER_KEY: &str = "ocr_folder";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const MODEL_KEY: &str = "ocr_model";
// ui-text-exempt: a file KEY, written into preferences.txt and parsed back.
const PROGRAMS_KEY: &str = "ocr_program_addons";
// ui-text-exempt: a file VALUE of PROGRAMS_KEY.
const PROGRAMS_REFUSE: &str = "refuse";
// ui-text-exempt: a file VALUE of PROGRAMS_KEY.
const PROGRAMS_ALLOW: &str = "allow";

/// The extra model folders and the remembered model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OcrModelPrefs {
    folders: Vec<PathBuf>,
    /// The discovery name of the model last run; `None` before the first run.
    pub model: Option<String>,
    /// Whether model folders that run a separate program are refused.
    pub refuse_programs: bool,
}

impl OcrModelPrefs {
    /// The most extra folders kept.
    pub const MAX_FOLDERS: usize = 16;

    /// The extra folders, in search order.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// Append `folder`; `false` when it is already listed or the list is full.
    pub fn add_folder(&mut self, folder: &Path) -> bool {
        if self.folders.len() >= Self::MAX_FOLDERS || self.folders.iter().any(|f| f == folder) {
            return false;
        }
        self.folders.push(folder.to_path_buf());
        true
    }

    /// Remove the folder at `index`, if there is one.
    pub fn remove_folder(&mut self, index: usize) {
        if index < self.folders.len() {
            self.folders.remove(index);
        }
    }
}

/// Read one `key = value` line into [`OcrModelPrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut OcrModelPrefs, key: &str, value: &str) -> KeyOutcome {
    let value = value.trim();
    match key {
        FOLDER_KEY | MODEL_KEY if value.is_empty() => KeyOutcome::BadValue,
        FOLDER_KEY => {
            prefs.add_folder(Path::new(value));
            KeyOutcome::Accepted
        }
        MODEL_KEY => {
            prefs.model = Some(value.to_owned());
            KeyOutcome::Accepted
        }
        PROGRAMS_KEY => match value {
            PROGRAMS_REFUSE => {
                prefs.refuse_programs = true;
                KeyOutcome::Accepted
            }
            PROGRAMS_ALLOW => {
                prefs.refuse_programs = false;
                KeyOutcome::Accepted
            }
            _ => KeyOutcome::BadValue,
        },
        _ => KeyOutcome::NotMine,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &OcrModelPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # Folders Recognise text searches for OCR models, after the models\n\
         # folder beside pdfcer-gui.exe. Repeat the key for more than one; they\n\
         # are searched in the order they appear here. Up to 16.\n\
         # ocr_model: the model Recognise text last ran, by its name.\n\
         # ocr_program_addons: allow or refuse model folders that run a\n\
         # separate program, such as Tesseract. Allowed when absent.\n\
         ",
    );
    for folder in &prefs.folders {
        // ui-text-exempt: a file KEY and its separator, never displayed.
        out.push_str("ocr_folder = ");
        out.push_str(&folder.display().to_string());
        out.push('\n');
    }
    if let Some(model) = &prefs.model {
        // ui-text-exempt: a file KEY and its separator, never displayed.
        out.push_str("ocr_model = ");
        out.push_str(model);
        out.push('\n');
    }
    if prefs.refuse_programs {
        // ui-text-exempt: a file KEY, separator and value, never displayed.
        out.push_str("ocr_program_addons = refuse\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_back(text: &str) -> OcrModelPrefs {
        let mut prefs = OcrModelPrefs::default();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            if let Some((key, value)) = line.split_once('=') {
                assert_eq!(
                    parse_key(&mut prefs, key.trim(), value),
                    KeyOutcome::Accepted
                );
            }
        }
        prefs
    }

    #[test]
    fn folders_and_the_model_round_trip_in_order() {
        let mut original = OcrModelPrefs::default();
        assert!(original.add_folder(Path::new("E:/models b")));
        assert!(original.add_folder(Path::new("E:/a")));
        original.model = Some("paddle-vl".to_owned());
        original.refuse_programs = true;
        let mut out = String::new();
        write_block(&original, &mut out);
        assert_eq!(read_back(&out), original);
    }

    #[test]
    fn a_folder_is_listed_once_and_the_list_is_capped() {
        let mut prefs = OcrModelPrefs::default();
        assert!(prefs.add_folder(Path::new("E:/a")));
        assert!(!prefs.add_folder(Path::new("E:/a")));
        for i in 1..OcrModelPrefs::MAX_FOLDERS {
            assert!(prefs.add_folder(&PathBuf::from(format!("E:/{i}"))));
        }
        assert!(!prefs.add_folder(Path::new("E:/one-too-many")));
        prefs.remove_folder(0);
        prefs.remove_folder(99);
        assert_eq!(prefs.folders().len(), OcrModelPrefs::MAX_FOLDERS - 1);
    }

    #[test]
    fn an_empty_value_is_ours_and_changes_nothing() {
        let mut prefs = OcrModelPrefs::default();
        assert_eq!(parse_key(&mut prefs, FOLDER_KEY, " "), KeyOutcome::BadValue);
        assert_eq!(parse_key(&mut prefs, MODEL_KEY, ""), KeyOutcome::BadValue);
        assert_eq!(
            parse_key(&mut prefs, "font_folder", "E:/a"),
            KeyOutcome::NotMine
        );
        assert_eq!(prefs, OcrModelPrefs::default());
    }

    #[test]
    fn program_add_ons_are_allowed_unless_refused() {
        let mut prefs = OcrModelPrefs::default();
        assert!(!prefs.refuse_programs);
        let mut out = String::new();
        write_block(&prefs, &mut out);
        assert!(!out.contains("ocr_program_addons ="), "{out}");
        assert_eq!(
            parse_key(&mut prefs, PROGRAMS_KEY, "maybe"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, PROGRAMS_KEY, " refuse "),
            KeyOutcome::Accepted
        );
        assert!(prefs.refuse_programs);
        assert_eq!(
            parse_key(&mut prefs, PROGRAMS_KEY, PROGRAMS_ALLOW),
            KeyOutcome::Accepted
        );
        assert!(!prefs.refuse_programs);
    }
}
