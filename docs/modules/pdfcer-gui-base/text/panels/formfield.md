# `pdfcer-gui-base/text/panels/formfield`

## Item notes

### `fn the_limitation_note_never_advises_deleting_the_field`

The test it replaces asserted the opposite — it required the string
`"delete this field"` to be present, on the reasoning that *"a note that
only said 'cannot be changed' would leave the operator stuck"*. That
reasoning was sound and its premise was false: the capability existed,
so the operator was not stuck, and the test was pinning a sentence that
recommended destroying a field's name, value and tab position for
nothing.

A test can pin a sentence and cannot know whether the sentence is
true. This one is written in the negative for that reason: it does not
try to say what the note should claim, only that it must not send an
operator down the destructive route again.

### `fn a_typeless_field_is_described_as_unfillable`

A `/FT`-less field is what a bare kid that lost its `/Parent` becomes,
and no viewer can fill it. "Unknown" would read as pdfcer failing to
look; this says what is true of the document.

### `fn page_number`

1-based, because that is what the page strip shows and what the operator
would say out loud. Every 0-based index in this shell stops at the boundary
with the person using it.

### `fn box_count`

Shown only when there is more than one, and it is a **disclosure** rather
than a statistic: a field drawn in three places can be changed from three
pages, and the two the operator is not looking at change with it. Nothing on
the page says so.

### `fn field_type`

A `/Btn` is three different controls and the spec tells them apart by
flag bits, not by type. Reporting all three as "button" would be accurate
about the format and useless to a person: a check box and a push button have
nothing in common from where they sit.

### `fn field_flags`

Only the ones that are **set**. A list of every flag with yes/no beside it
would be six rows of "No" on a typical field, and the reader has to search
it to learn anything. Naming the exceptions is what a person would do.

### `fn rename_label`

It asks for the **short** name and says so, because `rename_field` takes a
partial name and rebuilds the qualified one from the parent chain. An
operator who copied `Address.Line1` out of the row above and pasted it here
would author a `/T` containing a dot, which nothing can address again.

### `fn rename_refused`

# Drawn INSTEAD of the control, not under it

This is the sentence R9 asks for when a capability is refused *permanently
for this document*: the box and the button are not drawn, and this line
takes their place. A greyed box would say the state is temporary — a
certification signature is not — and would hide its own explanation behind a
hover.

It names **renaming**, not "editing", and that is deliberate. The pane
around it still offers seven editable properties, because a certified form
at `/P 2` permits filling and forbids restructuring (§12.8.2.2 Table 257). A
sentence saying "this document cannot be changed" would be false about the
controls directly below it, and an operator who believed it would stop
trying things that work.

It does not name the *cause* by name. `rename_refusal` answers for
encryption as well as certification, and this shell has already shipped one
structural-refusal string that names certification unconditionally — which
is silently wrong on an encrypted file and sends the operator looking for a
signature that is not there. Here the sentence says what is true of both:
the document forbids it.

### `fn delete_refused`

One sentence for both buttons, because both are refused by one gate:
deleting a field and deleting one of its boxes are both *structural* changes
to the form, which is precisely what a certification signature exists to
freeze. Two sentences saying the same thing twice, one above the other,
would read as two separate problems.

It says what the operator can do **instead**, and the answer is not
"delete it anyway" — it is that the form's structure is fixed and its
contents are not. Without that clause the sentence is a dead end, and a dead
end in a properties pane reads as a broken program rather than as a
protected document.

### `fn rename_disabled`

# Why this takes the engine's refusal rather than a boolean


That sentence was **wrong for the commonest case**. A freshly selected
field opens this panel with an empty box; hovering the greyed button then
named a rule about periods the operator had not broken, which reads as the
program having misunderstood what he typed rather than as a blank field.

Since `pdfcer-core` `7a0a9c2` the rule is askable —
[`pdfcer_core::forms_author::validate_partial_name`] — so the panel asks it
and hands the answer here. The wording is then chosen by the variant the
engine raised, which means this function cannot describe a refusal
different from the one about to happen.

And the rule may grow a clause without this going stale in the dangerous
direction. A new variant lands in the catch-all, which says the name cannot
be used and does not guess why — unhelpful, but true. The deleted model
would have kept greying the old set and let the new refusal arrive on the
status bar after the commit instead of on hover before it.

# The wordings

Each names the rule and the remedy, and none cites a clause number: the
operator is a draughtsman, and §12.7.3.2 is for this comment. The dotted
case is the short form of [`crate::text::fieldclip::name_is_a_path`], which
is the sentence shown after a commit that got through — both exist because
they are read at different moments, one while typing and one after.

### `fn delete_field_hover`

Names the count in the hover rather than after the fact, because that is
where it can still change the operator's mind. A confirmation that said
"deleted from 3 pages" afterwards is a report; this is a warning.

### `fn not_editable_note`

It read:

