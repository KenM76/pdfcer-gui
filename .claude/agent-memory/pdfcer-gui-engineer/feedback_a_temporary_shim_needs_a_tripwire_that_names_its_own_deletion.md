---
name: a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion
description: Ship a shell-side workaround with a debug_assert that fires when the engine lands the real fix — and key it on the OTHER side's source read through Cargo.lock, never on your own intention, or it is not a tripwire
metadata:
  type: feedback
---

**A workaround for something the engine will fix must carry a runtime tripwire
whose message names the file and the call site to delete.** A comment saying
"remove this when X lands" does not fire.

**Why:** 2026-08-28. `app::settings::colour_default` forced
`CmykIntent::Calibrated` because O52 reversed an earlier operator ruling and
`pdfcer-core` still defaulted to `NeutralBlack`. It shipped with:

```rust
debug_assert_ne!(
    Settings::default().cmyk_intent, CmykIntent::Calibrated,
    "pdfcer-core's default is now Calibrated, so `colour_default` is a second \
     source of truth for a value the engine already gets right. Delete it and \
     its call site."
);
```

`Pass 153.0` landed **two hours later** and it fired on the first debug build
after `cargo update`. The shim and its call site went the same hour. The same
mechanism had already worked once, on `text_edit`'s deprecated arm.

**How to apply:**

- The tripwire asserts the **condition that makes the shim unnecessary**, not
  the shim's own behaviour. `debug_assert_ne!(Upstream::default(), what_we_force)`.
- Its message is an **instruction**, with the symbol name in it. Somebody
  reading a panic at 2 a.m. should not have to work out what to do.
- `debug_assert` rather than `assert`: a release build must not die because
  upstream improved.
- The same discipline for a **workaround around an engine defect**: report it
  (decision 058) *and* leave a tripwire, so the deletion is forced rather than
  remembered. See [[delete-the-workaround-when-the-cause-is-removed]] — the
  engine answers within hours, and this is how you find out.

★ Two hours is not unusual here. Assume any shim you write against a filed
request will need deleting **this session**.

## The failure mode — a tripwire keyed on your own intention is not one

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

Related: [[a-lesson-in-a-docstring-is-not-an-instrument]],
[[a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion]],
[[a-check-that-cannot-fail-is-not-evidence]].
