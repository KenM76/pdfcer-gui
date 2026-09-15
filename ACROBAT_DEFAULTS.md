# ACROBAT_DEFAULTS.md — Acrobat's authored markup defaults, measured

The colours, icons, sizes and serialisation habits Adobe Acrobat DC writes into
annotations, read out of its own preference hive; read it before choosing a
default for any markup this shell authors. A colour this shell picks is written
into `/C` in the operator's file and travels to whoever he sends it to, so
values here are lifted from the reference application, never reasoned from
convention.

## Reading the hive

`HKCU\Software\Adobe\Adobe Acrobat\DC\Annots\cAnnots` is a live preference
hive, not a factory table. Its sibling `cAnnot` key carries `tauthor=Ken`, which
proves Acrobat has written to it, so any row below may be an operator override
rather than an Adobe default, and no field in the hive distinguishes the two.

The discriminator is a sibling key, not a plausibility judgement:

1. **Bit-identical across unrelated subtypes.** Nobody hand-sets a `/Sound`
   annotation's colour to match their highlighter. `cHighlight`,
   `cInk:InkHighlight` and `cSound` all carry `1.0 0.384308 0.0` to six
   decimals, so that is a shipped table.
2. **Many subtypes agreeing to six decimals.** An operator who recoloured their
   rectangle tool would not also have recoloured the cloud, the polyline, the
   stamp and the squiggly to the same six-decimal value.
3. **Every component landing exactly on a 1/255 boundary** — `0.858826 × 255 =
   219.000`. The weakest of the three, because a human picking from a
   byte-based picker lands on the same boundaries. It corroborates; it never
   decides.

A value repeated across keys nobody would set together is factory. A value
appearing once is unproven in either direction, and this file says so per row
rather than guessing. Apply the sweep symmetrically: a row about to be
dismissed as an override earns the same corroboration check as a row about to
be adopted. Take any value from here with its source — if one is later found to
be the operator's rather than Adobe's, the fix is one constant and one citation.

`crichDefaults\cfontFamily` and `crichDefaults\ctextColor` sit one level deeper
than everything else. A recursive dump and a two-level dump disagree silently
and the two-level one looks complete, so dump recursively.

`HKCU\Software\Adobe\Acrobat Reader\DC\Annots\cAnnots` exists too and is a
subset — only the four subtypes Reader lets you author. Where the two disagree,
prefer the Acrobat hive: Reader's is stale, and a value that lags is worse than
one that might be an override, because nothing marks it as old.

## Colours

Converted from the hive's `d1 d2 d3` float triples. The float is the truth; the
hex is for reading. `t0=T` means transparent / none, which is Acrobat's answer
to what a shape's interior defaults to.

| Acrobat key | what it is | stroke `/C` | hex | fill `/IC` | this shell |
|---|---|---|---|---|---|
| `cSquare` | rectangle | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cCircle` | ellipse | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cLine` | line | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cLine:LineArrow` | arrow | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cPolyLine` | polyline | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cPolygon` | polygon | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cPolygon:PolygonCloud` | revision cloud | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cInk` | freehand | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Shape` |
| `cStamp` | stamp | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Stamp` |
| `cSquiggly` | squiggly underline | `0.858826 0.203918 0.145096` | #DB3425 | none | `PenSlot::Squiggly` |
| `cUnderline` | underline | `0.074509 0.450974 0.909805` | #1373E8 blue | none | `PenSlot::Underline` |
| `cStrikeOut` | strikeout | `0.972549 0.392151 0.392151` | #F86464 | none | `PenSlot::StrikeOut` |
| `cText` | sticky note | `0.588242 0.262741 0.988235` | #9643FC violet | none | `PenSlot::Note` |
| `cFreeText` | text box | `0.972549 0.392151 0.392151` | #F86464 border | #FFFFFF white | not taken — see below |
| `cFreeText` text | text box's words | `0.858826 0.203918 0.145096` | #DB3425 | — | `PenSlot::TextBox` |
| `cFreeText:FreeTextTypewriter` | typewriter | none | — | none | — |
| `cCaret` | caret | `0.752945 0.215683 0.768631` | #C037C4 | none | grid cell only |
| `cFileAttachment` | attachment | `0.588242 0.262741 0.988235` | #9643FC | none | no slot |
| `cSound` | sound | `1.0 0.384308 0.0` | #FF6200 | none | no slot |
| `cHighlight` | highlight | `1.0 0.384308 0.0` | #FF6200 orange | none | `PenSlot::Highlighter` |
| `cInk:InkHighlight` | ink highlighter | `1.0 0.384308 0.0` | #FF6200 orange | none | `PenSlot::Highlighter` |

The highlighter is orange, not yellow. `canvas::markup::palette` stores these
as sRGB bytes — `MARKUP_RED`, `HIGHLIGHTER_ORANGE`, `UNDERLINE_BLUE`,
`STRIKEOUT_PINK`, `NOTE_PURPLE`, `CARET_MAGENTA`, `FREETEXT_GREEN`, `BLACK`,
`WHITE` — plus `CLASSIC_YELLOW` (`#FFFF00`), which is this shell's own
highlighter and is labelled as such rather than attributed to Adobe. The ten
form the `ACROBAT` swatch grid, `COLUMNS = 5`.

