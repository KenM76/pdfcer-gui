# `app::actions::xobject` — the verbs whose subject is a form XObject

One verb today: **give this page its own private copy of a shared drawing**,
so that a later edit to it changes this page and no other.


## Why this is a sub-enum file on day one, holding one variant

[`super::action`]'s rule is *"the next family of variants to **grow**"*, and
a family of one has not grown. Three answers, and the third is the one that
decides it:

1. **`action.rs` is at 1,441 of R2's 1,500 lines.** A variant with the doc
   comment this one needs — an operand derived through two hops, a
   granularity decision that is the engine's rather than ours, and a
   refusal ladder longer than any other verb's in this crate — does not fit,
   and trimming the prose to make it fit is the response R2's own message
   forbids: *"split the module along its seams rather than raising the
   limit"*, and the seam is not "wherever the count ran out".
2. **`super::attachments` is the precedent for arriving as a sub-enum**, and
   its header states the rule this follows: *"a family that arrives with
   three verbs at once has grown before anybody had to measure it."* This
   family arrives with one — see the next point for why it is nonetheless a
   family rather than a stray.
3. **The family is defined by its OPERAND, and the operand is unique in
   this crate.** Every other authoring verb here addresses either a
   paint-order index into one content stream (`super::vector`), a stable
   annotation `ObjId` (`super::annot`), an outline item `ObjId`
   (`super::bookmarks`), a page index (`super::pages`) or a byte span into a
   decoded buffer (`super::textstyle`). This one addresses **a form XObject
   stream object, paired with the page that invokes it** — a `(usize,
   ObjId)` whose two halves are not independent, because the verb's whole
   subject is the relationship between them. `EDITABLE_SURFACES.md` lists
   three further form-XObject verbs as open gaps; they land here, and they
   land here because of the operand, not because of the noun.

## The one fact a reader of this file must not get wrong

**The granularity is one PAGE, not one invocation**, and it is the engine's
decision rather than a simplification made here. From `unshare_form`'s own
documentation:

> If this page invokes the form under several names, **all of them** are
> re-pointed at the one copy. The unit decision 076 speaks of is the page;
> splitting two invocations *on the same page* would need a per-invocation
> identity the object model does not carry, and would make "unshare" mean
> something different depending on how many times the page happened to draw
> it.

⇒ So there is deliberately **no** variant here that takes a `TargetId`, a
paint-order index, or anything else naming *which* of a page's several
invocations was clicked. Adding one would be offering a granularity the
engine does not implement — a placeholder wearing an enum variant, which is
what `super`'s `OVERVIEW.md` forbids in as many words when it explains why
there is no `ResizeSelection`.

## What this file does NOT do, and where it happens instead

**It does not derive the operand.** The `ObjId` arrives already resolved,
from `crate::app::dispatch::format`, which reads the first leaf of the
selection and asks
`panels::objects::provider::ObjectModelProvider::containing_form_object` for
the **outermost** enclosing form. That method's doc comment carries the
whole argument for why position 0 of `FormLeaf::containment` is the only
element the verb accepts — the last element is the *innermost* form, which
is exactly the operand `EditError::FormNestedInAnotherForm` exists to
refuse.

Deriving it here instead was considered and is wrong for the funnel's own
reason, stated in `OVERVIEW.md`: an `Action` is *"a complete statement of an
operator's intent, resolvable after the frame that raised it"*. A `TargetId`
is not resolvable after the frame — an edit between the gesture and the
queue draining renumbers the page — and an `ObjId` is. The resolution
belongs on the near side of the funnel; the operand travels.

## Item notes

### `fn unshare`

# Every refusal is caught INSIDE the closure and worded


**This verb owes a sentence for every one of its refusals**, which is
unusual — `resize` words exactly one of six — and the reason is a property
of what a refusal *looks like* here rather than of how many there are:

| after a refusal, the operator sees | and infers |
|---|---|
| the page, unchanged | "it worked — the copy is identical, after all" |
| no outline moving, no colour changing | "…so I can safely type in the title block now" |

A silent decline on this command does not read as "nothing happened". It
reads as **success**, because success looks like nothing happening too. The
operator then edits a title block they still share with thirty-five other
sheets, believing they have privatised it. That is the most expensive
failure this shell can produce from one unworded branch, and it is why
`crate::text::unshare`'s sentences all end by restating that the sharing is
untouched.

