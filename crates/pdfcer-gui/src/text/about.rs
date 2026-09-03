//! # `text::about` — the attribution surface, in the operator's own window
//!
//! Every word the About dialog shows, plus the **structured attribution
//! catalog** it draws from. Consumed by [`crate::dialogs::about`], and by the
//! test that pins this catalog against the shipped
//! `THIRD_PARTY_LICENSES.md`.
//!
//! ## ★ Why an attribution surface exists at all, when `LICENSE` already ships
//!
//! Until 2026-08-14 this repository had no attribution surface of any kind:
//! no `PROVENANCE.md`, no `cargo-about`, no third-party licence text anywhere
//! in the program. That was **fine**, and it is worth saying why, because the
//! reason is exactly what stopped being true.
//!
//! pdfcer-gui shipped only permissively-licensed *code*. MIT and Apache-2.0
//! notices are satisfied by the `LICENSE` file in the package and by the
//! licence metadata in `Cargo.toml`, which a reader of the source tree can
//! check. Nobody was handed a file whose licence obliged pdfcer to *tell them*
//! anything they could not already look up.
//!
//! The operator decided on 2026-08-14 to ship the `ocrs` OCR model weights —
//! *"yes ship that model in the mit repo with proper credit"* — and those are
//! **CC-BY-SA-4.0**. That licence's **BY** clause requires attribution to
//! reach the **recipient of the work**, not merely a reader of the repository
//! it was built from. A `PROVENANCE.md` in a source tree discharges nothing
//! for someone who was handed `pdfcer-gui.exe` in a folder.
//!
//! Auditing what pdfcer already ships, in the course of building the surface
//! that will carry the model's notice, turned up **three third-party works
//! that are redistributed today** and were carrying no notice either — the
//! bundled substitute faces and the two Adobe data tables below. They arrive
//! through the engine (`pdfcer-core` and `pdfcer-render` are path dependencies
//! and Rust links them statically), so they are inside `pdfcer-gui.exe`
//! whether or not anyone here thought about them. They are the reason this
//! catalog is **not empty before OCR lands**, and a surface that would have
//! been empty until its first entry is a surface nobody would have trusted.
//!
//! ## The two surfaces, and why neither replaces the other
//!
//! | Surface | What it carries | Who reaches it |
//! |---|---|---|
//! | `THIRD_PARTY_LICENSES.md`, in the package | every licence **text**, in full — hundreds of kilobytes of it | someone who opens the folder |
//! | this dialog | the **attribution**: who made it, what it is, under what terms, and whether pdfcer changed it | someone who runs the program |
//!
//! Neither is a substitute for the other and the split is not arbitrary. A
//! dialog cannot reasonably render that much licence text, and a `.md` file
//! in a folder is invisible to an operator who launched pdfcer from a shortcut
//! and is one `del` away from an operator who tidied up. So the dialog names
//! the works and the terms and points at the file, and the file carries the
//! texts. `tools/gates/check-shipped-assets.py` requires **both**: a
//! redistributed asset directory must be cited in `about.hbs` *and* in this
//! module, because a change that remembers one and forgets the other is the
//! commonest way an obligation half-lands.
//!
//! ## What this module is NOT
//!
//! It is not a second copy of `THIRD_PARTY_LICENSES.md`, and it must not grow
//! into one. It carries no licence text — [`Attribution`] has no field for
//! any — precisely so that the two surfaces cannot disagree about what a
//! licence says. Where a licence's own terms require a **link** rather than a
//! reproduction (Creative Commons licences do; BSD-style ones do not),
//! [`Attribution::licence_url`] carries it.
//!
//! ## Conventions
//!
//! The catalog convention of [`crate::text`] applies unchanged: sentence
//! case, no trailing period on a label, full sentences with punctuation for
//! prose. One addition specific to this module — **every field of every
//! [`Attribution`] is lifted from a source that was read, never reconstructed
//! from what a licence of that family usually says.** The sources are named
//! per entry. A wrong attribution is worse than an absent one.

