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

## Item notes

### `const FONT_EXTENSIONS`

`.ttc` and `.otc` are **deliberately absent**. A collection holds several
faces in one file and the engine refuses one outright
(`EmbedBlocker::ProgramIsCollection`), so offering one as a donor would be
resolving a face to a file that is then refused by name — a press that
always fails, which is what this project spends its time removing.

### `fn stub`

`FontEnvironment::insert_named` does not parse, so a donor can be
registered without a real face. What that does **not** buy is coverage
of the parse — see [`real_files`], which exists precisely because every
test in this module would pass on a build whose parser was dead.

### `fn only_a_real_subset_tag_is_stripped`

The negative cases are the point. A five-letter prefix, a lowercase
one, and a name that simply contains a `+` are all names in their own
right, and treating any of them as a tag would search for a face that
does not exist — silently, since the result is just "no donor".

### `fn a_collection_is_not_offered_as_a_donor`

`.ttc` and `.otc` must stay out, and that is a capability decision
rather than an oversight: the engine refuses a collection by name
(`EmbedBlocker::ProgramIsCollection`), so offering one as a donor would
resolve a face to a file guaranteed to be rejected — a press that always
fails.

### `fn the_first_folder_to_offer_a_name_keeps_it`

The opposite of the renderer environment's last-wins, and the assertion
is load-bearing rather than decorative **now that this delegates to that
environment**: `insert_named` *is* last-wins, and nothing but
[`Library::offer`]'s guard stops the operator's stated search order from
being silently reversed by the crate underneath.

### `fn a_stem_match_is_distinguishable_from_an_exact_one`

The engine registers a filename stem beside a file's advertised names
and reports either as `Exact`, which is right for a renderer and not
enough for a disclosure. Losing this would tell an operator that pdfcer
found the face their document named, when what it found was a file with
a suggestive name.

### `fn a_bundled_face_answers_only_when_it_is_allowed_to`

`FontEnvironment::bundled()` is what this scans into, so pdfcer's own
standard-14 substitutes sit in the table the whole time and one `true`
in the wrong place puts them into somebody's document. The operator said
yes on 2026-08-28 (`OPERATOR_REQUESTS.md` **O47**) — and *"yes"* is a
decision that has to be carried, not a reason to stop checking.

Both halves in one test on purpose: an assertion that only proved the
`true` case would pass on a build that ignored the flag entirely, which
is the exact defect the licensing argument is about.

### `fn a_configured_face_outranks_pdfcers_own`

The property that makes it safe to leave the bundled rung on. It is the
LAST rung — reached only after an exact name match and a family
equivalence have both failed — so a machine with fonts configured never
sees it. A build that consulted it first would embed pdfcer's stand-in
into every drawing on a machine holding the real thing, and every row
would say so, and it would still be wrong.

### `fn an_unreadable_folder_is_noted_and_the_rest_are_searched`

The remaining folders are still searched. An operator with a removable
drive in their list has one folder that comes and goes, and a scan that
abandoned the rest of the list when it met one would make the feature
unreliable in a way they could not diagnose.

### `fn a_real_font_folder_yields_real_faces`

Every test above is about the INDEX — the tag rule, first-wins, which
extensions are attempted — and every one of them would pass on a build
whose parser never ran, because they register a stub and then ask for
it. `FontProgram::parse` is the one link this module does not own and
cannot fake, and *"the folders yielded nothing"* is indistinguishable
from *"the folders were empty"* without a folder that is not.

It uses the operating system's own font directory, which is the one
folder that certainly exists on the machine this ships for — and is
deliberately **not** what the product searches: `Prefs::font_folders`
starts empty and this module never adds to it, for the licensing reason
in the header. A test may look where a product may not.

SKIPPED rather than failed where that directory is absent, because its
absence is a fact about the machine and not about this code.

### `const MAX_FONT_FILE_BYTES`

Sixteen mebibytes, matching `pdfcer`'s own ceiling. It is not about
memory — it is that a "font file" above this size is nearly always
something else that happens to have a font extension, and reading it costs
an operator a visible pause for an answer that will be *"not a usable
font"*. The skip is reported rather than silent.

### `struct Donor`

Borrows from the [`Library`] that resolved it. The bytes are held once, in
the environment; a donor that owned a copy would clone a whole face on every
lookup, for a value most callers only read a name out of.

### `enum Match`

`pdfcer_render::font::EmbedMatch`'s three rungs minus its bundled one, plus
a distinction of this shell's own. See the module header for why `Stem`
exists here and not there.

### `fn source`

A path, or the words for a bundled face. `pdfcer` writes
`"bundled: FoxitSans"` for the same value and the engine's own field doc
says the string is *"never parsed; only reported"* — so it is prose, and
prose the operator reads belongs in [`crate::text`]. This is the join,
not the wording.

### `struct Library`

The names live in a `FontEnvironment` — which owns the bytes and answers
the three-rung question — and the **paths** live here, because the
environment has no notion of where a face came from and the operator is owed
exactly that.

### `fn scan`

# Later folders do NOT win

The first folder holding a name keeps it, which is the opposite of the
renderer environment's own precedence (*"duplicate-name precedence: last
wins"*) and is deliberate. That environment is built once per render and
the last registration is simply the surviving one; **this** list is the
operator's, in an order they typed, and the Settings hint promises
*"searched in the order they appear here"*. First-wins is what makes
that sentence true.

Enforced by [`Self::offer`] rather than by registration order, because
`FontEnvironment::insert_named` is last-wins and would silently reverse
it.

### `fn scan_with`

# The operator asked for this, and the licensing argument survives

`OPERATOR_REQUESTS.md` **O47**, answered *"yes"* on 2026-08-28. The
module header's argument — that pdfcer must not choose a font program on
somebody's behalf, silently, in a file that outlives the decision — is
not overruled by this. It is satisfied the same way **O50**'s checkbox
satisfies it: the operator decided, once, explicitly.

And it is the **last** rung, which is what makes it safe to leave on.
`resolve_for_embedding` consults the bundled table only after an exact
name match and after a standard-14 family equivalence have both failed,
so a machine with real fonts configured reaches a real face first and
this never fires. It is a floor, not a preference.

### `fn donor_for`

The subset tag is handled by the engine — `resolve_for_embedding`
strips it on its second rung — and it has to be: a §9.6.4 tag is six
uppercase letters and a `+`, minted per subset, so `ABCDEF+ArialMT` and
`GHIJKL+ArialMT` are the same face and neither is a name any font file
advertises. Matching without stripping would find nothing, ever, on
exactly the documents that need embedding most.

Whether pdfcer's own faces may answer is [`Self::scan_with`]'s
argument, carried on the library rather than passed here — the decision
is the operator's and belongs to the whole scan, not to one lookup.

### `fn strip_subset_tag`

Exactly six uppercase letters and a `+`, per the standard. Anything else
before a `+` is part of the name and is kept — `Foo+Bar` is a legal, if
unusual, font name, and treating it as a tag would look for a face called
`Bar`.

Kept even though [`Library::donor_for`] no longer calls it: the
**display** side needs it, because a row reading `ABCDEF+ArialMT` shows an
operator a tag that is an artefact of subsetting and means nothing to them.
Resolution and presentation happen to want the same rule, and only one of
them is the engine's.
