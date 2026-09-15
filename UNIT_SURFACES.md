# Every surface that shows or accepts a length

The register of every place in pdfcer-gui that prints or takes a length, what
unit choice each one offers, and the rules that govern unit handling across the
program. Read it before adding a surface that shows a length, and before
claiming a unit change is complete.

Every row concerns **ce dimensions** — the lengths pdfcer authors and displays.
The unit a **pdf dimension** was exported with is CAD-exported page content and
is out of scope: a unit choice that re-authors what the CAD system drew is a
defect, not a feature.

Paths are relative to `crates/pdfcer-gui/src/`, and cite a **symbol** rather
than a line, because a line number in a moving tree is a citation that decays
between the writing and the reading. Locate one with:

```sh
git grep -n '<symbol>' -- crates/pdfcer-gui/src
```

---

## 1. The table

Each row: what the operator would call it, where it lives, and what unit choice
is offered.

### 1a. Follows the operator's chosen unit — 3

These are correct and are the model the rest should reach. All three route
through `pdfcer_core::dimension::format_measurement`.

| # | Surface | Where | Unit behaviour |
|---|---|---|---|
| 1 | Ruler tick labels | `canvas/rulers.rs` — `Scale::label` | follows `DEFAULT_GROUP_ID`'s unit; falls back to raw `pt` under `ScaleState::NeverSet` |
| 35 | Measure tool, live perimeter total | `app/toolstatus.rs` — `perimeter_stage` | follows the pick's group |
| 36 | Measure tool, live radius / diameter | `app/toolstatus.rs` — `circular_stage` | follows the pick's group |

Two standing caveats on the ruler, neither a unit defect:

- `Scale::default` — under `NeverSet` the ruler renders ` pt` and ignores the
  unit field entirely. There is no `Unit::Point` in the engine, so this is the
  absence of a scale, not a missing dropdown entry.
- `Scale::of` reads the **default** group, never the *active* one. That is an
  open behaviour question, documented in place, and must not be folded into a
  unit change.

### 1b. Offers a unit menu — 5 controls

Every one iterates `Unit::all()`, so a unit the engine gains appears in all
five without an edit. Names come from `text/scale.rs::unit_name`, which
`text/panels/dimension.rs::unit_name` delegates to.

| # | Surface | Where | Menu contents |
|---|---|---|---|
| 53a | Set Scale — *Paper measured in* | `dialogs/scale.rs` — `unit_combo`, id `scale.basis` | `Unit::all()` |
| 53b | Set Scale — *Show dimensions in* | `dialogs/scale.rs` — `unit_combo`, id `scale.unit` | `Unit::all()` |
| 55a | Dimension Groups — selected group's unit | `panels/dimension_groups/mod.rs`, combo `dimension-group-unit` | `Unit::all()` |
| 55b | Dimension Groups — new group's starting unit | `panels/dimension_groups/mod.rs`, combo `dimension-groups-new-unit` | `Unit::all()`, default `Millimeter` |
| 37 | Per-dimension unit override | `panels/properties/dimension/overrides.rs` — the `unit` row | `Unit::all()` |

### 1c. Hard points, no control — 19

Every row prints or takes a document length in points with no unit control on
the surface. Five of them are deliberate exclusions — see §4.

