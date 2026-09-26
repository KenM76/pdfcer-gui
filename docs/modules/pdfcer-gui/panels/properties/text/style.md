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

### `enum StyleOutlook`

Until that afternoon this came from `preview_style_resolution` joined
against `preview_font_resources`. `preview_style_resolution` previews the
**R90 synthesis gate**, whose answer `StyleOutcome::WouldSynthesize` means
exactly *"no real face on THIS PAGE claims that style and covers this
run"*. That stopped being the same question as *"what will pressing Bold
do?"* the moment `Pass 179.0` added rung 2, because **the standard-14
sibling is by construction not on the page** — `Helvetica-Bold` needs no
font file at all — so the gate cannot see it and neither could this.

The visible effect was two instruments disagreeing by construction: the
tooltip promised thickened letters and the status line afterwards reported a
real `Helvetica-Bold`, on a CAD title block, which is the commonest page in
the operator's working set.

`EditSession::preview_style_ladder` (`Pass 295.0`) runs **the same planner
`format_text` runs**, walks the page's content once, and stages nothing. So
these variants are the engine's own rungs rather than a shell's inference
from an adjacent answer, and the preview and the commit are two readings of
one computation instead of two answers kept in agreement by hand.

⇒ No rule is re-implemented here and none ever was: engine invariant R74
forbids `pdfcer-gui` re-deriving `family_stem`, `name_claims_bold` or
`name_claims_italic`, and this asks a question rather than answering one.

### `struct TriedFace`

# The feature `passed_over` being prose cost outright

`StyleLadder::passed_over` was a `Vec<String>` of `"BaseFont (reason)"`
until `Pass 295.0`. Saying *"pdfcer will try `Times-Bold` and it has no
`o`"* in this shell's own voice would have meant splitting on `" ("` and
stripping a `")"` — a locator for `pdfcer-core`'s message format, living in
a GUI, breaking silently the first time a reason sentence gained a
parenthesis. A shell disciplined about not re-deriving engine facts keeps
quiet instead, so **the sentence was never written at all**: the operator
was told which rung bound and never which faces were tried and rejected,
which is the half he asks about.

`PassedOver` is now `{ base_font, reason, refusal }`, and
`Refusal::character` gives **the offending character**. That is what makes
this worth a type of its own here: the engine's `reason` is accurate and
technical (*"R-INV-1: character U+006F 'o' has no code in font
'Times-Bold'"*), and the hover wants *"no 'o'"*. The character is carried;
the prose is not.

`character` is `Option` because a face can be passed over for a reason
that is not one character — it could not be planned at all, say. The
sentence degrades to naming the face, which is still more than nothing was.

### `struct StyleForecast`

Two fields rather than a sixth [`StyleOutlook`] variant, because they
answer different questions and are independently present: a rung-2 bind can
pass over two page faces on the way, and a rung-1 bind can pass over none.
Folding them together would make the passed-over list a property of the
outcome, which it is not.

### `fn forecast`

# One engine call, and it is the SAME computation the press will run

`EditSession::preview_style_ladder` runs `plan_style_ladder` — the
function `format_text` runs — against the session's *staged* content, so
the preview and the commit that follows it are two readings of one
answer rather than two answers that have to be kept in agreement by
hand. It walks the page's content once, plans, and stages, commits and
caches nothing.

⇒ Nothing is re-derived here. Engine invariant R74 forbids `pdfcer-gui`
re-deriving `family_stem`, `name_claims_bold` or `name_claims_italic`,
and this function asks a question and maps the reply onto sentences.
The previous shape — joining `preview_style_resolution`'s `selector`
against `preview_font_resources`' accepted list to reconstruct an answer
neither call gave — is deleted; see [`TextStyleDraft::sync`].

# `options` is not optional, and passing the wrong one is a lie

`StylePolicy::Refuse` changes the answer: under it a ladder that reaches
rung 4 is a **refusal**, not a synthesis. The engine says so at the
method — *"pass the same `FormatOptions` the commit will use"* — and
this passes the operator's own posture out of the settings store, the
same value `crate::app::actions::textstyle` puts on the commit. A
preview run under a default posture would promise thickened letters to
an operator who had ticked *never fake it*, which is the exact defect
the deleted `Refuse`-pinned probe used to cause from the other side.

A single axis per call, because the two buttons issue two separate
single-axis requests; see [`TextStyleDraft::italic_outlook`] for why
neither may borrow the other's answer.

### `fn bold_hint`

# Seven sentences, and the last is the one that was there before

| outlook | what the operator reads |
|---|---|
| [`StyleOutlook::AlreadyStyled`] | *this text is already bold* |
| [`StyleOutlook::SiblingOnPage`] | *this page already carries **Arial-Bold**, the bold form of this text's own typeface* |
| [`StyleOutlook::OtherFamilyOnPage`] | *pdfcer will use **Arial-Bold**, a real bold face from a different typeface* |
| [`StyleOutlook::StandardSibling`] | *pdfcer will add **Helvetica-Bold**, a standard PDF face, without embedding a font file* |
| [`StyleOutlook::Synthesized`] | *no real bold face can show this text, so pdfcer will thicken the letters* |
| [`StyleOutlook::Declined`] | *your settings say never fake it, so Bold will be refused here* |
| `None` — the probe did not answer | the conditional hint, unchanged |

Plus, appended to any of the six, the **passed-over addendum** when the
ladder stepped over a face on the way: *"It will pass over Times-Bold (no
'o'), which cannot show this text."* That clause is the reason
[`StyleForecast`] carries `tried` alongside the outlook — the rung says what
will happen, and `tried` says what was rejected to get there, and an
operator staring at a title block full of `Times-Bold` wants the second.

The last row is not a fallback that should have been designed away. A probe
returns `None` for a page whose content cannot be planned, for an
`#[non_exhaustive]` rung this build has never seen — rung 3, the
`--font-dir` donor, is exactly that — and for an encrypted document; and in
every one of those the honest thing to say is the mechanism rather than a
prediction. That is what
[`crate::text::panels::properties::text_bold_hint`] already said, which is
why it stays.

# None of the seven greys the button, and that is still the engine's ruling

*"Do not grey out a bold button. Offer it, and surface the disclosure when
synthesis fires."* [`StyleOutlook::Declined`] is the row where greying could
now be argued — it is a **measured** prediction of a refusal, not a guess —
and it is still a sentence, because the thing that produces it is a setting
the operator can change. R9 reserves greying for the *temporarily*
unavailable and demands the reason on hover; this is the reason on hover,
and pressing it produces a refusal that names the same setting.

### `fn italic_hint`

It reads [`TextStyleDraft::italic_outlook`] and never the bold one. The
two are genuinely different answers on an ordinary page — one holding a real
`Arial-Bold` and no `Arial-Italic` gives `SiblingOnPage` for one button and
`Synthesized` for the other — and a shared sentence would be wrong on
exactly the pages an operator is most likely to be working on.
