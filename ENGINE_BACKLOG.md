# ENGINE_BACKLOG.md — every capability the engine says it has and this shell does not, and what we decided about each

**Written 2026-09-04, in answer to a failure rather than a question.**

On 2026-09-03 the operator asked the **engine** session for PNG / JPEG / SVG
export and for copy-paste of vector graphics into Word and Inkscape. The engine
shipped all of it the same day, across four passes, and sent a note — *"here is
what a shell wires"* — with the call for every capability, the clipboard format
order validated against a real Word paste, and a worked example.

**This shell built none of it and had no row for it.** It was found a day later
only because somebody happened to read the request folder for an unrelated
reason. It is now `OPERATOR_REQUESTS.md` O120.

The same morning, two new engine verbs — `set_encryption` and `set_permissions` —
arrived with **no note at all**, and *those were caught immediately*, by
`tools/gates/check-verb-coverage.sh`, because that gate reads the engine's API
and fails when this shell names none of it.

> ⇒ **★★★ A capability announced in prose has no such gate. A capability
> announced in an API does.** That asymmetry is the defect this file and
> `tools/gates/check-engine-backlog.sh` exist to close.

---

## ★★★ The measurement that already existed and was not read

`D:\Dev\pdfcer\docs\FEATURES.md` is a table whose first three columns are
`core | cli | gui`. **A row reading `[x]` under `core` and `[ ]` under `gui` is
the engine stating, in a machine-readable place, that it has something this
shell does not.**

That is not a note in a folder. It is a checkbox, in a file the engine's own
librarian updates on every filing, and it has been sitting there the whole time.
Nothing here read it.

**Measured at 2026-09-04: 259 table rows in that file, of which 90 read
`[x] core` and `[ ] gui`.** ★★★ **Re-measured 2026-09-05: it is now 34** — the
engine ticked 63 of those boxes on 2026-09-04, so this paragraph's own number
was out by a factor of nearly three within a day, which is the standing rule
below demonstrating itself in the sentence that states it. Re-measure before
quoting the number —
`bash tools/gates/check-engine-backlog.sh` prints it on every run, and this
project has a standing rule about counts in prose going stale while the prose
around them stays true.

⇒ **This file gives each of those rows a verdict and a reason in one
sentence** (97 entries today, answering 34 live rows plus 63 whose boxes have
since been ticked and whose arguments are kept). Nothing else. It is deliberately the same shape as
`EDITABLE_SURFACES.md`'s misses table, for the same reason: *the argument is the
valuable part*, and a register whose rows carry only a status trains the next
session to re-derive a conclusion somebody already reached.

---

## ★★ What the engine's `gui` column is worth, stated plainly

The single largest finding of this triage is that **the column is badly stale in
one direction**, and it is worth being precise about which:

- **A `[x]` is trustworthy.** The engine's own ticking bar is *"a row is ticked
  only when an operator can reach it in a real `pdfcer-gui` build — not when the
  code exists, not when a test passes"*, and nothing ticks itself.
- **A `[ ]` is a negative existential and is falsifiable by nothing.** The engine
  says so in its own header, about its own file: *"`[x]` is falsifiable by the
  build — delete the function, tests go red. `[ ]` is falsifiable by **nothing**:
  no test, no gate, no compiler notices a capability arriving."*
- **And a `[ ]` about a repository the engine does not build decays fastest of
  all.** That is not a criticism, it is a structural fact both projects have
  written up: a verdict about a repository this project does not build has no
  falsifier here and rots silently regardless of diligence on either side.

**Sixty-five of the ninety rows turned out to be reachable in this shell today** (★ 64 on the day this was written; the encryption-authoring row moved out of `blocked` the same day, on the operator's O119 ruling).
Some by weeks. The engine's evidence for a dozen of them is a single dated sweep
— *"confirmed 2026-08-19"* — taken before the work landed, and an absence
measured once is a claim with a date on it.

⇒ ★★★ **This file therefore runs in both directions.** Its `wanted` rows are
work for us. Its `shipped` rows are a **report back to the engine**, and sending
them is part of discharging this file rather than an optional courtesy: every
stale `[ ]` there is a row that will be quoted at somebody as a gap.

---

## ★★★ The five verdicts, and why there are five rather than three

Three verdicts were asked for. A fourth (`unknown`) was asked for as the honest
escape hatch. **A fifth was unavoidable, and refusing to add it would have meant
knowingly writing a wrong label ninety-six times out of a hundred rows.**

| verdict | means | and specifically |
|---|---|---|
| `wanted` | A real gap somebody should schedule. | The engine has it, an operator would use it, nobody has filed it. |
| `declined` | Deliberately not a GUI surface, with the argument. | The argument is the row. A `declined` with a bare verdict is worse than no row. |
| `blocked` | Wanted, but waiting on something **named**. | An operator ruling, or another surface that must exist first. The blocker is named or it is not a `blocked`. |
| `unknown` | Could not be settled, and saying so beats guessing. | An honest `unknown` is worth more than a guessed `declined`. |
| `shipped` | **The engine's `[ ]` is stale — an operator can reach it today.** | The row names the surface or the call site, and the date where this project's own record carries one. |

★★ **`shipped` is not an invention; it is `EDITABLE_SURFACES.md`'s own
vocabulary.** That register keeps rows for verbs that have *stopped* being
misses, marked SHIPPED, and states why: *"the argument is the valuable part: a
row saying why a verb was left alone for a fortnight is what stops the next
session re-deriving the same conclusion, or worse, reversing it without knowing
one was ever reached."* The same rule applies here, and it is why **no row is
ever deleted from this file** — a row whose `gui` box gets ticked keeps its
argument.

★ **Why not fold `shipped` into `declined`.** Because they say opposite things to
the next reader. `declined` means *do not build this*. `shipped` means *this is
built; go and tell the engine its column is wrong*. Collapsing them would hide
the largest actionable finding here inside the verdict that means "settled,
move on" — which is precisely the silence this project keeps paying for.

---

## ★★ Why nothing is `unknown`, which is itself a claim

**The `unknown` section is empty, and that deserves suspicion rather than
credit.** It was not for want of trying to use it. The evidence available for
every row was: the engine's own prose in the row (which frequently *states* the
verdict, and where it does it is quoted rather than re-derived); this project's
`FEATURES.md`, `EDITABLE_SURFACES.md` and `OPERATOR_REQUESTS.md`, which record
reachability with dates; and a call-site search over `crates/pdfcer-gui/src` for
the verb the row names.

**Two rows came closest to `unknown` and are named here so a reader can
challenge them rather than having to find them:**

1. *"Edit an image, text run or pasted object added in THIS session"* — verdict
   `shipped`, and it is the only `shipped` row whose evidence is a **linked
   revision** rather than a surface. The fix is entirely engine-side and needs no
   control of ours; `Cargo.lock` pins `04f7ec0`, well past `Pass 186.0`. There is
   no driven check of this project that asserts it. **If one verdict in this file
   is to be overturned, overturn that one.**
2. *"List the form XObjects a page paints"* — verdict `wanted`, and the argument
   for `declined` is real: this shell already discloses `invocations=` and
   `pages=` at the moment of an in-form text edit, which is where the number
   matters. It is `wanted` because a standing listing answers the question
   *before* the tool is picked up, and on a thirty-six-sheet SolidWorks set that
   is a different question.

---

## ★★★ What a `shipped` verdict rests on, and what it does not prove

Stated as plainly as `EDITABLE_SURFACES.md` states the same caveat about its own
instrument, because the two are the same caveat:

- **A named call site in `app::actions::*` or `panels::*` means the verb is
  reached through this shell's funnel** — an undo entry, a cache invalidation, a
  disclosure. That is much stronger than a bare grep hit, and every `shipped` row
  below cites one, a dated record in this project's own `FEATURES.md`, or an
  `OPERATOR_REQUESTS.md` row marked DRIVEN.
- **It does not prove an operator can reach it.** A call site behind a condition
  nothing sets is a call site and is dead in the running program. Only
  `tools/ui-verify`, driving the real binary, answers that — and this file was
  compiled without running it.
- ⇒ **So `shipped` here means *"the engine's `[ ]` is not supported by this
  repository's contents"*, which is enough to make the engine's row a claim
  worth re-checking, and is not enough on its own to tick their box.** Where a
  row's evidence is a **driven** check this project has run, the row says so.

---

## The gate, and why this file cannot be allowed to go quiet

`tools/gates/check-engine-backlog.sh` parses the live `FEATURES.md`, collects
every `[x] core / [ ] gui` row, and **fails when one is not accounted for
here**. It is deliberately weak in one direction and strong in the other, in
exactly the way `check-verb-coverage.sh` is:

- **Weak**: it does not judge the verdict. A row saying `declined` because
  somebody could not be bothered passes. This gate cannot read English and must
  not pretend to.
- **Strong**: a capability that appears in the engine's table and is discussed
  **nowhere** fails the build on the first `git pull` in the engine's checkout
  that brings it. Somebody has to look at it and write a sentence — which is the
  entire mechanism, and is exactly what did not happen on 2026-09-03.

The gate's row key is the **opening clause** of a row, not the whole row; its
header states why at length, and the short version is that the engine's prose
churns in the tail and is stable at the head. When the gate reports a row it
cannot find, read its message before adding anything: *a reworded row and a new
row look identical to a key and are opposite acts.*

★ **Re-run the gate before quoting any number in this file.** The engine files
daily.

---

## The tally, re-counted 2026-09-05

★★★ **Re-counted by parsing this file's own sections, not by editing the
previous numbers.** The 2026-09-04 tally read `65 / 18 / 5 / 2 / 0 = 90` and
was wrong the moment eight rows' *bodies* were rewritten while their *headings*
stayed put. Seven of those moved to `shipped` on 2026-09-05 and one was
retracted out of it.

