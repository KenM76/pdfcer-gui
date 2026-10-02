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

### `fn the_mode_refusals_are_distinct_sentences_and_the_gestures_reassure`

`DuplicateMarkup` says *"Nothing has been added"* rather than
*"removed"*, which is why it is asserted separately below rather than
folded into the cut loop: the doubt it answers is the opposite one — a
second comment landed somewhere I cannot see.

### `fn refusal`

One variant — [`Refusal::CutWouldNotSurvive`] — carries a **subtype**, and
its sentence is built around it. The alternative was a second function for
the one data-carrying case, which would put two clipboard refusals on two
surfaces and invite them to word the same idea differently.

Every other arm is still a literal, so `check-ui-strings` still sees them
all in this file.

### `fn cut_would_not_survive`

`pdfcer-core`'s `CutWouldNotSurvive { subtype }`, in the operator's terms.

# Why this exists even though the button is greyed

Because **a chord is not a button.** `Ctrl+X` is dispatched through the
keymap without consulting command enablement, so it reaches the handler
whatever the ribbon is showing. Greying the control removes the *invitation*;
this removes the *silence*.

⇒ And it carries what greying cannot: **which** thing. A greyed button has
one static tooltip and the operator may have several things selected.

# Why each subtype earns its own sentence

A generic *"that cannot be cut"* is true and useless: the operator's next
move differs completely between the three, and only one of them is a
limitation at all.

The `Redact` case is the one that will actually happen, and it is not an
apology. Refusing to put a redaction mark on the clipboard is pdfcer
protecting them from arming a destructive operation somewhere they did not
review — so the sentence says what it is *for*, and offers Delete, which is
almost certainly what they wanted.

### `fn partial_copy`

Rule 4, *"fuzzy never sneaky"*: a copy that quietly took three of four
selected things, or took a comment without its author and its opacity, looks
exactly like one that took everything. Nothing errors and nothing is marked,
which is the definition of sneaky.

# The two halves are different kinds of loss and are said differently

* **`left_behind`** — annotations that will not be on the clipboard at all.
  The operator will notice, eventually, and this is what stops it being a
  mystery. Worded by [`cannot_carry`], reused rather than re-phrased.
* **`thin`** — annotations that *will* paste, and will paste **without their
  author, date, note text and opacity**. This is the one nobody would ever
  report: the mark is on the page, it looks right, and what is missing lives
  in the pop-up. **Corrected 2026-09-05: it used to read *"a pop-up this
  shell does not draw"*, and that stopped being true the day
  `crate::canvas::notepopup` shipped** — which makes the loss *more*
  reportable, not less, because the operator can now open the pop-up and
  find it empty. The sentence is kept for the same reason it was written;
  only its false half is gone. It is `pdfcer-core`'s limit, not
  this shell's — `paste_clip_annotations` plants a modelled markup with
  `add_markup` rather than `add_markup_with` — and the sentence says so
  plainly, because an operator who believes it is their mistake will retry.

# Why it is not reachable today, said rather than implied

The engine's markup carrier holds all four keys itself since `Pass 270.0`
(`MarkupCarry`), so `thin` is zero for every clip this shell parks; and the three
refused subtypes are either routed elsewhere or refuse the whole copy, so
`left_behind` is empty. Both become live the day the selection model can
hold more than one annotation. The sentence is written now because the
alternative is writing it *after* the first silent partial copy.

### `fn os_marker`

It exists because of a toolkit constraint rather than a design wish:
`egui-winit` synthesises `Event::Paste` only when the OS clipboard holds
non-empty text, and swallows the `Ctrl+V` keystroke entirely otherwise — so
without something here, whether paste works depends on what the operator
last copied in another application. `canvas::clipboard::copy`
carries the full account.

# The wording

It is for a human who pastes into a text editor and wonders what they got,
so it says **what was copied and by what**, and does not pretend to be the
data. Naming pdfcer matters more than usual here: the paste may land in an
email, days later, with no other context.

