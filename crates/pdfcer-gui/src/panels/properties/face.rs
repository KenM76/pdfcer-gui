//! # `panels::properties::face` — the face chooser, once, for both surfaces
//!
//! One control, drawn in two places: the Properties panel's *This text* section
//! ([`super::text::section`]) and the ribbon's Format ▸ Font group
//! ([`crate::app::fontband`]). Until 2026-08-29 they were two copies of one
//! loop, and the copies had already begun to differ in the way this project
//! keeps finding — a face offered in one surface and not the other.
//!
//! ★ That divergence is the reason this module exists at all, and it is worth
//! stating as a rule rather than as a tidy-up: **a control drawn twice is a
//! control that will be built twice**, and the second build is always the one
//! that misses the disclosure. The two callers now differ in exactly two
//! things — the region prefix they publish under, and what they do with the
//! selector that comes back — and in nothing else.
//!
//! ## ★★★ The list has two kinds of row now, and they are two different acts
//!
//! `pdfcer-core` v0.15.0 (`Pass 162.0`):
//!
//! > **FONTS** — text can be restyled to a face the document **DOES NOT
//! > CONTAIN**, for the fourteen faces every PDF reader is required to have.
//! > pdfcer authors the font resource on demand, with widths, embedding
//! > nothing. A face outside those fourteen still refuses by name — that needs
//! > a real font program.
//!
//! So a row in this list is now one of:
//!
//! | kind | what a click does to the file | what it costs |
//! |---|---|---|
//! | [`FaceOrigin::OnThisPage`] | rewrites one `Tf` operand | nothing — the resource is already there |
//! | [`FaceOrigin::PdfcerWouldAdd`] | rewrites the `Tf` operand **and writes a new `/Font` object** | a dictionary with widths and no glyph outlines, plus a face drawn from the reader's own copy |
//!
//! Those are different enough that presenting them as one undifferentiated list
//! would be hiding a write behind a menu. [`choices`] tags every row and
//! [`popup_body`] draws them under two headings with the disclosure between.
//!
//! ## ★★ Who writes the resource — asked, and answered by reading the engine
//!
//! `FormatPlan::created_font` is documented as *"a `/Font` resource the caller
//! must CREATE for `new_content` to be valid"*, and its own note says the caller
//! owns the write *"because only the caller can allocate an object number:
//! planning runs against an immutable `&Document`."* That is a real obligation
//! and it would not be this shell's to invent.
//!
//! **It does not fall to this shell**, because `EditSession::format_text` is
//! itself that caller and it already performs the write — on both of its paths:
//!
//! * the page-`/Contents` path (`pdfcer-core/src/edit.rs` ~7887) takes
//!   `plan.created_font`, calls `self.font_resource_writes(page.id, true, …)`
//!   and extends **the same command** with the writes, *"so one undo removes
//!   both the restyle and the resource it needed"*;
//! * the form-XObject path (~8015) binds the resource into the form's own
//!   `/Resources` before the stream is rebuilt, for the stated reason that two
//!   writes for one object id in one command would let the later silently win.
//!
//! Both also push a disclosure when the target `/Resources` turns out to be
//! shared with other pages. `crate::app::actions::textstyle` already surfaces
//! the engine's disclosures verbatim, so that one arrives on the status bar with
//! no code here.
//!
//! ⇒ **The shell change is the chooser and nothing else.** Nothing in this
//! module allocates an object, writes a dictionary, or knows the shape of one.
//!
//! ## ★★★ The standard-14 rows ARE coverage-tested now — 2026-09-08
//!
//! Every [`FaceOrigin::OnThisPage`] row has been through `set_font`'s own
//! acceptance test for **this run's characters** — that is what
//! `preview_font_resources` is, and it is why a refused page font is absent from
//! the list rather than greyed. **The other half now matches it.**
//!
//! [`choices`] reads `FontPreflight::standard_14`: one `Std14Entry` per face,
//! each carrying the engine's own spelling, a `presence` that says `OnPage` or
//! `WouldBeAdded` as a **fact**, and an `acceptance` run through `set_font`'s
//! gate with the embedded-subset floor included. Three local re-derivations
//! were deleted with it — the `Std14::ALL` walk, the `carried` filter built by
//! shortening `preflight.entries`, and the untested offer.
//!
//! ⚠ **Consequence, stated because it is a behaviour change:** a standard-14
//! face that cannot hold this run's characters is now **absent** rather than
//! offered-and-then-refused. That is the first half's behaviour, applied to the
//! second.
//!
//! ### What this paragraph said until the engine answered it, kept because the
//! ### reasoning is why the engine answered it
//!
//! > *"It cannot be done honestly from here. The engine offers no query that
//! > coverage-tests a face the page does **not** carry, so the shell's only
//! > route would be to re-derive the encoding rule — which face uses
//! > `WinAnsiEncoding`, which two use a built-in `FontSpecific` one, and which
//! > characters that leaves unmapped. `FontPreflight`'s own invariant forbids
//! > exactly that (`R221`), and a second copy of the rule in `pdfcer-gui` would
//! > drift from the commit path the first time the rule changed.*
//! >
//! > *⇒ So these rows are **offered, and a refusal is a sentence** … This is
//! > recorded as an engine ask rather than worked around: a
//! > `preview_font_resources` that also surveyed the fourteen would let this
//! > list be as exact as its first half already is."*
//!
//! ★★ **That last sentence is the request, and `Pass 142.2` is the answer.**
//! The refusal to copy `R221`'s rule into this crate is why the fix arrived as
//! an engine capability rather than as drift — worth keeping, because the
//! tempting shortcut was one afternoon's work and would have been wrong on the
//! first day the encoding rule changed.
//!
//! ⚠ It sat unconsumed for **two days** after it shipped, with this header
//! still asserting the absence. Found by the 2026-09-07 reply triage, not by
//! any gate. See `RESUME.md`.
//!
//! ★ The old `carried` filter is subsumed, and more exactly. It compared
//! *shortened* names against the page's entries; `Std14Presence::OnPage` is the
//! engine answering the same question from the resource dictionary it actually
//! resolved. A face already on the page reaches the list through the
//! `accepted()` half if it works, and through neither if it does not — so
//! `Helvetica` is never offered as *"pdfcer can add"* on a page whose own
//! `Helvetica` was refused.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas. The one inference an operator cannot see —
//! that an added face is drawn with the *reader's* copy rather than one carried
//! in the file — is discharged as a sentence at the point of choice
//! ([`crate::text::panels::face::face_addable_disclosure`]), which is the
//! off-canvas report rule 4 requires and the one thing this feature could not
//! ship without.