| verdict | rows |
|---|---|
| `shipped` — the engine's `[ ]` is stale | **79** |
| `wanted` — a real gap | **12** |
| `declined` — deliberately no surface | **5** |
| `blocked` — waiting on something named | **2** |
| `unknown` | **0** |
| **total** | **98** |

⚠ **The engine-side figure moved too, and in the other direction.** The gate now
measures **34** rows reading `[x] core / [ ] gui` in the engine's file, against
the 90 recorded above — the engine ticked 63 boxes on 2026-09-04. A register
entry whose row has been ticked is **kept**, because the argument is the
valuable part, which is why 97 entries answer 34 rows and the gate reports the
difference as a `note:` rather than as a failure.

★★★ **And read this before trusting any PASS from that gate.** It keys a row on
its first six content words and, in its own header's words, *"does not judge the
verdict"* — it never reads which `##` section an entry sits under. Every one of
the eight stale rows corrected on 2026-09-05 passed it every single day. **The
gate proves each row is ACCOUNTED FOR; it proves nothing about whether the
account is true.**

---

## `wanted` — 11 of 97 (re-counted 2026-09-05)

A real gap. The engine has it, an operator would use it, and nobody has scheduled it. **These are the rows to read if you are choosing what to build next.**

### Document & pages

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Set a page's size (`/MediaBox`) — writing a value equal … | No operator-facing page-size control exists. `set_media_box` is called in exactly one place — `app::blank`, sizing a **new** blank document — so the verb is linked and the surface is not: a drawing that arrives on the wrong sheet size cannot be put right here. The Pages panel's context menu is where it belongs, beside rotate and extract. |

### Text

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| List the form XObjects a page paints, and how many places … | The engine answers *"how many places paint this form?"* only where this shell already asks it — `canvas::textedit::report` prints `invocations=` and `pages=` at the moment of an in-form **text** edit, via `text_edit::invocation_set`. There is no standing listing. ★ That matters more than it sounds on a SolidWorks set, where one title block is a single form drawn on thirty-six sheets: the operator wants to know the blast radius **before** picking up the tool, not in the disclosure afterwards. |

### Vector objects (Inkscape-style editing)

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Selectable-object diagnostics: when the model disagrees with the canvas. Four … | **One of the four is surfaced and three are not.** `app::status::notes::findings` reports `oc_sections_hidden` — content on a layer the operator's own file hid — among its nine render findings. `paths_with_undecoded_colour`, `paths_invisible_by_alpha` and `shadings_unmodelled` are nowhere, and they are the three that say *the object list and the picture disagree*, which is the one class of confusion an operator cannot diagnose by looking harder. The surface already exists: the Render-diagnostics dialog. |

### ce dimensions

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Move, insert or remove a vertex on a placed perimeter/path ce … | **✅ CLOSED 2026-09-05. All three verbs are wired.** `app::actions::dimensions` calls `move_dimension_vertex`, `insert_dimension_vertex` and `remove_dimension_vertex`, raised by `canvas::dimdrag::count_edit` on a Ctrl-drag and a Ctrl+Shift-drag from a corner handle with the Points tool armed. `vertex_edit_preview` is asked on every frame — the engine's own advice, *"grey the menu item from `vertex_edit_preview` rather than catch the error"*, applied to a gesture rather than to a menu — so the preview never promises a shape the release would refuse, and a refusal is a sentence (`Declined::VertexEditRefused`) rather than a silence. ⬜ The DISCOVERABLE form is still missing: the engine describes both verbs as *"what a right-click on a segment/vertex offers"*, and `canvas::menus` has no such items. See `OPERATOR_REQUESTS.md` O132. |

### Annotations & markup

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Read and edit a markup shape's vertices — `Annotation::vertices` … | **wanted — the capability landed 2026-09-05 and this shell has no anchors and no drag.** `Pass 255.0` (`35ca5be`) put `/Vertices`, `/L` and `/InkList` into the read model and shipped `reshape_annotation` / `move_annotation_vertex` / `insert_annotation_vertex` / `remove_annotation_vertex`, with Polygon and PolyLine taking all three (floors of 3 and 2, a remove below the floor refused by name), Line moving either endpoint, and Ink/Square/Circle/text markup refused by name with a reason a shell can show. ★ **This row was written by the O128 track because `check-engine-backlog` went red on it mid-session**, not by anyone who has scoped the work: the Pass shipped a few hours after that track's brief was cut, and the gate caught it within the minute, which is the case it exists for. What it would take, from the shape of the two verb families already wired: node anchors and a drag on a selected markup, mirroring `canvas::dimdrag`'s Ctrl-drag / Ctrl+Shift-drag for ce dimensions (row above, closed the same day) — and the engine's own advice there applies unchanged, *ask the preview verb every frame rather than catching the error*, so a gesture never promises a shape the release would refuse. |

### Redaction & security

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Detect an unencrypted wrapper (§7.6.7) and warn that the visible … | **Nothing here detects the §7.6.7 wrapper.** An unencrypted wrapper is a cover sheet standing in front of a document this reader cannot open, and pdfcer would render the cover, in full, silently — an operator looking at a plausible page has no way to know they are not looking at the drawing. It belongs in **O108's Security tab**, beside the encryption state, and it is the cheapest row on that tab. |
| Tell me what this document would run in Acrobat/Reader … | **Nothing here reads the action census.** *"What would this document run if I opened it in Acrobat, and does it reach outside itself?"* is the question an operator asks of a drawing that arrived by email, and the engine answers it across every place an action can live — page open triggers, navigation nodes, annotation triggers, chains — without executing anything. It is the second row of **O108's Security tab** and arguably the one with the most value per line. |

### Fonts & rendering

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| Reusable parsed page handle (display list) — record a page … | **No display list anywhere in this crate.** Every frame re-interprets the page, so a pan at fixed zoom pays full interpretation over and over: the engine measures 636 ms → 1.06 ms on a repeat render and 462 µs per pan frame at depth. ★ This is the largest single **felt** improvement available to a CAD sheet, and the engine has already paid for it — the key is `(page, epoch, scale)`, and this shell already tracks all three. |
| Opt-in sub-pixel culling — `render-page --fast-subpixel` … | **A toggle, and the engine says so plainly**: *"`gui` is a gap, not a shape mismatch — a toggle is a sensible thing to build and no GUI code path exposes one."* Measured 1 468 ms → 108 ms (13.6×) on a 342-form page with zero pixels different at page fit, and genuinely lossy near the threshold — so it is off by default and counted separately from the exact cull. ★ It is **not** a `Settings` key (it is `RenderOptions::subpixel_culling`), which is why the settings-completeness test cannot see it and why it needs a row here. |
| Colour spaces and PDF functions — all four function types … | **None of the twelve counters is read here.** `app::status::notes::findings` carries nine entries and every one is a *drawing* finding; unresolved colour spaces, ICC fallback and tint-transform failure are colour findings, and on a print-ready CAD file they are the difference between a colour that is wrong and a colour that is different. The Render-diagnostics dialog is the surface and it already exists; this is adding rows to a list. |
| Probe the ink at a pixel — report the four CMYK colorant tints … | **An ink eyedropper, and nothing here consumes `InkProbe`.** It is the only instrument that splits a colour error into the half that happened while compositing and the half that happened while converting to sRGB — a saved PNG cannot answer either. ★ Narrow, and worth the row anyway: this shell already surfaces the *settings* that change subtractive compositing, so it offers the controls without offering the measurement that says whether they helped. |
| Overprint (`/OP`, `/op`, `/OPM`) tracked and disclosed — Table 58's … | **The one shortfall that cannot be seen by looking.** The engine says it in its own row: a non-zero `overprint_refused` means *"the operator is seeing knocked-out backdrops where a press would show ink"* — a wrong picture that looks like a right one. None of the six counters reaches `app::status::notes::findings`. ★ The row also carries the engine's warning that an exhaustive list of them is *"a claim with an expiry date"* (it read "four" until 2026-09-01), so whatever surfaces them should enumerate rather than hard-code. |

### Export

★ **All four Export rows were being built on a concurrent track as this file was
written (2026-09-04)** — `dialogs::export_image`, `app::actions::imageexport` and
`text::export_image` are new and untracked in the working tree. They stay
`wanted` rather than becoming `shipped` on that evidence, because O120's own
Status line sets the bar and it is the engine's bar: *"they get ticked when the
GUI half is driven, not when it compiles."*

★★ **Updated later the same day, after O120's second pass.** Three of the four
rows are now written and one is not, and the split is worth stating plainly
because it is easy to read as "three quarters done":

* **PNG / JPEG / SVG / EMF are written.** Four radios, one window, one writer.
* **Copy-out is not written at all**, and that is a decision rather than a
  remainder. Its row below carries the whole argument; the short form is that
  the two dependencies it needs are in the lockfile and not in this crate's
  manifest, that the metafile placement needs `unsafe` in a crate that
  `forbid`s it, and that a copy-out missing its two vector entries makes Word's
  paste a flat picture — which an operator cannot tell from a broken feature.

⇒ **None of the four moves to `shipped`.** The three written ones are still
undriven — the Export-image window has not been opened in a running binary —
and the fourth does not exist. `D:\Dev\pdfcer\docs\FEATURES.md` is read-only
from here and no box in it is ticked by this pass.

