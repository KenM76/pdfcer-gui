# Every surface in pdfcer-gui that shows or accepts a length

**Measured 2026-09-13** against the working tree at that date. ⚠ **Two engine
pins, and the difference matters:** §1 (the 55-surface table) and §3 — §6 were
measured at `d86cb19`; §2 and §7 were **re-measured at `3e73a02`** after
`reply_G013` shipped three units and widened `Unit::all()` the same morning.
Where a section names its pin, believe the section, not this header.

This document is **the deliverable of O194 clause 1**, not a summary of it.

---

## Why this file exists, and why it is a file rather than a paragraph

The operator's report was:

> *"we need units (including km and miles) added as options to everything. Right
> now things in some places only have points as a dimension type."*

★★★ **The load-bearing words are "in some places".** He could not name the
places, and neither could we, because **nobody had ever listed them**. A defect
whose extent is unknown cannot be scheduled, cannot be checked off, and cannot be
proven fixed — every fix is followed by him finding another one, and from where
he sits that reads as the same bug never being fixed.

So the enumeration came first. It is written down, in the repository, with
`file:line`, because:

1. **A fix is only complete against a list.** "Units are everywhere now" is a
   claim; "all 55 rows below are green" is a measurement.
2. **The list decays.** A new dialogue with a `DragValue` and a `" pt"` suffix is
   added by ordinary work, not by anyone deciding to add a unit defect. This file
   is the thing a future session diffs against.
3. **The list already found two live defects that have nothing to do with
   units** — the conversion disagreement in §3 and the `text/markup.rs` size
   ceiling in §5. Enumerating a thing is how you find out what is wrong with it.

⚠ **Rule 15 throughout.** Every row concerns **ce dimensions** — the lengths
pdfcer authors and displays. The unit a **pdf dimension** was exported with is
CAD-exported page content and is not in scope: a unit choice that re-authors what
the CAD system drew is a defect, not a feature.

---

## 1. The table

Columns: what the operator would call it · where it lives · what it shows or
takes · what unit choice is offered **today** · the formatter or parser behind it.

### 1a. Surfaces that already follow the operator's chosen unit — 3

These are correct today and are the model the rest should reach.

| # | Surface | file:line | Unit behaviour |
|---|---|---|---|
| 1 | The ruler's tick labels | `canvas/rulers.rs:573-575` | follows `DEFAULT_GROUP_ID`'s unit via `pdfcer_core::dimension::format_measurement`; falls back to raw `pt` when `ScaleState::NeverSet` |
| 35 | The measure tool's live perimeter total | `app/toolstatus.rs:311-316` | follows the pick's group — `format_measurement` |
| 36 | The measure tool's live radius / diameter | `app/toolstatus.rs:351` | follows the pick's group — `format_measurement` |

⚠ Two caveats on the ruler, both pre-existing and both recorded here rather than
silently inherited:

- `canvas/rulers.rs:496-508` (`Scale::default`) — when nothing is calibrated the
  `NeverSet` branch renders ` pt` and **ignores the unit field entirely**. There
  is no `Unit::Point` in the engine (deliberately — `units.rs:495-498`), so this
  is not a missing dropdown entry, it is the absence of a scale.
- `canvas/rulers.rs:545-552` (`Scale::of`) — the ruler reads the **default**
  group, never the *active* one. Documented in place as an open behaviour
  question. It is not a unit defect and must not be folded into one.

### 1b. Surfaces that offer a unit menu — 4 controls

| # | Surface | file:line | Menu contents |
|---|---|---|---|
| 53a | Set Scale — *Paper measured in* | `dialogs/scale.rs:391-394`, combo at `:506-513` | `Unit::all()` — **9 entries** since `G013` |
| 53b | Set Scale — *Show dimensions in* | `dialogs/scale.rs:396-398` | `Unit::all()` — **9 entries** since `G013` |
| 55a | Dimension Groups — the selected group's unit | `panels/dimension_groups/mod.rs:609-641` | `Unit::all()` |
| 55b | Dimension Groups — a new group's starting unit | `panels/dimension_groups/mod.rs:720-737` | `Unit::all()`, default `Millimeter` |
| 37 | Per-dimension unit override | `panels/properties/dimension/overrides.rs:114-130` | `Unit::all()` |

Names come from `text/scale.rs:132-141` (`unit_name`), which
`text/panels/dimension.rs:340-342` delegates to.

### 1c. ★★★ Hard points, no control — ~18 surfaces

**This is what he is reporting.** Every row prints or takes a document length in
points with no unit control anywhere on the surface.