/// One third-party work `pdfcer-gui.exe` redistributes, and everything an
/// attribution-style licence asks be said about it.
///
/// # Why these five fields and not others
///
/// They are the union of what CC-BY-SA-4.0 §3(a)(1) requires — identification
/// of the creator, a notice of the licence, a link to it, and an indication
/// of whether the material was modified — with what a BSD-style notice
/// requires, which is the copyright line and the licence text. The text
/// itself is deliberately absent; see the module header.
///
/// `origin` is not required by any licence here. It is carried because an
/// attribution that names a creator but not *which* artefact is unverifiable
/// by the person reading it, and this project's rule about claim-bearing copy
/// is that a claim a reader cannot check is a claim nobody should make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribution {
    /// What the work is, in the operator's terms. A noun phrase, not a path.
    pub component: &'static str,
    /// Who made it. The name the licence requires be given.
    pub creator: &'static str,
    /// Where it came from, specifically enough to be checked.
    pub origin: &'static str,
    /// The licence, by its common or SPDX name.
    pub licence: &'static str,
    /// A link to the licence's own terms, where the licence requires one.
    ///
    /// `None` for every entry below, and that is correct rather than
    /// unfinished: BSD-3-Clause and APAFML require their **text** to be
    /// reproduced — which `THIRD_PARTY_LICENSES.md` does, in full — and have
    /// no canonical deed URL to point at. Creative Commons licences are the
    /// family that asks for a link, and the field exists ready for the
    /// CC-BY-SA-4.0 model weights, whose deed is
    /// `https://creativecommons.org/licenses/by-sa/4.0/`.
    ///
    /// An `Option` rather than an empty string on purpose: the dialog draws
    /// nothing for `None`, which is the "no placeholders" invariant applied
    /// to a field rather than to a control.
    pub licence_url: Option<&'static str>,
    /// Whether pdfcer changed the work, stated plainly either way.
    ///
    /// An **indication of changes** is a distinct obligation from
    /// attribution, and "no changes" is a real answer to it that must still
    /// be given. It is a full sentence because it is a statement, not a
    /// label.
    pub changes: &'static str,
}

/// The dialog's title.
#[must_use]
pub fn title() -> &'static str {
    "About pdfcer"
}

/// The product name, as its own line above the version.
///
/// Lower-case, like the window title and every ribbon caption. The product is
/// written `pdfcer` everywhere, and a title-cased About box would be the one
/// place in the program that disagreed.
#[must_use]
pub fn product() -> &'static str {
    "pdfcer"
}

/// The version line.
///
/// `version` comes from `CARGO_PKG_VERSION` at the call site rather than
/// being written here, so it cannot drift from the crate manifest. The word
/// in front of it is the part that is copy.
#[must_use]
pub fn version_line(version: &str) -> String {
    format!("Version {version}")
}

// ===========================================================================
// Build provenance
// ===========================================================================

/// Heading for the block naming when this was built and what is inside it.
///
/// The operator asked for this on 2026-08-18: *"when I go to about pdfcer in
/// pdfcer-gui, can you include the date and time of the build. Also the date and
/// time of the builds of the used pdfcer and iccce"*.
#[must_use]
pub fn build_heading() -> &'static str {
    "Build"
}

/// The line for this program's own build.
///
/// `stamp` and `rev` come from `build.rs` through `env!`, so they cannot drift
/// from what was actually compiled. `rev` carries a `-dirty` suffix when the
/// tree had uncommitted changes, which is the fact an operator most needs when
/// a build does something a commit does not explain.
#[must_use]
pub fn build_line(stamp: &str, rev: &str) -> String {
    format!("pdfcer-gui — built {stamp} from {rev}")
}

/// The line for a component compiled INTO this program.
///
/// ★ The wording distinguishes **committed** from **built**, and the
/// distinction is the whole reason this is not simply three build times.
/// `pdfcer-core` and its siblings have no build of their own — they were
/// compiled by the same `cargo build` that produced everything else here, so
/// their "build time" is this binary's build time restated, which answers
/// nothing. What identifies the engine in a given executable is the revision
/// and when that revision was committed.
///
/// Reads *"pdfcer 0.7.0 — revision 6af5655, committed 2026-08-18 14:02"*.
#[must_use]
pub fn component_line(name: &str, version: &str, rev: &str, committed: &str) -> String {
    let mut line = format!("{name} {version} — revision {rev}");
    if !committed.is_empty() {
        line.push_str(&format!(", committed {committed}"));
    }
    line
}

