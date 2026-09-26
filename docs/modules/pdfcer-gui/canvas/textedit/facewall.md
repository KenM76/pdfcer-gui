# `canvas::textedit::facewall` — one session, two verbs, and the second one refuses

**The experiment that decides whose defect O141's last step is**, kept in the
tree rather than run once and reported, because the answer it gives is a
statement about `pdfcer-core` and this project has learnt that a paragraph
about what the engine cannot do has a shelf life measured in hours
(`RESUME.md`, *"Where the claim can be an assertion, make it one"*).

## The question

`tools/ui-verify`'s `a_refused_character_offers_a_face_that_can_type_it`
drives the whole of O141 and fails at the last step. The operator types a
character the run's subset font cannot carry; the refusal names it; the
Properties panel offers faces that can; the offer is taken and **reaches the
document** (`text-style-applied … change=face applied=1 runs=1`); and then
the same character, committed again into the same run, is refused a second
time with a *different* sentence:

```text
this run cannot be edited in the first cut:
  the run's font resource is unresolvable in the target stream's resources
```

The identical pair **succeeds on disk**, measured with `pdfcer.exe` 0.40.0 on
this very fixture: `format-text --set-font Helvetica` then `edit-text
--replace ABCq` lands, and `extract-text` reads `ABCq` back. The only
difference between the working case and the broken one is **one process
versus two** — the command line reopens the file between the verbs, the shell
holds one live [`EditSession`] across both. So the defect is in session
state, and the two candidates are:

* **the shell**, handing the second `edit_text` an operand pinned before the
  restyle rewrote the content stream; or
* **the engine**, resolving the run's `Tf` name against a document revision
  that predates the resource the restyle just created.

## The experiment, and why it is decisive

[`the_engine_cannot_type_into_a_face_it_just_swapped_in`] makes **one**
[`EditSession`], calls `format_text` and then `edit_text` on it, and locates
both by `find` text alone — no pin, no span, no
[`super::pin::Pinned`], nothing this shell computes. Every input the shell
could have got wrong is absent from it. If it still refuses, no arrangement
of shell operands can make the route work and the defect is the engine's;
if it succeeds, the engine is fine and the shell is handing it something
stale.

[`two_sessions_do_what_one_session_will_not`] is the control that makes the
first one evidence. It runs the identical pair of calls with a **save and
reopen** between them — which is exactly what the command line does — and
asserts the second `edit_text` succeeds and the character is in the extracted
text afterwards. Without it, a failure above could be a fixture that cannot
take the edit at all, and the report would name the wrong subject.


**The engine's.** One session refuses; two sessions succeed; the fixture,
the request, the face and the character are the same in both.

The mechanism, read afterwards and stated here because the assertion alone
does not explain it: [`EditSession::format_text`] allocates the new `/Font`
object and puts it in the same undo command, through the engine's own
`font_resource_writes` — which correctly binds against `self.graph()`, the
overlay, *"not `self.base`"*, and says so. [`EditSession::edit_text`] then
plans with `plan_edit(&self.base, …)`, and `resolve_font_dict` dereferences
the `/Font` name through **that** document, `doc.resolve(o)` against the
`DocumentView` it was handed. The
name the restyle wrote into the stream names an object that exists only in
the overlay, so the deref against the base answers `None`, and `None` is
reported as *"unresolvable in the target stream's resources"*. The stream
read is the session's (`current_page_content`, which is what makes
sequential edits accumulate); only the object graph the names are resolved
through is the base's.

⇒ **Any session-path verb that creates an indirect object and then resolves
a name that points at it will do this**, which is why it is filed as a class
rather than as "the font swap is broken":
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\request_edit_text_resolves_font_names_against_the_base_revision_so_a_face_swapped_in_this_session_is_unresolvable.md`.

## What the shell does about it, and what it deliberately does NOT do

**It does not work around it.** The only in-process route that works is to
throw the `EditSession` away and rebuild it from the saved bytes, and that
silently destroys the operator's undo history — a document-level act, taken
without asking, to paper over a limit the engine itself prescribes the remedy
for (`EditSession::reflow_block`'s own documentation says *"Save and reopen
to reflow after an in-session edit of the same page"* about the same class of
staleness).

So `panels::properties::refusedchar` did the two honest things instead: it
**re-applies the operator's edit itself** the moment the face lands, so the
route is one gesture and lands outright on any document carrying a second
usable face; and when the retype is refused it says so
(`text::panels::face::refused_char_blocked`).



[`the_engine_types_into_a_face_it_just_swapped_in`] is the same test with its
assertion inverted. It went red on the first run after the pin moved to
v0.41.0, from its own `expect_err` message, which is the entire design
working: this project's standing failure mode is a sentence recording a
limitation that stopped being true and nothing noticing, and the fix for it
is to make the claim executable. `Pass 257.0` (`5e95805`) is the answer —
every text-edit planner and helper takes `&DocumentView<'_>`, every
`EditSession` verb passes `self.view()`, and with no `&Document →
&DocumentView` coercion the class is now a **compile error**.


It said: (1) delete the test; (2) delete
`text::panels::face::refused_char_blocked` and the `retried` arm of
`refusedchar::section` that shows it, *"the state becomes unreachable,
because the retype will land and the block will retire on the next frame"*;
(3) tighten the driven check to *"the character went in"*.

**Steps 1 and 3 were right and are done. Step 2 was wrong**, and the way it
was wrong is worth more than the instruction was:

> The `retried` arm is **not** reached by recognising this cause. It is
> reached by arithmetic — the retype was raised and `doc.edit_epoch` did not
> move — and that condition is agnostic about **why**. Other whys exist: an
> offered face that does not cover the character after all, a run whose
> operators a pinned request cannot span, a refusal nothing here has met.
> Deleting the arm converts every one of them into **silence**, which is a
> defect already on this project's own list.

⇒ **A state and its explanation have different lifetimes, and a plan
written from one cause will happily delete the handling for all of them.**
The explanation expired; the state did not. So the cause was struck from the
sentence and the state kept its voice. `refused_char_blocked`'s own doc
comment carries the full reasoning and its test now asserts that the sentence
names **no** cause — including, by name, the two the old wording used, so
that a later session improving the prose cannot reinstate a falsehood from
this file's git history.

Step 1 became an inversion rather than a deletion for the same family of
reason: the test still buys something. It holds the fix down in **both**
request shapes, and the `find` shape is the one whose regression would come
back quietly, as a "no match" that reads like a bad search.