| Row (`FEATURES.md`, wanted) | Why |
|---|---|
| **"Line weights off" hairline display mode** — `RenderOptions::stroke_display: StrokeDisplay { Actual, Hairline }` … | ✅ **BUILT 2026-09-05 — `view.line_weights`, View ▸ Display.** `OPERATOR_REQUESTS.md` **O137**, asked for by name: *"awhile ago you told me you removed the button to show all lines without their thickness — thin lines or something like cad has. The button never worked but I do want that display option!"* Both halves of that were correct. `view.thin_lines` was registered, drawn on the View tab and **inert**, and was unregistered on 2026-08-17 with six other `view.*` settings because *"thin lines and antialiasing have no `RenderOptions` field at all"*. That deletion was right — R8; a control that does nothing is exactly the defect he reported — and treating it as closing the question was not. The engine shipped the field the same day it was asked for (`Pass 254.0`, `8f9fb3e`, in this repo's lock at `b1033ab`), so under R8 the command was registered again, on a **new token (235)**: the retired 210-214 stay retired, because a token is an operator's saved keybinding. ★★★ **The convention is AutoCAD's, not Acrobat's** — off caps every stroke at ONE DEVICE PIXEL whatever the file declares (`LWDISPLAY` off, **thick → thin**), and is not *enhance thin lines* (**thin → thick**). Measured, not assumed: `render::hairline::line_weights_off_puts_less_ink_on_a_real_drawing` renders `fixtures/a1-titleblock.pdf` at scale 4 under both conventions and asserts **strictly LESS** ink — 484,078 → 398,578 dark pixels, 17.7 % — so a build that shipped the opposite goes red rather than looking like the feature working. Its twin pins the other end: at scale 1.0 the two are **identical**, because this is a CEILING and the §8.4.3.2 floor has already done the work. ★★★ **The engine's "do not complete the cli box" instruction is honoured on this side by a mechanism, not a paragraph.** Print, print preview and every export render the document's REAL widths; the field is set in exactly one function (`render::worker::render_on_worker`, which draws the canvas and nothing else) and `app::settings::tests::only_the_canvas_worker_sets_stroke_display` parses every `.rs` in the crate with `syn` and fails the build on a second assignment — falsified by planting one in `app::actions::export`, and by deleting the legitimate one, both of which go red. `every_export_path_renders_real_widths_with_line_weights_off` turns the mode ON and then asserts the funnel's options are still `Actual`, so it cannot pass vacuously. ★★ **The raster cache is keyed on it** — `RenderKey::new` takes it as a positional parameter, not an omissible builder, because a cached texture drawn under the other answer would make the toggle look exactly as inert as its predecessor. ★ **Fills untouched**, as the engine says, and the tooltip and the status line both say so, because an operator whose hatching is fill-based would otherwise expect it to thin. ★★ **Disclosed while it is on**: `status-group:line-weights` — *"Line weights are off — every line is drawn one pixel wide. Printing and exporting still use the real widths."* Off-canvas, in the status bar, live for as long as the mode is, with no epoch to clear. ⬜ **The driven check `line_weights` is WRITTEN AND NOT RUN** — another track held the pointer; it climbs to 250 %, presses the item, and compares canvas pixels either side. ⚠ `gui` stays `[ ]` in `D:\Dev\pdfcer\docs\FEATURES.md`, which is read-only from here; the engine's own row already says the shell owns this box. |


---

## `blocked` — 2 of 97

Wanted, and waiting on something named. Every row here says **what** it is waiting on — an operator ruling, or another surface that has to exist first. A `blocked` row with no named blocker is a `wanted` row wearing a better coat, and this project has found seven stale blockers already.

### Document & pages

| Row (`FEATURES.md`, blocked) | Why |
|---|---|
| Split a document — `EveryN` only; no bookmark- or size-based … | The engine's own row states the block and states it correctly: *"blocked on a decision (no boundary chooser, output directory or name template — no honest default exists for a UI that hasn't asked)"*. `EveryN` is the only criterion the engine offers, and a Split button that silently chose every-1-page and a directory would be R9's placeholder rule broken at the scale of a command. **Needs an operator ruling on the chooser**, not engineering. |

### Reading, navigation & printing

| Row (`FEATURES.md`, blocked) | Why |
|---|---|
| Author a named destination and point an outline item at it — `add_named_destination` … | The engine's row states the block and it is this project's own argument coming back: *"Shipped and deliberately NOT wired in `pdfcer-gui` — that shell has drag-to-reorder pages, and a destination resolved-and-baked at author time would look identical to a correct one until the next reorder moved the page it points at. Held until there is a surface where the destination-kind choice means something (the `insert_pages` bookmark-carry work)."* **Blocked on that surface**, not on the verb; `add_named_destination` is called nowhere here and should stay that way until then. |

★ **The Export row that used to sit here — *"Encrypt a document (AES-256, `/R` 6
only), set …"* — moved to `shipped` on 2026-09-04**, when the operator answered
the ruling it was blocked on with *"yes add encryption and permissions"*. It is
the first row in this file to move OUT of `blocked`, and the movement is what
the section's own opening sentence asks for: a blocked row names what it waits
on, so when the named thing arrives the row goes somewhere. Its reasoning is not
deleted — it is carried into the `shipped` entry, which is this file's standing
rule.

---

## `unknown` — 0 of 97

Could not be settled from the documents and the source, and saying so is worth more than a guess. **This section is empty**, and that is a claim: see *★★ Why nothing is `unknown`* in the header before you trust it.


---

## `declined` — 5 of 97

Deliberately not a surface here, with the argument. A `declined` row is the one that costs most when it is wrong — it tells the next reader the question has been settled — so each one carries the reasoning rather than a verdict.

### Document & pages

| Row (`FEATURES.md`, declined) | Why |
|---|---|
| Inspect and dump a document's own internal COS object … | Quoting the engine's row: *"No `pdfcer-gui` surface exists or is planned."* A COS object browser is a format-engineer's instrument, not a drawing tool — the audience for a reverse-reference map over `/ObjStm` membership is somebody already at a command line, and `pdfcer inspect` is that command line. ★ Declining this is not declining *diagnostics*: the Render-diagnostics dialog exists and is where an operator-facing finding goes. |
| Export a document's structure to an editable form and compile a hand … | Same argument as the row above, and the engine says the same thing in the same words: *"No `pdfcer-gui` surface exists or is planned."* Exporting a document's structure to an editable form and compiling a hand edit back is a **text-editor** workflow; a GUI that offered it would be offering a worse text editor. |

### Annotations & markup

| Row (`FEATURES.md`, declined) | Why |
|---|---|
| Note text on markup at author time — `/Contents`, `/T` and `/M` … | **Declined on the engine's own argument, which this shell reached independently and wrote down first.** Their row: *"a geometric markup has no text-entry moment (it is authored on mouse-release from geometry alone), so the route a reviewer UI actually needs is the sibling row directly below"*. Ours: *"a dialog on every shape a reviewer draws is the interaction nobody ships"*. Author-time note text is the wrong verb for this shell; `set_markup_note` on a finished shape is the right one, and it ships. |

### Fonts & rendering

| Row (`FEATURES.md`, declined) | Why |
|---|---|
| Form XObject viewport culling — a `Do` whose `/BBox`, mapped … | The cull is **lossless by construction** — §8.10.1 makes `/BBox` a clip, so a culled form cannot mark a pixel and the raster is byte-identical. `app::status::notes::findings` reports only findings that change *what the operator can see*, and it excludes `tolerated` and `compat_skipped` on exactly that argument. `forms_culled` belongs with those two, not with the nine. ★ It becomes wanted the day a **performance** readout exists; it is not wanted on a correctness one. |
| Decode `/BrotliDecode` streams (`Pass 123.0`, `4163ad9`) — read … | A decode filter has no control and should not have one. `/BrotliDecode` is read-only, reached through every path that opens a stream, and the operator's experience of it is that a file opens. ★ The only surfaceable fact would be a refusal — an inline image, or the nonstandard `/Br` abbreviation — and that already arrives as a load error rather than as a setting. |


---

## `shipped` — 79 of 97 (re-counted 2026-09-05)

**The engine's row is stale: an operator can reach this today.** Sixty-five of ninety (★ 64 → 65 on 2026-09-04, when the encryption-authoring row moved out of `blocked` on the operator's O119 ruling), which is the single largest finding of this triage. Each row names the surface or the call site, and where this project's own record dates the work, the date. These rows are kept, never deleted — `EDITABLE_SURFACES.md`'s own rule, and the argument is the valuable part.

### Document & pages

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Merge several files into one — three verbs share … | **Reachable — File ▸ Merge, `app::actions::pages::merge_into`.** It loads the source outside the edit closure, calls `merge_document` through `vector_edit` and reports `pages/fields/renamed/bookmarks` on the diagnostic channel. The engine's *"`pages.merge_into` falls through to `command-unimplemented`"* has been false since that wiring landed; the row is stale, not a gap. |
| Cut, copy and paste whole PAGES, within a document or between … | **Reachable — `app::dispatch::pageclip` calls `copy_pages`** and pastes through the same funnel, so a page clip is one `EditSession` command and one `Ctrl+Z`. The clip is a real openable PDF by the engine's own design, which is what lets it leave this process. |

### Text

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Text on a rotated baseline — a CAD title block's `Tm = [0 1 -1 0 e f]` … | **Consumed — `canvas::textsel` reads `line.direction`.** This is the field the engine's row says *"`pdfcer-gui` filed the request and has not yet consumed"*; it is read in the line model and in the selection geometry, which is what makes a swept selection follow a CAD title block's rotated baseline instead of boxing it page-axis. The row is stale. |
| Choose what pdfcer does when bold or italic needs … | **Reachable — Settings, honoured at `app::actions::textstyle`**, which reads `doc.settings.style_policy` and passes it into `FormatOptions`. The engine's row explains its `[ ]` as *"the three-option control shipped in `crates/pdfce-gui`'s settings window … `D:\dev\pdfcer-gui` has not been notified"* — it was notified, and the settings-completeness test would have failed the build if the key had no control. Stale. <!-- old-name-exempt: quoting the engine's own row, which names the deleted `crates/pdfce-gui` by its real historical path --> |
| Copy-on-write a shared form XObject onto one page … | **Reachable — `app::actions::xobject` calls `unshare_form`**, offered as *"Give this page its own copy"* with seven worded refusals (`EDITABLE_SURFACES.md`, the twelve-gap table). It is the option the engine's decision `112` names for an edit that would otherwise change every invocation. |
| Reflow within a block, including justified alignment. Not reachable … | **Reachable — Edit ▸ Reflow paragraph, and a right-click inside the text you are editing, 2026-08-28**, on the operator's own ask. `app::actions::textstyle` calls `reflow_block`. The engine quotes this project's *"three gates … Untouched"* line, which was true when written and stopped being true the day the command shipped; the save-and-reopen gate survives as a **worded refusal with a remedy**, which is a shipped behaviour rather than an absence. |
| Add new page text — point insert and wrapped multi-line … | **Reachable — Edit ▸ Add text**, click to place one line or **drag a rectangle** for a wrapped multi-line box (2026-08-21), committed with `Ctrl+Enter`; `app::actions::apply` calls `add_text`. The engine's *"no font, size or colour surface yet"* predates the Format work: new text is authored with a font, a size and a colour. Stale. |
| OCR as an edit to the open document, not a separate file — `EditSession::add_ocr_layer` … | **Reachable — `app::actions::apply` calls `add_ocr_layer`**, so recognition is an undoable edit to the open document. The engine's *"`gui` still routes through the free-function one-shot (`<stem>-recognised.pdf`, never in place)"* describes the route as it was; both routes exist now and the in-place one goes through the funnel. |

