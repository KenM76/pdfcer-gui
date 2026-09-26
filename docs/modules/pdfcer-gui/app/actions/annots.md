# `pdfcer-gui/app/actions/annots`

## Item notes

### `fn rect_rule_token`

# Why not `{:?}`

Because a `Debug` rendering is a formatting detail of somebody else's enum
and a driven check that greps for `rect_derived=Artwork` would go quiet the
day `RectDerivation` gains a field, gets renamed, or has its derive removed
— quietly, and in the direction that reads as *the feature stopped
happening*. A `{:?}` on a tuple is how a check comes to report the
opposite of the truth while quoting the truth in its own message.

`RectDerivation` is `#[non_exhaustive]`, so the match **must** carry a
wildcard and a rule this build does not know cannot be a compile error
here. The wildcard says `other` — an honest *"this build does not know
that one"* rather than a guess at which rule ran.

The tokens match what the engine's own CLI prints (`rect_derived=`), so a
trace here and a `pdfcer` command line can be compared without a lookup
table.

### `fn refusal_for`

# One function over every rotation verb, and it is the compiler's job to
keep it complete

[`rotate`], [`set_rotation`] and [`rotate_dimension`] all refuse from the
same short list, so a second copy of this mapping would be a second place
for a variant to be forgotten — and a forgotten variant here is a grip that
is dragged, released, and does nothing with no explanation.

# Why the fallback is a sentence rather than the error's `Display`

`tools/gates/check-ui-strings.sh`' exclusion 3 names the failure in as many
words: a `format!` of an `EditError` routes **diagnostic prose into the UI**.
[`crate::text::rotating::RotateRefusal::Other`] is a hand-written sentence
that says the one thing an operator needs about an unrecognised refusal —
*the page is exactly as it was* — and names no cause it does not know.

### `fn delete`

Reached from `format.delete` and from the canvas's Delete key, both only
while an annotation is selected.

# Why it goes through `vector_edit` like everything else

So the undo entry, the epoch bump, the cache invalidation and the
disclosure happen the one way they happen for every other document change.
The closure returns the disclosure list, which is where the **collateral**
goes: the operator named one annotation and the engine may legitimately
have removed or altered more — a `/Popup` companion (§12.5.6.14 is a
`shall`), replies orphaned, group members promoted.

# `page` is for the message, not for the verb

`delete_annotation` finds the annotation by id wherever it lives, and it
has to: a reply may sit on a different page from the comment it replies to,
so a page-scoped delete would miss it.

# This is not redaction

It removes an entry from `/Annots`. It does not touch page content, and an
incremental save leaves the previous revision in the file.
`docs/core-api/03-capabilities.md` §3.4 states that rule, and
[`crate::text::markup::deleted_collateral`] observes it in the wording it
chooses — never "removed".

### `fn move_annot`

Reached from `canvas::annotdrag` on the release of a drag, and from nothing
else.

# The disclosure is about the half the canvas cannot show

A move writes `/Rect` *and* the absolute-coordinate geometry keys, and the
canvas renders from the appearance stream, so the operator sees the same
picture whether one half was written or both. There is therefore nothing to
disclose about the move having worked -- they can see that.

What they cannot see is the **pop-up left behind**. §12.5.6.14 makes a
pop-up a separate annotation with its own placement and leaves whether it
follows to the reader; `pdfcer-core` reports the object number and says the
decision is the shell's. This shell does not draw pop-ups at all, so one
stranded across the sheet is invisible here and visible in Acrobat.

**That is R8b rule 4's surviving half exactly**: an inference or a
consequence the operator cannot see still owes an off-canvas report. Render
normally; report separately. Both.

# What is deliberately NOT disclosed

**`geometry_keys_moved` being empty**, which the engine warns about by name:
a Text note, a Stamp or a Link has no geometry key because its `/Rect` *is*
its geometry, so empty is a correct answer and reporting it would manufacture
an anomaly out of the commonest case.

**`rect_differences_untouched`**, for a different reason: `/RD` holds inset
distances rather than coordinates, translating them would deform the
annotation, and not translating them is therefore not a limitation to
confess but the only correct behaviour. A sentence about it would teach an
operator to worry about something that is right.

### `fn resize`

The disclosure is the operator's own ruling, carried through. He asked
for Inkscape's toggles — *"default should be what it said, but there should
be an option that they do scale with resize"* — and the sentence that
belongs beside a default is the one that says the default fired.

**`stroke_width: None` is the case that owes a sentence**, which is the
engine's own instruction: *"an operator who scaled a square 3× and expected
a heavier border needs telling it stayed."* That is R8b rule 4's surviving
half
— a line weight left alone is invisible on the canvas, because the shape
grew around it and nothing says the border did not.

