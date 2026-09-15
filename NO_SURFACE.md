# Shipped behaviour with a hard-coded value and no control

Every value in `crates/pdfcer-gui/src/` that an operator would plausibly want
to change and cannot — a compiled-in constant, a `Default` impl, or a field
only settable from code — with the `file:line` that defines it and the verdict
on whether it is owed a surface.

## How to read a row

A row says a value is hard-coded. It does **not** say the value should become a
control, and it does not say the value is correct. Follow the value to the code
that consumes it before acting: a row can need *build the surface*, *must not
exist*, or *the value is wrong and no control fixes it*.

`none` in the verdict column means unreached and undecided. A named verdict is
a decision already argued at the definition site; the argument is summarised
under the table it belongs to.

**This sweep finds constants a control could reach, and is structurally blind
to a behaviour the engine cannot be told about.** A value absent from the
vocabulary this shell and `pdfcer-core` share is not hard-coded anywhere in
this crate, so no grep finds it — and it reaches the operator identically: they
press nothing, and the mark comes out the way it always does. That second kind
is found by reading the engine's input structs (`MarkupSpec`, `MarkupOptions`,
`MarkupStyle`, `RedactSpec`, `RenderOptions`) against what an operator would
want, which is a different sweep from this one.

**When a constant is left in place of a control, write the test that fails when
the control arrives.** A doc comment naming its own seam is an asset only if
something checks the seam when it is filled; the person filling it is usually
working in another file. The test must assert a **relation** — whatever the pen
holds is what is authored — and not a magnitude, because two copies of one
constant cannot disagree.

**An engine field that exists, is documented, and is written into the file is
not evidence that anything reads it.** Distinguishing *supported* from
*accepted and discarded* takes following the value to its consumer.

**A blocker naming `pdfcer-core` is a claim this repository cannot re-check,
and it decays silently** — an internal blocker fails a test when it is removed,
an external one just goes on being read. Write it as a citation carrying the
symbol and the file it was read in, not as a verdict, and re-derive it before
acting on it. `D:\Dev\pdfcer` is read-only from here but it is greppable.

**A surface on an irreversible operation raises the bar.** A partly honoured
setting is normally a papercut; on redaction or flatten it is an operator
believing the wrong thing about content that no longer exists.

**Prefer a default the operator meets on every document to one they meet
once.** A value met once is a preference nobody misses; a value they must
correct by hand at every open is a feature that has quietly been made their
job.

---

## 1. Markup

| Tunable | Value | Defined | Surface / verdict |
|---|---|---|---|
| Preview arrow head length | 14.0 px, screen space | `canvas/markup/band.rs:198` | none, and must not have one |
| Preview arrow head angle | 0.42 rad | `canvas/markup/band.rs:200` | same |
| Highlight preview wash alpha | 90 of 255 | `canvas/markup/band.rs:325` | same |
| Ellipse preview tessellation | 48 segments | `canvas/markup/band.rs:188` | same |
| Pen width range | 0.25 to 12.0 pt | `canvas/markup/pen.rs:355,376` | none — the bounds of a live control |
| Pen opacity floor | 0.1 | `canvas/markup/pen.rs:367` | none, deliberate |
| Author `/T` and modification date `/M` on markups authored from geometry | never written | `app/actions/apply.rs:672,843` | none — nothing is blocked |

### The four preview constants are the cursor, not the mark

The head geometry, the highlight wash and the ellipse tessellation are
**screen-space preview** values. The committed annotation's `/LE` head is drawn
by the appearance stream at whatever size the engine chooses, and the committed
translucency is the pen's `/CA`; these numbers state direction and legibility
during the drag, and promise nothing about the authored mark.

A control over them would read back real and change nothing the operator
authors. It would also break the one thing the preview is for: the head is
fixed in **screen** points precisely so it does not shrink to nothing at 25 %
zoom and stop saying which end is the head.

`RIBBON_IA.md`'s Arrowheads group means the **annotation's** `/LE` style — open,
closed, diamond, none — a document property this shell does not author and
which has nothing to do with these numbers. Two rows that look identical in a
table can be a real gap and a category error.

