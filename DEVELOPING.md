# Developing pdfcer-gui

The engineering guide: how the tree is laid out, how to build and check it, and
the standards every source file and document in it follows.

| Looking for | Read |
|---|---|
| what pdfcer is, and a download | [`README.md`](README.md) |
| how to use it | [`MANUAL.md`](MANUAL.md) |
| what works today, measured | [`FEATURES.md`](FEATURES.md) |
| **a new session starts here** | [`RESUME.md`](RESUME.md) |
| the charter and the fold-in procedure | [`PROJECT_PLAN.md`](PROJECT_PLAN.md) |

---

## 1. What this workspace is

A replacement GUI for the `pdfcer` PDF engine, built as three crates:

- **`crates/pdfcer-gui`** — the application. Knows about PDF.
- **`crates/pdfcer-gui-base`** — the floor of that application: the modules
  that reference no other `pdfcer-gui` module. Admission test is mechanical —
  a fan-out of zero, measured by `python tools/module-graph.py` — and the
  point of the split is that cargo forbids a cycle between crates, so nothing
  in there can call back up into the application. It may name PDF concepts;
  that is what separates it from the shell below.
- **`crates/egui-shell`** — the reusable shell: ribbon, dock, modes, layout
  persistence, theme, command registry. Knows nothing about PDF, and a gate
  (`check-shell-purity.sh`) fails the build if it learns.

plus two small platform crates (`native-window`, `native-clipboard`) and the
verification harness in `tools/ui-verify`.

`pdfcer-gui` depends on `pdfcer-core`, `pdfcer-render` and `pdfcer-print` as
**git dependencies on the `main` branch** of `file:///D:/Dev/pdfcer`, and Rust
links them statically. The release binary therefore already carries the engine;
there is no integration step to perform before shipping.

A branch pin moves on its own. `Cargo.lock` records the resolved sha, so the
build is reproducible until someone runs `cargo update -p pdfcer-core`, but the
engine's **source** under `D:\Dev\pdfcer` changes under this repository at any
time, with no command run here. Two consequences bind everything below: run
`cargo update` for the three engine crates before a build that is meant to
carry the latest engine, and never cite the engine by line number (§5.1).

**`D:\Dev\pdfcer` is read-only from here.** Engine changes are written up and
handed to the operator, never applied. See `PROJECT_PLAN.md` §7.

## 2. Tree layout

```
crates/
  pdfcer-gui/src/
    app/          frame composition, state, command dispatch, actions, status bar
    canvas/       the page view: pick, move, zoom, tools, rulers, geometry
    dialogs/      modal and modeless windows
    panels/       the dockable panels
    text/         text extraction, editing, markup, forms, redaction, settings
    render/       the render worker, settle debounce, raster cache
    shell/        this app's manifest and command catalogue for egui-shell
    icons/        the icon set and its catalogue
  egui-shell/src/
    ribbon/       tab/group/item layout and the narrowing plan
    dock/         panel host over egui_tiles
    manifest/     the serializable ribbon/dock/mode/keymap description
    theme/        palette and egui style derivation
tools/
  gates/          the CI gates; run-all.sh runs every one
  ui-verify/      drives the release binary and asserts on pixels and traces
fixtures/         PDFs built by tools/gen-*.py, each with a PROVENANCE.md
evidence/         calibration inputs the harness reads; run output is untracked
mockups/          HTML mockups of the ribbon, the window and the modes
```

## 3. Build and check

```sh
cargo build --release
CARGO_BUILD_JOBS=2 cargo test --workspace
bash tools/gates/run-all.sh
```

**The release profile is incremental**, which cargo does not do by default —
because here the release build is also the development loop. A one-file edit
rebuilds in 28s rather than 60s, at the cost of 1.3% on the binary and no
change at all to the engine crates. The argument and the measurements are in
the workspace `Cargo.toml`; re-measure there before quoting either number.

**The job count on the test suite is not optional.** The workspace builds more
than twenty test binaries, the `pdfcer-gui` one alone linking tens of megabytes
of debug object. At cargo's default of one job per core, that many linkers run
at once, each holding well over a gigabyte, and the machine runs out of memory.
A run killed that way leaves orphaned `cargo`/`rustc`/`link` processes still
holding the memory, which must be killed before a retry. For the same reason,
run one cargo job at a time — never chain the gates, the tests and a build into
a single background command.

**`cargo test -p egui-shell` and `cargo test --workspace` do not compile the
same crate.** The workspace pins `egui` with `default-features = false`, and
`eframe`'s `default_fonts` arrives only through feature unification in a
workspace build. A layout test that measures text therefore has real glyph
metrics under one invocation and none under the other, so one assertion can be
meaningful in CI and vacuous when somebody re-runs it on the crate alone. Write
such a test to assert that a measurement **happened** — `Some(false)` rather
than `None` — and not only what it said. An assertion that also passes when the
measurement is absent is measuring the invocation, not the layout.

`run-all.sh` runs fmt, clippy and every gate in `tools/gates/`. It has three
states, not two: `0` pass, `1` fail, `3` a gate was **skipped** because its
precondition was absent. A skip is not a pass — CI must not go green on a gate
set that did not fully run.

Every gate that is a grep over source carries a `--self-test` that plants a
violation and asserts the gate catches it. The self-tests run before any gate
is trusted: a gate that cannot detect its own planted violation has no verdict
worth reading.

**A gate chooses the files it will examine, and choosing them with `git grep` or
`git ls-files` chooses the index instead of the working tree.** Those two sets
disagree about exactly one thing — the work just done and not yet staged — which
is the worst possible blind spot, because that is also the only moment anyone
runs the gates by hand. A session writes three files, runs the suite, sees
green, commits, and the same tree goes red on the next run with nothing edited
in between; all that changed was `git add`. **The tell is the timing, not the
content:** green before the commit, red after it, nothing edited. When you see
that, stop reading the gate's output and look at how it chose its files. Every
`git grep` / `git ls-files` under `tools/` therefore either reads the working
tree — `--untracked`, or a plain filesystem walk — or carries a written reason
for not doing so, and `check-gate-input-scope.py` enforces it. It is a gate
rather than a paragraph because the paragraph did not work: the generalisation
was already written into a docstring in this tree and the mechanism recurred in
the same directory regardless. Prose does not sweep.

### Driving the binary

