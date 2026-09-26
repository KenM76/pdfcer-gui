# `pdfcer-gui/panels/properties/text/style`

## Item notes

### `fn hint`

# One function for two axes, which is the opposite of what was here


The addendum is appended here rather than folded into each sentence,
because it is **orthogonal** — a ladder can pass over faces on its way to any
rung, including the one that ends in a refusal, and writing it into seven
sentences twice over is how a clause goes stale in six of fourteen places.

### `fn drafted`

A constructor rather than three field assignments after
`Default::default()`, which is what clippy's `field_reassign_with_default`
asks for and is better here anyway: what these tests vary is the pair of
forecasts, and a helper that takes exactly the pair says so.

### `fn every_outlook_earns_its_own_sentence`

If any two collapsed, the probe would be decoration: an operator whose
page carries the bold form of their own typeface and one whose page
carries a bold face from some other family would read the same words and
learn nothing either way — and those two results look quite different on
the page.

The three face-bearing rows all carry **the same face name** on
purpose. A test that varied the name as well as the variant would pass
on a build whose sentences were identical apart from the interpolated
string, which is exactly the bug this is here to catch: the distinction
that matters is *what pdfcer will do*, not *which face it names*.

The `None` row is included deliberately — it is the sentence that was
there before any preview existed, and it must remain distinguishable
from the six predictions rather than being absorbed into one of them.

### `fn the_two_buttons_do_not_borrow_each_others_answer`

The state this pins is the ordinary one, not an exotic one: a page
carrying a real `Arial-Bold` and no `Arial-Italic` gives `SiblingOnPage`
for one button and `Synthesized` for the other. A shared sentence — or a
copy-paste that read `bold_outlook` in both helpers — would be wrong on
exactly the pages an operator is most likely to be working on, and would
be invisible on every page where the two answers happen to agree.

### `fn the_synthetic_sentences_name_the_right_operation`

They are different synthetic operations — a weight is the regular face
stroked, a slant is the upright face sheared — and an operator who has
read one should not have to guess that the other means something else.

### `fn the_family_verdict_changes_what_the_hover_promises`

Both bind a real face already on the page and embed nothing, so a
sentence that stopped at *"pdfcer will use a real bold face"* would be
true of both and useful for neither. The difference is that one keeps
the drawing's typeface and the other changes it visibly — the engine's
own words for the second are *"a bigger change than a weight swap"*, and
rule 4 makes the visible half the half that must be disclosed.

Asserted on the *shape* claim rather than on the whole sentence, so
rewording the hover does not break the test while removing the
distinction silently would.

### `fn the_declined_case_names_the_setting_and_not_a_remedy`

`StylePolicy::Refuse` is the operator's own setting, so the honest
sentence says which setting and stops. Naming the face chooser as a
remedy would be this shell second-guessing pdfcer's font selection —
decision 058's exact case — and telling them to pick a different font
would be advice about a page the shell has not surveyed.

### `fn a_passed_over_face_is_named_with_its_missing_character`

The engine discloses `passed_over` **after** the commit and this shell
already surfaces that verbatim, so the value here is entirely in the
timing: the same fact one gesture earlier, where it can still change what
the operator does.

The addendum is checked on a rung that is **not** a refusal, because
that is the case a naive design gets wrong — passing over a face and then
succeeding is the ordinary path, not an error path.

### `fn a_passed_over_face_with_no_character_is_named_bare`

`Refusal::character` is an `Option` and a refusal about the whole run is
a real case; *"Times-Bold ()"* would be this shell rendering an absence
as a presence, which is the same defect class as a `{:?}` in an operator
string.

### `fn an_empty_passed_over_list_says_nothing`

The commonest case by far, and the one where an unconditional clause
would read as a warning about nothing. Falsifies the previous two tests:
without this, a build that appended the clause always would still pass
them.

### `fn an_unsynced_draft_promises_nothing`

`TextStyleDraft::default()` has never been synced, so both forecasts are
`None` — and the honest thing to say about a run nothing has been read
from is the mechanism, which is exactly what the hint said before any of
this landed. A build that guessed `Synthesized` here would tell an
operator their letters are about to be thickened on a page that carries
a real bold face.
