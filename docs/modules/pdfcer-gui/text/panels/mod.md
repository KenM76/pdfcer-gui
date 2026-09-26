# `text::panels` — every string the dock's panels show

One area of the catalog described in [`crate::text`]'s header. It covers
the panel bodies in [`crate::panels`]. The three document-structure
panels are in this file; Comments, Fonts, Objects and Properties each have
their own module, and Forms and Pages have their own areas one level up.

| Module | Panel |
|---|---|
| `mod.rs` (this file) | the three **document-structure** panels — Bookmarks, Layers, Signatures — plus [`byte_size`], which several areas share |
| [`attachments`] | the Attachments panel — the files a document carries inside itself, and the four verbs opposite them |
| [`comments`] | the Comments panel — every annotation on the document, what each one is, and the five disclosures a row can carry |
| [`face`] | the **face chooser**, drawn on two surfaces from one module, and the standard-14 disclosure it owes |
| [`fonts`] | the Fonts panel's inventory report |
| [`objects`] | the Objects panel, and the wording of every [`crate::panels::objects::summary::ObjectSummary`] fact |
| [`properties`] | the Properties panel |

## Almost every sentence here is salvaged verbatim, and that is the point

These strings came across from the old shell's `ui_text.rs` (7,912 lines,
1,193 entries) **with their doc comments**, because the doc comment is
usually the record of a defect the wording was changed to fix. Three
examples, all of which are below:

- [`signature_leaves_tail`] is worded as *under-protection* rather than
  as damage, because ISO 32000-1 §12.8.1 makes whole-file coverage a
  `should` — the document is conforming, and an operator told "invalid"
  about a legal file has been misled just as surely as one told nothing.
- [`layers_session_only_note`] exists because a panel of tickboxes over a
  document is, by every other application's convention, an editor — and
  this one is not. Its doc comment carries the reasoning the wording holds.
- [`fonts::font_verdict_removable`] and its four siblings are **two
  words each**, because a full sentence there clips at the dock's edge and
  takes the byte size with it.

⚠ **Rewriting any of those from fresh words re-derives a decision already
paid for**, and the rewrite has no access to the screenshot or the defect
that bought the current wording. Amend a string only with a reason, and
write the reason into its doc comment.

## The three panels in this file share a posture

**Each leads with the shape of what it is about to tell you**, because a
panel headed "Signatures" listing byte counts is the single likeliest
place in this application for an operator to take away more than was said.
The Signatures panel's opener is [`crate::text::trust::panel_intro`],
beside the rest of the trust copy. The Layers panel opens by saying that a
toggle changes what you see and not the document, and that nothing it does
is saved. The Bookmarks panel says when its own reader gave up.

That ordering is not stylistic. A caveat below a list arrives after the
operator has already drawn a conclusion.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have** — and never state
  the ABSENCE of one either, which is the half that rots. A denial is a
  claim about the engine with a shelf life of hours; see the note above
  [`signatures_none`].

## Item notes

### `fn each_structure_panel_leads_with_its_own_limitation`

Each of the three structure panels leads with a limitation, and the
value of doing so is entirely in the limitation being specific. Three
near-identical hedges would satisfy the convention and teach an
operator to skip the first line of every panel, which is worse than
having none.

### `fn the_layers_note_says_a_toggle_changes_the_view_and_not_the_document`

This test replaces `the_layers_note_states_that_switching_is_unavailable`,
which asserted the S3 truth — that the panel had no visibility control
— by pinning the words "not available". S4 gave the panel its control
back (`crate::app::actions::Action::SetLayerVisible`), so that clause
became a lie and came out **in the same commit as the checkbox**.

The test is rewritten rather than deleted because the *thing it
guards* did not go away, it inverted. Two directions now:

1. **The claim that must be present.** A panel of tickboxes over a
   document reads as an editor. "not the document" is the clause that
   says it is not, and it is the single most load-bearing phrase in
   this panel.