A passing test is not evidence that a UI works. `tools/ui-verify` launches the
release binary, opens a fixture, drives a scripted sequence through the OS, and
asserts on what the running program published about itself.

**It has three oracles, and none of them reads rendered text:** the
`PDFCER_DIAG` trace lines, the rectangles the application publishes for its own
widgets, and the captured pixels. There is no AccessKit, no OCR and no text
extraction in the harness, so a check can prove a surface was laid out and
cannot prove what it says. The Objects panel is the worked example: it
publishes one aggregate line, `objects-panel page=… objects=… rows=…`, and
`rows=` is a **layout** count. Asserting on it proves the panel produced rows.
It proves nothing about whether any row is true, legible or unclipped. A check
that needs content has to get the application to publish that content as a
trace field first.

```sh
cargo run --release -q -p ui-verify -- \
    --exe target/release/pdfcer-gui.exe \
    --pdf D:/Dev/pdfTests/SW41177/SW41177.pdf \
    --doc-point 0,300,500
cargo run --release -q -p ui-verify -- --list      # the registered checks
```

**`--doc-point` is a calibration, and there are two of them.** `0,300,500`
is the default and lands on plain geometry. `0,1211,1021` aims at a BOM row,
which makes it the right point for the text checks and the wrong one for
`rotate_handle_turns_a_selection` — that check passes at one point and fails
at the other, and one point passing is exactly what hides a grip whose centre
leaves the canvas. Any check sensitive to where the pointer lands is run at
both before its verdict is believed.

The full sweep takes about ninety-five minutes and aborts if any `.rs` or
`.toml` in the tree is edited while it runs. **Two independent guards fire, and
both consider only those two extensions**: `main::refuse_if_self_is_stale`
compares the running `ui-verify` binary against the newest source under
`tools/ui-verify/`, and `launch::staleness_complaint` compares each driven
`pdfcer-gui.exe` against its own source root. Either turns every remaining chunk
into an `rc=2` usage dump, and `sweep-full.sh` aborts the whole run on the first
`rc=2` — so a sweep forbids exactly the work its ninety-five minutes of wall
clock invite. Markdown, `.sh` and `.py`
edits are therefore safe during a sweep, **with one exception**:
`tools/ui-verify/sweep-full.sh` is the script bash is currently executing, and
bash reads a running script incrementally from a byte offset rather than loading
it whole. Editing it mid-sweep makes the live run execute shifted bytes from the
middle of a command. Leave that one file alone until `=== SWEEP-DONE`.

#### How a check is launched

**Each check gets its own copy of the binary and its own `userdata/`.** That is
the default; `--shared-profile` turns it off and exists only to reproduce an
old run. The consequence binds every check written for *remembered* behaviour:
only launches **inside one check** share persisted state, so a ladder proving
that a preference is stored, re-read when the document opens, and still right
after a mode change has to live in a single check rather than across a run of
related ones. Which trace event carries the answer is part of the assertion —
`off-page-seed` says the answer came from the store at document open,
`off-page-mode` says it came from the mode change — so a build that computes
the right boolean by the wrong route fails on the event name before a pixel is
compared. Assert the route, not only the value: a single global boolean is the
implementation anybody writes first, and it satisfies every rung that does not
name its source.

**`--second-pdf` must be a document of more than one page.** The refusal
messages in `page_drag_between_documents`, `document_tabs`,
`drop_onto_thumbnails`, `tab_reorder` and `attachment_clip` say only that it
must be *different* from `--pdf`, which is true and not sufficient: the engine
refuses to leave a document with no pages, so a one-page second document cannot
be moved out of, and all five fail on a perfectly valid file. Supply a
multi-page one, and then a failure is a failure.

**Launch from the repository root.** The driven suite runs a copy of the exe
out of `target/scratch/drive/` so a rebuild cannot swap the binary under a
measurement — but the fixture path is resolved against the working directory,
not against the exe. A relative `--pdf` with the working directory anywhere
else opens no document, and the application looks as though it refused to open
a file. The tell is a trace with no `status page=` line at all.

**A smoke launch is not idempotent.** The application writes its dock
arrangement back to `userdata/` beside the exe, so a run that closes a panel
makes the next run find that panel absent. The second run's report is
plausible, names a surface genuinely not on screen, and gives no hint that the
first run is the reason. A check asserting about a toggled surface must first
ask whether the surface is already drawing; it may never assume the arrangement
it last left behind. The blast radius is bounded only because the smoke launch
drives this repository's own release binary and not the operator's published
one — the same technique aimed at a copied exe writes beside that copy, which
is the other reason the driven suite copies first.

**Read the LAST `canvas-coverage` line, not the first.** It is a change log,
and the first line says `covered=0.000` because it is published before the page
has rendered. A pre-release check that reads the first line holds a good build.

**A check whose oracle is "nothing was clicked" must set `LaunchSpec::place` to
`false`.** `Session::place` moves and focuses every launched window
unconditionally, and it is on by default (`LaunchSpec::place` is `true` in
`LaunchSpec::new`), so a launch that asks for an off-screen viewport still ends
up wherever `place` puts it. Two costs follow, and the second is the one that
misleads: the window lands on the operator's screen while a doc comment says it
is nowhere near him, and a check written to prove *the application did this
without any input* has just had its harness supply the focus change it was
measuring the absence of. `load_anomalies`' status-bar half is the worked
example — it launches at `-4200,-4200` with `place` off, reads the trace and
nothing else, and can therefore run while the operator is working. The
off-screen constant and the flag must not be separated: without the flag the
coordinates are simply discarded.

**A keystroke is not a reliable harness primitive while a dock panel is open.**
A chord is routed through whatever holds focus, and an open panel holds it, so
the same step passes and fails across runs of one unchanged binary — the key
never arriving at all, or arriving on one attempt out of five. The remedy is
not a longer wait or a retry loop. Click the command (`view.tool_select`, say)
instead of pressing its chord.

**A driven failure is a claim about the check, and the check is the half nobody
examines.** Most of what a first driven run reports as an application defect is
the check itself: it compares two trace fields that were never the same shape,
or it asks for a control in a mode that correctly does not show it — a step
hunting the Pages tab in **Read**, where the manifest gives that mode File and
View and nothing else, reports a tab missing from a build that is behaving
exactly as `MODES_AND_PANELS.md` specifies. This concentrates in checks written
by a session that could not drive, because between writing such a check and
running it nothing tested its assumptions at all; the desktop requirement below
is why such sessions exist. The failure message is no help and is actively
misleading, because a well-written one names the missing region precisely and
even offers the right alternative reading — which makes a statement about *what
the check looked for* read as a measurement of the application. ⚠ **Triage
before fixing.** Read the trace for what the program actually published before
editing program code, or the repair is a change made to a working build to
satisfy a mistaken oracle.

