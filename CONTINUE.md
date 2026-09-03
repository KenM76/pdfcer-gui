# CONTINUE - handoff

## 2026-09-03 (evening) - his print dialog: four defects, and the scrollbars alone took four fixes

### What he reported

> *"two scroll bars in the pop up window that won't go away no matter how, and
> it doesn't close after I hit the print button that is so far off in the corner
> it is touching the edge the window, and it looks greyed out as though it
> doesn't do anything even when I hit print - but it is working, so after many
> clicks I checked the printer and of course there was a dozen jobs there."*

**Four separate causes.** Reproduced offscreen with `PDFCER_DIAG_INVOKE=file.print`
and photographed at five window sizes BEFORE touching anything; every symptom is
in the captures. Filed as O111, now closed.

### ★★★ The scrollbars needed FOUR fixes, and each wrong answer read as correct

In the order they were found, each one revealing the next:

| # | cause | how it was found |
|---|---|---|
| a | content forced to `available_width` measured **outside** the scroll area | reading the source |
| b | `auto_shrink([false, false])`, which **defines** content to be at least the pre-bar viewport | tracing egui's own `content_size` vs `inner_rect` |
| c | the two `item_spacing` gaps `horizontal_top` inserts between three children | same trace, `content_w=1260` against `outer_w=1276` with a bar still drawn |
| d | the preview's control strip, **379.9 pt laid out in a 340 pt column** | a second trace line added to measure it |

Each raised a horizontal bar; a horizontal bar consumes height; that raised a
vertical bar; the vertical bar consumed width, which kept the horizontal one.
**The two bars were each other's cause**, which is why resizing never helped.

★★ **The failure was INVERTED**, which walking the size series found and one
screenshot would not have: bars at 1000x760 and 1300x900 where nothing needed
scrolling, and **no bar at all** at 700x520 where the whole Paper section was
clipped and unreachable.

★★ **(d) had overflowed since the day it was written** — hidden by the forced
content width, and visible spilling past the divider in the very first capture
if anybody had looked.

★ **The first attempt at (c) was worse than the defect.** `item_spacing.x = 0`
removed the bar and inherited into every child, so the radio rows lost their
spacing: *"Subset ●Every page ○Odd only"*. Visible in the next capture. **Fixing
a layout defect by removing the layout is how one defect becomes several.**

★ **(d) is fixed with `horizontal_wrapped`, not a wider minimum.** A minimum
would be a constant asserting how wide seven buttons are — which depends on the
preset's font and button padding, so right in one theme and wrong in another. A
wrapped row is bounded **by construction**.

⇒ The rule now held by `print/layout.rs`: **every width and height is derived
from the space OUTSIDE the scroll area and from constants. Nothing is measured
from inside it.**

### ★★★ The button looked disabled, and it is defect D2 for the third time

`Host::buttons` filled the affirmative with `visuals.selection.bg_fill`, whose
comment congratulated itself on being *"never a literal, which
`check-theme-colors.sh` enforces"*. Every clause true. That role is
`rgba(90,140,220,70)` — a **27 % wash** whose real job is tinting canvas
selection — and over a light panel it composites **paler than an ordinary
button's opaque fill**. The default action rendered less solid than Cancel.

`Theme::accent_pair` is now the one spelling of *"paint this as the emphasised
action"*. **A gate that forbids invented values cannot enforce correct roles**;
a purpose-named pair is the mechanism that can.

### The other two, briefly

- **It would not close.** By construction — `show` returned
  `!frame.closed && !close_requested`. A **successful** print now records its
  receipt on the disclosure row and closes; a **failed** one does not, because
  the driver's words and the settings are what he needs next.
- **Everything touched the window edge, in all fourteen dialogs.** A viewport
  callback's root `Ui` has no `CentralPanel`, so no margin. `Host::BODY_MARGIN_PTS`,
  applied once in the host — and deliberately kept OUT of `Host::fit`'s
  measurement, because a margin fed back into a size grows the window every
  frame.

### ★★ The gap that let all four ship — third of its kind

`dialogs_open_in_their_own_window` sweeps from a **hand-written list, and Print
was not in it.** Its header rationalised the omission in prose. Print is now the
first entry, with the reason.

And `the_body_width_holds_both_columns` was **green throughout**, asserting a
relationship between our own constants while he was looking at two scrollbars.
A bar appears when content exceeds **egui's** viewport, which does not exist
until a frame is laid out. Retired; replaced by three relationships that ARE
ours plus a driven check that reads egui's numbers from a running process.

### Requests filed and their state

| | |
|---|---|
| **O111** | ✅ closed — the four defects above |
| **O112** | ◑ **half** — the preview is draggable (splitter, floors, double-click to reset, width survives a resize). The **pop-out window is NOT started**; it is a second `Host` keyed `print-preview` and `Frame::closed` is already the return path |
| **O113** | ⬜ **not started** — the clipping hatch should cover only what actually falls outside the printable area. His 1:1 drawings overhang by empty paper, so the hatch cries wolf on every sheet. Needs either an engine verb for a page's ink extent, or sampling the preview raster; **decide deliberately**, the second is a proxy |

### Not verified, and named rather than implied

**The window visibly disappearing after a REAL print.** The decision is unit
tested (`commit_notes`, extracted for exactly that reason — proving a window
closes must not require a job on his printer), but nothing has driven a spool.
Closing that needs a check that prints to a file device; `Microsoft Print to
PDF` is on this machine. Worth building. Not built.

### State

Engine **v0.28.0 at `e27c3b4`** — it moved under the packager **twice** in one
afternoon, so treat that as the standing hazard. 2,886 tests, 0 failing; 23 of
23 gates, 0 skipped. `print/mod.rs` hit R2 at 1,886 lines, so geometry moved to
`print/layout.rs`. OneDrive: **`pdfcer-gui2` is the new build**, `pdfcer-gui1`
holds the 14:13 one from before these fixes.

---

## 2026-09-03 (afternoon) - v0.5.0 released, and the rename had blinded four instruments

### The release

`v0.5.0` on **`KenM76/pdfcer-gui`** - the first release in the new repository;
`KenM76/pdfceGUI` is archived and holds v0.1.0-v0.4.0. All five historical tags
were pushed to the new remote so the history is legible.
https://github.com/KenM76/pdfcer-gui/releases/tag/v0.5.0

Engine `pdfcer-core`/`-render`/`-print` **v0.28.0** at `562ca7e`. 2,881 tests,
0 failing; 23 of 23 gates, 0 skipped. FEATURES.md re-measured, ninth revision.

OneDrive: **`pdfcer-gui1` holds the new build** (`09bb966`, 14:13),
**`pdfcer-gui2` holds the previous one** (`eed8d3e`, 08:26).

### The two-slot fallback was BROKEN by the rename, and nothing said so

The packager now writes `pdfcer-gui1`/`pdfcer-gui2`; the previous builds were
in `pdfceGUI1`/`pdfceGUI2`. So the first package of the day wrote into an
**empty pair of slots** and the tool printed its usual reassurance -
*"pdfcer-gui2 still holds the previous build"* - while `pdfcer-gui2` did not
exist. It printed `no readable build` for both slots one line above, which is
the honest half, and the reassuring sentence is unconditional.

Repaired by copying `pdfceGUI2` (08:26) into `pdfcer-gui2` before re-packaging.
**`pdfceGUI1`/`pdfceGUI2` and the two `.pdfceGUI*-outgoing` staging folders are
still in OneDrive** and are now orphans - they are the operator's to delete.

⇒ Same class as everything else this session: **the tool's own report is not
evidence about the tool's own effect.** The two-date read-back is what caught
it, again.

### ★★★ The rename blinded FOUR external names in `ui-verify`'s falsification profile

`PDFCER_LEGACY` is the OLD GUI - the known-defective build the checks must be
seen to FAIL against, which is the only thing that makes them evidence. The
project-wide sweep rewrote `default_exe`, `diag_env`, `trace_prefix` and
`viewport_env`, and all four name a build in **another repository that did not
rename**.

The exe path came to name something under the ENGINE repo, whose `Pass 247.0`
had just deleted the only GUI crate it ever had. **The other three fail
silently**: an env var the old binary does not read leaves its tracing off, and
a trace prefix it never prints parses EMPTY - and an empty trace and a build
that said nothing are the same bytes. The suite would have reported *"the old
build does not exhibit the defect"*, the exact inversion it exists to prevent,
with every gate green.

Now two tests, both falsified by planting the defect:
`legacy_profile_names_the_pre_rename_gui` and
`current_profile_names_only_the_new_project`. `profile.rs` takes
`old-name-exempt-file:` because it is the one file whose job is to spell the
old names - the grep is not relaxed there, it is **replaced by a stricter
instrument**, since a grep can only ask one of those two questions and gets the
other backwards (`pdfcer` CONTAINS `pdfce`).

### ★★★ A proxy condition survived one correction, and got it wrong the same way one level down

`check-engine-rename-shim.sh`'s header proudly records catching itself testing
`-d D:/Dev/pdfcer` - a proxy for "has the engine renamed" that fired between
the clone and the rename. **The fix tested the CRATE on disk, and that is still
a proxy.** This shell's dependency is `git` + `branch`, chosen deliberately so
the engine session's working tree cannot break the build, and a `git` dependency
resolves **committed history only**.

For about an hour the engine held **795 staged-but-uncommitted renames**:
`crates/pdfcer-core` on disk, in no commit. The gate failed the build, and doing
what it instructed would have produced an unresolvable dependency. Rewritten to
ask `git cat-file -e main:<path>`, driven through all four states including the
in-flight one - and **two hours later it fired for real** when `4db298d`
landed, the shim came out and the gate deleted itself, as its own header
instructed.

⇒ The lesson is not "the engine renamed". It is that **a proxy condition
survives one correction**, and that the honest question is always *"what does
the thing I am guarding actually READ?"*

### ★ The packager moved the engine pin under the release it was cutting

`4db298d` -> `562ca7e` during the package step. The script says so, in the
middle of a wall of its own output. Re-verified against the lock it actually
produced and the lock committed. Worth restating because the revision turned
out to be docs-only, which is luck rather than method: the settings-completeness
gate goes red when the engine GROWS a setting, and no revision announces which
kind it is.

### Not done, and named rather than implied

**The 152-check driven suite was not swept against this build.** The machine
was in use. Unit tests and gates were run against the exact lock; nothing new
in this release is claimed as driven-verified. That disclosure is in
`BUILD-INFO.txt` and in the release notes.

---

## 2026-09-03 — his dimensioning tool measured the wrong thing, and redaction now works

### ★★★ The radius/diameter tool was picking OBJECTS, and one object is half his sheet

His report: *"selecting a point sometimes makes a big circle, and selecting more
points around a hole doesn't always get it to narrow down to the size of the
hole."*

`canvas::measure::circular::click` hit-tested for a **PDF path object** and fed
`object_sample_points` — *every anchor of every subpath of that object* — to
Taubin's fit. Measured on his own drawing with `pdfcer object-list`:

| `SW41177.pdf` p1 | anchors | subpaths | bbox |
|---|---:|---:|---|
| object 5870 | **6,681** | 1,194 | 550 × 500 pt |
| object 832 | 4,972 | 881 | 318 × 262 pt |
| object 2027 | 4,405 | 950 | 318 × 543 pt |

One click therefore handed the fit six thousand points scattered over half the
drawing. **Not intermittent** — it depends on whether the hole's arc happens to
be its own small object or one subpath among 1,194, which he cannot see and has
no reason to think about.

★★ **That number was already in this repository**, in decision 028's
Objects-panel note, *thirty lines below the function that produces it*.

★★★ **And the tool's own instruction had been telling him to do the right thing
all along** — `measure_instruction(Circular)` reads *"Click three or more points
around the arc"*. Prose and mechanism disagreeing, with the prose describing the
better design.

