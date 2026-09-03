# RIBBON_SCALING.md — how a ribbon narrows, learned from Word by driving it

**Written 2026-08-24.** `OPERATOR_REQUESTS.md` O31.

> *"can you improve the ribbon bar? if you can learn how word handles when to
> have text labels, organization on two rows for some commands, and how it
> handles narrowing the window. for one thing it puts an arrow at the end to
> press to move over if there isn't room for all commands. also we should have
> flexibility to show or hide and commands and shift the space used depending
> on what exists."*

This document is the **evidence** and the **design**. It is not a proposal
about where commands live — `RIBBON_IA.md` owns that and is settled. It is
entirely about **presentation and scaling**: how wide a control is, how a group
gives up space, and what happens when there is not enough.

---

## 1. How the evidence was gathered

Word's ribbon scaling rules are **not in its object model**. `CommandBars` is
the legacy 2003 toolbar surface and says nothing about the ribbon; the ribbon
is RibbonX XML compiled into the product, and the scaling behaviour lives
inside the Office UI framework and is exposed nowhere.

So the instrument is the one this project already treats as authoritative for
any layout question: **a photograph**. `tools/word-ribbon-study.ps1` sets a
window width, waits for the re-layout, and captures — twelve widths from
1,884 down to 444 client points, largest first, because Word re-lays-out
incrementally and a series that *grew* would photograph the recovery path
rather than the collapse path. `tools/our-ribbon-study.ps1` is its twin,
pointed at `target/release/pdfcer-gui.exe`, at the same widths.

The two series are in `evidence/word-ribbon/` and `evidence/our-ribbon/`.

★ Both scripts had to find the window the same careful way. `Process.MainWindowHandle`
is *not* the visible window for a `winit` application — it can be an invisible
helper whose rect is nonsense, and sizing it succeeds, reports the size back,
and moves nothing on screen. The first run of the pdfcer study photographed the
same width six times because of it. `ui-verify`'s own win32 layer already
carried that comment; the study did not, until it did.

---

## 2. ★★★ The measurement that decides the work

Groups reachable **on the band**, without opening a menu:

| client width | Word | pdfcer |
|---:|---:|---:|
| 1,884 | 10 | 7 (all of them) |
| 1,284 | 10 | 7 |
| 884 | **10** | **3** — four in a `⏷ 4 more` menu |
| 604 | **7** + a scroll chevron | **1** — six in a `⏷ 6 more` menu |

At the width a laptop or a docked half-screen actually is, Word puts ten
groups' worth of commands in front of the operator and pdfcer puts three.

**It is not that our overflow is wrong.** The `⏷ N more` affordance is good,
it is reachable, it is tested at every width, and it is the arrow the operator
is describing. The problem is that it starts eating groups far too early,
because our controls are two to four times wider than they need to be and a
group has no way to give up space except to vanish entirely.

---

## 3. What Word actually does — three mechanisms, in order

### 3.1 Item sizes

Word has **three** presentations for a control, and a group mixes them freely:

| size | form | example |
|---|---|---|
| **Large** | icon **above** label, label may wrap to two lines, spans the whole band height | Paste, Dictate, Editor, the three Acrobat buttons |
| **Medium** | small icon, label to the **right**; three stack vertically | Find / Replace / Select in *Editing* |
| **Small** | **icon only**, optional dropdown chevron | B, I, U, the alignment buttons, the list buttons |

The Clipboard group is one Large plus a column of three Small. The Font group
is two combo boxes and thirteen Small. The Editing group is three Medium.

**This is where Word's density comes from**, and it is the mechanism pdfcer
lacks entirely: every control in our band is Medium, always, by a hard-coded
`shows_label: true`.

### 3.2 Per-group collapse, in a declared order

As the window narrows, groups do **not** disappear. Each one in turn collapses
to a **single Large button bearing the group's caption and a chevron**;
pressing it opens the group's full layout in a popup. Observed order on the
Home tab, as the width fell:

```
1500  Editing, Custom collapse.  Styles gallery 4 tiles -> 3.
1300  Styles, Adobe Acrobat collapse.
 900  Paragraph collapses.
 620  Font collapses.  Clipboard never does.
```

Two things to take from that. It is **not** simply right-to-left — Editing
collapsed before Styles, which is to its left. And **Clipboard never
collapses**: the group the author considers most important keeps its full
layout at every width. So the order is **authored**, not derived.