Singular and plural are spelled out rather than `{n} object(s)`, because a
parenthesised plural is the tell of a program that could not be bothered —
and this string's whole job is to be read by somebody who did not expect it.


The clipboard can now carry annotations, and *"1 object copied from
pdfcer"* pasted into an email about a revision cloud is a sentence about
the wrong thing. The operator's own word for these is **comment** — it is
what the panel is called — so that is the word here, rather than the file
format's *annotation* or the engine's *markup*.

The mixed line reads *"2 objects and 1 comment"* rather than *"3
items"*, because the two halves came from two different selections in the
operator's mind and a total tells them nothing about whether the copy took
what they meant.

### `fn copied_as_vector`

It names the OPERAND, and that is the whole reason this is a sentence
rather than a silence. `edit.copy_as_vector` copies the selection when there
is one and the whole page when there is not, and those two outcomes look
identical from the button — the operator finds out which they got when they
paste, in another application, possibly minutes later. A copy that quietly
took the sheet when three parts were selected is exactly the kind of thing
`DEFECTS.md` D4a calls *a sentence describing a different world than the one
on screen*, one step removed.

It does **not** list the clipboard format names. `image/svg+xml`,
`CF_ENHMETAFILE`, `CF_DIBV5` are wire identifiers — they belong in the trace,
where a developer looks, and `crate::clipboard::ClipFormat::name` is where
they live. What an operator can act on is *how many ways the receiving
program may read it*, and above all the promise that at least one of them is
vector, which is what the count and the second clause carry between them.

### `fn copy_out_refusal`

The [`CopyOut::WouldDegrade`] arm is the one this whole feature is built
around, and it is the reason a refusal is better than a success here. Placing
only the raster formats would produce a paste that **works**: Word accepts it,
it looks right at 100%, and it is a flat picture that cannot be scaled,
recoloured or taken apart. The operator would discover that days later and
report it as *"pdfcer's copy doesn't paste as vectors"* — indistinguishable
from the feature not existing, except that it cost them the time to find out.

⇒ So the sentence says the vector form could not be made **and** that nothing
was copied, in that order: the cause first, because the operator's next move
(try a different page, or export to SVG and place the file) depends on it.

Every arm ends by saying what is still on the clipboard. `native-clipboard`
stages every handle before it opens the clipboard, so all of these except the
partial-placement case leave the previous contents intact — which is a real
reassurance and not a platitude, because the operator may have had something
there that took work to produce.

### `enum ModeRefusal`

# Why the operand is in the variant and not only the verb

Because the remedy differs by operand, and the remedy is the whole point of
saying anything. In **Review** a paste of a comment is permitted and a paste
of page content is not, so *"this mode cannot paste"* would be false half
the time and useless the other half. The six sentences below name the mode
that CAN do it, which is a fact only the operand determines.

# The mode names are literals here, and a test pins them

`line` returns `&'static str` because [`crate::app::status::decline`]'s own
`line` does, and threading a `String` through that enum for one family would
change nineteen arms. So *"Edit"* and *"Review"* are written out rather than
built from [`crate::text::ribbon::mode_edit`] — and
[`tests::every_mode_refusal_names_a_mode_the_selector_actually_offers`]
asserts they are the selector's own words, so a rename of a mode fails here
instead of leaving the operator directed at a control that no longer exists.

### `fn line`

Each one does three things, in this order, and the order is the design:
it names **what was on the clipboard or under the pointer**, so the
operator knows pdfcer understood the gesture; it states **the stance**
rather than an error, because nothing went wrong; and it names **the
mode that can do it**, because that is the operator's next move and it
is one control away.

The three cut sentences add *"Nothing has been removed"*. A cut that
is refused after the operator has watched a selection sit there is the
one case in this family where they might reasonably fear the document
changed, and rule 4 says the disclosure goes where the doubt is.
