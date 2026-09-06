//! # `panels::signatures` — three facts about each digital signature, reported
//! separately and never merged
//!
//! Salvaged from the old shell's `panels_structure.rs` as a coverage-only
//! report, and **rewritten on 2026-09-05** when `pdfcer-core`'s `Pass 10.1`–
//! `10.5` made the other two facts answerable.
//!
//! ## ★★★ THE RULE THIS PANEL EXISTS TO KEEP
//!
//! **This is the one place in the product where a wrong answer is worse than no
//! answer.** A panel that said *"trusted"* about a chain it had not really
//! validated would be the exact failure mode the engine's design prevents — and
//! the engine prevents it by refusing to produce one verdict at all:
//! `SignatureVerdict` carries `integrity`, `coverage` and `trust` as **three
//! facts that never collapse into one bool**.
//!
//! So this panel draws three labelled lines per signature, always all three, in
//! that order. There is no badge, no tick, no colour that means "fine", and no
//! arithmetic anywhere in this file that combines two of the facts into a
//! third. The order is also the order of increasing uncertainty: integrity is
//! arithmetic, coverage is arithmetic, trust is a judgement about the world.
//!
//! ★★ And [`pdfcer_core::signature::Trust::NotChecked`] renders **as itself**.
//! Not as a soft "no", not as a grey tick, not omitted. Hiding it would make
//! this panel indistinguishable, on screen, from one that had checked and found
//! nothing wrong — which is the inversion the whole feature exists to prevent.
//! [`crate::text::trust`] holds four separate sentences for the four reasons
//! trust can go unchecked, because *"you turned it off"* and *"your Acrobat
//! store is corrupt"* are opposite calls to action.
//!
//! ## What changed at the rewrite, and what deliberately did not
//!
//! | | before | now |
//! |---|---|---|
//! | integrity | not computed; the panel said so in its first line | `verify_all_with_trust` |
//! | coverage | `byte_range_coverage`, per frame | unchanged — it is the same measurement, now printed beside the other two |
//! | trust | not computed | the operator's own Acrobat anchors, opt-in, off by default |
//! | the leading caveat | *"pdfcer does not check whether these signatures are valid — it cannot yet"* | replaced: that sentence became FALSE the moment this was wired |
//!
//! ★★★ That last row is this project's most expensive recorded failure shape —
//! a claim that was true when written and false within hours, with the prose
//! around it still true. The replacement
//! ([`crate::text::trust::panel_intro`]) names the three facts instead of
//! denying one of them, so it cannot go stale the same way: it describes the
//! SHAPE of the report rather than the limits of a build.
//!
//! ## The file length, and the bytes, are read from DISK
//!
//! `/ByteRange` is a claim about bytes, so it can only be checked against
//! bytes — which is why
//! [`pdfcer_core::signature::byte_range_coverage`] takes the length as a
//! parameter rather than reading it from the object graph: *"the object model
//! cannot check a claim about bytes against itself."* Verification needs the
//! same thing one step further: the real file, digested.
//!
//! What is used is the file **on disk right now**, not a length captured when
//! the document was opened, and not the session's rendering of it. It answers
//! *"do these signatures hold over the file as it currently exists"*, which is
//! the question worth asking. An unsaved edit is not in the file and cannot be
//! covered by a signature; the panel says which state it measured rather than
//! leaving an operator to assume.
//!
//! ## ★★ Why verification is automatic, and what stops it being per-frame
//!
//! The alternative considered was a *Check signatures* button. It was refused:
//! an operator who has opened a panel called Signatures has already asked, and
//! a button would leave the panel's resting state showing coverage numbers with
//! no integrity beside them — which is precisely the state this work exists to
//! end.
//!
//! What makes that affordable is [`crate::trust::cached_report`], whose key
//! carries the file's identity, its length, its modification time and **both**
//! halves of the trust configuration. A cache that missed any of those would be
//! worse than none: the panel would show a verdict about a file, a setting or
//! an anchor set that is no longer in force, and it would look exactly like a
//! correct answer.
//!
//! ## This panel raises no actions, and cannot
//!
//! Nothing about a signature is editable from anywhere in pdfcer — the engine
//! cannot sign, and this shell would have nothing to send it if it could. The
//! `actions` parameter is present because the dock calls every panel through
//! one signature; this one never pushes to it.

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

    // ★ The report is computed here rather than inside the row loop, because it
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

                // ★ Matched by INDEX, and the engine guarantees it: `verify_all`'s
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

                crate::diag::trace(|| {
                    // ★★★ **`field_token`, not `{:?}` — corrected 2026-09-06.**
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
                    // ★ Fixed at the emitter rather than in the reader, and the
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
///
/// `None` means the file could not be read at all, which is neither a pass nor
/// a failure and is reported as the coverage half already reports it: as
/// pdfcer's inability to look, not as a statement about the document.
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
        // ★ `Integrity` is `#[non_exhaustive]`, so this arm is required by the
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

/// Fact 2 — what the signature covers, unchanged from the panel's first
/// version.
///
/// The two malformedness reports come FIRST, because they change what the
/// coverage numbers mean: a reader that rejects the array computes something
/// else, or nothing.
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
///
/// ★★★ The four `NotChecked` sentences are chosen from the [`Anchors`] state
/// rather than from the verdict, and that is the whole design. The engine
/// reports `NotChecked` identically whether the operator opted out, has no
/// store, typed a wrong path, or has a corrupt store — it cannot know which,
/// because it was simply handed no anchors. This shell DOES know, because it is
/// the half that looked, and reporting all four as one sentence would be this
/// panel discarding the only thing it can contribute.
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
///
/// `None` — no report at all — is the never-saved document, and it takes the
/// opted-out sentence deliberately: there is no file to verify, the anchors were
/// never consulted, and inventing a fifth sentence for a case the coverage half
/// already reports would be two surfaces explaining one absence.
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
///
/// Drawn only when there ARE anchors. The three no-anchor states are explained
/// per signature instead, on the trust line, because that is where an operator
/// is asking the question — and a store disclosure above a list of signatures
/// whose trust was never checked would be a header about nothing.
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
///
/// ★ A `horizontal_wrapped` rather than a `format!("{label} {said}")`, so the
/// three labels line up as a column and the sentences wrap under themselves.
/// The alignment is not decoration: three facts printed as three unlabelled
/// paragraphs is three facts a reader has to sort out, which is the first step
/// back towards reading them as one verdict.
fn labelled(ui: &mut egui::Ui, label: &str, said: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(egui::RichText::new(label).small().weak());
        ui.label(said);
    });
}

