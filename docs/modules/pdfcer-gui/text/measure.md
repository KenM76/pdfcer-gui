# `text::measure` — what the measure tools say about what they inferred

## Rule 15

What these tools author is a **ce dimension** — one pdfcer writes. A **pdf
dimension** is CAD-exported page content pdfcer reads and must not alter.
This catalog avoids the bare word, exactly as [`crate::text::scale`] and
[`crate::text::dimension_groups`] do.

## Why this module exists at all

Because the two-line tool's output is an **inference**, and the shell was
swallowing every statement about it.


| fact | what it means | what happened without it |
|---|---|---|
| `TwoLineRefusal` | collinear or zero-length lines — refused **by name** | the second click did nothing, silently, and the operator clicked again |
| `measured_angle_degrees` | the true angle, reported **even when forced parallel** | a checkbox that hides the number it is overriding |
| `apex_is_real() == Some(false)` | the lines meet only if extended | ordinary in CAD, and a fact the operator may not have noticed |

`docs/core-api/03-capabilities.md` §1.5 obligation 4 states the second of
those and quotes the engine's own reason: *"a checkbox that hides the number
it is overriding is withholding the fact that makes the decision a
decision."*

## Why the refusal is worded HERE and not taken from the error's `Display`

`TwoLineRefusal`'s `thiserror` messages are written for an operator, and the
capabilities doc says to surface them. It is still not right to print them:
`check-ui-strings.sh`'s exclusion 3 says in as many words that an error
type's `Display` output being exempt *"is not permission to route UI text
through an error type"*.

So the variant is matched and the sentence lives in this catalog — which is
the same resolution [`crate::text::markup::deleted_collateral`] reached for
`DeletionReport`. The **refusal is still surfaced by name**, which is what
the obligation actually asks for; what changes is which crate owns the
English, and that is decision 002 R1.

## Item notes

### `fn a_forced_parallel_reading_states_the_angle_it_overrode`

The one assertion this module exists for. A build that says "read as
parallel" without the angle has a checkbox hiding the fact that makes
the decision a decision — `pdfcer-core`'s own words, and the reason it
populates the field precisely when the override fires.

### `fn two_line_refused`

Both sentences say **what to do next**, not only what went wrong. A
refusal an operator cannot act on is a dead end with better manners: the
gesture is still armed, both picks are still there, and the corrective is
one more click on a different line.

### `fn two_line_reading`

Returns `None` when there is nothing an operator could not already see: an
ordinary angular reading of two lines that genuinely cross, with no
override. A sentence on every commit is a sentence nobody reads, and it
would bury the two cases that matter.

The three cases it does speak for:

1. **Forced parallel.** The operator ticked the override and the lines are
   *not* parallel. The number they overrode is stated — that is the whole of
   obligation 4, and the engine populates `measured_angle_degrees`
   **especially** in this case for exactly this sentence.
2. **A virtual apex.** The lines meet only if extended. `pdfcer-core` is
   explicit that this is *"perfectly ordinary … so it is not refused; but it
   is a fact about the geometry the operator may not have noticed"*.
3. **Both.**

# Why it is a status-line sentence and not a mark on the drawing

Rule 4. The authored ce dimension renders exactly as a saved one will —
no tint, no badge, no dashed line saying "this apex is imaginary". The
inference is disclosed **off-canvas**, which is what this function is for,
and the canvas is left alone.
# Why it takes three facts and not the `TwoLineAuthoring`

The catalog convention — [`crate::text::markup::deleted_collateral`] takes
four primitives out of a `DeletionReport` for the same reason — and here it
is forced as well as conventional: `TwoLineRelation` is not publicly
re-exported from `pdfcer-core`, so a `TwoLineAuthoring` cannot be built in a
test in this crate at all. A wording function that cannot be tested without
a running gesture is a wording function nobody tests.

The caller does the extraction, in one place, at the one call site.

### `fn vertex_remeasured`

# Why BOTH numbers, when one of them is on the page

Because the operator can see the new one and cannot see the old one — the
geometry it was measured from no longer exists. A line reading `13.85 m`
tells them nothing they did not already have; `12.40 m → 13.85 m` tells them
what their drag cost, which is the only fact here they cannot recover.

The engine carries `previous_label` on `VertexOutcome` for exactly this, and
it is the reason this sentence can exist at all: nothing on this side could
reconstruct it afterwards.

# Why it is off-canvas, and why that is the RULE rather than a preference

Rule 4, as narrowed by decision 059: the reshaped dimension and its new
label render **exactly as they will render after Save** — no tint, no badge,
no "recently changed" marking on the page. The change is stated here, on the
status row, where it does not become a second rendering path for the same
content. Two rendering paths drift; a sentence does not.

# And why it is silent when the number did not move

The caller drops this when `previous == current`. A corner dragged along its
own segment changes the shape and not the length, and reporting `13.85 m →
13.85 m` would train the operator to ignore the one line that matters when
it does move.

### `fn vertex_inserted`

Same obligation as [`vertex_remeasured`] and one fact more. Adding a corner
re-measures the shape, so the two labels are owed for that function's
reason: the operator can see the new number and cannot see the old one,
because the geometry it was measured from no longer exists.

# Why the COUNT is in the sentence and the move's is not

Because the count is what the operator asked to change, and it is the one
thing a mis-aimed gesture gets wrong *invisibly*. A corner dropped on the
wrong segment still looks like a corner; a shape that now has seven corners
when the operator meant to add one to six is a fact only a number can
carry. A drag that MOVES a corner cannot change the count at all, which is
why its sentence does not carry one — a constant in a status line is noise
that trains the operator to stop reading it.

# Why it is off-canvas

Rule 4 as narrowed by decision 059, verbatim from [`vertex_remeasured`]:
the reshaped ce dimension renders exactly as it will render after Save — no
tint, no badge, no "recently changed" marking — and the change is stated on
the status row, where it does not become a second rendering path for the
same content.

### `fn vertex_removed`

> *"I also can't edit or delete nodes of a markup shape once it is drawn."*

Worded as its own sentence rather than sharing one with the insert, because
the two are the acts an operator most needs to tell apart after the fact:
both change the shape, both change the number, and a single sentence
reading "the corners changed" would leave them checking the drawing to see
which happened.

### `fn line`

Each names what is true rather than what the engine called it, and the
first names a remedy — which is `resize_not_rebuildable`'s rule and the
one `node_tool_needs_edit_mode` states: at the moment it is read the
operator has just released a drag and seen nothing happen, and what
they need is the next act, not a diagnosis.
