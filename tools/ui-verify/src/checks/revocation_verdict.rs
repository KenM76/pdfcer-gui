//! `signature_says_its_revocation_verdict` — **the Signatures panel reports a
//! signer the document's own revocation list names as revoked**, with no list
//! supplied by hand: `fixtures/signed-revoked-dss.pdf` carries its issuer's CRL
//! in `/DSS`, revoking the signer after the claimed signing time.
//!
//! The control is `signed-two-pages.pdf`, which carries no revocation list: a
//! panel that reported *revoked* or *good* without evidence fails there.

use crate::checks::revocation_sources::signature_line_matches;
use crate::checks::{Check, CheckContext};
use crate::report::CheckReport;

const EVENT: &str = "signature-revocation";
/// Revoked by the CRL in the document, after the claimed signing time.
const REVOKED: (&str, &str) = (
    "signed-revoked-dss.pdf",
    "verdict=revoked before_signing=false kind=crl source=dss",
);
/// No revocation list anywhere: nothing checked.
const UNCHECKED: (&str, &str) = (
    "signed-two-pages.pdf",
    "verdict=not-checked before_signing=none kind=none source=none",
);

/// See the module documentation.
pub struct SignatureSaysItsRevocationVerdict;

impl Check for SignatureSaysItsRevocationVerdict {
    fn name(&self) -> &'static str {
        "signature_says_its_revocation_verdict"
    }

    fn defect(&self) -> &'static str {
        "the Signatures panel shows a signature whose signer the document's own revocation \
         list names as revoked without saying so, or claims a revocation verdict for a \
         document that carries no revocation list"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut outcome = Ok(None);
        for (fixture, want) in [REVOKED, UNCHECKED] {
            outcome = signature_line_matches(ctx, &mut report, fixture, EVENT, want);
            if !matches!(outcome, Ok(None)) {
                break;
            }
        }
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}
