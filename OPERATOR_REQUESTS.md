# Operator requests — the standing backlog

> **Ken, 2026-08-20:** *"Where do you need to put these requests so they just
> get auto-repeated over and over again so I don't have to keep requesting they
> be done over and over again?"*

**Here.** This file is the answer, and this section is the contract.

## The contract

1. **Every request you make goes in this file, the moment you make it**, before
   any work starts on it. Not into a chat reply, not into a session summary,
   not into an agent's memory — into this file, which is in git, on disk,
   backed up, and read at the start of every session.
2. **Only you close a row.** I may move a row to *shipped-and-driven*; the row
   does not leave this file until you have used it and said so. A row I believe
   is done but you have not confirmed sits under **Shipped — awaiting your
   verdict**, not deleted.
3. **A row carries evidence, not a claim.** *"Done"* is not a status. The
   status is either a driven check by name, or a dated note saying exactly what
   was verified and how. If nobody drove it, it says NOT VERIFIED, in those
   words.
4. **A blocked row names what blocks it and where that is filed.** If it is an
   engine gap, the row names the file in
   `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`. A row that says
   "blocked" with nothing behind it is a row I have not done the work on.
5. **Nothing is silently rescoped.** If I ship half of what you asked for, the
   row stays open and says which half.

## Why this file and not something cleverer

The failure this fixes is real and it is mine: a request made in conversation
lives exactly as long as the conversation. Sessions end, context is compacted,
and an ask made in turn three of a long session is gone by turn thirty — which
is why you have had to repeat yourself. The agent-memory system is for *how you
work*; it is the wrong shape for *what you asked for*, because memories get
summarised and requests must not.

A file in the repository has none of those properties. It is read at session
start alongside `PROJECT_PLAN.md`, it survives compaction because it is on
disk, and its history is in git so a row cannot quietly disappear.

---

# ★★★ THE OPERATOR'S CURRENT DIRECTION — 2026-08-20

> *"put the text editing aside again. In the next version I just need the
> perimeter measuring tool to work with the group scale stuff the same as the
> other dimensioning tools have."*

**O1 is PARKED at his instruction.** Not closed, not solved — parked. The
findings stand and the engine request stays open. Do not pick it back up
without him.

**O3, the perimeter tool, is the single deliverable for the next build.** The
engine shipped its whole half on 2026-08-20; everything left is shell work.

## And the standing criticism, recorded rather than argued with

> *"We'll have to reconsider how you are going about the canvas later since it
> shouldn't take multiple 3 hour sessions each day to figure out how to get a
> cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

He is right and this belongs at the top of the file where it cannot be missed.
Two observations that are mine to act on, not his to have to make again:

1. **The basics were never audited as basics.** Ctrl+P had never been bound. A
   text caret had no index. Both are things every application in this class has
   on day one, and both were found by him rather than by us — because every
   test asked "does the thing I built work?" and nothing asked "does the thing
   everyone expects exist?". The keymap now has a list-shaped gate for exactly
   this reason. The canvas needs the same treatment and does not have it yet.
2. **Sessions have been spent diagnosing, not building.** Three of today's
   findings were engine defects that presented as shell defects, and each cost
   hours to localise. That is partly the boundary and partly that the driven
   checks were reading traces rather than pixels — a trace can say the verb ran
   and cannot say the screen changed. Two checks were fixed today to stop
   producing confident wrong diagnoses. That is the pattern to keep pulling on.

# OPEN

## O115 — ◑ **OPEN 2026-09-03 (night)** — three defects the driven sweep found, none of them from today

Found by running the full driven suite against the fixed build, and **all three
verified as PRE-EXISTING** by re-running them against the previous release
(`09bb966`, the 14:13 build): they fail there too. Not regressions from the
review work.

### ★★★ 1. The canvas stops taking the pointer after you scroll a long way down

`scrolling_far_keeps_the_canvas_its_pointer_input`. **2 pointer events before
the wheel and 2 after it**, at a scroll offset of 1125 pt. The page is still
drawn, its rect is still published — only the input is gone.

★ The check's own note is the important part: this is the condition **O23 was
blaming on its pasteboard**, and it reproduces with **no pasteboard in the
build**, from an ordinary wheel scroll. So it is not the pasteboard, and it is
something an operator meets whenever they scroll down a long drawing.

### ★★ 2. The resize grips consume the drag and commit nothing

`resize_scales_a_shape`. The click selects (`selection-set page=0 object=0
via=press` is in the trace) and the grip drag produces **neither `resize-commit`
nor `resize-declined`** — silent on both channels.

★ This one is a **regression from a known-good state**: the check's own text
says *"until 2026-08-19 every resize drag was consumed and thrown away, so a
build that has reverted to it is silent on both channels."* It was fixed, and it
is back.

### 3. The wheel-paging toggle does not turn pages

`the_wheel_turns_pages_when_the_operator_asks_it_to`. Still on page 1 after the
toggle was pressed and the wheel rolled. The check names the likely cause:
`OpenDoc::prefs` is a **snapshot** adopted when Settings is applied, so a wheel
preference writes the file correctly, draws the control correctly, and changes
nothing until Settings is opened and applied.

⚠ **Weaker evidence than the other two.** The run's own notes show the checks
share the preference file — *"the wheel setting was left on `flip` by an earlier
run"* — so this one could be harness state rather than the program. Confirm on a
clean profile before acting.

### ⬜ The sweep is INCOMPLETE and that is stated rather than implied

**33 of 153 checks have run**: 51 passed, 3 failed, 21 skipped. The remaining 120
have not, and a partial run proves nothing about them.

★★ **Two process findings, both mine, both costly:**

1. **A driven sweep locks the SOURCE TREE, not just the pointer.** Editing under
   `crates/` mid-run makes the staleness guard skip every remaining check —
   141 of 153 the first time.
2. **The harness's stdout is block-buffered when it is not a TTY, so a killed
   run yields NOTHING** — not even the checks that had already passed. Two full
   runs were lost that way. It is run in small chunks now, each written to its
   own file, so a kill costs one chunk.

★ And a third, external: a stray **Windows Search flyout** holding the desktop
turned 17 checks into SKIPs with a foreground error. The harness named the
holder, its class and its pid, and said no retry would help. **Read the skip
REASON, not the count** — a single dominant reason across unrelated checks is
the environment, not the suite.

---

## O114 — ◑ **INTAKE 2026-09-03** — an outside GUI review, twenty findings, one of them a crash

**Ken, 2026-09-03:** *"new feature request in d:/dev/featurerequests/pdfcer-gui.
I'm not at the PC. it is yours to use."*

An outside review from `D:\Dev\pdfcer-gui-consultant`, against the packaged
build `pdfcergui-20260903-1605-e27c3b4-49b4b4b-dirty-16a4bb6081b9`. Twenty
findings ranked by cost to a user, a proposed design direction ("Board"), 80
screenshots, and a PowerShell driver that never touched this repository.

**Read at** `D:\Dev\FeatureRequests\pdfcer-gui\` — `REVIEW.md`, `HANDOFF.md`,
`screenshots/INDEX.md`, `mockups/`, `logs/`.

### ✅ A1 — the crash — FIXED THE SAME SESSION

> `pdfcer ▸ Keyboard shortcuts` panicked at `dialogs/host.rs:943`:
> **"viewport callback ran twice"**, on a fresh launch, taking the open
> documents with it. Unsaved markup was lost in the reviewer's session.

**Reproduced in twelve seconds** with
`PDFCER_DIAG_INVOKE=file.shortcuts`. The cause: `Host::show` took the dialog
body as `FnOnce`, moved it into an `Option`, and `expect`-ed on the second
call. But `Context::show_viewport_immediate` takes **`impl FnMut`** — egui
reserves the right to run a viewport's callback more than once per frame and
exercises it routinely, whenever anything in the body triggers a discarded
pass to re-size itself.

★★★ **The choice the old comment defended — "crash rather than draw a blank
dialog" — was a FALSE DICHOTOMY.** The third option is what egui means by a
second pass: **draw again.** The body is now `FnMut` and is simply re-run, with
the last result returned, exactly as egui's own implementation keeps the last.

★ **It was not part of a pattern.** A sweep of the whole workspace found only
**11** panic sites on runtime paths, and every one asserts an invariant of
*ours* — *"handled above"*, *"dispatch does not own this command"*, *"two shell
commands claim the same id"* — each of which a test also covers. This one was a
category error: **a panic asserting a third-party library's contract, on a
condition that library documents as permitted.**

### ★★★ AND THE HARNESS REPORTED PASS ON THE CRASHING BUILD

The reviewer wrote that *"the suite drives many dialogs but not this one"*. That
is **wrong, and the truth is worse**: `dialogs_open_in_their_own_window` drives
`file.shortcuts` and had been reporting

> ★ Keyboard shortcuts is a real OS window: [[186.0 209.0] - [606.0 689.0]]

**PASS.** Not by luck — the `viewport-inner` line it greps for is written
*before* the panic, so the evidence the check wanted already existed by the time
the process died. The check was not wrong about what it asserted. It had no
opinion about whether the program was still alive, **and neither did any of the
other 152.**

⇒ Every trace-reading check in this harness could pass on a build that crashes,
provided the crash came after the line it greps for. Fixed **in the one function
they all call** rather than by a rule each must remember:

- `Session::trace` now refuses to hand back a trace from a process that exited,
  unless the check said `session.expect_exit()` first — greppable, and a
  statement rather than an omission. Two checks legitimately expect an exit and
  now say so.
- The refusal is **fatal**, not a skip. `Error::fatal` and
  `CheckReport::from_error` were added and **all 152 `Err` arms rewritten**,
  because this project's own record is that *a SKIP is not red, so a check can
  stop running unnoticed*, and a crashed program reported as "did not run" is
  barely better than one reported as a pass.
- **Falsified**: with the crash planted back in, the same check that reported
  PASS now reports **FAIL** and quotes the panic line.

### ◑ A2–A20 — triaged in full: **[`REVIEW_TRIAGE.md`](REVIEW_TRIAGE.md)**

Every finding read against the source, with the mechanism at `file:line` and the
ruling quoted where the project already made one. **Five more fixed** beyond the
crash (A3's real defect, A4, A5, A9's R9 hole, and the harness gap); thirteen
confirmed and queued; seven already decided against with the reasoning quoted;
**eight of the reviewer's statements are factually wrong** and are recorded as
such so they do not become facts by repetition.

★★★ **Five items are amendments to `RIBBON_IA.md`, which is settled and yours to
rule on — none has been actioned.** The sharpest is **A10**: the review wants
the four page-display buttons labelled, `RIBBON_SCALING.md` made them icon-only
on a measurement, and **`RIBBON_IA.md:143-147` already argues the reviewer's
side** — the two documents were never reconciled and the later one won by being
later. That one wants a decision from you.

★★ **The mockup's ribbon does not load.** It puts Fonts and Comments on two tabs
each, and `no_command_appears_twice_on_the_tabs` refuses it. Most of the mock is
the shipped manifest re-drawn; the genuinely new parts are a left icon rail
(~1 week, and gated behind the R128 fit-zoom cache), a bundled typeface, a
palette, and an 11 pt type floor.

★ **The 65 proposed icons are good art with a false delivery claim** — not the
asset format, six draw the wrong picture because the parser ignores
`stroke-dasharray`, two fail a closed-set test, all 65 lack the rationale
comment, and one is art for a command deleted six weeks ago. A day or two, not a
copy.

★ One is already answered by inspection and is recorded here so it is not
re-litigated: the **Keyboard shortcuts list does scroll** — 34 commands, 0
dropped, inside a `ScrollArea::vertical`. What is missing is the *affordance*:
egui's default scroll style is `floating()`, a 2 pt sliver that fades when the
pointer is elsewhere, so a capture shows fifteen rows and no bar. The Print
dialog already carries the fix and the reasoning (`ScrollStyle::solid` +
`foreground_color`); **4 of 37 `ScrollArea` uses in this crate have it.**

---

## O113 — ◑ **OPEN, 2026-09-03** — the clipping hatch should cover only what actually falls outside the printable area

**Ken, 2026-09-03:** *"also can you make it so the red pattern you put over the
page if it is going to print beyond the printable borders is only over the areas
that extend beyond the printable page? Our drawing get drawn 1:1 and the area
that isn't printed is just empty border."*

The preview hatches to say *"this will be clipped"*, and it hatches **the whole
overhanging region** rather than the part of it that carries ink. On his
drawings that is almost always wrong in the direction that matters: a CAD sheet
printed 1:1 overhangs by a margin of **empty paper**, so the hatch shouts about
losing something when nothing is being lost.

★ **This is a disclosure that is technically true and practically false**, which
is the worst kind. An operator who sees the same red band on every 1:1 drawing
learns to ignore it, and then does not see it on the one sheet where the border
really does have a title block in it. A warning that fires on the common
harmless case trains the operator out of reading it.

★★ **It is a rule 4 question and it should be checked against rule 4 before it
is built.** The hatch is drawn on the *preview*, not on the page and not on the
saved document, so it is disclosure rather than content marking and it is
allowed to exist. What is at issue is only its accuracy. But the fix must not
drift into marking: whatever is drawn stays inside the preview canvas, and the
page as rendered for printing is unchanged.

### What it needs, and the honest difficulty

The question *"is there ink in this region of this page?"* is not one the
current plan answers. `Job::clipped()` counts sheets whose page box exceeds the
printable rectangle — a **geometric** test on the `/MediaBox` against the
device, which is why it is cheap and why it is wrong here.

Answering it properly means asking what the page's **content** bounding box is,
which is `pdfcer-core`'s question rather than this shell's. Likely shapes:

- an engine verb returning a page's ink extent (a real content bbox, not the
  `/MediaBox`), which is the honest general answer and is a request to file;
- or, entirely within the shell, sampling the preview raster we have **already
  rendered** for the overhanging band and hatching only where it is not blank.
  Cheaper, needs no engine change, and is approximate at the edges.

⬜ **Not started, and not guessed at.** The second option is tempting because it
needs nobody else, and it answers *"is this band blank in the raster we drew"*
rather than *"is this band blank"* — which is the same class of proxy this
project has now been caught by three times in one day. Decide deliberately.

---

## O112 — ◑ **HALF DONE 2026-09-03** — the print preview should be resizable, and poppable into its own window

**Ken, 2026-09-03:** *"also the preview should be adjustable size, and even
better if it has the option to pop out into its own resizeable window - closing
the window pops it back into place on the print window."*

Two asks, and the second is the interesting one. Taken together with O111 below,
because they are the same surface.

1. **Adjustable in place** — the preview column is a hard-coded 340 pt
   (`preview::COLUMN_WIDTH_PTS`) and the sheet is drawn to fit inside it. On a
   1300 pt-wide dialog that leaves most of the window empty while the preview
   stays postage-stamp sized. It needs a draggable splitter between the two
   columns, and the preview must re-fit to whatever width it is given.
2. **Poppable into its own OS window**, and closing that window returns it to
   the print dialog.

★ The second is already structurally cheap and that is worth recording: every
dialog in this shell is a real OS window through `dialogs::host::Host`, which is
keyed on a name string, remembers its own position and reports its own close.
A popped-out preview is a second `Host` keyed `print-preview`. **The close is
already the return path** — `Frame::closed` is what the host reports, so
"closing the window pops it back" is the default behaviour rather than a feature
to build.

★ **What it must NOT do (R9):** while the preview is popped out, the print
dialog's preview column renders **nothing** — not a greyed rectangle, not a
"preview is in another window" placeholder box occupying the same space. The
column collapses and the options take the room. A stub is the thing this
project's no-placeholders rule exists to forbid.

### ✅ Ask 1 done — the preview is draggable

A splitter between the two columns. Drag it; double-click restores the default.
The preview re-fits to whatever width it is given, because `fit` was already
recomputed from the current rect every frame — that coupling was the *desired*
half of the feedback relationship the preview documents, so widening the column
simply shows a bigger sheet.

- Floors at both ends so neither column can be squeezed out of existence
  (220 pt preview, 400 pt options). Below their sum the body scrolls
  horizontally, which is the one case where scrolling is the right answer.
- **The width you choose survives a resize.** The first version wrote the
  clamped value back, so narrowing the dialog to 540 pt destroyed your setting
  and widening it again left the preview at the floor. It is a *preference*
  now, clamped only for layout.
- The affordance is a **cursor** and a hover-lift on the divider. Nothing is
  drawn on the previewed sheet — rule 4's pre-commit clause.

### ⬜ Ask 2 not started — the pop-out window

Still the right shape and still cheap: a second `Host` keyed `print-preview`,
with `Frame::closed` already being the return path. Deferred deliberately rather
than rushed in behind four defect fixes in the same file, which is now split
across `print/mod.rs` and `print/layout.rs`.

### Was blocked behind O111 on purpose

O111's defects are in the same layout code — the forced content width, the
missing margin and the fixed column widths are all in `PrintDialog::body`. Doing
this first would mean writing a splitter into a layout that is already
mis-measuring itself, and then not being able to tell which change fixed what.

---

## O111 — ✅ **FIXED 2026-09-03** — the print dialog: two permanent scrollbars, it will not close, and Print looks broken

**Ken, 2026-09-03:** *"I thought the print dialogue box had been fixed by
replacing with a more standard window one but with all our current acrobat style
controls and commands untouched. Instead I have two scroll bars in the pop up
window that won't go away no matter how, and it doesn't close after I hit the
print button that is so far off in the corner it is touching the edge the
window, and it looks greyed out as though it it doesn't do anything even when I
hit print - but it is working, so after many clicks I checked the printer and of
course there was a dozen jobs there because the button just looks greyed out and
broken."*

**★★★ Four separate causes, not one.** Reproduced offscreen on
`fixtures/a1-titleblock.pdf` and photographed at five window sizes; every
symptom is visible in the captures.

| # | symptom | cause |
|---|---|---|
| 1 | two scrollbars that never go away | the body forces its content to `max(764, available_width)`, so at any width over 764 the content is exactly as wide as the viewport **before** the scrollbar is subtracted. The vertical bar takes 10 pt, the content is then wider than what is left, the horizontal bar appears, that takes 10 pt of height, and the two keep each other alive |
| 2 | does not close after Print | by construction. `show` returns `!frame.closed && !close_requested`, and `close_requested` is set only by Cancel and the OS close button. Pressing Print prints and leaves the window open |
| 3 | the button touches the window edge | the dialog body is drawn into the viewport's **root `Ui`** with no frame and no inner margin. The trace shows content at `y = 0.0`. **True of all fourteen dialogs**, not just Print |
| 4 | Print looks greyed out | `Host::buttons` fills the affirmative with `visuals.selection.bg_fill`, which this theme sets to the **27 %-opacity canvas object-selection tint**. Over the panel it composites to a pale blue-grey — *paler than the ordinary Close button beside it*. This is defect **D2's exact shape**, in the one place the earlier fix did not reach |
| — | a dozen queued jobs | the consequence of 2 and 4 together: the button looks dead, the window does not close, so it gets pressed again |

### ✅ What was done

| # | fix |
|---|---|
| 1 | every width and height in the body is now derived from the space **outside** the scroll area and from constants — never measured inside it — and `auto_shrink` is `[true, true]`. Driven by `print_dialog_body_does_not_deadlock_its_scrollbars`, which reads egui's own `content_size` and `inner_rect` out of a running frame |
| 2 | a **successful** print records its receipt on the application's disclosure row and closes the window. A **failed** one does not close, because the driver's words and the settings that produced them are what the operator needs next |
| 3 | `Host::BODY_MARGIN_PTS` — 12 pt, applied once in the host, so **all fourteen dialogs** gain it rather than each remembering |
| 4 | `Theme::accent_pair` — one named accessor for *"paint this as the emphasised action"*, replacing the translucent `selection.bg_fill` in every dialog's affirmative button |

**★★★ The scrollbar defect took FOUR fixes, not one, and each wrong answer read
as obviously correct in the source.** In order: the content was sized from the
width *outside* the scroll area; then `auto_shrink([false, false])`, which
*defines* the content to be at least the pre-bar viewport; then the two
`item_spacing` gaps `horizontal_top` inserts between three children; then the
preview's control strip, measured at **379.9 pt laid out in a 340 pt column** —
it had overflowed since the day it was written, hidden by the forced content
width. Only the third and fourth were found by instrumenting the running
process; nothing in the source suggested either.

★ **A first attempt at the third fix was worse than the defect.** Setting
`item_spacing.x = 0` removed the bar and inherited into every child, so the
radio rows lost their spacing — *"Subset ●Every page ○Odd only"*. It was visible
in the very next capture. The gaps are in the arithmetic now, read from the
style rather than assumed to be 8, because this shell's gutter differs per theme
preset.

★★ **And the fourth fix is a wrapped row, not a wider minimum.** A minimum would
be a constant asserting how wide seven buttons are — which depends on the
preset's font and button padding, so it would be right in one theme and wrong in
another. `horizontal_wrapped` is bounded by its column **by construction**:
there is no number to get wrong.

### ✅ Verified, and how

- **Driven and falsified.** `print_dialog_body_does_not_deadlock_its_scrollbars`
  fails when the original width defect is planted back in (content 784 pt in a
  776 pt viewport) and passes when it is removed.
- **Photographed at five window sizes**, before and after. The before-captures
  show both bars at 1000 × 760 and 1300 × 900 and the clipped-with-no-bar case
  at 700 × 520; the after-captures show no bar where nothing needs scrolling and
  a bar where something does.
- **Print is now in `dialogs_open_in_their_own_window`'s list**, which it had
  never been in — see below.
- Three unit tests on the close-and-report decision, extracted into
  `commit_notes` so that proving the window closes does not require putting a
  job on your printer.

### ★★ The gap that let all four ship, recorded because it is the third of its kind

`dialogs_open_in_their_own_window` sweeps every command-reachable dialog from a
**hand-written list, and Print was not in it** — the dialog whose report started
that entire piece of work. The header rationalised the omission as *"Print was
fixed that evening and `print_dialog` asserts it"*, and `print_dialog` asserts
the job reaches the **spooler**, which is not a claim about the window.

**A hand-written list inside a completeness sweep is the gap.** Print is now the
first entry.

★ And a unit test named `the_body_width_holds_both_columns` was **green
throughout**, asserting a relationship between our own constants. A scrollbar
appears when content exceeds egui's **viewport**, which is smaller than anything
those constants describe and does not exist until a frame is laid out. It was a
test of the wrong quantity; it is retired, with the reason kept where it stood.

**★★ The scrollbars are INVERTED, which the size walk found and a single
screenshot would not have.** At 1000 × 760 and 1300 × 900 both bars are present
with two thirds of the window empty. At 700 × 520 the options column is clipped
below "Landscape" — the whole Paper section unreachable — with **no vertical bar
at all**. Bars when nothing needs scrolling; no bar when content is genuinely
cut off.

**★★ Why no gate or check caught any of it.** `check-theme-colors.sh` passes
because `Host::buttons` uses a theme *role* rather than a literal — the rule it
enforces is "no raw `Color32`", and this is a correctly-sourced colour used for
the wrong thing. The contrast test measures widget-state pairs and never sees
this ad-hoc pairing. And `dialogs_open_in_their_own_window` sweeps eight
command-reachable dialogs from a **hand-written list that does not include
Print** — the dialog whose report started that whole piece of work.

---

## O110 — ✅ **RELEASED 2026-09-03 (afternoon)** — release and publish the package on GitHub

**Ken, 2026-09-03:** *"please release and publish the package on github."*

**Release:** <https://github.com/KenM76/pdfcer-gui/releases/tag/v0.5.0>, with
`pdfcer-gui-v0.5.0-windows-x64.zip` (22.0 MB) attached. `main` pushed
`3fb06dd` → `09bb966`, and **all five historical tags pushed to the new
remote** — `v0.1.0` through `v0.4.0` had never existed there, because
`pdfcer-gui` is a *new* repository rather than a rename of `pdfceGUI`, so its  <!-- old-name-exempt: a record of the rename must name what was renamed -->
releases page was empty.

### What the release turned out to require first

The request was "release", and the tree was **red** when it was made — which is
the whole reason a release is not a single command.

★★★ **The engine's `Pass 247.1` (its own `pdfce` → `pdfcer` rename) landed  <!-- old-name-exempt: a record of the rename must name what was renamed -->
mid-session.** Our three dependency lines carried a temporary
`package = "pdfce-*"` bridge and a tripwire gate written to fail the build the  <!-- old-name-exempt: a record of the rename must name what was renamed -->
moment that bridge outlived its cause. It fired. The bridge came out, the call
site became `is_pdfcer_choice`, and the gate **deleted itself** as its own
header instructed. Engine locked at `562ca7e`, v0.28.0.

★★★ **And that tripwire's condition was wrong the same way twice.** It tested
the engine's *working tree* where this build reads its *committed history*, and
for about an hour the engine held **795 staged-but-uncommitted renames** — so it
failed the build for a false reason and instructed a fix that would not have
resolved. Its own header already recorded catching itself using a proxy once.
**A proxy condition survives one correction.**

★★★ **The rename had blinded the falsification harness, silently.**
`ui-verify`'s profile for the OLD GUI — the build the checks must be seen to
FAIL against — had all four of its external names swept to the new spelling.
Three of the four fail *quietly*: an env var the old binary does not read leaves
its diagnostics off, and a trace prefix it never prints parses to an empty
trace. The suite would have said *"the old build does not exhibit the defect"*.
Repaired, and held by two falsified tests rather than a comment.

★★ **The two-slot OneDrive fallback was broken and the tool said it was fine.**
The rename moved the slots to `pdfcer-gui1`/`2`; the previous builds were in
`pdfceGUI1`/`2`. The first package wrote into an **empty pair** while printing  <!-- old-name-exempt: a record of the rename must name what was renamed -->
its usual *"the other slot still holds the previous build"*. Repaired by hand —
`pdfcer-gui1` is the new build, `pdfcer-gui2` is the 08:26 one. ⚠ **`pdfceGUI1`,  <!-- old-name-exempt: a record of the rename must name what was renamed -->
`pdfceGUI2` and two `.pdfceGUI*-outgoing` folders are now orphans in your  <!-- old-name-exempt: a record of the rename must name what was renamed -->
OneDrive and are yours to delete.**

★ **`FEATURES.md` re-measured against the build**, ninth revision, before
packaging. Its Source row had read "~215,000 lines" and was **wrong rather than
stale** — the tracked Rust is 60,692 lines. The old figure named no method,
which is why nobody could check it; the row now carries the command.

★ Checked before pushing to a **public** repository, as for O109: the licensed
print-conformance suite's name is absent (gate green), and a sweep for keys,
tokens and private material found nothing outside fixtures and prose.

### ⬜ What is NOT claimed

**The 152-check driven suite was not swept against this build** — the machine
was in use. Unit tests (2,881) and gates (23 of 23) were run against the exact
lock the binary links. That disclosure is in the release notes and in the
shipped `BUILD-INFO.txt`, not only here.

---

## O109 — ✅ **RELEASED 2026-09-03** — release the latest source and exe to GitHub

**Ken, 2026-09-03:** *"And while you are doing that release the latest source and
exe to github!"*

`KenM76/pdfcer-gui` is **public** and was last pushed **2026-08-24** — **345
commits** behind. The last GitHub release is **v0.3.0** (2026-08-15), which
predates forms, attachments, multi-document, the measure rebuild and everything
in between.

★ **One fact that has to travel with the source**, because it is not a defect and
would read as one: `crates/pdfcer-gui/Cargo.toml` takes the engine as
`git = "file:///D:/Dev/pdfcer"`. **Nobody but you can build the published
source**, because that URL is a path on this machine. Pointing it at
`https://github.com/KenM76/pdfcer` would fix it and is a decision about what this
repository is for, not one to make while releasing.

### ✅ Done

**Source:** `main` pushed — 346 commits, `d4d8d7f` → `eed8d3e`.

**Release:** <https://github.com/KenM76/pdfcer-gui/releases/tag/v0.4.0>, with
`pdfcer-gui-v0.4.0-windows-x64.zip` (21.7 MB) attached — the portable build,
unzip and run.

★★ **It was repackaged first, and that mattered.** The build published to
OneDrive an hour earlier carried *"THE SHELL WAS BUILT FROM AN UNCOMMITTED
WORKING TREE"* in its own `BUILD-INFO.txt`, because the packager runs
`cargo update` and had built before the lock was committed. Shipping that to
GitHub would have put a binary nobody could tie to a commit behind a version
tag. The released build is stamped `eed8d3e` on engine `a436432`, and
`pdfcer-gui2` now holds the same build.

★ **The crate version stayed 0.1.0** rather than being bumped to match the tag,
and that is deliberate: `crates/pdfcer-gui/Cargo.toml` carries a comment saying
the crate is versioned by the pdfcer workspace it folds **into**, not by this
staging workspace, and `version.workspace = true` replaces the line at fold-in.
Bumping it would have contradicted a recorded decision to make two numbers
agree that are not the same number.

★ Checked before pushing to a **public** repository: the licensed
print-conformance suite's name is absent (its gate is green), and a sweep for
keys, tokens and private material found only test fixtures and prose.

---

## O108 — ◑ **AUDITED 2026-09-03, and the answer changes the ask** — one ribbon tab for every encryption and signature feature

**Ken, 2026-09-03:** *"can we get all of the encryption and signature features
that have been implemented in the engine under one new tab in the ribbon?"*

### What this is asking for, stated before any of it is built

Two things, and the second is the harder one:

1. **A new ribbon tab** collecting encryption and signatures in one place.
   `RIBBON_IA.md` places them nowhere as a group today — encryption is not in
   the IA at all and signatures exist only as a **panel**.
2. **"All of the features that have been implemented in the engine"** — which is
   a completeness claim, and this project has a standing rule about those: *a
   completeness question needs an instrument, not a document.* Our own files
   (`FEATURES.md`, `NO_SURFACE.md`, `GUI_ROADMAP.md`) are structurally unable to
   answer it, because none of them is keyed on `pdfcer-core`'s API. The audit has
   to enumerate from the **engine's** side.

★ `RIBBON_IA.md` is settled and normally not improvised around. A new tab is an
**operator decision**, so this supersedes it — and the amendment gets written
into that document rather than left as a divergence.

### ★★★ THE AUDIT, and the answer is not the one the question expects

`tools/security-coverage.py` is the instrument — new today, keyed on
`pdfcer-core`'s **own** API rather than on any document of ours, and reading the
revision `Cargo.lock` pins rather than the engine's working tree. Measured at
lock `a436432`:

> **61 public items: 12 reached, 31 NOT reached, 7 engine-internal, 11 too
> generic for a grep to attribute.**

★★★ **Every one of them is READ-SIDE.** `pdfcer-core` has **no**
`encrypt_document`, **no** `set_password`, **no** `remove_encryption`, **no**
`set_permissions`, **no** `sign_document`, no certificate validation and no
timestamping. What the engine has is the ability to **open** an encrypted
document, **report** its scheme and permission bits, and **count and describe**
the signatures a document already carries.

⇒ So a tab can be built and it will be an **information** tab, not a *Protect*
tab. That is worth saying plainly before it is drawn, because a row of controls
that cannot work is R9's placeholder rule broken at the scale of a whole
surface — and because the gap between what you asked for and what exists is a
request to the engine, not something to paper over.

### ★★★ And the audit found something worse than a missing tab

**An encrypted PDF cannot be opened at all.**

The shell detects the case correctly — `Status::NeedsPassword`, with a tab
tooltip reading *"This document is encrypted and pdfcer has not been given the
password"* — and then **there is no way to give it one.**
`Document::load_with_password` and `from_bytes_with_password` are named in
exactly one place in this crate: a doc comment in `app::blank` listing the four
loading entry points. Nothing calls either.

★★ That is why the coverage tool now strips comment-only lines before it
searches. Its first run reported `load_with_password` as **reached**, on the
strength of that one sentence — which would have recorded the single most
important missing capability in this whole area as already built.

### What is reached, and what is not

| | |
|---|---|
| **Signatures — mostly reached.** | `SignatureImpact`, `ImpactBasis`, `SaveMode` and `documentation_basis` drive the save-time warning; `byte_range_coverage` and `covers_to_eof` drive the Signatures panel. `SignatureCensus`, `forbids_structural_change` and `impact_of` are named nowhere |
| **Encryption — almost nothing.** | `Document::encryption()` is read in **one** place, as a boolean, in the Properties panel. `Permissions`, `PermissionBit`, `AuthKind`, `Cipher`, `strings_encrypted`, `streams_encrypted`, `password_may_need_normalisation`, `authenticate` — none reached. So the document's own permission bits, which say whether it forbids printing, copying, or changing, are **invisible** |

### The plan, in the order the value lands

1. **A password prompt**, so an encrypted document opens. Nothing else on this
   list matters if the file cannot be read.
2. **A Security tab** collecting: the encryption state (scheme, key length,
   whether you authenticated as user or owner), the **permission bits** in
   plain words, the signature census, and the byte-range coverage — the four
   questions an operator actually asks of a protected file.
3. **A request to the engine** for the authoring half: encrypt with a password,
   set permissions, remove encryption, sign. It has none of it, and this shell
   must not invent it. ✅ **Filed 2026-09-03**, as two files because they are two
   topics: `request_a_document_cannot_be_encrypted_or_have_its_permissions_set.md`
   and `request_a_document_cannot_be_signed.md`.

   ★★ The signature request asks for **validation before signing**, and says so:
   *"the bytes under this signature have not been altered, and nothing was
   appended after it"* is a real answer that needs no certificate store, and it
   is the question an operator receiving a signed drawing actually asks. Signing
   needs a key, a certificate, a store and a decision about PAdES levels, and
   nobody has asked for it by name.

★ `RIBBON_IA.md` gets the amendment written into it rather than left as a
divergence, because a tab that exists and is not in the IA is how two
information architectures start.

---



## O105 — ✅ **DRIVEN 2026-09-03** — the radius/diameter tool fits the wrong thing

**Ken, 2026-09-03:** *"can you check our radius/diameter dimensioning tool?
selecting a point sometimes makes a big circle, and selecting more points around
a hole doesn't always get it to narrow down to the size of the hole."*

### ★★★ The cause, and it is one line of code with a number beside it

**The tool does not pick points. It picks whole PDF path objects**, and feeds
*every anchor of every subpath of that object* to the circle fit.

`canvas::measure::circular::click` hit-tests for an object, then calls
`ObjectModelProvider::object_sample_points`, which is:

```rust
Some(VectorObject::Path(path)) => path
    .page_subpaths()
    .iter()
    .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
    .collect(),
```

★★ **The size of that set is already measured, in this repository, thirty lines
below the function that produces it.** Decision 028's note on the Objects panel:
*"on a measured CAD export one path object holds **6,681 anchors**"*. So one
click on a hole in such a drawing hands Taubin's fit 6,681 points scattered
across the entire sheet, and the best-fit circle through them is enormous.

⇒ *"selecting a point sometimes makes a big circle"* — exactly. It is not
sometimes-random: it is whether the hole's arc happens to be its own small path
object or part of a large one, which the operator cannot see and has no reason
to think about.

### And the second half follows from the first

*"selecting more points around a hole doesn't always narrow it down"* has two
causes, both from the same design:

1. **A second click on the same object toggles it OUT.** Clicking twice around
   one hole that is drawn as one object adds it and then removes it. Nothing on
   screen distinguishes that from "the click did nothing".
2. **A click on a different object adds another whole blob.** Adding a second
   large object makes the fit worse, not better — the opposite of what "add more
   points" means everywhere else.

### ⇒ The fix: the tool takes POINTS, which is what he is already doing

His own sentence — *"selecting more points around a hole"* — is the conventional
model and the one every drafting package uses: three points on an arc give the
arc. The machinery is already built and already used by the linear and perimeter
tools: `measure::resolve::Resolved` returns the snapped anchor under the pointer,
or the raw position when nothing is near. The circular tool is routed **around**
it deliberately (`measure::click` has a comment explaining why an object pick
should not go through point resolution), and that reasoning was correct for an
object pick and is what has to go.

★ The one thing lost is *"click the circle once and be done"*. That is worth
having back later at **subpath** granularity — a subpath is the drawn entity, an
object is not — but it is not what was asked for and is not being built in the
same change. Recorded here so it is a decision rather than an omission.

### ✅ Shipped, and the check was falsified in the direction that matters

A click is one point. Three points on an arc give the arc.

**Driven by `three_clicks_round_a_hole_measure_the_hole`**, on a fixture built
to carry your geometry — `fixtures/hole-in-a-big-object.pdf`, **one** path
object holding a 30 pt circle *and* forty unrelated segments across the page.
The check pins that fixture and ignores `--pdf`, because on a document whose
circles are their own objects the defect **cannot occur** and the broken build
would pass.

```
click 1 → action=add origin=node n=1 r=none
click 2 → action=add origin=node n=2 r=none
click 3 → action=add origin=node n=3 r=30.000 resid=0.0000
```

★★★ **Falsified**: with one stray point fed into the fit the same check reports
*"three clicks on the rim of a 30 pt hole fitted a circle of radius **299.78**"*
— which is your sentence, as a number.

★★ **You can also now watch it converge.** The Tool panel's live line reports
the count and the current radius or diameter, through the engine's own
formatter and the authoring group's scale — so it reads in the same units the
placed dimension will. There was no number to watch before: the circle was drawn
on the canvas and its value appeared only once the dimension had been placed, so
every correction was a commit and an undo.

---

## O106 — ◑ **BUILT 2026-09-03, driven only in unit tests** — a click with nothing under it should still be a point

**Ken, 2026-09-03:** *"also we should be able to click and it selects a position
on a page if there is no point to select under the cursor (so we can measure off
bitmaps too)."*

Falls out of O105's fix and is listed separately because it is a separate
promise: on a scanned or raster drawing there are no anchors to snap to, so
every pick is a free position and the tool must still work. `resolve::snapped`
already *returns the raw point unchanged when nothing is near* — its own doc
says so — so what is needed is for the circular tool to use that path at all.

★ It is also the honest disclosure boundary: a free position is the operator's
own judgement of where the edge is, and the residual the fit reports is the only
thing that says how well they did. That number already exists in the preview.

### ◑ Built, and the row stays open because the DRIVEN half is not

The tool takes `resolve::snapped`'s answer: the anchor under the pointer, or the
raw pointer when nothing is within the catch radius. Unit tests cover the
composition (`a_pick_with_no_snap_candidate_is_recorded_as_a_free_position`,
`three_free_positions_on_a_raster_still_produce_a_circle`).

★ **What is NOT driven is a click on an actual raster.** The circle check's
fixture is vector geometry and every rim click correctly snaps, so it exercises
the snapped path only. Driving the free path needs a raster fixture and a click
where the snap declines — a different check, named here rather than left
implied, and it is why this row is ◑ and not ✅.

★★ The Tool panel's list says **Free position** on any point that came from one,
so when you measure off a scan you can see which of your points were judgement
and which were geometry. That disclosure is off-canvas on purpose: nothing marks
a free pick on the page, because applied content must render exactly as saved
content will.

---

## O107 — ✅ **DRIVEN 2026-09-03** — see what is in the pick set, and take things out of it

**Ken, 2026-09-03:** *"also we should be able to unselect points/clicked
locations, and it should have a box in the side panel showing what is part of
our selection and we should be able to delete included points/locations from
there - clicking on a point or location listed should allow us to remove it."*

Two routes to one capability, and both are wanted:

| route | what it does |
|---|---|
| **the canvas** | clicking a point already in the set takes it out again |
| **the Tool panel** | a list of the picked points, each removable by clicking it |

★★ The panel half is the one that cannot be substituted. A pick set on a dense
CAD sheet is invisible — the operator cannot tell four picked points from five,
and cannot tell *which* four. A list is the only surface that answers *"what is
actually in this fit?"*, which is the question behind O105's whole report: he
could not see that his click had contributed six thousand points.

`panels::tool` is the home — it already shows the perimeter tool's live
measurement, so a picked-set list for the circular tool is the same kind of
thing in the same place.

### ✅ Both routes shipped and both driven

The Tool panel grows a **Points in this measurement** section: one row per
point, numbered, saying where it came from and where it is. Clicking a row takes
that point out; hovering says so first, because the removal does not go through
undo (a pick set is pre-commit state and never enters the document's history).
On the canvas, clicking within the snap radius of a point already picked removes
it.

Asserted inside `three_clicks_round_a_hole_measure_the_hole`:

```
the panel lists 3 rows, one per point
clicking `tool.measure_point.0` removed that point: n=2
clicking the canvas on a point already picked removed it: n=1
```

★★★ **Falsified** by making the row drawn-and-inert: the check then reports
*"the row is drawn and inert — which is exactly the placeholder R9 forbids"*.

---


## ★★★ THREE OF YOUR 2026-09-01 REQUESTS WERE BUILT WITHOUT EVER BEING WRITTEN DOWN

**Found 2026-09-01 while writing the handoff, by grepping this file for your own
words and getting zero hits.** O92, O93 and O94 below were all asked for, all
worked on, and **none of them were entered here** — which is rule 1 of the
contract at the top of this file, the one you set the file up to enforce.

★★ **The work was done, so nothing looked wrong.** That is the whole failure
mode: a row is not a to-do list, it is the *record*, and the record is what
survives a session ending. Three requests existed only in a chat transcript and
in commit messages, where you cannot see them and a cold session does not look.

⇒ **Write the row when he speaks, not when the work lands.** The rows below are
back-filled, and are marked as back-filled so the dates are not read as evidence
of a process that worked.

## O104 — ⬜ **INVESTIGATED 2026-09-03** — a selection cannot be narrowed once it is made

**Ken, 2026-09-03:** *"also I can't unselect things once I have selected them
for redaction."*

### What we measured, before proposing anything

| route | what it does today |
|---|---|
| **Shift + click** an already-selected object | **toggles it out.** This works. |
| **Shift + drag a band** over a selection | **only ADDS.** There is no way to subtract with a band. |
| **Ctrl** anything | **nothing at all.** `canvas::interact` reads `i.modifiers.shift` and no other modifier. |
| a placed `/Redact` mark | selectable on the canvas, and Delete routes to the engine's `delete_redaction_mark`; the Redact panel also lists every mark with a **Remove** button |

### ★★★ So the capability exists and the ROUTE HE REACHED FOR DOES NOT

Two things are wrong and they compound:

1. **Ctrl does nothing.** He is a SolidWorks user; in SolidWorks, in Windows
   Explorer, and in almost every list on this operating system, **Ctrl+click is
   the toggle**. Reaching for it and getting a fresh single selection instead —
   which is what a plain click does — reads exactly as *"I can't unselect
   things"*.
2. **A band can only add.** On a CAD sheet with hundreds of overlapping strokes,
   picking one object out of a selection by clicking it precisely is often not
   practical; a band is how you work. Ours has no subtract.

⇒ **This is a discoverability failure first and a capability gap second**, and
the standing rule is to fix the route that failed him *and* ship the literal
ask. See the plan below the O103 row.

---

## O103 — ✅ **FIXED BY THE ENGINE 2026-09-03, our half rebuilt the same day** — redaction refuses any region that touches an image

**Ken, 2026-09-03:** *"every time I've tried the redact feature it tells me it
can't because there is objects that weren't redacted."*

**Reproduced with `pdfcer` alone, so it is not ours.** He supplied eleven
drawings in `OneDrive\pdfTests\Redact\`. Nine redact cleanly. The two carrying
images refuse the moment the marked rectangle **touches** one:

> `redaction refused: redaction region on page 1 intersects an image; pdfcer
> cannot yet destroy image pixels (clipping or masking would leave them
> recoverable, ISO 32000-1 §12.5.6.23) — apply refused rather than producing a
> false redaction`

★★ **The gate is the mark's RECTANGLE, not the pixels it would remove**, and
that is what makes it bite so hard on his documents:

* **"Mark whole page" can never be applied** on any sheet with a logo or a
  scanned stamp — one of our three marking routes is dead on this class of file.
* A title-block value inches from a logo clips its bounding box and refuses.
* It is **all-or-nothing for the document**: twelve good regions and one that
  grazes a logo redacts nothing.

⇒ Which is exactly *"every time I've tried"*. He was not doing anything unusual.

### What we asked for, in preference order

`request_redaction_refuses_any_region_that_touches_an_image.md` — (1) gate on
the pixels rather than the bounding boxes; (2) **remove an image that is wholly
covered**, which needs no pixel surgery and is the common "redact this logo"
case; (3) **refuse per region, not per document**, so twelve good marks apply
and the one that cannot is disclosed as a residual like every other carrier.

★ (3) is the one that unblocks him today even if the others are expensive.

### ✅ Our half is BUILT and DRIVEN — disclosure, not a workaround

Warning at **mark** time when a region covers an image, so it arrives when the
rectangle is drawn rather than after twelve marks and an Apply. The content
still cannot be redacted; you just find out while redrawing it is one gesture.
Both routes: marking a selection, and marking the whole page.

★★ **It blocks nothing.** The mark is authored exactly as before — a mark is
reversible and costs nothing, and if the engine gains the capability the same
mark applies cleanly. The success is said first, with the caveat beside it.

★★★ **Driven, and falsified BOTH WAYS** —
`marking_over_an_image_says_so_before_apply`. It asserts the warning appears on
a document that is nothing but a raster image **and** that a CAD sheet with no
image is marked in silence. A check with only the first half passes just as
happily on a build that warns about every mark on every document, which is worse
than no warning: a caveat attached to everything is one you learn to scroll past,
and the day it matters you scroll past it too. Forcing `images = 0` reddens the
first half; forcing `images = 1` reddens the second.

### ✅✅ THE ENGINE ANSWERED THE SAME DAY, AND WENT PAST WHAT WE ASKED

**Two releases in one afternoon.** `pdfcer-core` **v0.26.0** (`Pass 245.0`) then
**v0.27.0** (`Pass 246.0`). All three of our asks, in the order we ranked them,
plus one we had not thought to make:

| what | now |
|---|---|
| the gate | on the image **samples**, not the bounding boxes — a region that merely touches a logo's rectangle destroys nothing and is not an image redaction |
| a covered image | the pixels are **destroyed** — decoded, overwritten, the matching part of any soft mask cleared, re-encoded losslessly. A wholly covered image is removed outright |
| an image it cannot decode | **that mark alone** is left unapplied; every other mark applies. Only when *no* mark can be applied does the whole thing refuse |
| ★ vector lines through a region | **cut at the region boundary** — strokes cut against the region widened by their stroke width, fills clipped to its complement, a path wholly inside deleted. **We never reported this and it is the bigger one on a drawing.** The engine found it by rendering before and after |

**Measured on your own files** by the engine: the corner mark on `17036-15`
cuts 25 path objects with zero residual and exits 0; the whole-page marks on
both drawings drop 780 and 1,089 objects and exit 0 — the runs that used to
refuse.

★ **And the black block is gone**: destroyed cells are now **paper**, not black
— white for grey/RGB, no ink for CMYK — so a mark with no fill colour leaves the
image area looking like the page behind it.

### ✅ Our half, rebuilt against it

* The mark-time sentence now reads **"This region covers part of N image(s) —
  those pixels will be destroyed, not hidden."** It used to say the apply would
  be refused, which became the exact opposite of the truth within hours.
* The report says what happened to the images — cleared, removed, and whether a
  rotated placement had a slightly larger rectangle cleared than you drew.
* A **shared** image gets its own line, because "I redacted the logo" and "the
  logo is gone from this file" are different claims: the other pages keep theirs,
  since you did not mark those.
* The report says how much **drawn geometry was cut**, and how much was deleted
  outright.
* Three new residuals go into the acknowledgement list: a **retained mark**
  (a region where nothing was removed), **geometry that could not be cut**, and
  a **clip whose outline had to be kept**.

★★★ **The one number that gates the word "redacted"** is `marks_retained`, and
the engine said so by name. A retained mark is a region where *nothing was
removed*, under a rectangle that says it was. It is now the first line of the
residual list.

---

## O102 — ✅ **DRIVEN 2026-09-02** — closing asks about unsaved work, document by document

**Ken, 2026-09-02:**

> *"also when I close the program it should prompt to save changes if there are
> any, and it should do what other programs do - switch focus to the document
> that is being prompted for, and cycle through each unsaved document while it
> prompts, but also have a save all button that saves all changed documents."*

**Not investigated.** Four separate requirements in one sentence, and they are
worth keeping separate because three of them are the ones that get skipped:

1. **Prompt on close when there is unsaved work.** The base case.
2. **★ Switch focus to the document being asked about.** A modal that says
   *"save changes?"* while showing a different document is asking about a file
   the operator cannot see. This is the one that makes the prompt trustworthy.
3. **★ Cycle through each unsaved document.** One prompt per dirty document, in
   turn — not one prompt for "some documents".
4. **★★ A Save all button**, which is what makes the cycle bearable. Without it
   an operator with six dirty documents answers six questions.

★ pdfcer is multi-document, so this is a real cycle rather than a single
question. `crate::dialogs::unsaved` already exists for the single-document case
(closing one tab); what is not known yet is whether the **application close**
path reaches it at all, and whether anything cycles.

### ★★★ What was there before: nothing

`dialogs::unsaved` has asked about **one** document since it was written — the
tab being closed — and asked it well: it focuses that tab first, it counts edits
from the last *save* rather than from zero, and it refuses to proceed on a save
that did not happen.

**None of it was reachable from the window's ✕.** eframe's close request was
never read, so pressing it — or `Alt+F4` — ended the process with every unsaved
document still unsaved. The only thing on the exit path was the layout flush.

⇒ The gap was not *"the prompt is wrong"*, it was *"there is no prompt"*, and it
was one keystroke away. ★ My own O80 check has been closing the program with
`Alt+F4` all day and never saw it, because that check opens a document and does
not edit it.

### ✅ Built 2026-09-02 — all four requirements

1. **Prompts on close** when anything is dirty. When nothing is, the close goes
   straight through — no dialog, no flicker.
2. **Focuses the document being asked about** before asking. A modal saying
   *"save changes?"* over a document you cannot see is asking about a file you
   have to guess at.
3. **Cycles**, leftmost tab first — which is the order you read them in.
4. **Save all**, labelled with the count (*"Save all 4"*), and drawn **only when
   more than one document is dirty**: with one it is the same act as Save, and a
   second button meaning the same thing is one you have to stop and think about
   while a modal stands between you and leaving.

**Cancel abandons the whole quit**, not one question — Word, VS Code and
Notepad++ all do that, and the alternative leaves some documents closed when you
asked for none of it.

### ★★ The cycle is derived, not remembered

One boolean; the queue is re-scanned from the document set every frame. A
remembered queue would be a second model of that set, and **every answer in the
cycle changes it** — a save cleans one, a discard closes one. Re-deriving cannot
drift.

★ Save all writes only documents that **have a file**. A never-saved one needs a
destination, which is a question only you can answer, so the cycle asks about
those individually afterwards — Word's behaviour.

★ One honest limit, stated rather than hidden: a document you already chose to
**discard** earlier in the cycle is already closed, and Cancel does not bring it
back. That matches every editor in the class — a discard is an answer, not a step
— but it is worth knowing.

### ⬜ NOT DRIVEN

Built, unit-tested and gated. Driving it needs a document edited, a close
requested and a modal answered — the pointer.

---

## O101 — ✅ **DRIVEN 2026-09-02** — the build time in the top bar

**Ken, 2026-09-02:** *"also in the next release add the local compilation time
to the top bar at the end of the date you added."*

### ✅ Built

The title now reads `… — 2026-09-02 06:25` where it read `… — 2026-09-02`.

★★ **He is closing a loop his own reports opened.** The date went into the title
on 2026-09-01 after he spent a morning reporting a defect that had been fixed,
against a build he did not know was old. **Two rows have now been closed by
"you were running an old build"** — O85 and O87 — and on a day with several
publishes the date alone cannot tell them apart. A date answers *is this
today's*; a date and a time answer *is this the one I just installed*, which is
the question that was actually being got wrong.

### ★★ The zone is shown only when it is NOT local, and that is the subtlety

`PDFCER_BUILD_TIME` has two producers and they disagree about zone:

| producer | stamp | zone |
|---|---|---|
| `package-portable.py` | `2026-09-02 06:25 +0100` | **local** — Python knows the offset |
| `build.rs` fallback | `2026-09-02 06:25 UTC` | UTC, and labelled so |

A packaged build's time is local, so the offset is noise to somebody standing in
that zone and is dropped. A dev build's is UTC, and showing `06:25` bare would
invite reading an hour that is not the wall clock — so `UTC` is kept.
`build.rs`'s own sentence is the rule: *a stamp that says the wrong hour is worse
than one that says a true hour in a named zone.*

★ That is also the assertion that would fail against the obvious simpler
implementation — truncate to sixteen characters and stop — and it has its own
test for exactly that reason.

★ Still derived by truncation from the one value with one producer, so the title
cannot disagree with what About shows.

### ⬜ NOT DRIVEN

Four unit tests over the rule; the title itself is a window property and wants a
driven check. Waiting on the pointer.

---

## O100 — ✅ **DRIVEN 2026-09-02; the preset half is ANSWERED and CONSUMED** — new colour-rendering options

**Ken, 2026-09-02:**

> *"the engine I think has a couple of new options for colour rendering that we
> might need to surface and set for our standards presets."*

**Right, and one of them was brand new.** `pdfcer-core 0.20` added
`Settings::spot_colorant_device_model` — *whether a spot ink keeps its own
printing plate, or is mixed down to process colour before anything is drawn.*

### ★★★ The gate that exists for this fired the moment the engine came forward

`every_setting_the_store_carries_has_a_control_in_this_window` enumerates the
engine's own `Settings::write_to_string` at runtime and demands a control for
each key. It was **green before the update and red immediately after**, naming
the setting and saying an operator could otherwise only reach it by hand-editing
`settings.txt`.

★ Worth recording because the same test's history is a list of times it caught
exactly this — three previous occasions are in its own comments — and because it
is the answer to *"how would we know?"*: not by reading a note, by linking the
crate.

★★ The **second** completeness test then fired for the copy: the window draws 29
settings and the catalog described 28. Two independent instruments, one demanding
the control and one demanding the sentences that go with it.

### ✅ Surfaced — Settings ▸ Colour, immediately after its sibling

Placed next to *Grey over a spot colour in print-ready files*, because the two
are one subject from two sides: **that** control is what *overprints* a spot
colour, **this** one is what a spot colour *is*. An operator arrives at either by
seeing white behave unexpectedly on a print-ready drawing, and finding only one
would leave them thinking they had exhausted the options.

★ Both values are **conformant** — this is a genuine choice, not a bug with a
switch:

| | renders for | white over a spot |
|---|---|---|
| *keep the ink on its own plate* (default) | a device that **has** the ink — §10.8.3 | leaves it showing, as a press does |
| *mix it down* | the **actual composite device** — §8.6.6.4's `shall` | knocks it out, as Acrobat's screen view does |

The copy leads with the **symptom** rather than the clause, because that is how
somebody gets here.

### ⬜ The preset half is the ENGINE's, and it is filed as a question

The standards presets are **not ours to set**: `Choice::apply` calls
`RenderPreset::for_standard(s).apply(settings)`, so the engine owns which axes a
standard pins — deliberately, and it is R8's registration rule reached through
the crate boundary, so a standard the engine adds appears here with no change at
all.

`RenderPreset` covers `page_blend_space_source` and does **not** cover the new
axis. ★ That may well be correct — the preset module's own reasoning notes that
*"a third of the grid is axes a standard does not reach"* — so it is filed as a
**question**, not a demand: *does a PDF/X or PDF/A level constrain the spot
colorant device model, and if so should the preset pin it?*

Filed as `done_2026-09-02-spot-device-model-REQUEST.md`.

### ✅ ANSWERED the same day, and the answer was the opposite of what we expected

**No PDF/X clause reaches the axis at all** — the vocabulary is absent from every
reachable line of ISO 15930 parts -1, -3, -4, -7 and -9. And **PDF/A's Scope
clause affirmatively excludes "operational details of rendering"**, in every part
from 2005 to 2020: a disclaimer, not a gap.

★★ The engine pinned it on PDF/X anyway, and the argument is worth keeping
because it generalises: *"the label creates an expectation, not because a clause
does."* The two values render **visibly differently**, and a control labelled
`ISO 15930-7` carries the promise *"show me what the press will get"* — so
leaving it alone would not decline to answer, it would silently ship whatever
global override was last set into a view read as authoritative. That is a rule-4
problem. Graded `implied`, never higher, and the sentence ends by saying no
clause requires it.

⇒ **On our side the row needed one thing and could not do it.** The engine asked
us to show the entry's `why`, because the divergence is invisible on the page and
reads as somebody's bug. `RenderPreset::disclosures()` emits a `why` **only for
keys a preset leaves alone** — so the reasoning behind every value a preset
actually *sets* never left the crate. Invisible until now, because until this
Pass every set value was `best-effort` and the count sentence covered those
fairly. A count is not a fair summary of a **claim**.

So the row now reads `entries()` directly and prints the `why` of every set entry
whose evidence is a claim about the standard — derived, never keyed on the axis,
bounded at two by a measured test, and carrying a tripwire that goes red if the
engine ever widens `disclosures()` so the workaround gets deleted rather than
duplicating their output. Reported back as
`note_disclosures_drops_the_why_of_every_value_a_preset_sets.md`, explicitly not
as a Pass request.

---

## O99 — ✅ **DRIVEN 2026-09-02** — the tab-order list drags, with the caret

**Ken, 2026-09-02:**

> *"the tab order list is supposed to be able to be reordered by dragging and
> dropping rows around like we can with pages in the page preview, and have clear
> markers of where the field is going to move to."*

**Not investigated.** Two named requirements: the drag itself, and **a clear
marker of where the row will land** — which he names by comparison with the page
rail, so that is the bar and the implementation to copy rather than reinvent.

★ The page rail already does drag-reorder with an insertion marker. Whatever it
uses is the thing to reuse; a second drop-marker implementation on one surface is
two behaviours to learn.

### ★★★ There is no verb that can commit it

Tab order on a page is the order of that page's **`/Annots` array** (§12.7.4.2).
Every annotation verb the engine has was checked:

| verb | what it does |
|---|---|
| `move_annotation(id, dx, dy)` | moves it **geometrically**. Not the array. |
| `delete_annotation` | removes it |
| `copy_annotations` / `cut_annotations` | the clipboard pair |
| `reorder_pages(&[usize])` | pages, not annotations — and exactly the shape needed |

**Nothing reorders `/Annots`, and there is no `/Tabs` setter either.**

### ★★ Why it was not decomposed out of what exists

The standing rule is to decompose rather than declare a blocker — three previous
"blockers" here were not real — so both candidates were considered and both are
destructive:

- **Cut and re-paste in the new order** destroys and re-creates the widgets: new
  object ids, a re-registration into `/AcroForm`, and the loss of `/AA` triggers
  and any `/Parent` chain a radio group depends on. Dragging a row down one place
  would quietly rebuild the form.
- **Delete and re-add** is the same, worse — `add_text_field` authors a *new*
  field from a spec, so everything not in that spec is gone.

A reorder must move the reference and touch nothing else, which is the shape
`reorder_pages` already has for pages.

### ⬜ Filed, and NOTHING was built

`request_a_pages_tab_order_cannot_be_changed_at_all.md`, asking for
`reorder_annotations(page, &[usize])` with `reorder_pages`' contract — plus the
one question that cannot be answered from outside: **should it also write
`/Tabs /A`**, so the order you arranged is the order the file *states* rather than
an order some readers happen to follow?

★★ **The drag gesture was deliberately not built** at filing time. R9 says an
unavailable capability renders **nothing**, and a drag with a drop marker that
cannot commit is a control that lies — worse than the read-only list that was
there. That was a change from how two earlier gaps were handled, where the shell
half was built and left waiting; it was the right call here because **the gesture
*is* the feature**. There is no useful half.

### ✅ The verb shipped the same day, and both halves are now built

`EditSession::reorder_annotations(page, &[ObjId])` landed hours after the request
went out. Wired, then the gesture, then the driven check — commits `df8e81d`,
`73e8436`, `6112f28`.

★ **Ids, not indices, and the engine asked for that by name.** Our sketch said
`&[usize]`; their reply said the index we hold is almost never a raw `/Annots`
index, because `page_annotations` skips null and non-dictionary entries. Worse
than they knew: the number our rows carry is `position`, which is 1-based, counts
**widgets only**, and skips entries with no id — wrong on three axes at once, and
on a well-formed single-purpose form all three cancel, so it would have worked on
every fixture we own and failed on real files. Rows now carry a `slot`.

★★★ **Widgets move among widget slots; nothing else moves.** `/Annots` order is
**paint** order, so a permutation that carried a widget past a `/Link` would
change what is drawn over what — a visible change to the page, from a gesture
whose whole subject was tab sequence. The engine's `non_widgets_moved` disclosure
is what revealed the trap; our route reports zero by construction and the driven
check asserts it.

⬜ **It does not write `/Tabs`, and the open question in the request is answered
— against our instinct.** We asked whether the reorder should write `/Tabs /A`
and said we thought yes. Sourced answer: **no**. `/A` is PDF 2.0 only, PDF/UA-1
§7.18.3 *requires* `/S` (Matterhorn 28-009 fires on anything else), nothing
requires a writer to state a tab order at all, and Acrobat's own manual tab order
is an `/Annots` permutation with no `/Tabs` written.

⇒ **The consequence we now own:** on a page that states `/S`, `/R` or `/C`, a
drag changes the array and a conforming reader may still tab in the stated order.
The per-page `/Tabs` sentence — unconditional, above the rows, on every page —
is what stops that reading as a bug in the drag. `set_page_tabs` is available for
the asking and is deliberately **not** asked for: the trigger would be an
operator reporting that an arranged order did not survive into another reader.

### ✅ Driven — and the feature was never the problem

`tab_order_drag_moves_a_field_and_shows_where`. The caret is drawn 73 pt wide
across the landing row, the release lands at gap 2, and the engine reports
`entries=2 moved=2 non_widgets=0 pinned=0` — so the two fields swapped and no
`/Link` or markup moved with them.

★★★ It took four wrong diagnoses to get there and **not one was a defect in the
drag**. The rows published a rectangle while scrolled out of view, so the pointer
landed 37 pt below the window and the check reported "the row does not sense a
drag" — about a row that senses fine. A ribbon press closed the panel it needed.
The wheel was eaten by a nested scroll area. And the real cause was that the
Forms pane is the bottom of three stacked panes, 396 pt tall, leaving about 42 pt
for rows about 30 pt each — the content had **nowhere to go**, so no amount of
scrolling could help. The check now drags the dock splitter, which is what an
operator does when a docked list is too short.

⇒ All four are written up in `D:/dev/rag/egui/`, because every one of them will
happen again to the next check written against a docked panel.

---

## O98 — ✅ **DRIVEN 2026-09-02** — the panel points, the canvas lights up

**Ken, 2026-09-02:**

> *"when we have the fill form panel visible and I click on fields in it instead
> it should highlight the field on the canvas that is being filled."*

**Not investigated.** The panel and the canvas are two views of one form and
today the panel does not say which box on the page it is filling — so on a
drawing with a dozen fields you fill one and cannot see where it went.

★ Note the direction: **panel → canvas**. The canvas → panel direction shipped as
O53 (clicking a field on the page selects it and fills the Properties panel).
This is the other half of the same relationship.

### ★★★ This was already written down as missing — and as permitted

The Forms panel's own header has carried the gap since the panel was written,
and it settles the rule-4 question in advance:

> *"the old shell's Forms panel did draw on the canvas: hovering a row
> highlighted the field's rectangle on the page … It was answering a real
> question — 'which of these is the one I am about to type into?' — and the
> answer is welcome under rule 4's fourth clause, which permits 'a snap
> indicator, a hover highlight, a rubber-band, a selection handle — these are
> the cursor'. It is still not carried, and the reason has **changed**: the
> mechanism now exists … so what is missing is only the panel→canvas channel."*

⇒ **Nothing had to be designed.** `canvas::forms` already places every fillable
widget in canvas space; this is the channel, and it is one temp-store slot.

### ✅ Built 2026-09-02

Focus a field's box in the panel and its box on the page is **outlined**.

★★ On **focus**, not on click, and the difference is your own word *"filled"*. A
click lands in the box and focuses it, so clicking lights it up — but so does
arriving by Tab, and so does still being there three keystrokes later. A
click-only trigger would put the light out the moment you started typing, which
is exactly when you want to know which box you are in.

★★ **Every widget of that field, not one.** A field can be painted in several
places — a header repeated per page, a radio group — and lighting one would
answer *"where is this field"* with a half-truth.

★ **An outline, where the O96 shade is a fill.** Every fillable field already
wears the wash, so a spotlight that was merely a stronger wash would be a
difference of degree you have to compare two boxes to notice. An outline is a
difference of kind and reads at a glance. It is the selection colour, because
that is what you are doing — picking this one out of a list.

★ The panel **clears the channel before its rows draw** and a focused row lights
it again, so no transition has to be tracked and a hidden panel puts the light
out by construction.

### ⬜ NOT DRIVEN

Built, unit-tested and gated. The oracle is a rendered window, which takes the
pointer.

---


### ✅ Driven, and it works — `clicking_a_form_row_lights_the_field_on_the_page`

Click a fill row, the canvas outlines that field: `field=FullName drawn=1
candidates=1`. It asserts the starting state too (nothing lit before the click),
so it cannot pass on a build that spotlights something unconditionally.

★★ Four failures on the way there and **not one was this feature**. The rows
published a rectangle while scrolled out of view so the pointer landed below the
window; a ribbon press closed the panel it needed; the wheel was eaten by a
nested scroll area; and the docked pane was too short for its content to exist
in. All recorded in `D:/dev/rag/egui/`.

### ⬜ ONE THING FOR YOU TO DECIDE — the spotlight does nothing in EDIT mode

The canvas's entire form overlay is gated on the **Select tool**
(`offered_in(tool)` is `CanvasTool::Select` and nothing else). Edit mode does not
start there, so on entering it the overlay stops drawing: no wash, no spotlight,
no clickable boxes. Measured — the canvas traced its last spotlight line at the
exact frame the mode changed to edit.

★ For the **wash** and the **click targets** that gate is defensible: "these are
the boxes you can click" is only true under Select, so marking them under a tool
that cannot click them would be a promise the app does not keep.

⇒ **The spotlight is different, and that is the question.** It is a read-only cue
driven entirely by the panel — you clicked a row, it says which box that row
fills. Your words were *"when we have the fill form panel visible and I click on
fields in it"*, with no mention of a tool. An operator filling a form from the
panel while some other tool happens to be active gets nothing, and nothing
explains why.

★★ **Not changed, deliberately.** Ungating it would mean the canvas draws form
boxes in modes where it currently draws none, which is a wider change than it
looks and touches the wash's own reasoning. Two shapes if you want it:

1. **Spotlight only, ungated** — the outline shows whatever the panel points at,
   in any mode. Smallest change; the cue and the click-target stop agreeing.
2. **Selecting a row switches the canvas to Select** — the panel's action
   implies the tool. Bigger, and it moves the operator's tool without asking,
   which this project usually refuses to do.

The driven check runs in **review** mode so it measures the feature in the state
it is designed for, and says so in its header rather than hiding the gap.

## O97 — ✅ **DRIVEN 2026-09-02** — the display buttons are on two rows

**Ken, 2026-09-02:**

> *"our display buttons should be on two rows to save space."*

### ★★★ The cause, and it was not a layout bug

The ribbon **only wraps a group when it runs out of width.** `wrap_group`
searches for the narrowest packing *once the group no longer fits*; a group that
fits stays on one row however wide that row is. That is right for a Font group
and wrong for a **radio** — four square icon buttons in a row is a strip, and the
same four as a 2 × 2 block is half the width and reads as one control.

### ✅ Built 2026-09-02

A new manifest field, `Group::prefer_rows` — *"lay this group out on several rows
even when one row would fit"* — and the page-display group asks for two.

★ It is a **hint, not a height**, and three properties are pinned by their own
test: asking for one row changes nothing; the band's row ceiling still wins, so a
group asking for four rows in a two-row band gets two (the band's height is fixed
— R128 — and a manifest must not be able to break it); and asking does not
*force* the count, because the planner still returns the narrowest packing.

★★ **No second packing algorithm.** The hint is exactly the right to skip the
*"it already fits"* short-circuit; everything after it is the search that was
already there, so a preferred layout and a pressured one cannot disagree about
how a group wraps. A group that prefers two rows still goes to three on the
collapse ladder.

★ **R7 holds:** `egui-shell` reads a number. It has no idea which of an
application's groups is a radio, and the manifest is where the application
already says everything else about its ribbon. `check-shell-purity` is green.

### ⬜ NOT DRIVEN

Built, unit-tested and gated. **The oracle for this one is a screenshot** —
`D:/dev/rag/egui/`'s standing rule is that layout defects have exactly one
oracle, a rendered window — and that needs the pointer, so it waits until you
are away.

---

## O96 — ✅ **DRIVEN 2026-09-02** — the fillable fields are shaded

**Ken, 2026-09-02:**

> *"in our display section we should have an option to shade the form fields like
> acrobat does."*

**Not investigated.** Acrobat draws a pale blue wash over every fillable field so
you can see at a glance what can be typed into, with a preference to turn it off.

### ✅ Built 2026-09-02

**Settings ▸ Display ▸ Show which boxes can be filled in**, on by default, which
is Acrobat's answer and the useful one: the operator who does not know a form is
fillable is the person this exists for, and they will not go looking for a
setting to reveal it.

### ★★★ How it stays inside rule 4, stated rather than assumed

The standing rule is *applied content renders exactly as saved content will
render*. This looks like a tint over part of the page and is not one:

**A field is a control, not content.** The wash is the affordance that says *this
box accepts typing* — the same class as the pointing hand that already appears
over a widget, and the same class as a snap indicator. It marks no inference and
says nothing about pdfcer's confidence in anything.

★★ The property that keeps that true is **where it is painted**: in the canvas
overlay, over the finished page texture, first so every other overlay lands on
top of it. It reaches no rasterizer, so it cannot appear in a print, an export, a
Save or a `render-page`. The radius line says so in the operator's own words.

★ The colour is the theme's **hyperlink** role at low alpha — not `selection`,
which would make every fillable field look selected, and not a literal, which
would not track light and dark. Hyperlink is the theme's *"this is interactive"*
role and that is what a fillable field is. The alpha is lower than the marquee's,
because this sits under the field's own text for as long as the document is open.

### ✅ DRIVEN 2026-09-02 — and it runs without touching the pointer

`fillable_fields_are_shaded_on_the_page`. The preference defaults to on and the
fixture opens on the page with the fields, so nothing has to be clicked; with
`PDFCER_DIAG_VIEWPORT` the window lays out without taking focus. One of only two
checks in the suite that can run while you are using the machine.

★★ The trace had to learn to distinguish **three** states first, because drawing
nothing is the outcome of all three and only one is a defect: the wash turned
off, the wash on with no fields in the document, and the wash on with fields
present and none painted. Before it carried `on=` and `boxes=` as well as
`drawn=`, a build with the feature entirely dead would have looked identical to
a correct build run against a document with no form.

Falsified by disabling the wash and rebuilding — red, with the right diagnosis.

### ⬜ AN OPEN QUESTION THE CHECK SURFACED, and it is YOURS to settle

The fixture carries **two** widgets and the wash paints **one**. That is not a
bug: the text field has no `/AP` `/N`, so the page draws nothing there, the
canvas census deliberately excludes it, and the Forms panel already discloses it
off-canvas with the remedy named — *"N field(s) are not drawn on the page, so
they cannot be clicked there. Fill one here and it becomes drawn."*

⇒ But your words were *"like acrobat does"*, and a field that draws **nothing**
is arguably the field that most needs a wash saying *"you can type here"* — it
is exactly the invisible-field case the feature exists for.

★★ **Not acted on, and deliberately.** Two reasons:

1. It is a real design question rather than a defect. Widening the wash to
   undrawn fields would mark boxes that **cannot be clicked**, so the mark would
   promise a gesture that is not there — which is its own R9 problem, pointed
   the other way.
2. I have **not verified what Acrobat actually does here.** It is installed, and
   checking needs the screen. Asserting its behaviour from memory is precisely
   the failure this project forbids, so it is recorded as a question rather than
   as a finding.

If it should change, the shape is probably *"wash the undrawn ones too, in a
different weight"* — but that is a call about your own workflow, not one to make
from a fixture.

---

## O95 — ✅ **DRIVEN 2026-09-02** — Save As, and then keep editing the NEW file

**Ken, 2026-09-02:**

> *"we need a Save As option so that we are then making edits in the save as file
> instead of the original just like other programs have it."*

**Not investigated**, and the second half is the whole request. `file.save_copy`
already writes a copy — but it is a *copy*: the session stays pointed at the
original, so the next `Ctrl+S` goes back to the file he was trying to leave.

★ That is the difference between **Save a copy** and **Save As**, and both are
real commands with different meanings. Acrobat, Word and every editor he uses
have both, and Save As **rebinds the document**: the title bar, the tab, the
recent list and the next save all follow the new file.

### ✅ Built 2026-09-02

**File ▸ Save ▸ Save as…**, between Save and Save a copy — which is the group's
own stated order of increasing consequence and is also where Word and Acrobat
put it.

**The answer to the question above: nothing is closed and nothing is reopened.**
`doc.path` is re-pointed and the session continues, so **the undo stack, the
selection and the history all survive** — which is what every other editor does
and what you would expect. A write-close-reopen would have discarded every undo
step silently, which is a data loss with no warning on it.

**Four things move, in one place** (`PdfcerApp::save_as_somewhere`), because a
document whose path moved while something else did not is a document whose next
`Ctrl+S` writes a file you are not looking at:

1. `doc.path` — the title and the tab both recompute from it every frame;
2. `doc.saved_epoch` — the new file has every edit, so the document is clean and
   the tab loses its unsaved marker;
3. the recent list — you will look for the new name there;
4. **a receipt**: *"Saved as X. You are now editing that file — the original is
   untouched."* The rebinding is otherwise invisible until the next save, and by
   then the surprise has happened.

★ The signature question is asked as a **copy's**, not an in-place one, which
reads like a mistake and is not: the question is about the bytes being written,
and this writes a *new* file. There is a test whose only job is to say so.

### ⬜ NOT DRIVEN

Built, unit-tested and gated; **not driven**, because you are at the PC and the
harness takes the real cursor. Under R1 that means not shipped.

### ★★ What it cost, and it is a scheduling fact rather than a complaint

Three files crossed the 1,500-line R2 ceiling at once, because they were at
1,499 / 1,497 / 1,491 before this. Two were split along seams the tree already
uses — `text/commands/file.rs` and `app/actions/saving.rs` — and the third,
`app/actions/action.rs`, is now at **exactly 1,500**.

⇒ **The next command added to `Action` forces a split of that enum.** The seam
is ready and named: the five inline redaction variants should become a sub-enum
like the ten that already are (`Annot`, `Page`, `Field`, `Vector`, …), which is
about 100 lines moving and 11 call sites.

---

## O94 — ✅ **BUILT AND DRIVEN 2026-09-01** *(back-filled — see above)* — OCRed text can be copied

**Ken, 2026-09-01:** *"also I can't seem to copy and paste text we have OCRed"*

Real, and **the regression was mine, from that same morning.** A scanned page is
an image with an invisible text layer over it. The Read-mode click handler
resolved the image first, so a sweep across OCRed words grabbed the picture and
the text was never reachable.

**Fix:** text under the pointer beats the picture under it — the image arm now
runs only when no text run contains the point. Plus `word_at()`, which does
strict **containment** over the run boxes, deliberately *not* the existing
`hit()`, which falls back to the nearest line and would therefore select a word
you were not pointing at on a page of sparse OCR.

**Evidence:** `text_on_a_scan_can_still_be_swept_over_the_image`. Commits
`273d117`, `ebee870`.

## O93 — ✅ **BUILT AND DRIVEN 2026-09-01** *(back-filled)* — OCR says what it is doing, and Stop and Cancel differ

**Ken, 2026-09-01:**

> *"can you make it so the recognizing ocr gives feedback on what it is doing
> when it is running (pages done, words/characters detected, etc) so that the
> user can see that it is doing something and hasn't frozen on large documents?
> Maybe a cancel and stop button too. The cancel throws away what was done, and
> the stop finished the page it is on and keeps the work it has done."*

Built exactly as worded. Pages done, running word and character counts, and two
controls that mean different things: **Stop** finishes the page in flight and
keeps everything; **Cancel** discards. ★ Where both are pressed, **Cancel wins
in either order** — unit-tested both ways, because a Stop arriving after a
Cancel must not resurrect the work the Cancel threw away.

### ✅ Driven, on the operator's own scanned parts manual

**The fixture arrived on 2026-09-01:** *"there is a large document in the pdftest
folder … that is all images with text you can try ocr on."* That is
`Parts Manual TH83 Telehandler.pdf` — **883 pages, 266 MB, every page an image,
`/Rotate 270`, not one extractable character anywhere in it.** Eight pages were
extracted to `D:\Dev\pdfTests\scan	elehandler-8pages.pdf` and the whole
manual measured at **2.6 s and ~440 recognised words a page**, which is where
his *"I wouldn't do the entire thing as that takes ages"* comes from: the full
manual is about **38 minutes**.

**Three driven checks, in `tools/ui-verify/src/checks/ocr_progress.rs`**, each
run twice — once on a committed eight-page synthetic fixture and once on his
scan:

| check | what it asserts | on his scan |
|---|---|---|
| `ocr_says_how_far_it_has_got_while_it_runs` | the tally is **drawn** and its numbers **advance** | 7 distinct values, 2,218 words / 12,431 chars |
| `stopping_ocr_keeps_the_pages_it_had_already_done` | Stop ends early **and the session takes the pages** | stopped at 2 of 8, 758 words kept |
| `cancelling_ocr_throws_away_what_it_had_done` | Cancel leaves **no** `ocr-applied` and **no** `ocr-layer` anywhere | abandoned at 2, nothing applied |

★★ **The pair was falsified, not just passed.** `job.stop()` was temporarily
replaced with `job.cancel()`, the workspace rebuilt, and the Stop check failed
with *"STOP THREW THE WORK AWAY … the CANCEL path"*. Restored and green again.
Without that, three green checks would only have shown that the two buttons do
*something*.

### ★★★ Two things the driving found that no unit test could

**1. A rect is not an oracle for "the user can see it is working."** The dialog
published where the progress label was drawn, and a build whose label read
`Page 1 of 8` for the whole run would publish exactly the same rect — while
being the frozen program the request is about. The shell now traces the
**numbers** (`ocr-progress attempted= of= words= chars=`, on change) and the
check asserts they move.

**2. ★ Live progress was resting on a decorative widget.** egui is
immediate-mode and idle; the OCR worker is on another thread and generates no
input events, so **nothing was asking for the next frame.** It worked only
because `egui::Spinner` calls `request_repaint()` for its own animation
(egui 0.35, `widgets/spinner.rs:40`). Anyone swapping the spinner for a progress
bar — a completely reasonable change — would have silently taken live progress
with it and left every test green. `dialogs::ocr` now asks for the repaint
itself and says why.

### ★ And a check that had been quietly not running

`ocr_recognises_a_page_and_the_document_keeps_it` was reporting **SKIP**, not
PASS: at the window's default width the `file` tab's Recognise group
**collapses**, so `ribbon.item.file.ocr` is never declared and the harness
reported *"no control to click"* — which reads as the command having been
removed. A SKIP is not red, so nothing prompted a look. `Session::maximize`'s
own doc comment describes this precise symptom; the call site simply never got
it. Both OCR check paths now maximize, and that check passes for the first time
in an unknown number of runs.

★ One measured behaviour that is **correct** and looks like a defect: the tally
ends at **7 of 8**, never 8. `Job::poll` drains the channel, so the frame that
reads the last page's report is the same frame that reads `Finished`; the dialog
leaves the working phase before drawing it. The outcome then states the true
totals. The check asserts a band (`>= scope - 1`) and its source carries the
reasoning, so nobody re-tightens it.

## O92 — ✅ **SHIPPED AND DRIVEN 2026-09-02** — reaching an object dropped off the side of the page

**Ken, 2026-09-01:** *"we should be able to select things offside of the page,
especially since I sometimes drop objects there, and when I do I can't get them
back."*

### ✅ Select All on the page — 2026-09-01

There is a Select All that takes every object on the sheet **including the ones
outside its boundary**, so a dropped object can be grabbed and dragged back.

★ Its first version selected **nothing**. It asked for `Rect::EVERYTHING`, whose
infinities become **NaN** through the canvas-to-PDF transform, and every
comparison against NaN is false — so a rect meaning "all of it" is
arithmetically identical to a rect meaning "none of it", and the failure is
silent. A finite `1.0e6` rect selects 25 of 25. Commit `a2ea73b`.

### ✅ …and the gesture you would actually reach for — 2026-09-02

**Drawing a box in the margin now works**, and it needed no new code: it is
O88's crossing window. A right-to-left drag takes what it **touches**, so a band
started on the sheet and dragged out into the grey reaches an object lying
entirely off the edge. An *enclosing* band over the same rectangle surrounds
nothing and returns zero, which is what shipped before.

**Driven**, on a purpose-built fixture, and **falsified** — with `mode_for`
stubbed to return `Enclosed` for both directions it fails with *"THE BAND
REACHED INTO THE MARGIN AND FOUND NOTHING"*; restored, it passes.

```
marquee-mode crossing=true mode=touched hits=1 paths=1 text=0
canvas-selection via=pv.marquee sel=1 first=object:1
```

### ★★★ Why `hits == 1` is an airtight oracle here

`fixtures/off-page-object.pdf` is a 200 × 200 page with **two** filled squares
and nothing else: **A** at x 40–100, y 40–100 (on the page) and **B** at
x −160…−40, y 100–140 (**entirely left of the media box**).

The band runs from `(160, 170)` — blank paper — out to `(−100, 120)`. It

- **misses A**, whose top edge is y = 100 against the band's bottom at y = 120;
- **touches B**;
- **cannot enclose B**, which reaches x = −160 while the band stops at −100 —
  deliberately, or the check would pass under the old mode too and stop
  distinguishing them.

⇒ One hit can only be B. No index, no ordering assumption, no kind census.

### ★★ One harness thing had to change, and the first attempt at it was wrong

`CanvasMapping::doc_to_window` **refuses every point outside the media box**, and
that refusal is right for every other caller. This needed a separately named
`doc_to_window_off_page` — a caller has to say the words.

★★★ Its first version bounded the result against `image_rect`, and **that is the
page's own rectangle**, so every off-page point is outside it by construction.
The whole class the function exists for was rejected, with a message about *"not
enough margin on screen"* — plausible, and completely wrong. The bound that means
something is the **canvas viewport** (`ui-rect name=canvas-viewport`), whose grey
margin is where a dropped object lives.

### ⬜ Nothing shipped in the binary for this row

O88's build already contained the fix. What 2026-09-02 added is the **evidence**
— a fixture, a driven check and a harness conversion — so the published build on
`pdfcer-gui1` already behaves this way.

## O91 — ✅ **SHIPPED AND DRIVEN 2026-09-01 (evening)** — a clickable table of contents works

**Ken, 2026-09-01.** The second half of the same message as O90.

> *"also I don't think I have an example in that folder, but I think the what's
> new pdf on the desktop might have a table of contents that you can click on
> and be sent to the appropriate section. I could be wrong though as I am not at
> the PC to try. anyway, it didn't do that in ours, but I didn't confirm in
> Adobe either."*

**You are right that it does not, and the shell-side cause is total: there is no
link-following code path at all.** Not a broken one — none. Clicking a `/Link`
does nothing, and nothing is drawn to suggest it would.

### Why it is not a shell fix

A `/Link` annotation's **destination cannot be read**. `pdfcer-core`'s
`Annotation` carries `action_type` — the `/S` name, so `GoTo` — by an explicit
and documented decision (*"the `/S` NAME only, deliberately — not the action
dictionary"*), which is the right model for `list-annotations`, whose job is to
print one token of disclosure. It is the wrong model for a viewer, whose entire
job with a `GoTo` is to **perform** it. `outline.rs` exposes no public
destination parser to point at an arbitrary `/D`, and the shell has no raw
object-graph access — nor should it, or the §12.3.2.2 name-tree walk would exist
twice and the two copies would drift.

Filed 2026-09-01 as
`request_a_links_destination_cannot_be_read_so_a_table_of_contents_is_dead.md`.

### ★ What I could have shipped and deliberately did not

A hand cursor over the link's rect plus *"action=GoTo"* in the status line —
disclosure without navigation. Rejected: it advertises a capability that does
not exist, and R9 says an unavailable capability renders **nothing**. The
absence stays silent until the reader lands.

### ✅ The engine answered the same evening, and it shipped

`pdfcer-core` `Pass 222.0` (engine `94d640c`) added
`outline::DestinationReader`, `annot::page_link_destinations`,
`Annotation::destination`, five synthetic link fixtures and
`pdfcer list-links`. `action_type` was left untouched, which was the right
call and was the request's own argument: the fix is a second entry point, not a
changed contract.

**Clicking a link now navigates.** In Read and Review; in Edit a `/Link` stays
an annotation you can move, resize and delete, which is the conventional split
and is the same predicate the fill-versus-author split already uses.

### ★★★ The part that is not "make links work"

`Destination` has **five** variants and only one navigates. The failure this
feature was most likely to ship is not *"links do nothing"* — that is loud, and
somebody reports it in a minute. It is a viewer that treats all five as
navigable, resolves the four it cannot perform to a defaulted page 0, and sends
you confidently to the front of the document. **That has no symptom.** The
cursor changes, the click lands, a page appears — and you conclude the
document's links are wrong.

So the four non-navigating cases each get **their own sentence**, off-canvas, on
the click:

| what it is | what it says |
|---|---|
| target page not in this document | the page was deleted, or this file is a range of a larger one |
| a name nothing defines | the name table was lost when the file was made |
| `/GoToR` — another file | names the file and the page, so you can open it |
| `/URI`, `/JavaScript`, `/Launch` | says what it is; **recognised and disclosed, never executed** |

### ★ The affordance is a cursor, and nothing is drawn on the page

A pointing hand over a link that can be followed, and **nothing at all** over
one that cannot — R9. No border, no tint, no dashed rectangle over the `/Rect`.
A screenshot of the canvas is identical to a screenshot of the same document
saved and reopened, which is rule 4's one-line test.

★ The hand-cursor-plus-`action=GoTo` compromise above stays rejected, and for
the reason it was rejected: it advertises a capability that does not exist. What
ships instead is a sentence *after* you ask.

### ★★ Evidence — and one falsification that FAILED and had to be repaired

Two driven checks on the engine's own synthetic fixtures:
`a_link_goes_to_the_page_it_names` and
`a_link_it_cannot_follow_says_so_instead_of_jumping`.

The second was falsified by planting the plausible wrong implementation — every
destination fed to the navigator with a defaulted page 0 — **and it passed
anyway.** The fixture opens on page 0, so a defaulted jump to page 0 moves
nothing, and *"the page did not change"* was true of the broken build too.

⇒ The check now **zooms in first**, so the view is nowhere near where any
defaulted navigation would land, and asserts page, zoom **and** scroll offset
are all unchanged. Re-planted: it fails. Restored: it passes. Without that
repair it was a check that could not fail, which is not evidence.

★ The engine's own fixture note asks for the property this rests on and it is
worth repeating: **no link in those fixtures targets page 1**, because a fixture
whose links all point at the first page passes against an implementation that
resolved nothing at all.

### Not verified in Acrobat either

You said you had not confirmed it there. Neither have I — and I could not find a
link-bearing PDF anywhere in `pdfTests\` or on your desktop; every annotation in
both is a `Widget`. If you have a file where Acrobat's TOC works, that is the
fixture this needs.

## O90 — ✅ **FIXED 2026-09-01** — a bookmark lands on the detail it names

**Ken, 2026-09-01:**

> *"in Acrobat clicking on the nested bookmarks in the drawing package takes you
> to a zoomed in area of the page for the drawing bookmark that was clicked on.
> when we click on ours it just jumps us to the correct page, but doesn't send
> us to the spot on the page the bookmark actually points to."*

**Exactly right, and it was one discarded field.** `Destination::Page` carries
both a `page_index` and a `view`, and the panel matched
`Some(Destination::Page { page_index, .. })`. The `..` was your zoom.

### Why it looked like the outline was fine

On `TR-0461-1500-copy.pdf` — your own drawing — sheet 1 has two nested
bookmarks pointing at *different* rectangles:

```text
  "Drawing View64"  /FitR 493, 119, 1104, 558
  "Drawing View65"  /FitR  76, 119,  687, 558
```

Both arrived in the same place, which is indistinguishable from both being
broken. ★ It is also why a check asserting *"the page changed"* would have
**passed against the defect** — the page was always right. The new one asserts
the zoom rose instead.

### Evidence

`a_bookmark_lands_on_the_detail_it_names` — opens your A1 sheet, clicks
"Drawing View64", and measures the canvas: **0.382× fitted → 0.766× framed**.
Shipped to `pdfcer-gui1` 2026-09-01 19:57.

★ The first run of that check FAILED at 0.382 → 0.382 — because I had rebuilt
the harness and not the shell. Recorded because it is the second time this week
a stale binary produced a confident wrong diagnosis.

### Please check

Whether the *other* nested bookmarks in your drawing package land where you
expect. All five destination kinds are handled, but only `/FitR` is exercised by
your fixture, and `/XYZ` — the one Acrobat writes most — is the one carrying the
null-versus-zero rule that is easiest to get backwards.

## O89 — ◑ **"I don't see where I am able to edit the color of text, vectors, etc."** — one half is hidden, the other does not exist

**Ken, 2026-09-01.** Two different answers in one sentence.

### ✅ Text colour EXISTS — and you have to sweep the text first, which is why you could not find it

There are two colour swatches and both are real:

- **Format tab ▸ Font ▸ the colour button**, beside Bold and Italic.
- **Properties panel ▸ Text**, the same control.

★★★ **Both are gated on a TEXT selection, not an object selection.** Clicking a
piece of text selects the *object*; the Format tab's Font group stays greyed.
You have to arm the **Text tool** and **sweep across the words** — then the
colour button lights up and applies to exactly what you swept.

⇒ That is a real route and a bad one to have to guess. It is also the same shape
as the last two of these: the capability shipped, the way in did not.

**Not yet decided how to fix the route**, and it should not be improvised. The
candidates:

| | |
|---|---|
| a colour control on a *selected text object* that sweeps it for you | closest to what you tried |
| the Properties panel naming the missing step where the swatch would be | cheapest, and honest |
| the greyed button saying *"sweep the text first"* on hover | R9's own rule, and it is not doing it today |

### ✅ Vector colour SHIPS — same day, and finding it turned up a real bug

**Properties ▸ Colour**, with a selected line or shape: a **Fill** swatch and a
**Line** swatch, each opening on that object's own colour.

★★★ **A spot ink gets no swatch.** Where the colour is a named ink — a
`/Separation`, the kind your printed drawings use — the panel names it instead:
*"PANTONE 300 — a named ink. pdfcer will not overwrite it with a screen colour,
because that would look right here and change what prints."* A colour picker
that opened on black over a spot ink would be one click from destroying a plate,
and it would look completely normal while it happened.

**One object at a time, for now.** The engine will recolour a whole selection;
the control does not offer it yet, because when the objects disagree there is no
honest colour to open on and picking the first one's would quietly propose
flattening the rest to it.

### ★★★ Asking for this found pdfcer writing wrong colours into saved files

Before writing the setter the engine went looking for *"what does pdfcer think
this path's colour is"* — and the answer was **nothing**. The object model
tracked only the basic colour operators and had no handling at all for spot
inks, `/DeviceN`, `/ICCBased`, `/Indexed` or `/Lab`. A path in any of those
inherited **a stale colour from an unrelated earlier object.**

⇒ **That was reaching your documents.** Copy a spot-coloured line, paste it, and
pdfcer invented an RGB colour from that stale value and **wrote it into the
file**. Your own copy and paste, on your own drawings, on exactly the kind of
file you work in. Fixed: a paste now emits no colour at all for an ink it cannot
decode — visibly wrong and undoable, rather than invisibly wrong and permanent.

★ It was found because a colour control has to *show* the current colour, and
asking whether pdfcer knew it audited everything else that thought it did.

### ⬜ And one of their side-findings is your open "line won't select" report

Their audit turned up that `/LW` line widths are read stale too, which feeds the
click tolerance — *"the operator clicks a visible line and nothing selects."*
**You have reported that.** We had put it down to box-selection geometry, which
is a real cause and evidently not the only one. Recorded against that row as a
second source.

<details><summary>The original diagnosis</summary>

### ⬜ Vector colour did not exist AT ALL — filed with the engine

Measured rather than assumed. Every colour verb `pdfcer-core` has works on an
**annotation** or on **text**: markup style, ce-dimension style, redaction-mark
style, text fill. **There is no verb that changes the fill or stroke colour of a
path in page content.**

So a line, a rectangle, a CAD drawing's every stroke: selectable, movable,
deletable — and not recolourable. No shell control could exist yet because there
is nothing for it to call.

**Filed:** `open/request_a_paths_colour_cannot_be_changed_at_all.md`, asking for
the verb **and its reader** — a swatch that cannot show the object's current
colour is one that silently discards it on first touch — plus a named refusal
for spot inks, because writing `DeviceRGB` over a named separation would look
right on screen and destroy the plate on your drawings.

★ This is the half you will notice more. A drawing office recolours lines far
more often than it recolours type.

</details>

**Status:** ◑ **Vectors SHIP. Text works and is still unfindable** — the route
question above is undecided and is yours.

---


## O88 — ✅ **SHIPPED AND DRIVEN 2026-09-02** — a right-to-left box takes what it touches

**Ken, 2026-09-01, on `TR-0461-1500-copy.pdf`:**

> *"I can't box select the tables in the left or right top corners using the
> mouse — it only picks up the lines of each table, so I can't drag the entire
> thing and move it somewhere else, or cut/copy and paste it elsewhere."*

### What was measured, read-only, without touching your mouse

- That page has **no form XObjects at all** (`forms=0` from a render). So this
  is not the wrapped-drawing case, and the tables are ordinary page objects —
  paths and text side by side.
- The engine's marquee **does** include text: it selects any object whose
  bounding box satisfies the mode, with no filtering by kind.
- This shell asked for exactly one mode, everywhere: **`MarqueeMode::Enclosed`**
  — an object counts only if the band **completely surrounds** it.
- The engine also has **`MarqueeMode::Touched`** — *"selected if its page bbox
  touches the marquee (any overlap)"* — and **nothing in this shell had ever
  asked for it.**

### ★★★ Why "enclosed only" makes your corner tables uncatchable

Both tables sit hard against the sheet edge. To surround one you would have to
start the band **outside the page**, and at fit zoom there is barely a pixel of
margin to start in. So the only band you can actually draw is one *inside* the
table — which surrounds a few short rules and nothing else.

⇒ **"It only picks up the lines" is what an enclosing band returns when it
cannot be drawn big enough.** Not a hit test that excludes text.

### ✅ What was built, 2026-09-02

**AutoCAD's direction-sensitive marquee**, which SolidWorks drawings use too:

| drag | AutoCAD's name | selects |
|---|---|---|
| **left → right** | a *window* | only what is completely surrounded (what pdfcer did before) |
| **right → left** | a *crossing window* | anything the band **touches** |

No modifier key, nothing new to learn. Illustrator selects on touch always;
Inkscape encloses and puts touch on `Alt`. The direction rule is the
drawing-office one, and this is a drawing program — your own standing
instruction is to use the conventional interaction rather than invent one.

★ **The enclosing band's answer is unchanged.** `Enclosed` is still what a
left-to-right drag does and is still the right default on a dense sheet.
Decision 011's reasoning is untouched; what was wrong was that it was the only
answer available.

**Where the code is:** `canvas/marquee.rs` (new — `mode_for`, `select`,
`without_page_wrappers`), `canvas/gesture/outcome.rs` (the `crossing` flag),
`canvas/target.rs` and `panels/objects/provider` (the mode parameter).

### ★★★ A hazard the change introduces, found by a failing test rather than by thinking

A crossing band **touches a page-sized form XObject wherever it is drawn.** So
on a drawing wrapped in one — which `ncored-benchmark-cad-drawing.pdf` is —
every right-to-left drag would have silently included the whole sheet, and the
next gesture would move it. Under `Enclosed` this could not happen: a band that
*surrounds* a page-sized form has to surround the page, which cannot be drawn.

`canvas::marquee::without_page_wrappers` drops it, using **the shell's existing
rule** — `container_is_worth_selecting`, which already answers *"is this
container really just the sheet?"* against the page extent and which
`canvas::smart` already applies to the click ladder. No second threshold exists.

★ **Only a hit that contains another hit is tested.** A lone path covering the
whole sheet — a drawing border, which is on almost every sheet this program is
for — is not a container and stays selectable. There is a test whose only job is
that case, because the obvious simpler implementation (`retain(worth_selecting)`)
fails it.

★ Measured rather than assumed: of four of your drawings, `TR-0461-1500-copy` and
`SW41177` have `forms=0`, `ncored-benchmark` has 1, and `TRP5187 - Weber Supply`
has 75.

### ✅ DRIVEN 2026-09-02, and getting there took three wrong diagnoses

`a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` **passes**, on your
own drawing, and was **falsified**: with `mode_for` stubbed to return `Enclosed`
for both directions it fails with *"THE BAND ENCLOSED THE TABLE AND SELECTED
NOTHING"*; restored, it passes.

```
marquee-mode crossing=true mode=touched hits=3 paths=2 text=1 other=0
canvas-selection via=pv.marquee sel=3 level=Object first=object:7
```

Paths **and** text — which is the kind you reported missing.

### ★★★ Three wrong diagnoses, and every one of them looked like the feature

**1. "The harness drove the band above the canvas."** True of the first run — the
file is ten pages shown continuously and the view had inherited a scroll — and
fixed by fitting the page first. It also **masked the next two**.

**2. The Select tool was never armed.** Fixed, and the check still selected
nothing.

**3. ★★★ The band's origin was on ink.** The trace carried
`selection-set page=0 object=23 via=press` and **no marquee line at all**: the
press selected the object under it and the drag became a *move*.
`canvas::presspick` documents exactly this — *"a press on empty paper still
marquees"* — and pressing on ink does not.

★★ **"Empty" is far wider than it looks.** The pick tolerance is 4 *screen*
pixels converted to page units, so at the fitted zoom this check drives (0.38×)
it is over **ten page points**. The old origin sat 6 pt from the sheet border —
visually in the margin, and inside the catch radius. The new one was chosen by
rendering the page at 1 pt per pixel and looking: 80 pt clear of anything.

⇒ **A check that fails for three different reasons in three runs is one whose
later diagnoses nobody looked for.** Each fix was correct and each revealed the
next.

### ★★★ …and then the check's own oracle turned out to be an assumption

It asserted a **count**, reasoning that *"a table's rules are one path object per
line and its words are one text object per cell, so a band over this table should
return well into double figures"*, failing anything under four.

**Measured on that sheet: `objects n=25 paths=19 text=6`.** The *entire drawing*
— two tables, a title block, an isometric view, dozens of labels — is
**twenty-five objects**. A band returning three is a large fraction of the page,
and the threshold was rejecting a correct result while calling it your defect.

★★ It could never have expressed your complaint anyway. *"It only picks up the
lines"* is a claim about **a kind being missing**. One path and one text is a
pass; nine paths and no text is the defect — and a count ranks those two the
wrong way round at every threshold.

⇒ `canvas::marquee::select` now reports the breakdown from the provider's own
classifier (`paths=`, `text=`, `other=`), and the check asserts **both kinds are
present**. That is the first oracle here that can actually fail for the reason
the check is named after.

### ⬜ What is still not driven, and it is your own complaint

**The enclosing direction is not driven, and cannot be on this sheet.** To
surround a table hard against the edge the band must start outside the page, and
every corner it could be started from is on ink — which is precisely what you
reported. The crossing window is the answer to that, and it is what is driven.

★ A **second cause** stays recorded against this row from the original
diagnosis: a stale `/LW` can make a visible line unselectable, which presents
identically to a band that missed it.

---

## O87 — ✅ **NOT A DEFECT — an old build.** Paste lands at the cursor

> **Ken, 2026-09-01, an hour later:** *"Just realized windows wasn't opening the
> latest version. paste puts things where the cursor is."* And: *"it wasn't you.
> It was me. I had linked the default pdf opener to a different location. I
> thought I had relinked it to the new one but it didn't take."*
>
> ### ★★★ The part that IS ours, and it is now fixed
>
> **Nothing on screen could have told either of us which build was running.**
> That is the whole cost of this row, and it does not depend on how the wrong
> build got launched — an operator describes a defect that was fixed, and an
> engineer investigates a version nobody is running.
>
> The build stamp existed the entire time. `build.rs` sets it and the About
> window shows it — two clicks behind a menu nobody opens while they are
> working.
>
> ⇒ **The build date is now in the window title.** Taskbar, Alt-Tab, a
> screenshot, the accessibility window list: all read it, and none of them can
> see a menu. The day rather than the minute, because the question a title has
> to answer at a glance is *"is this today's?"* — the exact time and the commit
> stay in About for anyone comparing two builds precisely.
>
> ★ The diagnostic added while chasing this stays, and it earns its place: the
> paste's fallback now says WHICH half was missing (`offset-no-cursor`,
> `offset-no-anchor`, `offset-neither`) rather than only that it fell back. Two
> causes, opposite investigations, and the trace used to be silent about which.

<details><summary>The original report and the analysis it prompted</summary>

## O87 (original) — Paste lands near the copy, not at the cursor

**Ken, 2026-09-01:**

> *"copy and paste still doesn't paste where the cursor is, it just pastes near
> the copied object."*

### The rule needs TWO things, and either one missing degrades silently

Pasting at the cursor is computed as *"move the clip so its centre lands under
the pointer"*. That needs:

1. **where the pointer is** — resolved at paste time, and only honoured when the
   pointer is over the canvas;
2. **where the clip's centre was** — computed at **copy** time, from the bounds
   of what was selected.

If either is missing the paste falls back to the old rule: a small offset from
the original. Which is exactly what you are seeing.

### ★★ What was wrong is that nobody could tell which

The diagnostic said `at=offset` — the outcome, not the cause — and the two
causes want opposite investigations:

| missing | means | where to look |
|---|---|---|
| the cursor | no canvas frame, or the pointer was not over the page | the ROUTE you pasted by — a ribbon or menu press has the pointer somewhere else entirely |
| the centre | the clip carries none | the COPY, and whether the object model could answer for the page the selection was on |

⇒ Now it says `offset-no-cursor`, `offset-no-anchor` or `offset-neither`. One
word, and the next report is diagnosable from a log instead of from guesswork.

**Status:** ⬜ **OPEN — instrumented, not fixed.** The next step needs one run
with the diagnostic on, doing the paste **the way you do it** — the route
matters, and a ribbon press and `Ctrl+V` differ precisely in where the pointer
is at the moment the command fires.

★ If you can say which one you use — the keyboard, the right-click menu, or the
ribbon button — that alone may settle it without any driving.

</details>

---


## O86 — ✅ **FIXED 2026-09-01** — filled fields size themselves to the box

> **Your text now fits the field it is in.** On your Weber form:
>
> | | box | was | now |
> |---|---|---|---|
> | a description row | 27.8 pt | 12 pt | **22.4 pt** |
> | a header field | 13.1 pt | 12 pt | **9.6 pt** |
>
> The second row is the half that was about to bite you: 12 pt in a 13 pt box
> overflows, and every header field on that form is that size.
>
> **And pdfcer now tells you which way it decided**, in the words it uses:
> *"fitted to the field's HEIGHT; make the box taller to change it"*, or
> *"shrunk to fit the field's WIDTH"*, or — the honest third case — *"held at
> pdfcer's legibility floor; the box is too small for this text, which will
> overflow"*.
>
> ### ★★ It lands about 16% larger than Acrobat, deliberately
>
> Acrobat applies one more step: it shrinks by a constant 1.165 that nobody has
> been able to explain. Eleven of its own appearances in your file agree on that
> number to within 0.4%, and it is **not** a fit to the text width — the strings
> are nowhere near the edges. The engine declined to divide by an unexplained
> constant, and that is the right call: your complaint was 12 against 18, not 21
> against 18.
>
> ★ The measurement that would settle it needs an Acrobat-filled field in a font
> other than Helvetica. **Every form you have uses Helvetica** — all four Weber
> forms and every conformance test page in that folder — so it stays unexplained unless a
> different file turns up.
>
> ### ★ One thing in my own report was wrong
>
> I told the engine the second step was a width fit, from two numbers that
> looked like one. They measured it and it is not — the text fits comfortably at
> the larger size. The first half of my report was measured; the second was a
> plausible story about a gap, and it read the same because it sat in the same
> table. Recorded because it is the kind of mistake a reader cannot catch.

<details><summary>The original report and the derivation</summary>

## O86 (original) — Filled fields come out the wrong SIZE

**Ken, 2026-09-01:**

> *"when I fill out the form fields on `TRP5188 - Weber Supply.pdf` the font
> doesn't match what is set for the fields. Adobe uses the same font and size as
> in the first filled out row below the headers. 'TC-10 Wheel Chocks' is in the
> font that should be showing for the other fields."*

### ★★★ It is the size, not the typeface — and pdfcer announces it

Every field in that document declares the same default appearance:
**`/Helv 0 Tf`**. `0` means *auto-size* — the reader works out what fits.

pdfcer's answer is **12 pt, always**, and `fill-field` says so on the way past:
*"auto-sized to 12 pt (a reviewable pdfcer heuristic; §12.7.3.3 mandates no
formula)."*

| | box height | Acrobat wrote | pdfcer writes |
|---|---|---|---|
| `DescriptionRow1` | 26.40 pt | **18.08 pt** | 12 pt |
| `WO` | 13.08 pt | **8.21 pt** | 12 pt |

⇒ On the description rows your text comes out **two-thirds** the right size; on
the header fields 12 pt would be **150%** and overflow a 13 pt box. One constant
cannot serve both, and your form has both on the same page — which is why it is
obvious at a glance.

### ★★ Acrobat's formula, measured off your file

It writes two `Tf` operators and the first is the candidate:

```
/Helv  21.0975 Tf     <- from the box height
/Helv 18.082 Tf       <- shrunk so the words fit the width
```

`(box height − 2) ÷ 1.156` gives 21.0975 on the description box and 9.5396 on
the WO box. **1.156 is Helvetica's own bounding-box height** — the same number
falls out of two boxes that differ by a factor of two, so it is the rule rather
than a coincidence. Then it shrinks to fit the width.

**Filed:** `open/request_auto_sized_field_text_is_a_flat_12pt_and_acrobat_fits_the_box.md`,
with both measurements and the derivation.

**Status:** ✅ **FIXED** in `pdfcer-core` `Pass 215.0`, `d5d012e`.

</details>

★ Not blocking you — you can fill in Acrobat — and you did not ask for a
workaround.

---


## O85 — ✅ **NOT A DEFECT — an old build.** Ctrl+S closing the program

**Closed by Ken, 2026-09-02:** *"the save bug was due to running an old version
and is no longer present."*

★ Worth keeping rather than deleting, and for the same reason O87 was: **this is
the second report closed by "you were running an old build"**, and two of a kind
is a pattern rather than a coincidence. The published slots alternate, so the
folder he opens is not always the newest — and nothing in the program tells him
which he is running without opening About.

⇒ That is a discoverability finding about the *publishing scheme*, not about the
program, and it is now on the record twice.

<details><summary>The original investigation, kept because the reasoning is still sound</summary>

## O85 — ⬜ **"I pressed Ctrl+S to save and it closed"** — NOT REPRODUCED YET

**Ken, 2026-09-01:** *"can you try doing an edit and save? I did this and
pressed ctrl+s to save and it closed."*

### What was done immediately

A driven check now exists and is permanent:
`ctrl_s_after_an_edit_saves_and_the_program_is_still_running`. It makes a real
edit, presses a **real** `Ctrl+S` through the keyboard, and asserts four things
in this order — the order matters, because a program that has exited writes no
trace line, and an absent line is this harness's commonest *false* signal:

1. the edit landed, so there is something to save;
2. **the process is still running** after the chord;
3. the save committed (`save-in-place outcome=ok`);
4. **the document is still open**, on the same page count.

★★ Step 4 is `O65` — *"it closes the document after saving"* — which was fixed
on 2026-08-31 and marked **NOT DRIVEN**. It is driven now, for the first time,
and it holds.

**It PASSES.** So this particular path — a page rotation, then `Ctrl+S`, on a
plain drawing — saves, keeps the program, and keeps the document.

### ★★★ What that means, and the one thing needed to close it

The check is not the report. What it proves is that `Ctrl+S` is not broken *in
general*, which narrows the fault to something about **the edit** or **the
document**, and those are the two things this end cannot guess.

⇒ **The question, and it is one line to answer:** *what kind of edit was it?*

| candidate | why it is a candidate |
|---|---|
| a **text** edit, with the caret still in the page | `Ctrl+S` arriving while the typing guard owns the keyboard is a route nothing here drives |
| a **markup or form** edit on the canvas | needs a click, so it is not in the seam-driven path above |
| a **signed or certified** document | a save on one opens the invalidation window first, and that is a second surface between the chord and the write |
| a document opened from a path that no longer exists | `Save` becomes `Save a copy`, which opens a native picker — a different code path entirely |

Also useful, if it is quick: whether the **whole window** went, or the
**document** went and the program stayed. Those are different faults with
different fixes, and *"it closed"* fits both.

### Ruled out so far

- `Ctrl+S` is bound to `file.save` and to nothing else (manifest, asserted).
- All seven driven chords dispatch what the manifest binds them to.
- `file.save` through the harness seam saves and returns, twice, on a clean
  document and on an edited one.
- The save path contains no `panic!`, `unwrap` or `expect` outside `#[cfg(test)]`.

**Status:** ⬜ **OPEN, and blocked on one answer from you.** Everything that can
be checked without knowing which edit it was has been checked.

---


## Q3 — ⬜ TWO THINGS THE ENGINE SHIPPED TODAY THAT ONLY YOU CAN SCOPE

**Not a request of yours — a question to you**, filed here rather than asked in
conversation because that is what this file is for. `pdfcer` released **0.18.0**
on 2026-09-01 and two of its headline capabilities have **no GUI surface and no
plan for one**, because whether they should is a product decision rather than an
engineering one.

Both work today from the command line. Neither costs anything to leave alone.

### Q3a — *"You can now see inside a PDF"*

`dump-object`, `dump-structure` and `list-objects` show the object graph, where
each object physically lives, what references it, and the file's own layout.
Objects hidden inside compressed streams — invisible to any text search — are
reachable for the first time. **Acrobat has no machine-readable equivalent at
all.**

> **The question:** is that a *pdfcer* feature or a *diagnostic tool* feature?
>
> My instinct is the second, and the engine agrees — its own note says no GUI
> surface is planned. It has already been useful **to me**: the last four defect
> reports I filed against the engine each needed a reproduction I built by hand,
> and two would have been sharper with this.
>
> ⇒ **Say the word and it becomes a panel**; otherwise it stays a thing I use to
> diagnose your files when something looks wrong, and you never see it.

### Q3b — *"You can edit a PDF's internals in a text editor and compile it back"*

`export-structure` writes a readable PDF with the compression removed; you edit
it in Notepad and `import-structure` appends **only what you changed**, leaving
the original bytes untouched — so a signature over an untouched part survives.
**qpdf's own issue tracker lists that capability as unimplemented.**

> **The question:** what would *"edit the internals"* mean as a button?
>
> This is genuinely powerful and genuinely sharp. The signature-preservation
> property is the interesting half. But a menu item called *Edit internals* that
> opens 40 MB of PDF syntax in Notepad is not a feature, it is a trap — and I do
> not know what the safe shape is without knowing what you would use it for.
>
> ⇒ **If you have a case in mind, tell me the case** and I will design to it. If
> not, it stays on the command line, where the people who want it can find it.

★ **Neither is blocked and neither is waiting on the engine.** Both are waiting
on you, and both are fine left alone indefinitely.

---


## ★★★ THE DRIVEN SWEEP, 2026-08-31 — 67 / 0 / 52 on your machine

**Everything below that says BUILT has now been driven**, on `SW41177.pdf` at
`--doc-point 0,1140,62`, 119 checks, against a copy of the binary in a scratch
folder rather than the one you keep — so the suite's side effects (layout,
preferences, recent files, remembered page display) landed in
`target/sweep-scratch/userdata` and not in your own state.

| | |
|---|---|
| passed | **67** |
| failed | **0** |
| skipped | 52 |

★ **The skip count is the same as it was before today's work** — 52 either way,
with two checks moving in each direction. So none of this cost coverage. Most of
the 52 are the aim point landing on a text run, which has no anchors; that is a
fixture matter and is recorded as such.

### ★★★ It caught a regression I had shipped that morning

`resize_scales_a_shape` and `shift_constrains_a_resize` failed with *"the grip
drag committed nothing and declined nothing"* — the exact state the resize
feature exists to fix. **O72's marquee guard was second-guessing a press on a
resize grip.** A grip sits on the box's edge, outside the object's geometry, so
the "did this land on the selection?" test answered no for all eight of them and
the press re-selected instead of resizing. One `matches!(grip, Grip::Move)`
fixed all three failures.

The eight grips and the rotate handle are **drawn**. A press on one is
unambiguous and must never be re-interpreted. `Grip::Move` is the only member
with no visible mark of its own, which is exactly why it is the only one worth a
second question — and I had applied the question to the whole set.

⇒ Filed to `D:/dev/rag/egui/`, because the general form is worth keeping: when
two call sites share a predicate, extracting the predicate is half the job —
they must also agree on **when to ask it**.

### ★★ And one finding that is a consequence of a fix rather than a break

`the_line_weight_switch_reaches_the_resize` failed because the Tool panel was
**closed**. Its launch presses `view.panel_tool` believing that opens the panel;
it is a toggle, and the dock layout persists. It became reliably wrong **because
O80 wired the exit hook** — before that, a layout change in the last 750 ms
before exit was lost to the debounce, so the panel state usually did not carry
over and the check got away with it.

The program is more correct and the suite is less independent. The check now
looks before it toggles. Also filed to the RAG.

### ⬜ One flaky check, named rather than fixed

`the_format_tab_offers_font_controls_for_swept_text` skips in the suite and
passes alone — twice, having swept **266** characters and then **29**. A
text-sweep check whose count varies tenfold is measuring the harness's drag
timing as much as the feature. Not caused by anything here; it read the same way
on the pre-change binary.

### What the sweep confirms about today

`a_pan_keeps_the_fit_and_the_resize_keeps_the_position` passes with **both**
assertions, including the new falsifier: the page's drawn width went 468.0 pt →
308.0 pt across the resize, so the fit was still live after a pan. On the old
build the pan froze the zoom and that number would not have moved.

---

</details>

---

## O80 — ✅ **SHIPPED AND DRIVEN 2026-09-02** — a page-display choice reaches the next document

### ✅ Driven, 2026-09-02 — and writing the check found a defect in the disclosure first

`a_page_display_choice_survives_a_close_and_reaches_a_new_document` opens
`four-pages.pdf`, presses **Facing** on the ribbon, closes the window
**immediately with Alt+F4**, and then opens `paragraph.pdf` — **a document the
program has never seen** — in a second process. It opens facing, from the
standing preference.

```
page-display mode=facing source=preference ribbon-mode=read
```

**Falsified:** with the line that records the standing preference removed, it
fails with *"THE PREFERENCE WAS FORGOTTEN"*. Restored, it passes.

### ★★★ The trace could not tell the two answers apart, and was fixed first

There are **three** tiers — the document's own record, the standing preference,
the mode's rule — and the `page-display` line reported **two**. Anything that was
not the document's record was called `source=mode-default`, so a display that
came from **your preference** looked identical to one the mode had chosen.

⇒ That is exactly the pair this check has to separate, so a check written against
the old line would have passed against a build where the middle tier was never
read. `source=preference` now exists. **Second time in three days that writing a
driven check found a trace which could not distinguish the two states the check
existed for** — the OCR tally and the marquee census were the others.

### ★★ Why the close has to be Alt+F4 and not a kill

Dropping the harness's session **kills** the process, and a killed process runs
no exit hook. A check that killed the window and then found the preference intact
would only be asserting that the 750 ms debounce had already expired — true on a
slow run, false on a fast one. `Alt+F4` is a real `WM_CLOSE`, so `on_exit` runs.

### ⬜ One half is NOT established, and the check says so in its own report

`exit-flush layout-written=false`: the hook ran and the **layout** store had
nothing pending, because this check changes a page display and the layout file
carries the ribbon **mode**. So the *debounce rescue* is not exercised here — the
hook being called is, and the preference surviving is. Exercising the rescue
needs a mode change immediately before the close, which is a different property
and belongs in its own check.

<details><summary>The original entry</summary>

## O80 — ✅ BUILT 2026-08-31, not yet driven

> **Two causes, and the second was a function written for an exit path that did
> not exist.**
>
> ✅ **A standing page-display preference.** It was already remembered *per
> document*, written the moment you press the control — but there was no answer
> for a document the program had never seen, so a choice made on one drawing
> meant nothing on the next. Three tiers now: this document's own record, your
> standing preference, then the mode's rule. Pressing a page-display button
> records it as the standing preference, which is your sentence exactly.
>
> ★★★ ✅ **And the program had no exit hook at all.**
> `LayoutStore::flush` says what it is for in as many words — *"for an exit
> path, which must not lose the last change to a debounce that had not yet
> expired"* — and had **no production caller**. So the layout is written 750 ms
> after it changes and a change made in the last three quarters of a second
> before you closed the window was silently thrown away.
>
> That reaches you through page display: the active ribbon **mode** rides in the
> layout file, and the mode picks the default for a document with no remembered
> entry. Switch to Edit, close quickly, reopen in Read, get continuous.
>
> **Wheel paging was already persisted correctly** — status-bar toggle and
> Settings both write. One caveat found while checking: the toggle is hidden
> under a continuous display, so in Read mode the control you pressed last time
> is not on screen, which reads identically to "it forgot". The page-display fix
> above removes that.
>
> **Verified:** the prefs round trip (which caught that the writer must not
> invent a key for an unstated preference) and a precedence table. **NOT
> driven.**


**Asked:** 2026-08-31.

> *"Also it should remember my page display preferences from my last closing of
> the program. Example if I press show one page at a time and enable flip
> pages."*

### What is on disk today, and why he can still be right

Both settings **are** persisted, and they are persisted by two different
mechanisms with two different scopes — which is the most likely reason one of
them looks like it forgets:

| setting | where it is kept | scope |
|---|---|---|
| **page display** (single / continuous / facing / facing-continuous) | `viewer::remembered::remember(path, display)`, written from `Action::SetPageDisplay` | **per document path**, and only for a document that HAS a file |
| **wheel flips pages** (O30) | `Prefs::wheel_paging`, in `settings.txt` | global |

⇒ So a *page display* chosen on drawing A is remembered for drawing A and
means nothing when he opens drawing B for the first time — which is not "the
program remembered my preference", it is "the program remembered that
document". A per-document memory is right and should stay; what is missing is a
**default for a document it has never seen**.

★ And `PageDisplay::Single` is already the compiled-in default, so the half of
his example that would have looked like it works may have been working by
coincidence rather than by memory.

### What has to be checked before anything is built

1. **Is `wheel_paging` actually WRITTEN when the control is used?** It is read
   from `settings.txt` at startup and it has a Settings entry. If the *control*
   sets the in-memory value without scheduling a write, the setting survives a
   change of dialog and not a change of session — and that is exactly the shape
   of "it forgets".
2. **Is `viewer::remembered` flushed to disk at all**, or only held for the
   session?
3. **Which control is he pressing?** *"Show one page at a time"* is
   `view.page_single`; *"enable flip pages"* is the O30 setting. Both routes
   need checking, because a second route that sets the value without persisting
   it is this project's commonest defect shape.

**Status:** not investigated.

</details>

---

## O79 — ◑ HALF BUILT 2026-08-31, and the other half is a question for you

> ✅ **"The pages I picked in the thumbnails" now exists**, as a fourth scope in
> the Recognise-text window, drawn only when something is picked and labelled
> with the count. The rail's selection is already the operand for delete,
> extract, rotate and the page clipboard; OCR was the one page-scoped verb that
> ignored it.
>
> ⬜ **The half I cannot close from here: All pages and a typed range ALREADY
> EXIST**, and have since 2026-08-27, in the build you are running. All pages is
> even the default. So *"still only has a button to recognize this page"* is a
> report about the ROUTE, not about the feature — something is wrong with how
> that group presents and I cannot tell what from here.
>
> ★ The candidates are ones this project has been caught by twice: a control
> published to the harness but scrolled out of its pane, or a window whose first
> group is above the fold. There is no check asserting the scope group is
> **visible** rather than merely declared, which is its own defect.
>
### ★★★ 2026-08-31, his answer: *"there is only the option to do the page, no
### radio buttons or anything else."* — and I still cannot reproduce it

Everything checkable from this machine was checked, and every one of them says
the radios should be on his screen.

| checked | result |
|---|---|
| does the **published** build contain the radios? | **yes** — all three OneDrive slots (`pdfcer-gui1`, `pdfcer-gui2`, `pdfcer`), built 2026-08-30, contain `All pages`, `This page only` and the range hint |
| are the OCR **models** beside it? | **yes** — `models/ocrs/` with both `.rten` files in every slot |
| is he running one of those? | **yes** — `pdfcer-gui2/userdata/recent.txt` was written at 06:55 this morning |
| does the dialog draw them here? | **yes** — driven on this machine against his own `SW41177.pdf`: `ocr-scope pages=36 first=Some(0) last=Some(35)` |
| are the radios **clipped**? | **no** — `ocr-scope` occupies y 83–163 inside a 560×420 window; skip at 175, Recognise at 213 |
| is the window size remembered small? | **no** — `dialogs::host` remembers **position only**, and only for the session |

★★ **And a false lead worth recording, because it nearly became the answer.**
There are **two different programs named `pdfcer-gui.exe`** on this machine:
`D:\Builds\pdfcer-*` is the **old** GUI from `D:\Dev\pdfcer`, and
`D:\Builds\pdfcergui-*` plus the OneDrive slots are this one. Same executable
name, one character apart in the folder. That is a real hazard and it would
have explained the symptom perfectly — except that the old GUI has **no OCR at
all** (its only OCR string is a sentence saying a scanned page *"needs OCR"*),
so a window titled *Recognise text* cannot have come from it.

★ The lesson taken anyway: `ocr-scope` being **published** does not prove it is
**visible**. This project has been caught by that twice, and I reached for it as
proof before measuring the rectangle. The rectangle is what settled it.

### ⇒ The one thing left that only you can see

There is exactly one code path that draws the window **without** the radios:
`OcrDialog::preflight` refusing, which happens when the recogniser is not
compiled in or **the model files cannot be read**. It replaces the whole body
with a single sentence — no radios *and no Recognise button*.

**So: does the window say anything about model files, or about pdfcer not being
able to read them?** If it does, the models are not reachable from where you
launch it (a OneDrive "files on demand" placeholder would do exactly this), and
that is a deployment fact rather than a missing feature.

If it does **not** — if you genuinely see a working Recognise button with no
radios above it — then something is happening that nothing on this machine can
reproduce, and the next step is a screenshot rather than more of my guessing.


**Asked:** 2026-08-31.

> *"Also OCR still only has a button to recognize this page, I should have
> options to do the whole document, or the pages I have selected in the
> thumbnails."*

### ★★★ Two halves, and the first one is not what it looks like

**Whole document already ships, and so does a typed range.** `dialogs::ocr`
carries a `Scope` radio — **All pages** (the default since 2026-08-27),
**This page**, and **Pages…** with the print dialog's own range parser — and it
is in the build he is running (`d5c81a6` is an ancestor of the released
`60a84d0`). The `file.ocr` command is the only route to OCR in the program; the
Find bar's offer dispatches the same id.

⇒ So *"still only has a button to recognize this page"* is a **discoverability
report**, and per the standing rule that is a finding about the route rather
than about him. Something is wrong with how that group presents, and the
candidates are the ones this project has been caught by before: a control
published as a `ui_rect` but scrolled out of its pane, a window sized so its
first group is above the fold, or a radio set that does not read as a choice.

**It is not enough to say the feature exists.** The check that would have
caught it does not exist either: `ocr-scope` is published on every frame the
dialog is open, and nothing asserts it is **visible** rather than merely
declared. That distinction has cost this project twice.

### The half that is genuinely absent, and it is the CAD one

**"The pages I have selected in the thumbnails" is a fourth scope and it does
not exist.** `Scope` has three variants and none of them reads the Pages
panel's selection.

★ It is also the one that matters most on his own documents: a 36-sheet
SolidWorks set where four sheets are scans and the rest are vector is exactly
the case where *all* is wasteful, *this page* is four separate runs, and a
typed range is him reading page numbers off the rail and retyping them. The
panel already holds a multi-selection (`PagesPanelState::selection`), it is
already the operand for delete, extract, rotate and the page clipboard, and OCR
is the one page-scoped verb that ignores it.

**Status:** not started.

---

## O78 — ✅ BUILT 2026-08-31, not yet driven

> **All three, and the middle one resolved the way your two sentences allow.**
>
> ✅ **A resize keeps the centred point centred**, in or out of a fit. There was
> exactly one viewport-change detector in the shell and it was gated on being in
> a fit, so a document that was not in one got no resize handling at all — and
> it was worse than "the offset is kept in pixels": widening a dock by Δ slid
> the page across the screen by the *whole* of Δ.
>
> ✅ **A pan no longer leaves the fit.** Only changing the zoom does, which is
> the clause that survives in both of your sentences. It could go because
> preserving the centre defends your position in its own right — leaving the fit
> was only ever a proxy for that.
>
> ★★★ And it is a **theorem**, not a preference: on an axis a fit pins, holding
> the page's centre at the viewport centre solves to exactly the offset the fit
> would have chosen. So a fit-page document nobody has panned is re-centred for
> free, and the old resize path could be deleted rather than kept beside it.
>
> ✅ **An opened document starts centred.** Under the shipped default (fit page)
> the new expression is exactly the old one, so nothing common moves; what
> changes is an opening preference of Actual size on a sheet larger than the
> window, where you used to get the top-left corner of an A1 drawing.
>
> **Verified:** five unit tests including the theorem. The driven check was
> AMENDED rather than replaced and gained the assertion that makes it a
> falsifier. **NOT driven.**


**Asked:** 2026-08-31.

> *"when I change the size of the canvas window, whatever area was centered in
> the current canvas should stay centered, and unless I have manually changed
> the zoom after clicking one of the preset options, the pdf should maintain
> whichever option was selected - Fit Width, Fit Height, or Fit Page. Also when
> starting the view should be centered on the canvas when a pdf is first
> opened."*

Three asks, and the middle one is a **re-report against O55**, which this file
records as *"DONE 2026-08-28, driven and falsified"*. Read that row before
touching anything here.

### ★★★ And his sentence has CHANGED in one word, which is the whole finding

O55, 2026-08-28: *"unless the person has changed the zoom **or panned
around**."*

O78, today: *"unless I have manually changed the zoom."*

**Panning is gone from the condition.** That is either looseness or a
correction, and it must not be guessed at silently — `a_pan_leaves_the_fit` is
a driven check built to his earlier words, and reversing it on a misreading
would undo work he asked for three days ago.

⇒ **The first ask resolves it.** *"Whatever area was centered should stay
centered"* is a rule about **position**, and a fit is a rule about **zoom**.
Once position is preserved across a resize in its own right, dropping the fit
on a pan stops being necessary — it was only ever protecting the operator's
position from a re-placement. Both of his sentences can then be true at once,
which is the reading to build.

### The three, separately

1. **A resize preserves the centred point.** New. Nothing in the shell does
   this today: `ViewState::apply_fit` recomputes the *zoom* from the viewport
   every frame, and `canvas::fit::placement` re-places on a viewport change —
   but a document that is **not** in a fit keeps its scroll offset in pixels,
   so a wider dock shows a different part of the page.
2. **A fit survives until the zoom is changed by hand.** Half shipped (O55).
   The half that may now be wrong is which gestures leave it.
3. **An opened document starts centred.** Not checked yet.

**Status:** not investigated.

---

## ★★★ 2026-08-31 — FOURTEEN NEW ROWS, RECORDED BEFORE ANY WORK STARTED

Ken sent one message containing thirteen distinct complaints and one standing
instruction. They are split into one row each below (O64 … O77) because a merged
row gets partly dropped, which is the failure this file exists to stop. **None
of them has been investigated at the time of writing.** Each row is his words
first, then whatever is known.

The standing instruction, which governs all of them:

> *"Please don't just fix the bugs and add the features for the exact tools I am
> outlining. You need to do a proper sweep and diagnosis to ensure all tools and
> features."*

⇒ That is **O77**, and it is not a footnote — several of these rows are almost
certainly one cause wearing many faces (see O64/O13 and O76/O51, both of which
are re-reports of rows already believed shipped).

---

## O64 — ✅ FIXED BY THE ENGINE, same day, and our tests inverted

> **You can move a picture the moment you place it.** No save, no reopen.
>
> ★★★ **And the half you did not report was the dangerous one.** Chasing your
> sentence *"I assume this probably affects more than just images"* turned up a
> second symptom nobody had seen: after deleting a page, an edit made on what
> you see as page 1 was being committed to **a different sheet**, silently,
> with no refusal and no message. The engine team reproduced that before fixing
> it. It is gone too.
>
> **What it was:** every content-editing verb in the engine read the document
> *as it was on disk*, while everything that adds content wrote into the
> session. So the picture you had just placed did not exist as far as the verb
> that would move it was concerned — which is exactly why saving and reopening
> made it work.
>
> **How it went:** filed at 12:48 with a reproduction attached, answered at
> 14:25 the same day (`pdfcer-core` Pass 186.0). The three tests that proved the
> defect are now three tests that guard against it coming back, and two of them
> assert an outcome rather than an `Ok` — a verb that transformed nothing and
> reported success would be the same complaint wearing a different face.
>
> **Verified:** `cargo test --workspace`, on every commit.

<details><summary>The filing, kept — it is the record of how it was found</summary>

> **★★★ STATUS 2026-08-31 (superseded, see above): this is not our defect, and
> we have proof rather than an opinion.**
>
> `crates/pdfcer-gui/tests/engine_overlay_skew.rs` — three tests, all green,
> run by `cargo test --workspace` on every commit:
>
> 1. after `add_image` the shell's decomposition has N+1 objects and the
>    engine's own `page_objects` has N;
> 2. `transform_objects` on the new index answers `ObjectOutOfRange` — that is
>    the drag that does nothing;
> 3. **and the half nobody had reported:** after `delete_pages(&[0])` on a
>    four-page document, `page_objects(3)` still returns `Ok`. Page 3 of a
>    three-page document must be out of range. It is not.
>
> The cause is one line repeated eight times in `pdfcer-core`: every
> content-editing verb reads `page_tree::pages(&self.base)` — the document as
> it was **on disk** — while everything that adds content writes into the
> session overlay. So content added this session is invisible to the verbs that
> would edit it, and a page index computed against a page set this session
> changed resolves against the page set that was on disk.
>
> ⇒ The third finding is the serious one: after a page delete, an edit on what
> you see as page 0 is committed to **a different sheet** and returns `Ok`.
> Nothing refuses, nothing discloses.
>
> **Filed:** `open/request_edit_verbs_read_the_base_not_the_overlay.md`, with
> the reproduction attached, and indexed. `D:\Dev\pdfcer` is read-only to this
> project so the fix is not ours to make.
>
> The tests are written to **pass on the broken engine and fail on the fixed
> one**, each saying so in its own assertion message — a red test in a green
> repository gets muted within the week. When they go red, invert them and
> close this row.


**Asked:** 2026-08-31.

> *"When I add a new image to a pdf I can't edit it unless I save the document
> first, at which point it closes the document after saving. When I open it I
> can then edit the image. I assume this probably affects more than just
> images."*

★★ **This is a re-report against O13, which is recorded in this file as fixed
and driven on 2026-08-20.** O13 was *"the image does not appear until you save
and reopen"* — the page-tree walk returned early when the page object id had not
moved. The picture now appears. What he is reporting is one step further along:
the image **draws** but is **not selectable/editable** until a reload, which
points at a *different* cache — the decomposed object model
(`app::cache::page_objects`) rather than the render.

**His own generalisation is the important half** and is to be treated as the
scope of the row, not a guess: *"probably affects more than just images"*.
Every insert path must be checked — image, text, annotation, form field,
ce dimension, stamp, link, redaction mark.

**Status:** not investigated.

</details>

---

## O65 — ✅ BUILT 2026-08-31, not yet driven

> **Save never closed the document. What closed it was the step you took next,
> and the chain was driven rather than reasoned about.**
>
> A successful save recorded `saved_epoch` and **nothing in production read
> that number**. Every surface asking "does this have unsaved edits?" asked a
> different question a save cannot answer — so the tab kept its dot, the next
> Close raised the unsaved-edits prompt, and that prompt's only save button was
> "Save a copy…", a picker, which on success proceeds with the close. Press
> save, get asked for a filename, watch the document close.
>
> Now: one predicate, `save::has_unsaved_edits`, read by the prompt, the tab
> strip and the close arm. And the prompt has a **real Save** when there is a
> file to write over — absent, not greyed, otherwise.
>
> ★★ A green test was holding the old answer in place. It refused any label
> reading as a Save over the open file, *"which this build cannot do"* — true
> when written, false since Save landed on 2026-08-20, never revisited. It
> would have failed this fix.
>
> **Verified:** a truth-table unit test walking all five states. **NOT driven.**


**Asked:** 2026-08-31.

> *"… I can't edit it unless I save the document first, at which point it closes
> the document after saving."*

Split from O64 deliberately: it is a separate defect with a separate cause and
it would be lost inside the image row. Saving must leave the document open, on
the same page, at the same zoom, with the same selection.

**Status:** not investigated.

---

## O66 — ✅ BUILT AND DRIVEN 2026-08-31 — and driving it found a mirrored placement

> **The insert window now steps aside so you can point.** Press *Place it on
> the page…*, the window disappears, you click where the picture goes (or drag
> a box for its size), and the window comes back with the numbers filled in.
> Your half-typed size and everything else in it survive the trip. Escape
> abandons the placement and brings the window back.
>
> ★★★ **The driven check found a real bug on its first run, and it was mine.**
> A click was being recorded in the wrong coordinate space — screen-space y
> counts down from the top of the sheet, PDF y counts up from the bottom, and
> the click path skipped the flip. Clicking near the top of a drawing would
> have placed the picture near the bottom, mirrored. Nothing would have
> refused it, because a mirrored coordinate is an ordinary number on an
> ordinary page. The drag path had always been right; the two disagreed with
> each other and only the running program could say so.
>
> ★★ The eight unit tests under this feature were all green while that was
> true, because they asserted the numbers came out the other end rather than
> what space they were in. The check now asserts **where** the placement
> landed, not merely that one happened: 0.4 pt from the point clicked.
>
> **Verified:** driven — `the_insert_window_steps_aside_so_you_can_point`.

**Asked:** 2026-08-31.

> *"Also anything we are inserting like this should have an option in its
> dialogue box to place it with the mouse instead of by positional
> co-ordinates."*

Note *"anything we are inserting"* — this is a rule about the whole class of
insert dialogs, not a feature for the image dialog. It is built as a **shared
arm** for that reason: `canvas::placing` owns the pending record and the
gesture, `dialogs::placing` owns the button and the one derived predicate, and
a second dialog opts in with three lines and no new state.

**The one design decision:** a dialog is hidden for exactly as long as a
placement is pending for it, and *hidden* is **derived** from the pending
record rather than stored. The precedent this generalises — the Set-scale
calibration — uses a stored flag and is already broken in the way stored flags
break: press Escape mid-calibration today and nothing reopens the window. With
it derived, whatever clears the pending record un-hides the window, including
a route written next year by somebody who has never read the file. Stranding
is unrepresentable rather than merely handled.

**Today it is offered by the insert-image window**, which is the only dialog in
the crate that asks for a page position numerically. The mechanism is general;
the next dialog that needs it costs three lines.

**Status:** built, gates green, driven.

---

## O67 — ✅ BUILT AND DRIVEN 2026-08-31

> **Drop a drawing onto the thumbnails and its pages go in where you pointed.**
> The caret shows the gap while the file is still in the air, exactly as it
> does when you drag a page from another open document, and dropping several
> files at once stacks them in the order you dragged them.
>
> **Measured:** a 4-page file dropped on the left half of the second thumbnail
> of a 36-page drawing → 40 pages, inserted before page 2.
>
> ★★★ **The position does not exist in the toolkit and had to be asked of
> Windows.** `winit` receives the drop point from the operating system and
> throws it away — twice, once for the hover and once for the drop — and no
> mouse-move messages arrive during a drag either, so the toolkit's idea of
> where the pointer is was stale from before the drag started. Without that
> one syscall, *"drop it on the thumbnails"* and *"drop it anywhere"* are the
> same event and this could not have been built at all.
>
> **What a drop that is not a drawing still does:** exactly what it did
> before. An image goes to the placement window, an unreadable file opens so
> the parser can say what is wrong with it, and a drop anywhere else opens in
> a tab. Every refusal is a fall-through rather than a message, so the failure
> mode is *"it opened instead of importing"* — visible and undoable — and
> never a file that vanished.
>
> **Not driven, and said plainly:** the caret drawn while a file HOVERS. A
> harness cannot originate a drag from Explorer — that is a protocol between
> two processes — so the check holds a simulated drop back, parks the real
> cursor on a real thumbnail, and lets the application read the real position.
> The drop path is therefore driven end to end; the hover feedback that
> precedes it is not.
>
> **Verified:** driven — `a_drawing_dropped_on_the_thumbnails_becomes_pages`,
> and falsified: a build that ignores the pointer reports `gap=36` and the
> check names it as the position-blind case.

**Asked:** 2026-08-31.

> *"I should be able to drag and drop documents into the thumbnails section of
> another pdf to import the pages."*

Page drag *between two open documents* already ships (S5). This is the same
gesture sourced from the **operating system** — a file dropped from Explorer.
It reuses that gesture's geometry rather than growing a second copy: one
nearer-edge rule, one caret, one `InsertPosition`.

**Status:** built, gates green, driven.

---

## O68 — ✅ BUILT 2026-08-31, not yet driven — and it found two more

> **Merge files now works.** The engine had implemented Combine Files whole the
> entire time and nothing had ever called it; the missing half was two pickers.
>
> **Split files, Pages ▸ Split and View ▸ Sidebar are gone from the ribbon.**
> The mechanical sweep found four dead controls, not two — you had not reported
> `view.sidebar`, which was drawn FIRST in View ▸ Panels and enabled at
> startup. The three that are removed rather than built are removed because
> R9 says a capability that is not built renders nothing; the two splits need a
> boundary chooser that does not exist and come back together when it does.
>
> ★★★ **And the gate that should have caught them existed and did not fire**,
> because all four sat on an allow-list with a paragraph beside them. An
> allow-list whose entries are prose can only ever force an explanation, never
> a fix. That list is now empty and pinned at zero.
>
> **Still owed:** the durable replacement — a driven check that presses every
> registered id and fails on `command-unimplemented`. That is a claim about the
> running program, which no paragraph can satisfy.
>
> **Verified:** unit tests. **NOT driven.**


**Asked:** 2026-08-31.

> *"Also the Merge files and Split files buttons don't do anything."*

Two ribbon commands that are registered — so they are drawn, per R8 — and
either dispatch to nothing or dispatch to something that silently fails. Under
R9 a command that cannot act must not be drawn at all, so whichever it is, the
current state is wrong twice.

**Status:** not investigated.

---

## O69 — ✅ BUILT 2026-08-31, all four, not yet driven

> **The two remaining halves landed the same day.**
>
> ✅ **No box over the nodes.** The outline was stroked in an unconditional loop
> with no rung test anywhere on the path, so at the Part and Node rungs it was
> drawn on top of the very anchors you were trying to see. And the move ghost
> stroked a *translated copy* of it — for node drags too — so dragging a point
> gave you the box AND its ghost, which is O63's *"it just had a perimeter box
> around it"* surviving inside the gesture O63 was about.
>
> ✅ **The nodes are visible and hittable.** The unselected anchor was a 6 px
> HOLLOW square, 1 px of accent over black CAD linework. It is filled now, at
> 7 px, and the catch radius went to 8 — which is what a Bézier control point
> already got, so an anchor stopped being harder to hit than the handle hanging
> off it. Object picking is untouched.
>
> ★★★ **And on a dense path there were NO dots at all.** The 400-anchor cap
> counted every anchor in the path, so a contour over 400 points drew nothing
> and published no regions — you armed Points, clicked, watched the box change,
> and the program went quiet. It now counts what is **on screen**, so zooming in
> makes the dots appear, which is already the gesture you perform to work on a
> point and which made no difference at all before. And it now says so when the
> cap fires, which on that route it never did.


> **Two of your four complaints are fixed, and one of them was worse than you
> said.**
>
> ✅ **`Edit ▸ Edit Objects` is deleted.** You said we should not need it. It
> was worse than redundant: it was an alias for the **arrow tool**, so pressing
> it after arming Points put you back on the black arrow — the control you had
> been told to press in order to edit a drawing was the one that ENDED node
> editing. Its tooltip promised *"drag an anchor to move that node"*, which is
> the Points tool described exactly, by a button that armed a different one.
>
> ✅ **The Points tool is withheld outside Edit** instead of declining in
> silence. It has always needed Edit mode and its arm said nothing on screen —
> drawn, enabled and inert in two of three modes. The `A` chord still reaches
> it and now says *"Switch to Edit to work on points."*
>
> ⇒ The route is now one control: **View ▸ Navigate ▸ Points**, in the tool
> palette order every program in this class uses.
>
> ⬜ **Still open: the bounding box drawn around an object whose nodes are
> showing**, and the nodes being hard to see and hit. Both are understood — the
> outline is drawn unconditionally at every rung, and the grip radius is 6 px
> against Inkscape's 8 — and neither is done.


**Asked:** 2026-08-31.

> *"I'm still not entirely clear how to reliably get to a point where I can edit
> nodes. It seems like I click on Edit-> Edit Objects, but also have to click on
> View-> and the node selector under Navigate, then double click several times,
> but then the nodes are hard to see and click on. If we are at a point where we
> are showing the nodes in an editable state there shouldn't be a bounding box
> around the objects. We shouldn't even need an Edit Objects button. In edit
> mode I should be able to click Navigate -> select points, then single click an
> object to see its points and single click the points to edit."*

Four separate defects in one paragraph, all of which are in scope:

1. **The route is two controls in two menus** — `Edit ▸ Edit Objects` *and*
   `View ▸ Navigate ▸ node selector`. It should be one.
2. **`Edit Objects` should not exist.** Mode already says whether editing is on.
3. **A bounding box is drawn around an object whose nodes are showing.** Those
   are two different affordances for two different operations and only one
   applies at a time.
4. **The nodes are too small to see and too small to hit.** Related to O57, the
   grips row, which is still half open.

**Status:** not investigated.

---

## O70 — ✅ COMPLETE, BUILT AND DRIVEN 2026-08-31 → 09-01

> **Clicking a wrapped drawing now selects the drawing.** On a CAD sheet whose
> content was placed as one piece — a title block, a stamped detail, a symbol —
> a click used to select one line inside it and there was no way to select the
> piece at all except a Format-tab command you had to know existed. Now a click
> selects the piece and a **double-click goes inside**, which is the Inkscape
> convention you named.
>
> **The switch is in View ▸ Navigate**, beside the four pointer tools, on by
> default, and it remembers itself across restarts. Escape steps back out —
> once to drop the selection, again to leave — because losing the container you
> are working inside to a stray deselect would be worse than the extra press.
>
> **Measured, driving the real binary:** click → the container; double-click →
> `smart-enter` and the object inside it; Escape, Escape → out. The first
> Escape must NOT leave, and the check asserts that too.
>
> ★★ **A fixture had to be built for it, and the reason is worth reading.**
> Neither of your drawings could test this at the zoom they open at.
> `SW41177.pdf` contains no wrapped content at all. The benchmark site plan
> does — one container over 10,256 pieces — but pdfcer's click tolerance is six
> screen pixels, which on a sheet opened to fit is about fifteen points of
> paper, and at that radius the big page-level objects win everywhere. So the
> feature is reachable by you, zoomed in, and unreachable by a harness aiming
> at a fitted page. That is a fact about the document, not the feature, and the
> answer was a small purpose-made file.
>
> ### ✅ …and what you go inside can now be MOVED and DELETED — 2026-09-01
>
> Drag a line inside a title block and it moves. Press Delete and it goes. Both
> reach the engine as one undoable command, driven and asserted.
>
> ★ For the day between the two halves this shell could *reach* something
> inside a wrapped drawing and not move it, which is worse than not reaching
> it: the selection outline is a promise the gesture then breaks.
>
> ### ✅ …and the chain now runs all the way down — 2026-09-01
>
> Double-click again inside a container and you reach the piece's parts, with
> its points drawn, and dragging one commits. The ladder goes exactly as deep
> inside a wrapped drawing as outside one, which is what *"until a double click
> reaches the bottom"* asked for.
>
> ★★ Four things had to become true in the same commit, and any one alone would
> have made it worse: the hit test had to ask about the piece rather than a
> page position, the descent guard had to go, the points had to be drawn, and
> the drag had to reach a verb. Descending without the other three would have
> made the selection box **vanish** and offer nothing in its place.
>
> ★ And driving it caught the fifth thing. Everything above was right and the
> drag still refused — one line was still asking *"what kind of parts does this
> page object have?"* about something that is not a page object, so the answer
> was "none" and the refusal arrived after the operator had already entered the
> rung and seen the points.
>
> ### ✅ …and the drag looks the same inside as outside — 2026-09-01
>
> Dragging something inside a container shows **the shape moving**, not a box
> round it, and the curve handles are drawn and draggable there too. Driven:
> the preview builds its geometry while the drag is in flight.
>
> ★ Both were one line each once the piece could be asked what it is made of,
> which is the point of having built that first.
>
> ### ✅ …and double-clicking a text box types in it — 2026-09-01
>
> The last rung, and the one where *deeper* means something different: below a
> piece of text is **the words**, not a smaller shape. Double-click it and the
> caret lands where you clicked, with the text tool armed — which is what
> Inkscape and Illustrator both do, and what stops you being left with a caret
> blinking while the arrow is still the tool.
>
> **This row is now complete.** Every part of what you asked for in it is built
> and driven: the checkbox, the container click, the descent, the editing
> inside, the preview, the handles, and this.
>
> **Verified:** driven —
> `a_click_selects_the_whole_drawing_and_a_double_click_goes_inside`.

<details><summary>The row as filed</summary>

### ⬜ A Smart-Selector checkbox in Navigate, following Inkscape's descent convention

**Asked:** 2026-08-31.

> *"we should have a checkbox in navigate for a Smart-Selector option, in Edit
> Mode this makes it so if I click on an object and it is enabled to be selected
> in the Smart Selector it puts it in a bounding box with handles to move, resize
> and rotate, if a click selects an object that is made of multiple objects
> (group, form, etc) a double click should bring me further down the chain, until
> a double click reaches the bottom and lets me edit the nodes. If I recall this
> is similar to how Inscape does things and we should follow that convention.
> Selecting a text box or similar item does the same thing, but double-clicking
> inside the bounding box should edit the text."*

★ **He named the convention himself, so the convention is the spec** — see the
standing rule about never inventing an interaction model. Inkscape's actual
behaviour is to be checked and followed, not approximated from memory.

This row and O69 and O17 (the selection filter) are the same subject seen from
three angles. They should be designed together and shipped together.

**Status:** not investigated.

</details>

---

## O71 — ✅ BUILT AND DRIVEN 2026-08-31

> **Click a picture while reading, press Ctrl+C, and paste it into Word.** It
> arrives as a picture, not as a sentence.
>
> **Measured, driving the real binary:** a click in Read selects the image, the
> copy reports one object, and a **1237 × 1600 px** bitmap goes on the Windows
> clipboard with it.
>
> ★★★ **Two things had to be true and neither was.** The click in Read was
> swallowed by the text sweep, which owns every press in that mode — so a
> picture could not be selected at all. And the copy put a marker sentence on
> the clipboard and the real payload in pdfcer's own memory, which is exactly
> right for pasting back into pdfcer and worth nothing to Word.
>
> ★★ **And a third thing, which only driving it found.** With both halves
> built, `Ctrl+C` in Read still did nothing: the chord filter refuses a key
> whose command lives on a tab the mode does not show, and Copy lives on the
> Edit tab. The command itself was permitted in every mode; the keyboard could
> not reach it. Copy now escapes its tab — cut and paste do not, which is your
> own *copying is not authoring* ruling applied one layer further up.
>
> **What the picture is:** the selection rendered on its own, at up to 1,600 px
> on the long edge and never smaller than 1:1, composited onto white. Not a
> crop of the page — on a drawing almost everything's box overlaps a dozen
> neighbours, and a crop would paste the neighbourhood.
>
> **What it will not do, said plainly:** a picture on the clipboard cannot be
> transparent in a way every Windows program agrees about, so it comes out on
> white paper — the same white you were looking at.
>
> ### ✅ …and the right-click route, added 2026-09-01
>
> Right-click the picture and *Copy* is there, which is where Acrobat Reader
> puts it and where a hand goes without being told. It is a **two-row menu of
> its own** — copy, and zoom to it — rather than the editing menu with
> everything greyed out, because a mode that cannot delete should not draw a
> Delete.
>
> ★★ Fixing it turned up something bigger: **a right-click while reading used
> to do nothing at all.** Not the wrong menu — no menu. The check happened one
> step too early, before anything asked *which* menu, so even the zoom menu on
> blank paper was unreachable in Read and Review. Both modes now have it.
>
> **Verified:** driven —
> `read_mode_copies_a_picture_other_programs_can_paste`.

<details><summary>The row as filed</summary>

### ⬜ In Read mode the ordinary pointer must select an image so it can be copied out of pdfcer

**Asked:** 2026-08-31.

> *"In read mode the regular pointer should also allow us to select images so we
> can copy and paste them as well as text outside of the pdfcergui."*

Two halves: (a) selecting an image with the read-mode pointer at all, and
(b) `Ctrl+C` putting a **bitmap** on the Windows clipboard that Word or Paint
will accept — not pdfcer's private clipboard format. O18 covers the text half of
that and is a useful precedent for what went wrong last time.

**Status:** not investigated.

</details>

---

## O72 — ✅ BUILT 2026-08-31, not yet driven

> **The rubber band existed, worked, and was unreachable on a CAD sheet.**
>
> A band is already what a press means when it finds nothing to grab. What made
> it unreachable is that a selection's grab box is its **bounding** box — so
> select a title-block border once, which on your drawings is a hollow
> rectangle spanning the sheet, and every press anywhere on the drawing fell
> inside that box and became a move.
>
> A press over page content now has to land on something selected before it
> counts as a move. Resize and rotate grips are untouched — those are drawn,
> you can see them, and second-guessing a press on one would break a gesture
> that works.
>
> **Verified:** unit tests only. **NOT driven.**


**Asked:** 2026-08-31.

> *"Click and hold shouldn't select an object - it should allow me to draw a box
> around objects to select."*

**Status:** not investigated.

---

## O73 — ✅ BUILT 2026-08-31, not yet driven

> **Paste follows the cursor.** The pointer was not read, not threaded, and not
> present in any paste signature — there was one rule, ten points down and
> right, and that was all of it.
>
> The clip is **centred** on the cursor, which is Inkscape's rule and
> Illustrator's; you are pointing at where the thing should be, not where its
> bounding box should begin. One anchor for the whole clip, so a multi-object
> paste keeps its arrangement by construction.
>
> With the pointer over a panel or off the window it pastes into the middle of
> the view, through the same function the zoom anchor uses — one rule, not two.
> Markup, page content and form fields all go through it.
>
> **Verified:** unit tests only. **NOT driven.**


**Asked:** 2026-08-31.

> *"When I cut or copy and object, when I paste it should paste where the mouse
> cursor is sitting."*

**Status:** not investigated.

---

## O74 — ✅ BUILT 2026-08-31, not yet driven

> **Measured before it was built: on your own 36-sheet SolidWorks set, twelve
> visible thumbnails cost 666 ms of UI-thread work after every single edit,
> worst frame 282 ms — all of it between your click and its result.**
>
> The cause was a document-wide counter used as the invalidation key for four
> caches that hold one entry per page. An edit to sheet 12 threw away the
> pictures of the other thirty-five. The same bug was on the full-size page
> rasters, where one page costs ~950 ms on the benchmark drawing.
>
> ✅ **Per-page invalidation**, opted into per verb with document-wide as the
> safe default — a thumbnail kept wrongly would show you content you had
> already changed, which is worse than the slowness.
> ✅ **And your priority rule**, which is worth more: the rail now waits for a
> quiet moment — no input this frame, and 250 ms since the last edit. It can
> never sit between a click and its result.
>
> ★ The design had a hole and its own test found it: two independent counters
> compared with `max` do not compose, so a document-wide invalidation silently
> skipped exactly the pages most recently edited. Both bumps now draw from one
> counter.
>
> **Verified:** unit tests, both directions. **NOT driven — the win is a
> measured prediction until the harness runs.**


**Asked:** 2026-08-31.

> *"When I make edits or even just fill out a form I notice all of the page
> previews get re-rendered instead of just the one that is being changed, and it
> seems to really slow down clicking a checkbox in a form. The last thing that
> should matter is updating the preview, and it should just update the pages that
> were actually altered."*

★★ He has given the priority rule as well as the bug: **the thumbnail is the
lowest-priority work in the program.** It must never be on the path between a
click and its result. This is the same complaint as the 430 ms commit under O63
arriving from a second direction, and the two should be measured together.

**Status:** not investigated.

---

## O75 — ✅ BUILT 2026-08-31, not yet driven

> **The second half landed the same day** — see below the click-through note for
> what it turned out to be. The document section now starts collapsed whenever
> something above it is describing your selection, and it is collapsed rather
> than hidden because the document form is a real capability you may want.


> ✅ **The click-through is fixed, and it was one guard.**
>
> The canvas asked the **whole window** "did the primary button go down this
> frame?", took the press position from the same place, and mapped it straight
> through the page transform. That transform is unclamped, so any screen point
> converts to a valid page coordinate — a press on a text field in the
> Properties panel resolved to real page content and replaced the selection you
> were editing the properties of.
>
> It hid at fit zoom, where the dock maps off the sheet. Zoom past fit on a CAD
> sheet — every working session on an A1 — and the whole window maps inside the
> page. That is why it went unreported until you zoomed in.
>
> ★★ Same class as the Delete-key defect in `DEFECTS.md`: a guard asking
> exactly the right question of exactly the wrong object. Every other signal on
> the canvas already came from the page's own response; this one step reached
> past it. One term fixes it, and it rejects both docks, the ribbon, the tab
> strip, the status bar, the find bar, context menus and modal dialogs at once.
>
> ⬜ **Still open: Properties showing "This document".** The diagnosis is not
> what the row assumed — the selection-scoped sections DO exist and DO read the
> selection. What is true is that the document section is drawn
> **unconditionally**, so it is on screen every frame and is the only thing on
> screen whenever nothing else claims it — and the click-through above was what
> kept putting you in that state. Not yet collapsed.


**Asked:** 2026-08-31.

> *"When I am working in the right side panel objects are getting selected
> through the side panel when I am trying to edit fields in the Properties
> section. Also the Properties section is always showing the This document
> properties instead of just the properties of the objects I am editing."*

Two defects:

1. **Click-through.** A press inside a docked panel is reaching the canvas hit
   test. This is a layer/consumption bug and is a strong candidate for the same
   class as the Delete-key guard from `DEFECTS.md` — a condition asked of the
   wrong object.
2. **Properties shows the document, not the selection.** O39 shipped
   *"clicking a field shows its properties"*; this says that regressed or was
   only ever true for form fields.

**Status:** not investigated.

---

## O76 — ✅ FIXED 2026-08-31 — the engine answered and both routes are wired

> **A check box dragged bigger keeps the border weight it had.** The outline
> stops thickening with the box.
>
> **What it actually was — neither of the two things this row first guessed.**
> pdfcer was not writing a fatter border. It was not rewriting the artwork *at
> all*: the engine redrew a field's appearance for text and dropdown fields
> only, and a check box is a button, so the picture pdfcer had drawn at the old
> size was kept and simply stretched into the new box. Drag a 12 pt check box
> to 40 pt and its 1 pt border draws at about 3.3 pt.
>
> **Filed at 12:49, answered at 14:26** (`pdfcer-core` Pass 187.0), and scoping
> it found three more defects nobody had reported: text and dropdown fields
> were being rebuilt at the OLD size, a push-button caption change redrew
> nothing, and a resize of a field drawn in several places was silently
> discarded.
>
> **Our half, wired today:** your three resize switches now reach a form field
> at all — by the drag AND by typing numbers in the Properties panel, from the
> same answer, because a second route that quietly used a different setting is
> how the two drift.
>
> **Measured, driving the real binary:** a check box dragged from 190×146 pt
> outward reports `regenerated=true` — the appearance was rebuilt at the new
> size. A screenshot could not have told you that: a scaled 1 pt border and a
> redrawn 3 pt one are the same pixels.
>
> ⬜ **Still open, and it is a feature rather than a bug:** scaling the radii of
> rounded corners. A form field's border carries no radius and pdfcer's check-box
> artwork has square corners, so there is nothing to scale until a
> rounded-rectangle primitive exists. That is worth scoping with you rather
> than improvising under a bug report.
>
> **Verified:** driven — `a_resized_check_box_is_redrawn_not_stretched`.

<details><summary>The half-built state this row passed through, kept for its diagnosis</summary>

### ◑ HALF BUILT 2026-08-31 — the honesty half; the fix was an ENGINE row

> **The cause is neither of the two this row guessed.** The outline does not
> thicken because pdfcer wrote a bigger border width. It thickens because
> **nothing was rewritten at all**: the engine redraws a field's appearance for
> Text and Choice fields only, and a check box is a button — so the appearance
> pdfcer itself drew, at the ORIGINAL size with a hard-coded 1 pt stroke, is kept
> and the PDF placement matrix stretches it into the new box. Drag a 12 pt check
> box to 40 pt and its 1 pt border draws at about 3.3 pt.
>
> ★★ That is exactly the case the ANNOTATION resize **refuses by name** — *"a
> foreign appearance cannot be rebuilt without replacing somebody else's artwork
> with pdfcer's rendering of it"* — and the widget path takes it silently, on
> artwork pdfcer drew and could therefore rebuild exactly.
>
> ✅ **Shipped: pdfcer stops lying about it.** The sentence was chosen on
> "was it resized" and said *"its contents were redrawn to fit"* — a claim the
> very outcome it was reading denied on the next field. There is now a third
> case saying the contents are stretched, plus the trace this verb never had,
> which its two siblings have always emitted.
>
> ⛔ **The fix is the engine's** and is filed:
> `open/request_resizing_a_check_box_stretches_its_appearance.md`. It asks for
> two things — that a button's appearance be redrawn after a `/Rect` change, and
> that `WidgetEdit` carry the same three scale answers `ResizeOptions` already
> takes, so your Tool-row switches reach a form field at all.
>
> ⬜ **Corner radii: nothing to scale, and that is a finding rather than a
> refusal.** A form field's border style carries no radius, and pdfcer's own
> check-box artwork draws square corners. What you are seeing thicken is the
> square artwork, which the engine row fixes. A genuine "scale rounded corners"
> toggle needs a rounded-rectangle primitive to exist first, and that is a
> feature to scope with you rather than improvise under a bug report.

**Asked:** 2026-08-31.

> *"Form shape outlines of checkboxes and such scale when I drag them larger.
> There is supposed to be an option on the menu to choose the behaviour of
> resizing items - when scaling objects scale stroke width by the same proportion
> and when scaling rectangles scale the radii of the rounded corners"*

★★ **Re-report against O51**, *"Inkscape-style scale toggles: line weight follows
a resize, if you say so"*, recorded as shipped. Either the toggle does not reach
the form-field resize path, or its default is the wrong way round, or it is not
findable. The corner-radius half appears not to exist at all.

**Status:** not investigated.

</details>

---

## O77 — ★★★ ⬜ The standing instruction: sweep everything, do not fix only what was named

**Asked:** 2026-08-31.

> *"Please don't just fix the bugs and add the features for the exact tools I am
> outlining. You need to do a proper sweep and diagnosis to ensure all tools and
> features."*

This is the same instruction that produced O14 (fourteen gaps found by asking
what an application of this class must have) and O56 (confirm every editable
surface the engine implements has a surface in the GUI). It is now standing:
**a reported defect is a sample, not a specification.** Every row above is to be
generalised to its class before it is fixed, and the classes are to be swept.

**Status:** in progress from 2026-08-31.

---

## O63 — DONE SO FAR, 2026-08-30 (read this before the analysis below)

All three pieces he approved are **built, gate-clean and DRIVEN**:

| piece | what it does | proof |
|---|---|---|
| **the shape** | drag a line's end and the line bends; the real geometry, at pointer speed, no engine call | `dragging_a_node_bends_the_line` |
| **the erase** | the object's old footprint stops showing, so it is not on screen twice | same check (`erased=` on `canvas-shape-drawn`) |
| **the hold** | the preview outlives the release until the raster catches up, so nothing snaps back | same check, third assertion |

★★ **And the check was falsified**, which is the part that makes the green mean
anything: run against the **previous release** — which has none of this — it
fails on the first assertion and names it. A check that passes on both builds
would be measuring nothing.

★ `turning_a_field_right_turns_it_right` also passes now (O62), asserting
**270** after one right turn. The foreground rights that blocked every driven
check came back after a Windows restart.

### What is still open under this row

1. **Everything that is not a canvas gesture.** *"Live preview for everything we
   do"* also covers a colour change, a Bold press, a delete, a redaction mark.
   None of those has a sprite to slide and none is covered yet — see the
   analysis below on why a *rendered* preview is a second away.
2. **The engine's 430 ms commit, on the UI thread.** Filed; unanswered at the
   time of writing. Until it moves off the UI thread the window still stops
   answering the pointer for half a second per edit on a dense drawing, and no
   preview fixes that.
3. **Saying the page is catching up.** The third piece holds the picture; it
   does not yet say why. One sentence on the status line, owed.

---

## O63 — ◑ The program keeps up with your hand on the CANVAS — the rest is open

**Ken, 2026-08-30:** *"we need to make it so we have a live preview as we drag
and move and resize and rotate, etc around the canvas. The live preview should
remain while the update to the pdf structure runs in the background. This should
just cache each one as the user does their edits so to them everything looks
WYSIWYG and the delay in updating the actual isn't noticable. If the user gets
too far ahead, then it will pause and update."*

**Ken, clarifying, 2026-08-30:** *"to clarify live preview request is for
everything we do."*

★★★ **THE CLARIFICATION CHANGES THE SUBJECT, AND IT IS THE WHOLE POINT.**

The first message names four gestures and reads as a canvas-manipulation
feature. It is not one. *Everything we do* means every edit that changes what is
on screen: a colour, a size, a Bold press, a delete, a redaction mark, a field
moved, a page rotated, a bookmark, an annotation. **The unit of work is not "a
drag" — it is "an edit".**

⇒ That rules out the cheap answer before it is proposed. Compositing the dragged
object over the old page texture solves four gestures and **nothing else**;
there is no sprite to slide when the operator changes a fill colour or deletes a
run of text. Whatever is built must be general over edits, or it is a fifth of
the request wearing the name of the whole thing.

**Not started.** Raised while O62b was being fixed; this is the largest single
item on the list and it is architectural rather than additive.

### ★★★ THE THIRD MESSAGE IS THE ONE THAT MATTERS

**Ken, 2026-08-30:** *"yeah do both. but to be clear at least last time I checked
if I moved the end of a line, it didn't show me the shape change of the line, it
just had a perimeter box around it. this goes for anything I change right now.
there isn't a real preview like there is in inkscape."*

★★★ **He is right, and it means the first two messages were being read too
narrowly.** The complaint is not that the preview arrives late. It is that
**there is no preview of the SHAPE at all** — every gesture in this shell draws a
*bounding outline* and nothing else. Drag a line's endpoint and you get a
rectangle that changes size; you never see the line bend.

That is not an oversight. It is a **written convention being followed**, stated
at `canvas/handledrag.rs:216-222`:

> *"a preview shows the cursor, the render shows the document."*

⇒ **That convention is now overruled by the operator, explicitly, by
comparison to Inkscape.** It should be recorded as reversed rather than quietly
contradicted, because it is repeated in several modules and the next session
will otherwise re-derive it.

### ★★ AND IT MAKES THE PROBLEM EASIER, NOT HARDER

The whole analysis above assumed a preview had to come from the **rasteriser**,
which is why *"a two-pixel render costs 691 ms"* looked fatal. It does not.

`vector::decompose_page` already gives this shell the real geometry, and the
shell already caches it (`app::cache::page_objects`):

| what it carries | why it is enough |
|---|---|
| `PathObject::page_subpaths()` | page-space `Line` and `Cubic` segments, control points resolved |
| `style`, `line_width` | fill/stroke disposition and width |
| `fill_color`, `stroke_color` | the actual colours |

⇒ **egui can draw that directly, at pointer speed, with no engine call at all.**
A moved node, a resized box, a rotated shape — transform the cached geometry in
memory and paint the real path. That is exactly what Inkscape does, and it is
*not* fuzzy: for geometry it is **exact**.

★ Precedent already in the tree: `painting.rs:437-450` draws page-space segments
for the ce-dimension placement preview. The mapping and the painter exist; what
is missing is feeding them the selection's own geometry.

### The design, now that both halves are known

Three pieces, and he asked for all three (*"yeah do both"*, plus the shape):

1. **Draw the real geometry during the gesture.** Transform the cached
   `PageModel` objects by the live delta / scale / rotation / node move and paint
   the actual paths. Replaces the bounding outline as the primary affordance;
   the outline stays as the *selection* indicator, which is what it is for.
2. **Occlude the old position.** The stale raster underneath still shows the
   object where it was, so without this the operator sees it twice. This is the
   *"lift the pixels"* half and it is the one with a failure mode — the hole has
   to be filled with something, and on a CAD sheet there is usually content
   underneath.
3. **Keep it up until the fresh raster lands, and say so.** The preview is
   retained against the epoch the commit produced and dropped when the texture
   carrying that epoch arrives; the status line says the page is catching up.
   This is what removes the *snap-back*.

★★ Piece 1 is the one he actually asked for and the one with no downside. Pieces
2 and 3 are the O63-as-originally-read half. **Build them in that order** — 1 is
useful on its own and cannot be wrong; 2 is the only one that can lie.

### ★★★ MEASURED, 2026-08-30 — AND THE COMFORTABLE ANSWER WAS WRONG

`crates/pdfcer-gui/src/app/actions/latency.rs`, release build, on the 5.6 MB CAD
site plan. Written before any design work, because the last time this project
answered a performance question from architecture it was wrong.

| call | ordinary A1 title block | **dense CAD site plan** |
|---|---:|---:|
| `Document::load` | 0.4 ms | **3.6 ms** |
| `EditSession::view` | 0.000 ms | **0.000 ms** |
| `decompose_page` | 0.5 ms | **500.9 ms** |
| `move_objects` | 0.9 ms | **434.3 ms** |

**The prior was "it is obviously the raster". It is not.** One drag-move on this
drawing costs the engine **430 ms before anything is drawn at all**, and it runs
**on the UI thread** — so it is not a delay, it is a **freeze**: the window stops
answering the pointer for half a second, per edit, on a drawing he uses daily.

Three findings, in order of how much they change the plan:

1. **Opening is free, reading is free.** 3.6 ms to load 5.6 MB; `view()` is
   unmeasurable. ⇒ the cost is **the content stream**, not the file and not the
   object graph.
2. **`decompose_page` (501 ms) and `move_objects` (434 ms) are within 15 % of
   each other**, which reads as *the verb's cost is essentially one
   decomposition*. If so, every content edit on this page carries the same
   ~450 ms floor — moving one line costs what moving ten thousand costs.
3. ★★ **The shell then pays for a second one.** `app::cache::page_objects` is
   keyed on `(page, edit_epoch)` and the commit bumps the epoch, so the
   decomposition is discarded at the moment the verb returns and rebuilt on the
   next frame. **~500 ms of duplicated work per edit, pure loss** — the same
   stream parsed twice because the two parsers cannot see each other across the
   crate boundary. Filed:
   `request_one_edit_costs_two_decompositions_of_the_same_page.md`.

### The bill for one drag-move on that drawing

| step | cost | thread |
|---|---:|---|
| `render_worker.cancel_and_wait()` | 28.9 ms | **UI** |
| `move_objects` | ~430 ms | **UI** |
| re-`decompose_page` for the selection outlines | ~500 ms | **UI** |
| re-rasterise the page | ~1,000 ms | worker — stale frame stays up |

**≈ 1 s frozen, then ≈ 1 s stale.** Only the last row is already handled.

### ★★ AND THE PREVIEW IS DISCARDED AT EXACTLY THE WRONG MOMENT

Found while mapping the canvas. Every drag module returns before touching the
document while the gesture is in flight (`moving/mod.rs:722`, `resizing.rs:521`,
`rotating.rs:340`, and five more), draws an **outline ghost** at the new
position, and on release **discards the ghost and raises the Action**.

But the raster underneath still shows the **old** position for the next second
or two. So what the operator sees on release is:

> the ghost vanishes → the object is back where it started → *(a pause)* → the
> object jumps to where they put it.

★★★ **The object appears to snap back.** That is very likely the largest part of
what he is describing, it is not a rendering problem, and it is exactly his own
sentence: *"the live preview should remain while the update to the pdf structure
runs in the background."* The preview must **outlive the commit** — retained
against the epoch the commit produced, and dropped only when the raster carrying
that epoch lands.

### ★★★ AND THE OBVIOUS FIX IS DEAD — re-rendering just the changed region buys nothing

The natural plan, once the numbers above are in front of you, is: *after the
commit, re-raster only the area that changed — old bounding box ∪ new bounding
box — blit it into the standing texture, and do the full page behind that.* The
machinery for it already exists (`RenderKey::region`, `render/strip.rs`).

**It does not work on this drawing, and this project already measured why.**
From the superseded-tiling note earlier in `BENCHMARK.md`, measured by the
engine team with `render_page_region`:

| case | pixels | time |
|---|---:|---:|
| full page, scale 1 | 1,002,822 | 877 ms |
| region 400 × 300 pt | 120,701 | 699 ms |
| **a 1 × 1 POINT region** | **2** | **691 ms** |

**A two-pixel render costs 691 ms.** On a dense CAD sheet ~99 % of render cost
is **area-independent** — it is content-stream interpretation, not fill. So a
region render of a moved object costs essentially what the whole page costs.

⇒ On this drawing there is **no way to produce a correct picture in under ~0.7 s
after any edit**, by any arrangement of the existing renderer. That is not a
shell problem and it is not fixable by scheduling.

### ★★ WHICH FORCES THE REAL QUESTION, AND IT IS A TASTE DECISION, NOT A TECHNICAL ONE

If an exact picture is unavailable for ~1 s, then what he is asking for is
**necessarily a lower-fidelity transitional picture** — and the only question
left is *what the honest fuzzy looks like*.

The project already has the vocabulary and the precedent. `backdrop.rs` keeps a
low-resolution whole-page texture and paints it under the sharp one, and states
the rule: **"fuzzy is allowed, sneaky is not."** A *less sharp* version of the
truth is fine. A *differently-meaning* version is not.

Three candidate answers, and **this is an operator call**:

1. **Outline only, held until the raster lands.** Cheapest, honest, already
   drawn — but the stale raster still shows the object at its **old** position
   underneath, so the operator sees the object in two places at once. Probably
   worse than what happens today.
2. **Lift the pixels.** Copy the object's rectangle out of the current texture,
   paint it at the new position, and fill the hole it left with the page
   background. Looks genuinely WYSIWYG for a move. ★ **The hole is a lie
   whenever something was underneath the object** — on a CAD sheet, usually
   something is. It is fuzzy *and* slightly sneaky, and it is the option that
   will look best in the ordinary case and worst in the surprising one.
3. **Freeze the picture and show that work is happening.** Do not attempt a
   preview at all; make the ~1 s legible instead of invisible. Cheapest to get
   right, and the only one with no failure mode — and the one furthest from what
   he asked for.

★ Note (2) and (3) are not exclusive: lift the pixels *and* let the status line
say the page is catching up.

### What already exists, and is better than expected

* **A stale-frame fallback, three tiers.** `funnel.rs:60-72` (an edit no longer
  blanks the texture — that fixed *"the page goes blank and flashes after every
  change"*), a 12 ms inline render budget then async (`worker.rs:706`), and a
  kept low-resolution whole-page backdrop under the sharp one (`backdrop.rs:88`).
* **Region rendering.** `RenderKey::region`, `render/strip.rs`, and a painter
  already willing to draw the raster at a rect other than the page's
  (`present.rs:726-757`).
* **One gesture is already one undo entry**, decided in the gesture modules and
  enforced by the plural verbs taking slices (`moving/mod.rs:9-16`).

### The obstacle nobody had written down

`vector_edit` **begins** with `doc.render_worker.cancel_and_wait()` — it *joins
the render thread*, because `Arc::get_mut(&mut doc.session)` fails while a
worker holds a clone. Measured at **28.9 ms**. So any design that commits
per-frame during a drag would cancel and join a render **every frame**.

⇒ That rules out "commit continuously and let the engine catch up" in its
simplest form, and it is why the engine has been asked whether a mutating verb
can run off the UI thread at all.

### The old plan, kept for the record

### ★★ MEASURE FIRST, AND THE MEASUREMENT DECIDES THE DESIGN

`BENCHMARK.md` exists because an earlier session asserted a performance weakness
from architecture and was wrong. The same trap is open here, so the first job is
an instrument, not a plan. The question is **which half is slow**:

| candidate | what it would mean |
|---|---|
| **(a) the engine commit** — `EditSession`'s verb rewriting content streams and the object graph | his description is right as written: an optimistic edit model with a queue and backpressure |
| **(b) the re-rasterisation** — `pdfcer-render` redrawing the page after the edit epoch moves | a much smaller and much safer change: keep showing the last good frame, render the new one behind it, swap when ready. No optimism, no queue, no divergence between screen and document |
| **(c) both** | (b) first, because it is cheap and it may be the whole of what he can feel |

★ On a 129,758-object CAD sheet the prior is strongly **(b)** — but a prior is
not a measurement, and `tools/render-profile` is the standing instrument.

★★★ **If it is (b), most of the risk below evaporates.** The document is never
ahead of or behind the screen; the screen is simply a frame or two stale, which
is what every drawing program does. No refusal can arrive for an edit already
shown, because the edit really did happen before the frame was requested.

### The open questions, in the order they have to be answered

1. **Which half is slow.** See above. Everything below is conditional on (a).
2. **What the preview IS** for a non-geometric edit. There is no sprite for
   "this text is now bold"; the only general preview of an edit is *the page
   rendered with the edit applied*, which is the expensive thing being deferred.
3. **What happens when the engine REFUSES** an edit the preview already showed.
   This has no honest answer yet and it decides the design. Note it is
   **entirely a problem of (a)** — under (b) the engine has already accepted
   before anything is drawn.
4. **The queue depth**, and what *"pause"* looks like. A frozen pointer is worse
   than a slow one.
5. **Undo grouping.** One entry per gesture, not per frame — and `EditSession`
   has no grouping verb, which is already filed.

### ★★★ Rule 4 binds this hard, in the direction that is easy to get backwards

A preview must render **exactly** as the committed result will render. No ghost,
no outline, no provisional tint, no dashed rectangle, no "pending" badge. His own
words, recorded when redaction was scoped: *"the nagging and red flagging in the
original GUI made for a lot of extra bugs in the visibility when editing."*

A preview drawn differently from the commit is a **second rendering path for the
same content**, and two paths drift. The one-line test: *would a screenshot of
the canvas mid-preview differ from a screenshot of the same document after the
commit lands?* If yes, and the difference is pdfcer marking its own uncertainty,
that is the defect.

★★ What he described is not a rendering optimisation, it is a **decoupling**:
the picture the operator is dragging and the document pdfcer is rewriting stop
being the same object. The screen follows the pointer at pointer speed; the
engine catches up behind it; and the queue between them has a depth, past which
the shell stops accepting input until the engine is level.

★★★ **Rule 4 binds this hard and in a direction that is easy to get backwards.**
A preview must render *exactly* as the committed result will render — no ghost,
no outline, no "provisional" tint, no dashed rectangle. The operator's own
words, recorded when the redaction work was scoped: *"the nagging and red
flagging in the original GUI made for a lot of extra bugs in the visibility when
editing."* A preview drawn differently from the commit is a second rendering
path for the same content, and two paths drift.

Open questions to settle before any code:

* **what the preview IS** — a re-rasterised page, or the existing page texture
  with the moved object composited over it? The second is far cheaper and is
  only correct while the object does not interact with what is under it;
* **the queue depth**, and what "pause" looks like. A frozen pointer is worse
  than a slow one;
* **what happens when the engine REFUSES** a move the preview already showed —
  this is the case that has no honest answer yet, and it is the one that decides
  the whole design;
* whether undo sees one entry per gesture or one per frame. It must be one per
  gesture, and nothing in `EditSession` groups entries.

---

## O62b — ✅ Bold stopped using real bold fonts, and nothing failed

**Found, not reported.** It arrived inside the same `cargo update` that built
the O62 release, and it is the reason that release was rebuilt.

### What was wrong

Press **Bold** on a page that carries a real bold face — a title block set in
Calibri with Calibri-Bold sitting right there in the page's own font list — and
pdfcer **thickened the Calibri instead of using the Calibri-Bold**. Artificially.
Into the saved file. Every other viewer would show the fake.

### Why nothing caught it

The engine's `Pass 179.0` (2026-08-30) changed what asking for a synthetic bold
*does*. It used to **refuse** when a real face was available, naming the face —
and this shell was built on that refusal: it asks for the fake, reads the
refusal, and immediately re-asks for the real face the refusal names. One Bold
button, working on every page, built out of a decline.

The engine's new default applies the fake and reports the face it passed over.
So the refusal stopped arriving, the retry stopped happening, and the button
quietly became worse. **Nothing failed.** The verb succeeded, the page changed,
the epoch moved, the disclosure was written. One unit test caught it, and only
because it asserts the resulting face **by name** rather than asserting that
something changed.

⇒ ★★★ The finding worth keeping is not the bug, it is that **a shell built on a
refusal is a shell that breaks when the refusal is upgraded into a success.** A
decline is an API surface like any other, and this one changed without the
engine or this project noticing that it had.

### What it is now

The Bold button asks the engine with *"refuse if a real face exists"* pinned on,
whatever the operator's settings say — because for this shell the refusal is a
**question**, not an answer. It is never shown to anybody: the next thing that
happens is taking the offer it names.

★★ **And a third rung was added that was never there.** Where the named real
face turns out not to be able to show the text — `Times-Bold` has no `o`, so it
cannot set *hello world* on a page where `Calibri-Bold` can — the old shell
simply gave up and told the operator to use a face that had just failed. It now
fakes the weight and says which real face it tried and why it could not.

### The setting that came with it

**Settings ▸ Fonts ▸ Faking bold and italic.** Three choices: fake it quietly
(the default), fake it and say so plainly, or never fake it. A real face is
preferred under all three — that is stated under the group, because *"never fake
it"* reads like *"never change my font"* and is not.

### ★★ The instrument that found it, and the instrument that hid it

The settings window has a test that refuses to compile past a setting the engine
honours and the window cannot reach. It fired on `style_policy` within one
`cargo update` — third time it has caught exactly this.

Then it kept firing **after the control was written**, because its list of files
to search is hand-written and `fonts.rs` was not in it. A check that cannot see
a file reports that file's contents as absent, which looks identical to the
defect it exists to find. The list had a comment from two days earlier saying
*"if a third module is ever added, this line is the one to remember"* — a note
asking a future session to remember something is not a mechanism, and it failed
exactly as written. **Both hand-written lists are now derived-checked** against
the modules the directory declares.

---

## O62 — ◐ Turn a form field's box · Say something other than the measurement — BUILT, NOT DRIVEN

**Ken, 2026-08-30:** *"finish those 2 then release."*

Both are built, gate-clean and unit-tested. **Neither could be driven**, and the
reason is the machine rather than the code — see the last section, which is the
part that matters most.

---

### Turn a form field's box

In the field's **Properties**, beside its position and size: **Turn left** and
**Turn right**, a quarter turn each. It turns what is drawn *inside* the box; the
box itself stays where it is, and the hint says so because that is the surprise.

★★★ **The direction was the whole risk.** In PDF, a widget's rotation is
**counterclockwise** while a *page's* rotation is **clockwise** — and the
standard's two sentences are word for word identical apart from that one word.
The engine flagged it as *"the single most likely thing for a shell to get
backwards"*. A shell that missed it would ship two buttons that both work, both
write a legal angle, and both turn the box the wrong way, **with nothing
failing anywhere**.

So the controls say *left* and *right* — what you watch the box do — and the
negation happens in exactly one line, at the panel, which is where the engine
asked for it.

★ **The control moved while I was testing it.** With every properties section
drawn, it landed at y=1379 in a window 768 points tall: scrollable-to, but four
unrelated sections down. It now sits with position and size, which is where it
belongs anyway — turning a box is geometry, not captioning.

### Say something other than the measurement

A ce dimension's properties gained a text box. Type in it and the ce dimension
shows your words; clear it and the measurement comes back.

★★ **It does not change what was measured**, and that is the design rather than
a caveat. The measurement stays underneath, so clearing the box restores
**exactly the number that was there** — not a fresh calculation that might round
differently. The receipt names the number both times, going on and coming off,
because that is the only way to confirm it rather than trust it. On a drawing,
text that *replaces* a measured value and text that *changed* it are a note and
a lie respectively.

There is no Clear button: emptying the box **is** the restore.

### ⬜ The attachment clipboard, deliberately not done

The third item on the list I gave you, and you said two. It is also the least
valuable of the three: pasting an attachment is what *Attach a file* already
does, so the only new capability is lifting one out of a document to put in
another. Say the word.

---

### ★★★ NOT DRIVEN, AND THE EVIDENCE SAYS IT IS THE MACHINE

Every input-driving check now reports:

> *the window containing (1224, 538) could not be brought to the front. Windows
> refuses `SetForegroundWindow` to a process without foreground rights.*

**Including checks that passed an hour earlier in this same session, unchanged.**
`a_form_field_can_be_copied_and_pasted_both_ways` was green at 13:0x and skips
identically now. That is the tell: it is not the new code, it is that this
machine has stopped granting the harness the foreground.

⇒ So the two features above are **built and gate-clean and unverified by
driving**, and that is stated in those words rather than softened. R1 is the rule
this project was founded on: *"the tests pass" is not a report of working
software.* Two new checks are written and registered and waiting —
`turning_a_field_right_turns_it_right` asserts the rotation is **270** after one
right turn, which is the single number that catches a missing negation.

**Run them first thing next session** — the fifth documented way to run this
suite wrongly is a busy foreground, and it clears.

**Status:** ◐ **BUILT AND RELEASED, NOT DRIVEN.** Two checks written, blocked on
the machine, first job next session.

---

## O61 — ✅ pdfcer tells you when a document phones home · ✅ AND buttons can now be given actions

> ### ★★★ CLOSED 2026-09-01 — you can make a button do something, seven ways
>
> **Draw a button and pdfcer asks what pressing it should do.** Seven answers:
>
> | | reaches |
> |---|---|
> | Nothing | — |
> | Clear the form | nothing outside the document |
> | Go to a page | nothing outside the document |
> | Move through the pages (next / previous / first / last) | nothing outside the document |
> | Show or hide fields | nothing outside the document |
> | Open a web address | writes an address; pdfcer never opens it |
> | Send the form's data | writes an address and a declaration; pdfcer sends nothing and has no way to |
>
> Every one of the seven says which of those it is, in a sentence under the
> chooser — including the five that reach nothing, so the two that do are not
> the only ones carrying a line. A disclosure that appears only on the risky
> choice is one people learn to skip.
>
> **The submit says four things nobody can guess**, and Acrobat says none of
> them: hidden fields are sent, fields whose characters are masked are sent as
> plain text, a field that names a file on your computer sends that file's
> contents, and the message carries this document's own location on disk.
>
> ★ **No web address is blocked.** An unencrypted one is *said* to be
> unencrypted and you decide. Blocking it would be pdfcer inventing a rule the
> standard does not state.
>
> ★★ **And nothing is marked on the page.** A button that submits looks exactly
> like one that does nothing, because that is how the saved file will look.
> What it does is said off the canvas, never on it.
>
> ### The part worth your attention, because it is a process failure rather than a feature
>
> **The engine shipped this on 2026-08-30 and this shell did not notice for two
> days.** The reply even said *"please check your own copy — your surface is now
> saying something untrue."* It was read, filed and answered, and the Button
> tool stayed greyed anyway, because nothing here failed when the capability
> landed.
>
> That is fixed rather than apologised for: a new build gate now fails whenever
> pdfcer gains something this shell neither uses nor has written a sentence
> about. On its first run it found five more.


**Ken, 2026-08-30:** *"I think pdfcer added support for several button features
and protections for outgoing submits. implement everything available."*

### ★★★ You were half right, and the half you were right about was worth a lot

**The protections are real and they shipped — on the DETECTION side.** pdfcer can
now spot a push button that posts your data to a web server, an action that
launches a program, and a script that runs the moment a file is opened. It found
a defect of its own doing it: its scanner had been looking in the wrong place, so
*a form that submits to a web server reported nothing at all* — and, in their
words, *"a check that under-reports reads as a clean bill of health, because
silence and safety are indistinguishable to the reader."*

**This shell was not asking.** So the whole finding stopped at the boundary: the
engine could tell you the drawing somebody just emailed you will post its title
block to a server, and nothing on screen said so.

**It does now.** Open a document that reaches outside itself and the status row
says so, once, in one sentence — naming what it can do, and saying plainly that
pdfcer never does any of it but another viewer would.

★★ **It is silent on ordinary documents, and that is the half the check exists
for.** A form that computes a total is an ordinary form; warning about it would
train you to ignore the status row, and then the one sentence that matters is one
you have learned not to read. The driven check opens **two** documents and
asserts the second says nothing.

★ **Nothing in either fixture corpus had a submit action** — which is exactly why
this went unwritten. `tools/gen-submit-fixture.py` makes one: 753 bytes, one
button, one submit pointing at a host RFC 2606 guarantees can never resolve.

### ⬜ Buttons still cannot be given actions, and I checked rather than assumed

`tools/verb-coverage.py` at the current pin: **175 engine verbs, none of them
authors a button action.** The engine's own `FEATURES.md` lists it under
**"Planned, in predicted order"** — `Pass 131.0`, alongside a `pdfcerNet` plugin
for submits and a four-rung disclosure ladder. Unblocked as of 2026-08-26, not
built.

So the request I filed yesterday stands and is unanswered. Nothing to implement
here yet — and I would rather tell you that than build something that looks like
it works.

### The rest of "everything available", measured

Also unconsumed at this pin and **not** done in this pass: `rotate_widget` (turn
a form field's box 90°), `set_dimension_label` (override a ce dimension's text),
and the attachment clipboard. All real, all available. Say the word and they are
next.

### ★★ And one thing I checked because the engine warned about it

They flagged a data-loss bug: deleting the **default** dimension group could
silently destroy every group, every calibrated scale and every ce dimension, with
nothing looking wrong until the next save made it permanent. Their note said
*"if your dimension-groups panel lets an operator select the default group and
press Delete, it can destroy their measurement model today."*

**Ours does not, and never did.** The panel already refuses to draw a Delete
control for the default group, and the same for its visibility switch — R9,
absent rather than offered-and-declined. We had guarded it independently. Their
fix is in the pin regardless, as a backstop.

**Status:** ✅ **the phone-home disclosure is SHIPPED AND DRIVEN.** ⬜ **button
actions remain an engine policy decision, filed, unanswered.**

---

## O60 — ✅ Redact by selecting on the canvas · ✅ AND push buttons that actually do something

> ### ★★★ THE SECOND HALF CLOSED 2026-09-01
>
> *"do push buttons work for some features and can we now add them?"* — **yes to
> both, and the answer to the first half was always yes.** Buttons in somebody
> else's form have always kept working when pdfcer saves the file; it recognises
> every action type it meets and preserves all of them. Only pdfcer's **own**
> buttons were inert, and they are not any more.
>
> The full account is on **O61**, which is the row that asked for it.


**Ken, 2026-08-30:** *"the redaction tool — am I able to select objects on the
canvas and redact them that way yet? I only tried it when it only worked with
the search box and it didn't work for some things. it just told me it couldn't.
also do push buttons work for some features and can we now add them?"*

Two asks. One is done; the other is not mine to decide.

---

### ✅ Redact what you have selected

Select anything on a page — a shape, an image, a piece of text, several at once
— and press **Redact selection** on the Edit tab. It marks the box you can see
around what you picked.

**You were right that it couldn't, and right about why.** There were exactly two
routes: the search box, which reaches *text pdfcer can read as text*, and *mark
whole page*, which reaches everything. On a CAD drawing almost everything worth
redacting is in the gap between them — a title-block value drawn as **vector
strokes**, a scanned stamp, a logo, a signature image, a run in a font whose
encoding cannot be mapped. There is nothing you could have typed that would have
found any of them. *"It couldn't"* was the program being honest about a route,
not a bug in it.

★ **No engine change was needed.** `add_redaction` has always taken arbitrary
regions; what was missing was a way to hand it the selection.

★★ **It marks, it does not remove.** Nothing is destroyed until you press Apply,
and the marks go into the same review list as the other two routes. The
confirmation says so in those words, because *"3 objects redacted"* would make
you stop checking and save a document that still contained every one of them.

**One deliberate limitation:** it marks the **bounding box**, not the exact
outline. A redaction that follows a shape tells you what was there — the
silhouette of a signature is a signature, the outline of a part number is its
digit count.

**Driven and falsified:** `a_selected_object_can_be_marked_for_redaction`. It
asserts the mark count went up **and** that nothing was applied — a build that
quietly applied on marking would look completely correct and be the worst defect
this feature could have.

---

### ⬜ Push buttons — you can add one, and it can never do anything

**Yes, you can already place one**, and it is a correct button in any viewer.
**No, it cannot do anything**, and that is a deliberate pdfcer-wide rule rather
than a gap in this shell: the engine authors **no action of any kind** on a
button it creates — no submit, no reset, no navigate, no script. It says so on
every single creation, which is why you would have seen it disclosed.

The rule exists because `/A` reaches launch actions, network submits and
JavaScript, and pdfcer's standing position is that it recognises and preserves
those and never writes or runs one.

**So this is a policy question and it is not mine to answer.** I have put it
back to the engine with the narrowest useful version: **a Reset button**, and
only that. Reset touches nothing but this document's own fields, pdfcer already
performs a reset internally, and it is the button a person actually draws on a
form and expects to work. I explicitly asked them *not* to give us Submit — its
whole purpose is to send data somewhere, and no shell can audit a URL you typed.

**One thing I could not confirm and should:** you asked whether push buttons
*work for some features*, which sounds like you have met one in someone else's
form and seen it do something. Buttons that already exist in a document are
preserved when pdfcer saves it, so those keep working — I have asked the engine
to confirm that in writing.

**Filed:** `request_a_push_button_that_does_nothing_is_the_only_kind_we_can_make.md`

**Status:** ✅ **redaction-by-selection SHIPPED AND DRIVEN.** ⬜ **push-button
actions are an engine policy decision, filed, awaiting their ruling.**

---

## O59 — ✅ Cut, copy and paste for **everything**: the engine shipped it, the shell has not consumed it

**Ken, 2026-08-29, to the engine session:** *"can you make sure we have cut,
copy, and paste available for everything and if not implement?"* → *"yes do all
without stopping."* Then to this session: *"latest release of core engine
ready."*

The engine did all of it. **This shell has consumed none of it yet**, and the
row exists so that fact is on paper rather than in a chat reply.

### Measured at pin `0eb9119`, not assumed

`tools/verb-coverage.py`: **173 verbs, 152 named somewhere here, 21 named
nowhere** — up from 10 unconsumed this morning. Fourteen of the eleven new
misses are this one release's clipboard family:

```
copy_pages      cut_pages      paste_pages
copy_outline_item cut_outline_item paste_outline_item
copy_attachment cut_attachment paste_attachment
cut_annotations cut_selection  cut_field
```

### What is newly possible, in your terms

| | today in the shell | now possible |
|---|---|---|
| **Cut a comment** | copy, then a separate delete — **two undo entries** | one gesture, **one** undo entry, labelled *cut* |
| **Sticky notes, text boxes, stamps** | copy refused | copy, cut and paste, keeping the author, the date, the note text and the opacity |
| **Links** | copy refused | copied, and the destination **checked** on paste — dropped and disclosed if it does not resolve here, where Acrobat drops it somewhere arbitrary |
| **Whole pages** | nothing | copy, cut, paste — and the clip **is a PDF**, so it opens in anything |
| **Bookmarks and their subtrees** | nothing | copy, cut, paste, **between two documents** — which Acrobat cannot do at all, by Adobe's own documentation |
| **Attachments** | nothing | copy, cut, paste |
| **A copied comment in a saved clip** | silently lost | carried — the clip file used to drop every annotation |

### ★★★ Three things that must be asked BEFORE the press, not reported after

The engine named these itself, and each produces *a document that looks right
and is not*:

1. **Cut must be greyed, not failed.** A copy of something pdfcer cannot carry
   is free — the original stays. A cut of the same thing is a deletion wearing
   a clipboard's clothes, and the engine refuses it by name. So the control has
   to be disabled with the reason, which means asking `copy_selection` first and
   looking for an `Unsupported` marker.
2. **Pasting pages brings orphaned form-field boxes.** A page's `/Annots`
   reaches its widgets; the `/AcroForm` that owns them does not travel. They
   draw like fields and nothing can fill them. The engine measured two on its
   own smoke test.
3. **A pasted bookmark whose page does not exist here is silently dead.** It
   still shows, still has its title, and does nothing when clicked. Ask
   `deepest_page()` against the page count first.

### Six engine defects fixed on the way, two of which were ours to trip over

`copy_annotations` used to require the page to have **content** — so a comment
on a blank sheet could not be copied at all — and `paste_objects` required the
page to have `/Resources`. Both were the content path's preconditions applied
to a gesture that is not a content gesture. Neither had been reported from
here, which means neither had been hit yet; both would have looked like our
bug.

### ✅ 1 of 3 — Cut is greyed with a reason, and the chord is refused too (2026-08-29)

**Selecting a redaction mark now greys Cut**, and pressing `Ctrl+X` on one is
refused with a sentence naming what it was and offering Delete instead.

★★★ **Greying the button is not the fix, and that gap is what the driven check
is about.** A chord is dispatched through the keymap **without consulting
command enablement**, so `Ctrl+X` reaches the handler whatever the ribbon is
showing. A build that greyed the button and left the chord alone would look
perfect in every screenshot, pass every unit test of the gate, and delete a
redaction mark while putting nothing on the clipboard.

★★ **And it refuses BEFORE the copy**, which the check asserts separately. A cut
that refused *after* copying would leave the mark on the page **and a copy of it
on the clipboard** — so the next `Ctrl+V` arms a redaction nobody reviewed
somewhere else. That is the exact outcome the refusal exists to prevent, and
asserting only the refusal would have missed it.

**The gate mirrors the engine's rule rather than calling it, and that was a
measurement, not a preference.** The engine's advice was *"copy the selection
first, then look for an `Unsupported` entry"* — right about the oracle, wrong
about the budget: `copy_selection` decomposes the page with no cache anywhere,
and a ribbon condition is rebuilt **every frame**. On the benchmark drawing that
is a full decomposition per frame to decide whether one button is grey. The
mirror asks the same question from one dictionary read.

★ **The mirror is deliberately permissive**, and its test says why: the engine's
carryable set **grew** the same day — sticky notes, text boxes and stamps all
became copyable — so a mirror written a day earlier would have been greying Cut
over things that had since become perfectly cuttable, with nothing failing to
say so. A mirror that is too permissive costs one refusal sentence; one that is
too strict costs a capability, silently.

**Driven and falsified:** `cutting_a_redaction_mark_is_refused_before_anything_is_removed`,
against the engine's own three-mark fixture. With the gate stubbed out it fails
on its first assertion.

**Housekeeping:** `canvas/mod.rs` hit R2's ceiling for the **second time in one
day**, so the thousand lines of drawing moved to `canvas/present.rs` and that
file is now purely a module index. The first time, the fix was to shorten a doc
comment with a note saying the real seam was elsewhere — it was, and the note
was right.

### ✅ 2 of 3 — Whole pages can be cut, copied and pasted (2026-08-29)

**Pages ▸ Clipboard**, three controls: Cut, Copy, Paste. With sheets picked in
the Pages panel they act on those; with none picked, on the sheet you are
looking at — the same operand rule every other `pages.*` verb uses, so there is
one rule to know rather than two. A paste lands **after the current sheet**.

★ **What pdfcer holds is a complete PDF.** The engine chose that deliberately, so
a copied set of sheets is a document — which is worth knowing because it means
the eventual "paste into another program" costs no new work at all.

### ★★★ These are the only clipboard controls with no keyboard shortcut, and that is a decision

`Ctrl+C` belongs to the canvas and could not be shared. Every `pages.*` verb
falls back to the current page when nothing is picked, so a rule that asked
*"are there pages to copy?"* would answer **yes on every document** — and the
canvas would lose its clipboard permanently, with no state you could reach to
get it back.

Acrobat solves the same collision by **focus**: `Ctrl+C` in its thumbnails
copies pages. That is a good answer and it is not available here — this shell's
thumbnails are a dock panel whose focus egui does not model in a way a chord
dispatcher can read, and inventing a focus notion to serve one chord would have
it owning every other chord too. Named and rejected rather than quietly not
done; say the word if you want it and it becomes its own piece of work.

### Two things you will be told that you cannot see

- **at the copy** — a form field left behind, because parts of it sit on sheets
  you did not pick. Said then rather than at the paste, because at the copy you
  can still widen the pick; by the paste it is an autopsy.
- **at the paste** — boxes that look like form fields and belong to nothing. A
  page's annotations reach its field boxes; the form definition that owns them
  is a document-level entry and does not travel. They draw exactly like working
  fields and nothing can fill them. The engine flagged this as *"the one that
  produces a document that looks right and is not"*, and measured two on its own
  test. The sentence points at the Forms panel, which lists them and can adopt
  them.

**Driven:** `pages_can_be_copied_and_pasted` — copy, paste, and the document
goes from **4 pages to 5**. The oracle is the page count, not the trace lines:
every intent line would be present and correct on a paste that inserted nothing.

**One workaround reported to the engine.** `PageClip` is `#[non_exhaustive]`
with no way back from bytes, so a shell holding only the clip cannot rebuild one
to call `paste_pages` — which is odd for a type whose whole selling point is
that it *is* a PDF. The paste goes through `insert_from_view` instead, which is
what `paste_pages` does internally anyway, and reuses this shell's existing
insert disclosures rather than writing a second wording of the most consequential
one.

**Housekeeping:** the reach checker failed closed for the **sixth** time the
moment the three commands were registered, and `reach.rs` crossed R2's ceiling,
so the guard chain moved to `reach/guards.rs` — its header records what six
instances of the same lesson are evidence for.

### ✅ 3 of 3 — A bookmark and everything under it can be cut, copied and pasted (2026-08-29)

In the **Bookmarks panel**, beside Rename and Remove, because that is where
every other bookmark verb already is: a bookmark is edited where it is seen.
Copy and Cut act on the selected one **and everything filed under it**; Paste
puts them under the selection, or at the top level when nothing is selected —
the same rule Add already uses, so there is one place to learn it.

★★ **This is the one operation in the program Acrobat cannot do.** Between two
files it cannot do it at all, by Adobe's own documentation. Carrying a chapter's
bookmarks into another drawing has always been a hand job, one at a time.

**The warning comes before the press.** If the bookmarks point at pages this
document does not have, the panel says so beside the button — *"some of these
point at page 14, and this document has 6"* — because a bookmark that arrives
without its destination still shows, still has its title, and does nothing when
clicked. Nothing on screen tells them apart. After the press the engine reports
how many actually dropped, which is a different fact: the first is a choice,
the second is what happened.

### ★★★ It uncovered a real defect that had nothing to do with this work

While making the driven check work, the panel's own numbers stopped adding up:

```
panel body      y = 159.3 .. 447.7
bookmarks.delete    y = 500.3 .. 524.3
bookmark-copy       y = 528.3 .. 552.3
```

**Remove was 53 points below the bottom of its own panel.** It was drawn, it
reported a position, and it could not be clicked — and it has been that way
however long the panel has been this shape. It survived because those controls
appear only when a bookmark is **selected**, and no check had ever selected one
first.

The cause is a rule this project already had written down and had applied to
half the panel: the dock gives a panel body a fixed rectangle and no scrolling
of its own, so the body must make one. This body wrapped its **list** and left
everything above it in whatever space remained. Fixed by putting the controls
inside the same scroll area.

⇒ Worth knowing because it is a whole class: **a control that only appears
under a specific selection is invisible to every test that does not make that
selection.**

### The check took five attempts and each failure was a lie

Three of them produced the *identical* message — *"the Copy control is not on
screen"* — for three different reasons, none of which was the Copy control. It
aimed at the bottom row instead of a visible one; at the full-width strip
instead of the twelve-point-wide label; and at a rectangle that never parsed,
because a rect value contains spaces and the trace reader split it on the first
one. Then the whole thing alternated PASS/SKIP because the panel is a **toggle**
and the dock layout is **saved to disk**, so each run undid the last.

All five are written up in `D:/dev/rag/egui/` — the transferable rule is that a
diagnostic line written for every item is not a list of the items you can click.

**Driven and falsified:** `a_bookmark_subtree_can_be_copied_and_pasted` — the
outline goes from **5 bookmarks to 6**, three runs in a row. With the paste
stubbed out it fails on the applied-line assertion.

**Status:** ✅ **ALL THREE DONE — awaiting your verdict.** Cut refuses what it
cannot carry, pages copy and paste, bookmarks copy and paste.
**Was:** ⬜ **OPEN — engine ready, shell not started.** The pin is updated
and verified (2,662 tests, 19/19 gates, both driven clipboard checks green at
`0eb9119`), and nothing has regressed. What is not done is *using* any of it.

---

## O58 — ✅ Copy and paste a form field: `Ctrl+V` pastes a NEW field, `Ctrl+Shift+V` pastes a DUPLICATE

**Ken, 2026-08-29:** *"wire the request. ctrl v for paste as new. ctrl shift v
for paste as duplicate."*

This closes the one question that was his to answer. Copying a field has two
legitimate meanings and the engine refuses to guess between them by name
(`edit.rs:9364` — *"a renamed field is a DIFFERENT field … That is a decision
about your form, not a copy"*). He ruled that **both** are wanted, on two
chords:

| chord | meaning | the field's value |
|---|---|---|
| `Ctrl+V` | a **new, independent** field with a new name | its own |
| `Ctrl+Shift+V` | **another widget of the same field** | shared — type in one, both fill |

### ★★★ The counter-intuitive half: the DUPLICATE is the faithful one

`add_text_field` with an existing `/T` **appends a widget to the existing
field** rather than refusing — `edit.rs:13523`, `merged: true` — and the same
branch exists on all five authoring verbs. So `Ctrl+Shift+V` is one existing
call, and because the field object is never touched it inherits `/DA`, `/Q`,
`/V`, `/DV`, `/Ff` and `/AA` exactly.

`Ctrl+V` is the lossy one. It has to re-author through `New*Field`, which is
geometry-plus-booleans, so it drops the font, size, colour, alignment, default
value, calculation script and border colour — every one of them readable on
`forms::Field` and writable nowhere. A signature field cannot be authored at
all.

**Fifth stale blocker.** We arrived intending to ask for a widget-clone verb and
found half of it shipped. *A backlog row is a record, not evidence.*

### What is filed, and what is not blocked on it

Engine request:
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_form_fields_cannot_be_pasted_and_half_of_it_already_works.md`
— asks for one shape, `copy_field` / `paste_field` carrying a serialisable
`FieldClip` with a `NewField` / `AdditionalWidget` policy. Serialisable because
he copies **between drawings**, and the engine confirmed this morning that
`ObjectClip::to_bytes` does not carry annotations.

**Neither chord is blocked on the reply.** Both ship off what exists; until the
verb lands, `Ctrl+V` discloses what did not come along — off-canvas, in the
status line, never marked on the page (Rule 4).

### What still has to be built here regardless of the engine

Form fields are deliberately **excluded from canvas annotation selection**
(`canvas/selection/annot.rs:189` — the form surface owns a `/Widget`), and
`canvas::clipboard::copy` only ever looks at the annotation selection. So there
is currently **no path at all** from a selected field to `Ctrl+C`. That is
shell-side plumbing: a third `Clipped` variant fed from `doc.selected_field`, a
cut path over `delete_widget`, and the two paste chords.

### Shipped and driven, 2026-08-29 — same day

Both chords work end to end against the release binary. The driven check is
**`a_form_field_can_be_copied_and_pasted_both_ways`**, and it went red before it
went green, twice, for two different reasons — which is the only reason it is
worth anything:

1. **`Ctrl+V` did nothing.** `egui-winit` raises `Event::Paste` only when the OS
   clipboard holds non-empty text and swallows the keystroke otherwise. The
   existing content-copy path writes a marker for exactly this reason; the new
   field-copy path did not, because that workaround lives at each copy site.
   *A documented platform trap does not protect a code path written after it* —
   filed to `D:/dev/rag/egui/`.
2. **`Ctrl+Shift+V` would have pasted a NEW field.** egui's own
   `is_paste_command` does **not** exclude Shift, so `Ctrl+Shift+V` becomes the
   same `Event::Paste` as `Ctrl+V` with the raw key swallowed and no modifier on
   the event. The plausible assumption — that it arrives as an ordinary key and
   the generic keymap handles it — is wrong, and shipping on it would have made
   the chord silently do the other thing.

And one defect in the **check itself**, caught before it was trusted: its box
count summed every frame's census, so it read 1 → 3 → 6 and passed while
measuring repaints. It now counts distinct `(field, centre)` pairs and reads
**1 → 2 → 3 boxes with 1 → 2 → 2 names**, which is the merge observed from
outside the engine.

**What is NOT proved, said out loud:** that a duplicate *shares a value* with its
source. That needs typing into one box and reading the other, which this harness
has no gesture for. The box count proves a second widget arrived; it does not
prove the two are one field. The engine's own disclosure says they are.

### The naming, checked against Acrobat rather than chosen — 2026-08-29

You left this to me, so it was sourced instead of picked. The reference file
`forms__field_copy_paste_and_duplication.md` settles two things:

**`Text1` → `Text2` → `Text3`, and `Drawn By` → `Drawn By2`.** The first
version produced **`Text1 2`** and was wrong twice over. Acrobat's bulk
duplication auto-names copies `Date1`, `Date2`, `Date3`, and the sourced reason
is scripting: the suffix exists so a script can loop over every field sharing
*"the non-number part of the field name"*. **A space breaks exactly that.** And
our own placement dialog already names a new field `Text1`, so continuing the
number is the ordinary case, not the exotic one — appending would have given
`Text12`, which reads as field twelve.

**★★★ And one Acrobat convention is deliberately REFUSED.** A second sourced
account has Acrobat numbering copies `Text.0`, `Text.1` with a **dot**. pdfcer
must not, whichever account is right: `.` is the fully-qualified-name separator
(§12.7.3.2), so `Text.2` is not a field called that — it is a **child field
named `2` under a parent named `Text`**. That is a third shape, a shared
ancestor with independent children, and it is neither of the two things your two
chords mean. Adopting it would have given `Ctrl+V` a hierarchy nobody asked for.
The driven check now forbids both a dot and a space in the pasted name by name.

### ★★★ A FINDING FOR YOU, not a change: your `Ctrl+V` is the OPPOSITE of Acrobat's

Reported rather than acted on, because the chords are your ruling.

In Acrobat, **plain single-field Copy/Paste is the LINKED one.** Copy a field,
paste it, leave the name alone, and you get a second widget of the *same* field
sharing its value — what you assigned to `Ctrl+Shift+V`. Acrobat has no
"paste as independent" chord at all; you get independence only from the *bulk*
commands (Place Multiple Fields, Create Multiple Copies), which auto-name and
therefore produce separate fields.

**I think your split is the better one and I have not changed anything.** Two
reasons, both from the same source:

1. Acrobat's own linking default is a **documented, unresolved point of user
   friction** — there is a standing Acrobat DC request for an *"option to unlink
   form fields when copying"*, and the only workaround Acrobat offers is
   renaming each duplicate by hand afterwards.
2. Acrobat's own guidance frames the bulk commands as *"the recommended approach
   when you want each field to behave independently"* — which is the title-block
   case, and the common one. You put the common intent on the common chord.

⇒ **But it is a divergence from the reference app, it is now written down, and
it is yours to reverse.** Swapping the two chords is a one-line change if you
would rather match Acrobat exactly.

### ★★★ You asked whether we could just BE able — and by then we were, 2026-08-29

*"shouldn't we make it so we are able and then do follow Acrobat's
conventions?"*

**Yes, and it took no work.** The engine had already shipped it. The request
this shell filed at 11:56 was answered at **13:03** — `Pass 167.0`,
`pdfcer_core::formclip`, `copy_field` / `paste_field` — and the reply was sitting
unread in the channel while the workaround was being published.

**An unsigned signature field now copies and pastes normally.** Which is more
than parity: it hands this shell signature-field *authoring* it never had, since
there is still no `add_signature_field` verb at all. A **signed** field is
refused at the copy, and that refusal is correct everywhere, not a pdfcer limit —
a signature is a byte-range assertion about the document it was made in, so no
program can duplicate one and have it stay valid. What the engine specifically
declines to do is carry the *"signed by"* artwork into a file nobody signed,
which would be a plausible-looking void graphic.

**And `Ctrl+V` stopped being lossy in the same move.** Everything the status row
used to apologise for now travels: the font, its size and colour — **and the
actual font file**, installed into this document's resources and renamed if that
name is taken here — the alignment, the default value, the calculation script
with its registration, the border and background colours, the border styles, the
flags no authoring form can express, and the baked appearance with its whole
resource closure. Radio groups travel whole.

**Deleted from this shell as a result:** the fidelity table, the loss sentence,
eighty lines of translation, and **two of our own refusal messages that had gone
stale within an hour of being written**. Every disclosure is now the engine's,
word for word.

⇒ **Fifth confirmation of a rule that keeps costing this project a day at a
time:** a reply arriving is not a capability landing. The workaround was
published at 12:49 and replaced at 14:0x; the gap was ninety minutes and it
still cost a build and a `FEATURES.md` row that was true for one hour.

### ★★★ SETTLED — it is a setting now, 2026-08-29

*"let's make it an option to have it swap to match Acrobat or work the way we
have it now."*

**Settings ▸ Display ▸ Copying a form field**, two choices:

| | `Ctrl+V` | `Ctrl+Shift+V` |
|---|---|---|
| **pdfcer's order** (default, your original ruling) | a separate field with its own value | another box that fills in step |
| **Acrobat's order** | another box that fills in step | a separate field with its own value |

**It swaps the CHORDS, never what a command means.** `edit.paste` is always
*"paste as a new field"* and `edit.paste_duplicate` is always *"paste as another
box"* — only the keys move. Swapping what the *commands* do was the obvious
implementation and was rejected: a ribbon button reading **Paste as duplicate**
would have pasted a new field, and no tooltip rescues a control whose name is
wrong. Moving the binding instead means the ribbon, the context menu, the
shortcuts dialog and the keyboard all agree by construction, because every one
of them reads the same keymap.

★ **Both pastes stay available either way.** This exchanges two keys; it never
removes a capability. If you forget which order you chose, both are on the Edit
tab under their own names.

★★ **The default stays yours.** You made the split before the Acrobat
divergence was found, were told about it, and asked for an option rather than a
swap — so a fresh install behaves exactly as it did this morning.

**Driven, both ways, and falsified.** `a_form_field_can_be_copied_and_pasted_both_ways`
drives the default order; `the_acrobat_paste_order_swaps_which_chord_does_which`
drives the other in a second process and asserts the **mirror** — under the
Acrobat order `Ctrl+V` must add a box *without* a name and `Ctrl+Shift+V` must
add one. Measured: it does.

⇒ And it was **falsified before it was trusted**: with `apply_paste_chords`
stubbed out, the Acrobat check fails on its first assertion and its message
names the cause — *"if the OTHER order passes, the paste itself is fine and the
SETTING is what did not reach the keymap."* That is the defect this check exists
for: a preference that saves, reloads, reads back correctly in the pane and
**changes nothing when the key is pressed**. No unit test can see it, because
the keymap, the chord translation and the dispatcher all sit between the
preference and the keystroke.

**Status:** ✅ **SHIPPED, LOSSLESS, OPTIONAL AND DRIVEN — awaiting your
verdict.** Nothing is open on this row and no question is outstanding.

---

## O57 — ⬜ The grips swallow small objects, and half of it is still open

**Not his report — found by a driven check on 2026-08-29**, and filed here
because the remaining half is a design decision that is his to make.

### What is wrong

At a fitted zoom, a small object's own resize grips cover the whole of it, so
there is nowhere left to press that means *move*. Measured: a **160 × 20 pt**
form field at 29.55 % is **47.3 × 5.9 px**, and a mid-edge grip reaches 6 px in
— so the centre of the field is inside its own North grip. Dragging it to move
it committed a degenerate resize, which the engine refused by name:

```
resize-widget-commit … grip=North sy=-42.5314
edit-widget-refused  … rectangle has no area
```

⇒ From his chair: **the field does not move, and nothing says why.** On an A1
sheet at fit zoom this is every form field, every dimension label and every
short markup.

### The half that is fixed

A mid-edge grip is now withheld when the box is thinner than **20 px across the
perpendicular axis** — the axis that grip eats into. The existing rule only ever
checked a grip against its *own* axis, so it stopped grips piling onto each
other and never stopped one swallowing the body. Corners survive, because they
are the grips a small box actually needs. Two tests, falsified.

### ★★★ The half that is open, and it is his call

On a box that is small in **both** axes, the four corner grips cover it too, and
the body survives only in the gaps between their x-ranges. There is no threshold
that fixes that, because the grips genuinely do not fit.

**Every program in the class solves it the same way: when the box is too small
to hold them, the grips are drawn OUTSIDE it.** Illustrator, Inkscape and Figma
all do this. His standing tie-breaker is *"make it work the way other programs
do"*, which points straight at it — but it is a change to the **painter** as
well as the hit test, it changes what a selection looks like on every dense CAD
sheet, and the rotate handle already sits outside and would need to move further
out to stay clear.

⇒ Not done unilaterally at 08:00 after a build. **The question for him:** should
a selection too small for its grips draw them outside the box, as other editors
do — accepting that a dense drawing at fit zoom will show grips overlapping
neighbouring objects?

**Status:** ⬜ **OPEN — half shipped, half is a question.** The shipped half is
in `canvas::handles::MIN_BODY_STRIP_PX` with its measurement. Not driven: found
by `widget_move`, whose own press had to be moved to a quarter of the box width
to get past the grips at all — which is itself the report.

---

## O56 — ★★★ "Confirm that you have built every editable surface into the GUI that has been implemented in pdfcer"

**His ask, 2026-08-28**, verbatim:

> *"confirm that you have built every editable surface into the GUI that has
> been implemented in pdfcer. continue and loop until the handoff items and these
> other things are done."*

### ★★★ It could not be answered from this project's own documents, and that is the finding

`FEATURES.md` says what the GUI does. `NO_SURFACE.md` lists compiled-in values
with no control. `GUI_ROADMAP.md` says what is planned. **None of the three is
keyed on the engine's verb list**, so none of them could answer *"is there a
verb `pdfcer-core` implements that nothing in this shell calls?"*

⇒ The answer required an instrument, and the instrument is the durable half of
this row: **`tools/verb-coverage.py`** parses `impl EditSession` out of
`edit.rs`, takes every `pub fn`, and greps this crate for each name. The
register it feeds is **`EDITABLE_SURFACES.md`**, which carries a reason per
miss and is re-derivable rather than trusted.

**157 verbs. 22 named nowhere.** Half of those are session queries or alternate
spellings of a verb this shell calls in another form; the rest were real.

### What the sweep found, and the pattern in it

| Gap | What it was |
|---|---|
| `set_markup_note` / `clear_markup_note` | The Comments panel could not write one word onto any annotation. Shipped in answer to **this shell's own request** four days earlier |
| `add_markup_with` | Markup could not be authored translucent. Shipped in answer to **this shell's own request** |
| `set_outline_title` / `delete_outline_item` | Bookmarks could be created and never changed |
| `set_quad_point_order` | ★★★ **A live defect** — a setting he can change that was honoured by nothing |
| `delete_pages_with` | ★★★ The same, one file along: his separation policy never reached the verb |
| `rotate_annotation` / `rotate_dimension` | No rotate gesture for annotations or ce dimensions |
| `attach_file` / `detach_file` | File attachments had no surface at all |
| `unshare_form`, `copy_annotations`, `delete_field_group`, `field_defaults` | Still open — see the register |

★★ **Three of the first four were capabilities the engine shipped because this
shell asked for them, and then never consumed.** *A reply arriving is not a
capability landing.* That is the lesson and it is why the instrument exists
rather than a promise to remember.

★★★ **The two settings defects are the ones worth his attention**, because
they are the kind he would report as *"I changed that and nothing happened"*:
`quad_point_order` and `separations` were both persisted, validated, drawn in
the Settings window, and consulted by nothing. `app::settings` exists precisely
to prevent that class and a `syn` check enforces it — **and both were invisible
to it**, because the check is keyed on option *constructors* and these two
arrive through a *setter on a session* and a *parameter on a verb*. A guard
shaped around one delivery mechanism cannot see a second one.

**Status:** ★ **PARTIALLY SHIPPED, and the row stays open until every gap in
`EDITABLE_SURFACES.md` is either wired or carries a dated reason.** Shipped
2026-08-28: the note editor, author-time opacity, bookmark rename and delete,
both settings fixes, the instrument and the register.

★ **NOT DRIVEN.** `a_note_can_be_written_onto_a_shape_that_exists` and
`a_bookmark_can_be_renamed_and_removed` are written and have never run — the
machine was his. Every driven claim in this row is owed a sweep.

---

## O55 — ★★★ A fit should CENTRE, and a canvas resize should keep the fit until the operator leaves it

**His words, 2026-08-28:**

> *"I want the buttons that zoom to page, width, height etc when pressed to
> center the page, or width, or height on the canvas window. if the canvas
> window is resized the pdf should resize to match unless the person has
> changed the zoom or panned around."*

### ★★ Two asks, and only one of them is new — which matters for how to read it

**(a) Centre on fit.** This is **O28 again**, in his words a second time:
*"If I press the Fit width or fit page button the view should center to the
width as well or center the page."* That shipped on 2026-08-24 —
`canvas::fit::placement` and `geometry::fit_placement_offset` — and
`a_fit_command_puts_the_page_on_screen` passes today.

⇒ **A repeat request against shipped code is evidence about the code, not
about him.** The check's name is the tell: it asserts the page is *on screen*,
which is a weaker claim than *centred*, and the implementation matches the
weaker claim — `fit_placement_offset` returns **0 on a pinned axis**, which is
the page's own top-left, and leaves the unpinned axis wherever it was. For
fit-page both axes are pinned and the centring margin does the rest; for
**fit-width the vertical is not touched** and for **fit-height the horizontal
is not**, so the thing he pressed the button for does not happen on two of the
three.

**(b) Re-fit on resize, unless zoomed or panned.** The first half already
works: `FitMode` is a *live* mode and `ViewState::apply_fit` recomputes the
zoom from the viewport every frame, so a dock resize or a window resize
re-fits by construction.

★★★ **The second half does not.** `set_zoom` drops out of the fit — so zooming
leaves it, as he expects. **Panning does not.** `doc.last_scroll_offset` is
written every frame from the scroll area's own state and nothing consults the
fit, so an operator who fits, then pans to look at a corner, is still *in* fit
mode — and the next resize throws their position away.

⇒ That is the half worth having: it is the difference between a fit that is a
**mode you are in** and one that is a **command you pressed**. His sentence
draws the line exactly where the code does not: *"unless the person has changed
the zoom or panned around."*

### ★ What "centre" has to mean per axis, since it is not one rule

| | horizontal | vertical |
|---|---|---|
| **Fit page** | centred | centred |
| **Fit width** | fills, so centring is trivially satisfied | **centred** — currently left alone |
| **Fit height** | **centred** — currently left alone | fills, trivially satisfied |

★ On the axis a fit *fills*, there is nothing to decide. On the other axis
there are two defensible answers — keep where you were, or centre — and he has
now asked for centre twice.

### What shipped, 2026-08-28, and the trap in the middle of it

**(a) was already right** and the driven check proves it: fit page centres,
fit width fills the width, fit height fills the height — before *and* after a
resize. What was missing was **(b)**, and it split into two changes that only
work together.

**1. A fit re-places when the VIEWPORT changes.** It used to place once, on the
button press: `canvas::fit::placement` read a one-shot set by `Action::Fit`.
Meanwhile the *zoom* was recomputed every frame. So a resize re-scaled the page
correctly and left it anchored wherever it sat — **the scale right, the
position stale**, which is O28's complaint arriving through a different door.

**2. A pan leaves the fit.** `set_zoom` always did; panning did not.

### ★★★ The trap: "a fit is a mode, so re-place every frame" is WRONG

That is the obvious reading and it was written, built and run. Under **Fit
page** both axes are pinned, so the placement returns the page's origin on
every frame and **the wheel cannot scroll at all** — in a continuous display,
the document becomes unnavigable.

⇒ Caught by `a_fit_command_puts_the_page_on_screen`'s own precondition, which
scrolls into the pasteboard and **asserts it got there** before pressing
anything. It reported *"the pan did not move the page"* and SKIPPED. **A setup
step refusing to proceed**, which is exactly the shape a precondition should
have and the reason that check was written to establish its own.

★ The fix is to key on the **viewport**, not the frame — which is what his
sentence says: *"if the canvas window is **resized**"*. Resized, not redrawn.

### ★ And the wheel deliberately still keeps the fit

Scrolling a fit-width document is how every reader in the class is read. A
wheel notch that dropped the fit would stop the page re-fitting the moment
anybody looked at the second half of it. **A pan is a deliberate
repositioning** — the middle button, or the hand tool — and that is the gesture
he named.

⇒ The two checks assert opposite outcomes for the two gestures, so a build that
treats all view movement alike fails one of them whichever way it goes.

**Status:** ★★ **DONE 2026-08-28, driven and falsified.**
`a_fit_command_puts_the_page_on_screen` gained a resize phase;
`a_pan_leaves_the_fit` is new. With the pan fix removed the latter reports
`margins l=8.0 r=8.0 t=108.4 b=108.3` — dead centre — against
`l=-39.0 r=-105.0 t=120.4 b=-27.3` for the correct build.

★ It would **not** have caught the state that shipped before today, and its
header says so: then, a resize re-placed nothing, so a panned page was not
re-centred and it would have passed for the wrong reason. **A guard written
with a fix guards the fix**, not the original defect.

---

## O54 — ★★★ The highlight tool should follow text the way Acrobat's does, and paragraph reflow should be offered

**His words, 2026-08-28:**

> *"Also the highlight tool - it's great that we can just drag a box to highlight
> an area, but we should be able to drag it along to just highlight text too like
> it works in adobe. Also I think the paragraph reflow was implemented ages ago
> in the pdfcer core, so we should have that option too."*

### ★★★ (a) The highlight tool draws a BOX where it should follow text

Both halves already exist and **they are not connected**:

| piece | state |
|---|---|
| sweeping text produces line-grouped quads | `canvas::textsel`, shipped |
| authoring `/Highlight` from quads | `Action::CommitTextMarkup { quads }`, shipped |
| **dragging the highlight TOOL over text** | draws a rectangle band |

So an operator can highlight text today only by sweeping it with the *select*
tool and then pressing a ribbon control. With the highlight tool armed —
the thing named after the job — a drag draws a box.

⇒ **This is O53 again, in a different costume.** The capability is reachable
through a panel-shaped route and not through the gesture the operator will try
first, and it reads as missing because it is.

★★ Acrobat's Highlight follows the text and it is the convergent behaviour of
the class. The fallback matters too: a drag that finds **no text** should still
draw the area box, because that is what pdfcer already does and is genuinely
useful on a scan — Acrobat draws nothing there. So the rule is *follow text
where there is text, box where there is not*, which strictly dominates the
reference.

★ It applies to all four text markups, not just highlight: underline,
strike-out and squiggly are the same gesture over the same quads.

### ★★★ (b) Paragraph reflow — he is right that the engine has it

`ReflowEngine` is named in `canvas::textedit` and has been in `pdfcer-core` for a
long time. **What this shell does with it has never been an operator choice**:
the editor decides between reflowing a block and keeping a single line by
inspecting the run's provenance, and the decision is invisible and unappealable.

⇒ *"we should have that option too"* is a request for a **control**, not for a
capability. What is owed:

1. Find what `ReflowEngine` actually offers and what the shell currently pins
   it to — **re-derive, do not trust the note.** This project has retired ten
   blocker claims that were stale, and *"implemented ages ago"* is exactly the
   shape that goes stale in both directions.
2. Surface the choice where the editing happens, not in Settings — it is a
   property of *this* edit, like the alignment fix beside it.
3. Disclose which branch ran, because a reflow that did not happen looks
   identical to one that did nothing.

### ★★ What the re-derivation found, and it changed the answer

Step 1 was done rather than assumed, and the note **was** stale — in the useful
direction. `EditSession::reflow_block(page, block, &ReflowRequest)` is a
**verb**, not a policy knob: it re-wraps a named paragraph on demand and
returns a report of what it did. The thing the shell pins invisibly is
`FollowerDisposition`, which is a different decision about a different
operation (what happens to the *rest of a line* when one run is edited).

⇒ So (b) is not *"surface a choice the editor is making silently"*. It is
*"ship a command the engine has and this shell never called"*. That is a
smaller, cleaner piece of work than the request assumed, and it is done.

**What shipped, 2026-08-28:**

- **Edit ▸ Reflow paragraph**, acting on the paragraph the caret is in.
- **A right-click inside text being edited** — `canvas.text`, a *third* canvas
  menu. The other two are keyed on a selected object and on blank paper, and a
  caret is neither, so O53's ruling would otherwise have been broken by a
  command that existed only on the ribbon.
- **Four refusals, each with its own sentence**: no caret, a caret placing new
  text, a run that is not in a paragraph, and the big one below.

### ★★★ The one thing an operator will meet and must be told, not left to find

`reflow_block` is planned against the document **as opened** — it re-extracts
the page to get position information the editing buffer does not carry — so it
**refuses a file this session has already changed.** One typed character is
enough.

The shell asks that question before the attempt and answers with the remedy in
words: *"Save this file and open it again, then reflow."* A refusal naming a
cause and no remedy leaves somebody trying things; this one has a specific,
cheap remedy and says it.

★ Point 3 of the ask — *"disclose which branch ran"* — is honoured by the
engine's own disclosures, forwarded verbatim, plus a line count the shell adds
because a reflow that changed nothing is a correct outcome that reads as a
failure in silence. The **page cropbox is supplied** so overflow is disclosed
too: re-wrapping can push a block below the bottom of the sheet, and the
default is not to check.

**Status:** ★★ **(b) DONE 2026-08-28**, not driven — he was at the machine. The
`ui-verify` check and its fixture are written and unrun:
`reflowing_a_paragraph_rewraps_it` against `fixtures/paragraph.pdf`, whose six
short ragged lines pack to five, so *"it ran and changed nothing"* fails rather
than passing.

★★★ **(a) SHIPPED EARLIER THE SAME DAY, in `d66f41d`** — and this line was
first written saying *"(a) is next"*, on the strength of the status field above
it rather than on the source. `canvas::markup::text::swept` follows the text
and falls through to the area band where there is none, which is the
*"strictly dominates the reference"* rule this entry asked for.

⇒ **A backlog row is a record, not evidence.** The same mistake this project
has now made four times, twice within a day of the thing shipping. The rule
stands: verify an absence claim against the source before writing it down,
including — especially — in the file that tracks the claim.

**Status:** ★★ **BOTH HALVES DONE 2026-08-28.** Neither is driven.

---

## O53 — ★★★ "always always always": the canvas is the primary surface, and a checkbox proved it is not yet

**His report and his ruling, 2026-08-28:**

> *"I'm noticing that when I make a checkbox, I can't select it on the canvas to
> move or resize. Note that if the engine is capable, I should be able to select
> the object and do all of the ordinary editing one would expect a GUI editor to
> be able to do. It is great that there is a properties box that allows this, but
> **always always always** I need objects on the canvas to be clickable and
> editable as one would expect given our research of other programs."*

### ★★★ The ruling is the bigger half and is now a standing acceptance criterion

**Anything the engine can do to an object, a gesture on the canvas must reach.**
The Properties panel is a *supplement* — the precise route for typed values —
and is never the answer to *"how do I move this."*

⇒ When an engine verb lands, the question is **"what gesture reaches it"**, not
"which panel gets a field". Four numbers and an Apply button are a form for
editing a rectangle; **dragging is how a person moves a box.**

★★ It has been got wrong twice, the same way: form-field geometry and then
annotation geometry were both reachable only by typing into the panel, and both
were reported as done. Both read as missing to him, because they were.

### The specific defect: the placement tool never disarms

He draws a checkbox, clicks it to select it — and the checkbox tool is **still
armed**, so the click places a *second* checkbox instead. `FieldAction::Commit`
authors the field and leaves the tool exactly as it was.

★★★ **This project's own harness has been working around it for a day**:

```rust
// ★ Escape first. The tool stays armed after a placement, exactly as a
// markup pen does, so a second click without this would place a SECOND
// field rather than select one
```

That comment is in `dragging_a_form_field_moves_it`, written yesterday, treating
the arming as normal because a markup pen behaves the same way.

⇒ **When a driven check needs a step the operator would never know to take, that
step is a bug report.** It was recorded as scenery instead. The lesson is filed.

### What the reference programs actually do, which settles it

Acrobat is the parity reference for forms. Placing a field there returns to the
selection tool and **selects the new field**, unless the operator has explicitly
ticked *Keep tool selected*. Word, PowerPoint and Visio all do the same for a
drawn shape. Illustrator and Inkscape keep the tool armed — but they are drawing
programs where placing twenty of a thing is the common case, and they select the
new object either way.

⇒ **Every one of them leaves the new object SELECTED.** That is the convergent
answer, and it is the half this shell gets wrong regardless of arming.

### What is owed, in the order it will be built

| # | piece | state |
|---|---|---|
| 1 | after placing a field, **select it and return to the Select tool** | building |
| 2 | **resize grips on a form field's box** — select, drag a corner | building |
| 3 | the same for every kind, not just the one he named | building |
| 4 | a driven check that does **not** press Escape, because an operator would not | queued — he is at the machine |

★ Moving a form field on the canvas shipped earlier today and is unaffected —
except that he could not reach it, because he could not select the thing first.

**Status:** ★ **ACCEPTED 2026-08-28.** The ruling is recorded as a standing rule
in agent memory as well as here, because it governs every future feature rather
than this one.


### ★★ Update 2026-08-28 (afternoon) — the right-click, and what it exposed

The five gestures this ruling covers are click, drag, grips, Delete and the
**context menu**. The first four reached a form field this morning; the fifth
did not, and a right-click on a text box offered four zoom levels.

`canvas.field` is now the fourth canvas menu — Properties, then Delete. Rename
goes through Properties deliberately: a menu cannot ask for text, and a second
half-built rename that could disagree with the panel's draft-and-commit box is
worse than one click further.

★★★ **Wiring it found a live divergence.** The Delete *key* has reached a
selected field since this morning; `format.delete`, the *command*, had not. So
the two acted on different things — exactly what `app::keyboard`'s single
dispatcher exists to make impossible — and nothing surfaced it, because the
command's only route was the Format tab, which is not drawn for a form
selection. Giving it a second door made it visible in ten minutes.

⇒ **A capability reachable by one route is a capability whose other routes are
untested.** Adding a route is therefore also an audit.

★ And a second, smaller one: both menu items were gated on `selection.any`,
which is **false** while a form field is selected. Every item disabled means
the menu never opens at all — the feature would have shipped as *"right-click
does nothing"*, which is the D1 shape it was written to remove.
---

## O52 — ★★★ Colour default becomes *Match other PDF viewers*, and the old formula goes entirely

**His instruction, 2026-08-28:**

> *"under the colour setting we are going to change our default to Match other
> PDF viewers. you can also remove the The old pdfcer formula from that section,
> even the code for it."*

### ★★★ This REVERSES an earlier ruling of his, and that is the important part

`CmykIntent`'s own type documentation in `pdfcer-core` currently reads:

> `Calibrated` — **Not the shipped default, despite being the best-evidenced
> option**
>
> `NeutralBlack` — **The shipped default, by operator ruling**

So `Calibrated` was always the strongest evidence in the register — tier (a)/(c),
Acrobat's shipped profile *and* pdfium both produce it — and it lost to a
deliberate operator decision about what calibrated rendering does to pure-K line
art on a CAD drawing.

⇒ **He has now looked at it again and changed his mind.** That is not a defect
report and must not be filed as one. What it means practically is that a doc
comment saying *"by operator ruling"* becomes a lie the moment the default
flips, and **the ruling it cites is the one being reversed** — so the change
carries a documentation obligation the code change alone does not discharge.

★★ It also removes the reason the *divergence note* exists. That sentence —
*"pdfcer's default deliberately differs from Acrobat here"* — was written so a
future session would not investigate a render-parity difference as a bug. With
the default matching, the note is not merely redundant: **it is backwards**, and
leaving it would tell somebody pdfcer diverges when it no longer does.

### What is whose

| piece | whose | state |
|---|---|---|
| the `#[default]` on `CmykIntent` | **`pdfcer-core`** | filed with the engine |
| deleting the `Naive` variant and its colour maths | **`pdfcer-core`** | filed with the engine |
| the third radio button and its copy | this shell | **done** |
| the divergence note | this shell | **done** — deleted, not reworded |
| what a fresh install gets today | this shell | **done** — seeded, see below |

★ `D:\Dev\pdfcer\` is read-only to this project until fold-in, so the first two
are a hand-off rather than a change I make. The request is
`request_cmyk_default_flips_and_the_naive_formula_goes.md`.

### ★★ You get the new default TODAY, before the engine lands

The engine's `Settings::load` returns its own default for anything a stored file
does not name, so until the `#[default]` moves, a fresh install would still get
black-ink-is-black. Rather than wait, this shell **seeds** `Calibrated` when
there is no stored value — one line, in the one place settings are loaded, with
the reason attached.

⇒ When the engine's default moves, that line becomes a no-op and should be
**deleted rather than left**: a shell that keeps overriding a default it agrees
with is a second source of truth waiting to disagree. It is written so the
compiler finds it — see the request.

★ An operator whose stored file already says `naive` is moved to the new default
on load, because the option is no longer on screen and leaving them on a value
they cannot see or change is worse than moving them.

**Status:** ★★ **BOTH HALVES DONE 2026-08-28.** The engine landed
`Pass 153.0` the same afternoon — `Calibrated` is its default and `Naive` is
deleted — so the shell-side seed is **gone**, exactly as its own tripwire
instructed. It fired on the first build after the `cargo update`, which is the
whole reason it was written as a `debug_assert_ne!` rather than a comment.

★ Two tests moved with it and the second is worth recording. One asserted the
dirty flag by setting `cmyk_intent = Calibrated` — which stopped being a change
the moment `Calibrated` became the default, and failed on a build whose dirty
flag works perfectly. **A test that names a value to prove "something changed"
is coupled to what the default is.** Both now assert their own premise, so the
next default move fails saying *"the test needs a CHANGE"*.

Not driven — he is at the machine.

---

## O51 — ★★★ Inkscape-style scale toggles: line weight follows a resize, if you say so

**His ruling, 2026-08-28**, on reading my answer to the engine about resize
semantics:

> *"if that was the resize question about scaling line weight, etc with resize
> it got the answer wrong. default should be what it said, but there should be
> an option that they do scale with resize. Inkscape has options for this and I
> want the same."*

### ★★★ The correction, and it is about reasoning rather than about strokes

I had told the engine that a resize must **not** scale stroke width, with three
arguments:

1. a CAD line weight is a **drafting standard**, not decoration;
2. a non-uniform scale makes a single `/BS /W` scalar **ill-defined**;
3. Acrobat does not scale a comment's border, and Illustrator ships *Scale
   Strokes & Effects* **off**.

All three stand. The **conclusion** does not.

⇒ **Convergence among reference implementations argues for a DEFAULT, not
against an OPTION.** *"Everyone does X"* and *"nobody should be able to do Y"*
are different claims, and the first does not establish the second.

★★ My own third argument contained the refutation and I walked past it:
*Illustrator ships the toggle off by default* — which means **Illustrator has
the toggle**. So does Inkscape, on the selector tool's control bar. **The
existence of the toggle is the industry answer**; the default is the separate
question, and that one was right.

★ This is the second time a well-reasoned answer of mine has been too narrow in
the same way — see `feedback_use_the_conventional_interaction_never_invent_one`.
The rule there was *the convergence of the product class IS the spec*. The
amendment is: **read the convergence completely.** Every one of those programs
converges on *"off by default"* **and** on *"available"*, and quoting the first
half as though it were the whole is how a correct observation becomes a wrong
conclusion.

### What Inkscape actually offers, which is the parity target

Four toggles on the selector tool's control bar. His words are *"Inkscape has
options for this and I want the same"*, so the target is the set, not one flag:

| Inkscape toggle | pdfcer equivalent | default |
|---|---|---|
| Scale stroke width | `/BS /W`, `/Border`'s width element | **off** |
| Scale rounded corners | — (no PDF annotation equivalent yet) | n/a |
| Move gradients | — | n/a |
| Move patterns | — | n/a |

Plus one the engine raised that Inkscape has no analogue for:

| `/RD` — the inset distances | **on**, and the opposite default to stroke width |

⇒ `/RD` scales because an inset **is a length in the space being scaled**;
leaving it fixed while the `/Rect` doubles changes the annotation's proportions.
That is the reverse of the translate case, where leaving it fixed *preserved*
them — the same key, opposite correct answers under two verbs.

### ★★★ And the engine found something that changes what "off" can even mean

**No per-axis stroke width exists**, in SVG or in PDF: `stroke-width` and `w`
are both scalars. So *"keep the line weight constant under a NON-UNIFORM
scale"* — the toggle-off case, my default — is **mathematically unsatisfiable**
whenever the stroke is drawn through a matrix applied after stroking.

Inkscape hit exactly this (Launchpad #1335376) and closed it **Invalid**: a
mathematical limit, not a defect. Its actual behaviour is to **silently produce
a distorted stroke**.

★★ It transfers to pdfcer unchanged, because an annotation's artwork is placed
through §12.5.5's matrix and a resize makes that matrix a non-uniform scale.
So carrying the appearance stream untouched — which is exactly right for a
*move* — is **wrong for a non-uniform resize**: the stroke distorts whatever the
toggle says.

Two cases, and only one is fixable:

- **An appearance pdfcer authored** can be rebuilt from the scaled geometry at
  the new size, and both toggle states become exactly satisfiable.
- **A foreign appearance** cannot be rebuilt without replacing somebody else's
  artwork with pdfcer's rendering of it. There the honest options are refuse, or
  proceed and **state the residual distortion** — never silently pick a fudge
  factor, which is the one thing the parity reference does.

⇒ The engine's own RAG recommends pdfcer be **better than the parity reference
here, not equal to it.** Agreed, and it is why the grips must report whether a
drag was proportional.

### What this shell owes

1. **The toggles**, where an operator expects them — Inkscape puts them on the
   selector tool's control bar, which is this shell's **Tool row**, not a
   settings dialog. They are a per-drag modifier, not a preference.
2. **The disclosure either way.** *"Border width left at 1.0 pt"* is the
   sentence rule 4 exists for, and the engine's outcome type names which branch
   ran.
3. **Proportionality reported to the verb**, so the engine can distinguish the
   satisfiable case from the one it must disclose.

### ★★ The blocker was already stale when it was written down

*"Blocked on `resize_annotation`, which the engine is building to this shape."*
It had **shipped** — `Pass 151.0`, with `ResizeOptions` carrying all three
fields — and this shell was already calling it. The row was a record of a
conversation rather than of the code, which is this project's fourth-commonest
defect and the reason the standing rule is to verify an absence claim against
source before writing it down.

### What shipped, 2026-08-28

**Three switches on the Tool row**, live whenever the Select tool is armed,
because a modifier is set *before* the gesture it modifies:

| switch | default |
|---|---|
| **Scale line weight** | off |
| **Keep the inner margins the same size** | off, i.e. `/RD` scales |
| **Allow the artwork to distort (borders may come out uneven)** | off |

★★★ **And the derivation they replaced is the interesting part.** `annots::resize`
was passing `scale_stroke_width: uniform` — taking the flag from whether the
drag was proportional rather than from anything anybody asked for. That was a
workaround for a refusal, defensible while no control existed, and it would have
**silently overridden the operator's answer** on exactly the resizes where they
were most likely to have one.

⇒ Making the same mistake twice in one file, in a request that is itself a
correction about that shape of reasoning, was close enough to be worth naming.

★★ **What replaced the workaround is a worded decline, not a different guess.**
The engine refuses when the artwork cannot be rebuilt, and the refusal was
trace-only — the operator dragged a grip, the shape snapped back, and nothing
anywhere said why. It now names the remedy, and **which remedy depends on the
drag**: proportional → *Scale line weight* makes it exact; not proportional →
only *Allow the artwork to distort* proceeds, and the sentence says the result
will be uneven rather than dressing it up.

**Status:** ★★ **DONE 2026-08-28.** Not driven — he is at the machine. The
check is written and unrun: `the_line_weight_switch_reaches_the_resize`, which
reads `stroke=` off the applied line because a screenshot cannot tell a border
that thickened because `/BS /W` changed from one that thickened because the
placement matrix scaled it.

---

## O50 — ★★★ A permanent font-folder setting, with a checkbox for the OS's own fonts

**His words, 2026-08-28:** *"for fonts there should also be a permanent setting
where we can add font locations to use from including just a simple checkbox to
include fonts from the OS installed font folders."*

### ★★ Half of this shipped yesterday, and saying so is not a deflection

**The permanent setting exists.** File ▸ pdfcer ▸ Settings ▸ **Fonts** holds a
list of folders, it persists in `userdata/preferences.txt` as repeated
`font_folder =` lines, order is search order, duplicates are refused, and Tools
▸ Font folders opens the same window as a second route.

It is recorded here rather than left implicit because **he asked for it, which
means he had not found it** — and a setting an operator cannot find is a setting
that does not exist to them. That is a finding about the *surface*, not about
the feature:

- The Fonts group **sits below the Settings window's fold**, with four siblings
  no driven check has ever reached either. `settings_headings_legible` says in
  its own notes that it does not drive the scroll.
- Nothing on the Embed window points at it **except when embedding fails**. An
  operator only meets the setting at the moment it is too late to have set it.

⇒ Both are on this row, not just the checkbox.

### The checkbox itself

**"Use the fonts installed on this computer."** Off by default. On, pdfcer also
searches:

| | |
|---|---|
| `C:\Windows\Fonts` | the machine's fonts |
| `%LOCALAPPDATA%\Microsoft\Windows\Fonts` | fonts installed for **this user only**, which Windows has had since 2018 and which most "install font" double-clicks now write to |

★ **Both, not just the first.** A font the operator installed themselves — which
is exactly the font they are most likely to have installed *for this drawing* —
lands in the second one on a modern Windows, and a checkbox that found only the
machine folder would miss precisely the case that motivated ticking it.

### ★★★ Why it is a checkbox and not the default, and this is the whole design

`app::fonts`' header argues at length that pdfcer must **not** search
`C:\Windows\Fonts` on its own:

> Embedding puts a font **program** — the actual outlines — inside somebody's
> document, which they then send to somebody else. Which font that is, is a
> licensing question with a different answer for every foundry, and a program
> that searched `C:\Windows\Fonts` on its own would be answering it silently on
> the operator's behalf, in a file that outlives the decision.

**A checkbox does not overrule that argument — it satisfies it.** The objection
was never to *using* system fonts; it was to pdfcer deciding *silently*. An
explicit, persistent, off-by-default switch is the operator making that decision
once, visibly, where they can find it again.

⇒ Recorded because the shape recurs: **when a capability is refused on the
grounds that the program must not decide, the answer is usually a visible
setting rather than a permanent no.**

### What ships with it

1. The checkbox, persisted beside the folder list.
2. **The folders it resolves to, drawn under it** — the operator sees
   `C:\Windows\Fonts` and their user folder listed, greyed, as the consequence
   of the tick. A checkbox whose effect is invisible is one nobody can verify.
3. The Embed window's donor rows already name the file each face came from, so a
   face that arrived via the OS folders is already identifiable. No new
   disclosure is owed.
4. A driven check that ticks it and embeds — the existing embed check uses a
   harness environment variable to supply a folder, which is deliberately **not**
   the same path an operator takes.

**Status:** ★★★ **SHIPPED AND DRIVEN, 2026-08-28.** The checkbox is in
Settings ▸ Fonts, off by default, and lists the two folders it resolves to
underneath it — including the per-user one at
`…\AppData\Local\Microsoft\Windows\Fonts`, which is where a plain
double-click installs a font on this machine.

★★★ **And the route now lands on it.** Tools ▸ Font folders opens the Settings
window with the Fonts group expanded and scrolled to, instead of at the top of
ten collapsed headings — which is the reason he asked for a setting that had
shipped the day before. Driven; with the landing removed the check reports the
three headings he would have seen.

---

## O49 — Zoom loses its place past about 300,000%, and I am telling you rather than quietly widening the test

**Found 2026-08-28 by the full driven sweep. Not something you have hit, and I
do not think you will — recorded because it is real and because the alternative
was to make the test stop asking.**

### What the sweep found

Zooming in past roughly **300,000%**, the point under your pointer drifts. Not
by much: at that magnification it is about twenty pixels after seven wheel
notches. Zooming back out from a million percent does the same thing on the way
down.

Reproducible — same notch, twice — so it is a boundary rather than noise.

### Why it happens, as far as I have taken it

pdfcer has **two** ways of remembering where the view is. An ordinary one that
works up to about a million percent, and a high-precision one that takes over
above that. The ordinary one starts losing accuracy at around 300,000%, and the
high-precision one does not engage until roughly 1,200,000%.

**There is a band between them where neither is quite good enough.** The
hand-over threshold was calibrated on a different question — the point at which
the ordinary numbers can no longer address every *pixel* — and holding your
place needs better than that.

### What I did NOT do

**I did not widen the test's tolerance**, which would have made the red go away
in one line. This project has a standing rule about that and it is the right
one: an extreme-end failure is usually the measuring instrument running out, so
the first job is to check the instrument — and here the instrument is fine. It
reads a high-precision number the application already publishes for exactly
this. The drift is the application's.

### What it costs you today

Nothing I can see. Your own reported working range tops out around 800%, and
300,000% is 375 times past that. The page is still drawn, still panned, still
correct; what moves is the *anchor point* under the cursor, by a fraction of a
point.

### What I would do about it

Lower the hand-over threshold so the high-precision path takes over where the
ordinary one actually stops holding, rather than where it stops addressing
pixels. That is a one-constant change **and it is not free** — the
high-precision path has its own two hand-overs, and this project has already
found seven distinct defects in them. Changing when they fire without driving
the whole ladder again would be trading a defect nobody meets for one somebody
might.

**Status:** ★★★ **FIXED AND DRIVEN, 2026-08-28.** Both checks are GREEN, and
the drift now holds at **33 % of tolerance all the way to 105 billion percent**
— it used to breach at 292,415 %.

★★ **The fix was one constant, and the interesting part is that it was never
wrong.** The hand-over fired where an `f32` offset stops addressing every
*pixel* — `2^24` — which is exactly the right question for the CAP that constant
used to be, and the wrong one for a tier hand-over. Holding a point under the
cursor is a **proportional** requirement and addressing a pixel is an
**absolute** one, so the two part company as the zoom rises: the `f32` error in
page points is constant, and the tolerance shrinks. `2^24` → `2^20`.

⇒ **A constant that changes what it gates has to be re-derived, not re-tuned.**
The number was never wrong; the question was.

★ The feared cost did not materialise: the deep tier now engages at 85,700 % on
his own sheet instead of 1,370,000 %, and every stage above it reports the same
33 %. Driven with the pixmap-ceiling check beside it, which also passes.

---

## O48 — Removing embedded fonts does not make the file smaller, and I think that is the wrong answer

**Found 2026-08-28, while wiring Tools > Remove embedded fonts. Not a defect —
a design decision that was made for a different reason and now has a cost.**

### What works

Remove embedded fonts is live. On a document carrying a removable font it takes
the outlines out, keeps every letter exactly where it was, and reports what it
did. Driven: one font removed, 15,025 bytes of program freed.

### And the number is a lie by omission unless the window says this

**pdfcer's Save will not make the file smaller.** It saves by appending your
changes to the end of the file and leaving the earlier version intact — that is
what the Save-a-copy tooltip has promised since day one — so the font outlines
stop being *used* and are still *there*. The file gets slightly bigger.

The window says so, in as many words, right above the button. But the whole
reason anybody removes an embedded font is to shrink a file, so today the
feature does the work and cannot deliver the point of it.

### Why Save works that way, and why it is a good reason

Appending is what lets pdfcer promise the previous version of your drawing is
still recoverable inside the file, and it is what keeps a **digital signature**
valid — a save that rewrites the whole file destroys every signature in it.
Those are not small things and I do not want to trade them away by default.

### What I would build if you want it

A second save that **rewrites the file completely**: smaller file, no previous
version kept, and any signature is gone. Named so it cannot be pressed by
accident — *Save a compacted copy…* — always to a new file, never over yours,
with a plain sentence about what it discards.

That would also help elsewhere: the same rewrite is what reclaims space after
deleting pages or images.

### What I need from you

1. **Build it** — a separate compacting Save, as above.
2. **Leave it** — the window explains the limit and that is enough.
3. **Something else** — e.g. do it automatically when the document is unsigned
   and has no earlier revision worth keeping.

My lean is **1**, as an explicit separate command. It is a day's work and it
makes three other features honest.

**Status:** ★★★ **SHIPPED AND DRIVEN, 2026-08-28.** File ▸ Save ▸ *Save a
compacted copy…*, always to a new file, with the three losses stated before the
picker opens.

Measured through the running binary on a fixture built to be exactly this
problem: **1,709,629 bytes → 43,073**. The fixture is a drawing that had fonts
embedded and then removed — which left it 889 bytes *bigger* than before the
removal, which is the sentence this row opens with, reproduced in a file.

★ Falsified: swapping the full writer for the incremental one makes the check
fail with `1709629` against `1709629` and name the substitution.

---

## O47 — A question, not a complaint: should pdfcer embed its OWN fonts?

**Asked 2026-08-28, by me, and it needs your answer before I build either
side of it.**

Embedding now works. Tools > Embed fonts, on a drawing that asks for a font it
does not carry, finds the file on your machine and puts the outlines in. Driven
end to end on the A1 title block: three fonts missing before, none after.

### The question

pdfcer **ships with its own copies** of the fourteen standard PDF fonts —
Helvetica, Times, Courier and the rest — because it needs them to draw a
document whose fonts are missing. Those are pdfcer's to embed.

Right now they are **not offered**. If none of your font folders answers for a
font, that font is reported as one pdfcer cannot embed, and the reason it gives
is *"add a folder that does"*.

The alternative is to offer them: press Embed on any drawing and every
standard-14 font gets one, with no folder configured at all.

### Why I did not just do it

Because embedding a stand-in **changes what the letters look like** in a
document you send to somebody else, and doing that because a substitute
happened to be available is the *sneaky* half of rule 4. It is your drawing and
your client's screen.

The command-line tool makes it an explicit switch for the same reason. Mine has
no switch, because that window has no settings by design — so this is a choice
about what pdfcer does, not a checkbox I can add without asking.

### What I need from you

One of three:

1. **Never** — leave it as it is. A missing font stays missing until you point
   pdfcer at a folder that has it.
2. **Always** — offer them, disclosed in the window as "pdfcer's own copy" so it
   is never silent.
3. **Ask** — a second button in the window, *Embed, using pdfcer's own fonts
   where yours have none*.

My own lean is **2**, disclosed loudly, because on your machine the alias rung
already substitutes Arial for Helvetica and nobody would call that wrong —
this is the same act one step further out. But it is your call and I am not
taking it.

**Status:** ★★★ **SHIPPED AND DRIVEN, 2026-08-28** — option 2, always,
disclosed loudly. With no font folder configured at all, a drawing asking for
Helvetica now embeds: 3 fonts, 46 KB, every row saying *"none of your fonts
matched, so pdfcer used its own copy. It is a stand-in, not the font the document
asks for."* It is the LAST rung — a real font on your machine wins every time —
and there is a driven check that turning it off makes fail by name.

★ No option number was named, so this took the one I recommended: If he meant option 3, say so and it becomes a second
button in the window rather than a change to the resolver.

★★ **And he added a fourth thing in the same breath, which is O50.** It changes
what this row is worth: with the OS font folders switched on, pdfcer reaches a
real Arial before it ever reaches its own bundled substitute, so the bundled
rung becomes the last resort it was always meant to be rather than the second
one. Build O50 first.

---

## O46 — ★★★ Editing should work like every other graphics program. It does not.

**Asked:** 2026-08-26. **THE LARGEST REQUEST IN THIS FILE**, and it is not a
list of defects — it is a judgement about the whole editing surface. Research
and gap analysis commissioned the same evening, on his instruction, **before any
code was changed**.

> *"The interface for this has gotten so wonky. I can't figure out how to click
> on objects to edit them. For example on the [conformance suite's composite
> page] There are obviously more
> than one item on the page, but when I click on one of the objects all I get is
> the page selected. When I double click on an object it doesn't select — it
> still only has the whole page selected. Also when I have an object selected
> like text the Tool tab doesn't switch to giving me the editable stuff for that
> object. And if I add an image I Expect to click on it to resize but dragging
> doesn't resize. Editing should work like 99% of the graphics programs out
> there. It should be intuitive. I have no idea why I can't just click an
> object, see its properties and edit them in the tool tab, move them, adjust
> them, etc, just like any other software. Somehow we have the most convoluted
> system, or it just isn't working for most of the things I expect it to.*
>
> *Also how do I OCR more than one page? Why does the tool stop at one? Why do I
> have to save a copy instead of just go back into my pdf and save over it or
> save from there? Where is the option to select more than one page? How did we
> end up with the most useless and un-userfriendly of options for the OCR? What
> program in the world works this way out of all the OCR programs out there?"*

### ★ The instruction, which is as important as the complaint

> *"BEFORE you make any changes, you are going to go and research how GUIs for
> graphics editing programs handle editing and navigating and then you are going
> to write a manual on how things should work in pdfcergui, and in parallel while
> you do that you are going to write up the manual for how to currently use
> pdfcergui, then you are going to compare the two documents and see the
> difference and determine just what you need to change."*

**No code changes until those three documents exist.** That is the method he
asked for and it is the right one for this: the symptoms are many and the cause
is probably few, and patching symptoms is how a system becomes convoluted in the
first place.

### The seven complaints, itemised so none is lost

| # | Complaint | Status as of 2026-08-27 |
|---|---|---|
| 1 | Clicking an object selects the whole page instead | ✅ **CONFIRMED BY YOU, 2026-08-27** — *"I checked and clicking works."* Driven by the harness the same day, then used by you on your own file |
| 2 | Double-clicking an object still selects the whole page | **partly** — a click now lands on the real object; a *double*-click on one inside a form does not descend into its subpaths, and cannot until the engine can edit inside a form. Said in words, not silently |
| 3 | Selecting text does not change the Tool tab to that object's editable properties | **done** — the Properties panel reads the canvas selection, and the status bar names what is selected |
| 4 | An inserted image cannot be resized by dragging | **done** — a press on an unselected object selects it and the same drag moves it; placement now arrives selected, so its grips are already up |
| 5 | OCR does one page only — no page range, no multi-page selection | **done** — All pages / this page / a typed range, All being the default |
| 6 | OCR forces Save-a-copy instead of saving into the open document | **done** — recognition is an ordinary edit; `Ctrl+S` saves it, `Ctrl+Z` takes it out |
| 7 | The whole editing model is unintuitive next to other graphics software | in progress — the three research documents are the deliverable, and 2 is the remainder of the click work |

#### ★★★ Complaints 1 and 2: what happened

The audit's verdict was that the engine did not enter form XObjects, so a
page-sized form was a page-sized hit target that won every click at every point.
That was filed as an engine request on 2026-08-26 and **`pdfcer-core` answered it
the next day** — Passes 136.0, 136.1 and 136.2:

* `decompose_page` now descends into every reachable form and returns the
  objects inside on a separate `PageObjects::leaves` list, each already mapped
  into page space and carrying its chain of enclosing forms;
* `hit_test_point_deep` excludes forms as candidates outright and interleaves
  the two lists on one paint order, so a click finds what is drawn rather than
  the wrapper.

#### ★★★ …and what the shell did about it, 2026-08-27 — SHIPPED, NOT YET DRIVEN

Consumed in three commits, each of which left the program working:

1. **`TargetId` became a two-variant type** — `Object(u64)` for the page's own
   paint order, `Leaf(u64)` for an object painted from inside a form. The
   compiler found **sixteen** sites that had to say which list they meant, not
   the 96 this file predicted: the id itself is well contained, and what the 96
   number really counted was places that resolve a paint-order *index*, most of
   which never see a `TargetId`. `page_object_index()` — which answers `None`
   for a leaf — is now the only supported way to obtain an edit operand, so a
   form-relative index cannot reach a page-stream verb by construction. Nothing
   behaved differently at that commit; it was the type change alone, with the
   compiler as the instrument.
2. **The pick went deep.** `hit_test_point_deep`, and the marquee gained the
   same reach so the two gestures cannot disagree. Eight tests over the
   engine's `forms-xobject` fixtures — falsified by putting the shallow call
   back, which turns three of them red, including the one that says a click on
   blank paper inside a page-sized form must select **nothing**.
3. **The surfaces stopped lying.** The status bar says *"Selected: Path ·
   12.4 × 8.0 pt · inside a form"*; Delete on such a selection says *"That
   object is inside a form — pdfcer cannot edit inside one yet"* instead of
   doing nothing; a drag says the same instead of *"nothing selected"* while an
   outline is on screen.

**And the new button: "Select the form".** Since the hit test now excludes
forms outright, a form had no route on the canvas at all — so it is offered as
a deliberate act instead of winning by default. It is on the Format tab and in
the canvas right-click menu, greyed when the selection is not inside a form,
and after pressing it the form is an ordinary object you can move, delete or
copy. Everything drawn inside it moves with it.

### ★★ What you will find that still does not work, said before you find it

| | |
|---|---|
| **You cannot edit an object inside a form** | not a shell decision: `pdfcer-core` writes a paint-order edit to the *page's* content stream, and a form-interior object lives in the form's. `FormLeaf::is_editable()` is `false` for every one of them today. Select the form and move that, or wait for the engine |
| **Double-click will not descend into one** | the Part and Node rungs exist to act on geometry, and there is no geometry to act on here. It stops at the whole object rather than descending into something you then cannot change |
| **The measure tools cannot pick a line inside a form** | the engine's line-pick does not see the leaf list. Filed. On the benchmark CAD sheet that is 10,256 lines the tool cannot see, and it was equally true before today — it was just hidden behind the selection defect |
| **`pdfcer object-list --hit` still answers with the form** | the CLI has not consumed the deep hit test, and its help says it is authoritative for the GUI's behaviour, which is now false. Filed |

### The numbers, measured today

| page | page objects | forms | objects inside them |
|---|---:|---:|---:|
| the conformance suite page you named, p1 | 28 | 4 | **242** |
| `ncored-benchmark-cad-drawing` p1 | 129,758 | 1 | **10,256** |
| `SW41177` p1 | 5,903 | 0 | 0 |

On the first two, nearly everything on screen was outside the model the shell
could select from. On the third — your SolidWorks export — nothing changes at
all, which is what tells you the fix is aimed at the right thing.

★ ~~**NOT YET DRIVEN.**~~ **DRIVEN, AND THEN CONFIRMED BY THE OPERATOR.**

Struck rather than deleted, because the sequence is the point and a later reader
should be able to see it:

1. Written against unit tests over the engine's `forms-xobject` fixtures, and
   said so in these words — *"that is not a report of working software"*;
2. **driven** on 2026-08-27 with the operator off the machine —
   `a_click_inside_a_form_selects_what_is_drawn_there` passed, and was
   **falsified in the same session**: with the shallow `hit_test_point_all` put
   back and the binary rebuilt, the check reports the operator's own sentence
   back at us;
3. **confirmed by him**, 2026-08-27: *"I checked and clicking works."*

Complaints 1, 3, 4, 5 and 6 are closed by that. **2 and 7 stay open** and are
what the rest of this row is now about — 2 because a double-click still cannot
descend into a form, and 7 because the three research documents exist but their
work list is not finished.

### ★★ One measured fact, before any of it

Page 1 of the file he named has **28 objects**, and one of them — a form
XObject — spans the entire sheet.

★★ **The hypothesis I opened the audit with was WRONG, and the audit killed it.**
I guessed that a full-page path with `paint=none` was swallowing the clicks.
It is not: the engine gives an unfilled, unstroked path a proximity band of the
click tolerance alone, so it is selectable only within 6 px of its outline, and
that is correct behaviour. I also said 29 objects; it is 28.

**The real cause is worse and is one level down.** The engine decomposes a page
into a flat list in paint order and **stops at the door of a form XObject** — it
emits the form as a single object and never enters it — then hit-tests that form
as a plain rectangle. On his file the four forms on page 1 contain *the entire
visible body of the sheet*. So the page-sized form wins every click at every
point, and every patch, swatch and panel he can see is **not in the object model
to be selected at all**, at any zoom, with any modifier.

*"All I get is the page selected"* was a precise and accurate report: he was
selecting a page-sized object. It just was not the one I guessed.

### Deliverables

| document | what it is |
|---|---|
| `HOW_IT_SHOULD_WORK.md` | the target interaction model, derived from how Illustrator, Inkscape, Figma, Affinity, CorelDRAW, PowerPoint and Acrobat actually behave |
| `HOW_IT_WORKS_TODAY.md` | an unflattering description of the current behaviour, cited to `file:line`, opening with *"if you just want to select an object, here is literally what you must do today"* |
| `INTERACTION_GAP.md` | the comparison, the four-way split (not implemented / unreachable / gated unexpectedly / working but undiscoverable), the work list ordered by operator-visible return, and **what must survive** |

★ The last of those matters as much as the others. A rewrite that discards
working behaviour because the surface around it was frustrating is the failure
mode here.

**This row stays open until he has used the result and said so.**

## O45 — Selecting a standard leaves Save greyed out

**Asked:** 2026-08-26. **Fixed the same day. Unit-tested and falsified; NOT yet
driven** — see the status at the end, which says so in those words.

> *"When I go to settings and select some of the standards the save button is
> greyed out and I can't save the change."*

### Both halves of that are literally true, and the second explains the first

Save is offered only when something has actually changed since the window
opened — deliberately, so it can tell you whether you have unsaved work. It
compared the **values**.

And **all eight PDF/X and PDF/A presets set exactly the same rendering values.**
That is not a bug and pdfcer already told you so on the selected standard's own
line: the standards genuinely ask the same thing of a *renderer* and differ in
what they ask of a *file*, which is a preflight question. So picking a second
standard moved nothing, and Save was correctly greyed about a draft that really
did equal what was already saved.

### ★ And the half you had not seen yet: your choice was being thrown away

Nothing recorded which standard you picked. On reopening, the window showed
whichever one your *values* looked like — always the first of the eight. **Pick
PDF/X-4, come back, and it said PDF/X-1a.** The window contradicting you about
what you asked for is the worse of the two problems, and you would have met it
next time you opened Settings.

### What changed

**Your choice is now remembered**, in `preferences.txt` beside the other things
that are yours rather than the engine's:

```
chosen_standard = pdf-x4
```

So picking a standard *is* a change, Save lights up, and next week the window
tells you what you chose rather than what it can infer. If you afterwards move
any control by hand, the settings stop being that standard's and the claim is
retired — the window stops showing it **and** the file stops naming it, together,
so the two can never disagree.

★ Nothing about the greying rule changed. There is simply now something to save.

### Status

| | |
|---|---|
| unit tests | 4 new. The main one runs over **every pair** of standards rather than the two you hit, because any two of the eight reproduce it; falsified by removing the one line that records the choice, and it fails naming a real pair |
| **driven** | **NOT YET.** Selecting a radio in the Settings window needs the pointer and you were using the PC. The window now publishes `settings-preset chosen=… stored=… dirty=…` so a check has an oracle — an enabled button and a greyed one are the same size and place, so a screenshot cannot answer this |
| one thing to know | the presets are in a **collapsed group** now (O44d — they had grown to fill the whole window). Open *Rendering standards* and they are as they were |

## O44 — Four things the first COMPLETE driven run found. Two were real, two were the test.

**Found:** 2026-08-26, by the first `ui-verify` run in this project's history in
which every declared check actually launched. **All four resolved the same day**
— and the honest tally is that **two were defects in the program and two were
defects in the tests**, which is worth stating plainly rather than counting four
fixes.

Evidence: `evidence/ui-verify-run-2026-08-26-rotated.txt` for the discovery run.
Every claim below was driven, and every driven claim was falsified first.

### ★★ O44a — The status bar's controls went off the window at a large UI scale. **REAL. FIXED.**

At `ui_scale = 1.80` the zoom control, the fit buttons, Find and the selection
filter were **off the left edge of the window at negative coordinates**, and the
left-hand notes were drawn underneath the fit group. Two points of the zoom
stepper and Find were also clipped off the *bottom*, at **every** scale
including 100 %.

Two independent causes, both now fixed:

* **The bar had no narrowing behaviour at all.** It now sheds, the way Word's
  and VS Code's do — biggest and least essential first. ★ The clause that makes
  that legitimate is enforced rather than promised: a control may only be shed
  if it has another home, checked against the real command registry. **That
  check immediately refused the obvious design** — which would have dropped the
  selection filter first, and the filter has no ribbon command, no menu entry
  and no shortcut. It exists only on that bar. Only the fit buttons and Find may
  go, and dropping the fit group alone is enough.
* **The bar was two points shorter than its own controls**, because its height
  was a constant written for 24-point controls and the shipped theme's are 28.
  It is taken from the theme now.

★ **A finding for you rather than a fix:** the selection filter and the zoom
stepper are reachable **only** from the status bar. If you would like either on
the ribbon, say so — it is one line each, and it would let the bar shed more
gracefully on a small window.

### ★★★ O44b — The Apply button for typed sizes could not be reached at all. **REAL, AND WORSE THAN FILED. FIXED.**

Filed as *"typing a width and pressing Apply does nothing"*. That was wrong in
an important way: **Apply was never pressed**, by anybody, because it could not
be seen.

The Properties panel drew its sections straight into the panel with **no scroll
area around them** — only the read-only metadata rows at the very bottom had
one, nested deep inside. So with an object selected you got Left, Bottom, Width
and Height, and Apply was **below the window edge with no scrollbar and no
gesture that would reach it**. The whole typed-geometry feature was complete,
wired, tested — and unusable.

★ It took a **screenshot** to see it. The coordinates said so all along and
three readings of them still reached the wrong conclusion. One scroll area round
the whole panel, and the driven check now scrubs the Width field, scrolls to
Apply, presses it, and watches the resize reach the engine.

### O44c — Shift-dragging pages between documents. **NOT A DEFECT. RETRACTED.**

It works. The run that reported it was given a **one-page** second document, and
a one-page document cannot be moved out of — by design, and no build could
change that. Re-run against a four-page document it passes end to end: the
target gains the page and the source loses it.

★ My error, not the program's: the check said so in its own skip message and the
first run reported the wrong half of it. The suite's invocation now uses a
multi-page second document.

### O44d — The Settings window published no headings. **REAL, AND NOT WHAT IT LOOKED LIKE. FIXED.**

It looked like a tracing bug. It was not. **Opening Settings showed nothing but
the list of rendering standards** — the presets section had grown to ten radios
and filled the entire window, so every group and every setting was below the
fold. The window was reporting no headings because there were none on screen,
which was exactly right.

The presets are now a collapsible group like every other section — still first,
because a preset sets all of them, but closed. Opening Settings shows the
groups again.

★ **Not changed to a dropdown**, though that is what most applications use for a
preset and it would cost one row instead of one click. You have just been given
the radio list and reported on it; swapping the control while fixing a layout
defect would be improvising. Worth proposing separately.

### ★ And two failures that were the TEST being wrong

Both would have gone on reporting red for ever on your own drawings, which
trains a reader to skip the section.

* **`blend_space`** claimed *"the page's colours have changed and nothing on
  screen says so"* about `SW41177.pdf` — a drawing with no transparency on it,
  which never asks for the colour buffer and is owed no disclosure. It computed
  the crossing from the page's **dimensions**, which says the buffer *would* be
  refused if the page asked for one.
* **`dimension_groups`** reported *"the panel declares no
  `dimension-groups.heading.add` region"* and then, in its very next sentence,
  **"Headings declared: dimension-groups.heading.add."** It contradicted itself
  in two consecutive lines: the region was there, and the dock was still
  settling so its position never held still long enough to click. A
  self-contradicting failure is worse than a silent one — it names a defect in
  the program for a condition that is entirely the harness's.

## O43 — Vertical text should behave like vertical text

**Asked:** 2026-08-26. **Shipped the same day. Driven check written and NOT YET
RUN** — see the status below, which says so in those words.

> *"I have text placed vertically on the bottom left corner of the SW41177.pdf.
> In Adobe when I hover over it the I cursor re-orients itself to match the text
> orientation, and when I select the text it shades each letter as part of the
> same block. when I copy and paste into notepad, I get the text on one line as
> expected. I need pdfcergui to have the same behaviour. as it is now the I
> cursor doesn't reorient and it pastes each letter onto its own line."*
>
> *"The last page has the vertical text."*

### ★★★ Three symptoms, one cause, and the cause is in the engine

`pdfcer-core` places every glyph by the §9.4.4 text rendering matrix and then
publishes four numbers out of it: the origin `x`, `y` — exact — plus the
`advance` and the `size`, **both of which are lengths**. The two basis
*vectors* are reduced to their magnitudes, so **which way the text runs is never
published at all**. Its own rustdoc still calls the advance *"horizontal"*.

Everything downstream then assumes the missing vector is `(1, 0)`:

| symptom | mechanism |
|---|---|
| *"pastes each letter onto its own line"* | the extraction breaks a line whenever the baseline y moves. Text advancing in **y** changes baseline at every glyph, so it inserts a line break between every letter — **71 of them** in your stamp |
| *"shades each letter as part of the same block"* not happening | a glyph box is taken as `x … x+advance` across. For 90° text that is the right size turned the wrong way and hung off the wrong corner, so the wash sits *beside* the letters |
| the I-beam does not turn | nothing the cursor can ask knows the direction |

There is a fourth you did not report and would have hit next: **clicking on a
vertical letter lands on the wrong one.** The engine's hit-test boxes are built
the same wrong way, so a press in the middle of a letter is outside every box
and the nearest-line fallback decides. Found by driving it: a sweep down a
six-letter string selected five, and a sweep along an upside-down one selected
nothing at all.

### What was done here, and what was asked of the engine

The direction is **recovered from the glyphs themselves**, and the measurement
is exact rather than a guess: a *chain* of three or more consecutive glyphs each
sitting exactly one advance from the last, along a common direction off the
page's x axis, is not a coincidence available to horizontal text — within a
line every step is horizontal, and the jumps *between* lines are separated by a
whole line, so they can never be consecutive.

★ **A page with no rotated text on it never reaches any of the new code.** That
is structural, not incidental: the direction census comes back empty and every
branch is keyed on it. Asserted against a real drawing sheet.

The engine request is filed —
`open/request_extraction_drops_the_writing_direction.md` — asking for
direction-aware segmentation and for the direction to be published. When that
lands, the shell-side recovery becomes a fallback and then deletes.

#### ★★★ It landed the next day, and the 1,303 lines are gone — 2026-08-27

`pdfcer-core` shipped Passes 139.0 / 139.1 / 139.2 within a day:
`ExtractedGlyph::direction`, `TextRun::direction()`, `Line::direction`, a public
`glyph_cell()`, and — the one that matters — segmentation resolved into the
**line's own frame** instead of the page's, so the spurious breaks are never
emitted. On their fixture the derived break count went 22 → 3.

So `canvas::textsel::writing` is **deleted**, not kept as a fallback, and the
same is true of the artefact filter that used to drop the spurious breaks: a
filter that removes something no longer produced is a filter that will one day
remove something real.

**What that changes for you: nothing you can see, and that is the point.** Every
one of the eleven rotated-text behaviour checks written for the shell's version
passes with the engine doing the work — one tall band, copies on one line,
upside-down text, the skewed band, the cursor tilt, and horizontal text on a
rotated page unchanged. Your own stamp on `SW41177.pdf` page 36 comes back as
one whole 79-character line, run deliberately rather than left ignored, and
falsified so the green is a result rather than a check that cannot fail.

★ One thing DID change and it is worth knowing: the direction is now sourced
from the §9.4.4 text rendering matrix rather than corroborated from geometry, so
the failure mode this row's own text warned about — *"a vertical label set in
wide letters would have produced no two-glyph run at all, and the feature would
have done nothing, silently, on exactly the page it was written for"* — is gone
by construction rather than by luck.

### Status, stated honestly

| | |
|---|---|
| your own file | **verified.** The stamp on page 36 of `SW41177.pdf` comes back as one line: `W:\Engineering\Products\SAM\SW41177 Toyota Pick up ROPS\SW41177-WELDED FOPS.SLDDRW`. Run it yourself: `cargo test -p pdfcer-gui --lib the_operators_own_vertical_stamp -- --ignored --nocapture` |
| unit tests | 22 new, against real extractions of a real fixture at 0°, 90°, 180°, 270° and 30°. Falsified in both directions before being quoted |
| **the driven check** | **WRITTEN, NOT RUN.** `ui-verify rotated_text_selects_and_copies_as_one_line` drives the release binary, sweeps the string, asserts `chars=6 quads=1`, asserts the cursor traced `deg=90`, and reads the OS clipboard from outside the process. It needs the pointer and the foreground, and you were using the PC. **Say when and it runs.** |
| the 30° case | the band it *marks* is a true parallelogram; the wash it *paints* is that band's bounding box, so it over-covers at the corners. Named rather than discovered. Quadrant rotations — every one a CAD exporter emits — are exact |

## O42 — Let me set the colour-blending buffer size myself

**Asked:** 2026-08-26. **Measured and filed with the engine the same day; needs
one change there before the setting can exist.**

> *"can the size of the buffer be increased? Allow the user to set the size up
> to the maximum possible?"*

**Shipped, 2026-08-26.** Settings ▸ Colour ▸ **"Colours changing when you
zoom"**. Type `default`, or a size — `512mib`, `1.5gb`, or a plain number of
bytes. It is uncapped, with no guard and no preflight, exactly the treatment you
chose for the zoom limit; the window states the cost and does not prevent the
choice.

★ **And you will very likely never need it**, because O41 below is fixed as
well: pdfcer now stops asking for a page image bigger than the colour buffer can
handle, so the colours no longer change with the zoom at all. The setting is
there for the case the automatic fix cannot cover — a very large monitor, where
even the visible-part render can exceed the default.

**Driven:** `ui-verify blend_space`, and the funnel that carries the number to
the renderer has its own test — falsified by breaking the one line that carries
it, because a settings field that saves a number and changes nothing would be
worse than not offering it.

**What it would cost, measured** — corrected 2026-08-26, see below:

| you want correct colours up to… | one buffer |
|---|---|
| 579 % (the zoom you were testing) | 302 MB — barely above today's 256 MB |
| 800 % | 641 MB |
| 1035 % | 1.0 GB |
| 1200 % | 1.4 GB |
| every zoom pdfcer allows on A4 (1946 %) | **4.0 GB**, plus the page image beside it |

**★ Two corrections to what I told you earlier, and both are mine.**

**(1) The percentages were labelled A4 and were not.** The page I measured on is
`596 × 791 pt`, which is neither A4 (`595 × 842`) nor Letter (`612 × 792`). The
mechanism and the bisection stand exactly as measured; only the label moves. On
real A4 the cap is reached at **518 %**, not 534 %, and the top of the whole-page
tier is **1946 %**, not 2071 %.

**(2) *"About 5 GB is the maximum"* is too low, which is the dangerous
direction.** That figure is for **one** buffer. A page with nested transparency
can hold several page-sized ones at once, so **peak memory can be about four
times the number you choose**. Pick with that in mind — the Settings control
will say so on its own line rather than leaving you to find out.

It also costs **about 50 % more time**: measured on the same page at the same
pixel count, blending in print colours took 1.4 s against 0.9 s. Correct colours
are slower, which is the actual trade.

**★★ And a finding that changes what I told you earlier.** I said the better fix
was for pdfcer to render only the visible part above that limit, which needs no
extra memory. That is true on a small screen and **not true on yours if it is
1440p or bigger**: the visible part plus the margin pdfcer renders around it
already needs 281 MB at 1440p and 633 MB at 4K — both over today's cap. So the
cap has to grow *as well*, or a big monitor gets approximate colours at every
zoom. Both changes are needed, not one.


## O41 — Colours change with the zoom level

**Asked:** 2026-08-26. **Cause found and disclosed the same day; the real fix is
filed with the engine.**

> *"seems I get different results depending on Zoom level. The [shading] boxes
> for example on zoom out the colors between our rendering and the references
> don't match, but they do when I am zoomed in. up to 474% they are mismatched,
> but at 579% they match. There's little problems like this in the rendering in
> others too, so probably all of them are related to one bug hopefully."*

**Your hunch was right — it is one bug, and your bracket contained it.**

pdfcer blends a page that uses transparency in *print* colours (CMYK), which is
the correct way to do it. That takes a big working buffer, and the engine caps
it at 256 MB. Past the cap it falls back to blending in screen colours instead.
On an A4 page the cap is reached at **zoom 534 %** — between the 474 % where you
saw a mismatch and the 579 % where you saw agreement. Measured: crossing it
moves those patches by up to 16 levels out of 255.

**What you get today:** the status bar now tells you when it has happened —
*"Colours are approximate at this zoom … zoom out to see the exact colours."*
Nothing is marked on the page itself.

**FIXED, 2026-08-26, and the colours no longer depend on the zoom.**

pdfcer should never have asked for a page image that big. Above a *different*,
much higher limit it already renders just the visible part instead — and a
visible-part render stays under the colour cap at any zoom. So the switch-over
now happens at the **colour** cap as well, and the page keeps its ink all the way
up.

**Driven, on the file that shows it.** At 801 % zoom — well past the 534 % where
this same page used to lose its ink — the trace now reads
`cmyk_buffer=true refused=0`. Before the change it read `refused=1` and the
status bar apologised. `ui-verify blend_space` asserts it, and its assertion was
falsified by disabling the mechanism and watching it go red.

### ★★★ The part worth knowing: it does NOT apply to your drawings

The obvious version of this fix applies the colour cap to every page, and it
would have been a serious regression **for you specifically**:

* on your own D-size sheet (1584 × 1224 pt) the cap falls at **263 % zoom** —
  well inside the range you work in;
* and that sheet is line work with **no transparency on it at all**, so it never
  asks for the colour buffer and nothing whatever would have been gained;
* about **0.4 %** of real documents use the buffer at all — 15 of 4,012 in the
  engine's own corpus.

So pdfcer **learns** instead: the renderer reports, on every page image, whether
it blended in ink, and only a page that has been seen doing so gets the lower
cap. Your drawings keep free panning at every zoom, exactly as they do today,
and a print-ready file gets its colours fixed. Asserted both ways.

★ **`534 %` in the paragraph above was mislabelled as A4.** On real A4 it is
**518 %**. The bracket you gave — mismatch at 474 %, agreement at 579 % — still
contains it, and nothing about the diagnosis changes.

**Verified:** driven — `ui-verify blend_space` zooms past the crossing and
checks the line appears, and that it is absent below it.

## O40 — Only one standard was selectable in Settings

**Asked:** 2026-08-26. **Shipped:** 2026-08-26.

> *"in the settings for the standards compatibility I can only select
> (ISO15930-1, -4). I want to be able to select all of them and especially
> PDF/X-4 (ISO 15930-7)."*

**You were describing it exactly.** The control worked out which preset was
selected by comparing your settings against each one — and all eight of the
PDF/X and PDF/A presets set *identical* rendering answers. So whichever you
clicked, it matched PDF/X-1a first and the dot jumped back there. All nine are
selectable now.

★ **But it will not change what you see, and the window now says so.** Those
standards differ in what they require of the *file* — embedded fonts, an output
intent, whether transparency is allowed — which is a preflight question. What
they ask of a *renderer* is the same, so pdfcer gives them the same answers.
Switching between them changes nothing on screen. Worth knowing before you use
it to compare against the conformance tests.


## O39 — All the form buttons working, and clicking a field shows its properties

**Asked:** 2026-08-26. **Shipped:** 2026-08-26.

> *"can you get all the form buttons on the ribbon working next along with
> adding all the form feature buttons. when I click one I should be able to
> click on the canvas to place the position or drag a box for size then a pop up
> lets me set the details for the feature."*
>
> *"remember last settings and leave push buttons on the ribbon but greyed out
> for now. also don't forget that when I click on an existing form field on the
> page it's properties should come up in our side pane for editing it's
> properties."*

**What you get.** Five buttons on Edit ▸ Forms — text field, check box, radio
button, drop-down, button. Click one, then click the page to place it at its
usual size, or drag a box for an exact one. A window asks for its details.
Nothing is added until you press Add, so a mis-drag costs nothing. The settings
you accept carry over to the next field you place. Click a field that is already
on the page and its properties appear in the Properties pane, where you can
rename it or delete it.

**The push button is greyed**, as you asked. If you reach it another way — a
keyboard shortcut, say — it now tells you why instead of doing nothing.

**Three things worth knowing:**

1. **In Edit mode, clicking a field selects it rather than filling it.** That is
   how every program that both fills and authors forms behaves, and it is the
   only way one click can mean both things. Filling on the page still works in
   Read and Review, and the Forms panel fills in every mode.
2. **Names must be different, and radio buttons are the exception.** Two fields
   sharing a name are ONE field with two boxes — type in either and both change.
   pdfcer numbers new fields so that cannot happen by accident. Radio buttons in
   one set are *supposed* to share a name, so those keep theirs and get
   different values instead.
3. ~~**Required, read-only, the tooltip and the border can only be set when a
   field is placed.** The engine has no way to change them afterwards yet.~~
   ★★★ **WRONG, and fixed on 2026-08-27 — see the note below.**

### ★★★ Correction, 2026-08-27: point 3 above was false the day it was written

**You can change a placed field's properties now.** Click a form field on the
page and the Properties pane offers **Required**, **Read only**, a **tooltip**,
and — for a text field — **multiple lines**, **hide as typed**, **equal cells**
and a **maximum length**. A drop-down gets its own two. Each is one press and
one Ctrl+Z.

★ **The pane used to tell you to delete the field and place a new one.** That
was bad advice as well as unnecessary: deleting a field loses its name, the
value in it, and its position in the tab order — all three of which anything
importing data into the form keys on. The sentence is gone.

**What went wrong, said plainly, because it cost you a day.** `edit_field` and
`edit_widget` shipped in the engine on 2026-08-26 — *the same day* that
sentence was written, and three commits before the version this build uses.
The engine also wrote us a full page of design notes saying so, which sat
unread. So the program spent a day telling you it could not do something it
could do.

★ The general shape, and it is the one this project keeps meeting: **a claim
that something is missing from the engine has a shelf life**, because the
engine moves daily. That claim was true when written and false within hours,
and nothing in either repository can fail a test about it.

**Still not editable, and now it is the honest list:** a box's **size**,
**border** and **visibility**. Those belong to one placement rather than to the
field — a field drawn in three places has one "required" and three borders —
and they are the next piece of work rather than a limitation.

**Verified:** driven. `ui-verify form_field` launches the real program, arms the
tool, clicks the page, watches the field get created, then clicks an existing
field and checks the Properties pane actually drew.


## O38 — A rendering preset for PDF/X-4 (ISO 15930-7) conformance, and a standards selector

**Asked:** 2026-08-25. **Investigated, not yet built.**

> *"I'd like a preset setting for rendering things to what the [print
> conformance suite] page needs to render correctly. We can't call it [that],
> but since it is for conformance to PDF/X-4 (ISO 15930-7)... I noticed touching
> some of our presets caused some test to show up as failed... maybe we should
> have a dropdown to select view options between the different standards."*

### ✅ Done immediately: the suite is no longer named here

We were naming it in two places. Both scrubbed, and
`tools/check-suite-name-absent.py` now fails the build if it comes back —
carried across from the engine, which had already made the same ruling. 18 gates.

### ★★★ MEASURED: the rendering settings change this file on every page

Not a theoretical concern. Rendering all six pages twice, changing **one**
setting — how images are sampled when drawn smaller than their pixel grid:

| page | pixels differing by >8 | worst channel delta |
|---:|---:|---:|
| 1 | 0.04 % | 95 |
| 2 | 0.27 % | 98 |
| 3 | 0.93 % | 139 |
| 4 | 0.31 % | 99 |
| 5 | 0.19 % | 64 |
| 6 | 1.02 % | 100 |

**Every page differs.** And the shape of the difference is the diagnostic part:
a *small area* changing by a *large amount*. That is not anti-aliasing spread
thinly over a page — it is specific patches shifting colour, which is exactly
what you described.

★ **Disclosure: I changed that setting today.** Image minification went from
point-sampling to smoothing this morning (O35), on your instruction, and this
measurement says that change moves every page of this file. Your report that
"touching some of our presets caused some test to show up as failed" may be
about your own change or about mine — the numbers above cannot tell us which,
but they do say the effect is real and worth pinning. **If you want it back the
way it was, it is one control in Settings ▸ Images and it stays changed.**

### What a preset is, and why it is the right shape

Not a new rendering mode — a **named bundle of settings that already exist**.
About seven of the twenty-three settings have a *render* radius, and each one
exists because the standard is genuinely silent and pdfcer had to choose. A
preset says: *for this standard, choose these.* Everything stays individually
editable afterwards.

Your "dropdown to select view options between the different standards" is the
same mechanism with more than one entry, and that is how it should be built:

- **pdfcer (recommended)** — today's defaults, including the two you ruled on
  personally (neutral black, and now image smoothing)
- **PDF/X-4 (ISO 15930-7)** — the conformance answers

### ✅ The mechanism is BUILT (2026-08-25), on your instruction to proceed

Settings ▸ top of the window, above every group because it sets all of them.
One entry today — **pdfcer recommended** — which is the half of your request that
was never blocked: you had changed several settings while investigating and
wanted a way back.

★ It restores the two answers **you** ruled on personally rather than reverting
to the engine's defaults: neutral black for line art (2026-08-08) and smoothing
shrunk pictures (2026-08-25). A "recommended" preset that quietly undid your own
decisions would be resetting, not restoring, and there is a test that fails on
the day the engine adopts either — so the restatement can be removed rather than
silently becoming a no-op.

★★ **PDF/X-4 appears by adding one entry to a list, and nothing else.** No
control to write, no layout to touch. Until its values exist it is *absent*
rather than greyed — R9 — because a greyed row labelled with a standard's name
would carry that standard's authority with none of its content.

Verified on screen, offscreen: the row publishes `settings.presets` at
`614 × 117 pt` at the top of the window.

### ✅ SHIPPED 2026-08-25 — ten standards, and each says how much it can back up

The engine answered within the hour, and answered better than asked: not a table
of six values but an API, with **every value graded for evidence quality**.

**Ten choices** now: pdfcer's own answers, plus PDF/X-1a, X-3, X-4, X-5g, X-6,
PDF/A-1, A-2, A-4 and PDF/UA-1.

★★★ **The important part is not the dropdown — and here is what it says.**
Choosing a standard now tells you how much of itself it can actually back up:

| standard | stated by the standard | inferred | chosen by pdfcer |
|---|---:|---:|---:|
| PDF/X-1a | 4 | 0 | 2 |
| **PDF/X-4** | **1** | 2 | 3 |
| PDF/A-2 | 1 | 0 | 5 |
| PDF/UA-1 | *sets nothing — that is its answer* | | |

**Exactly one of PDF/X-4's six answers is stated by the standard it is named
after.** Anyone pressing a button marked ISO 15930-7 would reasonably assume
six. It also names what each standard does *not* reach — in the same words as
the controls further down the window, so you can go and look.

 Only **one** of PDF/X-4's six
answers is a claim about the standard at all, and even that one is *implied*
rather than *sourced*. So choosing a standard also shows what it does **not**
say — by name, not blank — and any disclosure it owes you, quoted from the
standard rather than paraphrased. A row that showed the name and hid the grading
would be exactly the over-claim this request was careful to avoid.

★★ **Your black-generation question turned out to be the wrong question, in a
useful way.** I filed it as *contentious* — your 2026-08-08 ruling versus a
conformance render. The engine's answer: **no setting of it is conformant**,
because every PDF/X level guarantees a measured definition of ink and this
control picks among fixed built-in tables. So the two were never in tension.
It is one control standing in for something pdfcer cannot do yet, and the preset
says so on screen rather than leaving a colour conversion that silently did not
happen.

★ **PDF/UA is listed and correctly changes nothing** — measured, not assumed:
zero rendering requirements across all 197 of its rules. Listed rather than
hidden, because *"nothing, and here is the measurement"* cannot be mistaken for
unfinished work, whereas a missing entry can.

And the image-smoothing change from this morning is **gone as a special case**:
the engine adopted it as its own default, so the one-time migration deleted
itself exactly as designed.

## M1 — The PC starts pdfcer unreliably. The laptop does not. It is the PC.

**★ SETTLED 2026-08-26 by your laptop test, and the conclusion is the useful
part: pdfcer is exonerated.** The same portable build, the same files, works
normally on the laptop and fails roughly one launch in three on the PC. That is
a machine difference, not a program defect, and no more of my time goes on it.

**What this costs, and it is worth knowing rather than rediscovering:** the
automated test suite launches a fresh copy of pdfcer for every check, so on the
PC about a third of them cannot start. Those show up as skips that look like
failures. Any future session driving the suite **on this PC** should expect that
and not go hunting.

★★ And my earlier diagnosis was **wrong**, which is worth stating plainly rather
than quietly dropping. I found OneDrive holding 404,000 file handles, established
by controlled test that my publishing was feeding it, restarted it at your
request, and watched the count fall to 1,179 — and the crashes **carried on at
the same rate**. So the handle leak was real and worth fixing, and it was not the
cause. Correlation, measured carefully, and still the wrong mechanism.

The publishing rule stays regardless: 27,000 handles per published build is a
genuine cost whether or not it crashes anything, and the rule is in the packaging
tool with the measurement beside it.

### Original report — the handle leak, which was real but was not the cause

**Found 2026-08-26 while testing. Not a pdfcer bug — but it bites pdfcer.**

Roughly a third of my automated tests could not start the program at all. It
dies before showing a window, with a Windows error about **"not enough memory
resources"** coming from the accessibility layer.

**Measured cause:**

| process | open handles |
|---|---:|
| **OneDrive** | **349,208** |
| Outlook | 51,751 |
| Explorer | 12,206 |

349,000 handles in one process is roughly a hundred times normal. Windows starts
refusing to hand out the resources a new window needs, and pdfcer is simply the
next program that asks.

★ **It is intermittent, not constant** — three launches in a row gave two
successes and one crash, and pausing between them helped. So you may have seen
pdfcer fail to open occasionally and put it down to bad luck. It probably was not.

★★ **What it costs you:** any program can hit this, not just pdfcer. A restart of
OneDrive (or of the machine) will clear it. I have not touched it — that is your
sync and your call.

### ★★★ It IS me, measured — and I have changed what I do

I said I might be contributing. I tested it rather than leaving it as a guess,
by taking a reading, publishing nothing for half an hour, and taking another:

| period | publishes | handles gained |
|---|---:|---:|
| ~2 hours | 2 | **+55,000** |
| 32 minutes | **0** | **+6** |

Four orders of magnitude apart. **Each build I mirror to OneDrive costs your
machine roughly 27,000 handles, and OneDrive never gives them back.**

**So I have stopped publishing everything.** From now on a build goes to OneDrive
only when there is something you would actually notice — a fix you can feel, a
feature you asked for. Documentation, tests, refactors and engine re-pins with no
visible difference are commits only. The rule is written into the packaging tool
itself, with the measurement beside it, so it survives me.

★ **This does not undo what has leaked.** The 404,000 handles already taken stay
taken until OneDrive is restarted — which is worth doing, because at this level
roughly **one program launch in three fails**, and not only pdfcer's.

★★ And the error message is actively misleading, which is why this was never
going to be reported as a pattern: Windows says *"not enough memory resources"*
while the machine has plenty of memory. It is **handles**, not memory — measured
at 404,179 handles against a 15 MB working set.

## E3 — OCR put every word in the wrong place on rotated pages

**Not asked — found by the engine and fixed.** 2026-08-26, commit `fe087c4`.

Scanned pages are usually rotated by the *scanner driver* writing a rotation
flag rather than by turning the pixels. pdfcer honoured that flag when drawing
the page and **not** when placing the recognised words — so on any quarter-turn
page, every word ended up on the wrong axis at the wrong scale.

★ You could never have reported this, because there is nothing to see. The text
layer OCR adds is invisible by design, so a page with every word misplaced looks
identical to a page with every word right. The only symptom is that searching or
selecting picks the wrong thing — which anyone would blame on the recognition,
not the geometry.

**★ RE-MEASURED 2026-08-26, and the answer is better than the retracted one.**
Against the benchmark drawing's own text: 72 → 56.5 %, 100 → 56.7 %, 150 →
54.5 %, 200 → 53.9 %, 300 → **35.1 %**. So the headline you were given
originally is *confirmed* — **more scanning resolution makes OCR worse, and the
conventional 300 DPI is the worst of the five** — but the "150 is the sweet
spot" part was noise. The truth is that anything from 72 to 200 performs the
same, and then it falls off a cliff. pdfcer's setting sits inside that flat
range, so nothing needed changing; it was right for the wrong reason and is now
right for a measured one.

These are still dense CAD drawings, which are the hardest thing to read. On
ordinary text the engine now reads a blurred, skewed, noisy scan at **47 of 47
words**.

**The original OCR accuracy figures were withdrawn.** The engine's bundled
text-detection model had never worked. The numbers I reported — including "150
DPI is the sweet spot at 44.7 %" — were measurements of noise, not of pdfcer.
They are marked as retracted rather than quietly corrected, because the
*reasoning* behind them is probably still sound even though the values are not.
For scale, the fixed engine now reads a realistic synthetic scan at **47 of 47
words**. A proper re-measurement is outstanding.

## E2 — "Redact every match" could report success and leave the text in the file

**Not asked — found by the engine and fixed.** 2026-08-26, commit `a2518e5`.

The sibling of E1 below, and the dangerous one. Some PDFs store text with no
record of which letters it is — it renders and prints perfectly, and nothing can
search it. Ask pdfcer to redact every occurrence of a name in such a file and it
would mark nothing, report success, and leave the name in the document. Then you
send it.

The redact panel now says so, in the strongest wording anywhere in pdfcer: how
many fonts could not be read, and that any matches inside them **were not marked
and are still in the file**.

★ It is worded as a consequence rather than a mechanism, because on this one
operation there is no undo and no second chance to notice.

## E1 — Find said "no matches" over text it could never have searched

**Not asked — found by the engine and acted on.** 2026-08-25, commit `9f6ec1b`.

**The defect you would never have reported as a bug**, because it does not look
like one. A search can return "No matches" for a word that is plainly on the
page. Two situations produce that identical answer: the word really is not
there, or **the document stores its text in a way that records no letters** —
so nothing could ever have matched. The text renders perfectly. It prints. It
simply cannot be searched, and Find used to answer that with a confident "No
matches".

Find now says how many fonts in the document store unsearchable text, with a
hover explaining what that means and that recognising the page fixes it.

★ **Acrobat has exactly the same limit** and says nothing at all. This is a gap
in the *file*, not in pdfcer, and the wording says so — calling a file's own gap
a tool limitation would send you looking for a better tool that does not exist.

★★ It appears in the Find bar, never as a mark on the page. Marking content that
renders correctly would be a second way of drawing the same thing, and two ways
of drawing one thing drift apart.

Engine re-pinned to v0.11.0 for it.

## O37 — All the font tools Word has

**Asked:** 2026-08-25. **SHIPPED AND DRIVEN, 2026-08-27** — awaiting your verdict.
`RIBBON_SCALING.md` §6c.

### ★★★ What you can do now

**Sweep some text on the page and the Properties panel grows a "This text"
section.** Font, size, Bold, Italic, colour. Every control writes into the open
document; every one is a single Ctrl+Z.

**The route, because it is not guessable and that is a fair complaint:** you are
in Edit mode, so a drag with the Select tool draws a marquee round objects.
Press **T** first — that arms the text tool — then sweep across the words. The
panel follows.

★ **That is a discoverability gap and it is ours, not a limitation.** Nothing on
screen tells you to press T. It is written up and it is the next thing on this
row; it is here rather than quietly omitted because you would have found it in
ten seconds and concluded the feature was missing.

### ★★★ The table in the "Step 1 done" section below is WRONG, and it is left in

Every ✗ in its "on EXISTING text" column is false. `EditSession::format_text`
had shipped three weeks before that table was written, and had been extended
twice since. It reached this project only as a paragraph inside a note about
something else — which is the engine's own recorded defect, and half ours: an
absence claim about a crate you do not build is a claim about **every route**,
and that inventory checked one verb.

Struck rather than deleted, because a wrong table that cost six weeks is worth
more as a warning than as a gap.

### Driven, and falsified

`restyling_selected_text_reaches_the_document`, against **your** SW41177 title
block: Edit mode, press T, sweep, find the section, press Bold. **19 show
operators restyled across 14 runs in 1.1 seconds.** Falsified in the same
session by cutting the panel section out and rebuilding — it then fails with
your own symptom, *"266 characters are selected and the Properties panel says
nothing about them"* — and passes again when it is put back.

★ **Driving it found four defects the eight unit tests could not**, and the
first is the one worth knowing: a *run* of text is not a *show operator*. On
your drawing a title-block cell is one run made of several `Tj`s, so restyling
"the run" asked the engine for something it correctly refused, and the first
press turned one piece of a fourteen-piece selection bold and stopped. The unit
is the operator now.

### What is still missing, said before you find it

| | |
|---|---|
| **The Format tab has no Font group yet** | the panel is built first by design — `RIBBON_IA.md` §5.8 says so, because the tab's contents are a subset of the panel's and building the tab first means writing the editors twice. The tab also needs a decision about when it appears that is yours, not mine: today it appears on an *object* selection and a text sweep is not one |
| **Clicking a text OBJECT does not raise the section** | only a sweep does. The engine pins text by *run*; a clicked object is a paint-order index; nothing maps between them, and guessing would restyle text you did not select in a file you then send to somebody |
| **One sweep over N pieces is N presses of Ctrl+Z** | the engine has no undo-grouping verb. You are told the count rather than left to find out |
| **A CMYK or spot-colour run shows a sentence instead of a swatch** | changing it here would convert your ink to screen colour permanently, on a document heading for a printer that cares |

---

### The original inventory, ~~kept~~ struck — see above

**Planned**, not started.

> *"We should also have all the font tools available that Word does."*

Deliberately not started alongside the scaling work, because it is a
**capability** question and not a layout one.

### ✅ Step 1 done — the inventory, read out of the engine source

★★★ **The headline, and it decides the whole shape of this request: pdfcer can
choose how text looks when it is CREATED, and cannot change how existing text
looks at all.** `EditSession`'s text verbs are `add_text`, `edit_text` and
`delete_text_run` — and `edit_text` is find-and-replace that **re-encodes into
the run's existing font**. There is no restyle verb. (`set_font` exists, but it
is a low-level content-stream writer, not a session verb.)

| Word ▸ Home ▸ Font | on NEW text | on EXISTING text |
|---|---|---|
| Font name | ✅ 14 built-in faces, or embed any donor font | ❌ |
| Font size | ✅ | ❌ |
| Grow / shrink font | ✅ (arithmetic on the above) | ❌ |
| **Bold** | ✅ — as a *face*: Helvetica-Bold, Times-Bold, Courier-Bold | ❌ |
| *Italic* | ✅ — likewise: Oblique / Italic faces | ❌ |
| Bold + italic together | ✅ — the four combined faces exist | ❌ |
| Font colour | ✅ | ❌ |
| Alignment (L/C/R/Justified) | ✅ per text block | ❌ |
| Line spacing | ✅ (leading) | ❌ |
| **Change case** | ✅ | ★ **✅ — and this one is free** |
| Underline | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Strikethrough | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Highlight colour | ❌ as a text attribute | ✅ **as an annotation**, already shipped |
| Superscript / subscript | ❌ | ❌ |
| Character spacing / kerning | ❌ | ❌ |
| Text effects (shadow, outline, glow) | ❌ | ❌ |
| Clear formatting | ❌ | ❌ |

### Three findings worth acting on separately

**1. Change case is shippable today, with no engine work.** It is a string
transform followed by `edit_text` — and `edit_text` re-encoding into the run's
*existing* font, which is a limitation everywhere else on this table, is
exactly what makes case changing work: the glyphs stay in the same face. UPPER,
lower and Sentence case are the three worth having. This is real Word parity
for one afternoon.

**2. Three of Word's buttons already exist here, as something else.**
Underline, strikethrough and highlight are **annotations** in pdfcer
(`markup.underline`, `markup.strikeout`, `markup.highlight`) rather than
character attributes. ★ That is not a lesser answer for a review tool — it is
arguably the right one, since an annotation is reviewable, attributable and
removable without touching the page content. But it means *"we should have all
the font tools Word does"* is already **half true in a way the Font group would
hide**: putting an Underline button in a Format tab that authored a text
attribute would create a second, incompatible underline. **Recommend: do not
add these.** They are on the Markup tab, where they belong.

**3. The real gap is one capability, not fourteen buttons.** Everything marked
❌-on-existing-text is the same missing verb: *restyle a selected run*. Bold,
italic, size, face and colour on existing text are one engine feature with five
front ends. Filing five requests would misdescribe it.

### Still to do

- ✅ **Step 2 done** — the engine hand-off is filed as
  `request_restyle_an_existing_text_run.md`, deliberately as ONE request. It
  asks the two questions only the engine can answer: whether a restyle is
  representable for an arbitrary run at all (swapping Helvetica for
  Helvetica-Bold changes every advance width, so the run reflows or overruns),
  and whether the honest scope is narrower — restyling only text pdfcer itself
  authored, where the metrics are already known. If it is narrower, we would
  rather disclose a narrow capability than ship a wide-looking one.
- **Step 3** — the IA amendment. pdfcer's text lives under Edit ▸ Content and the
  contextual **Format** tab; there is no Home tab, and the Format tab is the
  natural home for anything acting on a selection.

★ The target remains the capability list, not the pixel layout: *"everything
Word lets me do to text, pdfcer lets me do to text"* — not a copy of two combos
and fourteen buttons onto a different selection model.

## O36 — Sections re-wrap onto more rows, and the scroll arrow is authorised

**Asked:** 2026-08-25. **Planned**, not started. `RIBBON_SCALING.md` §6a, §6b.

> *"put it in the plan to update so that tools within sections will re-wrap
> onto more rows when I resize, and do the scroll like Word. BTW the Font
> section in Word will wrap tools onto 3 lines when the window is narrowed
> enough, and other tools wrap in a similar way too."*

★★★ **He corrected a factual claim of mine, and he was right.** I had written —
in this file's O33 row, in a module header and in a commit message — that Word
does not re-wrap groups by window width. It does: the Font group is 2 rows at
1900 pt and 3 rows at 1000 pt, and **both photographs were already in
`evidence/word-ribbon/`** when I wrote the opposite. I had compared 1300 with
800, and by 800 the group has already collapsed, so the reflow appears in
neither frame. Sampling either side of a transition and concluding there is no
transition. O33's answer is corrected on the record rather than quietly edited.

**Scroll arrow: settled, no longer a question.** It replaces the `⏷ N more`
dropdown rather than joining it. Sequenced *after* the re-wrap, because
re-wrapping will move the width at which the dropdown appears again and there
is no sense tuning a scroll step against a threshold that is about to change.

## O35 — Image quality worse than Acrobat on normal pages

**Asked:** 2026-08-25. **Shipped:** 2026-08-25.

> *"there was also an update to an image quality setting to discard smaller
> details than the screen sees a while ago that I think has been enabled by
> default because image quality is a little worse on normal pages than it was
> whereas before it was on par with acrobat reader — this setting should be an
> option in our settings and disabled by default."*

**You named the mechanism exactly.** Images drawn smaller than their own pixel
grid were point-sampled: one texel per output pixel, the rest discarded. That
is every scan and every CAD raster at anything under 1:1, and the engine's own
note on it says *"aliasing, shimmer, dropped hairlines"*.

**One half of the report was a hypothesis and it was wrong, which is worth
saying because it would have sent me hunting.** It was not enabled recently —
it has been the shipped default all along, and wiring the setting through the
GUI changed nothing, because the render options and the settings file carry the
*same* default. So there is no regression commit to find; there is a default to
decide, and that decision is yours and is now made.

It was already an option (Settings ▸ Images, *"Shrinking a large image to
fit"*). What was wrong was the default. Changing only the default would have
fixed **nobody**: every real installation already contains an explicit
`image_minify = point_sample`, written by our own save into the engine's
generated template — both of your settings files did. So it ships as a
**one-time migration** with a marker in `preferences.txt`: flipped once,
recorded, and if you ever set it back it stays back.

**Verified by driving** three cases: unmarked installation flips and records;
second launch does nothing; marker present plus a deliberate `point_sample`
survives untouched.

**Engine hand-off filed** — `pdfcer-core` grades its own default "a guess" and
names the exact evidence that would flip it: a viewer-behaviour comparison.
You just supplied one, against Acrobat, on your own drawings.

## O34 — The print dialog grows for ever after printing

**Asked:** 2026-08-25. **Shipped:** 2026-08-25, commit `deb9853`.

> *"the print dialogue has a bug that when I press print, instead of closing
> after printing it just keeps expanding its size in little steps to infinity."*

It did. The footer drew its buttons first and the *"Sent N pages"* message
after — and the button pair uses a right-to-left layout, which anchors to the
right edge of whatever width it is offered whether it needs the room or not.
Anything placed after it lands past that edge. The dialog host then grows a
window whose content is wider than it is, the wider window offers a wider row,
the message lands past the new edge, and round it goes.

**Measured:** in a 400 pt row the old ordering produced 481.9 pt and the fixed
ordering produces exactly 400.0. That 81.9 pt was the step, once per frame, for
as long as the dialog stayed open.

The message now draws first — status left, actions right, which is the Windows
arrangement anyway. Separately, the dialog host gained a **growth budget**: any
dialog that asks to grow more than three times stops and records why. Two
existing guards were satisfied throughout this bug and neither helped, because
a guard against *repetition* cannot see monotonic creep — creep never repeats.

**Verified:** four unit tests including one that reproduces the overflow in a
real laid-out frame and fails on the old ordering. **NOT driven** — reproducing
it end-to-end would mean sending a job to your printer.

## O33 — Does the ribbon get the scroll arrow, and do groups re-wrap?

**Asked:** 2026-08-25. **Partly shipped** 2026-08-25, commit `10877a1`.
**One decision open — yours.**

Two questions, and they have different answers.

**"Will it wrap tools in their sections onto second lines when the window is
resized?"** They already wrap onto a second row, but by the *group's own
content width*, not by the window — and **Word does not re-wrap on resize
either**. Its Font group keeps the same two rows at 1900 pt and at 1300 pt.
What Word does instead is collapse whole groups, and that is what shipped
today: at 1600 one group collapses, at 1100 four do.

**"Does this include replacing the ⏷ N more dropdown with an arrow that shifts
the ribbon horizontally?"** That is S4 and it is *not* built. It is still the
plan and Word does exactly it — a `›` at the band's right edge, which appears
at 460 pt and not before. But the collapse ladder changed the argument, and the
number is the reason this row exists:

| window width | `⏷ N more` before today | after |
|---:|---|---|
| 1600 | yes | **no** |
| 1200 | yes | **no** |
| 1100 | yes | **no** |
| 1050 | yes | yes |

**The dropdown used to appear at 1600 and every width below. It now appears
only below about 1100.** So S4 would replace an affordance you will rarely see
— and it would replace it with something *less* informative, because a menu
names what is hidden and an arrow makes you hunt for it. Against that: the
arrow is the convention, you have asked for it twice, and a band you can scroll
never hides a group's caption.

**What I need from you:** say the word and the arrow replaces the dropdown. It
touches six tests and one driven check, so I would rather do it in a session
where I can drive the running application to prove it, which needs the machine.

## O32 — The commands whose tab was decided by mode exposure, not by subject

**Asked:** 2026-08-25 —

> *"the current commands for each are fine as is for now. there were just some
> commands you made a decision to put in a different tab than where they would
> normally go because exposure was tab based and not command based."*

**Status:** **FOUND AND LISTED. The operator's decision, per command — nothing
moved.** The mechanism that forced them is gone; whether each *should* move is
a `RIBBON_IA.md` question and the IA is his.

### The mechanism, and exactly what changed about it

A mode names **tabs**: Read is `["file", "view"]`. So a command on a tab Read
does not show is not merely inconvenient there, it is **unreachable** — no tab,
no band, no control, and `modes::capability::offers_command` refuses its chord
too. Four commands were therefore homed on File or View instead of where their
subject says they belong, and the codebase names the pattern and calls it a
rule:

> *"a command refused in a mode where the operator plainly needs it is evidence
> that the command's tab is wrong, not that the mode gate needs an exception."*
> — `RIBBON_IA.md` §5.7

★★★ **What changed, precisely:** `visible_when` (O31) hides an **item**. It
cannot make a **tab** appear. So on its own it does not undo any of these. What
undoes them is the pair — `visible_when` plus *a tab with nothing left to show
is not shown* — which together turn `Mode::tabs` from *"which tabs exist here"*
into *"which tabs may appear here"*. A mode can now be given a tab generously
and shown only the part of it that applies.

So the move is buildable now. It was not before, and the cost is no longer
"Read gains batch merge, split and font embedding" — those would simply be
hidden.

### The four, and what each would cost to move back

| command | where its subject says | why it is where it is | moving it back would |
|---|---|---|---|
| **`file.ocr`** | Tools ▸ Recognise (`RIBBON_IA.md` §5.7) | operator: *"if in read mode ocr should still be available"* | give **Read a Tools tab** showing one command |
| **`file.copy_page_text`**, **`file.copy_document_text`** | Edit ▸ Clipboard (§5.1, §5.4, §7) | `Ctrl+Shift+C` was refused in Read, *"a mode whose whole standard is Acrobat Reader, which copies text"* | give **Read an Edit tab** showing two commands |
| **`view.panel_forms`** (was `edit.form_fill`) | Edit ▸ Forms | operator: Read fills forms, because Acrobat Reader does | needs the command **re-invented**: it stopped being a verb and became a *panel toggle*, and panels live on View |
| **`view.tool_text`** | — | placed on View ▸ Navigate **pre-emptively**, to avoid being the fourth instance | nothing: it is a pointer tool beside select, node and hand, which is a coherent group on its own terms |

### ★ The recommendation, per command — and it is not "move them all"

Because on inspection **each landed somewhere defensible**, and two of them are
now better placed than the specification's original:

* **`file.ocr` — leave, and amend the spec.** File ▸ Recognise sits beside the
  verbs that make a document exist and write one out, and OCR's product is a
  new file. Tools answers *"what do I run **across files**, or configure
  once?"* — and OCR runs on **this** file. The spec's Tools placement is the
  weaker of the two, and it was written before the tab questions were.
* **`file.copy_*` — leave.** *Copying is not authoring*: it reads the page and
  writes to the clipboard, and cannot change a byte. File ▸ Export groups it
  with the other verbs whose destination is outside the document. ★★ And the
  alternative is worse than it sounds: a mode called **Read** showing a tab
  called **Edit** contradicts the stance the mode exists to state, even if the
  tab holds nothing but Copy.
* **`view.panel_forms` — leave.** Not a tab move to undo. It is a panel
  toggle, and every panel toggle is on View ▸ Panels.
* **`view.tool_text` — leave.** A pointer tool, in the group of pointer tools.
  `edit.text` (change text that is already there) is a different verb and is
  correctly on Edit.

### ★★★ What is genuinely wrong and should be fixed either way

**`RIBBON_IA.md` is internally inconsistent.** It records the OCR move in §5.7
and names the rule — and §5.1, §5.4 and §7 were never updated for the two
earlier ones. The spec still says today:

> §5.1: *"**Moved off this tab:** `Copy this page's text` and `Copy the whole
> document's text` go to **Edit ▸ Clipboard**"*
> §5.4: `| | Copy page text · Copy document text | **G** *(from File)* |`
> §5.4: `| **Forms** | Fill form | **G** |`
> §7: `| Edit ▸ Forms ▸ Fill Form | Edit ▸ Forms |`

Three sections route commands to a tab they have not been on since 2026-08-14.
Whatever is decided about moving them, the specification should say where they
**are** — a settled document that disagrees with the build is worse than an
unsettled one, because it is read as authority.

### One inverse case, for completeness

`markup.underline`, `markup.strikeout`, `markup.squiggly` are the **opposite**
problem and are already recorded as an accepted inversion: they are on their
natural tab and were permanently greyed in Edit, *"not fixable by hiding them,
because the Markup tab is in both Review and Edit and a command has one tab"*.
★ That is now fixable — `visible_when` is exactly the missing mechanism — but
the tension closed on its own when `CanvasTool::Text` landed, so there is
nothing left to fix.

## O31 — Improve the ribbon: learn from Word

**Asked:** 2026-08-24 —

> *"can you improve the ribbon bar? if you can learn how word handles when to
> have text labels, organization on two rows for some commands, and how it
> handles narrowing the window. for one thing it puts an arrow at the end to
> press to move over if there isn't room for all commands. also we should have
> flexibility to show or hide and commands and shift the space used depending
> on what exists. this would allow greater flexibility of where to place
> commands for read, review and edit modes, as what remains shown can be mixed
> on tabs. if you can, drive word as it is installed on this machine."*

**Status:** **RESEARCHED AND STAGED. S1 and S2 done 2026-08-24; S3 and S4
designed and not built.** The whole of it is `RIBBON_SCALING.md`.

### Word was driven, and it had to be photographed rather than asked

Word's ribbon scaling rules are **not in its object model** — `CommandBars` is
the 2003 toolbar surface and says nothing about the ribbon, which is RibbonX
compiled into the product with its behaviour inside the Office UI framework.
So `tools/word-ribbon-study.ps1` sets a window width, waits for the re-layout
and captures: twelve widths, 1,884 down to 444, largest first, because Word
re-lays-out incrementally and a growing series would photograph the *recovery*
path. `tools/our-ribbon-study.ps1` is its twin, pointed at our own build.

### ★★★ The measurement that decided the work

Groups reachable on the band without opening a menu:

| client width | Word | pdfcer, before |
|---:|---:|---:|
| 884 | **10** | **3** |
| 604 | **7** + a scroll chevron | **1** |

**Our overflow was not the problem.** The `⏷ N more` affordance is the arrow he
is describing, it works, and it is tested at every width. It was starting far
too early, because every control in the band was icon-plus-label and a group
had no way to give up space except to vanish.

### What landed

* **Three item sizes** — Large (icon above label, spans the rows), Medium
  (today's), Small (icon only). Declared per item in the manifest, defaulting
  to Medium, so a manifest that says nothing renders identically.
* **Small is earned**: it needs an icon, a tooltip *and* an installed painter,
  or it falls back to labelled. The tooltip is the icon's accessible name.
* **`visible_when` on an item** — hidden **before measurement**, so the space
  is reclaimed and the group re-flows; a group with nothing left is not drawn
  at all, separator included. That is his second paragraph, and it is what will
  let one tab definition serve Read, Review and Edit.
* Applied to pdfcer's manifest: icon-only for the four page displays, four
  pointer tools, five display toggles, two page rotations, cut/copy/paste, four
  text markups and seven markup shapes; Large for the six one-item groups.

★★ **The File tab is deliberately unchanged**, and that is the finding worth
keeping. Its commands are *named things* — "Export form data…", "Save a
copy…" — not iconic ones, and `band.rs`'s original argument was right about
exactly that case. Driving Word showed the argument is about **the command**,
not about the band.

### What is designed and not built

`RIBBON_SCALING.md` §5.2 and §6: **per-group collapse in an authored order** —
each group in turn becoming a single captioned button with its full layout one
click away, which is what Word actually does — and **scrolling** as the last
resort beneath it rather than the first. Both touch `plan_band`'s invariants
(`the_visible_groups_are_a_prefix_and_nothing_is_lost`), which is why they are
staged rather than rushed.

★ One open question for the operator, and it is his to answer, not this
project's: **which commands should differ between Read, Review and Edit?**
`visible_when` is built and tested and nothing uses it yet, because deciding
what appears where is `RIBBON_IA.md`'s territory and the IA is settled.

## O28 — A fit control must place the view, not only set the scale

**Asked:** 2026-08-24 —

> *"If I press the Fit width or fit page button the view should center to the
> width as well or center the page."*

**Status:** **FIXED 2026-08-24**, and driven —
`a_fit_command_puts_the_page_on_screen` pans thirty notches into the pasteboard
before pressing each button, asserts it got there, and then measures the page's
drawn rect against the canvas's. Measured after Fit page: page
`296,272 .. 764,633` in a canvas of `288,143 .. 772,762` — margins of 8 and 8
horizontally, 128.5 and 128.6 vertically. **Falsified**: with the placement
disabled the same run reports *"part of the page is outside the canvas; the
vertical margins are 261.5 and −4.4, so the page is not centred"*.

★ **This is a consequence of O23's pasteboard and it is the second one.** Before
the pasteboard, a page smaller than the viewport had nowhere to be except the
middle, so "fit" and "centred" were the same act and nobody had to decide which
one the button meant. The pasteboard added a whole viewport of slack on every
side — deliberately, so any corner of the page can be brought to any point of
the screen — and with it the state the operator is reporting: **the scale is
right and the page is not on screen.**

So `Action::Fit` sets the scale and must now also **place the view**:

| | |
|---|---|
| **Fit page** | centred on both axes. The page fits, so there is exactly one honest position for it |
| **Fit width** | centred horizontally; the vertical position is kept but clamped to the page's own range, so you do not lose your place in a long sheet and cannot be left looking at pasteboard |
| **Fit height** | the mirror: centred vertically, horizontal kept and clamped |

★ Keeping the other axis rather than resetting it to the top is deliberate.
"Fit width" on page 12 of a drawing set is a *scale* request; throwing the
operator back to the top of the sheet would be a navigation they did not ask
for. Clamping is what makes "kept" safe.

## O29 — Fit height, because Acrobat has it

**Asked:** 2026-08-24 — *"Adobe has fit height, so add that too."*

**Status:** **FIXED 2026-08-24**, and driven in the same check. Measured after
Fit height on the 1584 × 1224 sheet: page `288,151 .. 1068,754` in a canvas of
`288,143 .. 772,762` — the full height on screen, and the width overflowing by
296 points, which is the mode doing exactly what it is for.

A third mode beside Fit page and Fit width: recompute the zoom each frame so
the page's full **height** is visible. On a landscape CAD sheet in a portrait
window it is the useful one, and it is the mode this build has been missing
every time the operator wanted to read a title block down the right-hand edge.

Scope, taken as the whole expected behaviour rather than the sentence:

* the mode itself, recomputed on every window resize like its two siblings;
* the **status bar** control beside Fit width and Fit page;
* a **registered command** so it appears wherever the other two do — the ribbon
  included — because R8 makes registering the command the only way the shell is
  allowed to learn a capability exists;
* the **opening-fit preference**, so a document can be opened at fit-height the
  same way it can be opened at fit-width;
* the on-disk id, its round trip, and the exhaustive-variant tests that would
  otherwise pass while silently not covering it.

## O30 — In single-page view, choose what the wheel does

**Asked:** 2026-08-24 —

> *"when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when the
> scroll wheel is used."*

**Status:** **FIXED 2026-08-24**, and driven —
`the_wheel_turns_pages_when_the_operator_asks_it_to` makes five separate
claims, in order: the default is **silent** (a build that flipped
unconditionally could not pass), the toggle is on screen beside the page
buttons, the **very next** notch turns a page, rolling back returns to the page
before, and under a continuous display the control is **not drawn at all**.

★ Two defects were found by writing it, and neither was in the feature:
the check **mutates a persisted setting and did not normalise at the start**,
so its second run inherited its first run's toggle and accused the shipped
default; and its absence claim used `declared_since` with an event count where
that helper wants a line number, reporting a control as drawn when it was not.
Both are the standing lessons in a new costume. The application now publishes
`wheel=` on its status line so the check can read the state it is about to
change.

Two behaviours, chosen by a control **next to the page navigation buttons** in
the status bar:

| | |
|---|---|
| **Scroll the page** | today's behaviour: the wheel moves within the sheet and never leaves it |
| **Flip pages** | the wheel turns to the next or previous page |

★ **The control renders only where it means something** — R9. Under a
continuous display mode the wheel scrolls the whole document by definition and
there is no choice to offer, so nothing is drawn rather than a disabled stub.
Under Single and Facing it appears beside the page box, which is where the
operator is already looking when they are thinking about pages.

★ It is an operator setting and therefore persisted, like every other view
preference: a choice that resets on the next launch is a choice the operator
has to keep making.

## O26 — Zoom out throws the page off screen into a corner

**Asked:** 2026-08-24 —

> *"the zoom in function works flawlessly now. The panning works. Zoom out has
> a small bug where it sometimes seems to reposition the page so that it is off
> screen in the far bottom left corner. This happened when I zoomed back from
> around 2 million% but seems to happen at other junctions too."*

**Status:** **SEVEN CAUSES FOUND AND FIXED, 2026-08-24**, in two clusters:
O26a-d below, which relocate the page at ordinary zooms and were never about
zooming out in particular, and O26e-g, the missing hand-over out of the `f64`
position tier. Every one of them moves the page by a whole page or more.
Driven, with pixels for the first. A residual is filed separately as O27.

★★★ *"Seems to happen at other junctions too"* was the load-bearing half of the
report and it was right. The 2,000,000 % crossing was **one** of seven
independent faults with the same symptom, and it was the least often reached —
three of the other six are reachable at 30 %.

### O26a — one wheel notch at 30 % took the view from page 1 to page 8

**★ Found by pixels, in the first thirty seconds of driving**, and it is the
worst of the four. `Strip::page_at_view` takes a **strip-space** rect. It was
being handed `scroll_output.state.offset` — a **content-space** offset, which
since O23's pasteboard sits a whole viewport above and to the left of the
strip's origin.

**This is the second site of the omission O23 spent four attempts on.**
`geometry::scroll_to_strip` was added then, for `visible_rect`, and nobody
swept for the other callers.

Two failure modes, and the silent one had been shipping for longer:

* **No page at all.** The horizontal error is a whole viewport and the strip is
  only as wide as its widest page, so the displaced box usually misses the
  strip entirely, `page_at_view` returns `None`, and the branch never runs.
  **Scroll-driven current-page tracking — Phase 4.3, the whole reason the block
  exists — has been inert since the pasteboard landed.** Nothing said so; the
  page number simply stopped following the scroll.
* **The wrong page**, whenever the strip grows wide enough for the displaced
  box to clip its right-hand edge. That is a function of the zoom, so it
  arrives at one particular magnification and not the ones either side of it.
  **That is the operator's "other junctions".**

And a mis-reported page is not cosmetic, because `current_origin` — the frame
of reference every single-page solve in `canvas::zoom` and `find::reveal` is
handed — is *that page's* origin in the strip. Set it to page 7 and the next
anchored zoom converts its answer back through page 7's origin, so the view
moves by seven page pitches in one wheel notch.

Measured on `SW41177.pdf` at 30 %, one Ctrl+wheel notch: `page` 0 → 7,
`off` [484, 490] → [514, 2767], and the status bar read `8 / 36`. Screenshots
before and after are the evidence; no trace field says *"the wrong page"*.

### O26b — and then the wheel stopped zooming altogether

`if image_response.hovered()` gated Ctrl+wheel on the **acting page's** own
response. Three ordinary positions were therefore inert: the pointer over a
*different* visible page, the pointer in the gap between two, and the pointer
over O23's **pasteboard** — a whole viewport of it on every side, added
deliberately so any page corner can be brought to any point of the screen, and
therefore a position the operator is now *expected* to be in.

★★ It is also what turned O26a's catapult from a lurch into a **freeze**. Once
the tracker had thrown `page_index` seven pages down the strip, the acting page
was off screen, nothing under the pointer was it, and every subsequent
Ctrl+wheel did nothing at all — five further notches produced a byte-identical
trace. A view that jumps is a bug; a view that jumps and then will not zoom
back is what gets reported.

The gate is now the scroll area's own content response, which covers pages,
gaps and pasteboard, and which — being a real `Response` — still lets a
floating window over the canvas swallow the wheel. A `rect.contains` test would
not have.

### O26c — the acting page's rect and the acting page's extent were different pages

`acting` was `doc.view.page_index`, decided *before* the fallback that picks
`drawn.first()` when the current page is not among the drawn ones — and then
never revisited. The next two lines paired **that page's rect** with **the
current page's extent**.

On a document whose sheets are all one size the mismatch is invisible.
`SW41177.pdf` mixes 1584 × 1224 sheets with 1224 × 792 ones, and the trace
caught it exactly:

```text
canvas rect=[[-5634238.0 681671.0] - [5515170.0 7895993.0]] zoom=9108.99
canvas-pos … ext=1584.000,1224.000
```

11,149,400 × 7,214,300 is 1224 × 792 at that zoom while `ext` says 1584 × 1224.
`PageMapping` is built from both, so the pointer mapped to a page point that
was not where the pointer was — the same frame reported `page=(618.59, −74.79)`
for a pointer well inside the sheet — the anchor's `frac` came from that, and
the next solve asked for an offset far outside the range.

`acting` is now taken from the page that was actually chosen, so the rect and
the extent always describe the same sheet.

### O26d — the zoom anchor did not name its page

A page-local offset is measured from **one** page's top-left, and converting it
back into a strip offset means adding **that** page's origin. The canvas added
whichever page was current on the frame the anchor was *consumed* — and an
anchor is armed on frame N, while `show` runs and the wheel is seen, and solved
on frame N+1, once the zoom has landed. The current page tracks the scroll in
between.

When they disagree the answer is wrong by whole page pitches. At 900,000 % a
pitch is 1.1 × 10⁷ points, so the offset lands far outside the scrollable
range, `strip_offset` clamps it to zero — **and zero is the content's top-left
corner.** Driven, descending 970,851 % → 814,325 %: the page point under the
viewport centre went from 1164.82 to **−0.04**, the page's own top edge, and
stayed there for the rest of the descent.

`ZoomAnchor` and `CanvasFrame` now carry `page`, and the conversion uses it.
Under `PageDisplay::Single` there is one page at the strip's origin and this is
the identity it always was.

## O26e / O26f / O26g — the hand-over back out of the `f64` tier

**Status:** **FIXED 2026-08-24**, and driven. The operator's *"from around
2 million %"* is the same number as O24f's, and it is not a number he picked
either time: `SUB_PIXEL_CONTENT_EXTENT / page_height` is where the position
hands over between the `f32` scroll offset and the `f64` `DeepAnchor`.

### ★★★ O24f fixed the hand-over IN. There was never one OUT.

A hand-over is two functions. Seeding the anchor from the scroll offset on the
way in was written; converting it back on the way out was not. Coming down, the
anchor was discarded and the `f32` machinery resumed from the zero the deep
tier forces every frame.

**Measured before the fix**, descending through 1,185,799 %: the page point
under the viewport centre went from (791.93, 1152.34) to **(−0.02, −0.03)** —
the corner of the sheet, with twelve million pixels of drawing off screen.
1,152 pt of movement, or about eleven million screen pixels.

★★ **The suite could not see it because `zoom_keeps_place` climbs.** It climbs
to the ceiling, one notch at a time, with a tolerance fine enough to catch a
hundredth of a point, and then the run ends without ever rolling the wheel the
other way. Its own header calls the hand-over *"half of what this check is
for"*; that sentence was true of the **upward** crossing only. **A check that
travels in one direction tests one direction.**

Three pieces:

* **O26e — `CanvasFrame::offset` was a lie at the deep tier.** It was
  reconstructed from the scroll offset, which that tier **forces to zero**, so
  every deep frame recorded "the page is centred in the pasteboard". Nothing
  consumed the lie while the tier held; the first zoom that crossed back did,
  because `offset_before` is that field. It is now **measured from the drawn
  rect** — `geometry::offset_from_drawn`, `margin − (page_min − viewport_min)`
  — which is algebraically the same number below the threshold (asserted by a
  unit test over the same inputs) and the truth above it, because it never
  mentions the scroll offset at all.
* **O26f — the exit is solved in `f64`.** `offset_from_drawn` alone took the
  descent from 1,152 pt out to 0.005 pt out, but 0.005 pt at a million percent
  is fifty screen pixels: every term being subtracted has a magnitude near 10⁷,
  where an `f32`'s step is a whole pixel. `DeepAnchor::page_local_offset` forms
  `page × zoom` in `f64` and narrows once, on the frame that leaves the tier —
  and re-states the anchor about the pointer first, so the last notch out of
  deep zoom is not the one notch that fails to hold the cursor.
* **O26g — the strip is placed from the content's origin, not its centre.**
  `Rect::from_center_size(outer_rect.center(), display_size)` is the same
  rectangle and is a catastrophic cancellation: in a continuous mode the strip
  is `pages × page_height × zoom`, which on a 36-page set at a million percent
  is 4.6 × 10⁸ points, where an `f32`'s step is **32 points**. It formed
  `centre − strip/2`, two numbers near 2.3 × 10⁸ whose difference is about 619.
  `geometry::strip_origin_offset` evaluates the same quantity symbolically — a
  centring margin that is exactly zero once the strip exceeds the viewport,
  plus one viewport of pasteboard — so no large intermediate is formed. Proven
  equivalent by a unit test wherever the plain expression is still exact.

  ★ Honest note: **the measured jitter did not change with this one.** It is
  justified by the arithmetic and by the equivalence proof, not by an
  improvement anyone observed. See O27.

## O27 — The `f32` scroll tier jitters above about 100,000 %

**Found:** 2026-08-24, while driving O26. **Not reported by the operator.**

With all four O26a-d causes and all three O26e-g pieces fixed, an anchored zoom
notch still moves the view by **10–35 screen pixels** on the `scroll` tier
above roughly 130,000 %. On the `deep` tier the same measurement is **±0.05
px** across four readings — exact.

It is **bounded jitter, not drift**: sixteen consecutive readings at ~10⁶ %
oscillated within a band of 43 px and did not accumulate. The view shimmies; it
does not walk away.

★★ **Both zoom checks are RED on this, deliberately.** An earlier draft gave
them a "record instead of assert" hatch above a measured jitter zoom, with a
written argument for why that was a boundary on the subject rather than a
loosened tolerance. **On its very first driven run the hatch recorded a
movement of 1,161 pt — the whole page — and reported PASS**, hiding O26d on its
first outing. The hatch is gone. Two red checks that name a real residual beat
two green checks that swallowed a page.

★ Cause not established. The predicted `f32` accumulation in the anchor solve
is about ±2 px, so something an order of magnitude larger is in the chain and
has not been found; the candidates left are the acting page's own strip origin
(up to 4.5 × 10⁸ on this document, step 32) and `egui`'s own scroll-area
arithmetic at that content size. The structural remedy is probably to make
`viewer::deep_position_needed` test the **view's** magnitude rather than one
page's — the strip exceeds `f32`'s exact range earlier than the page does, by
exactly the page count — but that widens the deep tier considerably and this
canvas has three times been broken by a change that meant to affect only deep
zoom. Not attempted.

## O25 — Panning far, or zooming out, leaves the new area blank

**Asked:** 2026-08-23 — *"zoom is working amazing, and panning is fast, but if
I pan to far to one side when I am beyond 800% zoom it doesn't always render
the new exposed area, and the same thing happens usually when I zoom out."*

**Status:** **FIXED 2026-08-23.** Driven, and the check fails on a build with
the defect restored.

### ★★★ One missing comparison, and it explains both halves

Above the pixmap ceiling a raster covers the **visible region** rather than the
page, so two textures of the same page at the same scale can be pictures of
*different places*. `render::settle`'s staleness test asked two questions —
has a **discrete input** changed (page, annotations, layers), and has the
**scale** changed — and **the region was in the cache key without being in
either**.

So a pan that changed nothing but which part of the page is on screen was not
stale by any measure it applied, and **no render was ever requested**. The
picture he had kept being drawn correctly at its own region and simply slid
off, leaving the newly exposed area blank for as long as he cared to look at
it.

★ The zoom-out half is the same fault by a different route. A zoom *does*
change the scale, so a render is requested — but the request is built from
whatever region was current when it spawned, and by the time it lands the
gesture has moved on. Once the scale settles, nothing notices the region it
arrived with is the wrong one. **"Usually"** in his sentence is the tell: it
depends on whether the gesture outran the render.

### Where the new term went, and why not with the discrete ones

`stale_region` is grouped with the **scale**, on the same debounce. A region
changes under a continuous gesture, and a render started on every frame of a
drag would be cancelled by the next one — the worker is single-slot — so the
operator would pan for a second and receive nothing at the end of it.

★ It is already rate-limited in a way the scale is not:
`render::strategy::region_for` snaps to a half-viewport grid, so a region
changes at most once per half-screen of travel however smoothly the pointer
moves. The debounce is the second limiter, not the only one — which is why the
settle interval can stay tuned for zoom without making a pan feel slow.

### ★★ The check could not see it, and the reason is worth more than the fix

The first version of `panning_past_the_overscan_renders_the_new_area` watched
`region=` — the region the pixels on screen are a picture of. On the defective
build **that field never changes**: no render is requested, no new texture
arrives, so the field describing the texture stands still. The check read *"the
view did not move"* and reported **SKIP** against a binary with the defect
deliberately restored.

The trace now carries `want=` beside it — the region the shell wants next,
which moves the instant the view does. **The gap between the two is the
defect, and it takes two fields to measure a gap.** With `want=` the check
fails on the defective build, naming the cause, and passes three runs of three
on the fixed one.

### What was measured

| | |
|---|---|
| pan, 40 wheel notches at 4,155 % | wanted region moves; **2 renders complete**; canvas shows 45–46 distinct tones |
| then zoom out, 6 Ctrl+wheel notches | wanted region moves; **1 render completes**; canvas shows 46 distinct tones |
| the same, with `stale_region` removed | wanted region moves; **0 renders**; check FAILS naming `RenderKey::same_region` |

★ The zoom-out half is asserted separately rather than assumed fixed by the
pan case. They share a cause and they do not share a code path, and *"it is
probably the same bug"* is how the second half of a two-part report gets
shipped broken.

### Why nothing else caught it

`panning_at_deep_zoom_stays_where_it_was_put` asks whether the view **moves**
and whether the pixels are **placed** correctly — both were perfect throughout.
`the_page_still_renders_at_every_decade_of_zoom` photographs after a **zoom**,
which changes the scale and therefore does request a render. **Nothing in the
suite panned far enough to leave the overscan and then looked at the screen.**

---

## O24i / O24j — Screenshots at maximum zoom, and the two defects they found

**Asked:** 2026-08-22 — *"Can you confirm that rendering on screen is actually
happening at maximum zoom? zoom in on one of the michocondria structures and
post screenshots here to confirm. start with the full page first to confirm it
renders."*

**Status:** **CONFIRMED, and it was not confirming before he asked.**

### ★★★ Why nothing already in the suite could answer this

`zooming_does_not_throw_away_where_the_operator_panned` proves the view stays
where it is put to a trillion percent. `zooming_past_the_pixmap_ceiling_still_
renders` proves no raster is refused. **Neither looks at the screen**, and a
canvas can satisfy both while drawing blank paper: the arithmetic would be
perfect, the rasters would complete, and the operator would see nothing.

That is exactly what was happening.

### O24i — the region path narrowed to `f32`, and detail stopped at ~10⁷ %

`render::strategy::region_for` snaps the region's origin to a half-view grid:

```rust
snapped_x = (x0 / step_x).floor() * step_x
```

At a trillion percent `step_x` is about 2 × 10⁻⁸ pt while `x0` is an ordinary
page coordinate near 540. Their quotient is **2 × 10¹⁰** — past `f32`'s last
exactly representable integer of 2²⁴ ≈ 1.7 × 10⁷ — so `.floor()` was applied to
a number that had already lost its integer part.

**Measured before the fix:** from about 10⁷ % the region stopped shrinking and
floored at 2.4414 × 10⁻³ × 3.0213 × 10⁻³ pt, **fifty thousand times** the
4.8 × 10⁻⁸ × 6.2 × 10⁻⁸ the viewport was showing. Its raster was then painted
18,998,834 window points off the viewport. `drawn=1` was still traced and no
render failed, so every check passed; the operator saw a fraction of one texel
stretched across the window.

The whole path is `f64` now — `region_for`, `overscanned`, `page_region`,
`OVERSCAN`, and the canvas call site that used to cast `DeepAnchor::
visible_rect`'s `f64` result straight down to `f32`. That cast was the one
narrowing left in a path whose every other stage was already `f64`, and it was
narrowing the value the tier exists to compute.

★ The real ceiling of the fixed design, since it is worth knowing: the extent
is computed as `(x0 + w) − x0` at an absolute position near 540, where an
`f64` ULP is 1.1 × 10⁻¹³. At the maximum zoom `w` is 10⁻⁹ pt — about **8,800
ULPs**, or eighteen representable steps per screen pixel. Comfortable. The tier
below it ran out at 2²⁴; this one has room left.

### O24j — the status bar showed `4294967295%`

`ViewState::zoom_percent` returned a `u32` and `as u32` saturates, so past
about 42,949,672 % the readout showed **u32::MAX presented as a measurement**.
Seen in the screenshot gallery, which is the only instrument that reads the
number an operator reads.

★ The type was right when `MAX_ZOOM` was 8.0 and every reachable value fitted
in three digits. O24 raised the ceiling to 10¹² and did not revisit it — the
recurring shape of this whole request: **a limit lifted in one place while a
narrower type downstream keeps enforcing the old one silently.**

★★ It now reports **999999995904%** at the top, not 1000000000000%, and that
is correct rather than a rounding failure: `ViewState::zoom` is an `f32`, so
that is the nearest representable value and it is what the view actually is.
Pinned exactly, so a future change that starts rounding the display instead of
reporting it has to be deliberate.

### The gallery

`the_page_still_renders_at_every_decade_of_zoom` — new. Opens the document,
parks the pointer on a **document coordinate**, climbs by Ctrl+wheel and
photographs the window at each tier of the fixture's own scale chain. At every
step it asserts three things, because any two can hold while the third fails:

| assertion | rules out |
|---|---|
| the **canvas region** is not near-uniform | a blank page |
| the canvas traced `drawn ≥ 1` | space reserved with no raster in it |
| no `outcome=failed` render | a refused rasterization the shell swallowed |

★★ *The canvas region*, not the window. The first version asked
`capture::window_to_png` — which refuses a near-uniform **window**, and a
window always contains a ribbon and two panels, so it can never fire for the
reason this check cares about. It passed a screenshot of blank white paper on
that technicality. Third instance this session of an assertion aimed at the
wrong surface.

★ It also **re-aims between tiers**, from the `f64` position line. Zoom-to-
cursor holds the point to about half a per-notch tolerance, which is excellent
per notch and still accumulates over the ~120 notches to the ceiling — so
without it the run wanders off a 3 µm mitochondrion and photographs cytoplasm.
`CanvasMapping` cannot do the re-aiming: it converts through the `f32` `rect=`,
whose spacing at the ceiling is half a million points.

### What the screenshots show

| zoom | on screen | distinct tones in the canvas |
|---|---|---|
| 114 % | the whole banana | 318 |
| 2,785 % | the two cells | 293 |
| 13,794 % | cell labels, organelles | 487 |
| 45,799 % | labelled organelles, the easter egg | 707 |
| 504,845 % | one mitochondrion in cytoplasm, cell wall behind | 144 |
| 3,730,330 % | mitochondria with cristae | 238 |
| 41,120,084 % | mtDNA nucleoid, mitoribosomes, ATP synthase heads along a crista | 222 |
| 999,999,995,904 % | mitochondrial matrix — a 0.02 nm field, smaller than an atom | 15 |

★ The last row is a solid fill and that is **correct**: at the ceiling the
viewport spans 6 × 10⁻⁸ pt, and the fixture has nothing smaller than the 10 nm
ATP synthase heads. Rendering is still happening; there is simply nothing left
to draw.

### And an easter egg

`gen_banana.py` gained `easter_egg.py` today. Inside the pulp cell, readable
from about 100,000 %: **KEN ♡ EMILY — HAPPY 7TH ANNIVERSARY 2026.**

---

## O24h — "Can you test up to maximum zoom please?"

**Asked:** 2026-08-22 — *"can you test up to maximum zoom please? If you find
issues that probably can't be resolved it is ok at that point to say good
enough. that level of zoom is unheard of in any pdf software commonly available
and the performance is amazing."*

**Status:** **DONE 2026-08-22. Nothing had to be called good enough.**

### The result

Both driven zoom checks now climb until the application **saturates**, rather
than to a depth chosen in advance. Measured on `banana.pdf`, whose two cells
are drawn at life size and are the only thing on the sheet worth magnifying:

| | |
|---|---|
| ceiling reached | **1,000,000,000,000 %** — the configured maximum, exactly |
| stages to get there | 16, of 8 Ctrl+wheel notches each (128 notches) |
| notches that advanced | 117 of 128; the rest are the tail after saturation |
| tiers crossed | `scroll` → `deep` |
| worst per-notch drift of the point under the cursor | **54 % of tolerance** |
| panning at the ceiling | +960 px asked, +960 px moved, held for 90 frames |
| renders refused | none |

★ The saturation test asks the **application** where its ceiling is rather than
comparing against a constant. The maximum is an operator setting, so a check
that hard-coded 10¹² % would silently stop testing the ceiling the day he
changed it — the same silently-inert control this whole request began with.

### ★★★ Two harness faults the climb exposed, both of the same kind

Neither was an application defect, and both would have been reported as one.

**1. The instrument ran out before the application did.** `held()` derived the
page point from the `canvas` line's `rect=` and `zoom=`. At 41,000,000 % a
Letter page's rect holds a magnitude near 2.5 × 10⁸, where an `f32`'s spacing
is 32 — so the reading resolved to about 8 × 10⁻⁵ pt while the tolerance at
that zoom was 3 × 10⁻⁵. The check failed with *"moved 0.0000 pt, where 0.0000
is the tolerance"* against a build holding the point perfectly.

The tempting fix is to widen the tolerance, which would have hidden a real
defect at every zoom below that. The `canvas-pos` line already carries the same
quantity in `f64` — added for O24b for exactly this reason — so the fix was to
**read the instrument that can still see**. `RESOLUTION_FLOOR` now stops the
proportional tolerance from ever dropping below what any instrument here can
resolve: a floor where the proportional tolerance would be smaller, not a
widening where it is meaningful. Those are different changes and only one of
them is honest.

**2. A guard phrased as "every notch advances".** The ceiling is reached
partway through a stage, so the tail of that stage and the whole of the next
legitimately stand still. The guard exists to catch a wheel that is *panning*
instead of zooming — which advances on **zero** notches — so it is three
quarters now, with room to spare.

### And one more, in a check that was not part of this

`measure_hover_shows_what_it_will_take` failed on this fixture, having first
printed *"legitimate"* about the very condition that made it fail: the sweep
landed on the banana's outline, a curve has no endpoint to snap to, and the
assertion below can only be met by a straight run. It now SKIPs with the
finding named. **A check that fails on correct behaviour is worse than an
absent one, because its red gets quoted.**

### Full suite

`ui-verify` on `banana.pdf`: **36 verified, 0 failed, 36 skipped** — the skips
are checks needing a `--doc-point` or a fixture this sheet cannot provide.

---

## O24e / O24f / O24g — Zoom throws the view away, twice, and `−` undoes a hundredfold

**Asked:** 2026-08-22, one message, three separate faults:

> *"there is a little bug where if I am zoomed out to about page size, pan the
> cells to the center of the screen, then start to zoom, the page snaps back to
> near the center position. … I do lose the view at 2000000% magnification.
> Also clicking the negative button to zoom back snaps me back to 800% when I
> am over 800%."*

**Status:** **ALL THREE FIXED 2026-08-22.** Driven, and the driven check fails
on a build with the defects present.

### O24g — the `−` button was not the inverse of `+`

`ladder_step_up` grew a doubling branch when O24 raised the ceiling.
`ladder_step_down` did not, so a plain reverse search found the highest named
rung below the current zoom — **8.00** — from anywhere above it. One press
discarded a hundred-fold magnification.

★ The asymmetry is the defect, more than the snap. `viewer`'s own header
promises *"zoom-in/zoom-out exactly reversible"*, and two controls that
disagree about what a step is break the one property an operator relies on to
explore without losing their place. Pinned as a **round trip** — up then down
returns to where it started — rather than against fixed numbers, which would
keep passing if both were changed together in a way that broke it.

The ladder is now its own module (`viewer::ladder`), which R2 forced when
`viewer/mod.rs` reached 1,540 lines and which is a real seam: everything in it
answers *what is the next zoom?* and none of it knows what a page is.

### O24e — a stale clamp, in the wrong space, against the wrong extent

`geometry::zoom_anchor_offset` clamped its answer to `display − viewport`: the
range a page has when the scroll content is the page and **nothing else**. The
pasteboard (O23) made that false — `content_extent` now adds a viewport of
slack on every side, so the real range is larger and the page is only part of
it.

★★ The damage was worst exactly where he found it. At a fit-page zoom the page
is no **larger** than the viewport, so `display − viewport` is zero or
negative, the clamp range collapsed to `[0, 0]`, and every zoom forced the
offset to zero — which after the strip conversion is the centred position. Not
*"near"* the centre by accident: it **is** the centre.

The clamp has moved to `strip_offset`, the one place that knows the real
range and the only value actually handed to the `ScrollArea`. That is the
division of labour the module already stated and had stopped observing.

★ Two tests had to change with it, and both were pinning the pre-pasteboard
constraint — including one asserting that framing a region at the page's
corner *cannot* be centred, *"there is no page to the left of or above the
origin to scroll to"*. O23 was the request to make exactly that possible. **A
test that pins last year's constraint is how a stale clamp survives the
feature designed to remove it.**

### O24f — the deep tier's hand-over, in three parts

2,000,000 % is not a number he picked. The threshold is
`SUB_PIXEL_CONTENT_EXTENT / page_height` = 16,777,216 / 792 ≈ **2,118,000 %**
on a Letter sheet. Three faults met there:

1. **The seed read the previous frame's scroll offset.** Dividing it by the
   *new* zoom asks where a point is using one frame's distance and the next
   frame's scale. The zoom anchor is now consumed at this tier too, and the
   seed uses the offset it solved for.
2. **Nothing called `DeepAnchor::zoomed_about`.** The module exists for that
   one operation and it had no callers: the anchored page point stayed nailed
   to the viewport's top-left and everything the operator was looking at
   expanded off screen. It is now called on every zoom while deep, about the
   pointer — or the viewport centre when the pointer is elsewhere, matching
   what `+`, `−` and Ctrl+0 anchor on at every other zoom.
3. **★★★ The scroll area held its old offset for one frame.** The content is
   the viewport at this tier so zero is the only valid offset, and egui does
   clamp to it — one frame late. On the frame the tier flips, `outer_rect.min`
   is still displaced by the stale offset, so the anchor placed the strip
   relative to a displaced origin and the page landed at roughly **twice** the
   intended distance.

Measured at the hand-over, 2,047,244 % → 2,181,987 %: the position line said
the page origin belonged 6,676,376 px left of the viewport; it was drawn
12,940,650 px left. The difference is 6,264,274 — the stale scroll offset, to
four significant figures. The offset is now assigned rather than left to the
clamp, because the raster region is computed from the same placement: the
frame was not merely misplaced, it rendered a different part of the page.

### What was measured

`ui-verify --check zooming_does_not_throw_away_where_the_operator_panned`.
Pans off-centre, then Ctrl+wheels **one notch at a time** with the pointer on
the viewport centre, following the page point under that centre:

| stage | zoom | tier | worst per-notch drift |
|---|---|---|---|
| 0 | 76 % → 377 % | scroll | 43 % of tolerance |
| 1–5 | 377 % → 1,123,552 % | scroll | 52 % |
| 6 | 1,123,552 % → 5,564,985 % | **deep** | 52 % |

Before the fix it failed at notch 3 of stage 6 — the hand-over — with the
centred page point moving 556.5 pt against a tolerance of 0.0004.

★ Three properties of that check are deliberate and were each learned the hard
way today: it **pans first** (the centred position is what O24e snapped *to*,
so a check that skipped the pan would watch the view "stay" where the bug was
about to put it); it reads **per notch** (once per eight-notch stage compared
accumulated rounding against a one-notch budget and failed a correct build);
and it **refuses to pass** a run that never crossed the tier boundary.

---

## O24c / O24d — The page lurches backwards mid-pan, and bounces when zoomed

**Asked:** 2026-08-22, two reports minutes apart, and they are **one defect**.

> *"As I drag using the middle mouse button the pan will follow and work, but
> if I pan a little too far it jumps back in the opposite direction I was
> moving the mouse towards … It isn't exactly in the same place as it started.
> When I zoom in the image does seem to disappear from the screen sometimes …
> if I pan the other direction and cross the same area where I experienced the
> jump the pan location jumps back to being correct."*

> *"Up to 800% things work perfect. Over that … it seems to refresh the image
> zoom, then reposition to the cursor location, which causes the image to
> bounce around a bit before settling under the cursor."*

**Status:** **FIXED 2026-08-22.** Unit-tested. ★ Driven confirmation still
owed — the operator was at the machine and `ui-verify` cannot take the
foreground from him.

### ★★★ The cause, and the second report is what proves it

The current page's texture is served from its slot **without a staleness
check**, deliberately: that is what shows the last good picture during a pan
instead of blank paper, and it is his own requirement — *"I don't want the
affect that other readers have where you always have to wait for detail to
render after panning to a new area."*

But the destination rectangle was computed from `OpenDoc::region_for` — the
region the shell wants **next** — while the pixels were still the previous
region's. `render::strategy::region_for` quantises the wanted region to a
half-viewport grid, so the instant a pan crossed a grid line the destination
jumped a whole grid step while the picture did not change. Every detail
follows:

| his words | the mechanism |
|---|---|
| *"follows for a bit, then jumps back"* | smooth within a grid cell, one step at the boundary |
| *"the opposite direction I was moving"* | the grid steps against the pan |
| *"isn't exactly in the same place as it started"* | the step is the grid, not the drag |
| *"cross the same area and it jumps back to being correct"* | pure function of position — re-enter the cell, get the cell's rect |
| *"the image does seem to disappear"* | two steps at once, at a zoom where the grid is most of the window |

### ★★★ RETRACTED: "up to 800 % things work perfect" is not evidence

The first version of this entry claimed that row as the confirmation, on the
reasoning that 800 % is where the whole-page raster gives out and the region
tier takes over. **That is false and the trace says so.**

The region tier engages at `MAX_PIXMAP_EDGE / page_height` — 16,383 / 792 =
**about 2,070 %** on a Letter sheet. Below it the raster is whole-page and
`region=none`. 800 % is nothing but the **old maximum zoom**, and the plain
reading of his sentence is *"the range that existed before is fine; the new
range is not"* — a statement about what he had already tested, not about a
mechanism.

★★ This matters beyond the correction. The driven check was tuned to land at
1,867 %, just under the real threshold, so every run traced `region=none`, the
placement cross-check had nothing to compare, and **the check reported PASS
twice against a binary with the defect deliberately put back in**. The wrong
reading of the 800 % sentence is what made that look like agreement instead of
like a check that could not fail.

A sentence was promoted to a measurement because it agreed with a theory. The
theory happened to be right; the evidence for it was not evidence.

### The actual proof

With the zoom raised to land at **4,155 %** — inside the band where the region
tier is engaged and the position is still on the `scroll` tier, which is the
only place this defect can exist — the check:

* **FAILS** on a build with the placement reverted to the wanted region, by
  **309.5 points** at mid-roll 1, and
* **PASSES** three runs out of three on the fixed build.

`ui-verify` now refuses to report PASS on a run in which no reading described
a region raster (`REGION_TIER_REQUIRED`). A check that cannot fail is not
evidence, and this one was being quoted as evidence.

And the zoom bounce is the same thing seen on a different transient: a zoom
changes the wanted region wholesale, so the held texture was thrown to a
completely different rect for as long as the new raster took — *"bounces
around a bit before settling"*. His guess in the message (*"maybe what you are
doing now will fix that behaviour"*) was right.

### ★★ Why rejecting the stale texture would have been the wrong fix

It is the obvious fix and it is worse. Blanking the page on every grid
crossing is precisely the behaviour he ruled out by name. The fix is to draw
the stale pixels **where they belong**, so they slide with the page as the pan
continues and the new raster replaces them in place.

`RenderKey` already carried the region; nothing ever read it back. It does
now (`RenderKey::region`), the texture and its region travel together to the
paint site, and the placement asks the pixels rather than the request.

### What was added to make it checkable

`canvas-pos` gained `paint=`, `region=` and `ext=`. `ui-verify` recomputes
`region_on_screen` from those **independently** and compares — so a future
change back to the wanted region is caught rather than merely absent from the
tests. `RenderKey::region`'s round-trip is pinned bit-exact.

★ The check also now zooms with Ctrl+wheel **at the two cells** rather than
pressing `+`, on his correction: *"Right now you are just zooming into a blank
area on the canvas."* Zoom-to-cursor keeps them under the pointer, so one aim
covers every rung.

---

## O24b — "Can the huge intermediate be fixed? Is that why panning jumped back?"

**Asked:** 2026-08-22 — *"can the huge intermediate be fixed? is that the
challenge I was running into trying to pan over a little bit at high zoom, but
it would jump back to it's original location I panned from because I couldn't
pan to the next point?"* — with the clarification that he meant the release
**before** the deep-zoom one, not the build published on 2026-08-22.

**Status:** **ANSWERED AND MEASURED, 2026-08-22.**

### Both halves, answered

**The intermediate is fixed.** `render::region::region_on_screen_deep` computes
the drawn rect from the `f64` anchor, so the page's ~10¹²-pixel screen rect is
never formed. That is the change that took the ceiling from about two million
percent to a trillion.

**And yes, that is consistent with what he described.** On the previous
release the view's position lived entirely in an `f32` scroll offset over a
content space of `page × zoom` where one unit is one screen pixel. Past about
2²⁴ content points that `f32` can only address every second pixel, then every
fourth, and so on — so a small pan computes `last - delta`, rounds to the
nearest representable value, and lands back on `last`. The view does not
move. From the operator's seat that is indistinguishable from *"it jumped back
to where I panned from"*, and it is exactly *"I couldn't pan to the next
point"*.

★ Stated as consistent rather than as proven, deliberately: the build he saw
it on has been published over and cannot be driven any more. What **is**
measured is the current one.

### What was measured

`ui-verify --check panning_at_deep_zoom_stays_where_it_was_put`, on
`banana.pdf`, rolling the wheel three notches and reading the position again
ninety frames later:

| zoom | tier | before → after → settled |
|---|---|---|
| 102,400 % | `scroll` | 405304.625 → 405424.625 → 405424.625 |
| 999,999,995,904 % | `deep` | 1981027832031.25 → 1981027832151.25 → 1981027832151.25 |

**+120 px asked, +120 px moved, and it stayed** — the same 120 pixels at a
trillion percent as at a hundred thousand. In `f32` that second row is
impossible: the representable spacing near 2 × 10¹² is 262,144, so a 120-pixel
move could not be written down at all, let alone survive ninety frames.

### ★★ A harness defect found on the way, and worth more than the answer

The check's first version drag-panned with the **primary** button and reported
the view as stuck at 102,400 %. That was wrong. `canvas::input::pan_delta`
pans on the middle button always and on the primary button only under the hand
tool; the default tool is Select, so a primary drag correctly rubber-band
selected and correctly moved nothing. The harness had measured a gesture the
application never offered and blamed the application for not honouring it.

It is written down because it is the third instance this month of the same
shape — a measurement of the wrong surface, whose verdict line is
indistinguishable from a real defect. **Ask what the check sampled before
asking what is broken.**

### What this added to the shell

`canvas::trace::position` — a `canvas-pos at=… tier=…` line carrying the pan
position in `f64`. The existing `canvas` line's `rect=` and `off=` are both
`f32`, and at these depths their own representable spacing exceeds the pan, so
**neither can measure this**: a check reading either would report a stuck view
against a perfectly working build. `tier=` names which mechanism produced the
number, so a failure points at one file rather than two.

---

## O24 — A setting for the maximum zoom

**Asked:** 2026-08-21 — *"add a setting so the user can set the maximum zoom.
the pdfcer engine has been updated to handle at least 1,000,000,000,000%. I'm not
concerned about the practicality of offering such a high zoom. it is up to the
user to determine how much of a performance hit they want to take."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** ★ The engine claim is
**verified** — see below — and it changes what this row is. The setting is the
small half.

### The engine really does do it, and the commits say how

`D:\Dev\pdfcer`, both landed since this shell's current lock:

```
71f7055  Deep zoom now holds its viewport to a trillion percent, and the fix
         is one subtraction moved into f64
bd9844d  render-page --region: the flag that makes deep zoom a viewport
         question instead of a page-size one
```

That second title is the whole architecture of this row, stated by the engine
itself: **deep zoom is a viewport question, not a page-size one.**

### ★★★ Why raising a constant will not do it, and where it stops

`viewer::MAX_ZOOM` is `8.0`, and raising it moves the ceiling only until a
harder one binds. `viewer::max_zoom_for_page` computes:

```rust
let ceiling = (pdfcer_render::MAX_PIXMAP_EDGE - 1) as f32 / (longest * ppp);
ceiling.clamp(MIN_ZOOM, MAX_ZOOM)
```

`MAX_PIXMAP_EDGE` is **16,384** and is an engine constant. For an A1 sheet
(~1,584 pt on its long edge) at 1 device pixel per point that ceiling is
**≈ 1,034 %**. So today `MAX_ZOOM = 800 %` binds first and the raster ceiling is
just behind it.

**A setting alone therefore buys about one more doubling and then stops
dead** — and it stops *silently*, because `max_zoom_for_page` clamps rather
than refusing, so the operator would set 100,000 % and watch the zoom stop at
roughly a thousand with nothing said.

★ That is the shape this project keeps finding: a control that is drawn,
accepted, persisted, and then quietly overruled downstream. Shipping the
setting without the mechanism behind it would be exactly that.

### ★★ THE RELEASE INSTRUCTION — 2026-08-21

> *"when you complete the step 2 zoom release to git and put on OneDrive."*

**On completion of step 2 — the `f64` viewport — and not before:**

| | |
|---|---|
| **1** | `git push origin main` |
| **2** | `python tools/package-portable.py`, which mirrors to the older `OneDrive\pdfcer-gui*` slot and leaves the other as the fallback |

★ **This is the first push of the project.** `origin` is
`github.com/KenM76/pdfcer-gui.git` and the local branch is **253 commits
ahead**; the last tag is `v0.3.0`. So the push is not a routine increment —
it publishes the whole of this shell's history at once, and it is worth
doing deliberately rather than as a step in a script.

★★ **Preconditions, because a release is the worst place to discover any of
them.** Every one has bitten this project already:

1. **Clean tree, 17/17 gates, all tests.** The gates include the four self-
   tests that prove the gates can still fail.
2. **The full driven suite**, on his own drawing, with **both** `--doc-point`s
   — `0,300,500` and `0,1211,1021`. One point passing is what hid `O22` for a
   day.
3. **`cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` first**, then
   rebuild and re-run. `O24` depends on two engine commits (`71f7055`,
   `bd9844d`) that this shell's lock predates, so a release built on a stale
   pin would ship without the thing it is a release of.
4. **`FEATURES.md` re-measured against the build**, because he reads it to
   know what he has, and it has carried a false claim before.
5. **Anything still failing is named in the release note**, not omitted. At
   the time of writing that is `multi_node_move_moves_every_picked_anchor`,
   which has never passed on any build and is an unbuilt path rather than a
   regression.

★ **Not before step 2.** He named the trigger precisely, and step 1 landing
on its own is not it — the point of the release is the higher zoom
capability, and shipping the tiering without the tier that needs it would be
a release of preparation.

### ★★★ THE CONSTRAINT THAT DECIDES THE DESIGN — 2026-08-21

> *"can you build the first step and build the second one and put it as an
> option to use instead for higher zoom capability? that way I can test out
> both in case there are performance issues introduced at lower zoom. I don't
> want to lose our capability to pan around a page and still see high detail
> as we pan. I don't want the affect that other readers have where you always
> have to wait for detail to render after panning to a new area."*

★★★ **That sentence rules out the obvious implementation, and it is right
to.** Region rendering applied everywhere would produce *exactly* the defect
he is describing — and it would be a regression, not a trade.

Here is why, stated plainly because it is the whole design:

| | today | naive region rendering |
|---|---|---|
| what is rasterized | the **whole page**, once per zoom | the **visible rectangle**, once per *position* |
| what a pan costs | nothing. The texture already exists; the view moves over it | **a new raster every time**, because the rectangle changed |
| what the operator sees while panning | full detail, immediately | blur, or blank, until the new raster lands |

So the thing he values — *"pan around a page and still see high detail as we
pan"* — is a **property of rasterizing the whole page**, and it is free
precisely because the raster does not depend on where you are looking.

#### The design that keeps it: tiers, each used only where the last cannot work

| tier | when | how | panning |
|---|---|---|---|
| **A — whole page** | while `page × zoom` fits `MAX_PIXMAP_EDGE` (16,384) | today's path, unchanged | **free, full detail** |
| **B — region + overscan** | above that, to ~1,000,000 % | rasterize the viewport **plus a margin**, so small pans are already covered | free within the margin; a re-raster only when you leave it |
| **C — f64 viewport** | above ~1,000,000 % | the visible page rect in `f64` becomes the position | as B |

★★ **Tier A is where he lives, and it does not change at all.** On an A1
sheet the whole-page raster works to about 1,034 %; today `MAX_ZOOM` stops it
at 800 % first. So every zoom he uses now, and one more doubling beyond it,
keeps exactly the panning behaviour he has — **by construction, not by
tuning.** There is no low-zoom performance question to test, because at low
zoom nothing is different.

★ And tier B only ever engages **where today the zoom is simply unavailable**.
Nothing is taken away to pay for it. The worst case is that deep zoom pans
less smoothly than shallow zoom — which is true of every reader, and is the
cost he explicitly said is his to accept.

#### The overscan is the part to get right

Rasterizing exactly the viewport means every pixel of pan crosses the edge.
Rasterizing the viewport **plus half a viewport on each side** costs 4× the
pixels and makes any pan up to half a screen free. That is the dial, and it
should be a **named constant with its cost written next to it**, not a magic
number:

```text
overscan 0.0  →  1.0x pixels, every pan re-rasters
overscan 0.5  →  4.0x pixels, pans up to half a screen are free
overscan 1.0  →  9.0x pixels, pans up to a full screen are free
```

At tier B the viewport is a few hundred thousand pixels, so 4× is cheap in
absolute terms — the whole point of tier B is that the raster no longer
scales with the zoom.

#### What the option actually is

He asked for the second step *"as an option to use instead"* so he can
compare. Given the tiering, the honest control is **the threshold, not a
mode**: the setting says how far the whole-page path is allowed to go before
the region path takes over. Set it low and he is testing tier B at ordinary
zooms; set it high and he never leaves tier A.

★ That gives him exactly the comparison he asked for, **and** it is the same
control as the maximum-zoom setting rather than a second one — which is
better than a checkbox, because a checkbox would have to be explained and a
threshold explains itself.

### ★★★ HOW IT GETS THERE — asked 2026-08-21

> *"how do we get to the insanely high limit? … I've seen readers hit over
> 4000%, and none are limited to a mere 1000%. You should be able to have a
> new algorithm take over for bigger zooms?"*

**Yes, and it is two changes rather than one — with two different ceilings
behind them.** The first gets from ~1,000 % to roughly a million percent and
needs no new position model at all. Only the second needs the *"new
algorithm"*, and it is not about how pixels are made.

#### Step 1 — render the WINDOW, not the page. Ceiling: ~1,000,000 %

Today the shell rasterizes the whole page and lets the scroll area show part
of it, so the pixmap grows with the zoom and hits `MAX_PIXMAP_EDGE` at about
1,034 % on an A1 sheet. **Every reader that reaches 4,000 % does the other
thing:** it rasterizes only the visible rectangle, so the pixmap is always
about window-sized and the zoom does not enter its size at all.

★ The engine has already done its half, and its own measurement is the proof
— commit `71f7055`, a requested 800×600 viewport:

```
zoom factor      zoom %        raster before    raster after
          1         100          800x600          800x600
    100,000  10,000,000          800x592          800x600
  1,000,000 100,000,000          800x640          800x600
```

*"the fix is one subtraction moved into `f64`"* — at deep zoom the region's
device origin is a few billion while the region itself is 800 points, so the
difference vanishes in `f32`. The large magnitudes now exist only inside
`f64` and are subtracted out before anything is handed back.

So on the engine side this is **done and measured to 100,000,000 %**. The
shell has simply never called `render_page_region`.

#### Step 2 — stop letting the scroll area own the position. Ceiling: none

This is the *"new algorithm"*, and it is about **where the viewport's
position is stored**, not about rendering.

Today the position is an `egui::ScrollArea` offset into a content rectangle
of `page × zoom`, and those offsets are `f32`. `f32` carries 24 bits of
mantissa, so it can address about 16.7 million distinct units before the
spacing between representable values exceeds one:

| content size | smallest addressable step |
|---|---|
| 16,700,000 pt | 1 pt |
| 1,600,000,000 pt | ~128 pt |
| 16,000,000,000,000 pt | ~1,000,000 pt |

An A1 sheet is ~1,584 pt on its long edge, so the content reaches 16.7
million at a zoom of about **10,500× — roughly 1,050,000 %**. Past that the
scroll offset cannot express where you are: panning would jump in steps of
hundreds and then thousands of points, and the view would judder and then
stick.

★ **Computed, not estimated.** The three steps above were produced by taking
the actual `f32` successor of each value: `1.00`, `128.00` and `1,048,576.00`
points respectively. The threshold for a 1,584 pt page is `16,777,216 / 1584`
= **10,543×**, i.e. 1,054,300 %.

What is *not* yet measured is the behaviour — that panning really does judder
and stick there. Worth driving before it is relied on, but the arithmetic is
not in doubt.

**So above about a million percent the source of truth changes**: the visible
**page-space rectangle in `f64`** becomes the position, panning adds to that
rectangle, and the scroll area stops being the thing that remembers where you
are. That is exactly the shape the engine's `--region` commit describes —
*"a viewport question instead of a page-size one"* — carried one layer up.

#### What this means for the order of work

| | delivers | needs |
|---|---|---|
| **1** | 1,000 % → ~1,000,000 % | region rendering in the canvas. No new position model |
| **2** | ~1,000,000 % → whatever he types | the viewport rect in `f64` as the position |
| **3** | the setting | (1) at minimum, or it is a control the shell cannot honour |

★★ **Step 1 alone already beats every reader he has seen.** 4,000 % is
inside it by two and a half orders of magnitude, and it is the smaller and
far less invasive of the two changes — it touches the render worker and the
raster cache, not the canvas's coordinate model. **If only one thing is
built, build that one.**

★ And it is not speculative: `crates/pdfcer-gui/src/render/offpage.rs` already
drives `render_page_region` and asserts the pixmap matches the region asked
for. Those tests were written for `O23` and this is the same mechanism.

### What actually delivers it

**Render the viewport, not the page.** `pdfcer_render::render_page_region` takes
an arbitrary page-space rect, and at deep zoom the visible rect is a *tiny*
fraction of the page — so the pixmap stays small however large the zoom is.
That is what the engine's `--region` commit means.

★★ This shell **has never called `render_page_region`.** Established
2026-08-21 while answering `O23`: it appears twice in
`crates/pdfcer-gui/src/`, both times in prose explaining that a tiled path does
not exist. The render worker uses `render_page_with_view`, whole-page, every
time.

★ And it is already de-risked. `crates/pdfcer-gui/src/render/offpage.rs` drives
the region path with regions off, straddling and enclosing the page, and
asserts the pixmap matches the region asked for rather than its overlap with
the page. Those four tests were written for `O23` and they are the same
mechanism this row needs.

### So the row is two pieces, and they ship in this order

| | | |
|---|---|---|
| 1 | **Region rendering in the canvas** | the real work. The render worker asks for the visible rect at the current zoom instead of the whole page. Wants `display_list::record_page` + `replay_region` rather than N region renders, because a region render re-interprets the whole content stream and a moving view would pay that per frame |
| 2 | **The setting** | small, and honest only once (1) exists |

★ Doing (2) first is possible and is **not** recommended: it would ship a
control that accepts a number the shell cannot honour, which is the defect
class above.

### Two consequences to decide when it is built, not after

- **The zoom readout is 46 pt wide**, sized for four characters
  (`ZOOM_READOUT_WIDTH_PTS`, with a comment saying so) because
  `ZOOM_LADDER` tops out at `800%`. `1000000000000%` is fourteen. The readout
  needs a format — `1e12 %`, or `1.0 Tx` — decided rather than allowed to
  stretch the status bar.
- **`ZOOM_LADDER` is a fixed array** the `+`/`−` buttons step through, ending
  at `8.00`. Beyond it the ladder has to become generated — presumably
  multiplying by a constant factor per step — or the buttons stop working
  exactly where the setting starts mattering.

★ Neither is hard. Both are the kind of thing that gets discovered by an
operator rather than decided by an engineer if they are not written down first.

### And one thing that is genuinely free

Ken's *"it is up to the user to determine how much of a performance hit they
want to take"* removes the question this would otherwise turn on. The setting
does **not** need a guard, a warning, or a preflight. It needs to be honest
about what it does, and to actually do it.


## O23 — Free navigation: any part of the page to anywhere on screen, and objects off the page still reachable

**Asked:** 2026-08-21 — *"also objects should still be reachable even if they are
off the page. I should also be able to move the view of the corner of the page
to the center of the screen, or even all the way vertically to the opposite
corner if I want to."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** ★ This **answers `O22`'s open
convention question** — the pasteboard is what he wants — and then asks for more
than `O22` proposed. `O22`'s three candidate fixes are settled by this row:
candidate 3, sized as below.

### Two requirements, and they are not the same job

| | |
|---|---|
| **A — free scrolling** | any point of the page can be brought to any point of the screen |
| **B — off-page content is reachable** | an object whose geometry lies outside the `/MediaBox` can still be seen and selected |

A is a scroll-extent change. B is about what the canvas draws and hit-tests at
all. They are filed together because he asked for them together and because A is
a precondition for B — there is no point being able to select something you
cannot scroll to — but they will not be one change.

### A — how much pasteboard, derived from his own words rather than guessed

He gave two levels, and the second is the requirement because it subsumes the
first:

1. *"the corner of the page to the center of the screen"* → needs **half a
   viewport** of margin on each side.
2. *"even all the way vertically to the opposite corner"* → needs **a full
   viewport** of margin on each side. To bring the page's top-left corner to the
   screen's bottom-right, the content must extend one whole viewport past the
   page on the top and left.

★ So: **pad = one viewport extent on every side**, recomputed as the viewport
changes rather than fixed in points. A constant number of points would be too
small on a large monitor and absurd on a small one, and it would silently stop
satisfying his sentence the first time he resized the window.

That is also the standard approximation of an infinite canvas — it is what
Illustrator, Figma and every CAD package give you, and none of them makes the
operator think about it.

### ★★ The risk, named up front because it has bitten this project before

Everything in `canvas::geometry` that today treats the **strip's** size as the
**scroll content's** size becomes wrong, because those stop being the same
number. `strip_offset`, `page_local_offset` and `pan_offset` all take a
`display`/`strip` extent and use it both to compute the centring margin and to
clamp the scroll range.

The failure mode is not hypothetical and it is recorded in `canvas::mod`'s own
source: in the old GUI, a centring-margin error made selection outlines draw
**~105 px** from the object they outlined, and clicking directly on a visible
object missed it — worst at exactly the zoom an operator uses to see a whole
page, and invisible at high zoom where the margin is zero.

So the change is: introduce the **content extent** as a value distinct from the
**strip extent**, and audit every consumer. Not "add a pad to `outer`".

### B — off-page objects

Not yet investigated. What has to be established first, against source:

1. ~~Does the decomposition **include** objects outside the `/MediaBox`?~~
   **ANSWERED 2026-08-21 against `pdfcer-core` source: YES, with no culling of
   any kind.** The decomposer is never even *told* what the page box is —
   `decompose_page` has the `&Page` in hand and reads only its content
   stream, resources and fonts. Grepping `media_box|crop_box|page_box` across
   the whole `vector` module returns two hits, both in `clip.rs` and both
   about the synthetic clipboard PDF's own box. An object drawn at
   `(-5000, -5000)` is in `PageObjects::objects` with a truthful negative
   `page_bbox`.

   ★ Adjacent, and worth knowing before designing anything: **clipping is
   ignored in general, not just the page box.** The painting-operator
   dispatch has no `W`/`W*` arm at all, so a path used only as a clip is
   emitted as an ordinary object, and an object whose paint is entirely
   clipped away still arrives with its full unclipped geometry.
   `PaintStyle::is_invisible` exists so a caller can tell — nothing drops
   them. So *"everything the model contains"* is a larger set than
   *"everything the operator can see"*, by more than just the off-page
   objects.
2. ~~Does the hit test accept a canvas point **outside the page rect**?~~
   **ANSWERED 2026-08-21 — the hit test would, and it never gets the chance.**

   | | |
   |---|---|
   | the screen→canvas conversion | `canvas::mapping::to_page` is pure arithmetic with **no clamp** (`mapping.rs:189`), so a point past the page's edge maps to a canvas point past its extent, and would hit-test as ordinary geometry |
   | the pasteboard area | allocated `Sense::hover()` (`canvas/mod.rs:662`) — it senses the pointer and **cannot be clicked** |
   | each page | allocated `Sense::click_and_drag()` (`canvas/mod.rs:688`) |
   | where a press comes from | `response.clicked_by(..)` / `drag_started_by(..)` on **the current page's** response (`canvas/interact.rs:372-375`) |

   ★ So the gate is not the hit test and not the mapping — **it is the input
   surface.** A press only becomes a canvas gesture if it landed inside a
   page's rectangle. Content painted outside the `/MediaBox` sits over the
   area that senses hover and refuses clicks, so it can be pointed at and
   never pressed.

   ★★ That is worth knowing before any of part B is designed, because it
   means B is **not** a hit-test change. It is a change to what the canvas
   allocates as clickable — which is the same code the pasteboard touches,
   and is why A and B belong in one row even though they are two jobs.

   Hover, by contrast, is already unbounded: `interact` falls back to
   `ctx.pointer_latest_pos()` (`interact.rs:352`) and asks `over_canvas`
   against the scroll **viewport** rather than the page (`interact.rs:1310`).
3. ~~Is such an object **painted**?~~ **ANSWERED: no, and the engine already
   has the way to make it so — this shell has never called it.**

   The whole-page entry point sizes the pixmap to the **CropBox** and there
   is no explicit clip anywhere; the clipping is purely *implicit*, because
   geometry outside the pixmap is culled by the rasteriser. That is why the
   escape hatch works at all:

   **`pdfcer_render::render_page_region(doc, page, scale, region, options)`**
   takes an arbitrary page-space rect and **never clamps or intersects it
   with the crop box**. A region starting left of or below the page produces
   a negative origin and is translated into view. The only limits are
   finite-and-non-empty and `MAX_PIXMAP_EDGE` (16,384) applied to the region.

   ★ **Nothing in `RenderOptions` selects a box.** "Render a bigger area" is a
   different *function*, not a setting — so this is not a matter of passing a
   flag.

   ⚠ **Two caveats, both load-bearing:**

   - ~~**No test exercises a region outside the crop box.**~~ ★★ **CLOSED
     2026-08-21 — one now does, in this repository.**
     `crates/pdfcer-gui/src/render/offpage.rs` drives `render_page_region` with
     a region entirely off the page, one straddling its left edge, and one
     containing the whole page plus a margin. **All three rasterize, and the
     pixmap is sized to the region asked for rather than to its overlap with
     the page.** So the escape hatch is proven rather than merely
     unrejectable.

     ★ It lives here rather than in `pdfcer-render` for two reasons: that crate
     is read-only to this project, and a consumer asserting the contract it
     depends on is the right shape anyway — if an engine bump starts clamping
     the region, the failure lands on the shell that cared, naming the feature,
     instead of presenting as a blank canvas.
   - **A region render re-interprets the whole content stream.** N tiles cost
     N interpretations. For a view that moves, `display_list::record_page` +
     `DisplayList::replay_region` is the intended path and is documented as
     landing on byte-identical pixels.

4. **And the measurement that makes the whole thing tractable:**
   `PageObjects::page_bbox()` returns the union of every object's bounds —
   which, because of (1), **includes the off-page ones**. So
   `model.page_bbox().union(crop)` is a ready-made *"what must I be able to
   scroll to in order to reach everything"*, and it feeds straight into both
   the scrollable extent and `render_page_region`.

   Precision caveats it carries: text boxes are approximate (and say so, via
   `TextBoundsBasis`), stroke width is **not** included, and clip-only paths
   inflate the union because of the finding in (1).

### ★★★ The conclusion: NO ENGINE CHANGE IS REQUIRED

Off-page content is already fully present and fully selectable in the model.
The decomposer keeps it, `page_bbox` measures it, `hit_test_point_all` will
select it. **The only place it disappears is the raster**, and only because
the whole-page entry point sizes the pixmap to the crop box.

So both halves of this row are shell work:

| | |
|---|---|
| **A** | the scroll extent and the seeding — `O23`'s attempt above |
| **B** | make the canvas allocate the off-page area as clickable, and render it through `render_page_region` |

★ Verified against **this** shell rather than assumed: `render_page_region`
appears twice in `crates/pdfcer-gui/src/`, both times in **prose** explaining
that a tiled-progressive path does not exist. It has never been called. The
render worker uses `render_page_with_view`, i.e. whole-page-at-crop-box.

★★ **No feature request to the engine session is owed for this row.** That
was worth establishing rather than assuming: the reflex on hitting a wall
like *"the raster stops at the page edge"* is to file it as an engine gap,
and it is not one.

★ Rule 4 applies to the answer: if pdfcer can see content the operator cannot,
that owes an **off-canvas** report. It must not be marked on the page.

### ★★★ ATTEMPTED 2026-08-21 AND BACKED OUT. What it cost, and what it taught

The whole change was built, all 1,634 unit tests passed, all 17 gates passed
— **and it broke selection on the real application.** It was reverted the
same evening rather than left in the tree, because a build where clicking an
object does nothing is worse than one that cannot rotate near the top edge.

Nothing below is speculation. Every item was measured.

#### 1. There are TWO offset spaces and only one of them has a pasteboard

| space | origin | margin |
|---|---|---|
| the scroll offset egui is given | the content's top-left, pasteboard included | padded |
| the page-local offset the view stores | the page's own top-left | **plain** |

`strip_offset` and `page_local_offset` convert between them and must use
**one of each**, or the pad cancels and vanishes.

★ The trap: `anchor_screen_pos` and `offset_holding_anchor_at` look like
scroll-space functions and are **page-local** — `canvas::mod` converts before
building the `CanvasFrame`. Padding them doubles the pad, and the symptom is
*"zoom-to-cursor flies off"*, worst on a large window.

#### 2. ★★ The pasteboard must be measured against the OUTER viewport (R128)

The obvious `ui.available_size()` is measured **inside** the scroll area, so
it depends on whether the scrollbars are showing — and the pasteboard is what
makes them show. Feeding it back is a loop: content grows, scrollbars appear,
available shrinks, content changes.

Measured symptom when it happened: `ui-rect-gone name=canvas-viewport` — the
canvas region retired entirely and no page was drawn. That is R128 in a new
place, the same shape as the status bar that drifted 230 % → 224 % → 215 %.

#### 3. ⚠️ THE DIAGNOSIS THIS ROW GAVE FIRST WAS WRONG, AND IT WAS NOT MEASURED

This section said the page *"MOVES, one frame later, as the offset settles"*,
and gave numbers: the page's rect going from `y=143.0` to `y=269.7`.

**Those two numbers came from two different builds.** 143.0 was the shell
without a pasteboard; 269.7 was the shell with one. Comparing them and
calling the difference a per-frame transient is the same unsound inference
this file has now corrected three times in one day — and it was written
here, as a measurement, hours after the rule was recorded.

**Re-measured properly on 2026-08-21, within a single run**, by counting
distinct `canvas rect=` lines (the trace is a change log, so one line means
one stable value):

| build | distinct rects during startup |
|---|---|
| without the pasteboard | **two** — `y=139.0` then `y=143.0`, a 4 pt settle |
| with the pasteboard | **one** — stable from the first frame |

★ So the pasteboard does not merely fail to cause a jump; the layout it
produces is *steadier* than today's. The seeding works.

#### 3b. ★★★ WHAT ACTUALLY BREAKS, stated as what was observed

**The canvas stops receiving pointer input entirely.** Not a mis-aimed
click, not a coordinate error — no input at all.

| observation | value |
|---|---|
| `canvas-pointer` events in a driven run | **0**, at both `--doc-point`s tried |
| the page's published rect | `[[296.0 269.7] - [764.0 631.3]]` |
| the canvas viewport | `[[288.0 139.3] - [772.0 762.0]]` |
| where the page sits in it | **centred on both axes, wholly inside, fully visible** |
| rendering | unaffected — an offscreen run reaches `drawn=14` exactly as the baseline does |

So the geometry is right, the page is where it should be, it is drawn, and a
click computed from the application's own published rect lands inside it —
and the canvas never sees a pointer.

★★ **That is a much sharper clue than the one this row gave first**, and it
points somewhere entirely different: at input and widget allocation, not at
coordinates. The two candidates worth starting from, neither confirmed:

- The scroll content is allocated as one rect with `Sense::hover()`
  (`canvas/mod.rs:662`) and the pages are placed inside it with `ui.put` /
  `allocate_rect`. Before the pasteboard, that outer rect was exactly the
  strip; now it is larger than the strip on every side. Whether that changes
  which widget egui resolves a press against is the first thing to test.
- `visible_rect` is built from `doc.last_scroll_offset` — **the previous
  frame's** offset — and on the frames right after seeding that is still
  zero, which now names the far corner of the pasteboard rather than the top
  of the strip. Whether the pages allocated on those frames are the ones the
  pointer is over is the second thing to test.

★ Both are cheap to test and neither was tested, because the first
diagnosis was believed. **Test the input path before touching the
arithmetic again** — the arithmetic is not what is wrong.
#### 3c. ★★★ BISECTED 2026-08-21. One suspect cleared, the other located

Three driven runs, each changing **one** thing from the last, all at
`--doc-point 0,300,500` with `resize_scales_a_shape`:

| # | what was applied | result | `canvas-pointer` events |
|---|---|---|---|
| 1 | scroll content **+200 pt** each axis. No seeding, no arithmetic change | **PASS** | 19 |
| 2 | scroll content **+ a whole viewport** each axis. Still no seeding | SKIP | 9 |
| 3 | the full change: pasteboard **and** seeding | SKIP | **0** |

**Run 1 clears the allocation suspect outright.** Enlarging the
hover-sensing content rect so it is no longer exactly the strip does not
cost the canvas its pointer input — the check passes and the gesture
completes. Whatever is wrong is not that the pages stopped being the widget
egui resolves a press against.

**Run 2 explains itself and is not a defect.** With a full pasteboard and no
seeding, the scroll offset is still zero, which now names the far corner of
the pasteboard. Measured: the page's rect was
`[[780.0 761.7] - [1248.0 1123.3]]` against a viewport of
`[[288.0 143.3] - [772.0 762.0]]` — **the page is entirely outside the view**,
exactly one pasteboard away, which is precisely what seeding exists to fix.

★ It also turned up something a harness author needs to know: **the
application publishes `canvas rect=` for a page that is off-screen.** The
rect is the page's *allocated* rect, not its visible one. A check that maps a
document point through it will compute a screen point outside the window and
click on whatever is there — which is what run 2 did, landing at page
coordinates of `-2529`.

#### 3d. So the remaining mystery is narrow, and it is not geometry

In run 3 the seeding **works**: the page's rect is
`[[296.0 269.7] - [764.0 631.3]]` inside a viewport of
`[[288.0 139.3] - [772.0 762.0]]` — centred on both axes, wholly visible.
And the canvas receives **nothing**.

★★ The new clue, which run 2 makes visible by contrast: in run 3 the trace
carries **one** `canvas` line and `drawn=0` for the whole run, where run 2
climbs through `drawn=1 … 10`. **The application barely advances.** An
offscreen smoke launch with the same seeded build reaches `drawn=14`
normally, so it is not that seeding freezes the shell — it is something
about the seeded build *in a driven run*.

So the question to start from next time is **not** "where is the page" but:
*why does a seeded build stop advancing frames when the window is raised and
driven?* Candidates worth trying, cheapest first:

1. Call `.scroll_offset()` on **every** frame from the stored view rather
   than once behind a flag, and see whether input returns. If it does, the
   one-shot is interacting with `ScrollArea`'s own state rather than seeding
   it.
2. Check whether anything is requesting a repaint after the seed. `drawn=0`
   with pages visible means rasters were requested and never arrived, which
   is a repaint question, not a layout one.
3. Seed by writing `doc.view`'s stored offset **before** the scroll area is
   built, so the existing offset path carries it and no `.scroll_offset()`
   override is needed at all.

★ Candidate 3 is the one to try first on design grounds: it removes the
override entirely rather than tuning it, and the override is the only thing
run 3 has that run 2 does not.

#### 3e. ★★★ BISECTED FURTHER 2026-08-21. It is the SCROLL OFFSET ITSELF

Four more driven runs. Each changes one thing; all at
`--doc-point 0,300,500` with `resize_scales_a_shape`.

| # | seeding | offset that resulted | frames advance? | `canvas-pointer` | verdict |
|---|---|---|---|---|---|
| 4 | write egui's `scroll_area::State` before the area is built | **`[0,0]`** — did not apply | yes, `drawn=10` | 11 | page off-screen |
| 5 | `.scroll_offset(vec2(100, 100))` | `[100,100]` — applied | yes | 9 | page still off-screen |
| 6 | `.scroll_offset(vec2(484, 492))` — the magnitude the real seed produces | `[484,492]` — applied | eventually | **0** | **page ON-screen, input dead** |

**Run 4 kills the nominated fix.** Pre-writing `scroll_area::State` does not
take — egui reports `off=[0,0]` — almost certainly because it clamps a
restored offset against a content size it does not know on the first frame.
So *"seed the state instead of overriding"* is not available, and the
override is not avoidable that way.

**Run 5 clears the override mechanism.** `.scroll_offset(..)` with a small
value applies cleanly, the application advances, and the canvas keeps its
pointer input. Nothing about forcing the offset is inherently harmful.

**Run 6 is the whole defect, reproduced from a HARD-CODED CONSTANT.** No
pasteboard arithmetic is involved in choosing it — it is two literals. The
page's rect settles at `[[296.0 269.7] - [764.0 631.3]]`, one stable value,
wholly inside a viewport of `[[288.0 139.3] - [772.0 762.0]]`. A click
computed from that rect lands at roughly `(385, 484)`, comfortably inside
both. **The canvas receives nothing.**

★★ So the cause is neither the arithmetic, nor the allocation, nor the
seeding mechanism. **A large applied scroll offset costs the canvas its
pointer input**, while leaving layout, drawing and the published rects
entirely correct. That is a much smaller and much stranger problem than any
of the three this row has previously blamed.

#### 3f. ★★★ ANSWERED 2026-08-21: it is a SEQUENCING bug, not a shell defect

The experiment was run, as a permanent check —
`scrolling_far_keeps_the_canvas_its_pointer_input`:

```
[PASS] scrolling_far_keeps_the_canvas_its_pointer_input
       before scrolling: 1 pointer event(s)
       scrolled to an offset of 1600 pt
       after scrolling: 20 pointer event(s)
```

**1600 pt — more than three times the offset that killed input when it was
forced — reached with the wheel, and the canvas keeps its pointer.**

So today's shell is fine, the operator is not meeting this, and O23 was not
being blamed for somebody else's defect. Both good outcomes.

★★ **Which settles the diagnosis by elimination.** It is not the magnitude of
the offset, not the arithmetic, not the allocation, and not the override
mechanism. **It is forcing an offset on the frame the content is first laid
out**, before egui knows how big that content is.

That also explains run 4's failure to take: pre-writing `scroll_area::State`
was clamped away against an unknown content size. Same cause, other symptom.

#### 3g. The fix, now specific

**Seed one frame late.** Let the first frame lay the content out with egui's
own offset, and apply the seed on the second, when the content size is known
and the offset will neither be clamped nor arrive mid-layout.

The cost is one frame showing the unseeded view. At a full-viewport pasteboard
that frame shows blank paper, which is visible — so the seed wants to be
**silent**: either the canvas skips its first paint, or the pasteboard starts
at zero and grows on the second frame. The second is cheaper and has no
flicker, because a content size that grows under a correct offset moves
nothing on screen.

★ `canvas_offset_seeded` becomes a small counter rather than a flag, and the
row's earlier three candidates are all retired.
#### 4. What survived, and what it is worth

The pure arithmetic was written and proven before it was reverted, and it is
reconstructible in minutes from this row:

- `pasteboard(viewport) = viewport × 1.0` — the fraction comes from his two
  sentences: half a viewport reaches the screen's centre, a whole one reaches
  the opposite corner. **A fraction, never a constant number of points**, or
  it stops satisfying the requirement when the window is resized.
- `content_extent(display, viewport) = display.max(viewport) + 2 × pasteboard`
- `strip_margin = margin + pasteboard`, and `margin` stays as it is
- every scroll clamp moves from `display − viewport` to
  `content_extent − viewport`
- `strip_to_scroll(in_strip, strip, viewport)` for callers that already have a
  strip-space position — `strip::page_scroll_offset` is the one that exists

Five unit tests changed, each pinning the old unpadded model, and each
correctly. **That is the useful signal**: they are the inventory of what the
pasteboard changes.

### What it needs, in order

1. **A** first, because **B** is unreachable without it.
2. The `canvas::geometry` audit, with its unit tests extended to cover
   `content != strip` — that arithmetic is pure and is exactly what a unit test
   is good for.
3. A driven check per page **edge**, as a regression guard once the pasteboard
   lands. ⚠️ **Not** because the resize grips share the defect — they do not;
   see `O22`'s correction. Their centres sit ON the box edge, so their inner
   half is always inside the canvas and always grabbable. Only the rotate
   handle's centre is outside the box.
4. Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s. One
   point passing is what hid `O22` for a day.
5. **B**, as its own piece of work, starting with the three questions above
   answered against `pdfcer-core` source rather than assumed.


## O22 — An object near the top of the view cannot be rotated: its handle is off-canvas

**Found:** 2026-08-21, by driving `rotate_handle_turns_a_selection` at a second
`--doc-point`. **This is the cause of Ken's *"I also can't drag and rotate text
on the screen yet"*** (`O20`), and it is not about text.

**Status:** **CONFIRMED BY DRIVING, WITH NUMBERS. NOT FIXED.** The fix is a
convention question and is not being improvised.

### The evidence

```
--doc-point 0,300,500    rotate_handle_turns_a_selection   PASS
--doc-point 0,1211,1021  rotate_handle_turns_a_selection   FAIL
--doc-point 0,1211,1021  resize_scales_a_shape             PASS
```

Resize passes at the same point that rotate fails, so the object is selected,
the outline is drawn and the eight resize grips are reachable. Only the ninth is
not.

### The arithmetic, from the application's own trace

| | |
|---|---|
| the canvas viewport | `rect=[[296.0 143.0] - [764.0 504.6]]` |
| the selection outline | `rect=[[614.1 150.2] - [753.9 224.0]]` |
| `ROTATE_STEM_PX` | `20.0` (`canvas/handles.rs:335`) |
| `GRIP_SIZE_PX` | `8.0` (`canvas/handles.rs:100`) |

`Grip::Rotate.anchor()` is `(mid.x, bounds.top() - ROTATE_STEM_PX)` —
`handles.rs:271` — so the handle's centre is at **y = 150.2 − 20 = 130.2**, and
its square spans **126.2 → 134.2**.

**The canvas begins at y = 143.0.** The whole handle is 9 pixels above the top
of the canvas.

### Why it fails twice over

- **It is not visible.** The painter draws into the canvas's clip rect, so the
  handle is clipped away entirely. The operator sees eight grips and no ninth,
  and reasonably concludes rotate does not exist — which is precisely what Ken
  concluded, and what three of our own documents also said.
- **It is not reachable.** The press never arrives at the canvas widget at all;
  it lands on whatever occupies that strip of the window, which is the ribbon.

★ So this is convention `handles.md` **H7** — *a handle that cannot act is not
drawn* — failing in its more dangerous direction: the handle is not drawn **and
cannot act**, while the feature is present and correct everywhere else. That is
why it reads as "rotate is missing" rather than as "rotate is broken".

### The general shape

**Any selection whose top edge is within `ROTATE_STEM_PX + GRIP_SIZE_PX / 2` —
24 pt — of the top of the viewport cannot be rotated.** Nothing to do with what
kind of object it is. On a CAD sheet scrolled to the top, that is the title
block, the sheet number and the top row of a BOM: exactly the things an operator
reaches for first.

It is also why `O20` looked like a text problem. The BOM row that
`--doc-point 0,1211,1021` names happens to sit at the top of the sheet.

### ~~The fix is a convention question — do not improvise it~~ — ANSWERED

★ **Ken settled it on 2026-08-21: `O23`.** He asked for the pasteboard and
then for more of it than was proposed here — *"I should also be able to move
the view of the corner of the page to the center of the screen, or even all the
way vertically to the opposite corner"*. Candidate 3 below, sized at **one
viewport on every side**. The analysis is kept because the two rejected
candidates and their reasons are still the record of why.

Three candidates, and the standing rule is *use the conventional interaction,
never invent one*:

1. **Flip the handle below the box when there is no room above.** Cheap and
   local. ★ But it is an **invention** for this gesture: no program in the class
   flips a rotate handle, and an operator who learned "the rotate handle is
   above" would find it moving for reasons they cannot see.
2. **Clamp the handle inside the viewport.** Rejected on sight — it detaches the
   handle from the box it belongs to, breaking convention **C7** (*the drawn
   outline and the live target are the same shape*) to fix an H7 violation.
3. **★ Give the canvas scroll padding, so the page can always be scrolled away
   from the viewport edge.** This is what Illustrator, Acrobat and Inkscape all
   do — you can scroll past the edge of the page, and the pasteboard is why a
   handle at the extreme edge of the sheet is always reachable. It fixes a whole
   class of edge problems rather than this one symptom, including the eight
   resize grips on an object flush with the left or bottom edge, which have the
   same defect and no check yet.

**Recommendation: 3.** It is the conventional answer, it is the only one that
fixes the resize grips too, and it needs no new rule for the operator to learn.
The trace shows `off=[0.0 0.0]` — the canvas is scrolled hard against its own
top with nowhere further to go.

### ★ The check now names this cause instead of three wrong ones

Its first failure listed `Grip::is_resize`, `gesture::meaning` and
`needs_targets` — three real hazards, none of which is what happened, all three
inside the application, and all three in the ROUTING when the defect is in the
LAYOUT. A reader would have gone looking in the wrong file, with a specific and
plausible instruction to do so.

It now measures the handle against the canvas's own declared rect first, and
says so with numbers:

```
★★ THE ROTATE HANDLE IS OFF-CANVAS - defect O22, and NOT a routing problem.
   The selection's top edge is at y=150.2, the handle therefore spans from
   y=126.2, and the canvas begins at y=143.3. The handle is 17.1 point(s)
   above the top of the canvas ...
```

It still **passes** at `0,300,500`, so this is a diagnosis rather than a
blanket failure.

★★ The general rule it earned, and it is the third instance in one evening:
**a confident, specific, wrong accusation is worse than a vague one**, because
it is actionable and it aims somebody at the wrong file. A check that can rule
a cause OUT should.

### ⚠️ CORRECTION 2026-08-21: the resize grips do NOT have this defect

This row claimed, twice, that *"the eight resize grips have the same latent
defect on the left and bottom edges"*. **That is wrong, and it was written
here without being checked** — hours after this same file recorded the rule
that a claim about what the code does is verified against source, not
asserted. It was then repeated to the operator and promoted into
`CONTINUE.md` as scheduled work.

The geometry, from `canvas/handles.rs`:

| affordance | where its CENTRE sits | how far outside the box |
|---|---|---|
| the eight resize grips | **on** the box's edge or corner (`anchor`, `handles.rs:260-267`) | half a grip — `GRIP_SIZE_PX / 2` = **4 pt** |
| the rotate handle | `bounds.top() - ROTATE_STEM_PX` | **16 – 24 pt**, entirely outside |

`handles.rs:269` says it in the source, in as many words:

> *"Above the top edge, centred, by the stem's length. **The one grip whose
> centre is OUTSIDE the box**, which is what the offset is for."*

So a resize grip's centre — and its whole inner half — is inside the
selection box, and therefore inside the canvas whenever the object is
visible at all. **It can always be grabbed.** The rotate handle has no part
inside the box and can be entirely off-canvas, which is why it and only it
disappears.

★ **What is left of the claim, stated accurately**, because there is a
residual and it is cosmetic rather than functional: against a viewport edge
the outer half of a grip is clipped, so it is drawn as a 4 pt sliver rather
than an 8 pt square, and its effective target shrinks from 12 pt (8 + 2 slack
each side) to about 6. Harder to hit, never impossible.

**Consequence for the plan:** the per-edge driven check drops from *"needed
to cover a latent defect"* to *"a reasonable regression guard once the
pasteboard lands"*. `O23` is the whole of the work; there is no second
defect waiting on the left and bottom edges.

★★ The shape, for the third time in one day: **an unverified claim about an
ABSENCE or a DEFECT costs nothing at the moment it is written and is
expensive later**, because nothing fails when it is wrong — it just quietly
shapes a plan. Analysis, not driving: no fixture point flush with a page
edge was available to aim at, and this is labelled as reasoning from the
constants rather than as a measurement.

### What it needs

1. Scroll padding around the page in the canvas's scroll area, sized so every
   affordance of a selection flush with any page edge is reachable.
2. A driven check per edge — top, bottom, left, right — because the resize grips
   have the same problem and nothing has ever aimed at them there.
3. ★ Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s
   afterwards. One point passing is what hid this for a day.


## O21 — Move, resize and rotate ANY object; click nodes, select several, move them — all with live preview

**Asked:** 2026-08-21 — *"I think pdfcer implemented the capability to move and
resize and rotate any object. you'll have to confirm, but that is what I want. I
should be able to click individual nodes, or select several at once and move
them too, with live preview of everything if possible."*

**Status:** ★ **ENGINE CONFIRMED 2026-08-21 against `D:\Dev\pdfcer` source.**
You were right, with two boundaries worth knowing. Asking for it to be
confirmed rather than assumed was the correct instinct and it paid: the
confirmation also caught **a claim in this very file that was false**, and I
had re-published it an hour earlier — see `O20`.

### What the engine actually does

**`EditSession::transform_objects`** (`crates/pdfcer-core/src/edit.rs:7512`) is
**genuinely kind-agnostic**: one verb, one undo entry, doing move, scale,
rotate, shear and mirror on **paths, text objects, image XObjects, form
XObjects and inline images**. It is kind-agnostic *by construction* rather than
by a match — it wraps each object's byte span in `q … cm … Q` and never reads
an operand (`vector/edit.rs:996`). So *"any object"* is true for page content,
and it is true of text specifically.

**Three places it stops being true**, and they are worth knowing because each
is something you might reasonably try:

| | |
|---|---|
| **Annotations** — markup, form fields, ce dimensions | no transform verb at all. Translate only, or nothing. And a `/Rect`-based markup **cannot express a rotation**: the engine's own words, *"a rotated one has no spelling"* |
| **Below whole-object level** — subpaths, nodes, Bézier handles | **translate only.** There is no rotate or scale for a node selection |
| **Inside a placed block (form XObject)** | not addressable. The decomposer treats it as one object and does not recurse, so you can rotate the block but nothing within it |

### Nodes — better than expected

**`move_nodes`** (`edit.rs:8486`) moves **several anchors in one call**, each
to its own destination, as one command and one undo entry. ★ And a loop of
single moves would have been *wrong*, not merely slow — all four corners of an
`re` rectangle are the same four operands, so the second call would plan
against byte offsets the first had already replaced.

**Bézier control points are separately addressable** (`move_handle`,
`edit.rs:8542`), and it refuses a straight segment by name rather than quietly
turning a line into a curve.

★ The shell already has multi-node selection and already calls `move_nodes`
(`canvas/moving.rs:560`, `app/actions/vector.rs:469`). Whether you can *build*
that selection by pointing — marquee inside an object, Shift+click to add — is
the open question, not whether the verb exists.

### Live preview — already there for all three gestures

`canvas::overlay` draws a move ghost, a **rotate ghost** (four transformed
corners as a quadrilateral, deliberately not a growing rectangle) and a resize
ghost. They are outlines rather than re-rendered content, which is the correct
trade and is documented as such.

### ★★ The one real gap the confirmation found

**`transform_preview` is never called.** It is the engine's preflight and it
distinguishes two refusals the UI must treat differently: `DegenerateCtm`
means *this object can never be transformed — do not offer a handle*, and
`SingularTransform` means *this particular drag collapses it — offer the
handle, refuse on release*.

`canvas/resizing.rs:172` admits it in the source, in these words:

> *"A handle is currently offered for an object that can never be transformed,
> and the operator finds out by dragging it. That is a real gap."*

It is unbuilt for a measured reason rather than an oversight: the preview
**decomposes the whole page** — about 4 seconds in a debug build on your
benchmark drawing — so it cannot be asked per frame. It needs a cache keyed on
`(page, edit epoch, selection)`.

### And one thing to watch in the file you send on

Every `transform_objects` call adds a fresh `q`/`cm`/`Q` wrapper **per object,
per gesture**, and nothing folds them together. Forty nudges nest forty
wrappers and the file grows monotonically. The shell already dodges this for
the common case — an all-path move takes the lighter `move_objects`, which
rewrites coordinates and adds no bytes — but a rotate or a resize cannot.

This row supersedes nothing. It **subsumes** `O20`'s rotate half and `O11`'s
rotate paragraph, both of which say the same thing more narrowly: the verb
exists and there is no grip to reach it with.

### The four things asked for, separated because they are in different states

| | what he wants | state |
|---|---|---|
| 1 | **Move / resize any object** | move and resize ship (`O11`, `O12`) — *"any"* is the part to verify, not the verb |
| 2 | **Rotate any object** | the engine verb rotates. **There is no rotate grip on the canvas**, for any object kind. Shell work, unblocked since 2026-08-20 |
| 3 | **Click individual nodes; select several and move them** | the Node tool (`A`) and a multi-node move both exist. Whether a *multi-node selection* can be built by pointing is the open question |
| 4 | **Live preview of everything** | move, resize and rotate ghosts exist in `canvas::overlay`. Whether every path has one is the open question |

### ★ On "any object", which is the word to be careful with

*Any* is the operator's word and it is the right requirement. It is also the one
most likely to be quietly false in a specific place, and this project has the
shape on record already: `O11` shipped move-and-resize while
`transform_objects` refused a **degenerate placement matrix**, and the engine
said in as many words *"do not offer a handle"* — so the shell offers one and
the operator finds out by dragging it.

So the confirmation must answer, per verb, **which object kinds it refuses and
why**, not merely whether the verb exists.

### ★★ Live preview is a standing expectation, not a per-feature request

> *"I've never seen a program that doesn't live preview any change, and yet here
> I am having to ask for all the minute details as if you'd never been trained
> on it."*

He has now reported the absence on three separate features. Treat *"with live
preview of everything if possible"* as the default requirement for every drag
this row touches, not as an optional fourth item — a rotate that only shows its
result on release is not finished.

★ And `ui-conventions/handles.md` H1 already says it: *selecting something shows
how to manipulate it*, before any drag.

### What this needs, in order

1. **Confirm the engine, per verb and per object kind**, with `file:line`
   against `D:\Dev\pdfcer` source rather than against `docs/core-api/index.md`,
   which is a dated snapshot. Report what it refuses.
2. **The ninth grip** — rotate — painted and hit-tested from **one** predicate.
   `ui-conventions/handles.md` H7 and C7, and the trap this shell fell into
   once already: vertex handles painted for a selected dimension that could not
   be grabbed, because the painter asked about the selection and the hit test
   asked about a capability the mode lacked.
3. **Multi-node selection by pointing** — marquee inside an entered object, and
   Shift+click to add a node — feeding the multi-node move that already exists.
4. **A preview on every one of those drags**, and a check that each one
   *renders* rather than merely being constructed.
5. **The degenerate-matrix preflight**, so a grip that cannot act is not drawn
   (H7). Named in `O11` as needing a page decomposition cached per selection.

★ **Nothing here may be reported as working on the strength of a passing test.**
The Select button was green on 1,628 tests, 17 gates and a smoke launch on the
same day it reached him doing nothing at all.


## O20 — Dragging and rotating TEXT on the canvas

**Asked:** 2026-08-21 — *"I also can't drag and rotate text on the screen yet."*

**Status:** **RECORDED 2026-08-21. NOT STARTED.** Two separate things behind one
sentence, and they are in very different states, which is why they are written
out rather than merged.

### ~~Rotate — nothing to grab~~ — ⚠️ WRONG, AND WRITTEN BY ME, TODAY

**The rotate grip exists and has since 2026-08-20** (`560280a`). This section
was written on 2026-08-21 by reading `O11` and `O14` row 5 and trusting them
instead of the source. Both were stale; the claim was three weeks' worth of
true and one day's worth of false, and I re-published it as current.

★ **The lesson, which is the same one this file already carries twice:** a row
in the backlog is a record of what was true when it was written. It is not
evidence. `git log -S` and the source are evidence, and they cost a minute.

So the operator's *"I also can't drag and rotate text"* is **not** explained by
an absent grip. It needs driving. The candidates, none confirmed:

- **Grips are drawn at the Object rung only** (`overlay.rs:220`). If a click
  descended into the text object — to a run, or a caret — the box and its nine
  grips are gone, correctly, and there is nothing to grab.
- **A click on text may be arming the caret rather than selecting the object**
  — rung 2 of `clicking.rs`'s ladder beats rung 8.
- **The mode.** Content selection needs `edit_content`; Read and Review have
  no grips because they have no content selection.

★★ **ANSWERED 2026-08-21 by driving: see `O22`.** It is not about text at all.
The rotate handle sits 20 pt ABOVE the selection box, and the object he was
aiming at is near the top of the sheet — so the handle is drawn 9 pixels above
the top of the canvas, where it is clipped away and where a press lands on the
ribbon instead. Any selection within 24 pt of the top of the view has this,
whatever kind of object it is.

The superseded text follows.

~~`O11` and `O14` row 5 both already say this and neither has been actioned:~~

> The verb rotates. **There is no rotate handle on the canvas to reach it
> with.** … it needs a ninth grip above the selection box, a drag that measures
> an angle rather than a distance, and a preview.

★ **Nothing is blocked.** The engine verb shipped 2026-08-20. This is entirely
shell work and has been since then. It applies to *everything* selectable, not
only text — a picture, a shape and a text run all have the same absent grip —
so the sentence *"I can't rotate text"* is really *"nothing on the canvas can be
rotated by pointing at it"*.

`ui-conventions/handles.md` H2 already specifies the shape: eight resize grips,
a body, and a rotate handle above the box. This is the "use the conventional
interaction, never invent one" case with the convention already written down in
this repository.

### Drag — claimed as shipped, never verified on text specifically

`O12` says text became draggable on 2026-08-20, through the same verb as an
image, and its own row ends:

> **NOT YET DRIVEN** on a text object specifically — the driven checks aim at a
> shape, because that is what the fixture's `--doc-point` names.

So there are two possibilities and this row does **not** guess between them:

1. It works and he has not found the gesture — which would make it a
   discoverability defect rather than a functional one, and those are fixed
   differently.
2. It does not work on text, and the row that claimed it did was claiming a
   verb rather than a behaviour.

★ Given the afternoon of 2026-08-21 — a Select button that did nothing at all,
green on 1,628 tests and 17 gates — **possibility 2 gets no benefit of the
doubt.** The first action is to drive it on a text run, not to reason about the
verb.

### What it needs, in order

1. **Drive a drag on a text object** at a `--doc-point` that names one, and find
   out which of the two above is true. Cheap, and it decides everything else.
2. **The ninth grip.** Painted from the same predicate that hit-tests it —
   convention C7 and H7, and the trap this shell has already fallen into once:
   *"a set of vertex handles was painted for a selected dimension and could not
   be grabbed, because the painter asked about the selection and the hit test
   asked about a capability the mode did not have."*
3. **A live preview during the rotate drag**, because he has reported the
   absence of live preview on three separate features now, and *"I've never seen
   a program that doesn't live preview any change"* is a standing expectation
   rather than a per-feature request.
4. The **degenerate-matrix preflight** named in `O11`: an object whose own
   placement matrix is collapsed cannot be transformed and the engine says *do
   not offer a handle*. Offering a grip that cannot act is exactly what H7
   forbids.


## O19 — In single-page mode, an option to turn the page when you scroll past its end

**Asked:** 2026-08-21 — *"also in single page mode I'd like a little checkbox
below that option to go to the next page when scrolling, and unchecked it keeps
its current behaviour."*

**Status:** **RECORDED 2026-08-21, NOT STARTED.**

A checkbox positioned **below the Single page option** in the page-display
group, so it reads as a qualifier on that mode rather than as an independent
setting — which is what it is: it has no meaning in Continuous, Facing or
Facing-continuous, where scrolling already crosses page boundaries.

- **Checked** — scrolling past the bottom of the page moves to the next page,
  and past the top moves to the previous one.
- **Unchecked** — today's behaviour exactly, which is that the scroll stops at
  the page's edge. **This is the default**, because it is what the shell does
  now and changing what an existing control does without being asked is the
  regression R6 forbids.

### The parts that are not decided, and are not to be improvised

These are the questions the class has already answered and they should be
checked against a real program rather than guessed:

- **Does the new page arrive at its top or at its bottom?** Scrolling *down*
  onto page 4 should land at page 4's **top**; scrolling *up* onto page 3 should
  land at page 3's **bottom**. Anything else teleports the reader.
- **Is there resistance at the boundary?** Every reader in the class makes you
  reach the edge and then scroll *again* rather than sliding straight through,
  so that a fast flick down a long page does not overshoot into the next one.
  Acrobat, Preview and every browser PDF viewer do this.
- **Does it interact with zoom?** At a zoom where the page is narrower than the
  viewport there is no vertical travel at all, so the first scroll event is
  already at the boundary. The resistance rule above is what stops that
  becoming "one wheel click skips a page".

### Where it lives

The page-display controls are `View` ▸ page display, mirrored by
`viewer::PageDisplay`. The checkbox belongs beside them and **not** in the
Settings window: it is a view control the operator changes while reading, in
the same group as the mode it qualifies.

## O18 — Ctrl+C on selected TEXT puts "1 object copied from pdfcer" on the clipboard

**Asked:** 2026-08-21 — *"in the build from 9:50 this morning if I select text
in read mode, or edit and select text in an edit box in the canvas in edit
mode, and press ctrl+c to copy, then try to paste in notepad, it doesn't work.
I get a notice to paste it back into pdfc to place it."*

**Status:** ★ **CONFIRMED BY THE OPERATOR, 2026-08-21** — *"copy paste now
works!"* Fixed in all three places.

★★ **And it is now DRIVEN**, 2026-08-21, on the real application with the real
Windows clipboard:

```
ctrl_c_copies_text_to_the_os_clipboard   PASS
  the sweep selected 10 character(s); the clipboard holds 10 after Ctrl+C
  clipboard begins "- 22 - 250"
```

The check reads the **operating system's** clipboard from outside the process,
which is the only oracle that can see this defect: the failure was never in a
function's return value but in which of two handlers reached the OS last, and a
trace cannot see that either. It **clears the clipboard first** — without that,
*"the application did nothing"* and *"the application copied correctly"* are
the same observation whenever an earlier run left the right text behind.

The row still stays open until you close it.

What changed: `textsel::clipboard::pending_key` reads `Event::Copy` instead of
a key event that never arrives; `canvas/textedit/` gained Copy, **Cut and
Paste**, which it had none of; and `canvas::clipboard::text_owns_the_chord`
makes that module's oldest claim — *"text wins"* — actually true, so the object
path stands aside instead of writing its marker over the text.

`tools/gates/check-clipboard-chords.sh` now fails the build on any source file
that asks about `C`, `X` or `V` as a **key**. That gate is the part that
outlives this row: the real failure here was not the egui-winit quirk, which
had already been found and written up in capitals a day earlier, but that
nobody asked who else read the same broken signal.

★ **What to try, and what would still fail.** Sweep text in Read and Ctrl+C;
select inside a text box in Edit and Ctrl+C, Ctrl+X, Ctrl+V. A multi-line paste
will arrive as **one line** — the draft is single-line until O15, and that is
named here rather than left for you to find.

### The sentence he is seeing, and where it comes from

`crate::text::clipboard::os_marker` — *"1 object copied from pdfcer. Paste it
back into pdfcer to place it."* It is written to the operating system's
clipboard deliberately, by the **object** copy path, and it is not a bug in
itself: `egui-winit` synthesises a paste event only when the OS clipboard holds
non-empty text, so without *something* there, whether Ctrl+V works inside pdfcer
would depend on what the operator last copied in another application.

The defect is that this sentence is reaching the clipboard when the operator
copied **text**, which should have put the text there.

### Case 2 — inside a text edit box. CONFIRMED, cause known

**`Key::C` is handled in exactly one place in the whole canvas**:
`canvas::textsel::clipboard::pending_key`. That function opens with

> *"A canvas draft claims these chords too … Ctrl+C mid-word must not copy the
> page's text selection: the operator is composing, and the selection they made
> before the caret landed is not what those two keys mean any more."*

and returns `None` whenever `canvas::textedit::composing()` is true.

That reasoning was right about what Ctrl+C must **stop** doing and never
supplied what it must **start** doing. `canvas/textedit/` has no Ctrl+C
handler of any kind — no `Key::C` appears anywhere in it. So inside a draft the
chord falls straight through to the ribbon keymap, which binds it to
`edit.copy`, which is the **object** clipboard, which writes the marker.

★ Note what this means precisely: **selecting text inside an edit box and
pressing Ctrl+C has never copied that text.** Not since the draft selection
shipped on 2026-08-21. The gesture is new; the gap arrived with it.

The shape is worth recording because it is a recurring one: a guard was added
to stop a chord doing the *wrong* thing, and stopping it was treated as the
whole of the job. The chord then had no owner at all, and fell through to
whatever claimed it next.

### Case 1 — a text sweep in Read mode. ROOT CAUSE FOUND, and it is the same one

The first draft of this row listed three candidates and said the convenient
answer was the one to distrust. It was right to, and all three were wrong.

**`Ctrl+C` never reaches `textsel` at all, in the real application, and never
has.** `canvas::textsel::clipboard::pending_key` asks
`InputState::key_pressed(egui::Key::C)`. In a real window that is permanently
false, because of fifteen lines of `egui-winit-0.35.0/src/lib.rs`:

```rust
if is_cut_command(modifiers, active_key)   { events.push(Event::Cut);   return; }
if is_copy_command(modifiers, active_key)  { events.push(Event::Copy);  return; }
if is_paste_command(modifiers, active_key) { … events.push(Event::Paste(contents)); return; }
events.push(Event::Key { … });
```

**The `return` comes before the `Event::Key` push.** So `Ctrl+C` produces
`Event::Copy` and *no key event whatsoever*. A function asking "was C pressed
with Ctrl held" can never be told yes.

★★ **This project already knew.** `app::keyboard` carries that exact quotation
under a heading reading *"CTRL+C, CTRL+X AND CTRL+V NEVER ARRIVE AS KEY EVENTS,
AND THAT IS WHY THEY HAVE NEVER WORKED"*, written on 2026-08-20 after the
operator reported the chords dead twice. That module was fixed: it translates
`Event::Copy` through the keymap, which is why `edit.copy` fires at all.

`canvas::textsel::clipboard` was not, and nobody noticed the second reader of
the same broken signal. So the finding was recorded, the general lesson was
written down, and the sweep's own copy went on being dead beside it.

### Why the two cases produce the marker rather than silence

With the text path dead, the surviving handler is `app::keyboard`'s: it turns
`Event::Copy` into the keymap's `edit.copy`, which is the **object** clipboard,
which writes `os_marker` — the sentence Ken pasted into Notepad.

So there is one defect wearing two faces:

| | why the text is not copied | what writes the marker instead |
|---|---|---|
| sweep, Read | `pending_key` reads a key event that does not exist | `Event::Copy` → keymap → `edit.copy` |
| draft, Edit | no `Ctrl+C` handler exists in `canvas/textedit/` at all | the same |

### ★ The transferable lesson, which is the expensive half

**A finding recorded in one module is not a fix applied to its siblings.** The
question that was never asked on 2026-08-20 is *who else reads this signal?* —
and the answer was one grep away: `Key::C` appears in exactly one other file.

The gate this suggests is a real one: **nothing in this crate may ask
`key_pressed` about `C`, `X` or `V`**, because the answer is always false. That
is checkable by a script, unlike the behaviour it protects.

### What it needs

- `canvas::textsel::clipboard::pending_key` reads `Event::Copy` (and `Cut`)
  rather than `Key::C`. ★ Its unit tests inject `Event::Key { key: C }` and
  **pass**, which is how this survived — they must be changed to inject what
  winit actually sends, or they will keep certifying a dead path.
- `canvas/textedit/` gains Copy, **Cut and Paste** for a draft's selection. All
  three are missing, not just the one reported: a text box you cannot paste
  into is the next report.
- A driven check per case that asserts **the operating system's clipboard**
  holds the expected text, having cleared it first. A trace line cannot see the
  clipboard, and the clipboard is the thing that is wrong.
- A gate forbidding `key_pressed` on `C`/`X`/`V` anywhere in the crate.

## O17 — Selection is governed by a FILTER on the status bar, not by two menus at the top

**Asked:** 2026-08-21 — *"Can we change how editing works? On the bottom bar I
want a filter menu that pops up with all the options of what to enable
selecting of — text, points, lines, etc — all the object types (glyphs beside
each option). … We should put a view one beside it that allows the changing of
what objects show bounding boxes around them on screen. … This is to replace
the wonky content edit text and edit objects menu at the top. … we should also
add a right click feature to select other for objects that are under another
object."*

**Status:** **PARTS A AND C BUILT 2026-08-21. THE POPUP SHIPPED BROKEN THE
FIRST TIME AND IS FIXED.** Parts B and D not started.

| part | state |
|---|---|
| **A** — the Select filter popup | built; **shipped inert, fixed 2026-08-21** — see below |
| **B** — the View twin (bounding boxes, node markers) | not started |
| **C** — what a click means per mode | built: the filter gates the hit test in all three modes |
| **D** — right-click *Select other* | not started |

### ★★★ The first build of part A did nothing at all, and how that happened

The operator, within minutes of opening it: *"I see a Select button, but this
should be a menu that pops up to choose what I can select on the screen and
edit in editor mode."*

The button was drawn, in the right place, and clicking it did nothing.
`egui::Popup::menu` is defined as `from_toggle_button_response`, which
**already toggles the popup open on click**. The code called
`Popup::toggle_id` as well, against the same id, in the same frame — so the
popup opened and closed before it could be drawn. From outside, a popup
toggled twice in one frame is indistinguishable from one that was never
wired up.

★ **What makes this worth writing down is everything that was green.** 1,628
unit tests. 17 of 17 gates. An offscreen smoke launch that confirmed the
button's rect was published at the exact intended spot on the status bar. All
of them observed the **button**, and the button was never the broken part.
This is R1 stated as an incident rather than as a rule: *the tests pass* is
not a report of working software, and it reached you because nothing had ever
opened the popup before you did.

There are now four tests that click the button and assert the popup's open
flag directly. **They were checked by re-introducing the defect and watching
two of them fail**, which is the only way to know a regression test tests
anything.

★★ **And parts A and C are now DRIVEN**, 2026-08-21:

```
select_filter_changes_what_a_click_hits   PASS
  with every class on, the click selects
  with every class off, the same click selects nothing
  switching them back on restores it
```

Deliberately **not** *"the popup opens"*. That is already a unit test, and it
is also the one claim that stays true of an inert control — which is precisely
what this popup was for an hour this morning.

### The specification, unchanged

### What is being replaced, and why he calls it wonky

The two menus at the top ask the operator to **declare an intention before
pointing at anything** — *I am now editing text*, *I am now editing objects* —
and then hit-testing obeys the declaration rather than the drawing. That is the
wrong end of the gesture. It means the same click on the same pixel does
different things depending on a control the operator is not looking at while
they click, and it means reaching two levels into a ribbon to make a line
selectable.

Every program in the class solved this the other way round: **a persistent,
always-visible filter that says what is pickable, parked where it can be
glanced at without leaving the page.** AutoCAD's object snap and selection
filter, Illustrator's layer lock column, Inkscape's "Select Same" plus its
per-layer locks, Acrobat's own object-type restriction — the mechanism differs,
the shape does not. Ken's placement (status bar, popup on click, glyph per row)
is the CAD convention exactly, and per the standing rule the convergence of the
product class IS the specification.

### A — The Selection Filter popup

Lives on the **bottom status bar**. Click opens a popup listing **every object
class pdfcer can hit-test**, each with a glyph and a checkbox. Enabled = that
class accepts a click. Disabled = clicks pass straight through it to whatever is
behind, in every mode, with no exception.

The class list must be derived from what the hit test can actually distinguish
today, not invented — text, glyph/character, path, line segment, node/vertex,
image, shape/annotation, ce dimension, form field, markup, link, and whatever
else the selection enum already carries. **Every class in the enum gets a row;
a class with no row is a class the operator cannot reach.**

Needs, at minimum: All / None, and the state must persist across sessions like
any other operator preference.

### B — The View popup, beside it

Same placement, same shape, different question: **what is drawn as a bounding
box or a node marker while unselected.**

- Text **off** = renders exactly as it renders. Text **on** = a box around each
  text run, always, selected or not.
- Objects **on** = a box around each object.
- Nodes **on** = the vertices of paths shown as markers, so they can be seen
  before they are aimed at.

★ **This is disclosure furniture, not content marking, and the distinction has
to be held.** Rule 4 forbids styling *applied content* to signal uncertainty.
It does not forbid an operator-controlled overlay that reveals structure — that
is the same category as a CAD program's grid or an editor's whitespace marks:
**off by default, switched on deliberately, drawn as chrome over the page and
never mistaken for ink.** The test still applies: with every View toggle off, a
screenshot of the canvas must be indistinguishable from the saved-and-reopened
document. That is what makes this safe.

### C — What a click MEANS, per mode

| mode | single click on a filtered-in object |
|---|---|
| **Read** | selects it. From that selection: clipboard operations, and form filling |
| **Review** | selects it, and permits editing of the things review owns — markup, comments, form fields |
| **Edit** | selects it, and permits editing of anything |

**In all three modes the filter is authoritative.** A class switched off in the
filter is not selectable in Read, not selectable in Review, not selectable in
Edit. The filter sits *above* the mode, not inside it.

★ **Open question, and it is a convention question rather than a preference
one:** whether entering edit-on-an-object in Edit mode is the single click that
selected it, or a second click / double-click. He named the alternative himself
— *"or double clicking if that is the more common convention"* — which is the
right instinct and the right person to be asked. **The class answer is
double-click**: PowerPoint, Illustrator, Figma, Visio and Acrobat all use
single-click-selects, double-click-enters. Single-click-enters exists mainly in
programs with no selection concept at all. Proposed, for his ruling: **click
selects, double-click enters the object's editor**, with Enter as the keyboard
equivalent on a selection.

### D — Right-click, including "Select other"

The escape hatch for the topmost-wins rule (`click-selects` C3). When objects
stack, the click correctly takes the top one; **Select other** walks the stack
underneath the cursor. The class convention is a submenu listing each candidate
by type and hovering it highlights that candidate on the page before committing
— Illustrator and Visio both do exactly this.

Other entries that make sense on a canvas right-click, to be specified rather
than improvised: Cut / Copy / Paste, Delete, Properties, Bring/Send order where
it applies, and the object's own primary action (Edit text…, Edit dimension…).
Right-click on **empty** page gets its own short menu — Paste, Select All, and
the two filter popups so they are reachable without travelling to the status
bar.

### What this row does NOT decide

- The exact class list — that comes from reading the selection enum, and any
  class the enum cannot currently distinguish is a **finding**, not a row to
  quietly drop.
- Whether the top menus are deleted or left as a redundant path during
  transition. Ken said *replace*; that is read as delete, but it is his call.
- Glyph choices.

### Why this is a bigger change than it reads

Every one of these touches the same function: **the thing that turns a click
into a selection.** Adding a filter, adding a stack-walk, adding mode-dependent
consequences and adding node visibility are four demands on one code path that
currently answers one question. `click-selects` C8 says the priority order must
be *written down in one place and testable* — this row is the reason that rule
now has to be honoured rather than noted, because four new claimants are about
to arrive at the same press.

## O15 — Text editing should be MULTI-LINE

**Asked:** 2026-08-21 — *"I should be able to make it multi line."*
**Status:** **SHIPPED 2026-08-21 AND DRIVEN.** Awaiting your verdict.

Arm **Edit ▸ Add text** and **drag a rectangle**. Type into it. **Enter starts a
new line**; text that runs past the right edge wraps by itself. **Ctrl+Enter**
puts it on the page — or click away, which also commits.

A plain click still places a single line at a point, exactly as before, and
Enter there still commits. Two gestures, two behaviours, and the one you get is
decided by whether you dragged.

### Why it needs a rectangle, which is not a design choice

**A PDF has no paragraph.** Every visual line in a PDF is a separate instruction
at its own absolute position — there is nothing in the file that says "these
lines belong together". So something has to decide where the second line starts,
and the only thing that can is a width to wrap against. That is the rectangle.

### Driven

```
text-box-open page=0 box=301.2,438.8,500.9,499.7 w=199.7 h=60.9
add-text page=0 n=2 … boxed add: wrapped to 2 line(s) at 199.7pt box width,
         left alignment, top-anchored from the box top at 14.40pt leading
```

★ **And the check earned its keep on its first run.** Everything was correct —
the drag opened the box, Enter arrived, the right branch ran — and the newline
was **thrown away one function deeper**, by a filter that strips control
characters from typed text. Its own comment argued, correctly, that a control
character has no meaning in a PDF show string. True of typed text; not true of a
paragraph break. That is the **fifth** carefully-argued restriction in two days
to go false the week it was written.

### Still open, and named rather than implied

- **Turning existing text into multiple lines.** This ships *new* multi-line
  text. Making a line that is already on the page break into two is a *reflow*,
  which the engine has and which currently demands the page be saved and
  reopened first. Separate row when you want it.
- **Alignment and the box's own size.** A new box is left-aligned and cannot be
  resized after the fact. Both are surfaces rather than engine gaps.

## O16 — Reassemble lines into paragraphs, and move between blocks with the arrow keys

**Asked:** 2026-08-21 — *"there was an acrobat feature in the original pdfcer-gui
that attempted to reassemble individual lines into paragraphs and the cursor
would move to the next block of text using the navigation keys."*
**Status:** **SHIPPED 2026-08-21 AND DRIVEN.** Awaiting your verdict.

Put the caret in any piece of text and the navigation keys now work on **the
page**, not on the fragment you happened to click:

| key | where it goes |
|---|---|
| **↓ / ↑** | the line below / above — **including into the next block of text** |
| **End / Home** | the end / start of the line **you can see**, however many pieces it was drawn in |

A CAD sheet draws one visible row as four or five separate instructions, so
"the end of the line" and "the end of the thing I clicked" are different places.
End now goes to the first of those.

### It was SALVAGE, and the salvaged part was four lines

The old shell asked `pdfcer-core` four questions — `caret_up`, `caret_down`, and
`line_range_at`'s two ends — and that was the whole of its contribution. **The
reassembly was always the engine's**: `recognize` groups a page's instructions
into lines and lines into blocks by column band, and `caret_up` walks *lines*.
So a caret on the last line of one paragraph steps into the next without
anything in the shell knowing what a paragraph is.

This shell had not been asking, and at the time that was right: its caret is a
position inside **one** run, and a single run has no line above it. What changed
is not the caret — it is that the *page* is now the thing being navigated.

### Driven

```
text-edit-caret  kind=Edit page=0 run=232 len=18     (a BOM row: "SW41177 - 22 - 250")
text-caret-step  dir=Down from_run=232 to_run=240 to_caret=8
text-caret-line  end=true from_run=232 to_run=236 to_caret=1
```

Down crossed into the row beneath and Up came back; End crossed into the rest of
the same row. Three different runs, all reached with the keyboard.

★ **The first live run failed, and the failure was not a defect.** No caret
movement at all — and the trace could not say whether the keys had been eaten on
the way in (a bug) or whether the model had simply found nothing above or below
(a fact about *where the click landed*, because the engine never crosses a
column band and a lone label has nothing stacked over it). Both look like
silence. The fix was a second trace line for the *nowhere* outcome, so the check
now **skips** on the second cause and accuses only on the first. A trace that
can only report success cannot tell a broken build from an unlucky fixture.

### Still open, and named rather than implied

- **Showing which block the caret is in.** The recognition is there; drawing it
  is a separate surface and R8b rule 4 governs how — off-canvas, never a mark on
  the page.
- **A remembered column.** Three presses down and three back up return the caret
  to where it started only when the lines are of similar length. A true desired
  column survives short lines in between; that is a second piece of state.
- **Up and Down inside a text BOX** still do nothing, deliberately: a box's
  lines are the shell's own wrap, and answering with the page model would throw
  the caret to a run somewhere else on the sheet.

## O14 — The conventions sweep, 2026-08-20: fourteen gaps, found by asking

**Asked:** by you, as *"how can you learn from these other programs so that you
can build the missing parts more effectively?"*
**Status:** the mechanism is built. These are what it found on its first run.

`D:/dev/rag/ui-conventions/` is a corpus of five gesture classes — what every
program in the class already does, where the rule comes from, and the failure
mode when it is absent. `tools/gates/check-conventions.sh` makes each
interactive surface answer every row of its class, in its own source, and fails
the build on an unanswered one. Eleven surfaces registered; all eleven now
answer.

It cannot check behaviour and does not pretend to. It checks that **the question
was asked** — which is the whole of the problem, because every convention you
have had to report was one nobody had asked about rather than one somebody
decided against.

### What it found. None of this was known before today.

**Direct manipulation**

1. ~~**Shift does not preserve aspect on a resize.**~~ — **SHIPPED
   2026-08-20.** Hold Shift while dragging a corner and the shape keeps its
   proportions; the status row says *"Shift: keeping its proportions"* so you
   can tell the key did something. A side handle under Shift scales both axes,
   which is what Figma and Slides do.
2. ~~Shift does not constrain a move, a handle drag or a dimension drag to an
   axis.~~ — **SHIPPED 2026-08-20**, all four drags. A move, a dimension label,
   a perimeter corner and a Bézier handle each lock to whichever axis you have
   travelled furthest along, re-decided every frame — so you can start off
   crooked, commit to vertical mid-drag, and it follows. Let go of Shift and it
   comes straight back to the free path.

   ★ A Bézier handle locks to its **anchor's** axis rather than to where you
   grabbed it, because a control point's meaning is the tangent it defines.
   That is what Illustrator and Inkscape do and it is the one place the four
   drags differ.

   **Not built, and named rather than implied:** Alt to scale about the centre,
   Alt to break a smooth node's symmetry, a 45° diagonal lock, and a dimension
   label held to its *standoff* or its *slide* specifically rather than to a
   page axis. Each is a decision recorded in `canvas::constrain`'s header, not
   an omission.
3. ~~**A vertex drag does not snap**, while the tool that placed that vertex
   does.~~ — **SHIPPED 2026-08-20.** Drag a perimeter corner and it now snaps to
   endpoints, midpoints and intersections exactly as the tool that placed it
   does, with the same marker at the same size, honouring the same *"Snap to
   content"* switch. Hold **Alt** to refuse the offer for one drag.

   ★ **The snap overrides the grab point**, deliberately. If you grabbed the
   handle three pixels off centre, a corner that preserved that offset would
   land three pixels off the thing it snapped to — a corner that looks snapped
   and is not, which is the worst of the three outcomes.

   The **label** drag still does not snap and will not: a label's position is
   presentational, it changes no measured value, and snapping a caption to a
   wall would move it onto the drawing rather than clear of it.

   Driven: `measure_perimeter_traces_and_closes` now asserts the drag **asked**
   the snap query — the `snap=` field exists on the shell's own line. It does
   not assert a hit, because whether anything is near that destination is a
   fact about the fixture and not about the build. NOT YET RUN.
4. Neither a move, a resize nor a handle drag snaps to guides, grid or geometry.
5. ~~**No rotate handle** anywhere.~~ ⚠️ **CLOSED 2026-08-20 by commit
   `560280a` and not marked until 2026-08-21.** The ninth grip is painted and
   hit-tested from one predicate, with a ghost and 15° Shift-snapping. The
   struck text below is kept because the row's history is the point:
   ~~**the verb
   shipped 2026-08-20 and rotates**; what is missing is a ninth grip above the
   selection box to reach it with, a drag that measures an angle rather than a
   distance, and a preview. **Shell work, unblocked, and the next thing on the
   list unless you say otherwise.**
6. **No right-click to add or remove a perimeter point**, though both engine
   verbs and the preflight that greys the menu item already exist.
7. A zero-travel release still raises an action in three of the four drag paths.

**Selection**

8. **Only ce dimensions hit-test their real shape.** A `/Square` with no interior
   colour still claims its interior, so a large empty callout box is
   un-clickable-through. The mechanism to fix it now exists — that subtype needs
   a shape.

**Text**

9. ~~No live preview while typing~~ — **FIXED 2026-08-20.** An in-place editor
   box, sized to what you type, with the caret measured against the text as
   drawn. The design had always intended the characters to be shown *off-canvas
   in the status bar* and that half was never built, so they appeared nowhere at
   all.
10. Caret indices are characters, not grapheme clusters, so a combining mark or
    an emoji takes two presses. `unicode-segmentation` is already in the tree.
11. ~~**No selection inside a draft** — no Shift+arrow, no Ctrl+A, no
    drag-select.~~ — **SHIPPED 2026-08-21 (keyboard half) AND DRIVEN.**
    Shift+arrows, Shift+Home/End and Ctrl+A select; typing replaces the
    selection, Backspace and Delete remove it, and any move without Shift drops
    it. The highlight is drawn under the text, in the theme's own selection
    colour, measured against the characters you can actually see.

    ★★ **And Shift very nearly did not arrive at all.** The first driven run
    moved the caret and selected nothing. With Shift physically held down
    through three presses, the toolkit reported it as *held* to one half of the
    program and *not held* to the other, on the same frame, three times running:

    ```
    ev=Modifiers::NONE  frame=Modifiers { shift: true }
    ```

    A key event is stamped with the modifier state at the moment it is
    translated, and the modifier state itself arrives as a separate event; when
    the two land together with the key first, the key carries nothing. The shell
    now asks both, and the reason it is safe to is an asymmetry rather than a
    preference: reading Shift as held a moment after it was released extends a
    selection by one character and the next press fixes it, while reading it as
    absent **destroys the selection** and no keypress brings it back.

    ★★ **AND THE POINTER HALF LANDED 2026-08-21 TOO — but it is NOT yet
    driven, and that distinction is the whole of this paragraph.** Drag across
    the text in the editor box and it selects what you crossed; double-click a
    word and it takes the word. Both are unit-tested against the **real** text
    layout — the same one the caret is drawn from, so where the pointer lands
    and where the caret appears cannot disagree — and the driven check that
    sweeps the pointer across a live draft on your own drawing is **written and
    has not been run**, because you came back to the keyboard and the harness
    takes the cursor.

    Until that check runs, this row is *built and unit-tested*, not *verified*.

    ★ Two things it deliberately does: a sweep that **starts** in the box and
    runs off onto the page keeps selecting to the end of the text, the way
    every text field does; and a press that starts on the **page** never
    becomes a text selection however far it is dragged into the box, so a
    marquee that happens to cross the editor is still a marquee.

**Dialogs**

12. ~~**No dialog is a real OS window**~~ — **PRINT SHIPPED 2026-08-20.**
    Print now opens in its own window: a title bar you can drag, a taskbar
    entry, and you can put it on the second monitor or move it off the drawing
    to read the page underneath while you choose a range.

    Your words, recorded because the last sentence was the diagnosis:

    > *"Print dialogue box doesn't pop up in its own movable window. It is
    > locked within the boundaries of the program's window. Like, I just assume
    > you've been trained on a million lines of code and software that pops it
    > up in its own window."*

    The mechanism is one host, so **the other thirteen dialogs are one line
    each** rather than thirteen implementations. Print first because you said
    to start there.

    ★ **Verified, and without touching your mouse.** A new headless seam
    (`PDFCER_DIAG_INVOKE`) lets a diagnostic run press a ribbon command in an
    invisible window, so this was proved on the machine while you were using
    it. The evidence:

    ```
    diag-invoke id=file.print
    print-open printers=12 selected=8
    viewport-inner id="4206" rect=[[-3944 -3921] - [-3144 -3301]]
    ui-rect name=print.paper rect=[[393.9 480.0] - [601.4 504.0]] viewport="4206"
    ```

    An 800 x 620 OS window of its own, with its controls positioned inside it.

    ★★ **AND THE OTHER THIRTEEN SHIPPED 2026-08-21.** About, Render
    diagnostics, Export to DXF, Insert image, Insert pages, New document,
    Recognise text, Apply redactions, Set scale, Keyboard shortcuts, Settings,
    the note editor and the unsaved-changes question. Every one has a title
    bar, a taskbar entry and can go on the second monitor.

    Three of them are worth naming because the window is the *feature*, not a
    tidy-up: **Apply redactions** lists what will be removed and you could not
    check it against the page it was covering; **Render diagnostics** is read
    while zooming the very document it describes; and the **unsaved-changes**
    question appears in answer to a close, when you have already looked away —
    a modal question hidden behind the main window with no taskbar entry is
    the classic *"the program has frozen"*.

    ★ **Nine of the thirteen had no size to convert.** They were content-sized
    windows, so no number for how big they are existed anywhere — and a
    guessed size that is too small does not look wrong, it clips the bottom
    row, which on a confirmation is the row with the buttons on it. So a
    dialog now measures its own body and grows the window to fit; the declared
    size is an opening bid. The first version of that grew About from 560 px
    to 1,624 px in a few frames, and a driven launch caught it the same hour.

    **Driven:** eight of the thirteen, one launch each, no mouse touched.
    Every one opened at its declared size and none needed to grow.

    ★★ **AND CONVERTING THEM BROKE THE VERIFICATION, WHICH IS WORTH SAYING
    OUT LOUD.** Six driven checks failed and six more skipped on the next full
    run — every one of them clicking hundreds of pixels away from the control
    it named, with no error anywhere, because a dialog in its own window has
    its own coordinates and the harness was still adding the application
    window's. All six are fixed and the harness now knows the program has more
    than one window.

    ★★★ **One of them was a real defect that had shipped: every dialog drew
    on a BLACK background.** Dark text on near-black, legible only as an
    outline. Nothing caught it — the window opened, every control was where it
    said it was, and the driven check for *"a dialog opens in its own OS
    window"* passed on all eight. **A screenshot showed a black rectangle.**
    That is the standing rule earning its place again: a rendering defect has
    exactly one oracle and it is a picture.

    **NOT VERIFIED**, named rather than implied: three of the five reachable
    only by a gesture — Insert pages, Set scale and the unsaved-changes
    question. The note editor is now driven, and see the row below for what
    that found.

    ★★ **AND THE LAST ROW OF THIS ITEM IS CLOSED TOO, 2026-08-21: a dialog
    can no longer fall behind the main window.** It stood open because the
    toolkit has no way to say *"this window belongs to that one"* — thirty
    options in its window builder and not one of them is an owner. pdfcer now
    tells Windows directly, which is what every native dialog on your machine
    already does and why none of them has this problem. Confirmed on every
    dialog that opens: `dialog-owned owned=true`.

    Making it always-on-top instead stays refused, and the reason is worth
    keeping: it would break the driven checks in a way that produces confident
    wrong bug reports, and we have paid for one of those already today.
13. ~~**Enter is not the affirmative default**~~ — **PRINT SHIPPED
    2026-08-20**, and the pair is the host's, so every dialog converted on
    2026-08-21 inherited it. Type a page range, press Enter, it prints. Print is drawn
    filled in the theme's own accent so you can see what Enter will do before
    you press it, and Escape now closes the dialog exactly as the X does.

    The pair is drawn by the host, not by the dialog, so no future dialog can
    implement two of the three obligations and forget the third.

    **Known limit, named rather than found:** Enter is suppressed while a text
    field has focus, because the toolkit reports *"a text field has focus"*
    without saying whether it is multi-line — and a multi-line field must keep
    the ability to type a newline. So in a dialog whose last control is a
    one-line box, you may need to click out of it first. The fix is per-field.
13b. ~~⚠ **A note box may not take your typing until you click it.**~~ —
    **WITHDRAWN the same session, 2026-08-21. There was no such defect.**

    It was written up here in good faith and it was wrong, so the whole of it
    is left standing rather than deleted: this file is a record of what you
    were told, and a retraction that hides what it retracts is worth less than
    the mistake.

    **What was reported:** drag out a Text box or Sticky note, type without
    clicking the field, and the words go nowhere — with no message, because
    Accept is only enabled once the field has something in it.

    **What was actually true:** *the test* was clicking Accept through the
    main window's coordinates while the dialog had its own. You type, the
    dialog takes the characters correctly, and then the click that should
    commit them lands on the page instead. Converting that one line made the
    check pass. Typing into an unclicked note box has worked the whole time,
    and is now driven end to end — the check authors an annotation and
    confirms it changed the page.

    ★★ **The lesson, because it nearly cost a lot more.** Chasing the wrong
    culprit, the program was changed four times to hold the keyboard harder,
    and **each change appeared to help**: the dialog visibly held focus while
    the new code was asking for it and lost it the moment it stopped. That was
    real, repeatable, and completely beside the point. *A measurement that
    moves when you turn a knob is not proof the knob is the subject.*

    Two of those four changes were kept because they are right on their own
    terms — see item 12's *"the dialog can fall behind"* line, now closed —
    and the two that were only ever tuning were undone.

14. **PARTIAL, improved 2026-08-21.** Every dialog comes back where you left
    it, and it now **survives closing and reopening** — the position moved out
    of the dialog and into the application's own memory on the same pass that
    converted the other thirteen. It still does not survive a restart: a remembered position has to be checked against your
    current monitors, and a dialog that opens on a screen you have unplugged is
    worse than one that opens where Windows puts it. Tab order and modal
    focus-trapping are still untested.

### And two fixed on the spot, because writing the row exposed them

- The vertex drag converted screen→canvas **twice**, so it tracked at `1/zoom`
  and sat off by the scroll origin — *"the distance from the pointer varies as
  you move it."* Fixed.
- It also assigned the pointer straight to the vertex, so grabbing a handle
  slightly off-centre teleported the corner under the cursor before you had
  moved it. Now it moves by the delta and the grab point is preserved.

## O8 — **A Save button.** Not Save As. Save.

**Asked:** 2026-08-20 — *"can I please have a save button like every other
program in existence has? We're on week two of this and just have a save as
button."*
**Status:** **SHIPPED 2026-08-20 and DRIVEN.** Awaiting your verdict.

`Ctrl+S` saves over the file you opened. `Ctrl+Shift+S` is Save-a-copy. The
quick-access toolbar's second slot is Save now, and it carries the disk glyph;
Save-a-copy renders as text.

It writes to a temporary beside your file and then renames, so a crash or a full
disk in the middle leaves your original untouched rather than half-written. And
because pdfcer saves incrementally, the previous version of the document stays
inside the file — nothing is thrown away by pressing it.

Driven: `save_writes_over_the_file_you_opened` — the file grew 140,660 →
141,423 bytes, no temporary was left behind, and it still reads as a page tree.

★ The blocker that kept this out for a fortnight said *"in-place save is blocked
on autosave and crash recovery"*. That was aimed at the wrong hazard: pdfcer's
incremental format already WAS the crash recovery. What was actually unsafe was
the write, and that has a three-line answer nobody had written because nobody
was asking. Third time in two days that a blocker turned out to be a question
asked wrongly.

There is no defence for how long it took. `Ctrl+S` is bound to `file.save_copy`, which asks where to
put it, every time. Overwrite-in-place was written down as *"an operator scope
decision"* and then sat there being nobody's problem — which is the same failure
as `Ctrl+P` never being bound and the caret never having an index: **the basics
were never audited as basics**, because every test asked "does the thing I built
work?" and nothing asked "does the thing everyone expects exist?".

## O9 — A **length** tool: the perimeter tool that never closes

**Asked:** 2026-08-20 — *"add a length tool that works like the perimeter tool
without needing to close the profile."*
**Status:** **SHIPPED 2026-08-20 and DRIVEN.** Awaiting your verdict.

Measure ▸ **Length**, beside Perimeter. Same gesture, same snapping, same
preview, same running total, same group scale — it just never closes. Clicking
the first point again adds a point there, because a run of cable that loops back
is still a run of cable. Double-click the last point to finish.

It is a separate control rather than a checkbox on Perimeter because "Perimeter"
says closed, and nobody measuring a pipe run would go looking inside it.

Driven, in the same check as Perimeter and deliberately so: what is worth
proving about Length is a *negative* relative to Perimeter — that the
first-vertex click does **not** close it — and a negative is only meaningful
beside the positive it differs from. Two separate checks would let the pair
drift into being one tool.

```
★ the ring closed and the dimension reached the engine   (Perimeter)
★ the Length tool took all 5 clicks as vertices          (Length)
```

## O10 — Neither measuring tool previews while you trace

**Asked:** 2026-08-20 — *"both these tools need a preview just like the measure
tool has."*
**Status:** **FIXED 2026-08-20**, awaiting your verdict.

The preview arm was written and unit-tested for its segments, and it was
**unreachable**: `super::preview` returns early on
`MeasureState::gesture_in_progress()`, and that function had not learned about
the perimeter's pick. So the tool drew nothing at all while tracing.

It is the failure class this project keeps meeting — every part correct, the
*join* unobserved — and the driven check could not see it, because that check
asserts on the trace and a preview is pixels. `every_pick_kind_is_counted_as_a_gesture`
is now the guard.

## O11 — Move, resize and rotate a placed image on the canvas

**Asked:** 2026-08-20 — *"there was no way to reposition, resize, or rotate it
on the screen. Can I please please please have that too?"*
**Status:** **MOVE AND RESIZE SHIPPED 2026-08-20 AND DRIVEN. ROTATE IS NOT — see
below.** Awaiting your verdict.

Select a picture and drag it: it moves. Grab a corner and drag: it resizes.
Select several things at once — a picture, a box and a line — and one corner
drag resizes all three about the same point, as one undo entry.

Three refusals you may have seen are gone with it:

- *"pdfcer cannot resize text or pictures — only shapes drawn out of lines and
  curves."*
- *"pdfcer resizes one shape at a time."*
- *"This shape has no corners to move."*

**Driven, on your own drawing, 2026-08-20:**

```
resize_scales_a_shape           PASS — through transform_objects
geometry_fields_resize_a_shape  PASS — the typed W/H route, same function
shift_constrains_a_resize       PASS — and Shift keeps the proportions
```

### ★ ROTATE IS NOT BUILT, and it is a shell gap now rather than an engine one


> ⚠️ **CORRECTION, 2026-08-21. THE ROTATE GRIP EXISTS AND HAS SINCE
> 2026-08-20.** Commit `560280a`, *"The ninth grip - you can turn things now,
> which was the third word all along"*, added `canvas/rotating.rs` (424 lines),
> `Grip::Rotate` (`canvas/handles.rs:175`), its hit test
> (`handles.rs:412`, ahead of the eight resize grips), its painter
> (`overlay.rs:222` via `draw_grips`), a rotate **ghost**
> (`overlay.rs:612`), Shift-snapping to 15°, and the commit through
> `transform_objects` (`canvas/rotating.rs:273`).
>
> The paragraph below was true when written and was **never updated**. It was
> then re-quoted, in good faith, into two rows written on 2026-08-21 —
> propagating a false claim rather than checking it against the source. That
> is the failure this file exists to prevent, committed inside this file.

~~The verb rotates. **There is no rotate handle on the canvas to reach it with.**~~
That is `O14` row 5, and it stopped being blocked tonight — it needs a ninth
grip above the selection box, a drag that measures an angle rather than a
distance, and a preview. Say the word and it is the next thing.

### And one more thing that is not built, named rather than left to be found

A picture whose own placement matrix is degenerate cannot be transformed at
all, and the engine says so — *"do not offer a handle"*. pdfcer currently offers
one and you would find out by dragging it. The preflight that would grey it
needs a page decomposition cached per selection (**~4 seconds** on your
benchmark drawing in a debug build), which is a piece of work rather than a
line. Rare: it needs a producer to have emitted a collapsed matrix.

## O12 — Move text after placing it

**Asked:** 2026-08-20 — *"can I please please please have the capability to move
the text after?"*
**Status:** **SHIPPED 2026-08-20.** Select the text and drag it. Same verb as
O11, exactly as asked for — a placed image and a placed text run are the same
shape in a content stream, so they got one verb rather than two.

★ **A move still uses the lighter verb where it can**, and that is deliberate
rather than a leftover: for a selection made only of shapes, pdfcer rewrites the
coordinates in place and adds nothing to the file. The general verb wraps each
object in three extra operators every time you nudge it, and you nudge things
dozens of times in a file you then send to somebody. Shapes take the light
path; anything else takes the general one.

**NOT YET DRIVEN** on a text object specifically — the driven checks aim at a
shape, because that is what the fixture's `--doc-point` names. The verb is the
same one three passing checks exercise.

## O13 — Insert image does not appear until you save and reopen

**Asked:** 2026-08-20 — *"I tried a new document and inserted an image. Nothing
appeared on screen or in the tree, but after saving and reopening the image was
there."*
**Status:** **FIXED 2026-08-20 and DRIVEN.** Awaiting your verdict.

Your report split O4 in two, and this half was mine.

After every edit the shell re-walks the page tree and then compared *(page
object id, rotation)* against what it had. If nothing there moved it returned
early, with a comment saying *"the page vector already describes the
document"*. **That is false.** A `Page` is not an id — it is a resolved page,
with its `/Contents` and `/Resources` in it. `add_image` turns `/Contents` from
a stream into an array and adds an `/XObject`; the page's id does not move; the
early return fired; the canvas and the Objects panel went on reading a page as
it was before the edit.

Which is why saving and reopening worked: the bytes were right the whole time.

Markup never showed it (annotations are read from the session, not that vector)
and moving an object never showed it (that rewrites a stream *in place*, so the
stale reference still resolves to the right object). `add_image` is the first
verb that changes what `/Contents` **is**, so the bug had been there since it
was written and had never been reachable in a way anyone could see.

Driven, on your own JPEG, into a new document from the template:

```
made a blank document first
placed: add-image page=0 n=1 — 839 dpi
the page repainted: 118,580 of 256,878 pixels changed
```

and the Objects panel now reads *"1 object(s) on this page — 1 image(s). #0
Image · 6247 × 5010 px"*.

**O4 is still open** — that one is the engine corrupting `/Contents` when it is
already an indirect array, which is what your CAD sheets use, and it produces a
file pdfcer cannot reopen at all. Filed and unchanged.


## O1 — Editing text on the canvas, and editing text in a text box

**Asked:** repeatedly; restated 2026-08-20 — *"Still no editing text on top of
the canvas. Or editing text on a text box."*
**Status:** OPEN. Under investigation 2026-08-20; nothing claimed.

Two distinct things and I have been conflating them, which is probably part of
why it keeps coming back:

- **(a) Editing text that is already on the page** — click a run of existing
  page text, get a caret, retype it.
- **(b) Editing the text inside a text box you have added** — a `/FreeText`
  annotation, or a text object this shell authored: double-click it, get a
  caret in it, retype.

**(a) — driven 2026-08-20 on your own CAD drawing. It is reproduced, and it is
not the shell.** `text_edit_on_a_real_drawing`:

```
text-edit-caret kind=Edit page=0 run=44 len=1     ← the caret lands on real text
text-edit-typing draft=true text_events=1 len=2   ← keystrokes reach the draft
text-edit-typing draft=true text_events=1 len=3
text-edit-plan page=0 run=44 disposition=Pin reason=Rotated pinned=true
edit-text-refused page=0 n=1
  detail=text to edit ("p") was not found in an editable run on the page
```

So the tool arms, the caret lands on the right run, the typing arrives, the plan
is built and the commit reaches the engine — **and `pdfcer-core` refuses it.**
From your chair that is precisely "the tool responds and the page does not
change".

Root cause under investigation 2026-08-20. It will be either an engine gap (a
request, filed) or a wrong call on my side (a fix). Either way this row records
the answer.

★ And a second defect found in the process, on my side: the driven check for
this was reporting *"THE COMMIT NEVER REACHED THE ENGINE"* — which was **false**.
`edit-text-refused` is not `edit-text`, so a check asserting on the absence of a
line produced a confident, specific, wrong accusation about working code. Fixed:
a refusal is now asked about first and quoted verbatim.

**(c) — the caret cannot be moved inside a run. FIXED 2026-08-20**, and it was
worse than it looked: there was **no caret index at all**. The draft appended
text and Backspace popped the last character, so the painter drew its line at
the right edge of the run's box because that is the only position an
append-only draft has.

Now: a real caret. Click part-way into a run and it lands at that character
(measured against the run's own glyph advances); Left, Right, Home, End,
Ctrl+Left and Ctrl+Right move it; Delete eats forwards; typing and Backspace
act at the caret. Unit-tested, including your `SHEET 1 OF 4` case as a named
test. **NOT yet driven** — the harness was blocked by the on-screen keyboard.

**No selection yet** — no Shift+arrow, no Ctrl+A, no drag-select inside a
draft. That is a second feature and it is row **O7** rather than an implied gap.

**(a) — ★★★ IT WORKS. 2026-08-20, and it is the 99 % case.**

You can now click a label, a title-block field or a *pdf dimension* callout on
a CAD sheet, get a caret, and retype it. That text lives inside what the format
calls a form XObject — a block the drawing program placed — and until this
evening pdfcer could read it and not write it.

Measured on your benchmark drawing, which is why this mattered more than
anything else in the queue: **1,696 show operators of real drawing text inside
the block, against 3,007 metadata glyphs in the page's own stream.** Your own
words when you saw the split: *"I need that editing capability as it is 99% of
the text I will want to edit."* The engine escalated the work ahead of the
move/resize verbs on the strength of that sentence.

### ★ One thing you need to know, and it is not a pdfcer limitation

**A drawing program may place ONE copy of a block and paint it on six sheets.**
That is what the construct is *for* — the standard names a CAD system's
standard component as the illustration — and nothing in the format binds a
block to a page. So when you edit text inside a shared one, **it changes on
every sheet it appears on**, because there is exactly one copy of those letters
in the file.

pdfcer cannot make that not be true, so it tells you: after an edit that touched
shared content, the status row says *"SHARED CONTENT: this text is drawn from
shared content that appears in N place(s) on M page(s)"*. It is deliberately
silent on the ordinary case — a warning that fires every time is one nobody
reads, and this one is meant to make you stop.

**Nothing is drawn on the page.** No badge, no tint, no flag. Your own finding
about the old GUI's red-flagging stands.

**Not built, and named rather than left implied:** you are told *after* the
edit, not before you type. Telling you at the caret means asking the document
how many places paint that block, which is a walk of the whole file — cheap
once, not cheap on every click on text — so it needs a cache that does not
exist yet. Undo puts it all back in one press in the meantime. Say the word and
it moves up.

### What it cost on this side, and why it was one deleted line

The shell had a guard refusing the caret. When it was written, the request that
went with it said:

> *"my shell encodes a fact about your surgery's internals. The day form
> editing lands, my guard silently keeps refusing until I notice and delete
> it."*

So the engine published a query — *"is this run editable?"* — and the shell
asked it instead of modelling the answer. When the capability landed, the query
started answering *yes*, a deprecation warning pointed at the single line to
remove, and that was the whole job. A hand-rolled guard would have gone on
refusing 99 % of your text until somebody noticed.

★ **One thing added beyond deleting the guard.** The shell now names *which*
buffer it measured when it commits, rather than letting the engine search. On
your sheets the page's own stream holds 3,007 single-character operators, so
letting a byte offset be tried there first is a dense field of near-misses —
an edit that could succeed on the wrong glyph with no error anywhere.

**Driven:** `text_edit_on_a_real_drawing` now asserts the commit named a form,
and the old "this is an absent capability" skip has been **inverted** — if a
build ever refuses with the old reason again, that check fails loudly.
**NOT YET RUN.**

What remains under this heading is the refusal for text that has no letters
behind it at all — an `/ActualText` description the producer supplied instead
of glyphs. That one is genuinely unreachable and always was, and it now says
so in its own words rather than borrowing the form sentence.

**(b) — not yet driven.** Nothing claimed.

## O2 — Cut / copy / paste of PAGE CONTENT (`Ctrl+X` / `Ctrl+C` / `Ctrl+V`)

**Asked:** first week, and repeatedly since. Restated 2026-08-20: *"can you get
cut copy and paste working for objects I select on the canvas?"*
**Scope set by you, 2026-08-20:** *"oh I might want all cases so we shouldn't be
restrictive in our ask."*
**Status:** **SHIPPED 2026-08-20 AND DRIVEN.** Awaiting your verdict.

Select a line, a shape or a piece of text on the page and press **Ctrl+C**, then
**Ctrl+V**. It lands 10 pt down and right so you can see it is a copy, or in
place if you paste onto a different page. **Ctrl+X** cuts, as one undo.

Driven on your own drawing: a 108 KB clip out, one object back in.

### ★★★ And Ctrl+C had never once reached the keyboard map — that is why you kept reporting it

You said *"still no ctrl+c, ctrl+v, ctrl+x"* twice. On 2026-08-20 they were
bound, which was necessary and **not sufficient**: the toolkit intercepts those
three chords and converts them into its own clipboard events **before** the
keystroke reaches anything pdfcer can see. So the binding existed, every test
agreed it existed, the menu showed it next to the command — and the key did
nothing, for ever.

★ **Ctrl+V was worse.** The toolkit only raises a paste event if the *Windows*
clipboard already holds some text. With it empty, the keystroke vanished
completely — so whether paste worked depended on **whether you had recently
copied text in another program**. Not random, not reproducible, and nothing to
do with pdfcer.

That is why copying now also leaves a sentence on the Windows clipboard —
*"1 object copied from pdfcer. Paste it back into pdfcer to place it."* It is what
makes the key arrive, and if you paste it into an email by accident it reads as
an explanation rather than as garbage.

### What works today

- **Page content — a path, a line, a block of drawing, a picture, text:** cut,
  copy and paste, in any mixture, as one undo entry.
- Markup and comments: cut, copy and paste.
- Swept text: copy to the system clipboard.

### Still open, and named rather than left as a silence

- **Across two pdfcer windows.** Within one window it is lossless. Between two
  processes it needs the clip registered under a private Windows clipboard
  format, which is a call this shell does not make yet.
- **Copying to another program** — Illustrator, SolidWorks — needs the selection
  rendered as a standalone one-page PDF, which the engine has filed separately
  and deliberately did *not* fold into the same bytes: a one-page PDF cannot
  carry which byte range was which object, so re-deriving it on the way back in
  would make a pdfcer→pdfcer paste worse than a pdfcer→Illustrator one.
- **Dimensions and form fields** are annotations rather than page content, so
  these verbs cannot reach them at all. Filed.

### ★ And our reading of the engine was right in a way that mattered, and wrong in one place that was the whole job

We scoped the ask as *"expose the copy engine you already have at object
granularity"*, on the strength of a function that already copies object graphs
with every reference remapped. That was correct. What it misses is that a
drawing's content is not an object graph at all — it is **bytes inside a page's
content stream**, and those bytes name their fonts and images **by a nickname
that is local to that page**. On another page, `/F1` is a different font.

So a naive copy would have pasted the right letters in the wrong typeface, and
**nothing would have errored**. The engine built the name-rebinding half; our
reading identified the prerequisite. Worth recording for the next request scoped
that way.

### I nearly asked for a third of it

I was going to ask for `duplicate_objects` alone, on the argument that Ctrl+V in
one document decomposes into *duplicate + offset* and `move_objects` already
exists. That is true and it would have covered same-document duplication only —
not pasting into the other tab, not the system clipboard, not dimensions or form
fields. You stopped that, and the filed request is the whole capability:

1. **A portable object payload** — content *and* the resources it depends on.
   Kind-agnostic, so a mixed selection works; takes a `Matrix`, so paste-in-place,
   paste-offset, paste-scaled and paste-rotated are one verb; with a preflight so
   the menu item can be greyed rather than discovering the refusal by pressing.
2. **Serialisable**, which is what makes cross-document and cross-session paste
   fall out instead of being a second feature.
3. **The system clipboard** — a pdfcer-private format so pdfcer→pdfcer is lossless,
   plus a standalone PDF and an image so SolidWorks and your CAD packages can
   read it. Registering those is mine; I need the bytes from them.
4. **Cut as one undo entry**, or Ctrl+X then Ctrl+Z gives your objects back and
   leaves the clipboard changed.
5. **Dimensions and form fields refuse loudly** rather than pasting something
   subtly broken — a pasted dimension needs a sidecar record and a group, a
   pasted field needs a name that does not collide. Silent partial success is
   the one outcome I cannot work with.

**Reading vector data IN from other programs** (paste from Illustrator) is
explicitly *not* in the ask — that is foreign PDF/EMF/SVG parsing and a much
larger job. Named so it is a decision rather than an omission. Say the word if
you want it.

### ★ The finding that makes this smaller than it looks

`EditSession::import_object` already exists, privately: a recursive
cross-document object-graph copy with fresh object numbers, every reference
remapped, cycles handled, and stream payloads re-staged. It is what
`insert_pages` and `merge_document` use to bring pages across **with their
fonts, patterns, images and soft masks intact**.

That is the entire difficulty of pasting page content, already solved. The ask
is not "build a copy engine" — it is "expose the one you have at object
granularity."

**Sits below the transform verbs in priority**, deliberately: an operator who
can place an image and not move it is worse off than one who cannot copy a path.
The first is a feature that looks broken; the second is one that is absent.

## O3 — Perimeter measuring tool

**Asked:** 2026-08-20 — click around to make a shape, sum the segment lengths
into one dimension; right-click to add segments; drag the endpoints to adjust
the shape; all the scaling options of the other dimension tools.
**Status:** OPEN — **and no longer blocked.** Filed 2026-08-20; the engine
shipped the whole thing the same day (commits `9940acf`, `ae06440`): a
`Perimeter` kind carrying its vertices and an open/closed flag, verbs to move,
insert and remove a vertex, and a preflight so a right-click menu can be greyed
correctly rather than by guessing.

The value goes through the same group path as every other dimension, so scale,
unit, precision, drafting standard and layer come free. The label sits at the
vertex centroid, so it drifts smoothly when you drag a corner instead of
teleporting across the shape.

**SHIPPED 2026-08-20 and DRIVEN.** Measure ▸ Perimeter arms a tool; click
around the shape; the polyline previews as you go with a rubber band to the
cursor; the Tool panel shows the running total in the group's own units;
clicking the first point closes the ring and commits; double-click finishes an
open path; `measure.finish` on the ribbon does the same.

It is a real dimension in a real group, so the scale, unit, number format,
drafting standard, layer and style cascade all apply exactly as they do to a
linear dimension — which is the half you asked about specifically.

Driven on the benchmark sheet by `measure_perimeter_traces_and_closes`:

```
Measure ▸ Perimeter armed the tool
four vertices taken; running total: -0.0 → 378.8 → 655.8 → 1023.6
★ the ring closed and the dimension reached the engine: add-dimension page=0 n=1
```

The first driven run FAILED, correctly: the ring would not close, by eight
canvas units. The first vertex is stored where the *snap* put it, so the closing
click was being measured against a target that had already moved — and it was
being measured with the click tolerance instead of the snap tolerance. Fixed.

**Dragging the endpoints SHIPPED 2026-08-20 and is driven.** Select a perimeter
and its corners get handles; drag one and the shape follows, previewed as you
go. The status row reports what it cost you:

```
That corner changed the measurement: 621.45 pt is now 1226.84 pt.
```

Both numbers, because you can see the new one and cannot see the old one — the
geometry it came from is gone. Silent when the number did not move, so the line
means something when it appears.

You can also drag a perimeter's **number** now, the same way as a linear
dimension's — and more freely, because a perimeter's label is anchored in page
axes rather than to an axis, so it lands where you drop it instead of being
flattened onto a line.

**Still open in this row:** right-click a segment to add a point, right-click a
point to remove one. Both engine verbs exist, and so does the preflight that
tells a menu whether to grey the item, so this is shell work only.

## O4 — Insert image does nothing

**Asked:** 2026-08-19, restated 2026-08-20 — *"No it always hasn't worked."*
**Status:** **BOTH CAUSES FIXED 2026-08-20.** Awaiting your verdict.

You were right, twice, and it was never a misunderstanding. There were two
separate defects sitting on top of each other:

**The engine's.** `add_image` corrupted the page's `/Contents` whenever it was
an indirect reference to an array — which is what every CAD-exported sheet uses.
The verb returned success, the status bar reported the resolution, the picture
was not on the page, and **the saved file could not be reopened by pdfcer at
all**. Filed with an eight-line repro; fixed in `Pass 111.0`. Files already
damaged by an older build now open, render, and say so through a counted
disclosure rather than being silently patched.

Verified here, headlessly, on your benchmark drawing:

```
pages BEFORE: Ok(1)
pages AFTER:  Ok(1)     ← was Err("/Contents is neither a stream nor an array")
reloaded:     Ok(1)
```

**Mine.** The shell re-walked the page tree after every edit and returned early
when the page's object id had not moved — with a comment claiming *"the page
vector already describes the document"*. False: a `Page` carries its `/Contents`
and `/Resources`, and `add_image` changes what `/Contents` **is**. So the canvas
and the object tree went on reading the page as it was before the edit. That is
row O13, fixed the same day and driven.

Between them they explain everything you reported, including why saving and
reopening showed the picture: the bytes were right the whole time.

## O7 — Selecting text inside a draft

**Asked:** not by you. Recorded 2026-08-20 because it is the obvious next thing
after the caret and I would rather name the gap than leave it implied.

Shift+arrow, Ctrl+A, and dragging across a draft to select part of it, so that
typing replaces the selection. Not started.

## Q1 — ANSWERED 2026-08-21: the rest of the line MOVES ALONG

> *"It should move along."*

**Settled. Nothing changes** — `FollowerDisposition::Reflow` was already the
default and stays it. The three automatic exceptions stay too: rotated text,
right-aligned or centred text, and a line drawn as several separate pieces are
each pinned, because in those three cases "moving along" would move the tail the
wrong way, off its margin, or drag a neighbour that is not part of your edit.

Recorded rather than deleted, because the engine will ask again the next time it
touches reflow and the answer should not have to be re-derived.

The question, kept for that reason:

When you retype a piece of text and the new words are longer, pdfcer has to
decide what happens to whatever is drawn after it on the same line. Two answers:

- **Push it along** (today's default). Right for a paragraph; the sentence stays
  a sentence.
- **Leave it exactly where it is**, and absorb the difference invisibly. Right
  for a drawing, where a label beside a label is not a sentence and moving one
  is a change nobody asked for.

pdfcer already picks *leave it* automatically in three cases: rotated text,
right-aligned or centred text, and a line drawn as several separate pieces —
which is most of a CAD title block. The question is whether **drawing content
should default to leaving it** rather than relying on those three to catch it.

The engine's own view: *"`Pin` is the safe posture for drawing content … worth
offering on a per-edit basis rather than as a global preference, since it is
right for a CAD label and wrong for a paragraph."*

**Nothing is built on this and nothing will be until you answer.** Recorded here
rather than decided quietly, because it changes what happens to your drawings
when you type.

## O5 — Horizontal / vertical dimension constraint, from a drop-down

**Asked:** 2026-08-20.
**Status:** OPEN, not started. `LinearPick::constraint` exists in the shell and
is never written outside tests — there is no control for it. No engine work
needed.

## O6 — The scale ratio field follows the dimension you set

**Asked:** 2026-08-20 — *"when I set the dimension the editable ratio one shown
should change to match it."*
**Status:** OPEN, not started.

---

# SHIPPED — awaiting your verdict

Rows here are built, gated and driven. **They stay here until you have used
them and said so**, then they move to CLOSED with the date you confirmed.

## S1 — Move a placed dimension, with a live preview

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `469d4d7`.
Press inside a selected ce dimension and drag: the dimension line follows,
previewed from the same function a committed dimension is drawn from, and the
release commits `place_dimension`. The measured points never move, so the
printed number cannot change. Linear dimensions only — angular refuses at the
press rather than starting a drag that could not finish.
**Verified:** unit tests only (4). **NOT yet driven through the harness.**

## S2 — The measure sidebar no longer hides half its controls

**Asked:** 2026-08-20. **Shipped:** 2026-08-20, commit `c2de963`.
The group list was a four-column grid 209 pt wider than its own column, clipped
with no scrollbar in that axis. Now a block per group, and the panel measures
its own overflow so it cannot come back quietly.
**Verified:** `no_row_in_this_panel_outruns_a_narrow_dock`, which failed at
209 pt before the change. **NOT yet looked at on screen.**

## S3 — `Ctrl+P` opens Print

**Asked:** 2026-08-20. **Shipped:** 2026-08-20.
It had never been bound. Print was on the ribbon, in the QAT and in a menu, so
every surface that lists commands showed it and only the keyboard did not.
`the_keymap_offers_the_chords_a_document_application_must` now asserts the whole
list of universal chords rather than this one line.
**Verified:** unit test. **NOT yet driven.**

## S4 — Imperial sheet sizes

**Asked:** 2026-08-19. **Shipped:** 2026-08-19, commit `815036a`.

## S5 — Multiple documents, page drag between them, tab reorder

**Asked:** 2026-08-19/20. **Shipped:** 2026-08-20.
**Verified:** driven — `document_tabs`, `page_drag_between_documents`,
`tab_reorder`.

---

# CLOSED

*(Nothing yet. A row lands here only when you have said it works.)*
