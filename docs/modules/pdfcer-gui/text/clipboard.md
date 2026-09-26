# `text::clipboard` — what the clipboard verbs say on the status row


1. **The object clipboard's four refusals** ([`refusal`]), which are about
   copying *within* pdfcer. These are the original contents of this file and
   the paragraphs below are about them.
2. **The vector copy-out's disclosure and refusals**
   ([`copied_as_vector`], [`copy_out_refusal`]), which are about copying
   *out* of pdfcer — `OPERATOR_REQUESTS.md` O120. The copy-out is the one
   clipboard verb that says something on **success** as well, because it
   alone has two possible operands and the button cannot show which was
   taken.

Each refusal exists because the
alternative is a keystroke that does nothing and says nothing. That is how
the operator experienced the absence of cut, copy and paste in the first
place — *"the standard copy/paste … aren't implemented"* — and a build that
implemented them and stayed silent when it could not act would read
identically.

## Why two of the four name the ENGINE and two name the selection

Because they are different kinds of "no" and an operator's next move differs:

- *nothing selected* / *nothing copied* — **select something, or copy
  something.** The operator's own next act fixes it.
- *a path is selected* — **nothing the operator can do fixes it today.**
  `EditSession` has no verb that puts page content back on a page, so there
  is no sequence of clicks that would make the copy work.

The second kind has to say so, or the operator spends the afternoon trying
different objects. `NO_SURFACE.md` §1c's rule about dated citations applies
to the sentence as much as to the code comment: it says *what pdfcer cannot
do*, not *that something went wrong*.

## Item notes

### `fn cannot_carry`

# Why each subtype earns its own clause rather than one general refusal

Because the operator's next move differs, and in one case the refusal is
protecting them rather than admitting a limit:

| subtype | what they should do instead |
|---|---|
| `/Widget` | select it in Edit mode — a form field has its own copy, which asks the naming question a blind copy would have to guess at |
| `/Popup` | copy the comment it belongs to; the pop-up travels with it |
| `/Redact` | nothing — and that is the point. A redaction mark is a **pending destructive operation**, and pasting one arms a redaction in a document nobody reviewed |

The `/Redact` line is the one that must not be softened into *"pdfcer
cannot copy this"*. It can; it declines to, and an operator who reads a
capability limit will go looking for a workaround for something that is a
safeguard.

The catch-all is **named, not guessed**. `pdfcer-core` may refuse a
subtype this shell has never seen — a ce dimension whose sidecar record is
missing is the documented fourth case — and the honest answer says which it
was rather than inventing a reason for it. Same posture as
[`cut_would_not_survive`] one function up, and the same reason.

### `fn clipboard_refusal`

Split out so the four Win32 outcomes get four different next moves rather
than one shrug. *Another program is holding it* is transient and the answer is
to press the button again; a **partial** placement is the one case where the
clipboard has genuinely changed, and the operator needs to know that what is
there now is neither the old contents nor the whole copy.

### `fn every_refusal_is_a_sentence`

Asserted as a length floor rather than by matching words, because the
property is *"this is a sentence, not a label"*. A four-word refusal is
the failure this whole module exists to prevent, and it is the shape a
future edit would most plausibly introduce while "tidying".

### `fn a_partial_copy_says_what_arrived_and_what_did_not`

The wording trap here is real and one-directional: *"could not be
copied"* over a copy that mostly worked sends the operator back to
press `Ctrl+C` again, which changes nothing and costs them the
afternoon. It has to open by saying the copy happened.

### `fn the_os_marker_counts_objects_and_comments_separately`

`os_marker(0, 1)` was the case that did not exist before 2026-09-05
and is the one an operator now meets most: a comment copied and pasted
into an email says *"1 comment copied from pdfcer"*, not *"1 object"*.

### `fn every_mode_refusal_names_a_mode_the_selector_actually_offers`

`ModeRefusal::line` returns `&'static str`, so *"Edit"* and *"Review"*
are written out rather than built from `crate::text::ribbon`. This is
what stops that being a quiet duplication: rename a mode and this fails
and names the sentence, instead of leaving the operator directed at a
control that no longer exists.

### `fn the_seven_mode_refusals_are_seven_sentences_and_the_gestures_reassure`

`DuplicateMarkup` says *"Nothing has been added"* rather than
*"removed"*, which is why it is asserted separately below rather than
folded into the cut loop: the doubt it answers is the opposite one — a
second comment landed somewhere I cannot see.