| # | Surface | file:line | What it is |
|---|---|---|---|
| 4 | Properties panel — typed X / Y / W / H | `panels/properties/geometry.rs:764-771, :888` | four `DragValue`s, `fixed_decimals(2)`, **no suffix at all** |
| 5 | ★★★ The note under *Position and size* | `text/panels/properties.rs:415-417` | *"Points, measured to the bottom-left corner. Y increases upward."* — **the literal sentence he is describing** |
| 6 | Properties panel — position readout | `text/panels/properties.rs:271-272` | `"{x:.1}, {y:.1} pt"` |
| 7 | Properties panel — size readout | `text/panels/properties.rs:305-306` | `"{width:.1} × {height:.1} pt"` |
| 8 | Properties panel — line width | `text/panels/properties.rs:294-295` | `"{width:.2} pt"` |
| 9 | Markup line-weight spinner (ribbon) | `app/markupband.rs:1132` + `text/panels/properties/markup.rs:83-85` | fixed `" pt"` |
| 10 | Markup line-weight spinner (properties) | `panels/properties/markup/rows.rs:310` | fixed `" pt"` |
| 11 | Pen width on the markup swatch | `canvas/markup/swatch.rs:217` + `text/markup.rs:355-357` | fixed `" pt"` |
| 12 | ⚠ Dimension group text height / arrow size / gap | `panels/dimension_groups/style.rs:148, :168, :187` + `text/dimension_groups.rs:490-492` | fixed `" pt"` — **inside the dimensioning feature itself.** See §4: this one is arguably correct |
| 13 | Import-text margin and size | `dialogs/import_text.rs:306, :330` + `text/import_text.rs:110-112` | fixed `" pt"` |
| 20 | Page-size off-sheet warning | `text/page_size.rs:295-306` | `"{right:.0} pt past the right edge"` ×4 |
| 38 | Tolerance ± magnitude | `panels/properties/dimension/tolerance.rs:51-55, :100` | **no suffix, deliberately**; the unit is named once in a note below the fields |
| 39 | Stamp size chooser | `text/textannot.rs:246, :262-265`; `dialogs/textannot.rs:969` | `StampSize::Points(pt)` → `"{pt} pt"` |
| 45 | ⚠ Widget border-width spinner | `panels/properties/widgetedit.rs:518` | **no suffix, no unit, no note** — bare `range(0.0..=72.0)`. The only surface that shows a length with no unit stated anywhere |
| 40 | Stamp text size | `panels/properties/markup/textannot.rs:693` + `text/panels/textannotstyle.rs:234-236` | fixed `" pt"` — **see §4, excluded** |
| 41 | Font size on the ribbon | `app/fontband.rs:356` + `text/panels/properties.rs:639-640` | fixed `" pt"` — **see §4, excluded** |
| 42 | Font size in properties | `panels/properties/text.rs:748` | fixed `" pt"` — **see §4, excluded** |
| 43 | Text-pen size | `panels/properties/tool.rs:212` + `text/tool.rs:636-638` | fixed `" pt"` — **see §4, excluded** |
| 44 | Form-field font size | `text/forms/mod.rs:583, :1071, :1101, :1122` | `"{sz} pt"` — **see §4, excluded** |

### 1d. Hard millimetres, no control — ~12 surfaces

Equally a defect, and easier to miss because millimetres *look* like a real
answer. An operator working in feet gets no more choice here than in §1c.

| # | Surface | file:line | What it is |
|---|---|---|---|
| 14 | Insert-image X / Y / W / H | `dialogs/insert_image.rs:611` + `text/images.rs:193-195` | fixed `" mm"` |
| 15 | Image's declared size readout | `text/images.rs:121` | `"{w:.0} × {h:.0} mm at the {x:.0} dpi…"` |
| 16 | ⚠ Page-size Width / Height | `text/page_size.rs:226, :232` | **`"Width (mm)"`, `"Height (mm)"` — the unit is welded into the label**, not a suffix |
| 17 | Page-size sheet summary | `text/page_size.rs:237-241` | pt **and** mm, both fixed |
| 18 | Page-size preset list | `text/page_size.rs:153-155` | `"{name} — {:.0} × {:.0} mm"` |
| 19 | Page-size custom-range refusal | `text/page_size.rs:247-251` | mm only |
| 21 | ⚠ New-document Width / Height | `text/new_document.rs:229, :235` | unit welded into the label; `text/new_document.rs:21-35` is an **explicit documented refusal to add a unit toggle** — see §6 |
| 22 | ⚠ New-document preset list | `text/new_document.rs:128-135` | **mixed and implicit** — ISO entries in mm, US/ANSI entries in inches, chosen per entry by the code, never by him |
| 23 | New-document inch fraction | `text/new_document.rs:180-197` | a **second** fractional-inch formatter, nearest 1/16, independent of core's `FeetInches` |
| 24 | New-document sheet summary | `text/new_document.rs:238-260` | mm + in + pt, all three, fixed |
| 32 | Page thumbnail tooltip | `text/pages.rs:179-182` ← `panels/pages/mod.rs:203` | mm only |
| 33 | Document Properties — page size | `text/panels/docprops.rs:241-242` | mm only |
| 34 | Document Properties — mixed-size row | `text/panels/docprops.rs:253-254` | mm only |

### 1e. Print — 5 conversion sites, millimetres only

All in `text/print.rs`, each re-declaring the same closure. Listed separately
because the file has **63 lines of headroom** and needs all five replaced.