| # | Surface | Where | What it is |
|---|---|---|---|
| 4 | Properties — typed X / Y / W / H | `panels/properties/geometry.rs` — `field` | four `DragValue`s, `fixed_decimals(2)`, no suffix at all |
| 5 | The note under *Position and size* | `text/panels/properties.rs` — `geometry_units_note` | *"Points, measured to the bottom-left corner. Y increases upward."* |
| 6 | Properties — position readout | `text/panels/properties.rs` — `value_position` | `"{x:.1}, {y:.1} pt"` |
| 7 | Properties — size readout | `text/panels/properties.rs` — `value_size` | `"{width:.1} × {height:.1} pt"` |
| 8 | Properties — line width | `text/panels/properties.rs` — `value_line_width` | `"{width:.2} pt"` |
| 9 | Markup line-weight spinner (ribbon) | `app/markupband.rs` — `width`; `text/panels/properties/markup.rs` — `markup_width_suffix` | fixed `" pt"` |
| 10 | Markup line-weight spinner (properties) | `panels/properties/markup/rows.rs` — `width_row` | fixed `" pt"` |
| 11 | Pen width on the markup swatch | `canvas/markup/swatch.rs` — `show`; `text/markup.rs` — `width_suffix` | fixed `" pt"` |
| 12 | Dimension group text height / arrow size / gap | `panels/dimension_groups/style.rs` — `show`; `text/dimension_groups.rs` — `points_suffix` | fixed `" pt"`, inside the dimensioning feature itself. See §4 — provisionally excluded, and the one exclusion worth raising with the operator |
| 13 | Import-text margin and size | `dialogs/import_text.rs` — `body`; `text/import_text.rs` — `points_suffix` | fixed `" pt"` |
| 20 | Page-size off-sheet warning | `text/page_size.rs` — `overhang` | `"{right:.0} pt past the right edge"` and three siblings |
| 38 | Tolerance ± magnitude | `panels/properties/dimension/tolerance.rs` — `show` | no suffix, deliberately; `tolerance_unit_note` names the unit once below the fields |
| 39 | Stamp size chooser | `text/textannot.rs` — `stamp_size_label`; `dialogs/textannot.rs` | `StampSize::Points(pt)` renders `"{pt} pt"` |
| 45 | Widget border-width spinner | `panels/properties/widgetedit.rs` — `border_rows` | no suffix, no unit, no note — a bare `range(0.0..=72.0)`. The only surface that shows a length with no unit stated anywhere |
| 40 | Stamp text size | `panels/properties/markup/textannot.rs` — `size_row`; `text/panels/textannotstyle.rs` — `stamp_text_size_suffix` | fixed `" pt"` — excluded, §4 |
| 41 | Font size on the ribbon | `app/fontband.rs` — `face`; `text/panels/properties.rs` — `text_size_suffix` | fixed `" pt"` — excluded, §4 |
| 42 | Font size in properties | `panels/properties/text.rs` — `section` | fixed `" pt"` — excluded, §4 |
| 43 | Text-pen size | `panels/properties/tool.rs` — `block_for`; `text/tool.rs` — `text_pen_size_suffix` | fixed `" pt"` — excluded, §4 |
| 44 | Form-field font size | `text/forms/mod.rs` — `form_field_rich_text_summary` and the auto-size notes | `"{sz} pt"` — excluded, §4 |

### 1d. Hard millimetres, no control — 13

Equally a gap, and easier to miss because millimetres *look* like a real
answer. An operator working in feet gets no more choice here than in §1c.

| # | Surface | Where | What it is |
|---|---|---|---|
| 14 | Insert-image X / Y / W / H | `dialogs/insert_image.rs` — `spinner`; `text/images.rs` — `millimetres` | fixed `" mm"` |
| 15 | Image's declared size readout | `text/images.rs` — `natural_size` | `"{w} × {h} mm at the {x:.0} dpi…"` |
| 16 | Page-size Width / Height | `text/page_size.rs` — `custom_width`, `custom_height` | `"Width (mm)"`, `"Height (mm)"` — the unit is welded into the label, not a suffix |
| 17 | Page-size sheet summary | `text/page_size.rs` — `sheet_summary` | pt and mm, both fixed |
| 18 | Page-size preset list | `text/page_size.rs` — `size_entry` | `"{name} — {w} × {h} mm"` |
| 19 | Page-size custom-range refusal | `text/page_size.rs` — `custom_refused` | mm only |
| 21 | New-document Width / Height | `text/new_document.rs` — `custom_width`, `custom_height` | unit welded into the label; the module header is an explicit documented refusal to add a unit toggle — see §6 |
| 22 | New-document preset list | `text/new_document.rs` — `size_entry` | mixed and implicit: ISO entries in mm, US/ANSI entries in inches, chosen per entry by the code |
| 23 | New-document inch fraction | `text/new_document.rs` — `inches` | a second fractional-inch formatter, nearest 1/16, independent of the engine's `FractionMode` / `Unit::FeetInches`. Its arithmetic routes through `crate::units`; its **fraction rule** does not, and the two will disagree on ties |
| 24 | New-document sheet summary | `text/new_document.rs` — `sheet_summary` | mm, in and pt, all three, fixed |
| 32 | Page thumbnail tooltip | `text/pages.rs` — `page_tile_tooltip` | mm only |
| 33 | Document Properties — page size | `text/panels/docprops.rs` — `page_size` | mm only |
| 34 | Document Properties — mixed-size row | `text/panels/docprops.rs` — `page_size_mixed` | mm only |

