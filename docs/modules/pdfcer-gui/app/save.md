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

★ **The exact width of that refusal, which is narrower than it sounds.**
`save_full` refuses only a hybrid whose `/XRefStm` **does not parse** — the
file says it hides objects and pdfcer cannot tell which. Ordinary hybrid
files, which is most of what Microsoft Office exports, rewrite fine. The
wide refusal — any hybrid at all — stands in `save_full_encrypted` and
`save_full_decrypted`, so **adding or removing encryption refuses every
hybrid**. The posture being cited here is the same either way.

### ★★★ 1.1 …EXCEPT while a redaction is staged

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

## 3. ★ What happens to the edit epoch and the dirty state: **nothing**,
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

## 6. ★★★ What a save says about a DIGITAL SIGNATURE

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
