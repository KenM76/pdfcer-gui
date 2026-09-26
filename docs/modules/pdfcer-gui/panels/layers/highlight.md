# `pdfcer-gui/panels/layers/highlight`

`panels::layers::highlight` — which layer the current selection is on.

The operator's ask, verbatim: *"selecting an object highlights that
layer"*.

# THE ENGINE ANSWERED. This file is the second half arriving.


> `vector::decompose`'s walk counted `/OC` sections into
> `DecomposeDiagnostics::oc_sections` and threw the group identity away;
> `pdfcer-render`'s interpreter resolved the same reference and pushed a
> `bool`. Two places knew which layer an object was on, and neither could
> say.

**The workaround was refused, and the refusal is what produced the
capability.** This shell *could* have re-tokenized the page with
`ContentStream::from_page`, kept its own `/OC` stack and indexed by
`VectorObject::tokens()` — about forty lines of public API. It was refused
because it would have been **a second implementation of `/OC` resolution
beside the engine's**, blind to OCMD `/VE` visibility policy, forking a
`pub(crate)` helper, and destined to disagree with the renderer on some
file nobody would ever debug. A request was filed instead. `Pass 250.0`
answered it **within hours**: `oc: Option<ObjId>` now sits on
`PathObject`, `TextObject` and `ImageObject`, read through
[`pdfcer_core::vector::VectorObject::oc`] and
[`pdfcer_core::vector::FormLeaf::oc`].

⇒ **A shell that nearly resolves optional content ships a defect and
silences the request that would have fixed it.** That is the transferable
half, and it is now this project's third instance.

# What the engine gives, verified at `pdfcer-core` v0.38.0 (`b01964f`)

| fact | `file:line` in `crates/pdfcer-core/src/vector/decompose.rs` |
|---|---|
| `PathObject::oc` | `386` |
| `TextObject::oc` | `484` |
| `ImageObject::oc` | `780` |
| `VectorObject::oc()` | `1064` |
| `FormLeaf::oc()` | `1300` |
| `DecomposeDiagnostics::oc_sections` | `1165` |
| `DecomposeDiagnostics::oc_unresolved` | `1171` |
| the walk that fills them (`current_oc`) | `1485` |
| `Annotation::oc` (the half that already worked) | `annot.rs` |

And the contract on each `oc` field, quoted because this module's whole
three-valued design turns on it:

> *"`None` means the object is on NO layer; it does NOT mean 'could not
> tell' (see `DecomposeDiagnostics::oc_unresolved`). Membership only: an
> OCMD is reported as its own `ObjId`, never expanded, and visibility is
> NOT resolved here."*

# THE TWO DIVERGENCES THE SECOND ROUTE FOUND

This project's standing finding is that **adding a second route to a
capability audits it**. Building the page-object route beside the
annotation route found two places where the engine's answer is a *partial*
that its own type cannot express, because `Option<ObjId>` has no third
value:

### D1 — a form leaf does not inherit the `/OC` its `Do` was painted under

[`pdfcer_core::vector::FormLeaf::oc`] delegates straight to the wrapped
object, and the engine's own doc comment names the gap: *"A page-level
`BDC /OC` enclosing the form's `Do` is NOT folded in here … a documented
partial for the nested case."* `collect_form_leaves` has `img.oc` in hand
— the enclosing form object's own membership, correctly resolved one line
earlier — and does not pass it down.

**So a leaf inside a form on layer *Grid* reports `None`, which the field
contract defines as "on NO layer".** That is not a missing answer; it is a
*wrong* one, and it is the exact failure the operator's bar forbids:
highlighting nothing while asserting a fact.

⇒ **This module repairs the depth-1 case from the engine's own two
answers** — see [`for_leaf`] — by consulting the enclosing form object at
[`pdfcer_core::vector::FormLeaf::paint_order`]. That is composition of two
engine results, **not** a second `/OC` implementation: no content stream is
re-tokenized and no `/Properties` key is resolved here. At depth **> 1** an
intermediate form's own membership is unreachable — the nested form
container is deliberately dropped from `leaves` — so the honest answer is
[`Unresolved::NestedForm`] and it is stated rather than guessed.

### D2 — `oc_unresolved` does not count what happens inside a form

`collect_form_leaves` bumps `form_cycles` and `form_depth_overflows` on the
page's diagnostics and **discards `nested.diagnostics` entirely**. So the
counter the engine names as *"how a shell tells the two apart"* is blind to
every form interior. A leaf under an unresolvable `BDC /OC` reports `None`
with nothing anywhere to contradict it.

⇒ Not worked around. Honouring the page counter for leaves as well is the
best available reading and is what [`for_leaf`] does; the residual is
**reported**, on the `ENGINE_BACKLOG.md` row and in the request follow-up,
rather than absorbed. Making every form-interior object [`Unresolved`]
instead was considered and rejected: it would retire the feature on exactly
the CAD drawings it was built for, in exchange for a case the engine
measured at 0.6 % of files carrying optional content at all.

# Why the answer is FIVE-valued, which is the whole design

