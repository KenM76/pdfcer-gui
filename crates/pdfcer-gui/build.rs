//! # `build.rs` — put the application icon inside the executable
//!
//! One job. The operator asked, on 2026-08-18, for *"a pdf icon to the exe so
//! it shows as the icon when I associate it with pdfs"*, and an icon Explorer
//! can show is an icon in the executable's `.rsrc` section. Nothing loaded at
//! run time can satisfy that: the shell reads the icon **without running the
//! program**.
//!
//! `assets/pdfcer-gui.rc` is the resource script and carries the reasoning for
//! what is in it — the icon's ID, and why the `VERSIONINFO` block is worth
//! having. This file is only the plumbing.
//!
//! ## Why `embed-resource` rather than invoking `rc.exe` here
//!
//! Because finding a resource compiler is the entire problem. `rc.exe` lives
//! inside a versioned Windows SDK directory that is not on `PATH`, its location
//! differs per SDK version and per machine, a cross build wants `windres`
//! instead, and the MSVC and GNU toolchains want the output linked differently.
//! `embed-resource` is a build-dependency that exists to know all of that, and
//! re-deriving it here would be a machine-specific path in a repository.
//!
//! It is a **build** dependency: it runs on this machine and nothing from it is
//! linked into `pdfcer-gui.exe`. That is why it does not appear in
//! `THIRD_PARTY_LICENSES.md`, which `cargo-about` generates from the crates the
//! binary actually carries.
//!
//! ## Why `manifest_optional` and not `unwrap`
//!
//! An icon is **cosmetic**. A machine with no Windows SDK — a container, a
//! fresh CI image, a cross build — must still produce a working
//! `pdfcer-gui.exe`, and failing the build over a missing resource compiler
//! would trade a program that opens PDFs for a program that does not exist.
//! `manifest_optional()` is `embed-resource`'s own name for exactly that
//! trade-off, and it is the one the crate's README recommends *"if the manifest
//! is cosmetic (like an icon)"*.
//!
//! The consequence is stated rather than hidden: on such a machine the build
//! succeeds and the executable has no icon. The `cargo:warning` below is what
//! says so, because a silent absence here looks identical to this file never
//! having been written.
//!
//! ## Why the `cfg` is on the CALL and not on the file
//!
//! A `build.rs` runs on the **host**, so `cfg!(windows)` here is a fact about
//! the machine doing the building. Gating the file itself is not possible —
//! Cargo compiles it either way — and gating the *dependency* in `Cargo.toml`
//! by target would be wrong for the same reason: `[build-dependencies]` are
//! host dependencies, and `[target.'cfg(windows)'.build-dependencies]` selects
//! on the **target**, so a Linux-hosted cross build to Windows would ask for a
//! crate it had not been given.

fn main() {
    // Re-run when either half of the resource changes. Without these, editing
    // the `.rc` or regenerating the `.ico` leaves the previous resource in the
    // executable and the change looks like it did nothing — which is a
    // particularly confusing failure for an icon, because Explorer caches them
    // too and the reader ends up blaming the wrong cache.
    println!("cargo:rerun-if-changed=assets/pdfcer-gui.rc");
    println!("cargo:rerun-if-changed=assets/pdfcer-gui.ico");

    // The second job, added 2026-08-18 at the operator's request: stamp the
    // executable with when it was built and what engine is inside it. See
    // [`provenance`].
    provenance();

    #[cfg(windows)]
    {
        // `NONE` is `embed-resource`'s spelling of "no preprocessor
        // definitions"; the `.rc` needs none.
        let result = embed_resource::compile("assets/pdfcer-gui.rc", embed_resource::NONE);
        if let Err(error) = result.manifest_optional() {
            // A warning, not a failure. See the header: the alternative is
            // trading a working program for a missing one over an icon.
            println!(
                "cargo:warning=pdfcer-gui: the application icon was not embedded ({error}). The \
                 executable will build and run; Explorer will show it with the default icon. A \
                 Windows SDK (for rc.exe) is what supplies the resource compiler."
            );
        }
    }
}

// ===========================================================================
// Build provenance — what About reports
// ===========================================================================

