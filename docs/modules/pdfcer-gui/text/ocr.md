# `text::ocr` — every word the Recognise-text surface says

Consumed by [`crate::dialogs::ocr`] (the dialog that runs recognition and
reports what it inferred) and by [`crate::find::bar`] (the offer that
appears when a search found nothing on a page that has no text to find).

## Why this catalog is unusually careful, and it is not house style

**OCR is the single largest inference pdfcer makes.** `pdfcer-core`'s own
`ocr::layer` header says it in those words — *"every word here is a
guess"* — and project rule 4 (*"fuzzy, never sneaky"*) therefore binds this
surface harder than any other in the program. Two of its clauses bite here
and they pull in different directions, which is why the copy is written the
way it is:

1. **The result must look normal.** The operator asked for exactly that:
   *"I want OCRed stuff to look normal when the command is executed too."*
   Mode 3 is not a compromise, it is the whole mechanism — nothing visible
   is added, the page renders pixel-identically, and there is **no**
   highlighting of doubtful words baked into the document. So none of the
   copy below promises a visible mark, and none of it should ever grow one.
2. **The uncertainty must be stated anyway**, off-canvas, before the
   recognition becomes a file. That is what this dialog is for.

## The one fact this surface exists to carry

**`ocrs` reports no confidence at all.** Not "low confidence", not
"confidence pending" — its output type is a character and a rectangle, and
there is no score on a character, a word, a line or the page.
`OcrsEngine::reports_confidence()` returns `false` and
`pdfcer_core::ocr::engine_ocrs`'s header is explicit that this is *"a fact
about the world"* rather than a stub awaiting improvement.

The consequence for copy is sharp, and it is the reason [`no_confidence`]
is worded as a negation of a specific wrong reading rather than as a
neutral note: **an absent score and a high score must never look the
same.** A dialog that reported "0 words need review" would be true of an
engine that scores nothing and would read as a clean bill of health. So
this surface never says that, and `OcrPage::words_needing_review` already
encodes the same principle on the other side by counting an unscored word
as needing review.

## Where the *engine's* sentences come from, and why they are not here

[`crate::dialogs::ocr`] renders `OcrLayerReport::disclosures()` — a
`Vec<String>` built inside `pdfcer-core` — as a list, verbatim, beneath the
headings below. That is deliberate and it is the engine's own instruction:
the disclosures are built *"here rather than at each call site so the GUI
and the CLI cannot disagree about what was disclosed."*

They are therefore **data at run time**, not literals in this crate, and
`tools/gates/check-ui-strings.sh` is untouched by them. What lives here is
the shell's own framing — the headings, the buttons, the refusals — which
is exactly the split rule R1 is about: a catalog owns the words this
program chose, not the words another crate reported.

## Conventions

[`crate::text`]'s, unchanged: sentence case and no trailing period on a
label, full sentences with punctuation for prose, an ellipsis on a control
that asks a question before acting. One addition — **no sentence here
makes a claim about accuracy.** pdfcer has never measured this engine
against a real scan (`FEATURES.md` records that its only test documents are
vector PDFs that already contain text), so "accurate", "reliable" and
"high quality" are words this surface is not entitled to.

## Item notes

### `fn nothing_here_claims_the_recognition_is_accurate`

The module header's rule, asserted rather than trusted. pdfcer has never
run this engine against a real scan, so any adjective implying measured
quality would be a claim with nothing behind it — and marketing
adjectives are exactly what a copy pass adds without thinking.

### `fn the_scored_sentence_says_the_score_is_not_a_check`

The one string on this surface that must not be softened. It is here as
a test rather than only as a doc comment because "no confidence
reported" reads as neutral, and a future copy pass tidying it into
something neutral would delete the disclosure while leaving a sentence
in its place.

### `fn the_outcome_says_where_the_text_went_and_how_to_undo_it`

This replaced a test called
`the_write_control_offers_a_new_file_and_never_an_overwrite`, which
asserted that the only way out of this dialog was a Save-as. That was
true, it was enforced, and it was the thing the operator objected to:
*"Why do I have to save a copy instead of just go back into my pdf and
save over it?"*

It was never a policy. `ocr::layer::add_ocr_layer` took an immutable
document and returned a whole file, so a Save-as was the only shape
available. The engine's Pass 135.0 made recognition an edit, and the
three facts below are what the operator now needs to be told instead.

