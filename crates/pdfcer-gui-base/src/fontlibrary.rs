//! # `fontlibrary` — turning the operator's font folders into donors an embed
//! can use
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/fontlibrary.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pdfcer_render::FontEnvironment;

/// The largest font file this will read, in bytes.
pub const MAX_FONT_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// The extensions this will attempt.
const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "pfb", "cff"];

/// One font program that can stand in for a face a document is missing.
#[derive(Debug, Clone)]
pub struct Donor<'a> {
    /// The file it came from, or `None` for one of pdfcer's **own** faces.
    ///
    /// `Option`, and the `None` is not an absence of information — it is a
    /// different KIND of donor. A bundled face has no path because it was never
    /// on this machine's disk; it is compiled into the program. Reporting an
    /// empty string, or the executable's own path, would both be answers to a
    /// question the operator did not ask. [`Donor::source`] turns it into the
    /// sentence instead.
    pub path: Option<&'a Path>,
    /// The name that matched — the document's own, an equivalent family's, the
    /// file's stem, or a bundled face's own label.
    ///
    /// Owned, unlike the rest of this struct, and the reason is the bundled
    /// rung: the engine returns that name in a value that dies with the lookup,
    /// so there is nothing for a borrow to point at. One `String` per missing
    /// font, a handful of times per embed — measured against the alternative,
    /// which is a `Cow` in a public type to save an allocation nobody can find.
    pub face_name: String,
    /// The program bytes, exactly as they will be embedded.
    pub program: &'a [u8],
    /// **How** it matched, which the operator is owed.
    pub matched: Match,
}

/// How a donor was matched to a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// A file advertising the name the document spells, tag stripped.
    Exact,
    /// **One of the faces pdfcer itself ships**, used because nothing the
    /// operator pointed pdfcer at could answer.
    ///
    /// The most inferred rung, and the engine says so in as many words:
    /// *"nothing on the operator's machine was consulted."* Offered only
    /// because the operator asked for it — `OPERATOR_REQUESTS.md` **O47**,
    /// answered *"yes"* on 2026-08-28 — and disclosed loudly wherever it fires,
    /// because a document that goes out with pdfcer's Helvetica substitute in it
    /// looks different from one with the operator's own, and nothing on the
    /// canvas says which happened.
    Bundled,
    /// A **standard-14 family equivalence** — the document says `Helvetica` and
    /// the folder holds `Arial`. Metric-compatible by design, and the advances
    /// come from `/Widths` regardless, but the letterforms are a different
    /// designer's.
    Alias,
    /// The file's **filename stem** matched where its advertised names did not.
    /// The weakest answer, and named separately so it can be disclosed.
    Stem,
}

impl<'a> Donor<'a> {
    /// Where this donor came from, for the engine's `SuppliedFont::source` and
    /// for the operator's row.
    #[must_use]
    pub fn source(&self) -> String {
        self.path.map_or_else(
            || crate::text::fonts::bundled_source(&self.face_name),
            |p| p.display().to_string(),
        )
    }
}

impl Match {
    /// Whether this is something other than the face the document named.
    #[must_use]
    pub const fn is_inferred(self) -> bool {
        !matches!(self, Self::Exact)
    }
}

/// Everything the configured folders offer.
#[derive(Debug, Default)]
pub struct Library {
    /// The engine's resolver, populated by [`Self::scan`].
    env: FontEnvironment,
    /// Every registered name → the file it came from.
    ///
    /// A `BTreeMap` rather than a `HashMap`: iteration order is stable, and
    /// this is read to build a report an operator compares between runs.
    paths: BTreeMap<String, PathBuf>,
    /// The names that came **only** from a filename stem. See the header.
    stems: BTreeSet<String>,
    /// Whether pdfcer's **own** standard-14 faces may answer.
    ///
    /// `false` unless the operator asked. See [`Library::scan`].
    allow_bundled: bool,
    /// Files that were skipped and why, in the order they were met.
    ///
    /// Kept rather than discarded, because *"pdfcer could not embed
    /// HelveticaNeue"* and *"pdfcer skipped HelveticaNeue.ttf because it is 40
    /// MB"* are the same event to the program and completely different events
    /// to the operator. The second is actionable.
    pub skipped: Vec<String>,
}

impl Library {
    /// Read every font file in `folders`, in order, and index what they offer.
    #[must_use]
    pub fn scan(folders: &[PathBuf]) -> Self {
        Self::scan_with(folders, false)
    }