#### The desktop the driven suite needs

**It needs a machine somebody is sitting at, and that is the opposite of what
the read path implies.** Windows grants `SetForegroundWindow` only to a process
that is already foreground, received the last input, or has recent user input
behind it. After a few hours with no human at the desk, *every* driven check
reports that the window could not be brought to the front — with no code change
and every gate green. So the natural inference, that a driven run is the thing
to schedule overnight, is exactly wrong: schedule it for when the operator
hands the machine over, not for when nobody is there.

A window-activation SKIP is an **environment** verdict, not an application one.
It survives re-running, which is precisely what makes it read like a defect.

The corollary decides what an unattended session can do at all: **screen
capture needs no foreground rights; only synthesised input does.** So the
unattended set is the unit tests, the gates, an off-screen launch under
`PDFCER_DIAG_VIEWPORT` / `PDFCER_DIAG_INVOKE` asserting on the trace, and the
capture checks. Anything that clicks or types waits for a manned desk.

`PDFCER_DIAG_INVOKE` takes a **comma-separated list** of command ids and
presses one per frame through the real dispatcher, so a whole sequence runs
with no pointer at all:

```sh
PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT=-4000,-4000,1200,850 \
PDFCER_DIAG_INVOKE=file.print target/release/pdfcer-gui.exe file.pdf
```

Clear the desktop first — `(New-Object -ComObject Shell.Application).MinimizeAll()`,
and `UndoMinimizeALL()` afterwards — because a covering window produces a SKIP
that reads like a defect. Two on this machine are named:

- **`wscript.exe`.** A Windows Script Host error dialog raised by the
  operator's `SWRecovery` scheduled tasks parks itself over the application.
  The harness detects it by name and refuses to click through it, which is
  right. Closing the `wscript.exe` process clears it. The scheduled tasks are
  deliberately not modified, so it recurs.
- **`osk.exe`.** The on-screen keyboard covers the ribbon and swallows
  synthetic clicks: it is UIPI-protected, so input posted to it from an
  ordinary process is discarded with no error. A driven failure on this machine
  is a harness question before it is an application one.

**Never kill by image name.** `pdfcer-gui.exe` is also the operator's daily PDF
reader, and `taskkill /F /IM pdfcer-gui.exe` takes his open documents with it.
Kill by PID, and verify the PID's path against the binary the run launched.

That rule sets the kill *order* when two sweeps end up driving one pointer.
Killing a `ui-verify` while its driving shell loop survives lets the loop start
the next check, and a second detached launch then runs beside the first — two
harnesses on one cursor, and for the next half-minute neither verdict means
anything. Kill the **loop** first, by command line; then the harness; then the
GUI. Confirm exactly one `ui-verify` is left before trusting a single verdict.

#### A panic is not automatically a failure

`Session::trace()` refuses to hand back a trace from a process that died, or
from one whose worker threads died, unless the check said it might.
`Session::expect_thread_panic()` is how a check says so, and it is a blunt
instrument: it excuses **every** panic for the rest of that session, so a check
driving the zoom ladder would go green on a build whose render worker died of
something unrelated. One check calls it and argues its case in writing
(`checks/raster_wall.rs`). Anything else that needs it should get an
event-checked form instead — `expect_thread_panic_reported_as(event)`, matched
against the trace `trace()` has just read — so that the excuse is scoped to the
one panic the check provoked.

