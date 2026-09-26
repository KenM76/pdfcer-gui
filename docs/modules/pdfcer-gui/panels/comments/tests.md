# `pdfcer-gui/panels/comments/tests`

## Item notes

### `fn row_by`

Built by hand rather than by `model::collect`, because the subject is a
decision about **one field** and collecting a row would make the test
depend on a document, an annotation and a walk — three things that can
fail for reasons this assertion is not about.

### `fn a_note_with_an_author_keeps_it`

The mistake `pdfcer-core` warned about by name when it shipped
`set_markup_note`: writing all three keys unconditionally *"would
silently strip the author and date on every correction, leaving a review
comment from nobody, dated never, looking exactly like a note somebody
else had mangled."*

`true` here means the action sends **no `/T`**, which is what leaves the
existing one alone.

### `fn a_note_with_no_author_is_ours_to_sign`

Asserting only the preservation case would pass on an implementation
that never writes `/T` at all — every comment anonymous, which is the
same defect wearing the other value.

### `fn the_comments_command_is_reachable_from_the_ribbon`

The check three panels in the old shell shipped without: they had a
body, a rail entry and a diagnostic step, and *"no control an operator
could click"*, so every verification passed while they were unreachable
in a real build.

Two assertions, and both are needed. A command **the manifest
references** is one the ribbon draws a control for; a command **the
registry holds** is one that has a label, a tooltip and an enable
predicate. Either alone is half a control.

`crate::panels::tests::every_panel_is_reachable_from_the_ribbon` sweeps
the same property across every panel; this one names *this* panel in
its failure message, which is what a reader who has just added it
wants to see.

### `fn the_panel_enum_and_this_module_agree`

Two spellings of one id is two things to keep in step, and the failure
when they drift is a panel that opens from the ribbon and draws nothing
in the dock — which looks like a rendering bug and is not.

### `fn the_page_index_travels_zero_based_and_prints_one_based`

The off-by-one that would otherwise be invisible.
[`crate::app::actions::Action::GoToPage`] takes a 0-based index — the
same convention `crate::panels::bookmarks` pins from its own side — and
every string a human reads takes the number one higher. Getting it
backwards produces a panel that navigates one page past every comment,
which looks like a document defect.

Asserted against a real fixture rather than a constructed row, so the
indices are ones the collector actually produced.

### `fn the_delete_control_reaches_the_engine`

# Why this is a test and not a paragraph

Because the paragraph it replaces was **wrong for three weeks** and
nothing could tell. This module's header said *"there is no Delete,
because `Action` has no variant that could carry the intent"* while the
variant, the dispatch arm and the engine verb all existed. Prose cannot
go red.

`RESUME.md` states the rule this is an instance of — *"a sentence about
what the engine cannot do is a dated citation with a shelf life
measured in hours … where the claim can be an assertion, make it one"* —
and it names the day a unit test asserting such a claim went red the
moment the engine shipped, *"which is the behaviour a paragraph cannot
have."*

# What it asserts, in the order the panel does it

1. the fixture carries an addressable annotation — otherwise the rest
   proves nothing;
2. `annotation_deletion_refusal` says the document permits deletion,
   which is the predicate [`delete_control`] gates the button on;
3. `delete_annotation` **succeeds** on it;
4. and the annotation is **gone from the session's own view** — not
   merely that the call returned `Ok`. A verb that reported success and
   changed nothing is the exact failure a return-value check cannot
   see, and this project has been bitten by *"the verb did nothing"*
   twice.

### `fn a_ce_dimension_row_says_ce_dimension_and_still_says_line`

Rule 15 at the point of use. The bracketed `/Line` is not decoration:
the exclusion argument in this module's header turns on ce dimensions
*being* `/Line` annotations, and a heading that hid that would quietly
contradict the argument that put the row in the list.

### `fn a_reply_is_postable_only_when_it_says_something`

# Why this guard exists at all, which is not obvious


