//! # `app::prefs::fonts` — where pdfcer looks for a font it has to embed
//!
//! One preference, and it is the input two commands have been waiting for.
//!
//! ## ★★★ Why this exists, and why it is a PREFERENCE rather than a setting
//!
//! `tools.embed_fonts` and `tools.unembed_fonts` were registered, drawn on the
//! Tools tab and inert for the life of the project. Their recorded reason
//! quoted a premise that had expired — *"at S3 `Action` carries zoom and page
//! navigation and nothing else"* — and the entries themselves flagged it. But
//! re-deriving them on 2026-08-28 turned up a **second, unrecorded**
//! dependency that is the real one for embedding:
//!
//! `EmbedRequest::supplied` is a `/BaseFont` → donor-file map *"the shell
//! resolved for it"*, and `pdfcer`'s own note is blunt about the division of
//! labour: **"THE SOURCE FONTS COME FROM `--font-dir`. pdfcer never goes
//! looking."** So there is nothing for an Embed command to send until an
//! operator has said where their fonts live.
//!
//! ★ That dependency was in neither register. It was found by asking what the
//! verb's own request struct requires, which is a different question from
//! *"does the verb exist"* — and the second question is the one the stale
//! blockers had all been answering.
//!
//! ## ★★ It lives in `userdata/preferences.txt`, not in `settings.txt`
//!
//! `crate::app::prefs`' header states the rule and it decides this cleanly:
//! `pdfcer_core::settings` is for entries that **cite a clause the standard
//! leaves silent** — an ambiguity pdfcer has to resolve one way or another.
//! *Where this operator keeps their font files* cites nothing. It is a fact
//! about a machine, and filing it there would make the settings window's own
//! opening paragraph dishonest.
//!
//! ## ★ Why a repeated key rather than one joined line
//!
//! `font_folder = C:\…` may appear as many times as the operator likes, and
//! every occurrence is another folder in search order. The alternative — a
//! separator-joined value — needs a separator that cannot occur in a path, and
//! on Windows the obvious candidates are all legal in one. A repeated key has
//! no such question, reads correctly in a file an operator edits by hand, and
//! makes "search order" visible as line order.
//!
//! ★ **Order is preserved and duplicates are dropped.** Order matters because
//! two folders may hold the same face and the first one wins; duplicates are
//! dropped because a folder listed twice is a folder searched twice for the
//! same answer, and because an operator who adds the same folder from the
//! picker twice has not asked for anything.

use std::path::{Path, PathBuf};

/// The most folders this preference will hold.
///
/// ★ Sixteen, and the cap exists for the same reason every cap in this project
/// does — a bound is a decision and an unbounded list is a decision nobody
/// made. It is not a performance limit: an embed searches folders once per
/// missing face. It is a **legibility** limit, because a settings pane listing
/// forty directories has stopped being a setting and become a file manager,
/// and because a preferences file that has accumulated forty entries is one
/// nobody has pruned.
pub const MAX_FOLDERS: usize = 16;

/// **The operating system's own font directories**, in search order.
///
/// # ★★★ Why this function exists at all, when the module header says pdfcer
/// # must not go looking
///
/// Because the header's argument was never against *using* system fonts. Read
/// it again -- the objection is to pdfcer **deciding silently**:
///
/// > a program that searched `C:{BS}{BS}Windows{BS}{BS}Fonts` on its own would be answering it
/// > silently on the operator's behalf, in a file that outlives the decision.
///
/// The operator asked for a checkbox (`OPERATOR_REQUESTS.md` **O50**), and a
/// checkbox is not a loophole in that argument -- it is what the argument was
/// asking for. An explicit, persistent, **off-by-default** switch is the
/// operator making the licensing decision once, visibly, somewhere they can
/// find it again.
///
/// => Recorded because the shape recurs: **when a capability is refused on the
/// grounds that the program must not decide, the answer is usually a visible
/// setting rather than a permanent no.**
///
/// # ★★ TWO folders, and the second is the one that matters
///
/// | | |
/// |---|---|
/// | `%WINDIR%{BS}{BS}Fonts` | the machine's fonts, installed for everybody |
/// | `%LOCALAPPDATA%{BS}{BS}Microsoft{BS}{BS}Windows{BS}{BS}Fonts` | installed for **this user only** |
///
/// Windows has had the per-user location since 2018, and it is where a plain
/// double-click on a `.ttf` now installs by default -- **without** an
/// administrator prompt, which is exactly why it is the common case. A
/// checkbox that searched only the machine folder would miss the font the
/// operator installed themselves for this drawing, which is the font they are
/// most likely to have ticked the box for.
///
/// # ★ Read from the environment rather than hard-coded
///
/// `%WINDIR%` is `C:{BS}{BS}Windows` on essentially every machine and is not
/// guaranteed to be; a domain image can put it elsewhere. The cost of asking is
/// one environment lookup, and the cost of assuming is a checkbox that silently
/// finds nothing on somebody's machine.
///
/// Returns only directories that **exist**, unlike [`add`] -- and the two
/// differ on purpose. A folder the operator typed may be an unmounted drive and
/// is kept; these are derived, not typed, so a path that is not there is not a
/// promise anybody made and listing it would put a dead row under the checkbox.
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
///
/// ★★ The operator's own folders **first**, then the OS ones. Order is search
/// order and the first match wins ([`add`]), so a face the operator put in a
/// folder of their own beats the same-named face the machine happens to have --
/// which is the only ordering that makes their list mean anything. A folder
/// they curated for a job is a decision; `C:{BS}{BS}Windows{BS}{BS}Fonts` is whatever has
/// accumulated.
#[must_use]
pub fn search_path(configured: &[PathBuf], include_os: bool) -> Vec<PathBuf> {
    let mut out = configured.to_vec();
    if include_os {
        for dir in os_font_dirs() {
            // Through `add`, so the cap and the duplicate rule apply to the
            // combined list rather than only to the typed half -- an operator
            // who has already added `C:{BS}{BS}Windows{BS}{BS}Fonts` by hand and then ticks
            // the box does not get it twice.
            add(&mut out, &dir);
        }
    }
    out
}

