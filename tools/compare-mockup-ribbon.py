"""Compare the mockup's ribbon against the shipped manifest, group by group.

★★★ WHY THIS EXISTS

The operator asked, in one sentence: *"is our ribbon menu now set up exactly in
the same order and with the commands moved and glyphed and labeled with exactly
the same layout as the mockup has exactly?"*

That is a **completeness question**, and this project's standing lesson is that a
completeness question needs an INSTRUMENT, not a document and not an opinion. A
reading of two files by one session answers it for that session and rots the
moment either side moves. This script answers it every time it is run.

★★ WHAT IT COMPARES, AND WHAT IT DELIBERATELY DOES NOT

It compares the **group inventory and the group order** of the ribbon: which
captioned bands exist, and in what sequence, across the fixed tabs and the
contextual ones.

★★★ THREE THINGS IT DOES NOT COMPARE, AND THE LIST IS LONGER THAN IT LOOKS

**1. Appearance.** Paddings, heights, framing, label placement and type are not
in either data source — they are in the mockup's CSS and in the shell's theme
metrics, and the only oracle for whether they match is a rendered screenshot of
the running binary. **This script passing does not mean the ribbon looks like
the mockup.** Saying so here because that is exactly the inference somebody will
draw from a green line.

**2. ITEMS — ★★★ THIS CHANGED ON 2026-09-05, LATER THE SAME DAY.** Item
**presence** and **order** ARE compared now, per group, **by icon key** — see
`icon_by_id`, which carries why the icon is the one thing both sides spell when
the label is not. Item **size** (Small / Large) and **label** are still not.

The paragraph this replaces said items were "not commensurable at that level"
and named the label/id mismatch as the reason. That was true of labels and it
was never true of icons: the mock writes `['Copy as vector','copy-as-vector']`
and the catalog writes `.with_icon("copy-as-vector")`, in plain text, in files
this script already reads. **The limitation was real and the reason given for it
was wrong**, which is why it survived being written down twice.

The first run of the new phase found **sixteen** groups whose item sequences
differ. Most are the mock naming a DIFFERENT GLYPH for the same command —
`folder` where the product draws `open`, `printer` where it draws `print`,
`scissors` where it draws `cut`, `ruler` where it draws `measure` — which is not
a cosmetic difference at all: it means the two are drawing different pictures on
the same button. That list is the measured backlog and is recorded in
`RIBBON_IA.md`'s 2026-09-05 amendment.

The old text, kept because the correction is the interesting half:

> *"It does not compare the items inside a group — not their presence, not
> their order, not their size, not their icon key, not their label."*
The two sides are not commensurable at that level and this is not a scheduling
gap: the mockup stores a **label** (`'Copy as vector'`) and the RON stores an
**id** (`edit.copy_as_vector`), and the map between them lives in
`crate::text::commands`, which is Rust that this script deliberately does not
build (see the note on `ribbon_slice` about running on an unbuilt checkout).

★ An earlier version of this docstring claimed items *were* compared —
*"carrying which items, in what order, at what size, wearing which icon key,
under which label"* — and none of the five was true. That sentence is the exact
failure mode the whole file is written against: **a completeness instrument
overstating its own coverage is worse than no instrument**, because it converts
"nobody has checked" into "it has been checked", and a green line then licenses
a claim nobody measured. Corrected 2026-09-05, and the correction found real
item-level drift the moment somebody read the two files by hand: `Copy as
vector` marked unbuilt after it shipped, `Encrypt…`/`Permissions…` the same,
`Select all` marked icon-less while carrying `select-all`, and `Export text…`
absent from the mock entirely. **Not one of those moved this script's verdict.**

**3. The rail, the QAT and the trailing strip.** Those are not the ribbon. See
`ribbon_slice`, which exists because this script used to read them and reported
three of the rail's groups as missing from an approved design.

⇒ So the honest summary of a green run is: *the same bands, in the same order,
and nothing else has been checked by anything.*

★ The mockup is the SPEC side. Where they differ, the mockup is right by
definition — the operator approved it. A difference is therefore a defect in the
product, never in the mock, unless the mock is drawing something deliberately
unbuilt (which it marks, and which this script skips).

USAGE
    python tools/compare-mockup-ribbon.py            # human-readable diff
    python tools/compare-mockup-ribbon.py --tab file # one tab

EXIT
    0  the two agree on every group and every item sequence this script can see
    1  they differ; every difference is printed with its side
    2  a source could not be read (a refusal, never a silent pass)
"""

