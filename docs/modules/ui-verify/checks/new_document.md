# `ui-verify/checks/new_document`

`new_document_makes_a_page` — File ▸ New makes a document, and the page
actually draws.

# What this is for

`file.new` is the first command in this shell that produces a document out
of **compiled-in bytes** rather than out of a file the operator named. Every
link in that chain has a unit test — the template parses
(`app::blank::tests`), the command registers, the arm raises `Action::New`,
`new_document` replaces the status — and **not one of them can observe the
chain being performed**. That is the same gap `markup_rectangle` was written
for: every link green, the join undriven.

It is also the check for a failure mode this project has hit twice and which
a unit test structurally cannot see: **a document that is open and will not
draw.** A blank page is the one document where "open" and "renders" are most
easily confused, because a blank page and a page that failed to rasterize
produce the same screenshot — which is why this check reads the canvas's own
`drawn=` count rather than looking at pixels.

**A screenshot cannot distinguish "drew nothing" from "drew something
featureless".** 2,450 hairlines and a flat wash are the same picture at a
glance, and so are a blank sheet and a sheet that did not render. Wherever
that ambiguity exists, the oracle has to be a count the application
publishes, not a pixel.

# What it asserts, in order, and why each step is separate

| # | Assertion | The failure it separates out |
|---|---|---|
| 0 | a document with **more than one page** is open first | without it, "New produced one page" is satisfied by New doing nothing to a one-page fixture |
| 1 | `ribbon.item.file.new` is declared | the command is registered and on no tab — a real state, and `super::manifest::CUSTOM_BACKED` exists because of it |
| 2 | the click produced a **new** `ribbon-command-invoked id=file.new` | the click missed, the control is disabled, or no pointer reached the window. **SKIP, never FAIL** — see below |
| 3 | a `new-document` line appeared | the command was invoked and the dispatch arm is missing. Distinguished from step 2 precisely because `command-unimplemented` is the shape this project keeps finding |
| 4 | `open ok pages=1 path="Untitled 1.pdf"` | `new_document` ran and the template did not become a document — a corrupt asset, or a `Status::Failed` |
| 5 | the canvas drew it: a `canvas` line with `pages=1 drawn=1` | the document is open and the page will not rasterize. **This is the "confirm it renders" step** |
| 6 | a second New makes `Untitled 2.pdf` | New is idempotent — it produced a document once and the control now does nothing, which steps 3–5 would all still pass |

# The falsifying phase: what wrong implementation would pass this?

Asked before the check was written, per `checks/mod.rs`'s rule that a check
must fail against a build where the wiring is absent. Four answers, and the
first three are why the steps are shaped the way they are:

1. **A `file.new` with no dispatch arm.** `ribbon-command-invoked` still
   appears — the *shell* publishes it, before the application sees the token
   — so a check that stopped at step 2 would pass against a build in which
   pressing New does literally nothing and traces
   `command-unimplemented id=file.new`. Step 3 is what fails there, and the
   failure message says which of the two it is.
2. **An `Action::New` matched *after* the document guard** in
   `app::actions::apply`, so it is silently dropped whenever nothing is
   open. Steps 3 and 4 fail. This one is not hypothetical: the guard exists,
   `Action::Open` and `Action::Close` are matched before it *for this
   reason*, and a fourth variant added below it would compile.
3. **A New that opens a document nothing can draw.** Steps 3 and 4 pass —
   the status really is `Open` with one page — and the operator sees an
   empty canvas. Step 5 is the only assertion in the workspace that fails
   there, because no unit test in this project rasterizes.
4. **A New that works once.** Every step above passes on the first press. It
   is step 6 that fails, and the reason it is worth a second click is that
   the ordinal is *application* state (`PdfcerApp::created_documents`) while
   everything else is per-document — a distinction whose whole failure mode
   is invisible until the second press.

## The falsification this check is held to

`PROJECT_PLAN.md` §4 stage S1 requires every check here to have been run
against a deliberately broken build and seen to fail — not reasoned about.
For this one the break is deleting `"file.new" => actions.push(Action::New)`
from `app::dispatch`.

