# `pdfcer-gui/text/markup/edits`

## Item notes

### `fn a_dropped_rich_copy_is_disclosed_and_an_absent_one_is_not`

Both directions, because either alone is satisfiable by a broken build:
a function returning `Some` unconditionally passes the first, and one
returning `None` unconditionally passes the second. This project has
shipped the second shape — a disclosure wired to a field nobody set —
and the only symptom was silence.

The silent case is the *common* one and that is why it is asserted at
all: pdfcer's own annotations never carry `/RC`, so every comment this
operator writes and then edits takes the empty path. A sentence that
fired on all of them would be read once and skipped thereafter,
including on the one edit where it mattered.

### `fn the_rich_text_sentence_is_in_his_words_not_the_specs`

The wording rule this catalog follows everywhere: name the thing in the
operator's vocabulary. A drawing-office reviewer has no idea what `/RC`
or `/DS` are, and the only fact they can act on is that a comment they
had styled elsewhere is now plain text.

It must also **not open with an apology or a loss**, because the net
effect of this change is that their document stopped contradicting
itself. Before it, `/Contents` held the new words while `/RC` held the
old ones and some readers showed the old ones — on a `/FreeText`, on the
page itself.

### `fn a_delete_with_no_collateral_produces_no_sentence`

The overwhelmingly common annotation has no pop-up, no replies and no
group. A sentence that appeared on every selection would be read the
first three times and skipped for ever after, which is exactly what makes
the interesting case invisible when it finally arrives.

### `fn the_two_tenses_name_the_same_four_consequences`

Pinned by counting: for one set of counts each function names every
consequence the other names. What is deliberately NOT asserted is that
the strings are equal or mechanically derived — they are not, because
*"1 reply is left"* and *"1 reply will be left"* are different English —
and a test that demanded a shared template would forbid the difference
that makes both of them readable.

### `fn neither_tense_promises_redaction`

Deleting an annotation takes an entry out of `/Annots`; it does not touch
page content, and an incremental save leaves the previous revision in the
file. `docs/core-api/03-capabilities.md` §3.4 — *"delete is not
redaction"* — and a preview that promised removal would be the exact
wording `crate::text::redact`'s header forbids, stated one gesture
earlier than the disclosure that already observes the rule.

### `fn the_locked_sentence_names_the_file_as_the_author_of_the_rule`

§12.5.3's `Locked` is a statement the producer wrote into the annotation.
Wording it as pdfcer's own decision would send an operator looking for a
pdfcer setting to turn off, and there is not one.

### `fn deleted_collateral`

# Why a deletion needs to say anything at all

Because the operator named **one** annotation and the engine may legitimately
remove or alter more. `AnnotationDeletion` reports three such cases and each
is a fact about the file rather than about pdfcer:

* a `/Popup` companion goes with its parent — §12.5.6.14 is a `shall`, a
  pop-up *"shall not appear alone but is associated with a markup
  annotation"*, so leaving it would be a clause violation. The spec
  requiring it is a reason, **not a licence to stay quiet**;
* replies hanging off it as `/IRT` targets are **orphaned**, not deleted —
  the thread survives and its root does not;
* group members are **promoted** when the group's primary goes.

Rule 4, in its second clause: pdfcer did something the operator did not ask
for, so pdfcer says so, off-canvas, in words.

# What this deliberately does NOT say

**That the content is gone from the file.** It is not: deleting an
annotation removes an entry from `/Annots` and does not touch page content,
and the previous revision is still in the file after an incremental save.
`docs/core-api/03-capabilities.md` §3.4 states the rule this observes —
*"delete is not redaction"* — and the redaction surface is where that
distinction is made loudly. Saying "removed" here would be the exact wording
`crate::text::redact`'s header forbids.

Returns `None` when nothing but the named annotation was affected, which is
the ordinary case: a disclosure that fires on every delete is one nobody
reads by the third time.

### `fn popup_left_behind`

**The one consequence of a move that this program cannot show.** §12.5.6.14
makes a pop-up a separate annotation with its own placement and leaves to the
reader whether it follows; `pdfcer-core` reports the object it left behind and
says the decision is the shell's.

This shell does not draw pop-ups, so a stranded one is invisible here and
perfectly visible in Acrobat — which is Rule 4's surviving half in its
purest form: render normally, report separately, both.

It says pdfcer did **not** move it, rather than offering to. Moving it
would be a second undo entry for something the operator cannot see, and a
gesture that produces two entries is one `Ctrl+Z` away from a state nobody
can explain.

### `fn stroke_width_unchanged`

