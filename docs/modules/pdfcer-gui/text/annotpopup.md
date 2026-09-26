# `text::annotpopup` — every string the note pop-up on the canvas shows

The copy for [`crate::canvas::notepopup`] — the window that opens when an
operator clicks a comment on the page, and the tooltip that appears when
they hover one.

## The surface this exists for, and the report that commissioned it


> *"I could add a yellow sticky note but even in read mode I don't think I
> could figure out how to read it. the review features should look and act
> the same as they do in Acrobat Reader."*

Before this catalog there was **no string anywhere in the crate that
displayed a note's `/Contents` on the canvas**, in any mode. The only route
to a comment's words was the Comments panel, on a tab Read is not shown.

## Why this is not part of [`crate::text::panels::comments`]

Because they are two surfaces answering two questions, and the wording
follows the question rather than the data.

The panel is a **work list**: its rows are headed by subtype and page
because a reviewer scanning forty of them needs to tell two clouds on sheet
three apart, and every caption on a row is a *disclosure about the list*.
The pop-up is **one comment, beside the thing it is about**: the operator
already knows which annotation they clicked, so a heading that repeated the
page number would be answering a question the click just settled.

⇒ Which is why, for instance, [`popup_heading`] takes no page number and
`comment_row_heading` does.

## What is shared rather than restated

**The byline.** `crate::text::panels::comments::comment_row_byline` is
called directly by the pop-up rather than copied here, and that is
deliberate: it carries a settled ruling about `/M` — §12.5.2 gives its type
as *"date **or** text string"* and requires a reader to accept any format,
so pdfcer shows it verbatim rather than writing a parser whose failure mode
is rejecting a legal value. Two surfaces showing one comment must not show
two different dates for it, and the only way that cannot happen is one
function.

## R9 governs what is ABSENT here, and two absences are deliberate

*"An unavailable capability renders nothing, not a disabled stub. Greying
is reserved for temporarily unavailable, and must explain on hover."*

1. **There is no Reply control on THIS surface, and the reason changed on
   2026-09-06.** It used to be *"`pdfcer-core` v0.38.0 reads `/IRT` and
   `/RT` and has no verb of any kind that writes either"*, filed as
   `request_a_reply_can_be_read_and_never_written.md`. **That is no longer
   true**: `EditSession::add_reply` shipped as `Pass 253.0` and this shell
   authors replies from the **Comments panel**, whose catalog carries the
   wording (`crate::text::panels::comments::comment_row_reply` and its four
   neighbours).

   ⇒ So the absence here is now a **scope** decision rather than a
   capability one, and the distinction matters: an R9 absence is a
   statement about the program and expires when the engine moves, while
   this one is a statement about one surface and expires when somebody
   decides the canvas window should compose as well as display. The pop-up
   **shows** the thread already — [`popup_replies`] and its two neighbours
   — so adding composition here is wiring, not a new capability.
2. **There is no Accepted/Rejected control and no string for one.**
   `/State` and `/StateModel` (§12.5.6.4 Table 171) are **absent from the
   engine entirely** — zero occurrences, read or write. Filed as
   `request_review_status_is_not_modelled_at_all.md`.

⇒ The one place this catalog *does* speak about a missing capability is
[`popup_read_only`], and the difference is the rule: Read mode's inability
to edit is **temporary in the operator's own hands** — the mode selector is
two clicks away — so it is exactly the case R9 permits to be explained.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **The operator's own words are never decorated.** [`popup_body`] is a
  passthrough for the same reason `comment_row_body` is: the operator is
  reading somebody else's remark, and a catalog entry that framed it would
  be putting pdfcer's voice inside a quotation.
- **Rule 15**: never a bare *dimension*. [`tests::no_string_here_says_a_bare_dimension`]
  sweeps every entry, exactly as the Comments catalog does — this is a
  catalog, which is the kind of file where a bare noun slips in during a
  late reword.

## Item notes

### `fn truncate`

**Characters, not bytes** — slicing a `String` by byte index panics in the
middle of a multi-byte character, and a note is arbitrary operator text
that may be in any script. Newlines are collapsed to spaces for the same
reason the length is bounded: a tooltip is one gesture's worth of
information, and a note's own paragraph breaks would make it a document.

### `fn no_string_here_says_a_bare_dimension`

The same sweep `crate::text::panels::comments` runs, over this catalog,
and for its reason: **ce dimensions** are the ones pdfcer authors and
**pdf dimensions** are CAD-exported page content, they have opposite
properties, and the ambiguity has already sent one investigation down
the wrong path. A catalog is exactly the kind of file where a bare noun
slips in during a late reword, so this is swept rather than reviewed.

