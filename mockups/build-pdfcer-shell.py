"""Build `mockups/pdfcer-shell.html` — the revised shell-layout mockup.

=====================================================================
WHAT THIS IS
=====================================================================

A **mockup generator**. It inlines two glyph tables into
`pdfcer-shell-template.html` and writes one self-contained HTML file the
operator can double-click. Nothing here is product code. No Rust gate reads it.
Nothing under `mockups/` is compiled, linked, or shipped.

The artifact exists to support ONE decision: whether, and in what order, to
schedule the four shell-layout proposals analysed in `SHELL_LAYOUT_PROPOSAL.md`.
It is a picture of that analysis, not a specification of the build.

=====================================================================
★★★ WHAT IT IS NOT, AND THE FAILURE MODE THAT MAKES THIS WORTH SAYING
=====================================================================

**A mock that drifts from the product is how a deleted feature comes back as a
picture of itself.**

That sentence is not a slogan; it is a description of something that already
happened in the delivery this file is derived from. The reviewer's
`board-shell.html` drew a `Thin lines` toggle in View ▸ Display, with new art,
marked "DRAWN FOR THIS MOCK". There is no `thin_lines` field on `RenderOptions`
and no such command anywhere in the tree — the capability was deleted six weeks
earlier, with evidence. `GLYPH_ADOPTION.md` caught it only because somebody
re-read the mock against the code. Had nobody, the next reader would have found
a picture of the feature and had no way to know the picture was the only place
it still existed.

So three rules govern this generator, and they are the reason it is a generator
at all rather than a hand-kept HTML file:

1. **The glyph inventory is READ OUT OF THE PRODUCT**, never typed. See
   `extract-shipped-glyphs.py`. "This glyph is in the app today" is true by
   construction. Delete an asset from `crates/` and the next rebuild loses it
   here too.
2. **Every control drawn in the mock is either shipped, or marked.** The mock's
   legend drawer (the "What changed, and why" strip along the bottom) carries
   one line per divergence with its state — SHIPPED / PROPOSED / REJECTED /
   NOTED — and the reason. A silent difference is the defect; a stated one is
   the deliverable.
3. **A rejected proposal is drawn as rejected, not omitted.** The one-line tool
   strip is not in this mock, and the legend says in one line why not. Omitting
   it silently would leave the operator comparing two pictures and guessing.

=====================================================================
INPUTS
=====================================================================

    glyphs/shipped.json    One key per .svg in
                           crates/pdfcer-gui/src/icons/assets/, plus one per
                           live alias key that draws another role's art per
                           mapping.rs — of which there are currently NONE, the
                           last three having been given purpose-drawn assets on
                           2026-09-04. No count in this prose on purpose; the
                           script prints the real one every run.
                           REGENERATE with extract-shipped-glyphs.py.

    glyphs/proposed.json   28 keys — the review's 65 proposed glyphs, minus the
                           36 adopted on 2026-09-04, minus `thin-lines` (dead
                           art, see above). Each carries the verdict and the
                           one-line reason it was not taken.

    pdfcer-shell-template.html
                           The mock itself, with two placeholders:
                           {{SPRITE}} and {{GLYPHS}}.

=====================================================================
WHAT IT DOES
=====================================================================

1. Turns every glyph in both tables into an SVG `<symbol id="g-NAME">` inside
   one hidden sprite. The mock references them with `<use href="#g-NAME">`.
   Inline, because the artifact must open from disk with no network and no
   sibling files.
2. Substitutes the sprite for `{{SPRITE}}` and a JSON inventory for
   `{{GLYPHS}}`. The inventory feeds the mock's glyph-sheet overlay, which
   shows each key at 32 px and at the 16 px it ships at, labelled with its
   origin — and, for a proposed one, the reason it was not adopted.
3. **Reports every glyph key the template references that neither table
   holds**, and exits non-zero. This is the check that would have caught
   `thin-lines` in the other direction: a control drawn with art that does not
   exist anywhere renders an empty box, and an empty box in a mockup reads as a
   rendering fault rather than as a missing decision.

=====================================================================
RUN
=====================================================================

    python mockups/build-pdfcer-shell.py

    # and, when the product's icon set has changed:
    python mockups/extract-shipped-glyphs.py && python mockups/build-pdfcer-shell.py

Exit codes: 0 written; 1 a referenced glyph is missing, or an input is absent.

=====================================================================
RELATIONSHIP TO THE REVIEWER'S DELIVERY
=====================================================================

`D:/Dev/FeatureRequests/pdfcer-gui/mockups/` holds the outside reviewer's
original — `board-shell.html`, its template, and its build script. That tree is
the delivery of record and is READ-ONLY from here. This directory is a fork of
it, and the divergences are exactly the ones the legend drawer enumerates.
Do not edit the FeatureRequests copy to "keep them in sync"; the two are
supposed to differ, and the difference is the deliverable.
"""

import io
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).parent

