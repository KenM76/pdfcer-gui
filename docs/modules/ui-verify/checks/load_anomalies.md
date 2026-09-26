# `ui-verify/checks/load_anomalies`

**A PDF that contradicts itself says so, and a PDF that does not stays
quiet** — the driven half of the load-anomaly disclosure.

# What this file is for

`crate::app::status::anomalies` (in the shell) turns the engine's
`Document::load_anomalies()` into two operator-facing surfaces:

* a one-line **census** in the status bar — *"1 duplicate dictionary key"* —
  published as the trace region `status-group:load-anomalies`;
* a **row per anomaly** in the Document properties panel, naming the object,
  the key, the value pdfcer used and the value it left — published as
  `properties.load-anomalies` with `properties.load-anomalies.N` per row.

Every part of that is unit-tested, including against the real engine loader
on a fixture authored for it. What no unit test can see is whether the two
surfaces are **connected to a running window**: whether the status bar draws
the line on the frame the document opens, and whether the panel draws its
rows where an operator can read them. That is this file's whole subject, and
it is the standing rule R1 — *verify by driving the binary, not by a passing
test*.

# The second launch is the check

Each of the two checks below launches **twice**: once on
`fixtures/contradicts-itself.pdf`, asserting the disclosure is THERE, and
once on `fixtures/four-pages.pdf`, asserting it is NOT.

Without the control launch, "the region is declared" is satisfied by a build
that declares it on every file — a heading permanently pinned to the status
bar and a section permanently pinned to the panel, saying *"this file
contradicts itself"* about files that do not. That is a worse defect than
the one the check is for, it is the defect R8b's *fuzzy, never sneaky*
clause is most alert to (a disclosure that cries wolf trains an operator to
stop reading it), and it wears exactly the same green tick.

The control fixture's cleanliness is not assumed either: the shell's own
`the_control_fixtures_a_driven_run_uses_are_genuinely_clean` asserts through
the engine that both files this module names really do load without an
anomaly. If that ever changed, the absence assertion here would go red and
blame the program for something that is true of the fixture — so the
tripwire lives in the crate that runs on every `cargo test`, not here.

# The third check is not a disclosure at all — it is the way out

`RereadingUnderTheOtherValueIsOffered` drives the **control** that arrived
at the foot of that same block on 2026-09-10. The operator's ruling is that
*"if the user can intervene in a decision that should always be an option"*,
and until that day this shell reported the decision and offered no way to
revisit it.

It is the check with the most between its ends. A press on that button
travels through an `Action`, both of `app::actions::document`'s guards, a
`PendingIntent` that carries the operator's chosen `LoadOptions` across a
dialog that may or may not appear, `reread_active_document`, and finally
`Document::load_with_options`. **Eight of those steps have unit tests and
not one of them can see the step in front of it** — the standing lesson of
this project, paid for by a feature that passed eight green tests while
doing one fourteenth of its job. So the two trace events at the far ends
are asserted together: the button was pressed, *and* a loader started.

# Why two checks and not one

Because they cost different things. The status-bar half needs **no pointer
and no keyboard**: the bar draws on the first frame after the document
opens, so the harness launches, waits, and reads the trace. The panel half
has to click a mode segment, a ribbon tab and a ribbon item.

Fusing them would make the cheap, always-runnable half inherit the
expensive half's `--no-input` SKIP — and a SKIP is not red, so the whole
disclosure would silently stop being evidence on every run where the
machine's pointer belonged to somebody else. That is a failure mode this
project has paid for repeatedly. Split, the status-bar half runs on every
sweep including the ones nobody can watch.

# What these do NOT prove

That the *wording* is right. The trace publishes a region name and a
rectangle, not a string, so a build that drew the census clause in the wrong
order, or spelled the discarded value where the kept one belongs, would pass
both. The wording is asserted in `crate::app::status::anomalies`' own tests,
against the same fixture, through the same engine call — deliberately,
because a string is exactly what a unit test CAN see. What it cannot see is
the window, and that is what is here.

## Item notes

### `const CONTRADICTS`

Built by `fixtures/contradicts-itself.PROVENANCE.py`, which computes its own
xref offsets so that the file's cross-reference table is **sound** — that
matters, because a scanned-and-rebuilt file lights `Document::recovery()`
instead, which is a different disclosure with a different status region, and
a check reading the wrong one would be green about a surface it never
touched.

### `const CLEAN`

⚠ Asserted clean through the engine by the shell's own
`the_control_fixtures_a_driven_run_uses_are_genuinely_clean`. Do not swap it
for another fixture without adding the new name there; an absence assertion
against an unverified control is an assertion about nothing.

### `const REREAD_REQUESTED`

**Two events and not one, because they are two different claims.** The
first says a button was hit; the second says a re-load actually started.
Between them sit both of `apply_reread_with_duplicate_keys`' guards, and a
build where those guards mis-fire — a stale `save_pending`, an unsaved
dialog raised over a document with nothing to save — traces the first and
never the second. That gap is precisely what a unit test on the arm cannot
see, and it is why this check exists at all.

### `const KEEP_FIRST`

⚠ Asserted as a **string**, against `crate::app::state::policy_token`'s
output, and that function exists so this comparison is against a contract
rather than against `{:?}` on somebody else's `#[non_exhaustive]` enum.
A `Debug` rendering is not a machine-readable field and this project has
already had one driven check report the opposite of the truth while
quoting the truth in its own message.

### `const OFFSCREEN`

**Off the desktop, on purpose.** Neither status-bar launch reads a single
pixel — the whole verdict comes from the trace — so the window does not need
to be anywhere a human could see it, and putting it where a human cannot
means this check can run while the operator is working. Every on-screen
alternative either covers his window or races him for it.


The size is the harness's usual 1400x900 and is not arbitrary: a narrower
window folds status-bar groups away, and a folded group publishes no region
— which would read as "the disclosure is missing" when it is merely elided.
The `SAFE_ORIGIN + size` arithmetic that binds an on-screen check does not
bind here precisely because nothing is ever aimed at this window.

### `fn repo_fixture`

The shared resolver carries the whole account of why the path comes from
`CARGO_MANIFEST_DIR` and never from `--source-root`. What stays here is the
sentence that is about THIS check: why its two documents are not
substitutable for whatever `--pdf` happened to name.

### `fn open_properties`

Reuses `properties_metadata`'s opener rather than spelling the two clicks
again. It is the same ribbon item and the same toggle hazard — pressing
`file.document_properties` while the panel is up CLOSES it — and two copies
of that guard would be two places for the next ribbon move to have to be
applied.
