<!-- Recovered verbatim from the subagent transcript at
     .../tasks/ac6f84c7521e51076.output (240 JSONL records, the final
     substantive assistant message). Dispatched pre-compaction; the report
     was never re-emitted into the conversation. Independent of
     shell-form-surface.md and it AGREES with its section 3 on every
     mechanism -- two separate reads of the same code reaching the same
     conclusion about why a drop-down cannot be filled on the page. -->
# Canvas form-field interaction — mechanical truth

> **One row no longer describes the program, and the body is left alone because
> it is a verbatim record rather than a specification.** Escape on a field
> editor **commits** the draft; it does not abandon it, and there is no exit
> that discards one. The trace it names, `form-abandon`, is not emitted by
> anything — the line is `form-escape`. `FEATURES.md` carries the live claim.

## 0. The two surfaces, decided by mode

`canvas::forms::overlay` is the single entry point, called from `canvas/present.rs:949` (before `interact` at `present.rs:1065`), with `authoring = caps.edit_content`.

| mode | branch | what a click does |
|---|---|---|
| **Edit** (`caps.edit_content`) | `canvas/forms.rs:785–816` → `return` | **selects only.** No filling at all, for any kind. Stated at `forms.rs:773–778`: *"filling on the page is not available in Edit mode."* |
| **Read / Review** | `forms.rs:818–877` | **fills**, gated by `offer()` (`forms.rs:881–883`) = `offered_in(tool) && doc.annotations_visible() && fill_refusal().is_none()`; `offered_in` is `matches!(tool, CanvasTool::Select)` (`canvas/forms/boxes/mod.rs:325–327`) |

---

## 1. The full click path, function by function

### A. Fill path (Read/Review)

1. `canvas::present` (≈`present.rs:949`) → `forms::overlay(ui, doc, &page_views, &drawn, active_tool, caps.edit_content, actions)`
2. `forms::overlay` `forms.rs:742` → clears the Escape flag (`forms.rs:756`)
3. `forms::offer` `forms.rs:881–883` — three document-wide gates
4. `forms::placed` `forms.rs:586–691` — cached on `(PathBuf, edit_epoch)`; calls `pdfcer_core::forms::parse_acroform` (`forms.rs:597`), `doc.session.widget_rects(page)` (`forms.rs:601`), then `boxes::place` (`forms.rs:602`)
5. `boxes::place` `boxes/mod.rs:524–648` — one walk producing **two** sets:
   - `targets.push(...)` at `boxes/mod.rs:572–580` — **selectable**
   - `boxes::classify` at `boxes/mod.rs:596` → `WidgetBox` pushed at `boxes/mod.rs:604–617` — **fillable**
6. `boxes::classify` `boxes/mod.rs:414–473` — **the gate that decides everything below**
7. `tabbing::advance` `forms/tabbing.rs:59–119` (Tab arrivals)
8. `form_marks::shade` `form_marks.rs:124` / `form_marks::spotlight` `form_marks.rs:64`
9. `forms::editor` `forms.rs:962–1176` — draws the focused text editor; returns `claimed`
10. `forms::click` `forms.rs:1183–1283` — only `if !claimed` (`forms.rs:874`):
    - `ctx.pointer_interact_pos()` `forms.rs:1191`
    - page whose `d.response.clicked_by(Primary)` `forms.rs:1194–1200` — **the click is read off the page's own `Response`**, not a widget of this module's (`forms.rs:1180–1182`, header §4 at `forms.rs:172–194`)
    - `map.to_page(pos)` `forms.rs:1205`
    - `boxes::hit(list, page, point)` `forms.rs:1206` → `boxes/mod.rs:659–663`: `.rfind(|b| b.page == page && b.rect.contains(point))` — **plain containment, no tolerance** (decision at `forms.rs:196–200`)
    - trace `form-hit` `forms.rs:1209–1225`
    - `match &widget_box.kind` `forms.rs:1227` — three arms (below)
