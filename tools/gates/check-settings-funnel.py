#!/usr/bin/env python3
"""check-settings-funnel.py — every operator setting must reach the engine.

★★★ WHY THIS EXISTS, AND IT IS A DEFECT REPORT
==============================================

`app/settings.rs` opens with the sentence this gate enforces:

    **A setting is a promise.** Storing one that does nothing breaks it
    silently, which is worse than not offering the choice.

On 2026-09-16 that module was breaking six of them. Four `RenderOptions`
members — `page_blend_space_source`, `overprint_zero_tint_scope`,
`spot_colorant_device_model`, `mesh_patch_padding` — were offered in
Settings › Colour, written to the settings file, read back on the next
launch, and never chained onto the builder. Two `EditSession` members —
`widget_tab_tail`, `tab_row_tolerance` — were offered in Settings › Forms
and never handed to a session, so an operator who widened the row tolerance
got the engine's default in the very tab ring he had set it for.

WHY NOTHING CAUGHT IT
=====================

There was already a guard, and it is a good one:
`app::settings::tests::no_call_site_builds_its_own_options` parses every
`.rs` with `syn` and forbids constructing `ExtractOptions`, `RenderOptions`
or `SaveOptions` anywhere but the funnel. It is keyed on **constructors**,
so it proves nobody bypasses the funnel — and says nothing whatever about
whether the funnel assigns every field. Bypass and omission are different
defects and only one had an instrument.

The other thing that should have caught it was prose, and prose is why it
survived:

* the module header said `Settings` is *"thirteen operator choices"* when
  the struct carried twenty-three;
* `render_options`'s heading said *"Five settings"* while the chain
  assigned six and four more existed;
* `open_session` said *"`quad_point_order`, and nothing else, because that
  is the only member of `Settings` with a session-level setter"*, backed by
  a measured setter count of fifteen. There were twenty-nine, and three of
  them were configuration-shaped.

Three counts, all stated as measurements, all wrong, and **no count in a
comment can fail a build**. That is the whole argument for this file
existing: the guarantee has to be derived from the engine's own source on
every commit, not restated by hand in a doc comment that the engine can
grow past while nobody is looking.

WHAT IT MEASURES
================

For every `pub` field of `pdfcer_core::settings::Settings` **as of the
revision `Cargo.lock` pins**, at least one non-comment read of that field
must exist in `crates/pdfcer-gui/src/` outside the three zones that only
*handle* a setting without honouring it:

* `dialogs/settings/` — renders and edits the **draft**. A field written
  here and nowhere else is a control with no consequence, which is exactly
  the defect.
* `app/settings_window.rs` — decides what pressing a button in that window
  means. It compares and stores; it does not render or edit with the value.
* `app/prefs/` — the on-disk format. Persistence is how a broken promise
  survives a restart, not how it is kept.

Test code is excluded too: `*_tests.rs` files whole, and every
`#[cfg(test)]` item excised by brace-matching. A field consumed only by a
test is a field the program does not use — a guarded capability is not a
capability of the file.

WHY THE PIN AND NOT THE WORKING TREE
====================================

`tools/engine_path.locate()` resolves `D:/Dev/pdfcer`, which is engine
`main` and moves under us — the engine session ships several commits a
day. **The operator can only set a setting that exists in the crate we
compile against**, so the oracle is `Cargo.lock`'s revision, read through
one `git show` against the engine clone. This is the same resolver shape
`check-engine-api-drift.py` uses, and for the same reason.

EXIT CONTRACT
=============

`run-all.sh` classifies purely by exit code.

* **0 — PASS.** Every pinned field has a consumer, or a written exemption.
* **1 — FAIL.** A field is offered and discarded; or an exemption names a
  field that does not exist, or a field that *is* consumed (a stale
  exemption is a lie about the code and gets the same red as a defect); or
  the field list came back empty, which means the parse went blind and is
  a failure rather than a vacuous pass.
* **2 — SKIP, and only for one condition:** the pinned revision is not in
  the engine clone. That is a real state on a fresh machine and is not
  evidence about the funnel. A missing engine *directory* is a FAIL, via
  `engine_path.require`, because that was the condition that once made
  `check-verb-coverage` print "PASS: all 0 uncalled verb(s)" having
  examined nothing.

★ A skip is not red, so a gate can stop running without anyone noticing.
If this gate appears in `run-all.sh`'s SKIPPED block on a machine that has
the engine, that is a finding.

THE EXEMPTION
=============

    // settings-funnel-exempt: <field> — <reason of at least 40 characters>

One token and a written reason, never a list of classifications inside the
rule: a rule with a taxonomy in it is where the next exception goes. The
reason is not validated for truth — nothing can do that — but it is
validated for **existence and length**, so the cost of exempting a field
is writing a sentence a reviewer can disagree with.

There are no exemptions today. Every one of the twenty-three fields has a
consumer, and five of them reach the engine at their own call site rather
than through the funnel — `separations` through `app/actions/`,
`style_policy` through the text-style verb, `acrobat_trust_store` through
the signature panel, `theme` through `app/frame.rs`,
`parallel_epsilon_degrees` through `canvas/measure/`. That is fine and
deliberate: those are not members of any options struct, so there is
nothing for the funnel to assign.
"""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

