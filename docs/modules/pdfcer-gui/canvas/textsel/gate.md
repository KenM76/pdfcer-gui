# `canvas::textsel::gate` — **who owns the primary button**, and the whole
argument for the answer

One public function, [`takes_the_press`], and the ~190 lines of reasoning
behind it. It answers a single question — *does a press on the page mean
"select text"?* — for exactly two callers, so they cannot disagree about
what a press meant: [`crate::canvas::gesture::press_kind`], which turns the
answer into a [`DragKind`](crate::canvas::gesture::DragKind), and
`canvas::interact`, which routes the resulting click.

## ★ Why this is its own file, and why the seam is here

[`super`] crossed `PROJECT_PLAN.md` R2's 1,500-line limit when the text tool
landed, for the second time — the first split took the two keyboard verbs out
into [`super::clipboard`], and that module's header records the R2
measurement that forced it.

**The seam is a subject, not a line count**, and it is the same seam the
parent module's own numbering already drew: everything else in `textsel`
answers *"given that this press is a text gesture, what range does it
resolve to and what does that range carry?"* — the projection, the line
grouping, the staleness epoch, the one pass that produces the string and both
sets of boxes. This file answers the question one step earlier and of a
different kind: *"whose press is it?"* It reads a [`CanvasTool`] and a
[`Capabilities`], and it touches no page, no extraction, no geometry and no
document. Nothing in it can be tested with a fixture and nothing in the rest
of the module can be tested without one, which is the practical form the same
seam takes.

It is also the file that **changes for a different reason**. Every amendment
this predicate has had is a *taxonomy* decision — the operator's *copying is
not authoring* ruling, the mode gate derived from a tab list, the arrival of
`CanvasTool::Text` — and none of them is a change to what a range means.


---

## ★ THE MODE GATE — no new capability, and NOT `edit_content`

(This was [`super`]'s header §3 until the split; the parent still carries a
one-line pointer under that number, so a reader following a cross-reference to
"`textsel` §3" from anywhere in the crate lands somewhere that sends them
here.)

This is the part the brief flagged as most likely to be got wrong, in both
directions: getting it wrong permissively re-opens `DEFECTS.md` **D6**, and
getting it wrong restrictively fails the very ask this module exists for.

**The answer is that text selection needs no capability at all**, and the
argument has three steps.

### It is not an authoring gesture, so it is not gated as one

`app::modes::capability` §4 lists what is deliberately **not** gated: *"Pan,
zoom, the hand tool, marquee zoom, Find, guides, rulers, grid. Navigation
and inspection, none of which touches the document."* Selecting text reads
the page and writes to the clipboard. It changes no byte, bumps no
`edit_epoch`, and touches no `EditSession`. It belongs in that list, beside
Find — which is the closest neighbour in every respect: Find also extracts
the page's text, also derives quads from it, and also draws a wash over the
result, and nobody gated Find on a capability.

`MODES_AND_PANELS.md` Part 1 had already said so, in the Read row of the
gesture table, before any of this was built:

> | **Canvas gestures** | pan, zoom, **text selection for copy**, follow links |


### …but it still has to be told apart from the content marquee

What text selection *does* need is the primary button, and in a mode that
selects page content the primary button is already spoken for: a drag on
empty paper is a marquee, a drag on a selected object is a move. Two
meanings, one button, and no way for the canvas to guess which the operator
meant.


The shipped answer follows Inkscape, and the argument for preferring the
minority is at [`CanvasTool::Text`](crate::canvas::tool::CanvasTool::Text) —
summarised three subsections below, under the tool's arrival.


> **A press means text when the text tool is armed, *or* when the select tool
> is active and the mode cannot select content.**

The second half is the original rule, unchanged: that is Acrobat Reader in a
mode that reads, and it leaves the authoring modes' *un-armed* canvas exactly
as it was.

### ★ The first half arrived, and it is the paragraph below that predicted it


> ★ **Edit is the row worth staring at, and it is a known gap rather than an
> accident.** A reviewer can select text and an editor cannot, which is an
> inversion […] The closing move is a `CanvasTool::Text` armed by a
> `view.tool_text` command, at which point [`takes_the_press`] gains one
> disjunct (`tool.is_text()`) and nothing else in this file changes.

That is what happened, to the disjunct. The tool is
[`CanvasTool::Text`](crate::canvas::tool::CanvasTool::Text), armed by
`view.tool_text` in **View ▸ Navigate** beside the hand — the group that
already holds the other pointer-tool toggle, on the one tab every mode is
shown. Its own docs carry the admission argument this module deliberately
declined to make.

What it closed is **two** things, and the second is why it stopped being
optional:

1. An editor can now sweep text, so the inversion is gone.
2. The three text-markup controls (`markup.underline`, `markup.strikeout`,
   `markup.squiggly`) are drawn on the Markup tab in Edit and, before this,
   **could never enable** there, because `selection.text` was never true.
   That was a live tension with `RIBBON_IA.md` **P3**, which reserves greying
   for *temporarily* unavailable and says an absent capability should render
   nothing — and it could not be fixed by hiding them, because a command
   lives on exactly one tab and the Markup tab is in both Review and Edit.
   See [`crate::canvas::markup::text`] §2, which is where that consequence
   was recorded and is now discharged.

### ★ The reference applications disagree, and the argument is in `tool.rs`

Not repeated here, because a decision restated in two files is a decision
that drifts. In one line: **Acrobat and SolidWorks resolve text-versus-object
contextually within one tool; Inkscape uses a separate Text tool; Inkscape
wins**, because an object marquee over *vector content* is a surface Acrobat
does not have at all — so the conflict this is about exists only in the
Inkscape-shaped mode. The whole argument, including the concrete failure a
contextual press would produce on a drawing sheet, is at
[`CanvasTool::Text`](crate::canvas::tool::CanvasTool::Text).

### ★ What the second disjunct costs: exclusivity moves from construction to
### precedence

This paragraph replaces one that is now **false**, and it is replaced rather
than reworded because the false version was load-bearing for a reader:

> The two meanings are **mutually exclusive by construction** rather than by
> precedence: `caps.edit_content` is the same flag on both sides of the
> branch, so there is no state in which one press could do both, and no
> ordering for a future reader to get wrong.

With the tool armed in Edit, both underlying facts *are* true at once: the
mode can select content, and the operator has asked for text. So there is now
an ordering, and it is the one every armed tool already relies on —
[`crate::canvas::gesture::press_kind`]'s rung 1, **an armed tool takes the
press**. That is not a rule invented for this; it is what `Markup` and
`Measure` have used since they landed, and it is asserted over every
capability combination in `gesture::meaning`'s own tests rather than only in
the three shipped modes.

The property that survives untouched is the one an operator can feel: **one
press has one meaning.** `press_kind` returns a single `DragKind`, and
`canvas::interact` routes the click through *this same function* rather than
inferring from a flag, so the drag's meaning and the click's routing cannot
disagree.



The interesting part is that the *shape* was right for a reason its own
argument got wrong. They are two fields on [`crate::app::state::OpenDoc`]
rather than one enum, which was justified as "the exclusivity belongs to the
gesture gate, not to the type" — and had the exclusivity been encoded in the
type, this change would have required a data-model migration instead of one
disjunct. [`crate::canvas::keys`]' rung 5 is where the two are ordered for
Escape, and its header carries that ordering's own argument.

### What this yields, mode by mode

| mode | `edit_content` | select tool press means | text tool press means |
|---|:-:|---|---|
| `read` | ✗ | **text** | **text** (the tool is redundant here, not absent — see [`crate::canvas::tool::toggle_text`]) |
| `review` | ✗ | **text** | **text** |
| `edit` | ✓ | content selection, as before | **text** |

Note what Edit never lost, even before the tool: `file.copy_page_text` and
`file.copy_document_text` are offered by every mode
(`app::modes::capability`'s `both_text_copy_commands_are_offered_by_every_mode`),
and both are wired. An editor could always copy the page's text; what they
could not do was sweep a *range* with the pointer, which is what the tool
restores.

### Why the *retirement* half still applies

`app::gating`'s two-mechanism rule — *refuse what is new, retire what was
already there* — is not weakened by there being no capability. A selection
made in Read survives a switch to Edit, where nothing can clear it: a click
selects an object, Escape ascends the object ladder, and the highlight sits
on the page with no gesture that answers it. That is the *"visible control,
silently inert"* failure `MODES_AND_PANELS.md` Part 1 forbids, arriving from
the other side. So `PdfcerApp::on_mode_capabilities_changed` clears it on the
way into any mode that does not offer the gesture, exactly as it clears the
object selection on the way into one that does not offer *that*.

★ **And it widened with the tool for free, because it was written against the
predicate rather than against the flag.** `app::gating` asks
`!takes_the_press(tool::selected(ctx), caps)` — the gesture's own question,
of the tool the operator actually has — rather than spelling `!caps.
edit_content` a second time. So a selection made in Read now **survives** a
switch to Edit if the text tool is armed, which is correct (the gesture that
answers it came too), and is still cleared if it is not. That comment in
`gating` predicted this in terms: *"a future `CanvasTool::Text` widens this
and the press rule together or widens neither"*. It widened both, and no line
in that file changed.
