# `ui-verify/profile`

<!-- old-name-exempt-file: notes on the falsification profile for the pre-rename GUI, which must spell that build's names. -->

## Item notes

### `const SETTINGS_HEADINGS_LEGACY`

# Provenance, stated in full because a calibrated number is only as good as
its provenance


The calibration is [`Calibration::Image`], so these fractions can only be
used against that image. They are **not** valid against a live window of
the old GUI: the crop is a sub-rectangle of the dialog, not the client
area, so the same fractions would sample the wrong part of a live capture.
Driving the live old binary to its Settings dialog needs its own
calibration pass, and until somebody does that pass the live-mode check
SKIPs saying so — which is the honest report, and is not the same as a pass.

### `fn legacy_profile_names_the_pre_rename_gui`

# Why this test exists


* the exe path came to name a binary in the ENGINE repository, whose
  `Pass 247.0` had just deleted the only GUI crate it ever had --
  a path that can never exist;
* the diagnostic environment variable came to name one the old binary
  does not read, which leaves its tracing OFF;
* the trace prefix came to name one the old binary never prints, which
  parses to an EMPTY trace.

Each of the last two is silent. An empty trace and a build that said
nothing are the same bytes, so the falsification suite would have
reported "the old build does not exhibit the defect" -- the exact
inversion the suite exists to prevent -- with every gate green.

This asserts the shape rather than the spelling: the four fields must
carry the old stem and must NOT carry the new one. It is deliberately
a test and not a comment, because a comment is what was there.

### `fn pdfcer_gui`

Every name here was read out of `crates/pdfcer-gui/src`, not guessed:

* `canvas rect= zoom= page= pages= off=` — `canvas/mod.rs`, traced
  through the de-duplicating gate so there is one line per document
  open and one more per layout change (`PROJECT_PLAN.md` §4.3
  requirement 1, landed at S2 — which is why the layout-probe click in
  [`crate::checks::delete_key`] is no longer needed against this
  binary, only against the old one).
* `ui-rect name= rect=` — `diag.rs::ui_rect` (§4.3 requirement 2).
* `objects n= page= paths= text= images= forms=` —
  `pdfcer-gui-base/src/opendoc/objectcount.rs::trace_object_count` (§4.3 requirement 3).
* `start` — emitted unconditionally, so an empty trace can be told
  from a trace the diagnostic switch never reached.

## Two fields that name events this binary does not emit yet

`click_event` and `delete_event` keep the old binary's spellings. That
is not an oversight and it is not a claim that this binary emits them:
a vocabulary entry is the name a check will *look for*, and looking for
a name that is absent is how a check discovers a subsystem has not been
built and reports SKIP. Nothing is gained by blanking them — a `None`
there would produce the same SKIP with a vaguer reason.

Likewise `canvas_selection_field`. This binary deliberately does **not**
emit `sel=` on its `canvas` line, because there is no selection
subsystem at S2 and `sel=0` would be a false statement about the
document that turns an honest SKIP into a FAIL blaming code nobody has
written. [`crate::checks::delete_key`] handles the absence explicitly;
see the `(None, None)` arm there.

### `fn pdfcer_legacy`

* `canvas … rect= zoom= sel=` — traced only on pointer events
* `vector-click … hits= newsel=`
* `delete-objects n=`
* `start` — emitted unconditionally

`object_count_event` and `ui_rect_event` are `None` because this binary
has neither, and **that is load-bearing rather than incidental**. It is
what keeps the D1 reproduction honest: with no object count to read,
[`crate::checks::delete_key`] falls back to the weaker
absence-of-`delete-objects` oracle, which is the oracle that produced
the recorded FAIL. If this profile were given the new binary's
vocabulary, the check would look for a count that is never emitted and
the fallback would still fire — but the *reason string* would then name
an event this binary cannot produce, and a skip or failure reason that
misidentifies the blocked component sends the reader to the wrong file.

### `fn object_count`

`None` covers three situations that the caller must **not** collapse
into "zero objects":

1. this binary has no object-count event in its vocabulary;
2. it has one and has not emitted it yet;
3. it emitted `objects-unavailable page=… reason=…` instead, because
   the page's content streams would not decode.

The third is why the application's contract is that failure is a
*different event* rather than the success event with a missing field:
an `objects` line is a claim that the count was measured, so a check
comparing before against after can trust it. A missing `n=` on an
`objects` line is therefore a harness-side parse bug, and reading it as
zero would turn a parse bug into "the page is empty" — a confident,
wrong statement about the document.

### `const PDFCER_GUI`

It exists as of S2 and it speaks all three of the dialects
`PROJECT_PLAN.md` §4.3 asked it for: an unconditional `canvas` line, a
`ui-rect` line per named region, and an `objects` count. What it does
**not** have yet is a ribbon, a selection subsystem or a Settings dialog —
so the checks that need those still report SKIPPED, and the reason each
gives now names the missing *subsystem* rather than the missing trace
channel. That difference matters: a reason that blamed the trace channel
would send a reader to `diag.rs`, which is finished.

### `const PDFCER_LEGACY`

**Read-only, always.** The harness launches it and photographs it; nothing
in this crate writes anywhere near it.

Its purpose here is falsification. A check suite is only evidence if it has
been seen to fail on a known-defective build, and this profile is how that
is demonstrated.

# EVERY NAME IN THIS PROFILE IS AN OLD NAME, DELIBERATELY


| field | swept to | actually |
|---|---|---|
| `default_exe` | `\pdfcer\…\pdfcer-gui.exe` | `\pdfce\…\pdfce-gui.exe` |
| `diag_env` | `PDFCER_DIAG` | `PDFCE_DIAG` |
| `trace_prefix` | `pdfcer-diag` | `pdfce-diag` |
| `viewport_env` | `PDFCER_DIAG_VIEWPORT` | `PDFCE_DIAG_VIEWPORT` |

The swept `default_exe` is worse than merely wrong: the engine's
`Pass 247.0` **stripped the in-repo GUI crate** from the new repository, so
that path can never exist — the falsification profile was pointing at a
binary nothing will ever build. The three trace names would each have failed
*quietly*, which is worse still: an environment variable the old binary does
not read simply leaves diagnostics off, and a trace prefix that does not
match reads an EMPTY trace — indistinguishable from a build that emitted
nothing.

Guarded by `legacy_profile_names_the_pre_rename_gui`, which is the
mechanism; this comment is only the reason.