The **shape of the failure it must produce** is the contract, and it is
what the step separation above buys:

> press 1: `file.new` was invoked and traced no new `new-document
> name="Untitled 1.pdf"`. The application traced
> `command-unimplemented id=file.new`, so the token arrived at
> `app::dispatch` and there is no arm for it — the fix is one match arm,
> not a wiring hunt. Documents it did report creating this run: none.

Two properties of that message matter. It fails at **step 3 and not step
2** — the click did reach the control and the check says so — and it names
the *right file*. A check that stopped at step 2 would pass against that
binary; a check whose failure said only "New is broken" would send a reader
through the manifest, the registry and the ribbon before reaching the one
missing line.

And what this check would **not** catch, stated so nobody reads it as
covering more than it does:

* **the page being the wrong size.** A Letter template would pass every
  assertion here. `app::blank::tests::the_template_page_is_a4` is what pins
  that, and it is a unit test because a page size is a *number in a file*
  and this harness's job is the things a number in a file cannot be.
* **the page being blank.** Nothing here reads a pixel. A template with a
  watermark on it would pass. `app::blank::tests` and the 443-byte size
  assertion are the guard against that, and
  `crates/pdfcer-gui/src/app/assets/PROVENANCE.md` is the reason it matters.
* **anything about saving it.** There is no save in this build, for any
  document — see `app::blank` §5.

# Why step 2's failure is a SKIP

`driving`'s own rule, and `find_bar`'s recorded incident: a check that could
not deliver a click has learned nothing about the application, and naming a
feature as the culprit when nothing was ever clicked at it is worse than no
check at all. That is why the invoke count is taken **before and after** the
click rather than asked as "is there a line for `file.new`?" — the second
press would otherwise be satisfied by the first press's line.

# It uses the mouse only

`Ctrl+N` is bound and is deliberately **not** what this check drives. Two
reasons, and the second is the operative one: the chord and the control
reach the identical dispatch arm, so driving both would test one thing
twice; and chord delivery is the part of this harness that has already
produced a false negative once (`find_bar`'s first run reported Find broken
on a build where Find worked). A check about a new capability should not
also be a bet on the least reliable channel. Whether `Ctrl+N` is *bound* is
pinned by `app::keyboard`'s
`every_chord_the_manifest_binds_can_be_spelled` family; whether it is
*delivered* is `find_bar`'s subject, not this one's.

## Item notes

### `const FIRST_NAME`

Spelled out rather than derived, because the derivation lives in
`crate::text::files::untitled` inside the application and a harness that
recomputed it would agree with a wrong implementation.

### `fn press_new`

`ordinal` is which press this is, and it is what the expected document name
is derived from — so the second press asserting `Untitled 2.pdf` is the same
code path as the first asserting `Untitled 1.pdf`, rather than a
copy-and-pasted variant that could be weakened independently.

Returns `Ok(None)` for a clean press, `Ok(Some(_))` for a failure sentence,
and `Err` only for the states that are the *harness's* business — no click
delivered, no trace readable.

### `fn the_expected_names_survive_the_trace_quoting_them`

`PathBuf` is traced through `{:?}`, so the trace reads
`name="Untitled 1.pdf"` — quoted. **`TraceLine::get` strips a value's
surrounding quotes**, so these constants are written BARE, and this test
is what keeps the two in step: it parses a real quoted trace line and
asserts the bare constant matches it.

The quoting exists because a value can contain structural characters —
a chord spelled `[` in an unquoted `chord=[` opens a bracket the field
splitter never sees closed, and swallows every field after it on the
line. The application quotes such values and `get` unwraps them, so no
caller has to know which values those are.

⚠ Getting this backwards makes the check report New as broken on a build
where it works — the exact false negative it exists to avoid.

### `fn the_canvas_line_carries_whether_anything_was_rastered`

The distinction step 5 rests on, pinned against literal trace text so
that a change to the canvas line's shape fails here rather than turning
step 5 into an assertion that always reads `drawn=0`… which, being a
FAIL, would at least be loud. The opposite — a parse that always yields
a non-zero `drawn` — is the quiet one, so both are asserted.
