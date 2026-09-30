//! # `panels::signatures` — three facts about each digital signature, reported
//! separately and never merged
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/signatures.md`.

use pdfcer_core::signature::{ByteRangeCoverage, Integrity, SignatureVerdict, Trust};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels as t;
use crate::text::trust as tt;
use crate::trust::Anchors;

/// The region the panel body publishes, so a driven check can read it.
pub const REGION_BODY: &str = "panel:signatures"; // ui-text-exempt: trace region name, never displayed

/// Draw the Signatures panel.
pub fn body(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    _state: &mut PanelsState,
    _actions: &mut Vec<Action>,
) {
    crate::diag::ui_rect(REGION_BODY, ui.max_rect());

    // A stat, not a read. Cheap enough per frame, and the alternative is a
    // cached number that silently describes a file that no longer exists in
    // that form.
    let Ok(meta) = std::fs::metadata(&doc.path) else {
        ui.label(t::signatures_file_unreadable());
        return;
    };
    let coverage = pdfcer_core::signature::byte_range_coverage(&doc.session.view(), meta.len());

    if coverage.is_empty() {
        ui.label(t::signatures_none());
        return;
    }

    // The framing sentence FIRST. Everything below it is three measurements per
    // signature, and a measurement read as a single verdict is the failure this
    // prevents.
    ui.label(egui::RichText::new(tt::panel_intro()).small().weak());
    ui.label(
        egui::RichText::new(t::signatures_measured_on_disk())
            .small()
            .weak(),
    );

    // The report is computed here rather than inside the row loop, because it
    // is one act over the whole FILE — one digest, one anchor-store read — and
    // computing it per row would both be wasteful and make the anchor
    // disclosure a per-row sentence, repeated as many times as there are
    // signatures.
    //
    // ⚠ `stored_under`, not `path`: a document created in this session has a
    // NAME rather than a file, and there is nothing on disk to verify. The
    // coverage half above already reports that case honestly through the
    // metadata failure.
    let report = doc.stored_under().map(|path| {
        crate::trust::cached_report(
            ui.ctx(),
            &doc.session.view(),
            path,
            doc.settings.acrobat_trust_store,
            &doc.prefs.acrobat_trust_store_path,
        )
    });

    // Where the anchors came from — once, above the list, because it is a fact
    // about the whole report rather than about any one signature.
    if let Some(Ok(report)) = report.as_ref() {
        anchor_provenance(ui, &report.anchors);
    }
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("signatures-rows")
        .show(ui, |ui| {
            for (index, c) in coverage.iter().enumerate() {
                let name = c
                    .field_name
                    .clone()
                    .unwrap_or_else(|| t::signature_unnamed().to_owned());
                ui.label(egui::RichText::new(tt::signature_heading(&name)));

                // Matched by INDEX, and the engine guarantees it: `verify_all`'s
                // own documentation says *"order and count match
                // `byte_range_coverage`"*. Matching by `field_name` instead
                // would look safer and be worse — two unnamed signature fields
                // both key on `None`, and a document with two of those would
                // pair each row with the wrong verdict silently.
                let verdict = match report.as_ref() {
                    Some(Ok(r)) => r.verdicts.get(index),
                    _ => None,
                };
                integrity_line(ui, verdict);
                coverage_line(ui, c);
                trust_line(ui, verdict, report.as_ref().and_then(|r| r.as_ref().ok()));
                revocation_lines(ui, verdict);

                crate::diag::trace(|| {
                    //
                    // This line read `field={:?}` over an `Option<String>`, so it
                    // emitted `field=Some("SignHere")` — quotes, brackets and all
                    // — into a `key=value` trace that checks parse by splitting on
                    // whitespace and `=`. The signing check's own verdict rests on
                    // this field.
                    //
                    // ⇒ The project's standing rule, met for the third time this
                    // week: **never Debug-format a value a machine reads.** It has
                    // already produced two false failure reports, one of which
                    // announced that a surface did not name a character while
                    // quoting the surface naming it. A `{:?}` also makes the
                    // trace's vocabulary a consequence of a Rust derive, so it
                    // changes silently when the type does.
                    //
                    // Fixed at the emitter rather than in the reader, and the
                    // absent case gets the same `none` spelling every other token
                    // helper here uses — so a check comparing two surfaces is
                    // comparing one language.
                    format!(
                        "signature-row field={} covered={} tail={} pairs={} well_formed={} integrity={} trust={}",
                        field_token(c.field_name.as_deref()),
                        c.covered,
                        c.uncovered_tail,
                        c.pair_count,
                        c.ranges_well_formed,
                        verdict.map_or("unmeasured", |v| integrity_token(&v.integrity)),
                        verdict.map_or("unmeasured", |v| trust_token(&v.trust)),
                    )
                });
                ui.separator();
            }
        });
}

/// Fact 1 — whether the signed bytes are what was signed.
fn integrity_line(ui: &mut egui::Ui, verdict: Option<&SignatureVerdict>) {
    let Some(verdict) = verdict else {
        return;
    };
    let said = match &verdict.integrity {
        Integrity::Verified {
            digest_algorithm,
            signature_algorithm,
        } => tt::integrity_verified(digest_algorithm, signature_algorithm),
        Integrity::DigestMismatch => tt::integrity_digest_mismatch().to_owned(),
        Integrity::SignatureInvalid => tt::integrity_signature_invalid().to_owned(),
        Integrity::Unverifiable { reason } => tt::integrity_unverifiable(reason),
        // `Integrity` is `#[non_exhaustive]`, so this arm is required by the
        // compiler — and it must not fall silent. A variant this build does not
        // know is still a verdict the engine reached, and rendering nothing
        // would make an unrecognised answer look like a missing one. The engine
        // puts its own words in `notes`, which are drawn below regardless.
        _ => tt::integrity_unverifiable(&format!("{:?}", verdict.integrity)),
    };
    labelled(ui, tt::integrity_label(), &said);
    // The engine's own disclosures for this signature — a weak digest, non-zero
    // `/Contents` padding, an odd CMS version, extra signers. Printed verbatim,
    // because each names something the operator cannot see from the verdict.
    for note in &verdict.notes {
        ui.label(egui::RichText::new(note).small().weak());
    }
}

