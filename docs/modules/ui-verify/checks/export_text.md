# `ui-verify/checks/export_text`

`export_text_writes_the_documents_words` — the text reaches disk, and the
file holds exactly the characters the shell said it wrote.


Said here, in the module's own words, rather than left for an absent result
to imply. Another session owned the desktop while this was written, and this
harness drives a real window with real clicks — two of them running at once
is two sessions fighting over one focus, and the loser's verdict is noise
that looks like a finding.

`export_image_emf` carries the same disclosure for the same reason, on the
same afternoon. **An unrun check is not a passing check**, and this project's
standing rule is that no UI change is done until it has been verified by
driving the running binary. This one has not been.

# The gap this closes

`file.export_text` was in `manifest::registers`' planned list for the life of
the project, marked `C` — *"pdfcer-core extracts text already. Needs a save
dialog and nothing else."* It shipped 2026-09-04 on the operator's ask:
*"also the engine can export PDFs as text. we should have export/import for
that."*

# Why this needs driving

The writer is not the interesting half — `text_extract` is tested in
`pdfcer-core`, and the joining, the encoding and the filename derivation are
unit-tested in `app::actions::exporttext`. What no unit test can reach is the
six links between a ribbon press and a file:

1. the arm builds a window against the **open document's** page list;
2. the window raises an `Action` carrying a `TextExportPlan`;
3. the apply arm extracts through the **settings funnel**, over
   `session.view()` — the revision the operator is looking at, not the file
   on disk;
4. it refuses, before the picker, if nothing came out;
5. it opens a save dialog — a modal OS window, which is why this is an
   action at all;
6. it writes the bytes and discloses what could not travel.

Link 3 is the one that cannot be unit-tested: `extract_pages_view` needs a
live `EditSession` over a real document, and whether the plan's page indices
still name pages when the queue drains is a question about a **running
frame**.

# The assertion that makes this more than a smoke test

**The characters in the file are counted and reconciled against the trace,
by an exact identity rather than a bound.**

At the default plan the file is the pages' text joined by one U+000C each,
with no byte-order mark and no line-ending rewrite. `chars=` in the trace is
`Assembled::characters`, which counts the pages' own characters and
deliberately **excludes** pdfcer's separators. So:

```text
    characters in the file  ==  chars=  +  (pages= - 1)
```

— exactly, for every document. A build that reported an outcome from one
extraction and wrote another's bytes passes every assertion above this one
and fails this. So does a build that silently dropped a page, wrote a
leading or trailing separator, or applied an encoding transformation it did
not disclose.

The identity is asserted rather than a `>=` bound — the opposite of
[`super::export_dxf`]'s deliberate looseness — because unlike DXF entity
markers, nothing else in a text file can contribute a character. A loose
bound here would buy no safety and would give up the whole assertion.

# And it asserts the SEPARATOR the identity depends on

The trace carries `separator=` and `bom=`. If a later build changes the
window's defaults, the identity above stops holding and this check must say
*"the defaults moved"* rather than *"the export is broken"*. Those are
opposite findings and a check that cannot tell them apart is worse than no
check — so the preconditions are read out of the trace and reported as their
own failure.
