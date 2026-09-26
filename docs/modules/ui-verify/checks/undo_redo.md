# `ui-verify/checks/undo_redo`

`undo_redo_round_trip` — **the check for the pair of commands that could
not take anything back**: author a change, undo it, redo it, and prove all
three states from outside the process.

# What was wrong, and why nothing noticed

`edit.undo` and `edit.redo` have been registered since the ribbon landed.
They are on the **quick-access toolbar**, which is drawn in every mode over
every document; they are bound to `Ctrl+Z`, `Ctrl+Y` and `Ctrl+Shift+Z`;
their tooltips name those chords. They had **no dispatch arm**, so every
press traced `command-unimplemented` and did nothing.

That is `crate::checks::save_copy`'s defect at the other end of the same
day, and v0.1.0 made it worse rather than better: saving now works, so every
authoring feature this shell has — dimensions, seven markup kinds, text
marks, form fills — was reachable by an operator with no way to take any of
it back.

The whole suite was green throughout, and it had to be. `pdfcer-core` tests
`EditSession::undo` exhaustively; `shell::commands` tests the registration
and the two predicates; `app::conditions` tests that the ribbon reads the
set it publishes. What no test in the workspace can observe is the **join** —
that pressing the control reaches an arm, that the arm reaches the engine,
that the engine's answer reaches the caches, and that the surfaces an
operator reads move back with it.

# The five links, and where each is otherwise covered

| # | Link | Its own test |
|---|---|---|
| 1 | the QAT click reports the command | `egui-shell`'s `qat::render` — yes |
| 2 | dispatch raises `Action::Undo` | `app::dispatch`'s unit test — yes |
| 3 | the apply reaches `EditSession::undo` | **nothing** |
| 4 | the **epoch** moves, so every epoch-keyed cache rebuilds | **nothing** |
| 5 | the **cached texture** is dropped, so the canvas re-rasters | **nothing** |

Links 4 and 5 are the reason this check exists in the shape it does. They
are the two steps of `app::actions::apply`'s four-step protocol that have no
observable consequence *inside* the process a unit test could reach: a build
that mutated the session correctly and skipped both would satisfy every
count anyone could read from the engine, and show the operator the state
they had just taken back.

# The six phases

| Phase | Does | Expected |
|---|---|---|
| A | Review, Comments panel brought to the front | a `comments-panel listed=N` baseline published **after** the `mode-changed` line, and an `objects` line to key phase D on |
| B | click **Undo** with an empty log | **no** `ribbon-command-invoked`: the control is correctly greyed |
| C | arm Rectangle, drag on the page | `add-markup`, and `listed=N+1` |
| D | click **Undo** | `undo kind=AddAnnotation`, `undo-applied … epoch=`, `listed=N`, **a new `objects` line**, **a new `render-spawn`** |
| E | click **Redo** | `redo kind=AddAnnotation`, `listed=N+1`, and the same two invalidation signals again |
| F | click **Redo** again | **no** `ribbon-command-invoked`: the stack emptied and the condition followed |

# The falsifying phases, and the build each one catches

`crate::checks`' rule for a new check is that *"it must fail against a build
where the wiring is absent"*. The counts alone do not satisfy that, and
saying why is the most useful paragraph in this file.

## The count is a weak oracle on its own

`comments-panel listed=` is derived by walking the session's annotation list
afresh **every frame** — the panel holds no cache. So a build whose undo did
this:

```ignore
Arc::get_mut(&mut doc.session).map(EditSession::undo);   // and nothing else
```

would move every count in this check correctly. The annotation really is off
the session; the panel really does list one fewer. And the operator would be
looking at a page that still has the rectangle on it, with a selection
resolved against a revision that no longer exists, a decomposition listing
objects from the old one, and a page-text cache to match. **Every number
this check could read would already be right.**

## D-objects catches: *the session was mutated and the epoch never moved*

