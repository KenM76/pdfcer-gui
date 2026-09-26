# `text::forms` — every string the Forms panel shows

One area of the catalog described in [`crate::text`]'s header, covering
[`crate::panels::forms`] — the panel that lists an `/AcroForm`'s fields
and lets an operator **fill** them.

It sits beside [`crate::text::panels`] rather than inside it, which is a
deliberate placement rather than an oversight: the Forms panel is not one
of the six report panels that module covers, it is the first panel in this
build that **changes the document**, and its copy is dominated by
*disclosures* and *refusals* rather than by field labels. Keeping it in its
own file means the reviewer of a disclosure sentence is reading a file that
contains nothing but disclosure sentences.

## Almost every sentence here is salvaged verbatim

Carried across from the old shell's `ui_text.rs` **with its doc comments**,
because the doc comment is usually the record of a defect the wording was
changed to fix. ⚠ **Fresh words re-derive a decision already paid for and
have none of the evidence that bought it**, and this area carries more such
decisions per line than any other:

- [`forms_no_acroform`] does not say "no fields found". It says a page can
  *look* like a form without carrying one, because otherwise "no fields"
  reads as pdfcer failing to find something that is plainly on the page.
- [`form_field_password_tooltip`] exists because a masked box reads as
  "secure" to anyone not told otherwise, and the value really is stored as
  plain text in the file.
- [`forms_data_export_carries_rich_text`]'s counterpart in the old catalog
  carries a `` recording that the sentence outlived the behaviour it
  described by one commit. That string is **not** salvaged here (this build
  has no export), and the lesson is: a disclosure that has gone stale is a
  false statement the operator has no way to check.

## The three sentences that are NEW, and why each had to be

| Function | Replaces | Why |
|---|---|---|
| [`forms_xfa_note`] | the old shell's post-fill `fill_xfa_may_disagree` | The old note appeared *after* a value was typed. Whether the document carries an XFA packet is a property of the FILE, knowable before the operator touches anything, so it is said up front. |
| [`form_field_no_on_state_note`] | the old shell's post-toggle `form_field_no_appearance_for_state` | The old one was computed from a predicate that could never be true (see [`crate::panels::forms::rows`]' header, "The salvaged appearance check could not fire"). This one asks a question the model can actually answer. |
| [`forms_no_fillable_fields`] | — | This build derives the "N you can fill here" count from what the panel actually offers, and the count can legitimately be zero on a form full of fields. A silent zero looks like a bug. |

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have.** This build fills;
  it does not create, delete or rename a field, and no string here implies
  otherwise.
- **A warning glyph is never the only cue.** Every `⚠` sentence reads
  correctly with the glyph stripped, because a glyph is a colour-class cue
  and `RIBBON_IA.md` R84 forbids carrying state in one.

## Item notes

### `mod authoring`

Re-exported below, so this split is invisible at every call site — see that
module's header for why this is the seam R2 forced and why it was the right
one anyway.

### `fn every_refusal_explains_a_different_refusal`

Each exists because, without it, the operator's only available reading
of an inert control is *"pdfcer got it wrong"*. Two that read alike
would send them looking for the wrong cause — and two of these
describe gates that genuinely disagree with each other on the most
common certified document there is.

### `fn the_structural_refusal_does_not_claim_values_are_locked`

Not a tautology of the test above. These two are the pair most likely
to be collapsed into one string by someone tidying up, because on most
documents they are both absent and on a fully-locked one they are both
present. The case that matters is the ordinary certified fillable form
(`/P 2`), where filling is permitted and flattening is not — and an
operator told "form values cannot be changed" while happily typing
into the fields has been misinformed by their own tool.

### `fn a_warning_glyph_is_never_load_bearing`

R84 — never a colour-class cue alone. A `⚠` is exactly that, and a
sentence whose meaning depended on it would be unreadable to anyone
whose font lacks the glyph or who is listening rather than looking.

### `fn an_unreadable_rich_value_is_not_reported_as_an_unformatted_one`

The single most consequential distinction in this file. Both cases
render as a row with no formatting listed, and only one of them is a
reason to stop before pressing Convert: an unreadable `/RV` means the
operator is about to discard formatting **nobody has seen**.

### `fn emphasis_is_grouped_before_typography`

Pins the ordering decision recorded in
[`form_field_rich_text_summary`]'s header — the one found by reading a
rendered panel rather than the code. `bold` and `italic` must end up
adjacent even when they arrive on runs separated by a run carrying the
`/DS` size and family.

