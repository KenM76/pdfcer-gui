# What `pdfcer-core` can do to a form field at the pin

The **engine column** of `FORMS_PARITY.md`. Read out of
`D:\Dev\pdfcer` at HEAD `5d43d2ea0e99fce8afeb06cc62ab3a9a3e6039ab`
(branch `main`, working tree clean, *"librarian: the 560th filing, plus the two
things it found and handed back"*). Every row below was read from the tree at
that sha. Nothing was written to `D:\Dev\pdfcer`.

**A cell in the shell column that says "cannot" is only honest if the row here
says "ABSENT".** If the engine has the verb and the shell does not reach it,
that is GUI work, and O205 is about exactly those rows.

> ⚠️ **Do not trust the core-api docs' line numbers.**
> `docs/core-api/02-editing-and-saving.md` and `03-capabilities.md` were
> verified at `5c37c7c` / `e194b46` and **every `edit.rs` line number in them is
> stale by ~15,000 lines** — they cite `add_text_field` at `edit.rs:7087`; it is
> at `edit.rs:22681`. The *prose* is accurate and in places richer than the
> source comments; the *citations* are not. Two further doc defects:
> `edit.rs:25133` cites `EditError::DegenerateFieldRect`, a variant that does
> not exist (the real one is `FieldRectDegenerate`, `edit.rs:7162`), and
> `03-capabilities.md:617` says "other `/MK` keys are not read" while `/BG`,
> `/BC` and `/R` are all modelled.

---

# 1. The read surface

All in `pdfcer_core::forms` unless noted. **Nothing is re-exported at the crate
root — spell the full path.**

| What it reads | Verb / type | `file:line` | Verdict |
|---|---|---|---|
| Parse the whole form | `forms::parse_acroform<G: ObjectGraph + ?Sized>(graph: &G) -> Option<AcroForm>` | `forms.rs:1202` | **PRESENT** — generic, so it runs over a `Document` *and* over a live `EditSession` overlay (`&session.graph()`). Tolerates every malformed shape; never panics |
| Document-level form state | `AcroForm { fields, groups, need_appearances, sig_flags, signatures_exist, append_only, calc_order_count, calc_order, has_default_resources, default_appearance, quadding, xfa, inline_field_roots }` | `forms.rs:992` | **PRESENT** — `fields` in DFS order |
| Enumerate fields | `fillable_fields()`, `field_by_name(&str)`, `fields_named(&str)`, `descendants_of(&str)` | `forms.rs:1087`, `:1098`, `:1104`, `:1148` | **PRESENT** — `fields_named` exists because several representations can share one FQN |
| Pure grouping nodes | `FieldGroupNode { id, fully_qualified_name, partial_name }`; `AcroForm::groups` | `forms.rs:979` | **PRESENT** — **deepest-first (post-order)**. Never derive these by splitting an FQN; a `/T` may contain a literal period |
| Per-field everything | `Field { id, fully_qualified_name, partial_name, alternate_name, mapping_name, rich_value, default_style, field_type, button_kind, flags, value, default_value, default_appearance, quadding, max_len, options, top_index, selected_indices, widgets, merged, has_additional_actions, shares_parent_name, parent }` | `forms.rs:693` | **PRESENT** — `/T /TU /TM /RV /DS /FT /Ff /V /DV /DA /Q /MaxLen /Opt /TI /I` all read, inheritance resolved through `/Parent` |
| Field kind | `FieldType { Button, Text, Choice, Signature }`; `ButtonKind { Push, Check, Radio }` | `forms.rs:228`, `:281` | **PRESENT** — `from_name()` / `from_flags()` are `pub(crate)`, so a shell cannot reconstruct the mapping itself |
| Field flags | `FieldFlags(pub u32)` + `READ_ONLY REQUIRED NO_EXPORT MULTILINE PASSWORD NO_TOGGLE_TO_OFF RADIO PUSHBUTTON COMBO EDIT SORT FILE_SELECT MULTI_SELECT DO_NOT_SPELL_CHECK DO_NOT_SCROLL COMB RICH_TEXT RADIOS_IN_UNISON COMMIT_ON_SEL_CHANGE`; `read_only() required() no_export() has(bit)` | `forms.rs:106` | **PRESENT** — every Table 226/228/230 bit |
| Bit-26 overload | `Field::is_rich_text()`, `Field::radios_in_unison()` | `forms.rs:886`, `:898` | **PRESENT** — type-gated. **Never test the raw bit** |
| Value, per kind | `FieldValue { Absent, Name(Vec<u8>), Text(Vec<u8>), Choice(Vec<Vec<u8>>), Signature }`; `display_text()` | `forms.rs:319`, `:354` | **PRESENT** — `Absent` distinct from empty |
| Option list | `ChoiceOption { pub export: Vec<u8>, pub display: Vec<u8> }`; `Field::options` | `forms.rs:375` | **PRESENT** — export and display separate, both read |
| Per-widget geometry / chrome / state | `Widget { id, rect, appearance_state, on_states, has_off_appearance, page, caption, background, border_color, rotation, border, visibility, annot_flags, has_normal_appearance, merged }` | `forms.rs:461` | **PRESENT** — `/Rect /AS /AP/N` keys, `/P /MK{CA,BG,BC,R} /BS`(→`/Border` fallback) `/F` |
| `/MK` colours | `MkColor { None, Gray, Rgb, Cmyk }`; `from_array(&Object) -> Option<Self>`, `to_array(self) -> Object` | `forms.rs:397`, `:418`, `:442` | **PRESENT** — `None` (key absent) vs `Some(MkColor::None)` (Table 189 empty array) are distinct facts. DeviceCMYK never converted |
| `/F` visibility | `Widget::visibility: Option<Visibility>` + `Widget::annot_flags: AnnotFlags` (raw) | `forms.rs:461` region | **PRESENT** — exact-or-`None`: `None` always means *present and unmappable*, never absent (absent `/F` = 0 = `ScreenOnly`) |
| `/BS` border | `Widget::border: Option<BorderSpec>`; `BorderSpec::table_166_default()` | `forms.rs:1723`, `annot_author.rs:203` | **PRESENT** — `None` means **the file states no border**; substituting a default is the defect the field was added to prevent |
| `/MK /R` rotation | `Widget::rotation: Option<i64>` | `forms.rs:461` region | **PRESENT** — not normalised; `None` = file silent, ≠ `Some(0)` |
| Appearance presence | `Field::has_appearance()`, `Widget::has_normal_appearance`, `on_states`, `has_off_appearance` | `forms.rs:908` | **PRESENT** |
| XFA | `XfaPresence { None, Stream, PacketArray { packets } }`, `is_present()` | `forms.rs:922` | **PRESENT** — detected and disclosed only; **never parsed, never written** |
| `/AA` presence | `Field::has_additional_actions: bool` | `forms.rs:1584` | **PRESENT (disclosure only)** — recognition, not decoding |
| Script census | `forms::scan_javascript(graph) -> FormJavaScript`; `form_script::inventory::inventory(&DocumentView) -> ScriptInventory` | `forms.rs:2181`, `form_script/inventory.rs:166` | **PRESENT** — classifies, never executes |
| Native recompute plan | `form_script::recompute::plan(&DocumentView, CommaPolicy) -> RecomputePlan` | `form_script/recompute.rs:263` | **PRESENT** — non-mutating; `order_source` / `is_pdfcer_choice()`, `coerced_operands()`, `SkippedField` |
| Format-helper render | `form_script::format::render(&FormatHelper, stored_value, CommaPolicy) -> FormatOutcome` | `form_script/format.rs:274` | **PRESENT** — display only; **never touches `/V`** |
| Per-page tab order, computed | `EditSession::page_tab_sequence(&self, page_index: usize) -> Result<TabSequence, EditError>` | `edit.rs:29596` | **PRESENT** — `TabSequence { stated: PageTabs, basis: TabOrderBasis, excluded: Vec<(ObjId, TabExclusion)>, … }`. `TabOrderBasis` = `StatedArrayOrder \| StatedWidgetOrder \| ComputedRowOrder \| ComputedColumnOrder \| ArrayOrderByConvention \| NotDerivedStructure` (`edit.rs:19954`); `TabExclusion` = `Hidden \| NoView \| TrapNet \| Popup` (`edit.rs:19889`) |
| Tab-order policy knobs | `set_widget_tab_tail(WidgetTabTail)`, `set_tab_row_tolerance(f64)` | `edit.rs:9143`, `:9166`; `WidgetTabTail { ArrayOrder, RowOrder }` `settings/mod.rs:1625` | **PRESENT** — session settings, not document writes |
| Widget hit-test rects | `EditSession::widget_rects(&self, page_index: usize) -> Vec<(ObjId, [f64;4])>` | `edit.rs:45077` | **PRESENT** |
| Read a push-button's action | `EditSession::button_action(&self, fqn: &str) -> Result<ButtonActionState, EditError>` | `edit.rs:36742` | **PRESENT** — `ButtonActionState = None \| Known(ButtonAction) \| Unmodelled(String) \| Foreign(String)` (`edit.rs:19088`). Answers for the **first** widget; picks rather than reconciles |
| Copy-settings-from source | `EditSession::field_defaults(&self, source: &str) -> Result<FieldDefaults, EditError>` | `edit.rs:26324`; `FieldDefaults { field_type, button_kind, max_len, options, on_state, on_state_ambiguous, caption }` `edit.rs:1943` | **PRESENT** |
| Reset dry-run | `EditSession::reset_preview(&self, only: Option<&[String]>) -> Vec<ResetPreviewRow>` | `edit.rs:36634` | **PRESENT** — `&self`, non-mutating; a row for **every** field in scope, ineligible ones included |
| Group-delete dry-run | `field_group_deletion_preview(&mut self, fqn) -> Result<FieldGroupDeletion, EditError>` | `edit.rs:24314` | **PRESENT** — ⚠️ takes `&mut self` although it writes nothing |
| Adopt dry-run | `adopt_preview(&self, widget: ObjId, name: Option<&str>) -> Result<AdoptOutcome, EditError>` | `edit.rs:42945` | **PRESENT** |
| Pre-flights | `fill_refusal()`, `flatten_refusal()`, `deletion_refusal()`, `rename_refusal()` → `Option<EditError>` | `edit.rs:36023`, `:36049`, `:36109`, `:36138` | **PRESENT** — ⚠️ `fill_refusal` is a **strict subset** of what a fill enforces (no encryption, no `/Size`). Preflight for the label, handle the `Err` for the truth |
| Export current values | `EditSession::export_form_data(&self) -> Option<crate::fdf::FormData>` | `edit.rs:39026`; `FormData::from_acroform` `fdf.rs:160` | **PRESENT** — **omits any field with no value**, so an import never clears an unnamed field |
| Serialise | `FormData::to_fdf(Option<&str>)`, `to_xfdf(Option<&str>)`, `formcsv::to_csv(&FormData) -> CsvExport` | `fdf.rs:208`, `:256`, `formcsv.rs:111` | **PRESENT** — CSV is **two-column only**; formula-neutralisation disclosed and reversible |
| Render widgets | `pdfcer_render::annot::AnnotationClass::FormField`, `AnnotationScope::FormFieldsOnly`, `paints_class()`, `appearance_placement()` | `pdfcer-render/src/annot.rs:171`, `:405`, `:421`, `:993` | **PRESENT, and that is all** — `pdfcer-render` has **no form model**. It classifies `/Widget`, filters by scope, and paints the baked `/AP` through the §12.5.5 matrix. No field awareness, no value rendering, no `/MK` synthesis |

---

# 2. The write surface

All on `EditSession` (`pdfcer-core/src/edit.rs`). **The modules `forms`,
`forms_author`, `fdf`, `formcsv`, `form_script`, `richtext`, `vartext` are
read / parse / serialise only** — there is no `forms::fill(…)`.

## 2.1 Field value, per kind

| Kind | Verb | `file:line` | Verdict |
|---|---|---|---|
| `/Tx` text | `fill_text_field(&mut self, fqn: &str, text: &str) -> Result<FillOutcome, EditError>` | `edit.rs:36210` | **PRESENT** — refuses a rich-text field (`FieldIsRichText`) |
| `/Tx` rich text | `fill_text_field_downgrading_rich_text(&mut self, fqn, text) -> Result<FillOutcome, EditError>` | `edit.rs:36254` | **PRESENT** — lossy and deliberate: clears `/Ff` bit 26, **deletes** `/RV`, regenerates a plain appearance. Present it *as a downgrade* |
| `/Btn` check + radio | `set_button_state(&mut self, fqn: &str, on_state: &str) -> Result<(), EditError>` | `edit.rs:36449` | **PRESENT** — writes `/V` + every widget's `/AS`. **No appearance regeneration** (body `36449`–`36630` read: no regen call) |
| `/Ch` combo + list | `set_choice_value(&mut self, fqn: &str, selections: &[&str]) -> Result<FillOutcome, EditError>` | `edit.rs:38709` | **PRESENT** — matches export-first then display, stores the **export** in `/V`; `/V` is an array only when `MultiSelect`; rewrites or removes `/I` (`:38920`/`:38924`) and `/TI` (`:38933`/`:38936`); an editable combo accepts free text; every same-FQN representation updated |
| `/Sig` | — | — | **ABSENT as a fill.** Signature values come only from `sign(...)` (`CommandKind::AddSignatureField`). `import_form_data` skips `Signature` and untyped targets and counts them |
| Bulk | `import_form_data(&mut self, data: &crate::fdf::FormData) -> Result<ImportOutcome, EditError>` | `edit.rs:39051` | **PRESENT** — dispatches by the **target** field's modelled type. ⚠️ **N undo entries, one per field.** Unknown names counted, never an error |
| Reset | `reset_form(&mut self, only: Option<&[String]>) -> Result<ResetOutcome, EditError>` | `edit.rs:37823` | **PRESENT** — `/V` is **removed** where no `/DV` exists, never blanked; never writes `/DV`; never recomputes. `ResetIneligible { PushButton, Signature, ReadOnly }` (`edit.rs:17198`) kept distinct |
| Burn to page content | `flatten_fields(&mut self, names: Option<&[&str]>) -> Result<FlattenOutcome, EditError>` | `edit.rs:39320` | **PRESENT** — appends `q <cm> /Name Do Q`, original streams byte-verbatim, field+widget dicts deleted. **Strict** certification gate. Values remain recoverable in the prior revision under incremental save |
| Apply a recompute plan | — | — | **ABSENT.** `grep -rn "apply_recompute" crates/pdfcer-core/src/` → **0 hits**. Applying a `RecomputePlan` is a loop the shell writes over `plan.changes`, calling `fill_text_field` |

## 2.2 ★ The option list `/Opt` — O205's second named defect

**PRESENT in the engine.** One verb, and it is a whole-list replace.

| Question | Answer | `file:line` |
|---|---|---|
| Is there **any** verb? | **Yes.** `edit_field(&mut self, fqn: &str, edit: &FieldEdit) -> Result<FieldEditOutcome, EditError>` | `edit.rs:25496` |
| The carrier | `FieldEdit::options: Option<Vec<forms::ChoiceOption>>` — *"the complete option list, REPLACING the existing one… A caller that wants to add one option sends the whole list with one more in it"* | `edit.rs:21382` struct; `options` in the `:21444` region |
| Builder | `FieldEdit::with_options(Vec<ChoiceOption>) -> Self` | `edit.rs:21815` |
| The write | `dict.insert(Name::from(b"Opt"), choice_opt_array(options))` | `edit.rs:25744` |
| Add an option | **PRESENT** — whole list with one more | |
| Remove an option | **PRESENT** — whole list with one fewer | |
| Reorder options | **PRESENT** — whole list in the new order (`/Opt` order is what every reader displays) | |
| Rename an option's display text | **PRESENT** | |
| Change an export value | **PRESENT** | |
| Export ≠ display | **PRESENT** — a one-string option writes a bare string, a two-string option writes the `[export display]` pair | `forms.rs:375` |
| Sorting | **ABSENT as an action, PRESENT as a claim + disclosure.** `FieldEdit::sort` writes `/Ff` bit 20, but **pdfcer never reorders `/Opt`** — Table 230 makes the stored order significant. `FieldEditOutcome::sort_claim_unmet` is set by comparing the effective display list against `is_sorted()` (~`edit.rs:25709`) | `edit.rs:22080` |
| Duplicates | **Refused** — `EditError::ChoiceOptionDuplicate` | `edit.rs:7354` |
| Stale selection after a removal | **Not repaired, disclosed.** `FieldEditOutcome::value_no_longer_fits: Option<String>` — a ready-made sentence. pdfcer neither truncates the data nor re-points the selection, and does not refuse the edit | `edit.rs:22080` |
| Appearance | **Regenerated** — `options_after.is_some()` is in `layout_changed` | `edit.rs:25825` |
| At creation | `NewChoiceField { options, combo, editable, multi_select, sort, … }` | `edit.rs:2764` |
| Empty `/Opt` | Allowed at creation, disclosed via `FieldAuthorDisclosures::has_no_options` — the field is **unfillable** | `edit.rs:1835` |
| Copy an option list | `FieldDefaults::options` and `formclip`'s copied-key list both carry `/Opt` | `edit.rs:1953`, `formclip.rs:241` |

## 2.3 `/DV` default value

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Set / remove `/DV` | `FieldEdit::default_value: Option<Option<String>>`; `with_default_value(…)`, `clearing_default_value()` | struct `edit.rs:21382`; builders `:21713`–`:21907`; writes `:25790`/`:25793` | **PRESENT** — three-state: untouched / set / removed. Type follows `/V`: **a name for `/Btn`**, a text string for `/Tx` and `/Ch` |
| `/DV` at creation | — | — | **ABSENT.** No `default_value` on any `New*` spec (`NewTextField` `:1609`, `NewCheckBox` `:2243`, `NewRadioButton` `:2333`, `NewChoiceField` `:2764`, `NewPushButton` `:3073`). A test at `:50581` asserts *"no /DV was invented for Drop"* |

## 2.4 `/Ff` flags, named individually

Written by `edit_field` (`edit.rs:25496`); `FieldEditOutcome` reports
`flags_before` / `flags_after` (`edit.rs:22080`). ⚠️ **Producer gates are
checked against the RESULT, not the request.**

| Bit | Name | `FieldEdit` member | Builder | Verdict |
|---|---|---|---|---|
| 1 | ReadOnly | `read_only: Option<bool>` | none; field is `pub` | **PRESENT** |
| 2 | Required | `required: Option<bool>` | none | **PRESENT** |
| 3 | NoExport | `no_export: Option<bool>` | `with_no_export` | **PRESENT** |
| 13 | Multiline | `multiline: Option<bool>` | none | **PRESENT** |
| 14 | Password | `password: Option<bool>` | none | **PRESENT** |
| 15 | NoToggleToOff | `no_toggle_to_off: Option<bool>` | none | **PRESENT** |
| 16 | Radio | — | — | **ABSENT** — type-defining; see §2.14 |
| 17 | Pushbutton | — | — | **ABSENT** — type-defining; see §2.14 |
| 18 | Combo | `combo: Option<bool>` | none | **PRESENT** — turning it off on an editable drop-down is refused (`ChoiceEditWithoutCombo`) |
| 19 | Edit | `editable: Option<bool>` | none | **PRESENT** — `Edit` without `Combo` refused |
| 20 | Sort | `sort: Option<bool>` | none | **PRESENT as a flag**, no reordering — §2.2 |
| 21 | FileSelect | `file_select: Option<bool>` | `with_file_select` | **PRESENT** |
| 22 | MultiSelect | `multi_select: Option<bool>` | none | **PRESENT** |
| 23 | DoNotSpellCheck | `no_spell_check: Option<bool>` | `with_no_spell_check` | **PRESENT** |
| 24 | DoNotScroll | `no_scroll: Option<bool>` | `with_no_scroll` | **PRESENT** |
| 25 | Comb | `comb: Option<bool>` | none | **PRESENT** — requires `/MaxLen`; `CombPreconditionUnmet` otherwise, even when the request never mentioned comb |
| 26 (`/Tx`) | RichText | — | — | **ABSENT as a setter.** Bit 26 can only be **cleared**, by `fill_text_field_downgrading_rich_text`. Nothing authors `/RV` or `/DS`: `grep -rn 'b"RV"'` → one write, `updated.remove(b"RV")` at `edit.rs:36381`; all others are reads or key-allowlists |
| 26 (`/Btn`) | RadiosInUnison | `radios_in_unison: Option<bool>` | none | **PRESENT**; also `NewRadioButton::radios_in_unison` (`edit.rs:2364`) |
| 27 | CommitOnSelChange | `commit_on_sel_change: Option<bool>` | `with_commit_on_sel_change` | **PRESENT** |

## 2.5 `/MaxLen`, `/Q`, `/DA`

| Key | Verb | `file:line` | Verdict |
|---|---|---|---|
| `/MaxLen` | `FieldEdit::max_len: Option<Option<i64>>` → insert / remove | `edit.rs:25726`, `:25729`; at creation `NewTextField::max_len` → `:22855` | **PRESENT** — set and remove. Shortening below the current value is **not** a truncation; `value_no_longer_fits` discloses |
| `/Q` quadding | `FieldEdit::quadding: Option<Option<i64>>`; `with_quadding`, `clearing_quadding` | `edit.rs:25751`, `:25754` | **PRESENT** — validated `0..=2`, else `QuaddingInvalid` (`edit.rs:7775`). ⚠️ **GAP AT THE PIN, CLOSED UPSTREAM: `/Q` was NOT in `layout_changed`** (`edit.rs:25825`), so the justification changed in the dictionary and the baked `/AP` kept the old alignment until something else forced a redraw. Filed as G022 and **shipped in `503ad9d4`** (Pass 308.4), one commit past this document’s pin. The reply adds a fact the request did not have: `/Q` is **inheritable** (§12.7.3.2), so `clearing_quadding()` resolves through `EditSession::inherited_quadding` and means *inherit*, not *left* — a GUI clear control labelled "Left" would be wrong under a centred parent |
| `/Q` at creation | — | — | **ABSENT** — no `quadding` on any `New*` spec |
| `/DA` font / size / colour | `FieldEdit::appearance: Option<FieldAppearance>`; `with_appearance`; `FieldAppearance { font: FieldFont, size: f64, color: vartext::TextColor }` | `edit.rs:21287`; write `:25766` | **PRESENT** — `FieldFont::Standard(Std14)` authors the resource under Acrobat's short keys (`Helv TiRo Cour ZaDb`…); `FieldFont::Resource(key)` only **checks**, refusing `FieldFontNotInResources` (`:7756`) **and listing what is available**. `size == 0.0` = auto-size, written as zero. Setting it regenerates the appearance. Stated limit: a `Resource` face falls back to **Helvetica metrics** for auto-size while writing the caller's name |
| `/DA` at creation | — | — | **ABSENT as a parameter.** Creation hard-codes it — `d.insert(Name::from(b"DA"), Object::String(da.clone()))` at `:22844` (text), `:26595` (push button), `:26842` (choice), `da` built inside the verb |
| `/AcroForm /DA`, `/DR` | Seeded when absent (`edit.rs:27383`); merged key-by-key with the target winning on document merge (`:41594`) | | **PRESENT as a consequence, no direct verb** |

## 2.6 `/MK` — every sub-key

| Key | Verb | `file:line` | Verdict |
|---|---|---|---|
| `/CA` caption | `WidgetEdit::caption: Option<String>`; `with_caption`. `Some("")` removes | struct `edit.rs:21582`; write in the `:25301`–`:25336` `/MK` merge block | **PRESENT** — also **redraws the artwork** |
| `/BG` background | `WidgetEdit::background: Option<MkColorEdit>`; `with_background(MkColor)`, `without_background()` | `MkColorEdit { Set(MkColor), Remove }` `edit.rs:21942`, `resolved()` `:21963` | **PRESENT** — all three states reachable: absent / Table 189 empty array / a colour. Gray, RGB **and CMYK** (painted as `k`/`K`, never converted) |
| `/BC` border colour | `WidgetEdit::border_color: Option<MkColorEdit>`; `with_border_color`, `without_border_color()` | same | **PRESENT** — also the control's **ink**: a check box's tick and a radio's dot take `/BC` (pdfcer's reading, not a clause) |
| `/R` rotation | `rotate_widget(&mut self, fqn: &str, index: usize, degrees: i64) -> Result<WidgetRotation, EditError>` | `edit.rs:24982` | **PRESENT for `/Tx` and `/Ch`. WRITE-ONLY AND SILENT FOR EVERY `/Btn`** — the key is written, the artwork never turns, and nothing discloses it. See §2.15 |
| `/RC` rollover caption | — | — | **ABSENT.** `grep -rn 'b"RC"'` → **1 hit**, `edit.rs:32360`, a FreeText `drop_keys` entry. Not modelled on `Widget`, not writable |
| `/AC` down caption | — | — | **ABSENT.** `grep -rn 'b"AC"'` → **0 hits** |
| `/I` `/RI` `/IX` icons | — | — | **ABSENT.** `grep -rn 'b"RI"'` → 0; `'b"IX"'` → 0. (`b"I"` hits are the choice `/I` index array and the `Inset` border byte at `edit.rs:1493`) |
| `/IF` icon fit | — | — | **ABSENT.** `grep -rn 'b"IF"'` → **0 hits** |
| `/TP` caption position | — | — | **ABSENT.** `grep -rn 'b"TP"'` → **0 hits** |
| `/MK` at creation | `chrome: annot_author::WidgetChrome` on all five `New*` specs; `.with_background()` / `.with_border_color()` | `edit.rs:1662`, `:2287`, `:2390`, `:2814`, `:3105` | **PRESENT** — one command, one undo entry. Creation verbs have **no unset** |
| Empty-`/MK` hygiene | A removal that empties `/MK` removes the dictionary (`edit.rs:25030`, `:25334`); other entries survive (a check box's `/CA` tick style survives `without_background()`) | | **PRESENT** |

## 2.7 `/BS` border

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Style + width | `WidgetEdit::border: Option<BorderSpec>`; `with_border`. `BorderStyle { Solid, Dashed, Beveled, Inset, Underline }` | `BorderStyle` `edit.rs:1471`, `BorderSpec` `:1513`; write `:25202` | **PRESENT** — per widget |
| At creation | `border: BorderSpec` on all five `New*` specs | `edit.rs:1652`, `:2270`, `:2374`, `:2807`, `:3098` | **PRESENT** |
| Scale `/BS /W` on resize | `ResizeOptions::scale_stroke_width` (geometric mean under non-uniform scale); reported as `WidgetEditOutcome::stroke_width` | `edit.rs:17624`; write `:25257`–`:25262` | **PRESENT**, off by default |
| `/RD` | `ResizeOptions::keep_rect_differences`; `WidgetEditOutcome::rect_differences_scaled` | `edit.rs:17632` | **PRESENT** |
| `/Border` (Table 164 array) | — | — | **Read-only.** `forms.rs:1723` falls back to `/Border` when `/BS` is absent; `formclip.rs:241`–`242` carries it. No verb writes it — the writer always emits `/BS` |
| ⚠️ Limit | `/BS /W` **does not** change a check box's or radio's drawn border, which pdfcer authors at a fixed 1.0 — the existing artwork contract, documented | | |

## 2.8 `/F` annotation flags

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Widget visibility | `WidgetEdit::visibility: Option<Visibility>`; `with_visibility`. `Visibility { VisibleAndPrints, ScreenOnly, PrintOnly, Hidden }` | `edit.rs:1540`; `WidgetEdit` `:21582` | **PRESENT** — four combinations out of a flag word admitting dozens |
| At creation | `visibility: Visibility` on all five specs; creation also writes `/F 4` (Print) explicitly (`:22849` region) | | **PRESENT** |
| Arbitrary `/F` bits on a widget | — | — | **ABSENT by design.** `set_annotation_flags` **refuses a `/Widget` by name** — two writers of one key with different vocabularies is how a field reaches a state its own editor cannot describe. A file may legitimately carry `Print \| NoZoom`, which `Visibility` cannot express; `Widget::annot_flags` exists so a control can say *"these flags are not something pdfcer can set"* |

## 2.9 Check-box / radio glyph style (check, cross, diamond, circle, square, star)

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| The six styles | `CheckStyle { Check, Cross, Star, Circle, Square, Diamond }`, `from_mk_caption_char`, `as_str`, `ALL` | `annot_author.rs:3781`, `~3829`, `~3843`, `~3877` | **PRESENT** — `/MK /CA` char mapping `annot_author.rs:3742`–`3760` (`Cross`→`b'8'`/a24, `Star`→`b'H'`/a35, `Diamond`→`b'u'`/a78, …) |
| Drawn how | **Vector artwork, not a ZapfDingbats `Tf`+`Tj`** | `annot_author.rs:4035`–`4116` | **PRESENT** — the glyph shapes are reimplemented as paths |
| Set at creation | `NewCheckBox::style`, `NewRadioButton::style` | `edit.rs:2282`, `:2385` | **PRESENT** |
| **Change on an existing field** | — | — | **ABSENT as a first-class verb.** `grep -n "CheckStyle" edit.rs` → only the two `New*` members, their two `default()`s (`:2433`, `:2583`), a comment at `:23840`, and one **read-back** at `:38568` inside `build_button_states`. No `style` member on `FieldEdit` (`:21382`) or `WidgetEdit` (`:21582`). **The only route is indirect:** `WidgetEdit::caption` writes `/MK /CA`, and the next appearance rebuild decodes that first byte back into a `CheckStyle`. **A GUI style picker must write the mapped character, not a style name.** |

## 2.10 `/AA` additional actions (format / keystroke / validate / calculate)

| Capability | `file:line` | Verdict |
|---|---|---|
| Read + disclose presence | `Field::has_additional_actions` (`forms.rs:1584`); `form_script::inventory` (`inventory.rs:181`); `forms.rs:2208`, `:2250`, `:2342`, `:2974`; `FieldClip::carries_actions()` (`formclip.rs:378`), `carries_calculation()` (`:393`) | **PRESENT** |
| Classify a script without running it | `form_script::classify(js, trigger) -> ScriptClass` (`form_script/mod.rs:355`); `Trigger` `:89` (`/C` calculate is the only trigger whose helper may change `/V`); `CalcHelper` `:185`, `FormatHelper` `:211`, `AdvisoryHelper` `:273` | **PRESENT** — *"A false positive is far worse than a false negative."* An unreadable script is always `Custom`, never optimistically a known built-in |
| Author or edit `/AA` | — | **ABSENT.** `grep -rn 'b"AA"'` → every hit is a read, a `contains_key`, or a key-allowlist entry, **except two removals**: `field.remove(b"AA")` at `edit.rs:48704` and `dict.remove(b"AA")` at `:49030`. `/AA` can be **carried by a paste** (`formclip.rs:241`) or **stripped**; it cannot be written |
| Execute `/AA` | — | **ABSENT, permanently.** No interpreter, no expression evaluation, no DOM, no `event` object, no trigger dispatch. Recompute is **operator-invoked and undoable**, never a load or save side effect; the source script is left in place |

## 2.11 `/A` push-button actions

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Write / replace / remove | `set_button_action(&mut self, fqn: &str, action: Option<ButtonAction>) -> Result<ButtonActionChange, EditError>` | `edit.rs:36852` | **PRESENT** — `None` removes any action, **including one pdfcer would never author**; `ButtonActionChange::replaced` names it |
| `/ResetForm` | `ButtonAction::ResetForm { scope: ResetScope }`; `ResetScope { All, Only(Vec<String>), Except(Vec<String>) }` | `edit.rs:19268`, `:18736` | **PRESENT** — `All` **omits** `/Fields` (an empty array means "reset zero fields") |
| `/SubmitForm` | `ButtonAction::SubmitForm(SubmitSpec)`; `SubmitFormat { Fdf(FdfOptions), Html { get, coordinates }, Xfdf, WholeDocument }` | `edit.rs:19279`, `:18887` | **PRESENT** — format is an enum, not a flag word, so most illegal Table 237 combinations are unrepresentable. Destination must be absolute + 7-bit ASCII (a *decidability* rule). **Nothing is ever sent — `pdfcer-core` has no network code** |
| `/GoTo` | `ButtonAction::GoToPage { page_index, view: PageView }`; `PageView { WholePage, FullWidth, TopLeft }` | `edit.rs:19292`, `:19062` | **PRESENT** — page written as an **indirect reference**, so it survives a reorder |
| `/Named` | `ButtonAction::Named(NamedAction)`; `NamedAction { NextPage, PrevPage, FirstPage, LastPage }` | `edit.rs:19302`, `:19022` | **PRESENT** — Table 211's four |
| `/URI` | `ButtonAction::Uri { uri }` | `edit.rs:19374` | **PRESENT** — authored as data; pdfcer never follows one |
| `/Hide` | `ButtonAction::SetHidden { targets: Vec<String>, hidden: bool }` | `edit.rs:19341` | **PRESENT** — `/H` written explicitly both ways (its spec default is `true`). A grouping field name is **refused** (`ButtonActionHideTargetNotTerminal`). An assignment, not a toggle. Reaches **widgets only** — a `/Text` note or a stamp is not expressible |
| `/JavaScript`, `/Launch` | — | — | **ABSENT, refused permanently.** `ButtonActionState::Foreign(String)` names such an action on read so a control knows not to offer to replace it |
| Other `/A` subtypes (`GoToR`, `Movie`, `Thread`, `SetOCGState`, `Rendition`, `Trans`…) | — | — | **ABSENT** — `ButtonAction` is `#[non_exhaustive]` with the six variants above only |
| Retarget / orphan on rename + delete | `FieldRename::action_targets_retargeted` (repairs, same undoable command); `FieldDeletion::action_targets_orphaned` (**counted, never repaired**) | `edit.rs:22350`, `:20362` | **PRESENT** — ⚠️ JavaScript is **never** rewritten, and a dangling-reference census cannot see it: a name is not a reference |

## 2.12 `/Lock` and `/CO`

| Key | `file:line` | Verdict |
|---|---|---|
| `/Lock` | `grep -rn 'b"Lock"'` → **2 hits**: `edit.rs:27041` (a **read** at signing time, feeding a `/FieldMDP` transform, §12.8.2.4) and `formclip.rs:242` (carried-key list) | **READ-ONLY. No writer.** A GUI cannot author a field-lock dictionary |
| `/CO` calculation order | `forms.rs:1218` (read into `calc_order` / `calc_order_count`); `edit.rs:40109`–`40145` (**pruned** on field delete; the prune also never *creates* the array, because `co_pruned` is an `and_then` over `co_original` — **that is a reading of the code, not a comment in it**); `:48902`–`:48907` (**appended** when a paste carries a calculation) | **PRESENT only as a consequence. No reorder verb, no add/remove verb** — deliberately left out of `FieldEdit` as structural |

## 2.13 `/T`, `/TU`, `/TM`

| Key | Verb | `file:line` | Verdict |
|---|---|---|---|
| `/T` rename | `rename_field(&mut self, fqn: &str, new_partial: &str) -> Result<FieldRename, EditError>` | `edit.rs:25971` | **PRESENT** — `new_partial` is **one path segment**, never an FQN; a period is refused (`EmptyNameSegment` / `DottedPartialName`). `FieldRename::descendants_renamed` — one request can rename six fields, their FQNs re-deriving with **no object of their own written**, breaking every FDF and JavaScript reference. Same-name create **merges**; same-name rename **refuses** (deliberate asymmetry) |
| `/T` validate | `forms_author::validate_partial_name(&str) -> Result<(), FormAuthorError>` | `forms_author.rs:465` | **PRESENT** — askable, not only enforced |
| `/TU` tooltip | `FieldEdit::tooltip: Option<TooltipChoice>`; `with_tooltip`. `TooltipChoice { Undecided, Text(String), Declined }` | `edit.rs:1751`; writes `:25736`/`:25739` | **PRESENT** — `Option<String>` cannot distinguish *"chose not to"* from *"nobody thought about it"*, so `Undecided` **refuses** creation (`TooltipDecisionRequired`, `:7375`). `FieldEditOutcome::tooltip_removed` reports a removal. Also `tooltip: TooltipChoice` on all five `New*` specs |
| `/TM` mapping name | `FieldEdit::mapping_name: Option<Option<String>>`; `with_mapping_name`, `clearing_mapping_name` | writes `edit.rs:25776`/`:25779` | **PRESENT** — `Some(None)` removes, reverting export to the field's own name; not the same as an empty string |

## 2.14 Create, delete, duplicate, merge/split, adopt

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Create text field | `add_text_field(&mut self, spec: &NewTextField) -> Result<FieldAuthorOutcome, EditError>` | `edit.rs:22681` | **PRESENT** |
| Create check box | `add_check_box(&mut self, spec: &NewCheckBox) -> …` | `edit.rs:23754` | **PRESENT** |
| Create radio button | `add_radio_button(&mut self, spec: &NewRadioButton) -> …` | `edit.rs:23999` | **PRESENT** — ⚠️ **the one verb whose `Err` does not imply "nothing happened"**: on the merge-into-existing-group branch it commits `AddFormField`, then calls `set_button_state`, which runs its own guards. On that error path a shell must `undo()` or re-query |
| Create push button | `add_push_button(&mut self, spec: &NewPushButton) -> …` | `edit.rs:26527` | **PRESENT** — `FieldAuthorDisclosures::push_button_inert` (`:1908`): *the only creation verb whose successful result is a control that does not work* |
| Create combo / list box | `add_choice_field(&mut self, spec: &NewChoiceField) -> …` | `edit.rs:26764` | **PRESENT** |
| Create signature field | via `sign(...)` with a `field_name`; `CommandKind::AddSignatureField` | signing path | **PRESENT** — no `add_signature_field` verb of its own |
| Create barcode field | — | — | **ABSENT, never planned** (`03-capabilities.md:620`) |
| Creation atomicity | field dict + widget + baked `/AP` + page `/Annots` + `/AcroForm /Fields` in **one undo entry**, `CommandKind::AddFormField` | | **PRESENT** |
| Creation disclosures | `FieldAuthorDisclosures { tooltip_declined, tagged_document, structure_tab_order, has_no_options, group_flags_ignored, defaults_type_mismatch, defaults_on_state_ambiguous, push_button_inert, push_button_no_caption }` | `edit.rs:1805` | **PRESENT** |
| Delete a field | `delete_field(&mut self, fqn: &str) -> Result<FieldDeletion, EditError>` | `edit.rs:24224` | **PRESENT** — every widget + dict + registration + emptied grouping nodes |
| Delete a grouping subtree | `delete_field_group(&mut self, fqn) -> Result<FieldGroupDeletion, EditError>` | `edit.rs:24353` | **PRESENT** — refuses a terminal with `NotAGroupingNode`, deliberately **not** redirected to `delete_field` |
| Delete one widget | `delete_widget(&mut self, fqn: &str, index: usize) -> Result<FieldDeletion, EditError>` | `edit.rs:24640` | **PRESENT** — siblings survive |
| Delete a widget via the annotation verb | — | — | **ABSENT by design** — `delete_annotation` refuses with `EditError::AnnotationIsWidget` |
| Duplicate a field | `copy_field(&self, fqn) -> Result<formclip::FieldClip, EditError>` + `paste_field(&mut self, clip, page_index, rect, policy: &FieldPastePolicy) -> Result<FieldPasteOutcome, EditError>` | `edit.rs:48021`, `:48081` | **PRESENT** — `FieldPastePolicy::NewField { name, tooltip, copy_value, copy_actions }` (an independent field; `copy_value` and `copy_actions` **default OFF** — a value is content, an action is behaviour) or `AdditionalWidget { existing }` (another widget of the same field — the **high-fidelity** route). `FieldClip::to_bytes()`/`from_bytes()` (`formclip.rs:463`/`:534`) makes it a real clipboard |
| Cut a field | `cut_field(&mut self, fqn) -> Result<formclip::FieldCut, EditError>` | `edit.rs:48633` | **PRESENT** — copy + delete coalesced into one undo entry |
| Add a widget to an existing field on another page | `paste_field(.., &FieldPastePolicy::AdditionalWidget { .. })` | `edit.rs:48081` | **PRESENT** — the *only* route to spreading one field across pages. Places **one** widget even from a multi-widget clip, and says so (adding all would give duplicate export values, i.e. radio buttons that select together) |
| Move an existing widget to another page | — | — | **ABSENT.** `move_widget` takes `dx`/`dy` only (`edit.rs:26145`); `WidgetEdit` (`:21582`) has no `page` member. Only delete-and-paste |
| Split a field into independent fields | — | — | **ABSENT.** `grep -rn "split_field"` → 17 hits, **all** `forms_author::split_field_path` (a name *parser*, `forms_author.rs:498`) or its tests. `grep -rn "merge_field"` → 0. Promoting a terminal into a grouping node is documented as *"a different verb with its own confirmation… **It is not built.**"* |
| Register an orphaned widget as a field | `adopt_widget(&mut self, widget: ObjId, name: Option<&str>) -> Result<AdoptOutcome, EditError>` | `edit.rs:42857` | **PRESENT** — writes **no** geometry, appearance or value; only `/AcroForm` registration (creating `/AcroForm` if absent) plus `/T` when renaming. Predicate is `/T`, not `/FT`. A bare `/Kids` widget (a radio member) **cannot** be adopted |
| Merge whole documents' forms | `merge_document(...)`; `pageops::assemble` | `pageops/assemble.rs:857` | **PRESENT** — collisions **renamed**, not fused (`fields_renamed`); `/Parent` written back so §12.7.3.2 inheritance resolves; ⚠️ `parse_acroform` walks downward and **cannot see** a `/Parent`-only link |
| Insert pages carrying fields | `insert_pages(...)` | `edit.rs` | **PARTIAL, by design** — *the widgets arrive, their fields do not*. `InsertOutcome::orphaned_widgets` is exact; a field whose widgets straddle inserted and non-inserted pages leaves a permanent residue |
| **Change a field's type** | — | — | **ABSENT, by design and permanently.** `grep -rn "fn set_field_type\|convert_field"` → **0 hits each**; `FieldEdit` has no `field_type` member, and its doc says why: *"Acrobat has offered no field-type conversion since Acrobat 6 — the only route is delete-and-recreate — and pdfcer models the same limit by making the request **unrepresentable**."* |
| **Change a field's value through `FieldEdit`** | — | — | **ABSENT, by design** — `FieldEdit` has no `value` and no `name`; use the fill verbs and `rename_field` |

## 2.15 Widget rotation — and what it actually turns

| Fact | `file:line` |
|---|---|
| `rotate_widget(&mut self, fqn: &str, index: usize, degrees: i64) -> Result<WidgetRotation, EditError>` | `edit.rs:24982` |
| Writes `/MK /R`. Rotating to 0 **removes the key**, and removes `/MK` if that emptied it | `edit.rs:25020`–`:25032` |
| Multiples of 90 only — else `WidgetRotationNotQuarterTurn` (`edit.rs:6645`). Reduced into `[0,360)`, with `WidgetRotation::normalised` reporting it | `edit.rs:22135` |
| **`/Rect` does not move.** What turns is the *artwork*: the appearance is rebuilt into a `w`/`h`-swapped `/BBox` and stood upright by `/Matrix` (`quarter_turn_matrix` `edit.rs:38667`, applied `:38124`; 0 emits nothing) — **but only on the `/Tx` and `/Ch` path.** The `w`/`h` swap and the matrix both live in `regen_field_appearance` (`edit.rs:38090`-`:38124`), which no `/Btn` ever reaches | `edit.rs:24806` |
| ⚠️ **`/MK /R` is COUNTERCLOCKWISE while a page's `/Rotate` is clockwise.** Table 189 (=2.0 T192) and Table 30 are word-for-word parallel and the direction word is the only difference | |
| `WidgetRotation { name, index, was: Option<i64>, now: Option<i64>, normalised, appearance_regenerated, appearance_stale, siblings_untouched }` — **`was: None` means the file is SILENT, and it is not `Some(0)`** | `edit.rs:22135` |
| Rotation is a **widget** property: one field's widgets can each be rotated differently; `siblings_untouched` counts the ones left alone | |
| When pdfcer cannot redraw, `/MK /R` is written and **the pixels do not move** — and per erratum #56 a conforming PDF 2.0 reader shows that widget upright however `/R` reads. `appearance_stale` carries a sentence; show it, do not paraphrase | `edit.rs:25058`-`:25069` |
| ★ **BUT `appearance_stale` IS NOT THE SET OF CASES THAT DO NOT TURN, AND THE SENTENCE ITSELF MISDESCRIBES THEM.** It is set only `if !appearance_regenerated` (`edit.rs:25058`). For **every** `/Btn` kind — check box, radio **and** push button — `regen_after_property_change` forks to `regen_button_appearance` (`edit.rs:24806`), which reads `pending.rect_for` (`:38389`), `pending.caption_for` (`:38393`) and `pending.chrome_for` (`:38400`) and **never `rotation_for`**; it then calls `build_button_states(field, kind, w, h, caption, chrome)` (`:38402`), whose signature (`:38534`-`:38549`) **takes no angle**. `grep -c rotation_for` over the whole engine: **3 hits, one of them the only call — `edit.rs:38070`, inside `regen_field_appearance`.** So a check box whose artwork pdfcer drew rewrites byte-identical streams, returns `Ok(true)` (`:38423`), leaves `appearance_stale` **`None`**, and moves **zero pixels** while `WidgetRotation::now` reports the new angle. Silent success. **This is O205's "the rotate buttons don't work for check boxes", and it is engine-side** | `edit.rs:38349`-`:38423` |
| The sentence's own enumeration is wrong in both directions: it names *"a push button's caption artwork"* as a case that fires, but a push button pdfcer drew takes the `Ok(true)` path and never sees it; and when it does fire for a **foreign check box** it tells the operator the stream is a push button's caption artwork. Do not read the sentence as a description of which kinds turn | `edit.rs:25059`-`:25068` |
| The renderer cannot compensate, by design: `grep -rn 'b"MK"' crates/pdfcer-render/src/` → **0 hits**, and §12.5.5 is implemented from `/BBox` + `/Matrix` alone. `annot_author`'s module invariant fixes *"every appearance here is authored with `Matrix` = identity and `BBox` = the annotation `/Rect`"* (`annot_author.rs:31`-`:33`), so a button state has no `/Matrix` to turn. **The fix is therefore the same one the text path already took** — swap `w`/`h` and emit `quarter_turn_matrix` — not a renderer change, which erratum #56 forbids | `annot_author.rs:31`-`:33` |
| Second-order, and it limits what a fix can be *seen* to do: three of the six check styles are rotationally symmetric. `Circle` (`annot_author.rs:4186`) and `Square` (`:4197`) are invariant under any quarter turn, and `Cross` (`:4155`) under 90°. Only `Check`, `Star` and `Diamond` will visibly change | |

## 2.16 Tab order

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Read the computed sequence | `page_tab_sequence(&self, page_index)` | `edit.rs:29596` | **PRESENT** — reports which of the six `/Tabs` states produced it, and every excluded annotation with its reason |
| Change the order | `reorder_annotations(&mut self, page_index: usize, new_order: &[ObjId]) -> Result<AnnotsReorder, EditError>` | `edit.rs:30278` | **PRESENT** — permutes `/Annots`, which **is** the tab order under `/Tabs /R`/`/C`/absent. Moves references and nothing else, so every widget keeps its id, its field, its `/Parent` chain and its `/AA`. Refuses `AnnotsNotAPermutation`, `AnnotsDuplicateReference`, `TrapNetMustStayLast`, `AnnotStatesMismatch` |
| Write `/Tabs` | — | — | **ABSENT.** `grep -rn 'b"Tabs"'` → **4 hits, every one a `.get()` read**: `edit.rs:23140`, `:29619` (`PageTabs::from_entry`), `:30300`, `:53981` (a test). `grep -rn "set_tabs\|set_tab_order"` → **0 hits each**. `reorder_annotations`' own doc explains the refusal at length (`/A` is a PDF 2.0 value; PDF/UA-1 §7.18.3 requires `/S`) and says *"Recording the order is therefore a separate, explicit act with its own verb"* — **that verb does not exist at this sha** |
| Structure tab order | — | — | **ABSENT.** Under `/Tabs /S` a newly created field has **no tab position at all**; disclosed via `FieldAuthorDisclosures::structure_tab_order` (`edit.rs:1832`) and `TabOrderBasis::NotDerivedStructure` (`:19985`), never fixed |
| Radio-group collapse | — | — | **ABSENT by design** — one radio group is several annotations and **one** tab stop; that is a grouping of *fields*, so the shell must collapse it with `Field::widgets` after reading the sequence |

## 2.17 Geometry

| Capability | Verb | `file:line` | Verdict |
|---|---|---|---|
| Translate a widget, carrying its artwork | `move_widget(&mut self, fqn: &str, index: usize, dx: f64, dy: f64) -> Result<WidgetMove, EditError>` | `edit.rs:26145` | **PRESENT** — **no regeneration needed**: §12.5.5 step b makes matrix **A** a pure translation. `WidgetMove::siblings_left_behind` |
| Resize a widget | `WidgetEdit::rect: Option<page_tree::Rect>`; `with_rect` → `edit_widget` | `edit.rs:25136` | **PRESENT** — a changed **extent** rebuilds the appearance, because the same §12.5.5 clause would otherwise *scale* the old artwork (a text field dragged twice as wide would render its text twice as wide instead of gaining room). `WidgetEditOutcome::resized` says which path was taken |
| Move/resize via the annotation verbs | — | — | **ABSENT by design** — `move_annotation` and `resize_annotation` **refuse a `/Widget` by name** |

---

# 3. Appearance regeneration

Governing rules: pdfcer paints the baked `/AP` and **never** reconstructs an
appearance from `/MK` at display time, so a colour must go into the stream.
**One** §12.7.3.3 appearance engine, one regeneration path. `/NeedAppearances`
is disclosed, never a silent on-load regenerate.

## 3.1 The dispatcher

`fn regen_after_property_change(&mut self, field: &forms::Field, flags:
forms::FieldFlags, objects: &mut Vec<ObjectWrite>, pending:
&PendingWidgetEdit, appearance: Option<&FieldAppearance>) -> Result<bool,
EditError>` — `edit.rs:24758`

| `FieldType` | Rebuilt? | How |
|---|---|---|
| `Text` | **YES** | decoded `/V` (empty when unfilled), multiline per `MULTILINE`, through `vartext::build_variable_text` (`vartext.rs:727`) |
| `Choice` | **YES** | `choice_display_text(field)` (`edit.rs:22464`) — maps each selected export to its **display** string and joins with newlines; multiline = `!COMBO` |
| `Button` | **YES** | `regen_button_appearance(...)` → `build_button_states` (~`edit.rs:38560`): Check → `CheckStyle::from_mk_caption_char(/MK /CA first byte).unwrap_or_default()` then `build_check_box_appearances(w,h,style,chrome)`; Radio → `build_radio_button_appearances(w,h,chrome)`; Push → `build_push_button_appearance(w,h,caption,&da,&fonts,chrome)` |
| `Signature` | **NO** | `_ => return Ok(false)` |
| untyped | **NO** | same arm |

Fonts offered are always `Helv` plus the caller's key. `widget_chrome(widget)`
(`edit.rs:38611`) = `WidgetChrome::new(widget.background,
widget.border_color)`. The merged field+widget shape folds the new `/AP /N`
into the field-dict write (one merged write per id per command).

## 3.2 Trigger matrix

| Change | Regenerates? | Evidence |
|---|---|---|
| `fill_text_field`, `fill_text_field_downgrading_rich_text` | **YES** | `edit.rs:36210`, `:36254`; `FillOutcome::{applied_autosize, applied_autosize_bound, unencodable_chars}` |
| `set_choice_value` | **YES** | `edit.rs:38709` — `/V` + `/I` + `/TI` + regenerated `/AP` |
| **`set_button_state`** | **NO** | `edit.rs:36449` — writes only `/V` and each widget's `/AS`, relying on the pre-existing on/off appearance states. Verified by reading the whole body `36449`–`36630`: no regen call |
| `reset_form` | **NO recompute**; values removed, not blanked | `edit.rs:37823` |
| `move_widget` | **NO** — correct; matrix A is a pure translation | `edit.rs:26145` |
| `edit_widget` with a changed **extent** | **YES** | `edit.rs:25384`: `let needs_regen = resized \|\| edit.border.is_some() \|\| edit.caption.is_some() \|\| edit.background.is_some() \|\| edit.border_color.is_some();` |
| `edit_widget` `/BS` border | **YES** | same line |
| `edit_widget` `/MK /CA` caption | **YES** | same line (the caption is drawn *into* a push button's plate) |
| `edit_widget` `/MK /BG` or `/BC`, colour only | **YES** | same line — the retirement of the old *"pdfcer's renderer does not paint `/MK` colours"* limit |
| `edit_widget` `/MK` colour **removal** (`MkColorEdit::Remove`) | **YES** — the builder falls back to its own default | `edit.rs:21942` |
| `edit_widget` `/F` visibility only | **NO** — `NotNeeded` | `edit.rs:25384` (absent from the list) |
| `edit_field` `multiline`, `comb`, `combo`, `max_len`, `appearance` (`/DA`), `options` (`/Opt`) | **YES** | `edit.rs:25825`: `let layout_changed = edit.multiline.is_some() \|\| edit.comb.is_some() \|\| edit.combo.is_some() \|\| edit.max_len.is_some() \|\| edit.appearance.is_some() \|\| options_after.is_some();` |
| **`edit_field` `quadding` (`/Q`)** | **NO at the pin; YES from `503ad9d4`** | `edit.rs:25825` — `/Q` was written at `:25751` and absent from `layout_changed`, so the baked appearance kept the old justification. G022, Pass 308.4: `/Q` is in the gate **and** the cloned snapshot is repaired beside the `/DA` patch, which is what stops a rebuild-from-stale reading as a fix |
| `edit_field` any other flag, `/TU`, `/TM`, `/DV`, `no_export`, the four advisory flags | **NO** | same |
| `rotate_widget` | **YES for `/Tx` and `/Ch`.** For **every `/Btn`** the regenerator runs and reports success while ignoring the angle — `regen_button_appearance` never reads `pending.rotation_for` and `build_button_states` takes no angle, so byte-identical streams are rewritten and `Ok(true)` returned. **A `/Btn` therefore reports `appearance_regenerated = true`, `appearance_stale = None`, and does not turn.** `/Sig` and untyped take `_ => Ok(false)` and *are* disclosed | `edit.rs:24806`, `:38402`, `:38423`, `:38534` |
| `add_*` creation verbs | **YES** — the baked `/AP` is part of the single undo entry | `edit.rs:22681` etc. |
| Whole-document | `regenerate_appearances(&mut self) -> Result<RegenOutcome, EditError>` — `edit.rs:39180` | Rebuilds every text/choice field with no usable `/AP`, **or**, when `/NeedAppearances true`, every variable-text field. **Buttons are untouched.** Then removes `/NeedAppearances` |

## 3.3 Outcome vocabulary

`WidgetEditOutcome { name, index, rect_before, rect_after, resized,
appearance: AppearanceOutcome, appearance_regenerated, stroke_width,
rect_differences_scaled, siblings_untouched, appearance_stale }` —
`edit.rs:22261`

`enum AppearanceOutcome { NotNeeded, Regenerated, RecordedNotPainted(String) }`
— `edit.rs:22220`. `needs_disclosure()` is the one-call question;
`RecordedNotPainted` **owes the operator its string**.

## 3.4 The ownership test

**Ownership is decided by bytes, not by parsing.** pdfcer rebuilds a button's
artwork only when the existing `/AP` is **byte-identical** to what pdfcer would
draw for those properties *at the size it is currently drawn at* — the same
test `resize_annotation` and `set_markup_style` use. *"Could pdfcer draw a check
box like this?"* is true of every conforming check box in existence, which is
precisely the set that must not be touched. A foreign `/DA` fails the
push-button match and the button is treated as foreign. A resize that would
stretch unrebuildable artwork returns `EditError::ResizeAppearanceNotRebuildable
{ subtype: "Widget", .. }` unless `ResizeOptions::allow_appearance_distortion`
is set.

## 3.5 `/NeedAppearances`

| Behaviour | `file:line` | Verdict |
|---|---|---|
| Read + disclosed | `AcroForm::need_appearances` (`forms.rs:1208`) | **PRESENT** — the producer's assertion, never acted on silently |
| **Never** silently regenerated on load | | standing rule |
| Cleared on pdfcer's own output | `clear_need_appearances_write` `edit.rs:40214`/`:40232`/`:40249` | **PRESENT** — `RegenOutcome::need_appearances_cleared` |
| Set `true` | Only as a **logical OR** on import/merge: `edit.rs:41533`–`:41536`, `pageops/assemble.rs:806`/`:867` | **PRESENT only there.** The only `insert`s are those two OR sites. No verb turns it on |

## 3.6 Known appearance limitations

| Limitation | Backing |
|---|---|
| **A list box's selection highlight is never drawn.** `choice_display_text` (`edit.rs:22464`) reduces the selection to display strings joined with newlines and hands them to `build_variable_text` as plain multiline text. No selection rectangle, no inverted band, no `/TI` scroll clipping in the artwork. `grep -rni "highlight" vartext.rs` → **0 hits**; every `highlight` hit in `annot_author.rs` is the `/Highlight` markup annotation (`:284`, `:680`, `:704`). `/TI` is computed only for the `FillOutcome::top_index` disclosure via `vartext::visible_line_count` (`vartext.rs:492`) |
| **Signature appearances are never rebuilt** — the `_ => Ok(false)` arm (`edit.rs:24758`) |
| **Untyped fields (no `/FT` anywhere up the chain) are never rebuilt** — same arm |
| **`/Q` changes do not redraw** — §3.2 |
| **A `/Btn` rotation is written and never drawn, and reports success** — §2.15 and §3.2. `regen_button_appearance` (`edit.rs:38349`) ignores the staged angle; `build_button_states` (`:38534`) takes none. Distinct from every other row here in that **nothing discloses it**: `appearance_stale` is gated on `!appearance_regenerated` (`:25058`) and the button rebuild returns `Ok(true)` (`:38423`) |
| **A `FieldFont::Resource` face falls back to Helvetica metrics** while writing the caller's resource name |
| **`/BS /W` does not change a check box's or radio's drawn border**, fixed at 1.0 |
| **No font substitution in variable text** — `VarTextError::FontUnresolved` refuses rather than substituting, because a substituted font changes glyph advances |
| **A push button created by an older build carries `/MK` RGB triples** while the artwork painted DeviceGray; the ownership test reads that as foreign and `edit_widget` reports `RecordedNotPainted`. Re-setting either colour rebuilds it |
| **Rich text is never authored** — `/RV` can only be deleted; rendering rich text at all *"is policy, not conformance"* |

---

# 4. Refusals — every `EditError` a form verb can return

`enum EditError` at `edit.rs:6120`, `#[non_exhaustive]`, **135 variants**.
Deliberately **no catch-all "edit failed"**, **no `impl EditError` block and no
`is_*` classification helpers** — callers discriminate with `matches!`.
`RefusalKind` is implemented only for `text_edit::EditError`, **not** for this
enum.

| Variant | `edit.rs:` | What it means |
|---|---|---|
| `FieldNotFound { name }` | 8185 | No field with that FQN |
| `NoInteractiveForm` | 8309 | No `/AcroForm` at all |
| `NotAGroupingNode` | group-delete guard | `delete_field_group` on a terminal — deliberately **not** redirected |
| `FieldNotFillable` | 8245 | Push button, signature or read-only |
| `FieldIsRichText` | 8238 | `fill_text_field` on a `/Tx` with bit 26 — the downgrade verb is the only route |
| `FieldStateUnknown` | 8254 | A button whose current state cannot be determined |
| `ChoiceValueNotInOptions` | 8298 | A selection not in `/Opt` on a non-editable choice |
| `ChoiceRequiresMultiSelect` | 8284 | More than one selection without bit 22 |
| `ChoiceOptionDuplicate` | 7354 | Duplicate export in a new `/Opt` — the fill verb resolves to the first match |
| `ChoiceEditWithoutCombo` | 7222 | The **resulting** field would have `Edit` without `Combo` |
| `ChoiceEditRequiresCombo` | 7257 | An editable **list** box at creation |
| `CombPreconditionUnmet` | 7176 | `Comb` needs `/MaxLen` and forbids multiline/password/file-select — checked against the **result** |
| `QuaddingInvalid` | 7775 | `/Q` outside `0..=2`, validated before anything is written |
| *`MaxLen` shortening* | — | **not** an error — `value_no_longer_fits` discloses instead |
| `FieldFontNotInResources` | 7756 | `FieldFont::Resource(key)` not in `/AcroForm /DR /Font` — **and it lists what is available** |
| `FieldPropertyTypeMismatch` | 7200 | A `FieldEdit` property that does not apply to this `/FT` |
| `CheckBoxOnStateInvalid` | 7343 | e.g. `Off` as an on-state |
| `RadioExportValueTaken` | 7310 | Two members of one group with the same export |
| `RadioGroupUsesPositionalOpt` | 7333 | The group indexes `/Opt` positionally — pdfcer will not guess |
| `FieldRectDegenerate` | 7162 | Zero-extent `/Rect` (⚠️ `edit.rs:25133`'s doc misnames this `DegenerateFieldRect`) |
| `WidgetIndexOutOfRange` | 7271 | `index` past `field.widgets.len()` |
| `WidgetRectMissing` | 7289 | The widget has no `/Rect` |
| `WidgetRotationNotQuarterTurn` | 6645 | `rotate_widget` with a non-multiple of 90 |
| `ResizeAppearanceNotRebuildable { subtype: "Widget", .. }` | resize guard | A resize would stretch artwork pdfcer did not draw; `allow_appearance_distortion` proceeds and discloses |
| `FieldNameEmpty` | 7385 | Empty `/T` |
| `FieldNameTaken { name }` | 7545 | Resulting name exists — **never auto-suffixed** |
| `FormAuthoring(FormAuthorError)` | 7756 region, `#[from]` | Wraps `forms_author.rs:147`'s `FieldTypeCollision`, `NameIsGroupingNode`, `FieldPathCrossesTerminal { fqn, terminal }`, `RenameCollision`, `DottedPartialName`, `EmptyNameSegment`, `EmptyName`, `PathTooDeep`. **Unwrap and present the inner** |
| `FieldAuthoringRefusedXfa` | 7149 | Field **creation** refused on an XFA document. Deliberate asymmetry: **filling succeeds and discloses** (`xfa_may_disagree`) because a one-sided *add* makes two viewers disagree about how many fields exist |
| `TooltipDecisionRequired` | 7375 | `TooltipChoice::Undecided` at creation or paste |
| `NotAWidget { id }` | 7414 | `adopt_widget` on a non-`/Widget` |
| `WidgetHasNoFieldIdentity { id }` | 7437 | `adopt_widget` with no `/T` and no `name` |
| `WidgetAlreadyOwned { id }` | 7444 | Already reachable from `/AcroForm /Fields` |
| `FieldHasNoWidget` | 7513 | `copy_field` on a value-only terminal (legal under Table 220) |
| `FieldClipUntyped` | 7540 | `paste_field` from a clip whose source had no `/FT` |
| `SignedFieldNotCopyable` | 7527 | `copy_field`/`cut_field` on a **signed** `/Sig`. An **unsigned** `/Sig` copies normally |
| `CutWouldNotSurvive { subtype }` | cut guard | A `/Widget` refuses the *annotation* cut — `cut_field` is its verb |
| `FieldObjectIsInPageTree { name, object }` | 6776 | The file says one object is both a form field and a page. Guards `delete_field`, `delete_field_group`, `delete_widget` **and `flatten_fields`** |
| `AnnotationObjectIsStructural` | annot guard | An `/Annots` entry is the catalog, **the `/AcroForm`**, a page-tree node or a page |
| `AnnotationIsWidget` | annot guard | `delete_annotation` on a widget |
| `CarrierIsNotAStream { key, id, what, verb }` | categorical | e.g. `/AP /N` holding a non-stream. An absent or null id passes |
| `ButtonActionWrongFieldType` | 6659 | `set_button_action` / `button_action` on a non-push-button |
| `ButtonActionDestination` | 6689 | Submit destination not absolute + 7-bit ASCII |
| `ButtonActionSubmitFlags` | 6707 | `only_current_user_annotations` without `include_annotations` |
| `ButtonActionHideTargetNotTerminal` | 6735 | `SetHidden` naming a grouping field — Table 210 says nothing about descendant expansion, so both readings are available and one is a silent no-op |
| `ButtonActionHideNoTargets` | 6790 | `SetHidden` with an empty target list |
| `VariableText(VarTextError)` | 7900, `#[from]` | Wraps `vartext.rs:224` (`FontUnresolved`, …) — **unwrap and present the inner** |
| `FormData(FdfError)` | 8312, `#[from]` | Wraps `fdf.rs:136` |
| `CsvError` | `formcsv.rs:165` | A CSV row missing a column is an **error**, not a best-effort skip; a value beginning `=`/`+`/`-`/`@` cannot survive the round trip |
| `FieldLockedBySignature` | cert gate | The fill-aware certification gate separately refuses any `/FieldMDP` |
| `CertificationForbidsChange` | 6722 region | The **strict** gate — authoring, deletion, rename, move, flatten |
| `DocumentEncrypted` | 6765 region | Every form verb sits behind this. ⚠️ `fill_refusal` does **not** include it |
| `ObjectCreationWouldExposeHiddenObjects` | 6355 region | The `/Size` guard on verbs that author objects |
| `AnnotsNotAnArray`, `AnnotsNotAPermutation`, `AnnotsDuplicateReference`, `TrapNetMustStayLast`, `AnnotStatesMismatch` | reorder guards | `reorder_annotations` — the tab-order verb |
| `SelectionTooLargeForOneUndo { targets, limit }` | cut guard | Selection > `MAX_UNDO_DEPTH` (256) |
| `SignApplyError::{FieldAlreadySigned, FieldNotSignature, FieldHasKids, RectRefusedForExistingField, SeedValueViolated, SeedValueUnevaluable, FieldNameTaken}` | `sign::apply` | A **separate** enum for signing into an existing `/FT /Sig` field |

**Not form-field errors despite the name:** `FormLeafOutOfRange`
(`edit.rs:6442`), `FormNotOnPage` (`:6509`), `FormNestedInAnotherForm` — these
belong to **form XObjects** (§8.10.1, CAD content), as do `unshare_form`
(`:11867`) and the ten `*_in_form` vector verbs (`move_node_in_form` `:15283`,
`move_nodes_in_form`, `move_handle_in_form`, `move_subpath_in_form`,
`move_text_run_in_form`, `move_objects_in_form`, `delete_text_run_in_form`,
`delete_subpath_in_form`, `delete_node_in_form`, `delete_objects_in_form`).
**Do not count these as form-field verbs.**

**Guards, by verb family.** Authoring / deletion / rename / `move_widget` /
`flatten_fields` take the **strict** certification gate; fills,
`set_choice_value`, `set_button_state`, `import_form_data`,
`regenerate_appearances` take the fill gate (`/P >= 2`).
⚠️ *"There are documents where filling is offered and deletion is refused. They
are not rare — a certified fillable form is the ordinary case."* Gating a
Delete control on `fill_refusal` ships a button that always errors. Correct
pattern: **preflight for the label, handle the `Err` for the truth.**

---

# 5. What landed in the form area since 2026-09-01

`git log --oneline --since=2026-09-01 -- '*form*' '*widget*' '*edit*'` → **105
commits**. The ones actually about **interactive form fields** (the rest are
form-XObject, text-edit, stamp, signing or annotation work the `*edit*` glob
swept in):

| Commit | What landed | At HEAD? |
|---|---|---|
| `20bfb259` | **"absent was a one-way door, and now it is not"** — `MkColorEdit { Set, Remove }`, `without_background()` / `without_border_color()`; empties `/MK` away; other `/MK` entries survive | **YES** — `edit.rs:21942`, `:21963` |
| `86ede66b` | **"colour before placement, and a key that was a lie"** — `chrome: WidgetChrome` on all five `New*` specs; removed the `/MK /BC [0 0 0]` that `add_text_field`/`add_choice_field` wrote while drawing no frame | **YES** — `edit.rs:1662`, `:2287`, `:2390`, `:2814`, `:3105` |
| `bd8059f2` | **"widget colours are painted, not just recorded"** — `/BG` and `/BC` baked into `/AP`; `needs_regen` gains both; `AppearanceOutcome` three-state; CMYK painted as `k`/`K`; `/BC` is also the tick/dot ink | **YES** — `edit.rs:25384`, `:22220`, `:38611` |
| `729cf6db` | **"a page's tab order, computed and disclosed"** — `page_tab_sequence`, `TabSequence`, `TabOrderBasis` (6), `TabExclusion` (4), `PageTabs`, `WidgetTabTail`, `set_widget_tab_tail`, `set_tab_row_tolerance` | **YES** — `edit.rs:29596`, `:19954`, `:19889`, `:9143`, `:9166`, `settings/mod.rs:1625` |
| `7a0a9c2e` + `766c52a5` | `validate_partial_name` made public (enforced at three sites, askable at none); **`PeriodInPartialName` renamed `EmptyNameSegment`** — a **breaking** rename | **YES** — `forms_author.rs:465` |
| `93f329b2` + `f0d1dc7b` | `adopt_widget` and `sign` refuse a dotted partial name (which had authored a field nobody could address); ten widget-adoption tests that had never executed now run | **YES** — `edit.rs:42857` |
| `2d0d1ac0` + `bc37e95a` | **"the last five field properties"** — the four advisory flags and `/TM` | **YES** — members `edit.rs:21382`, builders `:21713`–`:21907`, writes `:25776`/`:25779` |
| `220065aa` | **"/DA — a field's font, size and colour, at last settable"** — `FieldEdit::appearance`, `FieldAppearance`, `FieldFont::{Standard, Resource}`, `FieldFontNotInResources` | **YES** — `edit.rs:21287`, `:21247`, `:7756`, write `:25766` |
| `43192792` | **"three field properties that were readable and unwritable"** — `/Q`, `/DV`, `NoExport` | **YES** — `:25751`/`:25754`, `:25790`/`:25793`; `QuaddingInvalid` `:7775` |
| `dabdfa6c` | **"six check-box glyph styles, drawn as artwork rather than set as a font"** | **YES** — `annot_author.rs:3781`, `4035`–`4116`; `NewCheckBox::style` `edit.rs:2282`, `NewRadioButton::style` `:2385`. ⚠️ **still creation-only** |
| `fad0d2d3` | **"the `/MK` colours that had read and write on opposite keys"** — `/BG` was writable but `/BC` was not readable | **YES** — `Widget::border_color` now read |
| `98d0abbc` + `a4939fae` | Rotation composes and its angle is readable; the two rotation readers' **different ranges** signposted from both sides | **YES** |
| `ab40127e` + `02bb1ba0` + `187fa091` | Signing **into** a pre-placed empty `/Sig` field — `/Lock` → `/FieldMDP`, `/SV` seed values enforced in full; certifying signatures with `/DocMDP` + catalog `/Perms` | **YES** — `/Lock` read at `edit.rs:27041` |
| `caf4c1d1` + `44a24855` | Form-clipboard fixes — copy-paste solidified a dash and lost three more things; the clip shipped without a format version | **YES** — `formclip.rs:463`/`:534` |

**Nothing in that 105-commit window added:** an `/Opt` sorter, an `/AA` writer,
a `/Lock` writer, a `/CO` reorder verb, a `/Tabs` writer, a field-type
converter, a check-style change verb, a `/MK` icon (`/I` `/IF` `/TP` `/AC`
`/RC`) surface, or an `apply_recompute`.

---

# 6. The short list this project should act on

**Reachable from the engine today — a GUI can wire it straight away:** the full
`/Opt` list editor (add / remove / reorder / rename / export≠display), `/DV`,
every `/Ff` bit except the two type-defining ones and `RichText`, `/MaxLen`,
`/Q`, `/DA` (font + size + colour), `/MK` `/CA` `/BG` `/BC` `/R`, `/BS`, `/F`
(as four visibility combinations), `/T` `/TU` `/TM`, create × 5 kinds, delete
field / group / widget, copy-cut-paste including **AdditionalWidget across
pages**, adopt an orphaned widget, rotate, move, resize, reset with preview,
flatten, FDF/XFDF/CSV both ways, the six check-box glyph styles **at creation**,
`/A` push-button actions (6 subtypes), per-page computed tab order + `/Annots`
reordering, and a script census with a native recompute plan.

**Acrobat features the engine cannot reach at all** — a GUI must not offer
them, and the parity table records them as **engine work, not UI work**:

1. `/AA` authoring — format / keystroke / validate / calculate. Read and
   strippable only.
2. `/Lock` authoring (field-lock dictionaries). Read at signing only.
3. `/CO` reordering — no verb; only prune-on-delete and append-on-paste.
4. `/Tabs` writing — the "record this tab order into the page" verb the source
   itself says should exist.
5. Field-type conversion — unrepresentable by design; delete and re-place.
6. Check-box / radio **style change on an existing field** — only indirectly,
   by writing the `/MK /CA` character.
7. `/MK` icons and captions beyond `/CA`: `/AC`, `/RC`, `/I`, `/RI`, `/IX`,
   `/IF`, `/TP` — zero code in either direction.
8. Rich-text authoring (`/RV`, `/DS`, `/Ff` bit 26 on) — only a disclosed
   downgrade that deletes it.
9. Splitting a field, promoting a terminal to a grouping node, or moving a
   widget to another page in place.
10. A list box's **selection highlight** in the appearance; a signature field's
    or an untyped field's appearance at all.
11. **`/Q` does not redraw** — a justification change is invisible until
    something else forces a rebuild. A plain bug, one line at `edit.rs:25825` — **shipped in `503ad9d4`**, and no longer a gap once the pin moves.
12. Wide/batch CSV, the static-XFA half, barcode fields, and executing
    JavaScript — the four permanent non-goals.