⇒ So the decision is this shell's, and it is deliberately the opposite of
the one the note editor makes: `AnnotAction::SetNote` permits an empty note
by name, because an empty sticky is what an operator has just placed and is
about to type into. An empty **reply** is a new annotation added permanently
to somebody's thread that says nothing and that this panel offers no later
way to give words to.

Both directions, and the whitespace case explicitly: a guard that only
stopped `""` would stop only the operator who pressed Post with the cursor
at position zero, and `"   "` renders in every surface exactly as an empty
reply does.

### `fn a_reply_resolves_to_the_comment_at_the_head_of_its_thread`

# The defect this stops, which arrived with the Reply control

`add_reply` places a reply at **its parent's own `/Rect`**, so
`canvas::notepopup::model::notes_on` excludes replies from the notes it
draws windows for — otherwise the newest answer to a comment sits exactly on
top of it and takes every click meant for the comment itself. With that
exclusion in place, a *Go to* that asked for a reply's own window would ask
for a window that is never drawn: the operator presses the button, the page
changes, and nothing opens.

The identity case is the control and it is not a formality — it is the
overwhelmingly common one, and a `thread_root` that walked to the first row
in the list, or returned the last id it saw, would pass a reply-only test
and break every ordinary comment in the document.

### `fn a_malformed_thread_resolves_to_something_real_rather_than_hanging`

§7.3.10 makes a dangling reference not an error and says nothing at all
about a circular one, and `pdfcer-core` models `/IRT` *"unresolved … a
dangling `/IRT` is modelled, not repaired"* in `Annotation::in_reply_to`. So a file that
says `a` replies to `b` and `b` replies to `a` is a file this panel must
survive — and an unbounded upward walk over one hangs **the frame that is
trying to draw**, which is the worst available outcome on a display surface.

The dangling case is asserted beside it, because it terminates for a
different reason — the parent is not in the list at all — and a build that
handled the cycle by bounding the loop while panicking on a missing parent
would pass the first half of this test.

### `fn a_reading_stance_draws_no_reply_editor_even_with_a_reply_draft_open`

# Why this needs its own test beside the count above

`a_reading_stance_offers_no_control_that_writes_to_the_document` drives the
panel with **no draft open**, so it proves that the *Reply* button is
withheld and says nothing whatever about the editor that button opens. The
reply editor is reached down a different path — `note_controls` returns on
the stance check *before* it asks whether a draft is open — and a build that
moved that check one line later would draw a live Post control in Read, with
the count assertion above still green.


# The positive control

Review with the same seeded draft, asserted to draw *something*. Without it,
the Read assertion passes on a build where the draft is dropped before
anything is drawn, on a fixture with no comments, or on a panel that crashed
early — none of which is the property being claimed.

### `fn escape_on_an_edited_note_writes_it`

The whole point of the rule, and the only assertion here whose failure is a
data-loss defect rather than a tidiness one: an operator who typed a
paragraph and pressed the key that every other program treats as *discard*
gets a `set_markup_note`, not silence. A commit by mistake is one `Ctrl+Z`;
a discard by mistake is unrecoverable, because a draft lives in this panel's
own state and never reaches the undo stack.

### `fn escape_on_an_untouched_note_writes_nothing`

`set_markup_note` on text identical to what is already there is a call whose
entire effect is an undo entry, and an operator who opens an editor to read
a long note and presses Escape has changed nothing and must be told nothing.
This is also the assertion a "write it unconditionally" implementation fails
— the cheap way to satisfy the test above.

### `fn escape_over_an_untouched_description_writes_nothing`

The editor seeds from either — §12.5.2's `/Contents` carries both meanings —
so a comparison that only knew about `Note::Text` would read a description
as an empty note and write an "edit" that changed nothing, on every Escape
over a `/Link`.

### `fn escape_on_a_reply_posts_it`

The destination is the one thing in this file that cannot be recovered
afterwards: a `SetNote` raised from a reply draft writes the answer **over
the comment being answered**, and on screen that looks exactly like the
reply having worked.
