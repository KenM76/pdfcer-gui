# HANDOFF — the long-form record

> # ★★★ START AT `RESUME.md`, NOT HERE.
>
> `RESUME.md` carries the measured state, what to do next, and what not to do.
> It is one screen and it is current.
>
> **This file is the institutional record**: the standing rules, the phase
> order, the six obligations of registering a command, and every finding this
> project has accumulated. It is authoritative and it is long, and its §1
> status table has been superseded four times — which is exactly why the
> resume point moved out of it.
>
> Read this one when `RESUME.md` points you at a section, or when you need the
> reasoning behind a rule rather than the rule.

## 1. Where things stand, in one screen

> ### ★★ 2026-08-18, at `ed58337` — read `RESUME.md` for the live numbers
>
> The tables below are a **layered record**, each dated, each superseded. Do
> not read down them looking for current state; `RESUME.md`'s one table is the
> current state and it is re-measured, not edited.
>
> What this section is still for is the *history* of a number, which turns out
> to matter: on 2026-08-18 `catalog.rs`'s icon-coverage heading was found
> claiming *86 of 101 named, 15 refused* against a registry of 94 — the
> **fifth** drift of that pair, and the test added after the fourth had not
> caught it, because it pins the split against its own literals and the
> sentence was never one of them. That heading now carries no numbers.
>
> Landed this day: the print dialog's **paper list, driver Properties… button
> and restored tray control**, and `file.new_from_template`'s **page-size
> chooser**. ★ **Neither has been driven** — `ui-verify` needs the operator's
> desktop and the two checks are written and unrun. Under §2 that means both
> are *implemented and unverified*, and `RESUME.md` says so in those words.


