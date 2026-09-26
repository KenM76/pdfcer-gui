# `canvas::markup::text` — underline, strikeout and squiggly: markup whose
operand is a **selection**, not a drag

Three kinds whose one blocker the operator stated in his own terms — *"they
mark **text** and there is no text-selection gesture yet."* The gesture is
[`crate::canvas::textsel`], and this module is what it buys: no new
subsystem, one rule applied to a selection that already exists.

---

## 1. ★ THE INTERACTION DECISION, and it is the whole of this module

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

★ **[`crate::canvas::tool::CanvasTool::Text`] makes (b) cheap and does not
make it right.** (b)'s cost was never mainly the tool enum; it is objection 1
— a marking sweep is a **second text-range resolver** beside `drag` and
`click`, and the moment it exists the wash the operator sees and the
`/QuadPoints` written to the file come from two functions that can disagree.
That objection is untouched by the variant's existence. What the variant does
settle is reachability: model (a) works in Edit, by the route that costs no
second derivation.

### 1.1 ★ The route is the ribbon, and Acrobat's is a menu on the selection

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

## 2. ★ THE MODE INTERSECTION, and it is narrower than either half

This is the finding a reader most needs, because neither capability alone
predicts it. Marking text needs **both** halves, and they do not overlap in
the way anybody would guess:

| mode | can select text? | `author_markup`? | can mark text? |
|---|:-:|:-:|:-:|
| `read` | ✓ | ✗ | **no** — nothing to mark *with* |
| `review` | ✓ | ✓ | **YES** |
| `edit` | ✓ **with the text tool armed** | ✓ | **YES** |

★ **The Edit row is the one that costs a tool.** Edit's primary button is
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

★ The rule is worth stating as a rule, because the tempting move when P3 is
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

## 4. ★ There is no second preview, and that is the decision

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