#### ★★★ Text export to a FILE — 2026-09-04, and it is a row this gate structurally cannot see

**Not a table row above, and the tally is deliberately unchanged**, because
this is not one of the ninety. `FEATURES.md`'s *"Extract and copy text — search
index, `ToUnicode`, reading order, page/document clipboard copy"* is
`[x] core / [x] gui`, so `check-engine-backlog.sh` never collects it, so nothing
in this file was ever going to mention it. Recorded here anyway, because the
**shape** is worth more than the row:

> A capability whose `gui` column is already `[x]` can still be **half
> reachable**, and no gate in this project can tell.

That row's `gui` tick was earned by `file.copy_page_text` and
`file.copy_document_text` — the **clipboard** half, shipped 2026-08-14 and wired
2026-08-20. The engine's own words in the row are *"page/document clipboard
copy"*, and they are accurate. What did not exist until 2026-09-04 was writing
that text to a **file**, which is the half an operator asks for when they want
to search a drawing set, diff two revisions, or paste a specification's words
into a specification.

* **The surface**: `file.export_text`, File ▸ Export, beside Export DXF /
  Export image / Export form data. Page scope through
  `imageexport::resolve_pages` → `dialogs::print::tabs::parse_page_range`;
  U+000C or a visible page marker; UTF-8, optional BOM, optional CRLF.
  `app::actions::export::text` + `app::actions::exporttext`.
* **The invariant**: at its defaults it writes byte-for-byte the string
  `file.copy_document_text` already puts on the clipboard. One answer to *"what
  is the text of this document"*, not two.
* **The refusal**: a document with no readable text refuses **before the save
  picker**, and names `File ▸ Recognise text`. A scanned drawing extracts
  successfully and returns nothing, and a zero-byte `.txt` looks exactly like a
  successful export.

★★ **The IMPORT half the operator asked for in the same sentence does not exist
in the engine at all**, and that is a genuine gap with nowhere else to live:
there is no document builder, no page-level text replace, and `add_ocr_layer`
takes positioned words rather than a file. `add_text` is the nearest verb and
stops at pagination — its overflow is *emitted, never clipped* (R76), so a
two-page text file would produce one page with the second page painted off the
sheet. Filed as
`open/request_there_is_no_route_from_a_text_file_back_into_a_pdf.md`; the
durable record is `app::actions::exporttext`'s module header. **No control was
drawn for it.**

### Vector objects (Inkscape-style editing)

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Edit a Bézier handle, with grab/hover/live preview. … | **Reachable — `app::actions::vector` calls `move_handle`**, and the stage table records Phase 1 complete *"but for the clipboard (selection, move, resize by grip and by typed number, multi-node move, Bézier handles)"*. The engine's evidence was `grep -i bezier` over the canvas on 2026-08-19, which is four days before the work; **an absence measured once is a claim with a date on it**, and this one expired. |
| Select several nodes and move them as one surgery, one … | **Reachable — `app::actions::vector` calls `move_nodes`** with a slice, so several nodes move as one surgery and one undo entry. Same stale measurement as the Bézier row above, from the same 2026-08-19 sweep. |
| Edit geometry INSIDE a form XObject — `move_node_in_form` … | **Reachable — `app::actions::vector` calls `move_node_in_form`** and its siblings, which is the whole point of the deep hit test below: click a line inside a title block and drag *that line*. The engine's disclosure data (`FormSurgeryOutcome { invocations, pages }`) is what the shell reports back, deliberately as two numbers rather than one. |
| Edit an image, text run or pasted object added in THIS session … | **Reachable by linking the fix, which is the whole of it.** `Pass 186.0` made every content-editing verb resolve a page through the session overlay; `Cargo.lock` pins `04f7ec0`, well past it, and this shell's call sites are unchanged by design. ★ This is the one row in the *shipped* column whose evidence is a **revision** rather than a surface — there is no new control to point at, and no driven check of this project asserts it. If a reader wants to overturn one verdict in this file, overturn this one. |
| Ask whether a page's model has changed — `EditSession::page_content_generation(page_index) -> u64` … | **Consumed — `app::cache` calls `page_content_generation`** to decide whether a cached decomposition still describes the page. This is the verb the engine's row says was *"asked for by name by `pdfcer-gui`"*; it is the answer to that ask, and it is in use. |

### ce dimensions

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Author a perimeter/path-length ce dimension — one number … | **Reachable — the Perimeter measure tool**; `canvas::dimdrag` handles `DimensionKind::Perimeter` and `app::actions::dimensions` calls `place_dimension`. Phase 7 shipped Linear, Two-line and Radius/diameter with snapping and dimension groups. |
| Rotate a placed ce dimension about any pivot — `rotate_dimension(id, pivot, degrees)` … | **Reachable — the ninth grip on the selection box, routed by kind** to `rotate_dimension` (`EDITABLE_SURFACES.md`). The engine's *"notified 2026-08-28"* is the notification; consuming it happened. |
| Override a ce dimension's printed text, or clear the override … | **Reachable — `app::actions::dimensions` calls `set_dimension_label`.** This is `2X <DIM> TYP` on a drawing: the caption is overridden and still tracks the geometry through a later re-scale, and clearing it restores the measurement with no re-measurement. The engine's *"not yet notified as of 2026-08-30"* is out of date. |