11. `forms::raise_button` `forms.rs:1327–1339` → `Action::Field(FieldAction::Edit(FormEdit::SetButtonState{..}))`
12. `forms::commit` `forms.rs:912–939` → `FormEdit::FillText{field, value}` at `forms.rs:932–938`
13. `forms::cursor` `forms.rs:1362–1378`
14. Later the same frame: `interact::interact` `interact.rs:306` → `clicking::click` `interact.rs:570` → the 12-rung ladder in `clicking.rs:230+`. **Rung 4 (`annot_hit`) deliberately excludes `/Widget`** — `selection/annot.rs:207`: *"the form field surface owns it — a click there focuses an editor, and two owners of one press is how a field becomes unfillable."*
15. `app::actions::forms::apply` `app/actions/forms.rs:593–658` → `FieldAction::Edit(edit) => panels::forms::edit::apply(doc, &edit)` at `forms.rs:647` → `panels/forms/edit.rs:run` → engine verb.

### B. Select path (Edit)

`forms::overlay` `forms.rs:785` → `selecting::select_click` `forms/selecting.rs:47–128` → `boxes::hit_target(targets, …)` `selecting.rs:96` (→ `boxes/mod.rs:672–676`) → `actions.push(FieldAction::Select(picked).into())` **`selecting.rs:127`** → `apply` sets `doc.selected_field` at `app/actions/forms.rs:597`. Then `selecting::select_cursor` `selecting.rs:182` and `selecting::selection_overlay` `selecting.rs:217–285`.

### C. Authoring path (placing a *new* field)

`clicking.rs:675–711` (`CanvasTool::Form(kind)` rung 9) → `formfield::ghost::click_rect` → `FieldAction::Begin{page,kind,rect}` `clicking.rs:703`; or the drag, `interact.rs:809–838` `GestureOutcome::FormField` → same `FieldAction::Begin` at `interact.rs:825`. Dialog → `app/actions/forms/author.rs`.

---

## 2. Per field kind — can the operator set the value on the canvas?

The canvas's own kind enum is **`BoxKind`**, `canvas/forms/boxes/mod.rs:88–163`, with **exactly three variants**. Its doc comment at `boxes/mod.rs:82–86` states the design:

> *"Three variants, not five: choice fields and everything in `NotOnCanvas::NotOffered` have no canvas gesture at all, so they are absent rather than present-and-inert."*

| Field kind | Set value on canvas? | Path / omission |
|---|---|---|
| **Text field** | **YES** | `classify` `boxes/mod.rs:427–450` → `BoxKind::Text`; `forms::click` arm `forms.rs:1241–1262` stores `Focus`; `forms::editor` `forms.rs:962–1176` puts a real `egui::TextEdit` (`forms.rs:1063–1067`, `ui.put` at `forms.rs:1114`); commit on focus loss `forms.rs:1152–1159` → `forms::commit` `forms.rs:912–939` → `FormEdit::FillText` `forms.rs:932–938`. Refused if rich text (`boxes/mod.rs:426`) or page `/Rotate != 0` (`boxes/mod.rs:428–430`, `NotOnCanvas::RotatedPage`). |
| **Check box** | **YES** | `classify` `boxes/mod.rs:451–469` → `BoxKind::Check`; `forms::click` arm **`forms.rs:1266–1274`** toggles `"Off"`↔`on_state` → `raise_button` `forms.rs:1327–1339` → `FormEdit::SetButtonState`. Also Space/Enter via `tabbing::activate` `tabbing.rs:350–357`. |
| **Radio group member** | **YES** | `classify` `boxes/mod.rs:451–469` → `BoxKind::Radio`; `forms::click` arm **`forms.rs:1276–1281`** (clicking the already-on one is a deliberate no-op); `tabbing::activate` `tabbing.rs:359–363`; arrow keys `tabbing.rs:309–321`. |
| **Combo box / drop-down (`/Ch`)** | **ABSENT** | `classify` falls through to **`boxes/mod.rs:471`: `_ => Err(NotOnCanvas::NotOffered)`**. No `BoxKind` variant exists. Never becomes a `WidgetBox`, so never reaches `forms::click`. |
| **List box (`/Ch`)** | **ABSENT** | Identical — same `_` arm, **`boxes/mod.rs:471`**. `FieldType::Choice` covers both (`pdfcer-core/src/forms.rs:234–235`), and nothing in `canvas/` distinguishes them. |
| **Push button** | **ABSENT** | Caught **twice**: `boxes/mod.rs:418–420` (`panels::forms::rows::block_reason(field).is_some() → NotOffered`, and `block_reason` returns `Some` for `ButtonKind::Push` at `panels/forms/rows.rs:280–283`), and again by `boxes/mod.rs:471` since the tuple match at `boxes/mod.rs:451` only admits `Check \| Radio`. |
| **Signature (`/Sig`)** | **ABSENT** | `boxes/mod.rs:418–420` via `block_reason` `panels/forms/rows.rs:279`, and `boxes/mod.rs:471`. |
| *(read-only, any kind)* | ABSENT | `boxes/mod.rs:418–420` via `block_reason` `rows.rs:275–277`. |

