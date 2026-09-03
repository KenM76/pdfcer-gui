//! # `panels::fonts` — what fonts the document declares, and what they cost
//!
//! Salvaged from the old shell's `panels_structure.rs`. **The report came
//! across; the two controls did not.**
//!
//! # Where this panel lives, and why it moved
//!
//! `file.fonts`, on File ▸ Document, beside Properties — not View ▸ Panels
//! where the old shell had it. `RIBBON_IA.md` §7's migration map moves it,
//! and the reason is one sentence in the tab's own module docs: the Fonts
//! panel answers *"what is inside this file"*, not *"what is on my screen"*.
//!
//! # ★ Why the panel says *why*, when the parity reference does not
//!
//! Acrobat refuses to unembed a font whose character codes are glyph indices
//! into its own embedded program, and it refuses **silently** — the font
//! simply is not in its unembed list, with no reason shown anywhere.
//! Corroborated by a user whose largest, most size-costly font was absent
//! from the list with no explanation offered.
//!
//! A shorter list is not actionable. *"This font's character codes are
//! positions inside this specific embedded program"* is. That is rule 4
//! applied to a refusal rather than to a suggestion, and it is this panel's
//! main reason to exist.
//!
//! `pdfcer-core` is built for it: `Removability`'s own doc comment says the
//! unembedding verb *"consumes this exact value, so a font pdfcer shows as
//! blocked and a font pdfcer declines to unembed are the same set by
//! construction."*
//!
//! The measured stakes, from a 64-file survey of the PDFBox corpus: of the
//! 30 files that embed fonts, **87 % embed subsets, 40 % use `Identity-H`,
//! and only 50 % carry `/ToUnicode`**. So the common case for "just remove
//! the embedded fonts" is a case where removal destroys the document, and
//! the operator has no way to know that from a font list alone.
//!
//! # ★ The coverage note is above the list, not beneath it
//!
//! A font inventory that quietly misses a surface and prints a confident
//! list is this project's most-repeated defect shape. So the panel states
//! which font-bearing surfaces were searched **and the one that was not**,
//! unconditionally, before the list. Acrobat's own coverage here is recorded
//! as an unconfirmed gap, so pdfcer states its own scope rather than assuming
//! parity with a behaviour nobody has measured.
//!
//! The page-scan failure sits above even that, because it changes what an
//! empty list beneath it *means*: without it, "0 fonts" and "pdfcer could not
//! walk the page tree" render identically and an operator reads the second
//! as the first.
//!
//! # ★ What did NOT come across: unembed, and embed
//!
//! The old panel carried two controls — remove a font's embedded program,
//! and embed a missing one — each in a batch form under the summary and a
//! per-row form at the foot of an expanded row, with a confirmation window
//! for the destructive half. Neither is here.
//!
//! Both push a mutation through `pdfcer_core::edit::EditSession`, and at S3
//! [`crate::app::actions::Action`] carries zoom and page navigation and
//! nothing else. There is no command log, no undo, and
//! `crate::app::state::OpenDoc::edit_epoch` names itself as *the documented
//! seam* the first mutating arm must bump. A control that cannot commit is
//! an affordance for something that cannot work (`RIBBON_IA.md` P3, R83) —
//! and the destructive one would be worse than that: the old shell's
//! confirmation window exists because **three of unembedding's four
//! consequences are invisible on the canvas** (a broken PDF/A claim, an
//! invalidated signature, a renamed font), so a control that appeared to
//! work and did not would be indistinguishable from one that had.
//!
//! Two consequences of the omission are worth naming, because both are
//! recoveries the report itself has to make:
//!
//! 1. The old panel's **batch embed block** was the only place a document's
//!    "how many fonts are missing a program" count appeared. It is now a
//!    plain sentence — [`crate::text::panels::fonts::fonts_missing_programs`]
//!    — because that count is a fact about the file, not about a plan.
//! 2. The old panel's per-row **"you did this in this session"** lines are
//!    gone with the actions that produced them, and correctly: after an
//!    unembed the row's verdict is the one a font that *arrived*
//!    non-embedded carries, so the line existed to stop the panel erasing
//!    the operator's own action. With no action, there is nothing to erase.
//!
//! # Every verdict is drawn at the same visual weight
//!
//! As a plain label, never error-styled. A blocked verdict is a fact about
//! the **file**, and error styling would make it read as a pdfcer failure.
//! With no removal control, that rule is easier to keep than it was: the old
//! panel's only difference between a removable row and a refused one was the
//! presence of the control, and now there is none.
//!
//! Discoverability does not suffer, because the **collapsed header already
//! carries the verdict word** — "No blocker" against "Locked to program"
//! answers the question at a glance, without opening a single row.
//!
//! # Rows are largest first
//!
//! The operator opening this panel is usually asking *"which font is costing
//! me the most"*, and that ordering answers it with no control to find. Ties
//! keep discovery order, which `sort_by_key` preserves.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::byte_size;
use crate::text::panels::fonts as t;