import io
import re
import sys
from pathlib import Path

# ★★★ WITHOUT THIS, THE SCRIPT CRASHES ON THE ONE LINE THAT MATTERS MOST.
#
# Windows gives a redirected Python stdout the ANSI code page — `cp1252` on
# this machine — and every star, bullet and em-dash in the output below is
# outside it. Measured 2026-09-05: the run printed all seven differences, then
# raised `UnicodeEncodeError` on `'★'` and exited **1 from the traceback**
# rather than from the comparison.
#
# ★ That is not a cosmetic bug, for two reasons that pull in opposite
# directions and are both bad:
#
#  · The caveat it died on is *"remember what this does NOT say: it compares
#    structure only"* — the single sentence this script exists to keep in front
#    of whoever runs it. It was being swallowed on exactly the runs that print
#    differences, i.e. the runs somebody reads.
#  · A crash exit and a DIFFER exit are both 1, so a caller checking the exit
#    code cannot tell "the ribbons disagree" from "the script fell over". The
#    AGREE path prints stars too, so a *passing* run on a fresh console would
#    have exited 1 with a traceback and been read as a failure.
#
# `errors='replace'` rather than a plain reconfigure: a console that genuinely
# cannot represent a star should show a question mark, not abort the report.
# ⇒ **An instrument must not be able to fail in a way that looks like its own
# verdict.**
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(encoding='utf-8', errors='replace')
    except (AttributeError, ValueError):  # pragma: no cover - not a tty/py<3.7
        pass

ROOT = Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / 'mockups' / 'pdfcer-shell-template.html'
RON = ROOT / 'crates' / 'pdfcer-gui' / 'src' / 'shell' / 'ron' / 'built_in.ron'


def mock_groups(text):
    """Every `{cap:'…', items:[…]}` literal, in file order, as (caption, [labels])."""
    out = []
    for m in re.finditer(r"\{cap:'((?:[^'\\]|\\.)*)',\s*items:\s*\[", text):
        cap = m.group(1).replace("\\'", "'")
        # Walk to the matching bracket so nested item arrays survive.
        i = m.end() - 1
        depth = 0
        while i < len(text):
            if text[i] == '[':
                depth += 1
            elif text[i] == ']':
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = text[m.end() - 1:i + 1]
        labels = [lm.group(1).replace("\\'", "'")
                  for lm in re.finditer(r"\['((?:[^'\\]|\\.)*)'\s*,\s*'", body)]
        out.append((cap, labels))
    return out


# The top-level RON keys, at exactly four spaces of indent. `built_in.ron` is
# machine-written by `shell::ron::tests::rewrite_built_in_ron`, so its layout is
# a generator's output rather than a human's habit and anchoring on the indent
# is safe. A `tabs:` at twelve spaces is a MODE's tab-name list, which is why
# the anchor is `^    ` and not a bare word match.
_RIBBON_START = re.compile(r'^    tabs:\s*\[', re.M)
_RIBBON_END = re.compile(r'^    (?:qat|trailing|rail|keymap):', re.M)


