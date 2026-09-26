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