/// Draw the Fonts panel.
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    _state: &mut PanelsState,
    _actions: &mut Vec<Action>,
) {
    use pdfcer_core::fontinfo::{Program, Removability};

    // The document's own inventory. It moved from `PanelsState` to `OpenDoc`
    // at S4 so the Properties panel's `/BaseFont` join reads the same sweep
    // this list is drawn from, rather than a second one that could disagree
    // with it about what is embedded.
    let inv = doc.font_inventory();

    // The page scan failing FIRST, above everything, because it changes what
    // an empty list beneath it means.
    if inv.diagnostics.page_scan_failed {
        ui.label(t::fonts_page_scan_failed());
        ui.separator();
    }
    if inv.diagnostics.resource_scan_truncated {
        ui.label(t::fonts_scan_truncated());
        ui.separator();
    }

    if inv.fonts.is_empty() {
        ui.label(t::fonts_none());
        ui.label(egui::RichText::new(t::fonts_coverage_note()).small().weak());
        return;
    }

    ui.label(t::fonts_count(inv.fonts.len()));
    // The document total, from the same per-font numbers the rows below
    // show, so the two cannot disagree.
    let total = usize::try_from(inv.embedded_bytes()).unwrap_or(usize::MAX);
    ui.label(t::fonts_total_size(&byte_size(total)));

    // The end state, or the count. Both are facts about the file rather than
    // about a plan pdfcer could carry out, which is what makes them safe to
    // state with no control beneath them.
    let missing = inv
        .fonts
        .iter()
        .filter(|f| matches!(f.program, Program::NotEmbedded))
        .count();
    if missing == 0 {
        ui.label(t::fonts_all_embedded());
    } else {
        ui.label(t::fonts_missing_programs(missing));
    }

    ui.label(egui::RichText::new(t::fonts_coverage_note()).small().weak());
    ui.separator();

    // Largest first — see the module docs.
    let mut rows: Vec<&pdfcer_core::fontinfo::FontRecord> = inv.fonts.iter().collect();
    rows.sort_by_key(|f| std::cmp::Reverse(f.stored_bytes()));

    egui::ScrollArea::vertical()
        .id_salt("fonts-rows")
        .show(ui, |ui| {
            for (row_index, f) in rows.iter().enumerate() {
                // Keyed by object identity, not by row index: two independent
                // subsets of one face de-prefix to the SAME display name, and
                // an index-keyed header would swap its expanded state under
                // the operator when the sort order moved.
                let key =
                    f.id.map_or_else(|| format!("direct-{row_index}"), |id| format!("{}", id.num));
                let verdict = match &f.removability {
                    Removability::Removable => t::font_verdict_removable(),
                    Removability::BlockedIdentityEncoded { .. } => {
                        t::font_verdict_blocked_identity()
                    }
                    Removability::BlockedType3 => t::font_verdict_blocked_type3(),
                    Removability::NotEmbedded => t::font_verdict_not_embedded(),
                    _ => t::font_verdict_unknown(),
                };
                let display = f.family_name().unwrap_or_else(|| t::font_unnamed());
                let size = byte_size(f.stored_bytes());
                let header = t::font_row_header(display, &size, verdict);

                let response = egui::CollapsingHeader::new(header)
                    .id_salt(format!("font-{key}"))
                    .default_open(false)
                    .show(ui, |ui| row_body(ui, f));

                // The subset tag lives here rather than in the row, because
                // the row shows the DE-PREFIXED name and two independent
                // subsets of one face therefore render identically. Without
                // somewhere for the tag to resurface, two adjacent identical
                // rows read as a rendering fault instead of as the real fact
                // that the document subsetted the face twice.
                if let Some(full) = f.base_font.as_deref() {
                    response
                        .header_response
                        .on_hover_text(t::font_full_name_tooltip(full));
                }
            }
        });

    // Bound before the closure so the borrow of `inv` ends with the loop.
    let (count, embedded, bytes) = (inv.fonts.len(), inv.embedded_count(), inv.embedded_bytes());
    let verdicts = inv.verdict_counts();
    crate::diag::trace(|| {
        format!("fonts-panel rows={count} embedded={embedded} bytes={bytes} verdicts={verdicts:?}")
    });
}