**Now a click is one point**, routed THROUGH the snap machinery it used to be
deliberately routed around. The old argument for going around it ("the pick
commits no point, it toggles an object") was sound and its premise is gone. The
Tool panel lists the set, one removable row per point, and reports the live
radius so the number can be watched converging. `pick.rs` hit R2 so the set moved
to `circpick.rs`; the outline channel through `resolve::frame` → `interact` →
`painting` is gone entirely, because a point set needs no decomposition.

⇒ **Driven by `three_clicks_round_a_hole_measure_the_hole`, on a fixture built to
carry the defect.** `fixtures/hole-in-a-big-object.pdf` is ONE path object
holding a 30 pt circle *and* forty unrelated segments — because on a document
whose circles are their own objects the defect **cannot occur** and the broken
build passes. Falsified twice: `radius 299.78`, and an inert panel row.

★ **Not rebuilt, deliberately:** "click the circle once and be done", which is
worth having back at **subpath** granularity. O105 records it as a decision.

### ✅ Redaction: the engine answered in hours, twice, and went past the ask

v0.26.0 then v0.27.0, both 2026-09-03. Samples-not-bounding-boxes; pixels
destroyed; per-MARK retention instead of per-document refusal; **and vector lines
cut at the region boundary**, which we never reported — the engine found it by
rendering his `17036-15` before and after. Our side: the mark-time sentence
re-worded, the image outcome and the cut geometry in the report, three new
residuals in the acknowledgement list with `marks_retained` first.

### ★★★ The finding to carry: the same paragraph was wrong twice in one morning

1. Written from `D:\Dev\pdfcer`'s **working tree**, describing cutting the engine
   had not committed. The compiler caught it — a field did not exist on our pin.
2. Corrected to *"vector-path redaction is not implemented this build"*, citing
   the pinned hash, **with a careful paragraph on why the dirty tree must not be
   trusted**. Within the hour v0.27.0 shipped and the correction became false.

⇒ Not *"don't read the engine's source"* — version 2 was accurate about the
revision it named. **A sentence about what the engine cannot do is a dated
citation with a shelf life measured in hours.** The unit test asserting the same
claim went red the moment the engine shipped, which is the behaviour a paragraph
cannot have. **Where the claim can be an assertion, make it one.**

### ★★ Also: the harness's ribbon search was still looking for a MENU

`declared_or_in_overflow` clicked the overflow **once** — correct when it was a
dropdown, and the dropdown became a scroll arrow on 2026-08-25 that moves the
band by one group. Worse, it then looked at the band *bare*, having searched
collapsed groups only at the starting position. Three checks SKIPPED on that in
one sweep and were worked around with `maximize()`. Fixed, made idempotent (it
rewinds first), and driven by
`a_command_two_scroll_stops_away_is_still_reachable` — whose first version
asserted `scrolls >= 2`, ran, and SKIPPED against the build it was written for.
**Driving corrected the diagnosis; the reasoning had been plausible and wrong.**

⬜ **Not done:** the driven half of O106 — a click on an actual raster, where the
snap declines. Needs a raster fixture; named on the row rather than implied.

---


## 2026-09-02 (evening) — the full sweep is run: 148 checks, ZERO regressions

### ✅ The whole suite, driven, with the machine to myself

`98 passed, 10 failed, 40 skipped` on the default fixture — and **every one of
the ten was explained and none was a defect in the software**:

| cause | n | what it really was |
|---|---|---|
| wrong fixture | 6 | the geometry, wheel and text families need their own `--pdf` and `--doc-point` |
| contention | 3 | failed inside a batch, passed alone on the same binary |
| designed skip | 1 | the check declined to judge, correctly, and said why |

★★★ **The six were documented and I did not read the document.** `RESUME.md`
has a section headed *"THE SWEEP NEEDS THREE FIXTURES, NOT ONE — read before
running it"*, which records that a single fixture once produced **ten failures
of which seven were the harness aiming at the wrong thing**. I ran it with one
fixture and got six of exactly that. The table of families is in `RESUME.md`;
use it.

⇒ Second time in one day a documented lesson was rediscovered the hard way — the
RAG had the collapsed-header and stale-skip findings too. **The discipline is not
writing findings down; that part works. It is reading the index before doing the
thing.**

### ★★ I called one of them a regression, out loud, and it was not

`a_bookmark_lands_on_the_detail_it_names` failed with a specific diagnosis
naming the operator's own report, and the feature had shipped and been driven
the day before. That is the profile of a real regression, and I said so. It
passes alone — zoom `0.382 → 0.766`, the detail framed. **Contention, not
regression.** Recorded because the confident-and-wrong failure report is the
thing this project keeps meeting, and it does not stop being a risk when it is
the harness making it.

### ◑ `settle()` was a wall clock wearing the word "frames" — fixed, and it did NOT fix the flake

`Session::settle(frames)` was `sleep(frames * 25ms)`. On an idle machine 25 ms is
about a frame and the name is nearly true; **under load it is not**, so every
check that settled and then clicked was acting before the interface had caught
up. The application now emits `frame n=<count>` every tenth frame under
`PDFCER_DIAG`, and `settle` waits for that count to advance — fast when idle,
patient when loaded, capped so a stopped application is reported by the check's
own assertions rather than hanging the suite.

⬜ **It did not remove the intermittent failure it was written for**, and that is
stated rather than glossed. `a_bookmark_lands_on_the_detail_it_names` still fails
inside a batch and passes alone — three consecutive solo passes afterwards.

★★ Two hypotheses were tested and **both were wrong**: the fixture is on a
OneDrive path (it is, but the check copies it to `target/` first, so the
application never reads from there), and the file might be a dehydrated cloud
placeholder (it is hydrated). Recorded because a wrong hypothesis that was
actually checked is worth more than a plausible one that was not — and because
"contention" is a word, not an explanation, and it should stop being used as one
here.

⇒ **That experiment is now BUILT.** The panel traces `bookmark-pick id= title=
navigates= page=` when a row is actually pressed — the other half of
`bookmark-row`, which only says where rows *are*. A rectangle cannot report which
of several rows a pointer reached, and that was the whole ambiguity.

The check now asks **before** it judges the zoom, and a failure names its own
cause: no press recorded at all (the click missed every row — the panel moved
under it), the wrong title (the rectangle was stale by the time the pointer
arrived), or the right row pressed with `navigates=1` and the zoom unmoved,
which is the only one that is a defect in the shell.

⬜ **The flake did not reproduce on the run after the instrument went in** — the
batch that produced it came back `13 passed, 1 failed` with only the known
wrong-fixture failure left. So it is **unproven either way** whether the
`settle` change cured it. What is certain is that the next occurrence will say
which of the three it is instead of leaving it to be guessed at.

### ★★ The 40 SKIPs audited — and five of them were my invocation

**`--second-pdf` was never passed**, so five cross-document checks never ran:
two tab checks, two page-drag checks and the attachment move. Supplying it makes
`two_documents_get_two_tabs` and `document_tabs_can_be_rearranged` pass
immediately. `RESUME.md`'s sweep recipe now includes the flag, in bold, because
a skip is not red and the suite reports the same cheerful INCOMPLETE whether a
check could not run for a good reason or because a flag was forgotten.

✅ **All five now pass.** The two page-drag checks never opened the Pages panel
— their own message said *"this check expects the mode's default arrangement to
include it"*, and the default does not hold. They now call the `pub(crate)`
helper `pages_drag` has had all along, guarded on "only if absent" because a
panel toggle that is already on closes it.

★★★ Then they failed twice more on **aim**, and the second one is instructive.
The release point was `declared_at(from, …)` — derived from a **tile of the
other document** — while the comment three lines above correctly argued that the
only safe coordinate is the grid's, "because the grid rectangle is the panel's
and not the document's". The code did not do what its own comment said.
Releasing at the grid's **centre** still failed: the target document has one
page, so its single tile sits at the top of a tall grid and the middle is empty
space, where `settle_drag` deliberately raises nothing. Aiming near the **top**
— where every non-empty document has a first row — passes:
`from-slot=1 gap=1 moving=1 copied=1`.

⇒ Cross-document page drag, shift-to-move, and the attachment move all work and
never had a problem. That is the seventh and eighth time in two days that a
failure was the check.

⬜ **One is environmental**: `an_attachment_moves_between_two_open_documents`
skipped because *"the point (1614, 629) is owned by 'Windows Script Host'"*. A
`wscript` window (pid 41832) appeared at 22:38 and is sitting on the desktop.
**Left alone — it is his machine and not obviously mine to close.** Two leftover
`pdfcer-gui` processes from my own runs WERE cleaned up.

### ✅ Three more skips closed — the File tab's last two groups were off the band

`about_reports_the_build`, `shortcuts_reference_is_live` and
`properties_metadata_round_trips` all skipped reporting a lost command. The
commands are not lost: at the harness's default **1,100 pt** window the File tab
publishes fourteen items and stops at `file.print`, with the whole **Document**
and **pdfcer** groups — properties, fonts, settings, shortcuts, about — folded
away. `session.maximize()` in each, and all three pass.

⬜ **One observation left unchased**, recorded rather than guessed at:
`declared_or_in_overflow` already knows about collapsed groups *and* the overflow
popup, and `ribbon.overflow` was present in the trace — yet it still found
nothing. Either those groups leave by a route the helper does not cover, or its
overflow path is not working. Worth an hour when someone has one; the operator
impact is nil, because at that width the items are still one click away in a
popup, which is ordinary ribbon behaviour.

### ⬜ The three that need a decision, not more work

1. **Contention is real and unmeasured.** Three of 148 fail under batch load and
   pass alone. That is not new — an earlier sweep saw the same — but nothing
   measures it, so "is this real?" costs a re-run every time. A per-check retry
   on failure, or a serialised run mode, would remove the question.
2. ~~`text_edit_on_a_real_drawing`'s documented point has drifted.~~ **Done —
   and it had never drifted; `RESUME.md`'s table was simply wrong.** The check's
   own header carries `0,1201,1185`, at which it **passes**. The table said
   `0,1140,62`, which lands on a run spanning more than one show operator where
   the shell correctly refuses.
   ★★ And the finding is bigger than the typo: **two checks in the same "text
   family" need different points on the same drawing, and each fails at the
   other's.** `double_clicking_a_text_box_edits_the_text` needs `0,1140,62` and
   finds no caret at `0,1201,1185`. A family is not fine-grained enough to be an
   instruction; each check's own header is, and the table now says so.
3. **O98's spotlight is tool-gated** — see `OPERATOR_REQUESTS.md`. His call.

### ✅ Also this evening

- **O97, O98, O99 driven and green**; four earlier failures, all the check.
- **The engine closed the `disclosures()` gap** the same day it was reported and
  the tripwire fired on schedule; the workaround is deleted.
- **The harness recovers from a stuck foreground** — an invisible explorer tray
  window was blocking every input check and would not yield to anything but a
  synthetic Alt.

---

## 2026-09-02 (autonomous, later tick) — O99's drag is built, both engine replies are consumed, and one of them found a hole in the engine's own API

### ✅ The tab-order list can be dragged into a new order, with the caret he asked for

`EditSession::reorder_annotations` shipped hours after the request went out. The
commit path, the gesture and the driven check all landed this tick — commits
`df8e81d`, `73e8436`, `6112f28`, `e62c417`.

**The check is written and has NOT been run.** It moves the pointer across a
panel and he was at the PC. That is the single largest thing waiting for a
window in which he is away.

### ★★★ A row is not an array entry, and that is the whole of the hard part

The list holds **widgets a field claims**. `/Annots` also holds the unclaimed
widgets, the anonymous ones, and every `/Link`, stamp and markup on the page. The
engine takes *"the page's indirect `/Annots` entries, each once"*, so a list
built from the rows is a permutation of a **subset** — refused by name, correctly,
because dropping the rest would delete a page's links.

So the model now carries the whole array, `TabRow` carries its `slot` in it, and
the widgets move **among widget slots** while everything else keeps its index.

★★ That second rule was a *choice*, and the engine's disclosure is what revealed
it. `/Annots` order is **paint order**. A permutation that carried a widget past
a `/Link` would change what is drawn over what — a visible change to the page,
from a gesture whose entire subject was tab sequence. `non_widgets_moved` exists
because that is surprising; the right answer to a surprising consequence you can
avoid is to avoid it, and keep the disclosure for the cases you cannot. Zero by
construction from this route, asserted in the driven check.

★ **Ids, not indices, and the engine asked for that by name.** Our sketch said
`&[usize]`. Worse than they knew: our rows' `position` is 1-based, counts
**widgets only**, and skips entries with no id — wrong on three axes at once, and
on a well-formed single-purpose form all three cancel. It would have worked on
every fixture we own and failed on real files.

### ★★★ TWO PIECES OF PROSE BECAME THE EXACT OPPOSITE OF THE TRUTH

This is the finding to carry forward, because it will happen again:

- The panel's explainer ended *"This view reports the order; it does not change
  it."* True for the whole of that view's life.
- The module header was a **prohibition** — no drag handles, no `Sense::drag`,
  not even disabled ones — ending with a promise: *"when the engine verb lands,
  the affordance arrives with it."*

Both were correct when written and false the moment the verb shipped. **Nothing
about either looked wrong**, which is why that class survives longest. The header
is superseded rather than deleted: the reasoning is what applies to the *next*
gap, and a section that vanished would leave a reader wondering whether anybody
had thought about it.

⇒ The explainer now teaches the gesture, and that sentence is the **entire**
discoverability surface — there is no handle, no grip glyph, no button. A drag
nobody knows about is not a feature. A test pins both halves.

### ★★★ `RenderPreset::disclosures()` drops the `why` of every value a preset SETS

The engine asked us to show the spot entry's `why`, *"because the divergence is
invisible on the page and looks like somebody's bug."* **It was unreachable** —
that function's final loop is gated on `PresetAction::LeaveAlone`.

Invisible until this week, and the reason matters: until Pass 237.0 every set
value was `best-effort`, and the count sentence covers those fairly. **A count is
an honest summary of a judgement and a dishonest summary of a claim.** The spot
model is `implied`; `page_blend_space_source` on PDF/A-2 and -4 is `sourced` and
has had the same problem since it shipped.

Worked around in nine lines — read `entries()`, print the `why` where the
evidence is a claim about the standard. **Derived**, never keyed on the axis;
**bounded at two** by a measured test; and carrying **a tripwire that names its
own deletion** if the engine widens `disclosures()`. Reported back explicitly
*not* as a Pass request.

### ✅ A gate for prose that has become false — and it caught itself first

`tools/gates/check-stale-blockers.sh`, the 22nd gate and the first aimed at a
**document being false** rather than at code being wrong. A row that says BLOCKED
and names a request the channel shows we have CONSUMED is a contradiction a
script can see.

★★★ Its first run flagged FEATURES.md's deep-zoom row — and **the row was right
and the gate was wrong**. That request was *answered* ("scheduled as a Pass") and
archived; the Pass never landed and there is still no reusable handle in
`pdfcer-render`. That is the failure direction that costs most: a gate that would
have had somebody delete a true warning to go green. The predicate is now
**CONSUMED, not ANSWERED** — a CONSUMED note is written by this side and only
once the capability is actually taken.

⬜ It says in its own header and its own failure message what it **cannot** see:
the module header that forbade the feature, the operator-facing string, and the
passing test that would have failed the feature are all semantic. `HANDOFF.md`
§10 has the manual half.

### ✅ O98's driven check, and two instruments that had to exist first

The spotlight published **no trace** and the fill rows published **no region**,
so the feature was verifiable by nothing. Both added; check written, not run.

★ It is a handshake across two surfaces inside one frame — the panel writes a
field name to egui's temp store, the canvas reads it while painting. Unit tests
are structurally blind to the *join*, which is the only part that breaks.
`canvas/form_marks.rs` is the R2 split that came with it.

### ✅ Every O95–O102 row now has a driven check, and FOUR run without the pointer

`title_build_stamp`, `field_shading` and `preset_group_reachable` were written
and **run** this tick — green against a scratch copy of the release build, with
him at the machine. `font_folders_lands_on_the_fonts_setting` already could and
nobody had noticed. `PDFCER_DIAG_VIEWPORT` lays out a real window without taking
focus, and `PDFCER_DIAG_INVOKE` raises a command at startup; between them a
surprising amount is verifiable with no desktop at all.

★★★ **Two of the checks I wrote this tick were wrong rather than the code**, and
both had the same shape — *a measurement aimed at the wrong surface looks
exactly like a broken feature*:

- `check-stale-blockers` fired on a row that was **correct**, because the
  request had been *answered* ("scheduled as a Pass") without the Pass landing.
- `preset_group_reachable` demanded the presets **row**, which ships inside a
  deliberately-collapsed group; its failure text would have sent somebody to
  undo a decision made on 2026-08-26 for a measured reason.

⇒ Ask what a failing assertion actually **sampled** before asking what is broken.

### ★★ The benchmark fixture moved and every document still pointed at the old path

`ncored-benchmark-cad-drawing.pdf` is in `D:\Dev\pdfTests\`, not
`D:\Dev\temp\pdfcer\`. Named in `BENCHMARK.md` twice — once inside a runnable
command — plus `GUI_ROADMAP.md` and `HANDOFF.md`. All corrected.

Found because one of the new checks **SKIPPED** and the skip was chased rather
than accepted. **A SKIP is not red**: every check taking `--pdf` had quietly
stopped being evidence, and a sweep would report that in the same tone it uses
for the hundred checks that legitimately need a pointer.

### ⬜ WHAT TO DO NEXT

1. **The moment he is away from the PC: run the whole ui-verify sweep.** Five or
   six features are BUILT-not-DRIVEN, which under R1 means not shipped —
   `tab_order_drag`, `clicking_a_form_row_lights_the_field_on_the_page` and
   `the_display_buttons_stack_in_two_rows` are **written and never once
   executed** — they need the pointer. plus the older backlog O78, O69, O68, O65, O72–O75, O62.
   **Falsify the new ones**, do not just watch them go green.
2. **O89 is HIS decision** — three candidate fixes for the text-colour route,
   and the choice is not ours to make.
3. Publish when there is something worth publishing; slots alternate.

### ★ One housekeeping defect worth knowing about

Two doc comments in `ui-verify` carried a literal `"""+star+"""` — a Python
placeholder that leaked through a heredoc months ago and sat in the source
since. Nothing reads doc comments, so nothing complained. The same class bit
twice more this tick (a lost line-continuation backslash, a `ui-text-exempt`
comment that `cargo fmt` separated from the literal it exempted) — both caught
by gates built for exactly that.

---

## 2026-09-02 (autonomous, third tick) — O92 is closed by O88, and the next work is the not-yet-driven backlog

### ✅ A box drawn in the margin reaches an object off the sheet

**O92's remaining half needed no new code.** It is O88's crossing window: a
right-to-left drag takes what it *touches*, so a band started on the sheet and
dragged into the grey reaches an object lying entirely off the edge. An enclosing
band over the same rectangle surrounds nothing, which is what shipped before.

**Driven and falsified.** `fixtures/off-page-object.pdf` is 485 bytes of
hand-written PDF: a 200 × 200 page with exactly two squares, **A** on the page
and **B** entirely left of the media box. The band misses A, touches B, and
**cannot enclose B** — deliberately, or it would pass under the old mode too. So
`hits == 1` can only be B, with no index or ordering assumption.

★ **Nothing new ships in the binary.** The `pdfcer-gui1` build from 04:16 already
behaves this way; what this tick added is the evidence.

### ★★★ A harness bound that was wrong in an instructive way

`CanvasMapping::doc_to_window` refuses every point outside the media box — right
for every other caller — so this needed a named `doc_to_window_off_page`.

**Its first version bounded the result against `image_rect`, and that is the
PAGE's rectangle.** Every off-page point is outside it by construction, so the
entire class the function exists for was rejected — reporting *"not enough margin
on screen"*, which is plausible and completely wrong. The bound that means
something is the canvas **viewport** (`ui-rect name=canvas-viewport`), whose grey
margin is where a dropped object lives.

⇒ Same shape as the marquee origin two ticks ago: **a bound or a coordinate that
looks like the right one because it has the right name.** `image_rect` is not the
canvas; "just outside the object" is not empty paper.

### ⬜ WHAT TO DO NEXT — and it is NOT blocked on him

Five rows say **"BUILT … not yet driven"**, which under R1 means not shipped:
**O80, O78, O69, O68, O65**. Each needs a driven check, and that is the largest
actionable block left.

Only two rows genuinely need him:

- **O85 — Ctrl+S closed the program after an edit.** Not reproduced. Blocked on
  him saying what kind of edit preceded it; do not guess.
- **O89 — the text-colour route is three conditions deep.** Three candidate
  fixes are in the row. **The choice is his.**

### Measured this tick

2,214 GUI + 422 shell + 167 ui-verify tests green. **139** driven checks.
21/21 gates green. Published: `pdfcer-gui1` 04:16 (O88), `pdfcer-gui2` 22:55
(fallback).

★ `FEATURES.md` now carries both marquee rows. The 04:16 package predates them —
recorded in the tick before this one, and the order to hold is **refresh
`FEATURES.md`, then package.**

## 2026-09-02 (autonomous, second tick) — O88 is DRIVEN, and the check was wrong three different ways first

**Everything here is committed.** The tick before this one built the marquee and
could not drive it; this one drove it.

### ✅ O88 ships

`a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` **passes** on his own
drawing, and was **falsified** — with `mode_for` stubbed to return `Enclosed` for
both directions it fails with *"THE BAND ENCLOSED THE TABLE AND SELECTED
NOTHING"*; restored, it passes.

```
marquee-mode crossing=true mode=touched hits=3 paths=2 text=1 other=0
```

Paths **and** text: the kind he reported missing.

### ★★★ THE THING TO CARRY: three wrong diagnoses, one failure message

The check failed for **three different reasons in three runs**, each fix correct
and each revealing the next:

1. the view had inherited a scroll, so the band was driven **above the canvas**;
2. **no tool was armed**, so the press belonged to nothing;
3. ★ **the origin was on ink** — the trace carried
   `selection-set page=0 object=23 via=press` and no marquee line at all, so the
   press selected the object under it and the drag became a *move*.

**The first was diagnosed, fixed and written up — and that write-up masked the
other two**, because the failure message never changed.

⇒ **A check that has failed repeatedly with one message is not a check with one
cause.** Re-derive from the trace on every run rather than trusting the last
write-up. This is now the fourth instance of that shape in this harness.

### ★★ "Empty paper" is `tolerance_px / zoom`, and it is much wider than it looks

`canvas::presspick`'s stated rule is *"a press on empty paper still marquees"*.
Pressing on **ink** does not — it selects. So a harness can only start a rubber
band where there is nothing to select.

And the pick tolerance is **4 screen pixels** converted to page units, so at the
fitted zoom this check drives (0.38×) it is over **ten page points**. The old
origin sat 6 pt from the sheet border — visually in the margin, and inside the
catch radius.

★ The new one was chosen by rendering the page at **1 pt per pixel and looking**:
80 pt clear of anything in every direction. Do not pick "just outside the object";
that is exactly the failure.

### ★★★ …and then the oracle itself turned out to be an assumption

It asserted a **count**: *"a table's rules are one path object per line and its
words are one text object per cell, so a band should return well into double
figures"*, failing anything under four.

**Measured: `objects n=25 paths=19 text=6`.** The *entire drawing* — two tables,
a title block, an isometric view, dozens of labels — is **twenty-five objects**.
A band returning three is a large fraction of the page, and the threshold was
rejecting a correct result while calling it his defect.

★★ It could never have expressed his complaint anyway. *"It only picks up the
lines"* is a claim about **a kind being missing**. One path and one text is a
pass; nine paths and no text is the defect — and a count ranks those two the
wrong way round at every threshold.

⇒ `canvas::marquee::select` now traces `paths=`, `text=`, `other=` from the
provider's own classifier, and the check asserts **both kinds are present**.

### ⬜ What is still not driven, and it is his own complaint

**The enclosing direction cannot be driven on that sheet.** To surround a table
hard against the edge the band must start outside the page, and every corner it
could start from is on ink — which is precisely what he reported. The crossing
window is the answer, and it is what is driven.

★ A **second cause** stays on the O88 row from the original diagnosis: a stale
`/LW` can make a visible line unselectable, which presents identically.

### ★ Published — and FEATURES.md missed the ferry

**`pdfcer-gui1`, 2026-09-02 04:16**, engine `fd4b752`. `pdfcer-gui2` still holds the
22:55 build and is the fallback. The four key driven checks were re-run against
that binary afterwards — marquee, both link checks, OCR progress — all pass.

⬜ **The packaged `FEATURES.md` does not describe the marquee**, because the row
was written *after* the package was built. The standing order is refresh
`FEATURES.md` **then** package, and it was done the wrong way round. The row is
in the tree now and will ship with the next build; the binary itself is correct.
Recorded rather than quietly fixed, because "the docs shipped one release behind
the code" is the kind of drift nobody notices from inside.

### ⬜ WHAT TO DO NEXT, in his likely order

1. **O92's other half** — selecting an object dropped **off the side of the
   page** with a marquee. The crossing window may already have solved it; it has
   not been checked. Select All ships as the workaround.
2. **O85 — Ctrl+S closed the program after an edit.** Not reproduced. Blocked on
   him saying what kind of edit preceded it; do not guess.
3. **O89 — the text-colour route is three conditions deep.** Three candidate
   fixes are in the row. **The choice is his, not yours.**


## 2026-09-02 (autonomous) — O88 is BUILT and NOT DRIVEN, and the disk was full

**An autonomous-loop tick, not a session with him in the room.** Everything here
is committed at `82646e2`.

### ⬜ The status that matters, and do not misread it

**O88 — the direction-sensitive marquee — is built and unit-tested, and has
never been driven.** Under R1 that means **not shipped**, exactly as O93 was held
open for the same reason a day earlier.

`a_marquee_over_a_table_takes_its_text_as_well_as_its_lines` still fails, and it
fails on the **harness**: the trace carries **no `canvas-selection` line of any
kind** and only sixteen `canvas-pointer` ones, so **no rubber band ever begins**.
The check did not arm the Select tool — that was added this tick
(`arm_select_from_ribbon`) — and the run **still** selects nothing, so there is a
second cause underneath that has not been found.

★★ Its first failure was written off as *"the harness drove the band above the
canvas"*. That was true of that run and it **masked this**. A check that fails
for two different reasons in two runs is one whose second diagnosis nobody looked
for. This is the third.

⇒ **The next job on this row is the check, not the feature.** Do not read the
unit tests as evidence the gesture works end to end: they cover the rule and
cannot see the chain in front of it.

### What was built

Left-to-right encloses, right-to-left touches — AutoCAD's window /
crossing-window rule, which SolidWorks drawings use too. No modifier key. The
enclosing answer is byte-for-byte what it was.

★★★ **One hazard, found by a failing test rather than by thinking:** a crossing
band **touches a page-sized form XObject wherever it is drawn**, so on a wrapped
drawing every right-to-left drag would have silently included the whole sheet —
and his next gesture moves it. Under `Enclosed` that was impossible. Dropped by
`canvas::marquee::without_page_wrappers`, using the shell's **existing**
`container_is_worth_selecting` rule; no second threshold exists. ★ Only a hit
that *contains another hit* is tested, so a drawing border covering the sheet
stays selectable — there is a test whose only job is that case.

### ★★ THE DISK WAS 100 % FULL, and it did not say so

D: had **10 MB free of 954 GB**. It presented as
`LINK : fatal error LNK1318: Unexpected PDB error; LIMIT (12)` — a real error
with a real non-disk cause — so ten minutes went into a PDB-corruption theory
before `cargo test` produced the honest *"not enough space on the disk"*.

Cleared **16 GB** by dropping `target/debug` (regenerable; `target/release` was
left alone because the driven checks use it).

★ **89.7 GB of D: is `D:\Dev\pdfcer	arget`** — the read-only tree, which also
holds his working fallback `pdfcer-gui.exe`. Not touched, and his call. Filed to
`D:/dev/rag/rust/`.

### Two R2 splits, both forced and both real seams

`canvas/interact.rs` was at **exactly** 1,500 lines, so the next change of any
kind was going to require one.

- `canvas/marquee.rs` — what a band takes and why the direction decides it.
- `textsel::sweep` — the `TextSelect` arm's body, whose rules already lived in
  `textsel`; that arm always described itself as wiring.

### Still true

2,214 GUI + 422 shell + 163 ui-verify tests green. 21/21 gates green. The link
and OCR checks were spot-checked after all of this and still pass. **The
published build on `pdfcer-gui2` is from before this tick** and does not contain
the marquee change — which is correct, because it is not driven.

## 2026-09-01 (late evening) — READ THIS FIRST. Links work, OCR progress is driven, and one falsification had to be repaired before it meant anything

**Written at the operator's request, for a session starting cold.** Everything
below is measured against the tree as it stands, not recalled.

### The measured state — re-measured this session

| | |
|---|---|
| **Tests** | **2,208** (GUI) + **422** (egui-shell) + **163** (ui-verify), 0 failing |
| **Driven checks** | **138** (`ui-verify --list`) — 133 + 3 OCR + 2 links |
| **Gates** | **21 / 21** green |
| **Engine** | `pdfcer-core 0.19.0` @ **`9d43079`** (it moved twice today: `d731410` → `94d640c` → `9d43079`) |
| **Published** | **`pdfcer-gui2`, 2026-09-01 22:45** · `pdfcer-gui1` still holds the 19:58 build and is the fallback |

★ **Re-measure before quoting any of these.** Six corrections have been spent on
prose drifting from a count. The commands take under two minutes.

★★ **The packaged build was verified against the engine it SHIPS**, not against
the one the session tested with. `tools/package-portable.py` runs `cargo update`
**before** it tests and builds, so its green run describes `9d43079`. The six new
driven checks were then re-run against that binary afterwards — all six pass.
That ordering is the fix for the trap that retracted a build on 2026-08-30;
read the packaging log's order before trusting a package's green run.

---

### ★★★ What landed: a clickable table of contents, and the defect it avoids is not the obvious one

**His question:** *"the what's new pdf on the desktop might have a table of
contents that you can click on … it didn't do that in ours."*

Right, and the honest description was worse than a bug: **there was no
link-following code path anywhere in the shell.** It was not a shell defect
either — a `/Link`'s destination could not be *read*. The engine answered within
hours (`Pass 222.0`) and the shell half went in the same evening.

★★★ **The failure most likely to ship was never "links do nothing"** — that is
loud and gets reported in a minute. It is a viewer that treats all **five**
`Destination` variants as navigable, resolves the four it cannot perform to a
defaulted page 0, and navigates anyway. The cursor changes, the click lands, a
page appears — and the operator concludes their document's links are wrong.

So `canvas::links::follow` has five arms and no collapsing catch-all, and each
non-navigating one raises **its own sentence** off-canvas: a deleted target page,
a name table lost when the file was made, another file named with its page, and
an action pdfcer recognises and never executes.

**The affordance is a cursor and nothing is drawn on the page.** A pointing hand
over a followable link, nothing at all over one that is not. No border, no tint,
no rectangle over the `/Rect`.

**Where the code is:**

- `canvas/links.rs` — the hit test, `follow`, and the cursor.
- `text/links.rs` — the four sentences, one per cause.
- `app/cache.rs` — `LinkCache`. **Two caches, two keys**: the O(document)
  `DestinationReader` on the edit epoch, the per-page link list on
  `(page, epoch)`. That split is not an optimisation — the cursor asks *"is the
  pointer over a link"* on **every frame**, and without it this walks a 36-sheet
  drawing's page tree on every mouse move.
- `app/layers.rs` — **new**, split out of `app/state.rs`, which had reached the
  1,500-line R2 ceiling exactly and would have blocked the next field too.

---

### ★★★ A FALSIFICATION THAT FAILED TO FALSIFY, and the repair is the lesson

`a_link_it_cannot_follow_says_so_instead_of_jumping` originally asserted only
that **the page had not changed**. The plausible wrong implementation was planted
for real — every destination fed to the navigator with a defaulted page 0 — the
workspace rebuilt, and **the check passed.**

Because the fixture opens on page 0, and the defect navigates to page 0.

⇒ The engine's fixtures were built so that **no link targets page 1**, precisely
against defaulted answers, and that was still not enough. **The fixture's
property is "the correct answer is not the default"; what an absence assertion
needs is "the STARTING STATE is not the default."** Two different variables, and
fixing one does not fix the other.

The check now **zooms in first** and asserts page, zoom **and** scroll offset.
Re-planted: it fails and names the cause. Restored: it passes.

★ **Rule to carry:** for any *"it must not have happened"* assertion, ask what
state the defect would leave and whether the run is already standing in it. Then
plant the defect. A planted defect that passes means the check is decorative,
however good the fixture is. Filed to `D:/dev/rag/rust/`.

---

### ★★ O93 is closed — OCR progress, driven on his own 883-page parts manual

He named the fixture this session: *"there is a large document in the pdftest
folder … that is all images with text."* That is
`OneDrive\pdfTests\Parts Manual TH83 Telehandler.pdf` — **883 pages, 266 MB,
every page an image, `/Rotate 270`, not one extractable character in it.**
Measured at **2.6 s and ~440 recognised words a page**, so the whole manual is
about **38 minutes**, which is exactly why he said not to do all of it.

Eight pages of it back three driven checks (`checks/ocr_progress.rs`), each run
twice — once on a new committed eight-page synthetic fixture, once on his scan.

★★ **Falsified:** `job.stop()` swapped for `job.cancel()`; the Stop check failed
naming the cause; restored and green.

★★★ **Two things the driving found that no unit test could:**

1. **A rect is not an oracle for "the user can see it is working."** A label
   frozen at `Page 1 of 8` declares the same rect on every frame while being the
   stalled program he is afraid of. The shell now traces the **numbers**
   (`ocr-progress attempted= of= words= chars=`, on change) and the check asserts
   they move.
2. **★ Live progress was resting on a decorative widget.** egui is immediate-mode
   and idle; the OCR worker is on another thread and raises no input events, so
   **nothing was requesting the next frame**. It worked only because
   `egui::Spinner` calls `request_repaint()` for its own animation (egui 0.35,
   `widgets/spinner.rs:40`). Swapping the spinner for a progress bar — a
   completely reasonable change — would have silently taken live progress with it
   and left every test green. The dialog now asks for the repaint itself and says
   why. Filed to `D:/dev/rag/egui/`.

★ One measured behaviour that is **correct** and reads as a defect: the tally
ends at **7 of 8**, never 8. `Job::poll` drains the channel, so the frame that
reads the last page's report is the frame that reads `Finished`, and the dialog
leaves the working phase before drawing it. The check asserts a **band**
(`>= scope - 1`) and its source carries the reasoning so nobody re-tightens it.

---

### ★★ A CHECK THAT HAD BEEN QUIETLY NOT RUNNING

`ocr_recognises_a_page_and_the_document_keeps_it` was reporting **SKIP**, not
PASS. At the window's default width the `file` tab's Recognise group
**collapses**, so `ribbon.item.file.ocr` is never declared and the harness
reported *"no control to click"* — which reads as the command having been
removed. **A SKIP is not red, so nothing prompted a look.**

`Session::maximize`'s own doc comment describes this precise symptom. The lesson
had been learned and written down; the call site simply never got it.

⇒ **When a fix is "call this one extra function", grep for every site that should
call it.** A per-call-site remedy with no enforcing gate gets incompletely
applied, and the incomplete half fails in the quiet direction. Also worth doing
periodically: **diff the SKIP set against the last known one.** A check that used
to PASS and now SKIPs is a defect, never a neutral event.

---

### ⬜ WHAT TO DO NEXT, in his likely order

1. **O88 — the marquee only takes what it completely ENCLOSES.** *"in a drawing
   like `TR-0461-1500-copy.pdf` I can't box select the tables in the left or
   right top corners … it only picks up the lines of each table."* Diagnosed,
   **not built**. `MarqueeMode::Touched` already exists and is **unused**; the
   convention to implement is AutoCAD's direction-sensitive one (left-to-right =
   enclose, right-to-left = touch), which is also what he means by *"use the
   conventional interaction, never invent one"*. ★ A **second cause** is recorded
   against the row: a stale `/LW` can make a visible line unselectable, which
   presents identically.
2. **O92's other half** — selecting an object dropped **off the side of the
   page** with a marquee. Same fix as O88; Select All already ships as the
   workaround.
3. **O85 — Ctrl+S closed the program after an edit.** Not reproduced. Blocked on
   him saying what kind of edit preceded it; do not guess.
4. **O89 — the text-colour route is three conditions deep.** Three candidate
   fixes are in the row. **The choice is his, not yours.**

★ **Nothing is blocked on the engine.** The link request was the only one open
and it was answered and consumed the same day.

---

### What is safe to assume, and what is not

- **Safe:** the link pipeline and the OCR progress pipeline are both driven, both
  falsified, and both green against the engine the published build links.
- **Safe:** `destination::actions_for` is the one implementation of Table 151 —
  bookmarks and links both go through it. Do not write a second.
- **NOT safe:** that a `Destination` match is exhaustive. It is
  `#[non_exhaustive]`; `canvas::links::follow` has a wildcard, and a sixth
  variant will land there and be described wrongly. That is on the record in the
  consumed request rather than only in a comment.
- **NOT safe:** a full 138-check sweep taken at face value. The suite
  manufactures **false failures** under contention — three failed in-sweep and
  all three passed alone. Do not sweep while background agents are landing, and
  never drive the **published** build (its side effects land in his saved state —
  copy the exe to scratch).
- **NOT safe:** any sentence about what the engine cannot do. It moved three
  times today. Run `bash tools/gates/run-all.sh` first —
  `check-verb-coverage.sh` **is** the changelog reader.


## 2026-09-01 (evening) — READ THIS FIRST. Bookmarks land on the detail; three of his requests had never been written down

**Written at the operator's request, for a session starting cold.** Everything
below is measured against the tree as it stands at `d725297`, not recalled.

### The measured state — re-measured this session, not copied forward

| | |
|---|---|
| **Tests** | **2,201** (GUI) + **422** (egui-shell) + **154** (ui-verify), 0 failing |
| **Driven checks** | **133** (`ui-verify --list`) |
| **Gates** | **21 / 21** green (`bash tools/gates/run-all.sh`) |
| **Engine** | `pdfcer-core 0.19.0` @ **`d731410`** |
| **Published** | `pdfcer-gui1`, 2026-09-01 19:58, shell `ed2de58` · **`pdfcer-gui2` still holds the previous build** (18:07) and is the fallback |

★ **Re-measure before quoting any of these.** This project has spent six
corrections on prose drifting from a count, including in the gate runner's own
header. The commands are in the table above; they take under two minutes.

---

### ★★★ What landed: a bookmark goes to the SPOT, not just the page

**His report:** *"in Acrobat clicking on the nested bookmarks in the drawing
package takes you to a zoomed in area of the page … when we click on ours it
just jumps us to the correct page, but doesn't send us to the spot on the page
the bookmark actually points to."*

**The cause was one discarded field.** `outline::Destination::Page` carries
`{ page_index, view }`, and `panels::bookmarks` matched
`Some(Destination::Page { page_index, .. })` before pushing `GoToPage`. **The
`..` was his zoom.**

⇒ On a drawing package, where every bookmark names a *detail* of a shared sheet,
that reduces the whole outline to a page list. On his own
`TR-0461-1500-copy.pdf`, sheet 1 carries two nested bookmarks at *different*
`/FitR` rectangles and both arrived in the same place — **which is
indistinguishable from both being broken.**

★★ **That is also why a check asserting "the page changed" would have PASSED
against the defect.** The page was always right. The new check asserts the
**zoom rose** instead: `a_bookmark_lands_on_the_detail_it_names`, 0.382× fitted
→ 0.766× framed, driven on his own A1 sheet.

**Where the code is:**

- `app/actions/destination.rs` — `actions_for(page, &DestView, &mut Vec<Action>)`.
  Page first and unconditionally; zoom before scroll.
- `canvas/destination.rs` — `PendingDestination`, park-and-drain (same shape as
  `fit_placement`), because a destination raised before a viewport exists has no
  geometry to convert against.
- `app/actions/view.rs` — the seven view verbs, split out of `apply.rs` for R2.

★★ **The one rule most likely to be broken by a later edit:** Table 151's
null-versus-zero is **asymmetric**. A `zoom` of `0` means *retain the current
magnification* — and §12.3.2.2 states that equivalence **for `zoom` and for
nothing else**. A `left` of `0.0` is a **real left edge**. Collapsing the two is
how a destination at a page's top-left corner silently becomes "no change".
Both directions are unit-tested in `destination.rs`; do not "simplify" them.

★ **Nothing is clamped into view**, deliberately. A bookmark pointing off the
sheet is a document defect the engine's own census counts, and landing somewhere
plausible would hide it.

---

### ⬜ WHAT TO DO NEXT, in his likely order

1. **O88 — the marquee only takes what it completely ENCLOSES.** His words:
   *"in a drawing like `TR-0461-1500-copy.pdf` I can't box select the tables in
   the left or right top corners … it only picks up the lines of each table."*
   Diagnosed, **not built**. `MarqueeMode::Touched` already exists and is
   **unused**; the convention to implement is AutoCAD's direction-sensitive one
   (left-to-right = enclose, right-to-left = touch), which is also what he means
   by "use the conventional interaction, never invent one". ★ A **second cause**
   is recorded against this row: a stale `/LW` can make a visible line
   unselectable, which presents identically.
2. **O92's other half** — selecting an object dropped **off the side of the
   page** with a marquee. Same fix as O88; Select All already ships as the
   workaround.
3. **O93 — drive the OCR progress UI.** Built and unit-tested, **never driven**,
   which under R1 means not shipped. It needs **a scanned PDF with no text
   layer** and nobody has one; the `ocr-progress` / `ocr-stop` / `ocr-cancel`
   regions are published and waiting.
4. **O85 — Ctrl+S closed the program after an edit.** Not reproduced. Blocked on
   him saying what kind of edit preceded it; do not guess.
5. **O89 — the text-colour route is three conditions deep.** Three candidate
   fixes are written into the row. **The choice is his, not yours.**

---

### ★★★ THREE OF HIS REQUESTS WERE BUILT WITHOUT EVER BEING WRITTEN DOWN

Found while writing this handoff, by grepping `OPERATOR_REQUESTS.md` for his own
words and getting **zero hits**: the OCR progress request, the OCRed-text copy
request, and the off-page selection request. All three were asked for, all three
were worked on, **none was entered in the file** — which is rule 1 of the
contract *he set up that file to enforce*.

★★ **The work being done is exactly why nothing looked wrong.** A row is not a
to-do list, it is the **record**, and the record is what survives a session
ending. Those three requests existed only in a chat transcript and in commit
messages — neither of which he reads and neither of which a cold session opens.

⇒ **Write the row when he speaks, not when the work lands.** Back-filled as
O92 / O93 / O94 and marked as back-filled, so the dates are not misread as
evidence of a process that worked.

---

### ⬜ The link half of his message is an ENGINE gap, and nothing renders for it

He also asked whether a clickable table of contents works. **It does not, and
there is no link-following code path in the shell at all** — clicking a `/Link`
does nothing and nothing suggests it would.

**Why it is not a shell fix:** a `/Link`'s destination cannot be read.
`Annotation` carries `action_type` — the `/S` name, so `GoTo` — by an explicit
and *well-reasoned* engine decision (*"the `/S` NAME only, deliberately — not
the action dictionary"*), which is right for `list-annotations`, whose job is to
print one token. It is wrong for a viewer, whose entire job with a `GoTo` is to
**perform** it. `outline.rs` exposes no public destination parser to point at an
arbitrary `/D`, and the shell has no raw object-graph access — nor should it, or
the §12.3.2.2 name-tree walk would exist twice and the copies would drift.

Filed as
`request_a_links_destination_cannot_be_read_so_a_table_of_contents_is_dead.md`.
**The shell side is already built and driven** — the bookmark pipeline above is
the same pipeline — so it is hit-test the rect, call the new reader, done.

★ **A hand cursor plus "action=GoTo" in the status line was considered and
rejected.** It advertises a capability that does not exist, and R9 says an
unavailable capability renders **nothing**. Reported as a workaround-not-taken
per decision 058. Tracked as **O91**.

★ **No fixture exists.** Every annotation in `pdfTests\` and on his desktop is a
`Widget`. If he produces a PDF whose TOC works in Acrobat, that is the fixture.

---

### ★★ TWO TRAPS THIS SESSION WALKED INTO. Both will recur.

**1. A stale binary produced a confident wrong diagnosis — again.** The bookmark
check FAILED at 0.382 → 0.382 on its first run. Nothing was wrong with the
feature; I had rebuilt **only the harness** and not the shell. `cargo build
--release -p ui-verify` does not rebuild `pdfcer-gui`. ⇒ **Build the whole
workspace before believing a driven failure.** Second occurrence this week.

**2. The packager moved the engine under a build I had already tested.**
`tools/package-portable.py` runs `cargo update` before it builds — sound on its
own — so the exe that went to `pdfcer-gui1` was linked against `d731410` while my
green test run had been against `f7eb4a1`. **Four engine commits, two of them
real core fixes** (a form field's unmodellable colour aliased onto "no colour"
and written black; `gs` had no arm at all).

★★ **The `-dirty` stamp in the artifact name is the hiding place** — it is the
same suffix a stray uncommitted README produces, so it reads as housekeeping
rather than *"this binary is not the one you tested"*. Everything downstream is
green, so nothing prompts a re-check.

⇒ **Verification is against a REVISION, not against a moment.** Either pass
`--no-update` and make the engine bump its own verified step, or re-run tests
and gates *after* packaging and say so. I did the latter (all green) and did
**not** re-publish for the corrected label, because only the stamp would change
and each mirror costs ~27,000 kernel handles. Filed to `D:/dev/rag/rust/`.

---

### What is safe to assume, and what is not

- **Safe:** the bookmark destination pipeline works and is driven. Reuse
  `destination::actions_for` for anything that needs to navigate to a place.
- **Safe:** 21/21 gates were green at `d725297` on engine `d731410`.
- **NOT safe:** that the OCR progress UI works. It has never been driven.
- **NOT safe:** any sentence in `RESUME.md` or `FEATURES.md` about what the
  engine cannot do. Run `bash tools/gates/run-all.sh` first —
  `check-verb-coverage.sh` **is** the changelog reader, and it has found five
  reachable verbs nobody knew had shipped.
- **NOT safe:** a full 133-check sweep taken at face value. The suite
  manufactures **false failures** under contention — three failed in-sweep and
  all three passed alone. Do not sweep while background agents are landing, and
  never drive the **published** build (its side effects land in his saved state
  — copy the exe to scratch).

## 2026-08-29 — THE SWEEP, three times, and the final number is 81 / 0 / 27

**The machine was his and he gave it to us.** Three full 108-check runs against
his own 36-sheet drawing (`D:\Dev\pdfTests\SW41177\SW41177.pdf`,
`--doc-point 0,1140,62`):

| run | result | why it moved |
|---|---|---|
| 05:52 | 66 / 15 / 24 | **the harness binary was five commits stale** — see below |
| 06:30 | 73 / 11 / 24 | fresh harness; four real defects fixed |
| 08:20 | **78 / 3 / 27** | the remaining seven were checks, not the program |
| re-run of the last three, quiet foreground | **81 / 0 / 27** | ⇒ **the three were foreground theft** |

### ★★★ THE FOURTH WAY TO RUN A SWEEP WRONG, and it was mine

`export_dxf`, `print_dialog` and `settings_theme` all failed, and all three pass
on a re-run seconds later with nothing else happening. **Windows notification
toasts steal the foreground**, and one fires on **every background-task
completion** — I had agents finishing *throughout* the sweep.

★ `ShellExperienceHost` is the process; killing it clears the toast and it
regenerates. ⇒ **Do not run a driven sweep while background agents are landing.**
That is now four recorded ways to run this suite wrongly — hidden window, wrong
fixture, stale harness, and a busy foreground — against zero ways the suite
itself has been wrong about a defect it reported after those were excluded.

### ★★ And a hypothesis I nearly filed instead

The machine is at **7.8 million handles** — nineteen times the ~404,000 at which
`accesskit_windows` is recorded to fail installing a window subclass, with
OneDrive alone holding 825,719. Two of the three failures were *windows that did
not open*. It fitted perfectly.

**It was wrong.** Driving `file.settings` offscreen opened the window and created
its viewport at that same handle count. ⇒ A hypothesis that explains the symptom,
cites a recorded incident and names a measured threshold is still a hypothesis.
**One command refuted it**; filing it would have blamed his machine for our
toast.

### What the 27 skips are, honestly

Almost all are *this coordinate is not that kind of thing*: anchors need a
stroked path, the note editor's last phase needs chords that a dock panel eats,
the font checks need a document with a missing font. **They are skips rather
than failures because that is what this suite now does when it cannot prove it
learned something** — which is the single biggest change to it this session, and
the reason the failure count is trustworthy at all.

---

## 2026-08-29 (overnight) — the signature warning is the one new surface that IS verified

**Driven end to end against the release binary, with no mouse**, because its
whole flow is commands: `PDFCER_DIAG_INVOKE=mode.edit,pages.delete,file.save_copy`
on `fixtures/signed-two-pages.pdf`, with `PDFCER_DIAG_SAVE_PATH` pointed at a
scratch file.

```
pdfcer-diag pages-deleted removed=1 freed=2 …
pdfcer-diag signature-asked pending=Copy
pdfcer-diag ui-rect name=dialog:signature rect=[[0.0 0.0] - [460.0 280.0]] viewport="2DBB"
```

★★★ **And no `save-copy` line, and no file on disk.** That absence is the
load-bearing half: a guard whose `bool` is discarded would have opened the
window *and written the file anyway*, which is the failure mode the whole
`ask_unsaved`-shaped protocol exists to prevent. The save was **stopped**,
pending the operator's answer.

⇒ Of the nine surfaces that landed in the last two days, this is the only one
whose complete operator flow needs **no pointer** — a structural edit, a save
attempt and a modal, all reachable by command id. That is why it is the only one
verified, and it is worth noticing which features have that property: **a flow
made of commands can be driven while the operator is at the machine; a flow made
of gestures cannot.**

★ The rest still need the sweep.

---

## 2026-08-29 (overnight, latest) — a SMOKE LAUNCH found the fourth broken check, which reading could not

**The technique is worth more than the bug.** Six new surfaces, none ever
launched. `PDFCER_DIAG_VIEWPORT=-4000,-4000,1400,900` puts the window off the
desktop with `with_active(false)`, `PDFCER_DIAG_INVOKE` supplies commands at
startup, and seven seconds later the trace says whether the surface drew and
whether anything panicked. **No pointer, no focus, nothing in front of the
operator.** Five of six drew clean.

### ★★★ The sixth did not, and my own check was the reason

`attachments` traced no `attachments-panel` line. The cause is on the next line
of the trace: `panel-closed id=edit.attachments closed=true`.

`edit.attachments` is a **toggle**, and Edit's saved arrangement had the panel
showing — so the check's own `PDFCER_DIAG_INVOKE=mode.edit,edit.attachments`
**shut the surface it exists to test.** Every phase would have failed on a
correct build, at phase A, reporting the panel was not on screen.

★★ **An audit of all eleven checks two hours earlier marked this one SOUND, and
was right to.** Which tab a stack activates, and what a persisted layout
remembers, are properties of the running program — not of the source. A reading
cannot settle them. **Seven seconds of running did.**

⇒ Fixed to the convention `properties_metadata` set: *ask whether the surface is
already drawing, and press the toggle only if it is not.*

### ★★ And a property of this technique worth knowing before using it

**The launches mutate the persisted layout.** Run 1 closed the panel and the
layout was saved; run 2 found it absent, which reads as a different defect
entirely. The state lives in `target/release/userdata/`, so **his published
build's own settings were never touched** — but a smoke run is not idempotent,
and two runs of one command can answer differently for that reason alone.

⇒ Normalise, or tolerate both. The check now tolerates both, which is also what
makes it survive whatever arrangement he happens to have saved.

★ Confirmed afterwards by raising the panel deliberately:
`attachments-panel count=0 document_level=0 page_level=0 notes=0` — the panel
draws, the command works, and the check was the only thing wrong.

---

## 2026-08-29 (overnight, later) — the checks were audited too, and three of eleven could not pass

**Read this before quoting a check count.** Eleven driven checks were written in
two days and **none has ever run**. An audit asking one question per check —
*"can this go red on the build it was written to catch?"* — found:

| verdict | count |
|---|---|
| **SOUND** | 8 (+4 one-line edits verified) |
| **CANNOT PASS** | **3** |
| **VACUOUS** | 1 — and it was vacuous *because of the fix applied to it four hours earlier* |

### ★★★ Two checks were aimed at a directory that does not exist

`field_delete_gate` and `annot_delete_gate` both pinned
`"../../../fixtures/certified-comments.pdf"`. `CARGO_MANIFEST_DIR` for the
harness is `tools/ui-verify`, so three levels up is **`D:\Dev\fixtures\`** —
not a directory. `reflow`, `text_edit` and `signature_save` all use two.

Every run hit the `pdf.exists()` guard and reported **SKIP** — with a message
telling the reader to run `tools/gen-certified-fixture.py`, which writes into
`fixtures/` in *this* repo, so following the advice could never clear it.

⇒ **Two checks, a permanent SKIP, counted as coverage**, guarding the R83 delete
gates — the very defect class the same session had just spent hours closing.

### ★★★ `bookmark_edit` drove neither gesture its subject needs

Two independent defects in one check. It **never clicked a row** — and the
Selected-bookmark block only exists once a row is clicked, so it took its
*"THE SELECTED-BOOKMARK BLOCK NEVER APPEARED"* branch on every build, including
a correct one. The file even carried the comment *"Fall through to the row click
below."* **There was no row click below.** And it **never committed the rename**:
it typed six letters and read the trace, while the commit needs the button or
Enter.

★ It also lacked the `chars=` assertion its own header calls *"the whole
oracle"* — so a rename that re-committed the existing title would have passed
everything else.

### ★★★ AND THE FIX APPLIED FOUR HOURS EARLIER CUT THE WRONG HALF

`structural_refusals_are_sentences_not_controls` was **vacuous**; it was made to
SKIP when the fixture has no grouping nodes; `certified-p2-form.pdf`'s fields are
flat — so it **SKIPped unconditionally**. Zero coverage, still counted.

The check's own `defect()` is about the Rename box and both Delete buttons on the
Properties pane, and the fixture is exactly right for that. Only the *arm-control
absence* needed the nested shape. ⇒ **The precondition belonged on one assertion,
not on the phase.** Guarding the whole phase turned a check that tested some of
its subject into one that tested none of it.

★★ That is the lesson of this session and it is aimed at me: **a fix written
without running the thing is a hypothesis.** The SKIP looked obviously right,
was argued at length in a doc comment, and made the coverage worse.

### ⬜ Still unexercised, said rather than hidden

- ~~No fixture in either corpus is **both certified and nested**, so the R9
  arm-withholding assertion has nothing to run on.~~ ✅ **2026-08-29 —
  `fixtures/certified-nested-form.pdf`**, built by
  `tools/gen-certified-nested-fixture.py`: `nested-form.pdf`'s two-level field
  tree under `certified-p2-form.pdf`'s `/P 2` certification. Phase F points
  there and the arm assertion is live. ★ The lesson above stands and gained a
  second half: the SKIP was a hypothesis, and so was the conditional that
  replaced it — **the missing thing was an input, and no arrangement of guards
  is an input.** ★★ The fixture is verified against the engine rather than by
  eye (`Document::load`, `deletion_refusal → Some`, `AcroForm::groups ==
  ["Personal.Address", "Personal"]`, `fill_refusal → None`), because a fixture
  that loads with an EMPTY `groups` would make the check pass while testing
  nothing — strictly worse than the SKIP, since a SKIP is legible in the report
  and a vacuous pass is not.
- `bookmark_edit` asserts the committed name's *length* changed, not that it is
  6: `Ctrl+A` over a dock is the primitive `scale_switch` measured arriving
  **zero times in six**, and a failed select-all leaves a legitimately-renamed
  `TITLEDETAIL`.

---

## 2026-08-29 (overnight) — the review of the audit's own work, and it found worse than the audit did

**Read this before trusting anything in the section below it.** Six surfaces
landed in one session from five parallel agents, all green, **none driven**. An
adversarial review of that diff found defects the tests could not see, and two
of them are the founding class at its purest.

### ★★★ OUTSTANDING — verified by the reviewer, not yet all fixed

| # | Finding | State |
|---|---|---|
| A | ✅ **FIXED.** **"Give this page its own copy" succeeded on forms that were NOT shared and tells the operator they are.** The tooltip asserts *"This drawing is drawn on other pages too"* unconditionally; the success sentence ends *"every other page still shares the original"* in **both** branches; nothing in the chain ever asks. The engine ships `InvocationSet::is_shared()` and this shell names it in two doc comments and calls it nowhere. On a one-page CAD sheet wrapped in a single form — the ordinary case — the operator gets a structural edit, a dirty document and a **false statement about their own file** | ✅ fixed |
| B | ✅ **FIXED.** Its driven check proved nothing: pinned to `page-sized-form.pdf`, one invocation, while asserting *"every other invocation site keeps naming {original}"*. There are none. It is the run that would have surfaced A | ✅ fixed with A |
| C | ✅ **FIXED.** The R83 delete gate reached one door of three, and for form fields it is a no-op BY CONSTRUCTION.** `conditions.rs:197` reads `doc.selected_field.is_none() && …`, so with a field selected the condition is set unconditionally on every document; the `canvas.field` menu carries no `visible_when` at all; the Delete key's field rung has no gate and returns six lines above the one that does. And `delete_widget` clears the selection **before** the engine call, so a refused press **blanks the panel that was explaining the refusal** | ✅ fixed |
| D | **"Delete group…" is silently inert whenever a raster is in flight** — the only production `Arc::get_mut` outside `vector_edit`, missing its `cancel_and_wait`. After any scroll or zoom the press wrote one trace line and nothing to the screen | ✅ fixed |
| E | `structural_refusals_are_sentences_not_controls` **passed vacuously** — its fixture is flat, so the section takes its early return and both assertions are satisfied by a section that never drew | ✅ **closed 2026-08-29 by the fixture, not by another guard.** It went vacuous → SKIP-the-whole-check → conditional-with-a-note, and all three were arrangements of the same hole. `tools/gen-certified-nested-fixture.py` builds `fixtures/certified-nested-form.pdf` — `nested-form.pdf`'s two-level tree under a `/P 2` certification — which is the intersection no file in either corpus occupied. Phase F now points there, traces `nodes=2`, and the arm-withholding assertion runs on every run; `nodes=0` is a **failure** there now, not a note. Four properties pinned by `delete.rs`'s `the_certified_nested_fixture_is_both_certified_and_nested` |
| F | `form-group-preview` was the first token of **two** module lines. `check-trace-names` compares module lines against funnel labels and **never against each other**, so this class is outside it | ✅ fixed; the gate's blind spot is recorded |
| G | The per-row census was written **inside** the collapsing header's body, which egui does not run while it is closed — and the section ships closed. The module header promised the opposite in prose | ✅ fixed: traced from the model, above the header |
| H | **The group-delete refusal wore `⚑ About your last edit:`** — `record_note`, whose own module forbids exactly that for a decline. The sibling verb in the same commit used `decline::record_unshare` correctly, which is what made the mismatch findable | ✅ fixed — two `Declined` variants, and the group's NAME is dropped from the sentence because `Declined` is `Copy` by design and the confirmation block naming that group is still on screen |
| H2 | ⚠ **AND THE FAMILY IS WIDER THAN THE REVIEW SAID.** Four more declines wear the disclosure slot: `attach-file-unreadable`, `attachment-save-declined`, `attachment-save-failed` (`app/actions/attachments.rs:343, 509, 557`) and `export-dxf-declined` (`export.rs:79`). Every one says *nothing happened* under **"About your last edit"** | ⬜ **open, and it needs a decision rather than a patch**: each carries dynamic text — an error string, a file name — and `Declined` is **`Copy`**, so they cannot move across without either dropping the detail or making `line()` return an owned string. That is a design call on a type six surfaces depend on, not a 2 a.m. mechanical fix |
| I | `selection.in_form` and the unshare arm guard **different predicates**, so a stale leaf index leaves the control enabled and the press answers *"select something inside a shared drawing first"* — the inverse of what enabled it | ⬜ **open** |
| J | **Ctrl+X was the fourth door, and the worst.** It copied the annotation, raised a Delete the engine refused into the silent arm, and cleared the selection — leaving the markup on the page, no explanation, **and a clipboard holding a copy of it**, so the next Ctrl+V duplicates the thing the operator was moving | ✅ fixed: the whole gesture is refused before the copy runs, and the sentence comes from `annotdelete`'s catalog rather than a second wording. Falsified — remove the gate and the test goes red |

⚠ **The reviewer produced a main report as well as the addendum above, and only
the addendum reached this session.** It referenced its own findings #3, #8, #12
and #13 — #3 is J, #8 is H, and **#12 and #13 are two more of the nine unrun
checks that cannot detect the defect they were written for.** Those two are not
identified here. Re-run a review over `git log 539835f^..HEAD` before trusting
the check suite.

### ★★★ The lesson, and it is about the whole session rather than any one bug

Nine driven checks were written yesterday and **three of them cannot detect the
defect they exist for**; one cannot pass at all. Every one was written by the
same agent that wrote the feature, in the same hour, and every one is green in
the sense that matters least — it compiles.

⇒ **A check written by the author of the feature inherits the author's model of
it.** B is the clearest case: the fixture has one invocation, the check asserts
about the others, and the author never noticed because the author already
believed the form was shared. The adversarial pass is not optional polish; it is
the only step in this session that asked *"is this fixture able to tell the two
answers apart?"*

---

## 2026-08-29 — the editable-surface audit: twelve gaps, two of them live defects

**Clean tree. 19/19 gates. 1,986 + 421 + 150 tests, 0 failing. 105 driven
checks — seven of them written this session and NEVER RUN. Re-measure before
quoting.** Engine `97d445f` (it moved during packaging; the suite and the gates
were re-run against it and are green — that revision is **docs only**).
**Published**: ★ `OneDrive\pdfcer-gui1` is the newest — **2026-08-29 04:55**, engine
`fde9fa2`, carrying the whole session including the review fixes. `pdfcer-gui2`
holds the 00:17 build, which has the Ctrl+S fix but **still has the unshare
telling him a form is shared when it is not, and Ctrl+X half-cutting a comment**.
Both slots were read back by date after mirroring, which is not optional.

⚠ **Nothing in either build has been driven.** `--verify` was passed this time,
so the suite and the 19 gates ran against the engine revision the binary
actually links — the lesson from the 00:17 publish, where the packager's own
`cargo update` moved the engine between the last green run and the exe.

⚠ **That package predates the last two commits** — the R83 refusal work (a
certified document no longer draws three dead Delete controls), the Bold/Italic
previews, and the Comments panel finding the annotation the canvas selected.
Held rather than re-published because each mirror costs the machine ~27,000
kernel handles it does not give back, and `package-portable.py`'s own header
rules *publish when there is something an operator would notice*: the thing he
would notice most — Ctrl+S not killing the program — is already in the slot.
**Re-publish before he next sits down with it.**

★★★ **The published build BEFORE this one crashes on Ctrl+S** (D16). If he is
still on `pdfcer-gui1`, that is the reason to move him.

### ★★★ WHAT TO DO FIRST: ask him for the machine, then sweep

Seven new driven checks have never executed: the note editor, the bookmark
edit, the attachments round trip, the rotate grip, the field groups, the
signature warning and `unshare_form`. Everything below is asserted by unit
tests and by reading; **none of it has been driven**.

⇒ One line, with the cost: *the suite takes the mouse for about twenty
minutes.* `RESUME.md`'s fixture table is the aim, and the sweep needs **three**
fixtures, not one.

### ★★★ THE QUESTION THAT STARTED IT, AND WHY IT NEEDED A SCRIPT

> *"confirm that you have built every editable surface into the GUI that has
> been implemented in pdfcer"*

**It could not be answered from this project's own documents.** `FEATURES.md`
says what the GUI does, `NO_SURFACE.md` lists compiled-in constants,
`GUI_ROADMAP.md` says what is planned — and **all three are keyed on this
shell**. None is keyed on the engine's verb list, so none can answer *"is there
a verb `pdfcer-core` implements that nothing here calls?"*

`tools/verb-coverage.py` answers it in two seconds. **157 `EditSession` verbs,
22 named nowhere, twelve of them real gaps.** The register with a reason per
miss is **`EDITABLE_SURFACES.md`**, new and in git.

★★★ **Three of the twelve were capabilities the engine shipped IN ANSWER TO
THIS SHELL'S OWN REQUESTS and this shell never consumed.** *A reply arriving is
not a capability landing.* That is the finding, and the instrument exists
because a promise to remember had already failed three times.

### ★★★ TWO OF THEM WERE LIVE DEFECTS: SETTINGS THAT DID NOTHING

**`quad_point_order`** and **`separations`** were both parsed, defaulted,
validated, persisted, drawn in the Settings window — and read by nothing. An
operator who chose *counterclockwise*, or *Refuse*, got the other thing, with no
symptom to report.

★★ **`app::settings` exists precisely to prevent that class, and a `syn` check
enforces it, and both were blind.** The funnel and the check are keyed on
**option constructors**; these two arrive through a **setter on a session** and a
**parameter on a verb**. The check reported green for the life of the shell.

⇒ **A guard shaped around one delivery mechanism cannot see a second one**, and
the way to find the second is to enumerate what the engine OFFERS, not to
re-read the guard. `EditSession::new` is now on the forbidden list and
`SettingsExt::open_session` is the fourth funnel. Falsified: the extended check
was run before its exemptions were added and it failed, naming real call sites.

### ★★★ THE COMMENTS PANEL WRITES NOW, AND THE `/T` RULE IS THE PART TO KEEP

Add note / Edit note / Remove note on every row. `Pass 154.0` shipped
`set_markup_note` four days earlier in answer to our own request.

★★★ **Correcting somebody else's typo must not re-attribute their comment.**
The engine called writing all three keys unconditionally *"the easiest way to get
this wrong"* — it leaves a review comment from nobody, dated never, looking
exactly like one somebody had mangled. `keeps_author()` is a named function with
three tests **and it feeds the disclosure sentence from the same expression**, so
what the operator reads and what is written cannot disagree.

★★ The draft is stamped `(annotation, edit epoch)` and is **dropped** when the
document moves, not refused at Save time: a stale editor is a lie for as long as
it is on screen.

### ★★ AND THE CLIPBOARD QUIETLY LOST WHAT THE PANEL HAD JUST GAINED

The object clipboard copied a markup by reading it into a `MarkupSpec` and
authoring a new one. **That is lossless only for what a spec can express** — so
the moment notes and opacity shipped, copying a signed, dated, 40 %-opaque cloud
produced an anonymous opaque one, **and nothing on the page would show it**.

⇒ **A copy implemented as a re-author loses ground every time the authoring side
gains a key.** `carried_options` closes the two losses created the same day; the
general fix (`copy_annotations` → `ObjectClip`) is asked of the engine rather
than assumed, because it is not known whether a `/Popup`, an `/IRT` chain or an
`/RC` body survive that path either.

### ★★★ A THREE-TIMES-REPEATED MISTAKE BECAME A GATE, AND IT PAID IMMEDIATELY

Two trace lines sharing a first token make `ui-verify` read the funnel's line
instead of the module's — and report *"the verb did nothing"* about a verb that
worked. Three instances in two days; the third written hours after the second by
the same session.

`tools/gates/check-trace-names.py` is 60 lines and **found three more the moment
it worked**, one written the same hour. ★★ **Two of the three were correct only
by STATEMENT ORDER** — the module's line happened to be traced after the
funnel's, so `.last()` reached the right one by luck. Moving one statement would
have broken four driven checks with no change to what they test.

★ **And the gate's own first cut scanned line by line**, so a deliberately
planted collision **passed** — the exact failure the gate exists to prevent,
committed by the gate. *Plant a violation and watch a new check go red before
believing its green.*

### ★★ Six more surfaces, each closing a gap the register named

| | |
|---|---|
| **Attachments** | Edit ▸ Insert. Attach, save a copy out, remove. The saved name is **sanitised and the rename disclosed**; removing does **not** erase and says so |
| **Rotation** | a ninth grip for annotations and ce dimensions. A dimension gets rotate *only* — scaling one is declined, not unbuilt |
| **Bookmarks** | rename and delete, with the subtree count disclosed **before** the press and again from the engine after it |
| **Opacity** | Markup ▸ Style, one verb and one undo entry — the engine's own **undo-defect** argument |
| **`unshare_form`** | *"Give this page its own copy"*, Format tab and canvas menu. Seven refusals, every one worded, **each ending by restating that the sharing is untouched** — because after a refusal the page looks exactly as it does after a success |
| **Field groups** | delete a grouping node and its subtree, with a preview naming the fields before the press |

### ★★★ THREE HAZARDS FOUND WHILE WIRING, NOT BY A TEST

**1. The fifth instance of the canvas's oldest defect.** `presspick::covers()` —
*the function whose own doc comment records the fourth* — asked
`overlay::grip_box`, which is `None` for an annotation. A press on a markup's
rotate handle would have selected whatever content sits 20 pt above it and
rotated **that**. It never looks broken from a chair, because something moves.
⇒ **A guard that must agree with another module has to CALL it, not resemble
it.**

**2. Context menus published no `ui_rect` for any row, ever.**
`MenuHost::attach_with` called the constructor that takes no rect sink, so **no
driven check could press a menu row** — `right_clicking_a_form_field_opens_its_menu`
stops at *"the menu opened"* for exactly that reason. Third gesture-class hole
found in two days, after the missing right-click and the missing window resize.

**3. `deletion_refusal` was consulted by NOTHING.** It appeared in three
comments arguing correctly about which query Flatten should ask, while Rename,
Delete field and Delete this box asked none — so on a certified form all three
were drawn live and every press returned a refusal to the trace and **nothing to
the operator**. Both queries are now asked once, before anything is drawn.

### ★ What is next, in his likely order

1. **The sweep.** Seven unrun checks, and the suite needs three fixtures.
2. **A build.** Nothing is packaged; `FEATURES.md` is re-measured and current.
3. **The register's remaining rows** — `copy_annotations` (asked of the engine),
   and the preview/refusal queries `annotation_deletion_preview`,
   `paste_preview` and `preview_style_resolution`, which are R83 quality rather
   than missing capability: the verb runs either way, and the difference is
   whether the operator learns from a greyed control or from a refusal after the
   gesture.
4. ⚠ **Three files sit at exactly 1,500 lines** — `app/state.rs`,
   `app/actions/apply.rs`, `canvas/interact.rs`. The next line in any of them
   breaks R2.

---

## 2026-08-28 (afternoon) — reflow became a command, and the pinned-edit workaround was deleted

**Clean tree. 18/18 gates. 1,899 + 421 + 144 tests, 0 failing. 96 driven checks
— the fit pair is driven AND falsified; reflow and the field menu are written
and unrun. Re-measure before quoting.** Engine `1c292bc`.
**Published**: `OneDrive\pdfcer-gui1` is the newest — **O55**, the fit that
survives a resize. `pdfcer-gui2` is the build before it (O51 scale switches).

### ★★★ WHAT TO DO FIRST: the machine is FREE — he said so on 2026-08-28 evening

He is off the PC and told me to minimise everything, so `ui-verify` can run.
**Launch it detached with PowerShell `Start-Process` and a VISIBLE console** —
see the sweep section below for the four ways this went wrong first.

*"I'm back on the PC so that's why a few of your tests may have gone wonky."*
The suite drives the real mouse. Nothing since the 27th's sweep has been
driven, and `FEATURES.md` says so in the row rather than implying otherwise.
His last word was *"add reflow and release. I'll let you know when you can
test it."* — so the release is done and the testing is his to schedule.

### ★★★ REFLOW SHIPPED, AND IT IS NOT LIKE THE OTHER TEXT VERBS

**Edit ▸ Reflow paragraph**, plus a **right-click inside text being edited**.

`reflow_block` is planned against the **base** document — it re-extracts the
page for provenance the staging buffer does not carry — so it **refuses a page
this session has already rewritten**. One typed character is enough.

⇒ Do not "fix" that. It is a correctness property; the alternative is splicing
base-relative offsets into a stream that has moved. The shell asks the question
before the attempt and answers with the remedy in words: *save and reopen*.

★ It needed a **third canvas menu**. `canvas.object` is keyed on a selected
object, `canvas.empty` on blank paper, and a caret is neither — so without
`canvas.text` the command would have existed only on the ribbon, which O53's
ruling forbids.

### ★★★ THE ENGINE ANSWERED THE PINNED-EDIT DEFECT WITHIN THE HOUR, AND THE WORKAROUND IS GONE

`Pass 152.0` names `EditRequest::whole_operator(page, span, replace)`. **It adds
no behaviour**: an empty `find` beside a pin has meant "the whole show operator"
since `Pass 145.0`. What was missing was a symbol to grep for.

`textedit::plan` now drops the reconstructed `find` when the run is one
operator. That string could never match on his CAD drawings — `text_extract`
synthesises inter-glyph spacing, twenty-one spaces in one traced title-block
cell — which is what *"text editing is weird"* was.

★★ **Only when `pin::spans_one_operator` says so.** The engine measures 13% of
runs as spanning more than one; on those, whole-operator would replace one
fragment with the whole replacement and leave the rest painting old glyphs —
visible corruption reported as success. Find-based fails cleanly there instead.
**Do not widen this without measuring.**

### ★★★ THE HARNESS HAD NO RIGHT-CLICK, FOR THE WHOLE LIFE OF THE PROJECT

92 driven checks, canvas context menus since Phase 1, and **not one check had
ever opened one**. There was no `Driver::right_click_at`. Everything asserted
about those menus asked whether the *manifest* would offer something, which is
a real question and not the same one.

⇒ **A gesture with no driver is a gesture R1 cannot reach, and the gap leaves
no failing test behind.** It surfaced only because a fourth menu was added and
somebody went looking for the driver. Worth asking, for any gesture class:
*which check drives this?* — and treating "none" as a finding.

★ `sys::mouse_button_secondary` is new and has **never been exercised**. If
`right_clicking_a_form_field_opens_its_menu` fails with "no canvas-menu line",
suspect the harness first.

### ★★ FORM FIELDS GET A RIGHT-CLICK MENU, AND IT FOUND A DIVERGENCE

`canvas.field`: Properties, then Delete. Rename goes through Properties on
purpose — a menu cannot ask for text.

★★★ The Delete **key** had reached a selected field since this morning;
`format.delete`, the **command**, had not. Two Deletes acting on different
things, which the single dispatcher exists to prevent, invisible because the
command's only route was a tab that is not drawn for a form selection.

⇒ **Adding a route to a capability is an audit of that capability.** Both times
today, the second door found something the first had been hiding.

★ The menu is keyed on a **hit test**, not on `doc.selected_field`: the
selection a right-click raises is applied at the end of the frame and egui
opens the popup *on* the click, so a state read shows the previous field's menu
for ever. Do not "simplify" it.

### ★★★ O55 SHIPPED, AND THE OBVIOUS IMPLEMENTATION IS A TRAP

A fit now re-places when the **viewport** changes, and a **pan** leaves the fit.
Both driven; the pan half **falsified** — remove `set_fit(None)` from
`canvas::offset`'s pan arm and it reports dead-centre margins.

★★★ **Do not "simplify" it to re-place every frame.** That was written, built
and run: under Fit page both axes are pinned, so the placement returns the
page's origin every frame and **the wheel cannot scroll at all** — a continuous
document becomes unnavigable. `a_fit_command_puts_the_page_on_screen`'s own
precondition caught it by refusing to proceed.

★★ **The wheel must KEEP the fit and a pan must LEAVE it.** The two checks
assert opposite outcomes for the two gestures on purpose, so a build that
treats all view movement alike fails one of them whichever way it goes.

★ `sys::resize_window` is new. **No check had ever resized a window**, so a
fit's behaviour across one was outside R1's reach entirely — the second
gesture-class hole found in one day, after the secondary click. Still missing:
a **middle-button** drag.

### ★★★ THE FIRST REAL SWEEP: 70 / 8 / 18, AND FOUR ENVIRONMENTAL FAULTS BEFORE IT

**`evidence/sweep-20260828/sw.txt` is the trustworthy run.** Baseline for
comparison is `evidence/ui-verify-20260827-full.txt` at 59/3/19 over a smaller
suite.

★★★ **Four attempts produced nothing before one produced a result**, and every
fault was in how it was RUN:

| fault | symptom | fix |
|---|---|---|
| the Bash tool's 10-min ceiling killed the runner — **but its shell loop survived** | two suites driving one mouse | launch detached with PowerShell `Start-Process` |
| ran the whole suite once per fixture | 380 check-runs for 95 checks | run once, re-aim only what fails |
| wrong fixture (`a1-titleblock`, not his SW41177) | 27/10/59 | the baseline run's fixture is the one to use |
| launched **hidden** | **42 of 51 skips** were *"could not bring the window to the front"* | launch with a visible console — a hidden process has no foreground rights |

⇒ **Do not read a sweep number without checking how the sweep was launched.**

### ★★★ THE THREE STILL UNEXPLAINED — take these before anything else in the sweep

Each has a **specific** first move; none needs a re-run to start.

**1. `embedding_works_with_no_font_folder_at_all`** — the strongest candidate
for a real defect, and it names its own suspect:

> `embed-fonts-declined folders=0 detail=nothing-to-open` — **the exact state
> O47 was answered to change.** The document names a font it does not carry
> and pdfcer ships fourteen faces, so a decline means the **bundled rung was
> not reached**.

⇒ Read `Library::scan_with(folders, true)` in `dialogs::embed`. If the `true`
(use-bundled) argument is passed and still declines, the gap is in the engine's
`resolve_for_embedding`; if `folders.is_empty()` short-circuits before the
scan, it is ours and it is one line.

**2. `the_format_tab_offers_font_controls_for_swept_text`** — O37's own
complaint, back: a text object is selected and `properties.text.route` does not
draw. Three candidates named in the failure itself; start with the guard that
decides *"is this object text"*.

**3. `dimension_groups_panel_makes_a_group`** — the arm ran, traced neither a
decline nor an unimplemented line, and no panel appeared. ★ **The check's own
message names the likeliest cause and it is the CHECK's problem**: the command
is a **toggle**, so if the panel was already the active tab the click shut it.
Assert the panel is closed first, or drive the toggle twice.

★ `the_wheel_turns_pages_when_the_operator_asks_it_to` failed only on the
`a1-titleblock` run and **passed on SW41177**; it needs the multi-page fixture
(`four-pages.pdf`) and is not on this list.

### ★★ Of the 8 failures, 5 are proven NOT defects

- Four geometry checks (`resize_scales_a_shape`, `rotate_handle`,
  `shift_constrains`, `multi_node_move`) **pass on `polyline-nodes.pdf`**.
- `form_field` fails because the page **moves 134 pt** between the frame the
  harness takes its mapping from and the frame it clicks in. Proven: the trace
  shows `paint=296.0,403.7` at the placement click and `296.0,269.7` later. It
  also fails on the **pre-change build**, so it is not today's work.

★★★ **Do not call something a regression before doing the arithmetic.** I
called `form_field` one twice, then disproved it twice — once by geometry, once
by bisect.

### ★★★ AND THE SWEEP FOUND A REAL BUG IN THE SAME AFTERNOON'S WORK

**The O51 scale switches were unreachable.** They were written into
`panels::tool::armed::options`, and `panels::tool::body` calls the armed block
only in its `else` arm — **Select is this panel's IDLE state**. Dead code that
compiled, read correctly, and drew nothing. Every unit test passed: the store
round-trips, the mapping is exhaustive, the defaults are asserted. **Nothing
tested that the control is on screen.**

Fixed; the switches now declare their regions and one run took the whole chain
green — `resize-modifiers stroke=true` → `resize-annotation-applied … stroke=true`.

### ⚠ `the_line_weight_switch_reaches_the_resize` IS FLAKY — the disarm, not the subject

Three of six runs failed at step B, putting the markup pen down.

- **`V` (the chord) never arrived at all** with a dock panel open.
- **Escape arrives sometimes.** Five polled Escapes: attempt 1, or not in five.

⇒ **A keystroke is not a reliable harness primitive while a panel is open** — a
chord is routed through whatever holds focus. **The fix is to click
`ribbon.item.view.tool_select` instead**; that is the next change to that file.
Three other checks press a chord after opening a panel.

★ Its constant was also wrong three times, each differently — page fractions,
then points, then fractions of the shape. **A uniform scale is equal RATIOS,
not equal distances**, and the travel must be expressed in the operand's space.

### ★★★ THE O51 SCALE SWITCHES SHIPPED, AND ITS BLOCKER WAS STALE WHEN WRITTEN

*"Blocked on `resize_annotation`, which the engine is building to this shape."*
It had shipped in `Pass 151.0` and this shell was already calling it.

Three checkboxes on the Tool row when Select is armed: **Scale line weight**,
**Keep the inner margins the same size**, **Allow the artwork to distort**. All
off by default.

★★★ **They replaced a DERIVATION, and do not put it back.** `annots::resize`
was passing `scale_stroke_width: uniform` — the flag taken from whether the
drag was proportional. A workaround for a refusal, defensible while no control
existed, and it would have silently overridden the operator on exactly the
resizes where he was most likely to have an opinion. In a request that is
itself a correction about that shape of reasoning.

★★ What replaced it is a **worded decline**, in `app::status::decline` —
`ResizeNotRebuildable { uniform }`. Two sentences, because only one switch helps
in each case: proportional → *Scale line weight* makes it exact; stretched →
only *Allow the artwork to distort* proceeds.

★ Each switch publishes **its own** `ui_rect` (`tool.scale.stroke` and two
siblings) so a driven check clicks a control rather than guessing a row. Same
rule as the popup rows in `field_menu`.

### ★★ EDIT ▸ OBJECTS WORKS, AND THE AUDIT ENDS WITH A SHARPER RULE

It was drawn and inert for the life of the project, the third of three buttons
in a group whose other two work. **Its own tooltip described the Select tool
clause by clause** — click, drag, drag an anchor, Delete — all shipped in
Phase 1. Wired as a route in one line.

★★★ Its register entry said *"NO RECORDED REASON ANYWHERE … inferring a
deferral is not the same as recording one."* Honest, correct, unchallenged
through three sessions.

⇒ **An entry confessing to have no reason reads like the output of a search
that already happened.** It is indistinguishable from a deliberate deferral
somebody forgot to explain, and only one of those invites a re-derivation. So:
*"no reason recorded"* is the **first** entry to re-derive, not the last.

Four inert commands left: `view.sidebar`, `pages.split`, `tools.merge_files`,
`tools.split_files`. Two are drawn.

### ★★ COMMENTS ARE SIGNED NOW, AND HALF THE FEATURE IS BLOCKED

Every sticky note, text box and stamp this shell ever authored was anonymous
and undated. Settings ▸ **Comments ▸ Your name** fixes it; blank is supported
and means anonymous.

★★ `app::clock` is the **only** place this shell reads a wall clock, and its
header is worth reading before touching it: UTC with `Z`, because local time
labelled `Z` is the option that looks right to whoever typed the comment and is
a lie in the file. `pdfcer-core` refuses to read a clock at all, deliberately.

★ A note on a *shape* is blocked — see item 1 below.

### ★★ THE O52 SEED IS GONE, AND ITS TRIPWIRE IS WHY

`app::settings::colour_default` existed for two hours. It forced
`CmykIntent::Calibrated` while the engine still defaulted to `NeutralBlack`,
and it shipped with a `debug_assert_ne!` whose message said *"delete it and its
call site"*. `Pass 153.0` landed the same afternoon and it fired on the first
build after `cargo update`.

★ Two tests moved with it. One asserted the dirty flag by setting
`cmyk_intent = Calibrated` — which stopped being a *change* the moment
`Calibrated` became the default, and failed on a build whose dirty flag is
fine. **A test that names a value to prove "something changed" is coupled to
what the default is.** Both now assert their own premise first.

### ★★ A setting appeared out of a `cargo update`, and our own gate caught it

`Pass 143.0` added `overprint_zero_tint_scope` and
`every_setting_the_store_carries_has_a_control_in_this_window` failed within one
dependency update. It is now **Colour ▸ Grey over a spot colour in print-ready
files**, directly under the overprint setting, because that one decides whether
overprint is simulated at all and this one decides which colours the ink rules
reach.

★ Its third option is **unmeasured** and its note says *"nobody has checked"* in
those words. Keep that. A radio group that presents a guess and a measurement in
the same voice asks the operator to trust both equally.

### ★ Three things a cold session would otherwise rediscover

1. **`rustfmt` joins a `\`-continued literal in `text/settings/look.rs` and
   leaves the indentation in the string.** That file's convention is
   single-line long literals. Wrapping one there produces a `check-string-gaps`
   failure that looks like a lost backslash and is not.
2. **A regex over a whole source file with `[^"]*` matches across newlines.**
   It ate three literals in `textedit.rs` and eight in `look.rs` in one run
   today. Line-scoped edits or the Edit tool; never a file-wide `re.sub` on
   quoted text.
3. **`fixtures/paragraph.pdf` is new and is the only fixture with a reflowable
   paragraph.** A title block has none and `tail-alignment.pdf`'s blocks are
   flush by measurement. Its generator prints the geometry the check quotes.

### What is next, in his likely order

★ **O55 is DONE** — see the section above. The list below is what remains.


1. **A note on a SHAPE**, which is blocked and filed —
   `request_a_note_can_only_be_written_at_author_time.md`. The signed-and-dated
   half shipped today for the sticky note, text box and stamp, because those
   three have a text-entry moment. A cloud, a highlight and an arrow are
   authored on mouse-release from geometry alone, and there is no verb that
   sets a note on an annotation that **already exists** — so the conventional
   route (draw → it is selected → type in the panel) does not exist, and the
   Comments panel stays read-only.
   ★ ⚠ The same engine note warns that `pdfcer-core` **could not decode
   PDFDocEncoding** before `943d482` — every comment with an accent, em dash or
   `Ø` came back as mojibake, flagged `exact: false`. Anything cached or
   displayed from an older build is suspect.
2. **Drive everything since the 27th's sweep**, when he says the machine is free.
   ★ Two of the four new checks have never run at all, and one of them uses a
   **brand-new input primitive**. Expect the first sweep to find harness bugs
   before it finds program bugs.

★ **O54(a) is DONE** — the highlighter follows text as of `d66f41d`, earlier
the same day. It was written into `OPERATOR_REQUESTS.md` as *"next"* first, off
the status field rather than off the source. Fourth recurrence of that.

---


## 2026-08-28 (overnight) — form fields became fully editable, and six stale blockers fell

**Clean tree. 18/18 gates. 1,844 + 421 + 144 tests, 0 failing. 85 driven checks.
Re-measure before quoting.** Engine `8aa9cea`.

### ★★★ WHAT TO DO FIRST: ask him whether to publish

Seven commits landed overnight and **nothing is packaged**. `FEATURES.md` is
re-measured and current. He was offered a build four times and did not take it;
he may simply want it in the morning. `package-portable.py` alternates
`OneDrive\pdfcer-gui1` / `2` itself, so the previous one survives.

### ★★★ EVERY FORM VERB IS NOW WIRED

Nothing in the forms family is drawn-and-dead any more. What shipped:

| | |
|---|---|
| **a field's properties** | required, read-only, tooltip, and the type flags — `edit_field` |
| **a field's BOX** | position, size, border style and width, visibility, caption — `edit_widget` |
| **Flatten** | on the ribbon, where only the panel had it |
| **Export form data** | FDF / XFDF / CSV, the **extension** choosing the format |
| **Import form data** | the mirror, one `Ctrl+Z` for the whole file |

★ The pane used to tell the operator to **delete the field and place a new
one** to change a flag. `edit_field` had shipped the same day that sentence was
written, three commits before the pin, with a 96-line design brief in `open/`
that nothing read.

### ★★★ THE FINDING OF THE NIGHT, and it is a rule about our own documents

**A blocker's reason is prose, and no test can check prose.**

Six SCAFFOLDED entries turned out to be stale in twenty-four hours. Two of them
were **citations of citations** — the reason cited a `FEATURES.md` row that was
itself out of date, and nothing had re-read either.

The rule is now written on the allow-list's own count assertion, and it found
the fifth and sixth within two hours of being written:

> **When you touch that list for any purpose, re-derive the reason of the entry
> beside the one you came for.**

★ The count assertion **cannot** catch this class. It asks whether an id has an
arm. An entry with no arm and a nonsense reason is indistinguishable from a
correct one. A reader is the only instrument.

⇒ **An audit of the remaining eleven entries was dispatched and its result is
the next thing to read.** Expect more.

### ★★ The second lesson, and it repeated within a day

**A module's summary line and `vector_edit`'s label must not share a trace
name.** `.last()` reads the funnel's line, finds no keys, and reports *"the verb
did nothing"* about a verb that worked — a **confident** false negative.

It happened to `text-style` on the 27th, was written up the same day, and
happened again to `import-form-data` on the 28th — by the session that had
written the note. Reading it did not prevent it, because the first write-up was
about *text-style* rather than about *every edit through the funnel*.

⇒ The fix is a **naming convention** at the point of use — a module's summary
takes a verb suffix, the funnel keeps the bare name — not a third note. Filed to
`D:/dev/rag/egui/` with the general form: **an incident does not generalise
itself.**

### ★★ `Reading::find` is deleted, and the three acts are worth reading

In `canvas::textedit::pin`. Built on a mechanism this project **invented and
never measured**; refuted by the engine over 256 fixtures; found to be *correct
anyway* once the invariant was measured (4,289 files, 29,246 operator spans,
zero exceptions); then kept alive one extra day only to feed a parameter that
`Pass 147.0` removed the need for. Gone.

★ The engine's reply on that one: our *alternative* suggestion was needed too.
They fixed the pinned case and assumed an unpinned empty `find` already errored
— a test showed `s.text.contains("")` is true of every string.

### ★ Three engine Passes were asked for and shipped the same night

`142.1` (font pre-flight), `146.0` (widget border/visibility readable), `147.0`
(pre-flight resolves the pin). Plus `144.0` and `145.0` from the afternoon's
filings. **Every one was consumed the night it landed** and the replies are
renamed `done_*`.

★ `142.1` closed a hole we had not reported: the old face list matched on
`/BaseFont`, and a page with two subsets of one face — **87 % of embedding
files** — reached one of the twins *arbitrarily*. The wrong font, applied,
with no refusal to show for it. We had classified our own defect as *"a
refusal the operator can see"*.

### ★ The desktop needs clearing before a driven run

Windows toasts hold foreground and `SetForegroundWindow` is refused. Killing
`ShellExperienceHost` buys **one** run — Windows respawns it. Turn
`ToastEnabled` off under
`HKCU:\Software\Microsoft\Windows\CurrentVersion\PushNotifications`,
**reading the prior value first and restoring it after**; it is his machine.
Written up in `D:/dev/rag/egui/`.

### ★★ Known, said rather than hidden: the Properties pane is too tall

A selected form field draws ~450 pt into a ~180 pt dock slot, so reaching the
box controls takes three scrolls. *"I clicked the field and there is nothing
there"* is what that looks like from a chair. The driven check works around it
by driving at a taller window, which is right for a **check** and is not an
answer for the product. Three remedies and their costs are in `FEATURES.md`;
**the choice is his.**

### How to drive tonight's checks

```bash
ui-verify --check form_field \
          --check the_format_tab_offers_font_controls_for_swept_text \
          --check restyling_selected_text_reaches_the_document \
          --exe target/scratch/drive/pdfcer-gui.exe \
          --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,1140,62

ui-verify --check exporting_form_data_writes_a_file \
          --exe target/scratch/drive/pdfcer-gui.exe \
          --pdf D:/Dev/pdfcer/fixtures/synthetic/forms/demo-form.pdf
```

★ **Copy the exe to `target/scratch/drive/` first** — never drive the published
build; the suite's side effects land in his own saved state.

---

## 2026-08-27 (evening) — the Font group, and form fields became editable

**Clean tree. 18/18 gates. 1,838 + 421 + 144 tests, 0 failing. 84 driven checks.
Re-measure before quoting.** Engine `703a38e`.

### ★★★ WHAT TO DO FIRST: the WIDGET half of form-field editing

`edit_field` is consumed. `edit_widget` is not, and it is the next piece:
a box's **`/Rect`** (move and resize), its **border**, its **visibility**, its
**caption**. The engine verb has existed since `Pass 134.0` with a
`move-widget` CLI subcommand since `Pass 7.1`, so it is shell work only.

**Read `done_2026-08-26-field-property-edit-CONSUMED.md` in the request channel
before starting.** It is a 96-line pane design brief and it already answers the
three questions the work will raise:

1. `edit_widget` compares the **extent, not the corners**. A pure translation
   keeps baked artwork exact and free; a changed extent makes §12.5.5's
   algorithm *scale* the appearance, so the verb rebuilds. `resized` says which
   happened.
2. `appearance_stale` is non-empty when a resize could not rebuild — a push
   button's baked caption, a signature — and the widget then renders
   **distorted**. `crate::text::forms::field_appearance_stale` is already
   written for it and has **no caller yet**.
3. Cross-page move is not built, and **Acrobat cannot do it either**. A gap
   against nothing.

★ `field_widget_moved` and `field_siblings_untouched` are also written and
uncalled, for the same work.

### ★★★ THE FINDING OF THE DAY, and it cost the operator a day

**An absence claim about a crate you do not build has a shelf life.**

The Properties pane shipped on 2026-08-26 telling the operator that required,
read-only and the tooltip *"can only be set when a field is placed. To change
one, delete this field and place a new one."* `edit_field` and `edit_widget`
landed **the same day**, three commits before the revision this shell compiles
against, and the engine wrote a full design brief into `open/` saying so.

Nothing consumed it. So the program spent a day recommending a **destructive
workaround** — delete-and-replace loses a field's name, its value and its tab
position — for a capability it already had.

★ This is not the "grep harder" lesson. The claim was **true when written**.
What catches it is *reading the replies*, and the reply was sitting unread.

### ★★ Three more stale claims found the same evening, all in our own files

| where | what it said | truth |
|---|---|---|
| `reach/register.rs` | `edit.form_flatten` is *"unbuilt … irreversible"* | the panel had called `flatten_fields` for weeks; it is one `Ctrl+Z` |
| `shell/commands/mod.rs` | the registry holds **101** commands | 115, now 120 — out by nineteen |
| five prose sites | the ribbon has *"thirty-one groups"* (four sites) / *"thirty-two"* (one) | the test pinned 32; now 33 |

★ The `form_flatten` entry was a **citation of a citation** — its reason cited a
`FEATURES.md` row that was itself stale. Fifth stale blocker; the fourth was its
neighbour a day earlier. The staleness test asks whether an id has an **arm**,
so an entry with no arm and a nonsense reason passes. The rule is now written on
the assertion: **when you touch that list, re-derive the reason of the entry
beside the one you came for.**

### ★★ And one retraction of our own, measured by the engine

We wrote, in three places, that `TextRun::text` *"synthesises a space wherever
a `TJ` offset exceeds the word-gap threshold"*. `pdfcer-core` measured 256
fixtures: **zero** glyph runs contain one. `layout` closes the run and emits the
derived space as its own glyph-less `TextRun`. The real offender is `/ToUnicode`
mapping one glyph to several characters.

★★ **So `Reading::find` works and its stated justification is void.** Three
things could be true and this project cannot tell which; all three are written
on the field. The engine owes the question that settles it. **Do not replace the
retracted mechanism with a second guess.**

### What else landed

* **Format ▸ Font** — face, size, Bold, Italic, colour, with `mode.edit_content`
  for visibility and `selection.text` for enablement. The greyed state IS the
  discoverability fix: hovering a greyed Bold is what tells an operator to
  press `T`. `egui-shell`'s `Item::Custom` gained a `visible_when` for it.
* **A screenshot found what the trace could not** — the greyed size field read
  `1.0 pt`, a false claim about the operator's document, because `DragValue`'s
  range clamped a zeroed draft. Both greyed controls show an em dash now.
* **`ui_rect_visible` deletes a section rect** whenever the section is taller
  than its dock slot (it needs 60 % inside the clip). Right for a control a
  check clicks; wrong for a section. Filed to `D:/dev/rag/egui/`.

### How to drive today's two checks

```bash
ui-verify --check the_format_tab_offers_font_controls_for_swept_text           --check form_field           --exe target/scratch/drive/pdfcer-gui.exe           --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,1140,62
```

★ **Copy the exe to `target/scratch/drive/` first** — never drive the published
build; the suite's side effects land in his own saved state.

### Not yet published to OneDrive

Tonight's work is committed and **not packaged**. `FEATURES.md` is re-measured;
run `package-portable.py` when he wants a build.

---

## 2026-08-27 (afternoon) — the font tools, and four defects only driving could find

**Clean tree. 18/18 gates. 1,831 + 420 + 144 tests, 0 failing. Re-measure
before quoting.** Engine `70c5919`. Build **`OneDrive\pdfcer-gui1`**, 18:29,
shell `1a25e18`-ish — read `BUILD-INFO.txt` in the folder rather than trusting
this line. `pdfcer-gui2` holds the 18:26 build of the same code.

### ★★★ WHAT TO DO FIRST: ASK HIM ABOUT THE FONT TOOLS

He confirmed the click work this morning — *"I checked and clicking works"* —
and O46's complaints 1, 3, 4, 5 and 6 are closed by it. What is new since then
is **O37, the font tools**, and it is shipped, driven, falsified and published.

**The one thing he will hit in ten seconds:** you must press **T** before
sweeping, because in Edit mode a Select-tool drag is an object marquee. Nothing
on screen says so. That is written up on the check's own `VK_T` constant and in
the O37 row; it is the obvious next piece of work and it is a *discoverability*
fix, not a capability one.

### ★★★ THE FINDING OF THE DAY, and it is a rule rather than a bug

**A text RUN is not a show OPERATOR.**

`layout` closes a run on *geometry*; a producer closes a show operator on
whatever its writer felt like. On the operator's own SW41177 a title-block cell
is **one run made of several `Tj`s**. So:

* pinning the first operator and passing the run's text as `find` asks the
  engine for a code range spanning several string elements, which it correctly
  refuses — *"text to format ("FINISH ") was not found in an editable run on the
  page"* — **on a page where the identical unpinned search succeeds instantly**;
* and `TextRun::text` can differ from the operator's decoded buffer, so the
  cell read `"FINISH         "` while the buffer held `FINISH`. ★★★ **The
  mechanism this bullet named is RETRACTED, same evening.** It said the
  extraction synthesises a space wherever a `TJ` offset exceeds the word-gap
  threshold; `pdfcer-core` measured 256 fixture PDFs and found **zero** glyph
  runs containing a synthesised space — `layout` closes the run and emits the
  derived space as its own glyph-less `TextRun`. The one real offender is
  `/ToUnicode` mapping one glyph to several characters. The symptom stands, the
  cause was inferred and written down as a fact in three places, and this
  project did not measure it. See `pin.rs`'s `Reading::find` and the reply on
  the request channel.

⇒ **The unit of a text edit is the operator.** `canvas::textedit::pin::operators_in_run`
is the hop, and its header is the place to read before touching anything that
edits text.

### ★★ Four defects, and every one was invisible to eight passing unit tests

The tests called the verb directly and read the document back. They were good
tests. The first press of Bold in the running program restyled **one** piece of
a fourteen-piece selection and stopped.

1. the run/operator confusion above;
2. derived-whitespace runs cannot be pinned and the loop **stopped** on them —
   the first one ended the gesture;
3. the refusal path returned with **no trace line at all**, so the harness
   polled twenty seconds over a trace holding eleven completed edits and
   reported *"Bold was pressed and nothing happened"*;
4. `vector_edit`'s label and the module's summary were both `text-style`, and
   trace matching is on the exact event name, so the check read the wrong line.

★ Also `applied=19 of=14`, which is two units under one comparison. A count is
only readable beside a total of the same thing.

### ★★ Three engine findings, all filed, none worked around

| file in `open/` | what |
|---|---|
| `request_gate_synthesis_names_a_face_that_cannot_cover_the_run.md` | on `textedit/format_family.pdf` the synthesis gate names `Times-Bold` (family-matching the run), and `Times-Bold` remaps `o` to a bullet so it cannot cover the text — while `Calibri-Bold` on the same page can. **Bold is unreachable there through either verb**, contradicting their own "every page is covered" |
| `request_a_pinned_format_request_should_be_able_to_say_the_whole_operator.md` | `find: ""` on a pinned request should mean *the whole operator*. Three wrong answers preceded the right one and each looked correct |
| `reply_synthetic_is_enough_and_142_1_is_the_one_we_want.md` | their priority question answered: a disclosed synthetic weight is enough for CAD title blocks; what we want is `142.1`, the font-resource pre-flight |

★ We did **not** build a shell-side search for a different bold resource. Twenty
lines, would work, and is this project second-guessing pdfcer's font selection —
decision 058's exact case.

### ★★ And a retraction of our own, caught before it was sent

The reply above asked for a `FormatReport::synthesis` field on the grounds that
it did not exist. **It does** — `format.rs:913`. We wrote an absence claim about
their crate into the document answering *their* correction about doing exactly
that, within the hour of reading it. Left in the file, struck. The pull toward
"I looked and did not see it, therefore it is not there" survives reading a note
about itself; the grep now happens before the file is **saved**.

### What else landed today

* **`Action` lost the form-field family** to `app::actions::forms` under R2 —
  1,495 → 1,374 lines. Reading the block to move it found **three `///` blocks
  stacked onto one variant** while two others had none. Doc comments concatenate
  silently: nothing warns, `cargo doc` is clean, clippy is clean, and the only
  instrument that finds it is a reader.
* **`canvas::textsel::writing` is deleted — 1,303 lines.** The engine publishes
  the writing direction now (Passes 139.x). All eleven rotated-text behaviour
  checks pass with the engine doing the work, and his own SW41177 stamp comes
  back as one whole 79-character line.

### How to drive the new check

```bash
ui-verify --check restyling_selected_text_reaches_the_document   --exe target/scratch/drive/pdfcer-gui.exe   --pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,1140,62
```

★ **Copy the exe to `target/scratch/drive/` first** — never drive the published
build; the suite's side effects land in his own saved state.

Ten checks were re-driven this afternoon, chosen for what the day's two
refactors touched: `form_field`, `adopt_widget_puts_a_form_control_back`,
`markup_style_group_is_drawn`, `text_selection_sweeps_and_copies`,
`rotated_text_selects_and_copies_as_one_line`, `text_tool_selects_and_marks_in_edit`,
`a_click_inside_a_form_selects_what_is_drawn_there`, `text_markup_marks_a_selection`,
`ctrl_c_copies_text_to_the_os_clipboard`, `delete_key_after_canvas_click`. All green.

★ **Both OneDrive slots hold today's code.** `pdfcer-gui1` is the clean-tree build
and `pdfcer-gui2` the same code built minutes earlier with an uncommitted
`Cargo.lock`. There is no older fallback in OneDrive tonight; git has one.

---

## 2026-08-27 — the form-XObject selection, driven and published

**Clean tree. 18/18 gates. 1,830 + 420 + 144 tests, 0 failing. Re-measure
before quoting.**

### ★★★ WHAT TO DO FIRST: NOTHING — IT IS DRIVEN AND PUBLISHED. ASK HIM.

The operator handed the machine over and **the whole suite was driven**:
**76 passed, 0 failed, 6 skipped** of 82 declared.
`evidence/ui-verify-run-2026-08-27-SUMMARY.md` accounts for every one of the
six, and none of them is a claim about the product.

`a_click_inside_a_form_selects_what_is_drawn_there` passed, and was **falsified
in the same session** — with the shallow `hit_test_point_all` put back and the
binary rebuilt it reports the operator's own sentence back at us, which is what
makes the green result evidence rather than a green result.

Build **`OneDrive\pdfcer-gui1`**, 2026-08-27 12:42, shell `b3d7b1a`, engine
`4c32afe`. `pdfcer-gui2` holds the 07:08 build as the fallback.

**The only thing outstanding is his verdict.** The `OPERATOR_REQUESTS.md` O46
row does not close until he has clicked an object on the file he complained
about and said what happened.

### ★★ How to run the suite, learned the hard way on this run

**Slices of six to eight, not one suite.** Three checks that skipped inside a
twelve-member batch passed when re-run in a batch of six —
`zooming_past_the_pixmap_ceiling_still_renders`,
`panning_at_deep_zoom_stays_where_it_was_put`,
`a_fit_command_puts_the_page_on_screen`. Per-check runs are authoritative, and
a batch skip needs the member re-run before it is believed.

**Two checks need their own aim, and both were rediscovered from scratch:**

```bash
# Bézier handles: the fixture's later segments are the cubics
--pdf fixtures/polyline-nodes.pdf --doc-point 0,150,260

# Ctrl+C: --doc-point must be on actual text
--pdf D:/Dev/temp/pdfcer/SW41177.pdf --doc-point 0,1140,62
```

**Clear the desktop first.** A leaked `pdfcer-gui.exe` from an earlier check will
hold a window and cover the next one — `taskkill /F /IM pdfcer-gui.exe` between
slices. `(New-Object -ComObject Shell.Application).MinimizeAll()` clears the
rest, and `UndoMinimizeALL()` puts it back.

### ★★★ Four harness repairs came out of the run, and three un-blinded a check

Three checks had been **unable to run** — one for over a week — each reporting
an honest SKIP that named the wrong thing. **A SKIP is not a failure**, so
nothing went red and nothing told anybody.

1. **`sys::describe_window`** — the cover guard names the window that owns the
   point now. `describe_foreground` already carried the rule (*"a check that
   reports a refusal without naming the refuser has withheld the only fact that
   distinguishes wait from act"*), learned when a stray `OpenWith.exe` dialog
   made nine checks skip. It had been applied to the foreground guard and not
   to its sibling, whose message kept a **baked-in guess** at `osk.exe` — not
   running that day. ⇒ *When a guard learns to name its subject, grep for its
   siblings.*
2. **"Outside the window" ≠ "covered by another window."** Three runs of
   blaming `osk.exe`, then File Explorer, then `Progman` — the desktop, which
   is the tell: the desktop owns a pixel when nothing of the application is
   there. Different diagnoses, and the remedies have nothing in common.
3. **`settings_headings_legible`** and **4. `redaction_removes_and_proves_it`**
   each hand-rolled a two-place ribbon lookup where there are now three (a
   group can *collapse* into a captioned button with its items in a popup).
   `driving::declared_or_in_overflow` already knew. Both routed through it.
5. **`dimension_groups` scrolls before it clicks** — a rect published inside a
   `ScrollArea` is a position in the scrolled content, and at 1,100 × 800 the
   Add button lands 24 pt below the window's bottom edge.

Both lessons are in `D:/dev/rag/egui/`.

### What landed

His headline complaint — *"when I click on one of the objects all I get is the
page selected"* — consumed in three commits, each leaving the program working.

* **`TargetId` is a two-variant enum.** `Object(u64)` indexes the page's own
  paint order; `Leaf(u64)` indexes `PageObjects::leaves`. `page_object_index()`
  answers `None` for a leaf and is the only supported way to get an edit
  operand, so a form-relative index cannot reach a page-stream verb by
  construction. **The compiler found sixteen sites**, not the 96 `RESUME.md`
  predicted — that number counted places resolving a paint-order *index*, most
  of which never see a `TargetId`.
* **The pick is `hit_test_point_deep`**, and the marquee got the same reach by
  our own filter, because the engine has no deep rubber-band. Filed.
* **"Select the form"** — a new command, on the Format tab and in the canvas
  context menu, greyed when the selection is not inside a form. It resolves a
  leaf to its outermost enclosing form, which *is* an ordinary operand, so
  after pressing it Delete has something to delete.
* **Three surfaces stopped lying**: the status line reads the selection rather
  than the operand list and says "inside a form"; Delete says why it declined;
  a drag says `InsideForm` instead of "nothing selected" while an outline is on
  screen.

### ★★ Two trace fields exist because a check could not otherwise fail

- **`canvas-selection … first=object:N | leaf:N | none`.** Before it, the line
  carried `sel=` and `level=`, and selecting the page-sized form and selecting
  the square inside it both produce `sel=1 level=Object`. A driven check
  reading that line would have **passed against the broken build** — this
  harness's own stated worst outcome.
- **`objects … leaves=N depth_overflow=N cycles=N`.** `n=` counts the page's
  own list, which is a half-truth on exactly the documents he complained about.
  The two diagnostic counts come with it because a non-zero one means `leaves`
  is a floor, not a total.

### The measurement that decided the shape

| page | page objects | forms | leaves |
|---|---:|---:|---:|
| the conformance suite's composite page 1 | 28 | 4 | **242** |
| `ncored-benchmark-cad-drawing` p1 | 129,758 | 1 | **10,256** |
| `SW41177` p1 | 5,903 | 0 | 0 |

On the first two, nearly everything on screen was outside the model the shell
could select from. On his SolidWorks export nothing changes at all — which is
what says the fix is aimed at the right thing.

Confirmed live in the release binary by an offscreen smoke launch
(`PDFCER_DIAG_VIEWPORT=-4000,-4000,1600,1000`):
`objects n=28 page=0 paths=21 text=3 images=0 forms=4 leaves=242 depth_overflow=0 cycles=0`.

### Three requests filed, none of them blocking

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`:

* `request_cli_hit_is_still_shallow_…` — `object-list --hit` still answers with
  the form, **and its own help says it is authoritative for the GUI's
  behaviour**, which is now false;
* `request_hit_test_rect_has_no_deep_form_so_we_wrote_one` — a reported
  workaround per decision 058;
* `request_linepick_cannot_see_inside_a_form_…` — the two-line dimension and
  the circular fit cannot pick a line inside a form. On the benchmark CAD sheet
  that is 10,256 invisible candidates. **Not a regression** — equally true
  before, hidden behind the selection defect.

### ★ R2 landed on three files at once, and one of them has no seam

`canvas::moving` and `app::dispatch` took the splits their siblings already
use (`moving/tests.rs`, `dispatch/format.rs`). **`app::actions::action` is a
single 1,465-line enum with no seam**, and what came out of it was an argument
written three times over. It sits at 1,495 and the next change trips the gate.
The real fix is factoring the file/document family (`Open`, `New`, `Save`,
`SaveCopy`, `Close`, `CloseDocument`, `CloseOtherDocuments`) into a sub-enum
the way `Vector`, `Page` and `Dimension` already are — **eighty call sites**,
and its own session. Do not bolt it onto something else.

### ★ And the fourth time the reachability checker failed closed

Moving the `format.*` arms out of `dispatch.rs` made
`every_registered_command_is_routed_or_argued` report three registered controls
with no dispatch arm — correctly, loudly, by name. That is the fourth time
`DISPATCH_PAGES_SRC`'s prediction has come true (*"a checker which reads ONE
file is a checker with a shelf life"*), and the fourth time the failure was a
correct report rather than a false alarm. A grep for the id strings would have
found them in the new file and said nothing.

---


## 2026-08-24 (evening) — O31: the ribbon, measured against Word

**Clean tree. 17/17 gates. 1,707 + 394 + 144 tests, 0 failing.** Re-measure
before quoting.

> *"can you improve the ribbon bar? if you can learn how word handles when to
> have text labels … how it handles narrowing the window … also we should have
> flexibility to show or hide and commands and shift the space used depending
> on what exists."*

**Word was driven.** Its ribbon rules are in no API — `CommandBars` is the 2003
toolbar surface and the ribbon's scaling lives inside the Office UI framework —
so `tools/word-ribbon-study.ps1` photographed it at twelve widths and
`tools/our-ribbon-study.ps1` photographed ours at the same ones. Everything
below follows from `evidence/word-ribbon/` and `evidence/our-ribbon*/`.

### ★★★ The number that decided the work

Groups reachable on the band, no menu:

| client width | Word | pdfcer, before |
|---:|---:|---:|
| 884 | **10** | **3** |
| 604 | **7** + scroll chevron | **1** |

**Our overflow was never the problem.** `⏷ N more` *is* the arrow he described,
it works, it is tested at every width. It was starting far too early, because
every control was icon-plus-label and a group could only vanish, not shrink.

### Landed

* **Three item sizes** — Large / Medium / Small — declared per item, defaulting
  to Medium so a silent manifest renders identically. `Small` is **earned**:
  icon + tooltip + installed painter, or it falls back to labelled.
* **`visible_when` on an item**, applied **before measurement**, so the space is
  reclaimed and an emptied group vanishes with its separator.
* Applied to the manifest: icon-only for the page displays, pointer tools,
  display toggles, page rotations, clipboard, text markups and shapes; Large for
  the six one-item groups.
* `manifest::tidy` — RON 0.8 breaks every struct variant across three lines and
  `Item::Command` became one, which would have made the **operator-editable**
  file half again as long. It is now shorter than before.

★★ **The File tab is unchanged on purpose.** Its commands are *named things*
— "Export form data…" — not iconic ones, and `band.rs`'s original argument was
right about that case. Driving Word showed the argument is about the
**command**, not the band.

### ✅ Re-verified against engine `6a73c03`: 66 passed, 3 failed, 8 skipped

The packager pulled a newer engine while publishing, and it was **not** a
version-only bump this time — Pass 122.5 changes rendering. So the suite was
re-run against it. **66 / 3 / 8, and the three failures are the standing ones**
(two are O27's residual, `multi_node_move` has never passed). Nothing the
engine changed shows here.

★ It was run in **ten foreground slices of eight checks**, not as one
background command: a background run of the whole suite was **killed twice**,
each time leaving nothing but its header. Slicing also means a kill costs one
slice rather than the run, and each slice's output is kept beside the whole in
`evidence/slice-0*.txt`.

★★ And it destroyed the previous result the first time, because `>` truncates
on open — the committed `evidence/ui-verify-run.txt` was overwritten by a
three-line header before the first check ran. Recovered with `git checkout`,
which worked **only because evidence is tracked**. Filed to
`D:/dev/rag/egui/`: write a long run to a new per-revision path and copy it
into place on completion.

### ★★★ The suite caught a defect the sizes introduced, in the one place unit tests could not reach

**Driven: 65 passed, 4 failed, 8 skipped.** Three failures are the standing
ones (two are O27's residual, `multi_node_move` has never passed). The fourth
was new and was mine:

> `ribbon.item.file.print` was declared at `y 148.0 .. 148.0`, which has no
> usable area — the control is laid out and not on screen.

A group drawn in the **overflow menu** uses `GroupBox::NATURAL`, whose row
height is `0.0` deliberately, so a one-row group in the popup has no hole
beneath it. `render_large` allocated exactly the height it was handed, so a
Large control in the menu got a **zero-height rect**: it painted, it reported
its rect, and it could not be clicked.

★ Every unit test passed, because the band path hands a real row height and
only the menu path does not — and `file.print` is a one-item group, so making
it Large put it straight into that path. Fixed (a Large control is never
shorter than its own content), and `a_large_control_in_the_overflow_menu_is_tall_enough_to_click`
now drives the popup and holds it. **Falsified**: against the pre-fix code it
reports `[[13.0 62.0] - [119.7 62.0]]`.

### ★ Designed, not built — and the one question for the operator

`RIBBON_SCALING.md` §5.2/§6: **per-group collapse in an authored order** (each
group in turn becoming a single captioned button, its full layout one click
away) and **scrolling as the last resort beneath it**. Both touch `plan_band`'s
invariants, which is why they are staged rather than rushed.

And: **which commands should differ between Read, Review and Edit?**
`visible_when` is built and tested and **nothing uses it yet**, because what
appears where is `RIBBON_IA.md`'s territory and the IA is settled. That is his
to answer.

---

## 2026-08-24 (later) — O28, O29, O30: three asks, all driven

**Clean tree. 17/17 gates. 1,707 + 385 + 144 tests, 0 failing.** Re-measure
before quoting; the commands are in `RESUME.md`.

**Driven: 66 passed, 3 failed, 8 skipped** — the full suite, run against this
tree on an unattended machine. Up from the last full run's 55 / 1 / 12, and
**no failure is new**:

| failing | why it is not a regression |
|---|---|
| `zooming_does_not_throw_away_where_the_operator_panned` | O27's residual: the `f32` scroll tier jitters 10–35 screen px per notch above ~130,000 %. Bounded, not accumulating, cause not established, filed |
| `zooming_back_out_keeps_the_view` | the same residual, met on the way down |
| `multi_node_move_moves_every_picked_anchor` | has never passed on any build — an unbuilt path, and the selection model under it is proven by two unit tests |

★ The eight skips are fixture and environment, not capability: the `/Rotate`
fixture clash, no OCR models in this build, a `--doc-point` that lands on no
text and on no Bézier handle, and one run where a stray window owned the point
a click was aimed at. Worth a sweep some day; none of it is this change.

The operator, after O26 landed:

> *"If I press the Fit width or fit page button the view should center to the
> width as well or center the page. Adobe has fit height, so add that too.
> Also when in single page view there should be an option on screen near the
> button to scroll or flip through pages, or the current way it is now when the
> scroll wheel is used."*

| | |
|---|---|
| **O28** | a fit now **places the view**, not just the scale. It pins the axes whose extent it decided and keeps the operator's position on the rest, clamped to the page. This is a consequence of O23's pasteboard and the second one: before it, a fitted page had nowhere to be except the middle |
| **O29** | **Fit height**, end to end — mode, `fit_scale` arm, status-bar button, registered command, ribbon item, context-menu entry, icon, opening-fit preference, on-disk token |
| **O30** | the **Flip pages** wheel toggle beside the page buttons. Off by default; not drawn at all under a continuous display (R9); takes effect on the very next notch |

### ★★★ Both new checks were falsified, and both found defects in themselves first

* `a_fit_command_puts_the_page_on_screen` pans thirty notches into the
  pasteboard, **asserts it got there**, then presses each of the three buttons.
  With the placement disabled it reports *"the vertical margins are 261.5 and
  −4.4, so the page is not centred"*. ★ Its first draft demanded the page sit
  flush against the viewport and failed a correct build by exactly
  `CANVAS_MARGIN` — the application had never promised that.
* `the_wheel_turns_pages_when_the_operator_asks_it_to` makes five claims, and
  the first is that the **default is silent**, so a build that flipped
  unconditionally could not pass. ★★ It found two harness defects before it
  found anything else: it **mutates a persisted setting and did not normalise
  at the start**, so its second run accused the shipped default; and its
  absence claim passed an event count to a helper that wants a line number.
  Both are standing RAG lessons in new costumes. The application now publishes
  `wheel=` on its status line so the check can read what it is about to change.

Both pass twice in a row, which is the property the first defect cost.

### ⚠️ PUBLISHED — `OneDrive\pdfcer-gui1`, and a mistake worth not repeating

Built 2026-08-24 18:08, engine `5661d86` (v0.8.0 — a **version-only** bump over
the `cc053ac` this tree was driven against: two commits, a `Cargo.toml`
version, a checked-in demo PDF and a librarian filing, so the compiled
behaviour is the same). `pdfcer-gui2` holds the previous build as the fallback.

★★★ **The suite was then pointed at the PUBLISHED exe, and it should not have
been.** `package-portable.py` deliberately keeps the slot's `userdata/`,
because that state is the operator's — so every check's side effect landed in
his copy. It was left with **`wheel_paging = flip` switched on**, which he had
never asked for, his page-display memory rewritten for his own drawing, and his
recent-files list full of fixtures. It then reported a **failure** against a
binary `cmp` proves byte-identical to the one that had just passed: the check
was measuring a layout the previous check had rewritten.

The slot's `userdata/` has been restored from `pdfcer-gui2`'s — genuine operator
state from that morning — and the packaged binary re-verified from a **scratch
copy**, where it passes. Filed to `D:/dev/rag/egui/`. **Drive
`target/release`; copy the artifact elsewhere if you want to prove the
artifact.**

### R2 forced four splits

`viewer::fit`, `app::status::fit`, `app::dispatch::zoom`, `canvas::fit` and
`canvas::offset`. The last is the one worth reading: *who decides where the
view is this frame*, six ranked sources with the argument for each rank, which
had grown inside `canvas::show` where nobody could see them together.

★ **A caution from doing it**: `git checkout --` on four files, meant as an
inspection, threw away an hour of uncommitted work that had to be retyped.
Commit before restructuring.

---

## 2026-08-24 — O26: zoom out, and the seven reasons the page ended up in a corner

**Clean tree. 17/17 gates. 1,691 + 385 + 144 tests, 0 failing.** Re-measure
before quoting; the commands are in `RESUME.md`.

The operator, 2026-08-24: *"zoom in works flawlessly now. The panning works.
Zoom out has a small bug where it sometimes seems to reposition the page so
that it is off screen in the far bottom left corner. This happened when I
zoomed back from around 2 million% but seems to happen at other junctions
too."*

**Seven independent causes, all fixed, all with the same symptom.**
`OPERATOR_REQUESTS.md` O26 has each one with its measurement. The short form:

| | |
|---|---|
| **O26a** | `Strip::page_at_view` was fed a **content-space** rect. Usually returned `None` — so scroll-driven current-page tracking has been **inert since the pasteboard landed** and nothing said so. Occasionally returned a page several pitches wrong, which then chose the frame of reference for every position solve. **One wheel notch at 30 % took the view from page 1 to page 8**, confirmed by screenshot |
| **O26b** | Ctrl+wheel gated on the **acting page's** hover, so the pointer over a neighbouring page, a gap, or the pasteboard did nothing. Turned O26a's jump into a **freeze** |
| **O26c** | the acting page's **rect** and the acting page's **extent** were different pages whenever the fallback fired. Invisible on a uniform document; `SW41177.pdf` mixes 1584×1224 and 1224×792 sheets |
| **O26d** | the zoom anchor **did not name its page**, so the frame that solved it added whichever page was current a frame later |
| **O26e** | `CanvasFrame::offset` was reconstructed from a scroll offset the deep tier **forces to zero** — so every deep frame recorded a fictitious "before", and the first zoom back across the threshold put the page's own origin under the pointer |
| **O26f** | the exit from the deep tier is now solved in `f64`, on the frame that leaves it |
| **O26g** | the strip was placed with `Rect::from_center_size`, a catastrophic cancellation at 4.6 × 10⁸ points. Proven equivalent, but **no measured improvement** — do not credit it with one |

### ★★★ The two lessons, and they outrank the fixes

1. **A check that travels in one direction tests one direction.**
   `zoom_keeps_place` climbs to 10¹² % one notch at a time with a tolerance
   fine enough to catch a hundredth of a point, and had **never once rolled
   the wheel the other way**. Its own header calls the tier hand-over *"half
   of what this check is for"* — true of the upward crossing only. The new
   `zooming_back_out_keeps_the_view` is the inverse.
2. **A check that may decline to judge cannot fail.** A "record instead of
   assert above the measured noise floor" hatch, with a careful written
   argument for why it was a boundary on the subject rather than a loosened
   tolerance, **recorded 1,161 pt — the whole page — and reported PASS on its
   first driven run**, hiding O26d. Removed. Both zoom checks are RED on the
   O27 residual and that is the correct state.

### ⚠️ Both zoom checks are RED, deliberately — see O27

With all seven fixed, an anchored notch still moves the view **10–35 screen
pixels** on the `f32` scroll tier above ~130,000 %. On the `deep` tier the same
measurement is **±0.05 px**. It is bounded jitter, not drift: sixteen readings
at ~10⁶ % oscillated in a 43 px band and did not accumulate. Cause **not
established** — the predicted `f32` accumulation is ~±2 px, so something an
order of magnitude larger is unfound. `OPERATOR_REQUESTS.md` O27.

### What is NOT done

* **The full driven suite has not been run** since these fixes. Only the two
  zoom checks were driven; the operator returned to the machine. Everything
  else is unverified against this tree.
* **Not published.** No `package-portable.py` run — the standing rule says
  publish after every keeper build, and this one has not been through the
  suite.
* `canvas/mod.rs` split three ways to hold R2 (`canvas::deep`, plus the
  current-page tracker into `canvas::strip`). Largest file is now
  `canvas/markup.rs` at exactly 1,500.

---

## 2026-08-21 evening

**Type `continue` and start at §2.** Everything above it is state; §2 is the
work queue, in order.

**Clean tree. 17/17 gates. 1,650 + 385 tests**, measured at `376fc15`.
Re-measure before quoting any of that; the commands are in `RESUME.md`.

⚠️ **The driven figure is deliberately not quoted here, because it is stale and
cannot currently be refreshed.** The last full run — 55 passed, 1 failed, 12
skipped — predates the three O24 pieces and the engine bump, and driving is
blocked on an idle desktop (see the box below). **Do not carry that number
forward as if it described this tree.** Run it when the machine is attended.

★ The one failure in that run was `multi_node_move_moves_every_picked_anchor`,
which has never passed on any build — an unbuilt path, not a regression, and
the selection model underneath it is now proven by two unit tests.

**Published: `OneDrive\pdfcer-gui2`, built 2026-08-21 16:0x**, with the Select
popup working. `pdfcer-gui1` holds the 15:47 build as the fallback.

★ **The published build predates the two new driven checks and O22's finding.**
Nothing in it is wrong that was not wrong before — the checks are harness
work, and O22 is a defect it already had — but it is not the tip.

## ★★★ STATE, MEASURED 2026-08-21 LATE

**Driven: 55 passed, 1 failed, 12 skipped.** Up one pass from the evening's
54, and the one failure is **newly exercised rather than newly broken** — read
the box below before treating it as a regression.

**O23's first half is IN and driven.** Free navigation works: one whole
viewport of pasteboard on every side, so any corner of the page reaches any
point of the screen. It took four attempts and the cause of the first three
was a single missing coordinate conversion —
`geometry::scroll_to_strip`, absent from `visible_rect`.

★★★ **The lesson, and it is the biggest one of the day: READ WHAT THE
APPLICATION SAYS ABOUT ITSELF, FIRST.** Every failing trace from the first
attempt onward carried

```
pdfcer-diag canvas-unavailable reason=nothing-visible
```

which states the cause exactly. It was never grepped for, because each search
was for the SYMPTOM — no pointer input, stalled frames, a page rect that
looked right — and those sent the diagnosis to the arithmetic, then the
allocation, then the seeding mechanism, then the offset magnitude. All four
were innocent. **Grep the trace for what the program is reporting before
grepping it for what you are seeing.**

### ⚠️ THE ONE FAILING CHECK, AND WHY IT IS NOT (EVIDENTLY) A REGRESSION

`multi_node_move_moves_every_picked_anchor` **FAILS**, reproducibly, in
isolation as well as in the suite.

On the baseline it **SKIPPED**, with *"the subpath has one anchor. Aim
--doc-point at a polyline."* With the pasteboard it finds **two** anchors,
clicks both — the second with Shift — and reports that **one** ends up
selected.

So the check now reaches a rung it could never reach before and fails there.
That is a different operand, not a changed behaviour, and the pasteboard is
the likely reason the operand changed: a different scroll position makes a
different page current in a continuous strip.

★★ **NARROWED 2026-08-21: the model is correct, so the fault is downstream.**
Two unit tests were written for the rung nothing had ever covered —
`shift_picking_a_second_anchor_adds_it_rather_than_replacing` and
`shift_picking_a_selected_anchor_removes_it` — and **both pass**.
`SelectionState::pick_within` adds a Shift-picked anchor and toggles one that
is already picked, and `normalise` only collapses entries that span different
objects, which two anchors on one subpath do not.

So it is **not a pasteboard regression** (the check had never run before) and
**not a selection-model defect** (now proven, and guarded). What is left is
the driven path: either the harness's Shift+click is not delivering the
modifier, or the shift is not reaching `pick_within` from the real event, or
the second click resolved to the same anchor. All three are in
`canvas::clicking` / the harness, not in the model.

★ Worth knowing before chasing it: the check has **never passed**, on any
build. There is no evidence this ever worked, so treat it as an unbuilt path
rather than a broken one.
### ★★★ AND DRIVING FOUND THE DEFECT KEN IS BLOCKED ON — `O22`

**An object near the top of the view cannot be rotated. Its handle is drawn
nine pixels above the canvas.** `rotate_handle_turns_a_selection` passes at
`0,300,500` and fails at `0,1211,1021`, while `resize_scales_a_shape` passes at
both. The numbers are in `O22`; the short version is that
`Grip::Rotate.anchor()` is `bounds.top() - 20.0`, and a selection flush with
the top of the viewport puts that outside the clip.

It is not about text, which is what `O20` assumed. **Any selection within 24 pt
of the top of the view**, whatever it is made of.

★ **The fix is settled and was attempted.** Ken answered the convention
question on 2026-08-21 and asked for more than `O22` proposed — any corner of
the page reachable to anywhere on screen — which is `O23`. The pasteboard was
built that evening, passed 1,634 tests and 17 gates, **broke selection on the
real application, and was reverted.** `O23` carries the three things measured
on the way. ⚠️ Its first diagnosis — *"the page jumps a frame after opening"* —
was **wrong and was never measured**; it compared two builds. Re-measured, the
pasteboard layout is *steadier* than today's (one stable rect against today's
two). What actually breaks is that **the canvas receives no pointer input at
all**, while the page is centred, visible and correctly drawn.

★★★ **Bisected to two literals.** It is not the arithmetic, not the
allocation, and not the seeding mechanism: `.scroll_offset(vec2(484, 492))` —
a hard-coded constant, no pasteboard anywhere — reproduces it, while
`vec2(100, 100)` does not. **A large applied scroll offset costs the canvas
its pointer input**, leaving layout, drawing and the published rects correct.

★ `O23` §3f names the one experiment that decides what this is: drive the
WHEEL to a similar offset on today's unmodified shell and click. If input dies
there too, this is a pre-existing defect the operator meets whenever they
scroll far, and it outranks O23 entirely. Run that before building anything.

⚠️ Do not plan around this row's first claim that the eight resize grips share
the defect. **They do not** — their centres sit ON the box edge, so their inner
half is always inside the canvas and always grabbable. Only the rotate handle's
centre is outside the box. The correction, with the geometry, is in `O22`.

★★ **The transferable half: one `--doc-point` passing is what hid this for a
day.** A driven check aimed at a single point proves the gesture works *there*.

### Two harness lessons from the same evening, both now in the RAG

Both new checks accused the application before they were right, and both times
the harness was at fault:

- one asked whether a `ui-rect` region had been published. That channel is a
  **change log**, so a region that stopped being drawn is still in the trace —
  it reported *"THE FILTER IS DECORATIVE"* about a build whose own trace said
  `sel=0`;
- the other watched `text-selection`, which no build has ever emitted (it is
  `canvas-text-selection`), and reported SKIPPED blaming the fixture.

**Read the trace before believing the check.**

And one that is about harness design: **a driven check that mutates persisted
state must establish that state at the START.** Restoring it at the end only
runs when the check passed, which is the case that did not need it — the filter
check failed at step 2, left every class switched off on disk, and its next run
blamed `--doc-point`.
---

## ⚠️ DRIVEN RUNS NEED AN ATTENDED MACHINE — found 2026-08-22

After several hours with no human input, **every driven check began reporting**

```
the window ... could not be brought to the front. Windows refuses
SetForegroundWindow to a process without foreground rights
```

No code change between the working runs and the failing ones; unit tests and
all 17 gates stayed green. `SetForegroundWindow` is granted only to a process
that is already foreground, received the last input, or has recent user input
behind it — and on an idle desktop a background harness has none of those.

★ **So the idle machine that looks like the ideal time to run the suite is the
one time it cannot run.** Schedule driven runs when he hands the machine over,
not overnight. And read a window-activation SKIP as an ENVIRONMENT verdict, not
an application one — it survives re-running, so it reads as a defect.

★ Clean up orphans first: a killed harness run leaves the application running,
and that instance competes for the foreground. `taskkill //IM pdfcer-gui.exe
//F`. Note `taskkill //PID` against a pid from `ps -W` fails — that column is
Git Bash's pid, not Windows'.

★★ **How much is left unattended, measured rather than assumed: `--no-input`
verifies 4 checks of 68.** Nineteen skip on "this check clicks a mode segment"
alone. An input-disabled run is not a reduced suite, it is a different and much
smaller thing.

The refinement worth having: the four that ran include capture and dialog
checks, so **capture does not need foreground rights — only synthesised input
does.** The harness is not disabled on an idle desktop; its clicking is.

So the unattended set is: unit tests, the gates, an offscreen launch
(`PDFCER_DIAG_VIEWPORT`) asserting on the trace, and those four. Everything that
asserts *a gesture produces a result* needs an attended machine.

## ★★ A RELEASE IS OWED WHEN O24's STEP 2 LANDS

Ken, 2026-08-21: *"when you complete the step 2 zoom release to git and put on
OneDrive."*

**`git push origin main`, then `tools/package-portable.py`.** ★ This would be
the project's **first push** — `origin` is `github.com/KenM76/pdfcer-gui.git` and
the local branch is 253 commits ahead of it, last tag `v0.3.0`. Not a routine
increment.

`O24`'s release section carries the five preconditions, every one of which has
bitten this project already. The two most easily forgotten: **bump the engine
first** (`O24` depends on two commits this lock predates, so a stale pin ships
a release without the thing it is a release of), and **drive the suite at BOTH
`--doc-point`s**, because one point passing is what hid `O22` for a day.

★ **Not on step 1.** He named the trigger precisely.

## 0. ★★★ READ THESE FIRST, EVERY SESSION

### `OPERATOR_REQUESTS.md` — the backlog, and the only truth about it

Every ask goes in that file **the moment it is made**. **Only Ken closes a
row.** A status is evidence or the words NOT VERIFIED. A blocked row names the
request file. Nothing is silently rescoped.

★ Row 13b is a **withdrawn** defect report — read it. It was written up in good
faith, it was wrong, and the retraction is left standing beside what it
retracts. That shape is the standard for this file.

### `D:\dev\rag\ui-conventions\` — and the gate behind it

Five gesture classes, each a numbered list carrying where the rule comes from
and the failure mode when it is absent. `tools/gates/check-conventions.sh`
makes every registered surface answer every row in its own source. **It cannot
check behaviour and does not pretend to** — it checks that the question was
asked, which is the whole of the problem.

Of the fourteen gaps its first run found, item 11 (no selection inside a text
draft) closed on 2026-08-21, keyboard and pointer. The rest are O14.

### `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\`

Read it every session; **empty means nothing is owed**. The engine session runs
in parallel and answers within minutes. The live entries are `note_*` replies
awaiting consumption, not asks.

---

## 1. What happened this session (2026-08-21)

Seven things shipped. In the order they matter to Ken:

1. **The navigation keys walk the page, not the fragment you clicked.** Down
   and Up move to the line below and above **including into the next block of
   text**; End and Home reach the ends of the line he can *see*, however many
   show operators drew it — four or five on a CAD title-block row. **Salvage**:
   the old shell asked `caret_up`, `caret_down` and `line_range_at`, and that
   was its entire contribution; the reassembly was always `pdfcer-core`'s.
2. **Every dialog is its own OS window**, not just Print. All thirteen: title
   bar, taskbar entry, second monitor.
3. **A dialog is OWNED by the window it belongs to** — G3, which had been filed
   as impossible. It can no longer fall behind the application.
4. **There is a selection inside a text draft.** Shift+arrows, Shift+Home/End,
   Ctrl+A, and — later the same day — **drag to select, double-click for a
   word**. Closes conventions item 11.
5. **A defect that had shipped: every dialog drew on a BLACK background.** Dark
   text on near-black. Nothing caught it — the windows opened, every control
   was where it said it was, the driven check for *"it opens in its own OS
   window"* passed on all eight. **A screenshot showed a black rectangle.**
6. **The harness learned the program has more than one window.** Six checks
   were failing and six skipping, every one clicking hundreds of pixels from
   the control it named.
7. **The engine moved 14 commits**, including `Pass 97` compositing work and
   `Pass 120.2/120.4`, the engine half of the clipboard.

### ★★ The four findings worth carrying forward

- **A guard that stops repetition does not stop creep.** `Host::fit` grows a
  dialog to fit its body; its first version padded the measurement before
  comparing, so every frame asked for eight more pixels than the last and the
  once-per-size guard never fired *because every size was a new one*. About
  opened at 560×480 and was **1624×746** a few frames later.
- **A `Key` event's own modifiers can be EMPTY while the frame's are right.**
  Measured: `ev=Modifiers::NONE frame=Modifiers { shift: true }`, three
  presses, Shift held throughout. The exact **inverse** of the chord-matcher
  finding filed three days earlier, and **both are true** — choose by what the
  modifier MEANS. A chord is a command and must use the event's alone; a held
  qualifier must use `event || frame`, because over-reading costs one step and
  under-reading destroys accumulated state.
- **A measurement of the wrong surface is indistinguishable from a measurement
  of a broken one.** Twice in one afternoon: a contrast check capturing the
  wrong WINDOW (1.51:1 reported about two headings that render at 15.07:1), and
  `ui_rect_visible` publishing the wrong PART of the right one (a heading two
  points inside a scroll area's bottom edge, measured off the anti-aliased top
  rows of clipped glyphs).
- **A measurement that moves when you turn a knob is not proof the knob is the
  subject.** See §1b.

### ⚠ §1b — What LOOKED broken and was not, which cost the most time

`text_annot_takes_the_keyboard_unclicked` failed for hours with *"drag out a
note box, type without clicking the field, and the words go nowhere"*. It was
written up for Ken as a live defect. **It was not one.**

The check clicked its Accept button through the APPLICATION window's
coordinates while the dialog had its own. The typing worked the whole time —
the dialog took the characters, the field held focus, the draft was right — and
the click that should have committed it landed on a page. One call site,
converted to `frame_of` like the other six, and it passes.

★★★ Chasing the wrong culprit, the program was changed four times to hold the
keyboard harder, and **every change appeared to help**: the dialog visibly held
the foreground while the new code was asking for it and lost it the instant it
stopped. Real, repeatable, beside the point. `FOCUS_FRAMES` went 1 → 8 → 40 →
120 on exactly that evidence and is back at 8, with the story written into its
doc comment so the next reader does not repeat it.

Two of the four changes were kept because they are right on their own terms:
**G3** (above) and **a dialog's position is asserted once**, on the pass it
opens, rather than re-asserted every frame from a value read back out of the
window one frame earlier.

---

## 1c. ★★ WHAT IS PUBLISHED AND NOT VERIFIED

Two things are in the operator's hands and have **not** been checked against a
running binary, because he came back to the keyboard and the harness takes the
cursor:

1. **Drag-select and double-click-a-word inside a text draft.** Unit-tested
   against the real galley, three tests. The driven step is written — step 6 of
   `shift_arrows_select_text` — and has never been run.
2. **The engine, 14 commits forward** to `cbb1ede`. Four are `Pass 97`: the
   compositing formula, non-isolated group backdrops, knockout groups, and soft
   masks applied once per group instead of once per object. **That is the class
   of change that alters how every page rasterizes.**

**They are published anyway, and that is his instruction:**

> *"no it doesn't matter if it has been checked or not. I always want the
> latest build there."* — 2026-08-21

★★ **Understand the correction, do not merely obey it.** A release was held
back earlier the same day on exactly the opposite reasoning, and the reason
that was wrong is already built into the tool: **the other slot holds the
previous build.** He has a fallback by construction, so the cost of a bad build
is a folder swap — while the cost of withholding is that he does not have the
work at all. Driven verification gates *claiming a feature works*, not *putting
the binary where he can reach it*. The two were being conflated.

Disclosure moves rather than disappears: it goes in the report and in the
build's own `BUILD-INFO.txt` (`--note`).

---

## 2. What to do next

His standing instruction is *"continue looping through other tasks"*. In the
order that returns the most:

1. ★★★ **Fix `O22` — the rotate handle is off-canvas near the top of the
   view.** This is what Ken is blocked on, it is confirmed by driving with
   numbers, and it is the only item here that a person is currently waiting on.

   ★ Ken settled the approach on 2026-08-21 and asked for MORE than was
   proposed — `O23` supersedes the fix half of this row. It was attempted the
   same evening, broke selection, and was reverted; `O23` carries the three
   things that were measured, and the one that stopped it is that the page
   JUMPS a frame after opening. Solve the seeding before rebuilding the rest.

   ⚠️ And do not plan around this row's old claim that the resize grips share
   the defect — **they do not**, and the correction is in `O22`. Their centres
   sit ON the box edge; only the rotate handle's is outside it.

   ★ Re-run `rotate_handle_turns_a_selection` at **both** `--doc-point`s
   afterwards. One point passing is what hid this.
2. **Run the full driven suite**, and fix whatever the engine
   bump moved:
   ```
   ./target/release/ui-verify.exe --pdf D:/Dev/temp/pdfcer/SW41177.pdf \
       --doc-point 0,300,500
   ```
   Not to gate a release — that has already happened — but because he is
   running unverified code and the sooner that stops being true the better.
   **`0,300,500` is the calibrated point**; `0,1211,1021` aims at a BOM row and
   is right for the text checks and wrong for `rotate_handle_turns_a_selection`.
3. **The three gesture-only dialogs nobody has driven** — Insert pages, Set
   scale, and the unsaved-changes question. They are OS windows now and nothing
   has clicked them. `frame_of` and the driver's focus tracking are in place, so
   each is a check rather than an investigation.
4. **`Pass 120.2/120.4` is in the tree now** — selection to a standalone
   one-page PDF, and the clipboard's cross-application half. That closes
   `OPERATOR_REQUESTS.md` O2's remaining rows; the shell side is a paste target
   and a private Windows clipboard format.
5. **The rest of O14**: unfilled-shape hit testing (only ce dimensions carry a
   real shape), grapheme clusters in the caret, right-click to add or remove a
   perimeter point (both engine verbs exist), the zero-travel guard on three of
   four drag paths.
6. **The transform preflight**, a named gap in `canvas::resizing`: an object
   whose own CTM is singular cannot be transformed and the engine says *do not
   offer a handle*. `transform_preview` is the predicate and it decomposes the
   page, so it needs a cache keyed on `(page, epoch, selection)` shaped like
   `app::cache::FormRunCache`.
7. **Turning existing page text into multiple lines** — O15's remainder. That
   is a reflow, which the engine has and which currently demands the document
   be saved and reopened first.

---

## 3. Blocked on the engine

Nothing on the list above is engine-blocked. The two that were —
`transform_objects` and the object clipboard — both shipped.

The live entries in the request channel are `note_*` **replies awaiting
consumption**, not asks. Read them; several report that a blocker this project
filed was never real.

---

## 4. Environment gotchas

- **`ui-verify` takes the real cursor and keyboard.** If Ken is at the machine,
  every check that clicks will SKIP with a foreground refusal — which is
  correct, not a failure. **51 of 65 skipped once this session** for that.
- **★★ Do not edit source while the suite runs.** The staleness guard fires and
  every check refuses: a file edited mid-run makes every trace describe code
  that is not the code under test. One or the other, never both.
- **★ `--second-pdf` must have MORE THAN ONE PAGE.** A one-page source cannot
  be moved out of — the engine refuses to leave a document with no pages.
  `D:/Dev/temp/pdfcer/big.pdf` (5 pages).
- **★ Python heredocs eat the `\` continuation in a Rust string literal**, and
  the result COMPILES: what lands on disk is one long line with the indentation
  baked into the string. `.tmpwork/rewrap.py` repairs a file after the fact and
  `check-string-gaps` catches it at the gate. Use the Edit tool for anything
  with a continuation, or `r"""…"""`, or `chr(92)`.
- **`PDFCER_DIAG_INVOKE=<command.id>` presses one ribbon command once**, in an
  invisible window, through the real dispatcher — the way to verify while he is
  working:
  ```
  PDFCER_DIAG=1 PDFCER_DIAG_VIEWPORT=-4000,-4000,1200,850 \
  PDFCER_DIAG_INVOKE=file.print  target/release/pdfcer-gui.exe file.pdf
  ```
  `dialogs_open_in_their_own_window` is built entirely on this and needs no
  pointer at all.
- **`osk.exe` covers the ribbon and swallows synthetic clicks**, UIPI-protected.
  A driven failure on this machine is a harness question before it is an
  application one.
- **`python tools/package-portable.py --verify` after every keeper build**, and
  read the two slot dates it prints. **`--slot <name>` forces the target.**
  ★ `pdfcer-gui1` refused the mirror **three times on 2026-08-21** — `WinError 32`
  on the rename, with no process running from that folder, so it is OneDrive's
  own sync client. The failure is safe (a failed rename moves nothing) and the
  cost is that the fallback stops rotating. **If it happens again, find out what
  is holding it.**
- **`cargo update -p pdfcer-core -p pdfcer-render -p pdfcer-print` before every
  build.** The packager does it by default; `--no-update` holds a pin.
- `.tmpwork/edit.py` is the CRLF-safe edit helper.
- **Never `git checkout --` a dirty file.**

---

## 5. Standing rules this project has paid for

- **A trace can say the verb ran. It cannot say the screen changed.** Every
  layout, repaint or clipping defect has exactly one oracle: a rendered
  screenshot. Put a capture on the failure branch of anything that draws. The
  black-dialog defect (§1.5) is the newest instance and the most complete: four
  other oracles said the surface was perfect.
- **A check asserting on an ABSENT line must first ask what else happened.**
- **A fixture that cannot exhibit the hazard proves nothing.** When an upstream
  fix stops your falsifier firing, the assertions beside it stop measuring and
  go on passing. Invert the control or grow the fixture; never just delete it.
- **Two derivations of one position agree at first and separate under use.**
  Five instances now. `egui::Pos2` is screen, canvas, page AND per-viewport
  space, so the compiler cannot object. The newest fix is the right shape:
  `canvas::textedit::hit` publishes the galley that was **drawn**, so the
  pointer hit-tests the same layout the caret is painted from — one derivation,
  two questions.
- **A blocker is a measurement, and the question you measured is part of it.**
- **A predicate with two claimants must exist exactly once.**
- **A knob must not sit at a value chosen to fix something it does not fix.**
- **Registering a command is the only way the GUI may learn a capability
  exists** (R8), and **`egui-shell` never learns what a PDF is** (R7).
- **Unsafe code is quarantined.** `pdfcer-gui` and `egui-shell` both
  `#![forbid(unsafe_code)]`; the four `user32` calls that make a dialog owned
  live in their own `native-window` crate, to be deleted rather than ported
  when a toolkit grows an owner option.

---

## 6. His standing criticism — keep it in view

> *"it shouldn't take multiple 3 hour sessions each day to figure out how to get
> a cursor to move and edit text on it, or get shortcuts to work for basic
> functions."*

The largest bucket of this fortnight's defects is **conventions nobody
audited** — not engine gaps and not hard problems. The conventions corpus and
its gate are the structural answer; use them **before** building an interaction,
not after he reports it.

And when he reports something, **believe him and go find it.** Every report this
fortnight was precise and correct, including the ones that sounded at first like
misunderstandings — most recently *"text editing doesn't work"*, which was true
for 99 % of the text on his documents while every driven check was green,
**because the checks drove fixtures this repository authored.**

★ The converse arrived this session and is the harder discipline: **when *you*
report something broken, hold it to the same standard.** Row 13b was a defect
report written from a failing check, and the check was wrong. Before writing up
a defect, prove the measurement was of the thing you named.