The obvious type is `Option<ObjId>`, and it is wrong here in a way that
matters more than usual.

| | `Option` says | the truth |
|---|---|---|
| a stamp with no `/OC` | `None` | **on no layer** — a fact the engine established |
| a leaf three forms deep | `None` | **not known** — D1 above |
| two objects on two layers | one of them | **neither, alone** |

Collapsing those makes the panel unable to distinguish *"this mark is on no
layer"* from *"nobody can tell you"* from *"your selection spans two"*, and
the operator reads every one of them as the first. The operator's own note
on this feature is the bar: **highlighting the wrong layer is worse than
highlighting none** — and asserting "on no layer" about an object whose
layer is merely unknown is the same class of wrong answer, arriving as
silence instead of as a highlight.

⇒ [`Membership`] has five variants and they form a **join semilattice**,
so a multi-object selection folds without anybody writing an order down
twice. See [`Membership::join`].

# The operator's own finding: THE UNIT OF SELECTION IS NOT HIS

He measured it on his own drawing: **one PDF path object holds 6,681
anchors across half his sheet** (`RESUME.md`; `pdfcer object-list` on
`SW41177.pdf` p1 reports 4,405 / 4,972 / 6,681, the largest holding 1,194
subpaths across 550 × 500 pt).

`/OC` membership is a property of a **marked-content section**, which wraps
*paint operators*. A `BDC /OC` cannot begin in the middle of a subpath, so
every subpath and every anchor of one `PathObject` shares that object's
membership by construction — the relation is exact at object granularity
and **has no finer form to be exact at**.

⇒ So descending to the Part or Point rung does not refine the answer, and
this module deliberately ignores [`crate::canvas::selection::Selection`]'s
`subpath` and `node`. What that costs the operator is real and is stated
off-canvas rather than implied: the thing he *thinks* he selected — the
circle he clicked — may be one of 1,194 subpaths in an object that spans
two title blocks, and the layer named is the **whole object's**. See
[`crate::text::panels::layers::layer_selection_granularity`], which the
panel shows whenever the selected object holds more than one part.

# Rule 4 — this is DISCLOSURE, and none of it touches the canvas

Nothing is drawn differently on the page. No badge, no tint, no dashed
outline, no provisional layer painted over the selected content. The
selection handles are the *cursor* and are unchanged. Every statement this
module produces lands in two off-canvas places:

| surface | what it says |
|---|---|
| the Layers panel row | a background plate on the layer the selection is on |
| the status bar's selection line | the layer's name, as a clause on the line that already names the object |

The second exists because **the canvas is the primary surface, never a
panel.** The engine can now answer *"which layer is this on"*, so clicking
the object must be able to reach that answer with no panel open. A panel is
a supplement.

# The reverse relation, deliberately not built

