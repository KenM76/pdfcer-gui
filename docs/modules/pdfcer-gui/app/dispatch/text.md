# `app::dispatch::text` — the caret, and the commands whose operand it is

## What is here

| id | what it does |
|---|---|
| `edit.text`, `edit.add_text` | **arm** the caret |
| `edit.reflow_block` | **act** on the paragraph the caret is in |

The arming pair sits with the reflow rather than in [`super`], and that is
the seam rather than a convenience: these are the commands whose subject is a
text caret, so a reader asking *"what can I do to page text"* has one file to
read. Splitting on size alone would leave one subject in two places, which
costs more than it saves.

## Why a command with no operand in its id needs a module of prose

`edit.reflow_block` names a **block**, and nothing in the invocation says
which. Every other id on the Edit tab either arms a tool (`edit.text`) or
acts on a selection the shell already holds (`edit.cut`). This one has to go
and *find* its operand, and the answer is a chain:

```text
the draft in egui memory  →  page + Anchor::Run { run }  →  block index
```

⇒ Every link can fail, and **each failure means something different to the
operator**. That is the whole content of this module: a refusal per link,
each saying the thing that gets them unstuck, instead of one silence.

| what is missing | what they must do |
|---|---|
| no caret at all | click in the paragraph first |
| caret on bare page (`Origin`/`Box`) | the caret is placing NEW text; there is no paragraph yet |
| run not in a recognised block | this text is not laid out as a paragraph pdfcer can re-wrap |

## Why the CARET and not a selection rectangle

`use-the-conventional-interaction-never-invent-one`: in every word processor
the operator has ever used, a paragraph command acts on the paragraph the
**insertion point** is in. Word does not ask you to select a paragraph to
change its justification, and neither does this.

It also happens to be the only thing available — the shell's other
selections are annotations, widgets and vector objects, none of which is a
text run — but the convention is the reason, and it would still be the
reason if a run-selection existed.

## The mode guard is in the registry, not here

`edit.reflow_block` is registered `enabled_when("edit.content")` — a reflow
rewrites the page's content stream, so a reading stance must never offer it.
[`arm`] re-checks `capabilities().edit_content` for `edit.text` because that
id is *also* reachable from the tool row; the reflow is reachable only from
the Edit tab, so the registry's own gate is the single gate.

## Item notes

### `fn reflow`

Takes `&Status` rather than `&mut OpenDoc`: this resolves an operand and
raises an [`Action`]. The document is changed by
`crate::app::actions::textstyle::reflow`, one funnel later, which is where
every ENGINE-side refusal is worded — the added-text guard, the encrypted
document, the composite font. Those are about the page and the session; this
one is about the caret, and they are deliberately not merged.

⚠ The engine's refusals are a **set**, not one: the commonest on a real CAD
sheet is the composite-font one, not the save-and-reopen one.

### `fn decline`

One helper for all of them, so a further refusal cannot be added that
traces but does not tell the operator — the asymmetry that makes a feature
look broken while the log says it declined politely.

# It writes to the DECLINE slot, and that is not interchangeable with
the disclosure one

`crate::app::actions::record_note` draws under **`⚑ About your last edit:`**.
A correct sentence in that slot after a press that changed nothing is a small
lie told confidently — Ken: *"I haven't seen the reflow option actually work
with anything when I press it."* Nothing happens here, so the slot that says
so is `⊗`, and [`crate::app::status::decline::record_reflow`] is the door to
it.

⇒ **This function takes no `epoch`, and must not.** A disclosure is stamped
with the edit it describes so it can go stale and retire itself; a decline
describes **no** edit and is retired by the operator's next command instead.
An epoch here would be dating a sentence against an edit that never happened.