| | |
|---|---|
| **Shell HEAD** | `3e27b79` — clean tree |
| **Engine HEAD** | `D:\Dev\pdfcer` at `b943ea1` |
| **Tests** | 1,184 passing, 0 failing |
| **Gates** | 8 of 8, 0 skipped |
| **Commands** | 90 registered · 88 declared-and-deferred |
| **Latest build** | `D:\builds\pdfcergui-20260813-2248-b943ea1-a748414\` |
| **Requests owed by pdfcer** | **none.** Four filed and answered on 2026-08-14 — revision clouds, markup note text, markup opacity, tab-order authoring. Three are **accepted and scheduled, none started**; they block *items within* Phase 6, not Phase 6 itself. From the fourth, **`widget_rects(page_index)` shipped immediately** (engine `e8e9881`) and canvas form filling uses it. Tab-order *writing* stays blocked on their F4 — see §8. |

> **★ The table above is from 2026-08-13 and is superseded three times over.**
> A 2026-08-14 session landed the Read-mode gate and Phase 7; a
> 2026-08-15 session landed Phase 5's one-run text editor. Measured at
> the end of the latter: **1,744 tests passing, 0 failing · 10 of 10
> gates, 0 skipped · 101 commands registered · 31 groups · 31
> scaffolded.** `FEATURES.md`'s own header carries the same figures.
> Re-measure rather than quoting either table if any time has passed;
> the numbers above are the ones a test pins, and prose drifting from
> them is a defect this project has now had four times.
>
> **Measured 2026-08-17 at `8931f0b`: 1,818 tests passing, 0 failing ·
> 12 of 12 gates, 0 skipped.** Note the gate count is **12**, not the 8
> the table says and not the 10 the paragraph above says — it has grown
> twice since either was written, which is the same drift again. Run
> `bash tools/gates/run-all.sh` and read the summary rather than
> believing any of the three numbers on this screen.

### ★★★ 2026-08-17 evening — the machine was freed, and everything was driven

The morning's four commits landed with the operator using the PC, so nothing
was verified in pixels. That queue is now **worked through**, and driving it
found more than it confirmed. Measured at `a709afc`:

| | |
|---|---|
| **`ui-verify`** | **22 passed · 0 failed · 3 skipped** (was 19/1/4) |
| **Tests** | 1,823 passing, 0 failing |
| **Gates** | 12 of 12, 0 skipped |
| **Engine** | `D:\Dev\pdfcer` local `main`, via `git = "file://…", branch = "main"` |

**Three harness defects, each producing a confident wrong answer, and each
worth more than the check that found it.**

1. **`delete_key` was reporting the Read-mode gate as a selection defect.** It
   clicked page content and asserted a selection appeared, having never set the
   mode — and the shell's **default mode is Read**, where that click is
   correctly refused. What gave it away was not reading code: **six doc-points
   spread across a dense CAD sheet all reported `hit 0 object(s)`.** A hit test
   that misses everywhere is not a hit test, it is a gate. Two checks in this
   suite were asserting **opposite** things about the same gesture and only one
   established the mode.

2. **Chords never worked, for the whole life of the project, and the recorded
   diagnosis was too broad.** §8 said *"synthetic keyboard input does not reach
   the target window"*. It does — the same API delivers a plain Delete. Only
   **modified** chords failed, because `keybd_event` posts asynchronously and an
   egui app drains the queue once per frame: modifier-down and key-down in the
   same microsecond give it no frame in which the modifier is held and the key
   is not, so `Ctrl+2` arrives as a bare `2`. **Three 12 ms sleeps.**
   `find_opens_and_finds`, which had never passed here, now does.

3. **The `ui-rect` trace is a CHANGE LOG and could not say a control had
   stopped being drawn.** It emits only when a rect *moves*, so a control
   swallowed by the ribbon overflow left its last rect standing forever. That
   made the new UI-scale check report **18 controls as laid out outside the
   window**; all 18 were fine and the screenshot showed a clean ribbon with a
   *5 more* button. Fixed at the source: the application now emits
   `ui-rect-gone name=…` at frame end and forgets the retired rect. **Every
   region-based check in the suite is sounder for it.**

**And one I caused by arguing myself into it**, which is the one to read
twice. The UI-scale check writes `preferences.txt` and I left it holding 1.8
"on purpose", reasoning that tidying up would hide inherited state. Next full
run: **3 passed, 1 failed, 21 skipped.** Twenty-one checks could not begin,
because they are written against an 1100 pt window and met a 611 pt one. The
distinction I had missed is **who owns the state**: `layout.ron` and
`recent.txt` are written by the *application* as a consequence of being driven
and erasing them would hide what driving it does; `preferences.txt` here is an
**input the harness injects**, and an injected input is one it owes the suite
a return to neutral on. Restored via a `Drop` guard, because that function has
eleven returns.

**What the driving confirmed about the application**, as opposed to the
harness: UI scale is correct and shipping-quality — client area 1100→611 pt at
1.8× exactly, all measured chrome at ×1.80 of the window, all 68 declared
regions inside it, and the 180 % screenshot fully legible. One real app defect
fell out: a scaled profile **flashed at 1.0 for the first frame**, now fixed by
applying the scale in `lib.rs` before the first frame and tracing
`ui-scale-initial`.

**Known limit, not yet a defect worth fixing:** at 1.8× on the shipped
1100×800 window the two default dock panels take 550 of 611 pt and the canvas
is squeezed to a sliver at 10 % zoom. Correct behaviour from every component —
the docks are sized in points and points got bigger — but the *composition* is
poor, and there is no minimum-canvas guard. On a maximised window it does not
arise. Worth a `MODES_AND_PANELS.md` decision before anyone calls it a bug.

**Still not driven:** the opening-view trio and the freehand-at-0.25 pt stroke.
Both have their assertion shapes ready (`opening-view` and
`markup-commit raw=/kept=` are live trace lines) and neither has a check yet.

### ★★ 2026-08-17 — the operator's report, and what it turned up

> *"I tried a lot of the features that have been added only to find there is no
> surface for changing or editing the settings for them. please add the ones
> that are missing for all of the features currently supported in the gui. also
> port the settings dialog from the pdfcer gui. also the print dialogue didn't
> work."*

Three commits. Every one of them found something worse than the thing reported,
and all three failures are the **same shape** — wiring that no test in the
workspace could see:

| # | reported | found |
|---|---|---|
| `6d790db` | print didn't work | `pdfcer-print` was linked into every shipped binary and the adapter's four calls still returned `NotLinked`. **A green test held it there**: `every_hole_refuses_rather_than_guessing` asserted all four refused, which was correct while unlinked and became a lock the moment it was not — doing the right thing would have turned the suite red |
| `87b4f3d` | no settings surface | `file.settings` inert for the whole project — and, measured against the old shell, **nine of its thirteen settings were persisted, shown, edited and never read by anything**, discarded at every call site that built its own option struct. `app::settings` is now the one funnel and a `syn` check enforces it. D10's second half closed, proved in pixels |
| `4035b64` | can't change a markup's colour | Markup ▸ Style declared `Item::custom("colour_swatch")` since S2 and **no renderer ever matched the kind**, so the group shipped as a caption over an empty band. The manifest test asserted the item was *declared* and passed correctly |

**The generalisation is worth carrying**: there are now three distinct ways for
a control to ship inert, and each defeats a different guard.

1. **A command with no dispatch arm.** Caught by `shell::commands::reach`.
2. **A linked crate with a refusing adapter.** Caught by nothing — the
   adapter's own tests asserted the refusal.
3. **A declared `Item::Custom` with no renderer.** Caught by nothing, and it is
   the quietest: a `Custom` item carries no command id, so it is invisible to
   every check built on `command_references()`, and the manifest test that
   *does* see it is asserting the manifest, which was right.

Only one oracle sees all three: driving the binary and reading what it declared.

**D11 was also broken again, three days after being written, by someone who had
read it** — the settings window's headings used `.strong()` and rendered pale
grey on pale grey. `tools/gates/check-strong-text.sh` now enforces it, found a
third latent instance in `egui-shell` on its first run, and had to learn to
measure its window in *code* lines rather than source lines so a well-commented
fix would not fail a gate a terse one passes.

### ★ The three follow-ups, resolved — two built, one measured and blocked

| | outcome |
|---|---|
| **Measure ▸ Set scale** | ✅ `980971f`. The model was salvaged whole in Phase 7 and only a window was missing. **Manage groups stays deferred and its reason was rewritten**: it waited on "the same absent dialog", the dialog landed, and the entry stays — because rename and delete are *not in the shipped `EditSession` surface* and a management window missing half its verbs is worse than none |
| **The seven `view.*` settings** | ✅ `29cdc31`. **Two built, five deleted.** Four named capabilities that do not exist; the fifth, `app_initiative`, existed to switch off a behaviour pdfcer does not have. All seven unregistered on R8, the empty Render group deleted, 32 groups → 31 |
| **The Format contextual tab** | ⛔ `3784cca`. **Blocked, not unbuilt** — see below |

**The Format tab is the finding worth carrying.** §5.8 specifies twenty-four
property editors across six selection types and the tab can carry **one**,
`Delete`, which it already has. Two independent blockers:

1. **`EditSession` has no verb that modifies an annotation.** `add_markup`,
   `add_text_annotation`, `delete_annotation` and two deletion predicates is the
   whole surface. Delete-and-re-add is not a workaround — it loses the
   annotation's identity and with it its `/NM`, its z-order in `/Annots`, and
   any reply thread hung off it as `/IRT`. The **one** exception is the ce
   dimension row: dimensions have a style model and nothing else does.
2. **The canvas selection cannot address an annotation.** `Selection` is
   `page + object + subpath + node`, a paint-order index into page *content* —
   which is what makes it immune to zoom, and also means a markup or a dimension
   is not selectable at all.

The second is ours. The first is filed, along with a request for an
`annotation_at(page, point)` sibling of the `widget_rects` query that unblocked
canvas form filling — the exact precedent, and it worked.

### Four requests open to pdfcer, all filed 2026-08-17

`open/` was empty before this session. It now holds four, three of them from
one operator question about print:

| request | finding |
|---|---|
| `devicesettings_pick_tray_is_never_read` | the field is declared, documented, plumbed through `spool`'s signature, and **read nowhere**. The GUI shipped a checkbox for it; the checkbox is removed |
| `orientation_auto_is_per_job_not_per_page` | documented as per-page in a heading that says so, implemented per-job — `build_devmode` is called once with `first_page_pt` |
| `no_paper_size_selection_in_the_print_path` | no paper list, no way to request one, no route to the driver's properties dialog. The dialog now **discloses** which paper the job is planned against and that pdfcer cannot change it |
| `no_verb_modifies_an_existing_annotation` | the Format-tab blocker above |

### …and three more on 2026-08-17, so `open/` holds SEVEN

All three came out of working through `NO_SURFACE.md`'s rows, and **all three
are the same shape**: a GUI surface that looked unbuilt turned out to be an
engine capability that is absent, or present-and-unreachable.

| request | finding |
|---|---|
| `redaction_fill_is_unreachable_from_the_search_path` | `/IC` is written, read and painted — but `EditSession::author_text_matches` hard-codes `fill: None` (`edit.rs:11719`), so no caller can set it on a mark from *Find and mark*. A swatch would be honoured on whole-page marks and silently dropped on searched ones |
| `redaction_overlay_text_is_authored_and_never_drawn` | `/OverlayText` is **written into the PDF and never read**. `gather_page` does not look at it, `build_overlay` draws filled boxes only, and the annotation carrying the string is deleted at apply. Type *REDACTED*, get plain black boxes, no report row. **The sharpest form of it is the disclosure, not the paint**: `ARCHITECTURE.md` says the deferral is *"disclosed at mark time"* and there is no mechanism in the API that discloses it at mark time or ever — so a shell reading only the API cannot know to say anything |
| `no_verb_sets_a_pages_media_box` | nothing in `pdfcer-core` writes a `/MediaBox`, so `file.new` can only ever be A4 without ten template assets — and still no custom size. Priority **low**, stated as such |

#### ★★ Two pdfcer-gui sessions were running at once, and both filed the same request

The overlay-text finding was reached **independently by two sessions four
minutes apart** on 2026-08-17, and filed twice. The duplicate
(`request_overlay_text_is_recorded_and_then_dropped.md`) has been **withdrawn**
and its one unique contribution folded into the surviving file, which was both
earlier and better — it identified that the real gap is the missing
**disclosure** rather than the missing paint, which the duplicate did not.

Three things to carry:

1. **The channel's `open/` is shared mutable state and nothing locks it.** The
   README's *"one topic per file"* rule is written against a file carrying
   several asks; this was the mirror failure, several files carrying one ask,
   and it costs the same in triage — the reader answers one and the other
   lingers looking unanswered. **List `open/` again immediately before writing
   into it**, not only at session start.
2. **The other session found things this one missed**, and the difference was
   not effort: it read `carrier_detect_disclose` and `ARCHITECTURE.md` as well
   as the code path, and found a `/IC` **non-RGB** defect that is a third
   distinct problem in the same struct. Converging on the same file is
   evidence the finding is real; the *differences* between two independent
   write-ups are where the value is.
3. **Withdraw your own duplicate rather than the other's**, and say in the
   surviving file that you did. A deletion nobody records reads, to the next
   session, as a request that was never filed.

**★ The pattern across all three, worth carrying into the next sweep.** An
engine field that exists, is documented, and is even **written into the file**
is not evidence that anything reads it. Two of these three reach the PDF today.
Distinguishing *supported* from *accepted and discarded* takes following the
value to its **consumer** — which is §10's *"registration is not
implementation"* one layer down, and it is why a `NO_SURFACE.md` row is never
automatically a build-the-surface task.

### ★ The print set, put through the channel 2026-08-18 and being worked in parallel

The operator reported the print gaps a **third** time — *"pretty much every
program I have ever seen lets you press a properties button beside the selected
printer"* — and instructed that the requests be worked by the pdfcer session in
parallel rather than queued.

**The root cause is one design choice, and finding it took three passes.**
`pdfcer-print`'s `build_devmode` (`lib.rs:2188`) **synthesises a DEVMODE from
zero** — `DEVMODEW { dmSize, ..Default::default() }` — sets the two fields
pdfcer knows about, and hands that to `CreateDC`. It never asks the driver for
one. That single fact explains all three asks at once:

* **no sheet size**, because there is no DEVMODE to put a `dmPaperSize` *in*;
* **no properties dialog**, because `DocumentProperties` *returns* a fully
  populated driver DEVMODE and `spool` builds its own and accepts none — so
  whatever the operator configured is discarded;
* **no per-page paper**, for the same reason plus the two siblings.

The ask is therefore ONE change: start from the driver's DEVMODE and let a
caller supply one. The request was restructured to lead with that, the two
earlier symptom-level framings are kept as an appendix (they show why the first
two answers were wrong), and the three print requests are cross-linked with a
standing note in the channel README.

**★ The boundary question is worth carrying**, because it was genuinely
arguable and the answer generalises. `pdfcer-gui` links `pdfcer-print` and has a
window handle, so it *could* call `DocumentProperties` itself. It must not: the
dialog's **output** is a DEVMODE, and a DEVMODE is meaningless to anything but
`spool`. A GUI that opened the dialog and could not hand the result anywhere
would be rebuilding the `pick_tray` defect **deliberately** — an operator
configuring settings that are then thrown away. And `pdfcer` prints too, so
paper selection living only in the GUI is the same boundary error mirrored.

The test that decided it: **not "is this a UI call?" but "where does its output
have to go?"** A UI call whose result only one crate can consume belongs beside
that crate.

Nothing is built on this side and nothing should be. The GUI half is one
dropdown and one **Properties...** button beside the printer combo, and under
R9 neither can honestly exist until there is something behind them.

### `NO_SURFACE.md` — the inventory, kept

Everything below is condensed from it. The full list, with `file:line` for every
row, is in **`NO_SURFACE.md`** at the repo root: every tunable in the crate an
operator would plausibly want to change and cannot, plus the 22 registered
commands with no dispatch arm and the recorded reason for each.

It exists because an inventory that lives only in a session is one somebody
re-derives, and this one took a long sweep to produce. It also carries its own
correction: its headline finding — *"Markup ▸ Style renders an empty captioned
band"* — was fixed while the sweep was running, and the row is kept struck
through rather than deleted, because the **shape** of that failure recurs.

### What is still missing, from the inventory sweep

**★ This list was worked through on 2026-08-17 and four of its five rows turned
out to be something other than what they said.** The corrected list is below;
the original is kept in each row because *the shape of the mis-reading is the
reusable part*, and it recurs.

| the sweep said | what it actually was |
|---|---|
| **Redaction** — fill, overlay text, quadding are three `None`s, *"the engine takes all three"* | ⛔ **Blocked, both of them, and not visible from the type.** `fill` is honoured only when the GUI builds the spec — `EditSession::author_text_matches` hard-codes `fill: None` at `edit.rs:11719`, so every mark from *Find and mark* ignores it. `overlay_text` is **written into the PDF and never read**: `gather_page` does not look at `/OverlayText` and the annotation carrying it is deleted at apply. Two requests filed; **no control shipped**, because a half-honoured setting on the one irreversible operation is worse than none |
| **Markup** — arrowhead length and angle | ⛔ **A category error.** They are screen-space **cursor** constants and `band.rs` says so: *"this is not a promise about that size, it is a statement about direction."* The annotation's own `/LE` head is the engine's. A control would read back a value and change nothing about the authored arrow. `RIBBON_IA.md`'s Format ▸ Arrowheads means the `/LE` style, which is a different thing entirely |
| **Markup** — ink simplification tolerance | ✅ **Not a missing setting — a live defect, fixed.** It was a `const` derived from the pen's *default* width, and the pen became a control on 2026-08-17. At 0.25 pt the fixed tolerance was **4× the stroke's half-width**, so the simplification could author a curve the operator did not draw. Now `Pen::simplify_tolerance_pts`, asserted at both ends of the range |
| **Snap and drafting** — *"zoom min/max, default fit mode …"* | ⬜ **Partly closed.** Default fit mode and the rulers/grid/guides default visibility shipped as `app::prefs::opening` on 2026-08-17. Snap tolerance (10 px), selection tolerance (6 px), zoom min/max and the alphas are still open and still preference-shaped |
| **No UI-scale or base-font-size control anywhere** | ✅ **Built 2026-08-17** — `Settings ▸ Appearance`, a multiplier on the OS setting, live-previewed like the theme. It was not a hard-coded constant: `ctx.set_zoom_factor` was never called at all. **`egui` ships a built-in `Ctrl` `+`/`-`/`0` handler for it, and this shell switches that off** (`app::configure_context`) because those chords mean *page* zoom — so the one path that would have surfaced the gap was closed for a good reason, and closing it removed a capability nobody had decided to have |
| **New document** — always A4, on a machine whose drawings are A1 and A3 | ⛔ **Blocked on the engine, and this is the one that looked most like an ordinary GUI task.** **Nothing in `pdfcer-core` writes a `/MediaBox`** — verified three ways; `EditSession`'s page verbs are rotate, delete and reorder. So the only shell-side implementation is one checked-in template per size, which is **ten** once landscape is counted and still cannot answer a custom size. Filed as `request_no_verb_sets_a_pages_media_box.md` (priority **low** — nobody drafts a sheet in pdfcer) and **neither implementation built**, because ten assets that still fail the custom case is a half-capability that looks like progress and forecloses the real fix |

**★ A framework default you switch off may have been carrying a capability you
never decided to have.** That is the UI-scale row's lesson and it generalises:
the audit that finds these looks for *constants*, and this one was an absence.
Worth a sweep of everything `configure_context` and the theme layer disable.

**★ The generalisation, which is the point of keeping this table.** A row in
`NO_SURFACE.md` says *a value is hard-coded*. It does **not** say the value
should become a control, and it does not say the value is correct. Of the four
worked through, one had to stay hard-coded, one was a bug rather than a gap,
one was blocked in the engine in a way the type signature actively conceals,
and only part of one was the ordinary "build the surface" task it looked like.
**The only way to tell is to read what consumes the value** — the same move
that separates a registered command from an implemented one.

---

**Everything the operator asked for is shipped.** Phase 3 and Phase 4 are
complete, along with Print, Forms-fill, Icons, Open/Recent/Close and Find.
Every loose end the build agents reported has been closed.

**The operator gave the order on 2026-08-14**, and it is:

> **Phase 6 (markup) → Phase 7 (measure) → the three small unblocked items
> → OCR → Phase 5 (text editing).**

### ★ Where that order has got to, as of 2026-08-14 (second session)

| | |
|---|---|
| **Read mode is genuinely read-only** | Asked for mid-session and built. Capability is derived from the **mode's tab list in the manifest**, never from the string `"read"`, so the ribbon and the canvas read one sentence. Closes **`DEFECTS.md` D6**. Proven by `ui-verify` driving the real window, not by tests. |
| **Phase 7 — measure** | **The salvage landed 2026-08-14 and three tools place dimensions**: Linear (three clicks — what, to what, where), Two-line, and **Radius / diameter**. `measure_tool.rs` came across whole into `canvas/measure/{pick,scale,state}.rs`, the 12.M1 snap primitives into `canvas/snap.rs`, 45 tests carried, **no engine API had moved**. ★ **The radius/diameter blocker is closed, by operator decision.** This row used to say the gesture had no natural end and the only place to say "done" was an accept box decision 024 retired — true, and the operator's answer on 2026-08-14 was to give it **two** endings that are not boxes: a **double-click** on the canvas and a registered **`measure.finish`** command, both routed through one commit path in `canvas/measure/circular.rs` so they cannot author different dimensions. Finish is gated on a new condition, `measure.finishable` (the tool armed *and* a non-degenerate fit), because a Finish that is always enabled is a control that does nothing on almost every press. The snap query is also wired now. What remains: **Set scale** still has no dialog to ask the length in; Area and Count still need engine changes; Angular is core-complete with no tool. See `SALVAGE.md`'s Phase 7 entry for the three deliberate departures from the source and the axis collision it surfaced. |
| **The three small unblocked items** | Two are done — the **edit-disclosure surface** and the **chord/mode gate**. Panel toggles are the third and the operator has chosen the semantics. |
| **Four operator decisions taken** | 2026-08-14: chords gate on tab membership; radius/diameter gets **both** a Finish command and a double-click; an open panel's control **closes** it; and `⚠` was to be fixed by **adding font coverage**. |
| **★ …and the fourth decision was answered by a measurement instead** | The operator chose to bundle a font for `⚠`. **No font was needed: `⚠` was never missing and renders correctly.** The broken thing was the *gate's predicate* — `epaint`'s `Fonts::has_glyph` asks "is this drawn by a face other than the one supplying the substitution mark?", so it answers `false` for every codepoint whose first supporting face is that one. The unanswerable demonstration is that **`has_glyph(Monospace, 'A')` is `false`**. So `DEFECTS.md` D12's measured lists were an artefact of the instrument, and four of its thirteen "shipped tofu" sentences were fine all along. **No dependency, no font data, zero added bytes.** The lesson is the general one: *a measurement is only as good as the predicate behind it*, and D12 is rewritten with the wrong claim kept visible. The gate was then widened to every `text/` module and **found two real tofu boxes on its first run** — both now fixed; see D12. |

**Two taxonomy questions were open and are the operator's**, both of the same
shape as the `edit.form_fill` → `view.panel_forms` move that is already in
this file. ★ **The first is now closed — answered and shipped on 2026-08-14 —
and is kept here struck through rather than deleted**, because the pattern is
the useful part: a chord refused in a mode where the operator plainly needs it
is evidence that the *command's tab* is wrong, not that the gate needs an
exception. That is twice this has happened and twice the fix has been a tab
move.

1. ~~**`edit.copy_page_text` sits on the Edit tab**~~ — **CLOSED by the
   operator on 2026-08-14: the destination is File ▸ Export.** The question
   was that `Ctrl+Shift+C` was refused in Read while Acrobat Reader copies
   text, which is the standard Read is measured against. Copying is not
   authoring, so both verbs left the authoring tab: `edit.copy_page_text` →
   **`file.copy_page_text`**, `edit.copy_document_text` →
   **`file.copy_document_text`**, tokens 122 and 123, the chord following the
   command in the manifest keymap. Export rather than a new Clipboard band,
   because an export is content written out to somewhere that is not this
   document and the destination — clipboard rather than path — is what the
   label says. **Edit ▸ Clipboard was deleted rather than shipped empty**, so
   the group count is **31**, not 32; that number is quoted in six places and
   all six moved together. One test now stands under the restored property,
   `both_text_copy_commands_are_offered_by_every_mode`, because nothing else
   would notice a revert. (At the time that test was written neither command
   had a dispatch arm; **both were wired on 2026-08-14** by the canvas
   text-selection work, and both read the same page extraction the canvas
   does, so ribbon-copy and selection-copy cannot disagree.)
2. ~~**Worded decline**~~ **— CLOSED 2026-08-14, built as specified.** `ZoomOutcome::NoBounds`/`NoCanvas` are worded in the status bar through `app/status/decline.rs`; the ceiling-clamped region zoom is deliberately left unworded as a partial grant. One thing came back with it: no chord binds `view.zoom_selection`, and its ribbon control is greyed exactly when it would decline. **Settled 2026-08-14 under the reference-application instruction** (§3 item 4): SolidWorks and Acrobat both reach zoom-to-selection by right-click and only Inkscape binds a key, so it joined the `canvas.object` context menu and **no chord was invented** — Inkscape's key is a bare digit, this shell's chords are `Ctrl`-modified by construction, and `Ctrl+1/2/3` are the mode selector. A menu on an object implies a selection, so the decline sentence stays **race-only**, which is the right shape: it is a safety net for the case where bounds evaporate between the frame that drew the enabled control and the frame that applied it.

Phase 5 is therefore **last**, not next — which is worth stating plainly,
because it is the defect that began this project (*"text editing is weird
and doesn't just edit the existing box and move the text correctly as you
type plus flow to the next line doesn't work"*) and every earlier version of
this file treated it as the obvious next move. It is not. Do not start it
early.

Two things that order does not tell you, and that cost a day to find out:

- **Phase 6 and Phase 7 are both bigger than their rows implied.** Neither
  is "add kinds"; both begin by building a canvas tool substrate this shell
  does not have. See §8.
- **OCR is a licensing question before it is a GUI one.** The engine can
  recognise text end to end (`ocr::layer` writes the invisible sandwich at
  render mode 3, the `ocrs` weights ship at 12,240,008 B) and **no shell has
  a surface**. But `GUI_ROADMAP.md` records the blocker as *shipping a
  CC-BY-SA-4.0 model in an MIT repo* — "not a GUI problem". Settle that with
  the operator before building anything, and note the engine also says
  recognition quality is **unproven**: its only test documents are vector
  PDFs that already contain text.

---

## 2. The founding rule, which is not a slogan

> **Verify by driving the binary, not by a passing test.**

The project exists because two defects were invisible to a green suite.
Since then the count of defects found *only* by running the program and
reading its trace or its pixels has reached **eleven**:

1. `Ctrl+O` printed in a tooltip, in the keymap, bound to nothing.
2. The icon painter existed, was tested, and was never passed to the ribbon
   — the whole ribbon was text buttons.
3. Find's current-hit highlight completely covered the word it highlighted.
4. Find's bar drew 108 pt left of its place for one frame on every open.
5. An undrawn page used a fill that read as blank paper, so a page still
   rendering looked like an empty one.
6. That page's explanatory sentence was centred in the *page* rather than
   in the part of the page on screen — a metre below the window.
7. A newly added panel was invisible to anyone who upgraded.
8. The grid was a tint rather than a grid: a one-point minor step, ~2,450
   lines a frame. **A screenshot could not catch this one** — 2,450
   hairlines and a wash are the same picture. It was found by printing the
   ladder the running app had actually chosen.
9. The page-text extraction was paid **at open** rather than on the gesture
   that needs it — 392 ms on the benchmark sheet, charged to an operator who
   had touched nothing. **The suite was green and the cache was working**:
   exactly one extraction happened, which is all a test can ask about. What
   was wrong was *when*, and the only thing that carries a when is a
   timestamped trace line. Found by reading `page-text` in a driven run and
   noticing it sat beside `open` instead of beside the first sweep.
10. The freehand ink trail was read **after** the gesture machine had already
    cleared it. `GestureState::update` drops its own drag on the frame it
    reports `Complete`, and `ink::sync` was called after it — so on exactly
    the frame the release arrived the trail answered `None`, and every
    freehand stroke authored **two points**. Every unit test passed: they
    call `drag` directly, and **none of them can see the order
    `canvas::interact` calls two functions in.** Found from the trace line
    `markup-commit kind=Ink raw=2 kept=2` on a drag that was hundreds of
    points long — which is also why that line carries `raw=` beside `kept=`,
    since a build whose simplification did nothing emits an otherwise
    identical line.
11. The redaction panel's apply control was laid out **below the bottom of
    its own pane** — declared at `y = 801.7` inside a body ending at
    `y = 770.0`, on the shipped window size, with a mark already made. Every
    unit test passed. `MODES_AND_PANELS.md` already records the rule this
    proves twice over: *layout and clipping defects have exactly one oracle,
    a rendered screenshot* — and the control it hid was the one that applies
    an irreversible edit.

Number 8 carries the sharpest lesson available here: the existing test
passed because it asserted the grid was *finer* than the ruler, which it
emphatically was. **A test that checks a relation rather than a magnitude
is satisfied by any absurdity in the right direction.**

How to actually do it:

```bash
cargo build --release -p pdfcer-gui
PDFCER_DIAG=1 ./target/release/pdfcer-gui.exe "D:\Dev\temp\pdfcer\SW41177.pdf"
```

Test documents that matter:

| file | why |
|---|---|
| `D:\Dev\pdfTests\ncored-benchmark-cad-drawing.pdf` | A3, 129,758 objects, ~1.2 s per raster. The performance case. |
| `D:\Dev\temp\pdfcer\SW41177.pdf` | 36 SolidWorks sheets. The multi-page and mixed-size case. |

---

## 3. Standing instructions from the operator

> ### ★★★ 0. Scope a request to the whole expected behaviour — 2026-08-18
>
> The instruction that reframes every other one on this list:
>
> > *"when I ask for something, my expectation is usually that everything
> > surrounding that request is also done to where it would match the behaviour
> > a user would expect. Otherwise I am left typing out every little missing
> > detail."*
>
> and, on a decision not to ask the engine for two capabilities because nobody
> had requested them:
>
> > *"not adding such things just because they weren't explicitly asked for i
> > think is how we end up with partially finished features."*
>
> **The failure it names is real and it is ours.** The pattern is: ship the core
> of a request, enumerate the deferrals honestly in the commit message and in
> `RESUME.md`, and treat the enumeration as sufficient. It is not — it relocates
> the work onto the operator, who then has to notice the gap, remember it, and
> ask again. In one day: *Insert from file* landed with no position choice, no
> page range and no page count; *New at a chosen size* with no remembered
> default; *annotation selection* with no move or resize. Each defensible alone;
> together, a GUI that does the first 70 % of everything.
>
> **What to do instead.** Before calling a feature done, ask *"what would a
> competent user reach for next, within this same gesture?"* — a position, a
> range, a count, a preview, a default that is remembered — and build it.
>
> **Deferring is still allowed. It must be a decision with a reason**, not a
> scope boundary drawn where the request's sentence ended. *"Blocked on an
> engine verb"* is a reason. *"They only literally asked for X"* is not.
>
> **★ And it changes how the request channel is used.** R151's *"a verb with no
> caller is drift"* applies to a **convenience query** duplicating something
> already reachable — it is why `markup_rects` was rightly not shipped. It does
> **not** apply to the missing members of one feature: those make the feature
> permanently partial, and the partiality is discovered by a user rather than by
> us. Ask for the cluster.


These were given explicitly and are still in force.

1. **Check `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` periodically
   while waiting on anything from pdfcer.** Empty means nothing is owed. Read
   `INDEX.md` for the history; `archive/` is not read by default.
2. **Continuous scroll is an option, not a replacement.** Single page stays
   the default outside Read mode — *"the way I move around a page is great
   when working with drafting drawings"*. A change that makes single-page
   feel like a degraded continuous mode has failed regardless of the tests.
3. **Ignore the "nothing floats over the canvas" stance** when placing a
   transient bar. Superseded for Find; the bar floats, and the operator said
   to drop the argument.
4. **Make it work the way other programs do.** This retired the `Editing on`
   master toggle and it is the tie-breaker for interaction questions.

   **★ Sharpened 2026-08-14, and this is the most useful instruction in the
   file**: *"make your best educated guesses to match what inkscape, acrobat,
   and SolidWorks do."*

   Three named reference applications, and they are named for reasons that
   cover the product between them: **Acrobat** is what pdfcer replaces,
   **Inkscape** is the vector editor whose docking and tool model this shell
   already benchmarks against (`MODES_AND_PANELS.md` Part 2), and
   **SolidWorks** is where the operator's drawings come from and therefore
   where their muscle memory lives.

   What it changes in practice: **do not ask the operator how an interaction
   should behave.** Look at what those three do, pick, and *record which one
   you followed and why*, so the guess is auditable rather than merely made.
   Where they disagree, say so and say which won — that disagreement is
   usually the interesting part of the decision.

   Worked example, from the day it was given: *how is zoom-to-selection
   reached?* SolidWorks and Acrobat both put it on the right-click menu;
   only Inkscape binds a key (bare `3`, in its `1`–`6` zoom family). Two of
   three said menu, so it went on `canvas.object` — and no chord was
   invented, because Inkscape's family is *unmodified digits* while this
   shell's manifest chords are `Ctrl`-modified by construction and its
   `Ctrl+1`/`2`/`3` are the mode selector. Transposing `3` onto `Ctrl+4`
   would have matched the letter of neither convention and the muscle memory
   of nobody. The whole argument is at the registration site in
   `shell::menus`.

   **A second worked example, and it sharpens the rule.** *How does a
   polyline or polygon end?* All three double-click, so that half was not a
   judgement call. The disagreement was the *other* way out: Inkscape and
   SolidWorks both close a shape by clicking the first vertex, and **Acrobat
   does not**. Acrobat won — two against one — because `/Polygon` closes back
   to `/Vertices[0]` by ISO 32000-1 §12.5.6.13, so pdfcer is in Acrobat's
   position, and a click-the-first-vertex rule would author a duplicate
   vertex and a zero-length closing segment. **The majority had never faced
   the surface.** SolidWorks' Escape-to-end was refused outright on a
   different ground: in this shell Escape means *abandon*, and committing on
   the key an operator presses to say "no" is the least recoverable reading
   available.

   So the rule has two halves, and the second is the one that keeps being
   load-bearing: **match what they do, but first ask which of them actually
   has the surface you are deciding about.** That test has now decided three
   cases — zoom-to-selection, the Edit text tool, and this one — and in two
   of the three it overturned the head-count.

   The instruction does **not** license guessing about *claims* — refunds,
   licensing, what the engine does, what a file format permits. Those are
   still verified. It licenses guessing about **behaviour**, where a
   defensible convention beats a blocked question.
5. **★ Read may produce a new document; it may not modify this one.**
   Operator instruction, 2026-08-14, given as a rule about OCR:

   > *"if in read mode ocr should still be available, but it will prompt to
   > save changes as save as instead of save."*

   Recorded in its **general** form, because that is the form that decides
   future cases rather than one. It explains the two exceptions Read already
   carries — **form filling** (2026-08-14) and now **OCR** — as exceptions
   rather than inconsistencies: neither changes the document the operator was
   handed. It also settles in advance every capability of the same shape that
   is still to come: flatten, redact-apply, PDF/A convert, page export.

   The line Read's gate actually draws is therefore **not** "no writes". It
   is *no writes to **this** file*, and the enforcement point is the **save**,
   not the operation. That is worth knowing before anyone tries to make the
   canvas gate cover it: `app::modes::capability` governs *gestures*, and OCR
   is not a gesture.

   **Two things about it that are true today and easy to get wrong:**

   - **It is currently vacuous.** There is exactly one save command,
     `file.save_copy`, and pdfcer never overwrites the original unless the
     operator picks it — so every mode already behaves this way. The rule
     becomes load-bearing **the day in-place `Save` lands**, which is
     precisely when someone will be least likely to remember it. That is why
     it is written here against a command that does not exist yet.
   - **OCR is still blocked, and not on anything in this repo.** The blocker
     is shipping a CC-BY-SA-4.0 model in an MIT repo, plus the engine's own
     note that recognition quality is unproven. Both are the operator's to
     settle. See `FEATURES.md`'s OCR section for the Find-offers-OCR trigger
     rule, which has one trap in it: *"the document is images"* is not
     *"this search had no matches."*

6. **Dispatch subagents freely; do not ask permission.** See the global
   `CLAUDE.md`. `D:\Dev\pdfcer` is READ-ONLY to this project.

---

## 4. How the parallel work was actually run

This is the part that is easy to get wrong. Up to six agents ran at once,
and the only thing that prevented collisions was an explicit **territory
partition given in each prompt**.

**The pattern that worked:**

- Give each agent a *write territory* as an explicit list of directories,
  and an explicit **do-not-touch** list naming the other agents' territory.
- **Forbid command registration** to every agent but one. `shell/` is a
  single-writer resource — `commands.rs`, the manifest and the generated
  RON must move together.
- Tell each agent to **report entry points** rather than wire them, and do
  the wiring yourself afterwards.
- Tell them **not to commit**. Take the commits yourself, after verifying.
- Expect the crate not to compile at times. Tell agents to report breakage
  outside their files and carry on rather than fixing it.

**What still went wrong, so you can watch for it:** an agent added an
`Action` variant to `app/actions.rs` outside its territory because the
crate would not compile without it (harmless, and correct); another ran
`cargo build` inside `D:\Dev\pdfcer`, which touched only its gitignored
`target/` but violated the read-only rule.

**Coordination beats racing.** When the Pages panel needed five lines in
`shell/` and another agent owned it, the right move was to message that
agent with the patch rather than edit around it.

---

## 5. Registering a command has six obligations

Every one has a test that fails loudly. This is the single most common way
to break the build.

1. The registry count in `shell/commands.rs`.
2. The group count in `shell/manifest/mod.rs`.
3. Removal from the `PLANNED` list if the command is named there.
4. Regenerate the RON:
   `cargo test -p pdfcer-gui --lib rewrite_built_in_ron -- --ignored`
5. A `KNOWN` entry for any new `enabled_when` condition name.
6. **★ A dispatch arm — or an argued entry in
   `shell::commands::reach::SCAFFOLDED`.** Added 2026-08-15, and it is the
   only one of the six that asks whether the command *does anything*.
   `file.save_copy` passed the other five for the whole life of the project
   while doing nothing: registered, drawn on the quick access toolbar, bound
   to `Ctrl+S`, printed in its own tooltip and in the shortcuts reference.
   `edit.undo`, `edit.redo` and every page verb shipped in v0.1.0 the same
   way.

   The check parses `app/dispatch.rs` with `syn` — a `match` is not a regular
   language, so it is not grepped — and it does not parse the guard arms at
   all: it extracts *which function* each guards on and then **calls the real
   one** against the real registry. A set-equality test welds the two halves,
   so a new guard fails by name and a deleted one stops vouching for its
   family. `include_str!` makes a moved dispatcher a compile error, so
   "scanned nothing" cannot pass as "found nothing".

   **The list has already gone down twice**, which is the outcome this
   obligation exists to produce rather than a register that only grows:
   `SCAFFOLDED` 38 → 33 and its `★ P3` subset 11 → 8, by wiring three
   controls that had five surfaces and no behaviour. A fourth,
   `view.show_points`, was investigated and **stayed** — with its reason
   upgraded from *"no recorded reason anywhere"* to a cited blocker, which
   is the other honest outcome and the more common one. Then **33 → 31**
   on 2026-08-15 when `edit.text` and `edit.add_text` got real dispatch
   arms; `★ P3` unchanged at 8.

Adding a **panel** has its own set — see `panels/mod.rs`, whose header
explains that three panels once shipped with a body, a rail entry and *no
control anyone could click*, passing every verification for their whole
shipped life.

---

## 6. Invariants that are not up for renegotiation

- **Actions, not mutations.** A widget is handed `&OpenDoc` and pushes an
  `Action`; everything is applied after the frame, in one place. This is a
  compile-time fact, not a convention.
- **One choke point for dispatch** (`app/dispatch.rs`). The arms *route*;
  they do not compute. The moment an arm works out *how* to do something,
  that rule exists in two places and only one gets fixed.
- **No placeholders.** A capability that is absent renders **nothing**,
  never a greyed control that explains itself badly. An unknown icon key
  draws a visible "missing" mark rather than a blank — because the label
  fallback is decided *upstream* of the painter.
- **Disclosure lives off-canvas** (Rule 4). The one-line test: *would a
  screenshot of the editing canvas differ from a screenshot of the same
  document saved and reopened?*
- **Every operator-visible string lives in `text/`**; every colour is a
  named role in `egui-shell/src/theme/`. Both have gates, and both gates
  have self-tests, because a grep that stops matching prints exactly what a
  clean run prints.
- **No `.rs` file over 1,500 lines.** `app/mod.rs` has been split twice
  under this rule, into `dispatch.rs` and `conditions.rs`, both at real
  seams. The old GUI reached 25,005 lines in one `main.rs`.
- **`egui-shell` knows nothing about PDF.** Enforced by
  `check-shell-purity.sh`. It reports; the application decides.

---

## 7. Build and package

```bash
cargo test --workspace
bash tools/gates/run-all.sh                 # 8 gates; exit 3 means SKIPPED, which is NOT a pass
python tools/package-portable.py --verify --note "what this milestone added"
```

`--verify` runs tests and gates **before** building, so a failure costs
nothing and leaves no folder. When it is not run, `BUILD-INFO.txt` says so
in those words.

**"Integrated with pdfcer as a single exe" needs no fold-in.** Rust links the
engine statically, so the release binary already carries it. Folding this
shell into `D:\Dev\pdfcer` today would ship a *regression*, because measure,
redaction, the settings dialog and text editing still live only in the old
shell.

### ★★ The engine is PINNED, and it goes stale silently — checked 2026-08-17

The sentence above used to say *"depends on `pdfcer-core` and `pdfcer-render`
**by path**"*. **That has been false since 2026-08-14** and the consequence
is not cosmetic:

```toml
pdfcer-core   = { git = "https://github.com/KenM76/pdfcer", rev = "718d1e9d4", … }
pdfcer-render = { git = "https://github.com/KenM76/pdfcer", rev = "718d1e9d4", … }
```

It fetches from **GitHub**, not from `D:\Dev\pdfcer`. So the local engine tree
can move arbitrarily far ahead and nothing here notices — no compile error, no
warning, no test. It measured **seven commits behind** on 2026-08-17, and one
of the seven was `1e7a0be`: images in `Separation`, `DeviceN`, `Lab`, `CalGray`
and `CalRGB` went from `UnsupportedColorSpace` — *dropped from the raster
entirely* — to decoding. Eighteen pictures on the operator's own file. It also
fixed `/Separation /None` painting white, which **erases the backdrop** on a
real page.

So on that date **the old shell rendered a file correctly and this one did
not**, purely because the old shell uses `.workspace = true` and builds the
local engine. That is the reverse of what anyone assumes and it is the exact
failure mode a pin creates.

**Three things follow, and the third is the one that will be forgotten:**

1. **The direction is engine → shells.** Nothing this project builds flows into
   `D:\Dev\pdfcer`, and compiling the old GUI there inherits nothing from here.
   The operator asked this directly on 2026-08-17 and the assumption was the
   other way round.
2. **`origin/main` is what this shell can see**, not local `main`. On
   2026-08-17 `origin/main` was `718d1e9` and local `main` was `f08effd`, so
   the seven commits were **unpushed** — bumping the pin was impossible until
   they were pushed, which is the operator's act and not this project's.
3. **Bumping is scheduled work, not remembered work.** It has no failing test
   behind it — the same structural weakness as the RON regeneration in §10,
   and worse, because the RON at least fails for whoever next touches the
   manifest. Check it whenever a rendering complaint arrives:

   ```bash
   cd /d/Dev/pdfcer && git rev-list --count 718d1e9d4..origin/main
   ```

   Non-zero means this shell is rendering with an older engine than the
   repository has, and **a rendering complaint should be checked against that
   before it is investigated as a shell defect.**

**Known environment quirk:** `--verify` may report the gates as skipped
because a spawned bash does not inherit `~/.cargo/bin`. If that happens,
run the tests and gates by hand, then package without `--verify` and state
the results in `--note`.

---

## 8. What is left, in the operator's likely order

| | |
|---|---|
| **Phase 5 — text editing** | **Started 2026-08-15 on the operator's explicit instruction, and partly landed.** Of D4's three problems: **D4b's two wrong cases are FIXED** — aligned tails are pinned and rotated text is no longer shifted along the wrong axis (`canvas::textedit::disposition`), proved by a `ui-verify` check that re-opens the saved copy in a second process and asserts the untouched line's `Tm` survived, with the old `EditOptions::default()` build planted twice to confirm the check fails against it. **Not done:** per-keystroke re-layout — measured at 102.77 ms on a SolidWorks sheet and blocked on the engine, which keeps `plan_edit` `pub(crate)` so there is no dry run; **D4a's cross-run edit**, which needs a multi-run request core does not have and now refuses in a sentence rather than by a dead keyboard; **D4c's three gates**, untouched. `edit.add_text` is wired and unit-tested but not driven, and has no font/size/colour surface. `DEFECTS.md` D4 carries the measurement table and the honest single-line limit. |
| **Phase 6 — markup** | **In progress, and larger than this row used to imply.** The new shell has *no markup placement at all*: all eight `markup.*` commands draw and fall through to `command-unimplemented`, `CanvasTool` has two variants, and there is no `canvas/markup.rs`. So it is *build the tool substrate, then ten kinds*, plus the Comments panel (which does not exist here either). **Three items needed engine changes; all three were filed and answered on 2026-08-14, accepted and scheduled, none started.** Revision clouds land as `MarkupSpec::Cloud` plus `Square { border_effect }` — and the *rectangular* cloud ships first, being the gesture people actually reach for. Note text lands as `/Contents` + `/T` + `/M` together, `/M` engine-stamped and `/T` optional with **no invented placeholder**. Opacity is `/CA` **alone** — writing `/ca` into the appearance stream would encode a pdfcer render bug into the file format; see **`DEFECTS.md` D9**, which is the more urgent half of that exchange and is about *viewing*, not authoring. Polyline, polygon, ink, underline, strikeout, squiggly, width and fill are engine-ready and blocked on nothing. |
| **Phase 7 — measure** | **Three tools place dimensions**: Linear (three clicks — what, to what, where), Two-line, and **Radius / diameter**. `measure_tool.rs` came across whole into `canvas/measure/{pick,scale,state}.rs`, the 12.M1 snap primitives into `canvas/snap.rs`, 45 tests carried, **no engine API had moved**. ★ **This row used to name three remaining decisions and two of them are taken.** *Radius/diameter had no natural end to its gesture and the only place to say "done" was an accept box decision 024 retired* — the operator's answer on 2026-08-14 was **two** endings that are not boxes, a double-click and `measure.finish`, through one commit path in `canvas/measure/circular.rs`; the Finish control is gated on a new `measure.finishable` condition so it is live only when there is a non-degenerate fit to commit. *The snap query is unwired* — it is wired. What is left is **Set scale**, which still has no dialog to ask the length in. Area and Count still need engine changes; Angular is core-complete with no tool. See `SALVAGE.md`'s Phase 7 entry for the three deliberate departures from the source and the axis collision it surfaced. |
| **Salvage remaining** | Redaction (its true-removal proof exists **only** in the old shell), and the settings dialog. |
| **S6 — deep zoom** | ⛔ Blocked on the reusable parsed handle, which pdfcer has scheduled as `Pass 75.0`. Do not build tiling: measured as a 9× regression. |

Smaller, unblocked, and recorded in `FEATURES.md`:

- ~~Panel toggle semantics~~ — **done 2026-08-14.** An open panel's control
  closes it; `file.properties` and `markup.comments` deliberately do **not**
  toggle, because they answer *"tell me about this thing"* rather than *"is
  this panel open?"*. See `app/panels.rs`.
- ~~The **edit-disclosure surface**~~ — **done 2026-08-14**, and the two
  things that were waiting on it are settled: the zoom decline is built
  (`app/status/decline.rs`, same surface, *different* store), and the
  guide-count refusal can now follow the same pattern.
- ~~**A text tool for Edit**~~ — **done 2026-08-14.** `CanvasTool::Text`,
  armed by `view.tool_text` beside the hand tool in View ▸ Navigate. It
  closed **two** things: Edit could not sweep text, and the three
  text-markup controls were drawn on the Markup tab in Edit and could never
  enable — a live P3 tension, now observed closed by `ui-verify`'s
  `text_tool_selects_and_marks_in_edit`.

  ★ **The reference applications disagreed, and how that was resolved is the
  reusable part.** Acrobat and SolidWorks resolve text-versus-object
  *contextually inside one tool*; only **Inkscape** uses a separate Text
  tool. Inkscape won and **not by head-count**: an object marquee over
  vector content is a surface Acrobat does not have at all, so its
  contextual answer was not an answer to this conflict. The deciding
  argument was concrete rather than taxonomic — a contextual press would
  make a marquee over a region containing text unpredictable, and that is
  the commonest gesture in Edit. **When the three references disagree, ask
  which of them actually has the surface in question**; a majority that has
  never faced the problem is not a majority.

  One consequence worth knowing before touching the gesture layer: the new
  rung sits **above** the `caps.edit_content` branch, so text-versus-content
  exclusivity moved from *construction* to *precedence*. An object selection
  and a text selection can now both be non-empty, which is why
  `canvas::keys`' Escape ladder had to be re-argued rather than merely
  extended.

- ~~**Paper size, tray and printer Properties in the print dialog**~~ —
  **done 2026-08-18.** All three of this project's print filings were answered
  as one engine defect (`build_devmode` synthesised a zeroed `DEVMODE`, and
  tray, paper and orientation all resolve through it). The shell half is a
  combo, a button and a checkbox — plus one thing that is not a control:
  `plan` now reads `printer_caps_for(printer, config, paper)`, without which a
  job asking for A3 on a Letter-default device is *planned* for Letter and
  *printed* on A3, with no clip reported. Two disclosures the operator could
  not otherwise learn: a chosen paper is a **request** two measured drivers
  were found silently ignoring, and a job whose driver would not report its
  settings prints with media type, quality and finishing **fallen back**.

  ★ **The tray control is drawn in all three capability states**, which
  inverts R83's usual direction. `pdfcer-print` declined our `bool` proposal
  with a measurement: `DC_BINS` on Microsoft Print to PDF returns nothing
  while that same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE`.
  R83 forbids offering what the hardware *cannot* honour; it does not forbid
  offering what the driver merely declined to advertise.