### `fn the_count_line_is_scoped_to_this_panel`

The sentence says "you can fill **here**", not "fillable fields". That
word is what makes the count honest when a certification signature
disables every row: the panel is describing itself, not the model's
`is_fillable` predicate. See the function's own header for the bite
this closes.

### `fn the_recompute_explainer_states_a_rule_rather_than_a_gap`

"pdfcer never runs a document's JavaScript" is a project rule, and the
wording has to read as a decision rather than as an unfinished feature
— otherwise an operator waits for a version that will never come.

### `mod groups`

Re-exported below, so the split is invisible at every call site — the same
R2 seam [`authoring`] and [`tab_order`] were cut along, and for the same
reason: a reviewer of one surface's wording should be reading a file that
contains only that surface's wording.

### `fn forms_no_acroform`

States the distinction that actually matters to the operator: a page can
LOOK like a form — ruled boxes, printed labels — without carrying a single
interactive field. Without this, "no fields" reads as pdfcer failing to find
something that is plainly there on the page.

Salvaged verbatim except for the final clause, which named the old shell's
text tools. This build has none, so the sentence stops at the honest half
rather than pointing at a control the operator cannot find — the
"no placeholders" invariant (`PROJECT_PLAN.md` §3) applied to prose.

### `fn forms_empty_acroform`

A distinct sentence from [`forms_no_acroform`], because the two are
different facts about the file: one says the document never had a form, the
other says it declares one and lists nothing in it — which is a
malformation worth being able to see.

### `fn forms_field_count`

# `fillable` is what THIS PANEL offers, not what the model calls fillable

The obvious implementation counts `pdfcer_core::forms::Field::is_fillable`,
and it is wrong in a way the operator can see. `is_fillable` answers *"could
a fill edit change this field's value"* — it excludes read-only, signature
and pushbutton fields and nothing else. The panel additionally declines to
offer an editable control when a certification signature forbids filling the
whole document, and when a field holds rich text pdfcer cannot author.

So a certified fillable form would read "12 fields, 12 you can fill here"
above twelve disabled boxes.

This is `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md`'s third
bite — *"a returned count is not always the count to display"* — one verb
over. Its worked example is `set_group_style` returning members
**regenerated** rather than members that will visibly **move**; the shape
recurs wherever a count comes from a model's predicate and the sentence
describes the interface's behaviour. The count displayed must be derived
from what the panel will actually draw.

### `fn forms_inline_field_roots_note`

# The count line understates the file, by exactly this many

`pdfcer_core::forms::AcroForm::inline_field_roots` counts `/Fields` entries
that are **direct dictionaries rather than indirect references**. Table 218
admits only references, so such an entry is malformed — and
`parse_acroform` skips it, because a field with no object identity has
nothing a fill could write to.

