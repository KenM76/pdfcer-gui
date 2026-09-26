# `text::dimension_groups` — the words the Manage-groups window shows

## Rule 15, and why this catalog says "the dimensions you draw"

A **ce dimension** is one pdfcer authors. A **pdf dimension** is CAD-exported
page content pdfcer reads and must not silently alter. The distinction is
ours, not the operator's, and it has already sent one investigation down the
wrong path — so this catalog does what [`crate::text::scale`] does and avoids
the bare word entirely. On screen it is *"the dimensions you draw"*, which is
unambiguous without asking a drafter to learn a term that exists for our
benefit.

## The hardest thing this window has to explain

Not what a group *is* — a drafter already has that idea from every CAD
package they have used. What is genuinely new is that **a group edit reaches
backwards**: setting a scale, a standard or an appearance default rewrites
every dimension already placed in that group, wherever it is, including on
pages that are not on screen.

The operator named the fear himself, quoted in the ui-spec: *"cannot change
one and be surprised 40 others changed or didn't."* So every group-level
control in this window is accompanied by a **count of what will move**, and
the count is computed before the edit rather than reported after it.

## And the number in that count is NOT the engine's return value

`EditSession::set_group_style` returns the number of members **regenerated**,
which is every wired member — including the ones that override the property
being changed, because regenerating an overrider is byte-identical and free
in the diff. Showing that number would be worse than showing none: it is a
real number, plausibly labelled, answering a different question, and an
operator who reads *"40 dimensions will change"* and sees three change has
been misled by a fact.

[`members_that_will_move`] therefore takes the count the *caller* computed
from `StyleProvenance::follows_group()`, and the caller computes it before
the edit is applied. See `dialogs::dimension_groups::style`.

## Item notes

### `fn the_no_scale_disclosure_is_the_engines_own_words`

Asserted against the constant rather than against a literal, which is
the difference between a test that pins the *relation* and one that pins
two copies of a magnitude. `NO_SURFACE.md` records the day a test in
this crate asserted a literal triple against a function returning the
literal triple and could therefore never fail; this is the shape that
does fail if somebody paraphrases.

### `fn the_moving_count_says_which_kind_of_zero_it_is`

The one that matters is `(0, n)`: an operator pressing a control that
will change nothing on screen, because every member overrides the
property. Reporting "0" as a bare number would read as a failure; the
sentence says which of the two zeroes it is.

### `fn identity_heading`

Not "Name". The section carries **both** verbs that act on the group's
identity, and a fold captioned "Name" would read as a text field — which is
exactly what it looks like when folded shut, with Delete hidden inside it.
A caption is a promise about what is under it and this one has to name the
destructive half.

### `fn scale_heading`

One caption for both, because `set_group_scale` takes the scale and the
number format together: an operator changing the unit is calling the verb
that also carries the scale, and two folds would hide that from them.

### `fn intro`

Says what a group *carries*, because that is the fact every control below
depends on and the one nothing else on screen states. The second sentence is
the reach-backwards disclosure in its shortest honest form; the per-control
counts make it concrete.

### `fn draw_into_hint`

This is the control the operator asked for by name and could not find —
*"I still can't get to edit dimension groups when I click on it."* The
authoring group was fixed at the default for the whole life of the build
before this window, so a second group could be created from nowhere and
joined by nothing.

### `fn scale_phrase`

The `NeverSet` arm renders `pdfcer_core::dimension::NO_SCALE_DISCLOSURE`
**verbatim**. That string lives in the engine precisely so shells cannot
invent their own wording for it, and
`docs/core-api/03-capabilities.md` §1.5 obligation 2 requires it be shown
rather than paraphrased. Do not "improve" it here.

### `fn standard_hint`

Deliberately does **not** claim conformance. `pdfcer-core`'s own
`DimStandard` doc applies the same discipline — pdfcer draws *ISO-style*,
never *ISO 129-1 conformant*, because the standard is paywalled and was not
obtained. A window that promised conformance would be making a claim the
engine explicitly declines to make.

### `fn layer_hint`