/// Where each certificate says its revocation status lives, signer first.
/// Nothing is fetched; the trust line keeps saying revocation was not checked.
fn revocation_lines(ui: &mut egui::Ui, verdict: Option<&SignatureVerdict>) {
    let Some(verdict) = verdict else {
        return;
    };
    let named: Vec<&pdfcer_core::signature_verify::RevocationSources> = verdict
        .revocation_sources
        .iter()
        .filter(|s| !s.is_empty())
        .collect();
    crate::diag::trace(|| {
        let count = |f: fn(&pdfcer_core::signature_verify::RevocationSources) -> usize| -> usize {
            named.iter().map(|s| f(s)).sum()
        };
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "signature-revocation-sources certs={} crl={} ocsp={} issuers={} unreadable={}",
            named.len(),
            count(|s| s.crl.len()),
            count(|s| s.ocsp.len()),
            count(|s| s.ca_issuers.len()),
            count(|s| s.unreadable),
        )
    });
    if named.is_empty() {
        return;
    }
    ui.label(egui::RichText::new(tt::revocation_label()).small().weak());
    for s in named {
        ui.label(
            egui::RichText::new(tt::revocation_sources(
                &s.subject,
                &s.crl,
                &s.ocsp,
                &s.ca_issuers,
                s.unreadable,
            ))
            .small(),
        );
    }
}

/// Fact 2 — what the signature covers, unchanged from the panel's first
/// version.
fn coverage_line(ui: &mut egui::Ui, c: &ByteRangeCoverage) {
    if !c.ranges_well_formed {
        ui.label(t::signature_range_malformed());
    }
    if c.pair_count == 1 {
        ui.label(t::signature_single_range());
    }
    let said = if c.covers_to_eof() {
        t::signature_covers_whole_file(c.covered)
    } else {
        t::signature_leaves_tail(c.covered, c.uncovered_tail)
    };
    labelled(ui, tt::coverage_label(), &said);
}

