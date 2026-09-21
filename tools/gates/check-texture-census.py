#!/usr/bin/env python
"""check-texture-census - every texture upload is counted, or attribution lies.

WHAT THIS PROTECTS

`crates/pdfcer-gui/src/render/pressure.rs` reads the OpenGL error flag once a
frame and tries to say which upload raised it. GL's flag carries no provenance
at all, so the module works by elimination: it blames the canvas's whole-page
raster only when that raster was the frame's ONLY upload, and otherwise reports
that it cannot say.

That elimination is worth exactly as much as the census behind it. An upload
that does not record does not merely go unseen -- it makes a frame that
uploaded two things look like a frame that uploaded one, and the one still
standing is blamed for the other's failure. The concrete case: the icon sheet
and a page raster both upload on the same frame, the icon sheet is what fails,
and an unrecorded icon sheet turns that into "the page at scale 12 ran out of
memory" -- which, once anything acts on it, takes the operator's zoom away for
a reason that had nothing to do with zoom.

So the property is COMPLETENESS, and completeness is the one property a
hand-written list cannot hold. This gate reads the source instead.

WHAT IT CHECKS

Every `ctx.load_texture(` call in `crates/pdfcer-gui/src/` must have a
`crate::render::pressure::record_raster(` or `record_other(` call within
WINDOW lines above it, in the same file.

`record_raster` is reached through `crate::render::raster::texture_from_pixels`
rather than written at its two call sites, so that function's own body is what
satisfies this gate for both the canvas and the thumbnails.

WHY "ABOVE, NOT BELOW", AND WHY A WINDOW

Above, because the record must exist before the upload is ordered: a record
written afterwards is still in the same frame and would still be taken at the
right time, but an early return or a `?` between the two would drop it, and
that failure is silent. Ordering it first makes the drop impossible.

A window rather than an exact adjacency because the house style puts several
lines of comment between the two, and requiring adjacency would be a gate that
fights the documentation standard.

WHAT IT DELIBERATELY DOES NOT CHECK

That the `Surface` named is the right one. A thumbnail recorded as
`Surface::Canvas` is a real defect and this gate cannot see it -- the surface
is a judgement about what the upload is FOR, and no pattern in the source
distinguishes them. `render::pressure`'s own unit tests pin the consequence
(a thumbnail must never be blamed); this gate pins only that something was
counted.

It also does not see `egui`'s own font atlas, which never passes through
`load_texture`. That is why `Unattributed::NoUploads` means "nothing this
crate knows about uploaded" and is documented as such.

USAGE

  python tools/gates/check-texture-census.py             check the tree
  python tools/gates/check-texture-census.py --self-test falsify the detector

EXIT

  0 PASS   every upload records
  1 FAIL   an upload does not, or the self-test failed
  2 SKIP   the crate is not in this checkout
"""

import os
import re
import sys

SRC = os.path.join("crates", "pdfcer-gui", "src")

# The upload. `ctx` is the receiver everywhere in this crate; the pattern is
# deliberately loose about the receiver name so a rename does not blind it.
UPLOAD = re.compile(r"\.load_texture\s*\(")

# Either recorder, however it is pathed. Matching on the function name rather
# than on the full path is what lets a file `use` the module and call
# `record_other(...)` bare -- which `thumbnails.rs` does for `Surface`, and
# which a path-anchored pattern would report as a violation.
RECORD = re.compile(r"\brecord_(?:raster|other)\s*\(")

# How far above an upload the record may sit. Generous because the house
# documentation standard puts a paragraph of comment between them; the icon
# cache's record carries five lines of why.
WINDOW = 14

# A line that is only a comment cannot be a call. Excluded so that a doc
# comment mentioning `load_texture` -- and several do, at length -- is not
# audited as one. This is the same trap `check-verb-coverage` hit when 25
# verbs scored "consumed" on prose.
COMMENT = re.compile(r"^\s*(//|\*|/\*)")


def audit(lines):
    """Return the 1-based line numbers of uploads with no record above them."""
    bad = []
    for i, line in enumerate(lines):
        if COMMENT.match(line) or not UPLOAD.search(line):
            continue
        lo = max(0, i - WINDOW)
        if not any(RECORD.search(l) for l in lines[lo:i]):
            bad.append(i + 1)
    return bad