### The exact omitting match — `canvas/forms/boxes/mod.rs:414–473`

```
415  if !widget.has_normal_appearance { return Err(NotOnCanvas::NoAppearance); }
418  if crate::panels::forms::rows::block_reason(field).is_some() {
419      return Err(NotOnCanvas::NotOffered);
420  }
425  match (field.field_type, field.button_kind) {
426      (Some(FieldType::Text), _) if field.is_rich_text() => Err(NotOnCanvas::NotOffered),
427      (Some(FieldType::Text), _) => { … Ok(BoxKind::Text { … }) }          // 427-450
451      (Some(FieldType::Button), Some(kind @ (ButtonKind::Check | ButtonKind::Radio))) => {
459          … let Some(on_state) = widget.on_states.first() else { return Err(NotOffered) };
466          Ok(match kind { ButtonKind::Check => BoxKind::Check{…}, _ => BoxKind::Radio{…} })
470      }
471      _ => Err(NotOnCanvas::NotOffered),     // ← /Ch, ButtonKind::Push, /Sig, /FT absent
473  }
```

`(Some(FieldType::Choice), _)` **has no arm.** There is no `FieldType::Choice` reference anywhere in `canvas/` — the only `FieldType::` mentions in `canvas/` are lines `boxes/mod.rs:426`, `:427`, `:451`.

### The consequence cascade — a drop-down gets *nothing*

Because a choice field never becomes a `WidgetBox`, every downstream surface that takes `list: &[WidgetBox]` skips it:

- `forms::click` `forms.rs:1227` — match is exhaustive over three variants, no wildcard. A choice field is never in `list`, so the function cannot see it.
- `forms::cursor` `forms.rs:1371–1374` — no cursor change over it (not even `PointingHand`, because the `_ =>` arm only runs for boxes that exist).
- `form_marks::shade` `form_marks.rs:147` — no Acrobat-style field wash over it.
- `form_marks::spotlight` `form_marks.rs:82` — the Forms panel **cannot even flash its location on the page**.
- `ring::rings` `forms/ring.rs:202–261` — never a Tab stop.
- `tabbing::button_focus` `tabbing.rs:255–341` — unreachable for it.
- **`Routing` counts it as nothing.** `boxes/mod.rs:623–631` increments `undrawn` only for `NoAppearance` and `unreachable` only for `RotatedPage | NotPlaced`. `NotOffered` matches **neither**, so `panels::forms::canvas_routing` (`panels/forms/mod.rs:499–522`) prints **no note about it either**. The operator is told nothing, anywhere on the canvas.

It **is** selectable in Edit mode: `boxes/mod.rs:572–580` pushes the `FieldTarget` *above* the `classify` call, with the comment at `boxes/mod.rs:565–571`: *"A drop-down and a push button reach this line and are refused by `classify` one line down; they are still things the operator can see and must be able to click."*

---

## 3. Combo box / list box specifically

**Is a popup / option picker / dropdown drawn on canvas? NO.** There is no `egui::ComboBox`, no `egui::Area`, no `popup`, no option list anywhere in `canvas/forms.rs`, `canvas/forms/*`, or `canvas/form_marks.rs`. The only form-adjacent `egui::ComboBox` outside the panels is `dialogs/buttonaction.rs:72` (a push-button *action-kind* picker in a dialog).

**Is `/Opt` ever read on a canvas path? NO.** Crate-wide sweep for `Opt` / `options` / `choice` / `combo` / `listbox` / `list_box`:

