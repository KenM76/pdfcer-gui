#!/usr/bin/env python3
"""check-region-names.py — a declared trace region must be used by something.

===========================================================================
WHY THIS GATE EXISTS
===========================================================================

`tools/ui-verify` presses controls by NAME. A widget publishes its rectangle
with

    crate::diag::ui_rect(REGION_QUALITY, response.rect);

and a driven check later asks the trace for `export-image.quality` and clicks
its centre. The name is declared once, as a `pub const`, so that the two sides
cannot drift:

    /// The region the JPEG quality field publishes.
    pub const REGION_QUALITY: &str = "export-image.quality";

★★★ NOTHING IN THE TOOLCHAIN CAN SEE A DECLARATION NOTHING USES. The constant
is `pub`, and `pub` suppresses `dead_code` — that is the whole of the reason.
`rustc` is happy, `cargo clippy -D warnings` is happy, every gate in this folder
was happy, and the crate shipped four releases with **three** such names in one
window.

★★ AND THE SYMPTOM IS A FALSE DEFECT REPORT, WHICH IS WORSE THAN SILENCE. A
driven check that looks for `export-image.pages.typed`, finds no rectangle and
reports *"the control is missing"* is describing a radio button that is drawn on
screen every single time the window opens. The next session reads that report,
goes looking for a layout bug, and finds a perfectly healthy window. ⇒ *an
absence reported by a check is first a question about the check.* This gate is
the answer to that question, asked once and for all in the build rather than
once per investigation.

---------------------------------------------------------------------------
What was actually found, on 2026-09-13
---------------------------------------------------------------------------

`dialogs/export_image.rs` declared five region names and one `region_for_*`
helper. Three of the six were published by nothing:

  * `REGION_QUALITY` — `quality_group` called `ui.add(..)` and discarded the
    response, so there was no rectangle to publish.
  * `REGION_PAGES` — `pages_group` published no union at all.
  * `region_for_scope` — never called; the three page-scope radios had no
    individual rectangles, which is exactly what O196 needs to assert *which
    radio is selected when the window opens*.

The file's sibling twenty lines above, `format_group`, did all of this
correctly, and its doc comment explained why. ⇒ *a correct example in the same
file is not a mechanism.*

===========================================================================
THE RULE, AND WHY IT IS THE WEAK ONE
===========================================================================

The property actually wanted is *"this name reaches a `ui_rect` call at
runtime"*. That is a data-flow question and no regex answers it. Three cuts
were measured before settling, and the two rejected ones are recorded here
because each looked exactly like a working instrument:

  1. **"the name is the first argument of a `ui_rect(` call in its own file"**
     → reported **73 of 159** unpublished. Wrong twice: it did not know
     `ui_rect_visible` — the clipped-publisher variant — exists, and it did not
     know a region name is routinely handed to a local helper
     (`chip(ui, pen, slot, REGION_INK, ..)`) that publishes it. Nearly every
     one of the 73 was healthy.

  2. **"the name is mentioned anywhere in its own file"**
     → reported **9 of 159**. Still wrong: declaration and publisher are
     frequently in SIBLING modules. `canvas/notepopup/mod.rs` declares
     `REGION_OPEN_DEFAULT`; `canvas/notepopup/controls.rs` publishes it.

  3. **"the name is mentioned anywhere in the workspace"**
     → reported **0**, including for the three real defects. Wrong in the
     opposite direction, and this is the interesting one: `export_text.rs`
     declares its own `REGION_PAGES` and its own `region_for_scope` and uses
     both, so `export_image.rs`'s dead twins were discharged **by a different
     module's healthy constant of the same name.** ⇒ *a check keyed on a bare
     identifier is satisfied by any namesake anywhere.* This project has now
     recorded that shape three times in three different gates.

So the rule below is scoped by IMPORT, which is the cheapest thing that is
neither of those errors: a use site counts if it is the declaring file itself,
or if it names the constant through the declaring module — `module::NAME`,
`super::NAME`, `self::NAME`, or a `use` statement that mentions both.

⚠ **What this gate cannot see, stated rather than implied.** It proves the name
is *referenced*, not that the reference reaches a `ui_rect`. A constant passed
to a helper that drops it on the floor passes here. That is a real hole and the
only thing that closes it is a driven check pressing the region — which is what
`tools/ui-verify` is for, and which this gate exists to stop from producing
false reports. The two are complements, not substitutes.

⚠ **Private constants are out of scope on purpose.** A non-`pub` `const
REGION_*` that nothing uses is already a `dead_code` warning, and this
workspace builds with `-D warnings`. Widening to private names would add a
second instrument for a class the compiler already closes, and a redundant gate
is one more thing to keep true.

===========================================================================
USAGE / EXIT CODES
===========================================================================
  tools/gates/check-region-names.py              check the workspace
  tools/gates/check-region-names.py --self-test  falsify the mechanism

  0  clean
  1  a declared region name that nothing uses
  2  SKIPPED — no source tree to scan (see run-all.sh's three-state model)
"""