- ~~**New at a chosen page size**~~ — **done 2026-08-18.**
  `file.new_from_template`, beside `file.new` in File ▸ File, `Ctrl+Alt+N`:
  A0–A6, Letter, Legal, Tabloid, Executive, ANSI A–E, both orientations, plus
  a custom size in millimetres. `NO_SURFACE.md` had listed this as an unbuilt
  GUI surface and it was an **engine** gap — `app::blank` §3a refused the
  ten-assets implementation because it could not answer a custom size at any
  count, and filed instead. `set_media_box` shipped the next day and the
  answer is one asset.

  ★ **The document is serialized and re-parsed** rather than handed over as
  the `EditSession` that resized it. Otherwise a brand-new A1 sheet arrives
  already modified, with `Ctrl+Z` waiting to take it back to A4. A new
  document is not an edited document.

- Scoped reset chooser.
- `ui-verify`'s `find_opens_and_finds` **has never passed here**: synthetic
  keyboard input does not reach the target window from the session that
  wrote it. It reports SKIP rather than blaming Find, on purpose.

  **★ A lead was raised against this on 2026-08-14 and then failed to
  reproduce. Recorded because the next reader will otherwise have it
  again.**

  The canvas form-filling work reported driving typing, Enter *and*
  Escape into the real binary successfully, and attributed it to
  `SetForegroundWindow` on the target PID plus verifying the foreground
  actually changed. That would have made this SKIP a two-line gap in
  `ui-verify` rather than an environment limit, and would have recovered
  every keyboard-blocked check — including the Escape rules for markup
  and for a focused form field, both currently asserted by test alone.

  **I tried to reproduce it directly and could not.** With the foreground
  PID confirmed equal to the target's, `keybd_event` for `Ctrl+2` produced
  no `chord-command` line and no mode change. A **mouse** click sent by
  the same mechanism moments later landed and traced
  `canvas-selection via=click`, so the window was live, the process was
  reading input, and the pointer half of the same API worked — **only the
  keystrokes went nowhere.** Sending a click *first*, on the theory that a
  real click confers something `SetForegroundWindow` does not, changed
  nothing.

  `ui-verify` already does the raise and already checks `is_foreground`
  before typing; its own SKIP text says the window reported itself
  foreground. So the missing ingredient, if there is one, is **not**
  foreground rights and **not** a prior click. Two candidates remain
  untested: the harness's 48-frame wait may be too short, and the
  successful report may have used a different injection API
  (`SendInput` rather than `keybd_event`). Worth an hour if keyboard
  coverage ever becomes the blocker; **not** worth treating as solved.