import engine_path  # noqa: E402

#: The lockfile whose pinned revision is the oracle.
LOCK = pathlib.Path("Cargo.lock")

#: The shell's source tree — the population searched for consumers.
# BOTH GUI CRATES. A settings field consumed in `pdfcer-gui-base` — `acrobat`
# is reached with the operator's configured viewer path — must count as
# consumed, and a root list that names only `pdfcer-gui` would not say so: it
# would find no read and report the setting orphaned, or, for a field whose only
# read moved out, report a clean tree as broken.
SRCS = [
    pathlib.Path("crates/pdfcer-gui/src"),
    pathlib.Path("crates/pdfcer-gui-base/src"),
]


def sources():
    """`(path, rel)` for every `.rs` under either root.

    `rel` is relative to **that file's own root**, because every prefix in
    `HANDLERS_NOT_CONSUMERS` and every exemption marker is written against a
    crate-relative path. Making it relative to a single root would silently
    stop those prefixes matching.
    """
    for src in SRCS:
        if not src.is_dir():
            continue
        for path in sorted(src.rglob("*.rs")):
            yield path, path.relative_to(src).as_posix()


#: Paths that handle a setting without honouring it. Relative to a crate source
#: as a prefix, so a directory entry covers everything under it.
HANDLERS_NOT_CONSUMERS = (
    "dialogs/settings/",
    "app/settings_window.rs",
    "app/prefs/",
)

#: The exemption marker, and the minimum length of the reason after it.
EXEMPT = "settings-funnel-exempt:"
MIN_REASON = 40

_LINE_COMMENT = re.compile(r"//.*$", re.MULTILINE)
_BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)
_FIELD = re.compile(r"^\s*pub ([a-z][a-z0-9_]*)\s*:")

#: Literals, blanked before brace matching. Raw strings first, because a
#: `r#"..."#` body may contain an unpaired quote that would derail the plain
#: string pattern.
_RAW_STR = re.compile(r'r(#*)".*?"\1', re.DOTALL)
_STR = re.compile(r'"(?:\\.|[^"\\])*"', re.DOTALL)
_CHAR = re.compile(r"'(?:\\.|[^'\\])'")

#: The attribute whose item is not part of the program.
ATTR = "#[cfg(test)]"


def pinned_revision() -> str | None:
    """The engine commit `Cargo.lock` pins, from the source URL's fragment."""
    if not LOCK.is_file():
        return None
    for line in LOCK.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith('source = "git+file:') and "#" in line:
            return line.rsplit("#", 1)[1].rstrip('"')
    return None


def settings_source(repo: pathlib.Path, rev: str, crate: str) -> str:
    """The pinned `settings` module's text, through one `git show`.

    Both module spellings are tried because which one the engine uses is not
    this gate's business — only that exactly one of them answers.

    Raises `LookupError` when git will not produce the blob, which the caller
    turns into a SKIP.
    """
    candidates = [
        f"crates/{crate}/src/settings/mod.rs",
        f"crates/{crate}/src/settings.rs",
    ]
    errors = []
    for path in candidates:
        try:
            p = subprocess.run(
                ["git", "-C", str(repo), "show", f"{rev}:{path}"],
                capture_output=True,
            )
        except OSError as exc:  # git not on PATH
            raise LookupError(f"git could not be run: {exc}") from exc
        if p.returncode == 0:
            return p.stdout.decode("utf-8", errors="replace")
        errors.append(p.stderr.decode("utf-8", errors="replace").strip())
    raise LookupError("; ".join(errors) or "no settings module at that revision")


