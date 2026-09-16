# What ISO 32000 says a form field has

The **format column** of `FORMS_PARITY.md`. Acrobat decides what a user expects;
this decides what the file may legally carry, which is the ceiling on any
parity claim and the only citation that can justify a *"this kind cannot have
that"* cell.

Sourced from `D:\Dev\Rag-Specialized\PDF_Spec\iso32000\` — ISO 32000-1:2008 is
`free_primary` and quotable; ISO 32000-2:2020 material there is
`licensed_primary_private_rag`, cited in this repo but never published in a
user-facing artifact. The empirical layer is `C:\personal_rag\pdf\`.

**Table numbers corrected at the source.** The field-flag tables in
ISO 32000-1 are **221 (common), 226 (button), 228 (text), 230 (choice)**.
Table 227 is the check-box/radio `/Opt` array, *not* a flag table. Table 231 is
the choice field's `/Opt` `/TI` `/I`, *not* a flag table. Signature fields have
**no flag table of their own** — only Table 221's three bits.

---

## A. Common to every kind

### A.1 The field-dictionary half — §12.7.3.1, Table 220

| Entry | Type | Requirement | Clause | Note |
|---|---|---|---|---|
| `FT` | name | Required for terminal fields; **inheritable** | §12.7.3.1 T220 | `Btn`/`Tx`/`Ch`/`Sig`. Must be resolved up `/Parent` **before** `/Ff` can be decoded — see C. |
| `Parent` | dict | Required if the field has a parent | §12.7.3.1 T220 | The inheritance channel. |
| `Kids` | array | Optional | §12.7.3.1 T220 | "In a **terminal field**, `Kids` ordinarily shall refer to one or more separate widget annotations. **However, if there is only one associated widget annotation, and its contents have been merged into the field dictionary, `Kids` shall be omitted.**" |
| `T` | text string | **Required** | §12.7.3.1 T220, §12.7.3.2 | The *partial* name. "a partial name **shall not contain a PERIOD character**". |
| `TU` | text string | Optional; 1.3 | §12.7.3.1 T220 | The tooltip. "used in place of the actual field name wherever the field shall be identified in the **user interface** … also useful when extracting the document's contents in support of **accessibility**". |
| `TM` | text string | Optional; 1.3 | §12.7.3.1 T220 | Mapping name — "used when **exporting** interactive form field data". |
| `Ff` | integer | Optional; **inheritable** | §12.7.3.1 T220 | Default 0. |
| `V` | varies by `FT` | Optional; **inheritable** | §12.7.3.1 T220 | Type per kind — see B. |
| `DV` | varies by `FT` | Optional; **inheritable** | §12.7.3.1 T220 | "the default value to which the field reverts when a **reset-form action** is executed". |
| `AA` | dict | Optional; 1.2 | §12.7.3.1 T220 → §12.6.3 T196 | See D. |
| `DA` | string | **Required; inheritable** for variable text | §12.7.3.3 T222 | See A.4. |
| `Q` | integer | Optional; **inheritable** | §12.7.3.3 T222 | 0 left, 1 centre, 2 right. Default 0. |
| `DS` | text string | Optional; 1.5 | §12.7.3.3 T222 | Default *style* string. Collides with catalog `/AA /DS` "did save" (T197). |
| `RV` | text string **or stream** | Optional; 1.5 | §12.7.3.3 T222 | Rich-text value. See B.1. |

**The inheritable set, exactly:** `FT`, `V`, `DV`, `Ff`, `DA`, `Q` — plus
`/MaxLen`, which Table 229 marks inheritable on its own. Nothing else is.

**Same-FQN rule (§12.7.3.2, verbatim):** "field dictionaries with the same
fully qualified field name **shall have the same field type (`FT`), value
(`V`), and default value (`DV`)**."

**The merge (§12.7.1, verbatim):** "the **contents of the field dictionary and
the annotation dictionary may be merged into a single dictionary** … **If such
an object defines an appearance stream, the appearance shall be consistent with
the object's current value as a field.**" Empirically **~88% of terminal fields
are the merged shape** — `C:\personal_rag\pdf\lesson_20260801_field_widget_merge_shape_a_vs_b.md`.

**`/Kids` is heterogeneous per entry, not per node.** A kid *with* `/T` is a
child field; a kid *without* `/T` is one of this node's own widgets; §12.7.3.1
never says a node must be only one kind —
`C:\personal_rag\pdf\lesson_20260807_mixed_kids_node_is_both_parent_and_widget_carrier.md`.

**An FQN is not safely splittable on `.`** — nothing forbids a literal period
inside a `/T` —
`C:\personal_rag\pdf\lesson_20260810_field_t_may_contain_a_period_so_fqn_is_not_safely_splittable.md`.

### A.2 The widget-annotation half — §12.5.6.19, Table 188

Table 188's own key set is exactly `Subtype` `H` `MK` `A` `AA` `Parent`.

| Entry | Type | Requirement | Note |
|---|---|---|---|
| `Subtype` | name | Required | `Widget`. 87.8% of all annotations in the measured corpus. |
| `H` | name | Optional | Highlighting mode: `N` None, `I` Invert, `O` Outline, `P` Push, "**`T` (Toggle) Same as `P` (which is preferred)**". **Default `I`.** ⚠ A Hide action's `/H` is a *boolean* (default `true`) one indirection away on the same button — §12.6.4.10 T210. |
| `MK` | dict | Optional | See A.3. |
| `A` | dict | Optional; 1.1 | An action. **`/A` is not in Table 164** — only Link, Screen and Widget carry it. **The field dictionary has no `/A` in either edition**, so on a merged dict `/A` is unambiguously the annotation half. |
| `AA` | dict | Optional; 1.2 | Same key as the field's `/AA` — see D. |
| `Parent` | dict | Required if this widget is one of several in `/Kids` | |
| `BS` | dict | Optional; 1.2 | See A.5. **Erratum:** Table 188's "`AP` … shall take precedence over the **`L` and `BS`** entries" is a copy-paste from Table 175 (line annotation); a widget has no `/L`. Deleted in 2.0's Table 191. |

### A.3 `/MK` appearance characteristics — §12.5.6.19, Table 189

| Key | Applies to | Substance |
|---|---|---|
| `R` | **all kinds — the only `/MK` key with no type restriction** | "The number of degrees by which the widget annotation shall be **rotated counterclockwise** relative to the page. The value shall be **a multiple of 90**. Default value: 0." Character-identical in 2.0's T192. |
| `BC` | all | Border colour. **Array length selects the space:** 0 = transparent, 1 = DeviceGray, 3 = DeviceRGB, 4 = DeviceCMYK. |
| `BG` | all | Background colour, same length rule. |
| `CA` | **buttons of any type** | Normal caption. |
| `RC` | **pushbutton only** | Rollover caption — "displayed when the user rolls the cursor into its active area without pressing the mouse button". **Plain text string, not rich text.** |
| `AC` | **pushbutton only** | Alternate (down) caption. |
| `I` `RI` `IX` | **pushbutton only** | Normal / rollover / alternate icon. **Shall be indirect references to form XObjects.** |
| `IF` | **pushbutton only** | Icon-fit dictionary (1.7 T247 / 2.0 T250). "shall apply to **all** of the annotation's icons". |
| `TP` | **pushbutton only** | Caption/icon relation: 0 caption only, 1 icon only, 2 caption below icon, 3 above, 4 right, 5 left, 6 overlaid. Default 0. |

`/MK` carries no action, and occurs only four times per edition.

**The render-time trap, PDF-Association erratum #56 (`ISO approved`, against
2.0):** "When rendering the appearance dictionary, a PDF reader shall ignore
the values of the `C`, `IC`, `Border`, `BS`, `BE`, `CA`, `ca`, `H`, `DA`, `Q`,
`DS`, `LE`, `LL`, `LLE`, **`MK`**, and `Sy` keys." TWG minute: "OK to ignore
`MK` for Widget." So **`/MK` is generator input, never render-time input** —
and the standard then never says how a generator should use it (see E.5).

### A.4 `/DA` — §12.7.3.3, Table 222, verbatim grammar

> "Only operators that are allowed within text objects shall occur in this
> string. At a minimum, the string shall include a `Tf` … along with its two
> operands, `font` and `size`. The specified `font` value shall match a
> resource name in the `Font` entry of the **default resource dictionary**
> (referenced from the `DR` entry …). A **zero value for `size`** means that
> the font shall be **auto-sized**: its size shall be computed as a function of
> the height of the annotation rectangle."

At most one `Tm` may appear. `/DR` must carry a `/Font` at minimum (§12.7.2
T218).

### A.5 `/BS` border style — §12.5.4, Table 166

| Key | Value |
|---|---|
| `Type` | Optional; if present, `Border`. |
| `W` | Width in points. **0 ⇒ no border drawn. Default 1.** |
| `S` | `S` Solid, `D` Dashed, `B` Beveled, `I` Inset, `U` Underline. "A conforming reader shall **tolerate other border styles that it does not recognize and shall use the default value**." Default `S`. |
| `D` | Dash array. **Dash phase is not specified and shall be assumed 0.** Default `[3]`. |

Precedence (§12.5.4): `/AP` overrides everything; absent `/AP`, `/BS` overrides
Table 164's `/Border` ("If an annotation dictionary includes the `BS` entry,
then the `Border` entry is ignored"); with neither, "a **solid line with a
width of 1 point**"; and the border "shall be drawn **completely inside** the
annotation rectangle."

**Empirical, and it decides how a properties panel must be built:** the border
of a **button** is vector artwork *inside* `/AP`; the border of a **text or
choice** field is `/MK /BC` + `/BS`, *outside* `/AP`. An `/AP`-only renderer
therefore draws borders on some kinds and not others in the same document —
`C:\personal_rag\pdf\lesson_20260807_widget_border_lives_in_ap_for_buttons_mk_for_variable_text.md`.

### A.6 `/F` annotation flags — §12.5.3, Table 165

| Bit | Value | Name | Meaning |
|---|---|---|---|
| 1 | 1 | Invisible | Do not display/print an annotation of an **unknown** type having no `/AP`. |
| 2 | 2 | Hidden | Do not display or print; no interaction. |
| 3 | 4 | Print | Print with the page; if clear, **never print**. |
| 4 | 8 | NoZoom | Do not scale with the page. |
| 5 | 16 | NoRotate | Do not rotate with the page. |
| 6 | 32 | NoView | Not on screen, but may print. |
| 7 | 64 | ReadOnly | "**This flag shall be ignored for widget annotations**" — `/Ff` bit 1 is the operative one. |
| 8 | 128 | Locked | Do not delete or change properties; contents still editable. |
| 9 | 256 | ToggleNoView | |
| 10 | **512** | LockedContents | Contents shall not be modified. |

⚠ The Value column is **derived** (2^(bit−1)) — **Table 165 prints no
integers.** LockedContents is **512**.

### A.7 `/Rect` `/P` `/AP` `/AS` and the rest of Table 164 — §12.5.2

| Entry | Requirement | Note |
|---|---|---|
| `Rect` | **Required** | "the location of the annotation on the page in **default user space units**." Corners in any order ⇒ normalise per §7.9.5. The target box for the §12.5.5 algorithm. |
| `P` | Optional; 1.3 | Back-pointer to the page. **Often absent in practice — enumerate widgets by walking the page's `/Annots`, never by matching `/P`** — `C:\personal_rag\pdf\lesson_20260814_widget_p_key_optional_and_absent_in_practice.md`. |
| `AP` | Optional; 1.2 | "**Individual annotation handlers may ignore this entry and provide their own appearances.**" ★ An explicit `may` to substitute artwork *even when `/AP` is present*. **1 hit in 1.7, 0 in 2.0** — 2.0 deleted it and reversed the row, making an appearance dictionary effectively required of the **writer**, widgets included, with two exceptions (a degenerate `/Rect`; `/Subtype` ∈ {`Popup`,`Projection`,`Link`}). Both 2.0 sentences bind the writer and the file — neither obliges a reader to refuse or to generate. |
| `AS` | **Required if `AP` contains one or more subdictionaries** | |
| `Border` | Optional | `[hcr vcr width]` + optional dash. Default `[0 0 1]`. Ignored if `/BS` present. |
| `C` | Optional; 1.1 | **Not a general fill colour** — closed-icon background / pop-up title bar / link border only. |
| `NM` | Optional; 1.4 | Unique among annotations on its page. |
| `M` | Optional; 1.1 | "Conforming readers **shall accept and display a string in any format**." |
| `F` | Optional; 1.1 | Default 0. See A.6. |
| `StructParent` | Required if a structural content item | |
| `OC` | Optional; 1.5 | "**Before the annotation is drawn, its visibility shall be determined based on this entry as well as the annotation flags** … If it is determined to be invisible, the annotation shall be skipped, **as if it were not in the document**." ⇒ the visibility gate order is **`/OC` first (hard skip), then `/F`**. |
| `Contents` | Optional | For a widget it is purely an **accessibility alternate** (§12.5.6.2), not displayed text. |

**One-page rule (§12.5.2):** "A given annotation dictionary shall be referenced
from the `Annots` array of **only one page**. This requirement applies only to
the annotation dictionary itself, **not to subsidiary objects, which may be
shared among multiple annotations**."

**`/Tabs`** is the *page* object's key, not the field's: §12.5.1 + §7.7.3.3
T30, values `R` row, `C` column, `S` structure order. ISO 32000-2 adds `A`
(annotations-array order) and `W` (widget order). **Neither edition states a
fallback when `/Tabs` is absent.** Full treatment:
`D:\Dev\Rag-Specialized\PDF_Spec\iso32000\iso32000__ref__annots_array_order.md`.

### A.8 The AcroForm dictionary — §12.7.2, Table 218

`Fields` (**Required**); `NeedAppearances` (Optional, default **false**);
`SigFlags` (Optional; T219: bit 1 SignaturesExist, bit 2 AppendOnly);
**`CO`** ("**Required** if any fields in the document have additional-actions
dictionaries containing a `C` entry; otherwise optional"); `DR` ("At a minimum,
this dictionary shall contain a `Font` entry"); `DA`; `Q`; `XFA`.

**Consequence:** copying a field carrying `/AA /C` between documents
**obliges** appending it to `/CO` in the same operation, or the destination is
non-conformant —
`C:\personal_rag\pdf\lesson_20260829_acroform_co_required_when_field_carries_aa_c.md`.

---

## B. Per kind

### B.1 Text — `/FT /Tx`

| Aspect | Content |
|---|---|
| Flags | bits **13** Multiline, **14** Password, **21** FileSelect, **23** DoNotSpellCheck, **24** DoNotScroll, **25** Comb, **26** RichText — §12.7.4.3 T228 |
| `MaxLen` | integer, Optional, **inheritable**. Maximum length **in characters**; also the **comb cell count** when bit 25 is set — T229 |
| `V` | "a **text string** … or a **text stream**" (a stream permits > 65535 bytes) |
| `RV` | rich-text value; text string or text stream — T222 |
| Appearance | **GENERATED**: "the conforming reader **shall construct an appearance stream dynamically at viewing time**" — §12.7.3.3 |
| Comb gate | bit 25 "**may be set only if the `MaxLen` entry is present** … **and if the `Multiline`, `Password`, and `FileSelect` flags are clear**" |
| Password NOTE | "To protect password confidentiality, readers **should never store the value** of the text field in the PDF file if this flag is set." |

**The rich-text axis, which most implementations get wrong.** RichText set *and*
a value present ⇒ `/RV` is a **`shall`**. `/RV` present ⇒ RichText set is only a
**`should`**, and a flat-text `/V` alongside is also only a **`should`**.
**Appearance generation is bound to `/RV`, not `/V`**, when RichText is on —
hence the stale-`/RV` hazard. RichText is **not** excluded by Comb; RichText +
Password is permitted and nonsensical. §12.7.3.3–§12.7.3.4, T222–225.
**Erratum:** §12.7.3.3's RichText sentence cites "Table 226"; the correct table
is **228**.

**Auto-size has no spec formula.** "auto-siz\*" occurs **once in 756 pages**.
ISO 32000-2 changes it to "an **implementation dependent** function". Every
consumer picks its own —
`C:\personal_rag\pdf\lesson_20260801_variable_text_autosize_is_implementation_defined.md`.
Never present a chosen number as spec-mandated.

### B.2 Check box and radio — `/FT /Btn`, Pushbutton clear

| Aspect | Content |
|---|---|
| Flags | **15** NoToggleToOff (radio only), **16** Radio, **17** Pushbutton, **26** RadiosInUnison — §12.7.4.2 T226 |
| Classification | 17 clear + 16 clear ⇒ check box; 17 clear + 16 set ⇒ radio; 17 set ⇒ pushbutton; **both set ⇒ invalid** — T226's Radio row: "may be set **only if** the `Pushbutton` flag is clear" |
| `Opt` | array of text strings; Optional, 1.4 — "one entry for each widget annotation in the `Kids` array … a text string representing the **on state**" — T227 |
| `V` | "a **name object** representing the check box's appearance state, which shall be used to **select the appropriate appearance from the appearance dictionary**" — §12.7.4.2.3 |
| `/AS` | Must name a state present in `/AP /N`. Off state "shall be stored … under the name **`Off`**. **`Yes` should be used** as the name for the on state." |
| Radio `/V` | Held on the **parent**: "a name object corresponding to the appearance state of whichever child field is currently in the on state; the **default value for this entry is `Off`**" — §12.7.4.2.4 |
| RadiosInUnison | "the same behavior shall occur only if the `RadiosInUnison` flag is set. **If it is not set, at most one radio button in a field shall be set at a time**" |
| Appearance | **SELECTED, not generated** — `/AP /N` is a sub-dictionary keyed by state name, one stream per state; `/AS` picks — §12.7.4.2.3 + §12.5.5 |

⚠ Table 227's sentence mapping `/Opt` entries to positional `/AP` names (`/0`
`/1` `/2`) is flagged **NEEDS VERIFICATION** in the RAG — the source extract
was garbled. Do not build on it without re-reading the printed table.

### B.3 Push button — `/FT /Btn` + bit 17

- **No value at all:** "Because this type of button retains no permanent value,
  **it shall not use the `V` and `DV` entries.**" §12.7.4.2.2
- `/MK` keys exclusive to it: `RC` `AC` `I` `RI` `IX` `IF` `TP`, plus `CA`
  shared with other buttons.
- Behaviour rides on `/A` on the widget (T188) — typically JavaScript,
  SubmitForm, ResetForm or Hide.

### B.4 Choice — `/FT /Ch` (combo box and list box)

| Aspect | Content |
|---|---|
| Flags | **18** Combo, **19** Edit, **20** Sort, **22** MultiSelect, **23** DoNotSpellCheck, **27** CommitOnSelChange — §12.7.4.4 T230 |
| Combo vs list | bit 18 set ⇒ combo box; clear ⇒ list box |
| Edit gate | "**shall be used only if** the `Combo` flag is set" |
| DoNotSpellCheck gate | on `Ch` only: "**shall not be used unless the `Combo` and `Edit` flags are both set**" — a restriction the identical bit in T228 does **not** carry |
| Sort | "This flag is **intended for use by writers, not by readers**. **Conforming readers shall display the options in the order in which they occur in the `Opt` array**" |
| `Opt` | array; each element is "**either a text string** representing one of the available options **or an array of two text strings**: the option's **export value** and the **text that shall be displayed**" — T231 |
| `TI` | integer, Optional. Index in `/Opt` of the first visible option. **Default 0.** |
| `I` | array of integers, Optional, 1.4. Selected indices, ascending. "**If the items identified by `I` differ from those in `V`, the `V` entry shall be used.**" |
| `V` | text string, or — with MultiSelect — an **array** of text strings. Default **null**. |
| Appearance | **GENERATED**, same regime as text — §12.7.3.3 |

### B.5 Signature — `/FT /Sig`

| Aspect | Content |
|---|---|
| Flags | **only Table 221's three.** There is no signature flag table. |
| `V` | a **signature dictionary** (T252) — §12.7.4.5 T232 |
| `Lock` | dict, Optional, 1.5 — fields locked on signing — T232 → T233 |
| `Lock` contents | `/Action` ∈ {`All`, `Include`, `Exclude`} + `/Fields` (array of partial names; **required unless `/Action` is `All`**) |
| `SV` | seed value dictionary, Optional, 1.5 — T232 → T234 (+ T235 for `/Cert`) |
| Invisibility | "Signature fields that are **not intended to be visible shall have an annotation rectangle that has zero height and width** … Conforming readers shall **also treat signatures as not visible if either the `Hidden` bit or the `NoView` bit of the `F` entry is true**." |

⚠ **`/SV` has its own separate `/Ff`** (T234), `/Cert` has a third (T235), and
FDF has a fourth (T246) — **four unrelated `/Ff` vocabularies**, resolved by
containing dictionary (`iso32000__ref__field_flags.md` §6). In FDF, `/Ff`
*replaces*; `/SetFf` then `/ClrFf` apply in that order.

---

## C. The complete `/Ff` bit table

**Bit numbering, verbatim (§12.7.3.1):** "The value of the field dictionary's
`Ff` entry is an **unsigned 32-bit integer** … **Bit positions within the flag
word shall be numbered from 1 (low-order) to 32 (high-order).** … **All
undefined flag bits shall be reserved and shall be set to 0.**" ⇒ value =
2^(bit−1), **u32, not i32**.

| Bit | Value | Name | Kinds | Meaning | Table |
|---|---|---|---|---|---|
| 1 | 1 | ReadOnly | all | No modification, no interaction. | 221 |
| 2 | 2 | Required | all | Shall have a value when exported by a submit-form action. | 221 |
| 3 | 4 | NoExport | all | Shall not be exported by a submit-form action. | 221 |
| 13 | 4096 | Multiline | `Tx` | Multi-line; else single line. | 228 |
| 14 | 8192 | Password | `Tx` | Echo obscured; readers **should never store** the value. | 228 |
| 15 | 16384 | NoToggleToOff | `Btn` radio | Exactly one button selected at all times. | 226 |
| 16 | 32768 | Radio | `Btn` | "may be set only if the `Pushbutton` flag is clear". | 226 |
| 17 | 65536 | Pushbutton | `Btn` | No permanent value. | 226 |
| 18 | 131072 | Combo | `Ch` | Combo box; clear ⇒ list box. | 230 |
| 19 | 262144 | Edit | `Ch` | Editable text box accompanies the combo. Only if Combo set. | 230 |
| 20 | 524288 | Sort | `Ch` | For **writers, not readers**; readers **shall** use `/Opt` order. | 230 |
| 21 | 1048576 | FileSelect | `Tx` (1.4) | The text is a file pathname. | 228 |
| 22 | 2097152 | MultiSelect | `Ch` (1.4) | More than one item may be selected. | 230 |
| 23 | 4194304 | DoNotSpellCheck | `Tx` **and** `Ch` (1.4) | On `Ch` only: requires Combo+Edit. | 228 **and** 230 |
| 24 | 8388608 | DoNotScroll | `Tx` (1.4) | Input refused once the box is full. | 228 |
| 25 | 16777216 | Comb | `Tx` (1.5) | Requires `/MaxLen`; Multiline/Password/FileSelect clear. | 228 |
| **26** | 33554432 | **RadiosInUnison** on `Btn` ⟷ **RichText** on `Tx` | **COLLISION** | Same-name radios toggle in unison / rich text per `/RV`. | 226 **and** 228 |
| 27 | 67108864 | CommitOnSelChange | `Ch` (1.5) | Commit on selection change, not on exit. | 230 |

**Bit 26 is the only overloaded position, and it is why any flag decoder must
take a resolved `/FT`.** Bit 23 is the only other shared position, and its two
rows differ in restriction text.

**Validity gates the spec states:** Radio only if Pushbutton clear (T226); Comb
only with `/MaxLen` and Multiline/Password/FileSelect clear (T228); Edit only if
Combo set (T230); DoNotSpellCheck on `Ch` requires Combo+Edit (T230).

**Evidenced negatives** (`iso32000__ref__field_flags.md`): the spec prescribes
**no reader recovery** for a gate violation; `/Ff` **cannot** be used to infer
`/FT`; Pushbutton+Radio both set is **unresolved**; the scope of "undefined"
bits is unclear; and the spec **never states which flags a reader must
enforce** — `Required` binds at submit time only.

---

## D. Additional actions `/AA`

**§12.6.3, verbatim:** "An annotation, page object, or (beginning with PDF 1.3)
interactive form field may include an entry named **`AA`** … **Tables 194 to
197** show the contents of this type of dictionary."

Four carriers, four tables: annotation → **194** (`E X D U Fo Bl PO PC PV PI`);
page → **195** (`O C`); **form field → 196** (`K F V C`, 1.3); catalog → **197**
(`WC WS DS WP DP`).

**★ On a merged field+widget dictionary, one `/AA` legitimately holds the union
of both key sets** — the sets are disjoint, which is what §12.5.6.19's merge
NOTE ("the contents of the two kinds of dictionaries do not conflict") means. A
parser that assumes `/AA` is one table or the other silently drops half the
actions. `/AA` holding `/U` and `/C` at once is conforming.

### D.1 Field `/AA` — Table 196, all four Optional, PDF 1.3, typed `dictionary`

| Key | Verbatim | JavaScript-only? |
|---|---|---|
| `K` | "A **JavaScript action** that shall be performed when the user **modifies a character in a text field or combo box or modifies the selection in a scrollable list box**. This action **may check the added text for validity and reject or modify it**." | **Yes — typed as JavaScript by name.** |
| `F` | "A **JavaScript action** that shall be performed **before the field is formatted to display its value**. This action **may modify the field's value before formatting**." | **Yes.** |
| `V` | "A **JavaScript action** that shall be performed **when the field's value is changed**. This action **may check the new value for validity**." | **Yes.** |
| `C` | "A **JavaScript action** that shall be performed **to recalculate the value of this field when that of another field changes**. … **The order in which the document's fields are recalculated shall be defined by the `CO` entry in the interactive form dictionary.**" | **Yes.** |

`/E /X /D /U /Fo /Bl /PO /PC /PV /PI` are, by contrast, **generic** action
dictionaries — any of Table 198's eighteen types may sit there.

**The modality ladder inside Table 196, which must not be flattened:**
"shall be performed" on the named event is the **hollow `shall`**; `K` "**may**
check … and reject or modify it", `F` "**may** modify the field's value before
formatting" and `V` "**may** check the new value for validity" are permissions;
and `C`'s recalculation order "**shall** be defined by the `CO` entry" is **the
only hard, non-hollow `shall` in Table 196** — it constrains document structure
and processor sequencing and needs no JavaScript semantics to mean something.

**Why the execute-`shall` is hollow (§12.6.4.16):** the clause says a
conforming processor "shall execute a script", but ISO 32000-1 supplies no
JavaScript semantics, object model, `event`/`this`/`app`, field-access API or
security model — it delegates to a 1999 Mozilla language reference and a vendor
API reference. ★ **One leg of the usual argument is false and is retracted in
the RAG:** both JavaScript authorities are in **clause 3, Normative
references** — not the Bibliography — despite §12.6.4.16 itself saying "(see
the Bibliography)", a **systematic ISO 32000-1 erratum across ≥8 citation
sites**. The surviving argument is narrower: the invocation verb is descriptive
("give details on") where the same standard writes "shall conform to" for other
clause-3 Adobe documents. **In ISO 32000-2 the premise changed** —
§12.6.4.17 "ECMAScript actions", with **ISO 21757-1:2020** supplying an ISO
ECMAScript API. Do not repeat "no ISO document defines the JavaScript API" as
present tense.

**What §12.6.3 does not say** (`iso32000__s__12.6.3.md` §5):
- It **never orders `K` vs `F` vs `V` vs `C`** relative to one another, and the
  silence is **deliberate** — it orders triggers explicitly three times for the
  *annotation* family (T194 `PO` "shall be executed **after** the `O` action …
  and the `OpenAction` entry"; T194 `PC` "**before** the `C` action"; T195 `C`
  "**before any other page is opened**").
- `recalculat*` = **3 hits in 756 pages** (two in T196's `C` row, one in T218's
  `CO` row); `calculation order` = **1 hit**. The entire normative treatment of
  form calculation is those two table rows — no clause, no algorithm, no annex.
- **No normative text obliges a recompute to produce a particular value**, so
  there is no ISO-defined "correct result" to measure a processor against.
- A "format helper never writes `/V`" guarantee **cannot be anchored in the
  spec** — T196's `F` row grants the opposite permission, and NOTE 2 says the
  effects of a K/F/V/C action "are limited only by the action itself and **can
  occur outside the described scope of the event**", with an `F` event
  performing a calculation as the worked example.

### D.2 Annotation `/AA` — Table 194

Keys and versions: `E` Enter, `X` Exit, `D` Down, `U` Up, `Fo` Focus, `Bl` Blur
(all 1.2); `PO` page opened, `PC` page closed, `PV` page visible, `PI` page
invisible (all 1.5).

⚠ **The local RAG does not carry Table 194's per-key verbatim row text.** The
key list, versions, ordering sentences, mouse-event constraints, NOTE 1 and the
`/U` precedence sentence are sourced; the individual row prose for `E X D U Fo
Bl PO PC PV PI` is **not ingested** (measured: zero hits). Re-read the printed
Table 194 before quoting a trigger description as spec text.

**What *is* sourced verbatim:**
- **`/A` beats `/AA /U`:** the `U` row — "For backward compatibility, the **`A`**
  entry in an annotation dictionary, if present, **takes precedence over this
  entry**." A scanner walking only `/AA` is blind to the entry the standard says
  wins. (Its cross-reference "see Table 168" is an erratum — 168 is the
  appearance dictionary.)
- **Mouse-event constraints:** an `E` "may occur only when the mouse button is
  up"; an `X` "may not occur without a preceding `E`"; a `U` "may not occur
  without a preceding `E` and `D`"; and "in the case of overlapping or nested
  annotations, entering a second annotation's active area causes an `X` event to
  occur for the first annotation".
- **NOTE 1:** for `PO`/`PC`/`PV`/`PI`, "the values of the flags specified by the
  annotation's `F` entry … **have no bearing** on whether a given trigger event
  occurs" ⇒ **a hidden annotation still fires page-level triggers.**
- **An unusual `shall` on the reader's environment:** "Conforming readers
  **shall ensure the presence of such a device, or equivalent controls for
  simulating one**, for the corresponding actions to be executed correctly."
- Page `/AA` is **not inheritable** — do not walk the page tree for it.

**Key collisions, resolved only by containing dictionary:** `/C` = page close
(T195) vs calculate (T196) vs colour (T170) vs caption; `/V` = field value
(T220) vs validate action (T196); `/DS` = default style string (T222) vs "did
save" (T197); `/DV` = field default value vs 3D default view; `/CO` = calc order
(T218) vs line-caption offset (T175) vs 3D orbit centre; `/H` = widget
highlighting name (T188) vs Hide-action boolean (T210); `/T` = partial field
name vs the `/H` value `T` vs a Hide-action target.

---

## E. What the spec requires when a property changes

### E.1 Two appearance regimes — the organizing fact

| Regime | Kinds | Rule |
|---|---|---|
| **SELECT** a pre-authored state | `/Btn` check box, radio | `/AP /N` is a sub-dictionary keyed by state name; `/AS` names the state; `/Off` always present. Changing `/V` means changing `/AS` to the matching name. §12.7.4.2.3; §12.5.5 T168 |
| **GENERATE** at view time | `/Tx`, `/Ch` | "the PDF document cannot provide a statically defined appearance stream … Instead, **the conforming reader shall construct an appearance stream dynamically at viewing time.**" (2.0: "the **PDF processor** shall construct an appearance stream dynamically at **rendering** time" — sentence changed, rule intact.) §12.7.3.3 / 2.0 §12.7.4.3 |

### E.2 When regeneration is mandatory

**The finding that matters:** the sentence "**the entire annotation appearance
shall be regenerated each time the value is changed**" applies **only to
RichText fields**. For ordinary fields the mandate is only the
construct-at-viewing-time `shall` above, and the `/Tx BMC` … `EMC` splice
procedure is a `shall` about **how**, not **when**.

**The splice rule, verbatim (§12.7.3.3):** an update "shall **replace the
existing contents of the appearance stream from `/Tx BMC` to the matching
`EMC`**". The generated body is wrapped `/Tx BMC … EMC`, and **`BBox` =
`[0 0 Rect_w Rect_h]`**.

**`/DR` vs `/Resources` collision, verbatim:** when a resource name appears in
both, "the one already in the `Resources` dictionary **shall be left intact, not
replaced**."

### E.3 `/NeedAppearances`

**Table 218, verbatim (1.7):** "(Optional) A flag specifying whether to
construct appearance streams and appearance dictionaries for **all widget
annotations in the document** … **Default value: false.**"

- **Honouring it is effectively reader-discretionary, not a hard `shall`** —
  the row's sentence is descriptive; no imperative attaches anywhere.
- **The cross-reference to §12.7.3.3 is dangling** — `NeedAppearances` occurs
  **exactly twice in 756 pages**, and neither occurrence is in §12.7.3.3.
- **Widget-only.** No bearing on non-widget annotations in either edition.
- **ISO 32000-2, Table 226:** **deprecated in PDF 2.0**, plus a *writer*
  obligation — a writer `shall` include it with value `true` if it has not
  provided appearance streams for all **visible widget** annotations — and a
  NOTE that "appearance streams are required in PDF 2.0 and later".

### E.4 `/Rect` → `/BBox` — the §12.5.5 placement algorithm, normative

Appearance dictionary (**T168**): `/N` **Required** (stream *or* dictionary);
`/R` Optional (**default `/N`**); `/D` Optional (**default `/N`**). `/R` =
rollover (cursor over the active area, no button); `/D` = down. `/AS` is
**Required if `/AP` contains one or more subdictionaries** (T164).

1. Map the appearance's `/BBox` corners through its `/Matrix`; take the
   **smallest upright rectangle enclosing the transformed box**.
2. Compute a matrix **A** mapping that upright box to `/Rect`, **fitting it
   entirely** within the annotation rectangle.
3. **AA = Matrix × A** — the concatenation is what maps appearance space to
   default user space.

**Consequences the spec makes normative:** the fit is **anisotropic** — an
aspect-ratio-breaking stretch is what the standard prescribes, so an appearance
in a re-shaped `/Rect` legitimately distorts. Two **negative results**: nothing
is prescribed for a **degenerate** transformed box (zero width or height ⇒ A is
not computable), and nothing for a **missing `/AS`** when `/N` is a
sub-dictionary. For an `/AS` naming an undefined state, **NOTE 3** says a reader
"shall also attempt to provide reasonable behavior (such as **displaying
nothing**)".

An appearance stream's `/Resources` shall **not** be promoted into the outer
content stream's resource scope.

### E.5 Rotation — `/MK /R` counterclockwise vs page `/Rotate` clockwise

- **`/MK /R`** (T189, verbatim): "rotated **counterclockwise** relative to the
  page … shall be a **multiple of 90**. Default 0."
- **Page `/Rotate`** (§7.7.3.3 T30): degrees **clockwise**.
- **They compose.** §12.5.3's `NoRotate` clear (the default) means the
  annotation rotates with the page — and "it **shall not actually change the
  annotation's `Rect` entry**, which continues to describe the annotation's
  relationship with the unscaled, unrotated user space."

**And then the spec falls silent, measurably:**
- Clause 12.7 contains **0** occurrences of "rotat" and **0** of `MK`, in **both
  editions**.
- The standard's only appearance-construction recipe (§12.7.3.3) **never
  mentions `/R` or `/MK`** and specifies an **upright** `BBox`.
- No range or normalisation rule for `/R` — is −90 legal? 450?
- The comb axis, `/Q` left/right, Multiline line advance, DoNotScroll axes and
  1.7's auto-size-from-*height* are **all silent under rotation** ⇒ a rotated
  variable-text field has **no spec-defined layout**.
- At render time `/MK` is on erratum #56's **ignore list** — so the only
  sanctioned consumer of `/MK /R` is an appearance *generator*, which the
  standard then never tells how to use it.

### E.6 Other requirements, and conspicuous silences

- **§12.7.1:** if a merged field+widget object "defines an appearance stream,
  the appearance **shall be consistent with the object's current value as a
  field**." The only global consistency `shall`, and a property of the *file*,
  not a regeneration trigger.
- **§12.7.3.2:** same-FQN fields **shall** agree on `FT`, `V`, `DV`.
- **T196 `C` + T218 `CO`:** `/CO` is **Required** once any field carries
  `/AA /C`, and recalculation order **shall** follow it.
- **"Flatten" is not an ISO 32000-1 operation.** The word names no spec
  procedure; the only normative content nearby is the §12.5.5 placement maths.
  Empirically, overlay-append beats content-stream surgery —
  `C:\personal_rag\pdf\lesson_20260801_flatten_overlay_append_beats_content_stream_surgery.md`.
- **§12.5.2's `/AP` row:** "**Individual annotation handlers may ignore this
  entry and provide their own appearances**" — 1.7 only; 2.0 deleted it. pdfium
  does exactly this, **synthesizes** appearances for annotations with no
  `/AP /N`, and paints widget appearances **only** via `FPDF_FFLDraw`, never in
  a plain `FPDF_RenderPageBitmap` —
  `C:\personal_rag\pdf\lesson_20260801_pdfium_widget_appearances_need_fpdf_ffldraw.md`.
- **"Has an appearance" is a definitional choice, not a fact** — raw `/AP`-key
  presence over-counts versus a *usable* `/AP /N`: 228 files vs 224 on the same
  2,914-file corpus —
  `C:\personal_rag\pdf\lesson_20260801_usable_ap_n_vs_raw_ap_key_census_predicate.md`.

---

## Gaps in the local spec RAG — do not quote these as sourced

1. **Table 194's per-key verbatim row text** for `E X D U Fo Bl PO PC PV PI` is
   not ingested.
2. **Table 227's mapping of `/Opt` entries to positional `/AP` state names**
   (`/0` `/1` `/2`) is flagged NEEDS VERIFICATION — garbled extract.
3. **Tables 234 (`/SV`) and 235 (`/Cert`) per-key detail** — only the fact that
   they carry their own `/Ff` vocabularies is sourced.
4. **Tables 223–225** (rich-text `/RV` internals) — the RichText *modality* axis
   in B.1 is sourced; the `/RV` markup grammar is not.
5. **ISO 32000-2 clause bodies** are largely unverifiable in this corpus — the
   2.0 material above is table rows and errata text only.
6. **Auto-size, default insets, line spacing and quadding tie-breaks have no
   spec formula.** Any number this shell picks is an engineering choice and must
   not be presented as spec-mandated.

## Errata to carry into any implementation

| Site | Defect |
|---|---|
| T220 `/AA` row | "same meaning as the `AA` entry in **an annotation dictionary (see 12.5.2)**" — §12.5.2 has no `AA`. Fixed in 2.0 by PDF-Association issue #313 → "in a **Widget annotation** dictionary (see 12.5.6.19)". |
| T218 `/NeedAppearances` row | Cross-reference to §12.7.3.3 is dangling. |
| §12.7.3.3 RichText sentence | Cites "Table 226"; correct is **228**. |
| T188 | "`AP` … shall take precedence over the **`L` and `BS`** entries" — a widget has no `/L`; copied from T175. Deleted in 2.0's T191. |
| T165 | Prints **no integer values**; LockedContents is bit 10 = **512**. |
| T194/197 `U` row | "(see Table 168)" — 168 is the *appearance* dictionary. |
| T193 `/S` row | "see Table 194 for specific values" — the action-type registry is **Table 198**. |
| §12.6.4.16 | "(see the Bibliography)" — both JavaScript authorities are in **clause 3, Normative references**. Systematic across ≥8 sites. |
| Edition table map | 1.7 T188 widget / T189 MK = 2.0 T191 widget / T192 MK. 1.7 T184 is FileAttachment; 2.0 T184 is Rubber stamp — **never carry a bare "Table 184" across editions**. |
