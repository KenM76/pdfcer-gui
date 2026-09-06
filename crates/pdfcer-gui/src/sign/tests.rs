#![cfg(test)]
//! Tests for [`super`] — the headless half of signing.
//!
//! ## ★★★ What is asserted here, and what deliberately is NOT
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
///
/// ★ Built field by field rather than read off a document, because every test
/// below is about **the ladder**, not about the reading. `Standing::read` is
/// exercised by the driven check, where a real document is open.
fn clean() -> Standing {
    Standing {
        encrypted: false,
        redaction_pending: false,
        recovered: false,
        prior_signatures: 0,
        certification_permission: None,
        pages: 3,
        on_disk: true,
    }
}

/// **A document with nothing wrong with it is offered a form.**
///
/// The negative control for every test below it. Without this one, a build
/// whose `refusal` returned `Some` unconditionally would pass all five of the
/// refusal tests and never sign anything.
#[test]
fn a_clean_document_is_not_refused() {
    assert_eq!(clean().refusal(), None);
}

/// **An encrypted document is refused, by name.**
///
/// One of the two refusals the build brief names as *reachable rather than
/// theoretical*: File ▸ Security ▸ Encrypt… ships, so this shell can produce an
/// encrypted document and then be asked to sign it, in one session, without
/// leaving the application.
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

/// ★★★ **A pending redaction OUTRANKS encryption, and the order is the
/// operator's next move rather than the severity.**
///
/// Both are true of a document that was redacted and then encrypted — which
/// this shell can produce, because `set_encryption` ignores a pending redaction
/// (a defect already filed at the engine). One sentence is shown, so which one
/// matters: the redaction is **one press away** from being applied or called
/// off, and the encryption is a wall. Naming the wall while the gate beside it
/// is merely latched spends the only sentence the operator reads on the answer
/// they cannot act on.
///
/// A build that reversed this would look correct in every screenshot.
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
///
/// ★★ The arm most likely to be written wrong, and the engine's own comment
/// says why: *"Table 254: P = 1 permits NO changes; 2 permits form fill-in AND
/// signing; 3 adds annotations. Adding a signature is the act P = 2 exists to
/// allow."* A shell that refused every certified document would refuse the
/// commonest legitimate case there is — a document certified precisely so that
/// other people could sign it.
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
///
/// The second has no engine counterpart — see §4 of the module header. The
/// engine would sign a document with no file behind it; this shell will not,
/// because an incremental update is an appendix to a specific file and the
/// operator's next ordinary Save would write a different one.
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
///
/// ★ The one place this surface differs from [`crate::protect`], which refuses
/// a signed document outright because encrypting rewrites every byte a
/// signature covers. Signing appends, so a second signature is legitimate and
/// PDF is built for it. The window says so rather than staying silent, but it
/// does not stand in the way.
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
///
/// ★★ Asserted as four numbers rather than as "near the corner", because the
/// box is **content written into the operator's file** — R8b Rule 4 — and
/// `crate::text::sign::placement_where` states the measurements on screen. A
/// sentence on a window and a constant in a file that could disagree is the
/// shape of a disclosure that stops being true.
#[test]
fn the_visible_box_sits_where_the_window_says_it_does() {
    let letter = Rect::from_corners(0.0, 0.0, 612.0, 792.0);
    let r = default_rect(letter);
    assert!((r.urx - 576.0).abs() < 1e-9, "half an inch from the right");
    assert!((r.lly - 36.0).abs() < 1e-9, "half an inch from the bottom");
    assert!((r.urx - r.llx - 180.0).abs() < 1e-9, "180 pt wide");
    assert!((r.ury - r.lly - 60.0).abs() < 1e-9, "60 pt tall");
}

/// ★★★ **On a page smaller than the box, the box stays ON the page.**
///
/// The clamp, and it is not defensive tidiness. The engine accepts whatever
/// rectangle it is handed; a widget laid partly outside the media box is
/// present in the file and drawn by nothing, so the operator would tick
/// *"draw a signature box"*, get a valid signed document, and see no box. That
/// is a feature that traces perfectly and does nothing — the exact class this
/// project has shipped before.
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
///
/// Real CAD exports carry offset media boxes. Computing from `0` rather than
/// from the page's own lower-left would put the box a page-width away from
/// where the operator expects it, on exactly the documents this operator works
/// with and on none of the fixtures.
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

/// ★★ **An untouched field is OMITTED, and one holding only spaces counts as
/// untouched.**
///
/// `/Reason ()` in a signature dictionary is a claim that the operator gave a
/// reason and it was nothing — a different statement from a key that is not
/// there. And a field holding one space is an untouched field as far as anybody
/// looking at the screen is concerned; writing `/Reason ( )` into a legal
/// document because of a stray keystroke is the kind of thing nobody ever
/// finds.
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
