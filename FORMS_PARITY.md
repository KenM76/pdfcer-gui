# Form fields — the complete feature table, and what is missing from each tool

**What this answers.** O205, verbatim: *"make yourself a table of ALL the
features that should be available to match Acrobat, then note everything that is
missing from each tool."* And O206: *"whenever I ask for a feature, what I
really mean is that feature and all the options that would normally exist for
that feature"* — §7 turns that sentence into a lookup you can actually use
before starting work.

**Three columns, and why each exists.**

| Column | What it is | Where it comes from |
|---|---|---|
| **Acrobat** | What the operator expects, because it is what the reference product offers | Acrobat's own dialog strings, read out of `AcroForm.api` and `AcrobatRes.dll` — `ACRO:` citations |
| **Engine** | What `pdfcer-core` can do at the pinned sha. The ceiling on any shell work | `ENGINE:` citations, measured against `git show HEAD:` in the read-only tree |
| **Shell** | What this GUI does today | bare citations, `crates/pdfcer-gui/src/…` |

A capability Acrobat has and the engine cannot reach is **engine work** and gets
filed in the request channel. One the engine can reach and the shell does not
call is **GUI work** and is ours. Keeping those two apart is the whole point of
the table — without it every gap looks like the same size of job.

**Measured 2026-09-16** against engine pin `20e539a2` (what
`pdfcer-gui/Cargo.lock` pins) and Acrobat DC `AcroForm.api` file version
`25.1.20435.0`. Re-measure before quoting any count below; the commands are in
§10.

**The headline, in one paragraph.** The engine is far ahead of the shell. Of the
capabilities Acrobat offers and a PDF may legally carry, the shell reaches
roughly two thirds and the engine reaches roughly nine tenths — so most of what
Ken hit in O205 is *wiring*, not engineering. The exceptions are real and
named: `/AA` (format / validate / calculate / keystroke) is the single largest
hole and is engine-side, and button icons are engine-side. The one
operator-visible engine defect O205 named — a `/Btn` rotation that reported
success and never turned — is fixed upstream and live here at this pin.

---

## 0. The kinds — Acrobat has eight, this shell places six

Acrobat's per-kind Properties dialog titles, verbatim and in its own order
(`ACRO:strings.txt:4160`, offset 3018):

> `Field Properties` · `Button Properties` · `Check Box Properties` ·
> `Dropdown Properties` · `List Box Properties` · `Radio Button Properties` ·
> `Text Field Properties` · `Digital Signature Properties` ·
> `Barcode Field Properties`

Each also has a *Tool* twin (`Button Tool Properties`, …) — Acrobat lets you set
the defaults a tool will use before placing anything. This shell has no
equivalent and it is a real convenience gap, recorded in §8.

| Code | Acrobat's name | `/FT` + flags | Placed by this shell? |
|---|---|---|---|
| **TX** | Text Field | `/Tx` | ✅ `catalog/edit.rs:378-427` |
| **CB** | Check Box | `/Btn`, no bit 16/17 | ✅ |
| **RB** | Radio Button | `/Btn` + bit 16 | ✅ |
| **CO** | Dropdown (combo box) | `/Ch` + bit 18 | ✅ |
| **LB** | List Box | `/Ch`, bit 18 clear | ✅ — a radio sub-option of the Choice command, not a command of its own (`dialogs/formfield.rs:661-662`) |
| **PB** | Button (push button) | `/Btn` + bit 17 | ✅ |
| **SG** | Digital Signature | `/Sig` | ❌ — only as a side effect of signing. `FormFieldKind::ALL` is `[Self; 5]` (`canvas/formfield.rs:89-95`) |
| **BC** | Barcode Field | `/Btn` + `/AA` + barcode params | ❌ **permanent non-goal**, engine side too (`03-capabilities.md:620`) |

**Two things to fix in passing.** `canvas/formfield/action.rs:64-79` says "six"
above a list of five, and this document is the first place the "list box is a
sub-option" decision is written down where a reader will find it.

---

## 1. How to read a cell

| Mark | Meaning |
|---|---|
| ✅ | present and working |
| ⚠ | present but broken, mislabelled, or narrower than it looks — always with the defect named |
| ❌ | absent |
| ➖ | not applicable to that kind (per the spec, not per convenience) |
| 🚫 | deliberate non-goal, with the decision cited |

Owner tags in the gap register (§8):

| Tag | Meaning | What to do with it |
|---|---|---|
| `GUI` | the engine verb exists; the shell does not call it | ours, schedule it |
| `ENGINE` | `pdfcer-core` cannot express it | write it up, file it, never patch the engine here |
| `NONGOAL` | decided against, in one repo or both | leave it, and do not draw a disabled stub (R9) |
| `ASK` | needs an operator decision before it can be built | one line to Ken, with the cost |

---

## 2. Acrobat's tab structure, and what it proves

Acrobat's Field Properties dialog is a tab strip, and the resource table that
names the tabs is contiguous (`ACRO:strings.txt:4160`, offset 3625):

> `Options` `Options` `Options` `Options` `Options` `Options` `Format`
> `Validate` `Value` `Security` `Selection Change` `NewField` `General`
> `Appearance` `Position`

Six consecutive `Options` — one per kind that has an Options tab (TX, CB, RB,
CO, LB, PB) — then the kind-specific extras, then the three tabs every kind
shares. `Calculate` and `Signed` sit adjacent to their own value lists in the
same run (offsets 4122 and 4170), and `Actions` with its six triggers comes from
`ACRO:form-dialogs.txt` (`Mouse Up`, `Mouse Down`, `Mouse Enter`, `Mouse Exit`,
`On Focus`, `On Blur`).

**Why this matters more than it looks.** The tab set *is* O206's answer for
form fields. Acrobat groups a kind's options into a fixed, published structure;
if a capability appears on a tab, every kind that has that tab is expected to
have it. That is exactly the rule Ken asked for, and it is recoverable from the
binary rather than guessed. The per-kind tab map:

| Kind | General | Appearance | Position | Options | Actions | Format | Validate | Calculate | Selection Change | Signed | Value | Security |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| TX | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | |
| CB | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | |
| RB | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | |
| CO | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | | |
| LB | ✓ | ✓ | ✓ | ✓ | ✓ | | | | ✓ | | | |
| PB | ✓ | ✓ | ✓ | ✓ | ✓ | | | | | | | |
| SG | ✓ | ✓ | ✓ | | ✓ | | | | | ✓ | ✓ | ✓ |
| BC | ✓ | ✓ | ✓ | ✓ | | | | | | | ✓ | |

**This mapping is derived from string adjacency in a resource table, not from a
screenshot.** It is consistent with the kind-specific label blocks in §4 (a
`Selection Change` script only makes sense for LB; `Signed` only for SG), but it
is the one part of the Acrobat column that a photograph pass would firm up.
Marked `ASK` in §8 and costed there.