> ~~Required, read-only, the tooltip and the border can only be set when a
> field is placed. To change one, delete this field and place a new one.~~


The function is **kept and rewritten** rather than deleted, because there is
still something true to say in the same place: the properties that remain
out of reach are the *widget*-scoped ones — the box, the border, where it is
visible — and an operator who has just found six editable flags will
reasonably wonder where the seventh is. An absence with no explanation is
indistinguishable from an oversight; that was the right instinct in the old
sentence and it is the only part of it that survives.

It names no remedy now, because there is no honest one. Delete-and-replace
still "works" for a border and it is not advice this program should give.

### `fn editable_heading`

*"Properties"*, not *"Editable properties"*. The section directly above it
is headed with the facts that are genuinely read-only, and a heading that
advertised editability would invite the question of why the other section is
not editable — which is a fact about the file (a name, a type, a page) and
not a limitation.

### `fn label_tooltip`

*"Tooltip"* is the word the standard's own name (`/TU`, "alternate field
name") does not use and every application does. It is also what a screen
reader announces, which is the fact the hover carries.

### `fn label_default_value`

*"Default value"* is the term Acrobat's field properties uses and the one
the standard uses (§12.7.3.1, *default value*), so there is nothing to
invent here. The label says what it is; the hover says what it is **for**,
because the connection between this box and the Reset button is the part
an operator has no way to guess.

### `fn label_default_value_hover`

The hover carries the fact the label cannot: **this is what Reset
restores**, and a field with no default is emptied by it.


⚠ It does **not** promise the field is currently filled in with this value.
`/V` and `/DV` are separate: a field can show one thing and reset to
another, which is the whole point of having both.

### `fn label_alignment`

*"Alignment"* is what Acrobat's field properties calls it and what every
word processor calls it. The standard's own term is *quadding*, which is a
typesetter's word and appears nowhere an operator would look.

### `fn quadding_name`

# Why these words and not the enum's

`Quadding::Center` is spelled the American way because the standard is; this
shell writes British English everywhere else an operator reads. Naming the
variant would leak a spelling decision made by ISO into a form-properties
pane, so the operator-facing word is chosen here and the enum keeps its own.

*"Left"* rather than *"Left (default)"*. Table 222 does fix `0` as the
default, but a chooser that annotates one option is making a claim about the
document — and a field whose `/Q` is explicitly `0` and one with no `/Q` at
all are both *left*, which is the only thing this control can honestly say.

### `fn widget_heading`

*"This box"*, not *"Widget"*. A widget annotation is what the file calls
it and is a word no operator has any use for; what they are looking at is a
rectangle on a page. The distinction the heading has to carry is not the
spec's vocabulary but the **scope** — that these properties belong to this
one rectangle and the ones above belong to the field — and
[`widget_scope_note`] says that in the one state where it is visible.

### `fn widget_scope_note`

**The one sentence that makes the field/widget split legible**, and it
is deliberately conditional. On a one-widget field — the overwhelming
majority — there is no distinction to explain and the sentence would be
noise. On a radio group it is the difference between changing one button and
changing the answer, which is exactly the state where an operator would
otherwise expect this section to behave like the one above it.

### `fn label_widget_x`

The four are labelled X / Y / Width / Height rather than with the
standard's `/Rect` corners, because a corner pair is a spelling and a
position-and-size is what an operator is thinking about. `super::geometry`
made the same call for page objects and this matches it, so the two
surfaces read the same way.

### `fn widget_apply_hover`

Moving and resizing are the same gesture on this pane and different acts on
the file: a pure translation moves the baked artwork exactly and for
nothing, while a changed extent makes §12.5.5's algorithm *scale* it, so a
text field made twice as wide is redrawn rather than given room for more
text. An operator who expected the second and got the first — or the other
way round — has been surprised by something the program knew in advance.

### `fn widget_apply_disabled`

R9: greying is for a **temporarily** unavailable capability and must be
explained on hover. The capability is present and the operand — a number
the operator has changed — is not.

### `fn label_border_colour`

**"Border and mark", not "Border"** — O202 decision 2. This is the ink a
check box's tick and a radio button's dot are drawn in as well as the
outline, because `/MK` carries no third colour for the mark and the
engine's own appearance builders read `/BC` for both. A label naming only
the outline would mis-state what the swatch does on the two kinds where it
matters most.

### `fn colour_mark_unstated`

The same em dash the properties grid already uses for *no value*, and for
the reason that function's own doc comment gives: every property grid in
this class shows a dash for a field the document is silent about.

### `fn colour_mark_no_colour`

A word rather than a second dash. *The file is silent* and *the file says
there is no colour* are different facts and the whole reason `/MK`'s
colours are modelled as `Option<MkColor>` with an `MkColor::None` inside;
two controls showing the same glyph for both would throw that away on the
surface the operator actually reads.

### `fn background_remove_entry`

