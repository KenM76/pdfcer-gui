//! Design and rationale: `docs/modules/pdfcer-gui/sign/tests.md`.
#![cfg(test)]
//! Tests for [`super`] — the headless half of signing.
//!
//! ## What is asserted here, and what deliberately is NOT
//!
//! Everything in this file is a **pure function over a value**: the refusal
//! ladder, the visible box's arithmetic, the suggested filename, and the fields
//! that must not be written when the operator left a box empty. Not one of them
//! opens a certificate, and none of them signs anything.
//!
//! That is not a gap — it is R1 stated the right way round. **A passing unit
//! test is not a report of working software**, and the one claim this feature
//! rests on (*the signature is in the file*) cannot be made by a test in this
//! process, because the thing that would produce the file is the same thing
//! that would check it. The evidence for that claim is
//! `tools/ui-verify`'s `signing`, which drives the real binary, writes a real
//! file, and then **reopens it in a fresh process** and reads the signature back
//! through the verification side that shipped as `Pass 10.5`. A different
//! subsystem, in a different process, is the oracle.
//!
//! ⚠ There is also no test here that loads a `.pfx`, and that is a rule rather
//! than an omission: **no certificate is committed to this repository.** A
//! fixture certificate is either somebody's real identity, which must never be
//! in a git history, or a throwaway that expires and starts failing the suite
//! on a date nobody chose. The driven check generates its own, at run time, into
//! a scratch directory, and says so.

use super::*;
use pdfcer_core::page_tree::Rect;

/// A `Standing` with nothing wrong with it.
fn clean() -> Standing {
    Standing {
        encrypted: false,
        redaction_pending: false,
        recovered: false,
        prior_signatures: 0,
        certification_permission: None,
        pages: 3,
        on_disk: true,
        empty_fields: Vec::new(),
        certified: false,
    }
}

/// A `Standing` carrying one plain, signable, pre-placed signature field.
fn with_field(field: SigField) -> Standing {
    Standing {
        empty_fields: vec![field],
        ..clean()
    }
}

/// The plain case: a merged, unlocked, unconstrained, visible box on page 1.
fn plain_field() -> SigField {
    SigField {
        name: "SignHere".to_owned(),
        page: Some(0),
        invisible: false,
        locks: None,
        constrained: false,
        unusable: None,
    }
}

// ---------------------------------------------------------------------------
// `Pass 10.13` — signing into a box somebody else placed
// ---------------------------------------------------------------------------

/// **A pre-placed field reaches the request as a NAME and no rectangle.**
#[test]
fn a_pre_placed_field_is_named_and_carries_no_rectangle() {
    let placement = Placement::ExistingField {
        name: "SignHere".to_owned(),
    };
    // The type system is the assertion: there is no rectangle to set.
    assert!(matches!(placement, Placement::ExistingField { ref name } if name == "SignHere"));
    assert_ne!(placement, Placement::Invisible);
    assert_ne!(placement, Placement::Visible { page: 0 });
}

/// **A field whose widgets are under `/Kids` is listed and not selectable.**
#[test]
fn a_kids_field_is_listed_and_refused_by_name() {
    let field = SigField {
        unusable: Some(FieldBar::HasKids),
        ..plain_field()
    };
    assert!(!field.selectable());
    let standing = with_field(field);
    assert_eq!(standing.empty_fields.len(), 1, "listed, not filtered out");
    assert_eq!(
        standing
            .empty_fields
            .iter()
            .filter(|f| f.selectable())
            .count(),
        0
    );
}

/// **A lock and a seed-value dictionary are carried, not swallowed.**
#[test]
fn a_lock_and_a_seed_value_survive_the_reading() {
    let field = SigField {
        locks: Some("Include".to_owned()),
        constrained: true,
        ..plain_field()
    };
    assert!(field.selectable(), "a lock does not make a field unusable");
    assert_eq!(field.locks.as_deref(), Some("Include"));
    assert!(field.constrained);
}

// ---------------------------------------------------------------------------
// `Pass 10.12` — certifying
// ---------------------------------------------------------------------------

/// **A clean document may be certified.**
#[test]
fn a_clean_document_may_be_certified() {
    assert_eq!(clean().may_certify(), Ok(()));
}

/// **A document that already carries a signature cannot be certified.**
#[test]
fn a_signed_document_cannot_be_certified() {
    let standing = Standing {
        prior_signatures: 2,
        ..clean()
    };
    assert_eq!(
        standing.may_certify(),
        Err(CertifyBar::NotFirst { existing: 2 })
    );
}

/// **An already-certified document is refused for THAT reason, and the
/// order matters.**
#[test]
fn an_already_certified_document_names_the_certification_not_the_count() {
    let standing = Standing {
        certified: true,
        certification_permission: Some(2),
        prior_signatures: 1,
        ..clean()
    };
    assert_eq!(standing.may_certify(), Err(CertifyBar::AlreadyCertified));
}

/// **A document with nothing wrong with it is offered a form.**
#[test]
fn a_clean_document_is_not_refused() {
    assert_eq!(clean().refusal(), None);
}

/// **An encrypted document is refused, by name.**
#[test]
fn an_encrypted_document_is_refused() {
    let standing = Standing {
        encrypted: true,
        ..clean()
    };
    assert_eq!(standing.refusal(), Some(Refusal::Encrypted));
}