import io
import os
import re
import sys

# Both roots, for `check-orphan-docs.py`'s stated reason: `tools/ui-verify/` is
# source too, and a gate that silently reads 77% of the tree prints a file count
# that looks complete. It declares no regions today — it consumes them — but
# scanning it is what makes the USE side of the rule honest, because that is
# where a name would be referenced from if it ever were.
ROOTS = ["crates", "tools"]

# `fixtures/` holds other gates' deliberately-defective trees. Scanning them
# would make two instruments each other's false positives.
SKIP_DIRS = ("target", "fixtures")

CRLF = chr(13) + chr(10)
LF = chr(10)

# A `pub const REGION_FOO: &str = "...";` declaration.
DECL_CONST = re.compile(r"^[ \t]*pub const (REGION_[A-Z0-9_]+)[ \t]*:")
# A `pub const fn region_for_foo(..)` helper. These matter more than the
# constants, not less: a helper is what gives each member of a radio GROUP its
# own rectangle, and a group's rectangle cannot answer "which one is selected".
DECL_FUNC = re.compile(r"^[ \t]*pub const fn (region_for_[a-z0-9_]+)[ \t]*\(")

# Every `use ...;` statement, flattened across line breaks. `rustfmt` wraps a
# long import list over several lines, so a line-oriented search would miss
# `use super::{\n    CommentRow, Note, REGION_BOX, ...\n};` — which is the exact
# shape `panels/comments/editor.rs` uses for five of the names here.
USE_STMT = re.compile(r"\buse\s[^;]*;", re.S)


def module_of(path: str) -> str:
    """The module name a sibling would use to reach `path`'s items.

    `a/b/mod.rs` is module `b`; `a/b/thing.rs` is module `thing`. That is Rust's
    own rule, and it is what makes `reorder::REGION_DISCLOSE_PREFIX` in
    `panels/bookmarks/mod.rs` resolve to the declaration in
    `panels/bookmarks/reorder.rs`.
    """
    base = os.path.basename(path)
    if base == "mod.rs":
        return os.path.basename(os.path.dirname(path))
    return base[:-3] if base.endswith(".rs") else base


def strip_docs_and_decls(text: str) -> str:
    """`text` with doc-comment lines and region declarations removed.

    Both exclusions are load-bearing:

    * A doc comment mentioning `[`REGION_PAGES`]` must not discharge the
      assertion. That is this project's most-repeated gate defect — *a gate
      keyed on a name is discharged by prose* — and it has now been recorded
      against four separate instruments.
    * The declaration line itself obviously mentions the name, so leaving it in
      would make every declaration self-justifying and the gate vacuous.
    """
    kept = []
    for line in text.split(LF):
        stripped = line.lstrip()
        if stripped.startswith("///") or stripped.startswith("//!"):
            continue
        if DECL_CONST.match(line) or DECL_FUNC.match(line):
            continue
        kept.append(line)
    return LF.join(kept)


def declarations(files: dict) -> list:
    """Every `(path, name)` a file in `files` declares. Sorted, for a stable
    report: an unordered failure list reads as a different failure each run."""
    found = []
    for path in sorted(files):
        for line in files[path].split(LF):
            m = DECL_CONST.match(line) or DECL_FUNC.match(line)
            if m:
                found.append((path, m.group(1)))
    return found


