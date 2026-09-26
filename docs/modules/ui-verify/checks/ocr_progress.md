# `ui-verify/checks/ocr_progress`

**The three checks about a recognition run while it is still running** —
progress, Stop, and Cancel.

# Why this file exists at all, when `checks::ocr` already drives OCR

`checks::ocr` drives a **one-page** run and asserts its *result*: words came
back, the session took them, nothing was written to disk. Every one of those
is a statement about a run that has finished, and a one-page run has no
observable middle — it is started and then it is over.


> *"can you make it so the recognizing ocr gives feedback on what it is
> doing when it is running (pages done, words/characters detected, etc) so
> that the user can see that it is doing something and hasn't frozen on
> large documents? Maybe a cancel and stop button too. The cancel throws
> away what was done, and the stop finished the page it is on and keeps the
> work it has done."*

Three separable claims, and **none of them can be observed on one page**:

| claim | what a one-page run shows |
|---|---|
| the tally *advances* | one value, or none — a frozen label is identical |
| Stop keeps what was done | a Stop can only race the single page |
| Cancel discards what was done | there is nothing partial to discard |

So these checks drive `fixtures/synthetic-image-only-8pages.pdf`, which
exists for exactly this and whose header
(`crates/pdfcer-gui-base/src/ocr/fixture.rs`) argues the eight.

# What was already true, and why it was not enough


**They call the verb. They cannot see the chain in front of it.** Between
`Control` and the operator there is a dialog that must *draw* the tally, a
frame loop that must keep *waking up* to redraw it, two buttons that must be
hit-testable, and a poll that must route three worker outcomes to three
different phases. Every one of those is off the verb, and eight green unit
tests say nothing about any of them. `OPERATOR_REQUESTS.md` O93 is marked
**not shipped** on precisely that reading of R1, and this file is what
closes it.

# The one that would have been missed, and it is not the buttons

**A rect is not an oracle for "the user can see it is doing something".**

`crate::diag::ui_rect("ocr-progress", …)` says a label was drawn somewhere.
A build whose progress label read `Page 1 of 8` for the entire run would
declare a perfectly substantial rect on every frame — and would be the exact
frozen application the request is about. So the shell now traces the
**numbers** as well (`ocr-progress attempted=… of=… words=… chars=…`, on
change), and [`OcrSaysHowFarItHasGot`] asserts that two of them differ.

Writing that check is also what found the defect underneath it. egui is
immediate-mode and idle: the OCR worker is on another thread and generates
no input events, so **nothing would have requested the next frame** and the
window would have held the frame it drew when the run started. It worked
anyway — because `egui::Spinner` calls `request_repaint()` for its own
animation (`egui-0.35.0/src/widgets/spinner.rs:40`). The entire visibility of
this feature rested on a side effect of a decorative widget. `dialogs::ocr`
now asks for the repaint explicitly and says why; anyone who later swaps the
spinner for a progress bar will not silently take live progress with it.

# Which document these drive

The eight-page synthetic fixture, always, by default — for the same reason
`checks::ocr` pins its own: a run needs pages with **no text on them**, and
on any document that already has text the doubling guard skips everything
and the honest answer to *"how far did it get"* is *"it declined to look"*.
A suite-wide `--pdf` is a drawing; a drawing is the wrong subject here.

**`PDFCER_VERIFY_SCAN` overrides it, and should be used.** Point it at a
genuinely scanned, multi-page, text-free PDF and these checks run against
real material — scanner noise, skew, JPEG ringing, rotated pages, the lot.
The synthetic fixture is a *rendered* page and flatters the recogniser; it
establishes the plumbing and nothing about recognition quality, and every
report from these checks says so rather than leaving a green result to imply
otherwise.

The operator's own material for this is the eight pages extracted from
`Parts Manual TH83 Telehandler.pdf` — an 883-page scanned parts manual, all
images, `/Rotate 270`, measured at **2.6 s and 440 recognised words per
page** through `pdfcer ocr`. See `OPERATOR_REQUESTS.md` O93.