/// **A document with a redaction armed is refused, by name.**
///
/// The other reachable one: deferred redaction ships (`Pass 250.2`), and an
/// armed removal is an ordinary mid-session state.
#[test]
fn a_pending_redaction_is_refused() {
    let standing = Standing {
        redaction_pending: true,
        ..clean()
    };
    assert_eq!(standing.refusal(), Some(Refusal::RedactionPending));
}

/// **A pending redaction OUTRANKS encryption, and the order is the
/// operator's next move rather than the severity.**
#[test]
fn a_pending_redaction_is_named_before_encryption() {
    let standing = Standing {
        encrypted: true,
        redaction_pending: true,
        ..clean()
    };
    assert_eq!(standing.refusal(), Some(Refusal::RedactionPending));
}

/// **Only `/DocMDP` 1 refuses; 2 and 3 do not.**
#[test]
fn only_the_strictest_certification_refuses_a_signature() {
    for permission in [2_u8, 3] {
        let standing = Standing {
            certification_permission: Some(permission),
            ..clean()
        };
        assert_eq!(
            standing.refusal(),
            None,
            "/DocMDP {permission} exists to allow signing"
        );
    }
    let standing = Standing {
        certification_permission: Some(1),
        ..clean()
    };
    assert_eq!(
        standing.refusal(),
        Some(Refusal::CertificationForbids { permission: 1 })
    );
}

/// **A recovered base is refused**, and **a document that has never been saved
/// is refused too.**
#[test]
fn a_recovered_base_and_an_unsaved_document_are_both_refused() {
    assert_eq!(
        Standing {
            recovered: true,
            ..clean()
        }
        .refusal(),
        Some(Refusal::RecoveredBase)
    );
    assert_eq!(
        Standing {
            on_disk: false,
            ..clean()
        }
        .refusal(),
        Some(Refusal::NotOnDisk)
    );
}

/// **An already-signed document is NOT refused.**
#[test]
fn a_document_that_is_already_signed_can_be_signed_again() {
    let standing = Standing {
        prior_signatures: 2,
        ..clean()
    };
    assert_eq!(standing.refusal(), None);
}

// ---------------------------------------------------------------------------
// The visible signature's box
// ---------------------------------------------------------------------------

/// **On a US Letter page the box is exactly where the documentation says.**
#[test]
fn the_visible_box_sits_where_the_window_says_it_does() {
    let letter = Rect::from_corners(0.0, 0.0, 612.0, 792.0);
    let r = default_rect(letter);
    assert!((r.urx - 576.0).abs() < 1e-9, "half an inch from the right");
    assert!((r.lly - 36.0).abs() < 1e-9, "half an inch from the bottom");
    assert!((r.urx - r.llx - 180.0).abs() < 1e-9, "180 pt wide");
    assert!((r.ury - r.lly - 60.0).abs() < 1e-9, "60 pt tall");
}

/// **On a page smaller than the box, the box stays ON the page.**
#[test]
fn the_visible_box_is_clamped_onto_a_small_page() {
    let stamp = Rect::from_corners(0.0, 0.0, 100.0, 40.0);
    let r = default_rect(stamp);
    assert!(r.llx >= -1e-9 && r.lly >= -1e-9, "inside the page: {r:?}");
    assert!(
        r.urx <= 100.0 + 1e-9 && r.ury <= 40.0 + 1e-9,
        "inside: {r:?}"
    );
    assert!(r.urx > r.llx && r.ury > r.lly, "still a rectangle: {r:?}");
}

/// **A page whose origin is not (0, 0) still gets the box in ITS corner.**
#[test]
fn the_visible_box_follows_an_offset_page_origin() {
    let offset = Rect::from_corners(200.0, 100.0, 812.0, 892.0);
    let r = default_rect(offset);
    assert!((r.urx - 776.0).abs() < 1e-9, "right edge minus 36: {r:?}");
    assert!((r.lly - 136.0).abs() < 1e-9, "bottom edge plus 36: {r:?}");
}

// ---------------------------------------------------------------------------
// What is written, and what is left out
// ---------------------------------------------------------------------------

/// **An untouched field is OMITTED, and one holding only spaces counts as
/// untouched.**
#[test]
fn an_empty_or_blank_field_is_left_out_of_the_signature() {
    assert_eq!(non_empty(""), None);
    assert_eq!(non_empty("   "), None);
    assert_eq!(non_empty("\t \n"), None);
    assert_eq!(non_empty("  Approved  "), Some("Approved".to_owned()));
}

/// **The suggestion is never the source file.**
///
/// The standing rule for every write that produces a second document: a safe
/// default is a mechanism, and a warning is something to click past.
#[test]
fn the_suggested_name_is_never_the_document_it_came_from() {
    let source = std::path::Path::new("D:/drawings/SW41177.pdf");
    let suggested = suggested_path(source);
    assert_ne!(suggested, source);
    assert_eq!(suggested.parent(), source.parent(), "same folder");
    assert_eq!(
        suggested.file_name().and_then(|n| n.to_str()),
        Some("SW41177-signed.pdf")
    );
}
