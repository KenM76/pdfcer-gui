//! # `fontsearch` — where pdfcer looks for a font it has to embed
//!
//! One preference — an ordered list of folders — and it is the input
//! `tools.embed_fonts` cannot run without.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/fontsearch.md`.

use std::path::{Path, PathBuf};

/// The most folders this preference will hold.
pub const MAX_FOLDERS: usize = 16;

/// **The operating system's own font directories**, in search order.
#[must_use]
pub fn os_font_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut push = |path: PathBuf| {
        if path.is_dir() && !out.contains(&path) {
            out.push(path);
        }
    };
    // ui-text-exempt: environment variable names, never displayed.
    if let Ok(windir) = std::env::var("WINDIR") {
        // ui-text-exempt: a directory name, never displayed.
        push(PathBuf::from(windir).join("Fonts"));
    }
    // ui-text-exempt: environment variable name, never displayed.
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        // ui-text-exempt: directory names, never displayed.
        push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("Windows")
                .join("Fonts"),
        );
    }
    out
}

/// Every folder an embed may take a donor from, given the preference.
#[must_use]
pub fn search_path(configured: &[PathBuf], include_os: bool) -> Vec<PathBuf> {
    let mut out = configured.to_vec();
    if include_os {
        for dir in os_font_dirs() {
            // Through `add`, so the cap and the duplicate rule apply to the
            // combined list rather than only to the typed half -- an operator
            // who has already added `C:\Windows\Fonts` by hand and then ticks
            // the box does not get it twice.
            add(&mut out, &dir);
        }
    }
    out
}

/// Add `folder`, keeping order and refusing a duplicate or an over-long list.
pub fn add(folders: &mut Vec<PathBuf>, folder: &Path) -> bool {
    if folders.len() >= MAX_FOLDERS || folders.iter().any(|f| f == folder) {
        return false;
    }
    folders.push(folder.to_path_buf());
    true
}

