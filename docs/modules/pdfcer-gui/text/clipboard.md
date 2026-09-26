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