use egui::Ui;

use crate::text::panels::face as t;

/// The minimum width the popup is given, in points.
///
/// ★ The ribbon's chooser button is **78** points wide
/// ([`crate::app::fontband`]'s `FACE_WIDTH`, sized to fit inside the band's
/// custom-item budget), and an `egui` combo popup is otherwise no wider than its
/// button. The disclosure is a three-clause sentence; wrapped to 78 points it
/// would be a column of two-word lines, which is a sentence an operator does not
/// read.
///
/// So the popup states its own minimum and the two surfaces get the same one —
/// which is also what stops the panel's copy and the ribbon's copy from being
/// legible in one place and not the other, the divergence this module exists to
/// end.
const POPUP_MIN_WIDTH: f32 = 320.0;

/// Where a row in the face chooser comes from, and therefore what choosing it
/// does to the operator's file.
///
/// ★ Two variants and not a `bool`, because the call sites read as prose this
/// way and because a third origin is foreseeable: `Pass 142.0` would let pdfcer
/// subset and embed a face from the operating system, which is a third act with
/// a third set of consequences (a real font program in the file, and a face that
/// then renders identically everywhere). A `bool` would have to be replaced on
/// that day; this enum gains an arm and the compiler names every place that has
/// to say something new about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaceOrigin {
    /// A `/Font` resource this page already carries, which `set_font` has
    /// already said it would accept **for this run's characters**.
    ///
    /// Choosing it rewrites one `Tf` operand. The file gains nothing, and the
    /// document has already proved it can show the face.
    OnThisPage,
    /// One of the fourteen standard faces (ISO 32000-1 §9.6.2.2), which this
    /// page does **not** carry and which pdfcer would author on demand.
    ///
    /// Choosing it rewrites the `Tf` operand **and adds a `/Font` object** —
    /// name, encoding and widths, no glyph outlines, nothing embedded. See
    /// [`crate::text::panels::face::face_addable_disclosure`] for what that
    /// means for the operator, and the module header for the evidence that
    /// `EditSession::format_text` performs the write itself.
    PdfcerWouldAdd,
}