⚠ **Never match on `panic_message`.** The engine states in writing that the
panic text is third-party `tiny-skia` output and explicitly not a contract.
Match on the application's own published event instead: `raster-limit scale=…
region=…`, written by `render::worker`.

The panic is the delivered fix working, not something the check tripped over.
`pdfcer-render` catches the rasterizer's panic at its one `catch_unwind` and
returns `RenderError::RasterizerLimit`; the canvas learns a ceiling, backs off
×0.75 and ratchets, and the operator gets a sentence in
`status-group:raster-stop`. The engine's panic hook is deliberately not
silenced, so the harness will keep seeing the text.

**`pdfcer_render::MAX_GUARANTEED_REGION_SCALE` is not falsified by that panic**,
and the way to get this wrong is to read the wrong number. It is a region
**scale factor**, published as a floor — the scale below which the renderer
promises not to give out, never the scale at which it will. The last rung a
sweep report names is a **zoom percentage**. The zoom-gallery panic happened at
a region scale of 509,704, comfortably above the 250,000 floor, on a run whose
last reported rung was 3,099,514 %; reading that rung as the scale makes the
constant look two orders of magnitude wrong when it is exactly right.
`raster_wall` declares that wall; the deep-zoom checks climb into the same one
and do not.

#### Reading a failure

Three rules, each paid for by a check that reported the wrong thing
confidently.

**A check asserting on an absent line asks what else happened before it
reports a failure.** The absence of the line it wanted is the weakest
possible evidence about what the application did: the step may have
succeeded under a different event name, or a step before it may have
refused. `canvas/resizing.rs` carries the worked example — a driven check
reported *"Apply committed nothing"* while the same trace showed the
object's bounds moving 317.87 → 358.00. A failure message that names only
what was missing sends the next reader to the wrong half of the program.

**Anything that draws carries a screenshot capture on its failure branch.**
The harness has no AccessKit, no OCR and no text extraction, so when a
layout or a legibility assertion fails, the pixels are the only oracle that
can say why. A capture taken at the moment of failure costs nothing on the
passing path, and it is the difference between a defect that is diagnosed
and a defect that has to be re-driven to be looked at.

**A green check that has never been red is not evidence.** R1 says a UI change
is done when it has been asserted against the running binary; it does not say a
driven check certifies itself. A check that has only ever passed is
indistinguishable from a check that asserts nothing, so the moment it first goes
green is the moment to distrust it — falsify it before believing it, and falsify
**each clause separately**, changing one constant at a time.
`load_anomalies_reach_the_status_bar` asserts two independent things: that the
census line draws for a document with anomalies, and that it stays off for one
without. Pointing `CONTRADICTS` at the clean fixture and pointing `CLEAN` at the
dirty one are therefore two different experiments, and each must not only go red
but go red with **its own** complaint — one saying the status bar declared no
`status-group:load-anomalies` region, the other saying the line is on for a file
with nothing to disclose. One message covering both falsifications, or one
falsification that stays green, means a single clause is carrying the check and a
regression in the other half will be silent.

**Before any falsification, snapshot the post-edit state of every file the
experiment will touch.** A falsification works by restoring a *worse* earlier
state, so the backup taken before the edits is the wrong artefact to restore from
— it is there to undo the experiment, not to carry your work, and restoring it
over a file you also finished corrections in destroys them silently, behind a
green gate and a clean-looking tree. The tell is cheap and must be run
immediately after every restore, for every file touched: `grep -c` a phrase you
know you wrote, and treat **0** as data loss rather than as a search that missed.

**When an upstream fix stops a falsifier firing, the assertions beside it
stop measuring and go on passing.** A falsifier plants the condition a check
exists to detect; if the planted condition can no longer occur, every
assertion downstream of it is being evaluated against a case that never
arrives. Invert the control or grow the fixture so the condition is
reachable again. **A falsifier is never simply deleted** — deleting it
converts a check that measures into a check that cannot fail, with no
change in its colour. `DEFECTS.md` D52 registers one instance.

#### Fixtures a check may not assume

`fixtures/` holds the purpose-built documents, each with a `PROVENANCE.md` or a
generator beside it. Two facts about the set decide how checks get written.

**`fixtures/paragraph.pdf` is the only fixture with a reflowable paragraph.** A
title block has none, and `tail-alignment.pdf`'s blocks are flush by
measurement rather than by wrapping. A reflow check aimed at any other fixture
passes while measuring nothing. `paragraph.pdf`'s generator prints the geometry
a check quotes.

**A check that shares its document with the run's `--pdf` is a check whose
inputs are not under its own control.** The remedy is a `const FIXTURE` beside
the check. These are the ones still owed one, and what each needs:

| Check | Needs |
|---|---|
| `page_ops_round_trip` | `fixtures/four-pages-unrotated.pdf` — **not** the engine's `synthetic/pageops/four-pages.pdf` its own message suggests, whose four sheets are all US Letter while this repo's same-named file's differ. Correct the message in the same edit. |
| `pages_drag_shows_where_it_lands` | `fixtures/four-pages.pdf`; it needs three thumbnails or more |
| `blend_space` | `fixtures/transparency-cmyk.pdf` |
| `exporting_form_data_writes_a_file` | a document carrying an AcroForm — one fixture serves all three forms checks |
| `a_drawing_dropped_on_the_thumbnails_becomes_pages` | a multi-page **first** document. It already pins its second PDF and leaves the first shared, which is the trap: it reads as though both inputs were pinned. |
| `signature_trust_is_reported_as_its_own_fact` | not a fixture — the check must activate the Signatures panel before reading it, or no `dock.tab.view.panel_signatures` region is ever declared |

**The same trap exists in the unit tests, and there it is a constant rather
than a flag.** `app::state::FOUR_PAGES` is `"pageops/four-pages.pdf"` and
resolves through `open_fixture`, which joins it to the **engine's** read-only
corpus under `D:\Dev\pdfcer\fixtures`. This repository has a same-named file
whose four sheets differ in size where the engine's are all US Letter, so a test
that reuses the constant is measuring a document this repo does not own and
cannot change. `crates/pdfcer-gui/src/app/state/fixtures.rs` owns the rule and
states it once: two named openers, `open_fixture` for the engine's corpus and
`open_local_fixture` for `fixtures/` here, deliberately not one function taking
a root — a boolean got backwards picks the wrong tree silently, two names
cannot.

**Two checks must never be "repaired".** Counting either as a harness defect
would grow the repair list with work that makes the suite worse.

`a_save_that_would_produce_blank_pages_is_refused` is reporting a **fixed
engine**: the guard walks a real four-level page tree, finds nothing wrong, and
a file is written. Delete the check. ★ **The save-time guard itself stays** —
one tree walk at save time is all that stands between a regression and a file
that opens in Acrobat with blank pages on the end.

`removing_embedded_fonts_reaches_the_document` is reporting a **correct
refusal**: every embedded font in its document is identity-encoded or Type 3,
pdfcer judges unembedding unsafe, and the greyed button is right. It needs a
document with a font whose verdict is `removable`; `pdfcer list-fonts <file>`
names the verdict per font.

**And one pair must never be fused.**
`load_anomalies_reach_the_status_bar` and
`load_anomalies_are_listed_in_document_properties` assert the same disclosure at
two surfaces, and merging them into one check looks like an obvious tidy-up
because the fixture, the launch and the expectation are shared. They cost
different things: the status-bar half needs no pointer and no keyboard — the bar
draws on the first frame after the document opens, so the harness launches,
waits and reads the trace — while the panel half has to click a mode segment, a
ribbon tab and a ribbon item. Fusing them makes the cheap, always-runnable half
inherit the expensive half's `--no-input` SKIP, **and a SKIP is not red**, so
the whole disclosure would silently stop being evidence on every run where the
machine's pointer belongs to somebody else. Split, the status-bar half runs on
every sweep including the ones nobody can watch. The general shape is worth
carrying: two checks over one fact are not redundant when one of them can run in
conditions the other cannot.

#### Re-baselining after the canvas origin moves

An icon rail, a ruler gutter or a dock-width change moves where the canvas
begins. The coordinate half of that cost is already paid, structurally: a check
**cannot** write a screen coordinate, because `ScreenPoint` has private fields
and no constructor, and the only route to one runs from a `DocPoint` through
the `CanvasMapping` the application published this run and the `WindowFrame`
measured from the live window. Document-space aims survive any layout change,
and there are no window- or screen-space literals in `tools/ui-verify/src/checks/`
to go stale.

What does not survive is anything a check *names* rather than derives: the
published region names, the rects read back from them, and any expectation over
a captured pixel area. **Convert them in one pass, never a few at a time.** A
partial conversion is worse than none, because an aim that is wrong and happens
to land somewhere plausible is a green result reporting nothing — and it is
symptom-identical to a genuinely broken coordinate conversion, which is why the
last instance of this cost an investigation into a coordinate-space defect that
did not exist.

### Two gates worth building

Neither exists. Both instrument the same failure: a comment that was true when
it was written and that nothing checks. They are specified here so the next
build of them starts from the design rather than from the symptom.

**`tools/gates/check-const-doc-magnitude.sh` — a constant's doc must agree with
its value.** Scan every `.rs` file in the tree for `(pub )?const NAME: T =
<numeric literal>;` together with the doc-comment block immediately above it,
then apply two filters. First, keep the constants whose comment contains
digits — over this tree that is roughly 250, far too many to read. Second, of
those, keep the ones whose comment and literal have **disjoint number sets**:
every number the prose names is absent from the value it sits on. That leaves
about 128, which is a list somebody can work through. The gate then fails on a
constant whose doc comment carries a numeric literal, or a magnitude word —
*thousand*, *million*, *billion*, *trillion* — that disagrees with the value
declared beneath it. That is the exact shape this catches: a comment arguing a
constant sits "an order of magnitude inside the confirmed range" while the
literal sits at the edge of it, or a threshold described in percent after the
power of two behind it was cut.

A session samples; a gate enumerates. That is the whole argument for building
it — the filter has been run, and only a handful of its candidates were ever
traced to the commits that moved them, so the rest are scanned and unexamined
and will stay that way as long as this is somebody's afternoon rather than CI's.

⚠ **Precondition, every time.** Confirm the `Cargo.lock` engine pin equals
engine HEAD *before* running the scan. Without that, a comment disagreeing with
an engine-derived number cannot be told from a comment that is merely ahead of
the pin, and the audit stops being decidable.

**A citation in a doc comment must carry the revision it was read at.** The
cheaper gate: fail any `\w+\.rs:\d+` appearing in a doc comment that is not
accompanied by the revision it was read at. It does not stop drift; it makes
drift **visible**, which is the actual problem — a rotted line citation is
indistinguishable from a good one at the moment a reader checks it, so the
reader who checks it finds plausible code and stops reading. §5.1 forbids line
citations of the engine outright; this covers the ones inside this repository,
where a line number is permitted and still moves.

⚠ Read `D:/dev/rag/rust/gits_default_short_hash_length_grows_with_the_object_count_so_a_gate_joining_h_against_pinned_citations_fails_all_at_once_on_a_fresh_clone.md`
before building it. Git's default short-hash length grows with the repository's
object count, so a gate that joins `%h` against hashes already pinned into
comments fails every row at once on a fresh clone — and a gate that fails
everything is a gate somebody disables.

### Packaging

```sh
python tools/package-portable.py --verify --note "what this milestone added"
```

Writes `D:\builds\pdfcergui-<stamp>-<engine>-<shell>\` — one folder per build,
never an overwrite, because a running Windows executable cannot be replaced and
a half-updated folder is worse than either version. `<engine>` is
`D:\Dev\pdfcer`'s short HEAD, `<shell>` is this workspace's; either gains a
`-dirty` marker when its tree carries changes that can reach a compiler.

A source digest is recorded alongside, and joins the folder name when the shell
tree is dirty. It cannot say what the code was, only whether two builds came
from identical bytes — which is the question a defect report actually asks.

`--verify` runs the tests and the gates **before** building, so a failure costs
nothing and leaves no folder behind. When it is not run, `BUILD-INFO.txt` says
so in those words; an omitted line would read as "nothing to report" when it
means nobody checked.

**The test row of that block is a total over every `test result:` line, and it
has to be.** The tail of a workspace run is the *doctest* summary, which on
crates carrying no doctests reads `0 passed; 0 failed` — so a block quoting the
last lines is one a suite that ran 4,600 tests and a suite that ran none both
produce, identically. A run yielding no `test result:` line at all, or a total
of zero passed, is recorded as FAIL whatever the exit code was. `--self-test`
plants the last-line reading and requires the check to catch it.

**The packager moves the engine pin itself, as its first step, and two things
follow that catch people out.** It runs `cargo update` over the three engine
crates before it reads the build's identity — because `Cargo.lock` is what
`locked_engine_rev` reads, so updating afterwards would name one revision in the
folder and link another, and because a branch dependency otherwise ships an
engine older than `D:\Dev\pdfcer` has with nothing anywhere to say so. First:
**a `cargo test` run by hand five minutes earlier does not describe the binary
this produces.** The pin can move under the build, and a `--note` typed before
the update names one revision while the binary links another. `--verify` is what
closes that, precisely because it runs after the update; `--no-update` is the
other way to close it, and it is for reproducing an exact earlier revision
rather than for routine use — the standing instruction is to carry the latest
engine. Second: **`cargo update` rewrites `Cargo.lock`, so a build packaged from
a provably clean tree is named `-dirty` by construction**, the identity block
reading `git describe --dirty` a few lines later. Committing the lock and
re-packaging into the same slot is the obvious workaround and it has a quiet
second cost: `changelog` diffs against the build it finds in the destination,
which is now the retracted one, so the corrected package reports a single
pin-bump commit for a release carrying the whole run of them.

**Launch it from a shell whose `PATH` names Git Bash first, or know what it
picks instead.** `--verify` spawns `tools/gates/run-all.sh`, and the interpreter
it spawns is resolved from the inherited `PATH`. From PowerShell, Task Scheduler
or an IDE that is `C:\Windows\system32ash.exe` — the **WSL launcher** — which
sees a Linux `PATH` with no `cargo.exe` on it and Windows paths it cannot walk.
A tree that is `61 passed, 0 failed, 0 skipped` by hand then comes back
`25 passed, 22 failed, 14 skipped`, and nothing in that report names bash, so it
reads as a broken repository rather than as a measurement of the wrong
interpreter. `_bash()` excludes the launcher by identity and searches Git Bash's
own install locations; `--self-test` plants the condition — `PATH` cut down to
the system directory — rather than observing it, because read straight the
failure cannot occur on a machine where it is being asserted.

The packager also mirrors the finished build into the older of two OneDrive
slots, so the previous build stays reachable to fall back to and to compare
against. **That mirror can be refused, and the refusal is safe but not free.**
OneDrive's own sync client holds open the files it is uploading, so a slot can
be locked with no process of ours running from it; the copy then fails with
`WinError 32`. Nothing is lost — the sequence stages the new build, renames the
slot aside and renames the staging in, and a failed rename moves nothing — but
what it costs is the **rotation**. The slot that is supposed to hold the
previous build silently stops being updated, every run reports success, and the
fallback ages one build at a time. A refusal that repeats is worth chasing to
whatever holds the file, not retrying.

## 4. Standing rules

| | |
|---|---|
| **R1** | A UI change is done when it has been asserted in `tools/ui-verify` against the running binary — not when a test passes. |
| **R2** | No source file over 1,500 lines. `check-file-size.sh` enforces it. When a file approaches the limit, find the seam; do not raise the limit. |
| **R3** | Salvaged code is re-verified, never assumed. |
| **R4** | The gates apply from every commit, not at fold-in. |
| **R5** | The documentation is the logic: a reader should be able to reconstruct the program's behaviour from it. See §5 for the form that takes. |
| **R6** | Nothing in `FEATURES.md`'s shipped column regresses. |
| **R7** | `egui-shell` never learns what a PDF is. Add an extension point, not an exception. |
| **R8** | A capability's presence is expressed by registering its command. No `#[cfg(feature)]` in the ribbon; an item naming an unregistered command is dropped, with a `CapabilityAbsent` skip reason. |
| **R9** | No placeholders. An unavailable capability renders nothing. Greying is reserved for *temporarily* unavailable, and is always explained on hover. |
| **R10** | A request names a behaviour, not a widget. Deliver everything that behaviour implies — its states, its refusals, its undo, every surface it shows in — not the literal sentence asked for. |
| **R11** | Where an interaction is unspecified, take it from Acrobat, Inkscape or SolidWorks. Record which one you followed and why. Do not ask. |