---

## 3. General, Appearance, Position — the three tabs every kind shares

### 3.1 General tab

Acrobat labels, verbatim from `ACRO:strings.txt:3326`: `&Name:` `&Tooltip:`
`Common Properties` `&Form Field:` `&Orientation:` `&Read Only` `Require&d`
`&Group Name:`, with `&Form Field:` taking `Visible` / `Hidden` /
`Visible but doesn't print` / `Hidden but printable` and `&Orientation:` taking
`0` `90` `180` `270` `degrees`.

| Acrobat control | Kinds | Key | Engine | Shell | Verdict |
|---|---|---|---|---|---|
| Name | all | `/T` | ✅ `rename_field` ENGINE:`edit.rs:25971`; validator `forms_author.rs:465` | ✅ `formfield.rs:486-566` | ✅ |
| Tooltip | all | `/TU` | ✅ `TooltipChoice{Undecided,Text,Declined}` ENGINE:`edit.rs:1751`, writes `:25736`/`:25739` | ✅ `fieldedit.rs:487-514` | ✅ — but no `PDFCER_DIAG` region, so it cannot be driven-asserted |
| Form Field (visibility) | all | `/F` | ✅ `Visibility{VisibleAndPrints,ScreenOnly,PrintOnly,Hidden}` ENGINE:`edit.rs:1540` | ✅ `widgetedit.rs:546-591` | ✅ — exactly Acrobat's four. NoView/NoZoom/NoRotate are 🚫 in both (ENGINE `set_annotation_flags` refuses a `/Widget` by name, deliberately) |
| Orientation | all | `/MK /R` | ✅ ENGINE:`edit.rs:25026` — writes the key and turns the artwork for every kind pdfcer drew, buttons included (`:38508`) | ✅ controls present `widgetedit.rs:337-338` | ✅ — and a widget whose `/AP` pdfcer did not author still gets `/MK /R` plus the `appearance_stale` sentence, because redrawing somebody else’s artwork is the one thing worse than not turning it |
| Read Only | all | `/Ff` 1 | ✅ `FieldEdit::read_only` | ✅ `fieldedit.rs:163-176` | ✅ |
| Required | all | `/Ff` 2 | ✅ `FieldEdit::required` | ✅ `fieldedit.rs:150-162` | ✅ |
| *(no Acrobat control)* | all | `/Ff` 3 NoExport | ✅ `with_no_export` ENGINE:`edit.rs:21857` | ❌ | ❌ `GUI` — Acrobat hides this one under Options for some kinds; the engine has it and the shell has nowhere for it. Row goes beside `fieldedit.rs:150-176` |
| *(no Acrobat control)* | all | `/TM` mapping name | ✅ `with_mapping_name` ENGINE:`edit.rs:21551`, writes `:25776`/`:25779` | ❌ | ❌ `GUI` |
| Group Name | all | `/T` parent path | ✅ as a naming consequence; no create/merge/split verbs | ❌ delete-only `panels/forms/groups.rs` | ❌ `ENGINE` for split/promote, `GUI` for create-by-naming |
| Locked (dialog checkbox) | all | — | ➖ UI-only in Acrobat (locks the dialog, not the field) | ❌ | 🚫 not worth copying; the field-level `/Lock` dictionary is a different thing and is `ENGINE` |

### 3.2 Appearance tab

Acrobat labels (`ACRO:strings.txt:3326`): `Borders and Colors` — `&Border Color:`
`&Fill Color:` `Line &Thickness:` (`Thin` `Medium` `Thick`) `Line &Style:`
(`Solid` `Dashed` `Beveled` `Inset` `Underlined`); `Text` — `F&ont:`
`Font Si&ze:` (`Auto` `6` `8` `9` `10` `12` `14` `18`) `Te&xt Color:`;
and `&Digits:` (`0123456789`).