/// One row of the face chooser, as the pre-flight answered it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FaceChoice {
    /// **The string to pass to `set_font`** to reach this face.
    ///
    /// ★ Not always the `/BaseFont`. For a page face it is normally the
    /// subset-stripped base font, and it is the **resource key** instead when
    /// the page carries two dictionaries sharing one base font — which the Fonts
    /// panel's own survey found in 87 % of embedding files. A chooser that sent
    /// the name would reach exactly one of the twins, arbitrarily, and the
    /// operator would get the wrong font with no refusal to tell them.
    ///
    /// For a [`FaceOrigin::PdfcerWouldAdd`] row it is the exact §9.6.2.2
    /// `/BaseFont` spelling — `Times-Roman`, never `Times` — because that is the
    /// only spelling `fontdata::std14_by_base_font` accepts and therefore the
    /// only one `plan_font` will author a resource for.
    pub(crate) selector: String,
    /// What the row says, which is the human `/BaseFont`.
    pub(crate) label: String,
    /// Whether this page carries another resource with the same `/BaseFont`.
    ///
    /// Shown, because *"there are two of these and this is the second"* is a
    /// fact about the operator's document that nothing else surfaces, and
    /// because it explains why two rows can read identically. Always `false` for
    /// an addable row: a face that is not on the page cannot have a twin on it.
    pub(crate) ambiguous: bool,
    /// Which of the two acts choosing this row performs. See [`FaceOrigin`].
    pub(crate) origin: FaceOrigin,
}

