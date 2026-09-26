# `ui-verify/checks/embed_fonts`

`embedding_fonts_puts_a_program_in_the_document` — **press Embed and the
document stops missing a font.**

# What this is for

`tools.embed_fonts` was registered, drawn on the Tools tab and **inert for
the whole life of the project**, behind a `SCAFFOLDED` entry whose stated
premise had expired and whose real blocker was in no register at all. It was
wired on 2026-08-28, and this is the check that keeps it wired.

## ★★★ The oracle is `missing_after=0`, not "a dialog appeared"

Almost every link in this chain can be satisfied by a build that does
nothing. The window opens on a plan; a plan can be empty; an empty plan
draws a window with a greyed button and a list of refusals, and **that is a
legitimate outcome** for a document nothing on the machine can answer. So a
check that asserted only *"the window opened"* would pass on:

- a resolver that finds no donor for anything;
- a request whose `supplied` map is never populated;
- an Embed button whose click raises no action;
- an apply arm that calls the engine and drops the result.

The number that distinguishes all four from working software is the
engine's own `missing_after` — *"the end state the whole feature exists to
reach"*, in `EmbedPlan`'s own words. This check drives the real gesture and
asserts that it reaches zero.

## ★★★ Why `PDFCER_DIAG_FONT_DIR` exists, and why it APPENDS

Embedding needs a folder of font files, and in the product that folder comes
from a preference an operator sets in Settings and pdfcer stores in
`userdata/preferences.txt`. A harness must not rewrite that file — it
belongs to whoever is running the build, and a check that edited it would
leave it edited.

So `dispatch::fonts::folders` reads an environment variable and **adds** its
folders to the operator's. Adding rather than replacing is the load-bearing
half: a variable that replaced the preference would let this check pass on a
build whose preference plumbing was broken end to end, because the harness
would then be testing its own environment variable.

## ★★ Why the fixture is `a1-titleblock.pdf`, and what it proves that a
synthetic one would not

It asks for `Helvetica`, `Helvetica-Bold` and `Helvetica` again, on three
surfaces, with no program for any of them — which is what every CAD exporter
writes and is the exact case this feature exists for.

★ **No Windows machine has a font called Helvetica.** So this fixture cannot
be embedded at all unless the resolver's *alias* rung works, and a passing
run is therefore evidence for a claim no unit test in this project can make:
that `pdfcer_render::FontEnvironment`'s standard-14 equivalence is reached
from the shell, on a real font folder, through the real dispatch. The first
draft of that resolver had only an exact rung and would fail here while
every one of its own tests passed — they registered a name and then asked
for that name.

## What this check does NOT cover, stated rather than implied

**The saved file.** Embedding is one `EditSession` command and this asserts
on the session's own report of it; whether the program survives a write and
reopen is `save_copy`'s territory and is not asserted here. The engine tests
that round trip; this tests that the GUI reaches the engine.
