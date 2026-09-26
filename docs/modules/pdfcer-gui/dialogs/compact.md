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