---

## 9. Two open questions worth putting to the operator

1. ~~**Should Read mode fill forms?**~~ **Answered by the operator on
   2026-08-14: yes.** Acrobat Reader fills forms in its default view and
   replacing it is the stated goal. It cost the taxonomy amendment plus a
   tab move — `edit.form_fill` became `view.panel_forms` on View ▸ Panels,
   because Read is shown File and View alone and a command lives on exactly
   one tab. Edit ▸ Forms kept create, manage and flatten: **filling is not
   authoring** is the line that move draws.

   Canvas filling then arrived the same day with **no mode gate at all**,
   which means Read fills forms on the page as well as in the panel. That
   is the same answer reached twice by different routes, so it stands — but
   note it was reached the second time *by omission* rather than by
   argument, and if anyone ever wants a mode to be genuinely read-only,
   `canvas::forms` is the second place that would have to learn about it.

   **★ The operator asked for exactly that on 2026-08-14** — *"in read mode
   the document shouldn't allow editing"* — and the answer to the sentence
   above turned out to be **no, `canvas::forms` stays out.** Filling is not
   authoring; it is the primary reason most form documents exist, and
   Acrobat Reader fills forms in its default view. What the gate covers is
   the canvas *gestures* (`app::modes::capability`, `app::gating`), derived
   from the mode's **tab list** rather than from the id `"read"` — so the
   ribbon and the canvas cannot disagree about what a mode is. `forms.rs`
   was left untouched, deliberately, and its header's argument for that is
   now load-bearing rather than incidental.