def settings_fields(text: str) -> list[str]:
    """The `pub` field names of `pub struct Settings`, in declaration order."""
    found: list[str] = []
    inside = False
    for line in text.splitlines():
        if not inside:
            if re.match(r"^pub struct Settings\s*\{", line):
                inside = True
            continue
        if line.startswith("}"):
            break
        m = _FIELD.match(line)
        if m:
            found.append(m.group(1))
    return found


def _masked(text: str) -> str:
    """`text` with every literal's contents blanked, offsets preserved.

    Brace matching has to run over something where a `"{"` inside a test's
    format string cannot be mistaken for a block opener. Blanking rather than
    deleting keeps every index valid, so the slice indices found here can be
    applied to the original.
    """
    out = list(text)
    for pattern in (_RAW_STR, _STR, _CHAR):
        for m in pattern.finditer("".join(out)):
            for i in range(m.start(), m.end()):
                if out[i] != "\n":
                    out[i] = " "
    return "".join(out)


def without_test_items(text: str) -> str:
    """`text` with every `#[cfg(test)]` item removed.

    ★ An earlier version cut from the first `#[cfg(test)]` to end of file, and
    that is wrong in this crate: `canvas/measure/mod.rs` declares a test-only
    `const` at line 259 and the real consumer of
    `Settings::parallel_epsilon_degrees` sits at line 845, so the whole file
    below the attribute vanished and a working setting was reported discarded.
    A false FAIL is cheap; the same bug pointing the other way is a false PASS,
    which is the failure mode this gate exists to prevent.

    Each attribute is followed to the first `{` or `;` in the literal-masked
    text: a `{` is brace-matched to its close, a `;` ends the item there.
    """
    masked = _masked(text)
    cuts: list[tuple[int, int]] = []
    at = 0
    while True:
        at = masked.find(ATTR, at)
        if at == -1:
            break
        brace = masked.find("{", at + len(ATTR))
        semi = masked.find(";", at + len(ATTR))
        if brace == -1 and semi == -1:
            cuts.append((at, len(masked)))
            break
        if semi != -1 and (brace == -1 or semi < brace):
            cuts.append((at, semi + 1))
            at = semi + 1
            continue
        depth = 0
        end = len(masked)
        for i in range(brace, len(masked)):
            if masked[i] == "{":
                depth += 1
            elif masked[i] == "}":
                depth -= 1
                if depth == 0:
                    end = i + 1
                    break
        cuts.append((at, end))
        at = end

    if not cuts:
        return text
    kept = []
    previous = 0
    for begin, finish in cuts:
        kept.append(text[previous:begin])
        previous = finish
    kept.append(text[previous:])
    return "".join(kept)


def executable_text(text: str) -> str:
    """`text` with comments and test items removed.

    Comments go because this codebase argues about option types *in prose* —
    `ExtractOptions::default()` appears in a dozen doc comments — and a gate
    that counted prose would score a discarded field consumed by the very
    sentence claiming it was applied.

    ★ Comments come out FIRST, and the order is load-bearing:
    `app/settings.rs` carries a doc comment saying a neighbouring check *"skips
    `#[cfg(test)]` modules"*, and treating that sentence as an attribute
    removed the funnel itself. A file that argues about a mechanism contains
    the tokens that mechanism is keyed on.
    """
    return without_test_items(
        _LINE_COMMENT.sub("", _BLOCK_COMMENT.sub("", text))
    )


def is_handler(rel: str) -> bool:
    return any(rel == z or rel.startswith(z) for z in HANDLERS_NOT_CONSUMERS)


def consumers(fields: list[str]) -> dict[str, list[str]]:
    """`{field: [relative path, …]}` for every real read of each field.

    The pattern anchors a **word boundary** after the name. Without it
    `.acrobat_trust_store` matches `prefs.acrobat_trust_store_path`, a
    different field on a different struct — a substring match that would score
    one field consumed on the evidence of another.
    """
    patterns = {f: re.compile(r"\.%s\b" % re.escape(f)) for f in fields}
    found: dict[str, list[str]] = {f: [] for f in fields}
    for path, rel in sources():
        if is_handler(rel) or rel.endswith("_tests.rs"):
            continue
        body = executable_text(path.read_text(encoding="utf-8", errors="replace"))
        for field, pattern in patterns.items():
            if pattern.search(body):
                found[field].append(rel)
    return found


