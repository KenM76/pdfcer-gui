# Developing pdfcer-gui

The engineering guide: how the tree is laid out, how to build and check it, and
the standards every source file and document in it follows.

| Looking for | Read |
|---|---|
| what pdfcer is, and a download | [`README.md`](README.md) |
| how to use it | [`MANUAL.md`](MANUAL.md) |
| what works today, measured | [`FEATURES.md`](FEATURES.md) |
| **a new session starts here** | [`RESUME.md`](RESUME.md) |
| the charter and the fold-in procedure | [`PROJECT_PLAN.md`](PROJECT_PLAN.md) |

---

## 1. What this workspace is

A replacement GUI for the `pdfcer` PDF engine, built as two crates:

- **`crates/pdfcer-gui`** — the application. Knows about PDF.
- **`crates/egui-shell`** — the reusable shell: ribbon, dock, modes, layout
  persistence, theme, command registry. Knows nothing about PDF, and a gate
  (`check-shell-purity.sh`) fails the build if it learns.

plus two small platform crates (`native-window`, `native-clipboard`) and the
verification harness in `tools/ui-verify`.

`pdfcer-gui` depends on `pdfcer-core`, `pdfcer-render` and `pdfcer-print` as
**git dependencies on the `main` branch** of `file:///D:/Dev/pdfcer`, and Rust
links them statically. The release binary therefore already carries the engine;
there is no integration step to perform before shipping.

A branch pin moves on its own. `Cargo.lock` records the resolved sha, so the
build is reproducible until someone runs `cargo update -p pdfcer-core`, but the
engine's **source** under `D:\Dev\pdfcer` changes under this repository at any
time, with no command run here. Two consequences bind everything below: run
`cargo update` for the three engine crates before a build that is meant to
carry the latest engine, and never cite the engine by line number (§5.1).

**`D:\Dev\pdfcer` is read-only from here.** Engine changes are written up and
handed to the operator, never applied. See `PROJECT_PLAN.md` §7.

## 2. Tree layout

```
crates/
  pdfcer-gui/src/
    app/          frame composition, state, command dispatch, actions, status bar
    canvas/       the page view: pick, move, zoom, tools, rulers, geometry
    dialogs/      modal and modeless windows
    panels/       the dockable panels
    text/         text extraction, editing, markup, forms, redaction, settings
    render/       the render worker, settle debounce, raster cache
    shell/        this app's manifest and command catalogue for egui-shell
    icons/        the icon set and its catalogue
  egui-shell/src/
    ribbon/       tab/group/item layout and the narrowing plan
    dock/         panel host over egui_tiles
    manifest/     the serializable ribbon/dock/mode/keymap description
    theme/        palette and egui style derivation
tools/
  gates/          the CI gates; run-all.sh runs every one
  ui-verify/      drives the release binary and asserts on pixels and traces
fixtures/         PDFs built by tools/gen-*.py, each with a PROVENANCE.md
evidence/         screenshots and traces backing observational claims
mockups/          HTML mockups of the ribbon, the window and the modes
```

## 3. Build and check

```sh
cargo build --release
CARGO_BUILD_JOBS=2 cargo test --workspace
bash tools/gates/run-all.sh
```

**The job count on the test suite is not optional.** The workspace builds more
than twenty test binaries, the `pdfcer-gui` one alone linking tens of megabytes
of debug object. At cargo's default of one job per core, that many linkers run
at once, each holding well over a gigabyte, and the machine runs out of memory.
A run killed that way leaves orphaned `cargo`/`rustc`/`link` processes still
holding the memory, which must be killed before a retry. For the same reason,
run one cargo job at a time — never chain the gates, the tests and a build into
a single background command.

`run-all.sh` runs fmt, clippy and every gate in `tools/gates/`. It has three
states, not two: `0` pass, `1` fail, `3` a gate was **skipped** because its
precondition was absent. A skip is not a pass — CI must not go green on a gate
set that did not fully run.

Every gate that is a grep over source carries a `--self-test` that plants a
violation and asserts the gate catches it. The self-tests run before any gate
is trusted: a gate that cannot detect its own planted violation has no verdict
worth reading.

### Driving the binary

A passing test is not evidence that a UI works. `tools/ui-verify` launches the
release binary, opens a fixture, drives a scripted sequence through the OS, and
asserts on both the `PDFCER_DIAG` trace and the captured pixels.

