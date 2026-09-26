# `app::save` — writing a copy of the open document to a file the operator
names

The body of `file.save_copy`, and **the route by which anything an operator
authored in this shell leaves the process**. A command can be registered,
drawn on the File tab, drawn on the quick-access toolbar, bound to `Ctrl+S`
and print "(Ctrl+S)" in its own tooltip while having no dispatch arm at all
— it then traces `command-unimplemented` and does nothing, and every feature
this project has shipped (dimensions, markup, text marks, form fills, page
operations, a newly created document) is unwritable to disk. That is
`DEFECTS.md` D1's shape with the most consequential verb in an editor behind
it, which is why the dispatch arm is part of this module's contract rather
than an afterthought.

## 1. The save mode is **incremental**, and it was decided by a shipped
   promise rather than by this module

`crate::text::commands::file_save_copy`'s tooltip has said, on an
operator-visible surface, since the day the command was registered:

> *"…the edits are appended as an update so the previous version stays
> intact inside the file."*

That sentence is a description of `EditSession::to_incremental_bytes` and of
nothing else. §7.5.6 incremental update: the original bytes are kept
verbatim and a new revision is appended after them, so the file carries both
and the previous one can be recovered.

[`EditSession::to_full_bytes`] would satisfy every test that asks whether a
file was written and whether the edit is in it, and it would break the
promise in two ways an operator cannot see until it is too late: it
**destroys every existing digital signature** (§12.8.1) and it discards the
previous revision the sentence guarantees. So the choice is not this
module's to re-open. If a future change finds incremental genuinely
impossible for some input, the honest response is to **refuse and say so**,
not to fall back to a full rewrite — the engine already refuses a full
rewrite by name (`WriteError::HybridFullRewrite`) and points at incremental
as the supported path, which is the same posture from the other side.

**The exact width of that refusal, which is narrower than it sounds.**
`save_full` refuses only a hybrid whose `/XRefStm` **does not parse** — the
file says it hides objects and pdfcer cannot tell which. Ordinary hybrid
files, which is most of what Microsoft Office exports, rewrite fine. The
wide refusal — any hybrid at all — stands in `save_full_encrypted` and
`save_full_decrypted`, so **adding or removing encryption refuses every
hybrid**. The posture being cited here is the same either way.

### 1.1 …EXCEPT while a redaction is staged

There is exactly one state in which this module writes a **single-revision
full rewrite** instead, and it is not a fallback: a staged redaction.
All three save verbs go through
[`write_copy`], and [`write_copy`] asks `EditSession::has_pending_redaction()`
before it asks anything else.

Three facts about that, each of which somebody will otherwise rediscover:

1. **It is not optional.** While a redaction is staged the engine refuses
   `to_incremental_bytes` **and** `to_full_bytes` by name
   (`WriteError::RedactionPending`, raised from both writers), because the
   un-redacted content is still live in the session.
   A build that did not route would not leak — it would simply stop being
   able to save at all, loudly, on every verb.
2. **It suspends §1's promise, so §1's promise gets a sentence.** The
   previous revision is *deliberately* not carried into the output — that is
   the whole of engine rule R35 — and the operator has been taught by
   `file.save_copy`'s own tooltip to expect the opposite.
   [`crate::text::redact::saved_applying_redaction`] is where he is told.
3. **The staging survives the save.** `save_applying_redaction` takes
   `&self`; it neither mutates the session nor clears the flag. So the next
   save applies the redaction again, the ordinary modes stay refused, and
   the document on screen still shows the content — which is stated in the
   same sentence, because *"I saved it, so it is done"* is exactly the
   assumption this must not let stand.

## 2. Why this needs none of [`crate::app::actions`]' four-step protocol

Every other document verb in this shell goes through `vector_edit`: cancel
the render worker, mutate through `Arc::get_mut`, bump `edit_epoch`, drop
the cached texture. **A save does none of those and must not.**

The reason is one word in the engine's signature:

```text
pub fn to_incremental_bytes(&self, options: &SaveOptions)
                            ^^^^^
```

`&self`, not `&mut self`. The dirty set is computed at save time from the
current state, so writing is a **read** of the session. That has three
consequences worth stating, because each is a step somebody would otherwise
add out of symmetry:

