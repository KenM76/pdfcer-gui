# `canvas::markup::text` — underline, strikeout and squiggly: markup whose
operand is a **selection**, not a drag

Three kinds whose one blocker the operator stated in his own terms — *"they
mark **text** and there is no text-selection gesture yet."* The gesture is
[`crate::canvas::textsel`], and this module is what it buys: no new
subsystem, one rule applied to a selection that already exists.

---

## 1. THE INTERACTION DECISION, and it is the whole of this module

[`super`]'s four kinds are **drag-shaped**: press, rubber-band, release,
commit. These three are not, and there were two honest ways to build them:

| | model | what it costs |
|---|---|---|
| **(a)** | select text first, **then** press Underline — the selection is the operand | nothing new: no [`CanvasTool`](crate::canvas::tool::CanvasTool) variant, no gesture, no mode of `textsel` |
| **(b)** | arm Underline, then sweep, and the sweep both selects and marks | a tool variant, a gesture, a second mode of `textsel`, a second place a text range is resolved |

**Shipped: (a), and it came from Acrobat**, which is the reference
application that decides this one under the standing instruction — *"make
your best educated guesses to match what inkscape, acrobat, and SolidWorks
do"* — for the reason `textsel`'s own table gives: **Acrobat wins ties about
reading and
about marking up what has been read, because Acrobat is what pdfcer
replaces.** Inkscape and SolidWorks have no vote here at all: neither has PDF
text markup, and neither has anything shaped like it (Inkscape's text tool
edits its own text objects; a SolidWorks note is content, not a comment on
content).

### What Acrobat actually does, and the honest complication

Acrobat offers **both**. Select text with the Selection tool and a
context/hover menu appears carrying *Highlight text*, *Underline text*,
*Strikethrough text* — model (a) — and the Comment toolbar separately carries
the three as arm-then-sweep tools — model (b). Reader, the product this shell
is measured against, leads with (a): selecting is what its one tool does, and
marking is what you do to a selection.

So (a) is *"what Acrobat does"* without qualification, and (b) is *"what
Acrobat also does"*. Given a tie broken toward the smaller change, three
further things decide it and each is specific to this shell rather than to
taste:

1. **(b) would need a second text-range derivation.** `textsel`'s §2 already
   refused Acrobat's `Alt`+drag rectangular selection on exactly this
   ground — *"one derivation, so what is shown and what is copied cannot
   diverge"*. An armed marking sweep is a third resolver beside `drag` and
   `click`, and the moment it exists the wash the operator sees and the
   `/QuadPoints` written to the file are produced by two functions that can
   disagree.
2. **The operand is already visible.** A text selection paints a wash. Under
   (a) the operator can see precisely what will be marked *before* pressing
   anything, which is the pre-commit affordance rule 4 asks for — and it
   exists already, for free, drawn by the module that owns it (§4).
3. **(a) is the shape `format.delete` already has.** A command that acts on
   a selection is not a new idea in this shell; a tool that arms and marks
   would be.

What is deliberately **not** claimed: that (b) is wrong. It is the tool half
of Acrobat's answer and it is a reasonable thing to add later.

**[`crate::canvas::tool::CanvasTool::Text`] makes (b) cheap and does not
make it right.** (b)'s cost was never mainly the tool enum; it is objection 1
— a marking sweep is a **second text-range resolver** beside `drag` and
`click`, and the moment it exists the wash the operator sees and the
`/QuadPoints` written to the file come from two functions that can disagree.
That objection is untouched by the variant's existence. What the variant does
settle is reachability: model (a) works in Edit, by the route that costs no
second derivation.

### 1.1 The route is the ribbon, and Acrobat's is a menu on the selection

Half of model (a) is *where the operator finds the verb*, and here the
shipped answer and Acrobat's differ: Acrobat pops a context menu — and, in
recent versions, a floating hover toolbar — **on the selection itself**,
while this ships the three commands on **Markup ▸ Text markup** and nothing
else. Named as a gap rather than passed over, because a reviewer who selects
a phrase and right-clicks it will expect the verbs to be there.

What it would take, so the next hand does not re-derive it: a third canvas
menu context beside `CANVAS_OBJECT` and `CANVAS_EMPTY` — call it
`canvas.text` — chosen in [`crate::canvas::menus`] by the same
decide-at-the-click rule those two already use, with the decision being
*"was the pointer over a live text selection?"*. That is a real question with
a real answer (the selection's canvas quads are on the document and the
pointer position is in the same space), and it is a **menu taxonomy** change:
a new context id, its entry in the manifest's `menus`, and a rule for which
of three menus opens. Deferred on the same principle `textsel` deferred
`CanvasTool::Text` — the shape of the fix is written down; the fix is not
smuggled into the module that noticed it.

