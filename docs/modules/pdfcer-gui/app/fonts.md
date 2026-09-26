# `app::fonts` — turning the operator's font folders into donors an embed
can use

The half of font embedding that is **the shell's**, and the engine is
explicit that it is: `EmbedRequest::supplied` is a `/BaseFont` → donor map
*"the shell resolved for it"*, and `pdfcer`'s own note is blunter —
**"the source fonts come from `--font-dir`; pdfcer never goes looking."**

## Why pdfcer does not go looking, and why this module must not either

Embedding puts a font **program** — the actual outlines — inside somebody's
document, which they then send to somebody else. Which font that is, is a
licensing question with a different answer for every foundry, and a program
that searched `C:\Windows\Fonts` on its own would be answering it silently
on the operator's behalf, in a file that outlives the decision.

So the folders come from [`crate::app::prefs::Prefs::font_folders`], which
is empty until an operator puts something in it, and this module searches
**those and nothing else**. There is no fallback, no bundled directory, and
no "well, try the system fonts too". An empty list means an embed has
nowhere to take a font from, and that is reported rather than worked around.

## The WALK is the shell's; the RESOLUTION is the engine's


`resolve_for_embedding` tries, in order:

| rung | what it means |
|---|---|
| `Exact` | a file advertising the name the document spells, tag stripped |
| `Alias` | a **standard-14 family equivalence** — `Helvetica` → `Arial` |
| `Bundled` | a face pdfcer itself ships. **Offered only when the operator ticks the box** — see below |


⇒ Recorded because the shape generalises: *"the shell owns resolution"* was
read as *"the shell must implement resolution"*. It means the shell owns the
**filesystem** — `pdfcer-core` must not read a directory — and `pdfcer-render`
is not `pdfcer-core`.


This section has now been wrong in **both** directions and the history is
worth two sentences, because the shape recurs.

It originally read *"Why bundled faces are NOT offered"*, and said this
shell had no equivalent of `pdfcer`'s `--use-bundled-fonts` *"because the
Embed window has no settings by design"*. **That became false on
2026-08-28**, when `dialogs::embed` was changed to pass `true`
unconditionally — and the paragraph here went on asserting the opposite for
eight days, in the module that owns the argument, while
`ui-verify`'s `embedding_works_with_no_font_folder_at_all` passed against
the behaviour it denied.

⇒ The general form, and this project has paid for it six times: **a
paragraph explaining why something is not done outlives the day it is
done.** Nothing compiles differently when it goes stale.

**What is true now.** The third rung is offered as a **disclosed opt-in**:
`dialogs::embed` scans with `allow_bundled` true so that it can *say* what
pdfcer's own faces would answer for, names them, and commits a request that
includes them only if the operator ticks a box that is **off** when the
window opens. That is `pdfcer`'s own shape — `--use-bundled-fonts`, off by
default — and its own reason, which is not the letterforms but the licence:
the bundled faces are BSD-3-Clause, and embedding one puts it inside a file
the operator distributes.


## A stem match is still disclosed, and the engine does not distinguish it

`FontEnvironment` registers a file's filename stem alongside its advertised
names and reports a hit on either as `Exact`. That is right for the
*renderer*, where the question is only *"can I draw this"*. It is not enough
here: a stem match is this shell deciding that a file called `Helv.ttf` is
the face a document spells `Helvetica`, which is an inference, and rule 4's
surviving half says an inference the operator cannot see owes them a report.

So [`Library`] keeps its own record of which names came only from a stem and
re-grades such a hit to [`Match::Stem`]. **The engine is told `Alias`** for
one — see `dialogs::embed`, which explains why understating it would disable
the engine's symbolic guard from the outside.

## Determinism

Folders are searched in list order and files within a folder are **sorted**
before being read, so two runs over the same folders produce the same donor
for the same face. `pdfcer` sorts for the same reason and cites R19's
spirit: an OS directory-iteration order is not an order.