| step `vector_edit` takes | why a save must not |
|---|---|
| `RenderWorker::cancel_and_wait` | it exists only to make `Arc::get_mut` succeed. A save never calls it, so the render worker may keep its clone and a save during a raster costs the operator nothing |
| `Arc::get_mut` | there is nothing to mutate; `&*doc.session` is enough |
| bump `edit_epoch` | see §3 — it would throw away the decomposition, the page-text cache and any live disclosure to record that **nothing changed** |
| `page_texture = None` | the page on screen is still correct; a re-raster would be work with no cause |

## 3. What happens to the edit epoch and the dirty state: **nothing**,
   in both directions

This is the part that is easy to get wrong in the tidy-looking direction, so
it is written down as four separate claims.

### 3.1 `edit_epoch` is not bumped

`OpenDoc::edit_epoch` means *"which revision is on the screen"*. It is the
staleness key for the canvas selection, the object decomposition, the font
inventory, the per-page text cache, the form-fill disclosure and the edit
disclosure. A save changes none of those — the document in memory after a
save is byte-for-byte the document that was there before it — so bumping the
epoch would dissolve the operator's selection, discard several caches, and
silently retire a rule-4 disclosure sentence they may not have read yet, all
to record an event that changed nothing they can see.

### 3.2 `edit_epoch` is not **reset** either, and this one has teeth

The tempting move is `doc.edit_epoch = 0` — "the edits are saved now". It
would be wrong twice.

It is wrong in principle: the edits are saved *somewhere else*. The document
open in front of the operator is still exactly as unsaved as it was, at its
own path.

And it is wrong concretely. `edit_epoch` is only half of a **pair**: every
consumer that asks *"is there unsaved work?"* asks it against `saved_epoch`
— `save::has_unsaved_edits`, and `actions::acrobat`'s
`edit_epoch.saturating_sub(saved_epoch)`, which is the count the unsaved
dialog reads out. Zeroing one side puts `edit_epoch` **below** `saved_epoch`,
the subtraction saturates to `0`, and the unsaved dialog offers *"you have 0
unsaved changes"* over work that is genuinely unsaved. The warning still
appears — `has_unsaved_edits` asks `!=`, not `>` — so what the operator gets
is a prompt that contradicts itself, which is worse than either answer.

### 3.3 `save_pending()` still answers `false`

It asks *"is a save **in flight**"*, and gates New, Open and Close on the
answer. [`save_copy`] is synchronous: it is entered and finished inside one
`PdfcerApp::apply` call and no frame is drawn while it is part-way through,
so there is no moment at which the predicate could be true. See
`crate::app::files`' header, which carries the corrected rule, and
`PdfcerApp::save_pending`, which carries it beside the code.

### 3.4 `path` and `origin` do not move: Save a **copy** is not Save **as**

A created document saved to `D:\jobs\sheet.pdf` is still called
`Untitled 1.pdf` afterwards, still has `Origin::Created`, still gets no
Recent row, and still stores no per-document page-display or guide
preferences. That is Inkscape's *Save a Copy* exactly — the one reference
application that has this verb — and it is the difference between it and
*Save As*, which is a different command that does not exist here.

`OpenDoc::origin`'s own doc comment anticipated this and said *"a created
document that gains a file gains it through a save"*. It is worth being
precise about which save: **this is not that one.** A `file.save_as` would
write `path` and `origin`; this deliberately does not, and adding it here
would rename the operator's open document out from under them because they
asked for a copy.

## 4. Where the picker runs, and why the command raises an [`Action`]

`crate::app::files::pick_save_path` documents a **frame-timing
requirement**: it opens a native modal, and a native modal opened from
inside an `egui` layout closure blocks the frame it is being drawn in,
leaving a half-painted window behind a dialog. `dialogs::ocr` honours it
with a `save_requested` flag consumed after `Window::show` returns.

`file.save_copy` honours it by **raising
`crate::app::actions::Action::SaveCopy`** and doing the work in the apply
phase, which is step 3 of the frame — after every panel, the canvas, the
docks, the find bar and the dialogs have closed. That is the strongest
position available, and it is *also* the actions-not-mutations invariant
(`PROJECT_PLAN.md` §3) satisfied by the same line, which is why there was no
trade to make.

