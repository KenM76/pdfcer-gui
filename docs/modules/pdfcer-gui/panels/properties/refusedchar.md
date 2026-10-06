# `panels::properties::refusedchar` — the refusal, the character it names,
and the face that can type it

`OPERATOR_REQUESTS.md` **O141**. The operator, while trying to fix a typo:

> *"if the character isn't available in a pdf are we able to change to a
> different font?"*

**Yes**, and this module is the connection that makes the answer reachable.
Four pieces have to meet, and each belongs to someone else:

| piece | where it lives |
|---|---|
| the engine refuses by name, before touching bytes | `text_edit::encoding` — `CompositeEncoding::encode_str`, and the simple-font arm's four `Refusal` sites |
| the refusal carries **which character** | `Refusal::character`, `Option<char>`, public |
| a face chooser that offers the standard fourteen | [`super::face`] |
| `set_font` accepting a face the page does not carry | measured: `set_font=AAAAAA+Arimo-Bold->Helvetica-Bold`, after which the `€` goes in |

## A letter the font draws two ways

A refusal with `RInvTrigger::Ambiguous` names a letter the font *has*, under
two codes, so the sentence that says the font does not carry it is false.
`RefusedCharacter::two_ways` is set from the dispatcher's classification and
picks `refused_char_drawn_two_ways` over `refused_char_named`; the offer is
the same, because a face that spells the letter once takes the edit. The
trace line carries it as `two_ways=0|1`. The keystroke-time route records
`false`: the repertoire leaves an ambiguous letter out without saying why.

## Why a PANEL block and not a sentence in the status bar

[`crate::app::status::decline`] already words the refusal, and it cannot
carry this one. `Declined::line` returns `&'static str` — so the slot cannot
interpolate the character the engine named — and `disclosure_line` truncates
what it draws to 45 % of the bar and hangs the rest on hover. A route that
ends in a hover is a route the operator does not find.

[`super::disclose`]'s header settled this exact question for the text tools'
other refusals and its answer transfers word for word: *"A dock panel's
width is the dock's, decided before the body draws, so text wrapped inside it
cannot drive a width and R128 does not apply"*, and *"a refusal is a fact
about the last thing the operator tried to do to the document in front of
them. Properties is where this application already puts facts of that
shape."* The bar keeps its elided line — reworded to name the obstacle and
point here — and the readable copy, the character, and the control live in
the panel.

## Rule 4, and the half of it that binds here

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 is *fuzzy,
never sneaky*: pdfcer may make an inference and may never make one silently.
It **forbids marking the drawing** and **requires** the report off the
canvas.

Swapping the face is the operator's own instruction, so the changed
letterforms are not pdfcer marking its own uncertainty: nothing here badges,
tints or outlines a substituted letter, and the editing canvas goes on
looking exactly like the saved file.

But a standard-14 face is a **name**, not a font program — the engine says so
itself, *"no font program is embedded and no bytes of glyph outline were
added"* — so the client's reader supplies the letterforms. **That is the part
the operator cannot see on his own screen**, and it is exactly the case rule
4's surviving half exists for. It is discharged twice, off the canvas, both
times before he can act on it:

1. [`crate::text::panels::face::face_addable_disclosure`], drawn as a
   visible label in this block, **above** the chooser — a disclosure sits
   above the thing it qualifies, because a caveat below a list arrives after
   the operator has already drawn a conclusion;
2. the same sentence again inside the chooser's popup, drawn by
   [`super::face::popup_body`].

And a third time after the fact, without a line of code here: the engine's
own `format_text` disclosure — *"'Helvetica-Bold' was NOT a font resource
here, so pdfcer ADDED one as `/pdfceF6` … no font program is embedded"* — <!-- old-name-exempt: a PDF resource / BaseFont name the ENGINE writes into the file, quoted verbatim from its own output. It is data in the file format, not prose about this project, and the engine deliberately stopped the rename at that boundary because changing it would alter the bytes of every document pdfcer has ever produced. Same ruling as O141's. -->
arrives verbatim in [`super::disclose`], four lines above this block.