### 3.3 Scrolling, only when nothing can shrink further

When every group is at its smallest and the row still does not fit, a `›`
chevron appears at the right-hand end and **scrolls the group strip**. The tab
strip has its own, separate `›`, and tab labels **clip** rather than
ellipsise.

★ So the operator's *"arrow at the end to press to move over"* is the **last**
resort in Word, not the first. In pdfcer it is currently the **only** resort.

---

## 4. What pdfcer already has, and it is more than it looks

Worth stating plainly so the work is additive rather than a rewrite:

* a `⏷ N more` overflow with the group's **full layout** in the menu, captioned
  — the same idea as Word's collapsed-group popup;
* **two-row wrapping** inside a group, with an optimal contiguous split that
  minimises the widest row;
* a separate **tab-strip overflow** that pins the active tab;
* a **fixed band height** whatever the tab contains — R128, and it is load
  bearing next to fit-to-viewport zoom;
* captions on a **shared baseline** across groups of different row counts;
* an **unregistered command loses one control, not the band** — which is
  already half of what the operator is asking for in his second paragraph.

---

## 5. The design

### 5.1 Item size — `ItemSize { Small, Medium, Large }`

Declared per item in the manifest, defaulting to **Medium**, which is exactly
today's presentation. A manifest that says nothing renders identically to the
one before the change — which is what makes this safe to land in one go.

* **Medium** — icon, gap, label. Unchanged.
* **Small** — icon only. ★ **Earned, not asserted**: a control may go icon-only
  only when it names an icon, carries a tooltip, *and* a painter is installed —
  the identical rule `ribbon::qat::shows_label` already applies and already
  tests. The tooltip is the icon's accessible name; without one, an icon-only
  button is an unlabelled rectangle to a screen reader and a guess to
  everybody else. A `Small` that has not earned it **falls back to Medium**
  rather than rendering a mystery.
* **Large** — icon above label, spanning the band's rows. Its width is the
  wider of the icon and the label, so a long label makes a wide button; the
  label wraps to a second line before it does that.

### 5.2 Group collapse — a ladder, authored

Each group gains an optional **collapse priority**. On layout: measure every
group at its full presentation; while the row does not fit, collapse the
next group in priority order to its single captioned button; when every
group that *may* collapse has collapsed and it still does not fit, fall back
to today's `⏷ N more`.

★ A group may declare that it **never** collapses — Word's Clipboard. On a
document application that is the group carrying the verb the operator came to
the tab for.

### 5.3 Per-item visibility — `visible_when`

An item gains an optional condition name, evaluated against the same
`ConditionSet` that already decides *enablement*. A hidden item is removed
**before measurement**, so its space is reclaimed and the group re-flows; a
group with nothing visible left is not drawn at all, and its separator goes
with it.

This is the operator's second paragraph, and it is what lets one tab
definition serve Read, Review and Edit with different contents rather than
three near-identical tabs. It is also exactly R9 — *an unavailable capability
renders nothing* — extended from "the command was never registered" to "the
command does not apply here".

---

---

## 6. Staging

| stage | what | why in this order |
|---|---|---|
| **S1** | item sizes | The single change that moves the 884-point number. Nothing else needs it first, and every stage after it benefits from the narrower controls |
| **S2** | per-item `visible_when` | Small, and independent of S1. The operator's second ask |
| **S3** ✅ **DONE 2026-08-25** | the collapse ladder | The largest, and the one that touches `plan_band`'s invariants — `the_visible_groups_are_a_prefix_and_nothing_is_lost` and `widening_the_band_never_hides_a_group_that_was_visible` both have to be restated for a world where a group can shrink instead of vanishing |
| **S4** | scroll instead of a menu at the bottom of the ladder | Only worth having once S3 exists; until then the menu *is* the right last resort |

★ S1 and S2 were in the first change. **S3 landed 2026-08-25**; S4 is designed
here and not built.

### S3 as built, measured on the File tab

`plan::collapse` runs **before** `plan_band`, so a group gives up its rows
before anything is pushed into a menu. Priorities live in one table
(`shell::manifest::ladder`) rather than on each group, because a collapse
priority is a *ranking of groups against each other* and a ranking can only be
reviewed all at once.

Measured offscreen against the shipped build, `SW41177.pdf`, File tab:

| window width | groups collapsed | `⏷ N more` dropdown |
|---:|---:|---|
| 1600 | 1 | **no** (before this change: yes) |
| 1400 | 2 | no |
| 1200 | 3 | **no** (before: yes) |
| 1100 | 4 | no |
| 1050 | 4 | yes |

★ **The headline is the last column.** The dropdown used to appear at 1600 and
at every width below it. It now appears only below about 1100. S4 — the scroll
arrow that replaces it — is therefore a *rare-case* affordance rather than the
everyday one, which changes the argument for building it: the menu names what
is hidden and an arrow does not, so replacing a discoverable affordance that
almost never appears is a smaller win than it looked before S3 was measured.
**Operator decision, not an engineering one.**

---

## 6a. S5 ✅ DONE 2026-08-25 — groups re-wrap onto MORE rows as the window narrows

**Operator instruction, 2026-08-25**, and a correction of this document:

> *"put it in the plan to update so that tools within sections will re-wrap
> onto more rows when I resize... BTW the Font section in Word will wrap tools
> onto 3 lines when the window is narrowed enough, and other tools wrap in a
> similar way too."*

### ★★★ The correction, and how the error was made

Until today both this document and `plan::collapse`'s header asserted that
**Word does not re-wrap groups by window width**. That is **false**, and the
evidence disproving it was already in `evidence/word-ribbon/` when the claim
was written:

| photograph | Font group |
|---|---|
| `ribbon-1900.png` | **2 rows** — name + size + case + clear on row 1, B/I/U and the rest on row 2 |
| `ribbon-1000.png` | **3 rows** — name + size alone on row 1, B/I/U on row 2, colour/case/size-step on row 3 |

The widths compared when the claim was made were **1300 and 800**, and by 800
the group had already *collapsed* — so the reflow between them appears in
neither frame. A twelve-frame series was taken and three frames were read.

**The lesson is not "look harder".** It is that *sampling either side of a
transition and concluding there is no transition* is a specific, repeatable
error, and the guard against it is to walk the series rather than to pick
endpoints — the same discipline the collapse ladder's own monotonicity test
already applies, where it sweeps 600 widths instead of checking three.

### What has to change

| today | S5 |
|---|---|
| `GROUP_ROWS` is a constant **2** | rows become a **range**, 1..=3 |
| `GROUP_WRAP_WIDTH` is a fixed 440 pt trigger on the group's own content | the row count is chosen from the width the band can actually offer the group |
| `wrap_group` splits into exactly `GROUP_ROWS` rows | splits into the fewest rows that fit the offered width, capped at 3 |

### As built, measured on the File tab

| window width | Export group | groups collapsed | scroll arrow |
|---:|---:|---:|---|
| 2200 | **2 rows** | 0 | no |
| 1600 | 3 rows | 1 | no |
| 1200 | **3 rows** | 3 | no |
| 1000 | — | 4 | **yes** |

★★ **And the band's height is identical at every one of them** — the group box
is `47 .. 140` at 1200 and at 1600, and the document tab row begins at 144 in
both. That is the property the whole design is arranged around, and it is why
the third row needed more than a bigger constant. See below.

★ **Band height must not move.** R128 governs: the ribbon sits directly above
the canvas and a band that changed height would move the canvas under a
fit-to-page zoom, which is the feedback loop this project has already paid for
three times. Word's band is a fixed height at every width in the series, and it
achieves 3 rows by stacking *small* controls where 2 rows held *medium* ones.
So the height stays derived from the theme, and the row count is a property of
what is packed into it — never the other way round.

★★ **S5 goes BEFORE S3 in the ladder, not after.** A group that can reflow
needs collapsing less often, so re-wrapping first strictly reduces how often a
group loses its labels. The full ladder becomes:

1. item sizes (S1, done)
2. per-item visibility (S2, done)
3. **re-wrap to more rows (S5)** ← new, and first among the width responses
4. collapse whole groups in authored order (S3, done)
5. scroll (S4)

### Invariant to restate again

`widening_the_band_never_hides_a_group_that_was_visible` gains a third rung:
**widening never increases a group's row count.** Same construction as S3 —
compute from the width alone, never from the previous frame — and the same
sweep test rather than spot checks.

---

## 6b. S4 ✅ DONE 2026-08-25 — the scroll arrow

**Operator instruction, 2026-08-25:** *"do the scroll like Word."*

