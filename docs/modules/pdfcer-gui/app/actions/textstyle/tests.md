# `pdfcer-gui/app/actions/textstyle/tests`

## Item notes

### `fn a_size_change_reaches_the_document`

The operator's ask, reduced to its smallest true statement — press a number,
and the text on the page is that size afterwards.

Falsified by removing `StyleChange::stamp`'s `Size` arm: with nothing
stamped the request is empty, the engine returns `NoOp`, and the size read
back is unchanged, which fails here by name.

### `fn a_restyle_is_one_undoable_command`

The property that is easiest to lose and hardest to notice: a verb called
directly on the session still edits the document and still saves, and only
`Ctrl+Z` tells you it was wrong.

### `fn a_restyle_bumps_the_edit_epoch`

Its own test rather than an assertion inside the one above, because it is
a different failure: an edit that lands in the file and does not bump the
epoch shows the operator the *old* size in the panel for ever after, and the
page they are looking at is right while the numbers beside it are wrong.

### `fn a_multi_run_selection_restyles_every_run`

The case the descending-order argument exists for. It is asserted on the
*last* run as well as the first, because an implementation that restyled
only the head of the list would pass a test that looked at the head.

### `fn an_empty_selection_edits_nothing`

The guard that stops an empty gesture reaching the engine at all. Asserted
on the epoch rather than on the decline, because "nothing happened to the
document" is the claim that matters and the sentence is `text::status`'s
business.

### `fn an_out_of_range_run_edits_nothing`

The failure this guards is the expensive one: an out-of-range ordinal that
fell back to "the first run whose text matches" would restyle a piece of
text the operator never selected, in a file they then send to somebody.

### `fn bold_binds_a_real_standard_face_on_a_page_with_no_bold_face`

One thing: that `doc.edit_epoch` moved. Its own comment argued the point —

> Asserted by the *absence of a refusal* and the presence of an edit
> rather than by reading a "synthetic" flag out of the file: `R90`'s
> synthesis is deliberately not recorded in the PDF — it is re-detectable
> from the bytes, which is a different question from the one this test asks.

— and that reasoning is still correct about `R90`. It is the wrong
conclusion, because **the epoch moved on the bad outcome too**. This fixture
carries `Helvetica` and nothing else. For the five days between `Pass 179.0`
shipping and this shell calling it, pressing Bold here thickened the strokes
of `Helvetica` while `Helvetica-Bold` — a face every conforming reader is
required to carry, needing no font file — was one resource away. This test
was green throughout. A test equally green on both outcomes is not evidence
about which one shipped.

# What separates them, and it IS in the file

The synthesis argument is right that `R90` leaves no marker: a faux bold is
`Tr 2` plus a stroke width, and the run keeps its original `/Font` resource
key. Rung 2 does the opposite — it **authors a new `/Font` resource** for
the standard-14 sibling and points the run's `Tf` at it.

⇒ So the observable is the resource key, read back through
`pin::inspect`. Unchanged key ⇒ the letters were faked. Changed key ⇒ a real
face was bound. That is exactly the distinction the operator sees on a
plotter and exactly the one the old assertion could not make.

**Falsified, not assumed:** reverting `StyleChange::stamp`'s `Weight` arm
to `req.synthetic(…)` turns this red on the second assertion while the first
stays green — which is the whole finding, reproduced on demand.

### `fn bold_takes_the_covering_real_face_on_a_page_that_has_one`

`textedit/format_family.pdf` is a `/Times-Roman` run `hello world` on a page
that also carries `/F2` (`Calibri-Bold`, fully covering) and `/F3`
(`Times-Bold`, whose `/Differences` remaps `o` to `/bullet`, so it does NOT
cover the run).


Asking for synthetic bold was refused by `gate_synthesis` — *"a REAL bold
face is available"* — **naming `Times-Bold`**, because it matched the run's
family. This module took that offer and the offer was refused for coverage.
So **bold was unreachable on that page through either verb**, while
`format-text --set-font F2` succeeded from the command line throughout.

It was written as a **characterisation** test, with the engine revision
named, and it closed with a prediction:


## The prediction came true the same night

`Pass 144.0` (`cfa2c44`): *"the face a synthesis refusal names is now one
`set_font` accepts."* The engine reproduced all three commands on the
release binary before accepting the report, found the cause —
`gate_synthesis` decided *"a real bold face is available"* from **two string
tests on `/BaseFont`** and never asked whether the face could show this
run's characters — and moved the four acceptance conditions into one shared
predicate so a gate and a commit cannot disagree.