### The opacity floor is deliberate

Fully opaque writes no `/CA` at all, so a build whose operator never opens the
control authors the bytes it always did. The floor is a tenth because a control
whose bottom end authors an invisible mark is a defect report waiting to be
filed.

### The author name reaches text annotations and not geometric ones

`app::actions::apply` builds its `MarkupOptions` with `opacity`, `dash` and
`..Default::default()`, so a mark authored from geometry carries no `/T` and no
`/M`. The name is already a preference and already reaches text-bearing
annotations — `app::actions::textannot` reads `prefs.author_name` and applies it
with `MarkupNote::by`, and `/T` and `/M` are reachable only through the note.
The gap is one field on a struct this call site already builds.

### The ink simplification tolerance follows the live pen and must keep doing so

`Pen::simplify_tolerance_pts` derives from the pen's current width, not from a
constant. At a 0.25 pt pen a fixed 0.5 pt tolerance is four times the stroke's
half-width, so simplification can move the drawn centreline outside the stroke
and author a curve the operator did not draw. Asserted at both ends of the
operator's range. A `const` cannot follow anything.

### The dash can be written and cannot be read back publicly

`annot_author::read_border_dash` is `pub(crate)`
(`D:\Dev\pdfcer\crates\pdfcer-core\src\annot_author.rs:840`), and
`spec_from_dict` carries no dash: a dash cuts across `MarkupSpec`'s variants, so
the engine travels it in `AppearanceOptions` beside the spec rather than inside
it. There is therefore no public route from an annotation dictionary to *is
this mark dashed, and how*, and a chooser that cannot show the current value
shows an invented one.

`canvas::markup::linestyle::read` is this shell's copy of that function,
declared as a copy in its own header, with ISO 32000 Table 166's patterns
transcribed from the engine's doc comment rather than re-derived.

**What bounds the risk** is that the copy can only affect the *display*:
picking an entry sends an absolute `Set(pattern)` or `Clear` that does not
depend on what was read, so drift shows a wrong current style until the control
is touched and can never silently rewrite a pattern. Re-check that bound if the
reading is ever allowed to decide what gets **written**. If `read_border_dash`
is published, delete the copy and call it.

---

## 2. Canvas — snap, grid, guides, rulers, zoom

| Tunable | Value | Defined | Surface / verdict |
|---|---|---|---|
| Snap tolerance | 10.0 px, screen space | `canvas/snap.rs:136` | none |
| Selection tolerance | 6.0 px, screen space | `canvas/mapping.rs:94` | none |
| Object fallback tolerance | 3.0 | `panels/objects/provider/mod.rs:272` | none |
| Grid pitch | no spacing variable — ladder-derived, floor 8.0 pt | `canvas/grid.rs:73` | none |
| Grid minor and major alpha | 26 and 56 of 255 | `canvas/grid.rs:82,92` | none |
| Guide catch radius | 4.0 pt, screen space | `canvas/guides.rs:240` | none |
| Guide and discard alpha | 170 and 60 of 255 | `canvas/guides.rs:249,259` | none |
| Guides per document, and the store's cap | 256, store 200 | `canvas/guides.rs:225,211` | none |
| Ruler thickness | 22.0 pt | `canvas/rulers.rs:237` | none |
| Ruler minimum major pitch | 76.0 pt | `canvas/rulers.rs:252` | none |
| Ruler major and minor tick | 6.0 and 2.5 pt | `canvas/rulers.rs:265,273` | none |
| Ruler page-span alpha | 40 of 255 | `canvas/rulers.rs:282` | none |
| Ruler fallback number format | millimetres at precision 2 | `canvas/rulers.rs:505` | none — the scale dialog can set a format, the fallback's precision cannot be reached |
| Zoom minimum and maximum | 0.10 and 8.0 | `viewer/mod.rs:168,173` | none |
| Zoom-region minimum extent | 8.0 px | `canvas/zoom.rs:121` | none |
| Canvas fit margin | 16.0 | `canvas/present.rs:82` | none |
| Grip size and grab slack | 8.0 and 2.0 px | `canvas/handles.rs:100,109` | none |
| Page row and spread gap | 12.0 and 6.0 | `viewer/strip.rs:98,106` | none |
| Snap marker size | 6.0 pt | `canvas/measure/mod.rs:1269` | none |
| Arc preview steps | 24 | `canvas/measure/pick.rs:551` | none |
| Measure scale-entry seeds | real length in Meter, ratio basis Inch, ratio 1:100 | `canvas/measure/scale.rs:150-158` | none — `Default`-only |

