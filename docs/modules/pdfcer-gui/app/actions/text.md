# `app::actions::text` — the verbs that re-shape a page's own text

Every variant here has the page's existing text as its subject: the caret
commit, the free-text commit, the restyle and the reflow.

## The verbs compose, and that is asserted rather than stated

`CommitTextEdit`, `TextStyle` and `Reflow` all **accumulate**: they stage
onto the session, and a page may take twenty of them. Add text to a page,
then re-wrap a paragraph on it, and the add survives.

⚠ **Do not restate that as a prose limitation.** Whether reflow composes
with the other verbs is a property of a branch dependency, so a sentence
here is a citation of something that moves with no command run in this
repository — and a citation nobody re-measures is a claim about an engine
that has changed underneath it. `the_three_text_verbs_compose` below is the
re-measurement: it adds a run to `fixtures/paragraph.pdf`, reflows the
paragraph on the same page in the same session, and asserts both that the
reflow is accepted and that the added run is still on the page. A guard
reintroduced upstream turns that test red in the commit that bumps the pin.

## What is worth knowing about `Reflow`'s shape

`reflow_block` re-emits the page's **first** content object, and the commit
sweep empties every other one — that is by design, not a defect. It is safe
because the plan is read from the session's graph and
`ContentStream::from_page` concatenates every `/Contents` entry, so an
appended run is inside the plan's source and is carried through verbatim
rather than dropped. The engine discloses the collapse in its own report —
*"multi-stream page: N additional /Contents stream(s) were collapsed into
the first"* — and `app::actions::textstyle::reflow` forwards that
disclosure to the status line verbatim, which is the whole of what the
operator is owed about it.

⚠ One refusal a reader will meet on a CAD sheet, unrelated to any of the
above: a paragraph set in a **composite (Type 0 / CIDFont)** face is refused
by name, and [`crate::text::textedit::ReflowRefusal::FontIsComposite`] words
it. That is an engine feature not yet built rather than a guard, and it is
the answer to most reflows on a SOLIDWORKS-exported drawing.