def ribbon_slice(text):
    """The part of `built_in.ron` that IS the ribbon: `tabs` + `contextual_tabs`.

    ★★★ WHY THIS FUNCTION EXISTS, AND IT WAS A DEFECT IN THIS SCRIPT

    Until 2026-09-05 `ron_groups` scanned the WHOLE file for `caption: "…"`,
    and `built_in.ron` has captions outside the ribbon. The **rail** — the
    vertical strip down the left edge, which `MODES_AND_PANELS.md` is emphatic
    is a *panel and not a tab* — carries three captioned groups of its own:
    `Navigate`, `Select` and `Rotate`. All three were reported as *"in the
    product and not in the mockup"*, which read as three missing groups in an
    approved design and is three false accusations. A fourth difference came
    with them: the rail's `Navigate` collides by name with View ▸ Navigate, so
    the order line ended in a phantom trailing `Navigate` and the group counts
    disagreed by three.

    ⇒ **Four of the seven differences this script reported were the script
    reading something that is not a ribbon.** The instrument built to answer a
    completeness question was itself incomplete about its own subject, and the
    failure mode is the dangerous direction: it manufactured work rather than
    hiding it, so a session obeying it would have "fixed" the mockup by drawing
    three groups the ribbon does not have.

    ★ The lesson generalises past this file: **ask what the instrument SAMPLED
    before believing what it reported.** This project has recorded that finding
    against `ui-verify` at least four times, and it applies to a forty-line
    regex script exactly as it does to a driven harness.

    The slice is taken by position rather than by parsing RON because this
    script deliberately has no RON parser — it must run on a checkout with
    nothing built and no cargo available, which is what makes it usable as a
    pre-commit reading.
    """
    start = _RIBBON_START.search(text)
    if not start:
        return None
    end = _RIBBON_END.search(text, start.end())
    return text[start.start():end.start() if end else len(text)]


def ron_groups(text):
    """Every `caption: "…"` in the ribbon slice, with the command ids under it.

    `text` must already be the output of [`ribbon_slice`]. Passing the whole
    file is the defect that function's docstring describes.
    """
    out = []
    for m in re.finditer(r'caption:\s*"((?:[^"\\]|\\.)*)"', text):
        cap = m.group(1)
        tail = text[m.end():m.end() + 4000]
        stop = re.search(r'caption:\s*"', tail)
        if stop:
            tail = tail[:stop.start()]
        # ★ Only ids with a DOT. A group's own `id: "save"` sits ABOVE its
        # `caption:` line, so the slice that runs from one caption to the
        # next always ends with the NEXT group's id — which appeared in the
        # first item-level run as a phantom trailing control on every single
        # group. A command id is `family.verb` and a group id is one word,
        # so the dot is the discriminator, and it is a property of the id
        # scheme rather than a heuristic about this file's layout.
        ids = [i for i in re.findall(r'id:\s*"([a-z0-9_.]+)"', tail) if '.' in i]
        out.append((cap, ids))
    return out



CATALOG = ROOT / 'crates' / 'pdfcer-gui' / 'src' / 'shell' / 'commands' / 'catalog'


def icon_by_id():
    """`command id -> icon key` for every registered command, read from source.

    ★★★ THE BRIDGE THAT MAKES AN ITEM-LEVEL COMPARISON POSSIBLE AT ALL.

    This script's own docstring records, as a deliberate limitation, that the
    two sides *"are not commensurable at that level"* — the mockup stores a
    **label** (`'Copy as vector'`) and the RON stores an **id**
    (`edit.copy_as_vector`), and the map between them is in Rust this script
    will not build. That is true of labels and it is **not true of icons**.

    Both sides carry the icon KEY, in plain text, in a file this script can
    already read:

    ```text
    mockups/…-template.html   ['Copy as vector','copy-as-vector']
    shell/commands/catalog/…  command("edit.copy_as_vector", …).with_icon("copy-as-vector")
    ```

    So the comparison below is over **icon-key sequences per group**, which
    settles item presence and item order — the two things the four hand-found
    divergences of 2026-09-05 were about. It does not settle labels or sizes,
    and the closing note says so.

    ★★ A command with **no** icon maps to `None` and is compared as a hole.
    That is not a gap in the instrument: the mock spells the same fact
    (`['Actual size',null,{noicon:1}]`), so a control that gained or lost its
    glyph on one side and not the other is a difference this catches.

    ⚠ Read from the CATALOG rather than from a built registry, for the reason
    `ribbon_slice` gives: this script must run on a checkout with nothing built
    and no cargo available. The regex is deliberately anchored on
    `command("<id>"` — the one shape every registration in that directory uses
    — and a registration written some other way would go missing rather than be
    silently mis-mapped, because a missing id compares as `?` and prints.
    """
    out = {}
    if not CATALOG.is_dir():
        return out
    for path in sorted(CATALOG.glob('*.rs')):
        text = io.open(path, encoding='utf-8').read()
        for m in re.finditer(r'command\(\s*"([a-z0-9_.]+)"', text):
            cid = m.group(1)
            # The icon, if any, is chained onto the same expression: look no
            # further than the next `command(` so a neighbour's glyph cannot be
            # attributed to a command that has none.
            tail = text[m.end():]
            stop = re.search(r'\n\s*(?:large|icon_only|command)\(\s*"', tail)
            if stop:
                tail = tail[:stop.start()]
            icon = re.search(r'\.with_icon\(\s*"([a-z0-9-]+)"', tail)
            out[cid] = icon.group(1) if icon else None
    return out