2. **The claim that must be absent.** If a later change reverts to the
   S3 wording *without* removing the control, the panel is back to
   describing a program that does not exist — the failure the old
   shell's module header records happening twice, to two different doc
   comments, in this exact file. A control that says of itself that it
   is unavailable is not a copy-edit defect; it is the operator
   concluding the application is broken.

Asserted on clauses rather than on the whole string: a copy edit
should be free, and a capability claim should not.

### `fn the_reset_control_names_what_it_returns_to`

`Action::ResetLayers` restores *the document's own default*, which on
a file that declares a "Confidential" watermark off by default is
emphatically not "show everything". Those are two different acts and
only one of them is a disclosure event, so the control that performs
the safe one must not be readable as the other.

The label alone cannot carry that — "Reset" in a layers panel is
genuinely ambiguous — so the tooltip is where the distinction lives
and this is what keeps it there.

### `fn the_overridden_count_agrees_with_itself_about_number`

Same reasoning as [`layers_count`]: cheap to get wrong, immediately
visible, and it sits directly beside the Reset control where an
operator is deciding whether to click.

### `fn an_overridden_layer_says_which_way_the_document_asked`

The point of [`layer_overridden_tooltip`] is that an operator can see
what they are diverging from without resetting to find out. If both
arms read alike, the row that says "you are showing content this
document hides" — the one with a disclosure consequence — is
indistinguishable from its harmless twin.

### `fn the_delete_promise_and_the_delete_report_name_the_same_quantity`

This is the one piece of arithmetic in the bookmark wording and it has a
genuine off-by-one waiting in it:

* `bookmark_delete_takes_subtree` is handed the shell's **exclusive**
  count (how many are filed *under* the row), because that is what the
  panel can see and what `tree::descendants` returns;
* `bookmark_deleted` is handed the engine's **inclusive** count, because
  `EditSession::delete_outline_item` returns how many items went
  *including* the one that was clicked.

So the second must subtract one to name the same set the first named. If
it did not, an operator promised *"also removes the 11"* would be told
*"along with the 12"*, and the only conclusion available to them is that
pdfcer removed a bookmark it was not asked to.

The fixture is deliberately chosen so the right and wrong answers are
different strings — the discipline the engine's `Pass 156.0` note asks
for after its own delete test survived every sabotage: *"when you assert
that A and B differ, check your fixture can tell them apart."*

### `fn a_leaf_removal_says_nothing_about_a_subtree`

`delete_outline_item` returns `1` for a childless bookmark, and
*"along with the 0 filed under it"* is the shape of sentence that makes
a program look like it is filling in a template. The two branches must
therefore be genuinely different sentences rather than the same one with
a zero in it.

### `fn the_removal_disclosure_names_the_way_back`

`panels::bookmarks::edit`'s header carries the choice: delete is
*undoable* rather than *confirmed*, on the grounds that one press is one
engine command so `Ctrl+Z` restores the whole subtree. A disclosure that
reported the loss without naming the remedy would be the worst of both —
no question beforehand and no way out afterwards that the operator has
been told about.

### `fn the_subtree_warning_agrees_in_number`

It is read beside a button the operator is deciding whether to press, so
*"the 1 bookmarks filed under it"* is exactly the sort of seam that
makes a warning read as boilerplate and stop being read at all.

### `fn the_pages_line_names_the_pages`

The operator's reasonable fear beside a control that takes several
things at once is that the pages are what is going. An outline is a
document-level structure (§12.3.3) and removing a bookmark removes a way
of *reaching* a page, so the sentence is a fact rather than reassurance —
and it must actually mention the pages to do its job.

### `fn the_three_bookmark_states_are_distinguishable`

"Points at a page", "is a heading" and "pdfcer could not follow it"
are one good outcome, one normal outcome and one problem. Collapsing
any two would send an operator looking for a fault in a document that
has none, or reassure them about one that does.

### `fn full_coverage_and_a_tail_read_as_different_answers`

One is reassurance and one is a warning, and both are about a
conforming file. An operator who cannot tell them apart at a glance
gets no value from the panel at all.
