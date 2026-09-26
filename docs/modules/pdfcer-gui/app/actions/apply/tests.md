# `pdfcer-gui/app/actions/apply/tests`

## Item notes

### `fn an_undo_is_an_edit_and_moves_the_epoch_like_one`

# The two failures this pins, and why neither is visible anywhere else

1. **The epoch.** A build whose history arm called `EditSession::undo`
   directly — `Arc::get_mut(&mut doc.session).map(EditSession::undo)`,
   which is the obvious three-line version — would restore the bytes and
   leave `edit_epoch` where it was. Every count anybody could read from
   the engine would then be correct, and the decomposition, the
   page-text cache, the font inventory and the canvas selection would all
   go on describing the revision the operator just left. That is the
   build `tools/ui-verify`'s `undo_redo_round_trip` catches from outside
   the process; this is the half that can be caught from inside it.
2. **The empty stack.** The decline must cost nothing: no epoch bump, no
   dropped texture, no cancelled raster. A bump here would dissolve the
   operator's selection and discard several caches to record that
   *nothing happened* — which is `crate::app::save` §3.1's argument
   about a save, arriving at the same answer from the other direction.

# Why `is_modified` is asserted as well as the epoch

Because the epoch alone cannot tell an undo from any other edit: it
counts revisions, and it only ever goes up. `EditSession::is_modified`
asks the **dirty set**, which is the same question a save asks, and its
own doc comment says in as many words that *"an edit-then-undo reports
`false`"*. So it is the one available proof that the document really
went back rather than merely forward again — and the pair of them
together is the whole claim: *the document is where it started, and the
shell knows the revision changed.*

### `fn a_verbs_disclosure_is_live_for_the_revision_the_edit_produced`

[`plant_edit_disclosure_for_test`] proves the status bar can *draw* a
disclosure. It cannot prove [`vector_edit`] ever *records* one, and it
cannot prove the stamp is right — which is the failure this test
exists for, because that failure is silent in both directions:

- Stamp the epoch the edit ran **against** (the pre-bump value) and the
  sentence is invisible from the moment it is written. Nothing errors,
  no test that plants its own value notices, and the operator simply
  never learns their rectangle became four lines.
- Fail to record at all and the same thing happens, with the trace
  still cheerfully printing `disclosures=…` — which is exactly the
  "recorded, not disclosed" state this work was written to end.

So the edit closure here returns a disclosure list the way a real
`move_node` over an `re` rectangle does, and the assertion is made
against the epoch the *document* ends up on, read back through the
public accessor the bar uses.

### `fn a_disclosure_is_hidden_once_the_document_moves_past_it`

The staleness rule, and the whole reason nothing anywhere has to
remember to clear this sentence: an undo bumps the epoch, the epoch no
longer matches, and the bar stops drawing it. The comparison IS the
mechanism, so it is pinned rather than trusted — the same test, for the
same reason, as
`crate::panels::forms::edit::tests::a_disclosure_is_hidden_once_the_document_moves_past_it`.

Both directions matter and both are asserted. A *later* revision must
not show a note about an earlier one (the undo case, and the ordinary
"they carried on editing" case). An *earlier* one must not either —
that pairing is unreachable through `vector_edit`, which only ever
stamps the epoch it just produced, and it is asserted anyway because
the filter is what makes it unreachable.

### `fn selecting_from_the_objects_panel_produces_an_ordinary_canvas_selection`

The operator, 2026-08-26: *"when I have an object selected like text the
Tool tab doesn't switch to giving me the editable stuff for that object."*

There is ONE notion of *"the thing I am working on"* — the canvas
selection. A panel-local `focus` beside it is a second, and a panel reading
the second while the operator writes the third is the shape of that report.

This asserts the binding that holds it: the Objects panel raises
`Action::SelectObject`, and what it produces is an **ordinary canvas
selection**, indistinguishable from one made by clicking the page. That is
the property that makes the Properties panel, the row highlight, the
handles, Delete and every Format verb agree about the same object without
any of them being told twice.

### `struct SymbolicFontRefusal`

A real `EditError`'s `Display` is what these two tests are defending
against, and paraphrasing it would have made them defend a paraphrase.
`vector_edit`'s error bound is `Display` and nothing more (its own header
carries why), so a bespoke type carrying the
engine's exact sentence is a faithful stand-in for the value the funnel
really meets — and it keeps the test independent of
`pdfcer_core::edit::EditError`, which is `#[non_exhaustive]` and whose
variants this crate is deliberately not permitted to name.

### `fn a_refused_edit_is_a_sentence_rather_than_a_silence`

The founding defect class of this project, pinned at the one place every
document change passes through. An error arm that writes to `PDFCER_DIAG`
and stops tells an operator who armed Edit ▸ Edit text on a CAD drawing,
placed a caret, typed and committed **nothing at all** — and the engine's
refusal is correct, which is what makes that silence indefensible rather
than merely unhelpful.

# The four properties asserted, and why each would fail invisibly

1. **A decline is recorded.** Break the wiring and nothing errors, no other
   test notices, and the symptom is exactly the state this test exists to
   end — which is why it cannot be left to review.
2. **The document did not move.** The sentence says *"the document is
   unchanged"*, and that has to be true by construction rather than by
   intention: no epoch bump, so no cache invalidation and no undo entry.
3. **The verb's own sentence wins.** Verbs record their own refusals from
   inside the closure, and an unconditional write in the error arm would
   replace a sentence naming a one-click remedy with one naming nothing.
   This is the
   assertion that stops a future "simplification" of `BeforeTheVerb` into a
   bare `record`.
4. **Two presses are two events.** The second commit on the same
   unsupported text has to register; see `decline::BeforeTheVerb`'s
   repeatability section for why the take is what delivers it.

### `fn the_sentence_names_no_cause_and_borrows_none_of_the_engines_words`

Two rules that look like one and are not.

**No cause**, because there is no honest way to obtain one:
`pdfcer_core::edit::EditError` exposes no coarse discriminant a front end
may switch on, matching on its variants would be a second copy of its
taxonomy that drifts and then tells the operator the *wrong* reason, and
parsing its prose is greping a diagnostic that is theirs to reword. The
engine has been asked for the discriminant; until it lands, one
un-categorised sentence is the fallback the request itself specifies.

**No borrowed words**, because `check-ui-strings.sh`'s exclusion 3 says in
as many words that being a `Display` impl *"is not permission to route UI
text through an error type"*. That gate cannot see a `format!("{error}")`
that reaches a label at runtime; this can.

# How the second half is asserted, and why it is not a keyword list

Every word of the engine's prose is checked against every word of the
sentence, and a collision fails — except for a short, explicitly-named set
of ordinary English that any two sentences about the same event will share.
A keyword list would only catch the words whoever wrote it thought of; this
catches **any** leak, including the one that matters most — somebody
appending `format!(": {error}")` to make the message "more helpful".

`refused` is on the allow-list and is the interesting entry: it is the
plain English verb for what happened, not part of the engine's diagnostic
vocabulary, and both sentences are entitled to it.