The engine asks for this sentence by name — *"an operator who scaled a
square 3× and expected a heavier border needs telling it stayed"* — and it is
Rule 4's surviving half in its purest form: the shape grew around the border
and **nothing on the canvas says the border did not grow with it**.

It states the default as a **choice**, not as a limitation, because it is
one: on a CAD drawing a line weight is a drafting standard rather than
decoration, which is this project's own argument and the one the engine
promoted into the rule that decides every future case — *is the property a
length in the space being transformed?* An inset is; a line weight is not.

### `fn appearance_distorted`

Not a defect and not pdfcer's choice — an arithmetic limit. **Neither PDF
nor SVG has a per-axis stroke width**: both are scalars, so a stroke drawn
through a matrix applied *after* stroking cannot keep an even thickness under
a non-uniform scale. Inkscape closed the identical report **Invalid** and
silently produces the distorted stroke.

pdfcer says so instead, which is the whole difference. The operator can see
the result — a border thicker on one axis — and cannot see *why*, so the
sentence names the cause and the remedy: drag a corner with Shift held, or
accept it.

### `fn rich_text_dropped`

# What actually happened, because the operator cannot possibly guess it

PDF stores a comment **twice**. `/Contents` is the plain string; `/RC` is a
*rich-text version of the same comment* (§12.7.3.4), and §12.5.6.2 pairs
them in as many words — *"Contents (or RC and DS)"*. pdfcer writes
`/Contents` and cannot author rich text, so editing a note used to leave
the two **disagreeing**: the plain copy held the new words and the rich
copy still held the old ones.

⚠ **That is not lost content — it is WRONG content, stated confidently**,
and which copy an operator sees depends on their reader. On any markup,
Table 170 makes `/RC` the text *"displayed in the pop-up window"*; on a
`/FreeText`, Table 174 makes it *"used to generate the appearance"*, so
**the page itself** could have shown the old words.

The engine now removes the stale copy rather than regenerating it —
synthesising rich text from a plain string would invent formatting nobody
chose, and §12.7.3.4 gives no meaning to an empty rich value.

# Why this needs a sentence at all

Rule 4, in its narrowest and clearest form: **pdfcer dropped a key the
operator did not ask it to drop.** Nothing on the page changes, nothing in
the panel changes, and the only way to find out would be a diff of the
file. That is the exact shape of edit this project's disclosure rule
exists for.

⇒ And it is the *good* news, not a warning, which is why the wording leads
with the fix rather than with the removal. Before this the operator's
document was inconsistent and said nothing; now it is consistent and says
so. A sentence that opened *"pdfcer removed something"* would read as a
loss.

It names the **formatting**, not the keys. `/RC` and `/DS` mean nothing
to a drawing-office reviewer, and the only consequence they can act on is
that a comment they had styled somewhere else is now plain.

`None` on the ordinary case, which is nearly every comment: pdfcer's own
annotations never carry `/RC`, so this fires only on a note that arrived
from Acrobat or another rich-text editor. A disclosure that fired on every
edit is one nobody reads by the third time — the same rule
[`note_replaced`] follows.

### `fn note_replaced`

Rule 4's surviving half, and `pdfcer-core` commissioned this sentence
itself: *"those words are gone from the document and nothing on the page
shows that they were ever there"*. A shape does not change when its note
does. A sticky's words live in a pop-up window this shell does not draw. So
on every subtype an operator can comment on, overwriting a note is an edit
with **no visible consequence at all** — which is precisely the class this
project's disclosure rule exists for.

It carries **the text, not a count**, because the engine chose to return
the text and said why: a count lets a shell *mention* the loss, and the text
lets it *offer the words back*. They are on the status line for as long as
the edit epoch holds, so an operator who overwrote the wrong comment can
read what was there and retype it — `Ctrl+Z` restores it outright, and this
is the surface that tells them there is something to undo.

`None` when the annotation had no note, which is the ordinary case for
every shape this shell draws: a disclosure that fires on every save is one
nobody reads by the third time. Same rule as [`deleted_collateral`].

# The truncation, and why it is not a formatting decision

A `/Contents` may legitimately be a paragraph. The status line is one
bounded row that elides rather than wraps (`DEFECTS.md` R128), so a long
previous note would be cut by the *layout* with no indication that it had
been. Cutting it here, with an ellipsis and a stated character count, is the
difference between an operator seeing all of a short note and believing they
have seen all of a long one.

### `fn note_removed`

The same argument as [`note_replaced`] at its strongest — a removal leaves
the markup on the page looking exactly as it did — so this fires even for a
short note and never returns `None` for a note that had words.

