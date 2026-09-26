# `pdfcer-gui/text/forms/tab_order`

## Item notes

### `fn tab_order_explainer`

Three load-bearing clauses: what the order *is*; that it is also **paint**
order, the fact that would make a future reorder consequential and that
nothing else on screen could tell you; and that this view changes nothing —
said in prose, because the alternative is a disabled control (`RIBBON_IA.md`
P3 forbids one).

### `fn tab_order_empty`

Distinct from the panel's own empty states: this is reachable on a document
that *has* an `/AcroForm` full of fields, none of whose widgets any page
lists. Silence would read as a broken section rather than a fact.

### `fn tab_order_row`

`label` is the field's `/TU` when non-blank and its fully-qualified name
otherwise — the fill rows' preference, so the operator reads the string an
assistive technology speaks. The raw name is in [`form_field_row_tooltip`].

### `fn tab_order_row_where`

Always drawn, including for a single-widget field. A field with widgets on
several pages **appears more than once** — correct rather than a duplicate,
because tab order is per page and a field is document-level — and a row that
named the widget only when there happened to be more than one would leave
the operator working out which case they were looking at.

### `fn tab_order_no_tabs_entry`

**Reported as absent, and given no mode name** — not "manual", not
"unspecified". `D:\Dev\pdfcer`'s roadmap records what Acrobat's "Unspecified"
tab-order state mechanically denotes as **unsourced after two attempts**, so
a label here would assert what nobody has been able to support. What is said
instead is what the file says, plus the operationally useful half.

### `fn tab_order_tabs_on_ancestor`

The correction argued in `crate::panels::forms::tab_order::model`'s §4:
ISO 32000-2 Table 31 marks `Rotate` "(Optional; inheritable)" and `Tabs`
merely "(Optional; PDF 1.5)", and the table's preamble makes every unmarked
attribute non-inheritable.

Both halves have to be said. Treating the ancestor's value as the page's own
asserts an inheritance the standard denies; saying "no /Tabs" over a file
that plainly has one two levels up hides a fact that changes what another
viewer might do.

### `fn tab_order_unclaimed`

The old wording finished *"if the form declares entries pdfcer could not
read, these may be theirs"* — a speculation offered because there was
nothing better to offer. Nothing could be done with an unclaimed widget, so
the only honest thing left was to speculate about where it came from.


The inline-field-roots note is still one line away in
[`forms_inline_field_roots_note`] and is still not re-counted here — two
numbers about two things, related out loud rather than added together.

# Why it keeps its warning glyph now that it has a remedy

Because the remedy is a **chore the operator has not done yet**, not a
reassurance. Until they press Register, the boxes on the page in front of
them still cannot be filled, and that is a warning whether or not a button
sits under it. `RIBBON_IA.md` R84 also requires the glyph independently: the
sentence is drawn in `warn_fg_color`, and a warning carried by colour alone
is invisible to a reader who cannot distinguish it.

# Why it says "cannot be filled" rather than "are broken"

Because that is the operator-visible fact and it is the one that surprises.
The box **draws**. It has a border, it has a background, it looks exactly
like the field beside it. What it does not have is a name any filling verb
can address, so clicking it and typing produces nothing and no message.
This project's own recurring failure — a visible control that is silently
inert — arriving through a document rather than through a ribbon.

### `fn tab_order_unclaimed_row`

The position is what an operator uses to **find** it — press Tab that many
times and watch the focus ring land — which is the only handle they have,
because the thing has no name by definition. That is also why the row is
worth drawing at all rather than leaving the heading's count to stand: a
count cannot be pressed, and it cannot be pointed at either.

### `fn tab_order_register_name_hint`

Says what an empty box means, because empty is the common and correct
answer and a blank field with no hint reads as "required".

Most unclaimed widgets are **merged field-widgets** (§12.7.3.1): one
dictionary serving as both, carrying its own `/T`, `/FT` and `/V`. The
engine measured a real form and found 11 of 13 in that shape. For those,
registering with no name recovers the field exactly as it was — the name is
already in the file, and typing one would *override* it rather than supply
something missing.

### `fn tab_order_other_annots`

The one that stops the numbering reading as wrong. §12.5.1's tab order is
over **annotations**, not form fields: a link occupies a position in the
sequence, so a list of widgets is the fields *in* it, not the sequence.

### `fn tab_order_register_as`

# The name is in the FILE and not on screen, which is the whole point