| # | Surface | file:line |
|---|---|---|
| 25 | Paper list | `text/print.rs:599-601` |
| 26 | Paper-match note | `text/print.rs:646-652` |
| 27 | Oversize note | `text/print.rs:694-700` |
| 28 | Planned-sheet note | `text/print.rs:768-772` |
| 29 | Paper disclosure | `text/print.rs:820-826` (pt **and** mm) |

### 1f. Surfaces with no readout at all — 2

Not unit defects. Recorded so a future reader does not "discover" them.

| # | Surface | file:line | Why it is here |
|---|---|---|---|
| 46 | Guides | `canvas/guides.rs` (module) | a guide's position is **never shown to the operator**. There is no readout to put a unit on. ⇒ a *missing feature*, not a missing unit |
| 47 | Grid | `canvas/grid.rs:103, :134` | spacing is derived from the ruler ladder; there is no spacing entry to unit-switch |

### 1g. Adjacent, and deliberately out of scope — 6

| # | Surface | file:line | Why out of scope |
|---|---|---|---|
| 30 | Print scaling | `text/print.rs:374-392`; `dialogs/print/tabs.rs:278` | a **percentage**, not a length |
| 31 | Print resolution | `dialogs/print/tabs.rs:586` + `text/print.rs:909-911` | a **DPI** — inches are implied by the unit itself and it is not switchable in any print dialogue anywhere |
| 48 | DXF export *Units* control | `text/export_dxf.rs:141-166`; enum at `pdfcer-core/src/export/dxf.rs:156-163` | a **file-format field** (`$INSUNITS`), separate vocabulary, 2 entries. See §6 |
| 49 | DXF scale field | `text/export_dxf.rs:86, :136` | a unitless ratio |
| 51 | Parallel-tolerance slider | `dialogs/settings/measuring.rs:95` | an **angle** (`" °"`) |
| 52 | Markup opacity | `app/markupband.rs:1181`; `canvas/markup/swatch.rs:267` | a **percentage** |
| 50 | Find panel position | `text/find.rs:182-188` | `n of N`, not a coordinate |

---

## 2. The unit model

### `pdfcer_core::dimension::Unit` — `units.rs:46-72` at pin `3e73a02`

**Nine variants, not `#[non_exhaustive]`**, and that is load-bearing:

| variant | line | `abbrev()` :127-137 | `baseline_per_point()` :161-171 | `default_format()` :198-203 |
|---|---|---|---|---|
| `Millimeter` | :48 | `mm` | 25.4 / 72 | 2 dp |
| `Centimeter` | :50 | `cm` | 2.54 / 72 | 2 dp |
| `Meter` | :52 | `m` | 0.0254 / 72 | 3 dp |
| `Kilometer` | :54 | `km` | 0.000 025 4 / 72 | **4 dp** |
| `Inch` | :56 | `in` | 1 / 72 | 2 dp |
| `DecimalFeet` | :58 | `ft` | 1 / 864 | 2 dp |
| `FeetInches` | :61 | `ft` | 1 / 864 | 1/8 |
| `Yard` | :70 | `yd` | 1 / 2 592 | **3 dp** |
| `Mile` | :72 | `mi` | 1 / 4 561 920 | **4 dp** |

★★ **This table said *six variants* until 2026-09-13 and it was correct when
it was written.** The three new rows arrived with `reply_G013` at 09:19 the same
morning. The decimal places are the engine's call and are not ours to
second-guess: a yard takes **three**, not the two its magnitude would suggest,
because the third is the inch — `units.rs:193-197` argues it. Kilometres and
miles take four for the same reason at the other end of the scale.

★ **`Unit::all()` is `pub const fn all() -> &'static [Unit]`** (`:245`),
ordered *metric ascending, then imperial ascending*, `ft-in` beside `ft` because
it is a presentation of feet rather than a step in magnitude. It returned
`[Unit; 6]` until `G013`; the widening is what let all three of our dropdowns go
from six entries to nine **without one of them being edited**.

Supporting, all measured at pin `3e73a02`: `FractionMode` (`:302`), `NumberFormat`
(`:325`), `DecimalMarker` (`:352`), `ScaleState` (`:570`). ★ **These four moved by
roughly 150 lines when `G013` landed and nothing warned about it** — a line
number into another crate is the most perishable citation this file makes, and
the only defence is to re-measure them in the same breath as quoting them.

★ **There is no `Unit::Point`, and that is correct.** Verified at pin
`3e73a02`: the only `Point` in `units.rs` is `DecimalMarker::Point` (`:355`),
which is the decimal separator, not a length. Every ` pt` in the tables above is
therefore **the absence of a unit**, not a unit the operator selected — which is
precisely why his sentence *"only have points as a dimension type"* names the
symptom so accurately.