```sh
cargo run --release -q -p ui-verify -- \
    --exe target/release/pdfcer-gui.exe \
    --pdf D:/Dev/temp/pdfcer/SW41177.pdf \
    --doc-point 0,300,500
cargo run --release -q -p ui-verify -- --list      # the registered checks
```

The full sweep takes about ninety-five minutes and aborts if any `.rs` or
`.toml` in the tree is edited while it runs — the staleness guard walks the
source root and considers only those two extensions. Markdown, `.sh` and `.py`
edits are therefore safe during a sweep, **with one exception**:
`tools/ui-verify/sweep-full.sh` is the script bash is currently executing, and
bash reads a running script incrementally from a byte offset rather than loading
it whole. Editing it mid-sweep makes the live run execute shifted bytes from the
middle of a command. Leave that one file alone until `=== SWEEP-DONE`.

### Packaging

```sh
python tools/package-portable.py --verify --note "what this milestone added"
```

Writes `D:\builds\pdfcergui-<stamp>-<engine>-<shell>\` — one folder per build,
never an overwrite, because a running Windows executable cannot be replaced and
a half-updated folder is worse than either version. `<engine>` is
`D:\Dev\pdfcer`'s short HEAD, `<shell>` is this workspace's; either gains a
`-dirty` marker when its tree carries changes that can reach a compiler.

A source digest is recorded alongside, and joins the folder name when the shell
tree is dirty. It cannot say what the code was, only whether two builds came
from identical bytes — which is the question a defect report actually asks.

`--verify` runs the tests and the gates **before** building, so a failure costs
nothing and leaves no folder behind. When it is not run, `BUILD-INFO.txt` says
so in those words; an omitted line would read as "nothing to report" when it
means nobody checked.

## 4. Standing rules

| | |
|---|---|
| **R1** | A UI change is done when it has been asserted in `tools/ui-verify` against the running binary — not when a test passes. |
| **R2** | No source file over 1,500 lines. `check-file-size.sh` enforces it. When a file approaches the limit, find the seam; do not raise the limit. |
| **R3** | Salvaged code is re-verified, never assumed. |
| **R4** | The gates apply from every commit, not at fold-in. |
| **R5** | The documentation is the logic: a reader should be able to reconstruct the program's behaviour from it. See §5 for the form that takes. |
| **R6** | Nothing in `FEATURES.md`'s shipped column regresses. |
| **R7** | `egui-shell` never learns what a PDF is. Add an extension point, not an exception. |
| **R8** | A capability's presence is expressed by registering its command. No `#[cfg(feature)]` in the ribbon; an item naming an unregistered command is dropped, with a `CapabilityAbsent` skip reason. |
| **R9** | No placeholders. An unavailable capability renders nothing. Greying is reserved for *temporarily* unavailable, and is always explained on hover. |

Two rules inherited from the engine:

**Fuzzy, never sneaky.** Applied content renders exactly as saved content will
render — no badge, tint, flag or dashed outline drawn into the page view to mark
pdfcer's own uncertainty. Disclosure lives off-canvas: status line, results
panel, report, properties field. Pre-commit affordances (snap indicators, hover
highlights, rubber-bands, selection handles) are the cursor and are welcome.
The test: would a screenshot of the editing canvas differ from a screenshot of
the same document saved and reopened?

**Never write a bare "dimension".** **ce dimensions** are the ones pdfcer
authors; **pdf dimensions** are CAD-exported page content pdfcer reads and must
not silently alter. They have opposite properties. This applies in code,
comments, commits and specifications.

### 4.1 Registering a command carries six obligations

R8 makes command registration the only way the GUI learns a capability exists,
so registration is the seam where a new feature either arrives whole or arrives
as a button that does nothing. Six things must be true, and each fails loudly.
The first five check that the registration is internally consistent; the sixth
is the only one that asks whether the command *does* anything.

1. **The registry size.** `shell::commands::ledger` asserts a literal
   `registry().len()`, conditioned on the `signing` feature. Beside it, the
   icon-coverage test asserts the identity `named + refused == total` — a
   property, not two more literals that can drift apart.
2. **The ribbon shape.** `shell::manifest`'s shape test asserts the ordinary-tab
   count, the contextual-tab count, and the total group count across all tabs.
3. **Removal from `PLANNED`.** A command named in `shell::manifest::registers::PLANNED`
   is struck from it when it is registered; `DIRECTED` and `PLANNED` are
   asserted disjoint.