Worded as what the operator gets back rather than as what is deleted. The
two entries above it both leave the key present; this one is the only route
to the state a box has before anyone has touched its colour.

### `fn border_colour_remove_entry`

The border row offers this and no *no colour* twin, because the engine
draws an empty `/BC` and an absent `/BC` in the same black — so the only
change worth a press is the one that ends with the key gone.

### `fn border_colour_no_colour_note`

It says the box is **still drawn black**, which is the opposite of what
the phrase "no colour" suggests and is what the engine does. A box with no
border says so through a border width of 0.

### `fn colour_cmyk_mark`

**Not a converted approximation**, O202 decision 4. pdfcer owns no
rendering intent for a widget's chrome, so converting DeviceCMYK to
something a swatch could show would put a colour on screen that the file
does not contain — and the operator's first nudge of the picker would
commit pdfcer's guess at their separation as though it were their own.

### `fn border_unstated`

**Not "Solid, 1 pt".** `BorderSpec::default()` is solid/1 pt because that
reproduces the bytes pdfcer authors, which is correct for a writer and a lie
from a reader. The engine made the field an `Option` specifically so this
distinction survives, and their load-bearing test —
`a_widget_whose_file_states_no_border_reads_a_dash_not_a_default` — goes red
if the reader substitutes.

Distinct from a border of **width 0**, which Table 166 states as a value
meaning *no border*. That is the file saying something definite and it reads
as `0 pt`, not as this. Collapsing the two would tell an operator the file
is silent when it has spoken.

### `fn border_style_label`

Beveled and Inset are named by their **appearance** rather than by the
standard's word, because *"beveled"* describes a 3-D raised edge that a
person recognises on sight and cannot name, while *"inset"* is the same edge
the other way up. The other three need no help.

### `fn visibility_label`

Named by **where you see it** rather than by the flag combination, which
is the only framing that answers the question an operator is asking. `/F`'s
four settable combinations are Hidden, Print, NoView and their pairings;
*"on screen and on paper"* is what those mean.

### `fn visibility_unmappable`

The engine's mapping is **exact-or-`None`**, never nearest, and their note
on why is the same argument as the border's: `/F` admits dozens of
combinations and `Visibility` is the four pdfcer can write, so a file
carrying `Print | NoZoom` has no nearest of the four that is not a lie.

`None` here can never mean *absent*: Table 164 makes an absent `/F` equal
to `0`, which **is** one of the four. So this sentence is always about a
file that has said something pdfcer cannot express, and it says exactly that
rather than showing nothing.

### `fn widget_rotation_label`

`Widget::rotation` is `Option<i64>`, and the distinction is the same one
`Widget::border`'s docs call *"a fact to display, not a value to
substitute"*: `None` means **the file states none**, `Some(0)` means the
file says zero. They render identically and they are different facts, and an
operator debugging why a box looks wrong in another viewer wants to know
which their file carries.

### `fn widget_rotate_left`

*Left* and *right*, never *clockwise* and *anticlockwise*, and never a
signed number. `/MK /R` is counterclockwise while the page's `/Rotate` is
clockwise, and the standard's two sentences differ by exactly one word — so
a label that named a direction convention would be asking the operator to
hold the trap that caught the engine's own reviewers. *Left* is what they
watch the box do.

### `fn widget_rotation_hint`

It says the box stays put, because that is the surprise: `/MK /R` turns
what is drawn INSIDE the rectangle and leaves the rectangle itself alone
(§12.5.5 maps the appearance's `/BBox` into `/Rect`). An operator expecting
a tall box to become a wide one needs telling once.

### `fn widget_rotation_stale`

Not a failure: the file is correct and carries the new angle. The baked
appearance could not be regenerated, so the box keeps drawing at its old
orientation until something regenerates it — and an operator watching a box
refuse to turn is owed the reason rather than a mystery.

### `fn label_text_colour`

**"Text colour", not "Colour"** — it sits directly under Background and
Border and mark, and a bare "Colour" beside those two is the one label an
operator could reasonably read as a third property of the box.

### `fn text_font_embedded`

Shown verbatim rather than prettified: it is not one of the fourteen, so
this shell has no operator-facing name for it and the file's own key is the
only honest thing to write. A friendly name invented here would be a guess
presented as a fact about the document.

### `fn text_font_name`

The vocabulary rule this file's header sets, applied to font names: the
standard spells them `Helvetica-BoldOblique` and `Times-Roman`, and no font
menu this operator has ever used writes them that way. **Oblique is offered
as Italic** for the same reason — it is the word in every other program's
menu, and the two faces the distinction separates are not both on offer.

### `fn text_colour_unshowable`

R9 applied to a colour: the control is **absent**, not greyed, and this
sentence stands where it would have been. Converting the separation to
something showable would put a colour on screen the file does not contain,
and the operator's first nudge of the picker would commit pdfcer's guess as
though it were their own — O202 decision 4, which this row inherits from the
`/MK` rows above it.