It is deliberately **not** `file.open`'s shape. That arm calls the picker
*during dispatch* and pushes the answer, on the argument that the picked
path is an operand which cannot be re-derived after the frame. The argument
is sound for Open and does not apply here — a save has no operand to carry;
the suggestion is derived from the document — and the timing is the other
way round: **dispatch is not always outside a layout closure.**
`PdfcerApp::central` dispatches the canvas's context-menu tokens from inside
`egui::CentralPanel::show`, so an arm that opens a modal is one context-menu
entry away from doing it mid-layout. See the report accompanying this work.

## 5. What an operator sees

**On success: the dialog they drove, and the file where they put it.** No
sentence is added. A save-a-copy is the one operation in this shell whose
whole product is visible in the operating system's own file browser, at a
path the operator typed a moment earlier; a status line saying so would
narrate what they just did.

**On failure: a worded decline**, through `crate::app::status::decline` —
the surface built for *"this did not happen"*. Silence here is the exact
failure this project was founded on: the operator names a path, presses
Save, and no file appears. The engine's own reason goes to the trace, not to
the bar; `check-ui-strings.sh`'s exclusion 3 says in as many words that a
`Display` impl "is not permission to route UI text through an error type".

**On cancel: nothing at all**, which matches `crate::app::files::raise`'s
ruling for a dismissed Open — the operator changed their mind, and that is a
complete and correct outcome that must not put a line anywhere.

## 6. What a save says about a DIGITAL SIGNATURE

Saying nothing is the failure this section exists to prevent. A structural
edit followed by `Ctrl+S` writes a revision over a signed document, and a
shell that mentions it on no surface — before or after, on any of the three
write paths this module owns — has invalidated a signature silently.
`pdfcer-core` exposes `EditSession::signature_impact_of_save` and
`EditSession::changes_structure` specifically so a front end can answer the
question, so the answer is there for the asking and the only way to get it
wrong is not to ask.

The whole design lives in [`crate::dialogs::signature`]; what belongs in
this header is the half this module performs:

| when | who | what |
|---|---|---|
| **before** an invalidating save | `crate::app::actions::apply`, through `DialogsState::ask_signature` | a window, which the operator may cancel — so [`save_in_place`] and [`save_copy`] are simply **not reached** |
| **after** any successful write | [`signature_note`], here | one sentence on the disclosure row, for a signed document only |

Three properties of that split are worth stating where the writes are:

1. **An unsigned document is untouched by all of it.** The engine's own
   instruction for `SignatureImpact::None` is *"a front end should add no
   friction at all"*, and that is the case for nearly every document this
   operator opens.
2. **The note is recorded on the write paths rather than in the action
   arms**, so `crate::app::lifecycle::resume_after_unsaved` — which calls
   [`save_copy`] directly, from inside an already-answered question — gets
   it without a fourth call site having to remember to.
3. **The compacted path is not routed through any of it.** It is a full
   rewrite, `crate::dialogs::compact` already discloses the loss before its
   picker opens, in stronger words than this module is entitled to, and
   those words are correct there and only there. See
   `crate::dialogs::signature`'s §4 and §5 — including the trap that
   `SignatureImpact::documentation_basis` cannot see the `SaveMode` and so
   must not be asked about a rewrite.

## Item notes

### `fn signature_note`

`None` for the overwhelmingly common case — a document with no signature —
and that is the engine's instruction rather than an optimisation:
`SignatureImpact::None` means *"Nothing to say, and a front end should add
no friction at all."* A save of an unsigned drawing costs one census walk,
which is bounded (`signature::MAX_FIELD_TREE_NODES`), and produces no
string, no allocation past the census and no row on the bar.

# It is computed BEFORE the bytes are written, and the reason is the
engine's contract rather than caution

`EditSession::signature_impact_of_save`'s own documentation: *"A front end
asks this **immediately before Save**, not at edit time: per §11.1 the
dirty set is a diff computed at save time."* Asking after the write would
in fact return the same answer today — a save changes nothing about the
session, which is §3 of this module's header and is asserted by
`saving_a_copy_changes_nothing_about_the_open_document` — but it would be
asking a question the engine documented as a *pre*-save question, and the
day that stops being harmless is the day a save starts touching the
session.