/// One font's expanded body — the verdict's reason first, then the facts.
///
/// Split out so [`body`]'s loop stays readable, and because the reason
/// ladder is the substantive part of the panel: it is where the sentence
/// this panel exists to say actually gets said.
fn row_body(ui: &mut egui::Ui, f: &pdfcer_core::fontinfo::FontRecord) {
    use pdfcer_core::fontinfo::{Program, Removability, RemovabilityUnknown, Surface};

    // The verdict's REASON first, because it is the reason the row was
    // opened.
    let reason = match &f.removability {
        Removability::Removable => t::font_reason_removable().to_owned(),
        Removability::BlockedIdentityEncoded { to_unicode, .. } => {
            t::font_reason_blocked_identity(*to_unicode)
        }
        Removability::BlockedType3 => t::font_reason_blocked_type3().to_owned(),
        Removability::NotEmbedded => t::font_reason_not_embedded().to_owned(),
        Removability::Unknown(why) => match why {
            RemovabilityUnknown::SymbolicBuiltinEncoding => {
                t::font_reason_unknown_symbolic().to_owned()
            }
            RemovabilityUnknown::PredefinedCMap => {
                t::font_reason_unknown_predefined_cmap().to_owned()
            }
            RemovabilityUnknown::EmbeddedCMap => t::font_reason_unknown_embedded_cmap().to_owned(),
            RemovabilityUnknown::ProgramUnreadable => {
                t::font_reason_unknown_program_unreadable().to_owned()
            }
            RemovabilityUnknown::NoDescendant => t::font_reason_unknown_no_descendant().to_owned(),
            // `RemovabilityUnknown` is `#[non_exhaustive]`. A reason this
            // build does not know must render as the general "not
            // established" sentence, never as a confident one.
            _ => t::font_reason_unknown_subtype().to_owned(),
        },
        // `Removability` is `#[non_exhaustive]` too, and the same rule
        // applies one level up.
        _ => t::font_reason_unknown_subtype().to_owned(),
    };
    ui.label(reason);
    ui.separator();

    let kind = match &f.descendant_subtype {
        Some(d) => t::font_composite_type(f.subtype.label(), d.label()),
        None => f.subtype.label().to_owned(),
    };
    ui.label(t::font_type_line(&kind));
    ui.label(t::font_encoding_line(&f.encoding.label()));

    match &f.program {
        Program::Embedded(p) => {
            let key_label = match &p.subtype {
                Some(s) => t::font_program_key_with_subtype(p.key.label(), s),
                None => p.key.label().to_owned(),
            };
            ui.label(t::font_embedded_line(&key_label));
            ui.label(t::font_size_line(
                &byte_size(p.stored_bytes),
                p.stored_bytes,
            ));
            // Only when it differs — a line repeating the number above is
            // noise, and noise is how the lines that matter get skimmed
            // past.
            if let Some(decoded) = p.decoded_bytes
                && decoded != p.stored_bytes
            {
                ui.label(t::font_decoded_size_line(&byte_size(decoded)));
            }
            fs_type_lines(ui, &p.fs_type);
        }
        // "Declared but unreadable" is damage; the reason sentence above
        // already said so, and repeating a size of zero here would suggest a
        // measurement was taken.
        Program::Unreadable { .. } | Program::NotEmbedded => {
            ui.label(t::font_fstype_not_embedded());
        }
        _ => {}
    }

    ui.label(if f.has_to_unicode {
        t::font_to_unicode_present()
    } else {
        t::font_to_unicode_absent()
    });

    ui.separator();
    // An empty page list is NOT "unused" (core API trap T-9.4): a font
    // reached only through the AcroForm `/DR` has no page list and is a live
    // form-default font. Stated rather than left as an absence to infer.
    if f.pages.is_empty() {
        ui.label(t::font_no_pages_line());
    } else {
        ui.label(t::font_pages_line(
            &pdfcer_core::fontinfo::format_page_ranges(&f.pages),
            f.pages.len(),
        ));
    }
    for (surface, text) in [
        (
            Surface::AcroFormDefaultResources,
            t::font_found_in_form_resources(),
        ),
        (Surface::AnnotationAppearance, t::font_found_in_annotation()),
        (Surface::Type3CharProcs, t::font_found_in_type3()),
    ] {
        if f.surfaces.contains(&surface) {
            ui.label(text);
        }
    }
}