*Does clicking a layer indicate its objects?* Now derivable — a scan of
`PageObjects::objects` filtering on `oc()` — and still not built here,
because indicating them means **marking the canvas**, which Rule 4 forbids
for an inference. The shape it would have to take (a count, off-canvas, in
the row's tooltip) is a separate decision and a separate landing.

## Item notes

### `fn for_target`

Both index spaces are resolved **by the [`TargetId`] itself** rather than
by a caller that had to remember which list it was holding — the same
discipline `app::status::selected` uses, and for the same reason: the two
spaces are both `u64` and a mix-up is silent.

### `fn index_of_bbox`

Chosen by **geometry**, never by `oc()`. Picking the object by the
very field under test would make the assertion circular — it would
prove that `resolve` returns what `oc()` returns, which is a restatement
rather than a test.

**Exact bounds and not "contains this point", and the difference is
a defect this helper already had.** The first version took a point and
took the FIRST object covering it, which is how
`an_object_outside_every_section_is_on_no_layer` came to select the
fixture's `0 0 300 792 re W n` **clip path** — a real `PathObject`, on
layer *Clip Only*, whose bbox covers most of the page and which is
painted before the grey bar. The test failed with `Group(6)` where it
expected `None`, and the report would have been *"the shell reports the
wrong layer"* about a shell that was right and a helper that was
pointing at something else.

⇒ *Ask what the test SAMPLED before asking what is broken.* A rectangle
names one object; a point names whichever of several the helper's
tie-break happened to reach.

### `fn selecting_a_page_object_names_the_layer_it_is_painted_on`

The operator's ask, in a headless test: click the square inside
`/OC /L1` and the answer is *Visible Box*. Before `Pass 250.0` this was
unanswerable and this module returned `Unknown` for every content
object.

### `fn an_object_outside_every_section_is_on_no_layer`

This is the half a build cannot fake. Everything in the test above
passes against an implementation that ignores the selection and always
answers with the first group in the document; nothing here does.

### `fn the_innermost_section_is_the_layer_not_the_outer_one`

The square at (400,220) sits inside `/OC /L2 BDC … /OC /L4 BDC … EMC
EMC`. Answering *Hidden Box* — the outer section — would be a wrong
highlight rather than a missing one, and the operator's own bar rates
that worse.

It is selected directly rather than clicked, because both L2 and L3
are in the document's `/OFF` array: this object is in the model and is
not drawn. The membership relation is what is under test, not the
visibility one.

### `fn a_one_part_object_says_nothing_about_its_parts`

It is a measurement, not a disclaimer: the sentence exists because one
object on the operator's own drawing holds 1,194 subpaths, and a
warning printed on every single-part rectangle would teach him to skip
the line before he ever met one that mattered.

### `fn the_driven_checks_two_aim_points_land_where_it_thinks`

# Why a unit test owns a driven check's coordinates

Because a miscalibrated aim is this project's most expensive harness
failure and it is **silent**: the check clicks, something is selected,
an answer comes back, and the report is an articulate paragraph about
the wrong object. `RESUME.md` records it three times in one day, and
once as *"a wrong aim that happens to hit is a green result reporting
nothing"*.

This test asks the engine's own deep hit test — the one the canvas
uses — what is under each point, and pins the answer. It runs in the
sweep with no window open, so the driven check can never be quietly
aiming at something else.

# What it established, and it was not obvious

The fixture's `0 0 300 792 re W n` **clip path** decomposes into a real
`PathObject` on layer *Clip Only* whose bbox covers `(0,0)..(300,792)`
— so it sits under BOTH aim points geometrically. It is **not** a hit
candidate, because `n` paints nothing and `hit_test_point_deep` answers
with ink rather than with bounds. Measured, not assumed: the sibling
helper `index_of_bbox` had already been caught selecting that very
object by bounds.

### `fn not_on_a_layer_is_not_the_same_answer_as_cannot_tell`

If this ever fails to compile because the two were merged, the panel
has lost the ability to distinguish *"this mark is on no layer"* from
*"nobody can tell you"* — and the operator reads the second as the
first.

### `fn two_reasons_are_two_answers`

Two `Unknown`s with different causes must not compare equal, or the
panel could print one reason while holding another and no test would
see it.

### `fn only_a_known_group_highlights_anything`

The operator's bar, made mechanical: highlighting the wrong layer is
worse than highlighting none, so every state that is not a *positively
established* group must highlight nothing.

### `fn the_trace_vocabulary_separates_every_state`

`ui-verify selecting_an_object_names_its_layer` matches on `answer=`,
so these strings are a harness contract rather than decoration. Two
states sharing a word would make the check unable to tell them apart —
and the pair that matters is `no-layer` against `unknown`, which is the
whole distinction this type exists for arriving in the diagnostic
channel.

The empty-string assertion is about line SHAPE: a field whose value is
empty puts two spaces where every other line has one, and a parser that
gives a space structural meaning is entitled to read that differently.

### `fn an_unresolvable_section_demotes_no_layer_to_cannot_tell`

This is the engine's own contract consumed: *"`oc_unresolved` … is how
a shell tells the two apart"*. Without this arm an unnameable group
renders as the positive claim *"not on a layer"*.

### `fn a_leaf_one_form_deep_inherits_the_forms_layer`

`FormLeaf::oc()` delegates to the wrapped object and the engine's own
doc comment calls the omission *"a documented partial"*. Without this
arm, everything inside a form on layer *Grid* reports **"on no
layer"** — a wrong positive, not a missing answer.

### `fn a_leaf_two_forms_deep_is_not_guessed_from_the_outer_one`

The intermediate form's own `/OC` has no representative in
`PageObjects` — `collect_form_leaves` drops nested containers — so the
outermost form's group is *not* evidence about this leaf. Answering
`Group(4)` here would be the exact wrong-highlight the operator's bar
forbids.

### `fn a_layered_and_an_unlayered_object_are_mixed`

This is the marquee case on the operator's own drawings and the one a
naive `find_map` implementation gets wrong: it would report the first
group it saw and light a row, claiming a whole selection is on a layer
half of it is not on.

### `fn an_unknown_outranks_even_an_established_disagreement`

The first draft asserted `Mixed ⊔ Unknown = Mixed`, on the reasoning
that *"this selection spans several layers"* is a fact an unanswerable
third member cannot take back. The reasoning is sound and the rule is
**not associative** — `the_fold_does_not_depend_on_selection_order`
found it within a minute of being written, with the message
*"join is not associative: Group(1) Group(2) Unknown(Stale) — left:
Mixed, right: Unknown(Stale)"*.

⇒ A highlight that depends on the order the operator added objects to
the selection is a highlight that flickers for reasons nobody can
diagnose. `Unknown` is the top. See [`Membership::join`].

### `fn two_reasons_merge_by_priority_rather_than_by_position`

`Unknown(a) ⊔ Unknown(b)` taking the left operand would be
non-commutative, which is the same order-dependence one level down —
and invisible to any property test whose sample set held a single
`Unknown`, which the first one did.

### `fn the_fold_does_not_depend_on_selection_order`

Exhaustive over a representative set rather than argued in prose: a
fold whose result depended on selection order would produce a highlight
that flickered between two rows as the operator added objects, and no
single-pair test would catch it.