It says the markup itself stayed, because that is the thing an operator
pressing a button labelled *Remove note* most reasonably fears they have
just done, and the canvas cannot answer it: a shape with a note and the same
shape without one are the same picture.

### `enum AnnotDeleteRefusal`

# The defect this closes, stated as it was found

`annotation_deletion_refusal` is a **pure query**. Its own doc comment names
this call site in as many words — *"safe to call every frame from a UI (R83:
ask before offering the control)"* — and until this landed **nothing in this
shell called it**. On a certified drawing the Format tab's Delete, the
canvas right-click's Delete and the Delete key were all live, and every one
of them ended in `crate::app::actions::apply::vector_edit`'s `Err` arm,
which wrote one line to the trace and **said nothing at all to the
operator** — it words one un-categorised sentence since O116 (2026-09-04),
which names no cause and so replaces none of these.
That is the identical shape the forms panel's `deletion_refusal`
audit found the day before (`crate::panels::properties::formfield`), one
annotation kind along, and it was found the same way: by asking what the
engine offers rather than by re-reading this shell.

# Why an enum rather than one sentence

Because the two reachable causes are **different facts about the operator's
file** and only one of them is about a signature. An encrypted drawing and a
certified one look identical on the canvas; telling an operator that a
signature forbids the delete when in fact the file is encrypted sends them
hunting for a signature that is not there. The mapping is
[`crate::panels::properties::annotdelete::refusal_for`], a total match
written in the engine's own guard order — exactly the shape
`crate::app::actions::xobject::refusal_for` has for `unshare_form`, and for
the same reason: a `_ =>` that silently swallows a mistyped variant name
turns an instruction back into a dead end and the compiler stays happy.

# What none of these sentences does is offer a remedy it cannot back

The forms twin ends *"the values in it can still be filled in and changed"*,
which is true and checkable: `fill_refusal` allows at `/P 2` where
`deletion_refusal` refuses, so there is a second verb to point at. **There is
no such second verb here.** §12.8.2.2 Table 254 puts annotation *creation,
deletion and modification* on one line, so at `/P 2` an operator cannot add a
comment either, and a sentence suggesting they could would be an invented
remedy of exactly the kind this project's copy rule forbids.

⇒ Each of these therefore explains **why** instead. That is what keeps a
refusal from reading as a dead end without claiming something the engine
would refuse the moment the operator tried it.

### `fn annot_delete_locked`

# Why this is a free function and not a fourth [`AnnotDeleteRefusal`]
variant

Because it comes from a different place and has a different scope. Every
member of that enum is derived from an `EditError` and describes the **whole
document**; this is a bit in the selected annotation's own flags word, and
two annotations on one page can disagree about it. Folding it in would make
`crate::panels::properties::annotdelete::refusal_for` — a total match over
`EditError` — answerable for a variant no `EditError` produces.

# It is the more actionable of the two facts, and that is why it wins

A certified document and a locked annotation can both be true at once, and
the gate checks this one **first**. An operator told *"this comment is marked
as one that should not be changed"* has somewhere to go: they can look at
that comment, ask whoever placed it, or select a different one. An operator
told *"the document is certified"* can do nothing about one annotation. When
both are true, the sentence that leaves the operator with a next step is the
one worth saying.

It says *"the file marks"*, not *"pdfcer will not"*. §12.5.3's `Locked` is a
statement the **producer** wrote into the annotation, and this shell honours
it rather than imposing it. Wording it as pdfcer's decision would send an
operator looking for a pdfcer setting to turn it off.

### `fn deletion_would_take`

# The future-tense twin of [`deleted_collateral`], and why they live together

They are one vocabulary in two tenses and they must not drift. A preview that
says *"1 reply will be left without the comment it replied to"* followed by a
disclosure that says *"1 grouped annotation is now on its own"* has described
two different acts, and the operator has no way to tell which of the two
lied. Keeping them adjacent in one file is the cheapest guard available — a
reader editing either one sees the other — and the counts they render come
from **one engine body**: `plan_annotation_deletion` is shared between
`annotation_deletion_preview` and `delete_annotation` precisely so that
*"the warning cannot disagree with the act."*

# Why this is worth showing at all, in the engine's own words


A reply three rows down a scrolled Comments list is not visible, and this
shell's delete carries no confirmation dialog by deliberate decision
(decision 024 §4.4's no-confirm carve-out, which is **conditioned on the
result being visible**). So the only moment this fact can reach the operator
is while the annotation is selected and before the key goes down, which is
where the panel puts it.

# It returns `None` far more often than not, and that is the design

