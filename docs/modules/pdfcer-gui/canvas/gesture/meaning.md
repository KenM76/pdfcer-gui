# `canvas::gesture::meaning` — what a press MEANS, decided once and then remembered

One pure function, [`press_kind`], and the two enums it decides between:
[`DragKind`], which says what a drag is going to *do*, and [`MarqueeIntent`],
which says what a rubber band does when it is released. Nothing in this file
holds state, touches egui, or knows that frames exist — a press is
`(tool, grip, zoom_armed, capabilities)` in and one meaning, or `None`, out.
That is what makes the whole precedence testable as a table.

The state machine that *carries* a meaning across a press, a drag and a
release — [`PointerFrame`](super::PointerFrame),
[`GestureState`](super::GestureState) and
[`GestureOutcome`] — is the parent module, [`super`].
It calls this function on exactly one frame per gesture, the press frame,
and then never asks again.

The precedence itself — which meaning wins when two are available, and which
presses a mode refuses outright — is documented on [`press_kind`], because it
*is* the rule rather than a note about it.

[`press_kind`] deliberately has no case for the hand tool, and the absence is
load-bearing: `canvas::interact` hands the state machine a **blank** frame
while the hand is active, so no press ever arrives here to be classified.
That rule — *one state machine, one meaning per frame* — is stated in full in
[`super`]'s header, under "Marquee versus pan".

## Marquee-select versus marquee-zoom: one rubber band, two releases

Phase 3.4 adds a marquee that *zooms* to what it encloses. It is
deliberately **the same gesture**: same press, same in-flight rect, same
pixels on screen ([`crate::canvas::overlay::draw_marquee`] is not
duplicated), same normalisation, same Escape. What differs is one thing —
*what happens on release* — so what is carried is one value, [`MarqueeIntent`].

It is sampled **at the press**, exactly as `shift` is, and for the identical
reason: the one-shot arming is retired when the drag completes, and an
intent re-read at release would be read after something else had already
consumed it. A gesture means what it meant when it started.

## Item notes

### `enum DimensionPress`

Resolved by [`crate::canvas::pressing`] while it has the document and the
mapping in hand, so that the meaning function stays free of geometry — the
same division `grip` and `handle` already follow.

`None` at the call site means *no ce dimension is selected, or the press
missed it*, and the press falls through to the ordinary rungs.

### `enum RotatableAnnot`

Carried as a *variant* rather than as a bool, for
`canvas::selection::annot::AnnotKind`'s own stated reason and for a second
one this function needs: the two families are **gated by different
capabilities**, and a bool would have to be paired with a second bool
saying which gate to ask.

| | verb | capability | why that one |
|---|---|---|---|
| [`Self::Markup`] | `rotate_annotation` | `author_markup` | markup is authored in **Review**, where `edit_content` is false — an operator who has just drawn a shape there and wants to turn it is in the mode the content branch does not run in |
| [`Self::CeDimension`] | `rotate_dimension` | `author_measure` | turning a dimension is a **measure** edit: it writes the sidecar and one annotation and touches no page content. The same ruling the vertex drag already ships under, and for the same reason — a mode that may author a dimension may adjust the one it just authored |

Resolved by [`crate::canvas::pressing`] while it has the document and the
mapping in hand, so this module stays free of geometry — the same division
`grip`, `handle` and [`DimensionPress`] already follow. **It is `Some` only
when the press origin is actually on the handle**, because it is derived
from the very `grip` this function reads, through the very `GripSet` the
painter uses. One predicate; see [`crate::canvas::handles::GripSet`] H7.

⇒ That last property is the guard against **the hazard this canvas has
produced four times**: a working gesture aimed at the wrong verb. The most
recent was a `covers()` that tested the selection's *move box* alone — and
the rotate handle sits OUTSIDE that box, so a press on it selected the
object underneath and the rotate became a select-and-move. Nothing here asks
a second question about where the pointer is; it asks
`handles::grip_at`, which is the function the gesture machine asks.

### `fn name`

`Debug` on `Resize(SouthEast)` renders with brackets, and this value
goes into a `key=value` line a driven check parses. A `Debug` spelling
in a parsed field is banned in this tree, and not as a style rule: it
has produced **two** false failure reports here, one of which reported
the opposite of the truth while quoting the truth in its own message.

The **kind** and not the payload. *"Which grip"* is already in the
`grip=` field beside it, and two spellings of one fact can disagree.

⚠ Exhaustive with no wildcard, on purpose. A new drag kind is then a
compile error here rather than a press that traces as something it is
not — which is the failure this function exists to end.

### `struct PressMeaning`

Lifted out of `canvas::interact` when the markup tool arrived, because it
stopped being a two-case question the moment there were three tools and it
is exactly the kind of rule this module exists to hold: it is a decision
about what the pointer means, it is drivable with no window, and leaving it
as a `match` in the middle of the wiring is how the ordering below becomes
three separate opinions.

# The order is the rule

1. **An armed markup tool outranks everything**, including the grips. A
   markup drag that started on a selected object's resize handle must draw a
   shape, not resize — the operator armed a pen, and grips belong to a
   selection they are not currently acting on. (There is no resize verb to
   reach anyway; see [`crate::canvas::handles`].) It outranks the region
   zoom for the same reason: only one of the two can own the primary drag,
   and the one the operator armed *last* is not knowable here — but the one
   that authors content is the one whose loss would be silent.

   This rung sees only the **band** and **freehand** kinds. The two
   vertex kinds are answered by an early return above, beside the measure
   tools, because their gesture is clicks and they have no drag at all —
   [`crate::canvas::markup::MarkupKind::is_vertex`], and the comment at the
   branch itself.
