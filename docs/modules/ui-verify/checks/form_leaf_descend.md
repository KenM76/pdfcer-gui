# `ui-verify/checks/form_leaf_descend`

`the_ladder_goes_as_deep_inside_a_container_as_outside_one` — **double-click
past the object, into its parts, and drag one.**

# The request

`OPERATOR_REQUESTS.md` **O70**:

> *"a double click should bring me further down the chain, until a double
> click reaches the bottom and lets me edit the nodes."*


## ★★ What had to change together, and why the order was forced

Four things, and doing any one alone would have made the shell worse:

| | before | after |
|---|---|---|
| the hit test | `probe` mapped to a page index, so a leaf got `(None, None)` | asks `part_hits_of` with the `TargetId` |
| the descent | refused, by a guard, so the box stayed put | descends |
| the anchors | declined `leaf-in-form-xobject` | drawn |
| the drag | `Refusal::InsideForm` | `move_subpath_in_form` |

⇒ Descending without the other three would have entered a rung with nothing
addressable in it: no anchors drawn (`canvas::painting` declined), no
outline (`pressing::grabbable` withholds it below the Object rung), and a
drag that refuses. The operator's second double-click would have made the
selection **vanish** and offer nothing in its place. That is why the guard
existed for the day between the two halves, and why this check exists now.

## The sequence

| # | step | oracle |
|---|---|---|
| A | click, then double-click | `smart-enter`, `first=leaf:N` |
| B | double-click again | `canvas-selection … level=Part` |
| C | drag | `move-subpath-in-form page=0 n=1` |

★ Step B's oracle is the **rung**, not the trace of a click. `level=Part`
on a `first=leaf:` selection is the one statement that says the ladder went
deeper in the space that had no deeper rung — and a build with the descent
guard back in place reports `level=Object` while every other line looks
identical.