def mock_group_icons(text):
    """Every `{cap:…, items:…}` literal as (caption, [icon key or None])."""
    out = []
    for m in re.finditer(r"\{cap:'((?:[^'\\]|\\.)*)',\s*items:\s*\[", text):
        cap = m.group(1).replace("\\'", "'")
        i = m.end() - 1
        depth = 0
        while i < len(text):
            if text[i] == '[':
                depth += 1
            elif text[i] == ']':
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = text[m.end() - 1:i + 1]
        icons = []
        # `['Label','glyph'` or `['Label',null`. One pattern, both shapes, so a
        # deliberate no-icon entry is a value rather than a silent skip.
        for im in re.finditer(r"\['(?:[^'\\]|\\.)*'\s*,\s*(?:'([a-z0-9-]+)'|null)", body):
            icons.append(im.group(1))
        out.append((cap, icons))
    return out


def compare_items(mock_items, ship_items, icons):
    """Print per-group item differences; return how many groups differ.

    Compared as **sequences**, not sets: a group whose controls are in a
    different order from the mockup's is a different group to look at, and this
    project's own IA document is explicit that within-group order is meaning
    (the arrow, the white arrow, the type tool, the hand).
    """
    differing = 0
    mock_by_cap = {}
    for cap, seq in mock_items:
        mock_by_cap.setdefault(cap, []).append(seq)
    ship_by_cap = {}
    for cap, ids in ship_items:
        ship_by_cap.setdefault(cap, []).append(ids)

    for cap, mock_seqs in mock_by_cap.items():
        if cap not in ship_by_cap:
            continue          # already reported as a structural difference
        for mock_seq, ship_ids in zip(mock_seqs, ship_by_cap[cap]):
            ship_seq = [icons.get(cid, '?') for cid in ship_ids]
            if mock_seq == ship_seq:
                continue
            differing += 1
            print(f'  · {cap!r}')
            print(f'      mockup : {" ".join(str(x) for x in mock_seq) or "(none)"}')
            print(f'      product: {" ".join(str(x) for x in ship_seq) or "(none)"}')
            unknown = [cid for cid in ship_ids if cid not in icons]
            if unknown:
                print(f'      (no registration found for: {", ".join(unknown)})')
    return differing