### 1e. Print — 5 surfaces, millimetres only

All in `text/print.rs`. Each converts through `units::whole_mm_from_points`; what
none of them offers is a choice of unit.

| # | Surface | Where |
|---|---|---|
| 25 | Paper list | `paper_form` |
| 26 | Paper-match note | `paper_auto_matched` |
| 27 | Oversize note | `paper_auto_too_big` |
| 28 | Planned-sheet note | `paper_is_a_request` |
| 29 | Paper disclosure | `sheet_from_driver` — pt and mm |

### 1f. No readout at all — 2

Not unit defects. Recorded so a future reader does not "discover" them.

| # | Surface | Where | Why it is here |
|---|---|---|---|
| 46 | Guides | `canvas/guides.rs` (module) | a guide's position is never shown to the operator, so there is no readout to put a unit on. A missing feature, not a missing unit |
| 47 | Grid | `canvas/grid.rs` — `draw`, `lines` | spacing is derived from the ruler ladder; there is no spacing entry to unit-switch |

### 1g. Adjacent, deliberately out of scope — 7

| # | Surface | Where | Why out of scope |
|---|---|---|---|
| 30 | Print scaling | `text/print.rs`; `dialogs/print/tabs.rs` | a percentage, not a length |
| 31 | Print resolution | `dialogs/print/tabs.rs`; `text/print.rs` — `dpi_suffix` | a DPI — inches are implied by the unit itself, and it is not switchable in any print dialogue |
| 48 | DXF export *Units* control | `text/export_dxf.rs` — `units_heading`, `units_name`; enum in `pdfcer-core/src/export/dxf.rs` | a file-format field (`$INSUNITS`), separate vocabulary, two entries. See §2 |
| 49 | DXF scale field | `text/export_dxf.rs` | a unitless ratio |
| 50 | Find panel position | `text/find.rs` — `position` | `n of N`, not a coordinate |
| 51 | Parallel-tolerance slider | `dialogs/settings/measuring.rs` — `degree_suffix` | an angle |
| 52 | Markup opacity | `app/markupband.rs` — `opacity`; `canvas/markup/swatch.rs` | a percentage |

---

## 2. The unit model

### `pdfcer_core::dimension::Unit`

Nine variants, and **not** `#[non_exhaustive]` — that is load-bearing, see
below.

| Variant | `abbrev()` | `baseline_per_point()` | `default_format()` |
|---|---|---|---|
| `Millimeter` | `mm` | 25.4 / 72 | 2 dp |
| `Centimeter` | `cm` | 2.54 / 72 | 2 dp |
| `Meter` | `m` | 0.0254 / 72 | 3 dp |
| `Kilometer` | `km` | 0.000 025 4 / 72 | 4 dp |
| `Inch` | `in` | 1 / 72 | 2 dp |
| `DecimalFeet` | `ft` | 1 / 864 | 2 dp |
| `FeetInches` | `ft` | 1 / 864 | nearest 1/8 |
| `Yard` | `yd` | 1 / 2 592 | 3 dp |
| `Mile` | `mi` | 1 / 4 561 920 | 4 dp |

The factors are written as exact integer ratios rather than decimals, so a yard
and a mile round-trip without drift where `0.9144` and `1609.344` would not.