The consequence is that `fields.len()` is not the number of fields in the
file. This is the third bite in
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` in its most
literal form: the count the model returns is not the count to present as
the whole truth. An operator comparing pdfcer's "9 fields" against another
reader's "10" must be able to find out why here rather than concluding
pdfcer lost one.

### `fn forms_no_fillable_fields`

New in this build. A zero in a count line is indistinguishable from a
panel that failed to look, and on a real form — a signed contract, a
certified return, a read-only archive copy — zero is the correct and
unsurprising answer. Saying so converts a suspicious number into a
statement about the document.

Deliberately does NOT enumerate the reasons: each row already carries its
own, and a summary that tried to aggregate four different causes would
either be vague or would be a second place to keep them in step.

### `fn forms_need_appearances_note`

The real trap this closes: a value pdfcer writes is correct in the file, but
a viewer that honours `/NeedAppearances` draws it from the value while one
that does not draws the stale baked appearance — so the same document shows
two different things depending on who opens it.

### `fn forms_javascript_note`

**pdfcer never runs a document's JavaScript**, and that is a standing
project rule rather than an unfinished feature. Fields whose value a script
would have computed are therefore left exactly as last saved, and this
sentence is what stops an operator concluding the form is broken when a
total does not move.

The remedy is [`recompute_heading`]'s section, which reproduces a
whitelisted subset of Acrobat's built-in calculations natively.

### `fn forms_xfa_note`

# Why this is form-wide and up front rather than per-fill and after

The old shell surfaced this as `fill_xfa_may_disagree`, a status note
produced from `FillOutcome::xfa_may_disagree` **after** a value was
committed. That is a faithful reading of the engine's outcome and the wrong
moment for the operator: whether the document carries an XFA packet is a
property of the FILE (`pdfcer_core::forms::AcroForm::xfa`), knowable before
anything is typed, and identical for every field.

Saying it once, before the list, means the operator learns that their
typing may not stick *before* they do it — and it removes the need for this
panel to carry a note channel back from the action funnel at all. See
[`crate::panels::forms`]' header, "Nothing has to travel back from
`apply`".

Leads with the consequence rather than the mechanism, which is the old
string's decision kept: an operator cares that a value might not stick, not
that the document has two field descriptions.

### `fn forms_certification_note`

Salvaged from the old shell's `form_field_certification_disabled_tooltip`,
and **promoted from a per-row tooltip to a panel-wide line**. The reason is
the same one the old panel gave for asking the gate once: a certification
signature forbids filling the whole DOCUMENT, not one field, so repeating
it on forty rows is forty copies of one fact.

It is still attached to each disabled row as well (see
[`form_field_certification_disabled_tooltip`]), because a row an operator
clicks on and cannot type into must explain itself where they are looking.

### `fn form_field_row_tooltip`

The row's visible label prefers `/TU`, which is what a screen reader
announces; but an operator diagnosing why a value did not match needs the
technical name, and it may differ from the label or be absent from it.

### `fn form_field_page_suffix`

A leading space is part of the string because it is appended to a label
this catalog does not own — the field's `/TU` or its name, which come from
the document. Putting the separator here keeps the whole of the assembled
sentence's punctuation in the catalog.

### `fn form_field_certification_disabled_tooltip`

The per-row half of [`forms_certification_note`]. Both exist: the panel-wide
line is what an operator reads when scanning, and this is what they get when
they click the box that will not accept typing.

### `fn form_field_signature_note`

Listed rather than hidden (R83): an operator scrolling past a signature
field should see that pdfcer knows it is there.

**WHY THIS SENTENCE MAKES NO CAPABILITY CLAIM, HAVING TWICE BEEN
CORRECTED FOR MAKING ONE.**

It read *"pdfcer does not create or verify signatures yet."* Two claims in
one sentence, and when verification arrived
(`signature::verify_all_with_trust`, reached from
`crate::panels::signatures`) the half that stayed true kept the half that
had gone false looking true.

⚠ Splitting the conjoined claims did **not** save the row. The surviving
half went false in its turn the moment the engine shipped signing —
`crate::sign` reaches `EditSession::sign`, and `file.sign` is a registered
command — so a panel row was telling the operator *"pdfcer cannot sign a
document"* while the ribbon two clicks away offered to do exactly that.

**The lesson is NOT "split conjoined claims".** It is that **a
hard-coded sentence must not make a capability claim at all.**

R8: registering a command is the only way this GUI may learn that a
capability exists. A `&'static str` cannot consult the registry, so any
capability claim baked into one is a fact with no owner — nothing recomputes
it, no gate can read it, and it cannot go stale loudly. Signing is
additionally behind the `signing` Cargo feature, so even a claim that was
true today would be false in a build that strips it, which is precisely the
removability R8 exists to protect.

⇒ So this sentence now says only what a signature field **is** — a thing
that is signed rather than typed into, which is a property of the PDF field
type and true in every build — and points at the Signatures panel, which is
never stripped (`signing` gates the private-key side only; reading and
verifying are always compiled in). What pdfcer can *do* is disclosed the way
R8 requires: by `file.sign` being present in the ribbon, or absent from it.

### `fn form_field_length_caption`

Two numbers and a slash, deliberately wordless: it sits under every capped
field on the form and a sentence there would be read once and then be
noise. The limit itself is enforced live rather than at commit — see
[`crate::panels::forms::rows`] — so this caption describes a rule the
operator has already felt.

### `fn form_field_password_tooltip`

Says the masking is display-only. A masked box reads as "secure" to anyone
not told otherwise, and the value really is stored as plain text in the
file — the sneaky half of rule 4 if left unsaid.

### `fn form_field_commit_tooltip`

New in this build, and it earns its place because the commit rule is
invisible: a text field writes its value when focus LEAVES it, not on every
keystroke, so an operator who types and then looks at the page sees
nothing happen. (The rule itself is
[`crate::panels::forms::rows::commit`], and it exists so one typed word is
one undo step rather than a dozen.)

### `fn form_field_no_on_state_note`