/// Parse one `font_folder = …` line's value.
#[must_use]
pub fn parse_one(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

/// The `font_folder` lines for [`super::Prefs::write_to_string`], with the
/// comment that explains them.
#[must_use]
pub fn write_block(folders: &[PathBuf]) -> String {
    let mut out = String::from(
        "\n\
         # Folders pdfcer searches when it has to embed a font that a document\n\
         # names but does not carry. Repeat the key for more than one; they are\n\
         # searched in the order they appear here. Up to 16.\n\
         #\n\
         # pdfcer never goes looking on its own -- if this is empty, embedding\n\
         # has nowhere to take a font from.\n",
    );
    if folders.is_empty() {
        // ui-text-exempt: a file KEY inside a commented example line.
        out.push_str("# font_folder = C:\\Windows\\Fonts\n");
        return out;
    }
    for folder in folders {
        // ui-text-exempt: a file KEY, never displayed in the UI.
        out.push_str("font_folder = ");
        out.push_str(&folder.display().to_string());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A duplicate is refused and the list is unchanged.**
    #[test]
    fn a_duplicate_is_refused_and_says_so() {
        let mut folders = Vec::new();
        assert!(add(&mut folders, Path::new("C:/Fonts")));
        assert!(!add(&mut folders, Path::new("C:/Fonts")));
        assert_eq!(folders.len(), 1);
    }

    /// **Order is preserved**, because it is search order and the first match
    /// wins.
    #[test]
    fn order_is_search_order() {
        let mut folders = Vec::new();
        add(&mut folders, Path::new("C:/First"));
        add(&mut folders, Path::new("C:/Second"));
        assert_eq!(folders[0], PathBuf::from("C:/First"));
        assert_eq!(folders[1], PathBuf::from("C:/Second"));
    }

    /// **The cap holds**, and the seventeenth is refused rather than evicting
    /// the first — an operator who has hit the limit is told, not silently
    /// rearranged.
    #[test]
    fn the_cap_refuses_rather_than_evicting() {
        let mut folders = Vec::new();
        for i in 0..MAX_FOLDERS {
            assert!(add(&mut folders, &PathBuf::from(format!("C:/F{i}"))));
        }
        assert!(!add(&mut folders, Path::new("C:/OneMore")));
        assert_eq!(folders.len(), MAX_FOLDERS);
        assert_eq!(folders[0], PathBuf::from("C:/F0"), "the first survives");
    }

    /// **An empty or blank value is not a folder.**
    #[test]
    fn a_blank_value_is_not_a_path() {
        assert!(parse_one("").is_none());
        assert!(parse_one("   ").is_none());
        assert_eq!(parse_one("  C:/Fonts  "), Some(PathBuf::from("C:/Fonts")));
    }

    /// **The comment block is written even with no folders**, so the file
    /// teaches its own key.
    #[test]
    fn the_key_is_documented_even_when_unset() {
        let block = write_block(&[]);
        assert!(block.contains("font_folder"), "the key is named: {block}");
        assert!(
            block.contains("never goes looking"),
            "and the consequence of leaving it empty is stated: {block}"
        );
    }

    /// **A written list round-trips through the parser.**
    #[test]
    fn a_written_list_reads_back() {
        let folders = vec![PathBuf::from("C:/A"), PathBuf::from("D:/B")];
        let block = write_block(&folders);
        let read: Vec<PathBuf> = block
            .lines()
            .filter_map(|l| l.strip_prefix("font_folder = "))
            .filter_map(parse_one)
            .collect();
        assert_eq!(read, folders);
    }
}

/// The `use_os_fonts` line for [`super::Prefs::write_to_string`], with the
/// comment that explains it.
#[must_use]
pub fn write_os_flag(on: bool) -> String {
    let mut out = String::from(
        "\n\
         # Whether to search the fonts installed on this computer as well as\n\
         # the folders above: this machine's font folder and the one holding\n\
         # fonts installed for your user only.\n\
         #\n\
         # Off unless you turn it on. Embedding puts a font's outlines inside a\n\
         # document you may send to somebody else, and which font that is, is a\n\
         # licensing question -- so pdfcer does not answer it for you.\n",
    );
    out.push_str(if on {
        // ui-text-exempt: a file KEY and its VALUE, never displayed in the UI.
        "use_os_fonts = true\n"
    } else {
        // ui-text-exempt: a file KEY and its VALUE, never displayed in the UI.
        "use_os_fonts = false\n"
    });
    out
}

#[cfg(test)]
mod os_tests {
    use super::*;

    /// **The OS folders are searched AFTER the operator's own.**
    #[test]
    fn the_operators_own_folders_are_searched_first() {
        let mine = vec![PathBuf::from("C:/JobFonts")];
        let combined = search_path(&mine, true);
        assert_eq!(combined.first(), Some(&PathBuf::from("C:/JobFonts")));
        assert!(
            combined.len() > 1 || os_font_dirs().is_empty(),
            "the OS folders were not appended: {combined:?}"
        );
    }

    /// **Off means off.**
    #[test]
    fn unticked_adds_nothing() {
        let mine = vec![PathBuf::from("C:/JobFonts")];
        assert_eq!(search_path(&mine, false), mine);
        assert!(search_path(&[], false).is_empty());
    }

    /// **A folder already listed by hand is not added twice.**
    #[test]
    fn ticking_the_box_does_not_duplicate_a_hand_added_folder() {
        let Some(first) = os_font_dirs().first().cloned() else {
            eprintln!("no OS font directory on this machine — skipped");
            return;
        };
        let combined = search_path(std::slice::from_ref(&first), true);
        assert_eq!(
            combined.iter().filter(|p| **p == first).count(),
            1,
            "{combined:?}"
        );
    }

    /// **The real machine has at least one, and the parse link is live.**
    #[test]
    fn a_real_machine_reports_a_real_font_directory() {
        let dirs = os_font_dirs();
        if dirs.is_empty() {
            eprintln!("no OS font directory on this machine — skipped");
            return;
        }
        for dir in &dirs {
            assert!(dir.is_dir(), "{dir:?} was reported and does not exist");
        }
        eprintln!("OS font directories: {dirs:?}");
    }

    /// **The file teaches its own key, and states the consequence.**
    #[test]
    fn the_flag_is_documented_in_the_file() {
        for on in [true, false] {
            let block = write_os_flag(on);
            assert!(block.contains("use_os_fonts"), "{block}");
            assert!(block.contains("licensing"), "{block}");
        }
        assert!(write_os_flag(true).contains("= true"));
        assert!(write_os_flag(false).contains("= false"));
    }
}
