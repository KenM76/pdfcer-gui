//! # `redact::tests::reach` — the redaction reach setting, measured on the
//! saved bytes
//!
//! The seam from [`super`] is the reason to change, not the line count.
//! [`super`] proves the **apply path** is sound: that a redaction pdfcer
//! accepts leaves no recoverable trace, that a refusal refuses, that a
//! staging can be cancelled. Those assertions are about a fixed scope and
//! would read identically if the reach setting did not exist. This file is
//! about the setting itself — one operator preference, two outcomes in the
//! file, and the disclosure that has to accompany the narrower one.
//!
//! It carries its own fixture, and deliberately: [`super::secret_pdf`] hides
//! the secret nowhere but the page, which is exactly the document on which
//! every reach behaves the same. A fixture that cannot distinguish the values
//! under test would make every assertion here vacuous.

// The inner `#![cfg(test)]` is what `tools/gates/check-ui-strings.sh`
// exclusion 2 recognises as a test-only FILE.
#![cfg(test)]

use super::*;

/// A fixture whose page shows [`SECRET`] **and** whose document-information
/// dictionary holds a second copy of it.
///
/// `/Info` is the commonest carrier there is and the one an operator never
/// sees: a CAD exporter writes the drawing title into it, and the title is
/// very often the words he is redacting. The page copy is what he marks; the
/// trailer copy is what the reach setting decides the fate of.
fn secret_on_the_page_and_in_the_properties() -> Vec<u8> {
    let content = format!("BT /F1 12 Tf 20 100 Td ({SECRET}) Tj ( KEEPTHIS) Tj ET");
    let stream = format!(
        "<< /Length {} >>{nl}stream{nl}{content}{nl}endstream",
        content.len(),
        nl = "\n"
    );
    let info = format!("<< /Title ({SECRET}) /Author (KEEPTHIS) >>");
    assemble_with_trailer(
        &[
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 200] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
            &stream,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
            &info,
        ],
        "/Info 6 0 R ",
    )
}

/// **The reach setting changes the saved bytes, and it is the setting that
/// changes them.**
///
/// Threading a parameter through a call chain compiles whether or not the
/// value arrives anywhere that matters, and every test that asserts only
/// *"the argument was passed"* passes on a build where the engine ignores it.
/// So this one measures the file: the same marks, the same document, two
/// reaches, and the copy in the document properties survives exactly one of
/// them.
///
/// Three further properties ride along, and each is a defect if it inverts:
///
/// * **The page copy dies under both.** Without that control a build that
///   redacted nothing at all would satisfy the headline assertion under the
///   narrow reach.
/// * **`KEEPTHIS` survives both**, so neither reach is achieving its result by
///   destroying the document.
/// * **The narrow reach still saves.** It saves behind the residual
///   acknowledgement, and that is correct rather than a gate to remove: this
///   shell's own byte sweep of the finished file genuinely finds the secret in
///   it, and the tick is the operator confirming he is keeping a file that
///   still contains the text he marked. What must never happen is a refusal he
///   cannot satisfy, and this pins which of the two it is.
#[test]
fn the_reach_setting_decides_whether_the_copy_in_the_properties_survives() {
    for (reach, copy_survives) in [
        (RedactionReach::MarkedOnly, true),
        (RedactionReach::HiddenCarriers, false),
    ] {
        // The acknowledgement answers the residual, and under the narrow reach
        // there is one to answer: the copy this run is choosing to leave.
        let acknowledgement = if copy_survives {
            ResidualAcknowledgement::Given
        } else {
            ResidualAcknowledgement::Withheld
        };
        let doc = Document::from_bytes(secret_on_the_page_and_in_the_properties())
            .expect("the fixture must parse");
        let mut session = EditSession::new(doc);
        let created = session
            .mark_redactions_by_search(SECRET, false)
            .expect("the fixture's page text is extractable");
        assert!(!created.is_empty(), "the search must find the secret");

        let prepared = prepare_redaction_apply(&session, reach)
            .unwrap_or_else(|err| panic!("the apply must succeed under {reach:?}: {err:?}"));
        let target = scratch(&format!("reach-{}.pdf", reach.key()));
        let _ = std::fs::remove_file(&target);
        prepared
            .write_to(&target, acknowledgement)
            .unwrap_or_else(|err| {
                panic!(
                    "an acknowledged redaction must save under every reach, and did \
                     not under {reach:?}: {err:?}"
                )
            });
        let bytes = std::fs::read(&target).expect("the redacted file must exist");

        assert_eq!(
            proof::contains(&bytes, SECRET.as_bytes()),
            copy_survives,
            "under {reach:?} the copy in the document properties should \
             {} have survived the save",
            if copy_survives { "" } else { "NOT" }
        );
        assert_eq!(
            proof::survivors_in_content_streams(&bytes, &[SECRET.to_owned()]),
            None,
            "the marked copy on the PAGE must die under every reach, and did \
             not under {reach:?}"
        );
        assert!(
            proof::contains(&bytes, b"KEEPTHIS"),
            "under {reach:?} the redaction took content nobody marked"
        );
    }
}

/// **The narrow reach reports the copy it left, by carrier, in the report the
/// dialog draws from.**
///
/// The saved-bytes assertion above proves the setting reaches the engine; this
/// proves the *disclosure* does. A reach that silently leaves a copy behind is
/// the "sneaky" half of "fuzzy, never sneaky" — the operator chose a narrower
/// scope and is owed the census of what that choice costs, off-canvas, before
/// he commits.
#[test]
fn a_narrow_reach_names_the_copy_it_declines_to_touch() {
    use pdfcer_core::redact::CarrierAction;

    let doc = Document::from_bytes(secret_on_the_page_and_in_the_properties())
        .expect("the fixture must parse");
    let mut session = EditSession::new(doc);
    session
        .mark_redactions_by_search(SECRET, false)
        .expect("the fixture's page text is extractable");

    let narrow = prepare_redaction_apply(&session, RedactionReach::MarkedOnly)
        .expect("the narrow apply must succeed");
    let declined: Vec<&str> = narrow
        .report
        .carriers
        .iter()
        .filter(|c| c.action == CarrierAction::FoundNotScrubbed)
        .map(|c| c.carrier)
        .collect();
    assert!(
        !declined.is_empty(),
        "the narrow reach left a copy in the document properties and reported \
         nothing about it: {:?}",
        narrow.report.carriers
    );

    let wide = prepare_redaction_apply(&session, RedactionReach::HiddenCarriers)
        .expect("the wide apply must succeed");
    assert!(
        wide.report
            .carriers
            .iter()
            .all(|c| c.action != CarrierAction::FoundNotScrubbed),
        "the default reach declines nothing, so nothing may be reported as \
         declined: {:?}",
        wide.report.carriers
    );
}