| where | what |
|---|---|
| `panels/forms/rows.rs:749–826` | **`choice_row`** — the working picker. `egui::ComboBox::from_id_salt` at **`rows.rs:810`**, `selectable_label` at `rows.rs:819`, raising `FormEdit::SetChoice{field, values}` at `rows.rs:820–823`. Multi-select renders a checkbox stack instead (`rows.rs:771–798`). Empty `/Opt` → a weak "no options" label and `return` (`rows.rs:750–757`). Matches on **export or display** because `/V` holds either in the wild (`rows.rs:778–781`). |
| `panels/forms/rows.rs:198` | `(Some(FieldType::Choice), _) => choice_row(…)` — **the panel's kind match DOES have a choice arm.** |
| `panels/forms/edit.rs:340–365` | `FormEdit::SetChoice{field, values}` — the action variant; docs `/Opt` `[export display]` pairs at `edit.rs:354` and *"The selections, in the order `/Opt` lists them"* at `edit.rs:360`. |
| `panels/forms/edit.rs:765` | `session.set_choice_value(field, &refs)?` — **the only call to the engine's choice verb in the whole GUI.** |
| `panels/properties/fieldedit.rs:246`, `:66`, `:256` | choice-field property editing (`with_combo(false)` → `ChoiceEditWithoutCombo`) |
| `app/actions/forms.rs:781` | doc comment on `/Opt` order (Table 230) |
| `app/actions/forms.rs:1234–1236` | `outcome.disclosures.has_no_options → form_field_no_options()` |
| `canvas/formfield/draft.rs:122–134`, `:161–167`, `:219–220` | the authoring dialog's `options: String` (one per line), `combo: bool`, `edit`, `multi`, `sort` |
| `app/actions/forms/author.rs:166–187` | `K::Choice =>` builds `Vec<ChoiceOption>` via `ChoiceOption::plain` (`author.rs:172–176`), `spec.combo = draft.combo` (`author.rs:178`), `session.add_choice_field(&spec)` (`author.rs:187`) — **so pdfcer can author `/Opt`, it just can't pick from it on the page** |
| `dialogs/formfield.rs:570`, `:647–664` | `choice_rows`; `ui.radio_value(&mut self.draft.combo, false, t::list_box())` at `:662` |
| `canvas/forms/boxes/tests.rs:802`, `:820` | the codebase's own assertion: *"A drop-down (`/Ch`) is `NotOffered` — this shell has no canvas gesture"*; failure message *"a drop-down has no canvas fill gesture and must not offer one"* |
| `icons/catalog/mod.rs:700–706`, `shell/commands/catalog/edit.rs:371` | `Icon::DropDown`, `edit.form_choice` ribbon item ("Drop-down") |

`listbox` / `list_box` as identifiers exist only at `dialogs/formfield.rs:662` and `text/formfield.rs:244` — both UI strings in the authoring dialog.

### Precise reason picking an option is impossible on the canvas

All four of the caller's candidate reasons are true simultaneously, in this order:

1. **The match arm falls through.** `boxes/mod.rs:471` `_ => Err(NotOnCanvas::NotOffered)` — `FieldType::Choice` has no arm.
2. **The enum has no variant.** `BoxKind` (`boxes/mod.rs:88–163`) cannot represent a choice field at all, so there is nothing for a hit test to return or a match to dispatch on.
3. **No UI.** No popup surface is drawn on the page by any code in `canvas/`. Documented as an explicit decision in the module header, `forms.rs:250–257`: *"**[`NotOnCanvas::NotOffered`] — this kind has no canvas gesture.** … choice fields … A choice field would need a dropdown anchored to the page, which is a second popup surface with its own placement rules and no gesture the panel does not already have."*
4. **No `/Opt` read and no engine call.** `set_choice_value` is called from exactly one place, `panels/forms/edit.rs:765`, reached only from `panels/forms/rows.rs:810–826`.

**Where it does work:** the **Forms panel** (`panels/forms/rows.rs:810`). And there is a real secondary defect for discoverability: the panel's spotlight (`form_marks.rs:82`) cannot outline a choice field on the page, and `canvas_routing` (`panels/forms/mod.rs:508–521`) prints no note for it, so nothing connects the panel row to the box on the page.

---

## 4. Keyboard on a focused field

