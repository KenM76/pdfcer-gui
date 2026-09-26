# `status::selected` — what is selected, said in words

One line at the left of the status bar, naming the thing the operator has
selected and — when it matters — how many other things were under the same
click.

## The state this exists to make legible

The operator:

> *"when I click on one of the objects all I get is the page selected."*

That report is precise. A file that wraps the whole visible body of the
sheet in a page-sized form XObject gives that form a bounding box which
wins every click at every point, and the engine does not enter one — so
what is selected really is a page-sized object.

Without this line **nothing on screen says so**. The selection outline is
drawn round the page edge, which looks exactly like *"the page is
selected"* — a state this program does not have. No other surface says
*"you have selected a Form containing 214 objects"*, which is a diagnosis,
and from a diagnosis the next question follows on its own.

This line does not fix the selecting. It makes the selecting **legible**,
which is what turns an unexplainable interface into a solvable one — and it
is the surface every refusal sentence is printed on.

## Why the left, and why it is the thing that yields

The right-hand cluster is fixed controls the operator reaches for — page,
zoom, fit, Find, the pick filter — and `status::fitting` may shed only two
of those, because the rest have no other home. This is a **readout**: it
costs nothing to lose, because everything it says is also visible in the
Objects panel and in the selection outline.

So it goes on the left with the other narration, where `egui`'s left-to-right
run gives up its space first, and it elides rather than pushing. That is
`status`'s own rule about what yields, applied to the newest thing on the
bar rather than exempting it.

## What it says, and what it refuses to say

| state | line |
|---|---|
| nothing selected | nothing at all |
| one object | its kind, and its size in points |
| one object, more underneath | `… · 1 of 5 here` |
| several objects | `3 objects selected` |

**Nothing when nothing is selected**, rather than *"Nothing selected"*.
A status bar that narrates the absence of a thing spends a permanent line on
the most common state in the program. A tutorial string there may well be
worth having, but that would be a decision about **teaching** and this line
is a decision about **reporting**. They should not be made at once and they
should not be made by the same code.

## Item notes

### `const RUNG_SLOT`

# Why a label's own words need a trace line at all

[`crate::diag::ui_rect`] publishes WHERE this label was drawn and never
WHAT it says. That is the right division for a layout oracle and it is
useless for a content one: a build that drew the readout and dropped the
rung clause publishes a byte-identical region, so a check asserting the
region is satisfied by both outcomes and measures neither.

⇒ So the clause states itself. The line is emitted from the same arm
that builds the clause, out of the same numbers, on the frame the label
is drawn — so a harness that sees this line AND the region on the same
frame has measured that the sentence exists and that the bar drew it.
Neither half alone says that, which is why a check should assert both.

**"From the same arm" is load-bearing.** An emission placed ABOVE the
`match` and keyed on the same `PartKind` the arms are keyed on reads as
equivalent and is not: falsification recipe (4) of the driven check —
replace both arms with `(line, None)` — leaves such a trace firing and
the check PASSING on a build that discloses nothing. It goes through
[`trace_rung`], called from the two producing arms and from nowhere else.

It is a trace of the DECISION, not a transcription of the string.
Echoing the rendered text would make every wording change a harness
change and would tempt a check into asserting English; `kind`, `part`,
`held` and `of` are the four facts the clause is computed from, and a
build that gets any of them wrong gets the sentence wrong too.

Routed through [`crate::diag::trace_changed`] rather than
[`crate::diag::trace`], because this is drawn sixty times a second and
a selection that is sitting still would otherwise bury the channel —
on `canvas-pointer` a stationary pointer writes fifty identical lines in
nine seconds. The de-duplication is on the
rendered line, so a harness must not assume one press produces one line:
an EARLIER gesture that produced the identical clause suppresses the
later one. A check wanting a before/after verdict asserts that the
count before its gesture was ZERO, rather than that a new line follows
a mark.

### `fn with_part`

Returns the line with a rung clause appended, and the hover that belongs
behind it — or the line unchanged and `None`.

# Why this reads the level rather than the entry's `subpath`

Both would work today. `SelectionLevel` is the **stated** answer, kept in
step by `normalise`, and `subpath: Some(_)` is the representation that
happens to carry it; a readout that inferred the rung from the
representation would be a second definition of what Part means. The
level is asked first and the index is read only once the level has said
there is one.

# A leaf produces no clause, and that is not a hole

The Part rung is unreachable inside a form XObject: `part_hits_of`
matches on a page-object index and returns nothing for a leaf, so the
ladder caps itself at the object rung there by construction. Requiring
`page_object_index` here is therefore an assertion of that fact rather
than a case being dropped — and if it ever stops being true, the clause
goes quiet rather than printing a total it computed from the wrong index
space.

# The total is re-read every frame

From the same `page_objects` cache the outline was drawn from, keyed on
`(page, edit_epoch)`. A reflow that changes how many runs the object has
changes this number on the next frame, which is the only behaviour that
keeps *1 line of 27* from becoming a claim about a document revision the
operator is no longer looking at.

### `fn trace_rung`

Called from the two arms of [`with_part`] that build a rung clause, and from
nowhere else — which is the property the driven check depends on, and the
reason this is a function rather than four lines repeated twice: a second
call site added anywhere would be visible here, and a reader who wants to
know what can emit this line has one place to look.

Routed through [`crate::diag::trace_changed`] because the status bar is
built sixty times a second and an unconditional trace would write fifty
identical lines in nine seconds — the `canvas-pointer` lesson. The
de-duplication is keyed on the RENDERED line, which has one consequence a
harness must honour: a check wanting a before/after verdict asserts that the
count BEFORE its gesture was zero, rather than that a new line follows a
mark. A line identical to one already written is suppressed, and a
mark-relative assertion would read that suppression as the feature being
broken.

`held=` is the size of the set, and it is a separate field from `part=`
rather than a plural spelling of it: a check asserting the set survived the
press that began its drag needs a number it can compare, and `part=` names
only the first entry.

### `fn with_layer`

# Why this is on the status bar and not only in the Layers panel

**The canvas is the primary surface, never a panel.** `pdfcer-core`
answers *"which layer is this object on"* through `VectorObject::oc()`, so
clicking the object has to be able to *reach* that answer. A capability
whose only route is a panel is a capability the operator must already know
exists before they can use it — and this one is asked for as *"selecting
an object highlights that layer"*, which is a sentence about **clicking**,
not about a panel.

The panel is the supplement: it is where the answer can be acted on, by
switching the layer off. This is where it can be *seen*, with nothing open.

# Rule 4: nothing is drawn on the drawing

No badge, tint, dashed outline or provisional layer is painted over the
selected content to express its membership. The selection handles are the
cursor and are untouched. **Render normally; report separately. Both.**

# Silent on a document with no optional content, which is nearly all of
them

The engine measures **0.6 %** of a 500-file corpus as carrying optional
content at all. On the other 99.4 % *"which layer"* is not a question the
operator has, and `not on a layer` after every single click would be a
permanent line about a feature the document does not use — the same fault
as narrating "Nothing selected", which this module's header refuses for
the same reason.

So the clause appears only once `read_layers` says the document declares
groups. That read walks `/OCProperties` and its `/OCGs` array — a handful
of dictionary lookups, no content stream — and it is behind the
`targets.first()` guard above, so it costs nothing on a frame with nothing
selected. The Layers panel makes the same call every frame it is open.
