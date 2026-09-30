//! # `text::trust` — every word this shell says about whether a signature can
//! be trusted
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/trust.md`.

use pdfcer_core::trust_store::SourceCounts;

// ---------------------------------------------------------------------------
// The panel's leading disclosure
// ---------------------------------------------------------------------------

/// The sentence above the list, replacing the old *"pdfcer does not check
/// whether these signatures are valid — it cannot yet"*.
#[must_use]
pub const fn panel_intro() -> &'static str {
    "For each signature below, pdfcer reports three separate facts and never \
     merges them: whether the signed bytes are INTACT, what the signature \
     COVERS, and whether the signer can be TRUSTED. A signature can be intact \
     and untrusted, or trusted and cover only part of the file."
}

/// The heading before a signature's three facts.
#[must_use]
pub fn signature_heading(name: &str) -> String {
    format!("Signature: {name}")
}

// ---------------------------------------------------------------------------
// Fact 1 — integrity
// ---------------------------------------------------------------------------

/// The label the integrity line always begins with.
#[must_use]
pub const fn integrity_label() -> &'static str {
    "Intact:"
}

/// The digest matched and the CMS signature verified.
#[must_use]
pub fn integrity_verified(digest: &str, signature: &str) -> String {
    format!("yes — the signed bytes are exactly what was signed ({digest}, {signature}).")
}

/// A SHA-1 digest, disclosed beside a verdict that is otherwise good news.
#[must_use]
pub const fn integrity_weak_digest() -> &'static str {
    "This signature uses SHA-1, which is no longer considered strong enough to \
     rule out a forged document. The check passed; what it proves is weaker \
     than the same check with SHA-256."
}

/// The covered bytes were altered after signing.
#[must_use]
pub const fn integrity_digest_mismatch() -> &'static str {
    "NO — the bytes this signature covers have been ALTERED since it was \
     signed. This is not a coverage question and not a trust question: what \
     was signed and what is in the file are different."
}

/// The digest matched but the signature value did not verify.
#[must_use]
pub const fn integrity_signature_invalid() -> &'static str {
    "NO — the covered bytes are what was signed, but the signature itself does \
     not verify against the signer's certificate. The signature, the \
     certificate or the signed attributes have been changed."
}

/// pdfcer could not reach a verdict, and says why in the engine's words.
#[must_use]
pub fn integrity_unverifiable(reason: &str) -> String {
    format!("pdfcer could not tell — {reason}. This is not a pass and not a failure.")
}

// ---------------------------------------------------------------------------
// Fact 2 — coverage
// ---------------------------------------------------------------------------

/// The label the coverage line always begins with.
#[must_use]
pub const fn coverage_label() -> &'static str {
    "Covers:"
}

// ---------------------------------------------------------------------------
// Fact 3 — trust
// ---------------------------------------------------------------------------

/// The label the trust line always begins with.
#[must_use]
pub const fn trust_label() -> &'static str {
    "Signer:"
}

/// The signer chains to a trusted anchor.
#[must_use]
pub fn trusted(anchor_subject: &str, source: &[String], validity_checked: bool) -> String {
    let provenance = if source.is_empty() {
        String::new()
    } else {
        format!(" ({})", source.join(", "))
    };
    let validity = if validity_checked {
        "The certificates were inside their validity dates at the time of signing."
    } else {
        "This signature carries no signing time, so pdfcer could NOT check whether \
         the certificates had expired."
    };
    format!(
        "chains to a trusted certificate — {anchor_subject}{provenance}. Every link \
         was checked by signature, and the issuing certificates were entitled to \
         issue. {validity} This does not say whether a certificate was revoked; \
         the Revocation line below does."
    )
}

/// Trust was evaluated and the signer does not chain to a trusted anchor.
#[must_use]
pub fn untrusted(reason: &str) -> String {
    format!(
        "does NOT chain to any certificate in your trust list — {reason}. That is \
         a statement about who signed it, not about whether the bytes are intact; \
         a signature can be perfectly valid and still be from somebody your trust \
         list has never heard of."
    )
}

/// Trust was requested and the signer's certificate could not be parsed.
#[must_use]
pub const fn signer_unknown() -> &'static str {
    "could not be identified — pdfcer could not read the certificate embedded in \
     this signature, so trust could not even be attempted. This is not a \
     judgement about the signer."
}

/// The prefix every unchecked-trust sentence begins with.
#[must_use]
pub const fn not_checked_prefix() -> &'static str {
    "not checked"
}

/// The trust line when no anchors were available, with which of the four
/// situations applies.
#[must_use]
pub fn not_checked(why: &str) -> String {
    format!("{} — {why}", not_checked_prefix())
}

/// The setting is off.
#[must_use]
pub const fn not_checked_opted_out() -> &'static str {
    "pdfcer did not look at who signed this, because checking signers is turned \
     off. You can let pdfcer use the trust list your Acrobat has already \
     downloaded: Settings > Digital signatures."
}

/// The setting is on and this machine has no Acrobat trust list.
#[must_use]
pub fn not_checked_no_store(looked_in: usize) -> String {
    format!(
        "pdfcer looked for the trust list an installed Acrobat or Reader \
         downloads, in {looked_in} place(s), and found none on this machine. If \
         yours is somewhere else, point pdfcer at it in Settings > Digital \
         signatures."
    )
}

/// The operator configured a path and nothing is there.
#[must_use]
pub fn not_checked_configured_missing(path: &str) -> String {
    format!(
        "pdfcer was told to use the trust list at {path} and there is no file \
         there. Nothing else was tried, because a path you typed is a choice \
         rather than a hint — correct it in Settings > Digital signatures, or \
         clear the field to let pdfcer look in the usual places."
    )
}

/// A store was found and could not be read.
#[must_use]
pub fn not_checked_unreadable(path: &str, reason: &str) -> String {
    format!(
        "pdfcer found a trust list at {path} and could not read it — {reason}. \
         Signers were not checked at all; nothing here is a statement about this \
         document."
    )
}

/// The small heading over the revocation locations a signature's
/// certificates name.
#[must_use]
pub fn revocation_label() -> &'static str {
    "Revocation info at"
}

/// One certificate's revocation locations, as the certificate states them.
/// None of them was fetched, and the line says so, because a URL beside a
/// signature reads as a check that happened.
#[must_use]
pub fn revocation_sources(
    subject: &str,
    crl: &[String],
    ocsp: &[String],
    ca_issuers: &[String],
    unreadable: usize,
) -> String {
    let mut parts = Vec::new();
    if !crl.is_empty() {
        parts.push(format!("CRL {}", crl.join(", ")));
    }
    if !ocsp.is_empty() {
        parts.push(format!("OCSP {}", ocsp.join(", ")));
    }
    if !ca_issuers.is_empty() {
        parts.push(format!("issuer certificate {}", ca_issuers.join(", ")));
    }
    if unreadable > 0 {
        parts.push(format!("{unreadable} more this build cannot read"));
    }
    format!(
        "{subject}: {}. Not fetched — pdfcer does not go online.",
        parts.join("; ")
    )
}

// ---------------------------------------------------------------------------
// The revocation verdict — a fourth fact, never folded into trust
// ---------------------------------------------------------------------------

/// The label before the revocation verdict.
#[must_use]
pub const fn revocation_verdict_label() -> &'static str {
    "Revocation:"
}

/// Where the revocation lists came from: the document's own, or ones the
/// operator supplied.
#[must_use]
pub const fn revocation_list_origin(in_document: bool) -> &'static str {
    if in_document {
        "carried in the document"
    } else {
        "you supplied"
    }
}

/// The kinds of revocation evidence that answered, as a plural noun phrase.
#[must_use]
pub const fn revocation_evidence(crl: bool, ocsp: bool) -> &'static str {
    match (crl, ocsp) {
        (false, true) => "OCSP responses",
        (true, true) => "revocation lists and OCSP responses",
        _ => "revocation lists",
    }
}

/// One piece of revocation evidence, with its article.
#[must_use]
pub const fn revocation_evidence_one(ocsp: bool) -> &'static str {
    if ocsp {
        "an OCSP response"
    } else {
        "a revocation list"
    }
}

/// No revocation evidence was available, so nothing was checked.
#[must_use]
pub const fn revocation_not_checked() -> &'static str {
    "not checked — this document carries no revocation lists or OCSP responses, and \
     pdfcer does not go online to fetch one."
}

/// Every certificate in the chain was covered by evidence not showing it
/// revoked. `evidence` is [`revocation_evidence`]; `as_of` the newest
/// `thisUpdate` among them, when stated.
#[must_use]
pub fn revocation_good(
    certificates: usize,
    in_document: bool,
    evidence: &str,
    as_of: Option<&str>,
) -> String {
    let which = if certificates == 1 {
        "The signer's certificate is".to_owned()
    } else {
        format!("All {certificates} certificates are")
    };
    let when = as_of.map_or_else(
        || "as of when that evidence was issued".to_owned(),
        |d| format!("as of {d}"),
    );
    format!(
        "not revoked. {which} shown good by the {evidence} {}, {when}.",
        revocation_list_origin(in_document)
    )
}

/// A revocation list names a certificate in the signer's chain.
///
/// `before_signing` compares the revocation date with the signing time the
/// signature claims; `None` when either is unknown.
#[must_use]
pub fn revocation_revoked(
    subject: &str,
    date: Option<&str>,
    reason: Option<&str>,
    before_signing: Option<bool>,
    ocsp: bool,
    in_document: bool,
) -> String {
    let when = date.map_or_else(String::new, |d| format!(" on {d}"));
    let why = reason.map_or_else(String::new, |r| format!(" ({})", revocation_reason(r)));
    let order = match before_signing {
        Some(true) => {
            "That is before the time this signature claims, so it was made with a revoked \
             certificate."
        }
        Some(false) => {
            "That is after the time this signature claims, when the certificate was still \
             good — if that claimed time is honest."
        }
        None => "pdfcer cannot tell whether that was before or after signing.",
    };
    format!(
        "REVOKED — {subject} was revoked{when}{why}, per {} {}. {order}",
        revocation_evidence_one(ocsp),
        revocation_list_origin(in_document)
    )
}

/// Appended to a good verdict when some evidence names no next update: the
/// issuer promised no date by which it would have said otherwise.
#[must_use]
pub const fn revocation_open_ended() -> &'static str {
    "Some of that evidence names no date for its next update, so newer information \
     may exist."
}

/// Revocation lists were available and the chain could not be fully checked.
#[must_use]
pub fn revocation_undetermined(reason: &str) -> String {
    format!("could not be decided — {}.", reason.trim_end_matches('.'))
}

/// An RFC 5280 `reasonCode` name in plain words; an unknown one verbatim.
#[must_use]
pub fn revocation_reason(code: &str) -> &str {
    match code {
        "keyCompromise" => "its key was compromised",
        "cACompromise" => "its issuer's key was compromised",
        "affiliationChanged" => "the holder's affiliation changed",
        "superseded" => "it was replaced",
        "cessationOfOperation" => "it is no longer used",
        "certificateHold" => "it is on hold",
        "privilegeWithdrawn" => "its privileges were withdrawn",
        "aACompromise" => "its attribute authority was compromised",
        "unspecified" => "no reason given",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// The anchor set's provenance
// ---------------------------------------------------------------------------

/// **How many anchors, from where, and how old** — one sentence, always.
#[must_use]
pub fn store_line(path: &str, modified: Option<&str>, counts: &SourceCounts) -> String {
    let dated = match modified {
        Some(date) => format!("last updated by Acrobat on {date}"),
        None => "with no readable date, so pdfcer cannot tell you how current it is".to_owned(),
    };
    format!(
        "Using {total} trusted certificates from {path} — {dated}. \
         {aatl} from Adobe's approved list (AATL), {eutl} from the EU trusted \
         lists, {adbe} from Adobe itself, {other} from elsewhere.",
        total = counts.total,
        aatl = counts.aatl,
        eutl = counts.eutl,
        adbe = counts.adbe,
        other = counts.other,
    )
}

/// Entries in the store whose certificate could not be decoded.
#[must_use]
pub fn store_undecodable(count: usize) -> String {
    format!(
        "{count} entr(ies) in that list could not be read and were left out of \
         the check, so a signer that relies on one of them will read as untrusted."
    )
}

/// The at-own-risk disclosure, shown wherever the store is turned on or
/// inspected.
#[must_use]
pub const fn at_own_risk() -> &'static str {
    "This reads a file that belongs to Adobe's program, on your own machine, and \
     nothing leaves it. Whether relying on Adobe's downloaded trust list fits \
     your Acrobat or Reader licence is your decision — pdfcer does not make that \
     determination for you, which is why this is off until you turn it on."
}

// ---------------------------------------------------------------------------
// The Settings group
// ---------------------------------------------------------------------------

/// The group heading.
#[must_use]
pub const fn group_signatures() -> &'static str {
    "Digital signatures"
}

/// Setting 1 — the opt-in. Its title.
#[must_use]
pub const fn use_store_title() -> &'static str {
    "Checking who signed a document"
}

/// What the standard leaves open here.
#[must_use]
pub const fn use_store_silence() -> &'static str {
    "The standard says a reader should check who signed a document and cannot \
     say whose certificates you trust. Adobe's approved list and the EU trusted \
     lists are the answer most people mean, and neither is published in a form a \
     program can just download — the only copy on this machine is the one your \
     Acrobat or Reader already fetched."
}

/// Which way costs what.
#[must_use]
pub const fn use_store_radius() -> &'static str {
    "Affects only what pdfcer TELLS you about a signature. It never changes a \
     document, never writes anything, and never uses the network. It also \
     applies to the pdfcer command line, because it is one choice in one file."
}

/// The off option.
#[must_use]
pub const fn use_store_off_label() -> &'static str {
    "Do not check who signed (the default)"
}

/// The off option's note.
#[must_use]
pub const fn use_store_off_note() -> &'static str {
    "Signatures are still checked for whether their bytes are intact and what \
     they cover. Who signed them is reported as not checked."
}

/// The at-own-risk option.
#[must_use]
pub const fn use_store_on_label() -> &'static str {
    "Use the trust list my Acrobat has downloaded, at my own risk"
}

/// The at-own-risk option's note.
#[must_use]
pub const fn use_store_on_note() -> &'static str {
    "pdfcer reads Acrobat's own downloaded list of trusted certificates and uses \
     it to say whether a signer chains to one of them. It checks the certificate \
     chain, whether each issuer was entitled to issue, and the dates at the time \
     of signing. It does NOT check whether a certificate has since been revoked."
}

/// Setting 2 — where the store is. Its title.
#[must_use]
pub const fn store_path_title() -> &'static str {
    "Where the trust list is"
}

/// What is unsettled here.
#[must_use]
pub const fn store_path_silence() -> &'static str {
    "Adobe does not document where this file lives, and it moves between \
     versions. pdfcer looks in the places every Acrobat and Reader release has \
     used, which is a convention rather than a rule — a redirected profile or a \
     version pdfcer has not been told about will not be found."
}

/// Which way costs what.
#[must_use]
pub const fn store_path_radius() -> &'static str {
    "Changes only which file pdfcer reads certificates from. It is read-only and \
     pdfcer never writes to it."
}

/// The field's label.
#[must_use]
pub const fn store_path_label() -> &'static str {
    "Trust list file (leave blank to look in the usual places)"
}

/// The note under the field.
#[must_use]
pub const fn store_path_note() -> &'static str {
    "This is only a location. Whether pdfcer may read it at all is the setting \
     above."
}

/// The picker button.
#[must_use]
pub const fn store_path_browse() -> &'static str {
    "Browse…"
}

/// The picker button's tooltip.
#[must_use]
pub const fn store_path_browse_hover() -> &'static str {
    "Find an addressbook.acrodata file — the list of trusted certificates \
     Acrobat and Reader download."
}

/// The label of the picker's file filter.
#[must_use]
pub const fn store_path_filter() -> &'static str {
    "Acrobat trust list"
}

/// What pdfcer currently resolves, when a usable store was found.
#[must_use]
pub fn resolved_found(path: &str, modified: Option<&str>) -> String {
    match modified {
        Some(date) => format!("pdfcer will read {path}, last updated on {date}."),
        None => format!("pdfcer will read {path}. Its date could not be read."),
    }
}

/// What pdfcer currently resolves, when nothing was found.
#[must_use]
pub fn resolved_none(looked_in: usize) -> String {
    format!(
        "No trust list was found on this machine. pdfcer looked in \
         {looked_in} place(s). If you have Acrobat or Reader, open it once and \
         let it update its trusted certificates, or type the file's location \
         above."
    )
}

/// What pdfcer currently resolves, when the operator's own path is wrong.
#[must_use]
pub fn resolved_configured_missing(path: &str) -> String {
    format!("There is no file at {path}, so no certificates will be read.")
}

/// The button that reads the store and reports what is in it.
#[must_use]
pub const fn inspect_button() -> &'static str {
    "Show what is in it"
}

/// The inspect button's tooltip.
#[must_use]
pub const fn inspect_hover() -> &'static str {
    "Read the file now and report how many trusted certificates it holds and \
     when it was last updated. Nothing is copied and nothing is changed."
}

/// The inspect button's failure.
#[must_use]
pub fn inspect_failed(reason: &str) -> String {
    format!("That file could not be read — {reason}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`NotChecked` says "not checked", in every one of its four
    /// explanations.**
    #[test]
    fn every_unchecked_trust_sentence_says_not_checked() {
        let explanations = [
            not_checked_opted_out().to_owned(),
            not_checked_no_store(4),
            not_checked_configured_missing(r"D:\nope\addressbook.acrodata"),
            not_checked_unreadable(r"D:\a\addressbook.acrodata", "bad header"),
        ];
        for why in explanations {
            let line = not_checked(&why);
            assert!(
                line.starts_with("not checked"),
                "an unchecked-trust line must SAY it was not checked: {line}"
            );
        }
    }

    /// **The four explanations are four different sentences.**
    #[test]
    fn the_four_reasons_trust_was_not_checked_are_four_sentences() {
        let all = [
            not_checked_opted_out().to_owned(),
            not_checked_no_store(4),
            not_checked_configured_missing(r"D:\nope\addressbook.acrodata"),
            not_checked_unreadable(r"D:\a\addressbook.acrodata", "bad header"),
        ];
        for (i, a) in all.iter().enumerate() {
            for b in all.iter().skip(i + 1) {
                assert_ne!(a, b, "two of the four situations share one sentence");
            }
        }
        // And each names the thing that distinguishes it, so the difference is
        // not merely a comma.
        assert!(all[0].contains("turned off"), "{}", all[0]);
        assert!(all[1].contains("found none on this machine"), "{}", all[1]);
        assert!(all[2].contains("no file there"), "{}", all[2]);
        assert!(all[3].contains("could not read it"), "{}", all[3]);
    }

    /// **A `Trusted` verdict says it is not a revocation verdict.**
    #[test]
    fn a_trusted_verdict_never_claims_more_than_the_engine_checked() {
        let with_clock = trusted("CN=Some CA", &["AATL".to_owned()], true);
        assert!(
            with_clock.contains("does not say whether a certificate was revoked"),
            "{with_clock}"
        );
        assert!(with_clock.contains("AATL"), "{with_clock}");
        assert!(with_clock.contains("CN=Some CA"), "{with_clock}");

        // And with no signing-time clock, the sentence must say the dates were
        // NOT checked — the engine's `validity_checked == false`, which is a
        // second thing a `Trusted` does not prove.
        let no_clock = trusted("CN=Some CA", &[], false);
        assert!(no_clock.contains("could NOT check"), "{no_clock}");
        assert!(
            no_clock.contains("does not say whether a certificate was revoked"),
            "{no_clock}"
        );
    }

    /// **"Untrusted" is not allowed to sound like "tampered with".**
    #[test]
    fn untrusted_separates_itself_from_integrity() {
        let line = untrusted("the chain is incomplete");
        assert!(
            line.contains("not about whether the bytes are intact"),
            "{line}"
        );
    }

    /// **The store's count and its date are one sentence.**
    #[test]
    fn the_store_is_never_described_without_its_age() {
        let counts = SourceCounts {
            aatl: 211,
            eutl: 1576,
            adbe: 2,
            other: 0,
            total: 1789,
        };
        let dated = store_line(r"D:\a\addressbook.acrodata", Some("2024-05-27"), &counts);
        assert!(dated.contains("1789"), "{dated}");
        assert!(dated.contains("2024-05-27"), "{dated}");

        let undated = store_line(r"D:\a\addressbook.acrodata", None, &counts);
        assert!(
            undated.contains("cannot tell you how current it is"),
            "a store with no readable date must say its age is unknown: {undated}"
        );
    }

    /// **The three facts have three distinct labels.**
    #[test]
    fn the_three_facts_are_labelled_apart() {
        let labels = [integrity_label(), coverage_label(), trust_label()];
        for (i, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        assert!(
            panel_intro().contains("never merges them"),
            "{}",
            panel_intro()
        );
    }

    /// **Each kind of location is named, absent kinds are left out, and the
    /// sentence says nothing was fetched.**
    #[test]
    fn revocation_sources_names_each_kind_it_was_given() {
        let url = |u: &str| vec![u.to_owned()];
        let all = revocation_sources(
            "CN=Signer",
            &url("http://crl.example.test/ca.crl"),
            &url("http://ocsp.example.test"),
            &url("http://ca.example.test/ca.crt"),
            2,
        );
        for piece in [
            "CN=Signer: ",
            "CRL http://crl.example.test/ca.crl",
            "OCSP http://ocsp.example.test",
            "issuer certificate http://ca.example.test/ca.crt",
            "2 more this build cannot read",
            "Not fetched",
        ] {
            assert!(all.contains(piece), "{piece:?} missing from {all:?}");
        }
        let ocsp_only = revocation_sources("CN=S", &[], &url("http://o.test"), &[], 0);
        assert!(
            !ocsp_only.contains("CRL")
                && !ocsp_only.contains("issuer")
                && !ocsp_only.contains("more")
        );
    }

    /// The revocation verdict names what it read and never overstates: a
    /// good verdict dates itself to the lists, a revoked one orders the
    /// revocation against the claimed signing time, and an unknown reason
    /// code passes through verbatim.
    #[test]
    fn revocation_verdict_says_what_the_lists_said() {
        let good = revocation_good(1, true, revocation_evidence(true, false), None);
        assert!(good.starts_with("not revoked. The signer's certificate is"));
        assert!(good.contains("revocation lists carried in the document"));
        assert!(good.contains("when that evidence was issued"));
        let many = revocation_good(
            3,
            false,
            revocation_evidence(true, true),
            Some("2026-09-30"),
        );
        assert!(many.contains("All 3 certificates are shown good"));
        assert!(
            many.contains("revocation lists and OCSP responses you supplied, as of 2026-09-30.")
        );
        assert_eq!(revocation_evidence(false, true), "OCSP responses");

        let after = revocation_revoked(
            "CN=S",
            Some("2026-09-30T07:31:13Z"),
            Some("keyCompromise"),
            Some(false),
            false,
            true,
        );
        for piece in [
            "REVOKED — CN=S was revoked on 2026-09-30T07:31:13Z",
            "(its key was compromised)",
            "per a revocation list carried in the document",
            "after the time this signature claims",
        ] {
            assert!(after.contains(piece), "{piece:?} missing from {after:?}");
        }
        let before = revocation_revoked("CN=S", None, Some("madeUp"), Some(true), true, false);
        assert!(before.contains("revoked (madeUp), per an OCSP response you supplied"));
        assert!(before.contains("made with a revoked certificate"));
        assert!(revocation_revoked("CN=S", None, None, None, false, true).contains("cannot tell"));

        assert_eq!(
            revocation_undetermined("no issuer."),
            "could not be decided — no issuer."
        );
        assert!(revocation_not_checked().contains("does not go online"));
        assert!(revocation_open_ended().contains("newer information may exist"));
    }
}