R10 is the operator's own standing instruction, and it reframes every other
rule here:

> *"when I ask for something, my expectation is usually that everything
> surrounding that request is also done to where it would match the behaviour
> a user would expect. Otherwise I am left typing out every little missing
> detail."*

R11 names three applications, and each is named for a reason. **Acrobat** is
what pdfcer replaces, so it sets what an operator already expects a PDF tool to
do. **Inkscape** is the vector editor whose dock and tool model this shell
benchmarks, so it settles selection, handles, snapping and panel behaviour.
**SolidWorks** is where the operator's drawings and his muscle memory come
from, so it settles anything about dimensions, sheets and drawing convention.

The operative half is the second sentence. **Do not ask the operator how an
interaction should behave** — look at what those three do, pick one, and write
down which one you followed and why, in the doc comment on the code that
implements it, where the next reader will be standing when the question comes
up again. `FEATURES.md` and `ACROBAT_DEFAULTS.md` both cite this rule and give
worked examples; this is where it is stated.

Two rules inherited from the engine:

**Fuzzy, never sneaky.** Applied content renders exactly as saved content will
render — no badge, tint, flag or dashed outline drawn into the page view to mark
pdfcer's own uncertainty. Disclosure lives off-canvas: status line, results
panel, report, properties field. Pre-commit affordances (snap indicators, hover
highlights, rubber-bands, selection handles) are the cursor and are welcome.
The test: would a screenshot of the editing canvas differ from a screenshot of
the same document saved and reopened?

