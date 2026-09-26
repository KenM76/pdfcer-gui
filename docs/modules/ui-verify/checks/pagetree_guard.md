# `ui-verify/checks/pagetree_guard`

`a_save_that_would_produce_blank_pages_is_refused` — **the operator's own
bug report, driven through the running binary.**


# What this is for


> *"I tested deleting pages from a pdf. when I open the document in Acrobat
> there are blank pages at the end of the document equalling the number of
> pages I deleted."*

`pdfcer-core` v0.38.0's `delete_pages` decrements `/Count` on the removed
page's **immediate parent** and on no ancestor above it, so on any document
whose page tree has more than one level the root goes on declaring the
pre-delete page count. Acrobat builds its page list from the root `/Count`;
pdfcer walks `/Kids`. `crate::pagetree` — in the shell — refuses the save
rather than handing him the file.

## Why this needs a driven check at all, when the unit tests are green

Because the unit tests can only prove that `write_copy` refuses when it is
called with a stale document. They cannot prove that **pressing the delete
command and then pressing save in the running program** reaches `write_copy`
at all — and the whole class of defect this project keeps finding is a
correct mechanism that nothing calls. `check-verb-coverage.sh` exists
because of exactly that, and `RESUME.md`'s standing note is blunter: *"a note
is not a mechanism"*.

The chain being asserted here has four links, and the failure of any one of
them is invisible to `cargo test`:

1. `pages.delete` has a dispatch arm and reaches `EditSession::delete_pages`;
2. `file.save_copy` reaches [`crate::app::save::write_copy`];
3. the guard runs there, on the funnel, and refuses;
4. **the operator is told**, on an off-canvas surface, in a sentence.

## The fixture is PINNED, and a flat one would make this vacuous

`--pdf` is ignored. This check opens `fixtures/nested-page-tree.pdf` and
nothing else, and says so in its notes when a `--pdf` was supplied and
thrown away — because a sweep that silently ignored a flag is
indistinguishable from one that honoured it
(`three_clicks_round_a_hole_measure_the_hole`'s standing rule).


## The oracle: three things, and the third is the one no trace can fake

| # | assertion | what its failure means |
|---|---|---|
| 1 | `save-pagetree … levels=4 bad=2` appears | the guard ran, on a tree deep enough to exhibit the defect, and saw it |
| 2 | **no file exists at the target** | the guard traced a refusal and wrote the file anyway |
| 3 | the `status-group:edit-disclosure` region is on screen | he was **told**, rather than left with a save that silently did nothing |

Assertion 3 is the one this project has learned to insist on. A refusal
with no sentence is this shell's founding defect shape — a control that is
pressed and does nothing — and it is worse here than usual, because the
operator has just deleted pages and is pressing save: a silence reads as
*"it saved"*, and he goes looking for a file that is not there.

The region carries a **rect**, not the text. `status::disclosure` publishes
`ui_rect(region, rect)` and no more, so this check can prove a sentence is on
screen and cannot prove which sentence. The words are asserted headlessly in
`crate::text::pagetree::tests`, which is the right split: the catalog owns
the wording, the harness owns the reachability.

## What this does NOT cover

**That Acrobat shows blank pages.** That claim is the diagnosis, it was made
with `pdfcer dump-object` and an independent page-tree walk, and it is
recorded in the engine request. Nothing in this repository can drive Acrobat.
