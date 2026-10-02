# `canvas::textedit::reflow` — which paragraph the caret is in

One question, asked of the document: *the operator's caret is on run N; which
**block** is that, and is it one `reflow_block` can act on?*

## Why this is a module and not two lines at the call site

`OPERATOR_REQUESTS.md` **O54(b)**: *"I think the paragraph reflow was
implemented ages ago in the pdfcer core, so we should have that option too."*
He was right, and the re-derivation turned up the part that decides the whole
design — the integer this module returns is **not a description of a
paragraph**. It is an **index into a list the engine rebuilds for itself**,
and it means whatever that list says it means.

## THE RECOGNITION IS THE ENGINE'S, AND THIS MODULE HAD IT BACKWARDS


> The block recognition must match the one the caret was placed against —
> `BlockRecognitionOptions::default()`, the same as [`super::plan`]'s. *"The
> question is how did the thing the operator clicked get segmented, and
> asking it of a differently-recognised model would answer about a different
> segmentation."* Not `reflow_recognition_options()` … using it here would
> name a block index the operator's caret never pointed at.

Every sentence of that is true about *paragraphs* and false about *indices*,
and the difference is the whole defect. `EditSession::reflow_block` does not
receive a paragraph. It receives an integer, throws away everything the
caller knew, re-extracts the page itself, and re-recognises it with
[`pdfcer_core::text_edit::reflow_recognition_options`] — a **relaxed**
config that pushes `indent_ratio` out
of reach so ragged-left lines merge into one block instead of fragmenting
into one block per line. On a page with a ruled table it recognises with
the cells of `detect_cell_regions` (`recognize_with_cells`), which puts each
cell in a block of its own after the page's paragraphs. Then it indexes
*that* list, and `block_of_run` builds the same one; a page whose cells
cannot be read is one `reflow_block` refuses as well.

⇒ **So the only correct thing to send is an index into that list.** Asking
the caret's own recognition still answers *"which paragraph did he click
in"* — but numbers it in a list the engine will never build, and hands over
a subscript into a different array. The engine's own doc comment states the
contract in as many words: *"the GUI's caret-block resolution … call THIS
function, so the block index the GUI targets means the SAME block the engine
previews and the surgery re-emits"* — the doc on
`pdfcer_core::text_edit::reflow_recognition_options`. It believed we did.

### What it measured, on `SW41177.pdf` page 0, run 100

```text
caret recognition : block 106 of 144
reflow recognition: block  49 of  70
```

Two failure modes come out of that, and the quiet one is the bad one:

1. **Index ≥ the engine's block count** — 74 of the caret list's 144 indices
   on this page — is refused outright: *"block index 106 is out of range (70
   block(s) recognised)"*. The operator presses Reflow on a paragraph in
   plain sight and is told it does not exist. This is O198's *"seems the
   reflow works with each line but …"*: it works low on the page, where the
   two lists have not yet drifted apart, and stops working further down.
2. **Index < the engine's block count but past the first disagreement** —
   silently re-wraps **a different paragraph**. Nothing refuses, nothing is
   marked, the wrong text moves. No test in this crate could see it, because
   both option sets produce a valid model and a valid index.

The two lists agree at index 0 and diverge at the first place the relaxed
config merges what the default split — which on a business letter is never,
and on a CAD sheet is almost immediately. That is why this shipped: the only
driven check of the feature, `ui-verify`'s `ReflowingAParagraphRewrapsIt`,
runs on `fixtures/paragraph.pdf`, a flush-left six-line paragraph that is
block 0 of 1 under **both** recognitions. A check whose fixture cannot
distinguish the two answers is not a check of which one shipped.

### The trade-off is real, and it is disclosed rather than hidden

The relaxed recognition merges paragraphs the default separates, so the
block the engine re-wraps can be **larger than the line the operator
clicked**. He is owed that fact and he already gets it: the engine's
[`ReflowApplyReport`](pdfcer_core::text_edit::ReflowApplyReport) reports
`lines_before`/`lines_after` and `crate::app::actions::textstyle::reflow`
forwards them to the status line verbatim. A second sentence written here
would be a second author for one fact — the rule that module already keeps.

⇒ Nothing is drawn on the canvas about it. R8b rule 4: applied content
renders exactly as saved content will; the extent goes off-canvas, in words.

## Item notes

### `fn the_block_is_numbered_the_way_the_engine_will_read_it`

A source assertion, and the WEAKER of the two instruments in this module
— the behavioural one below is the real check and should be read first.
This one survives because it fails with a sentence naming the intent,
where a behavioural failure names only a number: both option sets
produce a valid model and a valid block index, so a build using the
wrong one either re-wraps **a different paragraph** than the operator
clicked in — silently, correctly — or is refused for an index out of
range on a paragraph plainly on the screen. Measured on the operator's
own drawing: run 100 is block 106 of 144 to the caret's recognition and
block 49 of 70 to the engine's.

⇒ This assertion ran green for the whole time the WRONG option set was
in place — it was written to hold the opposite, on reasoning about
paragraphs rather than about subscripts. It is inverted here rather than
deleted, because the mistake is symmetrical and the next author will
reach for the caret's recognition for exactly the reason the last one
did.

### `fn block_of_run`

`None` for a page whose text cannot be extracted, a run the model does not
place in a block, or a caret that is not on a run at all — three states that
are one answer here (*"there is no paragraph to reflow"*) and are told apart
by the caller only insofar as it says so.

The run index is the SAME integer in both recognitions — both recognise
one extraction, and `BlockRecognitionOptions` groups runs into blocks
without renumbering the runs. So asking the relaxed model
`block_at(run)` still asks *"which paragraph is the operator's run in"*.
Only the answer's numbering changes, and its numbering is the one the
engine will read it in.