**Never write a bare "dimension".** **ce dimensions** are the ones pdfcer
authors; **pdf dimensions** are CAD-exported page content pdfcer reads and must
not silently alter. They have opposite properties. This applies in code,
comments, commits and specifications.

### 4.1 Registering a command carries six obligations

R8 makes command registration the only way the GUI learns a capability exists,
so registration is the seam where a new feature either arrives whole or arrives
as a button that does nothing. Six things must be true, and each fails loudly.
The first five check that the registration is internally consistent; the sixth
is the only one that asks whether the command *does* anything.

1. **The registry size.** `shell::commands::ledger` asserts a literal
   `registry().len()`, conditioned on the `signing` feature. Beside it, the
   icon-coverage test asserts the identity `named + refused == total` — a
   property, not two more literals that can drift apart.
2. **The ribbon shape.** `shell::manifest`'s shape test asserts the ordinary-tab
   count, the contextual-tab count, and the total group count across all tabs.
3. **Removal from `PLANNED`.** A command named in `shell::manifest::registers::PLANNED`
   is struck from it when it is registered; `DIRECTED` and `PLANNED` are
   asserted disjoint.
4. **Regenerating the RON.** `shell/ron/built_in.ron` is generated. Rewrite it
   with `cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored`; the
   round-trip test fails until you do.
5. **A `KNOWN` entry** in `shell::commands::tests::KNOWN` for any new
   `enabled_when` condition name, walked by the predicate test.
6. **Reachability.** The command must be named by a literal arm of
   `PdfcerApp::dispatch_command`, claimed by one of its guard arms, or listed in
   `shell::commands::reach::SCAFFOLDED` with a written reason.
   `shell::commands::reach` parses the dispatcher and asserts this. It is what
   stops a fully-registered, fully-consistent command from being inert.

### 4.2 Working several agents at once

Up to six agents can work this tree in one day, and what keeps them from
colliding is a written partition rather than coordination between them.

**Give each agent a write territory as an explicit list of directories, plus an
explicit do-not-touch list naming the others'.** An agent told only what it owns
will reach outside it the first time the crate does not compile — one has
already added an `Action` variant outside its territory to get a build going
again, which is a change nobody reviewed landing in somebody else's file.

**`shell/` is a single-writer resource.** `shell/commands`, `shell/manifest`
and the generated `shell/ron/built_in.ron` have to move together (§4.1), so in
a parallel run exactly one agent may register a command. Everybody else
**reports** the entry point their work needs and lets that agent wire it.

**Tell them not to commit**, and expect the crate not to compile for stretches
of the run. An agent that finds breakage outside its own files reports it and
carries on; it does not fix it. The other failure mode seen is an agent running
`cargo build` inside `D:\Dev\pdfcer`, which violates the read-only rule in §1 —
name that boundary in every agent's brief, not only in the brief of the agent
whose work touches the engine.

### 4.3 Five rules that are easy to violate and expensive to find

None of these is deducible from the code in front of you, and each has
already cost a driven defect.

**A control that must be reachable cannot be placed after an unbounded
`ScrollArea`, and reserve-and-hope is the same defect with a tuning
parameter.** An egui `ScrollArea` with no height budget takes all the space
there is, and anything laid out after it is drawn outside the panel — not
clipped-and-visible, simply absent, with every gate green because the widget
was constructed and its rect exists. Reserving a fixed strip for the control
and giving the scroll area the rest only moves the failure to the first
window size the reservation was not tuned for. Budget the control first and
give the scroll area what is left. `panels/dimension_groups/mod.rs` builds
its whole layout on this rule and cites the defect that bought it: an Add
button rendered at y=958 inside a body that ended at y=793.

