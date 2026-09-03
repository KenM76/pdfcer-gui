---
name: a-rename-can-blind-an-instrument-silently
description: A hard-coded external path turns a rename into a green check over an empty scan; derive the path from the manifest instead
metadata:
  type: feedback
---
<!-- old-name-exempt-file: this memory is about the rename itself, so the old name IS its subject. -->


**An instrument that hard-codes an external path goes blind when anything
renames, and it goes blind reporting success.** Derive the path from the
manifest the compiler actually uses.

**Why:** the 2026-09-03 rename to `pdfcer` broke four things at once and three
said nothing:

* `check-verb-coverage` printed **"PASS: all 0 uncalled verb(s)"** having read
  nothing — a *gate*, green, over an empty scan;
* `check-shipped-assets` SKIPPED, leaving 111 redistributed files unchecked for
  licensing;
* `package-portable.py` would have shipped a portable build with **no OCR
  models**, silently;
* `build.rs` reported no engine version — the only one whose own test caught it.

Every one was a literal like `ENGINE = Path("D:/Dev/pdfcer")` that the rename
dutifully updated to a directory nobody had created yet.

**How to apply:** `tools/engine_path.py` reads the `git = "file:///…"` URL out of
`crates/pdfcer-gui/Cargo.toml` — the same answer the compiler used — so there is
one claimant for *"where is the engine"*. Any new tool reaching outside the repo
uses it. And make the failure loud: `require()` exits rather than returning a
default, because the whole lesson is that a missing input must not read as an
empty result.

## ★★★ AND IT HAPPENED AGAIN THE SAME AFTERNOON, TO THE FALSIFICATION HARNESS

**A fifth instrument, found while cutting v0.5.0.** `ui-verify`'s
`PDFCER_LEGACY` profile is the OLD GUI — the known-defective build the checks
must be *seen to fail against*, which is the only thing that makes them
evidence. The sweep rewrote **all four** of its external names, and every one of
them belongs to the frozen build at `D:\Dev\pdfce`, in another repository,
which did not rename with us:

| field | swept to | consequence |
|---|---|---|
| `default_exe` | a path under the ENGINE repo | that repo's `Pass 247.0` had just **deleted** its only GUI crate; the path can never exist. **Loud.** |
| `diag_env` | `PDFCER_DIAG` | the old binary does not read it, so its tracing stays **off**. **Silent.** |
| `trace_prefix` | `pdfcer-diag` | it never prints that, so the trace parses **EMPTY**. **Silent.** |
| `viewport_env` | `PDFCER_DIAG_VIEWPORT` | offscreen launch never engages. **Silent.** |

**An empty trace and a build that emitted nothing are the same bytes.** The
suite would have reported *"the known-defective build does not exhibit the
defect"* — the exact inversion it exists to prevent — and that reads as the
checks being *more* trustworthy, not less.

⇒ **The generalisation, which is bigger than paths:** a rename rewrites every
string that names something **outside the repository** — sibling repo paths,
*other programs' environment variables*, their log prefixes, their binary names,
URLs, registry keys, IPC channel names. Those do not rename with you, and the
ones consumed by a **parser** fail silently rather than loudly.

**Mechanical proxy, cheap:** after any rename, grep the diff for lines that
*also* contain an absolute path, an `_ENV`/`_VAR`-shaped identifier, a URL
scheme, or a `.exe`. Each is a candidate for "this name belongs to somebody
else".

★ **And the fix could not be a grep.** `profile.rs` now carries
`old-name-exempt-file:` and two tests instead:
`legacy_profile_names_the_pre_rename_gui` (the four must carry the old stem and
must NOT carry the new one) and `current_profile_names_only_the_new_project`
(every `pdfce` in this build's own profile is followed by an `r`). Both
falsified by planting the defect. A grep can ask only one of those two
questions and gets the other backwards — which is the same asymmetry the third
corollary below names.

★ Two corollaries from the same day. **`pdfcer` contains `pdfce`**, so the
substitution is not idempotent and "did I get them all?" cannot be answered by
grepping the old name — only by the stem *not followed by* `r`. And a tripwire
must test the **real** condition: the shim gate first fired on the engine's
*directory* appearing, but the rename is two Passes and the clone comes first.
Related: [[feedback_a_check_that_cannot_fail_is_not_evidence]],
[[the-project-is-pdfcer-gui-since-2026-09-03]].
