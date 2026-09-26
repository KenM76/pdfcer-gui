# `pdfcer-gui/app/actions/textcommit`

## Item notes

### `fn commit_text_edit`

The body of `Action::CommitTextEdit`, with the variant's four fields as
arguments: which page, which run within that page's decomposition, what the
operator believes is there now, and what they want instead.

`original` is carried through rather than trusted: `canvas::textedit::plan`
is what decides whether the run still says what the caret thinks it says,
and what to do when it does not. Nothing here second-guesses that.

## The three things that must happen in this order, and why

1. **`PageLevelForms::of(doc)` before the edit.** It reads a `Ref` into the
   decomposition cache while `vector_edit` wants `&mut OpenDoc`, so gathering
   it afterwards does not borrow-check — and gathering it afterwards would
   also be measuring a document the edit has already changed.
2. **`report::read_line` before the edit, and again after it.** The *before*
   reading is the one that cannot be recovered later, and it is the whole
   of O213's oracle: a check that can only see the page afterwards cannot
   tell a line that was always at x=312 from one that was at x=72 and moved.
3. **`plan.one_operator` copied out before the closure.** The closure takes
   `plan` by reference, and the refusal classifier needs that one `bool`
   after the engine has answered.

All three are load-bearing rather than stylistic, which is why they are
stated here as well as at their sites.