/// Render one font's `fsType` state.
///
/// ★ Four states, and **none of them may look like `0`.** `fsType == 0`
/// genuinely *means* Installable — the most permissive value the field can
/// express — so a blank, a dash, or an empty line for "we could not read it"
/// would assert the broadest embedding right there is on the strength of
/// bytes nobody read. The OpenType specification defines no default for the
/// absent case, so pdfcer defines none either: unknown says the word
/// "Unknown" in its own sentence, and "this format has no such field" says
/// that instead.
///
/// A free function because it needs nothing but the bits and a `Ui`.
fn fs_type_lines(ui: &mut egui::Ui, fs: &pdfcer_core::fontinfo::FsType) {
    use pdfcer_core::fontinfo::{EmbeddingPermission, FsType};
    match fs {
        FsType::NotApplicable => {
            ui.label(t::font_fstype_no_field());
        }
        // Both failure states say "unknown" in words. They differ in cause
        // and not in what an operator can conclude, which is nothing.
        FsType::ProgramNotDecoded | FsType::Unreadable(_) => {
            ui.label(t::font_fstype_unknown());
        }
        FsType::Known(bits) => {
            ui.label(match bits.permission {
                EmbeddingPermission::Installable => t::font_fstype_installable(bits.raw),
                EmbeddingPermission::Restricted => t::font_fstype_restricted(bits.raw),
                EmbeddingPermission::PreviewPrint => t::font_fstype_preview_print(bits.raw),
                EmbeddingPermission::Editable => t::font_fstype_editable(bits.raw),
                EmbeddingPermission::Ambiguous => t::font_fstype_ambiguous(bits.raw),
                _ => t::font_fstype_unspecified(bits.raw),
            });
            if bits.no_subsetting {
                ui.label(t::font_fstype_no_subsetting());
            }
            if bits.bitmap_only {
                ui.label(t::font_fstype_bitmap_only());
            }
            if bits.version_gated_bits_ignored {
                ui.label(t::font_fstype_version_gated());
            }
        }
        // `FsType` is `#[non_exhaustive]`. A state this build does not know
        // must render as unknown, never as a permission.
        _ => {
            ui.label(t::font_fstype_unknown());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::panels::objects::test_support::engine_fixture;
    use pdfcer_core::fontinfo::{Program, Removability};

    /// Build the inventory a panel body would read, from a fixture.
    fn inventory(rel: &str) -> pdfcer_core::fontinfo::FontInventory {
        let path = engine_fixture(rel);
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        pdfcer_core::fontinfo::inventory(&doc.view())
    }

    /// **★ A font whose codes are glyph indices into its own program is
    /// reported as blocked, with a reason.**
    ///
    /// The sentence this panel exists to say, asserted against a real file.
    /// Acrobat refuses the same font and shows no reason at all; a shorter
    /// list is not actionable and this is the difference.
    ///
    /// Asserted on the verdict and on `to_unicode`, not on the wording:
    /// `crate::text::panels::fonts` already tests that the two tiers of the
    /// sentence differ, and a copy edit should not break a panel test.
    #[test]
    fn an_identity_encoded_font_is_blocked_and_says_which_tier() {
        // The fixture must **embed** the program: a font that is not
        // embedded has nothing to remove, so core reports
        // `Removability::NotEmbedded` and the headline case never fires.
        // (`text/identity-h-no-tounicode.pdf` is Identity-encoded and NOT
        // embedded, which is why it is the wrong fixture for this and was
        // the wrong one to reach for first.)
        let inv = inventory("text/cidfonttype2-nocmap-embedded.pdf");
        let blocked: Vec<&Removability> = inv
            .fonts
            .iter()
            .map(|f| &f.removability)
            .filter(|r| matches!(r, Removability::BlockedIdentityEncoded { .. }))
            .collect();
        assert!(
            !blocked.is_empty(),
            "this fixture exists to carry an Identity-encoded font; without one \
             the panel's headline case is untested"
        );
        // This fixture has NO /ToUnicode, which is the worse of the two
        // tiers: the text cannot be drawn without the program AND the
        // characters cannot be recovered either.
        assert!(
            blocked.iter().any(|r| matches!(
                r,
                Removability::BlockedIdentityEncoded { to_unicode: false }
            )),
            "the no-/ToUnicode tier must be reported, or the panel understates \
             what removal would cost: {blocked:?}"
        );

        // …and the OTHER tier is reachable too, from a document that does
        // carry the map. Both are asserted because the panel's sentence
        // branches on exactly this flag, and a fixture set that only ever
        // produced one tier would leave the other's wording unexercised.
        let with_map = inventory("text/composite-editable.pdf");
        assert!(
            with_map.fonts.iter().any(|f| matches!(
                f.removability,
                Removability::BlockedIdentityEncoded { to_unicode: true }
            )),
            "the /ToUnicode tier must be reachable"
        );
    }

    /// **The panel's "n fonts have no program" count and the row verdicts
    /// agree.**
    ///
    /// Two independent readings of the same inventory — the summary counts
    /// `Program::NotEmbedded`, the rows print `Removability::NotEmbedded` —
    /// and they are computed in different places. `Removability`'s own docs
    /// promise they mean the same thing ("There is no embedded program"), so
    /// a document where they disagree is a document where the summary is
    /// lying about the list beneath it.
    #[test]
    fn the_missing_program_count_matches_the_row_verdicts() {
        for fixture in ["text/simple-winansi.pdf", "vector/mixed.pdf"] {
            let inv = inventory(fixture);
            let by_program = inv
                .fonts
                .iter()
                .filter(|f| matches!(f.program, Program::NotEmbedded))
                .count();
            let by_verdict = inv
                .fonts
                .iter()
                .filter(|f| matches!(f.removability, Removability::NotEmbedded))
                .count();
            assert_eq!(
                by_program, by_verdict,
                "{fixture}: the summary counts {by_program} fonts with no program \
                 and the rows show {by_verdict}"
            );
        }
    }

    /// **The document total is the sum of the rows.**
    ///
    /// `embedded_bytes()` and the per-row `stored_bytes()` are two
    /// computations over one inventory, and the panel prints both. A total
    /// that does not add up is the fastest way to lose an operator's trust
    /// in every other number on the panel.
    #[test]
    fn the_document_total_is_the_sum_of_the_rows() {
        let inv = inventory("text/subset-simple-embedded.pdf");
        assert!(!inv.fonts.is_empty(), "the fixture must declare a font");
        let summed: u64 = inv
            .fonts
            .iter()
            .filter(|f| matches!(f.program, Program::Embedded(_)))
            .map(|f| f.stored_bytes() as u64)
            .sum();
        assert_eq!(inv.embedded_bytes(), summed);
    }

    /// **A row's collapsed header is enough to answer "which of these can
    /// go", without opening anything.**
    ///
    /// The panel's discoverability argument in one assertion: every font
    /// gets a verdict word, so the question is answerable from the collapsed
    /// list. A record that fell through to no verdict at all would be a row
    /// an operator has to open to learn nothing.
    #[test]
    fn every_font_gets_a_verdict_word_in_its_collapsed_header() {
        use crate::text::panels::fonts as t;
        let inv = inventory("vector/mixed.pdf");
        for f in &inv.fonts {
            let verdict = match &f.removability {
                Removability::Removable => t::font_verdict_removable(),
                Removability::BlockedIdentityEncoded { .. } => t::font_verdict_blocked_identity(),
                Removability::BlockedType3 => t::font_verdict_blocked_type3(),
                Removability::NotEmbedded => t::font_verdict_not_embedded(),
                _ => t::font_verdict_unknown(),
            };
            assert!(!verdict.is_empty());
            let header = t::font_row_header(
                f.family_name().unwrap_or_else(|| t::font_unnamed()),
                &crate::text::panels::byte_size(f.stored_bytes()),
                verdict,
            );
            assert!(header.starts_with(verdict), "{header}");
        }
    }
}