★★★ **This paragraph carried a fabricated quotation until 2026-09-13 and the
correction is kept here rather than quietly applied.** It read:
*"`units.rs:495-498`: 'points are what a measurement is before a unit is
chosen.'"* **That sentence does not exist** — not at that line, not in that
file, not anywhere in `pdfcer-core`, at this pin or in the live read-only tree.
It was a sentence I wrote to support the argument I was making, given a line
number, and it read exactly like a citation. The **claim** is true and is now
stated in a form anyone can falsify in one grep. ⇒ **A citation is a
measurement. If it is not a copy of something that is on disk, it is a
paraphrase, and it must be spelled as one.**

★ **`Unit` must stay exhaustive, and on 2026-09-13 that stopped being an
argument and became a measurement.** Two matches in this crate are exhaustive on
purpose — `unit_name` in `text/scale.rs` and `unit_abbrev` in
`text/dimension_groups.rs`. When the engine shipped three units **both refused to
compile** until the arms existed, which is precisely the outcome the paragraph
predicted. A `_ =>` arm would have let all three reach the operator with no
English name and **no gate would have seen it**. The compile error *is* the
instrument.

★★★ **And the test that was supposed to be the instrument was not.**
`every_unit_is_named_distinctly` carried a **hand-written list of six variants**.
It would have passed, green and silent, with `Kilometer`, `Yard` and `Mile`
unlabelled — the guard that actually caught them was the compiler next door, and
the test had been decoration since it was written. It reads `Unit::all()` now and
was falsified with a planted duplicate label before being believed. ⇒ **A
completeness test that carries its own copy of the set is testing the copy.**
Grep this repository for any other `let xs = [` inside a test whose name contains
`every_`, `all_` or `complete`; that is the shape.

### A second, unrelated unit enum

`pdfcer_core::export::dxf::DxfUnits` — `dxf.rs:156-163` — `Inches`,
`Millimetres`, written to the DXF `$INSUNITS` header as codes 1 and 4
(`dxf.rs:167-172`). Mapped from `Unit` lossily by `DxfUnits::for_unit`
(feet → inches, metres → millimetres). **Keep it separate.**

### Kilometres, miles, yards — ★★★ SHIPPED, and this section is the record of how fast it went stale

**Status: delivered, adopted and in the build.** Requested 08:58 on 2026-09-13,
answered 09:19, pinned and shipping by 10:12 the same morning.

This section read, at 08:00: *"None of the three exists anywhere in either
repository … adding them is an engine change."* Both sentences were true when
measured and **false ninety-one minutes later**. They are kept here, struck,
because the useful part is not the old state — it is the shelf life. ⇒ **A
limitation sentence is a citation with an hours-long shelf life.** Anything in
this file of the form *"X does not exist"* is a dated observation about a moving
crate, and must be re-measured before it is quoted, not inherited.

What actually landed (`reply_G013`, adopted in full):

- `Kilometer`, `Yard` and `Mile`, on the exact integer-ratio factors the request
  proposed — `1 / 2 592` and `1 / 4 561 920` are exact, so a yard and a mile
  round-trip without drift the way `0.9144` and `1609.344` would not.
- `Unit::all()` widened from `[Unit; 6]` to `&'static [Unit]`. **This is the half
  that mattered.** All three of our unit dropdowns went from six entries to nine
  with no edit to any of them; the four call sites took `.iter().copied()` and
  nothing else changed.
- Our `fn units() -> [Unit; 6]` shim in `dialogs/scale.rs` — reported as a
  workaround, which is why it was fixed rather than tolerated — is **deleted**.
  Its cause was removed, so it went with it.
- ★ §5(d)'s proposal to split `abbrev()` into a file-format spelling and a UI
  spelling was **declined with its clause**; see §7.

★ **What is still ours is the whole of steps 1 and 2** — the ~30 hard-`pt`
and hard-`mm` surfaces, and the one conversion table below. Neither ever waited
on the engine and neither waits on anything now.

---

## 3. ★★★ The conversions already disagree — a live defect, found by enumerating

There is **no shared points-to-millimetres constant in this crate.** There are
three independent spellings:

**Spelling A — a private constant, divide form.** Six copies, two names:

| file:line | name | type |
|---|---|---|
| `panels/pages/mod.rs:203` | `PTS_PER_MM` | **f32** |
| `panels/docprops/mod.rs:538` | `PTS_PER_MM` | **f32** — duplication defended in a doc comment at `:534-537` |
| `dialogs/insert_image.rs:138` | `PTS_PER_MM` | f64 |
| `dialogs/new_document.rs:117` | `PT_PER_MM` | f64 |
| `dialogs/page_size.rs:145` | `PT_PER_MM` | f64 |
| `text/page_size.rs:68` | `PT_PER_MM` | f64 |

**Spelling B — an inline closure, multiply form.** Seven identical
re-declarations, never shared: `text/print.rs:600, :646, :694, :768, :824`;
`text/new_document.rs:134, :261`.

**Spelling C — the engine's own.** `units.rs:88`, `Millimeter => 25.4 / 72.0`.

### What actually goes wrong