### The gate
`canvas::keys::keys` reads `forms::escape_spent(ctx)` as **claimant 0 at `keys.rs:513`, before** the `text_edit_focused()` guard at `keys.rs:539–542` — because egui's `TextEdit` surrenders focus on Escape *before* `keys` runs (`keys.rs:503–512`, `keys.rs:143–162`). While a field editor holds the keyboard, `keys.rs:539` returns early: the canvas Escape/Delete/arrow ladder is off.

### Per kind

| key | Text field | Check box | Radio member | Choice / Push / Sig |
|---|---|---|---|---|
| **typing** | goes to the real `egui::TextEdit` (`forms.rs:1114`); multiline vs `singleline().password()` chosen at `forms.rs:1063–1067`; `/MaxLen` via `truncate` `forms.rs:1013` → `boxes/mod.rs:744–749` | nothing | nothing | **never focusable** |
| **Enter** | multiline: inserts a newline. Singleline: egui drops focus → `response.lost_focus()` at `forms.rs:1152` → `commit` → `FormEdit::FillText` | **toggles** — `tabbing.rs:307–308` → `activate` `tabbing.rs:350–357` | **selects** — `tabbing.rs:307–308` → `activate` `tabbing.rs:359–363` | — |
| **Space** | inserts a space | **toggles** (same, `tabbing.rs:307–308`) | **selects** (same) | — |
| **Escape** | abandons the draft, `forms.rs:1142–1150` (`store_focus(None)`, `note_escape`, trace `form-abandon`, `return true`) — read *before* the commit branch on purpose | drops focus, `tabbing.rs:294–302` | same | — |
| **Tab / Shift+Tab** | `tabnav::take(Scope::Field)` `tabbing.rs:65` → `ring::rings` `tabbing.rs:68` → `tabnav::step(&table.lens(), at, backwards, CROSS_PAGES)` `tabbing.rs:96` → `move_focus` `tabbing.rs:150–201`, which commits the field being left (`tabbing.rs:157–162`), pushes `Action::GoToPage` (`tabbing.rs:188`) and `Action::RevealRect{… why: WHY_TAB}` (`tabbing.rs:194–199`). `CROSS_PAGES = true` `tabbing.rs:48`. Tab is stolen from egui in `raw_input_hook` (`canvas/tabnav.rs:145–168`), and `tabnav::publish(Scope::Field, id)` is called every frame the editor draws (`forms.rs:1120`) and every frame a button holds focus (`tabbing.rs:282`) | same ring | **one stop per group** — `ring::assemble` collapses radio groups, `ring.rs:97–136`, flagged by `radio: matches!(widget_box.kind, BoxKind::Radio{..})` `ring.rs:228` | **not in the ring at all** — `rings` is built from `list: &[WidgetBox]` (`ring.rs:202–261`) |
| **arrows** | move the caret inside the `TextEdit` | `sibling` returns `None` when `group.len() < 2` (`tabbing.rs:400–402`), so **nothing happens on a lone check box** | Down/Right = forward, Up/Left = backward (`arrow` `tabbing.rs:374–384`) → `sibling` `tabbing.rs:391–410` (filters `BoxKind::Radio` at `tabbing.rs:397`) → `move_focus` `tabbing.rs:315` **plus `activate` `tabbing.rs:320`** — arrowing onto a radio *selects* it | — |
| **Delete/Backspace** | edits the draft (`keys.rs:539` returns before the canvas Delete ladder) | *(button focus is `focusable_noninteractive`, `tabbing.rs:271`; no Delete handling)* | same | in **Edit** mode with `doc.selected_field` set, `keys.rs:916–958` pushes `FieldAction::DeleteWidget{field, widget}` — gated on `caps.edit_content` and `field_delete_refused` (`keys.rs:942–950`). This arm is **first** in the Delete ladder (`keys.rs:903–911`) |

Caret seating: exactly once, at the **end** of the text (`forms.rs:1127–1136`) — never select-all, because the click that opened the editor was consumed by the page so there is no caret position, and select-all would turn the next keystroke into a wipe.

---

## 5. The field-kind enums and every match site

### The engine's enum — `pdfcer_core::forms::FieldType`
`D:\Dev\pdfcer\crates\pdfcer-core\src\forms.rs:228–238` — four variants: `Button` (`/Btn`, refined by `ButtonKind`), `Text` (`/Tx`), `Choice` (`/Ch` — *"list box or combo box"*), `Signature` (`/Sig`).
`ButtonKind`: `D:\Dev\pdfcer\crates\pdfcer-core\src\forms.rs:281–290` — `Push`, `Check`, `Radio`.

