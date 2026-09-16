# What this shell can do to a form field today

The **shell column** of `FORMS_PARITY.md`. Read-only measurement, 2026-09-16, at
`b26f741` with the engine pinned to `5d43d2ea`. No file was edited to produce it.

Paths are relative to `crates/pdfcer-gui/src/` unless prefixed `ENGINE:`
(= `D:\Dev\pdfcer\crates\pdfcer-core\src\`, read-only), `UV:`
(= `tools/ui-verify/src/`) or `DOC:` (repo root).

> **5 below was the load-bearing finding of this measurement, and it won an
> argument.** It contradicted `engine-form-verbs.md` 2.15 / 3.2, which
> credited `rotate_widget` with working for any kind and disclosing the
> exceptions through `appearance_stale`. Re-measured directly against the
> engine at `5d43d2ea`: **5.4 is right.** `regen_button_appearance`
> (`edit.rs:38349`) never reads the staged angle, `build_button_states`
> (`:38534`) has no angle parameter, and `rotation_for` has exactly one call
> site in the crate (`:38070`, the text/choice path). The engine document was
> wrong because it was built from `appearance_stale`'s own sentence rather
> than from the code path that sets it -- and that sentence names cases that
> do not reach it. Corrected there; filed as `request_G023` in the channel.

---

## 0. The seven kinds are not seven things in this codebase

| Requested kind | How this shell models it | Citation |
|---|---|---|
| Text field | `FormFieldKind::Text`; `FieldType::Text` | `canvas/formfield.rs:60-85`; ENGINE:`forms.rs:228-238` |
| Check box | `FormFieldKind::CheckBox`; `Button` + `ButtonKind::Check` | `canvas/formfield.rs:60-85`; ENGINE:`forms.rs:281-290` |
| Radio button | `FormFieldKind::Radio`; `Button` + `ButtonKind::Radio` | same |
| Combo box | `FormFieldKind::Choice` with `draft.combo == true` | `canvas/formfield/draft.rs:129`, default `true` at `:220` |
| **List box** | **the same `Choice` kind with `draft.combo == false`** — a radio pair inside the Drop-down placer, **not a sixth command** | `dialogs/formfield.rs:661-662` → `app/actions/forms/author.rs:178` |
| Push button | `FormFieldKind::PushButton`; `Button` + `ButtonKind::Push` | `canvas/formfield.rs:60-85` |
| **Signature** | **not in `FormFieldKind` at all** — read / fill-block / sign / delete only | `canvas/formfield.rs:89-95` (`ALL: [Self; 5]`); `text/fieldclip.rs:69` *"there is still no `add_signature_field`"* |

`canvas/formfield.rs:52-58` states the rule: *"Exactly the five `pdfcer-core` has
verbs for."*

Consequence for every table below: **combo and list box share one code path
except where `FieldFlags::COMBO` / `EDIT` / `MULTI_SELECT` are read, and the
shell reads only `MULTI_SELECT`** (`panels/forms/rows.rs:771`). A combo box and a
list box are therefore indistinguishable to this shell unless multi-select is
set.

---

## 1. Authoring / placement

### 1.1 Commands

| Kind | Command id | Definition | Enable condition | Icon / label |
|---|---|---|---|---|
| Text | `edit.form_text_field` | `shell/commands/catalog/edit.rs:378` | `doc.pages` | `form-field` |
| Check box | `edit.form_check_box` | `catalog/edit.rs:381` | `doc.pages` | — |
| Radio | `edit.form_radio_button` | `catalog/edit.rs:384` | `doc.pages` | — |
| Combo **and** list box | `edit.form_choice` | `catalog/edit.rs:387` | `doc.pages` | label "Drop-down", icon `drop-down` |
| Push button | `edit.form_push_button` | `catalog/edit.rs:425-427` | `forms.push_button_runnable`, set at `app/conditions/mod.rs:154` inside `if !doc.pages.is_empty()` — equivalent to `doc.pages` | — |
| **List box (own command)** | **ABSENT** — would be a sixth entry beside `catalog/edit.rs:387` and a sixth `FormFieldKind` variant at `canvas/formfield.rs:60-85` | | | |
| **Signature** | **ABSENT** — a sixth `FormFieldKind` variant plus an `add_signature_field` verb the engine does not have (`text/fieldclip.rs:69`) | | | |

Two non-placing verbs: `edit.form_manage_fields` `catalog/edit.rs:435` (routes to
`view.panel_forms`, `app/dispatch/routes.rs:62`) and `edit.form_flatten`
`catalog/edit.rs:441`.

Arming: `app/dispatch.rs:976-983` → `app/dispatch/forms.rs:66-92` (requires
`edit_content`, else traces `command-declined … reason=mode-cannot-edit-content`)
→ `canvas/tool/arm.rs:285-295` `arm_form()`. Tool variant
`canvas/tool/mod.rs:425` `Form(FormFieldKind)`. Command→kind map
`shell/commands/mapping.rs:381-382`.

### 1.2 Default geometry and naming

| Kind | Default size (pt) | Name prefix | Ghost preview |
|---|---|---|---|
| Text | 160 × 20 `canvas/formfield.rs:159-163` | "Text" `:199-205` | identical 1.5 px rect for every kind `canvas/formfield/ghost.rs:79-127` |
| Check box | 14 × 14 | "Check Box" | same |
| Radio | 14 × 14 | "Group" | same |
| Combo / list box | 160 × 20 | "Dropdown" | same |
| Push button | 80 × 22 | "Button" | same |

**No kind gets a distinguishing ghost** — `ghost.rs:79-127` draws a
selection-ink rectangle only, so a check box and a text field look identical
while placing.

### 1.3 Placement dialog rows, per kind — `dialogs/formfield.rs`

Kind fork at `:565-573` `fn specific()`.

| Row / option | TX | CB | RB | CO | LB | PB | Citation |
|---|---|---|---|---|---|---|---|
| Name `/T` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:505-545`, `NAME_MAX=120` |
| Tooltip `/TU` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:548-562`, `TOOLTIP_MAX=240` |
| Initial value `/V` | ✅ | ✅ (checked) | ✅ (selected) | ABSENT | ABSENT | N/A | TX `:578`; CB `:620`; RB `:644` |
| Multiline `/Ff`13 | ✅ | — | — | — | — | — | `:584` |
| Password `/Ff`14 | ✅ | — | — | — | — | — | `:585` |
| Max length `/MaxLen` | ✅ checkbox + `DragValue 1..=1000` | — | — | — | — | — | `:589-599` |
| Comb `/Ff`25 | ✅ gated on `comb_ok()` | — | — | — | — | — | `:610-615`; gate `canvas/formfield/draft.rs:176-178` (Text + `max_len>0`) |
| Export value (on-state) | — | ✅ default `"Yes"` | ✅ auto-increments per sibling | — | — | — | `:623`; `:640`; `draft.rs:196-230`, `:263-293` |
| Group note | — | — | ✅ | — | — | — | `:637` |
| **Option list `/Opt`** | — | — | — | **✅ multiline TextEdit, 4 rows** | **✅ same** | — | `:648-655`; blank lines dropped `draft.rs:161-168` |
| Combo vs list radio pair | — | — | — | ✅ | ✅ | — | `:661-662` |
| Editable combo `/Ff`19 | — | — | — | ✅ | **absent, not greyed** (`editable` forced `false`) | — | `:664-666` (R9) |
| Multi-select `/Ff`22 | — | — | — | **absent** | ✅ | — | `:667-669` |
| Sort `/Ff`20 | — | — | — | ✅ | ✅ | — | `:673` |
| Caption `/MK /CA` | — | — | — | — | — | ✅ | `:680-681` |
| Button action `/A` (7 kinds) | — | — | — | — | — | ✅ | `:682` → `dialogs/buttonaction.rs:67-138` |
| Required `/Ff`2 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:722` |
| Read-only `/Ff`1 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:724` |
| Border width `/BS /W` `0..=12` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:729-733` |
| **Border style `/BS /S`** | **ABSENT for every kind** — hard-coded `BorderStyle::Solid` | | | | | | `app/actions/forms/author.rs:77-80`; would go at `:729` |
| Background `/MK /BG` | ✅ | ✅ | ✅ round swatch | ✅ | ✅ | ✅ | `:748-767`; `disc = kind == Radio` `:746` |
| Border colour `/MK /BC` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `:769-791` |
| **Alignment `/Q`** | **ABSENT at placement** (available later in Properties) | | | | | | would go at `:576-616`; ENGINE:`edit.rs:21825` |
| **Font / size / colour `/DA`** | **ABSENT at placement** (available later) | | | | | | would go at `:576-616`; ENGINE:`edit.rs:21287` |
| **Check-box glyph style** | — | **ABSENT** | — | — | — | — | would go at `:619-627` + a `Draft` field at `draft.rs:66-151`; engine has it at ENGINE:`annot_author.rs:3826-3836` |
| **Rotation `/MK /R`** | **ABSENT at placement for every kind** | | | | | | nothing in `author.rs:51-375` writes `/MK /R` |

Accept gate `:489-497` (`is_authorable()` = name non-empty, `draft.rs:187-189`).
Per-kind window height `:258-276`. Harness hook `PDFCER_DIAG_FORM_ACCEPT`
`:807-810`.

**The placement dialog offers strictly more per-kind control than the Properties
panel does** — it is the only `/Opt` editor, the only sort / editable /
multi-select author point, and the only button-caption author point.

### 1.4 What the write path emits — `app/actions/forms/author.rs`

| Kind | Engine spec | Arm | Post-steps |
|---|---|---|---|
| Text | `NewTextField` | `:120-140` | — |
| Check box | `NewCheckBox` | `:141-155` | — |
| Radio | `NewRadioButton` (joins existing `/T`) | `:156-171` | sibling export auto-numbered `draft.rs:263-293` |
| Combo / list | `NewChoiceField` + `combo`/`editable`/`multi_select`/`sort` | `:172-187`, `add_choice_field` `:187` | `/Opt` via `ChoiceOption::plain` `:172-176` — **export and display deliberately identical; no second column** (comment `:172-175`) |
| Push button | `NewPushButton` then `set_button_action` `:219` | `:189-297` | two edits folded by `coalesce_last(2, AddFormField)` `:268-272` |
| Signature | **no arm exists** | — | — |

Common: `BorderSpec { style: Solid, width }` `:77-80`;
`WidgetChrome::new(background, border_color)` `:117`; the placed field becomes
`doc.selected_field` `:368-374`.

### 1.5 Push-button actions (PB only)

`canvas/formfield/action.rs:90-97` `ALL: [Self; 7]` — Nothing, ResetForm,
GoToPage, Named, ShowHide, Uri, SubmitForm. Chooser `dialogs/buttonaction.rs:72-94`,
per-kind rows `:118-128`, `:146-170`, `:173-182`, `:192-206`, `:209-212`, `:227+`.
Blockers `action.rs:233-283`. Conversion `action.rs:320-352`. Reach-outside
disclosure `action.rs:109-111`.

**Stale-count defect:** the doc comment at `action.rs:64-79` says "six" directly
above `ALL: [Self; 7]`.

---

## 2. The Properties panel

37 rows: `panels/properties/mod.rs:473` → `panels/properties/formfield.rs:266-406`
(early returns `:272-285`). FF = `panels/properties/formfield.rs`,
FE = `…/fieldedit.rs`, WE = `…/widgetedit.rs`, MK = `…/mkcolour.rs`,
SW = `…/swatch.rs`, TP = `text/panels/formfield.rs`.

### 2.1 Read-only rows (all kinds)

Heading FF:350 · Name FF:414 · Type FF:415 (strings TP:104-123) · Page
FF:417-421 · Box count when >1 widget FF:427-434 · Value FF:435-437 · `/Ff`
flag line FF:441-443 (wording TP:154-195).

### 2.2 Field-scope editable rows — `fieldedit.rs`

| Capability | Key | TX | CB | RB | CO | LB | PB | SG | Citation |
|---|---|---|---|---|---|---|---|---|---|
| Required | `/Ff`2 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | FE:150-162 |
| Read only | `/Ff`1 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | FE:163-176 |
| Tooltip | `/TU` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | FE:290, 487-514 (**publishes no diag region**) |
| Multiline | `/Ff`13 | ✅ | — | — | — | — | — | — | FE:187-200, gate FE:186 |
| Password | `/Ff`14 | ✅ | — | — | — | — | — | — | FE:201-214 |
| Max length | `/MaxLen` | ✅ `0..=32767`; 0 ⇒ row absent | — | — | — | — | — | — | FE:215, 371-408 (`:393`) |
| Comb | `/Ff`25 | ✅ (also forces `/MaxLen`=10 at FE:474) | — | — | — | — | — | — | FE:216, 427-463 |
| Default value | `/DV` | ✅ | — | — | — | — | — | — | FE:223, 544-581; empty ⇒ `clearing_default_value()` FE:566-570 |
| Alignment | `/Q` | ✅ | — | — | — | — | — | — | FE:224, 608-636; `ALL_QUADDINGS` FE:975 |
| No-toggle-to-off | `/Ff`15 | — | — | ✅ only when >1 widget | — | — | — | — | FE:228-243, gate FE:228 |
| Drop-down (display) | `/Ff`18 | — | — | — | ✅ | ✅ | — | — | FE:247-260 |
| Allow several | `/Ff`22 | — | — | — | ✅ | ✅ | — | — | FE:261-274 |
| Font | `/DA` | ✅ | — | — | ✅ | ✅ | ✅ | — | FE:700-745 over `Std14::ALL` FE:721-727; embedded face shown-not-selectable FE:867-874; group gate FE:283-286 |
| Size | `/DA` | ✅ | — | — | ✅ | ✅ | ✅ | — | FE:753-791 (≤0 renders "Auto" FE:764-775) |
| Text colour | `/DA` | ✅ | — | — | ✅ | ✅ | ✅ | — | FE:801-847 + SW:140-146, 168, 203 |

### 2.3 Widget-scope editable rows — `widgetedit.rs`

**`widgetedit.rs` contains zero kind branches** apart from one cosmetic one
(WE:689). Every row below is drawn for **all seven kinds including Signature**.

| Capability | Key | Citation | Note |
|---|---|---|---|
| X / Y / W / H + Apply | `/Rect` | WE:211, 363-424 (`with_rect`+`with_resize` WE:411-418) | |
| **Turn left / Turn right** | `/MK /R` | WE:223, 284-346; buttons WE:337-338 | **no-op for CB / RB / PB — see §5** |
| Border style | `/BS /S` | WE:225, 465-499 (list WE:456-462) | 5 styles |
| Border width | `/BS /W` | WE:507-532 | **row not drawn at all when `widget.border` is `None`** WE:504-506 |
| Background | `/MK /BG` | WE:230, 692-716; MK:43-108 | round swatch when `button_kind == Radio` WE:689 (cosmetic only) |
| Border colour | `/MK /BC` | WE:230, 718-743 (`no_colour_entry: None` WE:724) | |
| Visibility | `/F` | WE:232, 546-591 — one four-value combo WE:554-559 | replaced by a hex sentence when the bits are unmappable WE:561-565 |
| **Caption** | `/MK /CA` | WE:234, 604-629; hint *"The words on a button"* TP:723 | ★ **for a check box this is the glyph-style chooser** — §2.5. Publishes no diag region |
| Delete field / Delete this box | — | FF:378, 597-647 | gated by certification |

`WE:47-55` records that `WidgetEdit` carries **seven** properties (rect, resize,
border, border_color, background, caption, visibility) and names the stale-count
defect class.

### 2.4 Rename

All seven kinds: TextEdit + Button + Enter — FF:486-566 (`:504-508`, `:541`,
`:556`, `FieldAction::Rename` `:557-565`); refusal sentences FF:332, FF:493-496.

### 2.5 ★ The Caption row is the check-box glyph chooser (undocumented, working)

Chain, all engine-side:

1. `/MK /CA`'s **first byte** selects the glyph — ENGINE:`annot_author.rs:3826-3836`:
   `4`→Check, `8`→Cross, `H`→Star, `l`→Circle, `n`→Square, `u`→Diamond; anything
   else → `None`.
2. `build_button_states` recovers it for the Check arm and repaints —
   ENGINE:`edit.rs:38563-38569`, falling back to `Check` on an unrecognised char.
3. A caption edit stages it: `edit_widget` builds
   `PendingWidgetEdit { caption: edit.caption.clone(), … }` ENGINE:`edit.rs:~25340-25355`;
   the comment at ENGINE:`edit.rs:25356` names caption change as one of five
   appearance invalidators.
4. `regen_button_appearance` reads `pending.caption_for(widget.id)`
   ENGINE:`edit.rs:38392`, then calls `build_button_states` ENGINE:`edit.rs:38402`.

⇒ **`widgetedit.rs:604-629` is a functioning check-box style chooser labelled
"The words on a button".** This also refutes DOC:`ENGINE_BACKLOG.md:118`
(*"Creation only — the engine offers no restyle of an existing box"*).

### 2.6 Properties-panel ABSENT verdicts, with the insertion point

| Capability | Key | Kinds | Where it would go | Engine support? |
|---|---|---|---|---|
| **Option list editor** | `/Opt` | CO, LB | `fieldedit.rs:246-275` | ✅ `FieldEdit.options` ENGINE:`edit.rs:21446`, `with_options` ENGINE:`edit.rs:21815` |
| Editable-combo toggle | `/Ff`19 | CO | `fieldedit.rs:246-275` | ✅ ENGINE:`edit.rs:21424` — read-only display today (TP:187-189 *"free text allowed"*) |
| Sort toggle | `/Ff`20 | CO, LB | `fieldedit.rs:246-275` | ✅ ENGINE:`edit.rs:21437` |
| Commit on change | `/Ff`27 | CO, LB | `fieldedit.rs:246-275` | ✅ ENGINE:`edit.rs:21539` |
| Top index / selected indices | `/TI`, `/I` | CO, LB | `fieldedit.rs:246-275` | modelled but **never read by the panel** (enumerated TP:1404-1433) |
| Export values (display ≠ export) | `/Opt` pairs, `/AP /N` names | CB, RB, CO, LB | `fieldedit.rs:246-275` + a Button block beside FE:228-243 | partial — `ChoiceOption` has both columns; authoring collapses them (`author.rs:172-176`) |
| Rich text | `/Ff`26 | TX | `fieldedit.rs:186-225` | display only; convert-to-plain exists in the Forms panel |
| Do-not-scroll | `/Ff`24 | TX | `fieldedit.rs:186-225` | ✅ ENGINE:`edit.rs:21530` |
| Do-not-spell-check | `/Ff`23 | TX, CO | `fieldedit.rs:186-225` | ✅ ENGINE:`edit.rs:21878` |
| File select | `/Ff`21 | TX | `fieldedit.rs:186-225` | ✅ ENGINE:`edit.rs:21871` |
| No export | `/Ff`3 | all | beside FE:150-176 | ✅ ENGINE:`edit.rs:21857` |
| Radios in unison | `/Ff`26 (Btn) | RB | beside FE:228-243 | ✅ ENGINE:`edit.rs:21419` |
| Mapping name | `/TM` | all | beside FE:487-514 | ✅ ENGINE:`edit.rs:21551` |
| Clear alignment | `/Q` removal | TX | FE:608-636 | ✅ `clearing_quadding` ENGINE:`edit.rs:21835` — **no control** |
| **Format category** (number/date/percent/special) | `/AA /F`, `/K` | TX, CO | **no `/AA` surface exists anywhere** | — |
| **Validation range** | `/AA /V` | TX | same | — |
| **Calculation** | `/AA /C` | TX | same | — |
| **Keystroke script** | `/AA /K` | TX, CO | same | — |
| Check-box glyph (labelled) | `/MK /CA` | CB | relabel / fork `widgetedit.rs:604-629` | ✅ ENGINE:`annot_author.rs:3826`, ENGINE:`edit.rs:38563` |
| Button icon / layout | `/MK /I`, `/TP` | PB | `WidgetEdit` (WE:46-55) carries neither | not in `WidgetEdit` |
| Field locking | `/Lock` | all | — | DOC:`ENGINE_BACKLOG.md:117`: the flags verb *"refuses a `/Widget` by name, so the Forms surfaces must not grow a Lock control from it"* |
| Tab order (per field) | `/Annots` index | all | drag-only today, `panels/forms/tab_order/mod.rs:388-600` | §4.3 |
| No-view bit | `/F` bit 6 | all | `AnnotFlags::toggle_no_view` — **zero occurrences under `src/`** | ✅ engine has it |
| Field-type conversion | `/FT` | all | **deliberately absent**, argued at FE:72-79 | — |

**Every kind branch in the whole Properties tree, exhaustively:** FE:186 (Text),
FE:228 (Button && >1 widget), FE:246 (Choice), FE:283-286 (Text|Choice|Button),
WE:689 (Radio, cosmetic), plus label / flag strings TP:104-123 and TP:173-190.
**Nothing else forks on kind.**

---

## 3. Filling on the canvas

### 3.1 The gate

`canvas/present.rs:949` calls `forms::overlay(..., authoring = caps.edit_content, ...)`
before `interact` at `:1065`.

| Mode | Behaviour | Citation |
|---|---|---|
| **Edit** (`edit_content`) | click **selects only** — `selecting::select_click` → `FieldAction::Select` → `return`. **No filling of any kind, for any kind of field.** | `canvas/forms.rs:785-816`, stated `:773-778`; `canvas/forms/selecting.rs:127` |
| **Read / Review** | filling offered, subject to `offer()` | `canvas/forms.rs:818-877`; gate `:881-883` = `offered_in(tool) && doc.annotations_visible() && fill_refusal().is_none()`; `offered_in` = `matches!(tool, CanvasTool::Select)` `canvas/forms/boxes/mod.rs:325-327` |

There is **no `fill_forms` capability** — `app/modes/capability.rs:162` has only
`edit_content`, `author_markup`, `author_measure`; the operator decision is at
`:100-104` and Read grants *"Nothing but navigation and form filling"* `:196`.

### 3.2 The classifier — the mechanical answer to the drop-down complaint

`canvas/forms/boxes/mod.rs:88-163` defines **three** box kinds:

```rust
pub enum BoxKind {
    Text  { multiline, password, max_len, align },   // :91-142
    Check { on_state, on },                          // :144-149
    Radio { on_state, on },                          // :157-162
}
```

Doc at `:82-86`: *"Three variants, not five: choice fields and everything in
`NotOnCanvas::NotOffered` have no canvas gesture at all, so they are absent
rather than present-and-inert."*

`classify()` `boxes/mod.rs:414-473`:

| Kind | Arm | Result |
|---|---|---|
| Text | `(Some(Text), _)` `:427-450` | `Ok(BoxKind::Text{..})`, reading MULTILINE, PASSWORD, `/MaxLen`, `field.quadding` |
| Text, rich | `(Some(Text), _) if is_rich_text()` `:426` | `Err(NotOffered)` |
| Check / Radio | `(Some(Button), Some(Check\|Radio))` `:451-470` | `Ok`, or `Err(NotOffered)` at `:459` when `widget.on_states.first()` is `None` |
| **Combo** | **no arm** | `_ => Err(NotOffered)` `:471` |
| **List box** | **no arm** | `:471` |
| **Push button** | `ButtonKind::Push` has no arm | `:471` |
| **Signature** | no arm | `:471` |
| any, no `/AP /N` | `:415` | `Err(NoAppearance)` |
| any, blocked by the panel | `:418-420` | `Err(NotOffered)` |

**`(Some(FieldType::Choice), _)` appears nowhere in `canvas/`.** The only three
`FieldType::` mentions in the whole `canvas/` tree are `boxes/mod.rs:426`,
`:427`, `:451`.

### 3.3 Per-kind canvas gesture table

| Kind | Click sets value? | Cursor | Shade | Spotlight | Tab stop | Space/Enter | Arrows | Citation |
|---|---|---|---|---|---|---|---|---|
| Text | ✅ inline editor, caret at end | I-beam | ✅ | ✅ | ✅ | no-op | no-op | `forms.rs:1241-1262`; caret `:1127-1136`; cursor `:1371-1374`; Escape abandons `:1142-1150`; explicit no-op arm `canvas/forms/tabbing.rs:366` |
| Check box | ✅ toggles `"Off"` ↔ on-state | hand | ✅ | ✅ | ✅ | ✅ toggles | no-op | `forms.rs:1266-1274`; keys `tabbing.rs:307-308` → `:350-367`; lone box + arrows `tabbing.rs:400-402` |
| Radio | ✅ selects; clicking the on one is a deliberate no-op | hand | ✅ | ✅ | ✅ one stop per group | ✅ selects | ✅ moves **and** activates | `forms.rs:1276-1281`; group collapse `canvas/forms/ring.rs:97-136`, flagged `:228`; arrows `tabbing.rs:315`, `:320` |
| **Combo box** | **ABSENT** | none | none | **none** | **none** | — | — | `boxes/mod.rs:471`; reason `canvas/forms.rs:250-259`; the absence is asserted by a test at `canvas/forms/boxes/tests.rs:802`, `:812`, `:820` |
| **List box** | **ABSENT** | none | none | none | none | — | — | identical path |
| **Push button** | **ABSENT** (no activation gesture at all) | none | none | none | none | — | — | `boxes/mod.rs:471` |
| **Signature** | **ABSENT** | none | none | none | none | — | — | `boxes/mod.rs:471`; also pre-blocked `panels/forms/rows.rs:279` |

Where a combo box **would** have to be added: a fourth `BoxKind` variant at
`boxes/mod.rs:88-163`, a `(Some(FieldType::Choice), _)` arm at
`boxes/mod.rs:451-470`, a fourth arm in the `match &widget_box.kind` at
`forms.rs:1227-1282`, and an on-page popup surface. The reason it was declined is
written out at `canvas/forms.rs:250-259`: *"A choice field would need a dropdown
anchored to the page, which is a second popup surface with its own placement
rules and no gesture the panel does not already have."*

### 3.4 ★ The omission is silent — nothing anywhere tells the operator

| Would-be channel | Why it says nothing | Citation |
|---|---|---|
| `Routing.undrawn` counter | incremented only for `NoAppearance` | `boxes/mod.rs:623-631` |
| `Routing.unreachable` counter | incremented only for `RotatedPage \| NotPlaced` | `boxes/mod.rs:623-631` |
| Forms-panel routing note | prints from those two counters only | `panels/forms/mod.rs:499-522` |
| Cursor | `_ => PointingHand` never reached — no box exists | `forms.rs:1371-1374` |
| Field shading | no box ⇒ nothing painted | `canvas/form_marks.rs:147` |
| Spotlight from the Forms panel | cannot flash a field with no box | `canvas/form_marks.rs:82` |
| Tab ring | no stop | `canvas/forms/ring.rs:202-261` |

A drop-down **is** still selectable in Edit mode (`boxes/mod.rs:565-580` pushes
`targets` at `:572-580` independently of `classify`), so it can be moved,
resized, renamed and deleted — just never filled on the page.

### 3.5 Where option picking *does* work

Forms panel only: `panels/forms/rows.rs:810`
`egui::ComboBox::from_id_salt(("pdfcer-forms-choice", index))` → `selectable_label`
`:819` → `FormEdit::SetChoice { values: vec![export] }` `:820-823` →
`panels/forms/edit.rs:765` `session.set_choice_value(field, &refs)` — **the only
call to the engine's choice verb in the entire GUI**.

An **editable** combo box (`/Ff` EDIT) can receive a free-text value **nowhere in
the shell** — not on canvas, not in the panel (`rows.rs:749-835` offers only the
option list), not in Properties.

### 3.6 Click-ladder ownership

`canvas/selection/annot.rs:207` — rung 4 of the 12-rung ladder deliberately
excludes `/Widget`: *"the form field surface owns it — a click there focuses an
editor, and two owners of one press is how a field becomes unfillable."*

---

## 4. The Forms panel

One vertical stack, no tab strip: `panels/forms/mod.rs:203 pub fn body`.
`COMMAND_ID = "view.panel_forms"` `mod.rs:189`. Four collapsing headers, all
`default_open(false)`: Calculated `mod.rs:550-641`, Reset `mod.rs:663-719`, Tab
order `tab_order/mod.rs:269-272`, Field groups `groups.rs:217-224`. Always
visible: header `mod.rs:382-423`, fill disclosure `:449-469`, canvas routing
`:499-522`, whole-form buttons `:736-785`, fill list `:793-854`.

### 4.1 Fill list, per kind — `panels/forms/rows.rs`

Gate first (`rows.rs:163-184`), then the kind match (`rows.rs:186-204`).

| Kind | Row fn | Control | Citation |
|---|---|---|---|
| Text | `text_row` | multiline or `singleline().password()`; `/MaxLen` truncates live; `"{len}/{max}"` caption; mirrors the canvas draft; spotlights on focus | `rows.rs:188-195`, `:418-549`, `:465-472`, `:495-503`, `:448-453`, `:519-521`; region `forms.fill.row.{index}` `:67`, `:537-541` |
| Text, rich | `rich_text_row` | **disabled** box + "Convert to plain text…" → `ConvertRichTextToPlain` | `rows.rs:187`, `:333-391` |
| Check box | `check_row` | checkbox; **disabled + hover text when no on-state** | `rows.rs:196`, `:592-631`; `check_on_state` `:643-648` |
| Radio | `radio_row` | one `ui.radio` per distinct on-state in **widget order, never sorted**; `Clear` only when `!NO_TOGGLE_TO_OFF` | `rows.rs:197`, `:662-706`; `radio_states` `:717-728`; Clear `:694` |
| **Combo** | `choice_row` | ✅ `ComboBox` + `selectable_label` per `/Opt` entry | `rows.rs:198`, `:749-835`; picker `:810-823`; match accepts export **or** display `:804`, `:818`; `/V` not in `/Opt` ⇒ note `:828-834`; empty `/Opt` ⇒ note + return `:750-757` |
| **List box** | `choice_row` | ✅ checkbox stack when MULTI_SELECT, else the same ComboBox | `rows.rs:771-799` vs `:810` |
| Push button | blocked row + action editor | plain disabled label, then `panels/forms/button.rs:72-172` | `rows.rs:163-184`, `:280-283`; blocked render `:298-308` |
| Signature | blocked row, never fillable | `rows.rs:279` |
| any read-only | blocked row | `rows.rs:275` |

`block_reason` `rows.rs:274-286`. Flags the fill list reads: REQUIRED `:233`,
MULTILINE `:454`, PASSWORD `:455`, NO_TOGGLE_TO_OFF `:694`, MULTI_SELECT `:771`,
plus `read_only()` `:275` and `is_rich_text()` `:187`. **It never reads
`FieldFlags::COMBO` (18) or `EDIT` (19).**

### 4.2 `FormEdit` — the complete write vocabulary (eight variants)

`panels/forms/edit.rs:299-406`: `FillText` `:305`, `ConvertRichTextToPlain`
`:324`, `SetButtonState` `:341`, `SetChoice` `:357`, `Recompute` `:381`, `Reset`
`:391`, `RegenerateAppearances` `:400`, `Flatten` `:405`. Applied only by
`edit::apply` `:445-541` (four-step protocol `:24-60`); labels `:421-432`;
`Reset` → `reset_form(None)` `:809`; `Flatten` → `flatten_fields(None)` `:827`.

### 4.3 Tab-order list — kind-agnostic, drag-only

| Property | Value | Citation |
|---|---|---|
| Scope | one block per page | `tab_order/mod.rs:388-600`; `MAX_LIST_HEIGHT = 260.0` `:157` |
| Row text | `"{position}. {label}"` | `:496-505` |
| Reorder gesture | **drag only** | `Sense::click_and_drag()` `:497-505`; `drag::consider` `drag.rs:266-303`; 2 pt caret at 0.35 alpha `drag.rs:98`, `:107`, `:239-258`; `drag::settle` → `FieldAction::ReorderAnnotations` `drag.rs:322-368`; landing rule `drag.rs:226-228` |
| Up/down buttons, numeric renumber, keyboard route | **ABSENT** — asserted absent by a test | `mod.rs:991-1006` `no_row_carries_a_labelled_reorder_button`; the accessibility gap named at `:976-989` |
| **Kind filter** | **none** — no `block_reason` call anywhere in the tab-order code; every `/Widget` a listed field claims is included regardless of kind, read-only, rich text or certification | `model.rs:511-589` |
| Cross-page drag | not offered | `drag.rs:56-60`, `mod.rs:460` |
| `/Tabs` | read, **never written** | `drag.rs:41-43`; note printed per page `mod.rs:623-640` |
| Non-widget `/Annots` entries | keep their index | `drag.rs:188-216` |
| Exclusion counters (4) | `fields_without_widgets` `model.rs:239`/`:494`, `unclaimed` `:301`/`:556-562`, `anonymous` `:304`/`:533-542`, `other_annots` `:331`/`:526-529` | `model.rs:171-186` |
| Adopt an unclaimed widget | ✅ `FieldAction::Adopt`, label and enabled-ness driven by `session.adopt_preview` | `register.rs:196-379`, `:276-312`; refusal hints `:423-446` |

### 4.4 Field groups — delete only

`groups.rs:328-383` rows, `:409-448` disclosure, `MAX_LISTED_NAMES = 8`; two
presses (`ArmGroupDeletion(Some(name))` → `DeleteGroup { group }` / Cancel);
renders nothing when empty `:206-211`; refusal path `:195`, `:237-252`. **No
create, rename, merge or split anywhere.**

### 4.5 Push-button action editing (PB only)

`panels/forms/button.rs:72-172` — current-action sentence `:97-112`, `Change…`
`:128-148`, shared rows `:153`, Apply gated on `does.blocker().is_none()`
`:159-171` → `FieldAction::SetButtonAction`. A `Foreign` action renders nothing
`:124-126`. Classification `:26-47`, `:71-80` (reads
`doc.session.button_action(&fqn)`).

### 4.6 Forms-panel ABSENT list

Rename (`rows.rs:42-49`, `mod.rs:126-133`), delete field, delete widget, create
field, duplicate, move / resize / rotate, read-only toggle, required toggle,
tooltip editing, **`/Opt` editing**, clearing a choice field back to unset, free
text for an editable combo, per-field reset / flatten / regenerate, rich-text
editing, group create / rename / merge / split, radio option add / remove /
reorder, RADIOS_IN_UNISON toggle, FDF/XFDF/CSV in-panel (shipped instead as
`file.export_form_data` / `file.import_form_data`), row virtualisation
(`mod.rs:135-143`).

---

## 5. ★ Rotation: the check-box no-op, mechanically

### 5.1 The controls

| Control | Offered for | Gate | Citation |
|---|---|---|---|
| Turn left (+90) | **every kind, including Signature** | **none** — plain `ui.button`, no `add_enabled`, no kind argument | `widgetedit.rs:337`; built `:300`; called unconditionally `:223`; entered from `formfield.rs:368` with no kind argument |
| Turn right (−90) | same | none | `widgetedit.rs:338` |
| Canvas rotate grip | **never for a widget** | `GripSet::scale_only()` — *"The eight and NOT the ninth"* | `canvas/rotating.rs:539-551`, `canvas/pressing.rs:279-296` |

Math: `let next = (current + delta).rem_euclid(360)` `widgetedit.rs:317`,
`current = widget.rotation.unwrap_or(0)` `:291`. Regions `:144`, `:153`, `:155`.
Strings `text/panels/formfield.rs:1120`, `:1126`, `:1137`, `:1143`.

### 5.2 The chain

`widgetedit.rs:337-338` → `FieldAction::RotateWidget` (`app/actions/forms.rs:229`)
→ dispatch `app/actions/forms.rs:602-606` → `app/actions/forms/widget.rs:28-51`
(`session.rotate_widget`, traces `rotate-widget-applied … regenerated={} siblings={}`,
pushes `widget_rotation_stale(why)` only when `report.appearance_stale` is `Some`)
→ ENGINE:`edit.rs:24982-25085`.

### 5.3 What the engine does

| Step | Behaviour | Citation |
|---|---|---|
| Quarter-turn check | non-multiples of 90 refused | ENGINE:`edit.rs:24992-24994` |
| `/MK /R` write | **succeeds for every kind** — inserts `R` `:25026`, removes it when 0 `:25024`, drops an emptied `/MK` `:25030`, and **creates `/MK` when absent** via `.unwrap_or_default()` `:25019-25023` | ENGINE:`edit.rs:25019-25032` |
| `/Rect` | **never touched, by design** | ENGINE:`edit.rs:25040-25044`, rationale `:24906` |
| Repaint | `regen_after_property_change(…, PendingWidgetEdit { rotation: Some(quarter), .. }, None)` | ENGINE:`edit.rs:25046-25057` |
| Staleness sentence | set only when the repaint returns `false` | ENGINE:`edit.rs:25059-25069` |
| Erratum #56 | quoted in the header | ENGINE:`edit.rs:24932-24940` |

### 5.4 The defect

`regen_after_property_change` ENGINE:`edit.rs:24758-24808` forks by `/FT`:

| `/FT` | Destination | Reads the staged rotation? |
|---|---|---|
| `/Tx` | `regen_field_appearance` ENGINE:`edit.rs:24773` | ✅ `pending.rotation_for(widget.id).or(widget.rotation)` ENGINE:`edit.rs:38069-38071`, with the `(w,h)` swap for 90/270 at `:38090-38094` |
| `/Ch` | same ENGINE:`edit.rs:24783-24786` | ✅ same |
| **`/Btn`** | **`regen_button_appearance` ENGINE:`edit.rs:24805-24807`** | ❌ **never calls `rotation_for`** — it reads `pending.rect_for` `:38389`, `pending.caption_for` `:38393`, `pending.chrome_for` `:38400`, then `build_button_states(field, kind, w, h, &caption, chrome)` `:38402`, whose signature ENGINE:`edit.rs:38534-38549` **takes no angle** |
| `/Sig` / other | `_ => return Ok(false)` ENGINE:`edit.rs:24808` | ❌ |

**`PendingWidgetEdit::rotation_for` has exactly one call site in the whole engine:
ENGINE:`edit.rs:38070`.** (`PendingWidgetEdit` ENGINE:`edit.rs:18415-18476`,
`rotation` field `:18451`, accessor `:18471`. `edit_widget` stages
`rotation: None`, ENGINE:`edit.rs:~25351`.)

Underneath, a check-box `/AP` is structurally incapable of carrying a rotation
even if it wanted to: ENGINE:`annot_author.rs:32-33` fixes the invariant *"every
appearance here is authored with `Matrix` = identity and `BBox` = the annotation
`/Rect`"*, and `CheckBoxStateAppearance::ap_dict` ENGINE:`annot_author.rs:3701`
emits `/BBox [0 0 w h]` with no `/Matrix`. The renderer implements §12.5.5 from
`/BBox` + `/Matrix` only — ENGINE:`pdfcer-render/src/annot.rs:832-845` — and
`grep 'b"MK"'` across `pdfcer-render` returns nothing.

### 5.5 Per-kind rotation verdict

| Kind | `/MK /R` written? | Artwork rotates? | Operator sees | Citation |
|---|---|---|---|---|
| Text | ✅ | ✅ | correct rotation | ENGINE:`edit.rs:24773` → `:38069-38071` |
| Combo / list | ✅ | ✅ | correct rotation | ENGINE:`edit.rs:24783-24786` |
| **Check box** | ✅ | **❌** | **nothing moves; the status bar says "Turned to 90° anticlockwise."** | ENGINE:`edit.rs:24805-24807`, `:38402` |
| **Radio** | ✅ | ❌ | same | same |
| **Push button** | ✅ | ❌ | same (a rotated caption would be the visible tell) | same |
| **Signature** | ✅ | ❌ (`Ok(false)`) | staleness caveat **is** shown | ENGINE:`edit.rs:24808`; caveat `app/actions/forms/widget.rs:28-51` |

Two observable outcomes for a `/Btn`:

- **pdfcer's own artwork** → the rebuild succeeds byte-identically → `Ok(true)`,
  `appearance_stale = None` → **silent success, zero pixels moved.** This is the
  operator's complaint.
- **foreign artwork** → `Ok(false)` → `appearance_stale` set → the caveat is
  shown. Pinned by
  `D:\Dev\pdfcer\crates\pdfcer-cli\tests\rotate_widget.rs:418-447`
  `a_widget_pdfcer_cannot_redraw_is_rotated_and_disclosed`.

### 5.6 Hypotheses tested

| Hypothesis | Verdict |
|---|---|
| (a) an absent `/MK` makes the write a no-op | **REFUTED** — `.unwrap_or_default()` creates it, ENGINE:`edit.rs:25019-25023` |
| (b) the `/AP /N` state subdictionary is the blocker | **PARTLY CONFIRMED** as the reason for the `/Btn` fork, but not the cause of failure — the subdictionary path is handled correctly at ENGINE:`edit.rs:38484-38519` |
| **(c) `regen_button_appearance` never reads the staged rotation** | **✅ CONFIRMED — this is the defect** |
| (d) a kind-dependent enable predicate greys the button | **REFUTED** — no predicate exists, `widgetedit.rs:284-346` |
| (e) a square `/Rect` hides the change | **CONFIRMED as fact, REFUTED as cause** — text fields have the same untouched `/Rect` (ENGINE:`edit.rs:25040-25044`) and rotate visibly |

**Second-order finding:** even after (c) is fixed, only Check, Star and Diamond
would show a visible 90° change — Circle (ENGINE:`annot_author.rs:4186`) and
Square (`:4197`) are rotationally symmetric, and Cross (`:4155`) is 90°-symmetric.

**The fix is engine-side.** The GUI needs no change. A stale comment in the GUI
claims otherwise: `canvas/pressing.rs:284-287` still says the rotate verb *"does
not exist yet"*.

---

## 6. Tests and driven checks

### 6.1 Driven `ui-verify` checks touching form fields

| CLI name | File | Kind driven | Last sweep |
|---|---|---|---|
| `form_field` | UV:`checks/form_field.rs:259` (name `:262`) | **Text** — arm, place, select, then Properties required `:199`, default value `:213`, alignment `:221`, typed geometry `:251`+`:253` | **PASS** |
| `right_clicking_a_form_field_opens_its_menu` | UV:`checks/field_menu.rs:108` | Text | **PASS** |
| `field_delete_gate` | UV:`checks/field_delete_gate.rs:202` | **Signature** (certified vs ordinary) | **PASS** |
| `field_group_delete_removes_the_subtree` | UV:`checks/form_groups.rs:230` | Text tree | **PASS** |
| `structural_refusals_are_sentences_not_controls` | UV:`checks/form_groups.rs:256` | Text + Signature | **PASS** |
| `fillable_fields_are_shaded_on_the_page` | UV:`checks/field_shading.rs:101` | Text + Check box (observation only) | **PASS** |
| `a_placed_button_can_be_given_something_to_do` | UV:`checks/button_action.rs:113` | **Push button** | **PASS** |
| `dragging_a_form_field_moves_it` | UV:`checks/widget_move.rs:92` | Text | SKIP (env) |
| `a_resized_check_box_is_redrawn_not_stretched` | UV:`checks/checkbox_resize.rs:110` | **Check box** | SKIP — **substantive**: the drag-out reached `add-form-field` but at 50×35 px the eight 8-px grips overlap |
| **`turning_a_field_right_turns_it_right`** | UV:`checks/widget_rotate.rs:115` | **Text only** — places a text field `:204-210`, scrolls to the rotation row `:225-250`, clicks Turn right `:253`, asserts the trace `now` contains `"270"` `:272-286`. **Never a check box; never a pixel.** | SKIP (env) |
| `a_form_field_can_be_copied_and_pasted_both_ways` | UV:`checks/field_clipboard.rs:230` | Text | SKIP (env) |
| `the_acrobat_paste_order_swaps_which_chord_does_which` | UV:`checks/field_clipboard.rs:208` | Text | SKIP (env) |
| `tab_order_drag_moves_a_field_and_shows_where` | UV:`checks/tab_order_drag.rs:128` | Text/Check rows | SKIP (env) |
| `clicking_a_form_row_lights_the_field_on_the_page` | UV:`checks/forms_spotlight.rs:102` | Text | SKIP (env) |
| `a_field_too_small_for_its_text_says_so` | UV:`checks/autosize_overflow.rs:134` | **Text — the only driven fill-value check in the project** | SKIP (env) |
| `exporting_form_data_writes_a_file` | UV:`checks/export_form_data.rs:93` | fixture-dependent | SKIP — **substantive**: `export-form-data-declined reason=no-acroform` |
| `adopt_widget_puts_a_form_control_back` | UV:`checks/adopt_widget.rs:118` | Text (orphan) | SKIP (env) |
| `an_invalidating_save_is_warned_about` | UV:`checks/signature_save.rs:139` | Signature | SKIP (env) |
| `tab_moves_between_form_fields` | UV:`checks/tab_navigation.rs:78` | Text | never swept (added post-sweep) |
| `the_outline_follows_the_pointer_and_is_what_gets_placed` | UV:`checks/form_ghost.rs:89` | Text | never swept |

Helper, not a check: UV:`checks/formaim.rs:110` `WidgetBox`.

**★ No driven check anywhere arms `edit.form_radio_button`, `edit.form_choice`,
`edit.form_manage_fields` or `edit.form_flatten`.** Radio, combo and list box
have **zero** driven coverage of any kind.

**Excluded as false positives** ("form" = Form XObject, not AcroForm):
UV:`checks/form_selection.rs`, `form_leaf_move.rs`, `form_leaf_descend.rs`,
`unshare_form.rs`, `geometry_fields.rs`, and the whole `forms-xobject/` fixture
family. Locally, `fixtures/form-xobject.pdf` and
`fixtures/annots-with-everything.pdf` carry no widget despite their names.

### 6.2 Unit-test coverage by kind × capability

`D` = driven + unit, `U` = unit/integration only.

| Kind | Placement | Properties edit | Fill value | Rotation | Tab order | Delete |
|---|---|---|---|---|---|---|
| **Text** | **D** | **D** UV:`form_field.rs:196-253`; U `fieldedit.rs:1114-1206`, `widgetedit.rs:915-955` | **D** (SKIP); U `rows.rs:865-912`, `edit.rs:1010`, `boxes/tests.rs:526`/`:549`/`:568` | **D** (SKIP); U `ghost.rs:172`, `boxes/tests.rs:210` | **D** (SKIP + unswept); U `ring.rs:322-403`, `tabnav.rs:294-341` | **D** PASS ×3; U `delete.rs:387-444`, `keys/tests.rs:229-368` |
| **Check box** | **D** partial (drag-out only) | **U only** `dialogs/formfield.rs:952`, `draft.rs:374`/`:395` | **U only** `boxes/tests.rs:324`, `:445` | **U only** `ghost.rs:172`, `boxes/tests.rs:210` | **U only** (via `demo-form`) | **U only**, kind-agnostic |
| **Radio** | **UNCOVERED driven**; U `draft.rs:356` | **UNCOVERED** | **U only** `boxes/tests.rs:294` | **U only** `ghost.rs:172` | **U only** `ring.rs:353`, `:369` | **UNCOVERED** |
| **Combo** | **UNCOVERED driven**; U `draft.rs:435` | **UNCOVERED** | **U only** `boxes/tests.rs:812` | **U only** `ghost.rs:172` | **UNCOVERED** | **UNCOVERED** |
| **List box** | **UNCOVERED everywhere** — `Draft::fresh` sets `combo: true` (`draft.rs:220`) and **no test anywhere sets it false** | UNCOVERED | UNCOVERED | nominal only | UNCOVERED | UNCOVERED |
| **Push button** | **D** PASS | **D** PASS (action wiring); U `panels/forms/button.rs:267-318`, `formfield/action.rs:432-545` | N/A | **U only** `ghost.rs:172` | **U only** | **U only** (orphaning) |
| **Signature** | **N/A — not authorable** | **UNCOVERED** | **U only** (signing, not filling) `sign/tests.rs` ×18 | **UNCOVERED** | **U only** | **D** PASS |

Three facts the matrix turns on:

- **The only all-kinds rotation test in the repo is `canvas/formfield/ghost.rs:172`**
  (loops `FormFieldKind::ALL` × every page rotation) — and it tests the *ghost
  rect*, not the rotate verb.
- **The only driven fill-value check is `autosize_overflow`** (Text), and it
  skipped environmentally.
- **The check-box rotation defect in §5 is entirely uncovered** —
  `widget_rotate.rs` uses a text field and asserts a trace string.

### 6.3 Fixtures

Local, 13 of 68 PDFs carry fields: `all-field-kinds.pdf` (★ all seven kinds, 9
fields / 10 widgets, authored 2026-09-16 with a `.PROVENANCE.py`, and
**referenced by nothing in the repo**), `three-text-fields.pdf`,
`text-field-with-appearance.pdf`, `autosize-field.pdf` (`/Helv 0 Tf`),
`action-names-field.pdf`, `submit-button.pdf`, `orphan-widget.pdf` (no
`/AcroForm` at all), `certified-nested-form.pdf`, `certified-comments.pdf`,
`threaded-comments.pdf`, `signed-two-pages.pdf`, `encrypted-aes-128.pdf`.

Engine corpus (`D:\Dev\pdfcer\fixtures\synthetic\forms\`, read-only):
`radio-choice-form.pdf` is the richest — radio ×3 kids `/Ff 49152`, combo
`/Ff 131072` with export/display pairs, multi-select `/Ff 2097152`, plus text —
**and is referenced nowhere in this repo**. Also `demo-form.pdf`,
`nested-form.pdf`, `radio-group-form.pdf`, `unfillable-fields-form.pdf`,
`multi-widget-form.pdf`, `rich-field-form.pdf`, `js-carriers-form.pdf`,
`xfa-hybrid-form.pdf`, `mixed-kids-form.pdf`, `group-delete-*.pdf`.

**The radio / combo / list-box driven checks could be written today without
authoring a fixture.**

---

## 7. Adjacent surfaces that touch form fields

| Surface | Per-kind behaviour | Citation |
|---|---|---|
| **Context menu** | two items, **kind-agnostic**: `format.properties` + `format.delete` (the latter `shown_when DELETE_PERMITTED`) | `shell/menus.rs:641-644`; menu id `:215`; chosen at `canvas/menus.rs:477-491`; gates `:373-377` |
| **Clipboard** | copy/cut/paste carries `/DA`, `/Q`, `/DV`, `/AA`, `/MK` colours, `/BS`, `/Ff` (incl. DoNotSpellCheck, DoNotScroll, FileSelect, RichText, CommitOnSelChange) and the baked `/AP` for **every kind**; radio groups travel whole and translated; **an unsigned signature field copies and pastes normally — de-facto signature-field authoring the placers do not offer**; a signed one is refused at the copy, by the engine | `canvas/fieldclip.rs:40-110`, `:68-72`, `:101-110`, `:238-267`, `:273-300`, `:321+` |
| **Settings** | two form knobs, kind-agnostic: widget tab tail (`ArrayOrder`/`RowOrder`) and row tolerance (pt slider) | `dialogs/settings/forms.rs:36-57`, `:70-86` |
| **Signing** | `AddSignatureField` command kind exists in the sign module (`SigField { name, page, invisible, lock }`) but no engine verb backs it; empty `/Sig` fields are read and signable | `sign/mod.rs:203`, `:400-425`, `:494`, `:505`; `text/fieldclip.rs:69` |
| **Comments panel** | a `/Widget` is excluded and counted | `panels/comments/model.rs:526` |
| **Pick-read** | form fields can never reach it | DOC:`DEFECTS.md:993-998` (D49) |

---

## 8. Documented claims the source contradicts (14)

| # | Claim | Where | Contradicted by |
|---|---|---|---|
| C1 | "all four of `WidgetEdit`'s box properties" | DOC:`FEATURES.md:449` | there are **seven** — `widgetedit.rs:47-55`, which names this exact defect class |
| C2 | ⬜ "Push buttons can be placed and can never do anything … Reset only … Submit is argued against" | DOC:`FEATURES.md:455` | **every clause false** — `canvas/formfield/action.rs:90-97` (`ALL: [Self; 7]` incl. Uri and SubmitForm), `:320-352`, `panels/forms/button.rs:71-80`/`:164`, UV:`checks/button_action.rs:1`/`:117`, `catalog/edit.rs:425-427` + `app/conditions/mod.rs:154`. Also self-contradicts DOC:`ENGINE_BACKLOG.md:299` and DOC:`MANUAL.md:677-690`, which already say "seven ways" |
| C3 | "The circle is offered, and it routes by kind" | DOC:`ENGINE_BACKLOG.md:304` | no circle at all (`canvas/pressing.rs:279-296` `GripSet::scale_only()`), and `rotation_row` `widgetedit.rs:284-346` has **no kind gate whatsoever**. DOC:`FEATURES.md:205` says the opposite and is correct |
| C4/C5 | the five tab-sequence verbs are `wanted` | DOC:`EDITABLE_SURFACES.md:148`, `:159` | all reached — `canvas/forms/ring.rs:271`; `dialogs/settings/forms.rs:45`, `:52`, `:79`, `:107-120` |
| C6 | `WidgetChrome` / `AppearanceOutcome` / `toggle_no_view` all `wanted` | DOC:`EDITABLE_SURFACES.md:160` | two of three reached — `app/actions/forms/author.rs:117`, `app/actions/forms.rs:904`. Only `AnnotFlags::toggle_no_view` is a real gap (zero occurrences under `src/`) |
| C7 | "`/MK /BC` is NOT consumed" | DOC:`ENGINE_BACKLOG.md:296` | it is — `widgetedit.rs:720-741`, `dialogs/formfield.rs:741-780`, `mkcolour.rs:43-108`. DOC:`ENGINE_BACKLOG.md:121` already records the correction, so the file contradicts itself |
| C8 | ✅ "Field list in tab order" | DOC:`FEATURES.md:451` | the panel numbers from `/Annots` order — `tab_order/mod.rs:7`/`:11`/`:37`/`:43`, `model.rs:34`/`:118`/`:168`/`:281`. DOC:`FEATURES.md:446` admits this in the same section |
| C9 | "whether it is visible and whether it prints" (two switches) | DOC:`MANUAL.md:704` | one four-value combo — `widgetedit.rs:546-591` — and sometimes **no control at all** `:561-564` |
| C10 | caption listed as a push-button property | DOC:`MANUAL.md:704` | drawn for **every** kind — `widgetedit.rs:604-628`, argued `:593-598`; and for a check box it is the glyph chooser (§2.5) |
| C11 | cites `boxes/mod.rs:71,78` | DOC:`NO_SURFACE.md:233` | actual `EDITOR_TEXT_RATIO` `:68`, `EDITOR_TEXT_RANGE` `:75` |
| C12 | cites `canvas/forms.rs:690` | DOC:`NO_SURFACE.md:234` | `MAX_TRACED_BOXES` declared `:701` |
| C13 | ✅ "A placed form field's properties are editable" | DOC:`FEATURES.md:448` | overstates — `/Opt` is authorable only at placement and never afterwards. DOC:`OPERATOR_REQUESTS.md:573-577` files this by name |
| C14 | "Fill a field on the canvas" | DOC:`FEATURES.md:230` | does not disclose that **only three of seven kinds are ever offered** (`boxes/mod.rs:88-163`), nor that `RotatedPage` blocks text fields only |
| ★ | "Creation only — the engine offers no restyle of an existing box" | DOC:`ENGINE_BACKLOG.md:118` | **refuted** — ENGINE:`edit.rs:38563-38569` + ENGINE:`annot_author.rs:3826-3836` recover and repaint the glyph from `/MK /CA` on any caption edit |

Stale source comments found in passing: `canvas/formfield/action.rs:64-79` says
"six" above `ALL: [Self; 7]`; `canvas/pressing.rs:284-287` says the rotate verb
"does not exist yet".

Docs that are correct: DOC:`ACROBAT_DEFAULTS.md` (no form claims);
DOC:`OPERATOR_REQUESTS.md:561` (O205), `:569` (the canvas drop-down complaint,
filed), `:573-577` (the `/Opt` gap, filed), `:578-581` (the rotation complaint —
filed, not measured; `:579` *"The controls are drawn, so this is worse than
absent"*); DOC:`EDITABLE_SURFACES.md:94`, `:143`; DOC:`ENGINE_BACKLOG.md:104`,
`:105`, `:114`, `:116`, `:117`, `:219`. DOC:`ENGINE_BACKLOG.md:115` (`/DA`
font/size/colour "wanted") is now **stale** — the group ships at
`fieldedit.rs:700-847`.

---

## 9. Summary matrix — every capability × every kind

Legend: ✅ present · ➖ N/A for that kind · ❌ ABSENT · ⚠ present but broken or
mislabelled.

| Capability | TX | CB | RB | CO | LB | PB | SG | Primary citation |
|---|---|---|---|---|---|---|---|---|
| Placeable by a command | ✅ | ✅ | ✅ | ✅ | ✅ (sub-option) | ✅ | ❌ | `catalog/edit.rs:378-427`; `canvas/formfield.rs:89-95` |
| Distinct ghost while placing | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | `canvas/formfield/ghost.rs:79-127` |
| Initial value at placement | ✅ | ✅ | ✅ | ❌ | ❌ | ➖ | ➖ | `dialogs/formfield.rs:578`, `:620`, `:644` |
| Option list `/Opt` — at placement | ➖ | ➖ | ➖ | ✅ | ✅ | ➖ | ➖ | `dialogs/formfield.rs:648-655` |
| **Option list `/Opt` — afterwards** | ➖ | ➖ | ➖ | **❌** | **❌** | ➖ | ➖ | would go `fieldedit.rs:246-275`; engine ENGINE:`edit.rs:21815` |
| Export value ≠ display | ➖ | ✅ (on-state) | ✅ | ❌ collapsed | ❌ collapsed | ➖ | ➖ | `author.rs:172-176` comment |
| Default value `/DV` | ✅ | ❌ | ❌ | ❌ | ❌ | ➖ | ➖ | `fieldedit.rs:544-581` |
| Alignment `/Q` | ✅ | ➖ | ➖ | ❌ | ❌ | ➖ | ➖ | `fieldedit.rs:608-636`; clear-`/Q` ❌ (ENGINE:`edit.rs:21835`) |
| Max length `/MaxLen` | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:371-408` |
| Comb | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:427-463` |
| Multiline | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:187-200` |
| Password | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:201-214` |
| Rich text | ❌ (display + convert-away only) | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `rows.rs:333-391`; would go `fieldedit.rs:186-225` |
| Do-not-scroll | ❌ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:186-225`; ENGINE:`edit.rs:21530` |
| Do-not-spell-check | ❌ | ➖ | ➖ | ❌ | ❌ | ➖ | ➖ | ENGINE:`edit.rs:21878` |
| File select | ❌ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ENGINE:`edit.rs:21871` |
| No export `/Ff`3 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | beside `fieldedit.rs:150-176`; ENGINE:`edit.rs:21857` |
| Mapping name `/TM` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ENGINE:`edit.rs:21551` |
| Format / validation / calculation / keystroke (`/AA`) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | **no `/AA` surface exists** |
| Editable combo (free text) | ➖ | ➖ | ➖ | ❌ (author-only flag; no entry field) | ➖ | ➖ | ➖ | `dialogs/formfield.rs:664-666`; `rows.rs:749-835` |
| Multi-select | ➖ | ➖ | ➖ | ➖ | ✅ | ➖ | ➖ | `fieldedit.rs:261-274`; `dialogs/formfield.rs:667-669` |
| Sorting `/Ff`20 | ➖ | ➖ | ➖ | ✅ author-only | ✅ author-only | ➖ | ➖ | `dialogs/formfield.rs:673`; **❌** afterwards |
| No-toggle-to-off | ➖ | ❌ | ✅ (>1 widget) | ➖ | ➖ | ➖ | ➖ | `fieldedit.rs:228-243` |
| Radios in unison | ➖ | ➖ | ❌ | ➖ | ➖ | ➖ | ➖ | beside `fieldedit.rs:228-243`; ENGINE:`edit.rs:21419` |
| Check-box style glyph | ➖ | ⚠ **works via the "Caption" row, mislabelled** | ➖ | ➖ | ➖ | ➖ | ➖ | `widgetedit.rs:604-629`; ENGINE:`edit.rs:38563`, ENGINE:`annot_author.rs:3826` |
| Button caption | ➖ | ➖ | ➖ | ➖ | ➖ | ✅ | ➖ | `dialogs/formfield.rs:680`; `widgetedit.rs:604-629` |
| Button icon `/I` / layout `/TP` | ➖ | ➖ | ➖ | ➖ | ➖ | ❌ | ➖ | `WidgetEdit` (`widgetedit.rs:46-55`) carries neither |
| Button action `/A` (7 kinds) | ➖ | ➖ | ➖ | ➖ | ➖ | ✅ | ➖ | `dialogs/buttonaction.rs:67-138`; `panels/forms/button.rs:72-172` |
| **Rotation `/MK /R`** | ✅ | ⚠ **written, never drawn** | ⚠ | ✅ | ✅ | ⚠ | ⚠ (disclosed) | `widgetedit.rs:337-338`; ENGINE:`edit.rs:24805-24807` |
| Read-only | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `fieldedit.rs:163-176` |
| Required | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `fieldedit.rs:150-162` |
| Tooltip `/TU` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `fieldedit.rs:487-514` (no diag region) |
| Visibility `/F` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `widgetedit.rs:546-591`; no-view bit ❌ |
| Border style `/BS /S` | ✅ Properties, ❌ placement | same | same | same | same | same | same | `widgetedit.rs:465-499` vs `author.rs:77-80` |
| Border width `/BS /W` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `widgetedit.rs:507-532` (row absent when `border` is `None` `:504-506`) |
| Background `/MK /BG` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `widgetedit.rs:692-716` |
| Border colour `/MK /BC` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `widgetedit.rs:718-743` |
| Font / size / colour `/DA` | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | `fieldedit.rs:283-286` gate, `:700-847` |
| Rect / resize | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `widgetedit.rs:363-424` |
| Rename `/T` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `formfield.rs:486-566` |
| Delete field / widget | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ gated | `formfield.rs:597-647` |
| Tab order (drag) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `tab_order/mod.rs:388-600` — **no kind filter** `model.rs:511-589` |
| Tab order (buttons / numeric / keyboard) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | asserted absent `tab_order/mod.rs:991-1006` |
| Field locking `/Lock` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | DOC:`ENGINE_BACKLOG.md:117` |
| Field-type conversion | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | deliberate, `fieldedit.rs:72-79` |
| **Fill on canvas** | ✅ | ✅ | ✅ | **❌ silent** | **❌ silent** | ❌ | ❌ | `boxes/mod.rs:88-163`, `:471`; reason `canvas/forms.rs:250-259` |
| Fill in the Forms panel | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ (action editor) | ❌ blocked | `rows.rs:186-204`, `:279-283` |
| Keyboard fill (Space/Enter) | ➖ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | `tabbing.rs:307-308`, `:350-367` |
| Arrow keys within a group | ➖ | ❌ | ✅ (moves and activates) | ❌ | ❌ | ➖ | ➖ | `tabbing.rs:315`, `:320`, `:400-402` |
| Copy / cut / paste | ✅ | ✅ | ✅ (whole group) | ✅ | ✅ | ✅ | ✅ unsigned only | `canvas/fieldclip.rs:68-72`, `:101-110` |
| Context menu | ✅ 2 items | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | `shell/menus.rs:641-644` |
| Flatten / reset / regenerate / recompute | whole-form only, no per-field | | | | | | | `panels/forms/edit.rs:381-405`, `:809`, `:827` |
| Group create / rename / merge / split | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | `panels/forms/groups.rs` — delete only |
| Driven test coverage | ✅ | partial | **❌** | **❌** | **❌** | ✅ | ✅ (delete only) | §6.1 |

---

## 10. The seven highest-value gaps this measurement exposes

1. **`/Opt` has no post-placement editor** — `fieldedit.rs:246-275` is where it
   goes; the engine verb is ENGINE:`edit.rs:21815`. DOC:`FEATURES.md:448`
   currently overclaims.
2. **`regen_button_appearance` ignores the staged rotation** —
   ENGINE:`edit.rs:38402` needs an angle argument, the way
   ENGINE:`edit.rs:38069-38071` already takes one. Engine-side, one function.
3. **`NotOffered` is silent** — `boxes/mod.rs:623-631` counts it into neither
   bucket, so `panels/forms/mod.rs:499-522` can never explain to an operator why
   clicking a drop-down does nothing.
4. **The Caption row needs a kind fork** — `widgetedit.rs:604-629` is a glyph
   chooser for check boxes and a word box for buttons, and says the latter for
   both.
5. **`/AA` has no surface at all** — format, validation, calculation and
   keystroke are the single largest Acrobat-parity hole, with no insertion point
   yet chosen.
6. **`fixtures/all-field-kinds.pdf` is wired to nothing**, and
   `radio-choice-form.pdf` already exists in the engine corpus — the radio /
   combo / list-box driven checks can be written without authoring a fixture.
7. **An editable combo box can never receive free text anywhere in the shell** —
   not canvas, not panel, not Properties.