2. **Per-mode memory of the page-display choice.** Deliberately not built:
   it is a second axis that collides with per-document, which is what was
   actually asked for.

---

## 10. Things that will bite you

- **★★★ When a blocker clears, the prose that DESCRIBED the blocker is the
  most dangerous thing in the tree — and no test on either side can see it.**

  On 2026-09-02 `EditSession::reorder_annotations` shipped hours after the
  request that asked for it. Wiring it up took a morning. Finding everything
  that had *asserted the gap* took longer, and three of the four places were
  found only by looking:

  | where | what it said | how long it had been true |
  |---|---|---|
  | the panel's explainer, on screen | *"This view reports the order; it does not change it."* | the view's whole life |
  | the module header | a **prohibition** — no drag handles, no `Sense::drag`, not even disabled ones — ending *"when the engine verb lands, the affordance arrives with it"* | 3 days |
  | a unit test | `no_string_in_this_view_offers_a_reorder`, **passing**, and it would have *forbidden the feature* | 3 days |
  | `FEATURES.md` | a ⛔ row: *"the operator asked for it; no verb can do it"* | 19 days |

  Plus a fifth in the **engine's** `docs/FEATURES.md`, saying the verb was
  *"Not reachable in `pdfcer-gui`"* — a statement about **our** surface in
  **their** document, which is the shape neither side re-checks.

  ★★ **Every one of them was correct when written.** That is what makes the
  class survive: nothing about a true-when-written sentence looks wrong, and no
  gate evaluates it. `check-ui-strings` proves a string is *in the catalog*, not
  that it is *true*. A green suite is silent on all five.

  ★★★ **The test is the worst of them, and note the inversion.**
  `no_string_in_this_view_offers_a_reorder` was a *correct* tripwire while the
  verb did not exist — it guarded against somebody shipping a disabled drag
  handle "ready for when it lands", the placeholder R9 forbids. The day the verb
  shipped it became a test that would fail the feature the operator had asked
  for, **while still passing**, because the strings had not changed yet. A
  tripwire aimed at an absence must be re-aimed when the absence ends; the right
  move is to replace it with what it was protecting all along, pointed the other
  way, and say so in its doc comment rather than delete it quietly.

  ⇒ **The procedure, when any blocker clears:** grep for the *feature's* words
  before writing a line of it — the verb, the noun, "read-only", "blocked",
  "cannot", "no verb", "when … lands". Check `FEATURES.md` on **both** sides,
  the module headers, the operator-visible strings, and any test whose name
  contains `no_`, `never_` or `not_`. Budget it as part of the work, not as
  tidying afterwards; the on-screen sentence is the one the operator reads, and
  it is usually the one nobody thinks to change.

  ★ The discoverability corollary, which arrived in the same hour: the sentence
  that *replaced* the false explainer is now the **entire** discoverability
  surface for a drag with no handle, no grip glyph and no button. Deleting the
  stale claim was necessary and not sufficient — a stale sentence removed and
  not replaced would have shipped an invisible feature.