/// Fact 3 — who signed, and whether they chain to a trusted anchor.
fn trust_line(
    ui: &mut egui::Ui,
    verdict: Option<&SignatureVerdict>,
    report: Option<&std::sync::Arc<crate::trust::Report>>,
) {
    let said = match verdict.map(|v| &v.trust) {
        Some(Trust::Trusted {
            anchor_subject,
            source,
            validity_checked,
        }) => tt::trusted(anchor_subject, source, *validity_checked),
        Some(Trust::Untrusted { reason }) => tt::untrusted(reason),
        Some(Trust::SignerUnknown) => tt::signer_unknown().to_owned(),
        // `NotChecked`, an unrecognised future variant, or no report at all.
        // All three are honestly "not checked", and the reason comes from what
        // this shell did with the anchors.
        _ => tt::not_checked(&why_not_checked(report.map(|r| &r.anchors))),
    };
    labelled(ui, tt::trust_label(), &said);
}

/// Which of the four explanations of `NotChecked` applies.
fn why_not_checked(anchors: Option<&Anchors>) -> String {
    match anchors {
        Some(Anchors::NoStore {
            configured_missing: Some(path),
            ..
        }) => tt::not_checked_configured_missing(&path.display().to_string()),
        Some(Anchors::NoStore { looked_in, .. }) => tt::not_checked_no_store(looked_in.len()),
        Some(Anchors::Unreadable { path, reason }) => {
            tt::not_checked_unreadable(&path.display().to_string(), reason)
        }
        // `Used` reaching here means the engine returned `NotChecked` while
        // anchors WERE supplied — which the engine does not do today, and would
        // be a boundary change rather than an operator error. The opted-out
        // sentence would be a lie in that case, so it takes the honest one: the
        // anchors were there and no verdict came back.
        Some(Anchors::Used { .. }) => tt::signer_unknown().to_owned(),
        Some(Anchors::OptedOut) | None => tt::not_checked_opted_out().to_owned(),
    }
}

/// Where the anchors came from, once, above the list.
fn anchor_provenance(ui: &mut egui::Ui, anchors: &Anchors) {
    let Anchors::Used {
        path,
        modified,
        counts,
        undecodable,
    } = anchors
    else {
        return;
    };
    let date = modified.and_then(crate::trust::modified_date);
    let mut said = tt::store_line(&path.display().to_string(), date.as_deref(), counts);
    if *undecodable > 0 {
        said.push(' ');
        said.push_str(&tt::store_undecodable(*undecodable));
    }
    ui.label(egui::RichText::new(said).small().weak());
    ui.label(egui::RichText::new(tt::at_own_risk()).small().weak());
}

/// One fact: its label, then its sentence.
fn labelled(ui: &mut egui::Ui, label: &str, said: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(egui::RichText::new(label).small().weak());
        ui.label(said);
    });
}

/// The signature field's name as one trace token, or `none`.
fn field_token(name: Option<&str>) -> &str {
    name.unwrap_or("none")
}

/// The integrity verdict as one trace token.
const fn integrity_token(integrity: &Integrity) -> &'static str {
    match integrity {
        Integrity::Verified { .. } => "verified",
        Integrity::DigestMismatch => "digest-mismatch",
        Integrity::SignatureInvalid => "signature-invalid",
        Integrity::Unverifiable { .. } => "unverifiable",
        _ => "unknown-variant",
    }
}

