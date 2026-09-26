# `text::dimension_groups` — the words the Manage-groups window shows

## Rule 15, and why this catalog says "the dimensions you draw"

A **ce dimension** is one pdfcer authors. A **pdf dimension** is CAD-exported
page content pdfcer reads and must not silently alter. The distinction is
ours, not the operator's, and it has already sent one investigation down the
wrong path — so this catalog does what [`crate::text::scale`] does and avoids
the bare word entirely. On screen it is *"the dimensions you draw"*, which is
unambiguous without asking a drafter to learn a term that exists for our
benefit.

## ★ The hardest thing this window has to explain

Not what a group *is* — a drafter already has that idea from every CAD
package they have used. What is genuinely new is that **a group edit reaches
backwards**: setting a scale, a standard or an appearance default rewrites
every dimension already placed in that group, wherever it is, including on
pages that are not on screen.

The operator named the fear himself, quoted in the ui-spec: *"cannot change
one and be surprised 40 others changed or didn't."* So every group-level
control in this window is accompanied by a **count of what will move**, and
the count is computed before the edit rather than reported after it.

## ★ And the number in that count is NOT the engine's return value

`EditSession::set_group_style` returns the number of members **regenerated**,
which is every wired member — including the ones that override the property
being changed, because regenerating an overrider is byte-identical and free
in the diff. Showing that number would be worse than showing none: it is a
real number, plausibly labelled, answering a different question, and an
operator who reads *"40 dimensions will change"* and sees three change has
been misled by a fact.

[`members_that_will_move`] therefore takes the count the *caller* computed
from `StyleProvenance::follows_group()`, and the caller computes it before
the edit is applied. See `dialogs::dimension_groups::style`.