Changing a pen default changes the next mark only. `/C` is written per
annotation at author time, and the one verb that recolours a placed mark is
`set_markup_style`, driven from a selection.

### Where this shell diverges, and why

- **Text-box border.** Acrobat splits `cFreeText` into a pink border and red
  words; `canvas::textannot` authors one ink used for both. `PenSlot::TextBox`
  ships at Acrobat's **text** colour, so a text box here has a red frame where
  Acrobat's would be pink.
- **Text-box fill.** No geometric markup is authored with a filled interior —
  `spec` passes `interior: None` for every shape — because on a CAD sheet an
  unfilled comment shape is the only kind that does not hide the content it is
  about. White is a palette cell, not a default fill.
- **Caret and green.** Nothing defaults to `CARET_MAGENTA` or `FREETEXT_GREEN`;
  both are grid cells, offered because they are hues Adobe itself marks up in.
- **Point size.** Acrobat's 12 is recorded below; `TEXT_SIZE_PT` is 11.0,
  chosen as a legible caption on a drawing sheet and stated explicitly so the
  size does not vary with how big a box the operator dragged.

## Non-colour values

| fact | value | key |
|---|---|---|
| default sticky-note icon | `Comment` | `cAnnot` `tnoteIcon` |
| default attachment icon | `Paperclip` | `cAnnot` `tattachIcon` |
| text box / typewriter point size | 12 | `crichDefaults` `dtextSize` |
| text box font family | Helvetica, falling back to `sans-serif` | `crichDefaults\cfontFamily` |
| text box alignment, weight, style | left, normal, normal | `crichDefaults` |
| highlight-with-note opacity | 0.40 | `cHighlight:HighlightNote` `dopacity` |
| plain highlight opacity | absent, therefore 1.0 | `cHighlight` has no `dopacity` |

`tnoteIcon` is a singleton and cannot pass the sibling test — there is no second
key in the tree carrying a sticky-note icon name, and it lives in `cAnnot`, the
key whose `tauthor` proves Acrobat wrote to it. `DEFAULT_STICKY_ICON` adopts
`Comment` anyway, on the ground that matching the program on the operator's desk
means matching what its note tool opens on, not a factory value he has never
seen. The cost of being wrong is one constant.

`Pen::default`'s `opacity` is `1.0`, and `opacity_option` answers `None` there
so no `/CA` key is written. The 0.40 belongs to `HighlightNote` — the
highlight-and-add-a-comment tool, a different Acrobat tool — not to plain
Highlight, which carries no `dopacity` and is fully opaque. Misread one key to
the left, it would make every highlight this shell authors 40 % transparent for
a reason nobody could later find.

`cLine:LineDimension` is Acrobat's dimension line and is deliberately not read.
Acrobat's "dimension" is a **ce dimension** in this project's vocabulary — a
`/Line` with `/IT /LineDimension` that this shell authors — and its style
belongs to `set_dimension_style`, a different verb with a different model. A
**pdf dimension** is CAD-exported page content and has nothing to do with
either. `cRedact` is likewise present and likewise not markup; `text::redact`
owns that surface.