- **A and B are not the same floating-point operation.** `x / (72.0/25.4)` ≠
  `x * 25.4 / 72.0` in IEEE-754 — the constant `72.0/25.4` is itself inexact, so
  the two differ in the last ulp. Currently masked only because every consumer
  rounds hard.
- **★★★ Rounding is inconsistent for the same quantity.** `.round() as i64`
  (`text/print.rs` ×5, `text/new_document.rs` ×2, `dialogs/page_size.rs:285-286`)
  is half-away-from-zero; `{:.0}` display-rounding (`text/pages.rs:181`,
  `text/panels/docprops.rs:242, :254`) is half-to-**even**. ⇒ **A sheet of
  exactly 210.5 mm renders `210` in the page thumbnail's tooltip and `211` in the
  print dialogue, today.** Two surfaces, one document, two answers.

  ★★★ **CORRECTED 2026-09-13 13:30, and the correction is worth more than the
  claim was.** This illustration was first written with its two ends the other
  way round — tooltip `211`, print `210` — and it reached three project
  documents and a published release note in that form. It was then MEASURED, by
  compiling both expressions against the same input:

  ```text
  210.5 mm authored into points   ->  f64 596.6929133858    f32 596.6929321289
  tooltip  {:.0} on the f32 divide   ->  210.5000000000  ->  "210"
  print    .round() as i64 on f64    ->  210.5000000000  ->  "211"
  ```

  ★★ **And the second error is the instructive one: the working explanation of
  *why* blamed the f32/f64 split.** It is not that. Both paths land on exactly
  `210.5000000000`; the disagreement is the rounding rule alone. The precision
  split is real and is the bullet immediately below this one — on a 14,400 pt
  sheet it costs about three decimal digits — but it contributes nothing to
  this example. ⇒ *When two things differ in two ways, the difference that
  already has a column in the analysis is not automatically the cause.* The
  bullet above named the rounding rule correctly and then illustrated it
  backwards, because the analysis was reasoned and the illustration was never
  run.

  ⇒ **This sharpens the remedy.** One conversion table is necessary and not
  sufficient: the fix must also settle **the rounding rule for an
  operator-facing length**, in one place, with the choice argued in the source.
  Half-away-from-zero is the CAD convention and is already what every
  `.round() as i64` site above does; `{:.0}`'s half-to-even is a Rust formatting
  default that nobody in this project ever chose, and it is the minority.
- **Precision differs by crate width.** The thumbnail and Document Properties
  paths convert in **f32**; everything else in f64. On a 14,400 pt A0-class sheet
  the f32 path loses roughly three decimal digits.
- **A second fractional-inch formatter exists** — `text/new_document.rs:180-197`,
  nearest 1/16 — entirely independent of `FractionMode::Fraction` /
  `Unit::FeetInches`. They will disagree on ties.
- Adjacent: **DPI → scale** is spelled `dpi / 72.0` in three places
  (`ocr/mod.rs:405`, `app/actions/imageexport.rs:427`,
  `dialogs/print/preview.rs:463`) and **pixels-per-metre** as `dpi / 0.0254` at
  `clipboard.rs:435`. A fourth and fifth unshared spelling of the same physical
  constant. These agree with each other; they are listed so the eventual cleanup
  knows where they are.

⇒ **O194 clause 2 — one conversion table — is the remedy, and it is the clause
with the highest return.** Delete all six constants and all seven closures; route
everything through `Unit::baseline_per_point`. ★ **This defect is not a
consequence of the unit work and does not wait for it.**

### ✅ DELIVERED 2026-09-13 — `crates/pdfcer-gui/src/units.rs`

Everything above this line is the diagnosis and is left standing, because the
enumeration is the evidence. What follows is what was actually built, **and two
corrections to the diagnosis that only appeared once it was measured.**

| what | where it went |
|---|---|
| six private constants | **deleted** — all six files call `crate::units` |
| seven `\|pt: f64\| pt * 25.4 / 72.0` closures | **deleted** — `use crate::units::whole_mm_from_points as mm;` |
| the two `f32` conversion paths | **widened to f64** at the call site |
| every `{something_mm:.0}` and the two positional `{:.0}` mm pairs | **`{}` over an `i64`**, rounded half away from zero |
| `dpi / 72.0` ×3 and `dpi / 0.0254` ×1 | `units::scale_from_dpi` / `units::pixels_per_metre` |
| the rule itself | `tools/gates/check-unit-conversion.sh`, self-tested, in `run-all.sh` |

The table's API, in the order a caller usually wants it:

```rust
units::mm_from_points(pt)          units::points_from_mm(mm)
units::inches_from_points(pt)      units::points_from_inches(in)
units::from_points(pt, unit)       units::to_points(v, unit)     // any engine Unit
units::whole(v)                    units::whole_mm_from_points(pt)
units::scale_from_dpi(dpi)         units::pixels_per_metre(dpi)
```

#### ★★★ Correction 1 — "the two differ in the last ulp" was the wrong SHAPE of claim

The bullet above says spellings A and B "differ in the last ulp". Both halves of
that are wrong, and the test written to defend the paragraph is what found it:

- **There are three spellings, not two.** `Unit::baseline_per_point` returns a
  **pre-divided** `25.4 / 72.0` — one constant, folded at compile time — and
  multiplying by it is a third operation, distinct from both A and B. The
  paragraph above had said the engine used the multiply form. It does not.
- **They do not differ in "the last ulp"; they differ on SOME INPUTS.** Swept
  over 10 000 deterministic inputs:

  ```text
    engine  vs closure form    3_003 of 10_000 inputs disagree
    engine  vs divide  form      825 of 10_000 inputs disagree
    closure vs divide  form    3_441 of 10_000 inputs disagree
  ```

  ⚠ **A4's 841.89 pt is bit-identical in all three.** So a program holding all
  three cannot be shown to be inconsistent by checking a page size — the
  disagreement waits for a sheet nobody thought to test. That is the same
  failure mode as the rounding half, which is the half that reached him.

⇒ *A test written to defend a paragraph can be the thing that falsifies it.*
Two separate failures, both in this file's reasoning rather than in the code.

#### ★★ Correction 2 — the escape hatch needed a second class, and running the gate is what found it

`check-unit-conversion.sh` was written expecting one exempt class: **type size**
(§4 below). On its first run against the real tree, **all four remaining hits
were tests** — `canvas/measure/scale.rs:517` (25.4 **metres**, an answer rather
than a factor), `:721` (`in_inches * 0.0254`, a hand-computed oracle) and
`canvas/rulers.rs:1299` (`"25.40 mm"` as an expected string).

Routing any of those through `units.rs` would make the test assert that
`units.rs` equals `units.rs`. So the gate carries **two** markers:

```
NOT A DOCUMENT LENGTH:      this is a type size, not a length
ORACLE, NOT A CONVERSION:   this is a test's expected value
```

⚠ Class 2 does **not** license pinning a *format*. A test asserting `"210"` out
of `{width_mm:.0}` is not an oracle, it is the half-to-even defect written down
as an expectation. Mark the arithmetic; never mark the rounding.

#### ★ One unrelated product defect, found on the way

`text/new_document.rs::inches()` printed **`9 1/1`** for a 719.5 pt sheet
(9.993 in). It truncated to a whole inch and rounded the remainder separately,
and that shape cannot carry. Shipped 2026-08-20 and never seen, because every
named imperial sheet is an exact multiple of a sixteenth — but `sheet_summary`
calls it for **any** size an operator types, so it was reachable the whole time.
Fixed by rounding once in sixteenths and then splitting, with a regression test.

#### What §3 does NOT claim to have finished

- The **type-size** surfaces in §4 are untouched and un-annotated. They do not
  trip the gate, so a marker on them would be decoration; §4 is their
  instrument, and the gate's header states that it is blind to the mirror case
  (a type size routed *correctly* into millimetres).
- Bare `/ 72.0` is a stated, deliberate hole in the gate.
- Clauses 1, 3 and 4 of O194 (the unit control itself, and the abbreviation
  catalogue) are **not** delivered by this work.

---

## 4. ⚠ What must be EXCLUDED, and why the exclusion has to be written down

**Type size is not a document length.** These five surfaces print `" pt"` for the
**typographic** point. It is the same 1/72 inch, but a font size in millimetres
is not a thing any drawing or publishing program offers, and Acrobat does not:

`app/fontband.rs:356` · `panels/properties/text.rs:748` ·
`panels/properties/tool.rs:212` · `panels/properties/markup/textannot.rs:693` ·
`text/forms/mod.rs:583, :1071, :1101, :1122` · `text/textannot.rs:246, :262-265`

⇒ **Annotate them in place as deliberately excluded.** Not because the sweep
would fail without it, but because an un-annotated exclusion is indistinguishable
from an omission: a later reader re-opens the question, re-derives the same
answer, or worse, does not, and offers him a font size in kilometres.

**A judgement call that is NOT settled here:** row 12 — dimension group text
height, arrow size and gap. `text/dimension_groups.rs:488` defends points
explicitly: *"10 pt tall whatever the drawing is scaled at"* — i.e. these are
**paper sizes**, deliberately independent of the drawing's scale, and switching
them to metres would be meaningless. That reasoning is sound and they are
provisionally in the same excluded class as type size. ⚠ **But they sit inside
the dimensioning feature, which is where he was looking when he wrote the
report**, so this is the one exclusion worth putting in front of him rather than
deciding silently.

---

## 5. Line-count ceilings — what cannot be touched without a split first

Hard limit: **1,500 lines** (R2).

| lines | file | headroom | relevance |
|---:|---|---:|---|
| **1500** | `text/markup.rs` | **0 — AT THE LIMIT** | hosts `width_suffix()` (row 11). **Must be split before it can be touched.** |
| 1479 | `app/markupband.rs` | 21 | rows 9, 52 |
| 1459 | `panels/pages/mod.rs` | 41 | row 32, and one of the two f32 conversions |
| 1437 | `text/print.rs` | 63 | **rows 25–29 — five conversion sites in a file with 63 lines of room** |
| 1416 | `canvas/rulers.rs` | 84 | rows 1–3 |