**New in this build**, replacing a salvaged string that could not fire —
see [`crate::panels::forms::rows`]' header for the full account.

# Why the wording is "cannot", not "will look the same"

The obvious sentence — *"the value changes but the page will not"* — is
what the old shell's equivalent tried to say, and it is **false against
this engine**. `EditSession::set_button_state` refuses any state other than
`Off` that no widget defines, by name:
`EditError::FieldStateUnknown { name, state, available }`.
`pdfcer_core::forms::Widget::on_states` lists the states the widget's `/AP`
`/N` sub-dictionary defines, **excluding `Off`** — so an empty list means
there is no state pdfcer may select, and a tick here would be an affordance
for a call that always errors (R83).

The control is therefore drawn **disabled and explained**, exactly like a
`/Locked` layer row, and this is the explanation. It says the document is
what offers nothing, not pdfcer.

### `fn form_field_choice_value_not_listed`

New in this build. `/V` may legitimately hold a value that `/Opt` does not
list — set by another program, or left behind when the option list was
edited — and the row shows it verbatim rather than blank, because showing
blank would claim the field is unanswered when it is not.

Said out loud because a value that appears in the box and in none of the
choices below it looks like a rendering fault.

### `fn form_field_choice_multi_value_not_listed`

A check-box stack has no box for a value matching no option, so "it is
shown as it is stored" cannot be true and there is nothing for the
operator to replace. The value is dropped from the selection the row
writes — carrying it into `set_choice_value` earns
`ChoiceValueNotInOptions`, a refusal naming a value the operator never
touched — and this says so, because a silent drop is an inference the
operator cannot see.

### `fn form_field_rich_text_note`

The row is read-only, and the reason is **correctness** rather than
fidelity: pdfcer cannot author `/RV`, and §12.7.3.4 / §12.7.3.3 bind
appearance generation for these fields to `/RV` rather than `/V`, both with
`shall`. Writing plain text and leaving `/RV` behind would make conforming
readers rebuild the appearance from the OLD text — the document would
display words nobody typed.

### `fn form_field_rich_text_summary`

# Why a generic warning was not enough

[`form_field_rich_text_convert_tooltip`] already says "bold, colours and
fonts are DISCARDED". That is a category, not this document: it reads the
same on a field whose only formatting is 12 pt Helvetica as on one carrying
three colours and a superscript. The always-visible summary names what is
there; the per-run breakdown ([`form_field_rich_text_runs_tooltip`]) is a
hover away. Progressive disclosure, with the frequent question visible.

# Collected by category, emitted in a fixed order

Not in the order the runs happen to mention things. A single
accumulate-as-you-go list produced, on the old shell's shipped fixture,
`"bold, 12 pt, Helvetica, #FF0000, italic"`: run 0 is the bold one and
contributes the `/DS` size, family and colour with it, so `italic` from run
2 landed at the far end — the two facts an operator most needs to compare
were the two furthest apart. Found by reading the rendered panel, not the
code.

So emphasis first (what the words LOOK like), then the typographic
settings, then layout. Within each bucket, first-seen order, which is
stable because it comes from document order.

Only features actually SET appear. `richtext::Style` uses `None` for
"neither the run nor `/DS` specified this", which is not the same as a
default; listing unset properties would both bury the real ones and assert
something the file does not say.

### `fn form_field_rich_text_runs_tooltip`

The summary answers "what formatting is in here"; this answers "which words
have which". Both are wanted and only one fits on a form row.

A tooltip is the right home precisely because this is the OCCASIONAL
question. It is **not** a disclosure obligation: the destructive act's
consequence is already stated in the always-visible summary, so nothing
here is a fact the operator must see before clicking. If it were, a hover
would be the wrong place for it.

Text is shown quoted and elided so one long run cannot push the rest off
the screen — the point is which run, not the whole value, and the value is
already in the read-only box above.

### `fn recompute_pending`

The blank-operand clause is appended rather than being its own line so the
two facts an operator weighs together — how many fields move, and how many
of the inputs pdfcer had to read as zero — are read together.

### `fn recompute_skip_row`

`reason` is `pdfcer_core::form_script::recompute::Skip`'s own `Display`,
passed through rather than rewritten: core writes each as a complete clause
and replacing one with a shell paraphrase throws away the only part of the
sentence that helps.

### `fn recompute_apply_tooltip`

# It says "one undo step per field", and the old catalog said "one undo
step"