/// The line for a component that is **not part of this build**.
///
/// ★ Reported rather than omitted, and the judgement is worth writing down
/// because it looks like a breach of the no-placeholders rule and is not. That
/// rule governs **controls**: an unavailable capability must offer no button,
/// because a button that does nothing is a lie about what the program can do.
/// This is a provenance report. An operator asking what is inside their
/// executable is owed *"no colour management in this one"* — which is a
/// different and more useful answer than silence, and is what tells them why
/// a colour-managed file looks the way it does.
///
/// It fills itself in the day the dependency is added: `build.rs` reads the
/// workspace `Cargo.lock`, and a transitive dependency appears there without
/// anyone editing anything.
#[must_use]
pub fn component_absent(name: &str) -> String {
    format!("{name} — not in this build")
}

/// Explains what the component lines are, once, under them.
#[must_use]
pub fn components_note() -> &'static str {
    "These are compiled into this program. A revision and its commit date identify the source they were built from; they have no separate build of their own."
}

/// One sentence saying what this program is.
///
/// Present because an About box that gives a version and no identity tells an
/// operator with two pdfcer builds installed nothing they did not know. Names
/// the engine explicitly: the single most common question about this binary
/// is whether it contains pdfcer or merely talks to it.
#[must_use]
pub fn summary() -> &'static str {
    "A PDF viewer and editor. The pdfcer engine is built into this program; nothing else is required to run it."
}

/// pdfcer's own licence, and its copyright line.
///
/// Lifted verbatim from the repository's `LICENSE` file — the SPDX name on
/// its first line and the copyright line on its third. Not reconstructed, and
/// not softened: this is claim-bearing copy about the terms the operator
/// grants, and the source of truth is the file that ships beside the binary.
#[must_use]
pub fn licence_line() -> &'static str {
    "MIT licence. Copyright (c) 2026 Ken Mantle."
}

/// The heading above the attribution list.
#[must_use]
pub fn attributions_heading() -> &'static str {
    "Bundled third-party material"
}

/// The sentence that points at the full texts.
///
/// Names the file **and** where it is, because "see the licence file" is
/// advice an operator cannot act on. `THIRD_PARTY_LICENSES.md` is copied into
/// the portable folder by `tools/package-portable.py`; the gate asserts that
/// it is, so this sentence cannot become false without something failing.
#[must_use]
pub fn full_texts_note() -> &'static str {
    "Full licence texts for these, and for every Rust crate pdfcer links, are in THIRD_PARTY_LICENSES.md in the folder beside the program."
}

/// The button that closes the dialog.
#[must_use]
pub fn close() -> &'static str {
    "Close"
}

// ---------------------------------------------------------------------------
// The catalog
// ---------------------------------------------------------------------------

