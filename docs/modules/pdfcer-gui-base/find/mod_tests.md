# `pdfcer-gui/find/mod_tests`

## Item notes

### `fn the_default_search_is_literal`

The regression test for the defect this whole module's header is
about: the old shell's Find bar ran through `EditSession::find_text`,
which passes `with_wildcards(true)`, so a typed `?` matched every
character on the page. `to_core` is the ONE place a
`TextSearchOptions` is built in this crate, so asserting on it is
asserting on every search this shell can run.

### `fn a_wildcard_search_is_only_ever_asked_for_explicitly`

The other direction, which matters as much: the control has to work,
or the escape hatch from the literal default would be a dead
checkbox — the placeholder P3 forbids, in the one place the operator
went looking for a feature.

### `fn match_case_inverts_into_the_engines_polarity`

The shell says *Match case* and the engine says `case_insensitive`.
A dropped `!` here is a search that ignores the checkbox, which looks
like a search that ignores the operator.

### `fn the_word_rule_and_the_whole_word_flag_are_independent`

`TextSearchOptions::with_word_boundary`'s own docs require this:
choosing a rule must not switch the option on, and switching the
option off and on again must not reset the rule.

### `fn every_word_rule_the_chooser_offers_has_a_label`

The chooser is driven from [`FindOptions::WORD_RULES`] rather than
from a `match`, because a wildcard arm over a non-exhaustive enum
would silently drop a future variant instead of failing to compile.
This is the reminder that the list is the thing to extend.

### `fn an_edited_document_says_it_changed_rather_than_that_there_are_no_matches`

A document edited after a fruitless search must not say "No matches":
that would be a claim about the current revision, which the search
never examined.

### `fn changing_the_query_blanks_the_readout`

A different question is not an out-of-date answer to this one. The
operator who starts typing a new term should see the readout clear,
not see the old count go on standing next to new text.

### `fn stepping_an_empty_result_set_is_not_a_panic`

Unreachable through [`step_to`], which checks the readout first, and
handled anyway: an action can be raised from a customized keymap in
any state, and an index into an empty `Vec` is a crash waiting for
somebody to find it.

### `fn opening_another_document_forgets_the_hits_but_not_the_query`

Page indices and page-space rectangles describe one file. Carrying
them into another is not staleness — the epoch would still match,
because a freshly opened document's epoch is 0 — it is nonsense, and
it is why this seam exists rather than relying on the epoch alone.

### `fn a_hit_with_no_geometry_still_counts`

"We cannot draw a box on this page" is not "this hit does not exist",
and conflating them would make a document with one degenerate page
report the wrong number of hits.

### `fn a_real_search_finds_its_text_and_navigates_to_it`

The end-to-end check that the borrow protocol works: the render worker
is stopped, `Arc::get_mut` succeeds, the engine is asked, and the view
moves to the page the answer is on. It is deliberately driven through
[`apply`] rather than through [`search`] directly, because the thing
most likely to be wrong is the wiring rather than the arithmetic.

The fixture's text is asserted to exist first: a test that searched
for a string the fixture does not contain would pass on a build whose
search always returned nothing.

### `fn a_search_bumps_no_epoch_and_drops_no_texture`

`find_text_with` takes `&mut EditSession`, which makes it easy to
mistake for a mutation and to give it `vector_edit`'s epoch bump and
texture drop. Both would be wrong: the bump would make the results
stale by their own rule the instant they were produced, and the drop
would re-rasterize a CAD sheet on every Enter.