# The delivery's geometry contract, applied to every `p` path in proposed.json.
# Identical to `crates/pdfcer-gui/src/icons/assets`'s own contract, which is why
# the two sets sit side by side on the glyph sheet without reading as two sets.
STD = (
    'stroke="currentColor" stroke-width="2.5" '
    'stroke-linecap="round" stroke-linejoin="round" fill="none"'
)


def region(tpl: str, start: str) -> str:
    """The text of one top-level `const NAME = [ … ];` literal, or ''.

    Region-scoped scanning rather than one regex over the whole file, because a
    single loose pattern over 1,200 lines produces false positives that are
    worse than no check: the reviewer's build script matched `g('` inside
    `seg('`, and matched prose out of unrelated data tuples, so its "missing
    glyph" list was noise a reader learns to ignore. A check nobody reads is a
    check that is not there.
    """
    i = tpl.find(start)
    if i < 0:
        return ""
    open_at = tpl.find("[", i)
    if open_at < 0:
        return ""
    # Bracket-count to the matching close, over a copy whose quoted strings are
    # blanked out. Anchoring on a literal "\n];" instead looked simpler and was
    # wrong: RAIL and TOOLROWS close on the same line they open, so the region
    # ran on for two hundred lines and every lowercase word in the rendering
    # code came back as a missing glyph.
    masked = re.sub(r"'[^'\n]*'", lambda m: "'" + " " * (len(m.group(0)) - 2) + "'", tpl)
    depth = 0
    for k in range(open_at, len(masked)):
        if masked[k] == "[":
            depth += 1
        elif masked[k] == "]":
            depth -= 1
            if depth == 0:
                return tpl[i : k + 1]
    return tpl[i:]


def scan_references(tpl: str) -> set[str]:
    """Every glyph key the SHELL draws — not the glyph sheet, which lists all.

    Four reference shapes, each scanned where it actually occurs:

    1. `<use href="#g-NAME">` — literal markup. Unambiguous.
    2. `g('NAME')` — the render helper. Guarded with a left boundary so that
       `seg('preset', …)` and `tg('armed', …)` do not match.
    3. The `TABS` ribbon literal, where a glyph is the tuple's 2nd element.
       `modes:[…]` is stripped first: its members ('read', 'review', 'edit')
       are lowercase words in tuple position and would otherwise be reported
       as missing glyphs forever.
    4. The `RAIL`, `TOOLROWS` and `OBJECTS` literals, whose glyph slots are the
       only all-lowercase-and-dashes members of their tuples.
    """
    used = set(re.findall(r"#g-([a-z0-9-]+)", tpl))
    used |= set(re.findall(r"(?<![A-Za-z0-9_$])g\('([a-z0-9-]+)'", tpl))

    tabs = re.sub(r"modes:\[[^\]]*\]", "", region(tpl, "const TABS ="))
    used |= set(re.findall(r",'([a-z0-9-]+)'\s*(?=[\],])", tabs))

    for name in ("const RAIL =", "const TOOLROWS ="):
        used |= set(re.findall(r"'([a-z0-9-]+)'", region(tpl, name)))
    used |= set(re.findall(r"\[\d+,'([a-z0-9-]+)'", region(tpl, "const OBJECTS =")))
    return used


def load(name: str) -> dict:
    path = HERE / "glyphs" / name
    if not path.is_file():
        raise SystemExit(f"missing input: {path}  (run extract-shipped-glyphs.py?)")
    return json.load(io.open(path, encoding="utf-8"))