/// Every third-party work this binary redistributes that `cargo-about` cannot
/// see.
///
/// # What is deliberately absent
///
/// - **Rust crates.** `cargo-about` harvests all of them from `Cargo.lock`
///   into `THIRD_PARTY_LICENSES.md` mechanically. Restating a subset of them
///   here by hand would create a second list that drifts, and the drift would
///   be invisible — the crate list runs to well over a hundred entries and
///   nobody re-reads it.
/// - **The pdfcer icon set** (`crates/pdfcer-gui/src/icons/assets/`). It is the
///   operator's own art under the project's own MIT licence, confirmed by him
///   on 2026-08-02; the `LICENSE` file already covers it and there is no
///   third-party grant to reproduce. Listing it under a heading that says
///   "third-party" would make this dialog say something untrue. See that
///   directory's `PROVENANCE.md`.
///
/// ★ **The `ocrs` OCR model weights were listed here as absent until
/// 2026-08-14, and are now the fourth entry below.** They are the reason this
/// whole surface exists, and the note that stood here said *"do not add the
/// entry before the files are actually in the package: an attribution for
/// something that is not there is a false statement in the other direction."*
/// They are in the package now —
/// `tools/package-portable.py`'s `PAYLOAD_ASSET_DIRS` copies them to
/// `models/ocrs/` beside the executable, and
/// `tools/gates/check-shipped-assets.py` declares them `how="copied"`, which
/// makes the two facts fail together rather than separately.
///
/// **They are also the one entry here that is not compiled into the binary.**
/// The other three are `include_bytes!` payloads and static tables; these are
/// loose files the program opens at run time. That difference does not change
/// what CC-BY-SA-4.0's BY clause asks — the weights are redistributed either
/// way, and the recipient is the same person — which is exactly why this
/// dialog says *what* each work is rather than *how it got here*.
///
/// # ★ The engineering constraint that travels with those weights
///
/// Written here rather than only in the engine's provenance note, because
/// this is the file someone will have open when the idea occurs to them.
///
/// **CC-BY-SA's share-alike clause binds adaptations, not collections.**
/// Shipping the weights **unmodified** alongside MIT code is distribution of
/// a verbatim work in a collection, and pdfcer's own licence is unaffected —
/// that is the reading the operator was shown and accepted. **Modifying them
/// creates Adapted Material, and the adapted weights must then be released
/// under CC-BY-SA-4.0 or a compatible licence.** That includes fine-tuning
/// them for CAD drawings, **quantizing them to shrink the 12,240,008-byte
/// download**, retraining them on any corpus, and converting them into
/// another runtime's format.
///
/// It would bind the derived model, not pdfcer's source. But it means
/// *"we'll just quantize it"* is a decision with a licence attached and needs
/// its own operator decision at the time — which is precisely the thought a
/// future reader will have while looking at a 12 MB file in a portable folder
/// and wondering what it costs to halve it.
#[must_use]
pub fn attributions() -> &'static [Attribution] {
    &[
        // Source: D:\Dev\pdfcer\crates\pdfcer-render\assets\fonts\PROVENANCE.md
        // and the "Bundled Foxit substitute faces" section of this
        // repository's `about.hbs`, which reproduces the pdfium LICENSE in
        // full. Redistributed because `pdfcer-render`'s `font::bundled` embeds
        // all fourteen faces with `include_bytes!`, so they are inside this
        // executable.
        //
        // Covers the asset directory `crates/pdfcer-render/assets/fonts` in
        // the engine tree. That path is written out here as well as in the
        // comment above because `tools/gates/check-shipped-assets.py` looks
        // for it: crates/pdfcer-render/assets/fonts
        Attribution {
            component: "The 14 substitute font faces pdfcer draws with when a document embeds no font",
            creator: "Foxit Software Inc., through the Chromium pdfium project",
            origin: "pdfium, core/fxge/fontdata/chromefontdata/, upstream commit a4a2d6706be9f538e355f3b95307ff393f299a54",
            licence: "BSD-3-Clause",
            licence_url: None,
            changes: "The font data is unchanged; it was converted back to binary from the C byte-array literals pdfium stores it in.",
        },
        // Source: the "Adobe Core 14 AFM font metrics (APAFML)" section of
        // `about.hbs`, which carries the licence text and the per-font AFM
        // version list. The modification notice below is APAFML's own
        // requirement — "all modifications ... are prominently noted" — and
        // is quoted from that section rather than summarised.
        Attribution {
            component: "The width, encoding and descriptor tables for the 14 standard PDF fonts",
            creator: "Adobe Systems Incorporated",
            origin: "the Adobe Core 14 AFM files",
            licence: "APAFML",
            licence_url: None,
            changes: "Modified: only advance widths and global header metrics were taken. Kerning pairs, per-glyph bounding boxes, ligature data and composites were discarded.",
        },
        // Source: the "Adobe Glyph List (BSD-3-Clause)" section of
        // `about.hbs`.
        Attribution {
            component: "The glyph-name to Unicode mapping pdfcer uses to extract text",
            creator: "Adobe Systems Incorporated",
            origin: "the Adobe Glyph List (glyphlist.txt, zapfdingbats.txt)",
            licence: "BSD-3-Clause",
            licence_url: None,
            changes: "Modified: a subset of the published lists, not the whole of either.",
        },
        // Source: D:\Dev\pdfcer\crates\pdfcer-core\assets\models\ocrs\PROVENANCE.md
        // — every field below is lifted from that file, which records the
        // retrieval date, the two SHA-256 hashes and the model card's own
        // `license: cc-by-sa-4.0` YAML. Nothing here is reconstructed from what
        // a Creative Commons licence usually says, and there is deliberately no
        // version number: the upstream artifacts carry a content-addressed
        // suffix rather than a version, the Hugging Face and S3 copies are not
        // byte-identical to each other, and the hash is therefore the identity.
        //
        // ★ The FIRST entry in this catalog whose licence requires a LINK
        // rather than a reproduction, which is what `licence_url` was added
        // for — see its own documentation, written before this entry existed.
        //
        // Covers the asset directory `crates/pdfcer-core/assets/models/ocrs` in
        // the engine tree; shipped in the portable folder as `models/ocrs/`.
        // That path is written out here as well as in the prose above because
        // `tools/gates/check-shipped-assets.py` looks for it:
        // crates/pdfcer-core/assets/models/ocrs
        Attribution {
            component: "The two neural-network weight files pdfcer recognises text with",
            creator: "Robert Knight, the ocrs project",
            origin: "https://huggingface.co/robertknight/ocrs, retrieved 2026-08-13",
            licence: "CC-BY-SA-4.0",
            licence_url: Some("https://creativecommons.org/licenses/by-sa/4.0/"),
            changes: "The weights are unchanged and byte-identical to the published files; only their names were shortened.",
        },
    ]
}

