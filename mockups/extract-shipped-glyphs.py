"""Regenerate `glyphs/shipped.json` from the ICONS THE PRODUCT ACTUALLY SHIPS.

=====================================================================
WHAT THIS IS, AND — MORE IMPORTANTLY — WHAT IT IS NOT
=====================================================================

This is a **mockup generator**. Nothing under `mockups/` is product code,
nothing here is compiled, and no Rust gate reads it. `pdfcer-shell.html` is a
picture of a proposal, not a specification of the build.

★★★ THE FAILURE MODE THIS HEADER EXISTS TO PREVENT.

A mock that drifts from the product is how a deleted feature comes back as a
picture of itself. That is not hypothetical here — it already happened once, in
the delivery this tree is derived from. `GLYPH_ADOPTION.md` records it:

    "`thin-lines` was art for a command deleted six weeks ago. Verified: there
     is no such field on `RenderOptions` and no such command anywhere in the
     tree. … It was carried into the proposal sheet marked 'DRAWN FOR THIS
     MOCK', which is exactly how a deleted feature comes back: as a picture of
     itself in a document nobody re-checked against the code."

So the glyph inventory in this mock is **not** a hand-kept list. It is read out
of `crates/pdfcer-gui/src/icons/assets/*.svg` every time this script runs, which
means the mock's claim "this glyph is in the app today" is true by construction
rather than by somebody remembering to update a table. If an asset is deleted
from the product, the next rebuild of the mock loses it too, and the build
script reports it as a missing reference rather than drawing a stale picture.

=====================================================================
INPUT — read-only, and deliberately so
=====================================================================

    D:/Dev/pdfcer-gui/crates/pdfcer-gui/src/icons/assets/*.svg

Every `.svg` in that directory, however many there are — deliberately not a
number in this prose, matching `assets/PROVENANCE.md`'s own ruling that a count
nothing verifies is a count that rots (it carried "79 files" for a week while
the real figure was 85). The script NEVER writes into `crates/`. It opens
each file, reads its `viewBox`, strips the XML comments (which are the house
style's rationale blocks — often longer than the art, and always irrelevant to a
browser), and keeps the drawing elements verbatim.

Comments are stripped rather than kept because they are the single largest term
in the file size: `lock.svg` is 28 lines of ruling above 3 lines of path data.
Inlining every one of them would roughly triple the artifact for no visible
effect.

=====================================================================
ALIASES — keys that draw art belonging to another key. Currently EMPTY.
=====================================================================

`crates/pdfcer-gui/src/icons/catalog/mapping.rs` may map an `Icon` variant onto
an asset named for a different role. Such a key has no file of its own, so the
directory sweep above cannot see it, and ALIASES is how it reaches the mock.

★★ THE TABLE IS EMPTY AND THAT IS THE NEWS. It held three entries until
2026-09-04:

    properties     -> document.svg
    insert-pages   -> upload.svg     (shared with import-form-data)
    set-scale      -> convert.svg

Those three were live icon KEYS with no art of their own, and they were in
NEITHER of `GLYPH_ADOPTION.md`'s lists — 36 adopted + 26 deferred accounts for
62 of the 65 glyphs the review delivered, and these were the missing three. This
mock is what found them: it draws shipped art beside proposed art, and the
adoption count did not add up. All three have since been given purpose-drawn
assets, `document.svg` and `convert.svg` were kept as orphan variants so the art
stays under test, and the borrowing they recorded no longer exists.

⇒ An alias entry is therefore a live claim about `mapping.rs`, not a
convenience. Add one only when a key really has no file; the loop below prints a
NOTE and skips any entry whose key has acquired its own asset, and that NOTE is
an instruction to delete the line rather than a warning to live with.

=====================================================================
⚠ THE ROOT `<svg>` ATTRIBUTES ARE DROPPED, AND ONE OF THEM MATTERS
=====================================================================

`body_of` keeps the drawing elements and throws the opening tag away, so the
root's `fill="none"` goes with it. Every asset in the set relies on that
attribute: the paths carry `stroke="currentColor"` and say nothing about fill,
so a `<path>` lifted out of its root and dropped into a page that does not set
`fill` renders as a SOLID BLACK SILHOUETTE of its own outline. A padlock becomes
a black slab, a tick becomes a filled wedge, and every glyph in the set looks
like a different (and much worse) glyph.

`build-pdfcer-shell.py`'s artifact is fine because the mock's own stylesheet
sets `fill:none` on `.g`. ★ The trap is the throwaway preview page — the one
somebody writes to compare four glyphs side by side while judging a new drawing.
Written without `fill:none`, it renders every icon filled, and the judgement it
is used for is made about pictures the product does not draw. That happened on
2026-09-04 while these five were being reviewed, and the only thing that caught
it was the sheet looking obviously wrong.

⇒ If you consume `shipped.json` anywhere, set `fill:none` (or re-add
`fill="none"` to whatever element wraps the body). It is not kept in the JSON
because the mock's CSS is the right place for a set-wide constant, and because
`redact.svg`'s one deliberately-filled rect carries its own `fill` attribute and
therefore survives either way — which is the tell: if your preview shows the
redaction bar solid and everything else outlined, it is correct.

=====================================================================
OUTPUT
=====================================================================

    glyphs/shipped.json  — { "<key>": { "vb": "0 0 48 48",
                                        "body": "<path …/>…",
                                        "src": "lock.svg" } }

`src` is carried so the glyph sheet can say which asset a key draws, which is the
only way an alias is visible to a reader of the mock.

=====================================================================
RUN
=====================================================================

    python mockups/extract-shipped-glyphs.py     # rewrites glyphs/shipped.json
    python mockups/build-pdfcer-shell.py         # then rebuilds the artifact

Run the first only when the product's icon set changes; run the second after any
edit to `pdfcer-shell-template.html`.
"""

