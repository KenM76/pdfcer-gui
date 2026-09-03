//! # `redact::sealed` — the call-site monopoly, read from the syntax tree
//!
//! [`super`] §2.4. One property, asserted over **every `.rs` file in this
//! crate**:
//!
//! > [`pdfcer_core::redact::apply_redactions`] is *called* in exactly one place
//! > — [`super::prepare_redaction_apply`] — and that place always proves its
//! > output before returning it.
//!
//! ## Why this exists at all, when the bytes are already private
//!
//! Because the two mechanisms fail in opposite directions and neither covers
//! the other.
//!
//! [`super::PreparedRedaction`]'s private `bytes` field stops anybody
//! **exfiltrating the proven buffer**. It says nothing about a module that
//! bypasses the type entirely: `let (bytes, _report) =
//! pdfcer_core::redact::apply_redactions(&doc, &opts)?; std::fs::write(p,
//! &bytes)?;` is four lines, needs nothing from this module, and produces
//! exactly the artefact `Pass 72.0` warns about — *"a shell calling
//! `redact::apply_redactions` directly and writing the bytes ships an
//! unverified redaction and will not know."*
//!
//! It is not a hypothetical. `pdfcer`'s `redact-apply` does precisely that
//! at the engine's HEAD and exits `SUCCESS` on a file it never verified. The
//! failure has a worked example living in the same repository, written by
//! people who knew about the proof.
//!
//! ## ★ Why the syntax tree rather than a grep, and rather than a gate script
//!
//! `crate::shell::commands::reach`'s header makes the general argument at
//! length; this is the same one aimed at a narrower question, and the specific
//! false pass is easy to name. **[`super`]'s own module documentation contains
//! the identifier `apply_redactions` seven times**, in prose, explaining why it
//! must not be called twice. A text scan counts eight call sites in a crate
//! that has one — and, worse, the same scan run against a build where the real
//! call had been *moved* would still find seven and report the monopoly intact.
//! Comments are not in the tree.
//!
//! It is a **test** rather than a `tools/gates/` script for
//! `reach.rs`'s reason: what a gate script contributes over a test is a
//! precondition guarantee, and this has a stronger one than a script can offer.
//! [`CRATE_SRC`] is built from `CARGO_MANIFEST_DIR`, a compile-time constant, so
//! the directory is the one this crate is compiled from and not a path somebody
//! typed; and the sweep **fails closed** on two independent counts (below).
//!
//! ## Failing closed, twice
//!
//! `run-all.sh`'s three-state model exists because *"found nothing"* and
//! *"looked at nothing"* print the same thing. Both are closed here:
//!
//! 1. **A sweep that reads implausibly few files fails**, so a walker that
//!    silently stopped at the first directory cannot report a clean monopoly
//!    over the one file it managed to read.
//! 2. **A sweep that finds ZERO call sites fails**, and that is the more
//!    interesting of the two. Zero would mean the proof pipeline no longer calls
//!    the engine at all — which is either a rename this check has not been told
//!    about, or a redaction feature that has quietly stopped redacting. Reading
//!    zero as "the monopoly holds" is the exact shape of the vacuous pass this
//!    project has now shipped twice.
//!
//! ## What it does not claim
//!
//! It is scoped to **this crate**. Another crate in the workspace could call
//! the engine directly and this would not see it — which is not a gap so much
//! as a boundary: `tools/ui-verify` drives the binary and `egui-shell` is
//! forbidden from knowing what a PDF is (`check-shell-purity.sh`), so the only
//! other Rust in this workspace that could reach `pdfcer-core` is a harness that
//! does not ship. See [`super`] §2.5.

use std::path::{Path, PathBuf};

