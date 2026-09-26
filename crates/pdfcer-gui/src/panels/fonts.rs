//! # `panels::fonts` — what fonts the document declares, and what they cost
//!
//! Salvaged from the old shell's `panels_structure.rs`. **The report came
//! across; the two controls did not.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/fonts.md`.

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

    /// **A font whose codes are glyph indices into its own program is
    /// reported as blocked, with a reason.**
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