Decimal places follow one rule: **places rise as the unit coarsens**, so the
resolution on the page stays in a usable band instead of tracking the unit name.
A yard takes three rather than the two its magnitude suggests, because a metre
already takes three and giving a yard feet's two would resolve to 9 mm — the
coarsest number in the table, beside the finest. Kilometres and miles take four
for the same reason at the other end.

`Unit::all()` is `pub const fn all() -> &'static [Unit]`, ordered metric
ascending then imperial ascending, with `ft-in` beside `ft` because it is a
presentation of feet rather than a step in magnitude. It returns a **slice, not
an array**: an accessor whose type encodes the cardinality of a growing set
makes every addition an API break for every consumer, including ones that only
iterate. Nothing persists an index into it — `Unit::token` is the stable
serialisation — so insertions are safe.

Supporting types in the same module: `FractionMode`, `NumberFormat`,
`DecimalMarker`, `ScaleState`.

**There is no `Unit::Point`.** The only `Point` in the engine's `units.rs` is
`DecimalMarker::Point`, the decimal separator:

```sh
git grep -n 'Point' -- crates/pdfcer-core/src/dimension/units.rs   # in D:/Dev/pdfcer
```

Every ` pt` in §1 is therefore the **absence** of a unit, not a unit the
operator selected.

### `Unit` must stay exhaustive, and the compiler is the instrument

Two matches in this crate are exhaustive on purpose — `text/scale.rs::unit_name`
and `text/dimension_groups.rs::unit_abbrev`. When the engine gains a unit, both
refuse to compile until the arm exists. A `_ =>` arm would let a new unit reach
the operator with no English name and **no gate would see it**. Never add one.

A second instrument backs this: `tools/gates/check-completeness-tests.py` fails
any test whose name begins `every_` / `all_` / `each_` or contains `_complete` /
`_exhaustive` and which carries a literal array of three or more `Type::Variant`
paths sharing one type. **A completeness test that carries its own copy of the
set is testing the copy**, and is invisible for exactly as long as the set is
stable — which is exactly as long as nobody needs it.
`text/scale.rs::every_unit_is_named_distinctly` reads `Unit::all()`, the same
list the five dropdowns read, so the test and the product cannot disagree about
what the set is.

### A second, unrelated unit enum

`pdfcer_core::export::dxf::DxfUnits` — `Inches`, `Millimetres` — written to the
DXF `$INSUNITS` header as codes 1 and 4. Mapped from `Unit` lossily by
`DxfUnits::for_unit` (feet to inches, metres to millimetres). **Keep it
separate** from `Unit`; it is a file-format field, not an operator preference.

---

## 3. The one conversion table

`crates/pdfcer-gui/src/units.rs` is the only place in this program that converts
between PDF points and a length an operator reads or types. Nowhere else
declares a points-per-millimetre constant, a `25.4 / 72.0` closure, or a
`/ 72.0` DPI spelling.

```rust
units::mm_from_points(pt)          units::points_from_mm(mm)
units::inches_from_points(pt)      units::points_from_inches(inches)
units::from_points(pt, unit)       units::to_points(v, unit)     // any engine Unit
units::whole(v)                    units::whole_mm_from_points(pt)
units::scale_from_dpi(dpi)         units::pixels_per_metre(dpi)
```

**Contract.** Every function takes and returns `f64`; callers holding `f32`
convert in, not out. Conversion is a single multiply by
`Unit::baseline_per_point`, so two call sites given the same input produce
bit-identical results. No function here formats — they return numbers, and the
caller writes `{}` for a whole number or an explicit `{:.N}` for a decimal.

### Why one table, and not several that agree

The three plausible spellings of points-to-millimetres are **not the same
arithmetic**, and the difference is not a last-ulp curiosity:

- `x / (72.0 / 25.4)` — the divide form. `72.0 / 25.4` is itself inexact.
- `x * 25.4 / 72.0` — the multiply form.
- `x * Unit::baseline_per_point(Millimeter)` — one constant, divided once and
  folded at compile time. This is what `units.rs` ships.