def is_used(name: str, decl_path: str, files: dict) -> bool:
    """Does anything reference `name` as the item `decl_path` declares?

    Four routes, and nothing else counts:

    1. The declaring file itself mentions it outside its declaration and docs.
    2. Some file writes `module::NAME`, where `module` is the declaring
       module's name.
    3. Some file writes `super::NAME` or `self::NAME`. This is loose — it does
       not verify that `super` resolves to the declaring module — and it is
       loose deliberately: resolving `super` properly needs the module tree,
       and the failure mode of being loose here is a missed defect in a file
       that names a same-named constant in its own parent, which is a shape
       this workspace does not contain.
    4. Some file's `use` statement mentions both `name` and either the module
       or `super`/`self`.
    """
    module = module_of(decl_path)
    word = re.compile(r"\b" + re.escape(name) + r"\b")

    for path, text in files.items():
        body = strip_docs_and_decls(text)
        if not word.search(body):
            continue
        if path == decl_path:
            return True
        if (module + "::" + name) in body:
            return True
        if ("super::" + name) in body or ("self::" + name) in body:
            return True
        for stmt in USE_STMT.findall(body):
            if word.search(stmt) and (module in stmt or "super" in stmt or "self" in stmt):
                return True
    return False


def scan(files: dict) -> tuple:
    """`(declared, unused)` for the given `{path: source}` map.

    A pure function of the text, so the self-test can falsify it without a
    fixture tree on disk — and so that the count it reports and the list it
    reports come from the same pass. A gate that counts in one place and lists
    in another can print `0 unused` above a non-empty list.
    """
    declared = declarations(files)
    unused = [(p, n) for p, n in declared if not is_used(n, p, files)]
    return declared, unused


