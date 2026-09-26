# `pdfcer-gui/app/actions/importtext/tests`

## Item notes

### `fn clean`

Built by mutating `PlaceTextReport::default()` rather than by naming every
field, deliberately. `PlaceTextReport` has **23** of them and is
`#[non_exhaustive]`; a literal here would not compile, and a helper that
listed twenty-three zeroes would have to be edited every time the engine
learns to count something new — which is the moment this test is most
valuable and least likely to be touched.

### `fn a_perfect_import_discloses_only_how_many_pages_arrived`

The whole design of this receipt in one assertion. Six of the seven
sentences are conditional, and a build that emitted them unconditionally —
*"0 tabs collapsed"*, *"0 paragraphs split"* — would produce a form rather
than a receipt, and by the third import nobody would read the line that
mattered.

It also pins that the **undo** sentence is conditional on `coalesced`.
That one is the most tempting to make unconditional, because it sounds
reassuring — *"this can be undone in one press"* — and it is the reassurance
that would train an operator past the one case where it is false.

### `fn the_undo_warning_tracks_coalesced_in_both_directions`

`PlaceTextReport::coalesced` is documented as *checked, not assumed*: past
`MAX_UNDO_DEPTH` every page is still placed and they simply are not grouped.
A surface that promised one `Ctrl+Z` without reading this would be promising
something the engine has already said may not be true.

Both directions in one test. A test that only asserted the warning appears
would pass on a build that showed it always — which is the failure the test
above owns, and asserting the pair here is what stops the two tests from
being satisfiable by opposite bugs.

### `fn every_judgement_is_reported_when_it_happened_and_silent_when_it_did_not`

A table-driven test rather than six, because the property is *the same
property* six times and writing it once is what stops a seventh judgement
being added with no test. What it cannot check — and no test here can — is
that the sentence says something true; that is the doc comment's job on each
one in `text::importtext`.

### `type Judgement`

Named rather than written inline: clippy calls the inline form a
*"very complex type"*, and it is right that a reader meeting
`&[(&str, fn(&mut PlaceTextReport))]` has to decode it before the test
says anything.

### `fn the_engines_self_check_is_last_and_says_it_is_a_fault`

`box_overflow_lines` is documented as *"a self-check that must be 0"*, so a
non-zero value is a defect in the placer rather than a judgement about the
operator's file. If it were phrased and ordered like the other six, an
operator would file it under *"things imports do"* and never mention it —
and a fault nobody reports is a fault nobody fixes.

### `fn the_chooser_refusals_name_the_controls_rather_than_the_geometry`

`NoColumn` and `PageTooShort` are the only refusals here an operator can act
on before pressing again, and a sentence describing the geometry — *"the
margins exceed the media box width"* — would be true and useless. The test
pins that both name a control.

### `fn the_unmappable_refusal_carries_the_listing_and_offers_no_button_that_does_not_exist`

The refusal a real text file is most likely to meet — an em dash, a curly
quote, an accented name — and the one part of it no rewording improves:
`U+2014 '—' ×12` is what the operator needs in order to find those
characters in his own file.

It must **not** offer the engine's third remedy. `PlaceTextError::Unmappable`'s
own message ends *"or ask for them to be dropped"*, and this window has no
such control — a sentence naming a button that does not exist is worse than
one remedy fewer.
