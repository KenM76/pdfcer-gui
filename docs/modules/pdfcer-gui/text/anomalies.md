# `text::anomalies` — every word for *"the file contradicted itself, and
pdfcer decided"*

The catalog behind the two surfaces that disclose
[`pdfcer_core::document::Document::load_anomalies`]: the status bar's one
elided line ([`crate::app::status::disclosure`]) and the Document-properties
detail beneath it ([`crate::panels::docprops`]). Both read the **same**
derivation in [`crate::app::status::anomalies`], so the words here are
written once and cannot drift between the glance and the answer.

## What this catalog is actually about, and why the wording is careful

Engine `Pass 283.0`, decision 145 — *"fail-clean never meant refuse"* — was
written against a real file of the operator's. A 46 KB drawing that opens in
Acrobat was **refused whole** by pdfcer because its document catalog names
`/PageMode` twice with two different values. His ruling, verbatim:

> *"acrobat just picks one — but what if it is the wrong one? Would be great
> if the user could choose or also have pdfcer pick one for them. We should
> be making pdfcer so that it opens pdfs that have errors, and have a way
> that it manages those errors such that they aren't fatal, and if the user
> can intervene in a decision that should always be an option along with them
> not having to intervene."*

Three obligations fall out of that sentence and every string below is shaped
by them:

1. **Not fatal.** The document is open and usable before a word of this is
   read. So nothing here is phrased as a problem to resolve, a question to
   answer, or an action to take — the operator may read none of it and lose
   nothing.
2. **The operator can see WHAT WAS CHOSEN BETWEEN.** For a duplicate key the
   engine deliberately carries *both* values rather than a count, because
   *"a count says pdfcer chose; only the pair lets you show the operator what
   it chose between"*. [`duplicate_key_row`] is the reason that pair exists,
   and it is why [`value`] is in this file rather than a number being
   printed.
3. **The file is the subject, not pdfcer.** Every sentence names what *the
   file* said, then what pdfcer did about it. `crate::text::panels::docprops`
   settled this register for the recovered-index note next door — *"pdfcer
   had to repair this file" would read as pdfcer struggling; the file is the
   thing that is damaged, and the operator's next question is about the
   file.*

## Why there is no "this file opened cleanly" string

There is a true one available — `load_anomalies()` is empty for a sound file
and the engine tests that control — and it is deliberately not written.
`DEFECTS.md`'s "Not defects" table records what a permanent all-clear does to
a status bar: *"The first thing a user reads is the app talking about
itself."* A reassurance shown on every document forever costs bar width on
every document forever, and **R9** says an inapplicable capability renders
nothing rather than a placeholder saying it is inapplicable. Both surfaces
draw nothing for a clean file, exactly as the recovered-index note beside
them does.

## Contract with the rest of the crate

Pure functions over already-derived values. Nothing here reads a
[`pdfcer_core::document::Document`], counts anything, or decides what to
show; that is [`crate::app::status::anomalies`]' job. This file only turns
decided facts into English, which is what makes **R4** (`check-ui-strings`)
satisfiable: the two drawing sites contain no literal an operator can read.
