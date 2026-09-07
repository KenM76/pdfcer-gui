---
name: a-tripwire-keyed-on-your-own-intention-is-not-a-tripwire
description: A `const SHOULD_DELETE: bool = false` + debug_assert cannot fire; key the tripwire on the OTHER side's source, located through Cargo.lock so it follows the pin.
metadata:
  type: feedback
---

**A tripwire has to be keyed on the OTHER side's API, not on this side's
intention. `const ENGINE_HAS_TAKEN_OVER: bool = false` with a `debug_assert`
beside it cannot fire — nothing can set it but somebody who has already
noticed.**

**Why:** 2026-09-07, writing `canvas::annotquad` as a declared workaround for
`Annotation` carrying no rotation. The first tripwire was that constant. Clippy
flagged it — *"this assertion has a constant value"* — for a worse reason than
it knew: the lint is about the assert being degenerate; the real defect is that
the mechanism has no input.

What replaced it: a `#[test]` that reads `pub struct Annotation` out of the
**pinned engine checkout** and fails when it grows a `rotation`,
`appearance_matrix` or `matrix` field. Falsified both ways before it was
trusted.

**How to apply:**

- **Locate the source through `Cargo.lock`**, not through a hard-coded path.
  `~/.cargo/git/checkouts/<pkg>-<hash>/<rev-prefix>/…` are the exact bytes
  `rustc` read, and matching the rev by *prefix* against the lock's hash means
  the instrument follows the pin automatically. Do **not** read
  `D:/Dev/pdfcer` — the engine session's working tree moves several times a
  day, often ahead of what compiles here, so a tripwire on it fires on work
  that is not in the binary.
- **FAIL, do not SKIP, when the source cannot be found.** The crate could not
  have compiled without that checkout, so a red there means the locating is
  broken. A hard-coded external path turning a rename into a green check over
  an empty scan has already cost this project once.

Related: [[a-completeness-question-needs-an-instrument-not-a-document]],
[[a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion]],
[[a-check-that-cannot-fail-is-not-evidence]].