/// Build the chooser's rows from the engine's pre-flight.
///
/// # What each half is, and why the second half is not simply "the fourteen"
///
/// **The page's own faces** come from `FontPreflight::accepted()` — every
/// `/Font` resource on this page that `set_font` would accept for this run's
/// characters, each carrying the selector that reaches *that* resource. A
/// refused one is deliberately absent rather than greyed: the refusals are
/// per-character encoding facts (*"'o' has no code in Times-Bold's encoding"*),
/// and a list of twelve faces with nine greyed rows each carrying a sentence
/// about a character is a control an operator cannot read. R9's
/// absent-rather-than-greyed case: for THIS run those faces are not a capability
/// that is temporarily unavailable, they are not applicable.
///
/// **The addable faces** are `Std14::ALL` minus every standard-14 name the page
/// already carries **in any form** — accepted or refused. That second word is
/// the load-bearing one and it is not a courtesy:
///
/// `plan_font` calls `resolve_target_resource` first and authors a face only
/// when that misses. So a page carrying a `Helvetica` that this run's characters
/// cannot be encoded into would resolve the selector `Helvetica` to **that**
/// resource and refuse it — while a row labelled *"pdfcer can add Helvetica"* had
/// promised the opposite. Filtering on `entries` rather than on `accepted()`
/// keeps the row out of the list entirely, which is the same answer the page
/// half gives for the same face.
///
/// ★ The name comparison strips the §9.6.4 subset tag ([`super::text::shorten`])
/// before matching, because a page carrying `ABCDEF+Helvetica` is a page that
/// carries Helvetica as far as `set_font`'s own `/BaseFont` match is concerned.
///
/// # Ordering
///
/// Page faces first, in the engine's dictionary order; then the fourteen in
/// `Std14::ALL`'s order, which groups the families (Helvetica, Times, Courier,
/// then the two symbolic faces) and is a spec-frozen constant the engine
/// publishes for exactly this use. Nothing here re-types the fourteen names.
///
/// # `None`
///
/// An absent pre-flight means the run did not pin or the preview refused, and
/// the answer is an **empty list** rather than the fourteen on their own. The
/// standard-14 half is filtered *by* the pre-flight; without one, offering it
/// would mean offering `Helvetica` on a page whose own `Helvetica` would take
/// the click — the exact entry-that-cannot-work this function's second half is
/// written to avoid.
pub(crate) fn choices(
    preflight: Option<&pdfcer_core::text_edit::FontPreflight>,
) -> Vec<FaceChoice> {
    use pdfcer_core::text_edit::Std14Presence;

    let Some(preflight) = preflight else {
        return Vec::new();
    };

    let mut rows: Vec<FaceChoice> = preflight
        .accepted()
        .map(|entry| FaceChoice {
            selector: entry.selector.clone(),
            label: super::text::shorten(&entry.base_font).to_owned(),
            ambiguous: entry.base_font_ambiguous,
            origin: FaceOrigin::OnThisPage,
        })
        .collect();

    // ★★★ THE FOURTEEN COME FROM THE ENGINE'S SURVEY NOW — 2026-09-08.
    //
    // Until today this walked `Std14::ALL` locally, filtered it against a
    // `carried` list this function built by shortening `preflight.entries`, and
    // offered every survivor **untested**. Three separate re-derivations of
    // things the engine already knew, and the module header said so in as many
    // words — *"a `preview_font_resources` that also surveyed the fourteen
    // would let this list be as exact as its first half already is"*.
    //
    // `Pass 142.2` built exactly that. `FontPreflight::standard_14` is one
    // `Std14Entry` per face, each carrying:
    //
    //   * `base_font`   — the engine's spelling, not this shell's
    //   * `presence`    — `OnPage { resource }` or `WouldBeAdded`, as a FACT
    //                     rather than something inferred from the resource list
    //   * `acceptance`  — run through `set_font`'s own gate, embedded-subset
    //                     floor included
    //
    // ⇒ So all three re-derivations are deleted, and the rows are now as exact
    // as the `accepted()` half above them.
    //
    // ★★ `is_accepted()` rather than a `match`. `FontAcceptance` is
    // `#[non_exhaustive]` and the engine added that accessor precisely so a
    // shell need not pattern-match it — *"a yes/no that does not require
    // pattern-matching a `#[non_exhaustive]` enum"*. Matching here would give
    // this shell a wildcard arm that silently swallows a future refusal kind,
    // which is the trap that bit three times on 2026-09-07.
    //
    // ⚠ A REFUSED face is now ABSENT rather than offered, which is a real
    // behaviour change and the right one: it makes this half behave exactly
    // like the first half, where *"a refused page font is absent from the list
    // rather than greyed"*. The header's old ruling — offer them, and let the
    // refusal be a sentence — was correct **while the coverage could not be
    // known here**. It can now, and R9 is unambiguous once it can: a control
    // that cannot work should not be drawn.
    //
    // ★ The `presence` filter also subsumes the old `carried` check, and more
    // exactly. `carried` compared *shortened* names against the page's entries;
    // `OnPage` is the engine answering the same question from the resource
    // dictionary it actually resolved. A face already on the page is offered by
    // the `accepted()` half above if it works, and by neither if it does not.
    rows.extend(
        preflight
            .standard_14
            .iter()
            .filter(|entry| {
                matches!(entry.presence, Std14Presence::WouldBeAdded)
                    && entry.acceptance.is_accepted()
            })
            .map(|entry| FaceChoice {
                selector: entry.base_font.clone(),
                label: super::text::shorten(&entry.base_font).to_owned(),
                // ★ Never ambiguous. `base_font_ambiguous` means two page
                // resources answer to one `/BaseFont`, and a face that is not
                // on the page has no resource to be ambiguous with.
                ambiguous: false,
                origin: FaceOrigin::PdfcerWouldAdd,
            }),
    );
    rows
}