**Two derivations of one position drift apart under use, and the compiler
cannot object.** `egui::Pos2` is screen space, canvas space, page space and
per-viewport space at once — one type for four coordinate systems, so a
position computed twice by two routes type-checks whichever route is wrong.
The discipline is to compute once and pass it along: *the galley that was
drawn is the galley that is hit-tested* (`canvas/textedit/hit.rs`). Where a
second derivation is unavoidable, assert the two agree rather than trusting
that they will.

**A knob does not sit at a value chosen to fix something it does not fix.**
When a constant is raised and the symptom improves, the improvement is
evidence that *something* changed with it, not that the constant was the
cause. `dialogs/host.rs`’s `FOCUS_FRAMES` went 1 → 8 → 40 → 120 on evidence
that was real and repeatable at every step, and sits at 8 because none of it
was evidence about focus. A knob whose value cannot be derived from what it
controls is carrying somebody else’s bug, and it goes on carrying it after
that bug is fixed. State the derivation in the doc comment beside the
constant, or the next reader has only the number.

**Never `{:?}` a value a machine reads.** A `Debug` rendering is a formatting
detail of somebody else's type, and the engine's types are `#[non_exhaustive]`
and under active edit. A driven check that greps for `rect_derived=Artwork` goes
quiet the day `RectDerivation` gains a field, is renamed, or has its derive
removed — quietly, and in the direction that reads as *the feature stopped
happening*, so an unchanged build turns a green check red and a red one green
with nothing to look at in the diff. This project has already shipped a
machine-read field that inverted its meaning when an upstream `Debug` impl
changed shape, with the check quoting the truth in its own failure message while
reporting the opposite of it. So every trace field a check keys on is a **stable
token this repository owns**, emitted by an exhaustive match with no wildcard.
`app::actions::annots::rect_rule_token` and `app::state::policy_token` are the
worked examples, and both spell their tokens to match what the engine's own CLI
prints so a trace and a `pdfcer` command line compare without a lookup table.
Where the engine's enum is `#[non_exhaustive]` the final arm is the only
wildcard allowed, and it says `other` — an honest *this build does not know that
one* rather than a guess.

★ **And when one of these does break, fix the emitter — never loosen the
comparison.** A check relaxed to match either spelling is two surfaces
describing one fact in two languages, and the next change to the upstream
`Debug` impl passes both of them.

**Unsafe code is quarantined in two platform crates, and that is a property
to preserve rather than a fact to note.** `crates/pdfcer-gui` (both the
library and the binary) and `crates/egui-shell` carry
`#![forbid(unsafe_code)]`, and `forbid` cannot be relaxed by an inner
`allow`. Every `user32`, `kernel32` and `gdi32` declaration the **shipped
binary** reaches therefore lives in `native-window` and `native-clipboard`,
and a platform call written anywhere else does not compile. That is the
point: the boundary is enforced by the compiler rather than by review, so it
cannot erode quietly. `tools/ui-verify` is outside the quarantine by design
— it drives the OS, it is not shipped, and it reaches Win32 through
`windows-sys`.

Do not write down how many calls either crate declares. That number has been
wrong in three places at once, and nothing in the build checks it — grep the
`unsafe extern "system"` blocks when you need it.

## 5. Documentation and comment standard

The whole standard in one sentence: **write what the program is, never what it
was.**

Git holds the history. A reader — human or agent — opening a file wants the
current contract in the fewest words that carry it, and every sentence about a
past state is a sentence they must read and discard.

### 5.1 Rust

Follow rustdoc convention (RFC 1574).

**Module headers (`//!`)** — one sentence on what the module owns, then the
contract: the invariants it holds, what callers must not do, how it fits its
parents and siblings. Then non-obvious rationale, only where the code cannot
say it itself. Use `#` headings only when there are two or more real sections.

**Item docs (`///`)** — first line is one sentence, ends with a period, third
person: *"Returns the page rect in document points."* Blank line, then detail.
`# Errors`, `# Panics`, `# Safety`, `# Examples` where they apply. Never
restate the signature.

**Inline (`//`)** — only where the *why* is not obvious from the code. Never
narrate the *what*.

**Delete on sight:**

- Dates, commit hashes, version stamps, "measured on <date>".
- Tracking identifiers: request numbers, pass numbers, review-finding letters,
  session names, request filenames.
- Decorative emphasis markers and the shouting attached to them.
- Narrative: *"was split out when…"*, *"used to say…"*, *"this is the eighth
  instance…"*, *"the previous sentence was wrong because…"*, *"shipped on X
  saying Y"*.
- Quotations of the operator, of other documents' prose, or of past findings.
- Justifications for a file split that cite line counts or gate pressure.
- Restatement of the code on the next line.
- **Line numbers in citations of the engine.** Cite `pdfcer-core`,
  `pdfcer-render` and `pdfcer-print` by symbol — a function, a type, a method,
  a doc heading — never by `file.rs:NNNN`. The engine is a branch dependency
  (§1), so its source is split and edited under this repository with no command
  run here, and a line citation drifts. `EditSession::set_text_run` survives a
  rewrite of the engine's file layout; `edit.rs` line 4211 does not. **It does
  not drift into a dangling reference, which is what makes the class
  dangerous**: upstream insertion shifts a file uniformly rather than
  scrambling it, so the number still lands inside readable prose about a real
  function in the right file and reads exactly as a correct citation reads. Of
  92 engine citations measured in one sweep, 66 had drifted and none dangled;
  two pairs had come to name each other's type. A repair keyed on *does this
  line exist* passes every one of them — resolve the enclosing symbol instead.
  `check-engine-citation.sh` enforces the shape and states its own blind spot.
  A line citation of a file *inside* this repository is fine; it is the moving
  external pin that makes the engine different. The same argument covers the
  archived GUI at `D:\Dev\pdfce\crates\pdfce-gui`: a frozen tree is not
  frozen-correct, because
  its citations kept drifting until the freeze, so what froze was the error.

**Keep, always:**

- Invariants, contracts, and who owns a mutation.
- Units and coordinate spaces.
- Gotchas that still bite, stated as a present-tense rule with its mechanism:
  *"`RichText::strong()` borrows the active-widget foreground, which this theme
  fills with the accent — it is unreadable on a panel. Use an explicit colour."*