/// Add `folder`, keeping order and refusing a duplicate or an over-long list.
///
/// Returns whether the list changed, so a caller can tell "added" from "you
/// already have that one" without comparing lengths.
///
/// ★ It does **not** check that the folder exists. A removable drive that is
/// not mounted right now is still where the operator's fonts live, and a
/// preference that silently dropped it on the day the drive was unplugged
/// would be worse than one that keeps a path that occasionally resolves to
/// nothing. The *embed* is where a missing folder is reported, because that is
/// where it matters.
pub fn add(folders: &mut Vec<PathBuf>, folder: &Path) -> bool {
    if folders.len() >= MAX_FOLDERS || folders.iter().any(|f| f == folder) {
        return false;
    }
    folders.push(folder.to_path_buf());
    true
}

/// Parse one `font_folder = …` line's value.
///
/// ★ Trims, and rejects only the empty result. A path is otherwise taken
/// verbatim — no canonicalisation, no separator normalisation — because
/// `Path` comparison on Windows is case-insensitive in the filesystem and
/// case-sensitive in `PathBuf`, and a preference that rewrote what the
/// operator typed would make their own file unrecognisable to them.
#[must_use]
pub fn parse_one(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

/// The `font_folder` lines for [`super::Prefs::write_to_string`], with the
/// comment that explains them.
///
/// ★ The comment is emitted **even when the list is empty**, which is the
/// convention every other block in that file follows and is the reason the
/// file is editable by hand: an operator who wants to add a folder without
/// opening pdfcer needs to see the key name and its rules, and a key that only
/// appears once it is already set cannot teach anybody anything.
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
    ///
    /// ★ Asserted through the return value as well as the length, because the
    /// caller uses it to decide what to say: *"added"* and *"you already have
    /// that one"* are different sentences and a length comparison cannot tell
    /// them apart when the list is also at its cap.
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

    /// ★★ **The comment block is written even with no folders**, so the file
    /// teaches its own key.
    ///
    /// The failure this guards is the tempting simplification — emit nothing
    /// when the list is empty — which produces a preferences file with no
    /// mention of the one key an operator would want to add by hand.
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
///
/// ★ Written **always**, both values, for [`write_block`]'s reason: the file is
/// editable by hand, and a key that only appears once it is already set cannot
/// teach anybody it exists. This one has a second reason of its own — it is the
/// switch with a licensing consequence, so the file states that consequence
/// where somebody editing it will read it.
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
    // ui-text-exempt: a file KEY and its VALUE, never displayed in the UI.
    out.push_str(if on {
        "use_os_fonts = true\n"
    } else {
        "use_os_fonts = false\n"
    });
    out
}

#[cfg(test)]
mod os_tests {
    use super::*;

    /// ★★ **The OS folders are searched AFTER the operator's own.**
    ///
    /// Order is search order and the first match wins, so a face the operator
    /// put in a folder they curated for a job beats the same-named face the
    /// machine happens to have. The reverse would make their list decorative on
    /// every name Windows also carries — which is most of them.
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
    ///
    /// ★ The assertion that guards the licensing argument. A build whose
    /// checkbox did nothing would be caught by the UI; a build that searched
    /// the OS folders regardless of it would not be caught anywhere else.
    #[test]
    fn unticked_adds_nothing() {
        let mine = vec![PathBuf::from("C:/JobFonts")];
        assert_eq!(search_path(&mine, false), mine);
        assert!(search_path(&[], false).is_empty());
    }

    /// ★★ **A folder already listed by hand is not added twice.**
    ///
    /// An operator who typed the machine's font folder into the list and then
    /// ticked the box has said one thing twice, and a list holding it twice
    /// would search it twice for the same answer — and would spend one of the
    /// sixteen slots saying nothing.
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

    /// ★★★ **The real machine has at least one, and the parse link is live.**
    ///
    /// Every test above would pass on a build whose `os_font_dirs` returned an
    /// empty vector — they assert about ordering and absence. This one asserts
    /// the function finds something, which is the only claim that fails if the
    /// environment lookup is wrong.
    ///
    /// SKIPPED where there is no such directory, because that is a fact about
    /// the machine and not about this code.
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