**`CarriedDistorted` is the other one**, and it is not a defect: neither
PDF nor SVG has a per-axis stroke width, so a non-uniform scale of an
appearance pdfcer did not author produces an anisotropic border by
arithmetic. The engine refuses that case unless it is allowed; where it
proceeds, the operator is told.

### `fn rotate`

Reached from `canvas::rotating` on the release of a rotate-handle drag, and
from nothing else.

# There is no options type, and its absence is the feature

[`resize`] one screen up takes `crate::canvas::scaling::Modifiers` — the
operator's Tool-row switches — because a resize has a genuine question to
ask: *does a line weight scale with the shape?* A rotation has no such
question, because **a rotation is an isometry**. Every length is preserved,
including the drawn stroke width, so there is nothing for a switch to
decide and no switch is offered.

`pdfcer-core` drew the consequence for this shell's grip UI in one line, and
it is the line that shaped this whole gesture: *"if your grip UI offers
rotate and resize together, **rotate needs no confirmation step and no
distortion warning.** Resize does."*

# And unlike [`resize`], a FOREIGN appearance turns correctly

`resize_annotation` has to refuse artwork pdfcer did not draw — §12.5.5's
placement matrix scales it *after* stroking, and no scalar `/BS /W`
describes an anisotropic stroke. That refusal is why [`resize`] carries a
worded decline for `ResizeAppearanceNotRebuildable` at all.

**Rotation has no equivalent**, and the reason is in the standard rather
than in an implementation choice: step (a) transforms the appearance `BBox`
through its **own** `/Matrix`, so pdfcer composes the rotation into the
matrix a producer already wrote. Nothing is redrawn and nobody's artwork is
replaced — it works on a stamp Acrobat made.

# There is no disclosure, and the reason is the outline

**`/Rect` grows.** §12.5.2 requires it upright, and the upright box
bounding a rotated rectangle is larger at any angle that is not a quarter
turn. That is correct and normative.

It owes the operator nothing because **it is invisible**:
`canvas::annotquad` draws the selection outline at the mark's own angle
(`OPERATOR_REQUESTS.md` O147), so the box hugs the artwork rather than
swelling around it. R8b rule 4 asks for an off-canvas report of a
consequence the operator cannot see; here there is no consequence to see.
An outline drawn **from `/Rect`** instead would put one back, and
`text::rotating`'s own header carries why the deleted `rect_grew` sentence
must not be restored for the O145 growth defect.

# What is deliberately NOT disclosed

**`rect_differences_untouched`** (`/RD`), for the reason [`move_annot`]
already gives about the same key: at an angle that is not a quarter turn
**no** axis-aligned inset expresses the rotated result, so pdfcer does not
invent one and leaving it alone is the only correct behaviour. A sentence
about it would teach an operator to worry about something that is right.

**`appearance_matrix_updated`** — that is *how* a rotation is expressed, not
a consequence of it. It goes in the trace, where implementation facts
belong, and it is the field a wrong build would get wrong.

### `fn set_rotation`

Reached from the Properties panel's typed Angle field, and from nothing
else — the rotate grip is a drag and goes to [`rotate`], which is a delta.

# Why this is a second function rather than an argument on [`rotate`]

Because they take different things and refuse for different reasons. A
delta needs no starting angle and therefore cannot fail to read one; an
absolute set **must** read the current angle and refuses by name when it
cannot (`AnnotationRotationUnreadable`). Folding them would mean one
function whose failure modes depend on a boolean, and a caller reading the
refusal would have to know which mode it was in to know what the sentence
meant.

# The disclosure is about the RECTANGLE RULE, not about the turn

`/Rect` is a function of the artwork rather than of the previous
rectangle, so a rotation composes — *N* turns totalling θ draw
the same size as one turn of θ. **With one exception the engine named and
this shell must honour:** an annotation with **neither an appearance stream
nor rotatable geometry** — a `/Square` or `/Circle` with no `/AP` — has
nowhere an orientation could be recorded, because its artwork *is* its
rectangle and §12.5.2 requires that upright. `RectDerivation::PreviousRect`
is that case, it **still grows on every turn**, and the engine's reply is
explicit that ignoring it *"re-introduces the operator's bug one level up,
on exactly the annotations that cannot be fixed."*

[`crate::text::rotating::rect_still_grows`] is the sentence, and it fires
**only** on that rule. A disclosure that fired on every non-quarter turn of
everything would train the operator to ignore it.

### `fn rotate_dimension`

Reached from `canvas::rotating` on the release of a rotate-handle drag over
a selected ce dimension, and from nothing else.

# Why this is a second function rather than a branch in [`rotate`]

Because `rotate_annotation` **refuses a ce dimension by name** and points
here, with its reason attached: *"a ce dimension's orientation is part of
its measurement, so turning it must re-measure rather than spin a
rectangle."*