def main() -> int:
    shipped = load("shipped.json")
    proposed = load("proposed.json")
    proposed.pop("_contract", None)

    symbols: list[str] = []
    inventory: list[dict] = []

    for name, v in sorted(shipped.items()):
        symbols.append(
            f'<symbol id="g-{name}" viewBox="{v["vb"]}">{v["body"]}</symbol>'
        )
        entry = {"name": name, "origin": "shipped", "src": v.get("src", "")}
        # An alias key draws art named for another role. Say so on the sheet:
        # it is the only way a reader sees that `properties` is `document.svg`.
        if v.get("src", "") != f"{name}.svg":
            entry["alias"] = True
        inventory.append(entry)

    for name, v in sorted(proposed.items()):
        body = "".join(f'<path d="{d}" {STD}/>' for d in v.get("p", [])) + v.get("x", "")
        # A proposed key that ALSO exists in `shipped` is one of the three
        # aliases: the key is live and draws another role's asset, and the
        # review drew new art for it. Both are worth seeing, so the proposed
        # drawing gets its own symbol id — `g-p-<name>` — and the SHELL keeps
        # drawing the shipped art under `g-<name>`. A shared id would have let
        # a drawing the product does not have leak into the mock's chrome,
        # silently, which is the whole failure mode this file guards against.
        collides = name in shipped
        sym = f"g-p-{name}" if collides else f"g-{name}"
        symbols.append(f'<symbol id="{sym}" viewBox="0 0 48 48">{body}</symbol>')
        entry = {
            "name": name,
            "origin": "proposed",
            "sym": sym,
            "verdict": v.get("verdict", ""),
            "why": v.get("why", ""),
        }
        if collides:
            entry["shadowed"] = shipped[name].get("src", "")
        inventory.append(entry)

    sprite = (
        '<svg xmlns="http://www.w3.org/2000/svg" '
        'style="position:absolute;width:0;height:0;overflow:hidden" '
        'aria-hidden="true">' + "".join(symbols) + "</svg>"
    )

    tpl_path = HERE / "pdfcer-shell-template.html"
    if not tpl_path.is_file():
        raise SystemExit(f"missing template: {tpl_path}")
    tpl = io.open(tpl_path, encoding="utf-8").read()

    used = scan_references(tpl)
    have = set(shipped) | set(proposed)
    missing = sorted(u for u in used if u not in have)
    shell_only_proposed = sorted(u for u in used if u in proposed and u not in shipped)

    print(
        f"glyphs: shipped {len(shipped)}  proposed {len(proposed)}  "
        f"referenced by the shell {len(used)}"
    )
    if shell_only_proposed:
        # ★★★ THE CHECK THAT MATTERS. A key in `proposed` is art for something
        # the product does NOT have. If the shell references one, the mock is
        # drawing a control that does not exist — which is precisely the
        # `thin-lines` failure. Loud, and non-fatal only because a reviewer
        # may deliberately want to picture a proposal.
        print(
            "WARN the shell draws art for un-adopted proposals: "
            f"{shell_only_proposed}  — is each one deliberate, and stated in the legend?"
        )
    if missing:
        print(f"ERROR referenced but undrawable: {missing}", file=sys.stderr)
        return 1

    out = tpl.replace("{{SPRITE}}", sprite).replace(
        "{{GLYPHS}}", json.dumps(sorted(inventory, key=lambda x: x["name"]))
    )
    target = HERE / "pdfcer-shell.html"
    io.open(target, "w", encoding="utf-8").write(out)
    print(f"wrote {target.name}  {len(out) // 1024} KB")
    return smoke_test(target)


# Chrome, wherever this machine keeps it. Best-effort: the build succeeds
# without it, but then nothing has looked at the page.
CHROMES = [
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
]


def smoke_test(target: Path) -> int:
    """Render the artifact headlessly and assert the page actually drew.

    ★ WHY THIS IS IN THE BUILD AND NOT LEFT TO THE AUTHOR'S DISCIPLINE.

    The whole mock is drawn by one `<script>`. One unescaped apostrophe inside
    one legend string throws a SyntaxError, the script never runs, and what is
    written to disk is a valid HTML file containing an EMPTY WINDOW. Nothing
    about the build fails: the glyph tables resolve, the placeholders
    substitute, the file is 227 KB, and the exit code is 0. That happened once
    while this file was being written, and the only thing that noticed was a
    screenshot coming back 17 KB instead of 200 KB.

    So the build renders the page and asserts three markers are in the DOM —
    one per independently-rendered region. Each is a string only the running
    script can produce; finding them proves the script ran to completion, not
    merely that the file parsed.
    """
    exe = next((c for c in CHROMES if Path(c).is_file()), None)
    if exe is None:
        print("SKIP smoke test: Chrome not found — nothing has looked at the page")
        return 0
    import subprocess

    try:
        dom = subprocess.run(
            [
                exe, "--headless", "--disable-gpu", "--no-sandbox",
                "--virtual-time-budget=4000", "--dump-dom",
                target.resolve().as_uri(),
            ],
            # bytes, decoded explicitly: the DOM carries the mock's typographic
            # dashes and ★s, and Windows' cp1252 default cannot decode them.
            capture_output=True, timeout=90,
        ).stdout.decode("utf-8", "replace")
    except Exception as exc:                                  # noqa: BLE001
        print(f"SKIP smoke test: {exc}")
        return 0

    # ★ STRUCTURAL, not prose. The legend marker was the heading's own wording
    # until 2026-09-04, when the heading was rewritten for O123 and the build
    # failed on an artifact that had rendered perfectly — a check that fires on
    # a legitimate edit teaches the next person to ignore it. A class name the
    # render loop emits proves the same thing (the script reached the legend
    # and produced rows) and survives every rewording of what those rows say.
    markers = {
        "the ribbon rendered": 'class="rb',
        "the dock rendered": "Paint order",
        "the legend rendered": 'class="li ',
    }
    bad = [name for name, needle in markers.items() if needle not in dom]
    if bad:
        print(f"ERROR smoke test: {', '.join(bad)} — the page script did not run "
              f"to completion (a JS syntax error writes a valid, EMPTY file)",
              file=sys.stderr)
        return 1
    print(f"smoke test OK — {len(markers)} regions rendered ({len(dom) // 1024} KB of DOM)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