The three are swept against each other, rather than sampled, and each pair
disagrees on thousands of the inputs. Re-derive the counts rather than quoting
them:

```sh
cargo test -p pdfcer-gui the_conversion_is_the_engines_own_constant_not_a_transcription_of_it
```

A4's 841.89 pt is bit-identical in all three — so a program holding several
spellings **cannot be shown to be inconsistent by checking a page size**. The
disagreement waits for a sheet nobody thought to test.

### The rounding rule

A whole-number length shown to the operator rounds **half away from zero**:
`2.5` to `3`, `-2.5` to `-3`. That is `units::whole`, and it is the only function
in this program permitted to turn a length into a whole number for display.

Rust's `{:.0}` rounds **half to even**, which is the IEEE-754 default and the
right rule for summing long columns — and the wrong rule for a single sheet
size, because a CAD operator expects the schoolroom rule and because it is not
what the rest of the program does. A sheet authored at exactly 210.5 mm converts
to 596.6929133858 pt and back to exactly 210.5000000000 mm; under `{:.0}` that
renders `210` and under `.round() as i64` it renders `211`. One document, one
sheet, two surfaces an operator can have open at once.

**A length must not reach a format string as a bare `{:.0}`.**

### The gate

`tools/gates/check-unit-conversion.sh`, self-tested, in `run-all.sh`. It carries
exactly two escape markers, and each names the class it excuses:

```
NOT A DOCUMENT LENGTH:      this is a type size, not a length
ORACLE, NOT A CONVERSION:   this is a test's expected value
```

The oracle marker does **not** license pinning a *format*. A test asserting
`"210"` out of a `{:.0}` on millimetres is not an oracle, it is the half-to-even
defect written down as an expectation. Mark the arithmetic; never mark the
rounding.

Two deliberate blind spots, stated so nobody meets them as a surprise:

- **Bare `/ 72.0` is not flagged.** The points-per-inch divide appears in render
  scaling, DPI arithmetic and coordinate transforms, most of which are not
  lengths an operator reads; flagging them all would return dozens of hits on a
  clean tree and teach people to write exemptions. `scale_from_dpi` and
  `pixels_per_metre` exist for the DPI cases.
- **The mirror of the type-size exclusion.** The gate fails a type size that
  converts itself by hand; it cannot fail a type size routed *correctly* through
  `units.rs` into millimetres. `units::mm_from_points(font_size_pt)` is a clean
  line by every pattern in the file, and it is precisely the mistake §4 exists to
  prevent. The instrument for that one is §4 and a reader.

### The one rule that is not in the table

Row 23's fractional-inch formatter in `text/new_document.rs` is a second
fraction rule, nearest 1/16, independent of `FractionMode::Fraction` and
`Unit::FeetInches`. Its arithmetic routes through `crate::units`; its rounding
does not, so the two disagree on ties. It is a separate item from the unit work
and must not be closed as part of it.

`clipboard.rs::pixels_per_metre` wraps `units::pixels_per_metre` rather than
duplicating it. What stays local is the DIB's business: the guard against a
nonsense DPI, the zero returned instead, and the round-to-nearest that keeps a
300 DPI copy at 11,811 rather than 11,810 pixels per metre.

---

## 4. What is excluded, and why the exclusion is written down

**Type size is not a document length.** These surfaces print `" pt"` for the
**typographic** point. It is arithmetically the same 1/72 inch, but a font size
in millimetres is not a thing any drawing or publishing program offers, and
Acrobat does not:

`app/fontband.rs` · `panels/properties/text.rs` · `panels/properties/tool.rs` ·
`panels/properties/markup/textannot.rs` · `text/forms/mod.rs` ·
`text/textannot.rs`

Each carries an in-source note pointing at `crate::units`. **Annotate a new one
in place** — an un-annotated exclusion is indistinguishable from an omission, so
a later reader re-opens the question, re-derives the same answer, or worse does
not, and offers a font size in kilometres.