### Selection is tighter than snap, and they are siblings rather than duplicates

`SELECT_SCREEN_TOLERANCE_PX` is 6.0 and `SNAP_SCREEN_TOLERANCE_PX` is 10.0, and
the gap is the invariant: a snap is an **offer** and a selection is a
**commitment**, so the commitment asks for the closer aim. Two constants that
look like one is exactly the shape of an obvious tidy-up, and unifying them is
wrong. `canvas/input.rs` says so at the one call site that could make the
mistake — passing `SELECT_SCREEN_TOLERANCE_PX` there compiles, runs, and merely
drifts with zoom. Neither number is reachable from a control, so a
reader who meets only one of them has nothing to warn them about the other.

The scale-entry seeds are the row in this section worth acting on first, and
the reason is the class rather than the value: the dialog offers a unit combo,
but the starting values are not preference-backed, so a drafter whose drawings
are all 1:50 in millimetres re-picks the unit on every document.

---

## 3. Render, redaction, OCR, print

| Tunable | Value | Defined | Surface / verdict |
|---|---|---|---|
| In-frame render budget | 12 ms | `render/worker.rs:166` | none |
| Thumbnail cache | 64 | `panels/pages/thumbnails.rs:355` | none |
| Thumbnail width, and the tile floor | 140 pt, floor 112 pt at `panels/pages/mod.rs:177` | `panels/pages/thumbnails.rs:216` | none |
| Thumbnail quality | pinned `Normal` | `panels/pages/thumbnails.rs:1028` | none — **deliberate**, argued in place; do not "fix" |
| Overlay alphas: ghost, find hit, current hit, text selection | 150, 40, 96, 40 of 255 | `canvas/overlay.rs:235,850,884,957` | none — the find-highlight **colour** is unreachable |
| Redaction minimum verifiable length | 4 characters | `redact/proof.rs:149` | none — it governs the refusal half of the proof as well as the disclosure |
| OCR target pixels | 8,400,000 | `ocr/mod.rs:227` | none |
| OCR DPI ceiling and floor | 300 and 50 | `ocr/mod.rs:236,244` | none |
| OCR language | none — one model directory, no language selection | `ocr/mod.rs:252` | none |
| Print preview render DPI, and maximum side | 150, 2200 px | `dialogs/print/preview.rs:231,249` | none |
| Print preview zoom minimum, maximum, step | 0.25, 40, 1.25 | `dialogs/print/preview.rs:256,258,261` | none |
| Print preview strip height, canvas height bounds, fit margin | 96 pt, 160 to 1400 pt, 0.92 | `dialogs/print/preview.rs:191,196,204,220` | none |
| Default paper for a job that plans **no** pages | US Letter portrait, 612 × 792 pt | `dialogs/print/mod.rs:1464` | none — **and deliberately none** |
| Custom paper size, given by dimensions rather than by the driver's form list | not constructible — `PaperChoice` has no `Custom` variant | `dialogs/print/spooler/mod.rs:319` | partly reachable, deliberately |

**The find-highlight colour cannot be a control until the theme has a role for
it.** Every colour on this canvas comes from the theme
(`tools/gates/check-theme-colors.sh` enforces it) and the theme has no role
meaning *the search hit you are on*. Borrowing `warn_fg_color` would say
*warning* on a control warning about nothing, and would be wrong the first time
somebody restyled the warning colour for warnings. So the current hit is the
same colour more than twice as opaque **and** stroked — two weak signals rather
than one hue.

**The no-pages paper default governs an unreachable value.** Such a job spools
nothing, so the figure never reaches paper; it exists so the commit path
carries no `Option` for a case that cannot print.