### `fn a_long_note_is_cut_and_says_so`

The ellipsis is the disclosure — rule 4's *"an inference the operator
cannot see still owes a report"* in its smallest form. Without it a
note truncated mid-sentence reads as a note that ends mid-sentence.

### `fn a_multibyte_note_is_cut_safely`

The failure this guards is not cosmetic: slicing a `String` by byte
index inside a character panics, and the panic would be *in the frame
that is drawing the tooltip* — the worst available outcome, on a
document whose only fault is being written in a script this project's
tests do not otherwise use.

### `fn an_anonymous_note_gets_no_byline`

`/T` is legitimately absent — it means *anonymous*, never *unknown* —
so a placeholder byline would turn a correct fact about the file into a
claim about a person. `crate::text::panels::comments::comment_row_byline`
makes the identical ruling and this is it holding on the second
surface.

### `fn a_note_with_no_words_still_says_something`

The hover must answer *something* — a comment icon that produces no
tooltip is indistinguishable from one the hover missed, which is the
exact ambiguity this feature exists to remove.

### `fn only_a_ce_dimension_heading_says_ce_dimension`

Asserting both directions, because a heading function that returned the
ce-dimension wording for everything would pass a one-sided check and
would relabel every `/Line` markup an operator drew.

### `fn only_a_write_that_reached_nothing_says_so`

# Both directions, and the silent direction is the one that matters

`set_annotation_open` reaches up to two objects — the annotation's own
`/Open` when its subtype has one (§12.5.6.4 Table 172 gives it to
`/Text`; Table 169 gives it to nothing else), and the `/Popup`
companion's when there is one. **Any** write is a real edit with a real
undo entry, and confirming it would be noise: the tick is on screen and
it is what the operator asked for.

A build that spoke on every call would make the one sentence that
carries information — *"there was nowhere to record this"* —
indistinguishable from the two that carry none, which is the failure
this project calls a confirmation nobody reads.

### `fn popup_heading`

No page number, unlike the panel's row heading — see the module header. The
subtype is the file's own spelling (`Text`, `Square`, `Line`), because that
is the word the operator used when they placed it and the word every other
surface in this shell uses for it.

### `fn popup_ce_dimension_heading`

Project rule 15 at the point of use, and the same shape
`comment_row_ce_dimension_heading` takes: a **ce dimension** is a `/Line`
annotation, and the bracketed subtype is not decoration — the reason a
dimension appears on this surface at all is that it *is* one, and a heading
that hid that would quietly contradict the argument that let it in.

### `fn popup_no_note`

Worded as a fact about the document rather than as missing data. On markup
pdfcer itself drew this is the **expected** state: `MarkupSpec` has no
contents field on any variant, deliberately, so a shape this shell authored
has no note until somebody writes one.

### `fn popup_ce_dimension_note`

Its `/Contents` is **regenerated from the measurement** by
`author_dimension`, so it is never a remark somebody wrote and a note typed
over it would be silently thrown away. Rule 15 and R9 together: the
capability is not withheld with a greyed control, it is explained.

### `fn popup_close_tooltip`

# What changed, and why the wording had to change with it

This entry used to justify itself with *"`pdfcer-core` v0.38.0 has no verb
that can change an existing annotation's `/Open`."* `Pass 253.3` shipped
`EditSession::set_annotation_open` and that reason expired. The behaviour
did **not** change and must not: closing a bubble you were reading is a
reading gesture, and wiring it to the document would give a reviewer one
undo entry per comment they glanced at and a dirty file after a session in
which they altered nothing. `crate::app::actions::annot::AnnotAction::SetOpen`
carries the whole argument.

⇒ But an unchanged behaviour with an expired reason needs a **new**
sentence, because the old one now reads as a limitation that is not there.
So this points at [`popup_open_default`], which is the explicit act that
does write. Rule 4: the operator is told what this did *and* where the
other thing lives, which is the difference between an honest boundary and
a dead end.

### `fn popup_add`

Two labels rather than one, because *Add* and *Edit* are different acts and
a reviewer scanning a sheet's pop-ups can tell at a glance which comments
have been written on.

### `fn popup_remove_tooltip`

The distinction the engine draws by having two verbs:
`clear_markup_note` *"does **not** delete the annotation — the shape stays
and undo restores the words"*, while `delete_annotation` is the other
thing. Both controls are on this pop-up, so the difference has to be
legible without pressing either.

### `fn popup_delete_tooltip`

Three things, and each is required by `docs/core-api/03-capabilities.md`
§3.4: what it removes, that **delete is not redaction**, and — implied by
the second — that a previous revision of the file may still hold it.

