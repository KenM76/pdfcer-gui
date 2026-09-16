# -*- coding: utf-8 -*-
"""Read Acrobat's own form-field dialog vocabulary out of its binaries.

## Why this instrument exists

O205 asks for a table of every feature a form field should have "to match
Acrobat", and the reference-app rule says look at the reference application
rather than recall it. The obvious instrument is a camera: open Acrobat, open a
field's properties, photograph each tab. That works and it is what
`tools/word-ribbon-study.ps1` does for Word -- but it takes the real desktop,
the real mouse and the real keyboard for as long as it runs, so it can only be
done while the operator is away, and it answers about the dialogs somebody
remembered to open.

The plug-ins that implement those dialogs carry **the labels themselves**,
UTF-16LE, concatenated in the order the dialog templates reference them.
Reading them is offline, takes no input device, is repeatable, and cannot miss
a control because nobody clicked the tab it lives on.

## Two binaries, not one

`AcroForm.api` implements the panels that belong to a field: Options, Format,
Validate, Calculate, Signed, the barcode panels, and the field-kind names.
`AcrobatRes.dll` implements the panels a field **shares** with links and
annotations -- the Actions panel, its trigger list and its action list -- plus
the manual tab-order commands. A session that reads only the first concludes
Acrobat has no Actions panel, which is how a parity table acquires a hole in
the shape of its own instrument.

## What it can and cannot tell you

It gives the **vocabulary**: every label, every drop-down item, every tab name,
every warning sentence, verbatim, in Adobe's own words including the `&`
accelerator marks. That is exactly what a parity table's reference column needs.

It does **not** give the **structure**: which tab a label sits on, which field
kind shows that tab, or what is enabled when. The strings are stored as one run
per resource, so adjacency is strong evidence and nothing more. Any claim in
`FORMS_PARITY.md` about *which kind shows which tab* is therefore marked as
photographed or as unverified -- never derived from this dump.

## Usage

    python tools/acrobat-form-strings.py [--out evidence/acrobat-forms]

Writes `strings.txt` (every UTF-16 run of four or more printable characters,
unique, sorted) and `form-dialogs.txt` (only the runs carrying a form-dialog
anchor, in binary order, each labelled with the binary it came from). Prints
the file version of each source, because the vocabulary is a property of a
version. Exits 1 if any anchor matched nothing -- that means Acrobat renamed a
label and this dump is quietly narrower than it reads.
"""
import argparse
import io
import os
import re
import subprocess
import sys

NL = chr(10)

ACROBAT = r'C:\Program Files\Adobe\Acrobat DC\Acrobat'

DEFAULT_SOURCES = [
    ACROBAT + r'\plug_ins\AcroForm.api',
    ACROBAT + r'\AcrobatRes.dll',
]

# A run has to carry one of these to land in form-dialogs.txt. Chosen as labels
# that appear in no other Acrobat surface, so the filter does not drag in the
# distribution wizard or the barcode licence prose. Each one is also an
# assertion that the panel it names still exists under that name.
ANCHORS = (
    'Common Properties',            # General panel
    'It&em List',                   # Options, choice fields
    '&Multi-line',                  # Options, text field
    'Check Box &Style',             # Options, check box
    'Text Field Properties',        # the dialog titles
    'Select &format category',      # Format panel
    'Value is n&ot calculated',     # Calculate panel
    'Field value is n&ot validated',  # Validate panel
    '&Nothing happens when signed',  # Signed panel
    'Order Tabs by &Row',           # the Fields panel's tab-order commands
    'Icon top, label bottom',       # Options, push button layout
    'Add a Dropdown field',         # the Prepare Form toolbar
    '&Select Trigger:',             # Actions panel, in AcrobatRes.dll
    'Add an Action',                # Actions panel, in AcrobatRes.dll
    'Use &Row Order',               # Tab Order dialog, in AcrobatRes.dll
)


def file_version(path):
    """The file version of a binary, via PowerShell, because it dates the dump."""
    try:
        out = subprocess.check_output([
            'powershell', '-NoProfile', '-Command',
            "(Get-Item -LiteralPath '%s').VersionInfo.FileVersion" % path,
        ], stderr=subprocess.STDOUT)
        return out.decode('utf-8', 'replace').strip()
    except Exception as exc:                  # noqa: BLE001 - reported, not raised
        return 'unknown (%s)' % exc


def runs(blob):
    """Every UTF-16LE run of four or more printable ASCII characters.

    Adobe stores a dialog's labels as one resource with no separators, so a
    "run" is usually a whole panel's worth of text concatenated. That is why
    the output lines are long and why adjacency is meaningful.
    """
    for match in re.finditer(rb'(?:[\x20-\x7e]\x00){4,}', blob):
        yield match.group(0).decode('utf-16-le')


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument('--api', action='append', default=None,
                    help='a binary to read; repeatable. Default: AcroForm.api '
                         'and AcrobatRes.dll from the Acrobat DC install')
    ap.add_argument('--out', default='evidence/acrobat-forms',
                    help='directory for strings.txt and form-dialogs.txt')
    args = ap.parse_args(argv)
    sources = args.api or DEFAULT_SOURCES

    absent = [p for p in sources if not os.path.isfile(p)]
    if absent:
        for path in absent:
            sys.stderr.write('not found: %s%s' % (path, NL))
        return 2

    found = []          # (source, run) in binary order, source by source
    per_source = []     # (source, size, version, run count)
    for path in sources:
        with io.open(path, 'rb') as handle:
            blob = handle.read()
        these = list(runs(blob))
        found.extend((path, run) for run in these)
        per_source.append((path, len(blob), file_version(path), len(these)))

    unique = sorted({run for _, run in found})
    dialogs = [(src, run) for src, run in found
               if any(a in run for a in ANCHORS)]

    if not os.path.isdir(args.out):
        os.makedirs(args.out)

    lines = ['# Acrobat form-dialog vocabulary, read out of the Acrobat binaries',
             '#']
    for path, size, version, count in per_source:
        lines.append('# source:  %s' % path)
        lines.append('#          %d bytes, file version %s, %d runs'
                     % (size, version, count))
    lines += [
        '#',
        '# totals:  %d runs, %d unique, %d carrying a form-dialog anchor'
        % (len(found), len(unique), len(dialogs)),
        '#',
        '# Produced by tools/acrobat-form-strings.py. Labels are verbatim,',
        '# including & accelerators. Adjacency is evidence about a dialog, not',
        '# proof: see the module docstring.',
        '',
        '',
    ]
    header = NL.join(lines)

    with io.open(os.path.join(args.out, 'strings.txt'), 'w',
                 encoding='utf-8', newline=NL) as handle:
        handle.write(header)
        handle.write(NL.join(unique))
        handle.write(NL)

    with io.open(os.path.join(args.out, 'form-dialogs.txt'), 'w',
                 encoding='utf-8', newline=NL) as handle:
        handle.write(header)
        for src, run in dialogs:
            handle.write('>>> %d characters, from %s%s'
                         % (len(run), os.path.basename(src), NL))
            handle.write(run + NL + NL)

    unmatched = [a for a in ANCHORS if not any(a in run for _, run in found)]
    if unmatched:
        sys.stderr.write('WARNING: %d anchors matched nothing: %s%s'
                         % (len(unmatched), ', '.join(unmatched), NL))

    print('%d sources: %d runs, %d unique, %d form-dialog runs, '
          '%d anchors unmatched'
          % (len(sources), len(found), len(unique), len(dialogs),
             len(unmatched)))
    return 1 if unmatched else 0


if __name__ == '__main__':
    sys.exit(main())