| Acrobat control | Kinds | Key | Engine | Shell | Verdict |
|---|---|---|---|---|---|
| Border Color | all | `/MK /BC` | ✅ `MkColorEdit{Set,Remove}` ENGINE:`edit.rs:21942`; gray/RGB/**CMYK** painted natively | ✅ `widgetedit.rs:718-743` | ✅ — and it is also the tick/dot ink |
| Fill Color | all | `/MK /BG` | ✅ same | ✅ `widgetedit.rs:692-716` | ✅ |
| Line Thickness | all | `/BS /W` | ✅ `BorderSpec` ENGINE:`edit.rs:1513`, write `:25202` | ⚠ `widgetedit.rs:507-532` — **the row vanishes when `border` is `None`** (`:504-506`), so a field with no border cannot be given one | ⚠ `GUI`. Also ENGINE limit: `/BS /W` does not change a CB/RB's drawn frame, fixed at 1.0 |
| Line Style | all | `/BS /S` | ✅ all five of Acrobat's, exactly | ✅ in Properties `widgetedit.rs:465-499`; ❌ at placement `author.rs:77-80` | ⚠ `GUI` — placement can't choose it, Properties can |
| Font | TX CO LB PB | `/DA` | ✅ `FieldAppearance` ENGINE:`edit.rs:21287`, write `:25766`. Std-14 under Acrobat's short keys; a `Resource` face is checked and the available list returned on refusal | ✅ `fieldedit.rs:700-847`, gate `:283-286` | ✅ — gate excludes CB/RB/SG, which matches Acrobat |
| Font Size (incl. Auto) | TX CO LB PB | `/DA` | ✅ `size == 0.0` is auto | ✅ | ✅ — ENGINE limit: a `Resource` face auto-sizes on Helvetica metrics |
| Text Color | TX CO LB PB | `/DA` | ✅ | ✅ | ✅ |
| Digits (digit shape) | TX | — | ❌ | ❌ | 🚫 — Arabic-Indic digit shaping; not in scope for either repo |
| *(no Acrobat control)* | all | `/BS /W` scale on resize | ✅ `ResizeOptions::scale_stroke_width` ENGINE:`edit.rs:17624` | ❌ | ❌ `GUI` — a resize option the shell never offers |

### 3.3 Position tab

Acrobat labels: `Left:` `Bottom:` `Right:` `Top:` `Height:` `Width:` `Units:`
and `Do not change height and width when changing the position.`

| Acrobat control | Kinds | Key | Engine | Shell | Verdict |
|---|---|---|---|---|---|
| Left / Bottom / Right / Top | all | `/Rect` | ✅ `move_widget` ENGINE:`edit.rs:26145` (no regen needed — §12.5.5 step b is a pure translation) | ✅ `widgetedit.rs:363-424` | ✅ |
| Height / Width | all | `/Rect` | ✅ `WidgetEdit::rect` ENGINE:`edit.rs:25136` — a changed extent **rebuilds** the appearance, deliberately | ✅ | ✅ |
| **Units** | all | — | ➖ presentation | ❌ points only | ❌ `GUI` — **this is O207**, and it is not scoped to this tab; see §6 |
| Do not change H/W when moving | all | — | ➖ | ❌ | ❌ `GUI`, small |
| *(no Acrobat control)* | all | `/RD` preserve | ✅ `ResizeOptions::keep_rect_differences` ENGINE:`edit.rs:17632` | ❌ | ❌ `GUI` |

---

## 4. The Options tab, kind by kind

This is where Acrobat's per-kind vocabulary lives, and where O205's "partially
implemented for one item while another that should have the same options gets a
different set" is measurable.

### 4.1 Options — Text Field (TX)

Acrobat: `&Alignment:` `&Default Value:` `Field is used for &file selection`
`Limit &of [n] characters` `Com&b of` `&Multi-line` `&Password`
`&Scroll long text` `Chec&k spelling` `Allow &Rich Text Formatting`
`Ri&ght To Left` / `L&eft To Right`.

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Alignment | `/Q` | ⚠ at this table’s pin: `with_quadding` ENGINE:`edit.rs:25751`, validated `0..=2`, **but `/Q` is not in `layout_changed` (`:25825`), so the baked `/AP` keeps the old alignment**. **Fixed on engine `main` in `503ad9d4`, one commit past the pin** — not in this shell until the pin moves | ✅ `fieldedit.rs:608-636`; clear-`/Q` ❌ | ⚠ **Was G022, now shipped upstream.** Shell row present and correct. **The clear control cannot be labelled "Left":** `/Q` is inheritable (§12.7.3.2) and the fix resolves a clear through `EditSession::inherited_quadding`, so the honest word is *inherit* |
| Default Value | `/DV` | ✅ three-state ENGINE:`edit.rs:21382`, writes `:25790`/`:25793` | ✅ `fieldedit.rs:544-581` | ✅ TX only — see 4.2/4.4 for the siblings |
| File selection | `/Ff` 21 | ✅ `with_file_select` ENGINE:`edit.rs:21871` | ❌ | ❌ `GUI` |
| Limit of N characters | `/MaxLen` | ✅ set **and remove** ENGINE:`edit.rs:25726`/`:25729`; shortening discloses `value_no_longer_fits`, never truncates | ✅ `fieldedit.rs:371-408` | ✅ |
| Comb of N | `/Ff` 25 | ✅ — requires `/MaxLen`, else `CombPreconditionUnmet` | ✅ `fieldedit.rs:427-463` | ✅ |
| Multi-line | `/Ff` 13 | ✅ | ✅ `fieldedit.rs:187-200` | ✅ |
| Password | `/Ff` 14 | ✅ | ✅ `fieldedit.rs:201-214` | ✅ |
| Scroll long text | `/Ff` 24 (inverted) | ✅ `with_no_scroll` ENGINE:`edit.rs:21530` | ❌ | ❌ `GUI` |
| Check spelling | `/Ff` 23 (inverted) | ✅ `with_no_spell_check` ENGINE:`edit.rs:21878` | ❌ | ❌ `GUI` |
| Allow Rich Text Formatting | `/Ff` 26 | ⚠ **clear-only.** Nothing authors `/RV` or `/DS`; the one write is `remove(b"RV")` ENGINE:`edit.rs:36381` | ⚠ display + a disclosed downgrade `rows.rs:333-391` | ⚠ `ENGINE` to author; the shell's downgrade is correct and honest |
| Right-to-left / left-to-right | — | ❌ | ❌ | 🚫 both repos |

### 4.2 Options — Check Box (CB)

Acrobat: `Check Box &Style:` (`Check` `Cross` `Diamond` `Circle` `Star`
`Square` — `ACRO:strings.txt:4160` offset 5258) `E&xport Value:`
`Check box is checked by &default`. Plus Acrobat's own guidance string, worth
quoting in our own help: *"Check boxes should be used to create lists of items
where zero or more items can be selected at once. To make a list of items where
only one item can be selected, use radio button fields."*

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Check Box Style — all six | `/MK /CA` char | ⚠ **creation only.** `NewCheckBox::style` ENGINE:`edit.rs:2282`; no `style` member on `FieldEdit` or `WidgetEdit`. The only route afterwards is writing the mapped `/MK /CA` **character** and letting the rebuild decode it (`annot_author.rs:3742-3760`) | ⚠ **works, via a row labelled "Caption"** `widgetedit.rs:604-629` | ⚠ `GUI` — the capability is reachable and shipping; the label is wrong. Needs a kind fork so CB shows a glyph picker and PB shows a word box |
| Export Value | on-state name | ✅ as the on-state | ✅ `author.rs:172-176` | ✅ |
| Checked by default | `/DV` (a **name** for `/Btn`) | ✅ `FieldEdit::default_value` — type follows `/V`, so a name | ❌ | ❌ `GUI` — **the clearest O206 instance in the whole document.** `/DV` shipped for TX and for nothing else, though the engine verb is kind-agnostic |

### 4.3 Options — Radio Button (RB)

Acrobat: `Button &Style:` `Radio Button &Choice:` `Button is chec&ked by default`
`Buttons &with the same name and choice are selected in unison`. Guidance
string: *"To create a set of mutually exclusive radio buttons … give the fields
the same name but different button choices."*

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Button Style — all six | `/MK /CA` char | ⚠ creation only, as CB. `NewRadioButton::style` ENGINE:`edit.rs:2385` | ❌ — the Caption row is not offered for RB | ❌ `GUI` — a sibling of 4.2 that did not get the same treatment |
| Radio Button Choice | on-state name | ✅ | ✅ | ✅ |
| Checked by default | `/DV` | ✅ | ❌ | ❌ `GUI` — sibling of 4.2 |
| Radios in unison | `/Ff` 26 (`/Btn`) | ✅ `radios_in_unison` ENGINE:`edit.rs:21419`; also at creation `:2364` | ❌ | ❌ `GUI` — row goes beside `fieldedit.rs:228-243` |
| *(no Acrobat control)* | `/Ff` 15 NoToggleToOff | ✅ | ✅ `fieldedit.rs:228-243` — RB with >1 widget only | ✅ |

### 4.4 Options — Dropdown (CO) and List Box (LB)

Acrobat, shared: `&Item:` `E&xport Value:` `It&em List:` `&Add` `&Delete`
`&Up` `D&own` `Check s&pelling` `&Sort items`. CO only:
`Allo&w user to enter custom text` `Commi&t selected value immediately`. LB
only: `&Multiple selection`. Acrobat also warns *"Select an item in the list to
make it the default choice."*

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| **Item List — add** | `/Opt` | ✅ `FieldEdit::with_options` ENGINE:`edit.rs:21815`, write `:25744`; whole-list replace by design | ✅ both — placement dialog `dialogs/formfield.rs:648-655`, and after placement in `panels/properties/choiceopts.rs` (`add` + `add_button` regions) | ✅ O205's second named defect, closed |
| Item List — delete | `/Opt` | ✅ — whole list with one fewer; stale selection **disclosed** via `value_no_longer_fits`, never silently repaired | ✅ `panels/properties/choiceopts.rs`, `row{n}.remove` | ✅ |
| Item List — reorder (Up/Down) | `/Opt` | ✅ — `/Opt` order is display order (Table 230) | ✅ `panels/properties/choiceopts.rs`, `row{n}.up`/`.down`, each greyed at its own end of the list rather than swallowing the press | ✅ driven `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` |
| Item List — rename display text | `/Opt` | ✅ | ✅ `panels/properties/choiceopts.rs`, `row{n}.shown` | ✅ |
| Export Value ≠ display | `/Opt` pair | ✅ two-string option writes `[export display]` ENGINE:`forms.rs:375` | ✅ `panels/properties/choiceopts.rs` draws both columns — `row{n}.shown` and `row{n}.sent` | ✅ |
| Duplicate item | — | ✅ **refused** `ChoiceOptionDuplicate` ENGINE:`edit.rs:7354` | ➖ | ✅ the refusal is the right behaviour; surface it |
| Sort items | `/Ff` 20 | ✅ flag written, and **`sort_choice_options` is exported** (Pass 308.8) so one comparator serves both sides; bit 20 set *alone* still reorders nothing and is still disclosed as `sort_claim_unmet` ENGINE:`edit.rs:22080` | ✅ author-only `dialogs/formfield.rs:673`, and afterwards in `panels/properties/choiceopts.rs` (`sort` region), which calls the engine's sorter rather than a copy of it | ✅ E9 delivered; the 058 report closed by the export, and `FieldEditOutcome::options_sorted` is consumed as the drift tripwire |
| Allow custom text (CO) | `/Ff` 19 Edit | ✅ `editable`; `Edit` without `Combo` refused; `set_choice_value` already accepts a value absent from `/Opt` for this case ENGINE:`edit.rs:39790-39818` | ⚠ **flag settable at placement only, and no free-text entry exists anywhere** — not canvas, not panel, not Properties (`dialogs/formfield.rs:664-666`, `rows.rs:749-835`). The canvas does not read the bit at all: the box census forks on `Combo` and `MultiSelect` only (`canvas/forms/boxes/mod.rs:540-545`), so an editable combo opens the same closed picker a plain one does | ⚠ `GUI` — an editable combo is authorable and unfillable; 8.1 row 15 |
| Commit immediately (CO) | `/Ff` 27 | ✅ `with_commit_on_sel_change` | ❌ | ❌ `GUI` |
| Multiple selection (LB) | `/Ff` 22 | ✅ | ✅ `fieldedit.rs:261-274` | ✅ |
| Check spelling | `/Ff` 23 | ✅ | ❌ | ❌ `GUI` — sibling of 4.1 |
| Default choice | `/DV` | ✅ | ✅ `panels/properties/choiceopts.rs::default_row`, which offers *"Nothing — Reset clears the field"* as its first entry because a reset to empty is a different fact from the chooser having nothing selected, and writes the **export** value since `/DV` takes `/V`'s type (Table 228) | ✅ CO and LB done; CB and RB are still 8.1 row 4 |
| **Pick a value (fill)** | `/V` `/I` `/TI` | ✅ `set_choice_value` ENGINE:`edit.rs:38709` — export-first matching, `/I` and `/TI` maintained | ✅ both. Panel: `panels/forms/rows.rs:810` → `FormEdit::SetChoice` → `panels/forms/edit.rs:765`. Canvas: `canvas/forms/choosing.rs` — a click opens the option list on the field's own answered row, and **`Combo` decides where it is drawn** (`choosing.rs:473`): a combo drops outside the widget, a list box draws opaquely inside its own rectangle with a scroll bar when the rows do not fit, which is what Acrobat does | ✅ O205's first named defect closed, and O209's C1–C3 with it |

### 4.5 Options — Button / push button (PB)

Acrobat: `La&yout:` (`Icon top, label bottom` `Label top, icon bottom`
`Icon left, label right` `Label left, icon right` `Icon only` `Label only`
`Label over icon` — offset 4643) `Beha&vior:` (`None` `Push` `Outline`
`Invert` — offset 4765) `&State:` (`Up` `Down` `Rollover`) `La&bel:` `Icon:`
`Ch&oose Icon...` `Clea&r` `&Advanced...`, and under Icon Placement
`&When to Scale:` `&Scale:` `&Fit to bounds` `&Reset`.

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Label | `/MK /CA` | ✅ `with_caption`; `Some("")` removes; **redraws** ENGINE:`edit.rs:21582`, `:25301-25336` | ✅ `dialogs/formfield.rs:680`, `widgetedit.rs:604-629` | ✅ |
| Label for Down state | `/MK /AC` | ❌ `grep 'b"AC"'` → **0 hits** | ❌ | ❌ `ENGINE` |
| Label for Rollover state | `/MK /RC` | ❌ 1 hit, a FreeText `drop_keys` entry ENGINE:`edit.rs:32360` | ❌ | ❌ `ENGINE` |
| Icon (normal / rollover / down) | `/MK /I` `/RI` `/IX` | ❌ zero hits each | ❌ | ❌ `ENGINE` |
| Icon fit / scale / fit-to-bounds | `/MK /IF` | ❌ 0 hits | ❌ | ❌ `ENGINE` |
| Layout (icon vs label position) | `/MK /TP` | ❌ 0 hits | ❌ | ❌ `ENGINE` |
| Behavior (highlight mode) | `/H` | ❌ no verb | ❌ | ❌ `ENGINE` |
| Button actions | `/A` | ✅ six subtypes ENGINE:`edit.rs:36852`; `Foreign(String)` names one pdfcer would never author, so a control knows not to offer to replace it | ✅ `dialogs/buttonaction.rs:67-138`, `panels/forms/button.rs:72-172` | ✅ — the shell's strongest per-kind surface |
| *(disclosure)* | — | ✅ `push_button_inert` ENGINE:`edit.rs:1908` — *the only creation verb whose successful result is a control that does not work* | ✅ | ✅ |

### 4.6 Digital Signature (SG) — the Signed tab

Acrobat: `&Nothing happens when signed` / `&Mark as read-only:` (`All fields` /
`Just these fields` / `All fields except these`) /
`This &script executes when field is signed:` (offsets 4170 ff).

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Place an empty signature field | `/FT /Sig` | ✅ via `sign(...)`/`CommandKind::AddSignatureField`; no `add_signature_field` of its own | ❌ not a placeable kind | ❌ `GUI` + `ASK` — Acrobat treats "prepare a form with a signature box" as ordinary authoring |
| Mark as read-only on sign | `/Lock` | ❌ **read-only in the engine** — 2 hits, one a read at signing time ENGINE:`edit.rs:27041`, one a carried-key list | ❌ DOC:`ENGINE_BACKLOG.md:117` | ❌ `ENGINE` |
| Script on sign | `/AA` | ❌ | ❌ | ❌ `ENGINE` (part of the `/AA` hole) |
| Seed value enforcement | `/SV` | ✅ enforced in full when signing into a pre-placed field | ➖ | ✅ engine-side |
| Delete a signature field | — | ✅ | ✅ gated `formfield.rs:597-647` | ✅ |
| Fill / set a value | `/V` | 🚫 by design — a signature value comes only from signing | 🚫 blocked `rows.rs:279-283` | ✅ correct |

### 4.7 Format, Validate, Calculate, Selection Change — the `/AA` tabs

Acrobat's four field triggers (`ACRO:form-dialogs.txt`): `Validate Field`,
`Compute Field`, `Validate Keystroke`, `Format Field`. Its controls: `Format`
tab with number / percentage / date / time / special / custom
(`&Decimal Places:` `C&urrency Symbol:` `S&ymbol Location:` `&Separator Style:`
`Negative Number Style:` `Show pare&ntheses` `Use red te&xt`,
`Choose date format:`, `Mask using [c] character`, `Custom Fo&rmat Script:`,
`Custom &Keystroke Script:`); `Validate` tab (`Field value is n&ot validated` /
`Field value is in &range: &From: &To:` / `Run custom validation &script:`);
`Calculate` tab (`Value is n&ot calculated` / `&Value is the [sum (+) ·
average · product (x) · minimum · maximum] of the following fields:` /
`Simplified &field notation:` / `Custom calculation &script:`, plus
`Calculated Fields &Up &Down` for `/CO`); `Selection Change` for LB
(`Do n&othing` / `Execute this &script:`).

| Acrobat control | Key | Engine | Shell | Verdict |
|---|---|---|---|---|
| Read + disclose that a field has actions | `/AA` | ✅ `has_no…`/`has_additional_actions` ENGINE:`forms.rs:1584`; `FieldClip::carries_actions()` `formclip.rs:378` | ✅ read paths exist | ✅ |
| Classify a script without running it | `/AA` | ✅ `form_script::classify` ENGINE:`form_script/mod.rs:355`; *"a false positive is far worse than a false negative"* | partial | ✅ engine-side, strong |
| **Author or edit any `/AA` entry** | `/AA` | ❌ **every hit is a read, a `contains_key`, or an allowlist — except two removals** (ENGINE:`edit.rs:48704`, `:49030`) | ❌ **no `/AA` surface exists anywhere** | ❌ `ENGINE` — **the single largest Acrobat-parity hole in the product** |
| Number / date / percentage format presets | `/AA /F` | ❌ | ❌ | ❌ `ENGINE` |
| Range validation | `/AA /V` | ❌ | ❌ | ❌ `ENGINE` |
| Sum / average / product / min / max | `/AA /C` | ❌ as an author; ✅ as a **recognised** built-in via `CalcHelper` ENGINE:`form_script/mod.rs:185` | ❌ | ❌ `ENGINE` |
| Calculation order | `/CO` | ⚠ pruned on delete, appended on paste; **no reorder verb** ENGINE:`edit.rs:40109-40145` | ❌ | ❌ `ENGINE` |
| Execute any of it | — | 🚫 **permanent non-goal** — no interpreter, no `event`, no trigger dispatch. Recompute is operator-invoked and undoable | 🚫 | ✅ correct and deliberate |
| Apply a recompute plan | — | ⚠ `apply_recompute` → 0 hits; **the shell writes the loop** over `plan.changes` | whole-form only `panels/forms/edit.rs:381-405` | ⚠ `GUI` for per-field |

### 4.8 Actions tab

Acrobat's six widget triggers: `Mouse Up` `Mouse Down` `Mouse Enter`
`Mouse Exit` `On Focus` `On Blur` (`ACRO:form-dialogs.txt`).

| Trigger | Engine | Shell | Verdict |
|---|---|---|---|
| Mouse Up → `/A` | ✅ six subtypes, PB only | ✅ PB only | ✅ for PB; ❌ for other kinds (`ENGINE` — `/A` on a non-button widget has no verb) |
| The other five (`/AA /D /E /X /Fo /Bl`) | ❌ | ❌ | ❌ `ENGINE` |

---

## 5. Filling — the part O205 actually started from

Acrobat fills every kind on the page. This shell now fills five of the seven
there; the two surfaces still disagree on the other two.

| Kind | On the canvas | In the Forms panel | Keyboard | Verdict |
|---|---|---|---|---|
| TX | ✅ | ✅ | ➖ | ✅ |
| CB | ✅ | ✅ | ✅ Space/Enter `tabbing.rs:307-308` | ✅ |
| RB | ✅ | ✅ | ✅ + arrows move **and activate** `tabbing.rs:400-402` | ✅ |
| **CO** | ✅ `canvas/forms/choosing.rs` — the list **drops** outside the widget | ✅ | ✅ Up/Down move the highlight and write nothing, Enter or Space picks, Escape closes the list on the first press and gives up the ring on the second | ✅ |
| **LB** | ✅ same module — the list draws **in place**, opaquely over the appearance, with a scroll bar when the rows do not fit | ✅ | ✅ as CO; a multi-select list adds a value per pick and stays open | ✅ |
| PB | ❌ | ➖ action editor | ❌ | ⚠ `ASK` — should clicking a push button on the canvas *run* its action in Read mode? Acrobat does |
| SG | ❌ | 🚫 blocked | ❌ | ✅ correct |

**What is left, and it is no longer the fork.** `BoxKind` now has four variants
(`canvas/forms/boxes/mod.rs:88-210`) and `classify()` (`:461-549`) matches TX, CB, RB
and CO/LB alike; PB and SG fall to `_ => Err(NotOffered)` at `:547`. **That omission
is still silent**, which is 8.1 row 3: `Routing.undrawn` counts only `NoAppearance`
and `unreachable` counts only `RotatedPage|NotPlaced` (`:699-706`), so
`panels/forms/mod.rs:508-520` can never explain a dead click on the two kinds that
have no gesture. An independent measurement reached the same conclusion from the
other direction (`evidence/forms-parity/canvas-per-kind.md` §3.4), and it was taken
before the fork was built — read its CO and LB rows as history.

**This is the canvas-primacy rule biting.** If the engine can do it, clicking the
object must reach it — the panel is a second route, not the route.

---

## 6. O207 — units typed beside the number

Ken: *"I can't set units by typing them beside the numbers as sizes. Things
still only are able to be entered in as points."*

This is not a forms feature; it is every length entry in the product, and the
Position tab is where he hit it. Acrobat has a `Units:` selector on the Position
tab; **every CAD input on his desktop goes further and parses the unit inline.**

| Piece | State | Citation |
|---|---|---|
| The parser | ✅ **already exists in the engine** — `pdfcer_core::dimension::parse_length(input: &str, default_unit: Unit)` | ENGINE:`length_parse.rs:139` |
| Its grammar | ✅ accepts `12 mm`, `1/2"`, bare numbers against a default unit | ENGINE:`length_parse.rs` |
| The shell's helper module | ⚠ `crates/pdfcer-gui/src/units.rs` has ten public functions and **all of them convert; none parse** | `src/units.rs` |
| The entry sites | ❌ ~68 `DragValue` sites take a bare number as points | — |

So O207 is one shared helper plus a sweep, not a feature. It is `GUI`, it is
cheap, and the parser it needs is already on the other side of the path
dependency. **Scope note per O206: "length entry" is the whole set** — widget
rect, border width, margins, page setup, measure, markup, crop. Doing it for the
Position tab alone would reproduce exactly the complaint he filed.

---

## 7. O206 — the sibling rule, made into a lookup

Ken: *"whenever I ask for a feature, what I really mean is that feature and all
the options that would normally exist for that feature. Most of the time we just
end up with features partially implemented for one item while another that
should have the same options gets a different set."*

Recorded as contract clause 7 in `OPERATOR_REQUESTS.md`. The instrument is
below: **before starting any form-field work, find the row and do the whole
row.** A capability shipped for one member of a sibling set and not the others
is not "done" — it is the defect he is describing.

| Sibling set | Members | Rule |
|---|---|---|
| **Default value** | TX, CB, RB, CO, LB | `/DV` is kind-agnostic in the engine (the *type* follows `/V` — a name for `/Btn`, a string otherwise). Shipped for TX only. **Ship all five or none.** |
| **Glyph style** | CB, RB | Same six styles, same `/MK /CA` mechanism, same picker. Shipped for CB (mislabelled), absent for RB. |
| **The advisory `/Ff` flags** | TX (scroll, spell-check, file-select), CO/LB (spell-check) | All four builders exist in the engine; none are surfaced. They arrived in one engine commit and should leave in one GUI commit. |
| **Option list** | CO, LB | add · remove · reorder · rename display · export ≠ display · sort · default choice. **Seven operations, one row.** Shipping "add" alone is the complaint. |
| **Canvas fill** | every kind the engine can fill | TX, CB, RB ship; CO, LB do not. The engine can fill CO and LB. |
| **Alignment** | TX, CO, LB | `/Q` is legal on all three (Table 228 and 230 both carry it). Shipped for TX only. |
| **Colour and border** | all seven | `/MK /BG`, `/MK /BC`, `/BS /S`, `/BS /W` — the shell does treat these uniformly. **This set is the example of it done right.** |
| **Visibility, read-only, required, tooltip, rename, delete** | all seven | Also uniform today. Keep them that way. |
| **Length entry** | every numeric length in the product | O207. See §6. |
| **Rotation** | all seven | Uniform in the shell and, since `20e539a2`, uniform in the engine. The one asymmetry left is by design: pdfcer turns artwork it drew and discloses rather than redraw artwork it did not. |

**How to use this when a request arrives.** Find the capability in §3–§5, read
its row, then check §7 for the sibling set it belongs to. If the set has members
the request did not name, they are in scope — that is what he told us the
request means.

---

## 8. The gap register

Owner tags as defined in §1. `GUI` rows are ours and need no one's permission.
`ENGINE` rows are hand-offs; three are already filed.

### 8.1 `GUI` — the engine verb exists and the shell does not call it

| # | Gap | Where it goes | Engine verb |
|---|---|---|---|
| 1 | ~~`/Opt` list editing after placement (7 operations)~~ **BUILT** — `panels/properties/choiceopts.rs`, drawn at the end of the `FieldType::Choice` branch. All seven: add, remove, move up, move down, rename display, export separate from display, and keep-sorted; plus the default choice, and bits 19, 23 and 27 because Acrobat's Options tab groups them (O206). **Driven** — `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` asserts all six arrow states across three rows through `PDFCER_DIAG_SELECT_FIELD`, and was falsified against a planted live top-row arrow and against a broken seam | `panels/properties/choiceopts.rs` | ENGINE:`edit.rs:21815` |
| 2 | ~~Canvas fill for CO and LB~~ **BUILT** — `canvas/forms/choosing.rs`. A click on a combo or list widget opens its option list on the field's own answered row; Up/Down move the highlight and write nothing; Enter or Space, or a click on a row, writes one `SetChoice`; a multi-select list adds a value per tick and stays open; Escape closes the list on the first press and gives up the ring on the second. **Driven** — `a_drop_down_can_be_answered_on_the_page` asserts all seven steps on `fixtures/all-field-kinds.pdf` and was falsified three ways: a reverted focus lock, an arrow that answers the form, and a multi-select tick that replaces instead of adding. The driving is what found the defect — egui moves keyboard focus on a bare arrow and surrenders it on Escape unless the focused widget locks them, so the list opened, took one arrow, and then went dead to every further key, silently. Fixed by `canvas/forms.rs::keyboard_box`, which both on-page field editors now share | `canvas/forms/choosing.rs` | ENGINE:`edit.rs:38709` | **Rebuilt against Acrobat under O209**, which reported three defects in it: a pick was not remembered (the popup gave up on the *press* frame, because egui surrenders a focused widget's focus on any press that does not land on it and `choose()` early-returned on the loss — a click is a press and a release, and the popup has to survive the gap); the open list looked nothing like Acrobat's (a *maximum* width let it shrink to its longest label, 57 px under a 370 px field, inside egui's rounded shadowed pop-up card — now a fixed width in a square unshadowed zero-margin frame flush against the field, zero row spacing, full-width rows, and a selected row painted as a solid `Theme::accent_pair` plate, which is Acrobat's treatment in this shell's palette); and a list box was drawn as a drop-down (`/Ff` bit 18 `Combo` never reached the census — it now forks the presentation at `choosing.rs:473`). **Driven** — the pick, both presentations, the multi-select plate, and the scroll bar appearing with the last row clipped at a smaller zoom
| 3 | `NotOffered` is silent — nothing can explain a dead click. Narrower than it was: CO and LB now have a gesture, so what is left unexplained is PB, SG, a rich-text field and a choice field with an empty `/Opt` | `boxes/mod.rs:699-706` + `panels/forms/mod.rs:508-520` | ➖ |
| 4 | `/DV` for CB and RB. **CO and LB are done** — the default-choice chooser in `choiceopts.rs`, which offers *"Nothing — Reset clears the field"* as its first entry because a reset to empty is a different fact from the chooser having nothing selected | `fieldedit.rs:544-581` | ENGINE:`edit.rs:21382` |
| 5 | Glyph-style picker for RB, and a kind fork so the row is labelled right | `widgetedit.rs:604-629` | `/MK /CA` char |
| 6 | Radios-in-unison | beside `fieldedit.rs:228-243` | ENGINE:`edit.rs:21419` |
| 7 | Do-not-scroll | `fieldedit.rs:186-225` | ENGINE:`edit.rs:21530` |
| 8 | Do-not-spell-check (TX, CO, LB) | same | ENGINE:`edit.rs:21878` |
| 9 | File-select | same | ENGINE:`edit.rs:21871` |
| 10 | No-export `/Ff` 3 | beside `fieldedit.rs:150-176` | ENGINE:`edit.rs:21857` |
| 11 | Commit-on-selection-change | choice rows | `with_commit_on_sel_change` |
| 12 | Sort flag after placement, with `sort_claim_unmet` shown | choice rows | ENGINE:`edit.rs:22080` |
| 13 | Mapping name `/TM` | field rows | ENGINE:`edit.rs:21551` |
| 14 | Alignment `/Q` for CO and LB; clear-`/Q` for all, **labelled *inherit* rather than *Left*** — the engine resolves a clear through the `/Parent` chain and then `/AcroForm`, which this shell cannot see | `fieldedit.rs:608-636` | ENGINE:`edit.rs:21835` |
| 15 | Free-text entry for an editable combo (canvas, panel or Properties — any). The engine already takes it: `set_choice_value` writes export == display == the typed text when the field is an editable combo. The canvas does not read `/Ff` bit 19 at all — the box census forks on `Combo` and `MultiSelect` only (`boxes/mod.rs:540-545`) — so an editable combo opens the same closed picker a plain one does. Found under O209 while building the combo/list fork; not reported by the operator | `rows.rs:749-835`, `canvas/forms/boxes/mod.rs:540-545` | ENGINE:`edit.rs:39790-39818` |
| 16 | Border width row when `border` is `None` | `widgetedit.rs:504-532` | ENGINE:`edit.rs:25202` |
| 17 | Border style at placement | `author.rs:77-80` | ENGINE:`edit.rs:1652` |
| 18 | Resize options — scale stroke width, keep `/RD` | resize path | ENGINE:`edit.rs:17624`, `:17632` |
| 19 | **Units typed beside numbers** (O207), product-wide | `src/units.rs` + ~68 sites | ENGINE:`length_parse.rs:139` |
| 20 | Tab order by buttons / numeric entry / keyboard | asserted absent `tab_order/mod.rs:991-1006` | ENGINE:`edit.rs:30278` |
| 21 | Per-field flatten / reset / regenerate / recompute | `panels/forms/edit.rs:381-405` | verbs take a name list |
| 22 | Group create / rename by naming | `panels/forms/groups.rs` | naming consequence |
| 23 | Place an empty signature field | `canvas/formfield.rs:89-95` | signing path |
| 24 | Distinct placement ghost per kind | `canvas/formfield/ghost.rs:79-127` | ➖ |
| 25 | Per-kind tool defaults (Acrobat's "*Tool* Properties") | new | `FieldDefaults` ENGINE:`edit.rs:1953` |
| 26 | Driven `ui-verify` coverage for RB, CO, LB. **CO and LB are done** — `the_option_arrows_are_greyed_only_at_the_ends_of_the_list` drives the Properties pane and `a_drop_down_can_be_answered_on_the_page` drives the page, both pinning `fixtures/all-field-kinds.pdf` and both falsified. **RB is not** — no driven check presses a radio button, so the arrow-key sibling walk in `canvas/forms/tabbing.rs` is asserted only by unit test, and it shares the focus-lock defect row 2 found: a press that finds no sibling abandons the ring | `fixtures/all-field-kinds.pdf`; `D:\Dev\pdfcer\fixtures\synthetic\forms\radio-choice-form.pdf` also exists | ➖ |
| 27 | ~~A multi-select list carried a stored value the options do not list into an engine refusal~~ **BUILT** — `panels/forms/rows.rs::multi_choice_ticks`. The check-box stack copied `/V` and edited it, so a `/V` entry matching no `/Opt` entry — set by another program, or left behind when the option list was edited — rode along, and the operator's first tick arrived at `set_choice_value` as that value plus theirs, which the engine refuses as `ChoiceValueNotInOptions`: a refusal naming a value they never touched, in answer to a gesture that was valid. The row now rebuilds the selection by asking each option whether it is selected — the rule `canvas::forms::choosing::wanted` already followed, and whose doc comment already said *"which is not what the panel does"* — and discloses the drop in words, because a check-box stack has no box to show the value in and the single-select wording (*"It is shown as it is stored"*) is false there. Falsified both halves: copying `/V` back makes `a_value_the_options_do_not_list_is_dropped_and_disclosed` and `a_selection_stored_by_display_string_is_recognised` fail, and forcing the disclosure flag false fails the first alone. **Not driven** — the visible half needs a fixture whose `/V` names no option, which `fixtures/all-field-kinds.pdf` does not have; the defect itself was invisible on screen, which is why eight green unit tests of the verb never saw it | `panels/forms/rows.rs` | ENGINE:`edit.rs:38709` |

### 8.2 `ENGINE` — hand-offs

| # | Gap | Status |
|---|---|---|
| E1 | A `/Btn` rotation is written, reports success, and never turns | **CLOSED — `20e539a2`, Pass 308.5, and live in this shell.** `build_button_states` now takes the quarter turn, authors into an `h x w` `/BBox` for 90/270 and emits `quarter_turn_matrix`. Verified at the pin, not taken from the commit message |
| E2 | `/Q` written but never redrawn | **CLOSED — `503ad9d4`, Pass 308.4, and live in this shell.** `edit.rs:25912` gates the redraw on `edit.quadding.is_some()` and `inherited_quadding` resolves a clear to *inherit*. The shell still owes the clear control that says so — `GUI` work, not engine |
| E3 | `/AA` authoring: format, keystroke, validate, calculate | **FILED — `request_G024`.** Asks for an **emitter** over the three enums `classify` already parses, plus `/CO` maintenance and the `/F`+`/K` pairing, as one set per O206 |
| E4 | `/MK` button icons and state captions: `/I` `/RI` `/IX` `/IF` `/TP` `/AC` `/RC` | to file |
| E5 | `/H` highlight behaviour (None/Push/Outline/Invert) | to file |
| E6 | `/Lock` authoring | to file; `ENGINE_BACKLOG.md:117` |
| E7 | `/CO` reordering | to file |
| E8 | `/Tabs` writing — the verb `reorder_annotations`' own doc says should exist | to file |
| E9 | `/Opt` actually sorted when bit 20 is set | **DELIVERED - engine `Pass 308.8`, carried by the pin.** Both halves of the ask landed: `choice_option_order` and `sort_choice_options` are public, and `edit_field` sorts when `options` and `sort` arrive in one `FieldEdit`. Bit 20 set alone still reorders nothing and is still disclosed as `sort_claim_unmet`, which was never the complaint. **The shell's copy is deleted** - `choiceopts::sort_by_display` calls the engine's sorter - and `FieldEditOutcome::options_sorted` is consumed as the tripwire for the drift the export prevents |
| E10 | Glyph style change as a first-class verb on an existing field | to file |
| E11 | Rich-text authoring (`/RV`, `/DS`, bit 26 on) | to file |
| E12 | `/A` on a non-button widget; the five non-`Mouse Up` triggers | to file |
| E13 | List-box selection highlight in the appearance; SG/untyped appearance | to file |
| E14 | Move a widget to another page in place; split a field; promote a terminal to a grouping node | to file |
| E15 | `apply_recompute` — the shell writes the loop today | low priority, works |
| E16 | `edit_field` writes a duplicate `/Opt` export that `add_choice_field` refuses by name | **DELIVERED - engine `Pass 308.7`, carried by the pin.** The guard moved to `refuse_duplicate_exports` and both verbs call it, so no door writes a duplicate. A document that ARRIVED holding one stays editable, which was the requester's call and was the right one. **The shell's `refuse_duplicate` is kept, not deleted**, and this is a live 058 report: the engine's refusal reaches the funnel as the un-categorised `Declined::EditRefused`, which names neither the rule nor the repeated value, so the panel asks first in order to say which one. That leaves one rule spelled in two places - filed as **`request_G027`**, which asks for the predicate to be exported the way `G026` exported the ordering |

### 8.3 `NONGOAL` — do not build, do not stub (R9)

Barcode fields · executing JavaScript · field-type conversion (Acrobat dropped
it after Acrobat 6; the engine makes the request *unrepresentable*) ·
right-to-left / digit shaping · static XFA · wide/batch CSV · arbitrary `/F`
bits on a widget (two writers of one key with different vocabularies) ·
Acrobat's dialog-level "Locked" checkbox.

### 8.4 `ASK` — needs an operator decision

| # | Question | Cost of answering |
|---|---|---|
| A1 | Photograph Acrobat's per-kind **Options tabs** to firm up §2's adjacency-derived mapping. Still open — what was photographed under O209 is the *filling* side (how a drop-down and a list box look and behave in Reader), which is what §4.4's fill row and §5 are now measured against; the authoring dialogs have not been opened | needs the desktop and Acrobat in Prepare Form; ~20 min, and it must not run while he is working |
| A2 | Should clicking a push button on the canvas in Read mode *run* its action? Acrobat does | design call |
| A3 | Is an empty signature field a placeable authoring kind, or signing-only? | design call |

---

## 9. Counts

Derived from the tables above, not asserted:

| Owner | Rows |
|---|---|
| `GUI` | 27 |
| `ENGINE` | 16 (E1 and E2 closed and live, E3 filed, 13 to file) |
| `ASK` | 3 |

Re-derive with the commands in §10 before quoting these anywhere.

---

## 10. Sources, and how to re-measure

| Column | Evidence | How it was produced |
|---|---|---|
| Acrobat | `evidence/acrobat-forms/strings.txt` (5,033 runs), `form-dialogs.txt` — **not committed; regenerate with `python tools/acrobat-form-strings.py`.** This repository is public and those files are Adobe's resource strings verbatim; see `evidence/acrobat-forms/README.md` | `tools/acrobat-form-strings.py` — UTF-16LE runs out of `AcroForm.api` (20,508,568 bytes, v25.1.20435.0) and `AcrobatRes.dll`. **Vocabulary, verbatim; structure by adjacency** — see A1. ⚠ Line numbers and offsets in every `ACRO:` citation are version-specific: search for the quoted label, do not seek to the offset |
| Spec ceiling | `evidence/forms-parity/spec-field-properties.md` | ISO 32000-1 Tables 220/221/226/228/230, §12.7.3–§12.7.5, §12.5.5, erratum #56 |
| Engine | `evidence/forms-parity/engine-form-verbs.md` | measured against `git show HEAD:` at pin `5d43d2ea`, which is one revision behind the pin this table now quotes. E1 and E2 were re-measured against `20e539a2` directly; nothing else in the engine changed between the two |
| Shell | `evidence/forms-parity/shell-form-surface.md`, `canvas-per-kind.md` | two independent measurements of the same surface; they agree on every mechanism |

```
# gap-register row counts -- section 8 only. Counting backticked tags
# document-wide over-reports: a tag also appears in every verdict cell
# in sections 3-5. Re-measured 2026-09-17: 27 / 16 (E3 filed) / 3.
#
# The closing pipe of each row pattern is ESCAPED. An earlier revision of
# these four commands left it bare, where an unescaped trailing pipe is an
# ERE alternation with the empty string: the pattern then matches EVERY line
# and 8.1 reports 32 rows in a 26-row table. A count command can be wrong,
# not just its answer.
sed -n '/^### 8.1/,/^### 8.2/p' FORMS_PARITY.md | grep -cE '^\| [0-9]+ \|'
sed -n '/^### 8.2/,/^### 8.3/p' FORMS_PARITY.md | grep -cE '^\| E[0-9]+ \|'
sed -n '/^### 8.2/,/^### 8.3/p' FORMS_PARITY.md | grep -E '^\| E[0-9]+ \|' | grep -c FILED
sed -n '/^### 8.4/,/^---/p'     FORMS_PARITY.md | grep -cE '^\| A[0-9]+ \|'

# the engine pin this was measured against
grep -A2 'name = "pdfcer-core"' Cargo.lock | grep source
```

**Re-measure before quoting.** Every count, every `file:line`, and every
"ABSENT" in this document is a claim about a sha. The engine session ships fixes
within hours; a row here can be stale by the time you read it. The `ENGINE`
column is the one most likely to have moved — check the reply channel at
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\` first.

---

## 11. What this table is owed next

The same treatment for the other three tool families Ken named the same way —
**markup**, **measure** and **text**. The method transfers exactly: read the
reference product's vocabulary out of its binaries, measure the engine's verbs
against a clean sha, measure the shell's surface, and split the gaps by owner.
Forms went first because O205 named it.