Comfortable and touched: `dialogs/textannot.rs` 1308 · `canvas/guides.rs` 1294 ·
`panels/properties/geometry.rs` 1285 · `text/pages.rs` 1273 ·
`text/panels/properties.rs` 1239 · `dialogs/page_size.rs` 1050 ·
`panels/properties/mod.rs` 1041 · `panels/docprops/mod.rs` 1032 ·
`panels/properties/markup/textannot.rs` 1020 · `panels/properties/text.rs` 963 ·
`panels/dimension_groups/mod.rs` 921 · `canvas/markup/swatch.rs` 856 ·
`text/textannot.rs` 822 · `panels/properties/widgetedit.rs` 806 ·
`text/tool.rs` 804 · `text/page_size.rs` 711 · `dialogs/print/tabs.rs` 694 ·
`dialogs/insert_image.rs` 692 · `text/images.rs` 626 · `dialogs/scale.rs` 585 ·
`text/dimension_groups.rs` 564 · `panels/properties/markup/rows.rs` 550 ·
`panels/properties/dimension/overrides.rs` 540 · `dialogs/new_document.rs` 538 ·
`app/fontband.rs` 516 · `text/new_document.rs` 500 ·
`panels/dimension_groups/style.rs` 461 · `app/toolstatus.rs` 459 ·
`panels/properties/tool.rs` 451 · `text/scale.rs` **427 — the natural home for
new unit names, ~1,070 lines of headroom** · `text/export_dxf.rs` 425 ·
`text/panels/docprops.rs` 422 · `dialogs/import_text.rs` 405 ·
`panels/properties/dimension/tolerance.rs` 373 · `canvas/grid.rs` 230 ·
`text/import_text.rs` 209.

---

## 6. `ui_text` coverage, and the hole the gate cannot see

`crate::text::*` **is** the `ui_text` catalog (`PROJECT_PLAN.md:165`). The gate
requires every user-visible string to live there.

### Already catalogued

`unit_name(Unit)` `text/scale.rs:132-141` · `basis_label` / `unit_label` /
`fraction_label` `:94, :100, :106` · `fraction_name` `:159-181` ·
`points_suffix` `text/dimension_groups.rs:490-492` · `points_suffix`
`text/import_text.rs:110-112` · `millimetres` `text/images.rs:193-195` ·
`markup_width_suffix` `text/panels/properties/markup.rs:83-85` · `width_suffix`
`text/markup.rs:355-357` · `text_size_suffix` `text/panels/properties.rs:639-640`
· `text_pen_size_suffix` `text/tool.rs:636-638` · `stamp_text_size_suffix`
`text/panels/textannotstyle.rs:234-236` · `dpi_suffix` `text/print.rs:909-911` ·
`units_heading` / `units_name(DxfUnits)` `text/export_dxf.rs:141, :154-166` ·
`geometry_units_note` `text/panels/properties.rs:415-417`.

★ **Six separate `" pt"` const fns under five different names.** That is catalog
debt in its own right, and it is the mechanism by which a unit sweep misses one.

### ★★★ The hole: a unit welded into a `format!` template

These are unit strings baked into templates rather than nameable entries. They
live **inside** `text/`, so the string gate passes them — the gate checks *where*
a string lives, and **cannot see a unit fused into one**:

`text/panels/properties.rs:272, :295, :306` · `text/page_size.rs:154, :226, :232,
:239, :249-250, :295, :298, :301, :304` · `text/new_document.rs:132, :135, :229,
:235` · `text/print.rs:601, :648, :696, :770, ~:826` · `text/pages.rs:181` ·
`text/panels/docprops.rs:242, :254` · `text/images.rs:121` ·
`text/textannot.rs:265` · `text/forms/mod.rs:583, :1071, :1101, :1122`.

⚠ **Two of these are worse than the rest**, because the unit is in the *label*
rather than a suffix: `"Width (mm)"` / `"Height (mm)"` at
`text/page_size.rs:226, :232` and `text/new_document.rs:229, :235`. Switching the
unit there means **rewriting the label**, not the suffix — a different edit, and
the one a mechanical sweep will skip.

### Missing entirely

★ **Half of this is fixed.** "Kilometres", "Miles" and "Yards" are catalogued
(`text/scale.rs::unit_name`) and `km`, `yd`, `mi` are catalogued
(`text/dimension_groups.rs::unit_abbrev`), both added when `G013` landed.

What is still missing: `mm` / `cm` / `m` / `in` / `ft` **as standalone nameable
entries**. They reach the operator through the engine's `Unit::abbrev`, which is
outside the gate's reach and outside a translator's reach. ⇒ The nine-unit set
is now catalogued **inconsistently** — three units have a local abbreviation
entry and six do not — which is worse than the uniform gap it replaced, because
it looks finished. Fixing it means routing every abbreviation through
`unit_abbrev` rather than adding six more arms somewhere else.