### The FreeText body colour is stated twice and the two disagree

| key | value | verdict |
|---|---|---|
| `cFreeText\ctextColor` | `0.858826 0.203918 0.145096` → #DB3425 | corroborated — the same six decimals as ten shape subtypes |
| `cFreeText\crichDefaults\ctextColor` | `0.023529 0.541183 0.109802` → #068A1C green | singleton — offered in the grid, not a default |

A text box's words are painted from the rich-text default (`/RC`), so the green
is the one Acrobat would use. It is the one this shell declines as a default, on
the sibling test: #DB3425 appears in eleven keys and #068A1C in one. A lone
green in a hive that is provably operator-written is the shape of a setting
somebody once changed, and no second key anywhere in the tree corroborates it.

## Line width — there is no such registry value

A recursive search of the whole `HKCU\Software\Adobe\Adobe Acrobat\DC` tree for
`width`, `thick` and `border` returns only print N-up and multimedia keys.
Acrobat stores no default `/BS /W` for any annotation subtype, so colour and
width are not the same kind of question: colour is a lookup, width is an
inference from observed files.

Acrobat omits `/BS` entirely when the width is 1, so an Acrobat-authored
annotation with no `/BS` key is a 1 pt annotation, not a borderless one; 1 pt
has been established three independent ways from Acrobat-authored PDFs. An
absent key is evidence about the serialiser, not about the default — Acrobat
has a default width, it is 1 pt, and it is invisible in the registry by
construction.

This shell does not take it. `Pen::default`'s `width_pts` is 2.0, because a
hairline vanishes among a CAD export's own 0.25 pt linework, which is the
specific failure a markup on an engineering drawing has to avoid; Acrobat's
1 pt is tuned for a letter-size text document read at 100 %. The width is one
value for the whole pen, not per slot, clamped by its control to
`MIN_WIDTH_PTS = 0.25` through `MAX_WIDTH_PTS = 12.0`. Anything derived from it
must be derived at call time, not frozen at 2 pt:
`Pen::simplify_tolerance_pts` returns `width_pts / 4.0` — half the stroke's
half-width, so a simplified centreline stays inside the stroke the operator saw.

## Serialisation habits

Observed in Acrobat-authored annotations, not in the registry. These matter for
reading Acrobat's files, not for authoring:

- `/BS` is omitted at width 1. Absent `/BS` is not absent border.
- `/CA` is omitted at 100 %. Absent `/CA` is not transparent.
- `/RD` is written as W/2 on the subtypes that carry it.
- Colour and opacity components are float32-quantised, hence `0.858826` rather
  than `0.86`, which is why the table quotes the full float.
- `/Border` is never written; `/BS` supersedes it.

## Unsourced

No sourced value from a current Acrobat for: sticky-note `/C` as written into a
file (the hive says violet; nothing on this machine confirms it), Redact, Cloud
`/BE /I` intensity, Ink Highlighter, and the Stamp gallery's own colours.

One action closes all of them: draw one markup of each type in Acrobat, save,
and read `/C`, `/CA` and `/BS` out of the file. It needs Acrobat driven against
a real document.

## The dump

```powershell
$base='HKCU:\Software\Adobe\Adobe Acrobat\DC\Annots\cAnnots'
Get-ChildItem $base | ForEach-Object {
  $sub = $_.PSChildName
  $own = Get-ItemProperty $_.PSPath
  $line = "== $sub"
  foreach ($p in $own.PSObject.Properties) { if ($p.Name -notmatch '^PS') { $line += "  [$($p.Name)=$($p.Value)]" } }
  Write-Output $line
  Get-ChildItem $_.PSPath | ForEach-Object {
    $c = Get-ItemProperty $_.PSPath
    $s = "     - $($_.PSChildName):"
    foreach ($p in $c.PSObject.Properties) { if ($p.Name -notmatch '^PS') { $s += " $($p.Name)=$($p.Value)" } }
    Write-Output $s
  }
}
```