A ce dimension is a `/Line` with `/IT /LineDimension` and a record in the
document's `/PieceInfo` sidecar. It passes every *"is this markup pdfcer can
author?"* test. Turning it as an annotation would rotate the `/Rect` and the
baked `/AP` and leave the **sidecar geometry** — the thing the displayed
number is derived from — exactly where it was, so the dimension would draw
at one angle and measure along another.

This is the same routing obligation this module's header records for
`set_markup_style`, and `canvas::selection::annot::AnnotKind` carries the
distinction on the selected target precisely so the fork is a `match` the
compiler checks.

# The measured value CANNOT change, and nothing says otherwise

A rotation preserves every distance, so the number is identical either side
of it **by construction** rather than because pdfcer holds it. The engine
therefore returns no before/after pair — deliberately, and it says why:
reporting *"5.000 m → 5.000 m"* would invite a reader to look for a change
that cannot exist.

So there is no disclosure here saying the measurement is unchanged, and
there must not be. A live readout that does not move during the drag is
**correct, not a stale binding**.

# The one disclosure, commissioned by the engine by name

A `Linear` dimension may be constrained to `Horizontal` or `Vertical`. Turn
it 30° and that constraint can no longer describe what is drawn. Three
options existed and two are wrong — refusing makes rotation impossible for
most of a CAD drawing; keeping the constraint leaves the line and its own
stated constraint disagreeing, invisibly, until something regenerates from
it. The engine relaxes to `Aligned` and reports `constraint_relaxed`, with
this instruction attached:

> **Say so**: an operator whose dimension silently stopped being axis-locked
> will find out later and blame something else.

[`crate::text::rotating::axis_lock_relaxed`] is that sentence. It fires only
when the flag is set — a rotation by a whole number of turns leaves the
constraint alone, because nothing moved.

# Scaling a dimension is not here, and will not be

Not unbuilt — **declined**, by the engine and by the operator, on the ground
that it has no honest reading: either the displayed value stays fixed while
the geometry grows, so the dimension lies about the drawing, or both change,
so nothing was measured. The operation actually wanted is `set_group_scale`
— points per unit — which already ships on the Measure surface. That is why
`pressing::grabbable` hands a selected dimension `GripSet::rotate_only()`
rather than the full nine.

### `fn set_note`

Reached from the Comments panel's editor and from nothing else.

# The three keys are not written as a group, and that is the contract

`pdfcer-core` leaves an **omitted** key untouched rather than clearing it,
and its reply to this shell called getting that wrong *"the easiest way to
get this wrong"*:

> An implementation writing all three keys unconditionally would silently
> strip the author and date on every correction, leaving a review comment
> from nobody, dated never, looking exactly like a note somebody else had
> mangled.

So `author` is `None` on two quite different occasions and both must send
nothing: the annotation already has a byline that is not ours to move, or
the operator has left their name blank in Settings ▸ Comments, which is a
supported choice and means *comment anonymously*. `crate::app::actions::apply`
resolves which; this function only has to not invent one.

# `/M` is always written, and it is a modification date


# TWO disclosures, and they are about opposite things

**The words that are gone.** A note that replaced another one usually
leaves no trace on the canvas: the shape is unchanged, and a sticky's words
live in a pop-up window this shell does not draw.
`MarkupNoteChange::replaced` carries the previous text — the text, not a
count — precisely so the operator can be offered it back, which is what
`crate::text::markup::note_replaced` does.

*"usually"* is doing work there, and it is the one subtype that breaks the
family: a `/FreeText`'s `/Contents` **is** its painted words, so on one the
replaced text is on the canvas, and the engine re-bakes the appearance in
this same command so the page follows the edit.

**The half that did not move.** Which is the second disclosure, and it fires
on exactly one shape of outcome — a `/FreeText` whose appearance pdfcer did
not author, which is preserved rather than replaced. See the call site below
and `crate::text::textannot`'s edit-time banner, which carries the four-row
table.

### `fn clear_note`

Reached from the Comments panel's *Remove note* control and from nothing
else.

# It is not a delete, and the disclosure says so because nothing else can

The markup stays on the page with its geometry untouched. A shape with a
note and the same shape without one are **the same picture**, so an operator
who pressed the wrong button has no way to see either what they did or what
it cost them. `crate::text::markup::note_removed` states both — the words
that went, and the fact that the shape did not.

# Why a separate verb from writing an empty note

`pdfcer-core`'s reason, adopted rather than re-derived: *"an empty comment is
a comment, and a reviewer deleting their remark is not the same as leaving a
blank one."* An empty `/Contents` beside a `/T` and an `/M` says somebody
wrote nothing; no `/Contents` at all says nobody wrote anything.