**Not settled: row 12** — dimension group text height, arrow size and gap.
`text/dimension_groups.rs` defends points explicitly: these are *paper* sizes,
10 pt tall whatever the drawing is scaled at, deliberately independent of the
drawing's scale, so switching them to metres would be meaningless. That
reasoning is sound and they sit provisionally in the same excluded class as type
size. They also sit **inside the dimensioning feature**, which is where the unit
complaint came from, so this is the one exclusion to put in front of the operator
rather than decide silently.

---

## 5. File-size ceilings

Hard limit **1,500 lines** (R2), enforced by `check-file-size.sh`. A file at the
limit must be split before a row hosted in it can be touched, so measure before
planning an edit:

```sh
find crates/pdfcer-gui/src -name '*.rs' | xargs wc -l | sort -rn | head -20
```

`text/markup.rs` hosts `width_suffix` (row 11) and sits **at** the
limit. `text/scale.rs` is the natural home for new unit names and has room.

---

## 6. `ui_text` coverage, and the hole the gate cannot see

`crate::text::*` is the `ui_text` catalog. The string gate requires every
user-visible string to live there.

### Catalogued

`unit_name` and `fraction_name` (`text/scale.rs`) · `unit_abbrev` and
`points_suffix` (`text/dimension_groups.rs`) · `points_suffix`
(`text/import_text.rs`) · `millimetres` (`text/images.rs`) ·
`markup_width_suffix` (`text/panels/properties/markup.rs`) · `width_suffix`
(`text/markup.rs`) · `text_size_suffix` and `geometry_units_note`
(`text/panels/properties.rs`) · `text_pen_size_suffix` (`text/tool.rs`) ·
`stamp_text_size_suffix` (`text/panels/textannotstyle.rs`) · `dpi_suffix`
(`text/print.rs`) · `units_heading` and `units_name` (`text/export_dxf.rs`).

`unit_name` and `unit_abbrev` are each exhaustive over all nine units, so the
English names and the short tags are both complete and uniform.

Six separate `" pt"` const fns under five different names remain. That is catalog
debt in its own right, and it is the mechanism by which a unit sweep misses one.

### The hole: a unit welded into a `format!` template

These are unit strings baked into templates rather than nameable entries. They
live **inside** `text/`, so the string gate passes them — it checks *where* a
string lives and cannot see a unit fused into one:

`text/panels/properties.rs` — `value_position`, `value_line_width`, `value_size`
· `text/page_size.rs` — `size_entry`, `custom_width`, `custom_height`,
`sheet_summary`, `custom_refused`, `overhang` · `text/new_document.rs` —
`size_entry`, `custom_width`, `custom_height`, `sheet_summary` ·
`text/print.rs` — `paper_form`, `paper_auto_matched`, `paper_auto_too_big`,
`paper_is_a_request`, `sheet_from_driver` · `text/pages.rs` —
`page_tile_tooltip` · `text/panels/docprops.rs` — `page_size`,
`page_size_mixed` · `text/images.rs` — `natural_size` · `text/textannot.rs` —
`stamp_size_label` · `text/forms/mod.rs` — `form_field_rich_text_summary` and
the auto-size notes.

Four are worse than the rest, because the unit is in the **label** rather than a
suffix: `custom_width` and `custom_height` in both `text/page_size.rs` and
`text/new_document.rs`. Switching the unit there means rewriting the label, not
the suffix — a different edit, and the one a mechanical sweep skips.

### An argued refusal that a unit sweep would contradict

`text/new_document.rs`'s module header is an explicit, documented decision **not**
to offer a unit toggle in the new-document dialogue. Its reasoning: the A series
is defined in millimetres and A1 in inches is 23.39 × 33.11, a number nobody
recognises; the US and ANSI entries sit in the same list and would want inches,
so a toggle would be right for four entries of sixteen and wrong for twelve; and
this shell's units answer is per-dimension-group, a per-document drafting
convention rather than an application preference, so a switch here would be a
second, unrelated units concept in a dialogue that is open for four seconds. The
custom fields state millimetres in the label rather than a suffix that can be
missed, and `sheet_summary` echoes the result in both units.