/// **Stamp the executable with when it was built and what it was built from.**
///
/// The operator asked, 2026-08-18, for the About box to name *"the date and
/// time of the build … and the date and time of the builds of the used pdfcer
/// and iccce"*.
///
/// # ★ Why the engine's stamp is a COMMIT time and not a build time
///
/// `pdfcer-core`, `pdfcer-render` and `pdfcer-print` are compiled **into** this
/// executable from a git dependency. They have no build of their own: their
/// object code was produced by the same `cargo build` that produced everything
/// else here, so a "pdfcer build time" would be this binary's build time
/// restated, which answers nothing.
///
/// The fact the operator is actually after — *which engine is in this exe* —
/// is the **revision** and **when that revision was committed**. Two builds an
/// hour apart from the same engine revision share an engine; two builds a
/// minute apart across an engine commit do not. So the engine row reports the
/// revision and its commit timestamp, and says which it is.
///
/// # Why the timestamp can come from the environment
///
/// `PDFCER_BUILD_STAMP`, when set, wins. `tools/package-portable.py` sets it,
/// because Python knows the machine's local time and offset and this build
/// script — with no date crate available — does not. The workspace may not add
/// a dependency that is not already in pdfcer's lockfile (`PROJECT_PLAN.md` §2),
/// and `chrono` is not, so the fallback below computes **UTC** from
/// `SystemTime` with the civil-from-days algorithm and labels it UTC rather
/// than pretending to know the offset. A stamp that says the wrong hour is
/// worse than one that says a true hour in a named zone.
fn provenance() {
    // ★ Without these, this script does not re-run when the code changes and
    // the stamp is silently the time of some earlier build — which is the one
    // failure mode a build stamp must not have, because nothing about the
    // program looks wrong.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../../Cargo.lock");
    println!("cargo:rerun-if-env-changed=PDFCER_BUILD_STAMP");

    let stamp = std::env::var("PDFCER_BUILD_STAMP").unwrap_or_else(|_| utc_now());
    println!("cargo:rustc-env=PDFCER_BUILD_TIME={stamp}");

    // This crate's own revision, so a build can be tied to a commit.
    let rev = git(&["rev-parse", "--short", "HEAD"], ".").unwrap_or_default();
    let dirty = git(&["status", "--porcelain"], ".")
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    println!(
        "cargo:rustc-env=PDFCER_GUI_REV={}{}",
        if rev.is_empty() { "unknown" } else { &rev },
        if dirty { "-dirty" } else { "" }
    );

    let lock = std::fs::read_to_string("../../Cargo.lock").unwrap_or_default();

    // The engine: version, revision, and the revision's commit time.
    //
    // ★★ Looked up under BOTH names, newest first. `Cargo.lock` records the
    // package's REAL name, and under the temporary rename shim
    // (`Cargo.toml`'s `package = ...` keys) that is still the engine's
    // pre-rename one — so a single literal here found nothing, the About window
    // reported no engine version, and its own test caught it.
    //
    // ⇒ Falls away with the shim; `tools/gates/check-engine-rename-shim.sh`
    // fires when that day comes and names this among the places to clean up.
    let (version, rev, repo) = {
        let new = locked_git_package(&lock, "pdfcer-core");
        if new.0.is_empty() {
            // old-name-exempt: the engine's pre-rename package name, which is
            // what Cargo.lock still records while the shim is up.
            locked_git_package(&lock, "pdfce-core") // old-name-exempt: the engine's pre-rename package, as Cargo.lock records it
        } else {
            new
        }
    };
    println!("cargo:rustc-env=PDFCER_ENGINE_VERSION={version}");
    println!(
        "cargo:rustc-env=PDFCER_ENGINE_REV={}",
        rev.chars().take(7).collect::<String>()
    );
    let engine_time = if repo.is_empty() || rev.is_empty() {
        String::new()
    } else {
        commit_time(&rev, &repo)
    };
    println!("cargo:rustc-env=PDFCER_ENGINE_TIME={engine_time}");

    // ★ iccce, which as of 2026-08-18 is NOT linked into this build.
    //
    // Reported as absent rather than omitted. `RIBBON_IA.md`'s no-placeholders
    // rule governs *controls* — an unavailable capability offers no button —
    // and this is a provenance report, not a control. An operator asking what
    // is inside their build is owed "no colour management in this one", which
    // is a different and more useful answer than silence. It fills in by itself
    // the day the dependency is added.
    let (icc_version, icc_rev, icc_repo) = locked_git_package(&lock, "iccce");
    println!("cargo:rustc-env=PDFCER_ICCCE_VERSION={icc_version}");
    println!(
        "cargo:rustc-env=PDFCER_ICCCE_REV={}",
        icc_rev.chars().take(7).collect::<String>()
    );
    let icc_time = if icc_repo.is_empty() || icc_rev.is_empty() {
        String::new()
    } else {
        commit_time(&icc_rev, &icc_repo)
    };
    println!("cargo:rustc-env=PDFCER_ICCCE_TIME={icc_time}");
}