2. **An armed text tool**, which sweeps a range in *every* mode — including
   the ones whose primary button is otherwise the content marquee. It sits
   here, above the content branch, because that branch is total: below it this
   rung would be unreachable in exactly the mode the tool was built for. It
   yields to an armed region zoom, and only to that; see the comment at the
   branch itself for why the ordering is borrowed from the reading-mode text
   row rather than decided afresh.
3. **A grip** — resize on the six that resize, move on the two that do not.
4. **An armed region zoom**, which turns the marquee's release into a zoom.
5. **A plain marquee**, which is what an un-armed canvas does.

The hand tool is deliberately **absent** from this list, and its absence is
load-bearing: `canvas::interact` hands the gesture machine a *blank* frame
while the hand is active, so no press ever reaches this function to be
classified. One state machine, one meaning per frame — see the module
header.

# The mode gate lives here, and it is two answers rather than one

The mode's [`Capabilities`] are applied **here**, at the point where a press
is given its meaning, rather than at the several places that act on one.
That ordering is the whole design: a press whose meaning is forbidden never
becomes a drag, so there is no band to draw, no ghost to preview, no
release to refuse and no half-gesture to explain.

[`PressMeaning`] carries **two** answers because the canvas has two kinds of
tool and they take the primary button differently — see that type's header.
A single `Option<DragKind>` was the first shape of this gate and it was
wrong in a way that would not have shown up until Review mode was used in
anger: it made "a drag means nothing here" and "a click means nothing here"
the same fact, which is exactly false for the measure tools, whose entire
gesture is clicks.

Refusing at the *press* is also what keeps the safety rule intact
(`MODES_AND_PANELS.md`: *"It never makes a visible control silently
inert"*). Nothing visible is refused, because in a mode that cannot select
there is no selection, hence no handles and no outline — see
`app::modes::capability` §5 and `PdfcerApp::on_mode_capabilities_changed`,
which clears the selection on the way in precisely so that this function
never has to refuse a grip the operator can see.

Which capability each meaning needs:

| Meaning | Needs |
|---|---|
| [`DragKind::Markup`] | `author_markup` |
| a **vertex-markup** click — PolyLine, Polygon | `author_markup`, and it is the same flag on purpose: these author a comment, so a mode that draws rectangles draws polygons |
| a measure **click** | `author_measure` |
| [`DragKind::Resize`], [`DragKind::Move`] | `edit_content` |
| [`DragKind::Marquee`] with [`MarqueeIntent::Select`] | `edit_content` |
| a selecting **click** | `edit_content` |
| [`DragKind::Marquee`] with [`MarqueeIntent::Zoom`] | **nothing** — it is a navigation gesture that reads the document and changes none of it, so it is offered in every mode, Read included |
| [`DragKind::TextSelect`], and the click that goes with it | **nothing** — either because the operator armed the text tool, or *because* `edit_content` is absent, which is the one row here that reads backwards |

# The text row, and why it is not an inconsistency

Every other row above asks *"does this mode permit the gesture?"*. The text
row asks *"is the primary button free, or has the operator claimed it?"* —
which is a different question that happens to read the same flag in one of
its two halves. That is not a capability inverted.



* **un-armed** (`CanvasTool::Select` in a mode that cannot select content) —
  exclusive **by construction**, one flag on both sides of one branch, which
  is how it shipped;
* **armed** (`CanvasTool::Text`, in any mode) — exclusive **by precedence**,
  at rung 2 above, which is the rule `DragKind::Markup` has always used.


# Why two fields rather than one `Option<DragKind>`

Because the canvas has two kinds of authoring tool and they take the primary
button in genuinely different ways:

| tool | the gesture is | uses |
|---|---|---|
| markup — rectangle, ellipse, arrow, highlight | press, drag out a shape, release | the **drag** |
| measure — linear, radius/diameter, two-line, scale | click point A, click point B, click where the dimension sits | the **click** |
| select | either: click to select, drag to marquee | both |
| text | either: drag to sweep a range, click to take a word / a line / extend / clear | **both**, and this is the row that shows why the two fields cannot be collapsed even for a tool with no state — three of the gesture's four meanings are clicks (`canvas::textsel` §1) |

A single `Option<DragKind>` cannot express that, and the gate's first
version proved it: it suppressed the click whenever it suppressed the drag,
which is right for Read (neither means anything) and **wrong for Review**,
where a dimension must be placeable and page content must not be
selectable. The two facts have to be separable because a mode really does
grant one without the other.

Keeping them in one value rather than as two returns is what stops them
drifting apart: there is exactly one function that decides what a press
means, and it decides both halves in one pass over the same inputs.

### `fn dragging`

A constructor rather than a literal because it is what almost every test
of the state machine wants, and those tests are about press/drag/release
rather than about modes.

### `struct Press`

A struct because the argument list reached eight, and clippy is right
that eight positional parameters is a call nobody can read — three of them
are now bare `bool`s, and transposing two would compile and produce a
gesture aimed at the wrong verb. `dimdrag::Frame`, `annotdrag::Frame` and
`dragroute::Frame` all took the same shape for the same reason, so this is
the local convention rather than an accommodation.