### `fn a_skipped_page_and_an_unreadable_one_say_different_things`

`nothing_recognised` means the recogniser looked and found nothing;
`already_has_text` means it declined to look. Different facts, different
remedies — one is "there is nothing readable here", the other is "there
is already text here and doubling it would break Find". Collapsing them
would leave the operator unable to tell a blank scan from a document
that was recognised last week.

### `fn the_find_offer_reports_the_page_rather_than_the_search`

The trap the operator named, pinned. A sentence mentioning matches
would be the collapse of *"the document is images"* into *"this search
found nothing"* — the two the specification insists must not be one.

### `fn stopped_early`

A stopped run is a success and an incomplete one at the same time, and this
is the sentence that stops the first half hiding the second. Somebody who
ends a 200-page recognition at page 40 must not walk away believing the
document is done; they find out otherwise months later, searching for a word
on page 150 that is not in the layer.

It names both numbers. "Stopped early" alone leaves them to guess how much
they have, and the answer is the whole point of having pressed Stop rather
than Cancel.

### `fn cancelled`

It says the document is untouched, because that is the fact the operator
is actually checking for — a half-written layer is the thing they pressed
Cancel to avoid, and silence about it leaves them to wonder.

### `fn working_progress`

Operator request, 2026-09-01: *"so that the user can see that it is doing
something and hasn't frozen on large documents."*

Three moving numbers, and each answers a different worry. The page count
answers *"how far"*; the character count answers *"is it still alive"* —
it moves on a dense sheet where the word count barely does; and naming the
page it is ON rather than only the count tells an operator whose scan is bad
exactly which sheet to look at afterwards.

### `fn intro`

Says what the operation *does to the page*, because that is the first
question an operator has about a tool that rewrites a document they may
have to defend the provenance of. The answer — nothing visible changes, the
image is not re-encoded — is `ocr::layer`'s own guarantee and is worth
leading with rather than burying under a progress bar.

### `fn run`

**No longer "Recognise this page".** It said that because that was all it
could do, and the operator's 2026-08-26 report — *"how do I OCR more than
one page? Why does the tool stop at one?"* — was as much about the label as
about the capability: a button naming one page is a button that has already
answered the question, wrongly.

### `fn scope_all`

First in the list **and** pre-selected, which are two decisions and both
deliberate. First because the surveyed tools put it first; pre-selected
because recognising a scan means recognising the scan, not one sheet of it.
The old behaviour is the second option and one click away.

### `fn scope_current`

It names the number rather than saying *"the current page"* because the
operator can page the document while this window is up, and by the time
they read the label "current" may no longer mean what the run will do. The
number cannot drift.

### `fn scope_picked`

The operator: *"I should have options to do the whole document, or the
pages I have selected in the thumbnails."*

`count` is how many are picked, so the label states the operand rather than
naming a place the operator then has to go and count. *"Selected pages"*
alone would be a promise whose size is invisible from the dialog — and this
is a run that can take minutes, so the number is the part that decides
whether he presses the button.

# Why it is drawn only when something is picked

R9. With an empty rail selection this option has no operand at all, and a
greyed radio saying *"Selected pages (0)"* would be a control explaining
its own uselessness in a window that already has three working answers. The
remedy is not on this surface — it is *go and pick some pages* — so there
is nothing a hover could usefully say either.

The plural is written out for the same reason every count in this crate is:
*"1 pages"* costs credibility on a surface whose whole job is being
believed.

### `fn scope_range_hint`

Shows the syntax by example rather than describing it, because the syntax
is `dialogs::print::tabs::parse_page_range`'s and an example is both shorter
and harder to get subtly wrong than a description of it.

### `fn scope_range_unresolved`

Not an error — a **status**. A half-typed `1-` is an ordinary state of a
text field the operator is in the middle of using, and colouring it red or
popping a message would be scolding them for typing. The Recognise button is
simply not available until the range resolves, and this says why.

### `fn run_tooltip`

Names the cost in the operator's terms. There is no measured figure to
quote — see the module header on what this surface is not entitled to
claim — so it says *seconds* and says which page, which are both true and
checkable.

⚠ **The last clause is false.** The recogniser runs on a detached worker
(`crate::ocr::job`) and the dialog stays live with a spinner, a page count,
Stop and Cancel. `DEFECTS.md` D40.