**No chord is bound either**, deliberately and by the argument the operator's
own zoom-to-selection decision settled: this shell's manifest chords are
`Ctrl`-modified by construction, `Ctrl+U` is not an Acrobat binding to match,
and inventing one would match the muscle memory of nobody. The keymap stays
at nineteen bindings.

---

## 2. THE MODE INTERSECTION, and it is narrower than either half

This is the finding a reader most needs, because neither capability alone
predicts it. Marking text needs **both** halves, and they do not overlap in
the way anybody would guess:

| mode | can select text? | `author_markup`? | can mark text? |
|---|:-:|:-:|:-:|
| `read` | ✓ | ✗ | **no** — nothing to mark *with* |
| `review` | ✓ | ✓ | **YES** |
| `edit` | ✓ **with the text tool armed** | ✓ | **YES** |

**The Edit row is the one that costs a tool.** Edit's primary button is
the content marquee, so `textsel::takes_the_press` refuses Edit a text
selection unless [`crate::canvas::tool::CanvasTool::Text`] is armed — by
**`view.tool_text`**, in View ▸ Navigate beside the hand tool. An editor arms
it, sweeps a range, and presses Underline; `selection.text` is published from
the same live selection every other mode publishes, and the controls enable.

Without that route the three controls would be drawn on Edit's Markup tab and
**permanently greyed**, which is a `RIBBON_IA.md` **P3** violation and an
inversion besides — an editor may not mark text a reviewer may. The rule
[`mark`] implements does not know about any of this, and that is the property
to preserve: reachability is decided in `textsel` and in the tool enum, never
here.

**What the two remaining zeros are, and why neither is a defect:**

* **Read** is shown the File and View tabs only, so the Markup tab is not
  there and no `markup.*` command can be invoked; the dispatch arm declines
  anyway (`caps.author_markup`), which is the belt to that braces for a
  customized manifest that binds a chord. Read *not* authoring is the point
  of Read — `DEFECTS.md` D6. Note that Read *can* select text and now also
  carries the `view.tool_text` control (View is in every mode); arming it
  there changes nothing, because Read's select tool already swept text. What
  Read still cannot do is **mark** what it selected, which is the correct
  half of this table to be empty.
* **Edit without the tool armed** is still a zero, and deliberately: the
  controls are greyed exactly as long as there is no live text selection,
  which is *temporarily* unavailable in P3's own sense — the operator's next
  act can change it, from a control on a tab they are already being shown.
  That is the difference between the state before this change and the state
  after, and it is the whole of the difference: the pixels are identical, and
  the reachability is not.

### Greying, and the rule that now actually applies

`RIBBON_IA.md` P3 forbids a control that is *always live and does nothing*,
and reserves greying for *temporarily unavailable, explained on hover*. Every
greyed state these three controls can reach is temporary in that sense: sweep
some text and it ends.

The rule is worth stating as a rule, because the tempting move when P3 is
uncomfortable is to argue the greying is *nearly* temporary, or to invent a
hiding mechanism. **A rule being uncomfortable to satisfy is evidence about
the feature, not about the rule** — here it was evidence that Edit needed a
route to a text selection.

---

## 3. The three kinds are NOT `MarkupKind` variants, deliberately

[`super::MarkupKind`]'s contract is written down in its own docs and in
`shell::commands::mapping`: *a variant belongs in that enum when this rubber
band can draw it*, and every variant is required by test to have a command
that **arms a tool** and a `selected:` condition that lights while it is
armed. None of that is true of these three: their commands act immediately
and arm nothing, so a variant would be a tool that cannot be armed, a pressed
state that never lights, and a `CanvasTool` state no `GestureOutcome` can
reach — dead states in a type whose whole purpose is to say what the tool is
currently doing.

So they are a separate enum, [`TextMarkKind`], with its own `ALL` and its own
id mapping (`shell::commands::text_mark_command`), and the two families stay
disjoint — which the mapping module asserts in both directions, because
`app::dispatch`'s guard arms are tried in order and an overlap would swallow
one silently.

### Highlight is the fourth `/QuadPoints` subtype and stays where it is

