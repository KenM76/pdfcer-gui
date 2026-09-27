# `entry` — one reader for everything typed into a value box

Operator rows O243 (units shown and accepted in any written form) and O244
(arithmetic and relative entry) are one mechanism: every value box hands its
text to `preprocess(input, current, kind)` and receives a value or a refusal.

## Why one reader with a capability filter

A box differs from another only in what it may hold. `Kind` states that:

| Kind | Arithmetic | Units | Example |
|---|---|---|---|
| `Text` | no | no | a label — returned verbatim |
| `Count` | yes, result must be whole | no | copies, a page number |
| `Number(suffixes)` | yes | no; its own suffixes (`%`, `°`) are ignored | a percentage, an angle |
| `Length(unit)` | yes | any, converted into `unit` | a coordinate, a border width |

One reader means `+10px` means the same thing in every box, and a new box gets
all of it by naming its kind.

## Reading rules

- **Lexing** normalises what people paste: curly quotes and primes (`′ ″`),
  `×`, `÷`, `−`. A tight `int/int` is a fraction literal, so `5/8"` is one
  quantity rather than a division by inches.
- **Drawing notation.** Juxtaposed quantities sum (`1m 20cm`, `4' 7 1/2"`).
  `'-` after feet joins feet and inches (`4'-7 1/2"`). A bare number after
  feet is inches. Two bare numbers are a mixed number only when the second is
  a proper fraction; `1 000` is refused rather than read as 1.
- **Dimensional arithmetic.** Values are bare or lengths (held in points).
  Length ± bare treats the bare number as the box's unit. Length × length and
  bare ÷ length are refused; length ÷ length is a bare ratio.
- **Relative entry.** A leading `+ * / × ÷`, or `-` followed by a space,
  prepends the box's current value. `-10` against a digit stays a negative
  number, because that is what it means in every other program. Appending to
  the shown number (`100+10px`) reads the same way.
- **Refusal beats guessing.** Anything not consumed in full is an
  `EntryError`; the box keeps its value and a tooltip names the problem
  (`text::entry::describe`).

## Units

`pt`, `px` (the CSS pixel, 1/96 in — a screen pixel changes size with zoom and
so cannot be a document unit), `in`/`"`, `ft`/`'`, `mm`, `cm`, `m`, `km`, with
plurals and both spellings of metre, then anything `pdfcer_core::dimension::Unit::parse`
accepts. The synonyms the engine's parser lacks are listed here rather than
there; that duplication is reported to the engine as a boundary finding.

## The egui seam

`drag_value(value, kind)` returns a `DragValue` whose `custom_parser` calls
`preprocess` on every keystroke and a `Refusal`. `Refusal::show(response)`
attaches the hover help and, while the box has focus and its text is
unreadable, the reason. A `Length` box shows its unit as a suffix, which is
what answers "what unit is this number in".