Their own note on it is worth keeping: it is R221's third instance — *a
predicate deciding whether a capability applies, written by hand at a
different call site as a parallel description of when the real function
succeeds* — and it **inverts** the usual risk analysis, because a false
positive here removes the capability entirely rather than costing a slow
path or a wrong pixel.

# What is asserted NOW

The three things that were the point all along, with the first two flipped
from "nothing happened" to "the right thing happened":

1. **Bold reaches the run**, and the face it lands on is one that can
   actually show `hello world` — which on this page is `/F2`, not the `/F3`
   the old gate named;
2. **the epoch moves**, because the page really changed;
3. **nothing is declined**, because nothing refused.

The **face is asserted by name**, not merely "something changed". A build
that fell back to synthesis would also change the run and bump the epoch,
and would be a *worse* answer on a page that carries a covering real face —
so a test that only checked for movement would pass on the second-best
outcome.

### `fn an_engine_decline_with_no_discriminant_names_no_cause_and_promises_no_remedy`

`reflow_refusal` mapped **every** `ReflowApplyError::Unsupported` to
`ReflowRefusal::PageSetChanged`, which told the operator *"pages have been
added, removed or reordered since. Save this file and open it again."*


⚠ **3,865 tests were green throughout.** Nothing here touched
`reflow_refusal` at all — it was a `fn` with no caller in `#[cfg(test)]` —
so the arm was free to describe an engine that no longer existed.

### `fn a_named_cause_comes_from_a_named_engine_variant`

`Encrypted` is the control here and it is the reason this test is worth
writing: it proves the mapping CAN carry a specific cause, so the general
answer above is a considered choice rather than the only thing that works.

### `fn the_one_recoverable_refusal_keeps_its_remedy`

⚠⚠ **Read the name of this test as historical.** At engine `025d703d`
`PageEditedThisSession` is no longer recoverable, because it is no
longer PRODUCED: `G015` deleted the guard that constructed it, and the
engine kept the variant on purpose so that dropping it would be this
project's decision. The test is kept and still constructs the error by
hand, which is the only way to reach the arm now — and that is exactly
why it is kept. If a future engine reinstates the guard, the arm must
already be right; a mapping deleted because it was briefly unreachable
is how the remedy would be lost a third time.


> *"any `match` of yours ending in `_` just gained a variant it will not
> distinguish, and the one it will not distinguish is the one you care
> about."*

Exactly right. `ReflowApplyError` is `#[non_exhaustive]`, so the new variant
produced **no compile error** — it would have fallen into
`_ => ReflowRefusal::Other` and the only recoverable refusal in the whole
set would have lost its remedy for the **second time in one day**, silently.

⇒ The wildcard is gone; everything but `Encrypted` routes through
`ReflowApplyError::decline()`, and `ReflowDecline` is deliberately not
`#[non_exhaustive]`, so that match is compiler-proved complete. This test is
the belt to that braces: it asserts the *outcome* the operator gets, which a
future refactor could break without touching the enum.

### `fn each_engine_decline_reaches_a_refusal_that_suits_it`

This is the test that would have caught the original defect on the day
`Pass 257.0` landed. It walks **every** `ReflowDecline` — the type is not
`#[non_exhaustive]`, so this list cannot silently fall behind — and asserts
the four are not all the same answer, which is the whole reason the
discriminant was asked for.

### `fn a_composite_font_refusal_says_it_is_the_font`

# Why this test exists, and what it is really guarding

`O198`: the operator's 36-sheet SOLIDWORKS drawing sets its body text in
`AQHZBV+CenturyGothic`, a composite (Type 0 / CIDFont) face. Within-block
reflow of composite text is a deferred engine feature (`R-INV-4`, FF-E), so
**every** reflow he attempted on that sheet was refused — and until
2026-09-14 the sentence he got was [`ReflowRefusal::EngineDeclined`]'s
*"something about how this page was drawn stops it doing so safely"*. True,
honest, and useless: it gave him no way to know that trying the paragraph
next to it was pointless for exactly the same reason.

# The assertion that matters is the SECOND one


*A tripwire keyed on the other side's data survives the other side
changing; one keyed on our reading of it does not.*