/// The signature field's name as one trace token, or `none`.
///
/// ★★ Exists because the call site used to write `field={:?}` over an
/// `Option<String>`, emitting `field=Some("SignHere")` into a `key=value`
/// trace that readers split on whitespace. **Never Debug-format a value a
/// machine reads** — the third instance in this project in one week, and the
/// only one that had not yet cost a false failure report.
///
/// ★ `none` rather than an empty string, matching every other token helper
/// here: an empty value after `=` is indistinguishable from a truncated line,
/// and a check cannot tell the two apart.
fn field_token(name: Option<&str>) -> &str {
    name.unwrap_or("none")
}

/// The integrity verdict as one trace token.
///
/// ★ Not operator copy and not in the catalog: a driven check matches on it,
/// and a check that matched translated prose would break the day the prose
/// improved. Same argument as `crate::trust`'s anchor token.
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
    ///
    /// The first is a statement about the document. The second is a
    /// statement about pdfcer's ability to look, and rendering it as the
    /// first would be a claim about the operator's file made from an
    /// inability to read it.
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

    /// ★★★ **The leading sentence names three facts and says they are never
    /// merged.**
    ///
    /// This replaces `the_caveat_denies_validity_checking_explicitly`, which
    /// asserted the OLD caveat's *"does not check … valid"* wording. That
    /// wording became false the moment `verify_all_with_trust` was wired, and a
    /// test pinning it would have kept a false sentence on screen while passing
    /// — which is this project's most expensive recorded failure shape wearing
    /// a green tick.
    ///
    /// What is asserted instead is the property that does NOT expire: the panel
    /// tells a reader, before they start, that there are three answers and that
    /// pdfcer will not fold them into one.
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

    /// ★★★ **Every state this panel can be in produces a trust sentence that
    /// says "not checked", unless the engine actually reached a verdict.**
    ///
    /// The assertion that stands between an operator and a silent grey row.
    /// Driven over the four anchor states plus the never-saved case, through
    /// the same function the panel calls — so a branch added to
    /// [`why_not_checked`] without a sentence fails here rather than rendering
    /// an empty label.
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
    ///
    /// [`why_not_checked`] is a `match`, and a `match` with two arms returning
    /// the same string compiles perfectly. This is what refuses that: the four
    /// call for four different actions — turn the setting on, install Acrobat,
    /// fix your typo, your store is corrupt — and telling somebody the wrong
    /// one sends them to the wrong place.
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

    /// ★★ **The trace tokens are distinct, and there is one per variant.**
    ///
    /// A driven check reads `integrity=` and `trust=` off the row line. Two
    /// variants sharing a token would make a check that asserts *"the trust
    /// verdict changed when the setting was turned on"* pass against a build
    /// where it did not.
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