- **★★ Registration is not implementation, and five surfaces will lie about
  it.** `file.save_copy` was registered, drawn on the **quick access
  toolbar**, bound to `Ctrl+S`, listed in the shortcuts reference, and
  printed "(Ctrl+S)" in its own tooltip — with **no dispatch arm**. Nothing
  this shell built could be written to disk, for the whole life of the
  project, and it was within an hour of being released that way.

  An audit afterwards found the same shape in **`edit.undo`/`edit.redo`**
  (QAT, three chords) and in **every page operation**, six of which the
  Pages context menu offers while `panels/pages/select.rs` maintains a
  multi-select model to feed them.

  The audit is one command and is worth running before any release:

  ```bash
  # every registered id, against the ids dispatch.rs actually names
  # (remember the guard arms: markup_for_command, measure_for_command,
  #  Panel::from_command_id, page_display_for_command, chrome_for_command)
  ```

  **A `command-unimplemented` trace is the only honest signal**, and nothing
  reads it. The durable fix is a test asserting that every registered command
  is reachable by *some* arm — literal or guard — with an explicit,
  argued allow-list for the ones deliberately scaffolded. Until that exists,
  audit by hand.
- **★ The conventional value can be the worst one, and only measurement
  tells you.** OCR shipped with `OCR_DPI = 300` — the number every scanning
  guide gives. Measured against `SW41177.pdf` using its own vector text as
  ground truth: 72 DPI → 34.8 %, 100 → 20.0 %, **150 → 44.7 %**, 200 → 27.5 %,
  **300 → 3.3 %**. The conventional answer was the worst of the five by an
  order of magnitude, because `ocrs` resizes every image to its model's fixed
  input — so **pixel count governs, not resolution**, and 300 DPI on an A1
  sheet throws away almost everything in the downscale. The constant is now
  `TARGET_PIXELS`, with the table in its doc comment. **Before trusting a
  parameter because it is standard, ask what the standard was measured on.**