It is not `View ▸ Layers`. That one changes what *this window* draws and
nothing a save would write; this one writes the group's default visibility
into the document's optional-content configuration, so it is what the file
tells the next reader — in any viewer that honours optional content.

### `fn layer_default_group`

R9: an affordance that cannot be honoured is not drawn. The engine refuses
to hide the default group, so the control is absent and this sentence says
why — an omission with no explanation reads as a bug.

### `fn new_needs_a_name`

Greying with an explanation, not silence: this is the *temporarily*
unavailable case R9 reserves greying for, and the reason is one the operator
can act on in one keystroke.

### `fn delete_default_group`

R9 again, and the same shape as the layer switch above it: the engine
refuses, so the control is **absent** rather than offered and declined. The
sentence is what stops the omission reading as a bug.

### `fn delete_needs_a_home`

The engine refuses by default and puts the **count** in the refusal, and its
reply says why in a line worth keeping: *"this group is not empty"* and
*"this group holds forty dimensions"* prompt different decisions, and only a
surface can put that question in front of an operator.

So this is the question, with the number in it. The two answers below are
the only two the engine offers — and the third an operator might expect,
*delete the dimensions too*, is deliberately absent from the engine and is
therefore absent here. Saying so is [`delete_cannot_remove_members`]'s job.

### `fn delete_move_changes_labels`

Not a warning — a fact, and the one an operator would otherwise discover by
reading a drawing. A ce dimension's label is derived from its group's scale,
unit and number format, so members arriving in a different group are
**re-measured** and print different numbers. The engine's own measured
example: `70.6 mm` in a 1:1 millimetre group becomes `2.00 m` in a metre
group at 1 cm per point. Same geometry, different group, correctly different
label.

### `fn delete_cannot_remove_members`

Stated because it is the answer an operator may be reaching for, and its
absence is a decision on the engine's side with a reason worth passing on
rather than a gap. Deleting a ce dimension also removes its annotation from
the page, so doing it inside the group verb would be a second implementation
of that removal — and looping the existing one would make undoing a group
deletion take one press per member and be able to stop halfway.

### `fn unit_hint`

It goes through `set_group_scale`, because a unit lives inside the group's
`NumberFormat` and there is no narrower verb — the engine's reply called
that *"a discoverability problem, not a missing capability"*, and this
sentence is the discoverability half.

The consequence is real and is the reason the sentence exists: every member
is re-formatted and its appearance regenerated, exactly as a recalibration
does. An operator who expects a unit change to be cosmetic is expecting the
wrong thing.

### `fn set_by_group`

**The checkbox IS the `Option`.** `GroupStyle`'s seven fields are each an
`Option`: clear means *this group has not spoken, use the factory value*,
ticked means *this group says this*. Rendering the tick as "set by this
group" rather than as "enabled" is what keeps the two states legible —
"enabled" would imply the property is off when unticked, and it is not, it
is inherited.

### `fn members_that_will_move`

`moving` is computed by the caller from `StyleProvenance::follows_group()`
over the group's members, **before** the edit. It is deliberately not the
engine's returned count — see this module's header for why that number
answers a different question.

`total` is stated beside it so the difference is visible rather than
implied: *"3 of 40"* tells an operator that thirty-seven members have their
own value, which is the fact that stops the change being a surprise in
either direction.

### `fn prop_text_height`

**One vocabulary, shared with the CLI.** `pdfcer group-style` uses
`text-height`, `line-width`, `arrow-length`, `arrow-form`, `color`,
`tolerance` and `tolerance-places` for these same seven, and the ui-spec's
Amendment B §B.5 names the hazard of diverging: *"a panel using different
words for the same nine things is how an operator ends up unable to script
what he just clicked."* These are the same words, capitalised for a label
and spelled in the operator's own English where the flag is hyphenated.

### `fn points_value`

In the catalog rather than formatted at the call site because it is
operator-visible text — and because the unit belongs beside the number in
exactly one place. `10` and `10 pt` are different claims.

### `fn points_suffix`

Points, and said so, because a text height of `10` is meaningless without it
and because these are the one place in this window where the number is in
**paper** units rather than in the group's own unit — a dimension's text is
10 pt tall whatever the drawing is scaled at.
