//! # `redact::sealed` — the call-site monopoly, read from the syntax tree
//!
//! [`super`] §2.4. One property, asserted over **every `.rs` file in this
//! crate**:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/redact/sealed.md`.

use std::path::{Path, PathBuf};

/// **The identifiers whose call sites are counted, and how many of each this
/// module accounts for.**
///
/// The **last path segment**, not the whole path, because the path is a
/// spelling decision (`pdfcer_core::redact::apply_redactions` today, a `use`
/// away from a bare `apply_redactions` tomorrow) and the function is the fact.
/// The same reduction `reach.rs` makes for its guard functions, for the same
/// reason.
///
/// Each count is **exact**, not a ceiling, on this module's own fail-closed
/// reasoning: a range would let a call be ADDED and a call be REMOVED in the
/// same edit and report nothing, and *"the proof pipeline no longer calls the
/// engine"* is the failure this file exists to catch. When a legitimate route
/// lands or leaves, the number changes in the same commit as the route.
///
/// Note that `apply_redactions_with` and `apply_redactions_deferred` are
/// separate rows and cannot be confused for one another: [`calls_in`] compares
/// the identifier for **equality**, not by prefix, which is why the deferred
/// verb needed a row of its own rather than being absorbed into the first
/// one's count.
///
/// That same equality is why swapping a call for a **wider-signatured
/// twin** of itself is a silent hole in this seal rather than a compile error.
/// `apply_redactions` and `apply_redactions_with` are the same removal; the
/// first hard-codes the engine's default residual scope. Moving the call from
/// one to the other without moving the row here would leave a row matching
/// zero call sites — which the fail-closed assertion below catches — and,
/// worse, would leave the new name pinned by nothing at all had the row been
/// left as a ceiling instead of an exact count. It is an exact count.
// ui-text-exempt: Rust function names, matched against the parsed syntax tree.
const SUBJECTS: [(&str, usize); 4] = [
    // The free function: `prepare_redaction_apply`, which produces bytes and
    // proves them before returning. The `_with` form is the pinned one because
    // it is the only one this crate may call — the bare `apply_redactions`
    // hard-codes a residual scope the operator is offered a choice about.
    ("apply_redactions_with", 1),
    // `stage_into_session` — arms the removal and touches nothing else.
    ("apply_redactions_deferred", 1),
    // `save_applying_pending` — performs it at save time and proves the buffer.
    ("save_applying_redaction", 1),
    // `cancel_staged_redaction` — disarms it. See the header for why a verb
    // that removes nothing is pinned as hard as the three that do.
    ("cancel_pending_redaction", 1),
];

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
fn crate_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every source tree the monopoly covers: this crate and `pdfcer-gui` above
/// it, which can reach the engine's removal verbs as directly as this one.
fn swept_roots() -> [PathBuf; 2] {
    [
        crate_src(),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pdfcer-gui/src"),
    ]
}

/// [`sweep`] over each of `roots`, merged. A root that cannot be read is an
/// error, never an empty contribution.
pub(super) fn sweep_all(roots: &[PathBuf], subject: &str) -> Result<Sweep, String> {
    let mut out = Sweep::default();
    for root in roots {
        let one = sweep(root, subject)?;
        out.files_read += one.files_read;
        out.call_sites.extend(one.call_sites);
    }
    out.call_sites.sort();
    Ok(out)
}