- **★ The RON has now been found stale three times in one day**, by five
  separate changes that each missed it. See §5 obligation 4. Nothing about
  this is going to improve by asking people to remember; it wants either a
  non-`--ignored` test that regenerates and fails on a diff, or a pre-commit
  hook. Until then, run it after **every** manifest touch, and re-run it last
  when several sessions are landing at once.
- **★ A test fixture that is not themed like the running application hides
  spacing bugs.** The two-row ribbon's first cut padded rows by
  `rows×height + (rows−1)×spacing` — one gap short, because egui advances the
  cursor past *every* rect including the last. **Every test in the crate
  passed**, because `width_tests`' context installs a font but no theme, so
  egui's default `interact_size.y` (18 pt) sat 6 pt under the theme's
  `control_height` (24 pt) and the slack swallowed the error. It was visible
  only in the running binary's trace, as one group 68 pt tall beside another
  at 64. `height_tests::context()` now applies the theme, and
  `the_fixture_is_themed_like_the_running_application` guards it. **A layout
  fixture must be built like the thing it stands in for**, or its slack is
  the bug's hiding place.
- **★ A gate can be satisfied by a comment saying the thing is missing.**
  The shipped-assets gate's first self-test plant — declaring the OCR weights
  redistributed before writing their notice — **was not caught**, because
  `about.hbs`'s epilogue names that directory inside an HTML comment
  explaining it is deliberately absent, and a presence check found the string.
  Fixed by stripping comments before the check, and pinned by a sixth
  self-test case. The general form is worth carrying: **a check that greps for
  a string is a check on the file's *text*, not on its *output*** — render
  first, then assert.
