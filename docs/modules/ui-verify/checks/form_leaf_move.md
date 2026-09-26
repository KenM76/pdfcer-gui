# `ui-verify/checks/form_leaf_move`

`a_thing_inside_a_wrapped_drawing_can_be_dragged` — **go inside a
container, drag what is in it, and the engine gets the edit.**

# The request

`OPERATOR_REQUESTS.md` **O70**, and before it **O53**:

> *"if the engine is capable, I should be able to select the object and do
> all of the ordinary editing one would expect a GUI editor to be able to
> do."*

## What this is the second half of

`a_click_selects_the_whole_drawing_and_a_double_click_goes_inside` proves
the operator can *reach* something inside a wrapped drawing. Reaching it and
being unable to move it is a worse state than not reaching it, because the
selection outline is a promise the gesture then breaks — and that was
exactly the state this shell shipped in for the day between the two:
`Refusal::InsideForm`, a worded decline, honest and useless.


## The oracle, and why `n=` is half of it

```text
move-leaves-in-form page=0 n=1 epoch=2 disclosures=0
```

The funnel writes that line **after** the engine returns `Ok`, so its
presence is the engine's answer rather than the shell's intent. `n=` is the
operand count, and it is asserted because the interesting wrong build is not
one that refuses — it is one that sends the **page's** index space to a verb
expecting the form's. Both are `usize`; `TargetId`'s own header names that
failure *"in range and wrong"*, and on the operator's benchmark drawing the
two lists hold 129,758 and 10,256 entries, so almost every index is valid in
both and means something different in each.

⇒ A refusal would therefore be the *safe* failure. The dangerous one moves
something, and this check's fixture is built so that the difference is
visible: it has exactly one page object (the container) and three leaves, so
an index sent to the wrong space is out of range rather than plausible.

## The sequence

| # | step | oracle |
|---|---|---|
| A | click the drawing | `selection-set … object:` — the container |
| B | double-click it | `smart-enter`, then `canvas-selection … first=leaf:N` |
| C | drag the leaf's body | `move-leaves-in-form page=0 n=1` |
| D | press Delete | `delete-leaves-in-form page=0 n=1` |

Step C presses the **bar itself**, not the middle of its bounding box.
The fixture's strokes cross, so a press at the box centre could be on a
different one — and `canvas::pressing::body_under` requires the press to
land on the selected object's own geometry before a drag is a move rather
than a marquee.