## The offer is UNTESTED against the character, and says so

`preview_font_resources` coverage-tests the characters **already in** the
run, not the one the operator is about to type, so a row here can be a face
that then refuses their `€`. [`super::face`]'s header carries the reason this
is not silently filtered — `FontPreflight`'s `R221` forbids this crate
re-deriving the encoding rule, and a second copy would drift from the commit
path — and the standing ruling on this surface is the Bold button's: *offer
it, and surface the disclosure*. That is
[`crate::text::panels::face::refused_char_no_face`], drawn instead of the
chooser.

## The state machine, and why it is three states rather than one

```text
  (nothing)  --refusal naming a character-->  Offer
  Offer      --the operator picks a face--->  Swapped     (the edit epoch moved)
  Offer      --any other document change-->   (nothing)
  Swapped    --any document change-------->   (nothing)
```

**`Swapped` is not a courtesy.** The face swap is itself an edit, so it
retires the offer — and a block that vanished at that moment would leave the
operator at a caret with nothing telling them to type the character again,
which is a route that ends one gesture short of what it promised. It is also
the state in which the operator most needs a sentence, because on a
metric-compatible swap (his own file moved by **0.005 pt**) the page looks
exactly as it did and nothing on screen says the font changed.

Retirement is `doc.edit_epoch`, the number every other stamped read in this
panel uses, and it is honest in both directions: an undo moves it, a save does
not, and any edit the operator makes instead of taking the offer ends the
offer — which is right, because the refusal it reports is then two gestures
ago.

## Where the two halves are written and read

The refusal is recorded by `crate::app::status::decline::textedit`, from
inside `vector_edit`'s closure, at the same moment it writes the status bar's
sentence — one event, two surfaces, one classification. It travels through
[`PENDING`], a thread-local, for the reason
[`crate::app::status::decline`]'s own store is one: the writer is the
dispatcher and the reader is a body that is handed `&OpenDoc` **shared**, so
there is no `&mut` path between them and inventing one would trade
`panels`' founding invariant for a parameter.

## Item notes

### `type d`

`Ctrl+Enter` calls `commit_into` and then `abandon` unconditionally, so
by the time this block is on screen the draft is gone. Without this the
offer could change the face and could not finish the job — the operator
had to click back into the text and produce his own edit a second time,
from memory, which is the *"two gestures where there should be one"* half
of O141.

`None` when the plan that produced the refusal named a different
`(page, run)` than the refusal did, which is a disagreement between two
facts about the same commit and is not something to paper over: the block
then offers the face swap alone, and the operator retypes.

### `static PENDING`

A `take`, not a peek: adopting it stamps it against the epoch on
screen, and a slot that kept handing the same refusal back would re-stamp
it after every edit and never retire.

### `fn sync_faces`

# Why this block asks for its own pre-flight rather than borrowing the
panel's

`properties::text::TextStyleDraft` already holds a face list, and it is the
wrong one twice over. It is stamped on the **selection**, which is empty here
— the caret was abandoned by the commit that got refused — and even where a
selection survives, it need not be the run the refusal named. Borrowing it
would make the offer's rows depend on what happens to be selected, which is a
list that is right most of the time and silently wrong the rest.

The cost is one extraction with provenance capture plus one pre-flight, paid
once per `(page, run, epoch)` and only while a refusal is on screen.

### `fn typed`

Present rather than `None`, because the state machine's interesting
path is the one where the retype can actually be re-raised — a helper
that omitted it would let a build that dropped the operand pass every
test in this module.

### `fn a_recorded_refusal_is_adopted_exactly_once`

The `take` is the property: a slot that kept handing the same refusal
back would re-stamp it after every edit, so the block would never retire
and the operator would read a sentence about a gesture from ten minutes
ago.

### `fn an_unrelated_edit_retires_the_offer`

The refusal it reports is then two gestures ago, and
`app::status::decline`'s own retirement rule is the precedent: a sentence
about an earlier press, read after a later one, is a small lie told
confidently.

### `fn taking_the_offer_leaves_the_follow_up_behind`

