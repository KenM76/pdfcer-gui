---
name: a-compile-error-is-an-invitation-to-read-the-reply
description: An engine change that adds a parameter with a keep-old-behaviour default is silently declined by anyone clearing build errors — the compiler goes quiet and the tests stay green
metadata:
  type: feedback
---

**A compile error is an invitation to read the reply, not to satisfy the
compiler.**

**Why:** `pdfcer-core` made `pageops::merge` take a third argument — the
sources' file names — so that a bookmark opening **another of the files being
merged** is re-pointed instead of dropped. Its reply said plainly that `&[]`
preserves the old behaviour.

★ **`&[]` is exactly what a mechanical fix to the new signature would have
passed.** The build would have gone green, every test would have stayed green,
and a capability the engine had just written for us would have been silently
declined — by whoever was clearing build errors, with no decision recorded
anywhere.

The same afternoon produced the mirror case: the engine added
`EditError::FreeTextNoteConflictsWithText`, and this shell was **already
tripping it** — `canvas::textannot::spec` trimmed the operator's text while
`commit` built the note from the untrimmed parameter, so a text box authored
with a stray leading space would have been refused outright on the new pin.
That one *did* surface, as a refusal rather than a compile error, only because
somebody read the reply looking for it.

**How to apply:**

- When an engine bump breaks the build, **read the reply or the changelog entry
  for every broken call site before touching it.** A parameter added with a
  no-op default is the shape to watch for; so is a field added to a struct you
  construct, and a variant added to an enum you match.
- Distinguish the two mechanical fixes: `&[]` / `None` / `..Default::default()`
  **declines** the new behaviour, while `Some(the_real_value)` adopts it. The
  compiler cannot tell you which was meant.
- ⚠ Get the *shape* of the new argument right, not just its type. `merge` takes
  `titles` (bookmark captions — the file **stem**, an extension there is noise)
  beside `source_files` (matched against a file specification somebody else
  wrote, which **carries** the extension). Two `Vec<Vec<u8>>` lists, one call,
  and passing the stem to both compiles perfectly.
- The engine session answers within hours, so replies are usually on disk in
  `D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` before the bump reaches
  us — see [[the-engine-session-runs-in-parallel]].