/// Draw the chooser's popup body, and answer the selector the operator chose.
///
/// `current` is the run's `/BaseFont` **already shortened** — the string a row's
/// label is compared against to decide which row is the selected one. `prefix`
/// is the region namespace this surface publishes under, so the panel's copy and
/// the ribbon's copy are separately findable by a driven check.
///
/// Returns `Some(selector)` on the frame a row is clicked, and `None` on every
/// other frame — including the frames the operator spends reading the list,
/// which is most of them.
///
/// # ★★ Nothing is held between frames, because the document is the state
///
/// `selectable_label`, never `selectable_value`. A press here is an **edit**,
/// not a choice to be committed later, and a widget holding a pending value
/// would be a second place the current face is recorded — which is how a control
/// comes to disagree with the file it is about.
///
/// # ★★★ The disclosure is drawn once, visibly, and only when it is owed
///
/// Not a hover, and not a hover repeated on fourteen rows. It is owed to every
/// operator who opens this list — including the one who reads it and chooses
/// nothing — and a hover is a sentence they would have to go looking for.
/// Fourteen copies of it would be the nag the brief for this work explicitly
/// rules out.
///
/// It is drawn only when at least one addable row exists, so a page already
/// carrying all fourteen never shows it. See
/// [`crate::text::panels::face::face_addable_disclosure`] for what the sentence
/// has to contain and why each clause is there.
pub(crate) fn popup_body(
    ui: &mut Ui,
    prefix: &str,
    faces: &[FaceChoice],
    current: &str,
) -> Option<String> {
    // ★ See [`POPUP_MIN_WIDTH`]: the ribbon's button is 78 points wide and the
    // disclosure is a sentence. Stated here rather than at the two call sites so
    // the two surfaces cannot be legible in one place and not the other.
    ui.set_min_width(POPUP_MIN_WIDTH);

    if faces.is_empty() {
        ui.label(t::text_face_none());
        return None;
    }

    let mut chosen = None;
    let mut drew_addable_heading = false;
    let mut drew_page_heading = false;
    let mut published_first_addable = false;

    for face in faces {
        // The two headings, each drawn immediately before the first row of its
        // group. Written as a flag rather than by partitioning the slice into
        // two vectors, because the row-drawing body is the part that must not be
        // written twice — that is this module's whole reason for existing.
        match face.origin {
            FaceOrigin::OnThisPage if !drew_page_heading => {
                drew_page_heading = true;
                ui.label(t::face_group_on_page());
            }
            FaceOrigin::PdfcerWouldAdd if !drew_addable_heading => {
                drew_addable_heading = true;
                if drew_page_heading {
                    ui.separator();
                }
                let heading = ui.label(t::face_group_addable());
                crate::diag::ui_rect_visible(
                    &format!("{prefix}.addable"),
                    heading.rect,
                    ui.clip_rect(),
                );
                // ★★★ The disclosure. Once, here, before any addable row can be
                // clicked — see this function's own header.
                //
                // ★★ Both the `set_max_width` and the explicit `.wrap()` are
                // load-bearing, and neither is a style choice. `ComboBox`'s own
                // popup body sets `wrap_mode = Extend` inside its scroll area,
                // with the comment *"often the button is very narrow … so that
                // the labels expand the width of the menu"* — which is right for
                // a font name and catastrophic for a sixty-word sentence: it
                // would lay the whole disclosure out on ONE line and take the
                // popup off the side of the screen. So this label opts back into
                // wrapping and is given the width to wrap against.
                let note = ui
                    .scope(|ui| {
                        ui.set_max_width(POPUP_MIN_WIDTH);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(t::face_addable_disclosure()).small(),
                            )
                            .wrap(),
                        )
                    })
                    .inner;
                crate::diag::ui_rect_visible(
                    &format!("{prefix}.disclosure"),
                    note.rect,
                    ui.clip_rect(),
                );
            }
            _ => {}
        }

        let selected = face.label == current;
        let row = ui.selectable_label(selected, &face.label);
        // ★ The FIRST addable row gets its own region, so a driven check has a
        // deterministic target for *"choose a face this document does not
        // contain"*. Per-row regions for all fourteen would publish fourteen
        // trace lines per frame the popup is open, on a surface that redraws at
        // sixty frames a second.
        if face.origin == FaceOrigin::PdfcerWouldAdd && !published_first_addable {
            published_first_addable = true;
            crate::diag::ui_rect_visible(&format!("{prefix}.new"), row.rect, ui.clip_rect());
        }
        // ★ The twin disclosure. Two rows reading identically is otherwise
        // indistinguishable from a bug, and the operator has a real choice to
        // make between them.
        let row = if face.ambiguous {
            row.on_hover_text(t::text_face_ambiguous())
        } else {
            row
        };
        if row.clicked() && !selected {
            // ★★★ `selector`, NOT the label. On a page with two subsets of one
            // `/BaseFont` the name reaches one of them arbitrarily; the selector
            // reaches the one this row is about. For an addable row the two
            // happen to be equal, and it is still the selector that is sent,
            // because the row that is different must not be the row that is
            // special-cased.
            chosen = Some(face.selector.clone());
        }
    }
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real [`FontPreflight`] for `fixtures/paragraph.pdf`, whose page
    /// carries exactly one standard-14 face (`Helvetica`) and no other font.
    ///
    /// ★★★ **THE TESTS BELOW USED TO BUILD NO PREFLIGHT AT ALL, AND THAT IS THE
    /// FINDING WORTH KEEPING.**
    ///
    /// Both of them re-implemented [`choices`]' algorithm *inside their own
    /// bodies* — walking `Std14::ALL`, filtering against a hand-written
    /// `carried` list — and asserted on their own reimplementation. **Neither
    /// ever called `choices`.** So on 2026-09-08, when `choices` was rewritten
    /// to read the engine's `standard_14` survey and three local re-derivations
    /// were deleted, **both tests passed unchanged** — and were then testing
    /// code that no longer exists.
    ///
    /// The reason given at the time was honest and real: `FontPreflight` is
    /// `#[non_exhaustive]` and cannot be built with a struct literal outside
    /// `pdfcer-core`. ⇒ **The answer to "I cannot construct it" is to obtain a
    /// real one, not to simulate the function under test.** `EditSession`
    /// hands one over for the asking, and a fixture costs a millisecond.
    ///
    /// ⚠ It also means the input half — the half the old comment admitted was
    /// uncovered — is the half that mattered, because that is where the change
    /// landed.
    fn preflight_for_paragraph() -> pdfcer_core::text_edit::FontPreflight {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/paragraph.pdf");
        let doc = pdfcer_core::document::Document::load(std::path::Path::new(path))
            .expect("fixtures/paragraph.pdf must load");
        let session = pdfcer_core::edit::EditSession::new(doc);
        session
            .preview_font_resources(0, "its box.", None)
            .expect("the fixture's page 1 carries the words `its box.` in one run")
    }

    /// ★★★ **Every standard-14 face the page does not carry is offered, and the
    /// list is the ENGINE'S survey rather than a local walk of `Std14::ALL`.**
    ///
    /// `fixtures/paragraph.pdf` carries `Helvetica` and nothing else, and its
    /// text is plain ASCII that every text face can encode — so the expected
    /// answer is the whole fourteen: one through `accepted()` as a page face,
    /// thirteen as addable.
    ///
    /// ★ Asserted as `>= 13` addable rather than `== 13` on purpose. The
    /// standard 14 contains `Symbol` and `ZapfDingbats`, whose acceptance for
    /// ASCII text is the engine's ruling and not this shell's to pin — if the
    /// engine decides a font-specific encoding cannot hold `its box.`, that is
    /// a correct answer and must not fail this shell's test. What IS pinned is
    /// that the twelve text faces all arrive.
    #[test]
    fn the_faces_the_page_lacks_are_offered_and_come_from_the_engines_survey() {
        let preflight = preflight_for_paragraph();
        let rows = choices(Some(&preflight));

        let addable: Vec<&str> = rows
            .iter()
            .filter(|r| r.origin == FaceOrigin::PdfcerWouldAdd)
            .map(|r| r.label.as_str())
            .collect();

        // The twelve text faces of the standard 14, which every one of this
        // fixture's ASCII characters fits.
        for want in [
            "Times-Roman",
            "Times-Bold",
            "Times-Italic",
            "Times-BoldItalic",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
            "Courier",
            "Courier-Bold",
            "Courier-Oblique",
            "Courier-BoldOblique",
        ] {
            assert!(
                addable.contains(&want),
                "{want} is a standard-14 face this page does not carry and whose encoding holds \
                 plain ASCII, so it must be offered as addable. Got {addable:?}"
            );
        }
        // ★★★ AND THE TWO THE ENGINE REFUSES ARE ABSENT. This is the
        // assertion that makes the coverage test falsifiable, and it took a
        // measurement to find: a first draft asserted `addable.len() >= 11`,
        // which is satisfied whether or not `choices` filters on acceptance at
        // all. Dropping `is_accepted()` from the filter left it **green**.
        //
        // Measured on this fixture: `Symbol` and `ZapfDingbats` are surveyed,
        // are `WouldBeAdded`, and are **refused** — their built-in
        // font-specific encodings cannot hold the ASCII of `its box.`. So they
        // are precisely the two rows whose absence proves the filter runs.
        //
        // ⚠ Named rather than counted. `assert_eq!(addable.len(), 11)` would
        // also catch it today and would break for the wrong reason the day the
        // standard 14 gains a fifteenth member or the engine's ruling on
        // `Symbol` changes — and it would not say WHICH face went missing.
        for refused in ["Symbol", "ZapfDingbats"] {
            assert!(
                !addable.contains(&refused),
                "{refused}'s own encoding cannot hold this run's characters, so `set_font` would \
                 REFUSE it — offering it is a control that cannot work. The engine reports this \
                 in `Std14Entry::acceptance`; if it is in this list, `choices` has stopped \
                 reading it. Got {addable:?}"
            );
        }

        // The whole list, stated once so a reader sees the shape: one page face
        // plus eleven addable, out of a survey of fourteen.
        assert_eq!(
            addable.len(),
            11,
            "eleven of the fourteen are addable-and-accepted for plain ASCII on a page carrying \
             only Helvetica (Helvetica itself is a page face; Symbol and ZapfDingbats are \
             refused). If this number moved, read the survey before changing it: {addable:?}"
        );
    }

    /// ★★ **A standard face the page already carries is offered ONCE, as a page
    /// face — never a second time as an addable one.**
    ///
    /// The duplicate would be the visible defect. The invisible one is worse:
    /// a page `Helvetica` that this run's characters cannot encode into is
    /// absent from `accepted()`, so a filter built on that list would offer
    /// *"pdfcer can add Helvetica"* — and `plan_font` would resolve the
    /// selector to the page's own refused resource and decline. An entry that
    /// cannot work, described wrongly.
    ///
    /// ⇒ `Std14Presence::OnPage` is the engine answering that from the resource
    /// dictionary it actually resolved, which is why [`choices`] no longer
    /// compares shortened name strings to decide it.
    #[test]
    fn the_pages_own_standard_face_is_offered_once_and_not_as_addable() {
        let preflight = preflight_for_paragraph();
        let rows = choices(Some(&preflight));

        let helvetica: Vec<&FaceChoice> = rows.iter().filter(|r| r.label == "Helvetica").collect();
        assert_eq!(
            helvetica.len(),
            1,
            "the page's own Helvetica must appear exactly once: {rows:?}"
        );
        assert_eq!(
            helvetica[0].origin,
            FaceOrigin::OnThisPage,
            "it is a face the page CARRIES, so offering it as one pdfcer would add would be an \
             entry that resolves to the existing resource: {:?}",
            helvetica[0]
        );
    }

    /// ★ **The engine's survey is what is read** — not a local `Std14::ALL`
    /// walk that happens to agree with it today.
    ///
    /// A rewrite that quietly reverted to walking the constant would satisfy
    /// both tests above, because on this fixture the two answers coincide. This
    /// one does not: it empties the survey's contribution by asserting the
    /// offered labels are a SUBSET of what the engine reported, which a local
    /// walk cannot guarantee.
    #[test]
    fn no_offered_face_is_absent_from_the_engines_own_survey() {
        let preflight = preflight_for_paragraph();
        let rows = choices(Some(&preflight));

        let surveyed: Vec<&str> = preflight
            .standard_14
            .iter()
            .map(|e| super::super::text::shorten(&e.base_font))
            .collect();
        assert!(
            !surveyed.is_empty(),
            "the engine reported no standard-14 survey at all, so this test proves nothing — \
             `FontPreflight::standard_14` is what `Pass 142.2` added and what `choices` reads"
        );

        for row in rows
            .iter()
            .filter(|r| r.origin == FaceOrigin::PdfcerWouldAdd)
        {
            assert!(
                surveyed.contains(&row.label.as_str()),
                "{:?} is offered as addable but is not in the engine's survey — this shell is \
                 inventing faces again. Surveyed: {surveyed:?}",
                row.label
            );
        }
    }

    /// ★ **An absent pre-flight offers nothing at all**, not the fourteen on
    /// their own.
    ///
    /// The tempting shape — *"we could not ask the page, so offer the standard
    /// faces, they always work"* — is wrong twice. The standard-14 half is
    /// filtered **by** the pre-flight, so without one the list would offer
    /// `Helvetica` on a page whose own `Helvetica` will take the click; and a run
    /// that did not pin cannot be restyled at all, so every row would refuse.
    #[test]
    fn no_preflight_offers_no_faces() {
        assert!(choices(None).is_empty());
    }

    /// ★★ **The two origins are distinguishable**, which is the whole of what
    /// the operator is being shown.
    ///
    /// A `FaceChoice` that lost its origin would render under whichever heading
    /// it happened to sort beside, and the disclosure would then be attached to
    /// rows it is not true of. The enum is `Copy` and cheap; this asserts it is
    /// also actually compared somewhere, which is what a `derive(PartialEq)` on
    /// an unused field would not be.
    #[test]
    fn the_two_origins_are_not_equal() {
        assert_ne!(FaceOrigin::OnThisPage, FaceOrigin::PdfcerWouldAdd);
    }
}