# The two sentences, and why they are not one

| [`Disclosure`] | sentence | what it says |
|---|---|---|
| `Silent` | none | there is no signature |
| `NoteAfterSaving` | [`crate::text::signature::preserved_note`] | the bytes each signature covers are unchanged — **paired with** the fact that this is not the same as still being valid |
| `WarnBeforeSaving(_)` | [`crate::text::signature::invalidated_note`] | pdfcer reports this save as invalidating |

The third row is a **receipt for something the operator was asked about**
on the two routes that ask — and on `crate::app::lifecycle`'s
resume-after-unsaved route it is the whole of what they are told, because
that route deliberately does not stack a second window on an
already-answered question. `crate::dialogs::signature`'s §7 carries the
argument. Repeating it here for the routes that did warn is cheap and is
the right trade: a confirmation dismissed quickly is a confirmation not
read, and the bar is where every other consequence of a write is recorded.

### `fn redaction_receipt`

§1.1 item 2 and item 3, in one place so that all three save verbs get the
identical wording — and so that a fourth verb added later gets it by calling
one function rather than by remembering three facts.

The trace line is separate from `save-copy`'s on purpose. That line's fields
(`appended=`, `verbatim=`, `identical=`) are properties of an incremental
update and every one of them would be a fabrication here; this line carries
what a full-rewrite removal actually did. A reader of a trace must be able to
tell the two events apart, and the ink-trail rule is that they are told
apart by fields, not by hoping.

### `fn redaction_refusal_note`

`record_save_failure` puts *"the file could not be written"* on the bar,
which is right for a full disk and wrong here: nothing is wrong with the
disk, the document, or the operator's folder. What is wrong is that a
removal is armed over marks that no longer exist, and the remedy is a
control in a dialog he would have no reason to open.

It is added **beside** the failure rather than instead of it, on
`crate::text::redact::save_kept_pending_marks`'s standing reason: the save
genuinely did not happen, and replacing that fact with an explanation would
leave an operator unsure whether a file appeared.

Silent for every other [`SaveError`]. A missing folder and a broken
provenance span have their own sentence already and do not want a second.

### `fn page_tree_refusal_note`

[`redaction_refusal_note`]'s shape exactly, for its reasons, one variant
along: `record_save_failure` puts *"the copy was not written — check that
the folder exists and can be written to"* on the bar, which is right for a
full disk and wrong here. Nothing is wrong with the disk, the folder, or
anything the operator did. What is wrong is that the document no longer
agrees with itself about how many pages it has, and a file written from it
would open elsewhere with blank pages in it.

Added **beside** the failure rather than instead of it, on the same
standing reason [`redaction_refusal_note`] gives: the save genuinely did not
happen, and replacing that fact with an explanation would leave an operator
unsure whether a file appeared.

A **separate function** rather than a second arm inside
[`redaction_refusal_note`], and the reason is territorial rather than
aesthetic: that function belongs to the deferred-redaction work, which is in
flight in another track, and a shared `match` is a merge conflict in a file
two tracks are editing. The two are mutually exclusive by construction — one
[`SaveError`] value reaches both — so calling both costs nothing and neither
can overwrite the other's note.

**Which of the two sentences** is decided from structured data, never
from a message: [`crate::pagetree::Audit::root_disagreement`]. A root that
disagrees has an exact symptom the operator can verify (*n* blank pages at
the end); an interior-only disagreement does not, and promising one would be
the sneaky half of rule 4. See [`crate::text::pagetree`]'s header.

Silent for every other [`SaveError`], exactly as its sibling is.

### `fn write_copy`

The one place bytes leave this shell for a document the operator authored,
and it is deliberately free of any dialog so that a test can drive it — see
[`save_copy`]'s note on why the surrounding function cannot be.

# The options, field by field, chosen rather than defaulted-into

`SaveOptions::default()`, and each of its three fields is a decision:

* **`producer: ProducerPolicy::Set`** — the default, and **ignored on this
  path by construction**. `save_incremental` never rewrites `/Info` at all,
  *"because doing so would mean appending a revision to a document the
  operator did not change"*. So the value cannot affect a byte of the output
  here, and the reason it is left at the default rather than set to
  `Preserve` is that `Preserve` would advertise a policy that is not being
  applied — a reader would take it as evidence that this path suppresses a
  producer stamp, when what actually suppresses it is the writer.
* **`xref_entry_eol`** and **`trailing_eol`** — §7.5.4's and §7.2.3's
  permitted end-of-line forms, both left at the values pdfcer has always
  emitted. They apply to **newly written** cross-reference entries only, so
  an incremental save changes nothing about the base revision's bytes
  whichever way they are set. `SaveOptions::identity()`'s own documentation
  draws the line this follows: *"an operator-facing save path applies the
  persisted values explicitly; this one does not"* — and this shell has no
  settings surface, so there is nothing persisted to apply. The day the
  salvaged settings dialog lands and offers them, this is the call site that
  reads it.

# Why not `SaveOptions::identity()`

Because it names a **byte-comparison posture** for a harness, not an
operator-facing save. Its only difference from the default on this path is
the producer policy, which is ignored here — so it would change nothing and
would tell the next reader that this path is trying to be byte-identical to
its input, which is false the moment the operator has edited anything.

# Errors

[`SaveError`] — the engine refused to serialize, the redaction could not be
performed, the proof found removed text still present, or the file system
refused the write. They are kept apart because their remedies are: one is a
document pdfcer cannot express as an update, one is a staging the operator
has to cancel, one is a pdfcer defect, and one is a folder that does not
exist or cannot be written to.

### `fn suggested_path`

Pure, so every rule in it is asserted headlessly. Two cases, and the
difference is [`OpenDoc::stored_under`] — the one predicate that separates a
document with a file from one that only has a name:

| the document | suggestion | why |
|---|---|---|
| opened from `D:\jobs\sheet.pdf` | `D:\jobs\sheet-copy.pdf` | beside the original, where the operator will look for it, and **never the original itself** |
| created by `file.new`, called `Untitled 1.pdf` | `Untitled 1.pdf`, with no directory | there is no original to avoid overwriting, and no folder it came from; the OS picker supplies its own starting directory and the operator has a name to accept |

# Why the suffix, and why not for a created document

`crate::text::files::save_copy_suffix` carries the copy argument and the
reference-application head-count. The mechanical half is here: the promise
on `file.save_copy`'s tooltip is that *"the original is never overwritten
unless you pick it"*, and a **default that is the original** would make that
promise depend on the operator reading a pre-filled field before pressing
Enter. `crate::dialogs::ocr::suggested_path` enforces the identical rule with
`-recognised` and asserts it the identical way.

A created document has no original, so appending `-copy` would answer a
question nobody asked: it would offer `Untitled 1-copy.pdf` for a document
that has never been saved at all, which reads as though a first copy already
exists somewhere.

# The extension

Forced to `.pdf`, exactly as the OCR suggestion is, and for the same reason:
the bytes are a PDF whatever the source was called, and a copy of `SHEET.PDF`
landing as `SHEET-copy.PDF` would be correct and would be one more way for a
tool downstream to disagree about case. A created document's name already
ends in `.pdf` (`crate::text::files::untitled`), so this changes nothing for
it — which is why that function's own docs explain that the suffix is on the
name precisely so a save suggestion is not extensionless.

### `fn save_as`

The whole of the `Action::SaveCopy` arm. Takes `&OpenDoc` rather than `&mut`
— see §2: the engine's write verb takes `&self`, so nothing about the open
document changes and the type says so.

The three answers the picker can give are treated exactly as
`crate::app::files::raise` treats them for Open, and for the same reasons:
a path is the ordinary case; a cancel is a complete outcome and is silent;
and *this build cannot ask* is a **build** limitation rather than an
operator choice and gets a trace naming the gap, because a reader of a trace
from a machine they cannot see most needs that told apart from "the click
never arrived".

Not reachable from a unit test, and that is [`crate::app::files`]' rule 3
applied to the save side: with `PDFCER_DIAG_SAVE_PATH` unset this opens a
**real modal dialog** and blocks until a human dismisses it, so a
`cargo test` that applied `Action::SaveCopy` would hang the suite behind an
invisible window. Everything below the picker is therefore reachable with
the answer supplied directly — [`write_copy`] and [`suggested_path`] are
both pure of any dialog and both are tested — and the join is proven by
`tools/ui-verify`'s `save_copy_round_trip`, which answers the dialog through
the seam and then re-opens the file that came out.