The old wording was inherited from a control that writes one command, and
it is wrong here. `pdfcer-core` has no batch-recompute verb — part 3 of the
core API is explicit that *"applying a plan is a loop the shell writes
itself"* — so the shell calls `EditSession::fill_text_field` once per
planned change and each of those is its own undo entry. An operator told
"one undo step" would press Ctrl+Z once, see one field revert, and
reasonably conclude undo is broken.

Stated rather than fixed, because the fix is core's: a single
`apply_recompute` verb would make one command out of the loop. Until then
the honest sentence is the one that matches what happens.

### `fn recompute_order_is_a_guess`

A rule-4 disclosure of the purest kind: pdfcer inferred something (which
order to evaluate in), the inference changes the numbers, and no other
reader is obliged to agree with it.

### `fn reset_explainer`

Says the destructive part first. A section titled "reset" that opens with
how it works has buried the only sentence that changes the operator's
decision.

### `fn reset_to_empty`

Parenthesised because it is not a value — it is the *absence* of one, and
`/V` removed and `/V` set to the empty string are different bytes. A shell
that showed both as `""` would be describing the wrong edit.

### `fn reset_already_default`

Stated rather than left as a gap in the list. A field the operator expected
to see and does not is a question; "3 already hold their default" is the
answer, given before it is asked.

### `fn forms_regenerate_tooltip`

This is the operator-facing answer to [`forms_need_appearances_note`]: a
document carrying that flag asks viewers to draw field values themselves,
and viewers disagree about whether to. Regenerating bakes an appearance for
every field and clears the flag, so every viewer shows the same thing.

### `fn forms_flatten_tooltip`

Says what is lost, what survives, and — the part an operator cannot guess —
that under the default incremental save the old values are still present in
the file's previous revision. That last clause is why this is a tooltip and
not a blocking confirmation: flatten is not structurally irreversible the
way applying a redaction is. See [`crate::panels::forms::edit`]'s header for
the full argument, which was made against what each operation actually does
rather than by analogy.

### `fn forms_flatten_needs_redraw_note`

**New in this build**, and it closes a real data-loss path rather than a
cosmetic one.

Flatten works by invoking each widget's **existing** `/AP` as a page
XObject. A field with no normal appearance — which is exactly what
`/NeedAppearances` announces, and what
`pdfcer_core::forms::Field::has_appearance` reports per field — has nothing
to invoke, so flattening burns **nothing** for it and then removes the
field. The typed value disappears from the visible page.

The remedy is [`forms_regenerate_button`], which is why the two controls
sit side by side and why this sentence names it. Core's own guidance is
the same: regenerate first, then flatten.

### `fn forms_canvas_undrawn_note`

# Why this sentence exists at all

`crate::canvas::forms` lets an operator click a field where it is drawn.
The word *drawn* is load-bearing: a widget with no `/AP` `/N` paints
nothing, so a click target over it would be an invisible affordance — the
operator can only find it by accident and cannot find it again. The canvas
therefore declines it, and this is the panel telling them **where the field
went**, which is the whole difference between a routing decision and a
capability that quietly disappeared.

# The remedy sentence was wrong on its first draft, and driving the
binary is what caught it

It read: *"Use “Redraw values” to draw them, and they can then be clicked
where they sit."* That is **false for the case the sentence is about.**

Measured on `demo-form.pdf`, which carries exactly one undrawn field
(`Full name`, with no value): pressing Redraw values traced
`form-regenerate-appearances commands=0 (nothing to do)` and the field
stayed undrawn. `EditSession::regenerate_appearances` walks the fields and
`continue`s on any text field whose `/V` is not `FieldValue::Text` — an
**absent** value has nothing to draw, so an empty field is skipped by
design. Redraw only helps a field that is undrawn *and already holds a
value*, which is a real and common case (a form filled by another program
that never generated appearances, which is what `/NeedAppearances`
announces) but is not this one.

The remedy that always works is the one this panel is: `fill_text_field`
writes `/V` **and** regenerates the `/AP` of every widget of the field, so
filling an undrawn field here once makes it drawn — and therefore clickable
on the page from then on. So the sentence names both, in the order they
apply.

The rejected first draft is worth naming because of its SHAPE: it was
plausible, it read well, no test could contradict it, and it was a promise
to the operator that the engine would not keep. **A sentence no test can
contradict is the one to check against the engine by hand.**

### `fn forms_canvas_unreachable_note`

