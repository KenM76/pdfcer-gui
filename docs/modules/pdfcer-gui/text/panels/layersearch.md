# `text::panels::layersearch` — the four strings the Layers search says

**Surface:** the search field at the top of the Layers panel, and the
two lines that describe what it did.
**Consumer:** [`crate::panels::layers`], and nothing else.

Its own module rather than four more functions in
[`super`] — which is at 1,299 lines — for R2's reason, and for a better
one: these four are the only strings in this program that describe **a
filter's effect on a list**, and the wording rules they follow are
particular to that job. Keeping them together is what lets those rules
be written down once and asserted below.

# The three rules this copy follows

**1. Say the number, not the adjective.** *"3 of 16 layers"* and never
*"some layers are hidden"*. An operator scanning a filtered list needs
to know whether the thing they are looking for is absent from the
document or absent from the *view*, and a count answers that in one
glance. `panels::comments`' `comments_excluded` is the precedent and it
is quoted in its own docs: *"the panel discloses the filter in numbers
instead of in doctrine."*

**2. The empty case says the query back.** Not *"No matches."* — which
is what a broken panel says too — but the text that was typed, in
quotation marks. There is exactly one way for an operator to be sure a
search ran, and it is seeing their own query repeated by the thing that
ran it.

**3. Nothing is said when nothing was filtered.** R9's shape applied to
prose: a panel showing every layer must read exactly as it did before
this feature existed. That is why [`narrowed`] returns `Option` — the
`None` is not "no text available", it is *"there is nothing to
disclose"*, and a caller that unwrapped it to an empty string would put
a blank line above every unfiltered list forever.