**Returns whether a file was actually written.**

The caller that needs the answer is `crate::dialogs::unsaved`'s *Save a
copy…* button, which **only proceeds with the close or open it is standing
in front of if the save succeeded.**

The three false cases are not the same thing and it is worth saying so,
because a future hand will be tempted to distinguish them:

* the operator **cancelled the picker** — they changed their mind
  mid-transaction, and the least surprising reading is *"leave my document
  alone"*;
* the picker is **unavailable** in this build;
* the write **failed** — already reported on both channels by
  [`write_and_report`].

All three answer `false` and all three mean the same thing to that caller:
*do not destroy the document*. Returning a richer type so it could tell
them apart would invite a branch that proceeded on one of them, and there
is no member of that set it would be safe to proceed on.
**Save As** — write the document somewhere new, and *keep editing THAT file*.

# Why this is a different command from [`save_copy`], and not a flag

Operator, `OPERATOR_REQUESTS.md` O95:

> *"we need a Save As option so that we are then making edits in the save as
> file instead of the original just like other programs have it."*

**The second half is the whole request.** `save_copy` already writes the
bytes wherever he points it — what it does not do is *move the document*.
The session stays bound to the original, so the next `Ctrl+S` goes straight
back to the file he was trying to leave, which is the opposite of what he
asked for and is a way to overwrite something by doing nothing wrong.

So the two are different **acts**, and every editor he uses has both:

| | writes | afterwards you are editing |
|---|---|---|
| **Save a copy** | a snapshot, somewhere else | **the original** |
| **Save As** | the document, somewhere else | **the new file** |

Keeping both is deliberate. *Save a copy* is the right verb for "send
this to somebody" and collapsing it into Save As would take that away; a
single command with a checkbox would make the destructive difference a
setting nobody reads.

# What this function does NOT do, and why the caller does it

It does not touch [`OpenDoc`]. It picks a path, writes there, and reports
**where it wrote**. The rebinding — `doc.path`, `doc.saved_epoch`, the tab
label, the window title, the recent list — happens in the caller, which
holds `&mut` and can see all of them.

That split is not tidiness. Rebinding is the dangerous half: a document
whose path moved while its bytes did not is a document whose next `Ctrl+S`
writes the wrong file. Keeping the write pure and the rebinding in one
visible place means there is exactly one statement to read to know when the
binding moves.

# Returns

`Some(path)` when the bytes reached that path, `None` when the operator
cancelled, the picker is unavailable, or the write failed — the same three
outcomes [`save_copy`] flattens to `false`, and flattened here for the same
reason: **there is no member of that set on which it would be safe to
rebind the document.**

# The undo stack survives, and that is a decision

Nothing is closed and nothing is reopened, so the session, its history and
the operator's selection all continue. That is what every other editor does
and it is what he would expect: Save As is a save, not a round trip. The
alternative — write, close, reopen the new file — would silently discard
every undo step, which is a data loss with no warning attached to it.

### `fn has_a_file`

The question `file.save` asks before deciding between saving in place and
opening the picker, and it is answered by **asking the file system**, not by
inspecting the path.

# Why "does this file exist" and not "is this path absolute" or a flag

A blank document created in this shell is given a path like `Untitled 3.pdf`
— a name with no directory, and no file anywhere. A document opened from
disk has a path that names a file that is there. Those are the two cases,
and *"is there a file at this path"* separates them exactly.

The alternatives are worse in specific ways. A `created_here: bool` flag is
a second source of truth that has to be maintained through save-a-copy,
re-open and the document-tab machinery, and the failure mode when it drifts
is **writing over the wrong file**. "Is the path absolute" answers a
different question and would treat a relative path to a real file as
unsaved.

# The race, acknowledged rather than defended against

A file can be deleted between this check and the write. If that happens the
rename in [`save_in_place`] creates the file, which is the same thing every
other editor does and is the harmless direction: the operator gets their
document back where they expect it. The dangerous direction - writing over
something they did not choose - cannot be reached from here, because a path
that names no file goes to the picker.

