# `ui-verify/checks/new_document_size`

`new_document_sizes_the_page` — picking A3 landscape must produce an A3
landscape page, not an A4 portrait one with an A3 label.

# What this is about


`EditSession::set_media_box` shipped, so the implementation is: parse the
one template, resize page 0, rewrite the whole file, **re-parse it**, and
hand the result over as an ordinary new document with nothing pending and
nothing undoable.

# The failure this exists for, and why the unit tests cannot see it

There are four places the size can be lost, and each looks correct from the
one next to it:

| where | what it looks like |
|---|---|
| the radio does not reach `sheet_pt` | the summary line reads right and the page is portrait |
| `Action::NewSized` carries the wrong pair | the dialog is right and the document is transposed |
| `set_media_box` is called and the rewrite drops it | the request traces perfectly and the page is 595 × 842 |
| the re-parse reads a different page | everything traces perfectly and the canvas shows A4 |

The workspace's unit tests cover the first half of that chain and cannot
reach the second: `sheet_pt` is pinned in `dialogs::new_document::tests`,
and what happens after `to_full_bytes` is a property of a real parse of
real bytes. So this check reads `result_w` / `result_h` from the
`new-document-sized` trace line, which `app::blank` emits **after** the
re-parse from `pages[0].media_box` — the page as a reader of the file will
see it, not the rectangle that was asked for.

# What it drives

File tab → New from template… → open the size list → **A3** → **Landscape**
→ Create. Then it asserts the resulting page is 420 × 297 mm within a
millimetre, in that order — the wrong way round is the transposition defect
and is reported as such rather than as "the wrong size".

A3 rather than A4, deliberately: A4 is what the dialog opens on and what
`file.new` makes, so a build that ignored every control would still produce
it. A3 landscape differs from the default in **both** dimensions and in
orientation.

# No fixture, and that is the point

It launches with no `--pdf`. `file.new_from_template` is registered with no
`enabled_when` because an operator with nothing open is the one it exists
for, and a check that needed a document open would be testing a state the
command is least likely to be used from.

## Item notes

### `const A3_INDEX`

The list is A0, A1, A2, A3, … — largest-first, which is the engine's own
ordering and is deliberate: this operator's sheets are A1 and A3, and
burying them under A4 would make the common case the hard one. Spelled out
as a constant with the ordering stated so that a reordering of `ALL` fails
here with a readable reason rather than silently checking A2.

### `const TOLERANCE_MM`

A3 is 420 × 297 mm exactly by definition, and the points are derived from
that, so the round trip should be exact to well under a tenth of a
millimetre. One millimetre is a tolerance that cannot mask a real defect —
the smallest wrong answer available is A4 (210 × 297), which is 123 mm out.
