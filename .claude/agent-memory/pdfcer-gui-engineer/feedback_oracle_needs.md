---
name: an-oracle-built-from-the-system-under-test-needs-an-independent-calibration
description: a round-trip check (write with pdfcer, read with pdfcer) is only non-circular if the READER was first calibrated against an artifact pdfcer had no hand in producing
metadata:
  type: feedback
---

**A round-trip check — write it with pdfcer, reopen it with pdfcer, assert the
report — is circular unless the READER half was independently calibrated
first.** Build the fixture from the spec, by a script with **no pdfcer
involvement**, before you trust pdfcer's reader as an oracle for pdfcer's
writer.

**Why:** 2026-09-10, the O169 stamp-collection work.
`fixtures/stamp-collection.pdf` was written byte by byte from §7.9.6 and §12.5
by a Python script. *That* is what made
`stamp_collection_reaches_the_engine` — which writes a collection, relaunches
the binary on the file it wrote, and requires Document properties to disclose
four stamps — a real measurement rather than pdfcer agreeing with itself.

It was falsified against a planted defect that produced a **valid PDF and a
truthful-looking receipt** (`stamps=4 pages=4 skipped=0`). Every assertion up
to and including the disk read passed. **Only the round trip caught it.**

**How to apply:** for any author/reader pair, the order is (1) hand-build a
fixture from the standard, (2) drive the reader against it, (3) only then use
the reader as the writer's oracle. And when a check passes every intermediate
step and fails only at the far end, that is not a flaky check — that is the
falsification working. Related:
[[a-check-that-cannot-fail-is-not-evidence]],
[[unit-tests-that-call-the-verb-cannot-see-the-chain-in-front-of-it]],
[[a-driven-failure-is-a-claim-about-the-check-too]].

★ Mechanical note from the same work: engine types marked `#[non_exhaustive]`
**cannot be struct-literal-constructed** from this crate, so a falsification
plant cannot fabricate one. Fake the **consumer's** value instead — build the
type with `::default()` and mutate the public fields, which is what the five
`annotations_not_drawn` tests do.
