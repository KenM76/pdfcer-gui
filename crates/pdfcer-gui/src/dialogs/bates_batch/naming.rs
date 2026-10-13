//! # `dialogs::bates_batch::naming` — where each numbered copy is written
//!
//! The four naming modes and the two folder modes, and the checks that keep a
//! run from writing over an original or two copies over each other. Pure: no
//! filesystem access, so the window can run it every frame.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/bates_batch.md`.

use std::path::{Path, PathBuf};

use crate::text::bates_batch as t;

/// How a copy is named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameMode {
    /// The original file name.
    Keep,
    /// `before` + the original stem + `after`.
    Affix,
    /// `first-last`, the file's own label range.
    Range,
    /// `first-last` and the original stem, the combination Acrobat lacks.
    RangeAndName,
}

impl NameMode {
    pub const ALL: [Self; 4] = [Self::Affix, Self::RangeAndName, Self::Range, Self::Keep];
}

/// Where the copies go and what they are called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub mode: NameMode,
    pub before: String,
    pub after: String,
    /// `None` writes each copy beside its original.
    pub folder: Option<PathBuf>,
    /// The operator chose "one folder"; `folder` may still be unpicked.
    pub one_folder: bool,
    pub replace: bool,
}

impl Default for Output {
    fn default() -> Self {
        Self {
            mode: NameMode::Affix,
            before: String::new(),
            after: "_Bates".to_owned(), // ui-text-exempt: a default file-name suffix the operator edits
            folder: None,
            one_folder: false,
            replace: false,
        }
    }
}

/// The characters Windows refuses in a file name.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// One file's labels, as the naming needs them.
pub struct Labels<'a> {
    pub first: &'a str,
    pub last: &'a str,
}

impl Output {
    /// The file name `source` is written as, given its label range.
    #[must_use]
    pub fn name(&self, source: &Path, labels: &Labels<'_>) -> String {
        let stem = source
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let range = if labels.first == labels.last {
            labels.first.to_owned()
        } else {
            format!("{}-{}", labels.first, labels.last)
        };
        match self.mode {
            NameMode::Keep => source
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
            NameMode::Affix => format!("{}{stem}{}.pdf", self.before, self.after),
            NameMode::Range => format!("{range}.pdf"),
            NameMode::RangeAndName => format!("{range} {stem}.pdf"), // ui-text-exempt: a file name
        }
    }

    /// The path `source` is written to.
    #[must_use]
    pub fn target(&self, source: &Path, labels: &Labels<'_>) -> Option<PathBuf> {
        let dir = if self.one_folder {
            self.folder.clone()?
        } else {
            source.parent()?.to_path_buf()
        };
        Some(dir.join(self.name(source, labels)))
    }

    /// Why this output cannot be written for `rows` — `(source, labels)` in
    /// batch order — or every row's target.
    ///
    /// # Errors
    ///
    /// The first problem, worded for the window.
    pub fn plan(&self, rows: &[(PathBuf, Labels<'_>)]) -> Result<Vec<PathBuf>, String> {
        if self.one_folder && self.folder.is_none() {
            return Err(t::no_folder().to_owned());
        }
        let ranged = matches!(self.mode, NameMode::Range | NameMode::RangeAndName);
        let affixed = matches!(self.mode, NameMode::Affix);
        let bad = |s: &str| s.contains(FORBIDDEN);
        if (ranged && rows.iter().any(|(_, l)| bad(l.first) || bad(l.last)))
            || (affixed && (bad(&self.before) || bad(&self.after)))
        {
            return Err(t::label_not_a_name().to_owned());
        }
        // Windows names: case-blind, and either separator.
        let key = |p: &Path| p.to_string_lossy().to_lowercase().replace('\\', "/");
        let sources: std::collections::HashSet<String> = rows.iter().map(|(s, _)| key(s)).collect();
        let mut seen = std::collections::HashSet::new();
        let mut targets = Vec::with_capacity(rows.len());
        for (source, labels) in rows {
            let target = self
                .target(source, labels)
                .ok_or_else(|| t::no_folder().to_owned())?;
            let name = target
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            if sources.contains(&key(&target)) {
                let original = source
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                return Err(t::over_original(&original));
            }
            if !seen.insert(key(&target)) {
                return Err(t::same_target(&name));
            }
            targets.push(target);
        }
        Ok(targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows<'a>(names: &[&str], labels: &'a [(&'a str, &'a str)]) -> Vec<(PathBuf, Labels<'a>)> {
        names
            .iter()
            .zip(labels)
            .map(|(n, (f, l))| {
                (
                    PathBuf::from(format!("C:/in/{n}")),
                    Labels { first: f, last: l },
                )
            })
            .collect()
    }

    #[test]
    fn each_mode_names_the_copy() {
        let l = Labels {
            first: "A001",
            last: "A004",
        };
        let p = Path::new("C:/in/plan.pdf");
        let mut o = Output::default();
        assert_eq!(o.name(p, &l), "plan_Bates.pdf");
        o.mode = NameMode::Range;
        assert_eq!(o.name(p, &l), "A001-A004.pdf");
        o.mode = NameMode::RangeAndName;
        assert_eq!(o.name(p, &l), "A001-A004 plan.pdf");
        o.mode = NameMode::Keep;
        assert_eq!(o.name(p, &l), "plan.pdf");
    }

    #[test]
    fn keeping_the_name_beside_the_original_is_refused() {
        let o = Output {
            mode: NameMode::Keep,
            ..Output::default()
        };
        let why = o.plan(&rows(&["a.pdf"], &[("1", "2")])).unwrap_err();
        assert!(why.contains("a.pdf") && why.contains("replace"), "{why}");
    }

    #[test]
    fn a_copy_named_like_another_original_is_refused() {
        // b's copy "a.pdf" would land on the original a.pdf.
        let o = Output {
            mode: NameMode::Affix,
            after: String::new(),
            before: String::new(),
            ..Output::default()
        };
        assert!(o.plan(&rows(&["a.pdf"], &[("1", "1")])).is_err());
    }

    #[test]
    fn two_copies_on_one_name_are_refused() {
        let o = Output {
            mode: NameMode::Range,
            ..Output::default()
        };
        let why = o
            .plan(&rows(&["a.pdf", "b.pdf"], &[("1", "1"), ("1", "1")]))
            .unwrap_err();
        assert!(why.contains("1.pdf"), "{why}");
    }

    #[test]
    fn a_label_with_a_slash_cannot_name_a_file() {
        let o = Output {
            mode: NameMode::Range,
            ..Output::default()
        };
        assert!(o.plan(&rows(&["a.pdf"], &[("X/1", "X/2")])).is_err());
        let affix = Output::default();
        assert!(affix.plan(&rows(&["a.pdf"], &[("X/1", "X/2")])).is_ok());
    }

    #[test]
    fn one_folder_needs_a_folder() {
        let mut o = Output {
            one_folder: true,
            ..Output::default()
        };
        assert!(o.plan(&rows(&["a.pdf"], &[("1", "1")])).is_err());
        o.folder = Some(PathBuf::from("C:/out"));
        let t = o.plan(&rows(&["a.pdf"], &[("1", "1")])).unwrap();
        assert_eq!(t, vec![PathBuf::from("C:/out").join("a_Bates.pdf")]);
    }
}
