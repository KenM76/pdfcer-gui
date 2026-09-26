//! # `text::about` — the attribution surface, in the operator's own window
//!
//! Every word the About dialog shows, plus the **structured attribution
//! catalog** it draws from. Consumed by `pdfcer_gui::dialogs::about`, and by the
//! test that pins this catalog against the shipped
//! `THIRD_PARTY_LICENSES.md`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/about.md`.

/// One third-party work `pdfcer-gui.exe` redistributes, and everything an
/// attribution-style licence asks be said about it.
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
#[must_use]
pub fn product() -> &'static str {
    "pdfcer"
}

/// The version line, when this build **is** a released version.
#[must_use]
pub fn version_line(version: &str) -> String {
    format!("Version {version}")
}

/// The version line when the build is **past** a release rather than at one.
#[must_use]
pub fn version_line_after(version: &str, commits: u32, modified: bool) -> String {
    let since = match (commits, modified) {
        (0, _) => "with uncommitted changes".to_owned(),
        (1, false) => "plus 1 commit".to_owned(),
        (1, true) => "plus 1 commit and uncommitted changes".to_owned(),
        (n, false) => format!("plus {n} commits"),
        (n, true) => format!("plus {n} commits and uncommitted changes"),
    };
    format!("Version {version}, {since} — not the released build")
}

/// What the headline says when there is **no** release version to show.
#[must_use]
pub fn version_unreleased() -> &'static str {
    "No released version — the build details below identify this program."
}

// ===========================================================================
// Build provenance
// ===========================================================================

/// Heading for the block naming when this was built and what is inside it.
#[must_use]
pub fn build_heading() -> &'static str {
    "Build"
}

/// The line for this program's own build.
#[must_use]
pub fn build_line(stamp: &str, rev: &str) -> String {
    format!("pdfcer-gui — built {stamp} from {rev}")
}

/// The line for a component compiled INTO this program.
#[must_use]
pub fn component_line(name: &str, version: &str, rev: &str, committed: &str) -> String {
    let mut line = format!("{name} {version} — revision {rev}");
    if !committed.is_empty() {
        line.push_str(&format!(", committed {committed}"));
    }
    line
}

/// The line for a component that is **not part of this build**.
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
#[must_use]
pub fn summary() -> &'static str {
    "A PDF viewer and editor. The pdfcer engine is built into this program; nothing else is required to run it."
}

/// pdfcer's own licence, and its copyright line.
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
        // The only entry whose licence requires a LINK rather than a
        // reproduction, which is what `licence_url` exists for — see its own
        // documentation.
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
        // Source: D:\Dev\OCRcer\LICENSE and NOTICE, which ship beside the
        // model; `tools/package-portable.py`'s `PAYLOAD_OCRCER_FILES` copies
        // all three. `check-shipped-assets.py` looks for the destination
        // directory: models/ocrcer
        Attribution {
            component: "The OCRcer recognition model, the second OCR engine",
            creator: "Ken Mantle, the OCRcer project",
            origin: "the OCRcer repository's model build; its LICENSE and NOTICE ship beside it in models/ocrcer",
            licence: "MIT",
            licence_url: None,
            changes: "The model file is shipped as OCRcer builds it.",
        },
    ]
}