4. **Regenerating the RON.** `shell/ron/built_in.ron` is generated. Rewrite it
   with `cargo test -p pdfcer-gui rewrite_built_in_ron -- --ignored`; the
   round-trip test fails until you do.
5. **A `KNOWN` entry** in `shell::commands::tests::KNOWN` for any new
   `enabled_when` condition name, walked by the predicate test.
6. **Reachability.** The command must be named by a literal arm of
   `PdfcerApp::dispatch_command`, claimed by one of its guard arms, or listed in
   `shell::commands::reach::SCAFFOLDED` with a written reason.
   `shell::commands::reach` parses the dispatcher and asserts this. It is what
   stops a fully-registered, fully-consistent command from being inert.

## 5. Documentation and comment standard

The whole standard in one sentence: **write what the program is, never what it
was.**

Git holds the history. A reader — human or agent — opening a file wants the
current contract in the fewest words that carry it, and every sentence about a
past state is a sentence they must read and discard.

### 5.1 Rust

Follow rustdoc convention (RFC 1574).

**Module headers (`//!`)** — one sentence on what the module owns, then the
contract: the invariants it holds, what callers must not do, how it fits its
parents and siblings. Then non-obvious rationale, only where the code cannot
say it itself. Use `#` headings only when there are two or more real sections.

**Item docs (`///`)** — first line is one sentence, ends with a period, third
person: *"Returns the page rect in document points."* Blank line, then detail.
`# Errors`, `# Panics`, `# Safety`, `# Examples` where they apply. Never
restate the signature.

**Inline (`//`)** — only where the *why* is not obvious from the code. Never
narrate the *what*.

**Delete on sight:**

- Dates, commit hashes, version stamps, "measured on <date>".
- Tracking identifiers: request numbers, pass numbers, review-finding letters,
  session names, request filenames.
- Decorative emphasis markers and the shouting attached to them.
- Narrative: *"was split out when…"*, *"used to say…"*, *"this is the eighth
  instance…"*, *"the previous sentence was wrong because…"*, *"shipped on X
  saying Y"*.
- Quotations of the operator, of other documents' prose, or of past findings.
- Justifications for a file split that cite line counts or gate pressure.
- Restatement of the code on the next line.
- **Line numbers in citations of the engine.** Cite `pdfcer-core`,
  `pdfcer-render` and `pdfcer-print` by symbol — a function, a type, a method,
  a doc heading — never by `file.rs:NNNN`. The engine is a branch dependency
  (§1), so its source is split and edited under this repository with no command
  run here, and a line citation drifts downward until it names unrelated code or
  no code at all. `EditSession::set_text_run` survives a rewrite of the engine's
  file layout; `edit.rs:4211` does not — and a rotted citation is
  indistinguishable from a good one at the moment a reader checks it. A line
  citation of a file *inside* this repository is fine; it is the moving external
  pin that makes the engine different.

**Keep, always:**

- Invariants, contracts, and who owns a mutation.
- Units and coordinate spaces.
- Gotchas that still bite, stated as a present-tense rule with its mechanism:
  *"`RichText::strong()` borrows the active-widget foreground, which this theme
  fills with the accent — it is unreadable on a panel. Use an explicit colour."*
- Why a non-obvious choice is the right one.
- Cross-references to the module that owns a rule, so it is stated once.

A design decision is worth keeping when knowing it changes what a reader would
write next. It is not worth keeping because it was hard-won.

**Guard rail.** A comment pass must change comments only. Verify it
mechanically — strip all comments from the old and new revisions of every
touched file and diff the remainder — rather than by reading the diff.

### 5.2 Markdown

- Open with one sentence saying what the document is for and who reads it.
- Present tense, current state. No revision stacks, no dated addenda, no
  superseded measurements kept "for the reasoning".
- Give the **command** that produces a number rather than the number, wherever
  the number will move. A figure written in prose beside the thing it counts is
  a claim that decays.
- Tables for tabular data only. `check-doc-markup.py` rejects a row with more
  cells than its header, which is how an unescaped `|` silently truncates a row.
- Delete any document whose entire subject is a past event. Git has it.

### 5.3 Where the history goes instead