/// The one file permitted to call [`SUBJECT`], relative to the source root.
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
pub(super) fn calls_in(src: &str, subject: &str) -> Result<usize, String> {
    use syn::visit::Visit;

    /// Counts call expressions anywhere in a file.
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
    const MIN_FILES_SWEPT: usize = 100;

    /// The subject the self-test fixtures below are written around.
    const FIXTURE_SUBJECT: &str = SUBJECTS[0].0;

    // =====================================================================
    // THE CHECK
    // =====================================================================

    /// **Every removal verb is called in exactly one file — the one that
    /// proves — and exactly as many times as [`SUBJECTS`] accounts for.**
    #[test]
    fn every_removal_verb_is_called_from_exactly_one_place() {
        let roots = swept_roots();
        for (subject, expected) in SUBJECTS {
            let swept = sweep_all(&roots, subject).expect("both crates' source must sweep");

            // Fail closed #1: a walker that read almost nothing.
            assert!(
                swept.files_read >= MIN_FILES_SWEPT,
                "the sweep for `{subject}` read only {} file(s) under {}. That \
                 is not a monopoly holding, it is a walker that stopped — and \
                 'found nothing' must never print the same as 'looked at \
                 nothing'",
                swept.files_read,
                roots
                    .iter()
                    .map(|r| r.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            // Fail closed #2: zero call sites is not a pass.
            assert!(
                !swept.call_sites.is_empty(),
                "no call to `{subject}` was found anywhere in either crate. \
                 Either it has been renamed and this check has not been told, \
                 or the apply pipeline has stopped calling the engine — and a \
                 redaction feature that does not redact is the worse of the \
                 two. Reading this as 'the monopoly holds' is the vacuous pass \
                 this module exists to refuse."
            );

            let offenders: Vec<&PathBuf> = swept
                .call_sites
                .iter()
                .map(|(path, _)| path)
                .filter(|path| !path.ends_with(OWNER.iter().collect::<PathBuf>()))
                .collect();
            assert!(
                offenders.is_empty(),
                "★ `{subject}` is called outside `redact/mod.rs`: {offenders:?}\n\
                 \n\
                 That call reaches the engine's removal without the absence \
                 proof, which is exactly what `SALVAGE.md`'s Pass 72.0 note \
                 describes: a shell that ships an unverified redaction and will \
                 not know. Route it through `redact::prepare_redaction_apply`, \
                 `redact::stage_into_session` or `redact::save_applying_pending` \
                 — each of which either proves its own bytes or has none to \
                 prove and says so."
            );
            assert_eq!(
                swept.call_sites.len(),
                1,
                "exactly one file may call `{subject}`: {:?}",
                swept.call_sites
            );

            let (owner, calls) = &swept.call_sites[0];
            assert_eq!(
                *calls,
                expected,
                "★ `{subject}` is called {calls} time(s) in {}, and {expected} \
                 is what `SUBJECTS` accounts for. A call too many is either an \
                 unproven route to the engine's removal — `SALVAGE.md`'s \
                 Pass 72.0 artefact — or a second way to disarm one the \
                 operator confirmed. A call too few is a route that has been \
                 removed and this table has not been told, which is how the \
                 collapsing `apply_into_session` would have left in silence on \
                 2026-09-05 had this been a ceiling rather than a count.",
                owner.display()
            );
        }
    }

    /// **Nothing in `redact/` reaches for the incremental writer.**
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
        // The suite is a DIRECTORY, and this predicate has to keep saying so.
        // Keyed on a file name it would reclassify every test in
        // `redact/tests/` as production the day the suite outgrew one file.
        //
        // ui-text-exempt: a directory name inside this crate, never displayed.
        let suite: PathBuf = ["redact", "tests"].iter().collect();
        let (measurement, production): (Vec<_>, Vec<_>) = swept
            .call_sites
            .iter()
            .partition(|(path, _)| path.parent().is_some_and(|d| d.ends_with(&suite)));
        assert!(
            production.is_empty(),
            "★ `{FORBIDDEN_IN_REDACT}` is called inside `redact/`: {production:?}\n\
             \n\
             An incremental save appends a revision and leaves the ORIGINAL \
             bytes in the file. For a redaction that puts the removed content \
             one `startxref` hop away in a document pdfcer has told the operator \
             is redacted. Apply is a full rewrite or it does not happen."
        );
        let measured: usize = measurement.iter().map(|(_, n)| *n).sum();
        assert!(
            measured >= 2,
            "★★★ `redact/tests/` calls `{FORBIDDEN_IN_REDACT}` {measured} \
             time(s), and the deferred route's entire safety argument is that \
             an ordinary save of a STAGED session is refused by name. That \
             claim is the engine's; the measurement is ours, and it is gone. \
             Restore \
             `both_ordinary_save_modes_are_refused_by_name_while_staged` \
             before shipping anything that depends on it."
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
    fn fixture() -> String {
        let verb = FIXTURE_SUBJECT;
        format!(
            r####"
//! A doc comment naming {verb}, which calls nothing.

use pdfcer_core::redact::{verb};

/// Another mention of {verb}, in prose.
fn real() {{
    // {verb} in a line comment
    let _ = "{verb} in a string";
    let _ = {verb}(&doc, &opts);
}}
"####
        )
    }

    /// **A. The reader finds a real call.**
    #[test]
    fn the_reader_finds_a_real_call() {
        assert_eq!(
            calls_in(&fixture(), FIXTURE_SUBJECT).expect("the fixture parses"),
            1,
            "the reader missed a plain call expression"
        );
    }

    /// **B. Neither a comment, nor a doc comment, nor a string, nor a `use` is
    /// a call.**
    #[test]
    fn the_reader_counts_only_calls() {
        // The same fixture with the CALL removed and everything else left in
        // place — which is exactly the shape of a module that mentions the
        // engine's verb and does not use it.
        let built = fixture();
        let mentions_only = built.replace(&format!("let _ = {FIXTURE_SUBJECT}(&doc, &opts);"), "");
        assert_ne!(
            mentions_only, built,
            "the plant must actually change the fixture"
        );
        assert_eq!(
            calls_in(&mentions_only, FIXTURE_SUBJECT).expect("the fixture parses"),
            0,
            "a doc comment, a line comment, a string literal and a `use` are \
             not calls; counting any of them would report a monopoly broken \
             that never was"
        );
    }

    /// **C. A planted second call site is reported, wherever it hides.**
    #[test]
    fn a_planted_second_call_is_reported() {
        let verb = FIXTURE_SUBJECT;
        let planted = format!(
            "{}\nfn sneaky() {{ let f = || {{ let _ = \
             pdfcer_core::redact::{verb}(&d, &o); }}; f(); }}\n",
            fixture()
        );
        assert_eq!(
            calls_in(&planted, FIXTURE_SUBJECT).expect("the fixture parses"),
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
        let src = format!("fn f() {{ let _ = engine.{FIXTURE_SUBJECT}(&opts); }}");
        assert_eq!(calls_in(&src, FIXTURE_SUBJECT).expect("parses"), 1);
    }

    /// **E. A source that does not parse is refused rather than counted as
    /// clean.**
    ///
    /// Failing closed at the level of a single file. A syntax error in the one
    /// module that matters must stop the check, not remove that module from it.
    #[test]
    fn an_unparsable_source_is_refused() {
        let err = calls_in("fn f( {{{ ", FIXTURE_SUBJECT).expect_err("this is not Rust");
        assert!(!err.is_empty());
    }

    /// **F. A sweep of a directory that does not exist is an error, not an
    /// empty clean result.**
    #[test]
    fn a_missing_tree_is_an_error() {
        let missing = crate_src().join("no-such-directory-exists-here");
        assert!(
            sweep(&missing, FIXTURE_SUBJECT).is_err(),
            "an unscanned tree is not a clean tree"
        );
    }

    /// **G. The sweep really does descend, and really does report a planted
    /// file.**
    #[test]
    fn the_walker_descends_and_reports_a_planted_file() {
        let root =
            std::env::temp_dir().join(format!("pdfcer-gui-sealed-selftest-{}", std::process::id()));
        let nested = root.join("one").join("two");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&nested).expect("the fixture tree must be creatable");
        std::fs::write(root.join("clean.rs"), "fn a() {}").expect("write");
        std::fs::write(root.join("notes.txt"), format!("{FIXTURE_SUBJECT}(&d, &o)"))
            .expect("write");
        std::fs::write(nested.join("planted.rs"), fixture()).expect("write");

        let swept = sweep(&root, FIXTURE_SUBJECT).expect("the fixture tree must sweep");
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