The overwhelmingly common annotation has no pop-up, no replies and no group,
and there is nothing whatever to say about deleting it. A sentence that
appeared on every selection would be read the first three times and skipped
for ever after — which is the failure mode that makes the *interesting* case
invisible. R9: nothing to say renders nothing.

**It does not say "removed" and it does not mention the file.** Deleting
an annotation takes an entry out of `/Annots`; it does not touch page
content, and an incremental save leaves the previous revision in the file.
`docs/core-api/03-capabilities.md` §3.4 — *"delete is not redaction"* — and
[`deleted_collateral`] observes the same rule in its own wording, which is
the other half of why these two functions sit together.

### `enum ShapeWord`

A mapping and not a passthrough, and each row is a place the file's
vocabulary and the operator's disagree:

| `/Subtype` | what pdfcer's own ribbon calls it |
|---|---|
| `Square` | **rectangle** — the Rectangle tool draws it |
| `Circle` | **ellipse** — it is an ellipse inscribed in `/Rect`, not a circle |
| `Ink` | **freehand mark** — the Freehand tool draws it |
| `Highlight`/`Underline`/`StrikeOut`/`Squiggly` | **text mark** |

Showing "Square" to an operator who drew a rectangle is the surface
disagreeing with itself about what it just did, and it sends them looking
for a Square tool that does not exist.

[`ShapeWord::Other`] is the honest arm for a subtype this shell has never
heard of: the sentence it produces says *this kind of mark* rather than
inventing a name, because a wrong name is worse than a general one.

### `enum NodeEditRefusal`

`Copy` and fieldless-payloaded, because [`crate::app::status`]'s decline
store is `Copy` and its `line()` returns `&'static str`. That constraint is
the reason no sentence here quotes a *number* — a floor of 3 for a polygon
and 2 for a polyline would each need their own static string, and the
operator does not need the number to know what to do next. `VertexEditRefusal`
makes the identical choice for ce dimensions, and this enum is deliberately
its sibling rather than a reuse of it: the ce-dimension sentences say
*"measurement"*, which is the wrong word for a comment shape (R8b rule 15),
and one enum serving both would have to say something vague enough to be
true of either.

The engine offers a `reason: &'static str` on
`EditError::GeometryNotReshapable` and says a shell may show it verbatim.
It is deliberately not shown: those sentences name PDF keys and engine verbs
(*"author a PolyLine instead"*, *"use resize_annotation"*, *"/QuadPoints are
text-anchored quadrilaterals"*) and are written for a developer at a CLI.
They go to the trace, where that reader is. `canvas::annotnodes::refusal_for`
carries the argument at the mapping site.

### `fn line`

Each names what is true rather than what the engine called it, and each
names a **next act** where there is one — which is the rule
`resize_not_rebuildable` states and the reason the ce-dimension twin
gives: at the moment it is read the operator has just released a drag
and seen nothing happen, and what they need is what to do, not a
diagnosis.

### `fn no_nodes_line`

Every one of these says what the operator *can* do instead, because
each of them can do something: a rectangle and an ellipse resize, a
freehand mark moves and resizes as a whole, a line's two ends drag. A
sentence that only said "no" would leave them looking for a control that
does not exist.

**The freehand sentence changed meaning on 2026-09-09, and the old
one is recorded here so it is never written back.** Until `pdfcer-core`
`Pass 278.0` it read *"A freehand mark has no corners to edit — it is a
recorded pen stroke"*, on the engine's then-ruling that per-point ink
editing was refused on purpose. That ruling was overturned (*"parity
with Acrobat is this project's floor, not its ceiling"*) and a freehand
mark's points now drag, add and remove exactly like a polyline's. The
only way this arm is still reached is an `/Ink` whose `/InkList` the
engine could not read as an array — `Annotation::ink_list` is `None`,
`canvas::annotnodes::geometry` draws no anchors, and the sentence has to
describe **that** mark rather than freehand marks in general. A sentence
that said "has no corners to edit" after they became editable is the
stale-negative defect this project has shipped before.

### `fn measure_stale`

`ReshapeForecast::measure_not_recomputed` is `true` when the annotation
carries a `/Measure` dictionary (§12.9) whose number pdfcer did **not**
recompute. The engine's account of why it does not is the reason this
sentence exists rather than a silent fix:

> Acrobat recomputes the number and — a sourced user complaint — silently
> clobbers any manual override in doing so. pdfcer's markup bake draws no
> caption and reads no `/Measure`, so it neither recomputes nor clobbers.

⇒ The geometry moved and the text did not. Saying so is the whole of R8b
rule 4's honest half: an operator who is not told will read a number that
describes the shape before their drag.