Settled, no longer a question. Word's `›` appears at the right end of the band
at 460 pt (`ribbon-0460.png`) and shifts the band horizontally; it is the
**last** resort, after collapsing, and it replaces the `⏷ N more` dropdown
rather than joining it. Two affordances for one job is a defect.

**Known cost, to be paid deliberately:** six unit tests and one driven check
(`print_dialog_reaches_the_spooler`, which reaches Print *through* the overflow
menu) assume the dropdown exists. They are rewritten as part of S4, not
worked around.

**A note on ordering:** S3 moved the dropdown's first appearance from 1600 pt
down to below 1100. S5 will push it lower still. So S4 should be built
**after** S5, when the width at which it appears is known and stable — building
it now would mean tuning a scroll step against a threshold that is about to
move.

---

## 6c. Font tools — parity with Word

**Operator instruction, 2026-08-25:** *"We should also have all the font tools
available that Word does."*

Scoped separately from the scaling work because it is an **IA and capability**
question, not a layout one: it adds commands, and `RIBBON_IA.md` decides where
a command lives. Tracked in `OPERATOR_REQUESTS.md`; it needs, in order:

1. an inventory of Word's Home ▸ Font group against what `pdfcer-core`'s text
   editing can actually do — a control for a capability the engine lacks is a
   placeholder, and R9 forbids those;
2. anything missing written up as an engine hand-off rather than stubbed;
3. an IA amendment, because pdfcer's text lives under **Edit ▸ Content** and the
   contextual **Format** tab, not under a Home tab that does not exist here.

★ The parity target is the **capability list**, not the pixel layout. Word's
Font group is two combos and fourteen icon buttons; copying that arrangement
onto a PDF editor whose selection model is different would be cargo cult. What
is owed is *"everything Word lets me do to text, pdfcer lets me do to text"*.

---

## 7. What was deliberately not copied

* **Galleries that shrink by showing fewer tiles.** Word's Styles gallery drops
  from four visible tiles to three to one. pdfcer has no galleries, and adding a
  shrinking one to carry a scaling mechanism would be inventing a control to
  justify a rule.
* **Tab labels that clip without an ellipsis.** Ours truncate with `…`, which
  is better, and the tab-strip overflow already handles the rest.
* **Three body rows.** `GROUP_ROWS = 2` carries an R128 argument about
  feedback loops with fit-to-viewport zoom, and a Large button spanning two
  rows is a perfectly good Large button.

---

## 8. What S1 and S2 actually did — measured, 2026-08-24

The same camera, the same widths, after the change. `evidence/our-ribbon-after/`
is the File tab and `evidence/our-ribbon-view/` is the View tab.

**The View tab at full width**, which is where the icon clusters are:

| group | before | after |
|---|---|---|
| Page display | 4 labelled buttons | **4 icon-only** |
| Navigate | 4 labelled buttons | **4 icon-only** |
| Display | 5 labelled toggles | **5 icon-only** |
| Zoom, Panels, Window | labelled | unchanged |

Groups on the band at 884 client points:

| tab | before | after |
|---|---|---|
| View | *(not photographed before)* | **4** of 6, two in the menu |
| File | 3 of 7 | **3 of 7 — unchanged** |

★★ **The File tab is unchanged on purpose, and that is the finding worth
keeping.** Its commands are *named things* — "Export form data…", "Save a
copy…", "Keyboard shortcuts" — not iconic ones. An icon-only "Export DXF"
beside an icon-only "Export form data" is two mystery glyphs, and the original
argument in `band.rs` was exactly right about that case:

> *"icon-only belongs to the QAT … in the band there are forty and the label is
> the only thing that makes one findable."*

What driving Word showed is that the argument is about **the command**, not
about the band. `B`, `I`, `U`, the four page displays and the two page
rotations are findable by shape and position; `Export form data…` is not. So
the sizes are declared per item, in the manifest, and the tabs full of named
verbs keep their names.

★ Two sizes are also *earned* rather than asserted, which is what lets the
manifest be bold: a `Small` whose command has no icon, no tooltip, or no
installed painter renders labelled instead of rendering a mystery. Twelve of
pdfcer's 109 commands have no icon; none of them had to be audited.

**`Large` was applied only to groups with exactly one item** — Recognise,
Print, Insert, Comments, Diagnostics, Insert image. `sizing`'s layout rule is
that Large items *lead* their group, so marking one item of a multi-item group
Large would reorder it, and the order is `RIBBON_IA.md`'s to decide, not this
change's. In a one-item group, leading is a no-op.