### `fn popup_replies`

The count is in the heading rather than left to be counted, because a
thread scrolled past its third entry is one an operator cannot count by
eye — and the number is what tells them there is more below the fold.

### `fn popup_reply_is_group_member`

Rule 4, and it is the same disclosure `comment_row_is_group_member`
makes for the same reason: for a group subordinate the standard says its
own `/Contents`, `/M`, `/T` and the rest *"shall be ignored"* in favour of
the group primary's. `pdfcer-core` deliberately does not apply that rule,
so what is shown here is the raw dictionary value — and another conforming
reader will legitimately show something else.

### `fn popup_read_only`

# The one place this catalog explains an absence, and why R9 allows it

R9 reserves an explanation for a capability that is **temporarily**
unavailable, and Read mode is the purest example of that in the whole
program: the capability is not missing, the operator has *chosen a stance*,
and the control that changes it is a labelled three-position selector on
the ribbon. Saying so is not a placeholder; it is the answer to *"why can I
read this and not fix the typo?"*, which has exactly one correct answer and
it is short.

It names the mode to switch to rather than the mode you are in. *"You are
in Read mode"* is a fact the badge already states; *"Review lets you edit
comments"* is the sentence that gets the operator to the thing they wanted.

### `fn popup_locked`

§12.5.3 Table 165 bit 8: the file says the user interface *"shall not"*
allow the annotation's properties to be changed. R83 — the controls are
omitted rather than offered and refused — and this sentence is why they are
not there, because otherwise a locked comment is indistinguishable from a
broken pop-up.

### `fn popup_tooltip`

# Why the tooltip exists when a click opens the whole window

Because it is the cheap half of the same affordance and every reader in the
class has it: hovering answers *"what is this?"* without committing to
opening anything, which is what a reviewer skimming a sheet of forty marks
is doing. Acrobat shows author and text on hover; so does this.

# It truncates, and the truncation is visible

A tooltip that grew to a paragraph would cover the drawing it is about — a
note is arbitrary operator text and can be a page of it. The ellipsis is
the disclosure: it says there is more, and clicking is how you get it. The
pop-up itself never truncates.

### `fn popup_note_hint`

Escape **writes**, which is the opposite of what the key usually means, so
the sentence has to say it and has to name the control that does discard.
The ruling is at `crate::panels::comments::editor::escape_commits`; this
surface's half of it is `crate::canvas::notepopup::controls::save_draft`.

It announces a keyboard route at all because a reviewer typing has their
hands on the keyboard, and a keyboard route that nothing announces is a
keyboard route nobody finds.

### `fn popup_open_default`

# Why *Open by default* and not *Save open state*

Because the second names the mechanism and the first names the effect. The
operator's question is *"will this comment be showing when somebody else
opens the drawing?"*, and the label is the answer to it. It also reads
correctly as a checkbox caption in both states, which *Save* — a verb — does
not.

It is deliberately **not** worded as an instruction about the current
bubble. Ticking it does not open or close anything on screen: the operator
is already looking at the window, and moving it under them as a side effect
of recording a preference would be the surface acting on a gesture nobody
made.

### `fn popup_open_default_tooltip`

It names **the file** and it names **undo**, and both halves are
required. The first because this is the only control in the pop-up whose
effect is invisible on screen — nothing about the window changes when it is
pressed. The second because it is the only control in the pop-up that
*reads* like a view setting and is in fact a document edit, and an operator
who pressed it expecting a preference would otherwise find an entry on their
undo stack with no idea what put it there.

### `fn open_state_written`

# The one outcome an operator cannot tell from a defect

`set_annotation_open` writes `/Open` on the annotation only when its subtype
has one — Table 172 gives it to `/Text` and Table 183 to `/Popup`, and
**nothing else in Table 169 carries the key** — and on the `/Popup`
companion only when there is one. An annotation with neither is a legal,
ordinary shape: the call succeeds, writes nothing, and pushes **no undo
entry**.

The affordance is gated on `crate::canvas::notepopup::model::can_record_open_state`
under R83, so this should be unreachable from the control. It is worded
anyway, because *"the button did nothing and said nothing"* is
indistinguishable from a broken build, and because the gate is this shell's
reading of the subtype rules while this sentence is the **engine's own
answer** — if the two ever disagree, the operator hears about it rather
than the disagreement being swallowed.

# `None` on success, and that is not silence

A write that landed needs no sentence: the tick is on screen, it is what
the operator asked for, and a confirmation for every ordinary success is
the noise that makes the exceptional message invisible. Same rule
[`crate::text::panels::comments::reply_posted`] follows.