`OpenDoc::trace_object_count` emits one `objects n=… page=…` line per
**`(page index, edit epoch)` pair** and suppresses the rest — that
suppression is what makes it an epoch oracle rather than a page count. It is
written by a different subsystem from the one under test
(`app::state`, called from `render::settle`), about a cache it owns, and it
is *silent* unless the epoch moved.

So: no new `objects` line after the undo ⇒ `edit_epoch` did not change ⇒ the
decomposition, the page-text extraction, the font inventory and the canvas
selection are all still describing the revision the operator just left. The
planted build above passes every `listed=` assertion here and fails this one.

Note what it is **not**: an assertion that `n` changed. An annotation is not
a content object, so `n` is the same before and after — which is exactly
why the *line's existence* is the signal and its value is not.

## D-render catches: *the epoch moved and the texture was kept*

`render::worker` emits `render-spawn gen=… page=… scale=…` when a raster
starts. `settle_and_rasterize` keys the cached page texture on the page index
and the raster scale, and an undo changes **neither**, so nothing would ask
for a new raster unless `vector_edit`'s fourth step dropped the texture. This
is the one signal that speaks for the pixels an operator is actually looking
at, and it is independent of the epoch: a build that bumped the epoch and
kept the texture passes D-objects and fails here.

It is a **one-directional** oracle and this file says so rather than
implying otherwise: a re-raster the harness did not cause — a resize, a
scroll, a strip page arriving — would also raise the count, so a spurious
spawn could let a non-dropping build through. Nothing here moves the window
or the scroll between the phases, and the failure it would produce is a
*false pass*, never a false failure.

## B and F catch: *the conditions were published unconditionally*

`undo.available` and `redo.available` were absent from `app::conditions` for
the whole life of the project, with a comment saying so. The tempting way to
land them is to publish them beside `doc.open` and move on — which arms both
controls permanently, and looks identical to a working build in every phase
that presses one *after* an edit.

B presses Undo when the log is empty and F presses Redo when the stack is,
and both read the **absence** of `ribbon-command-invoked` as the evidence.
That absence is admissible under `crate::checks`' rule 4 for the reason
`crate::checks::text_markup` states: *a greyed `egui` control does not emit
the event at all*, and the same control is shown to invoke, in the same run,
once its stack is non-empty. A run that never reached D would have proved
nothing by B alone.

# Why the Comments panel is the oracle, and not a pixel

`crate::checks::save_copy`'s argument, unchanged: a 2 pt rectangle over a CAD
drawing is a few hundred antialiased pixels among a page already full of thin
dark lines, and no threshold in this crate separates the two. A count that
moves in both directions is a far better oracle than a screenshot here — and
the two invalidation signals above are what stop the count from being the
*only* one.

# Mouse only — `Ctrl+Z` is NOT driven

Every gesture here is a real `SetCursorPos` + `mouse_event` on a QAT control
or the page. `Ctrl+Z`, `Ctrl+Y` and `Ctrl+Shift+Z` are **not driven here**,
and [`crate::checks::chords`] drives them instead.


That matters more for this pair than for any other command in the shell,
because the keyboard is undo's *primary* route and the QAT is its secondary
one. What is proven here is the arm, the engine call and the invalidation;
what is not proven is that the chord reaches the arm.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given;
* a mode segment, a tab or a control was never declared, or took no click;
* the QAT dropped controls for want of width (`ribbon-qat-controls-dropped`);
* the application never traced an `objects` line, so phase D has no epoch
  oracle to read;
* the fixture carries annotations the Comments panel excludes, so `listed=`
  would not move by exactly one.

## Item notes

### `const MODE`

`crate::checks::save_copy`'s choice, for its reason: a markup tool that works
in Review works in Edit, and this check's subject is not the tool. Undo
itself is mode-independent — it is on the QAT, which no mode hides — so the
mode here is entirely about reaching an *edit* to undo.

### `const UNDO`