    /// [`Self::scan`], and whether pdfcer's **own** faces may answer when the
    /// folders cannot.
    #[must_use]
    pub fn scan_with(folders: &[PathBuf], allow_bundled: bool) -> Self {
        // `bundled()` rather than an empty environment, and it is safe: the
        // bundled faces live in the FALLBACK table, which
        // `resolve_for_embedding` consults only when it is passed
        // `allow_bundled`. It is passed `false` in [`Self::donor_for`], every
        // time, for the header's reason — and there is a test that presses on
        // exactly that.
        let mut library = Self {
            env: FontEnvironment::bundled(),
            allow_bundled,
            ..Self::default()
        };
        for folder in folders {
            library.scan_one(folder);
        }
        library
    }

    fn scan_one(&mut self, folder: &Path) {
        let entries = match std::fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(error) => {
                // A folder that will not open is a **note, not a failure**.
                // A removable drive that is not mounted is still where the
                // operator's fonts live — `prefs::fonts::add`'s stated position
                // — so the honest response is to say so and search the rest.
                self.skipped.push(crate::text::fonts::folder_unreadable(
                    folder,
                    &error.to_string(),
                ));
                return;
            }
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_file() && has_font_extension(path))
            .collect();
        // See the module header on determinism.
        files.sort();
        for path in files {
            self.read_one(&path);
        }
    }

    fn read_one(&mut self, path: &Path) {
        if let Ok(meta) = std::fs::metadata(path)
            && meta.len() > MAX_FONT_FILE_BYTES
        {
            self.skipped
                .push(crate::text::fonts::file_too_large(path, meta.len()));
            return;
        }
        let Ok(bytes) = std::fs::read(path) else {
            self.skipped.push(crate::text::fonts::file_unreadable(path));
            return;
        };
        // Parsed ONCE, and the borrow ends before the bytes are stored —
        // `pdfcer` notes the same discipline against R21. A second parse to
        // re-read a name would double the cost of a scan over a system font
        // folder, which is the case this is most likely to meet.
        let names = match pdfcer_render::font::program::FontProgram::parse(&bytes) {
            Ok(program) => program.face_names(),
            Err(error) => {
                self.skipped
                    .push(crate::text::fonts::not_a_font(path, &error.to_string()));
                return;
            }
        };
        let stem = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned);
        if names.is_empty() && stem.is_none() {
            self.skipped.push(crate::text::fonts::no_name(path));
            return;
        }
        // The bytes are wrapped ONCE and every `FontData` clone below is an
        // `Arc` clone. A file advertising four names would otherwise be four
        // full copies of one face in memory, and a system font folder holds
        // thousands of files.
        let data = pdfcer_render::FontData::new(bytes);
        for name in &names {
            self.offer(name, path, &data, false);
        }
        // The filename stem, as a FALLBACK and recorded as one.
        // `pdfcer` registers it too — *"so a match works even when the
        // internal name is odd or absent"* — and the difference here is that
        // this shell has to tell an operator which happened, because a stem
        // match is this program deciding a file called `Helv.ttf` is Helvetica.
        if let Some(stem) = stem
            && !names.contains(&stem)
        {
            self.offer(&stem, path, &data, true);
        }
    }

    /// Record `name` → this file, unless an earlier folder already claimed it.
    fn offer(&mut self, name: &str, path: &Path, data: &pdfcer_render::FontData, from_stem: bool) {
        if self.paths.contains_key(name) {
            return;
        }
        self.paths.insert(name.to_owned(), path.to_path_buf());
        if from_stem {
            self.stems.insert(name.to_owned());
        }
        self.env.insert_named(name, data.clone());
    }

    /// The donor for a document's `/BaseFont`, if the folders hold one.
    #[must_use]
    pub fn donor_for(&self, base_font: &str) -> Option<Donor<'_>> {
        let hit = self
            .env
            .resolve_for_embedding(base_font, self.allow_bundled)?;
        // A bundled face has no entry here and that is how it is
        // RECOGNISED, rather than by matching on `hit.quality`.
        //
        // The two agree today and the map is the safer of the two to ask,
        // because it answers a question about **this** library: a name the
        // walk never registered cannot have come off a folder, whatever the
        // engine calls the rung it took. If a future engine version reached a
        // bundled face under some other quality, this still reports it as
        // bundled — and the failure mode of the alternative is a face compiled
        // into pdfcer being disclosed as a file on the operator's disk.
        let Some((name, path)) = self.paths.get_key_value(hit.face_name.as_str()) else {
            return Some(Donor {
                path: None,
                face_name: hit.face_name,
                program: hit.data.bytes(),
                matched: Match::Bundled,
            });
        };
        let matched = match hit.quality {
            // The re-grade the header explains. The engine says `Exact` for a
            // stem hit because to a renderer the two are the same question; to
            // a disclosure they are not.
            pdfcer_render::font::EmbedMatch::Exact if self.stems.contains(name) => Match::Stem,
            pdfcer_render::font::EmbedMatch::Exact => Match::Exact,
            // `Bundled` is unreachable — `allow_bundled` is `false` — and it is
            // folded in with `Alias` rather than given an arm that claims to
            // handle it. Both mean "not the face the document named", which is
            // the only thing a caller does with this value.
            _ => Match::Alias,
        };
        Some(Donor {
            path: Some(path),
            face_name: name.clone(),
            program: hit.data.bytes(),
            matched,
        })
    }

    /// How many distinct names the folders answer to.
    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Whether the folders offered nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