/// The trust verdict as one trace token. See [`integrity_token`].
const fn trust_token(trust: &Trust) -> &'static str {
    match trust {
        Trust::NotChecked => "not-checked",
        Trust::Trusted { .. } => "trusted",
        Trust::Untrusted { .. } => "untrusted",
        Trust::SignerUnknown => "signer-unknown",
        _ => "unknown-variant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::panels as t;

    /// **"No signatures" and "pdfcer could not measure the file" are
    /// different sentences.**
    #[test]
    fn an_unreadable_file_is_not_reported_as_an_unsigned_one() {
        let none = t::signatures_none();
        let unreadable = t::signatures_file_unreadable();
        assert_ne!(none, unreadable);
        assert!(
            unreadable.contains("could not read"),
            "the failure must name itself: {unreadable}"
        );
        assert!(
            unreadable.contains("Nothing here is a statement about the document"),
            "and must actively deny the reading it would otherwise invite: {unreadable}"
        );
    }

    /// **The leading sentence names three facts and says they are never
    /// merged.**
    #[test]
    fn the_intro_promises_three_separate_facts() {
        let intro = tt::panel_intro();
        for word in ["INTACT", "COVERS", "TRUSTED"] {
            assert!(intro.contains(word), "the intro must name {word}: {intro}");
        }
        assert!(intro.contains("never merges them"), "{intro}");
        // And it must say out loud that the facts can disagree, or a reader
        // meets their first intact-and-untrusted signature with no framing.
        assert!(intro.contains("intact and untrusted"), "{intro}");
    }

    /// **Every state this panel can be in produces a trust sentence that
    /// says "not checked", unless the engine actually reached a verdict.**
    #[test]
    fn every_unchecked_state_produces_a_sentence_that_says_not_checked() {
        let p = std::path::PathBuf::from(r"D:\nowhere\addressbook.acrodata");
        let states: [Option<Anchors>; 5] = [
            None,
            Some(Anchors::OptedOut),
            Some(Anchors::NoStore {
                looked_in: vec![p.clone()],
                configured_missing: None,
            }),
            Some(Anchors::NoStore {
                looked_in: Vec::new(),
                configured_missing: Some(p.clone()),
            }),
            Some(Anchors::Unreadable {
                path: p,
                reason: "not a PPKLITE address book".to_owned(),
            }),
        ];
        for state in &states {
            let line = tt::not_checked(&why_not_checked(state.as_ref()));
            assert!(
                line.starts_with("not checked"),
                "state {state:?} produced a trust line that does not say so: {line}"
            );
        }
    }

    /// **The four no-anchor states produce four different explanations.**
    #[test]
    fn the_reason_trust_was_not_checked_is_specific_to_the_state() {
        let p = std::path::PathBuf::from(r"D:\nowhere\addressbook.acrodata");
        let lines = [
            why_not_checked(Some(&Anchors::OptedOut)),
            why_not_checked(Some(&Anchors::NoStore {
                looked_in: vec![p.clone()],
                configured_missing: None,
            })),
            why_not_checked(Some(&Anchors::NoStore {
                looked_in: Vec::new(),
                configured_missing: Some(p.clone()),
            })),
            why_not_checked(Some(&Anchors::Unreadable {
                path: p,
                reason: "not a PPKLITE address book".to_owned(),
            })),
        ];
        for (i, a) in lines.iter().enumerate() {
            for b in lines.iter().skip(i + 1) {
                assert_ne!(a, b, "two no-anchor states share one explanation");
            }
        }
        // The typo case names the path, because the fix is that field.
        assert!(
            lines[2].contains(r"D:\nowhere\addressbook.acrodata"),
            "{}",
            lines[2]
        );
    }

    /// **The trace tokens are distinct, and there is one per variant.**
    #[test]
    fn the_trace_tokens_tell_the_verdicts_apart() {
        let integrity = [
            integrity_token(&Integrity::Verified {
                digest_algorithm: "SHA-256",
                signature_algorithm: "RSA".to_owned(),
            }),
            integrity_token(&Integrity::DigestMismatch),
            integrity_token(&Integrity::SignatureInvalid),
            integrity_token(&Integrity::Unverifiable {
                reason: String::new(),
            }),
        ];
        for (i, a) in integrity.iter().enumerate() {
            for b in integrity.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        let trust = [
            trust_token(&Trust::NotChecked),
            trust_token(&Trust::Trusted {
                anchor_subject: String::new(),
                source: Vec::new(),
                validity_checked: true,
            }),
            trust_token(&Trust::Untrusted {
                reason: String::new(),
            }),
            trust_token(&Trust::SignerUnknown),
        ];
        for (i, a) in trust.iter().enumerate() {
            for b in trust.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }
}
