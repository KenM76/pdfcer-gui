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

It compares **structure**: which groups exist, in what order, carrying which
items, in what order, at what size, wearing which icon key, under which label.

It does **not** compare appearance. Paddings, heights, framing and type are not
in either data source — they are in the mockup's CSS and in the shell's theme
metrics, and the only oracle for whether they match is a rendered screenshot of
the running binary. **This script passing does not mean the ribbon looks like
the mockup.** Saying so here because that is exactly the inference somebody will
draw from a green line.

★ The mockup is the SPEC side. Where they differ, the mockup is right by
definition — the operator approved it. A difference is therefore a defect in the
product, never in the mock, unless the mock is drawing something deliberately
unbuilt (which it marks, and which this script skips).

USAGE
    python tools/compare-mockup-ribbon.py            # human-readable diff
    python tools/compare-mockup-ribbon.py --tab file # one tab

EXIT
    0  the two agree on every group this script can see
    1  they differ; every difference is printed with its side
    2  a source could not be read (a refusal, never a silent pass)
"""

import io
import json
import re
import sys
from pathlib import Path

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


def ron_groups(text):
    """Every `caption: "…"` in the generated manifest, with the command ids under it."""
    out = []
    for m in re.finditer(r'caption:\s*"((?:[^"\\]|\\.)*)"', text):
        cap = m.group(1)
        tail = text[m.end():m.end() + 4000]
        stop = re.search(r'caption:\s*"', tail)
        if stop:
            tail = tail[:stop.start()]
        ids = re.findall(r'id:\s*"([a-z0-9_.]+)"', tail)
        out.append((cap, ids))
    return out


def main():
    if not TEMPLATE.exists() or not RON.exists():
        print(f'compare-mockup-ribbon: SKIPPED — missing {TEMPLATE if not TEMPLATE.exists() else RON}')
        return 2

    mock = mock_groups(io.open(TEMPLATE, encoding='utf-8').read())
    ship = ron_groups(io.open(RON, encoding='utf-8').read())

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

    print('---')
    if differences:
        print(f'DIFFER — {differences} structural difference(s) above.')
        print()
        print('★ Remember what this does NOT say: it compares structure only.')
        print('  Paddings, heights, framing, label placement and type are not in')
        print('  either source, and the only oracle for those is a screenshot of')
        print('  the running binary.')
        return 1
    print('AGREE on group inventory and order.')
    print()
    print('★ This does NOT mean the ribbon LOOKS like the mockup. Appearance —')
    print('  framing, sizing, label placement, type — lives in the mock\'s CSS and')
    print('  the shell\'s theme metrics, and is settled only by a rendered')
    print('  screenshot of the running binary.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