/// Whether the path's extension is one this will attempt.
pub(crate) fn has_font_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|e| FONT_EXTENSIONS.contains(&e.as_str()))
}

/// A `/BaseFont` without its §9.6.4 subset tag.
#[must_use]
pub fn strip_subset_tag(base_font: &str) -> &str {
    match base_font.split_once('+') {
        Some((tag, rest)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => base_font,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A four-byte non-font, for the tests that only exercise the index.
    fn stub() -> pdfcer_render::FontData {
        pdfcer_render::FontData::new(vec![0u8; 4])
    }

    /// **A subset tag is stripped and nothing else is.**
    #[test]
    fn only_a_real_subset_tag_is_stripped() {
        assert_eq!(strip_subset_tag("ABCDEF+ArialMT"), "ArialMT");
        assert_eq!(strip_subset_tag("ArialMT"), "ArialMT");
        assert_eq!(strip_subset_tag("ABCDE+ArialMT"), "ABCDE+ArialMT");
        assert_eq!(strip_subset_tag("abcdef+ArialMT"), "abcdef+ArialMT");
        assert_eq!(strip_subset_tag("Foo+Bar"), "Foo+Bar");
    }

    /// **Only real font extensions are attempted.**
    #[test]
    fn a_collection_is_not_offered_as_a_donor() {
        assert!(has_font_extension(Path::new("C:/f/Arial.ttf")));
        assert!(has_font_extension(Path::new("C:/f/Arial.OTF")));
        assert!(!has_font_extension(Path::new("C:/f/Cambria.ttc")));
        assert!(!has_font_extension(Path::new("C:/f/Cambria.otc")));
        assert!(!has_font_extension(Path::new("C:/f/readme.txt")));
        assert!(!has_font_extension(Path::new("C:/f/Arial")));
    }

    /// **The first folder holding a name keeps it.**
    #[test]
    fn the_first_folder_to_offer_a_name_keeps_it() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/first/Arial.ttf"), &stub(), false);
        library.offer("ArialMT", Path::new("C:/second/Arial.ttf"), &stub(), false);
        let donor = library.donor_for("ArialMT").expect("indexed");
        assert_eq!(donor.path, Some(Path::new("C:/first/Arial.ttf")));
    }

    /// **A tagged `/BaseFont` finds an untagged donor.**
    ///
    /// The case that matters on real documents: a subsetted face is what needs
    /// embedding, and its name never matches a font file's.
    #[test]
    fn a_subsetted_base_font_finds_its_donor() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), &stub(), false);
        assert!(library.donor_for("ABCDEF+ArialMT").is_some());
        assert!(library.donor_for("ArialMT").is_some());
        assert!(library.donor_for("Wingdings").is_none());
    }

    /// **`Helvetica` resolves to Arial, and it is graded as a substitute.**
    ///
    #[test]
    fn helvetica_finds_arial_and_says_it_is_a_substitute() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), &stub(), false);
        let donor = library.donor_for("Helvetica").expect("the alias rung");
        assert_eq!(donor.face_name, "ArialMT");
        assert_eq!(donor.matched, Match::Alias);
        assert!(donor.matched.is_inferred());
    }

    /// **A stem match is re-graded, where the engine calls it exact.**
    #[test]
    fn a_stem_match_is_distinguishable_from_an_exact_one() {
        let mut library = Library::scan(&[]);
        library.offer("Helvetica", Path::new("C:/f/Helvetica.ttf"), &stub(), false);
        library.offer("Helv", Path::new("C:/f/Helv.ttf"), &stub(), true);
        assert_eq!(
            library.donor_for("Helvetica").expect("exact").matched,
            Match::Exact
        );
        assert_eq!(
            library.donor_for("Helv").expect("stem").matched,
            Match::Stem
        );
    }

    /// **A bundled face is offered ONLY when it was asked for.**
    #[test]
    fn a_bundled_face_answers_only_when_it_is_allowed_to() {
        let refusing = Library::scan_with(&[], false);
        assert!(refusing.donor_for("Helvetica").is_none());
        assert!(refusing.donor_for("Times-Roman").is_none());
        assert!(refusing.donor_for("Courier").is_none());
        assert!(
            Library::scan(&[]).donor_for("Helvetica").is_none(),
            "`scan` must be the refusing form: it is what a caller reaches for \
             without thinking about the question"
        );

        let allowing = Library::scan_with(&[], true);
        let donor = allowing
            .donor_for("Helvetica")
            .expect("pdfcer ships a standard-14 substitute for Helvetica");
        assert_eq!(donor.matched, Match::Bundled);
        assert!(
            donor.path.is_none(),
            "a bundled face has no path — it was never on this machine's disk"
        );
        assert!(
            donor.source().contains("pdfcer's own"),
            "the source must say whose face it is: {}",
            donor.source()
        );
        assert!(!donor.program.is_empty(), "the bundled bytes are real");
    }

    /// **A real folder still beats a bundled face.**
    #[test]
    fn a_configured_face_outranks_pdfcers_own() {
        let mut library = Library::scan_with(&[], true);
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), &stub(), false);
        let donor = library.donor_for("Helvetica").expect("resolved");
        assert_eq!(
            donor.matched,
            Match::Alias,
            "the bundled rung fired ahead of a real face"
        );
        assert_eq!(donor.path, Some(Path::new("C:/f/Arial.ttf")));
    }

    /// **A folder that will not open is a note, not a panic and not a stop.**
    #[test]
    fn an_unreadable_folder_is_noted_and_the_rest_are_searched() {
        let library = Library::scan(&[
            PathBuf::from("C:/definitely/not/here/at/all"),
            PathBuf::from("C:/nor/this/one"),
        ]);
        assert_eq!(
            library.skipped.len(),
            2,
            "both were noted: {:?}",
            library.skipped
        );
        assert!(library.is_empty());
    }
}