/// The identifier whose call sites are counted.
///
/// The **last path segment**, not the whole path, because the path is a
/// spelling decision (`pdfcer_core::redact::apply_redactions` today, a `use`
/// away from a bare `apply_redactions` tomorrow) and the function is the fact.
/// The same reduction `reach.rs` makes for its guard functions, for the same
/// reason.
// ui-text-exempt: a Rust function name, matched against the parsed syntax tree.
const SUBJECT: &str = "apply_redactions";

/// The engine verb this module's own files must **never** call.
///
/// [`super`] §1.1: an incremental save leaves the un-redacted content in the
/// previous revision, so the apply pipeline has no legitimate use for it and a
/// call would be a leak with a plausible-looking diff. The rest of this shell
/// uses it constantly — it is what `crate::app::save` is built on and what
/// `file.save_copy`'s tooltip promises — so this is not a crate-wide ban but a
/// statement about one directory.
// ui-text-exempt: a Rust function name, matched against the parsed syntax tree.
const FORBIDDEN_IN_REDACT: &str = "to_incremental_bytes";

/// The crate's source root, resolved at **compile** time.
///
/// `CARGO_MANIFEST_DIR` rather than a relative path from the working directory:
/// `cargo test` and a test run from an IDE do not agree about the latter, and a
/// path that resolved to nothing would be the "looked at nothing" failure this
/// module's header is about.
fn crate_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The one file permitted to call [`SUBJECT`], relative to the source root.
///
/// Compared as a path suffix rather than as a string, so the separator is the
/// platform's rather than this constant's — the same file is `redact\mod.rs` on
/// the machine this is built on and `redact/mod.rs` elsewhere.
const OWNER: [&str; 2] = [
    // ui-text-exempt: a directory name inside this crate, never displayed.
    "redact", // ui-text-exempt: a file name inside this crate, never displayed.
    "mod.rs",
];

/// What one sweep of a directory tree established.
#[derive(Debug, Default)]
pub(super) struct Sweep {
    /// How many `.rs` files were parsed.
    ///
    /// Reported so the "read nothing" state is visible rather than
    /// indistinguishable from a clean result — see the module header.
    pub(super) files_read: usize,
    /// Every file that calls [`SUBJECT`], with how many times.
    pub(super) call_sites: Vec<(PathBuf, usize)>,
}

/// **Count the calls to `subject` in one file's source.**
///
/// Split from [`sweep`] so the *rule* is testable against a fixture rather than
/// only against the real tree — `crate::diag::record_if_changed`'s shape, and
/// the same one `reach::read_arms` takes: a reader that can only be pointed at
/// the real file cannot be shown to bite.
///
/// Both a free call (`path::to::subject(..)`) and a method call
/// (`receiver.subject(..)`) count. The engine's is a free function, so only the
/// first can occur today; the second is counted because a future engine that
/// moved it onto a type would otherwise slip the monopoly silently, and because
/// counting one shape and not the other is the kind of narrowness that makes a
/// check answer a question nobody asked.
///
/// # Errors
///
/// The source did not parse as Rust. **Fails closed**: an unreadable file
/// stops the sweep rather than contributing a reassuring zero.
pub(super) fn calls_in(src: &str, subject: &str) -> Result<usize, String> {
    use syn::visit::Visit;

    /// Counts call expressions anywhere in a file.
    ///
    /// `syn::visit::Visit` rather than a hand-written recursion: "anywhere" has
    /// to include a closure body, a nested `fn`, a `match` arm, an `if let`
    /// scrutinee and every other place an expression can hide, and the variant
    /// a hand-written walker forgot would be a silent hole rather than a
    /// compile error.
    struct Counter<'a> {
        subject: &'a str,
        found: usize,
    }

    impl<'ast> Visit<'ast> for Counter<'_> {
        fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
            if let syn::Expr::Path(path) = &*node.func
                && path
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == self.subject)
            {
                self.found += 1;
            }
            syn::visit::visit_expr_call(self, node);
        }

        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if node.method == self.subject {
                self.found += 1;
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let file = syn::parse_file(src).map_err(|e| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("the source does not parse as Rust: {e}")
    })?;
    let mut counter = Counter { subject, found: 0 };
    counter.visit_file(&file);
    Ok(counter.found)
}