/// When `rev` was committed, formatted the way the build stamp is.
///
/// `%cd` with an explicit `--date=format:` rather than `%cI`, because ISO-8601
/// strict reads `2026-08-18T17:12:54-04:00` — correct, machine-friendly, and
/// not what belongs beside a human-readable build time three lines above it.
/// Seconds are dropped for the same reason: nobody comparing two builds needs
/// them, and the line is already long.
///
/// The offset is kept and printed as a number. A zone abbreviation is ambiguous
/// across regions; an offset never is.
fn commit_time(rev: &str, repo: &str) -> String {
    git(
        &[
            "show",
            "-s",
            "--date=format:%Y-%m-%d %H:%M %z",
            "--format=%cd",
            rev,
        ],
        repo,
    )
    .unwrap_or_default()
}

/// Run `git` in `dir` and return trimmed stdout, or `None`.
///
/// Every caller treats failure as "unknown" rather than as a build error. A
/// source tree exported without its `.git` still has to build, and an About box
/// that says `unknown` is a smaller problem than a program that will not
/// compile off a tarball.
///
/// ★ Reading `D:\Dev\pdfcer` is deliberate and permitted: the governing rule of
/// this project is that the engine repository is **read-only until fold-in**,
/// and `git show -s` writes nothing.
fn git(args: &[&str], dir: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Pull `(version, git revision, repository path)` for `name` out of a
/// `Cargo.lock`.
///
/// Hand-parsed rather than with a TOML crate, for the dependency reason in
/// [`provenance`]'s docs. The shape being read is fixed and small:
///
/// ```text
/// [[package]]
/// name = "pdfcer-core"
/// version = "0.7.0"
/// source = "git+file:///D:/Dev/pdfcer?branch=main#6af5655c…"
/// ```
///
/// Returns empty strings for anything absent, which is how a package that is
/// **not in this build** — `iccce`, today — reports itself.
fn locked_git_package(lock: &str, name: &str) -> (String, String, String) {
    let mut version = String::new();
    let mut rev = String::new();
    let mut repo = String::new();
    let mut in_package = false;
    for line in lock.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            in_package = false;
            continue;
        }
        if let Some(v) = line.strip_prefix("name = ") {
            in_package = v.trim_matches('"') == name;
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(v) = line.strip_prefix("version = ") {
            version = v.trim_matches('"').to_owned();
        }
        if let Some(v) = line.strip_prefix("source = ") {
            let src = v.trim_matches('"');
            if let Some((url, sha)) = src.split_once('#') {
                rev = sha.to_owned();
                // `git+file:///D:/Dev/pdfcer?branch=main` -> `D:/Dev/pdfcer`
                // ★ Only a `file://` source names a directory this build can
                // run `git` in. A crates.io or `https://` dependency is real
                // and will still report its version and revision; its commit
                // time simply comes back empty, which the About box renders as
                // the revision alone rather than as a wrong date.
                repo = url
                    .strip_prefix("git+file:///")
                    .map(|u| u.split('?').next().unwrap_or_default().to_owned())
                    .unwrap_or_default();
            }
        }
    }
    (version, rev, repo)
}

/// `YYYY-MM-DD HH:MM UTC` from the system clock, with no date crate.
///
/// Howard Hinnant's `civil_from_days`, which is the standard branch-free way to
/// invert the proleptic Gregorian calendar and is correct for any date this
/// program will ever be built on. It is here rather than as a dependency
/// because the workspace may add none that pdfcer's lockfile does not already
/// carry, and because forty lines that are provably right beat a supply-chain
/// entry for a string in an About box.
fn utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;

    // Shift the epoch to 0000-03-01 so leap days land at the end of the cycle.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        tod / 3600,
        (tod % 3600) / 60
    )
}