`ribbon.qat.` and not `ribbon.item.`: these two sit on **no tab**
(`shell::manifest`'s mode list says so in as many words), so the QAT is the
only surface a pointer can reach them on. That is also why this check never
switches tabs to press them — the QAT is drawn beside the tab strip in every
mode, so phases D and E press it with the Markup tab still active.

### `const UNDO_DECLINED_EVENT`

Read only to improve a failure message. Its presence in phases D or E would
mean the click arrived, reached the arm, and the arm disagreed with the
condition that armed the control — a different fix from a click that never
arrived.

### `const OBJECTS_EVENT`

Emitted by `OpenDoc::trace_object_count` once per `(page index, edit epoch)`
pair and suppressed for every repeat, so a *new* line means the epoch moved.
See the module header for the build this catches and why the count `n` is
deliberately not the thing asserted on.

### `const OBJECTS_UNAVAILABLE_EVENT`

Read for a SKIP reason: a fixture whose page cannot be decomposed emits this
instead of [`OBJECTS_EVENT`], and the epoch oracle is then unavailable for a
reason that has nothing to do with undo.

### `const RENDER_SPAWN_EVENT`

A raster starting. Nothing else asks for one after an undo, because the
texture's key carries only the page index and the raster scale and an undo
changes neither — so a new spawn means `vector_edit`'s fourth step ran.

### `const QAT_DROPPED_EVENT`

The shell's own disclosure, read for a SKIP: a dropped control declares no
rect, and "there is nothing to aim at" would otherwise be reported as though
the command were missing.

### `const DRAG`

`crate::checks::save_copy`'s drag, deliberately: two checks that author the
same annotation the same way are two independent readings of one gesture, and
a different rectangle here would add a variable neither of them is about.

### `fn invokes`

A **count**, never a presence: this check clicks the same two controls more
than once, and "has it ever been invoked?" would be answered `true` by a
click made ten seconds earlier — which is precisely the question phases B and
F must not ask.

### `struct Invalidation`

Read together and compared together, because the failure they exist to catch
is *one of the two steps was omitted* and a check that read them at different
moments could not attribute a change to the phase that caused it.

### `fn click_qat`

Returns `true` when the click produced a new `ribbon-command-invoked`, and
`false` when it did not — which is the answer phases B and F are asking for
and the reason this is not [`click_command`]. A greyed `egui` control takes
the click and emits nothing, so the two outcomes are both *expected results*
here rather than one being a failure to be raised from inside.

The rect is still required, and its absence is still a SKIP: a control that
was never drawn cannot have been clicked, and reading *that* silence as
"correctly greyed" would be the vacuous pass this crate's rule 4 exists to
forbid.

### `fn comments_count`

Both files carried the same eight lines and therefore the same defect: the
census was read with `Trace::last`, which searches the whole capture, so a
panel that had been sent to the back of its dock and had **stopped tracing**
kept answering with the count it published in the previous mode. On the
driven sweep of 2026-09-05 that made this check and `save_copy_round_trip`
fail in identical words against a panel that was working, and the sweep
report read the duplication as corroboration: *"two independent witnesses"*.

The reader now lives in [`crate::checks::comments_census`], once, and its
header carries the whole finding. Nothing in this file reads the census
directly any more, deliberately.

### `struct Step`

Factored because phases D and E are the **same** six assertions in opposite
directions, and two hand-written copies would be two chances for one of them
to lose the invalidation half — which is the half no other test in the
workspace makes.

### `fn the_selectors_match_the_shells_own_spelling`

Pinned for the reason every sibling check pins its own: the two crates
are joined by a **string** and nothing else, so a rename would leave both
sides compiling while every assertion here quietly stopped matching — and
a check that matches nothing passes vacuously.

### `fn the_invalidation_reading_counts_lines_rather_than_finding_one`

[`Invalidation`] is three lines of arithmetic and exactly the kind of
thing that gets "simplified" into a presence test — which would pass
against the build this check exists to catch, because an `objects` line
from the document's *first* frame is present whether or not the undo
produced a second one.