### `fn has_unsaved_edits`

`OPERATOR_REQUESTS.md` row **O65**, the operator:

> *"I can't edit it unless I save the document first, at which point it
> closes the document after saving."*

# What actually happened, because Save does NOT close the document

A successful in-place save records `doc.saved_epoch = doc.edit_epoch`, and
until today **nothing in production read that number**. It was written
once, traced once, and asserted in a single test. Every surface that asked
*"does this document have unsaved edits?"* asked a different question that
a save cannot answer:

| surface | asked | why a save could not clear it |
|---|---|---|
| `dialogs::unsaved::ask_for` | `edit_epoch == 0` | *"has anything EVER been edited"* — permanently true after the first edit |
| the document tab strip | `session.is_modified()` | the engine's *"differs from the BASE revision"*, and an incremental save takes `&self` so the base never moves |
| the close arm | `session.is_modified()` | same |

So after a perfectly good `Ctrl+S` the shell still believed the file was
dirty. The tab kept its unsaved marker, and the very next Close raised the
unsaved-edits question — whose only save button is *"Save a copy…"*, which
opens a picker and, on success, proceeds with the pending intent. From the
operator's chair: press a Save button, get asked for a filename, and watch
the document close. Exactly what he reported, arrived at without Save ever
closing anything.

Driven, not inferred: with `PDFCER_DIAG_INVOKE="mode.edit,pages.rotate_right,file.save,file.close"`
the shipped build traces `save-in-place outcome=ok` → `save-epoch-recorded
epoch=1` → `diag-invoke id=file.close` → `unsaved-asked`, and no `close
slot=` line anywhere.

# Both halves are load-bearing

- **`session.is_modified()`** is the engine's precise *"differs from the
  base revision"*, and it is what makes edit-then-undo come out **clean**.
  A pure epoch comparison would call an undone edit dirty, because an undo
  bumps the epoch like everything else.
- **`edit_epoch != saved_epoch`** is the only term that can see an
  in-place save, for the reason above: `to_incremental_bytes` takes
  `&self`, so the session's own answer cannot change when bytes are
  written.

| state | `is_modified` | epochs differ | answer |
|---|---|---|---|
| opened, untouched | no | no | clean |
| edited | yes | yes | **dirty** |
| edited, then saved | yes | no | clean |
| edited, saved, edited again | yes | yes | **dirty** |
| edited, then undone | no | yes | clean |

# What this must NOT be confused with

[`save_pending`](crate::app::PdfcerApp::save_pending) asks *"is a save in
flight"*, which is a different question with a different consumer, and
`dialogs::unsaved`'s own header explicitly forbids gating Open / New /
Close on dirtiness through it.

And `saved_epoch` must never be reset to make an answer come out right:
`edit_epoch` is the cache key for the decomposition, the page text, the
texture and every live rule-4 disclosure. Two numbers, one question each.
# The third term, and the silent loss it closes

**`session.has_pending_redaction()` — and it must be the *pending* verb,
never `has_applied_redaction()`.** [`crate::redact::stage_into_session`]
leaves base, overlay and undo untouched and sets one flag, so
`has_applied_redaction()` is **`false` for the life of every session this
shell creates**: a term that can never be true, which is worse than an
absent one because it reads as a guard.

**What the term is for at all.** A redaction that reaches the session
without passing through the edit stacks leaves `is_modified()` answering
**`false`** — correctly, on its own terms, because the session does not
differ from its base. With two terms this predicate then answers **clean**
on a document whose most consequential edit has not been written: no tab
marker, no question on Close, no question on Quit. Arm a removal, close the
document, and it is gone with nothing asked.

**And the hole it closes is a real, reachable state**, not a
theoretical one: open a drawing that already carries `/Redact` marks from an
earlier session, press *Review & apply*, choose *this document*. Nothing is
edited — the marks were already in the file — so `is_modified()` is `false`;
the funnel bumps `edit_epoch`, so the epochs differ. With two terms that is
**clean**, and the operator closes a document with an armed removal on it
and is asked nothing.

