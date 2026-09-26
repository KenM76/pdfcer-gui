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