Every `FieldType::` site in `crates/pdfcer-gui/src` (non-test):

| file:line | shape | choice arm? |
|---|---|---|
| `canvas/forms/boxes/mod.rs:426` | `match (field_type, button_kind)` guard | — |
| `canvas/forms/boxes/mod.rs:427` | `(Some(Text), _)` | — |
| `canvas/forms/boxes/mod.rs:451` | `(Some(Button), Some(Check\|Radio))` | — |
| **`canvas/forms/boxes/mod.rs:471`** | **`_ => Err(NotOnCanvas::NotOffered)`** | **NO — this is the omission** |
| `panels/forms/rows.rs:187–198` | the panel's twin match | **YES — `:198` `(Some(Choice), _) => choice_row(…)`** |
| `panels/forms/rows.rs:279` | `Some(Signature) => Some(signature_note)` in `block_reason` | `_ => None` at `:284` |
| `panels/forms/rows.rs:280–283` | `Some(Button) => match button_kind { Some(Push) => note, _ => None }` | `_ => None` at `:282` |
| `panels/forms/rows.rs:301` | `matches!(…, Some(Text))` | — |
| `panels/forms/edit.rs:1027` | `f.field_type == Some(Text)` | — |
| `panels/properties/fieldedit.rs:186` | `matches!(…, Some(Text))` | — |
| `panels/properties/fieldedit.rs:228` | `matches!(…, Some(Button))` | — |
| `panels/properties/fieldedit.rs:246` | `matches!(…, Some(Choice))` | yes |
| `panels/properties/fieldedit.rs:285` | `Some(Text \| Choice \| Button)` | yes |
| `sign/mod.rs:505` | `== Some(Signature)` | — |
| `text/panels/formfield.rs:107–115` | label match, all four | yes `:114` |
| `text/panels/formfield.rs:172`, `:183` | `matches!(…, Some(Text))` / `Some(Choice)` | yes |

### The canvas's enum — `canvas::forms::boxes::BoxKind`
`canvas/forms/boxes/mod.rs:88–163` — three variants: `Text { multiline, password, max_len, align }` (`:91–142`), `Check { on_state, on }` (`:144–149`), `Radio { on_state, on }` (`:157–162`).

Every `BoxKind` site (non-test):

| file:line | shape | notes |
|---|---|---|
| `boxes/mod.rs:431` | construction `Ok(BoxKind::Text{..})` | |
| `boxes/mod.rs:466–469` | `match kind { Check => …, _ => Radio }` | on `ButtonKind` |
| `forms/ring.rs:228` | `matches!(widget_box.kind, Radio{..})` | tab-ring radio flag |
| `forms/tabbing.rs:349–367` | `match &widget_box.kind` in `activate` | **`:366` `BoxKind::Text { .. } => {}` — an explicit no-op arm** |
| `forms/tabbing.rs:397` | `matches!(b.kind, Radio{..})` in `sibling` | |
| `forms.rs:999–1010` | `let BoxKind::Text{..} = kind else { return tabbing::button_focus(…) }` | the non-text bail-out at `:1009` |
| `forms.rs:1227–1282` | `match &widget_box.kind` in `click` | 3 arms, **no wildcard**; `:1237` is a guarded no-op arm protecting an in-flight draft |
| `forms.rs:1342–1347` | `kind_label` → `"text"/"check"/"radio"` | trace tokens |
| `forms.rs:1371–1374` | `match widget_box.kind { Text => CursorIcon::Text, _ => PointingHand }` | **the only `_ =>` on `BoxKind`** |

### The authoring enum — `canvas::formfield::FormFieldKind`
`canvas/formfield.rs:60–85` — **five** variants: `Text` `:62`, `CheckBox` `:64`, `Radio` `:71`, **`Choice` `:73`** (*"A drop-down or list of options."* `:72`), `PushButton` `:84`. `ALL: [Self; 5]` at `:89–95`. Header `:52–58`: *"Exactly the five `pdfcer-core` has verbs for."*

Match sites:

