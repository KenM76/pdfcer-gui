//! # `stamps::folder` — where Acrobat looks for the operator's own stamps
//!
//! One question, and it is the difference between a feature and a chore: after
//! pdfcer writes a stamp collection, **does the operator have to go and put it
//! somewhere?** If yes, the feature is "pdfcer can write a PDF", which he
//! already had. If pdfcer suggests the folder Acrobat scans, his stamps appear
//! in Acrobat's own menu the next time he starts it, which is what he asked
//! for in `OPERATOR_REQUESTS.md` **O169**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/stamps/folder.md`.

use std::path::{Path, PathBuf};

/// The directory name Acrobat scans for user stamp collections.
///
/// `// ui-text-exempt:` — a **filesystem name owned by Acrobat**, matched and
/// written on disk, never shown as a label. Translating it would point pdfcer
/// at a folder that does not exist.
const STAMPS_DIR: &str = "Stamps"; // ui-text-exempt: an Acrobat filesystem name, never displayed.

/// The vendor path under `%APPDATA%`, as its two segments.
///
/// Same exemption and same reason as [`STAMPS_DIR`].
const VENDOR: [&str; 2] = ["Adobe", "Acrobat"]; // ui-text-exempt: filesystem names, never displayed.

/// Folder names under `%APPDATA%\Adobe\Acrobat\` that are **not** an Acrobat
/// generation, measured on the operator's machine.
///
/// A deny-list rather than an allow-list of known generations, because a new
/// generation name is a thing that will happen and a new sibling utility
/// folder is a thing that already has. Getting the deny-list wrong costs a
/// wrong suggestion in a picker the operator is looking at; getting an
/// allow-list wrong costs the feature entirely on a future Acrobat.
const NOT_A_GENERATION: [&str; 3] = ["Privileged", "TypeQuest", "Preflight"];

/// The operator's Acrobat stamps folder, if one can be identified.
///
/// Returns the folder whether or not it exists — an operator who has never
/// made a stamp in Acrobat has no `Stamps` directory, and suggesting the path
/// it *would* have is more useful than suggesting nothing. The caller's
/// picker creates it if he accepts.
#[must_use]
pub fn user_stamps_dir() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    let root = PathBuf::from(appdata).join(VENDOR[0]).join(VENDOR[1]);
    let generation = pick_generation(&read_generations(&root))?;
    Some(root.join(generation).join(STAMPS_DIR))
}

/// The generation folder names present under `…\Adobe\Acrobat\`.
///
/// Separated from [`pick_generation`] so the choosing rule can be tested
/// against the exact list measured on this machine without a filesystem.
fn read_generations(root: &Path) -> Vec<(String, bool)> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_owned();
            let has_stamps = e.path().join(STAMPS_DIR).is_dir();
            Some((name, has_stamps))
        })
        .collect()
}

/// Choose the generation folder, given `(name, already_has_a_Stamps_dir)`.
///
/// The rule, in order:
///
/// 1. **A folder that already holds `Stamps` wins outright.** Acrobat made it,
///    which means the operator has made a stamp with that generation.
/// 2. Otherwise, the shortest name that is not on [`NOT_A_GENERATION`].
///    Generation keys are short (`DC`, `11.0`, `2020`); the sibling utility
///    folders are long descriptive phrases. Shortest-wins is a heuristic and
///    is admitted as one — it only ever decides a *suggestion* in a picker the
///    operator can overrule.
///
/// ⚠ Ties are broken by name order so the answer is deterministic. A
/// suggestion that differs between two runs on an unchanged machine is a
/// defect even when both answers are defensible.
fn pick_generation(found: &[(String, bool)]) -> Option<String> {
    let plausible = |name: &String| {
        !NOT_A_GENERATION
            .iter()
            .any(|bad| name.starts_with(bad) || name.contains(bad))
    };

    if let Some((name, _)) = found
        .iter()
        .filter(|(n, has)| *has && plausible(n))
        .min_by(|a, b| a.0.cmp(&b.0))
    {
        return Some(name.clone());
    }

    found
        .iter()
        .filter(|(n, _)| plausible(n))
        .min_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)))
        .map(|(name, _)| name.clone())
}

/// A filename for a collection whose category is `category`.
///
/// Acrobat writes an opaque key here and reads the category from `/Info`
/// `/Title`, so the filename is ours to choose and a readable one is strictly
/// better for anyone who ever opens that folder.
///
/// **`set_file_name`, never `set_extension`** is the rule at the picker; the
/// same hazard applies to building the string. A category of `Rev. 2` would
/// have `set_extension` replace `2` and produce `Rev..pdf`, so the extension
/// is concatenated, never substituted. The finding is
/// `app/actions/export.rs`'s and is repeated here because the next person to
/// write a suggested filename will not have read that one.
#[must_use]
pub fn suggested_file_name(category: &str) -> String {
    let cleaned: String = category
        .chars()
        .map(|c| if is_path_hostile(c) { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    let stem = if trimmed.is_empty() {
        crate::text::stamps::default_category_file_stem()
    } else {
        trimmed
    };
    format!("{stem}.pdf")
}

/// Whether a character cannot appear in a Windows filename.
///
/// The set is §"Naming Files, Paths, and Namespaces"'s reserved list. A
/// category name is free text an operator typed and `Approved / Rejected` is
/// an entirely reasonable thing to type.
const fn is_path_hostile(c: char) -> bool {
    matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20
}