import io
import json
import re
from pathlib import Path

HERE = Path(__file__).parent
ASSETS = HERE.parent / "crates" / "pdfcer-gui" / "src" / "icons" / "assets"

# Key -> asset file, for keys whose art belongs to another key. See the header.
# Empty since 2026-09-04, when the last three keys were given art of their own.
# See the header: an entry here is a claim about `mapping.rs`, and a claim with
# nothing behind it is the shape of defect this whole generator exists to catch.
ALIASES: dict[str, str] = {}

COMMENT = re.compile(r"<!--.*?-->", re.S)
OPEN_TAG = re.compile(r"<svg\b[^>]*>", re.I)
CLOSE_TAG = re.compile(r"</svg\s*>", re.I)
VIEWBOX = re.compile(r'viewBox\s*=\s*"([^"]+)"', re.I)


def body_of(path: Path) -> tuple[str, str]:
    """Return (viewBox, inner markup) for one asset, comments removed."""
    raw = io.open(path, encoding="utf-8").read()
    open_tag = OPEN_TAG.search(raw)
    if not open_tag:
        raise SystemExit(f"{path.name}: no <svg> element")
    vb_match = VIEWBOX.search(open_tag.group(0))
    # Every asset in the set is on the 48 grid; falling back rather than failing
    # keeps a hand-added asset from breaking the whole build over one attribute.
    view_box = vb_match.group(1) if vb_match else "0 0 48 48"
    inner = raw[open_tag.end():]
    close = CLOSE_TAG.search(inner)
    if close:
        inner = inner[: close.start()]
    inner = COMMENT.sub("", inner)
    # Collapse the whitespace the house style indents with. Harmless in SVG and
    # it is most of what is left once the comments are gone.
    return view_box, re.sub(r"\s+", " ", inner).strip()


def main() -> None:
    if not ASSETS.is_dir():
        raise SystemExit(f"asset directory not found: {ASSETS}")
    out: dict[str, dict[str, str]] = {}
    for svg in sorted(ASSETS.glob("*.svg")):
        view_box, inner = body_of(svg)
        out[svg.stem] = {"vb": view_box, "body": inner, "src": svg.name}
    real = len(out)

    for key, filename in sorted(ALIASES.items()):
        if key in out:
            print(f"NOTE alias '{key}' now has its own asset — drop it from ALIASES")
            continue
        source = ASSETS / filename
        if not source.is_file():
            raise SystemExit(f"alias '{key}' points at missing asset {filename}")
        view_box, inner = body_of(source)
        out[key] = {"vb": view_box, "body": inner, "src": filename}

    target = HERE / "glyphs" / "shipped.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    io.open(target, "w", encoding="utf-8").write(
        json.dumps(dict(sorted(out.items())), indent=1, ensure_ascii=False)
    )
    print(
        f"wrote {target.name}: {real} shipped assets "
        f"+ {len(out) - real} alias keys = {len(out)} drawable keys"
    )


if __name__ == "__main__":
    main()