Two causes, said as one sentence because they have one remedy — fill it
here — and because an operator does not need to know which of them applies
to which field in order to act:

1. **A rotated page.** `egui` cannot rotate a text box, so on a `/Rotate 90`
   page an in-place editor would run horizontally across text the
   appearance draws vertically. The click and the *placement* are both
   correct at every rotation; it is only the editor that cannot be.
2. **The file does not say which page the field is on.** `/P` is optional
   on a widget annotation, and without it there is no page to put a box on.

Deliberately **not** phrased as a limitation to be fixed ("pdfcer cannot
yet…"), because one half of it is a property of the file rather than of
pdfcer, and a sentence that promised a future version would be a promise
only half of which could ever be kept.

### `fn forms_fill_autosize_note`

A `/DA` of `0 Tf` means auto-size (§12.7.3.3): the field declines to state a
size and leaves the writer to pick one that fits. pdfcer picks one, and the
number it picked is what lands in the file — so **nothing in the saved
document says the number was pdfcer's rather than the author's**, and no
amount of re-reading the field afterwards can recover the distinction. That
is precisely the shape of thing rule 4 exists for: an inference, made on the
operator's behalf, invisible in the result.

It matters because another writer, filling the same field, will choose its
own number — so a form filled here and a form filled elsewhere can legibly
differ, and the operator is entitled to know why before they compare the
two.

The field is named because the disclosure is read somewhere other than
where the value was typed — in a panel, possibly beside forty other rows,
possibly after a fill made by clicking the page.

### `fn forms_fill_autosize_overflow_note`

# Why this is a separate sentence and not a suffix

`AutoFitBound::Floor` is the one outcome where pdfcer's answer is not an
answer. The engine says so at the branch that returns it — *"the one case
where the returned size does NOT fit the constraint that produced it"* —
and stops shrinking at a legibility floor rather than rendering something
unreadable.

⚠⚠ **A point size cannot carry that.** *"pdfcer chose 6.0 pt"* reads as a
decision whether it fitted or not, which is why this note exists beside
[`forms_fill_autosize_note`] rather than inside it. `OPERATOR_REQUESTS.md`
**O86** promises the operator that *"pdfcer now tells you which way it
decided … the box is too small for this text, which will overflow"* — and
**a promise the engine and the CLI keep is not kept by this shell until
this shell says the words.**

It names the **remedy**, because there is one and it is the operator's:
make the box bigger, or put less in it. A disclosure with an available
remedy that withholds it is a complaint.

### `fn forms_fill_autosize_width_note`

Worth its own sentence rather than folding into
[`forms_fill_autosize_note`], because the two point at **different edits**.
A height-bound field gets bigger text by being made taller; a width-bound
one does not — it was already going to be taller and got shrunk sideways,
so making it taller changes nothing and the operator would try that first.

⚠ It still carries the *"another program may choose differently"* clause,
because that is true of every auto-sized field regardless of which bound
won, and it is the half that matters when the sheet is opened in Acrobat.

### `fn forms_fill_unencodable_note`

The field's font is a Base-14 Latin face and `pdfcer-core` encodes into
`WinAnsi`; a character with no code there is written as `?`. The saved value
**is** the substituted one, so re-reading the field tells the operator what
pdfcer wrote and never that it wrote something other than what they typed.

This is the more serious of the two fill disclosures and is worded as such:
an auto-size is a difference of appearance, this is a difference of
*content*. The count is given rather than the characters, because listing
them would mean echoing the operator's own typing back into a surface that
may be screenshotted, and because the count is what tells them whether it
was a stray character or the whole name.

### `fn forms_structural_certification_disabled_tooltip`

Distinct from [`form_field_certification_disabled_tooltip`], and the
distinction is the one the old shell drew and argued for at length: filling
takes core's `/P`-aware gate, while flattening is a **structural** change to
the form and takes the strict one. On the ordinary real-world shape — a
certified fillable form at `/P 2` — filling is offered and flattening is
refused, so one sentence could not cover both without being wrong about one
of them.

### `fn widget_siblings_unmoved`

The engine's own words for why this is owed: *"A field with widgets on
pages 1, 2 and 3 looks like one thing to an operator who asked to move 'the
signature box'. Moving one and silently leaving two behind is the kind of
partial result that reads as a bug later."*

It says the move was **correct**, not that something went wrong. The
boxes are separate placements of one value and moving one is exactly what
the operator dragged — so the sentence's job is to stop them hunting for a
fault, not to apologise for one.