**Answer it, do not overwrite it.** Either that reasoning still holds and the
dialogue is a documented exception in §4's class, or a decision supersedes it and
the note is rewritten to say so and to say what changed. Silently deleting an
argued refusal is how the same question gets re-litigated by someone who cannot
find out it was already settled.

---

## 7. Machine-facing lengths — do not unit-switch these

Every entry writes a number a **machine** reads. A unit suffix, or a different
unit, breaks a parser.

| Site | Where | What it writes | Why it must not move |
|---|---|---|---|
| Guides sidecar | `canvas/guides.rs` | field-delimited `page`, `axis`, `at` | a parsed round-trip format; a suffix breaks reload |
| Diagnostic rects | `diag.rs` | `pdfcer-diag ui-rect name=… rect=…` | the driven-test harness parses these; changing them breaks every check in the repository |
| Prefs writer | `app/prefs/` | flat `key = value` | a stored numeric is a stored numeric |
| PDF content streams | `redact/proof.rs` | stream bodies | PDF user-space units, ISO 32000-2 §8.3.2.3 — never labelled |
| PDF `/Measure` `/U` | engine `Unit::abbrev` | the `/U` string, number format dictionary | it is **both** a file field and a UI label — see below |
| DXF `$INSUNITS` | `pdfcer-core/src/export/dxf.rs` | `1` or `4` | a header integer |
| Clipboard DIB | `clipboard.rs` — `dib_v5`, `pixels_per_metre` | pixels-per-metre `u32` | Win32 `BITMAPINFOHEADER` |
| Raster / OCR scale | `ocr/mod.rs`, `app/actions/imageexport.rs`, `dialogs/print/preview.rs` | `units::scale_from_dpi` | passed to the renderer, never displayed |
| Test assertion messages | `app/status/fitting.rs`, `canvas/rulers.rs`, `render/strategy.rs` and others | `"… {} pt …"` inside `assert!` | developer-facing, and these are **egui UI points** — layout units, not document lengths. Exclude the whole class |

The last row is the one that wastes a day: a naive grep for `pt` in this crate is
dominated by assertion messages measuring widget geometry, which have nothing to
do with document lengths and are not in the same unit system.

### Why `/U` is not split into a file spelling and a UI spelling

ISO 32000 §12.9 defines `/U` as *a text string specifying a label for displaying
the units represented by this dictionary in a user interface*. One role, two
consumers. Splitting `Unit::abbrev` into a file-format spelling and a UI spelling
would manufacture exactly the drift it was meant to prevent, and it would be
**undetectable** drift: `/U` carries no arithmetic, so a wrong label is a correct
number no reader can catch.

Two traps around it:

- **Cite the clause, not the table.** The number format dictionary is Table 263
  in PDF 1.7 and Table 268 in 2.0. The clause number, §12.9, is stable across
  both.
- **`/PDU` is a different key.** §12.10.2 defines `/PDU`, which *does* enumerate
  names including `KM` and `MI`. Anyone sweeping the measurement clauses for
  "kilometre" finds that table and concludes `/U` is an enumeration. It is a
  different key, dictionary, subtype, clause and PDF **type** — `/PDU` takes
  names, `/U` takes a text string.

Being a text string, `/U` **is encrypted** in an encrypted document (§7.9.2.2).
It cannot be optimised into a name.

---

## 8. How to use this file

1. **Before claiming a unit change is complete**, walk §1c, §1d and §1e and mark
   each row. "Units are everywhere now" is a claim; a green row is a measurement.
2. **When adding a surface that shows a length, add its row here in the same
   commit.** The list is only worth having if it stays true, and ordinary work —
   a new dialogue with a `DragValue` and a `" pt"` suffix — adds rows without
   anyone deciding to add a unit defect.
3. **Re-measure §5 before planning an edit.** Files grow, and a file at the
   ceiling must be split first.
4. **§6's argued refusal and row 23's second fraction rule are separate items.**
   Neither is gated on the unit work, and neither is closed as part of it.