def exemptions() -> tuple[dict[str, tuple[str, str]], list[str]]:
    """`({field: (path, reason)}, [complaints about malformed markers])`."""
    claimed: dict[str, tuple[str, str]] = {}
    bad: list[str] = []
    for path, rel in sources():
        text = path.read_text(encoding="utf-8", errors="replace")
        if EXEMPT not in text:
            continue
        for number, line in enumerate(text.splitlines(), 1):
            if EXEMPT not in line:
                continue
            tail = line.split(EXEMPT, 1)[1].strip()
            m = re.match(r"([a-z][a-z0-9_]*)\s*[\u2014:-]\s*(.+)$", tail)
            if not m:
                bad.append(
                    f"{rel}:{number}: marker is not "
                    f"`{EXEMPT} <field> \u2014 <reason>`: {tail!r}"
                )
                continue
            field, reason = m.group(1), m.group(2).strip()
            if len(reason) < MIN_REASON:
                bad.append(
                    f"{rel}:{number}: the reason for `{field}` is "
                    f"{len(reason)} characters; {MIN_REASON} is the minimum, "
                    f"because an exemption costs a sentence a reviewer can "
                    f"disagree with"
                )
                continue
            claimed[field] = (f"{rel}:{number}", reason)
    return claimed, bad


def skip(message: str) -> int:
    print("SKIP: " + message)
    return 2


def run(explain: bool) -> int:
    repo = engine_path.require("check-settings-funnel")
    crate = engine_path.crate_name("core")
    rev = pinned_revision()
    if rev is None:
        print(
            f"FAIL: {LOCK} names no `git+file:` source with a revision "
            f"fragment, so there is no pinned engine to measure against. "
            f"Reported as a failure and never as a skip: a gate that cannot "
            f"find its oracle must be loud."
        )
        return 1

    try:
        text = settings_source(repo, rev, crate)
    except LookupError as exc:
        return skip(
            f"the pinned revision {rev[:8]} is not in the engine clone at "
            f"{repo} ({exc}). Nothing was measured. Run `git -C {repo} fetch` "
            f"if this machine is expected to have it."
        )

    fields = settings_fields(text)
    if not fields:
        print(
            f"FAIL: parsed zero `pub` fields from `Settings` in "
            f"crates/{crate}/src/settings at {rev[:8]}. The struct was renamed, "
            f"moved, or its fields are no longer `pub <name>:` — either way "
            f"this gate went blind, and a check that cannot fail is not "
            f"evidence."
        )
        return 1

    found = consumers(fields)
    claimed, bad = exemptions()

    orphaned = [f for f in fields if not found[f] and f not in claimed]
    unknown = sorted(f for f in claimed if f not in fields)
    stale = [f for f in fields if found[f] and f in claimed]

    if explain:
        width = max(len(f) for f in fields)
        for field in fields:
            where = ", ".join(found[field]) or "(no consumer)"
            print(f"  {field:<{width}}  {where}")
        print()

    if bad:
        print(f"FAIL: {len(bad)} malformed exemption marker(s).")
        for line in bad:
            print("  " + line)
        return 1

    if unknown:
        print(
            f"FAIL: {len(unknown)} exemption(s) name a field that is not in "
            f"`Settings` at {rev[:8]}."
        )
        for field in unknown:
            print(f"  {claimed[field][0]}: `{field}` \u2014 no such field")
        print(
            "  An exemption for a field that no longer exists is a lie about "
            "the code that reads as coverage. Delete it."
        )
        return 1

    if stale:
        print(f"FAIL: {len(stale)} exemption(s) are no longer needed.")
        for field in stale:
            print(
                f"  {claimed[field][0]}: `{field}` is exempted but IS "
                f"consumed, at {found[field][0]}"
            )
        print(
            "  Delete the exemption when the cause is removed: a carve-out "
            "with no subject is the next author's permission slip."
        )
        return 1

    if orphaned:
        print(
            f"FAIL: {len(orphaned)} of {len(fields)} operator setting(s) are "
            f"offered and discarded."
        )
        for field in orphaned:
            print(
                f"  `Settings::{field}` has no non-comment read anywhere in "
                f"{' or '.join(x.as_posix() for x in SRCS)} outside "
                f"{', '.join(HANDLERS_NOT_CONSUMERS)}."
            )
        print(
            "\n  A setting is a promise. Storing one that does nothing breaks "
            "it silently,\n  which is worse than not offering the choice. "
            "Either hand it to the engine\n  \u2014 a `with_*` builder in "
            "`app/settings.rs`, a setter in `open_session`, or a\n  read at "
            "the call site that acts on it \u2014 or write\n"
            f"\n      // {EXEMPT} <field> \u2014 <why, in {MIN_REASON}+ "
            f"characters>\n"
        )
        return 1

    note = f", {len(claimed)} exempted" if claimed else ""
    print(
        f"PASS: all {len(fields)} `Settings` field(s) at {rev[:8]} reach the "
        f"engine{note}."
    )
    return 0