def self_test() -> int:
    """Falsify every clause: the gate must FIND a dead constant, and must NOT
    report any of the four healthy shapes that the three rejected cuts each
    mistook for defects."""
    ok = True

    def check(label, files, expect):
        nonlocal ok
        _, unused = scan(files)
        got = sorted(n for _, n in unused)
        if got != sorted(expect):
            print("SELF-TEST FAIL: " + label + " — expected " + str(sorted(expect))
                  + ", got " + str(got))
            ok = False

    # 1. The defect itself: declared, documented, referenced by nobody.
    check("a dead constant", {
        "crates/g/src/dialogs/export_image.rs":
            "/// The region the JPEG quality field publishes." + LF
            + "/// A driven check that cannot find [`REGION_QUALITY`] has found a PNG." + LF
            + 'pub const REGION_QUALITY: &str = "export-image.quality";' + LF
            + "fn quality_group(ui: &mut Ui) {" + LF
            + "    ui.add(egui::DragValue::new(&mut self.quality).range(1..=100));" + LF
            + "}" + LF,
    }, ["REGION_QUALITY"])

    # 2. Published in its own file — the ordinary healthy case.
    check("published in its own file", {
        "crates/g/src/dialogs/export_image.rs":
            'pub const REGION_QUALITY: &str = "export-image.quality";' + LF
            + "    crate::diag::ui_rect(REGION_QUALITY, response.rect);" + LF,
    }, [])

    # 3. Published by a SIBLING module through a wrapped `use super::{..}`.
    #    This is `panels/comments/editor.rs`, and it is what cut 2 got wrong.
    check("published by a sibling", {
        "crates/g/src/panels/comments/mod.rs":
            'pub const REGION_BOX: &str = "comments.note_box";' + LF,
        "crates/g/src/panels/comments/editor.rs":
            "use super::{" + LF
            + "    CommentRow, Note, REGION_BOX, RowSink," + LF
            + "};" + LF
            + "    crate::diag::ui_rect_visible(REGION_BOX, response.rect, clip);" + LF,
    }, [])

    # 4. Composed at runtime from a `*_PREFIX`, reached by module path. This is
    #    `panels/bookmarks/mod.rs` reaching into `reorder.rs`, and it is the
    #    other half of what cut 1 got wrong.
    check("composed through a module path", {
        "crates/g/src/panels/bookmarks/reorder.rs":
            'pub const REGION_DISCLOSE_PREFIX: &str = "bookmarks.disclose.";' + LF,
        "crates/g/src/panels/bookmarks/mod.rs":
            '    &format!("{}{}", reorder::REGION_DISCLOSE_PREFIX, item.id.num),' + LF,
    }, [])

    # 5. ★★★ THE NAMESAKE, which is cut 3's error and the reason this gate is
    #    import-scoped at all. Two modules declare the same name; one is
    #    healthy, one is dead. A bare-identifier search over the workspace sees
    #    a use of the name and passes BOTH.
    check("a healthy namesake does not discharge a dead twin", {
        "crates/g/src/dialogs/export_image.rs":
            'pub const REGION_PAGES: &str = "export-image.pages";' + LF
            + "fn pages_group(ui: &mut Ui) {}" + LF,
        "crates/g/src/dialogs/export_text.rs":
            'pub const REGION_PAGES: &str = "export-text.pages";' + LF
            + "    crate::diag::ui_rect(REGION_PAGES, start.union(ui.cursor()));" + LF,
    }, ["REGION_PAGES"])

    # 6. A doc comment must NOT discharge the assertion. The single most
    #    repeated gate defect in this project.
    check("a doc mention is not a use", {
        "crates/g/src/dialogs/export_image.rs":
            'pub const REGION_PAGES: &str = "export-image.pages";' + LF
            + "/// The union of the three radios — see [`REGION_PAGES`]." + LF
            + "/// `REGION_PAGES` is published by `pages_group`." + LF
            + "fn pages_group(ui: &mut Ui) {}" + LF,
    }, ["REGION_PAGES"])

    # 7. A `region_for_*` helper is in scope, and a dead one is the defect that
    #    costs the most: it is what a per-radio assertion needs.
    check("a dead region_for_ helper", {
        "crates/g/src/dialogs/export_image.rs":
            "pub const fn region_for_scope(scope: PageScope) -> &'static str {" + LF
            + '    "export-image.pages.current"' + LF
            + "}" + LF,
    }, ["region_for_scope"])

    # 8. A private constant is the compiler's business, not this gate's.
    check("a private constant is out of scope", {
        "crates/g/src/dialogs/export_image.rs":
            'const REGION_QUALITY: &str = "export-image.quality";' + LF,
    }, [])

    print("self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()

    roots = [r for r in ROOTS if os.path.isdir(r)]
    if not roots:
        print("check-region-names: SKIPPED — none of " + str(ROOTS) + " exist here")
        return 2

    files = {}
    crlf_files = 0
    for root in roots:
        for base, dirs, names in os.walk(root):
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            for fname in names:
                if not fname.endswith(".rs"):
                    continue
                path = os.path.join(base, fname).replace(chr(92), "/")
                raw = io.open(path, encoding="utf-8", newline="").read()
                # CRLF, normalized — `check-orphan-docs.py` shipped one run
                # blind to 126 files for want of this, and the regexes here
                # anchor on line starts rather than ends only by luck.
                if CRLF in raw:
                    crlf_files += 1
                    raw = raw.replace(CRLF, LF)
                files[path] = raw

    if not files:
        print("check-region-names: SKIPPED — found no .rs files under " + str(roots))
        return 2

    declared, unused = scan(files)

    # A tally that CAN be zero, printed unconditionally. A runner that prints
    # only on failure is indistinguishable from a runner that never ran, and
    # this project has lost a whole sweep to exactly that.
    print("check-region-names: " + str(len(files)) + " files scanned ("
          + str(crlf_files) + " CRLF, normalized), "
          + str(len(declared)) + " region name(s) declared, "
          + str(len(unused)) + " used by nothing")

    if unused:
        print("")
        print("FAIL — declared region name(s) that nothing references. Each of these")
        print("names a rectangle that DOES NOT EXIST at runtime. The control it was")
        print("written for may well be drawn on screen; what is missing is the")
        print("`crate::diag::ui_rect(..)` call that tells `tools/ui-verify` where it")
        print("is. A driven check pressing one of these reports a missing control and")
        print("sends the next session looking for a layout bug that is not there.")
        print("")
        print("To repair: find the widget this name was written for, capture its")
        print("`Response`, and publish `response.rect` under this name — the shape")
        print("`dialogs/export_image.rs`'s `format_group` uses. If the control was")
        print("removed, delete the constant with it.")
        for path, name in unused:
            print("  " + path + "  ->  " + name)
        return 1

    print("check-region-names: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