Recorded from **inside** the closure rather than before the call, for
`record_resize_not_rebuildable`'s stated reason: whether the engine will
refuse is a property of the FILE — is it encrypted, is it certified, is its
`/Size` suppressing entries, is this form reached only from inside another —
and none of those is knowable from the selection the dispatcher holds. The
one refusal that *is* a query the shell can answer itself
([`UnshareRefusal::NothingInAForm`]) is recorded in the dispatcher, which is
the same placement `record_inside_form` uses and for the same reason.

# The disclosure

`UnshareFormReport` names the copy, the original and how many references
moved. The two object ids go to the **trace** and the count goes to the
**status row**, which is `canvas::textedit::report`'s rule applied
unchanged: a number about a content stream is evidence, and evidence belongs
where a driven check can read it; a count of places on the sheet in front of
the operator is a disclosure.


[`fanout`] runs **first**, before `vector_edit` is called at all, and its
two possible answers are the two possible shapes of this whole verb:

| [`fanout`] answers | this function does |
|---|---|
| `None` — no other page draws it | **nothing**, and says so in a sentence |
| `Some(measurement)` | the edit, and discloses the measured number |

Before that question existed, the verb succeeded on a form invoked exactly
once and told the operator *"every other page still shares the original"*
about a document that had no other page. Neither half of that was defensible
— a byte-identical clone, a rewritten `/Resources`, an undo entry and a
dirty document, bought for nothing, and then a false statement about their
own file. [`UnshareRefusal::NotShared`]'s docs carry the full account.

The order matters: the walk is done **outside** `vector_edit`, not inside
its closure. `vector_edit` cancels the render worker and takes `&mut` on the
session before the closure runs, so a decline from inside it would have
stopped a raster mid-flight to learn that nothing was going to happen. From
out here a decline costs one document walk and touches nothing.

### `fn fanout`

Returns the measurement the disclosure is built from, or `None` when no
other page draws it — in which case the decline is already worded and
recorded and the caller must do nothing at all.

# Why this exists: the command shipped without ever asking

"Give this page its own copy" went out on 2026-08-28 and, for one day,
**nothing in its chain asked whether the form was invoked more than once.**
`catalog/format.rs` gates the control on `selection.in_form`;
`conditions.rs` defines that as *"a leaf id is in the selection for this
page"*; `dispatch/format.rs` adds only *"the leaf resolves to a containing
form"*. And `EditSession::unshare_form` itself guards encryption,
certification, `/Size` suppression, form-not-on-page and nesting — and has
**no is-shared check**, by design: it is a verb, and a verb does what it is
told. So on an ordinary one-page CAD sheet wrapped in a single form the
engine allocated an object, privatised `/Resources`, committed an undo entry
and returned `Ok`, and the shell told the operator that every other page
still shared the original. There were no other pages.

⇒ The question is the shell's to ask, and this is where it is asked.

# THE COST, and why a whole-document walk is affordable HERE

`pdfcer_core::text_edit::invocation_set` walks **every page in the document**
and decodes every form it finds, recursively. Its own documentation is blunt
about why nothing cheaper exists: *"nothing cheaper can prove a form is not
also reached from a page the caller did not ask about."* A form XObject is
bound to no page by the standard, so the only proof of absence is a complete
scan. On a thirty-six-sheet drawing set that is thirty-six content streams
parsed and every form in them decoded — call it tens of milliseconds.

**That is fine here, and it is fine for exactly one reason: this runs once
per operator press.** A press is already a frame the operator expects to
cost something; the alternative — the edit itself — allocates an object and
rewrites a resource dictionary, which is not cheap either.

## And why the same walk must NOT go in a condition — R9

The tempting shape is to grey the control when the form is not shared. It is
wrong, and `crate::app::conditions`' own budget says why: conditions are
evaluated **on every frame**, for every command in the ribbon plan, to
decide what is enabled. Putting a document-wide page walk behind
`selection.in_form` would pay for it sixty times a second, on a document
nobody is editing, to learn an answer that changes only when the document
does. `crate::app::status::decline::Declined::FlattenCertified` already
records this ruling in the same words for a certification census: *"putting
it in the per-frame path … would pay for it sixty times a second to learn an
answer that never moves."*