The face swap is itself an edit, so from the outside it is
indistinguishable from the case above — the epoch moved. Only this block
knows it caused it. Without `taken` the offer would vanish at exactly the
moment the operator needs to be told to type the character again, and the
route would end one gesture short of the thing it promised.

### `fn the_edit_that_lands_retires_the_follow_up`

A follow-up that survived every subsequent edit would be a permanent
*"type it again"* under a document where it had already gone in, which is
the same defect class as a decline that never retires.

### `fn a_second_refusal_replaces_the_first`

`app::status::decline`'s slot rule, and it matters more here: an operator
who meets two different missing characters in a row must be offered a face
for the second one, and a block still naming the first would send them to
a chooser aimed at the wrong run.

### `fn the_offer_reaches_the_panel_and_carries_faces_the_page_does_not_have`

# Why this is not another state-machine test

Every test above it exercises [`RefusedCharUi::advance`], which is the
part that is easy to get right and easy to test. **None of them would
fail on a build where `body_sections` never calls [`section`]**, where
the pre-flight is never asked, or where `choices` answers an empty list —
and this project's standing lesson is exactly that: *eight green unit
tests once passed while the feature performed one of fourteen steps.*

So this runs the **real** `panels::properties::body` through
`Context::run_ui`, on the real fixture, and reads what the frame left
behind. `panels::comments::tests` established the technique and its
header carries the argument in full.

# What each assertion would catch

| assertion | the build it fails on |
|---|---|
| the refusal was adopted | `body_sections` does not call [`section`] at all — the capability built and unreached, which is O141's own subject |
| the face list is non-empty | [`sync_faces`] asked the engine and got nothing: a pin that did not resolve, or a pre-flight that refused |
| at least fourteen rows are `PdfcerWouldAdd` | the offer is built from **the page's own fonts**, which is the offer that cannot work: this block only exists because those faces refused the character |

The third is the one that matters most, and it is a real risk rather
than a hypothetical: `preview_font_resources` enumerates the page's own
`/Font` resources, so a chooser built from `accepted()` alone leaves the
engine's standard-14 authoring present and unreachable from any surface,
on the one block whose whole purpose is to reach it.

# Two frames, not one

`panels::dimension_groups`' reason, which `panels::comments::tests`
repeats: an immediate-mode layout's first pass is a guess and the scroll
area's size settles on the second. This panel is one `ScrollArea` and
[`section`] publishes through `ui_rect_visible`, which answers nothing
for a rect outside a clip that has not settled.

### `fn the_offer_is_coverage_tested_for_the_refused_character_not_the_runs_own_text`

This is the assertion that proves the `candidate` argument reaches the
engine, and it needs a character that **discriminates**. `'q'` does not:
every standard-14 text face holds it, so the sibling test above passes
whether the offer was coverage-tested for `'q'` or for the run's own
ASCII. A test that cannot tell the two apart is not a test of the change.

`'中'` discriminates. No standard-14 face can encode it under
`WinAnsiEncoding` or under `Symbol`/`ZapfDingbats`' built-in encodings,
while the run's own characters are plain ASCII that all twelve text faces
hold. So:

| what the pre-flight is asked | addable rows |
|---|---|
| the run's own text (`None`) | twelve |
| the refused character (`Some("中")`) | **none** |

⇒ An empty offer is the CORRECT answer here. pdfcer genuinely cannot
type `'中'` with any face it can author, and a list of twelve faces that
would each refuse is a control that cannot work — the exact thing R9
forbids.

### `fn taking_the_offer_re_applies_the_edit_the_operator_already_typed`

# What a build that fails this does to the operator

It changes his letters and abandons his edit. He picked a font from a
list captioned *"pdfcer will put your change in with it"*, watched the
text on the page change shape, and got nothing typed into it — which
reads as the feature half-working, and is worse than the two-gesture
route it replaced because the caption promised otherwise.

# The three things asserted, and why each is separate

1. **The retype is raised at all**, and it is `Action::CommitTextEdit` —
   the same variant `Ctrl+Enter` raises, so the retype takes the
   identical route through `canvas::textedit::plan` and re-derives the
   pin and the follower disposition from the page **as the restyle left
   it**. A bespoke verb here would be a second path that drifts.
