# pdfcer GUI — Ribbon Information Architecture

**Status:** proposal, 2026-08-12
**Scope:** where every user-reachable command lives, and why.
**Companion documents:** `GUI_ROADMAP.md` (when each part gets built),
`DEFECTS.md` (what is broken today), `mockups/` (what it looks like),
**`MODES_AND_PANELS.md`** (the Read/Review/Edit selector and the
flexible panel system — an operator addition of 2026-08-13 that layers
on top of everything here).

> **Amendment, 2026-08-13.** The seven-tab layout below describes
> **Edit** mode. `MODES_AND_PANELS.md` adds a three-position selector at
> the far right of the tab row; **Read** and **Review** are subsets of
> this layout, not different ones. No command specified here moves, and
> nothing here is superseded — the selector governs which tabs and
> panels are *present*, not where a command lives when it is.

---

## ★★★ Amendment, 2026-09-05 (LATEST) — the band was DRIVEN, and the three-row change had reached nothing on screen

The amendment below this one ends with a block headed **"⬜ What is STILL
unmeasured, named rather than implied"**, which says that every number in it
came from the mockup's side and that *"the product's own rectangles were NOT
captured"*. They have been now. This section is what they said.

### ★★★ 1. AT 1400 PX, NOT ONE GROUP ON THE BAND USED A SECOND ROW

Release binary, launched off screen with
`PDFCER_DIAG_VIEWPORT="-4200,-4200,1400,900"` and `PDFCER_DIAG=1`, File tab in
Edit mode, `ribbon.item.*` and `ribbon.group.*` read out of its own trace:

```text
file.file                x=8..401    (393 pt)  items=4  rows=1  tops=[41.0]
file.recognise.collapsed x=415..494  ( 79 pt)                         <- collapsed
file.save                x=508..944  (435 pt)  items=4  rows=1  tops=[41.0]
file.export.collapsed    x=958..1016 ( 59 pt)                         <- collapsed
file.security            x=1030..1196(166 pt)  items=2  rows=1  tops=[41.0]
file.print               x=1210..1274( 64 pt)  items=1  rows=1  tops=[41.0]
```

**Six groups, every one of them one row, two of them collapsed into captioned
buttons — with 126 pt of band still unspent.** The band's row area is 68 pt and
it was carrying a single 21.7 pt row: forty-six points of air under every
control, exactly the picture the amendment below describes as *"the old band
with twelve points of air in it"*, only worse, because the budget had since
been raised to three rows and nothing consumed it.