⇒ So the control stays live and **answers in words when it is pressed**,
which is R9's own remedy and this project's founding rule: a refusal is a
sentence, never a silence.

# What "not shared" is measured as, and why `is_shared()` alone is not
# the test

`InvocationSet::is_shared()` is `sites.len() > 1` — *more than one `Do`
reaches this form*. That is the right predicate for the question the engine
asks with it (*will an in-place edit be visible somewhere the operator is
not looking?* — no, if every site is on the page in front of them). It is
**not** the right predicate for this verb, and the difference is a real
document rather than a hypothetical one: the engine's own corpus ships
`shared-form-twice.pdf`, one page invoking one form twice.

On that file `is_shared()` is `true`, and unsharing would move **both**
references to the copy and leave the original referenced by nothing — an
orphan object, a dirty document, and no sentence that is both true and worth
reading. So the test is the operator's question rather than the engine's:
**does any page other than this one draw it?**

| measurement | `is_shared()` | this function |
|---|---|---|
| drawn once, here | `false` | decline |
| drawn twice, both here | `true` | **decline** |
| drawn here and on sheet 12 | `true` | proceed, "1 other page" |

# An incomplete walk NEVER declines

`InvocationSet::is_lower_bound()` is true when some page's scan hit the
depth guard or a form pdfcer could not decode. On such a document the count
is a floor: pages pdfcer could not read may draw this form. Declining there
would assert *"nothing else draws it"* from a measurement that did not
finish — which is the same defect this function exists to fix, committed in
the opposite direction, and `invocation_set`'s docs name that class in as
many words: *"an under-count presented as a total is the same class of
defect as a silent edit."*

⇒ So a lower bound **proceeds**, and the disclosure it produces says *at
least* or, when it counted no other page at all, declines to name a number.
A `count() == 0` walk — the form was resolved from the page's object model
but the scan never saw it — is treated the same way and for the same reason:
the two disagree, and the honest response to a disagreement is not a claim.

# Read through `session.view()` ALONE, since engine v0.41.0

Until v0.41.0 this call took a **pair** — the object **graph** from
`session.document()` (the base revision) and the **bytes** from
`session.view()` (session-aware) — because that is the pairing
`EditSession` used at its own two internal `invocation_map` call sites.
`Pass 257.0` removed the split: a `DocumentView` now carries both halves,
every text-edit planner and helper takes `&DocumentView<'_>`, and there is
no `&Document → &DocumentView` coercion, so handing one of them a base
revision is a **compile error** rather than a latent wrong answer.

The behaviour this site wanted is unchanged and is now the only behaviour
available: a form whose stream *or whose resource objects* the operator has
already edited in this session is walked as it now stands rather than as it
was on disk. The half that silently read the base — the graph — was the
half that could disagree with the bytes, and it is gone.

### `fn refusal_for`

# A total match with a named catch-all, not a `_ =>` with a guess

Every variant the verb's own documentation names has an arm, in the order
the engine checks them, so this function and `EditSession::unshare_form`'s
guard ladder can be read side by side. `PageOutOfRange` and `PageTree` fall
to [`UnshareRefusal::Other`] deliberately rather than getting sentences of
their own: both mean the page vector moved under a queued command, and the
only honest operator-facing content for that is *nothing happened*.

It is a free function rather than a `From` impl because a `From` would
invite this mapping to be reused for another verb's errors, and it is not
reusable: `FormNotOnPage` earns a sentence about re-selecting *because this
command's operand is derived from a selection*, which is not true of every
caller the engine has.

### `fn each_documented_refusal_earns_its_own_sentence`

The failure this pins is the one that is invisible in a diff: somebody
adds an arm above, mistypes a variant name, and the compiler is happy
because `_ =>` catches it. The operator then meets *"pdfcer could not do
that"* where they should have met *"that drawing is drawn from inside
another one — use Select the form first"*, which is the difference
between a dead end and an instruction.

### `fn an_undocumented_error_falls_to_the_honest_fallback`

The catch-all is not a hole; it is the honest fallback. This pins that
it resolves to `Other` rather than to whichever arm happens to be first,
which is what a reordering accident would produce.