2. **It carries the operator's own words.** `replacement` is the one
   operand recoverable from nowhere else once the draft is abandoned, so
   a build that rebuilt it from the page would silently retype the
   ORIGINAL and report success.
3. **[`RefusedCharUi::retried`] is set**, which is what moves the block
   off a sentence promising a retype that has already happened.

Driven through [`RefusedCharUi::advance`] rather than by setting the
fields, because the transition into `Swapped` is the thing under test:
the epoch moves once, `taken` is consumed, and only then is the retype
owed. A test that assigned `swapped_to` itself would pass on a build
whose state machine never reached it.

### `fn a_refusal_with_no_carried_words_still_leaves_the_swapped_state`

`RefusedCharacter::typed` is `None` when the plan that produced the
refusal named a different `(page, run)` than the refusal did — a
disagreement between two facts about one commit, which this shell
declines to paper over. The face swap is still worth having, and
[`RefusedCharUi::retried`] must still be set: it means *this block has
done everything it can*, not *an action was pushed*. A build that tied it
to the push would sit for ever on a sentence promising a retype that is
never coming.

### `fn the_regions_name_this_block_and_not_the_text_section`

Worth asserting because the face chooser's regions are namespaced by a
prefix passed in at the call site, and a prefix copied from
`properties::text` would make this block's popup indistinguishable from
the *This text* section's in a trace — so a driven check would read the
wrong control's rectangle and click it.

### `const REGION`

Published only when it draws, so its **absence** is the evidence that no
character was refused — which is the distinction a driven check about this
feature is actually asking about, and the one a region declared
unconditionally could never provide.

### `const DISCLOSURE_REGION`

Its own region rather than a clause of [`REGION`], because *"the sentence
reached a rectangle"* is the one thing about this feature that no unit test
in the workspace can observe: the string is catalogued and asserted, and a
build that drew it off the bottom of the panel would pass every one of those
assertions.

### `fn record`

# Why the caller reads a field off `EditError` and this is not a second
taxonomy

`crate::app::status::decline::textedit`'s header forbids two shortcuts:
matching on `EditError`'s variants to *derive the operator's reason*, and
grepping its `Display` prose. Neither happens. The category still comes from
`RefusalKind`, which the engine made non-`#[non_exhaustive]` so a front end
can match it and have the compiler prove the sentences complete; what is read
here is **one datum that the coarse kind structurally cannot carry**, on the
identical licence `one_operator` already has.

### `fn forget_document`

`PanelsState::forget_document` resets that struct whole, which clears
[`RefusedCharUi`] — and [`PENDING`] lives outside it, so without this a
refusal recorded on the frame a document was closed would be adopted by the
**next** document's first panel draw. `(page, run)` names different text
there, and the offer would aim a face swap at a run nobody asked about. The
same hazard `PanelsState::bookmarks`' own note records for an `ObjId`,
arriving through the one field that reset does not reach.

### `struct RefusedCharUi`

Held on `PanelsState` for that struct's own stated rule — a panel body is
handed `&OpenDoc`, shared, and this is the operator's state rather than a
derived cache of the document's. It is reset with the document by
`PanelsState::forget_document`, which matters here for the reason a bookmark
parent does: a `(page, run)` pair names different text in a different file,
so an offer carried across would restyle a run nobody asked about.

### `fn section`

Returns `false` on every frame where no character has been refused, which is
nearly all of them — and it renders **nothing at all** in that case, heading
included, on [`super::disclose`]'s rule: *"a heading present on every frame
trains an operator to stop reading the region under it, which would waste the
one surface a disclosure has."*

Its answer is deliberately **not** folded into `body_sections`'
`something_drew`. That predicate is O75's and asks whether a
**selection**-scoped section has spoken; a refusal from the last edit is not
a description of the current selection, and letting it collapse the object
section would make the panel change shape for a reason unconnected to what is
picked. [`super::disclose`]'s call site records the identical exclusion for
the identical reason.
