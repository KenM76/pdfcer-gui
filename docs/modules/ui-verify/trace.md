# `ui-verify/trace`

Parse the application's `key=value` diagnostic trace.

## The format, and why it is worth a parser

With its diagnostic environment variable set, the application writes one
line per event to **stderr**:

```text
pdfcer-diag start argv1=Some("a.pdf") viewport=ViewportBuilder { .. }
pdfcer-diag canvas tool=None rect=[[240.0 96.0] - [1560.0 968.0]] zoom=1.5 sel=0
pdfcer-diag vector-click screen=[820.0 514.0] canvas=[580.0 418.0] hits=1 newsel=1
pdfcer-diag delete-objects n=1 indices=[7]
```

Those four are the **old** binary's. The one this project is building
speaks a wider dialect, and the three lines below are what
`PROJECT_PLAN.md` §4.3 asked it for:

```text
pdfcer-diag canvas rect=[[16.0 22.8] - [1084.0 777.2]] zoom=0.4480 page=0 pages=1 off=[0.0 0.0]
pdfcer-diag ui-rect name=canvas-viewport rect=[[8.0 8.0] - [1092.0 792.0]]
pdfcer-diag objects n=28 page=0 paths=13 text=15 images=0 forms=0
```

Note what does **not** appear in the second set: `sel=`. A field the
application omits because it has nothing to say is not a field with the
value zero, and this parser preserves that distinction rather than
flattening it — [`TraceLine::get_usize`] returns `None` for an absent field
and for a literal `None`, and every caller is expected to have an answer
for that case that is not "assume 0". The same rule governs failure: this
application reports a failure as a *different event*
(`canvas-unavailable reason=…`, `objects-unavailable page=… reason=…`),
never as the success event with a field missing. So a missing field on a
success line is a **parse bug in this module**, not a zero, and it should
be chased here rather than absorbed at a call site.

stderr because it needs no path, no open handle, no failure mode of its own,
and redirects with `2>`. `key=value` because the consumer is a grep, an LLM
or this parser — never a person reading a log.

A regex would nearly work and then fail on the interesting lines. The values
are Rust `Debug` output, so they contain spaces inside brackets
(`rect=[[0.0 0.0] - [16.0 9.0]]`), inside parentheses (`tool=Some(Obj)`) and
inside quotes (`argv1=Some("my file.pdf")`). Splitting on whitespace gives
`rect=[[0.0` and four fragments; splitting on `=` gives nonsense the moment
a value contains one. So the splitter below tracks bracket depth and string
state, and a key boundary only counts at depth zero.

That is not hypothetical fussiness: `rect=` and `zoom=` are exactly the two
fields [`crate::coords`] needs to convert a document point into a click, and
`rect=` is the one whose value contains spaces.

## What this module deliberately does not do

It does not interpret. `hits=1` is a string `"1"` until someone asks for it
as a number, and an event name is a string, not an enum. The vocabulary —
which event carries the selection count, which field holds it — lives in
[`crate::profile`], because it differs between the binary this project is
building and the binary it is replacing, and a parser that hard-coded one
of them could not be pointed at the other.

## Item notes

### `fn unwrap_debug_option`

`None` mapping to `None` is the point: a field whose value is literally the
string `None` is the application saying "there is nothing here", and a
caller asking for it as a number wants the same answer as if the field had
been missing.

### `fn parse_fields`

The algorithm, and why it is not a regex or a `split_whitespace`:

1. Walk the characters, maintaining bracket depth over `[`, `(`, `{` and a
   flag for being inside a double-quoted string (honouring `\` escapes).
2. A **key boundary** is an identifier starting at depth zero, outside a
   string, at the start of the tail or immediately after a space, and
   followed by `=`.
3. Each value runs from just after its `=` to just before the next key
   boundary, or to end of line for the last one.

Step 2's depth condition is the whole reason this is a function rather than
a one-liner: `rect=[[0.0 0.0] - [16.0 9.0]] zoom=1.5` contains no key
boundary inside the brackets, and a splitter that did not know that would
produce a field called `0` and lose `rect` entirely.

### `fn last_reads_a_fossil_where_last_after_reports_silence`

A surface publishes a census, something else happens, and the surface
**stops publishing**. `last` still answers with the stale census — which
is what made two checks report a working panel as broken — and
`last_after`, anchored on the cause, correctly answers `None`.