### Annotations & markup

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Revision clouds — a scalloped `/BE << /S /C /I n >>` border … | **Reachable — `markup.cloud`, its own glyph, `/BE /I 1.0`** (the operator's request #6). One of the eight markup kinds Phase 6 shipped. |
| Author a markup at an opacity, in one command and one undo entry … | **Reachable — Markup ▸ Style ▸ Opacity, 2026-08-28**, authored through `add_markup_with` so a translucent highlight costs **one** undo entry rather than author-then-restyle. The engine's *"the request originated there, the API now exists, consuming it is their work"* was discharged the day after it was written. |
| Write, correct or clear a note on an annotation that already exists — `EditSession::set_markup_note(annot_id, &MarkupNote)` … | **Reachable — the Comments panel writes, 2026-08-28**: Add note, Edit note, Remove note on every row, through `set_markup_note` / `clear_markup_note` at `app::actions::annots`. Correcting somebody else's typo deliberately leaves `/T` alone, so a fixed comment is not re-attributed to nobody. |
| Reorder a page's `/Annots` array — `reorder_annotations(page, &[ObjId])` … | **Reachable — O99, driven 2026-09-02: the tab-order list drags, with the caret.** `app::actions::reorder` calls `reorder_annotations`. The engine's *"its Tab-order panel is deliberately kept read-only until the drag is wired"* describes the state before that day; the drag is wired. |
| Resolve a link annotation's destination — `Annotation::destination` (one-shot … | **Consumed — `app::cache` calls `annot::page_link_destinations`**, so the links table resolves where each link goes. This is the *"dead-looking table of contents"* the engine's row names, and the reply it shipped the same day. |
| Move anything carrying a `/Rect` — markup, the four text markups … | **Reachable — dragging a markup moves it, driven 2026-08-28** (`dragging_a_markup_moves_it`, asserting travel in both axes). `app::actions::annots` calls `move_annotation`. Before that the gesture was *eaten* by a fork whose branches could both answer *not mine*, which is why the row was right for ten days and is wrong now. |
| Resize anything carrying a `/Rect` — `resize_annotation(annot_id, anchor, sx, sy, &ResizeOptions)` … | **Reachable — `app::actions::annots` calls `resize_annotation`** with the engine's anchor-and-factors shape, so a placed markup resizes by grip. The engine's *"notified 2026-08-28, consuming it is their work"* was consumed. |
| Rotate anything carrying a `/Rect` — `rotate_annotation(annot_id, anchor, degrees)` … | **Reachable — the ninth grip on the selection box**, calling `rotate_annotation` (`EDITABLE_SURFACES.md`). Widgets and ce dimensions route to their own verbs from the same grip, which is what the engine's *"refused by name"* pair is for. |
| Restyle a placed markup annotation — colour, interior fill, line … | **Reachable — the Properties panel's restyle section, 2026-08-19**: colour, line width and opacity on a selected annotation, through `set_markup_style`. The engine's evidence is *"zero call sites there, confirmed 2026-08-19"* — the same day, and the call site is `app::actions::annots`, whose header records being that verb's first caller. |

### Forms (AcroForm)

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Import and export form data — FDF, XFDF and two-column … | **Reachable — `app::actions::forms` calls `import_form_data` and `app::actions::export` calls `export_form_data`**, so FDF, XFDF and CSV all move. The engine quotes this project's own file listing them as ⬜, which it did before the Forms work. |
| Create a field — text, check box, radio, choice, push … | **Reachable — five commands on Edit ▸ Forms, 2026-08-26, on the operator's ask**: text field, check box, radio button, drop-down, push button; click to place or drag for an exact box, and the field exists once Add is pressed. The engine's *"it cannot create a field yet"* was true for nine days of a blocker that turned out not to exist. |
| Give a push button a declared action (`/A`) — `set_button_action` … | **Reachable — placing a button asks what pressing it does, seven ways, 2026-09-01**, driven by `a_placed_button_can_be_given_something_to_do`. This is the row `tools/gates/check-verb-coverage.sh` exists because of: the engine shipped `set_button_action` on 2026-08-30 with a note saying *"your surface is now saying something untrue"* and it sat unread for two days. |
| Read a push button's declared action — `EditSession::button_action(fqn)` … | **Reachable — `panels::forms::button` calls `doc.session.button_action(&fqn)`**, which is exactly the condition the engine's row sets: *"`pdfcer-gui` ticks `gui` when its panel ships"*. The panel ships, and the four states are what let it offer **replace** on an `Unmodelled` action and nothing on a `Foreign` one. |
| Delete a field, a single widget, or a grouping node and its subtree — selected … | **Reachable — `app::actions::forms::delete` calls `delete_field`, and `forms::groups` calls `delete_field_group`** behind Forms ▸ Field groups, previewed before the press (`EDITABLE_SURFACES.md`). The engine's *"the Forms panel is fill-only by design"* describes 2026-08-19. |
| Rename a field, reporting how many descendants the rename reached … | **Reachable — `app::actions::forms` calls `rename_field`**, and the outcome's `action_targets_retargeted` is what tells the operator how many buttons the rename repaired. Stale for the same reason as Delete above. |
| Move and resize a widget — a pure translation carries the artwork untouched … | **Reachable — `app::actions::forms` calls `move_widget` and `edit_widget`**, so a widget both moves and takes a new extent, with the engine's `Pass 187.0` redraw behind it. This is the row where a resize that did **not** redraw would ship magnified artwork, so the tick is worth having. |
| Rotate a widget — `rotate_widget(fqn, index, degrees)` / `rotate-widget` … | **Reachable — `app::actions::forms::widget` calls `rotate_widget`.** The engine quotes this project's own line *"a form field gets no circle at all"*, from 2026-08-28; the circle is offered now, and it routes by kind to the verb `/MK /R` needs rather than to `rotate_annotation`, which refuses a widget by name. |
| Change a field's properties after placing it, without delete-and-re-place … | **Reachable — `app::actions::forms` calls `edit_widget`, and the Properties panel carries the field-scope controls.** The engine's own split — field scope hits every widget, widget scope hits one — is the split the panel draws, because a required flag and a border colour are not the same kind of edit. |
| Copy, CUT and paste a form field — `cut_field` returns … | **Reachable — `canvas::fieldclip` calls `copy_field`, `app::actions::forms::paste` pastes**, with the operator's two chords (`Ctrl+V` a new field, `Ctrl+Shift+V` another widget of the same one). `cut_field` is deliberately **not** called: the panel's Cut is copy + `DeleteWidget`, because the operator pointed at a **box** and `cut_field` removes the whole field — argued at length in `EDITABLE_SURFACES.md`. |
| Recognise and disclose script-driven fields, and natively recompute a whitelisted … | **Reachable — the Forms panel's Recompute**, `panels::forms::edit`. No script is executed on either side; the whitelisted built-in subset is recomputed natively, and an ambiguous stored date is refused rather than guessed. |
| Read a widget's border and visibility — `forms::Widget::border` … | **Consumed — `panels::properties::widgetedit` reads `widget.annot_flags`** and words the unmappable case rather than collapsing it onto the nearest of the four visibilities. This is the reply the engine sent at 2026-08-27 23:13 and says is *"not yet consumed"*; the two properties controls it unblocks are drawn. |

### Fonts & rendering

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Rasterize an arbitrary page region, so magnification is bounded by viewport … | **Reachable — `render::worker` calls `render_page_region` for the canvas**, and `render::offpage` calls it for content outside the crop box. The engine's *"no GUI code path calls it at all"* is the single most consequential stale claim in this file: region rendering is what holds the cost of a zoom flat instead of quadratic, and it is what this shell's canvas is built on. |
| `/Separation`/`/DeviceN`/`Lab`/`CalGray`/`CalRGB` colour spaces on image … | **Reachable — it is what the canvas draws.** The shell rasterises through `render_page_with_view` / `render_page_region`, so an `/Indexed` duotone or a `/Separation` image XObject paints correctly on screen with no code of ours involved. The engine's *"GUI surface deliberately paused by the operator"* is a note about a **control** surface and about a project (`crates/pdfce-gui`) that has since been deleted. <!-- old-name-exempt: quoting the engine's own row, which names the deleted `crates/pdfce-gui` by its real historical path --> |
| Paint shading via the `sh` operator — axial and radial (types … | **Reachable — it is what the canvas draws.** Axial and radial `sh` shadings paint on screen through the same renderer the CLI uses. Same stale *"paused by the operator"* note as the row above. |
| Paint shading patterns (`PatternType 2`, named via `scn`) … | **Reachable — it is what the canvas draws.** A `PatternType 2` shading pattern selected by `scn` paints on screen, anchored to the pattern's own base CTM. Same stale note. |
| Paint mesh shadings — types 4 (free-form Gouraud … | **Reachable twice over — the canvas paints mesh shadings, and `mesh_patch_padding` has a control** in Settings ▸ Colour (`dialogs::settings::colour`). The engine's row names that setting as its own permanent-ambiguity switch and marks `gui` `[ ]`; the switch is drawn. |
| Subtractive (colorant) compositing buffer — a page whose group declares … | **Reachable — it is what the canvas draws.** A page whose group declares a subtractive space composites in four colorant planes on this shell's canvas exactly as it does at the CLI, because it is the same renderer. The 8.7 kB of argument in that row is all about the arithmetic, none of it about a control. |
| Choose how much memory pdfcer may spend blending … | **Reachable — Settings ▸ Colour carries `max_cmyk_buffer_bytes`** (`dialogs::settings::colour`). The engine's *"this column tracks `D:\dev\pdfcer-gui`, which asked for the API and has not wired it yet"* is stale: it was wired with the rest of the Colour group, and the settings-completeness test would have failed the build otherwise. |
| Page-group blend-space source, when `/CS` is undeclared … | **Reachable — Settings ▸ Colour carries `page_blend_space_source`** (`dialogs::settings::colour`), the three-value setting that decides whether an undeclared page group borrows a CMYK `/OutputIntent`. Surfaced with the rest of the Colour group; the engine's `[ ]` is stale. |
| Blend modes — the eleven separable modes (Multiply, Screen, Overlay … | **Reachable — it is what the canvas draws.** The eleven separable blend modes composite on screen in the group's own blending space. Same stale *"paused by the operator"* note as its siblings. |
| Non-separable blend modes — Hue, Saturation, Color, Luminosity … | **Reachable — it is what the canvas draws.** Hue, Saturation, Color and Luminosity are computed by pdfcer itself rather than handed to the rasteriser, on this canvas as at the CLI. |
| Transparency GROUP compositing — pdfcer owns the arithmetic (`pdfcer-render/src/compositor.rs`) … | **Reachable — it is what the canvas draws.** Transparency-group compositing, knockout groups and backdrop removal all happen in the raster this shell uploads. |
| Overprint SIMULATION — ISO 32000-1 §11.7.4.3 … | **Reachable — it is what the canvas draws.** Overprint simulation is on the screen; what is **not** on the screen is the counter that says when it was refused, and that is the row below, where it belongs. |
| Choose whether a grey fill knocks a spot backdrop out … | **Reachable — Settings ▸ Colour carries `overprint_zero_tint_scope`** (`dialogs::settings::colour`), *Grey over a spot colour in print-ready files*, surfaced under O100 beside its sibling. The engine's *"`D:\dev\pdfcer-gui` … has not been offered the setting"* is stale by a filing. |
| Soft masks from `ExtGState /SMask` — `/Alpha` and `/Luminosity` mask groups … | **Reachable — it is what the canvas draws.** `/Alpha` and `/Luminosity` soft masks are built and applied in the raster this shell uploads. |
| Type 3 font rendering — vector glyph procedures (`d0`/`d1` … | **Reachable — it is what the canvas draws.** Type 3 vector and bitmap glyph procedures render on screen. ★ Type 3 **search and copy** is a different row in *Text* with its own `/ToUnicode` gate, and rendering does not imply either. |
| Remove an embedded font's program, refusing by name (reason … | **Reachable — `app::actions::fonts` carries both document-level font verbs**, embedding and removing, and the Fonts panel's `removability` verdict is the control's own gate. The engine's *"a report, not an editor"* was measured 2026-08-19 and the editor arrived after it. |
| Embed a font that's referenced but missing — attach a program … | **Reachable — `app::actions::fonts::embed` calls `embed_fonts` through the funnel**, with `dialogs::embed` carrying the donor mapping. ★ Worth noting because the engine is explicit that it will **not** check the honesty of a donor match: `SuppliedFont::matched` is *"the inference rule 4 governs"*, so this shell owns that claim and its own module header says so. |
| Per-standard render presets (`Pass 128.1`) — a settings … | **Consumed — O100, 2026-09-02.** The presets are read through `entries()` and each set entry's `why` is printed, because `disclosures()` emitted a `why` only for keys a preset leaves alone. That work went back to the engine as a note, and it is the reason a preset's reasoning now leaves the crate at all. |

### Reading, navigation & printing

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Rename a bookmark, or delete one and its subtree — `set_outline_title(item_id, title)` … | **Reachable — `app::actions::bookmarks` calls `set_outline_title` and `delete_outline_item`** (`EDITABLE_SURFACES.md`: *"Bookmarks rename and remove"*). The subtree goes with the parent, as Acrobat does. |
| Reorder and re-parent a bookmark — `move_outline_item(item_id, OutlinePlacement)` … | **Reachable — `app::actions::bookmarks` calls `move_outline_item`.** These were the two verbs `EDITABLE_SURFACES.md` said to *"pick up the moment the engine commits them"* — they were uncommitted worktree at the time of that audit, they are in the lock now, and the panel drags. |
| Expand or collapse a bookmark, independent of moving it — `set_outline_open(item_id, open)` … | **Reachable — `app::actions::bookmarks` calls `set_outline_open`**, kept a separate act from the move for the engine's stated reason: a caller who wanted only one of the two could not undo just that half. The engine's *"No `pdfcer-gui` build reaches this yet"* is stale. |
| Cut, copy and paste a whole BOOKMARK SUBTREE, including between … | **Reachable — `panels::bookmarks::clip` calls `copy_outline_item` and `app::actions::bookmarks` calls `paste_outline_item`**, so a bookmark subtree moves between two open documents — which the engine's row notes **Acrobat cannot do at all**. |
| List, extract, attach and detach embedded attachments — detach removes … | **Reachable — Edit ▸ Insert ▸ Attachments, with extraction** (`EDITABLE_SURFACES.md`); `app::actions::attachments` calls `attach_file` / `detach_file` and words the attacker-controlled-name hazard rather than deriving a path from it. |
| Cut, copy and paste an EMBEDDED FILE — `copy_attachment` / `cut_attachment` … | **Reachable — SHIPPED 2026-09-01**: Copy and Cut on every document-level row, Paste at the top of the panel, drawn only when the clipboard holds one. `cut_attachment` is deliberately not called — the panel's Cut is `copy_attachment` then a `Detach` through the funnel, which is one undo entry and lets the copy fail before the delete is raised. |

### Shell & UX

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Build provenance stamp — `pdfcer --version` states the UTC build … | **Reachable — O101, driven 2026-09-02: the build time in the top bar.** The engine's row is honest about not having checked (*"gui not verified this filing … so this stays `[ ]` rather than being rounded up"*), which is the right posture and is also why the row is wrong. |

### Export

| Row (`FEATURES.md`, shipped) | Why |
|---|---|
| Encrypt a document (AES-256, `/R` 6 only), set … | **Reachable — File ▸ Security ▸ `Encrypt…` and `Permissions…`, wired 2026-09-04 (`OPERATOR_REQUESTS.md` O119).** `crate::protect` is the model, `crate::dialogs::protect` is the window, and all three verbs are called from one place — `protect::prepare`: `set_encryption` on the open session (it takes `&self`, so unsaved edits ride along), `set_permissions` and `remove_encryption` on a **throwaway** session re-opened from the file with the owner password. ★★★ The throwaway is not caution: both mutating verbs call `clear_encryption()` on the base, which would disarm `save_incremental`'s `EncryptedSaveUnsupported` guard and let the operator's next ordinary `Ctrl+S` append plaintext objects to a file of AES ciphertext, silently. It is also the authentication — `NotOwner { opened_as }` comes back from the load rather than from a second code path. ★ O119's three disclosures are all on screen and none waits for a press: `PERMISSIONS_DISCLOSURE` verbatim above the tick-boxes, the signed refusal **instead of** the form (with the count and the mechanism), and the owner-password note above the field it is about. The save is `dialogs::redact`'s mechanism part for part — new file by default, a suggestion that is never the source, replace behind one extra acknowledgement and no picker, atomic temp-then-rename. ⚠ **This row was moved from `blocked` on the ruling, not on a driven run**: the headless suite is green (`protect::tests`, `dialogs::protect::tests`) and the `ui-verify` checks are written and **were not run** — another agent held the desktop. It is `shipped` because the command is registered, on the ribbon and dispatched; the engine's *"ticked when the GUI half is driven"* bar has not been met and saying so here is cheaper than being asked. |
| Move, resize and rotate a content-stream object (path, text … | **Reachable — move, resize and rotate ANY object, 2026-08-20**, closing a request the operator made three times; `app::actions::vector` calls `transform_objects`, which wraps each object's operator run in `q <cm> … Q` and so is kind-agnostic by mechanism rather than by match arm. Driven by `resize_scales_a_shape` and `geometry_fields_resize_a_shape`. |
| Reach inside a form XObject for hit-testing — click and marquee … | **Reachable — clicking an object inside a form XObject selects THAT object, 2026-08-27**, and it closed the operator's largest single report: *"when I click on one of the objects all I get is the page selected"*. `hit_test_point_deep` is the pick and the marquee has the same reach, so the two gestures that both mean *select this* cannot disagree. |
| Import an installed Acrobat/Reader trust store … | **WIRED 2026-09-05 — `crate::trust`, and "importing" is a live READ rather than a copy.** Settings ▸ Digital signatures ▸ *Show what is in it* reads the operator's own `addressbook.acrodata` through `pdfcer_core::trust_store::load_from_path` and reports the anchor count by `/Source` together with the file's modification time; `crate::trust::candidate_paths` mirrors `pdfcer-cli`'s four-track list exactly, so the window and the command line look in the same places in the same order. ★★★ **Nothing is copied, and that is the design rather than a shortcut.** pdfcer keeps no anchor file of its own, takes no snapshot and caches no DER on disk: every evaluation reads the operator's file as it is at that moment. A snapshot has no way to say how old it is that anybody will read; a live read has one that costs nothing. ★★ Measured on this operator's own machine while the feature was built: the store is at `%APPDATA%\Adobe\Acrobat\DC\Security\addressbook.acrodata`, 3.4 MB, `%PPKLITE-2.1` header, **last written 2024-05-27** — sixteen months stale. That is exactly the condition a *"1,780 anchors ✓"* badge would have hidden, and it is why `text::trust::store_line` has no sibling that yields the count without the date. ⚠ **NOT DRIVEN.** The headless suite is green and `ui-verify`'s `signature_trust_is_reported_as_its_own_fact` was written and **not run** — the operator may be at his keyboard. The engine's *"ticked when the GUI half is driven"* bar has not been met, and the one thing most worth driving next is this control on his machine, where a real store exists. |
| Tell which layer (optional-content group) a selected page object is on … | **CONSUMED 2026-09-05 — the page-object half is built, and it is our own request coming back answered within hours.** Filed 2026-09-04 as `request_which_layer_is_this_object_on.md` when the layers-panel work found the relation unreachable: `vector::decompose` counted `/OC` sections and discarded the id, and the renderer resolved it and pushed a `bool`. `Pass 250.0` put `oc: Option<ObjId>` on `PathObject` (`decompose.rs:386`), `TextObject` (`:484`) and `ImageObject` (`:780`), read through `VectorObject::oc()` (`:1064`) and `FormLeaf::oc()` (`:1300`). `panels::layers::highlight` now folds that over the whole selection: the Layers panel plates the row, and **the status bar names the layer on the selection line**, because the canvas is the primary surface and a capability reachable only from a panel is one the operator must already know about. Driven by `ui-verify selecting_an_object_names_its_layer` on `layers/painted-layers.pdf` — two layers and an object on neither, so a build that highlights a constant cannot pass. ⚠ **That check has NOT been run**: written with the operator possibly at his machine, registered and never driven. ★★★ **Two divergences the second route found, and neither is worked around.** **D1:** `FormLeaf::oc()` delegates to the wrapped object, so a page-level `BDC /OC` enclosing the form's `Do` is not folded in — the engine's own doc comment calls it *"a documented partial for the nested case"*. `collect_form_leaves` has `img.oc` in hand and does not pass it down, so everything inside a form on layer *Grid* reports `None`, which the field contract defines as **on NO layer**. That is a wrong positive, not a missing answer. This shell repairs the depth-1 case by composing the engine's own two results (`objects[leaf.paint_order].oc()`), which re-parses nothing; **at depth > 1 it is unrepairable from outside** — the nested form container is deliberately absent from `leaves` — and is reported as `Unresolved::NestedForm` rather than guessed. **D2:** `collect_form_leaves` discards `nested.diagnostics` entirely, so `oc_unresolved` — the counter the engine nominates as *"how a shell tells the two apart"* — is blind to every form interior. A leaf under an unresolvable `BDC /OC` reports `None` with nothing to contradict it. ⇒ **The ask, if the engine wants it:** thread the enclosing `img.oc` into each leaf (or add `FormLeaf::effective_oc()`), and merge the nested decomposition's `oc_unresolved` into the page's. Both are inside `collect_form_leaves` and both are information the walk already holds. ★ **The refusal to fake it is what made the original arrive** — a 40-line shell-side re-parse would have shipped a second, weaker `/OC` implementation and nobody would ever have asked. The reverse relation (*which objects are on this layer*) is now derivable and is still not built: indicating them means marking the canvas, which Rule 4 forbids for an inference. |
| Apply a redaction as a SAVE-COMMITTED edit, not an immediate file write … | **RETRACTED 2026-09-05 — `shipped` while NOTHING CONSUMES IT, which is the opposite error from the seven above and the harder one to see.** `EditSession::apply_redactions` has **no call site in this shell**: `Pass 250.2` replaced the collapsing route entirely, `apply_into_session` was deleted, and `redact::sealed`’s pinned census recorded the count going **down**. The only surviving `apply_redactions(` call is the **free function**, which is the write-a-redacted-copy path and a different thing. ⚠ **`tools/verb-coverage.py` cannot catch this** — it counts a verb as covered when the NAME appears anywhere in the shell, and the name still appears as a free function and in doc comments, so a dead session verb reports as covered. ⇒ *A hit is weak evidence; only a call site is evidence.* The original cell follows and is kept for its account of the ruling he gave. **WIRED 2026-09-04 (evening) — `EditSession::apply_redactions`, `Pass 250.1`, `225db51`.** It is the default destination in the apply dialog: the removal lands in the open document, nothing is written, and Save / Save As decide where it goes. ★★★ **The property this row demanded was checked before wiring, and the engine does NOT enforce it the way the request asked — it removes the hazard instead, and that was verified rather than believed.** There is no refusal on `to_incremental_bytes`; the verb *collapses* the session onto the redacted bytes as a brand-new base with an empty dirty set, so an incremental save appends to already-clean bytes. Measured against `pdfcer-core` `8b24a0a`: straight after the apply the incremental output carries **no `/Prev` at all**, and after a further ordinary edit it carries one whose prior revision is the redacted base — the removed text is absent from the whole file in both states, on a synthetic fixture and on `fixtures/a1-titleblock.pdf`. The tests are `redact::tests::an_incremental_save_of_a_redacted_session_cannot_leak_the_removed_text` and `…both_save_modes…`, and both were falsified. ★ **What the engine's answer costs, disclosed rather than absorbed:** the verb FINALIZES — the whole undo log is cleared, not only the redaction — so the dialog states the step count above the confirm control, and `app::save::has_unsaved_edits` gained a third term because a collapsed session answers `is_modified() == false` and would otherwise have been closed without a prompt. |
| Undo-preserving DEFERRED redaction — a redaction STAGED at Save that leaves the live session and its FULL undo/redo history untouched … | **WIRED 2026-09-05 — `Pass 250.2`, `41095eb`, engine v0.38.0 (`b01964f`), and it REPLACED `Pass 250.1`'s collapsing route rather than joining it.** ★★★ The cost the operator was paying is gone: applying no longer clears the undo log, and his ruling *"finalizing the document and can't be undone is ok **for now**"* has had its *for now* spent. What he gets instead is a removal that is **armed** at the confirm control and **carried out at the save** — `crate::redact::stage_into_session` → `crate::app::save::write_copy` → `crate::redact::save_applying_pending`. **Why one route and not two:** two apply routes with different undo semantics, on one dialog, on the one operation that cannot be undone, is a choice the operator would have to understand in order to make it safely, so `apply_into_session` is deleted and `redact::sealed`'s pinned call count for `apply_redactions` went **down** from 2 to 1 in the same commit — an exact count rather than a ceiling is what made that deletion an edit somebody had to write down. **The five wiring points from the row this replaces, each discharged and each verified at `file:line` rather than from prose:** (1) the lock — the lead's; (2) `app::save`'s three verbs all converge on one `write_copy`, which forks on `has_pending_redaction()` — they would otherwise **all** start failing by name, since the engine refuses `to_incremental_bytes` at `pdfcer-core/src/edit.rs:8348` and `to_full_bytes` at `:8374`; (3) `prove_saved_bytes` is on that path and runs **twice**, once inside `save_applying_pending` over the report the removal itself produced and once in `write_copy` over the bytes about to be written, because the claims true of a set of bytes are the ones made by the removal that produced them and an edit between arming and saving changes them; (4) `text::redact::undo_will_be_cleared` is **deleted** and `removal_happens_at_save` stands in its place, in the same region, in the same warning role, saying the opposite and more surprising thing — *the page does not change* — and the third term of `app::save::has_unsaved_edits` moved from `has_applied_redaction()` (which can now never be true) to `has_pending_redaction()`, which is load-bearing on a document whose marks were already in the file it was opened from; (5) `redact::sealed` is re-pinned on **four** subjects rather than one. ★★★ **And Cancel exists, because a stageable operation that cannot be un-staged is a trap with teeth:** while a removal is armed the engine refuses both ordinary save modes, so an operator who changed his mind and had no way to say so could not save his document at all. `Phase::Staged` + `Action::PendingRedaction(Staging::Cancel)`. ★★ **Measured, not assumed, on the leak question the row demanded** — and the leak surface really is larger, because the un-redacted base is still live: `both_ordinary_save_modes_are_refused_by_name_while_staged` asserts `WriteError::RedactionPending` from **both** modes and that no bytes come back at all; `the_staged_save_removes_the_text_and_leaves_no_prior_revision` asserts absence in the raw bytes, absence in every decoded stream, no `/Prev`, and a positive control (`KEEPTHIS` survives); `a_real_drawing_survives_the_staged_route` repeats all of it on `fixtures/a1-titleblock.pdf` — compressed streams, subsetted embedded font — through pdfcer's own `extract-text`, with `DRAWING NO` as the positive control, and asserts the refusal there too, because a partial guard would hide on a compressed real document rather than on a four-object synthetic one. ⚠ **Two things this row does NOT claim.** The `ui-verify` check is rewritten (its E3/E4 phases now assert the staging disclosure, and the region was renamed `redact-apply-undo-note` → `redact-apply-staging-note` in both files) and **was not run** — no GUI was launched. And `EditSession::set_encryption` does **not** consult the pending flag, so `crate::protect::prepare` (`protect/mod.rs:675`) can write an encrypted file holding the un-redacted content and the marks; that hazard is new with this route, is unfixed, and is written up at length in `EDITABLE_SURFACES.md`. |
| Evaluate a signature's trust against those anchors … | **WIRED 2026-09-05 — `panels::signatures` calls `signature::verify_all_with_trust`, and the panel reports THREE facts and never one.** `crate::trust::examine` resolves the anchor pool and threads it in; the panel prints `Intact:`, `Covers:` and `Signer:` as three labelled lines per signature, in that order, which is also the order of increasing uncertainty. There is no badge, no tick, no colour that means "fine", and no arithmetic anywhere in `panels/signatures.rs` that combines two facts into a third. ★★★ **`NotChecked` renders as itself** — the words *"not checked"*, never a soft "no" and never omitted — and the shell supplies the half the engine structurally cannot: WHICH of four reasons applies. The engine reports `NotChecked` identically whether the operator opted out, has no store, typed a wrong path or has a corrupt one, because it was simply handed no anchors; `crate::trust::Anchors` has four variants because those four call for four opposite actions, and `panels::signatures::tests::the_reason_trust_was_not_checked_is_specific_to_the_state` refuses the cheap single-sentence implementation. ★★ The `Trusted` sentence carries what it did NOT check **in the same sentence** — revocation always, and validity dates when no signing-time clock existed — rather than in a footnote, and `a_trusted_verdict_never_claims_more_than_the_engine_checked` is what stops a later edit shortening it to what every other reader says. ★ **Integrity came along with it**, so `FEATURES.md`'s separate *"Verify a signature's integrity and coverage"* row is also answered: `verify_all_with_trust(.., None)` is by the engine's own documentation identical to `verify_all`, so the opted-out path is the same call with an empty hand rather than a second code path that can disagree. ⚠ **NOT DRIVEN**, and one thing is **not verified by anything here**: that a signature ever reads `trusted`. That needs a signer chaining to a real AATL/EUTL anchor, which needs somebody's real certificate in this repository (it expires) and the operator's own store on whatever machine runs the suite (it makes the verdict a report about the machine). The engine's own `trust_chain` tests cover it against synthetic chains; this shell's coverage of the positive path is **zero** and saying so is cheaper than being asked. |
| Persistent **"use Acrobat trust store (at own risk)" setting** … | **WIRED 2026-09-05 — Settings ▸ Digital signatures, off by default, with the store's path and its MTIME disclosed.** ★ This row's first cell was REWORDED on 2026-09-05 to follow the engine's own new opening clause; the entry is the same one that read *"Persistently use an Acrobat trust store as pdfcer's own …"*, and its original argument is kept below rather than replaced, per this file's standing rule. **The argument, unchanged and now discharged:** *importing a store is one act; keeping it as the anchor set across sessions is another, and it is a preference with a privacy shape — it reads a file belonging to another vendor's product. Belongs in Settings beside the other opt-ins, off by default, with the store's path and its mtime disclosed, because an anchor set that silently went stale is worse than one that was never imported.* **What was built:** two controls in one new group. The permission is the ENGINE's `Settings::acrobat_trust_store` (`Off` / `AtOwnRisk`), so the same choice governs `pdfcer verify-signatures` — one answer to *"may pdfcer read Acrobat's trust list?"*, in one file, for both front ends. The location is a SHELL preference (`Prefs::acrobat_trust_store_path`), because the engine deliberately models none: *"locating the file is the shell's job."* ★★★ **This is the row that made `dialogs::settings::tests::every_setting_the_store_carries_has_a_control_in_this_window` go red**, and the red was correct: an earlier session registered the engine setting and never gave it a control, so the only way to change it was hand-editing `settings.txt`. That test is the gate, and the fix was a control rather than an edit to it. ★★ **R9, both ways.** The *Show what is in it* button is ABSENT when no store was found — an unavailable capability renders nothing — while the path field is drawn **always**, because it is the remedy, and an absent capability whose remedy is also absent is a dead end. ⚠ **NOT DRIVEN**; see the two rows above. |
| Trust path validation — the DETERMINISTIC, no-network parts of RFC 5280 … | **`shipped` from this shell's side, and it arrived with NO NOTE at all — caught by this gate on the engine bump to v0.38.0.** `Pass 10.5` added certificate validity dates against the signing-time clock (§4.1.2.5, checked only when a clock exists), CA/`keyUsage` constraints on intermediates (§4.2.1.9, §4.2.1.3) and RSA-PSS certificate signatures (RFC 4055), and surfaced `PathChecks { validity_checked, constraints_checked, revocation_checked }` plus `Trust::Trusted { validity_checked }`. ★★★ **The shell's whole obligation here is DISCLOSURE, and it is discharged in the verdict sentence rather than in a footnote.** `text::trust::trusted` states, in one sentence: which anchor was reached, its `/Source` provenance, that every link was checked by signature and every issuer was entitled to issue, whether the dates were checked **or explicitly were not because there was no signing time**, and that revocation was NOT checked — with the reason, which is that `pdfcer-core` never touches the network. A certificate revoked the day after it was issued chains exactly as well as one that was not, and an operator who is not told that has been given a stronger answer than pdfcer computed. ★ `PathChecks` itself is not read by this shell: `verify_all_with_trust` already folds `checks.validity_checked` into `Trust::Trusted` and words the rest into `notes`, which the panel prints verbatim. Reading the struct as well would be a second derivation of one fact, which is how two surfaces come to disagree. ⚠ **Revocation stays deferred** (`Pass 10.6`, engine *Backlog*) and this shell does not fetch either — no CRL, no OCSP, no DSS/LTV parsing. If that lands it is a NEW disclosure, not a stronger one: the sentence above must gain a clause rather than lose one. |
| Copy, cut and paste a selection, within a document or across documents/sessions/processes … | **SHIPPED 2026-09-05, both halves, and the audit it produced is the row’s most important content.** `canvas::clipboard::copy` now makes ONE `copy_selection(page, &objects, &annots)` call with both index lists, and `paste_objects` plants both halves from one action. ★★★ **But a mixed marquee cannot be REACHED**, and the reason is the selection model rather than the clipboard: `canvas::selection::SelectionState` holds `annot: Option<AnnotSelection>` and makes content and annotations mutually exclusive by construction — *"One canvas, one selection."* So the clipboard is the half that is ready and the selection model is the half that is not; that is a different file, a different subject and another track’s. ★★ **Two engine facts a shell should know before it wires this:** (1) `paste_objects` commits its content command and THEN calls `paste_clip_annotations`, which authors through `add_markup`/`add_dimension` — so a **mixed paste is two undo entries**, disclosed by the engine in its own doc comment and not something a shell can fold together; (2) there is a `cut_selection` (`edit.rs:11324`) this shell deliberately does not call, for the standing reason it does not call `cut_objects` — the delete goes through the funnel so it lands one command by the same mechanism as every other edit — which means a **mixed cut is likewise two entries**. Both are named rather than hidden. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Copy and paste almost any annotation, with its appearance intact — sticky … | **SHIPPED 2026-09-05 — and it exposed a divergence in `pdfcer-core` that the row’s own promise does not hold for.** A sticky note, a stamp, a text box, a link and a file attachment now copy and paste with their baked `/AP`, through `copy_selection` + `paste_objects`. ★★★ **THE FINDING, and it is an ASK:** `EditSession::clip_annotation` (`edit.rs:10599`) tries `annot_author::spec_from_dict` FIRST and only falls back to the raw carrier on `Err`, and `paste_clip_annotations` (`edit.rs:10901`) plants a `ClipAnnotation::Markup` with **`add_markup`, not `add_markup_with`**. So for every subtype pdfcer MODELS — `/Square`, `/Circle`, `/Line`, `/Ink`, `/Polygon`, `/PolyLine`, the cloud, text markup — the "lossless" clipboard **drops `/CA`, `/T`, `/M` and `/Contents`**, which is exactly the loss `RawAnnotation`’s own doc comment (`vector/clip.rs:215`) lists as the model route’s cost and says `Pass 170.0` was written to end. It ended it for the kinds the model does not reach, and not for the kinds it does. **This shell therefore keeps its spec-plus-`MarkupOptions` route for that one shape**, forked on the engine’s own carrier choice read off the clip (`canvas::annotclip::Plan`), never on a subtype list here. ⇒ **The ask: pass a `MarkupOptions` through the `Markup` carrier and plant it with `add_markup_with`** — or fall back to `clip_raw_annotation` for a modelled markup that carries any of the four keys. Either would let `canvas::annotclip`’s whole spec route be deleted, and the test `the_engine_models_a_square_and_carries_a_sticky_note_whole` goes red on the day it lands, which is how this shell will find out. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Verify a signature's integrity and coverage — `signature::verify_all` … | **CORRECTED 2026-09-05 — this cell was wrong, and so was the engine’s `[ ]`.** It read *"Half consumed … `verify_all` is called nowhere, so it cannot say whether the covered bytes were **altered**."* `crate::trust::examine` calls `signature::verify_all_with_trust`, `panels::signatures` consumes it, and the panel prints the *Intact:* line. ★★ The row two screens below (trust evaluation) already said so on the same day and this one was never re-read — **one file disagreeing with itself about one afternoon**, which is the exact shape of the drift this register exists to catch in the engine’s file. The engine's three facts never collapse into a bool, and trust is honestly `NotChecked` — which is exactly the shape a shell can surface without pretending to a certificate store. The entry point went to this project on 2026-09-03. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Export page(s) to PNG or JPEG, with real transparency (PNG … | **BUILT — corrected 2026-09-05.** `file.export_image` is registered, dispatched and on File ▸ Export; `app::actions::export` calls `encode_png` and `encode_jpeg`, including the transparent path. ⚠ **The cell below is the account of the gap and is kept because the argument is the valuable part — but the gap is closed.** He asked the **engine** for PNG/JPEG/SVG export on 2026-09-03; the engine shipped it that day and sent a note; this shell built none of it and had no row. `export::encode_png(&pixmap, Some(dpi))` with `PageBackdrop::Transparent` is the whole call. ★ The `pHYs` DPI is not a detail: without it Word places a 300 dpi page four times too large, which is the difference between *supporting export* and *the thing you paste being the right size*. A transparent JPEG must be **refused by name**, never silently flattened. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Export page(s) to SVG 1.1 (vector), from the renderer's … | **BUILT — corrected 2026-09-05:** `app::actions::export` calls it, from the same registered `file.export_image` window as PNG and JPEG. **O120.** `svg::export_svg_view(...)` → `SvgExport { svg, outcome }`, from the renderer's own display-list recording, so it carries images, clips, transparency and blend modes rather than the editing model. Axial and focal-radial shadings go out as native gradients; what cannot be expressed exactly comes back in `ExportTally` and goes **off-canvas**, after the export, per rule 4. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Export page(s) to EMF (Windows Enhanced Metafile) — a hand-rolled … | **BUILT — corrected 2026-09-05.** ★★★ **Its stated reason for staying `wanted` was a bar this file does not otherwise use.** It read *"stays `wanted` … the window has not been driven in a running binary"*, while four other rows in the same file are `shipped` and say ⚠ NOT DRIVEN in the same breath. This file’s own definition of `shipped` is *"the engine’s `[ ]` is not supported by this repository’s contents"* — a question about **source**, not about a driven run. Undriven-ness belongs in `FEATURES.md`, which has a bar for it; mixing the two bars inside one register makes every row in it ambiguous. Original cell follows. `emf::export_emf_view(...)` is called by `app::actions::export::emf_bytes`; `ImageFormat::Emf` is the fourth radio in the Export-image window, with `EmfOptions::background: None` for the transparent case (*"EMF's natural state"*, per the engine's own CLI). The disclosure names all five reasons a part became a bitmap — see-through solids, blend modes, gradients, images, transparency groups — plus the LibreOffice 24 nonzero-fill warning, because that reader is the entire reason the format is offered. ⚠ `EmfOutcome` is `#[non_exhaustive]` **with no `Default`**, so no test outside `pdfcer-render` can build one; the shell copies it into its own `EmfCounts` purely so the counters-to-sentences mapping is testable. If the engine adds a counter, that copy must gain the field in the same commit or the disclosure silently omits it. ★ Still `wanted`: the window has not been driven in a running binary. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
| Copy page content to the OS clipboard as editable vector (Word/PowerPoint/Excel/Inkscape/LibreOffice) … | **BUILT — corrected 2026-09-05, and this was the most stale row in either project’s file.** ★★★ **Both blockers it named are gone, and one of them was cleared exactly as the row predicted.** It said the placement half *"belongs in a `crates/native-clipboard` — a new manifest plus a `members` entry"*: that crate exists, is a workspace member, carries the `unsafe` in its own manifest, and is consumed at `clipboard::place` with all four slots, behind the registered `edit.copy_as_vector`. ⇒ **A row that correctly designs the fix is the most dangerous kind to leave open**, because the fix arriving looks exactly like the row’s own proposal and nothing re-reads it. Original cell follows. **⚠ NOT BUILT, AND DELIBERATELY SO — this is the one row where a partial build is worse than none.** The format order is measured, not chosen — SVG, then EMF, then PNG, then `CF_DIBV5`, all in one transaction; place only the raster formats and Word degrades the paste to a flat picture that looks correct and cannot be scaled or ungrouped. **Two things block it, neither fixable from inside `crates/pdfcer-gui/src/`:** (1) `clipboard-win` 5.4.1 and `windows` 0.62.2 are already **linked into this binary** — measured with `cargo tree -p pdfcer-gui -i`, not read off a lockfile line: `clipboard-win` under `arboard` ‹ `egui-winit` ‹ `eframe`, and `windows` by two routes, under `accesskit_windows` ‹ `eframe` and under `pdfcer-print`, which is a direct dependency. Neither is a **direct** dependency of `pdfcer-gui` — adding them is a manifest edit; `arboard`, which this crate can already reach, has no registered-format API and so cannot place entries 1 or 3. (2) `CF_ENHMETAFILE` is a GDI handle, so it needs `SetEnhMetaFileBits` + `SetClipboardData` — `unsafe`, which `#![forbid(unsafe_code)]` in `lib.rs` and `main.rs` will not host and which `forbid` will not let a module relax. ⇒ The project has answered this exact question once already: `crates/native-window` is a whole crate for four `user32` calls, and the placement half belongs in a `crates/native-clipboard` on that precedent — a new manifest plus a `members` entry. **What IS built and proved:** `crate::clipboard` carries the ordered `ORDER`, `svg_payload` (Chromium's UTF-8-plus-one-NUL, what Office was validated against), `dib_v5` (premultiplied top-down BGRA, `BI_BITFIELDS`, sRGB) and `CopyPayload::degrades_word_to_a_picture`, all under test, so the next pass adds four Win32 calls rather than re-deriving a measurement. **No test touches the real clipboard**, on purpose — it is global state on the operator's machine and a test that placed bytes would destroy whatever he had copied. ★★★ **MOVED FROM `wanted` TO `shipped` 2026-09-05, on re-measurement.** The verdict SECTION had not moved even where this cell’s own first word already said SHIPPED — `check-engine-backlog.sh` keys on a row’s opening clause and, by its own header, *"does not judge the verdict"*, so a row filed under the wrong heading passes silently forever. |
