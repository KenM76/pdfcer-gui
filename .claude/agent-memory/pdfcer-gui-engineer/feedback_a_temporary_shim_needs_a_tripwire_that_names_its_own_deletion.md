---
name: a-temporary-shim-needs-a-tripwire-that-names-its-own-deletion
description: Ship a shell-side workaround with a debug_assert that fires when the engine lands the real fix — it worked twice within hours, where a comment would have rotted
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