[`super::MarkupKind::Highlight`] is engine-identical to these three —
`MarkupSpec::TextMarkup` with a different [`TextMarkupKind`] — and it remains
a **drag** across an area, because that is what it already is and because a
highlight over an image or a title-block cell is a thing operators want and
text markup cannot express. Acrobat has both there too (a text highlight from
the selection, an area highlight from the Comment toolbar).

Making `markup.highlight` mean *"mark the selection if there is one, else arm
the band"* was considered and refused: one control with two behaviours
decided by invisible state is precisely the thing an operator cannot predict,
and the trace would not say which happened. If the selection-highlight is
wanted, it is a **fourth entry in [`TextMarkKind`]** and a fourth command —
at which point the Text markup group reads Highlight-text, Underline,
Strikeout, Squiggly and the area Highlight sits with the Shapes it behaves
like. That is a taxonomy change and belongs to the operator, so it is
recorded here and not taken.

---

## 4. There is no second preview, and that is the decision

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 permits a
pre-commit affordance and requires that it describe *what will actually
commit*. [`super`]'s band satisfies that by being drawn in the shape and the
pen of the annotation it is about to author.

Here the affordance already exists and is drawn by somebody else: **the
selection wash is the preview.** Its boxes are [`TextSelection::quads`], and
the quads this module authors are [`TextSelection::page_quads`] — literally
the same boxes from the same pass (`textsel` §5.1). A preview cannot describe
what will commit more exactly than by being it.

The argument for adding a second one anyway, taken seriously and rejected: an
underline is not a wash, so a *shape* preview — a line at each quad's
baseline, in the pen colour — would show the operator the mark rather than
the range. Three things against, in order of weight:

1. **It would have to be drawn on hover of a ribbon button**, because there
   is no gesture in flight to hang it on. A canvas that changes while the
   pointer is in the ribbon is a canvas that flickers as the operator's hand
   passes over three adjacent controls.
2. **It would be a second geometry** — the baseline offset, the squiggle's
   amplitude — approximating an appearance stream `pdfcer-core` generates. Two
   drawings of one annotation is how the preview comes to lie, and the lie
   would be invisible until the file was reopened.
3. **The commit is instant and undoable.** There is no drag to abandon: press
   the button, see the annotation, undo it. The affordance a rubber band
   exists to provide — *aim before you commit* — is already provided by the
   selection.

So: no hover preview, no ghost, no second colour. The wash, then the
annotation.

---

## 5. What this module does not do

It does not touch a document, does not take an `EditSession`, and builds no
appearance stream. [`spec`] hands `pdfcer-core` a `MarkupSpec` and
`EditSession::add_markup` does the rest, which is the same route
`pdfcer`'s `markup-add` takes with the same value — the equivalence the
measure salvage's tests exist to protect, and the reason a canvas-authored
annotation is byte-identical to a CLI-authored one.

[`mark`] is pure and is where every rule lives; the dispatch arm calls it and
pushes what it returns. That is the same division [`super::action`] has, for
the same reason: a rule with a test beats a rule inside a `match` arm.

## Item notes

### `fn subtype`

The one place the shell's vocabulary meets the specification's, exactly
as [`super::spec`] is for the geometric kinds. The two enums are
deliberately not the same type even though three of the four names match:
`TextMarkupKind` carries Highlight as well, which belongs to the *band*
gesture here (§3), and a shared type would make that fourth value
reachable from a control that cannot mean it.

### `fn slot`

Split out from `rgb` so a test can assert the *routing* without asserting
a colour. The two are different claims: which slot a kind takes is a fact
about this module and must not change; what colour that slot holds is the
operator's and may.

### `fn pen`

Named rather than spelled `Pen::default()` at nine call sites so that the
tests which *are* about colour stand out by building their own — the
reader can tell at a glance which assertions would move if the default
moved.

### `fn each_kind_authors_its_own_subtype`

The failure this catches is the copy-paste one: three arms built from one
another, two of which say `Underline`. It would produce three ribbon
controls that all draw an underline, and nothing else in the system would
notice — the engine would author a perfectly valid annotation each time.

### `fn the_selections_quads_are_authored_unchanged`

The one-derivation promise at this end of it: the boxes the operator saw
washed are the boxes written into `/QuadPoints`. A build that merged,
clipped, re-ordered or de-duplicated them here would mark a different set
of glyphs from the one that was highlighted, and the difference would only
be visible after saving.