`session.has_pending_redaction()` is the term that sees it. It is an OR
beside `is_modified()` rather than a replacement for it; the
`edit_epoch != saved_epoch` term is what turns it off again once the
redaction has actually been written.

**It is the better behaved of the two terms in the one way that
matters.** `has_applied_redaction()` is permanently sticky — once true,
true for the life of the session — so `redact → save → edit → undo` answers
**dirty** on a document that matches its file, costing a spurious prompt
that can never be cleared. `has_pending_redaction()` is turned off
by [`crate::redact::cancel_staged_redaction`], so a document whose staging
the operator called off goes genuinely clean again.

⚠ **It is not a save gate**, and the distinction is an easy one to lose.
Saving while a redaction is staged is not merely permitted, it is
the *only* way the redaction ever happens — [`write_copy`] routes it through
`save_applying_redaction`. This predicate asks whether there is something to
save, never whether saving is permitted.

### `fn save_in_place`

The operator:

> *"can I please have a save button like every other program in existence
> has? We're on week two of this and just have a save as button."*

There is no defence for the shape he is describing: `Ctrl+S` bound to
[`save_copy`], which asks where to put it every single time, with
overwrite-in-place written down as *"an operator scope decision"* and then
nobody's problem. It is the same failure as an unbound `Ctrl+P` or a text
caret with no index: **the basics have to be audited as basics**, because a
suite that asks *"does the thing I built work?"* never asks *"does the thing
everyone expects exist?"*.

# It writes to a TEMPORARY FILE and renames, and that is not ceremony

`std::fs::write` truncates the target and then streams into it. Everything
between those two acts is a window in which **the operator's only copy of
the document is a partial file** - and the payload here is the whole PDF,
which for a CAD sheet is megabytes and takes real time. A crash, a full
disk, or a sync client holding a lock in that window destroys the original
and leaves nothing to fall back to.

Save-as does not have this problem, because its target is a file that did
not exist. Save-in-place is the one verb in this application that can
destroy the operator's work, so:

1. write the new bytes to `<name>.pdfcer-tmp` beside the target,
2. `fs::rename` it over the target — which either happens or does not,
3. on any failure, remove the temporary and leave the original untouched.

This is the same lesson as the portable-build packager, which destroyed the
fallback build **twice in one day** by clearing a directory before filling
it, and whose message asserted nothing had been replaced both times. Written
up in `D:/dev/rag/rust/`. The rule that came out of it applies here without
modification: *never destroy the current good state until the replacement is
fully materialised somewhere else on the same volume.*

The temporary sits **beside the target**, deliberately, not in the system
temp directory: a rename across volumes is a copy, and a copy has the unsafe
window back again.

# Why it returns the same `bool` as [`save_copy`]

So `crate::dialogs::unsaved`'s buttons can treat the two identically: the
question that dialog is asking is *may I now destroy this document*, and
only a successful write answers yes. See [`save_copy`]'s own note on why
distinguishing the failure modes there would invite a caller to proceed on
one of them.

# What it does NOT do

Fall back to Save-as. A document opened from a path always has one, and a
blank document created in this shell is given a path when it is first saved
— the dispatcher routes that case to [`save_copy`] before reaching here,
because *"where does this go?"* is a question only the operator can answer
and answering it silently is how a file lands in a folder nobody expected.

### `fn compacted`

`OPERATOR_REQUESTS.md` **O48**, and the counterpart to [`save_copy`] one
question along: that one asks *where*, this one has already asked *whether*.

# Why this does not serialise

`crate::dialogs::compact` did, before it opened, and its headline number is a
measurement of the result rather than an estimate. Serialising again here
would put a second computation between the number the operator accepted and
the file they receive — see `Action::SaveCompacted`, which carries the bytes
for exactly that reason.

# Why it never writes in place

Because it destroys things the original still has: the earlier revision, and
every digital signature (§12.8.1). A command that could overwrite the
operator's file with a copy that has lost both is one keystroke from a loss
nothing can undo — so this offers only [`files::pick_save_path`], and the
window says *"this always writes a new one"* before the picker opens.

The suggested name is [`suggested_path`]'s, shared with save-a-copy: the
operator's own file with a suffix, in its own folder. A second naming scheme
for the same act is how two commands come to disagree about what a copy is
called.