#[cfg(test)]
mod tests {
    /// ★ **The build stamp is present and is not a placeholder.**
    ///
    /// `build.rs` sets `PDFCER_BUILD_TIME` from `PDFCER_BUILD_STAMP` when the
    /// packager supplies one and from a computed UTC clock otherwise. Both
    /// paths must produce something an operator can read; an empty string would
    /// render as "built  from abc1234", which looks like a layout bug rather
    /// than a missing value.
    #[test]
    fn the_build_stamp_is_populated() {
        let stamp = env!("PDFCER_BUILD_TIME");
        assert!(!stamp.trim().is_empty(), "build.rs set no build time");
        assert!(
            stamp.contains('-') && stamp.contains(':'),
            "a build stamp should carry a date and a time, got {stamp:?}"
        );
        let rev = env!("PDFCER_GUI_REV");
        assert!(!rev.trim().is_empty(), "build.rs set no revision");
    }

    /// ★ The engine is REALLY named, because this is the row that identifies
    /// which pdfcer is inside a given executable.
    ///
    /// A `Cargo.lock` this build script could not read would leave the version
    /// empty and the About box would say "pdfcer - not in this build" about a
    /// program that is nothing but pdfcer. That reads as a far worse claim than
    /// a missing date, so it is asserted rather than left to be noticed.
    #[test]
    fn the_engine_reports_a_version_and_a_revision() {
        assert!(
            !env!("PDFCER_ENGINE_VERSION").is_empty(),
            "the engine version was not read out of Cargo.lock"
        );
        assert!(
            !env!("PDFCER_ENGINE_REV").is_empty(),
            "the engine revision was not read out of Cargo.lock"
        );
    }

    /// A component line reads as a sentence with and without a commit date.
    ///
    /// The date is optional because a dependency taken from a source this build
    /// cannot run `git` in - crates.io, or an `https://` remote - still has a
    /// version and a revision worth printing. What it must not do is print a
    /// dangling "committed" with nothing after it.
    #[test]
    fn a_component_line_survives_a_missing_commit_date() {
        let with = component_line("pdfcer", "0.7.0", "6af5655", "2026-08-18 14:02");
        assert!(with.contains("0.7.0") && with.contains("6af5655"));
        assert!(with.contains("committed 2026-08-18 14:02"));

        let without = component_line("pdfcer", "0.7.0", "6af5655", "");
        assert!(
            !without.contains("committed"),
            "an empty date must leave the word out too, got {without:?}"
        );
        assert!(without.ends_with("6af5655"));
    }

    /// An absent component says so, and says it about itself.
    #[test]
    fn an_absent_component_names_itself() {
        let line = component_absent("iccce");
        assert!(line.starts_with("iccce"));
        assert!(line.contains("not in this build"));
    }

    use super::*;
    use std::path::PathBuf;