★★ **`GROUP_ROWS` was 2 and became 3, and that was not the constant that
decides anything.** `plan::wrap_group` short-circuits on
*"it fits within `GROUP_WRAP_WIDTH` (440 pt) on one row, so leave it"*, and
**no File-tab group is 440 pt wide on one row** — the widest was 435. So the
row ceiling could have been three, or ten, and the band would have drawn one
row either way. The table in the amendment below (*"Zoom 2 × 3"*, *"Panels
3 × 3"*) describes a layout the running binary did not produce; it was computed
from the mock's rectangles and this shell's unit tests, and its own report said
so.

⇒ **The fix is one argument at one call site**, `band::measure_group_rows`:
a group now asks for the row ceiling it is being planned against unless its
manifest asks for something else. `wrap_group` still returns the *narrowest*
packing within that ceiling, so this demands nothing — a pair whose 1 × 2 is
narrowest still gets two rows, a single item still gets one, and O97's
`prefer_rows: 2` on View ▸ Page display still wins. What it removes is only the
short-circuit, which `prefer_rows`'s own doc already calls *"exactly the right
to skip the fits-already test"*.

### ★★★ 2. WHAT THE SAME MEASUREMENT SAYS AFTER — two more commands reachable

Same binary, same command line, rebuilt:

| | before | after |
|---|---|---|
| groups **on the band** at 1400 | 6 | **8** |
| File ▸ File | 393 pt, 1 row | **279 pt, 3 rows** |
| File ▸ Save | 435 pt, 1 row | **241 pt, 3 rows** |
| Recognise (`file.ocr`) | **collapsed** | **on the band, drawn** |
| Document, pdfcer | past the overflow | on the band |
| row tops in a 3-row group | — | **41.0 · 63.7 · 86.3** (22.7 pt pitch) |

The pitch is the mockup's: `.rb { height: 22px }` over `.grp .col { gap: 1px }`,
which is where `3 × 22 + 2 × 1 = 68` comes from. At 1700 px the whole File tab
fits with all eight groups open and Export drawing seven controls in 344 pt.

★ **The width series was walked, not sampled at its ends** — 1700, 1400, 1300,
1200, 1100, 1000, 900, 800, because two samples either side of a transition look
exactly like no transition. Distinct controls drawn and groups open, per width,
after the change:

| width | 1700 | 1400 | 1300 | 1200 | 1100 | 1000 | 900 | 800 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| groups open | **8** | 5 | 5 | 5 | 4 | 3 | 2 | 2 |
| groups collapsed | 0 | 3 | 3 | 3 | 2 | 2 | 2 | 2 |
| controls drawn | 25 | 12 | 12 | 12 | 11 | 10 | 8 | 8 |

The ladder still engages, monotonically, all the way down. ⚠ There is no BEFORE
column and it is not reconstructed: the launcher writes one trace per width and
the second run overwrote the first. What survives of the before state is the
1400 table above, quoted while it was in front of me. Traces are in
`D:\temp\pdfcer-scratch\geo\trace-<width>.txt`; `analyse.py` and `series.py`
beside them print these from the **last** frame of each run.

### ⬜ 3. WHAT IS STILL DIFFERENT, AND IT IS AN ALGORITHM, NOT A NUMBER

**The mockup is COLUMN-major and the product is ROW-major.** The mock stores a
group's items as an array of columns —
`{cap:'Panels', items:[[Pages,Bookmarks,Layers],[Objects,Signatures,Fill form]]}`
— and its CSS lays each column out as a flex column, so an operator reads *down*
the first column and then *down* the second. `wrap_group` partitions the same
items into contiguous **rows**, so the operator reads *across*. Both produce a
2 × 3 block of the same footprint; the item in each cell differs.

Not changed, and the reason is a schema fact rather than a preference: **the
column split does not exist in `built_in.ron`.** Matching the mock faithfully
means the manifest carrying columns, which is a change to
`egui_shell::manifest::Group`, to the RON generator, to `measure_group` and to
`captioned_group` — and the mock is not self-consistent about it either (its
Edit ▸ Content draws the small column *before* the Large run, where every other
band and the shell's own renderer put Large first). Named here rather than
implied; it is the largest open ribbon question.

⬜ **Also unmeasured: item SIZE.** The mock draws `Edit text` and `Add text` as
Large; the manifest registers both Small. `compare-mockup-ribbon.py` does not
compare size and says so.

---

## ★★★ Amendment, 2026-09-05 — the sixteen item divergences, and NINE OF THEM WERE THE INSTRUMENT

The amendment below records sixteen groups whose item sequences differ and
calls that list *"the measured backlog"*. **Seven of the sixteen were the
script comparing two private vocabularies, and two more were its parser
recognising one of three item kinds.** The residue was nine real ones, and this
is the ledger of which side moved for each.

### ★★★ The correction, because it is worth more than the fix

The item phase compared **icon KEY strings**, and the amendment below states
its finding in bold:

> ~~*"`folder` vs `open`, `printer` vs `print`, `scissors` vs `cut`, `ruler` vs
> `measure` mean the two sides are drawing **different pictures** on the same
> button."*~~

**Six of those eight name the same picture.** `icons/catalog/mapping.rs` says so
on one line each:

```rust
Icon::Open | Icon::FontFolders => assets::FOLDER,   // keys "open", "font-folders"
Icon::Print                    => assets::PRINTER,  // key  "print"
Icon::Measure                  => assets::RULER,    // key  "measure"
Icon::Export                   => assets::DOWNLOAD, // key  "export"
Icon::InsertImage              => assets::IMAGE,    // key  "insert-image"
Icon::EditText                 => assets::EDIT,     // key  "edit-text"
Icon::ImportFormData           => assets::UPLOAD,   // key  "import-form-data"
```

The mock's key is the **asset basename** — its generator reads the inventory out
of `icons/assets/`, which is what makes *"this glyph ships"* true by
construction there. The product's key is the **role**, and
`icons/catalog/mod.rs`'s header declares that deliberately: *"a distinct key
over shared art"*, so that `Open` and `FontFolders` can both be the folder and
remain two commands. Comparing the two key schemes asks a question **neither
side is answering**.

⇒ The script now resolves both sides to the **asset each control draws**
(`asset_by_icon_key`), and teaches its RON parser the other two item kinds
(`Custom`, `Separator` — see `ron_groups`). 16 → 9.

★★ **Both errors ran in the expensive direction: the instrument MANUFACTURED
work.** Obeying it would have re-pointed `file.open` at a key spelled `folder`,
changed nothing on screen, and closed a defect that never existed. That is the
third time this has been recorded about this one script, from the same cause —
*ask what the instrument SAMPLED.*

### The nine real divergences, and which side moved

| # | group | the difference | side that moved | why |
|---|---|---|---|---|
| 1 | File ▸ Document | mock `document`, product `properties` — **two adjacent buttons drawing the same picture** | **product** | `document.svg` shipped and was an ORPHAN, kept only so `every_icon_parses` would keep walking it. `catalog/edit.rs` scopes the shared-key convention to controls *with the same SUBJECT*; this pair has different subjects (the file, and whatever is selected on the page). The registration's *"the alternative is not 'draw one' but 'ask him for one'"* was written four lines under this document's own rule about verifying absences against the source |
| 2 | File ▸ File — Recent | mock draws an icon **and** a word; product drew a bare `menu_button` | **product** | `file.recent` has carried `.with_icon("recent")` since the glyph landed, and its registration named the exact file where the other half of the work lived. It sat unactioned. `app::recent::menu` now calls `menu_image_text_button` |
| 3 | View ▸ Panels — Objects / Signatures | mock puts Objects fourth | **product** | the group's only positional argument is about Forms being **last** (*"the operator meets the read-only surfaces first"*); nothing said why Signatures came fourth. Where the product has no argument and the mock has a position, the mock is the spec |
| 4 | View ▸ Panels — Properties, Comments, Fonts | mock has three extra controls | **mock** | ⚠ **each already appears on another tab** (`format.properties`, `markup.comments`, `file.fonts`), so adding them violates **P1** — which `Shell::validate` enforces, and an invalid manifest is not a local failure: `Capabilities::for_mode` returns `FULL` when the shell is absent, silently granting every authoring capability to every mode. Note the mock draws Comments in Markup ▸ Comments **as well**, so it violates P1 on its own face |
| 5 | View ▸ Navigate | mock: `pointer` for Select and `show-points` for **both** Points and Smart select | **mock** | `pointer.svg`'s own comment says it is *the Tool panel's* glyph — *"what you are holding"*; `cursor.svg` is the Select tool's arrow, *"the single most standardised glyph in the whole product class"*. And one band drawing `show-points` twice is the exact fault this project refused for the five form-field controls |
| 6 | Pages ▸ Clipboard | mock `scissors`, product `cut` | **mock** | `scissors.svg`'s own comment calls it *"placeholder"*; `cut.svg` is the authored glyph with a paragraph on why the blades cross above the rings. The mock's **own** Edit ▸ Clipboard already used `cut`, so it disagreed with itself |
| 7 | Pages ▸ Organise | mock puts Merge third | **mock** | §5.3 of this document lists the order — Delete, Extract, [Replace], Move up/down, [Split], **Merge** — and the product is that list with the unbuilt rows absent. The mock is the only one of the three that disagrees |
| 8 | Edit ▸ Clipboard | product has a second paste | **mock** | `edit.paste_duplicate`, O58, 2026-08-29. It shares `paste` deliberately: *"a second paste glyph would be a distinction the operator has to learn for no gain — the two are told apart by their labels and by the chord in the tooltip, which is how Word and Acrobat tell their paste variants apart too."* The mock gained the control, not a new glyph |
| 9 | Edit ▸ Content | mock puts Reflow second | **mock** | the product's order carries a causal argument — *"an operator reflows a paragraph because they have just retyped a sentence in it, so it follows the tool that does the retyping"* — and the mock's has none |
| 10 | Markup ▸ Style | mock draws a fourth control, `100 %` | **mock** | annotation opacity is `/CA`, which `pdfcer-core` does not write. **R9**: an affordance for something that cannot happen is the worst kind of placeholder — the mark would be authored fully opaque and the operator would have no way to tell |
| 11 | Tools ▸ Batch | mock `convert`, product `combine` (`link.svg`) | **mock** | the mock's own comment two groups above calls `convert.svg` *"an orphan variant so the art stays under test"* and its former alias *"a false claim"*. Merging files is two links joined |

*(Eleven rows for nine groups: View ▸ Panels carried two independent
differences, and File ▸ File's Recent control was found by READING, not by the
instrument — a `Custom` item carries no command id, so nothing on the data side
could say what it draws. It is held by the instrument now, through
`CUSTOM_GLYPHS`, which is a hand-maintained claim and says so at the table.)*

**Where a capability shipped after the mock was drawn, or the product's choice
carries a written argument and the mock's does not, the mock moved. Where they
simply disagreed, the product moved. Where the mock's picture would break a P1
invariant, the mock moved and the reason is recorded above rather than left to
be rediscovered.**

### The instrument, and its falsification

`python tools/compare-mockup-ribbon.py` exits **0** on both phases.

It was falsified against all four sources it reads — a byte copy taken first,
the plant asserted to have landed by reading the file back, and the script's own
`DIFFER` line required rather than its exit code alone:

| plant | result |
|---|---|
| `file.document_properties` icon back to `properties` | exit 1, names `'Document'` |
| `Custom(kind: "recent_files")` deleted from the RON | exit 1, names `'File'` |
| the mock's `Print…` glyph changed to `save` | exit 1, names `'Print'` |
| `Icon::Print => assets::SAVE` in `mapping.rs` | exit 1, names `'Print'` |

Restored, and green again. The fourth is the one that matters: it proves the
**asset bridge** is load-bearing rather than decorative — a comparison that had
quietly fallen back to key strings would still have been green there.

⚠ **What it still cannot see**, unchanged and now longer: item **size**, item
**label**, and the **column split** described in the geometry amendment above.
`CUSTOM_GLYPHS` in that script is a hand-maintained claim about what each
`Item::Custom` paints, and it says so at the table.

---

## ★★★ Amendment, 2026-09-05 (LATER THE SAME DAY) — the visual half was checked for the first time, and the band was TWO ROWS where the mockup is THREE

Everything in the amendment below this one is still true and is still not
enough. It says, in bold, that *"the visual half has never been checked once"*
and that its only oracle is a rendered screenshot. That screenshot was taken on
2026-09-05, and it found three things.

### ★★★ 1. THE MOCKUP HAD BEEN DEAD FOR A DAY — a single apostrophe

`mockups/pdfcer-shell.html` rendered **nothing**: no ribbon, no rail, no panels,
no dock, no status bar, no legend. The whole file is one `<script>` that builds
every region from data, and on 2026-09-05 a note added to the legend contained
the words *"I didn't refuse that."* inside a **single-quoted** JavaScript string.
The apostrophe closed the string; the parse failed; `renderAll()` never ran.

```text
Uncaught SyntaxError: Unexpected identifier 't'   (line 1740)
```

⇒ **The artifact the operator was comparing the product against was a blank
window frame.** Both the template and the generated file are fixed, and the
apostrophe is escaped.

⚠ **Its own generator has a smoke test, and the smoke test passed.**
`build-pdfcer-shell.py` prints *"smoke test OK — 3 regions rendered"* and printed
it on the broken file. Its DOM-size figure moved by **71 KB** between the broken
and the fixed build and it reported OK both times. A smoke test that cannot
distinguish a page that renders from a page that renders nothing is the vacuous
shape this project keeps finding; the reliable oracle is
`node --check` over the extracted `<script>`, which fails in half a second and
names the line.

### ★★★ 2. THE BAND IS THREE ROWS. It was two, and the mockup's own arithmetic says so

The operator: *"it looks like the edits to the ribbon got halfway done."* This is
the half that was left.

On 2026-09-04 the mockup's vertical numbers were adopted into
`egui_shell::theme::Metrics` — `ribbon_rows: 68`, `ribbon_pad_top: 6`, an 11 pt
caption, a 56 pt Large control. **The row count that produces 68 was not.** The
mockup's stylesheet says what 68 is made of:

```css
.rb        { height: 22px }     /* a small control          */
.grp .col  { gap: 1px }         /* between its rows         */
                                /* 3 × 22 + 2 × 1  =  68    */
```

`plan::GROUP_ROWS` was **2**, and a band 68 pt tall holding two 24 pt rows is not
the mockup's band — it is the old band with twelve points of air in it. Measured
side by side at 1400 px on the `View` tab:

| group | mockup | product before | product now |
|---|---|---|---|
| Page display | 2 × 2 | 2 × 2 | 2 × 2 (`prefer_rows: 2`) |
| Zoom | 2 cols × 3 rows | 3 × 2 | 2 × 3 |
| Display | 2 × 3 | 3 × 2 | 2 × 3 |
| Panels | 3 × 3 | 5 × 2 | 3 × 3 |
| Window | 2 × 3 | 3 × 2 | 3 × 3 (nine items — see below) |

The mockup fills 1,379 px of a 1,400 px window with those six bands. At two rows
the same six need roughly 460 px more than they are given, **so groups the mock
shows on the band were collapsing or scrolling at his window width.** That is
what "halfway done" looked like.

`GROUP_ROWS` is now **3**, and the third row costs *nothing*: the band's height is
`ribbon_rows`, a fixed budget the rows are laid into, so 68 pt before and 68 pt
after. The old refusal of three rows was written when the band was two rows tall
*by definition* and its screen-budget argument lapsed with that change; the whole
refusal, and why each half of it lapsed, is preserved at the constant.

### ★ 3. Item-level divergences, four of them, all corrected

Found by reading the two sides by hand — which is exactly what the instrument
does **not** do, and its green exit had never had anything to say about them:

| divergence | which side moved | why |
|---|---|---|
| the `pdfcer` band drew `Settings…` alone | **the mock** gained `Shortcuts` and `About` | the manifest has carried all three since `file.about` shipped, 2026-08-14 |
| `Copy as vector` was labelled in the mock, icon-only in the product | **the mock** | the product's choice is the considered one; a fifth long label widens the group past its neighbours |
| `Dock all panels` absent from the mock's Window group | **the mock** | shipped 2026-09-04, after the mock was drawn |
| `Auto-hide ribbon` / `Auto-hide left strip` absent | **the mock** | shipped 2026-09-05; see below |

**Where a capability shipped after the mock was drawn, the mock moved. Where they
simply disagreed, the product moved.** Said explicitly because the standing rule
is *the mockup is right by definition*, and three of these four are the exception
that rule already names.

### The two new commands, and why the Window group is now nine items

`view.ribbon_auto_hide` and `view.rail_auto_hide` — his instruction of
2026-09-05: *"we should also add the capability to auto hide the ribbon until we
hover over top of it… left rail should also have the option to auto hide as
well."* They sit in **View ▸ Window**, after `Dock all panels` and before
`Reset layout`, on that group's established two-tier order: the reversible
remedies first, the one that discards the operator's arrangement last.

**Neither renders pressed**, and that is the convention rather than an omission —
Office's own ribbon-display control is a caret that opens a menu, not a lit
toggle. The surface that carries the *state* is the Settings window, where both
are checkboxes. The `selected:` convention would light them and is one six-line
block in `app::conditions::armed`, a file a concurrent track owned on the day
this was written; that is the only reason it is not done.

### ★★★ 4. `compare-mockup-ribbon.py` NOW COMPARES ITEMS — and it exits 1

The amendment below says, in bold, that the script *"does not compare the items
inside a group"* and gives the reason: the mock stores a **label** and the RON
stores an **id**, and the map between them is Rust the script will not build.
**That reason was true of labels and was never true of icons.** Both sides spell
the icon key in plain text, in files the script already reads:

```text
mockups/…-template.html      ['Copy as vector','copy-as-vector']
shell/commands/catalog/…     command("edit.copy_as_vector", …).with_icon("copy-as-vector")
```

So the script grew a second phase that compares **item presence and order, per
group, by icon key**, and its exit code now carries the result. Size and label
are still not compared and it says so.

★★ The first run found **sixteen groups** whose item sequences differ, so
`python tools/compare-mockup-ribbon.py` now **exits 1**, where every prior note
in this document records it exiting 0. *That is the instrument starting to work,
not a regression* — the structural phase is still clean at 0 differences. The
list is the measured backlog and most of it is one shape:

| group | mockup | product |
|---|---|---|
| File ▸ File | `folder`, and a `recent` control | `open`; no `recent` glyph in the band |
| File ▸ Export | `download`, `upload` | `export`, `import-form-data` |
| File ▸ Print | `printer` | `print` |
| File ▸ Document | leads with a `document` glyph | starts at `properties` |
| View ▸ Navigate | `pointer`, `show-points` | `cursor`, `cursor-node` |
| View ▸ Panels | nine controls, incl. Properties, Comments, Fonts | six — the other three are on File and Markup, and `RIBBON_IA.md` P1 forbids a second placement |
| Pages ▸ Insert | `image` | `insert-image` |
| Edit ▸ Clipboard (Review) | `scissors` | `cut` |
| Edit ▸ Clipboard (Edit) | four items | five — the product has a second paste |
| Edit ▸ Organise | `merge` third | `merge` last |
| Edit ▸ Content | `reflow` second | `reflow` last, and `edit` is `edit-text` |
| Measure ▸ Dimension | `ruler` | `measure` |
| Tools ▸ Batch | `convert` | `combine` |
| Tools ▸ Fonts | `folder` | `font-folders` |
| Format ▸ Style / Font | four and six glyphless entries | two and none |

★ **Most of these are not cosmetic.** `folder` vs `open`, `printer` vs `print`,
`scissors` vs `cut`, `ruler` vs `measure` mean the two sides are drawing
**different pictures on the same button** — precisely the class the operator was
looking at when he said the ribbon does not follow the mockup's format. Which
side moves is a per-row judgement (the mock is the spec; a P1 violation is not)
and **none of it was done in this session** — the instrument was built, run, and
its output recorded here.

### ⬜ What is STILL unmeasured, named rather than implied

**Every number above is from the mockup's side.** The reference PNGs and the
per-element rectangle dumps are at `D:\temp\uvdrive\mockref\`, at 1400, 1100
and 900 px. **The product's own rectangles were NOT captured**, because
`pdfcer-gui.exe` and `ui-verify.exe` were running for another track for the whole
of the session and this project's rule is that a driven run needs the machine.
So the three-row change is argued from the *mock's* geometry and from the
product's own theme metrics and unit tests, and **not** from a screenshot of the
running binary. That capture is the first job of the next session with a free
machine: launch off screen with `PDFCER_DIAG_VIEWPORT`, read `ribbon.item.*` out
of the trace, and compare the row tops against `mock-1400.png`'s.

★ And one thing the mockup itself cannot show: it has a fixed minimum width and
**no collapse behaviour at all**, so its geometry is identical at 1400, 1100 and
900 — it scrolls rather than reflowing. The nine-item Window group now runs past
the right edge of a 1400 px mock window. That is the mock's limitation, not a
specification for the product, whose ladder collapses groups instead.

---

## ★★★ Amendment, 2026-09-05 (EARLIER) — the ribbon and the mockup now AGREE, measured

`python tools/compare-mockup-ribbon.py` exits **0**. The mockup's ribbon and
`shell/ron/built_in.ron` carry the same 35 captioned bands in the same order,
across the seven fixed tabs and the contextual one.

### ★★ Read the exit code for what it is, and not for more

The script's own closing lines say it and this section repeats it because the
inference is irresistible: **structural agreement is not visual agreement, and
it is not even agreement about the controls.**

| the question | answered by |
|---|---|
| the same bands, in the same order? | **the script. Yes, exit 0.** |
| the same controls in each band — present, ordered, sized, glyphed, labelled? | **nothing.** The mock stores a *label*, the RON stores an *id*, and the map between them is Rust the script deliberately does not build |
| the same paddings, heights, framing, label placement, type? | **nothing here.** Only a rendered screenshot of the running binary |

⇒ Four real item-level divergences were found **by hand** on 2026-09-05, and
**not one of them moved the script's verdict**: `Copy as vector` and the two
Security controls marked unbuilt after they shipped, `Select all` marked
icon-less while carrying `select-all`, and `Export text…` missing from the mock
entirely. A green line from an instrument that does not look at items is
evidence about bands and about nothing else.

### The one place the PRODUCT was the spec, and why that is not a precedent

The standing rule is unchanged and is not being softened: **the operator
approved the mockup, so where the two disagree the mockup is right by
definition and the difference is a defect in the build.** The Format tab is the
single exception, and it earns the exception on grounds that are checkable
rather than on judgement:

| the mockup drew | the product ships | which side moved, and why |
|---|---|---|
| **Arrange** — *Bring forward* / *Send backward*, greyed | nothing | **the mockup.** Neither is a registered command; both sit in `manifest::PLANNED` as **N**, one of them reading *"the whole Edit ▸ Arrange group is unbuilt, so the GROUP is absent too."* R9 forbids the picture the mock drew: an unavailable capability renders **nothing**, and greying is for something *temporarily* unavailable, explained on hover |
| **Object** — *Duplicate · Delete · Properties* | **Selection** — *Properties · Select the form · Give this page its own copy · Delete* | **the mockup.** `Delete` and `Properties` are the same two commands under a different band name; **`Duplicate` names nothing** — there is no `format.duplicate`, and the nearest id in the tree, `edit.paste_duplicate`, is a clipboard verb on another tab that does something else |
| *(absent)* | **Font** — face · size · | · Bold · Italic · colour | **the mockup.** §5.8's own 2026-08-27 amendment specifies this band, it shipped that day on O37, and the mock's Format tab was drawn on 2026-09-04 without it |

★ The transferable half, and it is why this section is in the specification
rather than in a session log: **a mockup is a specification only where it is
DERIVED.** `mockups/build-pdfcer-shell.py` states that rule about itself — the
glyph inventory is read out of `icons/assets/`, never typed, so *"this glyph
ships"* is true by construction — and the ribbon in that mock is **typed**. The
Format tab was not describing an older build. It was describing no build at
all, and nothing could have told anyone, because nothing was comparing.
`tools/compare-mockup-ribbon.py` is the derivation the ribbon does not have.

### ★★ And the instrument itself was reading something that is not the ribbon

Four of the seven differences it reported on 2026-09-05 were **its own**. It
scanned the whole of `built_in.ron` for `caption:` and the file has captions
outside the ribbon: the **rail** carries three bands of its own (`Navigate`,
`Select`, `Rotate`), all three were reported as missing from an approved
design, and the rail's `Navigate` collides by name with View ▸ Navigate so the
order line ended in a phantom.

⇒ **Ask what the instrument SAMPLED before believing what it reported.** This
project has recorded that against `ui-verify` at least four times; it applies
to a forty-line regex script identically. And note the direction of the
failure: it *manufactured* work rather than hiding it, so a session obeying it
would have drawn three bands into the mock that the ribbon does not have — a
completeness instrument's most expensive failure mode is the confident one.

⚠ **The rail is out of scope for that script and diverges from the mock.** The
shipped rail's `Select` band holds `edit.select_all`; the mock's holds a
**Lasso** that has no command and no asset and is drawn with the freehand pen's
borrowed art. That is a live O123 design question, not a ribbon defect, and it
is named here so the next reader does not mistake a green ribbon line for a
statement about the left edge.

---

## 1. What this document is for

pdfcer's ribbon today has six tabs and twenty groups. The grouping is
principled — there is a real rule set behind it, written down in
`crates/pdfcer-gui/src/ribbon.rs` — but the result does not match what a
user reaching for a command expects to find, and three of the six tabs
are underfilled to the point of looking unfinished.

This document specifies a replacement layout: seven tabs, each with a
stated question it answers, each command assigned to exactly one home,
and every currently-existing command explicitly migrated. It is written
so that the ribbon could be rebuilt from this document alone, including
the reasoning behind each placement, which is the standard the rest of
this project's documentation is held to.

It is a *specification*, not a wish list. Every command below is marked
with where it exists today:

| Mark | Meaning |
|---|---|
| **G** | Exists in the GUI now. Moving it is pure re-parenting. |
| **C** | Exists in `pdfcer-core` and/or `pdfcer`, but has no GUI surface. Needs a shell, not an engine. |
| **N** | Does not exist anywhere. Needs building. |

The **C** rows matter disproportionately: they are the cheapest possible
wins, because the hard half is already written and tested.

---

## 2. Principles carried forward

These come from the existing codebase and are kept, because they are
good and because breaking them silently would be worse than the problem
they cause.

**P1 — One command, one tab.** No command appears on two ribbon tabs.
The existing rule (`ribbon.rs:64-75`) exists because Pass 47.1 found
that mirroring undo/redo onto every tab was the reason the ribbon
rendered only the active band and undo became unreachable from the
Measure tab. A test asserts every group has exactly one owning tab
(`ribbon.rs:678`). Keep both the rule and the test.

**P2 — The ribbon picks the activity; the sidebar holds that activity's
controls.** Codified at `ribbon.rs:218-220`. The Measure tab arms
"Linear ce dimension"; the group picker, scale entry, number format and
drafting standard live in the Tool Options pane. This is correct and it
is why the Measure tab has one group. The fix for an underfilled tab is
never to move sidebar controls up into it.

**P3 — No placeholders.** An unavailable capability renders nothing, not
a disabled stub (`ribbon_ui.rs:241-244`). Greying is reserved for
*temporarily* unavailable — no document open, document encrypted, undo
stack empty — and is always explained on hover.

**P4 — Group captions are mandatory.** Every group draws its caption
beneath its controls, enforced by routing all groups through one closure
(`ribbon_ui.rs:76-117`) after two groups were found rendering without
one.

**P5 — Nothing floats over the canvas.** Stated at `FEATURES.md:182`.
Tool options live in the dock, not in canvas-anchored overlays, because
accept/reject boxes that moved on every zoom were a reported defect.

### One amendment is proposed

**P1 as written also forbids the Quick Access Toolbar and status bar
from carrying a command that appears on a tab.** That is why the File
tab today has no Open and no Save — they live only in the QAT — and why
the View tab has no zoom controls, because zoom lives only in the status
bar.

That reading is too strong, and it is the direct cause of two of the
worst discoverability failures in the product. A user who wants to open
a file looks under File. A user who wants to change how the page is
displayed looks under View. Finding neither there teaches them the
ribbon is not where commands live.

**Proposed amendment — P1a:** *the QAT and the status bar are shortcut
surfaces, not tabs. A command may appear on exactly one tab and
additionally on the QAT and/or the status bar.* This is precisely how
Office's QAT is defined, and it does not reintroduce the Pass 47.1
defect, which was about a command being reachable from **no** tab when
the band collapsed — the opposite failure.

With P1a, `RibbonTab::groups()` stays the single source of truth for
tab ownership and the existing uniqueness test is unchanged; the QAT and
status bar are simply outside its domain, which they already are.

---

## 3. What is wrong with the current layout

Observed directly by driving the built release binary (`evidence/`),
not inferred from source.

**The View tab contains no view controls.** It has two groups: `Panels`
(sidebar, Bookmarks, Layers, Signatures, Fonts, Objects) and `Show`
(annotations, points). There is no zoom, no page layout, no view
rotation, no read mode, no full screen. Read mode and full screen have
**no ribbon control at all** — they are keyboard-only (Ctrl+H, F11) on a
tab literally named View. This is the single most confusing thing in the
current ribbon.

**Page operations are hidden.** Insert, delete, extract, reorder,
split and merge exist and work, but live in the thumbnail rail's
selection action bar and in a `Tools ▸ Batch` pane. Nothing on any
ribbon tab says "pages". A user who wants to delete page 7 has no path
that starts at the ribbon. The Edit tab's `Pages` group contains only
Rotate left / Rotate right, which makes the absence louder — it looks
like page ops were considered and rejected.

**The File tab is a junk drawer.** Properties, Copy this page's text,
Copy the whole document's text, Export DXF, Print, Reset layout,
Settings, keyboard shortcuts. Text copying is not a file operation. Panel
layout reset is not a file operation. Meanwhile there is no New, no
Recent, no Close, and — per §2 — no Open and no Save.

**Three tabs are underfilled.** Measure has one group and four controls.
Tools has three groups of one control each. View has two groups. On a
1936 px window each of these leaves 1200–1700 px of empty band. An empty
ribbon band reads as an unfinished program regardless of how much
capability sits behind the controls that are there.

**Two Edit-tab buttons are unlabelled icons.** The `Pages` group's
rotate buttons carry no text, and the `Content` group's buttons read
`Aa`, `I⁺ Aa` and `Obj`. `Obj` is not a word. These are the primary
content-editing tools and they are the least legible controls in the
application.

**Group captions are set in a colour that is nearly invisible** in the
default Quiet theme — the same defect that hides the dock tab labels and
the Settings dialog headings. See `DEFECTS.md` §1. It affects the ribbon
too: the words `Document`, `Clipboard`, `Export`, `Print` under the File
tab's groups are present and technically legible, but only just.

---

## 4. The proposed tab set

Seven tabs. Each keeps the existing idiom of a one-line question it
exists to answer.

| # | Tab | The question it answers |
|---|---|---|
| 1 | **File** | What do I do with the file as a whole, or with pdfcer itself? |
| 2 | **View** | What is on my screen, and how is the page laid out? |
| 3 | **Pages** | What am I doing to the set of pages? |
| 4 | **Edit** | What am I changing about content that is already there? |
| 5 | **Markup** | What am I adding for someone else to read? |
| 6 | **Measure** | What am I measuring, and in what units? |
| 7 | **Tools** | What do I run across files, or configure once? |

Plus **one contextual tab**:

| Tab | Appears when |
|---|---|
| **Format** | A markup, dimension, image or vector object is selected |

### Why seven, and why these

Six tabs was one too few for the amount of capability behind them, and
the sixth was carrying two unrelated jobs. The change is essentially:
split page operations out of hiding into their own tab, give View the
view controls its name promises, and let File become an actual file tab.

`Review` is renamed **Markup**. What lives there is markup authoring —
shapes, notes, stamps. "Review" promises a review *workflow*: compare
revisions, resolve comments, track changes. pdfcer does not have that
yet, and when it does, it will want the name. `Markup` is also the term
this project's audience uses; Bluebeam and every drafting office call it
that.

`Format` as a contextual tab is how the P2 tension resolves. Today,
selecting a placed markup gives you nowhere to change its colour,
because the Markup tab's colour swatch sets the colour of the *next*
markup. A contextual tab is the standard answer and it is already named
in the code as an unbuilt slice (`ribbon.rs:42-52`).

---

## 5. Tab specifications

Notation: groups are `**Group**`, commands are listed with their status
mark. `⌄` means the control is a split button or dropdown.

---

### 5.1 File — *what do I do with the file as a whole, or with pdfcer itself?*

| Group | Commands | |
|---|---|---|
| **File** | New (blank) | **G** *(2026-08-14; `Ctrl+N`)* |
| | New from template… (page size) | **N** — see `manifest::PLANNED` |
| | Open… | **G** *(also QAT)* |
| | Recent ⌄ | **N** |
| | Close | **G** *(currently switcher-only)* |
| **Save** | Save | **N** — still absent, and the reason is unchanged: an in-place `Save` promises overwriting the operator's file, which needs autosave and crash recovery first. ★ **But `Save a copy` is no longer blocked and shipped 2026-08-14** — it had been registered, on the QAT and bound to Ctrl+S with no dispatch arm for the whole project. The engine was never the blocker: `to_incremental_bytes`/`to_full_bytes` both take `&self`. When in-place Save does land it inherits standing instruction 5 — *Read may produce a new document; it may not modify this one* — which is the rule that is currently vacuous precisely because every save is already a copy. |
| | Save a copy… | **G** *(also QAT)* |
| | Revert | **N** |
| **Export** | Export DXF… | **G** |
| | Export image… (PNG/JPEG/TIFF, DPI picker) | **C** |
| | Export text… | **C** |
| | Export form data ⌄ (FDF / XFDF / CSV) | **G** *(currently in Forms pane)* |
| **Print** | Print… | **G** |
| | Imposition… (n-up / booklet / poster) | **C** |
| **Document** | Properties | **G** |
| | Fonts | **G** *(currently View ▸ Panels)* |
| | Security | **N** |
| **pdfcer** | Settings… | **G** |
| | Keyboard shortcuts | **G** |
| | About | **N** |

~~**Moved off this tab:** `Copy this page's text` and `Copy the whole
document's text` go to **Edit ▸ Clipboard** — they are content
operations, not file operations.~~ `Reset layout…` goes to **View ▸
Window** — it resets panel geometry, which is a view concern.

★ **REVERSED by operator decision, 2026-08-14, and corrected here on
2026-08-25.** The two text-copy commands are `file.copy_page_text` and
`file.copy_document_text`, on **File ▸ Export**, and have been since that
date. This paragraph went on saying otherwise for eleven days, which is the
defect: a settled document that disagrees with the build is worse than an
unsettled one, because it is read as authority.

What the sentence above got wrong is that *a content operation is not
necessarily an **authoring** operation*. Copying reads the page and writes to
the clipboard; it cannot change a byte. What made the difference visible was
the chord/mode gate refusing `Ctrl+Shift+C` in Read — a mode whose whole
standard is Acrobat Reader, which copies text. It is the same rule §5.7 states,
and this is the second of its three instances.

**Why Fonts moves here from View:** the Fonts panel answers "what is
inside this file", not "what is on my screen". It sits with Properties
and Security as document-level inspection. This is a genuine
improvement in the current build — Fonts is excellent and nobody will
find it under View ▸ Panels.

**Note on Save.** A `Save` command that overwrites in place cannot ship
before autosave and crash recovery exist; that dependency is already
documented (`FEATURES.md:62`, `main.rs:49-54`). Until then the group
holds `Save a copy…` alone and `Save` does not render at all, per P3 —
it is not greyed out with a tooltip, it is absent.

---

### 5.2 View — *what is on my screen, and how is the page laid out?*

| Group | Commands | |
|---|---|---|
| **Page display** | Single page | **G** *(current behaviour, now named)* |
| | Continuous | **N** |
| | Facing | **N** |
| | Facing continuous | **N** |
| **Render** | Strategy: Whole page · Tiled progressive | **N** |
| | Raster scale ⌄ (quality) | **partial G** — exists as a constant |
| | Settle delay | **partial G** — `ZOOM_SETTLE` is a constant |
| | Thin lines | ✅ **BUILT 2026-09-05** — see View ▸ Display below; it landed there rather than in a Render group, because it is flipped while reading a sheet rather than set once (P2). |
| | Antialias ⌄ (text / vector) | **N** |
| **Rotate view** | Rotate view left / right | **N** |
| **Zoom** | Zoom to selection | **N** |
| | Zoom to region (marquee) | **N** |
| | Actual size · Fit page · Fit width | **G** *(status bar; P1a mirror)* |
| **Display** | Thin lines | ✅ **BUILT 2026-09-05 as `view.line_weights`** — his ask (O137), engine field `RenderOptions::stroke_display` shipped the same day. **Canvas only**: print, preview and every export render real widths, asserted three ways. |
| | Show annotations | **G** |
| | Show points | **G** |
| | Rulers · Grid · Guides | **N** |
| **Panels** | Sidebar ⌄ | **G** |
| | Pages · Objects · Bookmarks · Layers · Signatures · Comments · Forms | **G** |
| **Window** | Read mode | **G** *(keyboard-only today)* |
| | Full screen | **G** *(keyboard-only today)* |
| | Floating panels: Off · Allowed | **N** — *default Allowed* |
| | App initiative: Never · Ask · Allowed | **N** — *default **Never*** |
| | Save workspace… · Load workspace ⌄ | **N** |
| | Reset layout… | **G** *(from File)* |

**Page display is the important entry.** Per your correction: single
page stays the **default**, because paging one drawing sheet at a time
is the right model for drafting review and the existing navigation is
good. Continuous becomes a *mode you choose*, sitting beside it, for the
case where the document is a 40-page specification rather than a sheet
set. The four modes are radio-style — exactly one is active — and the
choice persists per document, not globally, so opening a drawing set
does not inherit a report's setting.

This is a larger build than it looks; `viewer.rs` holds a single
`page_index` and `object_provider.rs:392-399` returns nothing for any
page but the current one. See `GUI_ROADMAP.md` Phase 4.

**The Render group is an operator decision, 2026-08-12.** pdfcer caches
one whole-page texture and scales it with linear filtering during the
150 ms settle. Measured in use on a large drawing, that is *smoother*
to pan and zoom than the comparison product's progressive tile
rendering — no seams, no piece-by-piece fill-in — at the cost of a full
re-raster once motion stops. Those are two legitimate trades, not a
better and a worse, and which one wins depends on the sheet and the
machine.

So the strategy becomes a **choice on the View tab**, with whole-page as
the default because it is what measured better. This is R169 applied to
rendering: where the right answer is genuinely undetermined, pdfcer
states the trade and lets the operator pick, rather than deciding
quietly. `ZOOM_SETTLE` (`main.rs:367`) and the raster-scale multiplier
are constants today and become the two knobs beside it.

**Zoom on this tab does not duplicate the status bar in spirit.** The
status bar keeps the continuous controls a user reaches for constantly
(−/%/+, fit toggles). The View tab adds the two *targeted* zooms that
have no status-bar home — zoom to selection and marquee zoom-to-region —
and mirrors the three named zoom levels under P1a so that a user looking
under View for zoom finds zoom.

---

### 5.3 Pages — *what am I doing to the set of pages?*

| Group | Commands | |
|---|---|---|
| **Insert** | Insert blank | **C** |
| | Insert from file… | **G** *(Tools ▸ Batch pane)* |
| | Insert scan | **N** |
| **Organise** | Delete | **G** *(thumbnail rail)* |
| | Extract… | **G** *(thumbnail rail)* |
| | Replace… | **N** |
| | Move up / Move down | **G** *(thumbnail rail)* |
| | Split… | **G** *(Tools ▸ Batch pane)* |
| | Merge into this document… | **G** *(Tools ▸ Batch pane)* |
| **Transform** | Rotate left / right | **G** *(Edit ▸ Pages)* |
| | Crop… | **N** |
| | Resize… | **N** |
| **Stamp** | Watermark… | **N** |
| | Header & footer… | **N** |
| | Bates numbering… | **N** — *see `DEFECTS.md` §2* |

Every command here operates on **the current document's page set**, and
every one of them respects the thumbnail rail's current selection when
there is one. That is the tab's organising rule and it is what
distinguishes it from Tools: Pages changes *this* document, Tools
produces *new* files.

The thumbnail rail keeps its selection action bar. That is not a P1
violation — the rail is a panel, not a tab, and a selection-scoped
action bar next to the selection is correct. But the ribbon becomes the
discoverable path, and the rail becomes the fast path.

---

### 5.4 Edit — *what am I changing about content that is already there?*

| Group | Commands | |
|---|---|---|
| **Content** | Edit text | **G** — *relabel from `Aa`* |
| | Add text | **G** — *relabel from `I⁺ Aa`* |
| | Edit objects | **G** — *relabel from `Obj`* |
| **Insert** | Image… | **G** *(drag-drop only today)* |
| | Shape ⌄ | **N** |
| **Arrange** | Align ⌄ · Distribute ⌄ | **N** |
| | Bring forward / Send backward | **N** |
| | Group / Ungroup | **N** |
| | Flip horizontal / vertical | **N** |
| **Clipboard** | Cut · Copy · Paste · Paste in place | **N** *(object clipboard)* |
| | ~~Copy page text · Copy document text~~ | ★ **NOT HERE — `file.copy_*`, on File ▸ Export.** Operator decision 2026-08-14; see §5.1. *Copying is not authoring* |
| **Forms** | ~~Fill form~~ | ★ **NOT HERE — `view.panel_forms`, on View ▸ Panels.** Operator decision 2026-08-14: Read fills forms, as Acrobat Reader does, and Read is shown File and View alone. It also stopped being a verb and became a **panel toggle**, which is why it sits with the other panel toggles rather than in a group of its own. *Filling is not authoring* — Edit ▸ Forms keeps create, manage and flatten |
| | Create field ⌄ | **G** |
| | Manage fields | **G** |
| | Flatten | **G** *(in Forms pane)* |
| **Protect** | Redact ⌄ (mark page / by text / by pattern) | **G** |
| | Apply redactions | **G** |
| | Sanitise… | **N** |

**The three `Content` buttons must get real labels.** `Aa`, `I⁺ Aa` and
`Obj` are the primary editing tools and are currently the least legible
controls in the application. `Edit text`, `Add text`, `Edit objects` —
with the icons kept.

**The `Editing on` master toggle is removed. Operator decision,
2026-08-12: "make it work the same way other programs do."**

No mainstream editor has a global editing switch. Acrobat, Bluebeam,
Word and Illustrator all work the same way: selection and Delete are
always live, and picking a tool arms *that tool* until you press Escape
or pick another. There is no state in which a click does nothing without
the application saying so.

Concretely:

- `editing_enabled` (`main.rs:3235`, `3624`) is deleted, along with the
  ribbon toggle at `ribbon_ui.rs:721-736` and
  `ui_text::authoring_disabled_note()`.
- The four sites that currently gate on it — `main.rs:7095`, `8169`,
  `8194`, `16920` — lose the gate. Nothing replaces it: an unarmed
  canvas already does modeless select-and-delete, and every authoring
  gesture already requires its tool to be armed.
- **This supersedes `DEFECTS.md` D6.** That defect was "review mode does
  not actually block object deletion". With review mode gone there is
  nothing to enforce, so D6's fix becomes "delete the check sites"
  rather than "add the missing one". If a genuine read-only mode is ever
  wanted it should be a *document* state (open read-only, encrypted, or
  signature-locked) with a visible badge, not a hidden global toggle.

It was already `true` by default, and it is **not** what breaks
click-then-Delete (`DEFECTS.md` D1). Removing it is about eliminating a
class of failure, not fixing a current one.

---

### 5.5 Markup — *what am I adding for someone else to read?*

| Group | Commands | |
|---|---|---|
| **Shapes** | Rectangle · Ellipse · Line · Arrow | **G** *(Arrow is `Arrow line`)* |
| | Polyline · Polygon | **N** |
| | Cloud | **N** — revision clouds; AEC table stakes |
| | Ink (freehand) | **N** |
| **Text markup** | Highlight | **G** *(`Highlight band`)* |
| | Underline · Strikeout · Squiggly | **N** |
| **Notes** | Text box | **G** |
| | Sticky note | **G** |
| | Callout | **N** |
| | Stamp ⌄ | **G** *(`Draft stamp`; needs a gallery)* |
| **Style** | Colour · Line width · Fill · Opacity | **partial G** — colour only |
| **Comments** | Comments panel | **G** |
| | Clear page · Clear all | **N** |

Six of ten markup kinds are missing (`canvas.rs:255-262` defers Ink,
Polygon, PolyLine, Underline, StrikeOut, Squiggly to "slice 3"). Cloud
is not in that list and matters most for this audience.

**The `Style` group sets defaults for the next markup.** Changing an
*existing* markup's style happens on the contextual **Format** tab. Both
must exist; today only the first does, which is why a placed markup
feels final.

---

### 5.6 Measure — *what am I measuring, and in what units?*

| Group | Commands | |
|---|---|---|
| **ce dimension** | Linear | **G** |
| | Aligned | **partial G** — constraint exists, not a separate tool |
| | Angular | **N** |
| | Radius / Diameter | **G** |
| | Two-line | **C** — *core + CLI done; the gesture exists in the OLD shell and must be salvaged (see the correction below)* |
| **Quantity** | Distance · Perimeter · Area | **N** |
| | Count | **N** |
| **Scale** | Set scale | **G** |
| | Calibrate from a known length | **partial G** |
| | Manage dimension groups… | **G** |
| **Takeoff** | Schedule panel · Export CSV | **N** |

The **Scale ▸ group** model — named groups carrying a shared scale and
drafting standard — is genuinely better than what the comparison product
does, and nothing here should dilute it. Area and Angular are the
conspicuous absences for anyone doing takeoff on a drawing.

**Two-line dimensioning is a C row and is at the top of the project's
own queue** — core and CLI shipped and measured. It is a shell-only task.

> **★ Correction, 2026-08-14.** This paragraph used to end *"`pick_line`
> has no caller"*, and the same sentence was in `FEATURES.md`,
> `HANDOFF.md`, `SALVAGE.md` and `shell/manifest/mod.rs`'s `PLANNED`
> entry. **It is false, and it was false when written.** The old shell
> calls `pick_line_in_page` at `D:\Dev\pdfce\crates\pdfce-gui\src\main.rs:23564`
> and takes the pick at `:23592`; the whole gesture is there — hover
> highlight, picked-pair overlay, the verdict disclosure, Escape to clear
> — with `TwoLinePick` in `measure_tool.rs:361` behind 814 lines of tests,
> and pdfcer's own `docs/FEATURES.md` marks the row `gui [x]`.
>
> The likely origin is a misread of pdfcer's `ROADMAP.md:2778`, which
> explains *why* `pick_line_in_page` exists, one paragraph above the
> commit that added its caller.
>
> What changes: the caller that is missing is **ours**. The new shell has
> no measure tool at all — `CanvasTool` has two variants, no `measure.*`
> command has a dispatch arm, and nothing in `crates/pdfcer-gui` mentions
> `linepick`. So this is a **salvage** of `measure_tool.rs` (Class A,
> 1,230 lines) plus the canvas hosting, not a one-line hookup, and the
> "cheapest real feature in the backlog" claim that rode on it is
> withdrawn. Recorded rather than deleted, because a wrong claim repeated
> in five documents is worth knowing about.

---

### 5.7 Tools — *what do I run across files, or configure once?*

| Group | Commands | |
|---|---|---|
| **Batch** | Merge files… | **G** |
| | Split files… | **G** |
| | Batch print… | **N** |
| **Compare** | Compare documents… | **N** |
| **Fonts** | Font folders… | **G** |
| | Embed fonts · Unembed fonts | **G** *(Fonts pane)* |
| ~~**Recognise**~~ | ~~OCR…~~ | ★ **MOVED to File ▸ Recognise, 2026-08-14 — `file.ocr`.** This section specified Tools, and Tools is **not in Read's tab list** (`["file", "view"]`). The operator's instruction was that OCR *be available in Read*, so shipping it here would have satisfied this document and broken the instruction. **This is the third time the same pattern has decided a tab**, and it is now a rule rather than a coincidence: *a command refused in a mode where the operator plainly needs it is evidence that the command's tab is wrong, not that the mode gate needs an exception.* It moved `edit.form_fill` → `view.panel_forms` (filling is not authoring), `edit.copy_*` → `file.copy_*` (copying is not authoring), and now `tools.ocr` → `file.ocr`. The alternative — giving Read the **Tools** tab — was refused because it would hand a reading stance batch merge, batch split and font embedding. **File over View** because OCR's product is a new file, which is what the File tab is for. |
| **Validate** | PDF/A validate & convert… | **N** — *see `DEFECTS.md` §2* |
| | Optimise… | **N** |
| **Diagnostics** | Render diagnostics | **G** *(status bar; belongs here)* |

Tools is now the tab for things that either operate on files other than
the open one, or are configured once and rarely touched. Redact moved to
Edit ▸ Protect, where a user editing a document will actually look for
it.

---

### 5.8 Format — *contextual*

Appears only while something is selected; disappears on deselect.
Contents vary by selection type:

| Selection | Groups |
|---|---|
| Markup | Colour · Fill · Line width · Line style · Opacity · Arrowheads · Note text · Delete |
| ce dimension | Group · Scale · Precision · Units · Standard · Witness lines · Delete |
| Image | Size · Position · Crop · Opacity · Replace · Delete |
| Vector object | Stroke · Fill · Winding rule · Node tools · Delete |
| Text run | Font · Size · **Bold · Italic** · Colour · ~~Spacing · Alignment~~ · Delete — ★ **BUILT 2026-08-27, see the amendment below** |
| Pages (rail) | Rotate · Delete · Extract · Move |

This tab is the single largest usability change proposed here. It is
also what makes selection *mean* something — right now, selecting an
object gives you an object-tree row and no way to act on it.

### Both surfaces, not one — operator decision, 2026-08-12

The contextual tab and a **persistent properties panel** both ship. They
are not redundant; they answer different questions.

| | Format tab | Properties panel |
|---|---|---|
| Lives | in the ribbon, appears on selection | right dock, always available |
| Holds | the edits reached for mid-gesture | the complete property set |
| Survives a tab switch | no | yes |
| Costs | nothing when nothing is selected | ~200 px of width |
| Discoverable | high — it appears, which is itself the affordance | medium |

The division of labour: the **tab** carries what a user changes *while
working* — colour, width, style, align, delete. The **panel** carries
everything, including the read-only facts (winding rule, node count,
embedded-font status, exact geometry) that belong beside the Objects
panel's inventory rather than in a ribbon band.

The panel is also where the **editable geometry** lives — X, Y, W, H as
typed values. That is the surface through which `/Rect` move-and-resize
becomes reachable without a drag, which matters for drafting work where
the number is known and the mouse is imprecise.

Build order: **panel first, tab second.** The panel is the harder half
and the tab's contents are a subset of it, so building the tab first
would mean writing the property editors twice.

### ★★★ Amendment, 2026-08-27 — the Font group, and three changes to the row above

**Operator instruction:** O37, *"We should also have all the font tools
available that Word does"*, followed by *"make the text tool discoverable,
then the Format tab's Font group."* This section is settled and is not
improvised around; the amendment is recorded here because the operator
directed it, and it is written up rather than applied silently.

The **Text run** row shipped as one captioned band called **Font**, placed
**first on the tab**, ahead of Selection. Three departures from the row as it
was written, each with its reason:

**1. Bold and Italic were added.** The row did not name them and Word's Font
group is unusable without them; they are also, literally, the buttons the
operator pressed. They pass the build-order test — the Properties panel's
*This text* section already had them — so the tab's contents remain a *subset*
of the panel's, which is the constraint the build order exists to protect.

**2. Spacing and Alignment were dropped, and it is not a scheduling deferral.**
`EditSession` has no verb for either. `format_text` sets face, size, weight and
fill and nothing else, so nothing writes `Tc`, `Tw` or `TL` for an existing
run. Alignment is worse than missing: it is not a property a PDF text run
*has*, it is a consequence of where each show operator was positioned, so
re-aligning existing text means re-laying it out. Both stay in
`shell::manifest::PLANNED` with those reasons written in, replacing the earlier
"panel first" note that read as a scheduling fact and was not one.

**3. Grow and Shrink were considered and refused**, though Word has them, for
the build order's own reason: they exist in no panel section, so adding them
here would make the tab a **superset** of the panel — which is exactly the
"writing the editors twice" that the build order is written to prevent.

**Order within the group** is Word's: face, size, a rule, then Bold, Italic,
colour. The rule separates *which typeface* from *how it is set*.

**The group is first on the tab** because every row of the table above ends in
Delete, so reading left to right goes "change how this looks", then "describe
it", then "destroy it" — increasing commitment, which is the ordering rule the
Selection group already follows internally.

**Two conditions carry it**, and the split is R9 rather than convenience:
`mode.edit_content` is each item's `visible_when`, so the whole band is
**absent** in Read and Review, which cannot change page content at all;
`selection.text` is each command's `enabled_when`, so inside Edit the controls
**grey** until something is swept, with a tooltip that says how to sweep. That
greyed state is deliberate and is the answer to O37's own admission that
nothing on screen told an operator to press `T`.

**The tab's `visible_when` changed** from `selection.any` to
`selection.formattable` — the union of an object selection and a live text
selection. The tab now has two kinds of subject and neither operand is its
question.

A third surface, the **context menu**, carries the same commands again
for the user who right-clicks. That is not duplication in the P1 sense —
context menus are not tabs — and it is the path most users try after the
keyboard.

---

## 6. What deliberately does not go on the ribbon

**Quick Access Toolbar** — Open, Save a copy, Undo, Redo. Unchanged from
today, now understood as a P1a shortcut surface rather than the only
home for those commands.

**Status bar** — Find toggle, actual size, fit width, fit page, zoom
−/%/+, page ◀ n/N ▶, and a **new editable page-number box**. These are
the controls a user touches constantly; they belong where they never
disappear behind a tab change. The current render-diagnostics text moves
behind a disclosure (see `DEFECTS.md` §5).

**Context menus** — currently zero in the entire crate
(`grep context_menu` → no hits). Every selection type above needs one,
carrying the same commands as its Format tab section plus Cut/Copy/
Paste/Delete. This is not a ribbon question, but it is the other half of
making selection meaningful, and no amount of ribbon design substitutes
for it.

**Tool Options pane** — per P2, every armed tool's parameters. Unchanged
in principle; the pane needs the layout work described in the roadmap.

### ★★★ Amendment, 2026-09-04 — a fourth surface: the TRAILING region

`OPERATOR_REQUESTS.md` **O122**, the operator: *"beside our read-review-edit
buttons at the top there should be an open in acrobat button."*

This document specified three surfaces — the ribbon, the QAT and the status bar
— and the tab-strip row had three regions: QAT on the left, tabs in the middle,
the mode selector on the right. There was no way to put a control **past** the
mode selector, which is where he asked for one. So there is now a fourth
region, and it is a first-class part of the shell rather than a special case:

```text
[QAT] │ File View Pages ⏷ 2 more   ( Read │ Review │ Edit )   [Acrobat]
                                                              └ trailing ┘
```

**What may go here, and it is a narrow set.** The QAT is *"the handful of
controls that must never sit behind a tab switch"* — verbs used continuously,
on the left, where reading starts. This is the opposite end of the row, read
last, and the rule that follows is:

> A trailing control **ends the current activity** rather than advancing it.

*Open in Acrobat* qualifies exactly: it closes the document and hands the file
to another program. A control an operator reaches for during their work does
not belong here and belongs on a tab, or on the QAT if it is used all day.

**P1 is not weakened.** A trailing control may appear here **and** on a tab, on
the same amendment P1a already grants the QAT and the status bar: *a shortcut
to a known home is not a second place to hunt.* `Shell::validate`'s
one-command-one-tab rule walks tabs only, so nothing here relaxes it.

★ `file.open_in_acrobat` is nonetheless on **no** tab, which is the first
command in this manifest of which that is true. Its home is a fixed position in
chrome that is visible in every mode, which is a *stronger* discoverability
guarantee than a tab gives — and putting it on File as well would give two
places to press for one act, one of which comes and goes with what is installed.

★★ **P3 applies here with full force and is the reason the region is data
rather than a callback.** The control is absent on a machine with no Acrobat —
not greyed — through `visible_when` on the manifest item, which is the shell's
own R9 mechanism. The region holds `Item`s, resolved through the command
registry, precisely so that nothing can draw a control here that has no command
behind it.

★ It is also the one reserved region on that row that may be **squeezed out**.
The QAT, the mode selector and the two overflow affordances are promises the
interface has already made; a trailing control's absence is a state the
application already handles, so on a very narrow window it is dropped whole
(never a sliver) and the drop is disclosed as `ribbon-trailing-dropped`.

---

## 7. Migration map

Every command that exists in the GUI today, and where it goes. Nothing
is dropped.

| Today | Proposed |
|---|---|
| QAT: Open, Save a copy, Undo, Redo | QAT *(unchanged)* + File ▸ File/Save |
| File ▸ Document ▸ Properties | File ▸ Document |
| File ▸ Clipboard ▸ Copy page/document text | **File ▸ Export** — *corrected 2026-08-25; this row said Edit ▸ Clipboard, which has not been true since 2026-08-14. See §5.1* |
| File ▸ Export ▸ Export DXF | File ▸ Export |
| File ▸ Print ▸ Print | File ▸ Print |
| File ▸ Layout ▸ Reset layout | **View ▸ Window** |
| File ▸ Settings ▸ Settings | File ▸ pdfcer |
| File ▸ Help ▸ Keyboard shortcuts | File ▸ pdfcer |
| Edit ▸ Pages ▸ Rotate left/right | **Pages ▸ Transform** |
| Edit ▸ ContentTools ▸ Editing on | Edit ▸ Mode |
| Edit ▸ ContentTools ▸ Aa / I⁺ Aa / Obj | Edit ▸ Content *(relabelled)* |
| Edit ▸ Forms ▸ Fill Form | **View ▸ Panels** — *corrected 2026-08-25; see §5.4* |
| Edit ▸ BuildForm ▸ Create Field | Edit ▸ Forms |
| Review ▸ Markup ▸ Rectangle/Ellipse/Arrow line/Highlight band + Colour | **Markup ▸ Shapes / Text markup / Style** |
| Review ▸ Comments ▸ Comments | Markup ▸ Comments |
| Review ▸ Notes ▸ Text box / Sticky note / Draft stamp | Markup ▸ Notes |
| Measure ▸ Measure ▸ Linear / Radius-Diameter | Measure ▸ ce dimension |
| Measure ▸ Measure ▸ Set Group Scale / Manage ce-dimension Groups | Measure ▸ Scale |
| Tools ▸ Protect ▸ Redact | **Edit ▸ Protect** |
| Tools ▸ Across files ▸ Tools *(batch pane)* | **Pages ▸ Insert/Organise** + Tools ▸ Batch |
| Tools ▸ Font folders ▸ Font folders | Tools ▸ Fonts |
| View ▸ Panels ▸ Sidebar / Bookmarks / Layers / Signatures / Objects | View ▸ Panels |
| View ▸ Panels ▸ Fonts | **File ▸ Document** |
| View ▸ Show ▸ annotations / points | View ▸ Display |

Three commands change tab in a way a returning user will notice:
`Copy text` (File → Edit), `Redact` (Tools → Edit), and `Rotate page`
(Edit → Pages). All three are moving *toward* where a first-time user
would look, which is the trade this proposal accepts.

---

## 8. Questions — answered and outstanding

### Answered 2026-08-12

**`Editing on` master toggle** → *"make it work the same way other
programs do."* Removed entirely. See §5.4.

**Format tab vs. properties panel** → **both**, panel first. See §5.8.

**Page display** → single page stays the **default**; continuous,
facing and facing-continuous are modes chosen on the View tab, persisted
per document. See §5.2.

**Rendering** → whole-page raster stays the default because it measured
better in use; tiled progressive becomes an opt-in, with raster scale
and settle delay exposed beside it. New **View ▸ Render** group. See
§5.2.

### Still open

1. **Save semantics** — is autosave + true in-place `Save` wanted in the
   next year, or is `Save a copy` the permanent model? The File tab
   layout differs.
2. **Compare** — worth building, or out of scope? It is the one absence
   an AEC reviewer will name first, and it is a large build.
3. **Multi-run text editing** (`GUI_ROADMAP.md` Phase 5d) — is "edit one
   run at a time, clearly disclosed" an acceptable resting state for
   another year, or is this the thing that has to be right?