#[cfg(test)]
mod real_files {
    use super::*;

    /// **The scan reads a real font folder and finds real faces.**
    #[test]
    fn a_real_font_folder_yields_real_faces() {
        let dir = PathBuf::from(r"C:\Windows\Fonts");
        if !dir.is_dir() {
            eprintln!("no system font directory on this machine — skipped");
            return;
        }
        let library = Library::scan(&[dir]);
        assert!(
            !library.is_empty(),
            "a system font folder yielded no faces at all, which means the parse link is dead. \
             Skips: {:?}",
            library.skipped.iter().take(5).collect::<Vec<_>>()
        );
        // Printed rather than asserted on. Measured on the development
        // machine at **3,359 indexed names from one skip**, which is the number
        // that made this test evidence rather than a green tick — a build whose
        // parser was dead would index the filename stems alone and still be
        // "not empty". Not asserted, because it is a fact about somebody's
        // Windows install and would pin this test to a machine.
        eprintln!(
            "indexed {} name(s), {} skip(s)",
            library.len(),
            library.skipped.len()
        );
        // A name every Windows machine carries, matched the way a document
        // would spell it. Asserting a SPECIFIC face rather than a count is what
        // makes this a test of the join rather than of `read_dir`.
        assert!(
            library.donor_for("ABCDEF+ArialMT").is_some() || library.donor_for("Arial").is_some(),
            "neither `Arial` nor a subsetted `ArialMT` resolved out of {} indexed name(s)",
            library.len()
        );
        // **The claim this whole rewrite rests on, on a real machine.**
        //
        // `Helvetica` is what the fixture asks for and what every CAD exporter
        // writes; nothing on Windows advertises that name. If the alias rung
        // works, this resolves to an Arial out of the system folder and reports
        // itself as a substitute. If it does not, embedding a CAD drawing on
        // this platform does nothing at all — which is the state this module
        // shipped in for exactly one commit.
        let donor = library
            .donor_for("Helvetica")
            .expect("no donor for Helvetica out of a real Windows font folder");
        assert!(
            donor.matched.is_inferred(),
            "a real Helvetica was found, which no Windows machine has — the grading is wrong"
        );
        eprintln!(
            "Helvetica -> {} ({:?}) from {}",
            donor.face_name,
            donor.matched,
            donor.source()
        );
    }
}