### ★★★ An argued refusal that this work contradicts

`text/new_document.rs:21-35` is an **explicit, documented decision not to offer a
unit toggle** in the new-document dialogue. It is not an oversight; someone
thought about it and wrote down why.

⇒ **It must be answered, not overwritten.** Either the reasoning still holds and
that dialogue is a documented exception in §4's class, or O194 supersedes it and
the note is rewritten to say so and to say what changed. ★ **Silently deleting an
argued refusal is how the same question gets re-litigated in six months by
someone who cannot find out it was already settled.**

---

## 7. ⚠ Machine-facing lengths — do NOT unit-switch these

Every entry below writes a number a **machine** reads. A unit suffix, or a
different unit, breaks a parser.

| site | file:line | what it writes | why it must not move |
|---|---|---|---|
| Guides sidecar | `canvas/guides.rs:457, :598-608` | field-delimited `page/axis/at` | a parsed round-trip format; a suffix breaks reload |
| Diagnostic rects | `diag.rs:304, :470, :529, :647` | `pdfcer-diag ui-rect … rect=[[x0 y0] - [x1 y1]]` | ★ **the driven-test harness parses these.** Changing them breaks every check in the repository |
| Prefs writer | `app/prefs/mod.rs:30`, `file.rs:61-65`, `offpage.rs:177-236`, `printing.rs:586` | flat `key = value` | a stored numeric is a stored numeric |
| PDF content streams | `redact/proof.rs:737, :794, :928, :932, :1092, :1096, :1131` | stream bodies | PDF user-space units, ISO 32000-2 §8.3.2.3 — never labelled |
| ⚠ PDF `/Measure` `/U` | via core `Unit::abbrev` (`units.rs:127-137`) | the `/U` string, number format dictionary | **it is BOTH, and the split was declined — see the note under this table** |
| DXF `$INSUNITS` | `pdfcer-core/src/export/dxf.rs:167-172` | `1` / `4` | a header integer |
| Clipboard DIB | `clipboard.rs:253, :427-435` | pixels-per-metre `u32` | Win32 `BITMAPINFOHEADER` |
| Raster / OCR scale | `ocr/mod.rs:405`, `app/actions/imageexport.rs:427`, `dialogs/print/preview.rs:463` | `dpi / 72.0` | passed to the renderer, never displayed |
| Test assertion messages | ~40 sites incl. `app/status/fitting.rs:374-464`, `canvas/rulers.rs:1243-1247`, `render/strategy.rs:1025-1235` | `"… {} pt …"` inside `assert!` | ★ **developer-facing, and these are egui UI points — layout units, not document lengths.** They are the bulk of the `pt` grep noise; **exclude the whole class** |

★ **The last row is the one that will waste a day.** A naive grep for `pt` in
this crate is dominated by assertion messages measuring widget geometry, which
have nothing to do with the operator's report and are not in the same unit
system.

### ★★★ The `/U` row — the decline, and three corrections to this file's own citation

This file asked (request `G013` §5(d)) for `Unit::abbrev` to be split into a
file-format spelling and a UI spelling, on the reasoning that a string written
into the operator's drawing should not be the same string a dropdown widens.
**Declined**, and the clause settles it: ISO 32000 defines `/U` as *"a text
string specifying a label for displaying the units represented by this dictionary
**in a user interface**"*. One role, two consumers. A split would have
manufactured exactly the drift it was meant to prevent, and it is
**undetectable** drift — `/U` carries no arithmetic, so a wrong label is a
correct number that no reader can catch.

Three corrections, all to citations this file made:

1. ⚠ **The table number is crossed between editions.** The row above cited
   "§12.9 Table 263" for **2.0**. Table 263 is the **1.7** number; 2.0's number
   format dictionary is **Table 268**. The clause number (§12.9) is stable, so
   **cite the clause, not the table**, which is what the row now does.
2. ⚠ **`/PDU` is a trap, and it is the reason this request was filed at all.**
   §12.10.2 Table 269 defines `/PDU`, which *does* enumerate names including
   `KM` and `MI`. Anyone sweeping the measurement clauses for "kilometre" finds
   that table and concludes `/U` is an enumeration. It is a different key, a
   different dictionary, a different subtype, a different clause and a different
   PDF **type** — `/PDU` takes names, `/U` takes a text string.
3. **`/U` is a text string and is therefore encrypted** in an encrypted document.
   It cannot be optimised into a name even if someone wanted to.

---

## 8. How to use this file

1. **Before claiming O194 is done**, walk §1c, §1d and §1e and mark each row.
2. **When adding a surface that shows a length**, add its row here in the same
   commit. The list is only worth having if it stays true.
3. **Re-measure the line counts in §5 before touching anything** — `text/markup.rs`
   was at exactly the ceiling on the day this was written, and files grow.
4. §3 and §6's argued-refusal note are **defects in their own right**. They were
   found by enumerating, they are not gated on the unit work, and they should not
   be closed as part of it.