- **★ Attribution is a shipped artefact, not a source-tree one.** Building
  the OCR prerequisite found **three third-party works this shell had been
  redistributing with no notice at all** — the Foxit CFF faces, the Adobe
  Core-14 AFM metrics and the Adobe Glyph List, all compiled into the binary
  by the engine crates. Nothing detected it for the whole life of the project,
  because `cargo-about` sees Cargo dependencies and these are `include_bytes!`
  payloads. **If it is in the binary and someone else wrote it, it needs a
  notice**, and the only thing that finds those is a gate that reads what
  packaging actually copies.
- **★ The RON regeneration is the obligation that silently rots**, and it has
  now rotted twice. Of the five obligations in §5, four fail loudly — a
  count assertion, a group assertion, a `PLANNED` disjointness test, a
  `KNOWN` lookup. **Obligation 4 has no compiler behind it and no failure
  until someone else runs the round-trip**, so a session that forgets it
  leaves `shell/ron/built_in.ron` describing a ribbon the build does not
  have. On 2026-08-14 it was found stale by *five* separate changes at once
  — the text-copy move to File ▸ Export, Edit ▸ Clipboard's deletion, three
  text-markup commands, two measure commands, and a context-menu entry —
  none of which had written it back. Run it, every time, even when your
  change "obviously" did not touch the manifest:

  ```bash
  cargo test -p pdfcer-gui --lib rewrite_built_in_ron -- --ignored
  ```

  The round-trip test `the_ron_file_and_the_rust_agree` is what eventually
  catches it, which means the person who pays is whoever next touches the
  manifest rather than whoever broke it. **It has now been found stale twice
  in one day**, by two different sessions, which is the strongest available
  argument that this should not depend on anyone remembering.
- **★ A fixture can flatter the thing it measures, and the numbers will look
  fine.** The ink simplification was first measured against a synthetic trail
  whose disturbances were applied *along the arc's tangent* — so both of them
  only re-spaced samples along a path whose shape never changed. It reported
  17 points kept at a 0.5 pt tolerance and 33 at 0.125: a suspiciously flat
  response to a 16× change, and the tell. Recomputing retention independently
  exposed it; the fixture now offsets **radially** and carries an assertion
  that the worst deviation actually exceeds half the tolerance, so a future
  fixture that stops exercising the bound fails rather than flattering it.
  This is `HANDOFF.md` §2's grid lesson wearing different clothes: **a
  measurement that moves in the right direction is not evidence that it
  measures the right thing.**
- **Prose that quotes a number drifts from the number.** This has now
  happened five times: the command count in two module headers, the group
  count in six places, the test count in two documents, and — caught
  2026-08-14 — the icon coverage split, which read *"82 of 93 named, 12
  refused"* when 82 + 12 = 94 ≠ 93. Each was true when written. The fix that
  works is a **test that asserts the arithmetic**, not a comment asking the
  next reader to keep it current; `the_icon_coverage_split_adds_up_to_the_registry`
  is the current example.
- **`core.autocrlf` is true globally.** `.gitattributes` predates the first
  commit for that reason: CRLF normalization of PDF fixtures lands **in the
  index at `git add` time**, and a PDF's xref stores absolute byte offsets.
  Do not remove it. Do not add a binary type to it without `binary`.
- **`cargo test -p egui-shell` and `cargo test --workspace` compile with
  different `egui` features** (no fonts vs `default_fonts`). Layout tests
  can be entirely vacuous under one and not the other. Assert that a
  measurement *happened* (`Some(false)` rather than `None`), not just its
  value.
- **`ui-verify` refuses a stale binary.** That is the guard working; rebuild
  or point `--exe` at a packaged build.
- **Selection is an identity** — page, object, subpath, node — not a
  position. Paint-order indices survive `move_*` and do **not** survive
  `delete_*`.
- **~99 % of render cost is resolution-independent** on dense CAD. A small
  thumbnail is not a cheap thumbnail. A 1×1 *point* region costs 691 ms.
- **★ An `Options` flag that defaults off will silently neuter a correct
  decision function, and every unit test will stay green.** The text-edit
  disposition chooser is pure — it reads a text matrix, a CTM and an
  alignment, and picks `Pin` or `Reflow`. It was correct, and it was
  about to be permanently dead: `ExtractOptions::capture_provenance`
  **defaults to `false`**, and the shell's shared `page_text()` cache is
  built with `default()`. Fed from that cache the chooser would have
  received a `None` pin and identity matrices — so the rotation guard
  could never fire, on any document, while its own tests passed against
  hand-built matrices. `plan` now runs its own provenance-capturing
  extraction once per commit.

  The general shape: **a pure function's tests prove the function, not
  its inputs.** When the input arrives through a cache someone else
  configured, assert on a real document that the discriminating field is
  actually populated — or the feature is decorative.
- **Two files are now at the R2 ceiling**: `canvas/tool.rs` (1,487) and
  `shell/commands/reach.rs` (1,498), against a limit of 1,500. The next
  edit to either must split it first. `reach.rs` in particular grows
  with every scaffolded command that gets an argued reason, so it will
  hit the wall on ordinary work, not on a rewrite.

---

## 11. The relationship with `D:\Dev\pdfcer`

Another session works that repository live. It is **read-only** here.

The channel is `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`. Five
exchanges have completed, all answered within the hour. Two of them were
defects found from this side; **one of my four claims in a filing was
rejected, correctly** — `deletion_refusal` predicts deletion and matches
its guard exactly; I had compared it against *flatten*, which was my own
next item. Acting on it would have disabled a working Delete control, and
core now carries a test whose stated job is to stop a future reader
"correcting" a correct function on the strength of my report.

The lesson is worth carrying: **verify a claim against their source before
filing it**, and when a filing is wrong, record that it was wrong where the
next reader will find it rather than deleting it.

---

## 12. To resume

Read, in this order: this file, `FEATURES.md`, then whichever of
`PROJECT_PLAN.md` / `RIBBON_IA.md` / `MODES_AND_PANELS.md` /
`SHELL_FRAMEWORK.md` the task touches. `SALVAGE.md` before carrying
anything across from the old shell. `BENCHMARK.md` before making any claim
about rendering performance.

Then check `open/`, confirm the tree is green, and ask what to work on —
unless the operator has already said.
