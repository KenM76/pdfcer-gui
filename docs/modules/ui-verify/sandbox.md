# `ui-verify/sandbox`

**A profile directory per check** — the fix for a suite that measured the
order it ran in.

# The defect this module exists for

`pdfcer-gui` is **portable**: `pdfcer_core::settings::resolve_store()` looks
for a writable `userdata/` *beside the executable* first, and only falls back
to a platform config directory. `app::persistence`, `app::prefs`,
`app::recent`, `app::pickstore` and the shell's `userdata/shell.ron` all live
under that roof. One binary on disk is therefore **one profile**, and a
sweep that points every check at the same `--exe` points every check at the
same remembered state.

That was harmless for exactly as long as the application threw the stored
mode away on every launch — which it did, by accident, from the day the
ribbon shipped until **2026-09-06** (`f4aeee6`, *"Open in the mode you were
last in has been dead since the day it shipped"*). Fixing it turned a
dormant harness defect live in one commit:

> a check that clicks the Edit segment now leaves `mode: Some("edit")` in
> `userdata/layout.ron`, and **every later check in the sweep starts in
> Edit**.

## What it cost, the same afternoon

`a_link_goes_to_the_page_it_names` reported that clicking a link *"produced
nothing"* — no `link-click`, no `page-links`. That is symptom-identical to a
regression in the press ladder, and it is where the investigation went. It
was not a regression: **in Edit a click *selects* a link rather than
following it**, deliberately, so the hit test that would emit those lines is
never reached. On a fresh profile the same binary, unchanged, reaches the
link.

⇒ **A suite that shares persistent state measures the order it ran in, and
the contamination is invisible in the failing check's own report.** The
failing check's evidence — its trace, its screenshot, its reason — is
complete, articulate, and about the wrong subject; nothing in it mentions the
check that ran forty minutes earlier and changed the mode. That is the same
family as the wrong-`--doc-point` failures `RESUME.md` records: *an
articulate failure message about nothing*.

## Why the fix is not "clear `userdata/` between checks"

Two reasons, and the second is the one that decides it.

1. **A cleared profile is not a fresh profile.** `ui_scale::write_preference`
   already argues this for its own file: an *absent* `preferences.txt`
   exercises the absent-file path, which is a different state from a file
   holding defaults. Deleting a directory between checks makes every check
   after the first run against "first launch ever", which no operator's
   machine is after the first day.
2. **It cannot be made total.** `userdata/` is where the state we know about
   lives. A future build that remembers one more thing somewhere else — a
   sibling file, a lock, a cache — silently re-opens the hole, and the tell
   is again a confident failure about the wrong subject. Isolating the
   *directory the binary resolves from* closes the class rather than the
   instance: whatever the binary decides to write beside itself, it writes
   inside one check's sandbox.

# How it works

One directory per check, holding a **hard link** to the binary under test:

```text
<exe dir>/.ui-verify-profiles/<check name>/pdfcer-gui.exe   ← the link
<exe dir>/.ui-verify-profiles/<check name>/models/…         ← linked too
<exe dir>/.ui-verify-profiles/<check name>/userdata/…       ← what the check writes
```

The launched process resolves its own directory and finds that `userdata/`
— private to this check, and deleted with it.

⚠ It is **not empty**. [`seed_prefs`] puts exactly one preference in it
before the process starts, and its own note argues why and what that costs a
check written to measure the thing it suppresses. Nothing else of the
operator's is brought across; see below.

## Why a hard link rather than a copy

Three properties, each of which a copy would spend:

| | hard link | copy |
|---|---|---|
| cost | one directory entry | 28 MB × ~150 checks ≈ 4 GB of writes per sweep |
| **mtime** | the *same* file, so the same timestamp | `CopyFileEx` preserves it on Windows, but nothing in the standard library promises that |
| staleness gate | [`crate::launch::staleness_complaint`] compares the exe's mtime against the sources and is **on by default** | a copy whose mtime moved forward would make a genuinely stale binary look fresh |

The third is the one worth stating plainly: `--allow-stale` is off by default
because *a missing trace from an unbuilt change looks exactly like a broken
feature*. A sandbox that quietly refreshed the timestamp would disarm that
gate for the whole suite, and the failure it lets through is precisely the
one the gate was written to catch.

## Why the sandbox root sits beside the exe and not under `--out`


It also inherits the operator's own discipline. The standing rule is **never
drive the published build**: copy `target/release/pdfcer-gui.exe` to a
scratch directory and point `--exe` at the copy, so the suite's side effects
do not land in the operator's saved state. The sandbox root is created
beside whatever `--exe` names, so it lands in that scratch directory too.

A copy is still the fallback, for the case where the link cannot be made — a
filesystem without hard links, a permission refusal, a `--exe` on a volume
that will not take another entry. [`Sandbox::how`] says which happened, and
the run header prints it, because "this sweep copied 4 GB" is worth knowing
and "this sweep silently did something other than what the module says" is
not.

## What is brought across, and what deliberately is not

**Brought:** the executable, its sibling `models/` directory, and any sibling
`.dll`. `models/ocrs` is resolved *beside the executable* by
`crate::ocr::resolve_models`, and a sandbox without it would make
`ocr_finds_words_in_a_scan` report a missing model directory — which is a
SKIP, and a SKIP is not red, so the check could be dead for the rest of the
project's life while the suite looked healthy. That is the `repo_fixture`
shape [`crate::checks::CheckContext::out`] records paying for once already.

**Not brought:** `userdata/` — bar the single seeded key [`seed_prefs`]
writes. That is the entire point. A sandbox seeded with
the operator's remembered mode would isolate the checks from each other and
leave every one of them contaminated by whatever the last real session did.

## Lifetime

[`Sandbox`] deletes its directory on drop. A failure to delete is a **warning
on stderr, never a verdict**: a leaked 200-byte directory entry is a
tidiness problem, and downgrading a real pass because the harness could not
tidy up would be reporting the harness's housekeeping as a defect in the
program — `ui_scale`'s `RestoreScale` guard makes the same call for the same
reason.

A directory left behind by a killed run is removed when the next run creates
the same sandbox. That recovery assumes the project's standing **one driven
run at a time** rule, which is not this module's to enforce and is already
mandatory for a much harder reason: the harness moves the real pointer, so
concurrent runners fight over it and every verdict in the sweep is
worthless.

# Proving it works

An isolation fix that does not isolate is worse than none, because the next
contaminated failure is investigated with the contamination ruled out. The
falsification is two checks in one invocation, the first of which switches
mode and the second of which is sensitive to it:

```text
ui-verify --exe <scratch>/pdfcer-gui.exe --pdf fixtures/a1-titleblock.pdf \
          --doc-point 0,300,500 \
          --check clipboard_mode_switches_the_ribbon \
          --check a_link_goes_to_the_page_it_names
```

and then the same pair with `--shared-profile`, which restores the
unsandboxed behaviour. Sandboxed, both checks pass in either order;
`--shared-profile` makes the second one's verdict depend on what the first
one left in `userdata/`. If that difference ever stops appearing, the
isolation has been lost and this module is no longer doing its job.