A merged field-widget (SS12.7.3.1) carries its own `/T`. Registering it with
a blank name box recovers that name — a string the operator has never seen,
because nothing in the panel could show it: the widget belongs to no field,
so no field row names it.

The engine put it exactly right when the preview shipped: *"Register as
`Address`" is a decision; "Register" is a guess.*

### `fn tab_order_register_needs_a_name`

Says what typing a name will **produce**, not that one is required. The
distinction is the whole of it: this box was a bare kid, its name, field
type, radio flags and value all lived in a dictionary that is not in this
document, and a name typed here **creates a new field**. It does not recover
the old one.

### `fn tab_order_register_name_taken`

The short form of [`crate::text::status::adopt_declined_name_taken`], which
is the sentence shown after a press. Both exist because they are read at
different moments: this one while the operator is still typing, that one
after they have committed to a name.

### `fn tab_order_register_name_is_a_path`

# This hover exists because a guard moved, not because a rule changed

`FormAuthorError::DottedPartialName` was described in this shell as
reachable from this surface *after a press*, on the grounds that this box
is free text gated only on non-empty. True of the shell's own gate — and
beside the point, because the engine put `reject_dotted_partial` inside
`adopt_plan`, which [`pdfcer_core::edit::EditSession::adopt_preview`]
shares. The refusal therefore arrives in the preview, the button greys, and
the press never happens.

So the sentence the operator would have read on the status bar had to
become a hover, and without this it fell into
[`tab_order_register_unavailable`] — which says the reason is not one this
panel expects. The rule was being enforced correctly and the operator was
being told the program was confused.

The long form is [`crate::text::fieldclip::name_is_a_path`], which spells
out that the field would be clickable and never fillable. This one is read
while the operator is still looking at the box, so it states the rule and
the remedy and stops.

### `fn tab_order_register_name_has_a_bare_dot`

`a..b`, `.x`, `x.` — `FormAuthorError::EmptyNameSegment`, a period that
starts, ends or doubles up and so names a level with nothing in it.

Worth a sentence of its own rather than folding into
[`tab_order_register_name_is_a_path`], because the remedies differ: there,
remove the dot; here, remove it **or** put a name beside it. Telling an
operator who typed `Address..City` to use no dots would be a true sentence
that solves a problem he does not have.

### `fn tab_order_register_unavailable`

The catch-all for refusals this surface believes are unreachable. It says
the control is unavailable and does not guess why — a wrong reason is worse
than none, and by construction reaching here means the listing and the
engine disagree about what this widget is, which is a fault to find in the
trace rather than a chore to hand to an operator.

### `fn tab_order_register_no_type`

# The disclosure that would otherwise arrive too late

`/FT` is inheritable. A widget that was a field's kid could inherit its
type; once it is registered as a **top-level** field there is nothing left
above it, so it has no type at all and no viewer knows how to render or fill
it.

The registration still succeeds. So without this the operator presses a
button, is told it worked, and has a box that still cannot be filled — which
is rule 4's case exactly: an inference they cannot see, owed a sentence
precisely because nothing on the page looks wrong.

Said **before** the press now that `adopt_preview` makes it knowable.
Telling somebody afterwards tells them their successful action did not do
what they wanted.

### `fn tab_order_unclaimed_row_named`

# Why the name is here and not on the button

A label wraps to the pane width; a button does not. These rows live in a
dock panel about 314 pt wide, and a button reading *"Register as
CustomerName"* beside a name box is wider than that — it runs off the
right-hand edge and takes the next row down with it.

What the pre-flight was asked for is that the name be **visible before the
press**, not that it be printed on the control. On the line immediately
above the button it is both visible and wrappable.

### `fn reorder_moved_non_widgets`

The disclosure the operator would never predict. `/Annots` order is
**paint order** as well as tab order, so arranging a tab sequence can change
which annotation is drawn on top where two overlap. They asked to reorder a
list of fields and got a z-order change; the sentence says so in their terms
rather than in the file's.

### `fn reorder_pinned`

A list that did not fully take, said rather than discovered. These are
entries written into the page as direct dictionaries: they have no object id
to be named by, so they stay where they are and the rest flow around them.
Rare, and produced by a handful of writers.

### `fn reorder_copied_shared_array`

Nothing is wrong and nothing is lost. It is a structural change to the file
the operator did not ask for, which is the whole reason it is disclosed: this
project's rule is that a side effect they cannot see still owes a sentence
off-canvas.