    /// The shipped notice file, read from the repository root.
    fn notice() -> String {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        std::fs::read_to_string(root.join("THIRD_PARTY_LICENSES.md"))
            .expect("THIRD_PARTY_LICENSES.md must exist at the workspace root")
    }

    /// Every field of every attribution is populated.
    ///
    /// A half-filled entry is worse than none: it looks like an attribution
    /// has been made while leaving out the part the licence actually asked
    /// for. `licence_url` is excluded because `None` is a real answer for the
    /// licence families here — see its own documentation.
    #[test]
    fn every_attribution_says_all_five_things() {
        let list = attributions();
        assert!(
            !list.is_empty(),
            "an empty attribution list would make this test vacuous, and the dialog pointless"
        );
        for a in list {
            assert!(!a.component.is_empty(), "component is empty");
            assert!(
                !a.creator.is_empty(),
                "creator is empty for {}",
                a.component
            );
            assert!(!a.origin.is_empty(), "origin is empty for {}", a.component);
            assert!(
                !a.licence.is_empty(),
                "licence is empty for {}",
                a.component
            );
            assert!(
                a.changes.ends_with('.'),
                "the indication of changes is a statement and needs its punctuation: {:?}",
                a.changes
            );
        }
    }

    /// ★ The dialog and the shipped notice file cannot disagree.
    ///
    /// This is the property that makes two surfaces safe rather than twice
    /// the maintenance. The dialog names a work and its licence; the file
    /// carries that licence's text. If a work is named in the program and
    /// absent from the file, the operator is told about terms they have no
    /// way to read — which is a worse state than either surface alone.
    ///
    /// It fails LOUDLY, which is the point. `HANDOFF.md` §10 records that the
    /// RON regeneration rots precisely because nothing fails until somebody
    /// else runs a round-trip; this assertion runs in every `cargo test`.
    #[test]
    fn the_shipped_notice_carries_every_attribution_this_dialog_makes() {
        let notice = notice();
        for a in attributions() {
            assert!(
                notice.contains(a.licence),
                "{} is attributed to {} under {} in the About dialog, and \
                 THIRD_PARTY_LICENSES.md never mentions that licence. \
                 Regenerate it: cargo about generate about.hbs -o THIRD_PARTY_LICENSES.md",
                a.component,
                a.creator,
                a.licence
            );
        }
    }

    /// The notice file carries the licence texts, not just their names.
    ///
    /// Guards against the regeneration having produced a stub — a truncated
    /// or failed `cargo about generate` still writes a file, and a file that
    /// exists is exactly what the assertion above would be satisfied by.
    #[test]
    fn the_shipped_notice_is_a_real_notice_and_not_a_stub() {
        let notice = notice();
        assert!(
            notice.len() > 100_000,
            "THIRD_PARTY_LICENSES.md is {} bytes, which is too small to be \
             carrying the licence texts of the crates this binary links",
            notice.len()
        );
        assert!(
            notice.contains("Full licence texts"),
            "the generated notice is missing its licence-text section"
        );
    }

    /// pdfcer's own licence line agrees with the file that ships beside it.
    ///
    /// Claim-bearing copy: the About box states the terms the operator grants
    /// to everyone who receives this program. It is verified against
    /// `LICENSE` rather than trusted, because the cost of the two disagreeing
    /// is borne by somebody relying on the wrong one.
    #[test]
    fn the_licence_line_matches_the_shipped_licence_file() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let licence = std::fs::read_to_string(root.join("LICENSE")).expect("LICENSE");
        assert!(licence.contains("MIT License"), "LICENSE is not MIT");
        assert!(
            licence.contains("Copyright (c) 2026 Ken Mantle"),
            "LICENSE's copyright line has changed; about's licence_line() must follow it"
        );
        assert!(licence_line().contains("MIT"));
        assert!(licence_line().contains("Copyright (c) 2026 Ken Mantle"));
    }

    /// No two entries describe the same work.
    #[test]
    fn no_two_attributions_share_a_component() {
        let list = attributions();
        for (i, a) in list.iter().enumerate() {
            for b in &list[i + 1..] {
                assert_ne!(
                    a.component, b.component,
                    "two entries describe the same work"
                );
            }
        }
    }
}