### `fn no_confidence`

Worded to refuse a specific wrong reading rather than to state a neutral
fact, because the wrong reading is the one a reader arrives with: a page of
recognised text with no warnings on it looks checked. It is not checked. It
was never scored either way.

The engine emits its own version of this through
`OcrLayerReport::disclosures()`, and the two are deliberately both present:
that one appears in the list of disclosures beside the counts, this one is
the dialog's own heading-level statement, and the operator reads the second
before they read the list. Duplication is the point — this is the one fact
that must not be missed by someone who skims.

### `fn applied_to_document`

It says three things in one line, and each was a separate control before:
the words are *in the document*, an ordinary Save writes them, and an
ordinary Undo removes them.

The operator, 2026-08-26: *"Why do I have to save a copy instead of just go
back into my pdf and save over it or save from there?"* The answer was that
`add_ocr_layer` took an immutable document and handed back a whole file, so
this shell had nothing to put the layer *into*. The engine's Pass 135.0
(2026-08-27) made recognition an edit, and the honest sentence is now the
short one.

### `fn suggested_suffix`

A suggestion, not a rule — the operator can type anything. It exists so the
default answer is never the file they opened, which is the same protection
the label spells out in words.

### `fn models_missing`

`searched` is the engine's own list of every directory it tried, in order.
It is part of the message rather than a detail: *"models not found"* is
unactionable, and the list is what tells an operator either where to put
the files or — just as often — that they put them somewhere pdfcer never
looks.

It takes a **list**, not a pre-joined string, and the separator below is
why: a comma and a space between two paths is punctuation an operator reads,
so it is copy and belongs in this file rather than at the call site.
`tools/gates/check-ui-strings.sh` caught exactly that `", "` sitting in
`dialogs::ocr::sentence`, and it was right to.

### `fn engine_absent`

A named refusal rather than a greyed control, and distinct from
[`models_missing`] on purpose: *"cannot look for text"* and *"could not
find the files to look with"* call for completely different actions, and
the engine's own feature block insists the two never collapse into one
answer.

### `fn nothing_recognised`

Distinct from a failure: the engine worked, the page simply had nothing on
it a recogniser could read. Blank paper and a photograph of a wall both
land here, and so does a page whose ink is too faint.

### `fn already_has_text`

Distinct from [`nothing_recognised`], which reports that the recogniser
looked and found nothing. This reports that it **declined to look**, which
is a different fact with a different remedy — one is "there is nothing
readable here", the other is "there is already text here and I did not want
to double it". Collapsing them would leave the operator with no way to tell
a blank scan from a document that was already recognised last week.

### `fn pages_outcome`

Only shown when the run covered more than one page — a one-page run reports
its words and nothing else, because *"1 page recognised"* is a sentence that
tells the operator only what they already did.

### `fn failed`

The engine's own sentence is appended rather than replaced. `pdfcer-core`'s
error types name specific causes — an encrypted document, a page index past
the end, a model file the runtime rejected — and paraphrasing them here
would produce a second, vaguer account of a diagnosis that was already
precise.

### `fn offer`

It reports the *page*, not the search. That distinction is the whole rule
and the operator stated it: the trigger is *"this document is images"*, and
it is **not** *"this search had no matches"*. A search for a word that
simply is not in a text PDF is an ordinary empty result, and offering to
recognise it would be nonsense — so this sentence says what was actually
established, which is that there is no text on this page for any search to
have found.

### `fn offer_action`

Ellipsis, because it opens the dialog rather than recognising on the spot.
A search bar is the wrong place to start several seconds of work from a
single click.

### `fn layer_blend_label`

Worded as a **position between two things**, not as an opacity: the one
number moves the picture down and the text up at once, and "opacity" would
name only half of what the operator sees move.

### `fn layer_blend_tooltip`

Says what each end of the travel *is*, because the middle of the range is
self-explanatory and the ends are the two states worth reaching
deliberately — read the paper, or read what the recogniser thinks the paper
says, with nothing behind it to argue.

### `fn layer_blend_suffix`

Its own function rather than a literal in the band, for the reason every
suffix in this catalog is: the control shows it and the status bar shows
it, and a unit spelled twice is a unit that will one day be spelled two
ways.