**The custom paper size is reachable through the driver.** `pdfcer_print`'s
`PaperSelection::Custom` takes a sheet in tenths of a millimetre and this shell
has no size-entry field, so `PaperChoice` mirrors only `DeviceDefault`, `Form`
and `AutoFromPages`. **Properties…** is the driver's own dialog, has a size
entry, and a configuration naming a custom sheet is read back and disclosed.
Worth building only on evidence that the driver's route is inadequate — a
roll-fed plotter operator is the likely reporter.

---

## 4. Persistence, panels, shell chrome

| Tunable | Value | Defined | Surface / verdict |
|---|---|---|---|
| Recent-file cap | 10 | `app/recent.rs:133` | none |
| Recent presence TTL | 2 s | `app/recent.rs:140` | none |
| Layout autosave settle and maximum defer | 750 ms and 5 s | `app/persistence.rs:155,164` | none |
| Remembered per-document entries | 200 | `viewer/remembered.rs:134` | none |
| Navigator, inspector and Edit-inspector default width | 280, 320, 360 pt | `app/modes/defaults.rs:262,273,332` | none |
| Window initial and minimum size | 1100 × 800 and 640 × 480 | `lib.rs:223,230` | none |
| Icon size | 16.0 pt | `icons/mod.rs:171` | none for the 16 itself — it is a size *in points* that the UI-scale setting multiplies through `pixels_per_point` |
| Icon cache | 512 | `icons/cache.rs:92` | none |
| Status bar height and row height | 30.0 and 24.0 pt | `app/status.rs:435,516` | none |
| Object-tree point rows per part | 200 | `panels/objects/mod.rs:246` | none |
| Form editor text ratio and range | 0.62, clamped 9 to 22 pt | `canvas/forms/boxes/mod.rs:71,78` | none |
| Maximum traced form boxes | 64 | `canvas/forms.rs:690` | none |
| Glyph ascent and descent | 0.85 and 0.22 | `canvas/textsel.rs:415,419` | none |
| Font subset tag in a displayed name | always stripped | `text/panels/objects.rs:430`, `panels/properties/text.rs:978`, `panels/fonts.rs:185` | none — and the three must agree |

### The subset tag is one decision taken in three places

`AAAAAA+SpaceGrotesk-Bold` and `SpaceGrotesk-Bold` name **different** font
objects in a PDF, so stripping the six-letter ISO 32000 §9.6.4 tag for display
makes two distinct fonts read as one. The constraint is the consistency, not
the stripping: strip it on every surface or on none. Strip it in one and not
another and the same font reports two different names on one screen with
neither surface wrong, which is the failure no reader can attribute.

The Fonts panel is the model for a surface that strips: the row shows the
de-prefixed name and the header tooltip resurfaces the full `/BaseFont`, so two
identical adjacent rows read as *this document subsetted the face twice* rather
than as a rendering fault. `panels/properties/text.rs::shorten` strips for
display only and pushes the **full** name on the action, because `set_font`
accepts either and the shell is better off not owning the stripping rule.

`panels/objects/summary.rs` does not state the rule, and it is where someone
building a fourth readout would look.

### Where the value-editing widgets are

A census is a measurement, and a measurement taken across a moving tree belongs
in a command rather than in prose:

```bash
grep -rn "color_edit_button\|DragValue\|Slider\|TextEdit\|checkbox\|ComboBox" \
  crates/pdfcer-gui/src/ --include=*.rs | grep -v "mod tests"
```

### What a colour control costs the harness

The idiom is `color_edit_button_srgb` / `color_edit_button_srgba` against an
`egui::Color32`, converted at the seam; the markup pen's swatch is a grid of
named presets with the full picker one click below it.

`tools/ui-verify/src/checks/markup_style.rs` records the limit: **the picker's
popup publishes no regions**, so a check can assert the swatch was drawn and
driven but cannot aim at a hue inside it. Any new colour control inherits that,
and should be paired with a driveable non-popup control — as the Style group
pairs its swatches with a width — so the group's check has something it can
actually move.

---

## 5. Engine render counters no operator can see

`pdfcer_render::Diagnostics`
(`D:\Dev\pdfcer\crates\pdfcer-render\src\interpret.rs:207`) carries far more
counters than this shell reads. Two routes read any of them:

| route | what it shows |
|---|---|
| `app::status::notes::findings()` — a fixed table filtered to non-zero | shown in **both** the status line and the Diagnostics dialog |
| `dialogs/diagnostics.rs:239` alone | `tolerated` and `compat_skipped` |

Count both ends rather than quoting a number here:

```bash
grep -c "^    pub " D:/Dev/pdfcer/crates/pdfcer-render/src/interpret.rs
grep -rn "diagnostics\." crates/pdfcer-gui/src/ --include=*.rs | grep -v tests
```

**This is not "surface all of them".** Most of the unread fields are
measurements — `images_rendered`, `annotations_painted`, `ramps_sampled`,
`overprint_pixels` — and a dialog listing every one is the noise that trains an
operator to stop reading it, which is the failure the notes table was designed
against (`app/status/notes.rs`'s header).

The subset that is owed a surface is the **refusals and silent degradations**:
an inference the operator cannot see still owes an off-canvas report. Grouped by
what an operator would want to know, and read by nothing today:

| what happened | the counters |
|---|---|
| the engine was asked to composite and did not | `blend_modes_ignored`, `soft_masks_ignored`, `soft_mask_transfer_ignored`, `transparency_groups_knockout_approximated`, `overprint_refused` |
| it could not paint something it found | `shading.refused`, `shading.missing_function`, `shading.function_unloadable`, `shading.function_arity_mismatch`, `color.patterns_unpainted`, `images_codec_unsupported`, `codec_feature_unsupported`, `mask_refused`, `images_mask_unsupported` |
| it approximated a colour | `color.tint_transform_not_applied`, `color.separation_all_approximated`, `color.indexed_index_clamped`, `color.indexed_lookup_short`, `color.icc_alternate_used`, `color.icc_device_fallback_used`, `images_uncalibrated_colorimetry` |
| an annotation is not on screen | `annotations_hidden`, `annotations_appearance_state_missing`, `annotations_placement_degenerate`, `page_content_suppressed` |
| the file is malformed and the engine coped | `lzw_framing_anomalies`, `codec_geometry_mismatch`, `xobject_depth_overflows` |

`image_notes`, `annotation_notes`, `color.notes` and `shading.notes` are
**per-occurrence explanations rather than counts** — the natural body of a
Diagnostics section, and read by nothing at all.

**Not blocked on anything.** Every field is `pub` on a report this shell already
holds: `texture.diagnostics` is in hand at `dialogs/diagnostics.rs:223`. This is
a layout decision, not a capability gap.

---

## 6. Registered commands with no dispatch arm

`shell/commands/reach/register.rs` holds both allow-lists, and **both are
empty**: `SCAFFOLDED` (a command drawn on the ribbon and left unwired) and
`UNREACHED_ARMS` (a dispatcher arm no token can reach).
`the_p3_tension_is_counted` at `shell/commands/reach/mod.rs:1186` pins both
lengths, so adding an entry is a visible act. Read the register; a count copied
into a document nothing checks is how a number drifts.

A capability that is not built is expressed by **not registering** its command
(R8) and renders nothing (R9). Greying is reserved for the *temporarily*
unavailable and is always explained on hover, and *"the dialog was never
written"* is neither temporary nor a sentence anyone can write.

Three properties of that list are worth carrying into whatever replaces an
entry:

- **An entry with no recorded reason is not deferred work — it is unexamined
  work.** It reads as *somebody looked and found nothing*, which is
  indistinguishable from *somebody deferred this deliberately and forgot to say
  why*; the first invites a re-derivation and the second discourages one.
- **The list can prove an id has no arm. It cannot prove the reason is true** —
  a reason is prose, and a reader is the only instrument. The runtime check that
  can is in `tools/ui-verify`: press every registered id and fail on any
  `command-unimplemented` trace line.
- **A blocker naming a missing *host* is weaker than one naming a missing
  *capability*,** and goes stale the moment any other host will do — silently,
  because nothing changed to make it stale.

When you touch that list for any purpose, re-derive the reason of the entry
beside the one you came for.
