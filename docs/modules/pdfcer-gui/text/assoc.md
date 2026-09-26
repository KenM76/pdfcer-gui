# `text::assoc` — every word O173 puts on screen

`OPERATOR_REQUESTS.md` **O173**: *"we should have an easy way to make
pdfce-gui our default opener for pdfs. Ask once with a don't show me again (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
check box option. Then it should be in the top of our settings as a button
to execute the changeover."*

Three surfaces, one conversation, so one module: the **ask-once dialog**
that offers it, the **Settings group** that is where the offer lives
afterwards, and the **state line** that says what Windows actually thinks.
Plus the two registry strings Windows itself displays, which belong here for
the same reason every other displayed string does — they are read by the
operator, in the *Open with* menu and in Windows Settings, and a catalog
that held every string except the two with the widest audience would be a
catalog with a hole in it.

## ★★★ The sentence this module is written around

> **Windows will ask you to confirm.**

Every word here is shaped by a platform fact: **no program can make itself
the default PDF viewer on its own.** Windows 10 and 11 store the choice
behind a hash only Explorer can compute, and anything written there by an
application is detected and discarded. See [`crate::app::assoc`] for the
mechanism and the citation.

So the button cannot promise what its name suggests, and the wording has to
be honest about a two-step act without turning into an essay about the
registry. The shape chosen: **say what pdfcer does, then say what is left
for the operator, in that order, in two short sentences.** The button label
ends in an ellipsis for the same reason every other label that opens
something does — it is a promise that another surface is coming.

⇒ What is deliberately NOT written here: any string claiming the default was
changed. [`is_default`](crate::app::assoc::is_default) is the only thing
allowed to make that claim, and it makes it by reading what Windows says
rather than by remembering what pdfcer did.

## Vocabulary

- **Windows**, named outright. This is one of the few places where the
  operating system is a participant in the conversation rather than the
  floor it happens on, and *"the system"* would read as pdfcer being coy
  about which program it is talking about.
- **Open PDFs with pdfcer**, not *"file association"*, not *"default
  handler"*, not *"ProgID"*. The operator's question is *"does
  double-clicking a drawing open this?"* and the words should be that
  question's words.
- **Ask again**, not *"show this again"*, in the checkbox — the thing being
  suppressed is a question, not a notification.