def self_test() -> int:
    """Exercise the parser and the exemption reader on fixtures.

    ★ This proves the mechanism, not the codebase. The gate is falsified
    against the real tree separately, by deleting a `with_*` line and
    confirming red — a self-test that passes on a broken repository is the
    thing this project keeps getting caught by.
    """
    failures = []

    struct = "\n".join(
        [
            "pub struct Other {",
            "    pub decoy: u8,",
            "}",
            "",
            "#[derive(Clone)]",
            "pub struct Settings {",
            "    /// pub commented: u8,",
            "    pub alpha: Alpha,",
            "    pub beta: f64,",
            "    non_pub: u8,",
            "}",
            "",
            "pub struct After {",
            "    pub gamma: u8,",
            "}",
        ]
    )
    got = settings_fields(struct)
    if got != ["alpha", "beta"]:
        failures.append(f"field parse: expected ['alpha', 'beta'], got {got}")

    if settings_fields("pub struct Nothing {}") != []:
        failures.append("field parse: a missing Settings must yield []")

    body = "\n".join(
        [
            "//! The neighbouring check skips `#[cfg(test)]` modules.",
            "let a = x.applied;",
            "// let b = x.in_a_comment;",
            "/* let c = x.in_a_block; */",
            "#[cfg(test)]",
            "const PROBE: u8 = 1;",
            "let e = x.below_a_test_const;",
            "#[cfg(test)]",
            "mod tests {",
            "    fn t() { let d = x.only_in_tests; }",
            '    fn u() { panic!("a { brace in a string"); }',
            "}",
            "let f = x.below_a_test_module;",
        ]
    )
    kept = executable_text(body)
    for name, want in (
        ("applied", True),
        ("in_a_comment", False),
        ("in_a_block", False),
        ("only_in_tests", False),
        # A test item mid-file must not swallow the program below it: a
        # test-only `const` at `canvas/measure/mod.rs:259` did exactly
        # that and hid the real consumer 586 lines further down.
        ("below_a_test_const", True),
        # And an unpaired `{` inside a test string must not derail the
        # brace match, which is why literals are blanked first.
        ("below_a_test_module", True),
    ):
        if (name in kept) is not want:
            failures.append(
                f"comment/test strip: `{name}` should "
                f"{'survive' if want else 'be removed'}"
            )

    # The word boundary. `.acrobat_trust_store` must not be satisfied by
    # `.acrobat_trust_store_path`, which is the substring match that made the
    # first survey of this defect report a false consumer.
    pattern = re.compile(r"\.%s\b" % re.escape("acrobat_trust_store"))
    if pattern.search("prefs.acrobat_trust_store_path"):
        failures.append("word boundary: a longer field satisfied a shorter one")
    if not pattern.search("s.acrobat_trust_store)"):
        failures.append("word boundary: a real read was rejected")

    # Exemption parsing, over the three dash spellings and the length floor.
    for tail, want_field, want_ok in (
        ("alpha \u2014 " + "x" * MIN_REASON, "alpha", True),
        ("alpha - " + "x" * MIN_REASON, "alpha", True),
        ("alpha: " + "x" * MIN_REASON, "alpha", True),
        ("alpha \u2014 too short", "alpha", False),
        ("no dash at all", None, False),
    ):
        m = re.match(r"([a-z][a-z0-9_]*)\s*[\u2014:-]\s*(.+)$", tail)
        ok = bool(m) and len(m.group(2).strip()) >= MIN_REASON
        if ok is not want_ok:
            failures.append(f"exemption parse: {tail!r} accepted={ok}")
        if m and want_field and m.group(1) != want_field:
            failures.append(f"exemption parse: {tail!r} named {m.group(1)}")

    if failures:
        for line in failures:
            print("SELF-TEST FAIL: " + line)
        return 1
    print("SELF-TEST PASS: field parse, comment and test strip, word boundary "
          "and exemption reader all behave.")
    return 0


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()
    return run(explain="--explain" in sys.argv[1:])


if __name__ == "__main__":
    sys.exit(main())