- Why a non-obvious choice is the right one.
- Cross-references to the module that owns a rule, so it is stated once.

A design decision is worth keeping when knowing it changes what a reader would
write next. It is not worth keeping because it was hard-won.

**One formatting trap, in one module.**
`crates/pdfcer-gui/src/text/settings/` keeps its long operator-facing string
literals on single lines, and the convention is mechanical rather than
stylistic. Rust's backslash continuation eats the newline *and* the next line's
indentation; drop the backslash and the literal still compiles, still passes
every test that does not compare it to a hand-written expectation, and now
carries a run of spaces where the indentation was. `rustfmt` then joins the
source lines back into one, so the gap sits mid-sentence looking deliberate and
`check-string-gaps.sh` fails on something that reads exactly like a lost
backslash and is not. Adding the backslash back does not fix it. Leave the
literals on one line.

**Guard rail.** A comment pass must change comments only. Verify it
mechanically — strip all comments from the old and new revisions of every
touched file and diff the remainder — rather than by reading the diff.

### 5.2 Markdown

- Open with one sentence saying what the document is for and who reads it.
- Present tense, current state. No revision stacks, no dated addenda, no
  superseded measurements kept "for the reasoning".
- Give the **command** that produces a number rather than the number, wherever
  the number will move. A figure written in prose beside the thing it counts is
  a claim that decays.
- Tables for tabular data only. `check-doc-markup.py` rejects a row with more
  cells than its header, which is how an unescaped `|` silently truncates a row.
- Delete any document whose entire subject is a past event. Git has it.

### 5.3 Where the history goes instead

| Kind of fact | Where it lives |
|---|---|
| why this code is shaped this way | the doc comment, as a present-tense rule |
| what changed, and when | the commit message |
| what the operator asked for | `OPERATOR_REQUESTS.md` |
| what the engine owes us | `ENGINE_BACKLOG.md` |
| a lesson about a tool or ecosystem | `D:/dev/rag/` |
| a lesson about a real-world PDF | `C:/personal_rag/pdf/` |
| a session's narrative | nowhere. It is not a durable artefact. |

## 6. Licensing

pdfcer-gui is **MIT** (`LICENSE`), which covers everything in this repository
including the icon set — the operator's own art, recorded in
`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`.

It does not cover everything `pdfcer-gui.exe` contains. The binary statically
links `pdfcer-core` and `pdfcer-render`, which embed third-party font faces and
data tables, so this program redistributes work whose licences require their
notices to travel with it. Two surfaces carry them, and they are not redundant:

| Surface | Carries | Reached by |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, copied into every build | every licence **text**, in full | anyone who opens the package folder |
| **File ▸ pdfcer ▸ About pdfcer** | the **attribution** — who made it, what it is, on what terms, whether pdfcer changed it | anyone who runs it |

`THIRD_PARTY_LICENSES.md` is generated — `cargo about generate about.hbs -o
THIRD_PARTY_LICENSES.md` — from this workspace's real `Cargo.lock`. Never edit
it by hand. The `accepted` list in `about.toml` is permissive-only, so a
copyleft dependency entering the workspace makes generation fail and name the
crate; that failure is the licence audit.

`tools/gates/check-shipped-assets.py` enforces the arrangement: a
`PROVENANCE.md` beside every redistributed asset directory, a citation in both
notice surfaces unless the asset is our own work, the notice present in the
packager's payload, and the generated file not stale against its template.

**The OCR model weights are CC-BY-SA-4.0.** Shipping them unmodified is
distribution of a verbatim work in a collection and leaves pdfcer's own licence
untouched. **Modifying them — fine-tuning, retraining, quantizing, or
converting them to another runtime's format — creates Adapted Material, and the
result must be released under CC-BY-SA-4.0 or a compatible licence.** That is
an engineering constraint; see `crates/pdfcer-gui/src/text/about.rs`.

## 7. Version control

`.gitattributes` disables CRLF normalization for PDF and PNG fixtures.
`core.autocrlf` is true globally on this machine, and normalization lands **in
the index at add time**, not only at checkout. A PDF's cross-reference table
stores absolute byte offsets, so a normalized fixture is a corrupt one.

## 8. Reference documents

| Document | What it is |
|---|---|
| `FEATURES.md` | What works today. A row is ticked only when an operator can reach it in a real build. Authoritative for status. |
| `PROJECT_PLAN.md` | The charter: topology, build stages, the fold-in procedure, risks, open questions. |
| `GUI_ROADMAP.md` | The phased plan and the open scope questions. |
| `SALVAGE.md` | What carries over from the old GUI, file by file, and in what condition. Every status cell is a claim about the new crate and decays; re-measure before quoting one. |
| `RIBBON_IA.md` | Where every command lives, and why. The spec for the shell; do not improvise around it. |
| `MODES_AND_PANELS.md` | The Read/Review/Edit selector and the flexible panel system. |
| `SHELL_FRAMEWORK.md` | `egui-shell`: the manifest, the extension points, and R7. |
| `RIBBON_SCALING.md` | How the ribbon narrows, derived by driving Word. |
| `DEFECTS.md` | Known defects and the standing rules they earned. |
| `DESIGNS.md` | Designs argued and not yet built. A section is deleted when its design has been built **and driven**. |
| `OPERATOR_REQUESTS.md` | The standing backlog. Only the operator closes a row. |
| `ENGINE_BACKLOG.md` | Every capability the engine has that this shell does not reach, and the decision on each. |
| `FORMS_PARITY.md` | Every form-field capability Acrobat offers, against what the engine can write and what this shell reaches. Its gap register is the work list; its owner tags say who does each one. |
| `EDITABLE_SURFACES.md` | Every verb `pdfcer-core` implements, and where the operator reaches it. |
| `NO_SURFACE.md` | Shipped behaviour with a hard-coded value and no control. |
| `UNIT_SURFACES.md` | Every surface that shows or accepts a length. |
| `UI_TOOLKIT_PINS.md` | Which egui the shell is built against, and why. Read by `check-ui-toolkit-drift.sh`. |
| `BENCHMARK.md` | Measured rendering performance on a real CAD site plan. |
| `ACROBAT_DEFAULTS.md` | What Acrobat actually authors, measured from its own preference hive. |
| `mockups/*.html` | Interactive mockups of the ribbon, the window and the modes. |
