# `reachout` — **does this document reach outside itself?**

One question, asked once when a document opens, answered off-canvas.

## What this closes

**Ken, 2026-08-30:** *"I think pdfcer added support for several button
features and protections for outgoing submits."*

This module is the **detection** half of that. Authoring a button action is
a separate capability the engine also ships — `EditSession::set_button_action`
and `EditSession::button_action` — and no surface in this module writes
anything; it only reports.

## Why detection is a security-shaped question rather than an inventory

`pdfcer_core::forms::scan_javascript` answers *"what would this document run
in Acrobat/Reader?"*, and two properties of the answer decide how a shell may
present it:

- **A widget's primary action lives in `/A`, not `/AA`.** A scan that walks
  only the additional-action dictionary misses the push button that submits a
  form to a web server, and reports a network count of zero about it.
- **`/Next` chaining makes a per-carrier scan unsafe rather than merely
  incomplete.** An action dictionary may name a successor action, and those
  may chain further, so a document can park a benign `/GoTo` where a scanner
  looks and hang the `/SubmitForm` off its `/Next`. The engine's walk follows
  the chain to a bounded depth; a shell that reimplemented the question over
  one carrier would not.

⇒ **A check that under-reports reads as a clean bill of health**, because
silence and safety are indistinguishable to the reader. That is the whole
reason this shell asks the engine rather than inspecting annotations itself,
and the reason [`ReachOut::truncated`] is disclosed rather than swallowed.

## What is disclosed, and what deliberately is not

Three facts, and only when they are **true**:

| fact | why it is worth a sentence |
|---|---|
| it can **submit data somewhere** | the operator's drawing is about to leave the building |
| it can **launch a program** | §12.6.4.5, and it is the one that is not about forms at all |
| it **runs a script when opened** | it has already run by the time they read this |

**Not disclosed:** field-level calculate, format and validate scripts. A form
that computes a total is an ordinary form, and warning about it would train
the operator to dismiss the sentence that matters. `panels::forms` already
lists those for anybody who wants the inventory.

**`scan_truncated` is disclosed too**, and it is the subtle one: it means
the engine stopped walking. A truncated scan that reported *"nothing found"*
would be exactly the clean-bill-of-health failure above, so when the walk
gave up this says *"pdfcer could not finish checking"* rather than implying
an all-clear.

## Why it is a status line and not a dialog

Because pdfcer **executes none of these**. The engine's standing NF4 rule is
that actions are recognised and round-tripped, never run. So nothing is about
to happen, and a modal that stopped the operator to say *"this document
contains a submit button"* would be alarm without a decision attached — the
operator cannot act on it at open time and the drawing is not doing anything.

What they can do is *know*, before they hand the file on or press a button
in another viewer. That is a sentence, not a barrier.

**Render normally, report separately.** Nothing is drawn on the page and
no button is marked.

## Item notes

### `fn field_level_scripts_alone_are_not_worth_saying`

The single most important property here. A form that computes a total
carries calculate and format scripts, and a shell that warned about
those would put a sentence on the status row of every real form an
operator opens — which trains them to ignore the one that says their
drawing is about to be posted to a web server.

### `struct ReachOut`

A struct rather than the engine's whole `FormJavaScript`, because this
shell's question is narrower than the engine's: it asks *"does anything here
leave the document?"*, and most of that type's fields answer a different one
— how much of the document is click-activated, which triggers fired, how many
field-level hooks there are. `FormJavaScript` is `#[non_exhaustive]` and
grows; this projection is what keeps that growth from widening the
disclosure by accident.

### `fn scan`

# Cost, because this runs on every open

One graph walk, bounded by the engine's own `actions_scanned` ceiling — the
`scan_truncated` flag exists because that ceiling is real. It is the same
order of work as reading the outline, which this shell already does on open,
and it happens once rather than per frame.

⇒ Measured rather than assumed is the standing rule here, and this one has
**not** been measured on the benchmark drawing. It is bounded by
construction and it runs once; if a 129,758-object sheet ever opens visibly
slower after this, the scan is the first thing to time.