### `fn add_reply`

Reached from the Comments panel's editor when its draft is aimed at a
reply, and from nothing else — the write half of a surface that can
otherwise only read a thread.

# `/M` is OURS, and a reply without one is a note from nowhen

`pdfcer-core` reads no clock, by policy — determinism, and R8b rule 4's
refusal
to let a library invent a claim about when something happened
(`crate::app::clock`'s header carries the whole argument). So the stamp is
supplied here, from the crate's **one** wall-clock reader, exactly as
[`set_note`] supplies it. A reply that reached the file with no `/M` would
render in every reviewer UI in the class as a comment with a blank date
beside a parent that has one — which reads as a corrupted thread rather than
as a missing key.

[`crate::app::clock::pdf_date_utc`] returns `None` before the Unix epoch,
and in that case no `/M` is sent rather than a plausible one being invented.
Absent is honest; wrong is not.

# The author is the operator's, always, with no `keep_author` question

`author` here is the same `Prefs::author_name` [`apply_action`] hands
[`set_note`], filtered by the same rule at the same seam — one source, so
the two surfaces cannot come to sign comments differently. What is *absent*
is `SetNote`'s `keep_author`, and the asymmetry is load-bearing: that verb
may be correcting somebody else's comment and must be able to leave their
`/T` alone, whereas a reply is a **new annotation this operator is
authoring** and has no prior byline to preserve. See
`AnnotAction::Reply`'s own docs.

A blank preference means *comment anonymously*, which is a supported choice
and not a missing value — so no `/T` is written, and no name is invented.

# The disclosure: the reply's OWN `/Popup`, which nothing here draws

`add_reply` authors a `/Popup` companion for the reply (§12.5.6.14), and
`ReplyAdded::reply_has_popup` reports it because this shell asked to be
told:

> *"a reply that quietly acquired a second window at a second location is
> something we would rather be told about than discover on a screenshot."*

That question has a shell-side answer and it is deliberate:
`canvas::notepopup::model::notes_on` **excludes replies** from the notes it
draws windows for, so pdfcer shows a reply inside its parent's thread and
never as a second bubble on top of the comment it answers. See that
function's exclusion table for why the alternative is worse than untidy —
the reply is placed at the parent's own `/Rect`, so a shell that drew it as
an independent note would make the parent unclickable the moment anybody
answered it.

The window still **exists in the file**, and another reader will draw it.
That is a fact about the document that no surface in this program can show,
which is precisely the test for what belongs in a disclosure —
[`crate::text::panels::comments::reply_posted`].

### `fn set_open`

Reached from the canvas pop-up's *Open by default* control and from nothing
else. **Not** from opening or closing a bubble on screen: that stays with
`crate::canvas::notepopup::open`, which owns per-document interface state
and raises no action at all. `AnnotAction::SetOpen`'s docs carry the whole
argument for the split, and the short form is that a reviewer reading six
comments must not end the session with six undo entries and a dirty file.

# Both objects, and why one call rather than two

Table 170 gives geometric markup **no `/Open` of its own**, so a `/Square`'s
window state lives only on its `/Popup`; a `/Text` has one on itself and
`pdfcer-core`'s own author writes both. The engine therefore treats the
state as a property of the **pair** and writes whichever halves exist, in
one command and one undo entry. A shell that issued two calls would put a
`Ctrl+Z` between them and could leave the two disagreeing on exactly the
subtype the operator uses most.

# The no-op case is disclosed rather than hidden

When the annotation takes no `/Open` of its own **and** has no `/Popup`,
`set_annotation_open` succeeds, writes nothing and pushes **no undo entry**
— reported as a no-op rather than refused, so that a caller acting over a
mixed selection need not filter by subtype. The affordance is gated on
[`crate::canvas::notepopup::model::can_record_open_state`] under R83, so
this should not be reachable from the control; it is disclosed anyway,
because *"the
button did nothing and said nothing"* is the one outcome an operator cannot
tell from a bug.

### `fn apply_action`

Called from `apply`'s single `Action::Annot(_)` arm. The seam is `apply`'s
own, stated in its header: **that file routes by family, and the family
module decides** — one arm there instead of one per variant here.

# Why the author name is a parameter and not a read

[`crate::app::actions::annot::AnnotAction::SetNote`] carries `keep_author`
— a fact about the **document** the raising surface had in front of it —
and the name is a fact about the **operator** that only the apply scope can
see. A panel that carried a name would be reading preferences it is not
handed; a body that re-derived `keep_author` would be walking the annotation
a second time for something already known. So one travels on the action and
one travels as an argument, which is the split the field's own doc argues
for.

The empty string means *no name is set*, and it is filtered here rather than
at the call site so that every future caller of this router gets the same
answer to *"what does a blank author preference mean?"*.