| file:line | what | Choice arm |
|---|---|---|
| `canvas/formfield.rs:122–124` | `is_useful_once_placed` — one arm listing all five → `true` | yes |
| `canvas/formfield.rs:136–142` | `command_id` | `:140 "edit.form_choice"` |
| `canvas/formfield.rs:159–163` | `default_size_pt` | `:160 Text \| Choice => (160.0, 20.0)` |
| `canvas/formfield.rs:176–182` | `noun` | `:180 form_noun_choice()` |
| `canvas/formfield.rs:199–205` | `name_prefix` (a PDF `/T` stem, untranslated) | `:203 "Dropdown"` |
| `canvas/formfield.rs:214–216` | `drag_shape` — no match, always `MarkupKind::Rectangle` | n/a |
| `canvas/formfield/draft.rs:177` | `matches!(kind, Text)` | — |
| `canvas/formfield/draft.rs:283` | `matches!(kind, Radio)` | — |
| `dialogs/formfield.rs:258–276` | `content_height` | `:270` |
| `dialogs/formfield.rs:567–571` | row dispatch | `:570 Choice => self.choice_rows(ui)` |
| `dialogs/formfield.rs:746` | `== Radio` | — |
| `app/actions/forms/author.rs:166–187` | `K::Choice =>` build `NewChoiceField` | yes |
| `text/formfield.rs:41–45`, `:53–59`, `:74` | titles / intros / name label | `:44 "New drop-down list"`, `:58 "A list people can pick from."` |
| `canvas/tool/mod.rs:596` | `form_kind() -> Option<FormFieldKind>` | n/a |
| `shell/commands/mapping.rs:381–382` | `form_for_command` — scans `ALL` | n/a |
| `app/dispatch/forms.rs:66–92` | `arm` — gate `edit_content` `:67–77`, then `canvas::tool::arm_form(ctx, kind)` `:91`; no per-kind match | n/a |

`NotOnCanvas` (`boxes/mod.rs:170–186`): `NoAppearance`, `RotatedPage`, `NotPlaced`, **`NotOffered`** (*"This field kind has no canvas gesture."* `:184`). Only the first three are ever counted into `Routing` (`boxes/mod.rs:623–631`) — **`NotOffered` is silent everywhere.**

---

## Bottom line for the operator's complaint

Picking an option in a drop-down on the page is not broken — **it was never built, deliberately, and the omission is silent.** A `/Ch` field is refused at `canvas/forms/boxes/mod.rs:471`, so it never becomes a `WidgetBox`; `BoxKind` (`boxes/mod.rs:88–163`) has no variant for it; no popup is drawn on canvas; `/Opt` is never read on any canvas path; and because `NotOffered` is counted into neither `Routing.undrawn` nor `Routing.unreachable` (`boxes/mod.rs:623–631`), the Forms panel prints no note about it either. The operator clicks and gets no cursor change, no shade, no outline, no tab stop, and no explanation. The only working picker is the `egui::ComboBox` in the Forms panel at `panels/forms/rows.rs:810`, whose selection raises `FormEdit::SetChoice` (`rows.rs:820`) and reaches `session.set_choice_value` at `panels/forms/edit.rs:765`.

---

## Postscript from the same agent

The background grep completed (it was a duplicate lookup that timed out searching the cargo registry — the `FieldType` definition was already located directly at `D:\Dev\pdfcer\crates\pdfcer-core\src\forms.rs:228-238`, and that finding is already in the report above).

No changes to the findings. The report stands as delivered:

**Combo box / drop-down option picking on the canvas is absent by design, and silently so.** `/Ch` fields fall into `_ => Err(NotOnCanvas::NotOffered)` at `canvas/forms/boxes/mod.rs:471`, `BoxKind` (`boxes/mod.rs:88-163`) has no variant for them, no popup is drawn anywhere in `canvas/`, `/Opt` is never read on a canvas path, and `NotOffered` is counted into neither `Routing.undrawn` nor `Routing.unreachable` (`boxes/mod.rs:623-631`) — so the operator gets no cursor, no shade, no spotlight, no tab stop and no explanation. The working picker is the Forms panel's `egui::ComboBox` at `panels/forms/rows.rs:810` → `FormEdit::SetChoice` → `session.set_choice_value` at `panels/forms/edit.rs:765`.
