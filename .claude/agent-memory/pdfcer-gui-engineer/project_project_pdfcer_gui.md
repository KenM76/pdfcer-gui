---
name: the-project-is-pdfcer-gui-since-2026-09-03
description: Names, folders and repos after the 2026-09-03 rename, and the two things that deliberately did NOT move
metadata:
  type: project
---
<!-- old-name-exempt-file: this memory records the 2026-09-03 rename, so the old name IS its subject. -->


The product is **`pdfcer`** — *pdf create, edit, read*, pronounced
"pdf-see-er". `pdfce` is now its **pre-release code name**, not a mistake.

| | |
|---|---|
| this project | `pdfcer-gui`, at `D:\dev\pdfcer-gui`, repo `KenM76/pdfcer-gui` |
| the engine | `pdfcer`, at `D:\Dev\pdfcer`, crates `pdfcer-core` / `pdfcer-render` / `pdfcer-print`, CLI binary a bare `pdfcer` |
| frozen backups | `D:\Dev\pdfceGUI` and `D:\Dev\pdfce`, both untouched; the GitHub repo `pdfceGUI` is archived |
| trace / env | `pdfcer-diag` / `PDFCER_DIAG` |

**Why:** Ken judged `pdfce` (create, edit) a missed opportunity once reading
landed. Cloned rather than renamed in place so the old state survives.

**How to apply — two things did NOT move and must not:**

* **`D:\Dev\pdfce\crates\pdfce-gui`** — 37 citations of the OLD GUI being
  salvaged from. That crate is deleted in the engine's clone and survives only
  in the frozen tree, so the path is correct as written and breaks if renamed.
  **If `D:\Dev\pdfce` is ever removed, `SALVAGE.md` stops being checkable.**
* **`pdfce_FeatureRequests`** — the shared request channel. Both sides read it;
  renaming one side is how a channel goes silent. Move it only by agreement.

★ There is a temporary `package = "pdfce-*"` bridge in `Cargo.toml` until the
engine's `Pass 247.1`. `tools/gates/check-engine-rename-shim.sh` fails the build
when it can go. Related: [[a-rename-can-blind-an-instrument-silently]].
