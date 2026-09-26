# `text::fieldclip` - the sentences the FORM-FIELD clipboard can say



`Pass 167.0` shipped `pdfcer_core::formclip` and every one of those
properties now travels. The loss note is **deleted**, not softened, on the
engine's own instruction: *"you should not be maintaining a hand-written map
of which properties survive, because it rots silently every time we add an
authoring key."*

What replaced it is `FieldPasteOutcome::disclosures` - a `Vec<String>` the
**engine** writes, covering a dropped value, a carried calculation and its
`/CO` registration, a renamed font resource, an ignored rectangle size, the
tab-order position, a dropped structure-tree link and a reused accessibility
name. It reaches the status row through `vector_edit` like every other verb's
disclosures, and **not one word of it is written here**.

⇒ The rule that decided it is this shell's own and it has now removed two
sentences from this file in one day: **one fact, one wording.** The engine's
version is authoritative - it reports what the operation *did*, not what the
shell *intended* - and a second phrasing is a divergence waiting to happen.

## What remains

[`refusal`] - why nothing happened. Same posture as `text::clipboard`: a
keystroke that does nothing and says nothing is indistinguishable from a
broken keyboard.

[`os_marker`] - the sentence a copy leaves on the *operating system's*
clipboard, which is not a courtesy but a **requirement**; see its own header.

[`candidate_name`] - the spelling of a pasted field's name.

## Rule 4, in one line, because it still governs

A pasted field renders exactly as a saved-and-reopened one would - no badge,
no tint, nothing drawn on the page. The disclosure lives off-canvas, on the
status row. *Render normally; report separately.* **Both.**

## Item notes

### `fn an_engine_refusal_is_passed_through_verbatim`

No prefix, no suffix, no rewording. A shell that decorated the engine's
refusal would be maintaining a second copy of a taxonomy that moves - and
this variant exists because two of this file's own hand-written refusals
went stale within an hour of being written.

### `fn refusal`

Returns an owned `String` rather than a `&'static str` because
[`Refusal::EngineRefused`] carries the engine's own wording, which is not
static and must not be paraphrased. The four shell-owned variants are still
literals here, so `check-ui-strings` still sees them.

### `fn brings_a_script`

The one pre-press disclosure this shell owes, and it exists because the
fact is **invisible**: a form field carrying a calculation, a format script
or a validation looks exactly like one that does not, on the page and in
every screenshot of it. Everything else about a paste is reported afterwards
by the engine, which knows what actually happened; this has to come first,
because after the press the operator has already committed the gesture.

# Why it does not say WHICH script, or what it references

Because the engine deliberately does not resolve the field names inside it,
and that restraint is right. Acrobat is documented silently dropping a copied
JavaScript reference to a field the target document lacks — discovered only
on reopen, with nothing said at the time. Naming the uncertainty beats
half-analysing it and reporting a confident half-answer.

# Why it is not a warning, and does not block

It is usually what the operator wants. A title-block field that computes a
sheet count *should* bring its calculation to the next drawing — that is the
reason for copying it. The sentence exists so the outcome is not a surprise,
not to discourage the act.

### `fn name_is_a_path`

`FormAuthorError::DottedPartialName`, reaching here through
`Declined::DottedPartialName`. Raised by three engine verbs; reachable from
**one** surface in this shell — the Tab-order register panel's adopt boxes,
which take free text and gate only on non-empty.

# Why the sentence does NOT warn about losing anything

Because nothing would be lost, and a refusal that overstates its stakes is
the defect this surface already had once. `adopt_widget` does not touch an
existing field's `/Kids`: a pre-existing `Text` survives an adopt of
`Text.2` completely intact. [`name_crosses_a_field`] above is the one where
a field really would be destroyed, and the two sentences have to stay
distinguishable on exactly that point.

What WOULD be produced is a field **nobody can address**. §12.7.3.2 makes
its FQN that same dotted string, so every resolver splits on `.` first,
looks for `2` inside a group `Text`, finds a terminal there, and stops. It
renders. It accepts a click. And `fill_text_field`, FDF/XFDF import, a
`/CO` calculation-order entry and a reset-form `/Fields` array can none of
them reach it — **pdfcer's own fill verbs included**.

⇒ Hence *"can be clicked but never filled"*, which is the consequence in
the operator's terms. He is about to put a box on a drawing that nobody,
including pdfcer, can ever type into, and it will look perfectly normal.

# The wording

It echoes the name back, because the panel that reaches this shows a name
box per unclaimed widget and the bar has one sentence — without the name it
would not say which row. It says *the document is unchanged* first, because
that is the question a refusal raises. It explains the period rule in one
clause and does not cite the clause number — the operator is a draughtsman,
and §12.7.3.2 is for this comment, not for the bar. And the remedy is the
only one there is.

### `fn os_marker`

This exists because of a toolkit constraint, not a design wish, and
without it `Ctrl+V` does not work at all. `egui-winit-0.35.0` synthesises
`Event::Paste` **only when the OS clipboard holds non-empty text**, and
swallows the keystroke entirely otherwise — no key event, no paste event,
nothing. So a paste of something pdfcer holds in its own memory would depend
on whether the operator had recently copied text in some other application.


# The wording

For a human who pastes into a text editor and wonders what they got. It names
the field, because a form has many and *"a form field"* would not say which.
It names both chords, because the second one is the whole feature and an
operator who reads this sentence in an email has just been taught it.