### `fn the_operators_pen_reaches_every_text_kind`

# It asserts a RELATION, never a magnitude

A test that pins the literal triple [`TextMarkKind::rgb`] returns is two
copies of one constant, and two copies of one constant can only disagree
if somebody edits one of them — so such a test stays green through the
entire life of a defect in which the pen never reaches the spec at all.

So: whatever colour the kind's own slot holds, that is the colour the
spec authors. Driven with a pen whose eight slots are eight
distinguishable values, so a kind that takes a neighbour's pen names
itself. A planted pen with one ink could not catch a kind taking the
wrong *line* colour, because there would be only one line colour to take.

# What it deliberately does NOT assert

The shipped default values. Those are Acrobat's, pinned against the
registry keys they were read from by
`pen::tests::every_slot_ships_at_the_acrobat_value_it_was_measured_from`;
`Pen::default`'s own doc comment carries the argument for departing from
the "omits nothing" rule. Asserting them a second time here would be
exactly the two-copies-of-one-constant mistake above.

### `fn each_text_kind_takes_its_own_pen`

# What this replaces, and why the replacement is stricter

It was `no_text_kind_takes_the_highlighter`, which asserted the
two-instrument partition: *these three are lines and take the ink;
Highlight is a wash and takes the highlighter.* True while there were two
pens, and it would now pass on a build that had collapsed Underline,
StrikeOut and Squiggly back into one slot — which is precisely the
regression the operator's ask forbids, since Acrobat gives each of them
its own key and two of them different colours.

So the claim is now about **separation**: four kinds, four slots, no two
equal. It still catches everything the old one did — a future hand
"simplifying" `rgb` to one shared colour fails on the first pair — and it
catches the new failure as well.

It asserts on the **slot**, not on the colour. Two slots may legitimately
hold the same colour (Squiggly and Shape ship at the same Acrobat red, and
an operator may set any two the same), and a test that demanded distinct
*colours* would forbid a state the operator is entitled to choose.

# THE SEPARATION CLAIM ALONE WAS NOT ENOUGH, and running the
# falsification is how that was found

This test shipped its first draft asserting only *"no two of the four
share a slot"*, with a doc comment claiming it was falsified by pointing
`Squiggly` at `PenSlot::Shape`. **That falsification was run and the test
stayed green** — because `Shape` is a fourth distinct slot, so all four
were still different and the separation claim was still true. The mark
would have come out of the shape pen, moving whenever the operator
recoloured a rectangle, and this test would have said nothing.


Falsified twice, both actually run: pointing `Squiggly` at
`PenSlot::Shape` (fired — and did **not** fire before the identity row
existed), and pointing it at `PenSlot::StrikeOut` (fired). Both land on
the identity row, because it is checked first and is the stricter of the
two; the separation row is kept anyway, since identity alone would pass a
build where two kinds were each renamed to the other's slot. Restored
after each.

### `fn planted_pen`

Built from each slot's index rather than written out, so a ninth slot
gets a ninth distinct value with no edit here — the same construction
`pen::tests::planted` uses, and for the same reason.

### `fn a_live_selection_marks_its_own_page`

The page assertion is the load-bearing half and it is written as a
magnitude rather than a relation: the action must name page **7**, the
page the selection was made on, not "a page". A build that read
`doc.view.page_index` in the apply arm would author the mark on whatever
sheet was on screen — the same class of defect as the markup that landed
in the centre of the page, one axis over.

### `fn a_stale_selection_is_refused_and_says_so`

`canvas::textsel` §7's rule at the authoring end: after an edit the
recorded positions may name different glyphs, and writing a `/QuadPoints`
annotation from them would put a mark over possibly-wrong words *into the
file*. Distinguished from [`Refusal::NoSelection`] on the trace, because
the two have different answers — sweep again, versus sweep at all.

### `fn a_selection_with_no_boxes_authors_nothing`

Structurally unreachable through `textsel::resolve`, which answers `None`
instead — and guarded anyway, for the reason the geometric kinds guard
their degenerate drag: the shell never sends the engine an empty
`/QuadPoints`, so `validate_geometry` never has to refuse one and the
operator never sees an engine error for a shell decision.

### `fn every_kind_marks_and_refuses_alike`

Asserted over `ALL` rather than for one kind, because the plausible
failure is per-kind: a fourth entry added to the enum, given a subtype and
a command, and reaching a `mark` that quietly special-cases the three that
were there first.