def self_test():
    ok = True

    def check(name, text, expected):
        nonlocal ok
        got = audit(text.split("\n"))
        if got != expected:
            print("  self-test FAIL: " + name
                  + " expected " + str(expected) + " got " + str(got))
            ok = False
        else:
            print("  self-test ok: " + name)

    # 1. The defect this gate exists to catch: an upload with no record.
    check("a bare upload is caught",
          "fn up(ctx: &Context) {\n"
          "    ctx.load_texture(ID, image, LINEAR)\n"
          "}",
          [2])

    # 2. A recorded upload is clean -- the gate must not fire on the fix.
    check("a recorded upload passes",
          "fn up(ctx: &Context) {\n"
          "    crate::render::pressure::record_other(ctx, S::Icon, 32, 32);\n"
          "    ctx.load_texture(ID, image, LINEAR)\n"
          "}",
          [])

    # 3. A NON-violation that looks like one: the word in a doc comment. A
    #    gate that fires here would report the module header of the very file
    #    it is protecting, every run, and be turned off.
    check("a doc comment naming the call is not a call",
          "/// Uploads via `ctx.load_texture(` and records first.\n"
          "// ctx.load_texture(ID, image, LINEAR)\n"
          "fn nothing() {}",
          [])

    # 4. A NON-violation that looks like one: the record called bare, after a
    #    `use`. Path-anchoring the pattern would fail this.
    check("an unqualified record still counts",
          "use crate::render::pressure::record_raster;\n"
          "fn up(ctx: &Context) {\n"
          "    record_raster(ctx, surface, &key, w, h);\n"
          "    ctx.load_texture(ID, image, LINEAR)\n"
          "}",
          [])

    # 5. ★ The one that matters most: a record BELOW its upload is still a
    #    violation. It is the shape a well-meaning edit produces, it reads as
    #    correct, and an early return between the two loses the record with
    #    nothing to show for it.
    check("a record below the upload does not count",
          "fn up(ctx: &Context) {\n"
          "    let h = ctx.load_texture(ID, image, LINEAR);\n"
          "    crate::render::pressure::record_other(ctx, S::Icon, 32, 32);\n"
          "    h\n"
          "}",
          [2])

    # 6. ★ A record too far above does not reach. Falsifies the WINDOW itself:
    #    without a bound this gate would pass on a file with one record at the
    #    top and twenty unrecorded uploads below it.
    check("a record beyond the window does not reach",
          "record_other(ctx, S::Icon, 1, 1);\n"
          + "// filler\n" * (WINDOW + 1)
          + "ctx.load_texture(ID, image, LINEAR)",
          [WINDOW + 3])

    # 7. Two uploads, one recorded. The second must still be reported --
    #    an any-match-in-file gate would call this clean.
    check("a second unrecorded upload is caught",
          "record_other(ctx, S::Icon, 1, 1);\n"
          "ctx.load_texture(A, image, LINEAR);\n"
          + "// filler\n" * (WINDOW + 1)
          + "ctx.load_texture(B, image, LINEAR);",
          [WINDOW + 4])

    print("self-test: " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv[1:]:
        return self_test()

    if not os.path.isdir(SRC):
        print("check-texture-census: SKIPPED - " + SRC + " is not in this checkout")
        return 2

    files = []
    for root, _dirs, names in os.walk(SRC):
        for n in names:
            if n.endswith(".rs"):
                files.append(os.path.join(root, n))
    if not files:
        print("check-texture-census: SKIPPED - no Rust source under " + SRC)
        return 2

    violations = []
    uploads = 0
    for path in sorted(files):
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().split("\n")
        for i, line in enumerate(lines):
            if not COMMENT.match(line) and UPLOAD.search(line):
                uploads += 1
        for n in audit(lines):
            violations.append((path, n, lines[n - 1].strip()))

    # ★ Finding no uploads at all is not a pass. The pattern stopping matching
    # -- a `load_texture` renamed upstream, a moved source root -- looks
    # exactly like a clean tree, and this is the project's single most
    # repeated gate defect.
    if uploads == 0:
        print("check-texture-census: SKIPPED - no `.load_texture(` call found "
              "under " + SRC + "; the pattern or the source root has moved")
        return 2

    if violations:
        print("check-texture-census: FAIL - " + str(len(violations))
              + " texture upload(s) are not counted by render::pressure")
        print("")
        for path, n, text in violations:
            print("  " + path + ":" + str(n))
            print("      " + text)
        print("")
        print("  An uncounted upload does not merely go unseen. It makes a")
        print("  frame that uploaded two textures look like a frame that")
        print("  uploaded one, so `render::pressure` stops refusing to guess")
        print("  and blames whichever upload it can still see -- taking the")
        print("  operator's zoom away for a failure that was not the page's.")
        print("")
        print("  Add, immediately ABOVE the upload:")
        print("      crate::render::pressure::record_other("
              "ctx, Surface::<which>, w, h);")
        print("  or, for a page raster, route it through")
        print("      crate::render::raster::texture_from_pixels")
        print("  which records for you and makes you name the surface.")
        return 1

    print("check-texture-census: PASS - all " + str(uploads)
          + " texture uploads record with render::pressure.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
