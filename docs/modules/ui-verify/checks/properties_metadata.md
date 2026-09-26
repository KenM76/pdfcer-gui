# `ui-verify/checks/properties_metadata`

`properties_metadata_round_trips` — typing a title reaches the document,
and undo takes it back out of the box as well as out of the file.


It was **passing** before that session and it has **not been re-run since**:
the machine's pointer and keyboard belonged to another track, and a driven
run cannot share them. What changed here is which control opens the surface
(see [`open_document_properties`]); nothing about the four assertions moved.
⇒ A reader looking for evidence that the metadata panel still works after
the move will not find it in this file. Run it.


> *"the document properties are still always visible in the properties tab.
> it needs to get out of there and be in its own document properties tab."*

The four `/Info` editors were the last section of the **Properties** panel
and are now a panel of their own, `crate::panels::docprops`, opened by
`file.document_properties` on File ▸ Document. So this check clicks a
different ribbon item and brings a different dock tab to the front. It
asserts the same four things about the same four boxes.

★ **The region names did not change** — `properties.info` and
`properties.info.N`. That was the moving session's decision and it is stated
at the constants: one change at a time, so that the next run's verdict is
readable rather than being a race between two edits neither of which was
driven.

# The gap this closed originally

`file.properties`' own tooltip commissioned this from S3 until the split:
*"The document's own title, author, subject and keywords, and the properties
of whatever is selected on the page."* Only the second half existed, on a
recorded blocker — *"needs a `/Info` accessor that `pdfcer-core` does not
expose on `Document` at all"* — which was true when written and false when
read.

# ★ The assertion that is the whole check, and it is the SECOND one

Not *"a commit was traced"*. That proves the keystrokes arrived and the
action was raised, and it is satisfied by a build where the value never
reaches the document at all.

The real assertion is that **tabbing away a second time commits nothing**.
The panel's commit rule is *focus left AND the draft differs from what the
document holds*, so a second departure from an untouched field is silent
**only if the value is genuinely in the document now**. If it is not, the
draft still differs, and every focus change writes it again — a field that
looks edited, produces an undo entry per glance, and holds a value the file
does not have.

# ★ And the third: undo has to reach the BOX, not only the file

The drafts are re-seeded whenever `doc.edit_epoch` moves, and that is what
makes `Ctrl+Z` work here. Without it the box would still show the title the
document no longer has, and the next focus change would write it straight
back — **an undo the panel silently reverses**, which is worse than an undo
that does nothing because the operator watched it succeed.

That failure is invisible to every unit test in the crate: `InfoDrafts::sync`
can be tested, `set_info_field` can be tested, and the epoch bump can be
tested, and the defect lives in whether the three are connected across a
frame boundary. It is the same shape as every other check in this
directory.

# What this does NOT prove

That the bytes in a saved file are right. `save_copy` and a second process
reading it back is what proves that, and `checks::save_copy` already owns
that shape. This proves the panel and the session agree, which is the link
that did not exist yesterday.
