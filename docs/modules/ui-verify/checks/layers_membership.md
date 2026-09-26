# `ui-verify/checks/layers_membership`

`selecting_an_object_names_its_layer` — **clicking a page object says which
optional-content group it is on, and clicking one on no layer says that
instead.**

# ⚠ THIS CHECK HAS NOT BEEN RUN


**Do not read a green suite as evidence for this row until it has been
driven once.** Say so in whatever report cites it. A check nobody has run
is a specification, not a verification, and this project's founding rule is
that the two are not the same thing.

# The claim

`OPERATOR_REQUESTS.md` O126, in his words: *"selecting an object highlights
that layer"*.

Until `pdfcer-core` v0.38.0 only an **annotation** could be answered for —
`vector::decompose` counted `/OC` sections and discarded the group id.
`Pass 250.0` put `oc: Option<ObjId>` on `PathObject`, `TextObject` and
`ImageObject`, and this check is the driven half of consuming it.

# ★★★ Why the fixture is pinned, and it is the vacuous-pass argument

This check **ignores `--pdf`** and says so in its notes when one was
supplied. Its subject is a *relation between two things*, and a document
cannot exercise it unless it carries both:

| the fixture must have | or the check |
|---|---|
| **at least two** optional-content groups | passes under a build that highlights a constant |
| **at least one object on no layer at all** | passes under a build that highlights whatever the first layer is |
| the two reachable at **different points** | cannot separate "the answer follows the selection" from "there is one answer" |

★★ The middle row is the one that would have been missed. A fixture whose
every object is on the same layer makes *"the highlight follows the
selection"* true of a build that ignores the selection entirely, and the
check would go green while measuring nothing. That is this project's
standing failure shape — *ask what the check SAMPLED before asking what is
broken* — and the answer here is: two points, two different answers, one of
which is an established absence.

`layers/painted-layers.pdf` in the engine's read-only synthetic corpus
carries exactly that, in fourteen objects of hand-written syntax:

```text
/OC /L1 BDC  0 0 0 rg 60 60 120 120 re f  EMC        <- "Visible Box"  (obj 4)
/OC /L2 BDC  0 0 0 rg 400 60 120 120 re f            <- "Hidden Box"   (obj 5, /OFF)
  /OC /L4 BDC 0 0 0 rg 400 220 120 120 re f EMC      <- "Nested Inner" (obj 7)
EMC
/OC /L3 BDC  0 0 300 792 re W n EMC                  <- "Clip Only"    (obj 6, /OFF)
0.5 g 0 600 612 60 re f                              <- ★ ON NO LAYER
```

# The two clicks, and why those two

| # | point (PDF user space) | what is there | the assertion |
|---|---|---|---|
| 1 | `120, 120` | the *Visible Box* square, 60→180 in both axes | the answer is `group` and names **Visible Box**, and **exactly one** panel row carries the highlight |
| 2 | `150, 630` | the grey bar, drawn outside every `BDC` | the answer is `no-layer`, and **no** row carries the highlight |

★ Neither point is on a layer the document turns **off**. `/OFF` names L2
and L3, and both are avoided deliberately: an object the renderer does not
draw is still in the object model, so a click there selects something
invisible and a failure report about it would need a paragraph before it
could be read. The subject here is the membership relation, not the
visibility one.

# ★★ The two oracles, and why one would not do

| oracle | what it proves | what it cannot see |
|---|---|---|
| `layer-membership … answer= name=` (status bar) | the shell *computed* the right answer, with no panel open | whether anything is drawn |
| `layer-row … name= highlighted=` (Layers panel) | **which row is lit, and that only one is** | nothing, if the panel is closed |

The first is the **canvas route** — the answer reachable by clicking, with
no panel open, because *the canvas is the primary surface, never a panel*.
The second is the operator's literal word, *"highlights"*. A check reading
only the first would pass against a build that computed the answer and drew
nothing; one reading only the second would pass against a build whose only
route to the answer is a panel the operator has to know to open.

# ★★★ Rule 4 is asserted, not assumed

Both oracles are **off-canvas** by construction: a status-bar line and a
panel row. There is deliberately no assertion about the canvas here,
because there is deliberately nothing on the canvas to assert about — no
badge, tint, dashed outline or provisional layer is drawn over the selected
content to express its membership. If a future change adds one, this check
will not catch it, and that is worth saying out loud rather than implying
coverage the file does not have.

# What this check does NOT cover, stated rather than implied

* **The form-leaf path.** `for_leaf`'s repair of the engine's D1 partial
  (a leaf inheriting the `/OC` its `Do` was painted under) is held by unit
  tests only. It needs a fixture with a form XObject invoked from inside a
  `BDC /OC` section, and no such file exists in either corpus.
* **The multi-object fold.** `Membership::join` is unit-tested exhaustively
  for commutativity and associativity; no driven marquee exercises it.
* **The search-narrowed row.** The sentence shown when the highlighted
  layer has been filtered out of the list needs typing into the search
  field, which `layers_search`'s own header explains this harness has no
  seam for.