# Every way these report SKIP, and why none of them is a pass

* no binary, no `--no-input`, no diagnostic channel — the harness never
  began;
* the fixture is missing;
* a tab, a mode segment or a control was never declared, or took no click;
* the model weights are not beside the binary — the application says so by
  name, and blaming the feature for a packaging step that was not run would
  be a false failure;
* **the run finished before the harness could reach the button.** Stop and
  Cancel are checks about interrupting something, and something that is
  already over cannot be interrupted. That is a property of the machine's
  speed on the day, not a defect, and it is reported as a SKIP that names
  the timing rather than as a pass that hides it.

## Item notes

### `const FIXTURE_PAGES`

Used **only** to sanity-check the `of=` the application reports when the
default fixture is driven. Everything else derives the denominator from the
trace, so `PDFCER_VERIFY_SCAN` can point at a document of any length.

### `const SCAN_ENV`

An environment variable rather than a flag, deliberately: it is a property
of the *machine* — whether this box happens to have a scan on it — and not
of the run. A flag would have to be passed by every caller of the suite,
including the ones that have no such file, and would then be forgotten.

### `const RUN_FRAMES`

A frame here is 25 ms (`Session::settle`), so 1,600 frames is **40
seconds**. Measured inputs: the synthetic page recognises in about a second
in a release build, and the operator's scanned parts manual measured 2.6 s a
page — so eight pages is 8–21 s and this is roughly twice the worst of
those.

Generous on purpose, and the reasoning is `checks::ocr`'s: a budget that
was too short would report *"recognition never finished"* about a build that
was still working, which is the worst available failure message. This
harness also drives whichever binary it was pointed at, and a debug build is
twenty times slower.

### `const FIRST_PAGE_FRAMES`

One page's worth plus a wide margin. If no page has finished in twelve
seconds the run is either refused, stuck, or being driven in a debug build,
and all three want a different message from "the tally did not advance".

### `const SLICE`

Small enough that a Stop lands with pages still to go on a one-second-a-page
run; large enough that the loop is not re-reading a growing trace file forty
times a second.

### `fn workspace_root`

`tools/ui-verify/` → up two. Stable whatever the harness was invoked from
and whatever `--source-root` says — see `checks::ocr`'s `default_fixture`
for the two wrong ways this was done first, one of which overwrote the
repository's own fixture.

### `fn document`

Returns the path and whether it is the real-material one, because every
report says which it drove. A green result over the synthetic fixture and a
green result over a scanned manual are different amounts of evidence and
must not read the same.

### `fn wait_until`

Returns whether it held. **Checks before it sleeps**, so a condition that is
already true costs nothing — which matters for the Stop and Cancel checks,
where the whole question is whether the harness got there in time.

### `fn start_a_run`

Everything up to the moment work starts is identical in all three checks,
and duplicating it three times would be three places for the ribbon path to
rot. `Err` is a SKIP in every caller.

Returns the session, the driver, and the resolved document with its
real-material flag.

### `fn the_fixture_is_multi_page_and_image_only`

Pinned because both properties are load-bearing and each fails silently
on its own: a single-page fixture makes all three checks vacuous, and a
fixture with text makes the doubling guard skip every page so the run
reports nothing and the tally never moves — which reads as the very
defect being looked for.

### `fn the_two_endings_reach_two_different_controls`

The one assertion in this file that could not be got wrong by accident
and would invalidate everything if it were: the pair of checks is a pair
precisely because the two buttons must not be the same button, and a
harness that clicked one region for both would report that they behave
identically — correctly, and about itself.

### `fn the_default_document_is_the_committed_fixture`

Deliberately does not test the override branch: setting a process-wide
environment variable from a test races every other test in the binary,
and this crate runs its tests in threads. The branch is three lines and
its risk is a typo in the variable name, which this pins instead.