/// **Walk `root` and count `subject`'s call sites in every `.rs` file under
/// it.**
///
/// `root` is a parameter rather than a reach for [`crate_src`] for
/// [`calls_in`]'s reason, one level up: the self-test below points it at a
/// temporary tree containing a planted violation, and a sweep that could only
/// be aimed at the real crate could not be shown to report one.
///
/// # Errors
///
/// The directory could not be read, or a file in it did not parse. Both fail
/// closed.
pub(super) fn sweep(root: &Path, subject: &str) -> Result<Sweep, String> {
    let mut out = Sweep::default();
    walk(root, subject, &mut out)?;
    out.call_sites.sort();
    Ok(out)
}

/// One directory, recursively. Split out so [`sweep`] can sort and report.
fn walk(dir: &Path, subject: &str, out: &mut Sweep) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("could not read {}: {e}", dir.display())
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            // ui-text-exempt: a test failure message, never displayed to an operator.
            format!("could not read an entry of {}: {e}", dir.display())
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, subject, out)?;
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).map_err(|e| {
            // ui-text-exempt: a test failure message, never displayed to an operator.
            format!("could not read {}: {e}", path.display())
        })?;
        out.files_read += 1;
        let found = calls_in(&src, subject).map_err(|e| {
            // ui-text-exempt: a test failure message, never displayed to an operator.
            format!("{}: {e}", path.display())
        })?;
        if found > 0 {
            out.call_sites.push((path, found));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The least number of `.rs` files a sweep of this crate must see before
    /// its verdict means anything.
    ///
    /// A floor rather than an exact count, deliberately: an exact number is a
    /// figure in prose that drifts (`HANDOFF.md` §10's fifth bullet, five times
    /// and counting), while a floor only ever fails for the reason it exists —
    /// a walker that stopped early. The crate had around 150 source files when
    /// this was written.
    const MIN_FILES_SWEPT: usize = 100;

    // =====================================================================
    // THE CHECK
    // =====================================================================

    /// ★★ **`apply_redactions` is called in exactly one file, and it is the
    /// one that proves.**
    ///
    /// The assertion this module exists to make. A failure here means either a
    /// second path to the engine's removal has appeared — the `Pass 72.0`
    /// artefact, an unverified redaction that will not know it is one — or the
    /// one legitimate call has moved and nothing has been told.
    #[test]
    fn the_engines_removal_is_called_from_exactly_one_place() {
        let root = crate_src();
        let swept = sweep(&root, SUBJECT).expect("the crate's own source must sweep");

        // Fail closed #1: a walker that read almost nothing.
        assert!(
            swept.files_read >= MIN_FILES_SWEPT,
            "the sweep read only {} file(s) under {}. That is not a monopoly \
             holding, it is a walker that stopped — and 'found nothing' must \
             never print the same as 'looked at nothing'",
            swept.files_read,
            root.display()
        );

        // Fail closed #2: zero call sites is not a pass.
        assert!(
            !swept.call_sites.is_empty(),
            "no call to `{SUBJECT}` was found anywhere in this crate. Either it \
             has been renamed and this check has not been told, or the apply \
             pipeline has stopped calling the engine — and a redaction feature \
             that does not redact is the worse of the two. Reading this as \
             'the monopoly holds' is the vacuous pass this module exists to \
             refuse."
        );

        let offenders: Vec<&PathBuf> = swept
            .call_sites
            .iter()
            .map(|(path, _)| path)
            .filter(|path| !path.ends_with(OWNER.iter().collect::<PathBuf>()))
            .collect();
        assert!(
            offenders.is_empty(),
            "★ `{SUBJECT}` is called outside `redact/mod.rs`: {offenders:?}\n\
             \n\
             That call obtains the redacted bytes without the absence proof, \
             which is exactly what `SALVAGE.md`'s Pass 72.0 note describes: a \
             shell that ships an unverified redaction and will not know. Route \
             it through `redact::prepare_redaction_apply` and write through \
             `PreparedRedaction::write_to`, which proves the buffer between it \
             and the syscall."
        );
        assert_eq!(
            swept.call_sites.len(),
            1,
            "exactly one file may call it: {:?}",
            swept.call_sites
        );
    }

    /// ★ **Nothing in `redact/` reaches for the incremental writer.**
    ///
    /// [`super::super`] §1.1's *"there is no parameter anywhere that could make
    /// an apply write incrementally"*, restated as a property of the directory
    /// rather than of a reader's care.
    ///
    /// The hazard is specific to this shell and did not exist in the salvage
    /// source's world: `crate::app::save` is built on `to_incremental_bytes`
    /// and `file.save_copy`'s shipped tooltip promises it, so the verb is
    /// idiomatic here, well documented, and one autocompletion away from the
    /// one directory where it would leave the un-redacted content in a prior
    /// revision of a file the operator has been told is redacted.
    #[test]
    fn the_apply_pipeline_never_reaches_for_the_incremental_writer() {
        let root = crate_src().join("redact");
        let swept = sweep(&root, FORBIDDEN_IN_REDACT).expect("`redact/` must sweep");
        assert!(
            swept.files_read >= 3,
            "the sweep read only {} file(s) under {}",
            swept.files_read,
            root.display()
        );
        assert!(
            swept.call_sites.is_empty(),
            "★ `{FORBIDDEN_IN_REDACT}` is called inside `redact/`: {:?}\n\
             \n\
             An incremental save appends a revision and leaves the ORIGINAL \
             bytes in the file. For a redaction that puts the removed content \
             one `startxref` hop away in a document pdfcer has told the operator \
             is redacted. Apply is a full rewrite or it does not happen.",
            swept.call_sites
        );
    }

    // =====================================================================
    // THE SELF-TEST — the reader proves it bites
    // =====================================================================
    //
    // `check-file-size.sh`'s header states the rule these keep: a gate that has
    // never been observed to fail is not evidence of anything. Between them the
    // fixtures below plant every misreading that would turn this check green
    // while the unverified path shipped.

    /// A fixture module with one genuine call and every trap a text scan falls
    /// into.
    const FIXTURE: &str = r####"
//! A doc comment naming apply_redactions, which calls nothing.

use pdfcer_core::redact::apply_redactions;

/// Another mention of apply_redactions, in prose.
fn real() {
    // apply_redactions in a line comment
    let _ = "apply_redactions in a string";
    let _ = apply_redactions(&doc, &opts);
}
"####;

    /// **A. The reader finds a real call.**
    ///
    /// Without this, assertion B below could pass by finding nothing at all,
    /// which is the failure mode the module header's "fail closed" section is
    /// about, arriving inside the self-test instead.
    #[test]
    fn the_reader_finds_a_real_call() {
        assert_eq!(
            calls_in(FIXTURE, SUBJECT).expect("the fixture parses"),
            1,
            "the reader missed a plain call expression"
        );
    }

    /// **B. Neither a comment, nor a doc comment, nor a string, nor a `use` is
    /// a call.**
    ///
    /// The four false positives a grep produces, and the reason the count in A
    /// is `1` rather than `5`. The `use` line matters most: every module that
    /// imports the function without calling it would otherwise be reported as a
    /// breach, and a check that cries wolf gets its allow-list widened until it
    /// says nothing.
    #[test]
    fn the_reader_counts_only_calls() {
        // The same fixture with the CALL removed and everything else left in
        // place — which is exactly the shape of a module that mentions the
        // engine's verb and does not use it.
        let mentions_only = FIXTURE.replace("let _ = apply_redactions(&doc, &opts);", "");
        assert_ne!(
            mentions_only, FIXTURE,
            "the plant must actually change the fixture"
        );
        assert_eq!(
            calls_in(&mentions_only, SUBJECT).expect("the fixture parses"),
            0,
            "a doc comment, a line comment, a string literal and a `use` are \
             not calls; counting any of them would report a monopoly broken \
             that never was"
        );
    }

    /// **C. A planted second call site is reported, wherever it hides.**
    ///
    /// The real defect, in the two shapes it would actually take: a plain call
    /// in a function, and one inside a closure — which is where a hand-written
    /// AST walk would most plausibly have stopped, and which is why this uses
    /// `syn::visit` rather than a bespoke recursion.
    #[test]
    fn a_planted_second_call_is_reported() {
        let planted = format!(
            "{FIXTURE}\nfn sneaky() {{ let f = || {{ let _ = \
             pdfcer_core::redact::apply_redactions(&d, &o); }}; f(); }}\n"
        );
        assert_eq!(
            calls_in(&planted, SUBJECT).expect("the fixture parses"),
            2,
            "a call inside a closure was not seen — 'anywhere' has to mean \
             anywhere, or the monopoly is one nested block deep"
        );
    }

    /// **D. A method call of the same name counts too.**
    ///
    /// The engine's is a free function today. A future engine that moved it
    /// onto a type would otherwise slip the monopoly in silence.
    #[test]
    fn a_method_call_of_the_same_name_counts() {
        let src = "fn f() { let _ = engine.apply_redactions(&opts); }";
        assert_eq!(calls_in(src, SUBJECT).expect("parses"), 1);
    }

    /// **E. A source that does not parse is refused rather than counted as
    /// clean.**
    ///
    /// Failing closed at the level of a single file. A syntax error in the one
    /// module that matters must stop the check, not remove that module from it.
    #[test]
    fn an_unparsable_source_is_refused() {
        let err = calls_in("fn f( {{{ ", SUBJECT).expect_err("this is not Rust");
        assert!(!err.is_empty());
    }

    /// **F. A sweep of a directory that does not exist is an error, not an
    /// empty clean result.**
    ///
    /// The "looked at nothing" state, closed at the level of the tree. In the
    /// real check it cannot arise — [`crate_src`] is built from
    /// `CARGO_MANIFEST_DIR` — and it is asserted anyway, because the reason it
    /// cannot arise is a property of one line that a refactor could change.
    #[test]
    fn a_missing_tree_is_an_error() {
        let missing = crate_src().join("no-such-directory-exists-here");
        assert!(
            sweep(&missing, SUBJECT).is_err(),
            "an unscanned tree is not a clean tree"
        );
    }

    /// **G. The sweep really does descend, and really does report a planted
    /// file.**
    ///
    /// A and C prove the *reader* bites; this proves the *walker* does. A
    /// fixture tree is built under the OS temporary directory with the
    /// violation two levels down, because a walker that only read its top
    /// directory would pass every test above and report the real crate clean —
    /// the crate's own offender would have to be in `src/` itself to be seen.
    #[test]
    fn the_walker_descends_and_reports_a_planted_file() {
        let root = std::env::temp_dir().join("pdfcer-gui-sealed-selftest");
        let nested = root.join("one").join("two");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).expect("the fixture tree must be creatable");
        std::fs::write(root.join("clean.rs"), "fn a() {}").expect("write");
        std::fs::write(root.join("notes.txt"), "apply_redactions(&d, &o)").expect("write");
        std::fs::write(nested.join("planted.rs"), FIXTURE).expect("write");

        let swept = sweep(&root, SUBJECT).expect("the fixture tree must sweep");
        assert_eq!(
            swept.files_read, 2,
            "two `.rs` files and no `.txt`: {swept:?}"
        );
        assert_eq!(
            swept.call_sites,
            vec![(nested.join("planted.rs"), 1)],
            "the walker did not reach a file two directories down, so its \
             verdict on the real crate covers only `src/` itself"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