def main():
    if not TEMPLATE.exists() or not RON.exists():
        print(f'compare-mockup-ribbon: SKIPPED — missing {TEMPLATE if not TEMPLATE.exists() else RON}')
        return 2

    ribbon = ribbon_slice(io.open(RON, encoding='utf-8').read())
    if ribbon is None:
        # A refusal, never a silent pass. If the generator ever stops writing
        # `    tabs: [` at four spaces this script would otherwise compare the
        # mockup against nothing and print a cheerful AGREE.
        print('compare-mockup-ribbon: SKIPPED — no top-level `tabs:` in '
              f'{RON}. The RON layout changed; fix `ribbon_slice`.')
        return 2

    mock = mock_groups(io.open(TEMPLATE, encoding='utf-8').read())
    ship = ron_groups(ribbon)

    mock_caps = [c for c, _ in mock]
    ship_caps = [c for c, _ in ship]

    print(f'mockup groups : {len(mock_caps)}')
    print(f'shipped groups: {len(ship_caps)}')
    print()

    only_mock = [c for c in mock_caps if c not in ship_caps]
    only_ship = [c for c in ship_caps if c not in mock_caps]
    differences = 0

    if only_mock:
        differences += len(only_mock)
        print('IN THE MOCKUP AND NOT IN THE PRODUCT — the mockup is the spec, so these are gaps:')
        for c in only_mock:
            n = dict(mock)[c]
            print(f'  · {c!r}  ({len(n)} item(s): {", ".join(n[:6])})')
        print()

    if only_ship:
        differences += len(only_ship)
        print('IN THE PRODUCT AND NOT IN THE MOCKUP — either the mock is behind, or the group should go:')
        for c in only_ship:
            print(f'  · {c!r}')
        print()

    shared = [c for c in mock_caps if c in ship_caps]
    order_mock = [c for c in mock_caps if c in shared]
    order_ship = [c for c in ship_caps if c in shared]
    if order_mock != order_ship:
        differences += 1
        print('GROUP ORDER DIFFERS on the groups both sides have:')
        print(f'  mockup : {" · ".join(order_mock)}')
        print(f'  product: {" · ".join(order_ship)}')
        print()

    # ------------------------------------------------------------------
    # PHASE 2 — the ITEMS, by icon key. See `icon_by_id`.
    #
    # ★★ Reported separately from the structural count and, deliberately, it
    # DOES move the exit code. The whole lesson of this file's own history is
    # that a lenient instrument gets quoted as proof: until 2026-09-05 this
    # script's green line was read as "the ribbon matches the mockup" while
    # four real item-level divergences sat under it, because nothing compared
    # items and the docstring's warning was not something a script reads.
    # ------------------------------------------------------------------
    icons = icon_by_id()
    item_diffs = 0
    if icons:
        mock_icons = mock_group_icons(io.open(TEMPLATE, encoding='utf-8').read())
        # Counted first and printed second, because the heading has to be able
        # to say whether there is anything under it. Printing the heading and
        # then discovering the list is empty is the shape that leaves a report
        # asserting a difference it does not go on to name.
        import io as _io_
        buf = _io_.StringIO()
        _real = sys.stdout
        sys.stdout = buf
        try:
            item_diffs = compare_items(mock_icons, ship, icons)
        finally:
            sys.stdout = _real
        if item_diffs:
            print(f'ITEM SEQUENCES DIFFER in {item_diffs} group(s) — compared by ICON')
            print('KEY, which is the one thing both sides spell (the mock stores')
            print('labels, the RON stores ids). See `icon_by_id`.')
            print(buf.getvalue(), end='')
            print()
        differences += item_diffs
    else:
        # A refusal, never a silent pass — `ribbon_slice`'s rule.
        print(f'compare-mockup-ribbon: item comparison SKIPPED — no readable')
        print(f'  command catalog at {CATALOG}. Bands were still compared.')
        print()

    print('---')
    if differences:
        structural = differences - item_diffs
        print(f'DIFFER — {structural} structural difference(s) and '
              f'{item_diffs} item-level one(s) above.')
        print()
        print('★ Remember what this does NOT say: it compares structure only.')
        print('  Paddings, heights, framing, label placement and type are not in')
        print('  either source, and the only oracle for those is a screenshot of')
        print('  the running binary.')
        return 1
    print('AGREE on group inventory and order.')
    print()
    print('★★★ WHAT THIS DOES NOT SAY — read it, because a green line above is')
    print('    routinely quoted as more than it is:')
    print('  · NOT that the ribbon LOOKS like the mockup. Framing, sizing, label')
    print('    placement and type live in the mock\'s CSS and the shell\'s theme')
    print('    metrics, and are settled only by a screenshot of the running binary.')
    print('  · NOT that the two agree on the LABELS or the SIZES of the controls.')
    print('    Item PRESENCE and ORDER are compared since 2026-09-05, by icon key —')
    print('    see `icon_by_id` — which is what the four divergences found by hand')
    print('    that day were about. A control whose label or Small/Large size moved')
    print('    on one side only is still invisible to this script.')
    print('  · NOT anything about the rail, the QAT or the trailing strip. Those')
    print('    are not the ribbon and are deliberately not read.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