| Kind of fact | Where it lives |
|---|---|
| why this code is shaped this way | the doc comment, as a present-tense rule |
| what changed, and when | the commit message |
| what the operator asked for | `OPERATOR_REQUESTS.md` |
| what the engine owes us | `ENGINE_BACKLOG.md` |
| a lesson about a tool or ecosystem | `D:/dev/rag/` |
| a lesson about a real-world PDF | `C:/personal_rag/pdf/` |
| a session's narrative | nowhere. It is not a durable artefact. |

## 6. Licensing

pdfcer-gui is **MIT** (`LICENSE`), which covers everything in this repository
including the icon set — the operator's own art, recorded in
`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md`.

It does not cover everything `pdfcer-gui.exe` contains. The binary statically
links `pdfcer-core` and `pdfcer-render`, which embed third-party font faces and
data tables, so this program redistributes work whose licences require their
notices to travel with it. Two surfaces carry them, and they are not redundant:

| Surface | Carries | Reached by |
|---|---|---|
| `THIRD_PARTY_LICENSES.md`, copied into every build | every licence **text**, in full | anyone who opens the package folder |
| **File ▸ pdfcer ▸ About pdfcer** | the **attribution** — who made it, what it is, on what terms, whether pdfcer changed it | anyone who runs it |

`THIRD_PARTY_LICENSES.md` is generated — `cargo about generate about.hbs -o
THIRD_PARTY_LICENSES.md` — from this workspace's real `Cargo.lock`. Never edit
it by hand. The `accepted` list in `about.toml` is permissive-only, so a
copyleft dependency entering the workspace makes generation fail and name the
crate; that failure is the licence audit.

`tools/gates/check-shipped-assets.py` enforces the arrangement: a
`PROVENANCE.md` beside every redistributed asset directory, a citation in both
notice surfaces unless the asset is our own work, the notice present in the
packager's payload, and the generated file not stale against its template.

**The OCR model weights are CC-BY-SA-4.0.** Shipping them unmodified is
distribution of a verbatim work in a collection and leaves pdfcer's own licence
untouched. **Modifying them — fine-tuning, retraining, quantizing, or
converting them to another runtime's format — creates Adapted Material, and the
result must be released under CC-BY-SA-4.0 or a compatible licence.** That is
an engineering constraint; see `crates/pdfcer-gui/src/text/about.rs`.

## 7. Version control

`.gitattributes` disables CRLF normalization for PDF and PNG fixtures.
`core.autocrlf` is true globally on this machine, and normalization lands **in
the index at add time**, not only at checkout. A PDF's cross-reference table
stores absolute byte offsets, so a normalized fixture is a corrupt one.

## 8. Reference documents

| Document | What it is |
|---|---|
| `FEATURES.md` | What works today. A row is ticked only when an operator can reach it in a real build. Authoritative for status. |
| `PROJECT_PLAN.md` | The charter: topology, build stages, the fold-in procedure, risks, open questions. |
| `GUI_ROADMAP.md` | The phased plan and the open scope questions. |
| `RIBBON_IA.md` | Where every command lives, and why. The spec for the shell; do not improvise around it. |
| `MODES_AND_PANELS.md` | The Read/Review/Edit selector and the flexible panel system. |
| `SHELL_FRAMEWORK.md` | `egui-shell`: the manifest, the extension points, and R7. |
| `RIBBON_SCALING.md` | How the ribbon narrows, derived by driving Word. |
| `DEFECTS.md` | Known defects and the standing rules they earned. |
| `DESIGNS.md` | Designs argued and not yet built. A section is deleted when its design has been built **and driven**. |
| `OPERATOR_REQUESTS.md` | The standing backlog. Only the operator closes a row. |
| `ENGINE_BACKLOG.md` | Every capability the engine has that this shell does not reach, and the decision on each. |
| `EDITABLE_SURFACES.md` | Every verb `pdfcer-core` implements, and where the operator reaches it. |
| `NO_SURFACE.md` | Shipped behaviour with a hard-coded value and no control. |
| `UNIT_SURFACES.md` | Every surface that shows or accepts a length. |
| `UI_TOOLKIT_PINS.md` | Which egui the shell is built against, and why. Read by `check-ui-toolkit-drift.sh`. |
| `BENCHMARK.md` | Measured rendering performance on a real CAD site plan. |
| `ACROBAT_DEFAULTS.md` | What Acrobat actually authors, measured from its own preference hive. |
| `mockups/*.html` | Interactive mockups of the ribbon, the window and the modes. |
