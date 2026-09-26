# `app::actions::xobject` — the verbs whose subject is a form XObject

One verb today: **give this page its own private copy of a shared drawing**,
so that a later edit to it changes this page and no other.


## ★★★ Why this is a sub-enum file on day one, holding one variant

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
3. ★★ **The family is defined by its OPERAND, and the operand is unique in
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

## ★★★ The one fact a reader of this file must not get wrong

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

## ★★ What this file does NOT do, and where it happens instead

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
