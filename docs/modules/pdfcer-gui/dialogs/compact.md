# `dialogs::compact` — the window before a full rewrite

`file.save_compacted`, wired 2026-08-28. `OPERATOR_REQUESTS.md` **O48**,
answered *"yes to all three"*.

## It writes the file BEFORE it opens, and that is the design

The window's headline number is *"a compacted copy would be 1.0 MB instead of
4.2 MB"*, and it is a **measurement of this document**, obtained by actually
serialising it — not an estimate from a heuristic.

That is a real cost: `to_full_bytes` walks and re-emits every object, which
on a dense CAD sheet is not free. It is paid because the alternative is
worse. The operator is being asked to accept three losses — a revision
history, possibly every signature, and the original file's role as the
canonical one — **in exchange for a saving**. A predicted saving that turned
out wrong would mean they accepted the losses for nothing, and there is no
way to give it back.

⇒ **When a window asks somebody to trade something irreversible for a
benefit, the benefit must be measured rather than predicted.**

The bytes are then **kept** and written to whatever the picker names, so
the file the operator receives is byte-for-byte the one the window measured.
Re-serialising after the picker would be a second computation of the same
answer, and the two could differ — the session is not editable behind this
window, but that is a property of today's shell rather than of the code.

## Where the space goes

A fold under the size line lists `pdfcer_core::space_audit::audit` for the file on disk (*Now*) and for the measured copy (*Compacted*). The rows are sorted by the larger of the two, and categories empty in both are left out. Each column sums to its file's length, which is the audit's own invariant. *Earlier versions kept in the file* is usually the row that explains the saving.

- **It is measured only when the fold opens.** That costs two parses and two audits, which a dense sheet does not need to pay just to see the headline number.
- ***Now* is the file, not the session.** It matches the `before` the size line quotes. A document with no file shows a dash.
- **A copy that does not parse back lists nothing.** A breakdown of half the trade would be misleading.

## Why the picker opens AFTER this window and not instead of it

`app::save`'s `save_copy` opens a picker straight away, correctly: a copy
costs nothing and the only question is where. This costs three things, and a
native file dialog is the wrong surface to state them on — it has nowhere to
put a sentence, and an operator halfway through choosing a folder has already
decided.

## Rule 4

Nothing here marks the canvas. The document is not changed at all — this is a
**save**, and the open session is untouched by it, which is why it needs none
of `app::actions`' four-step protocol and why `app::save`'s header applies
unchanged.

## Item notes

### `fn open_for`

`Err` carries a sentence rather than a flag, because the one thing that
can go wrong here is the engine refusing by name — a hybrid file whose
`/XRefStm` does not parse, or one whose object numbering is too sparse for
§7.5.4's single-section table. Both are facts about the operator's file
that they can act on, and collapsing them to *"could not"* would waste the
only useful thing the refusal carries.

The first of those read *"a hybrid-reference file"* until 2026-09-11.
`Pass 281.0` narrowed the engine's refusal to the unparseable case, so
compacting an ordinary hybrid now succeeds where this comment said it
could not. The sentence the operator reads is the engine's own and was
never wrong; only the account of it here was.

## Recompress

A switch under the size line, on by default, runs the engine's lossless optimize preset (`Preset::Lossless`) on the measured copy. That re-encodes every stream that comes out strictly smaller, then rebuilds the structure under the preset's `RebuildOptions`: unused objects dropped, identical ones merged, non-stream objects packed into object streams with a cross-reference stream. The window words each `RebuildReport` count that is non-zero, and says so when the header was raised to PDF 1.5. The work happens in `dialogs::recompress`.

- **It runs on a scratch session parsed from the copy.** The open document's session is untouched, so no undo step appears and nothing on the canvas changes. Optimizing the live session would have put an Optimize command in the operator's history for a save that is a copy.
- **It runs off the UI thread.** A dense sheet takes seconds at level 9. Save is greyed meanwhile, explained on hover, because the size line would otherwise promise one file and write another.
- **The size line quotes what Save writes.** That is the recompressed copy when the engine answers `OptimizedFile::Smaller`, and the plain compacted copy otherwise. The breakdown is re-measured against the same bytes.
- **A signed copy proceeds** (`allow_signed`). The window already states that the copy keeps no signatures, so a second refusal would only hide the switch. An encrypted copy is refused by the engine, and that sentence is shown with the plain copy kept.
- **Disclosure is off-canvas and complete.** The window lists how many parts were recompressed, each reason a selected part was left as stored with its count, and the engine's notes verbatim.
- **The navigation clean-up has its own two switches.** *Remove links and bookmarks that go nowhere* (on) sets `CleanupOptions::invalid_links` and `invalid_bookmarks`; *Remove named places nothing in the file links to* (off) sets `unreferenced_destinations`. Each is a separate switch because each deletes something the operator can see or navigate to, unlike recompression. `text::compact::cleaned` counts the links and names each bookmark removed. With recompression off, the plan leaves streams as stored (`StreamAction::Leave`) and the structure alone (`RebuildOptions::none`).
- **A removal is saved even when the copy is no smaller.** The engine answers `OptimizedFile::NotSmaller` when the plan removes content; the window saves it and says it was kept for what it removes. A run that only saves bytes and saves none keeps the plain copy.
- **The document-object discards are six switches, all off.** *Remove all bookmarks*, *Remove tags*, *Remove print settings*, *Remove alternate images*, *Remove form submit and reset* and *Remove links to other files* set `DiscardOptions::with_object`. Each removes something the operator relies on, so none is on by default. A class the engine adds is not shown until `text::compact::discard_toggle` words it; `text::compact::discarded` says what each took.
- **Discarding user data by kind is not offered here.** File ▸ Remove metadata runs the same engine code item by item.
- **One switch, not five presets.** Every `Preset` currently has the same settings. A picker would offer five names for one behaviour, and it can arrive once the presets diverge.
