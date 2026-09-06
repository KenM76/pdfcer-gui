# ACROBAT_DEFAULTS.md — what Adobe Acrobat actually authors, measured

**Measured 2026-09-06** on this machine, from the installed **Adobe Acrobat DC**'s
own preference hive. Not from documentation, not from a web page, not from
memory. The command that produced it is at the bottom so anyone can re-run it
and disagree with a number rather than with an opinion.

## Why this file exists

The operator's instruction of 2026-09-06:

> *"Also make sure you've used the same default colours and style look for these
> things as Adobe."*

That is a **claim-bearing** requirement: a colour this shell picks is written
into `/C` in the operator's file and travels to whoever he sends it to. The
standing rule on claim-bearing copy is *verify the source, don't invent* —
plausible defaults reconstructed from convention are exactly the failure that
rule exists to prevent. So the values below are lifted, not reasoned.

It is also the third worked example of this project's **reference-app rule**
(`FEATURES.md`, the polygon-ending row): where a product class has converged on
an answer, the convergence **is** the spec, and an invented model is a defect
even when it works. Acrobat is the program this one is replacing on the
operator's desk.

## ★ THE ONE CAVEAT, AND IT IS LOAD-BEARING

`HKCU\…\Annots\cAnnots` is a **live preference hive**, not a factory table. Its
sibling `cAnnot` key carries `tauthor=Ken`, which proves Acrobat has written to
it. **Any row below may therefore be an operator override rather than an Adobe
default**, and there is no field in the hive that distinguishes the two.

### ★★ The test that separates them, and it is the useful part of this file

Two sessions read this hive independently, minutes apart, and reached the same
numbers. The second one also produced the argument the first had missed, and it
is a good one — **three independent signs that a value is factory rather than
personal**:

1. **Bit-identical across UNRELATED subtypes.** Nobody hand-sets a `/Sound`
   annotation's colour to match their highlighter. When `cHighlight`,
   `cInk:InkHighlight` and `cSound` all carry `1.0 0.384308 0.0` to six
   decimals, that is a shipped table, not a preference.
2. **Ten shape subtypes agreeing to six decimals.** An operator who recoloured
   their rectangle tool would not have recoloured the cloud, the polyline, the
   stamp and the squiggly to the same six-decimal value.
3. **Every component lands exactly on a 1/255 boundary.** `0.858826 × 255 =
   219.000`. This one is the weakest of the three — a colour a *human* picked
   from a byte-based picker lands on the same boundaries — so it corroborates
   and never decides.

⇒ **The discriminator is a SIBLING, not a plausibility judgement.** A value
repeated across keys nobody would set together is factory. A value that appears
**once** is unproven in either direction, and this file says so per row rather
than guessing.

★ That test, applied honestly, immediately overturned this file's own first
verdict on the highlighter — see the struck-through paragraph below. The
reasoning was written before the test existed, which is exactly how a
plausible-sounding dismissal survives.

⚠ **Anything you take from here, take with its source.** If a value is later
found to be Ken's rather than Adobe's, the fix is to change one constant and
one citation — which is only cheap because the citation is written down.

## The table

Converted from the hive's `d1 d2 d3` float triples. The float is the truth; the
hex is for reading. `t0=T` means **transparent / none**, which is Acrobat's way
of saying *no fill* — worth noting, because it is the answer to *"what does
Acrobat default a shape's interior to?"* and the answer is **nothing**.

| Acrobat key | what it is | stroke `/C` | hex | fill `/IC` | adopted here |
|---|---|---|---|---|---|
| `cSquare` | rectangle | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cCircle` | ellipse | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cLine` | line | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cLine:LineArrow` | arrow | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cPolyLine` | polyline | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cPolygon` | polygon | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cPolygon:PolygonCloud` | revision cloud | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cInk` | freehand | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cStamp` | stamp | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cSquiggly` | squiggly underline | `0.858826 0.203918 0.145096` | **#DB3425** | none | ✅ |
| `cUnderline` | underline | `0.074509 0.450974 0.909805` | **#1373E8** blue | none | ✅ |
| `cStrikeOut` | strikeout | `0.972549 0.392151 0.392151` | **#F86464** | none | ✅ |
| `cText` | sticky note | `0.588242 0.262741 0.988235` | **#9643FC** violet | none | ✅ |
| `cFreeText` | text box | `0.972549 0.392151 0.392151` | **#F86464** border | **#FFFFFF** white | ✅ |
| `cFreeText` text | text box's words | `0.858826 0.203918 0.145096` | **#DB3425** | — | ✅ |
| `cFreeText:FreeTextTypewriter` | typewriter | none | — | none | — |
| `cCaret` | caret | `0.752945 0.215683 0.768631` | #C037C4 | none | n/a |
| `cFileAttachment` | attachment | `0.588242 0.262741 0.988235` | #9643FC | none | n/a |
| `cHighlight` | **highlight** | `1.0 0.384308 0.0` | **#FF6200** orange | none | ✅ **adopted — operator ruling, see below** |
| `cInk:InkHighlight` | ink highlighter | `1.0 0.384308 0.0` | **#FF6200** orange | none | ✅ |

Non-colour facts from the same hive:

| fact | value | key |
|---|---|---|
| default sticky-note icon | **`Comment`** | `cAnnot` `tnoteIcon` |
| default attachment icon | `Paperclip` | `cAnnot` `tattachIcon` |
| text box / typewriter point size | **12** | `crichDefaults` `dtextSize` |
| text box font family | **Helvetica**, falling back to `sans-serif` | `crichDefaults\cfontFamily` |
| text box alignment, weight, style | left, normal, normal | `crichDefaults` |
| highlight-**with-note** opacity | **0.40** | `cHighlight:HighlightNote` `dopacity` |
| plain highlight opacity | **absent ⇒ 1.0** | `cHighlight` has no `dopacity` |

### ⚠ The FreeText body colour is stated TWICE and the two disagree

| key | value | verdict |
|---|---|---|
| `cFreeText\ctextColor` | `0.858826 0.203918 0.145096` → **#DB3425** | **corroborated** — the same six decimals as ten shape subtypes |
| `cFreeText\crichDefaults\ctextColor` | `0.023529 0.541183 0.109802` → **#068A1C** green | **singleton — not adopted** |

A text box's words are painted from the rich-text default (`/RC`), so the green
is the one Acrobat would *use*. It is nonetheless the one this shell declines,
and the sibling test above is the whole reason: **#DB3425 appears in eleven
keys; #068A1C appears in one.** A lone green in a hive that is provably
operator-written is the shape of a setting somebody once changed, and there is
no second key anywhere in the tree to corroborate it.

★ Note the asymmetry with the highlighter, because it is not inconsistency.
Orange was a singleton **until the sibling test was run**, at which point two
unrelated keys carried it and the dismissal collapsed. Green was put through the
same test and **failed** it. The rule did the work in both directions, which is
the point of having one.

⚠ It is also a third-level key. The first sweep of this hive walked **two**
levels and never saw it — `crichDefaults\cfontFamily` and
`crichDefaults\ctextColor` sit one deeper than everything else in the table
above. ⇒ **A recursive dump and a two-level dump disagree silently**, and the
two-level one looks complete.

## ⚠ The three rows this shell does NOT take, and why

**1. ~~`cHighlight` = #FF6200 orange.~~ ★ OVERRULED BY THE OPERATOR, 2026-09-06,
WITHIN THE HOUR — and the overruling is the most instructive thing in this file.**

This section argued for keeping yellow:

> ~~Every reader on earth ships a yellow highlighter and pdfcer has shipped
> yellow since the tool existed. A lone orange in a hive that is provably
> operator-written is far more likely to be a personal choice than Adobe's
> factory value — and the cost of being wrong is asymmetric: adopting it makes
> every highlight in every file the operator has ever marked up change colour,
> on the strength of one registry read. **Yellow stays.**~~

The operator read that reasoning and answered it in one line: ***"change the
highlighter colour to match adobe."*** So **#FF6200 is adopted.**

★★ **Every premise above was reasonable and the conclusion was still wrong, and
it is worth being precise about which step failed.** Not the measurement — the
hive really does say orange. Not the caution about overrides — it really might
be one. What failed is the framing: *"is this Adobe's factory default or Ken's
personal setting?"* is the wrong question, because **the operator asked to match
the program on his desk, and the program on his desk highlights in orange.** A
factory default he has never seen is not what *"the same as Adobe"* means to the
person who says it.

★ **And the corroboration was sitting in the same dump, unread.**
`cInk:InkHighlight` — the freehand highlighter, a *different tool* — carries the
identical `1.0 0.384308 0.0`. Two tools agreeing is exactly the evidence this
file demanded for `#DB3425` and accepted there; the same standard was not
applied here because the conclusion felt safe. ⇒ **A row dismissed as "probably
an override" deserves the same corroboration sweep as a row you are about to
adopt.** The sweep costs nothing once the dump is already on screen.

The asymmetric-cost argument survives as a *fact* and dies as a *reason*:
existing highlights in existing files are untouched — `/C` is written per
annotation at author time and nothing rewrites a placed mark — so the change
affects the next highlight and no previous one. That was worth checking before
it was used as an argument, and it was not.

**2. The 0.40 opacity.** It belongs to `HighlightNote` — the *highlight-and-add-a-
comment* tool, a different Acrobat tool — and **not** to plain Highlight, which
carries no `dopacity` at all and is therefore fully opaque. This is the kind of
row that is trivially misread one key to the left, and doing so would have made
every pdfcer highlight 40 % transparent for a reason no one could later find.

**3. `cLine:LineDimension`.** Acrobat's dimension line, red on red on red. Not
adopted, and **Rule 15** is why: Acrobat's "dimension" is a **ce dimension** in
this project's vocabulary — a measurement pdfcer authors — and its style is
`set_dimension_style`'s, a different verb with a different model. A **pdf
dimension** is CAD-exported page content and has nothing to do with either. The
two are opposites and the ambiguity has already sent one investigation down the
wrong path.

## Line width — ★ THERE IS NO SUCH REGISTRY VALUE, AND THAT IS A FINDING

**Not in this hive, and not anywhere in it.** A recursive search of the whole
`HKCU\Software\Adobe\Adobe Acrobat\DC` tree for `width`, `thick` and `border`
returns only print N-up and multimedia keys. Acrobat stores **no** default
`/BS /W` for any annotation subtype.

So the colours and the width are **not the same kind of question**, and treating
them as one row of one table would have been the mistake here. The colours are a
lookup. The width is an inference from observed files:

- Acrobat **omits `/BS` entirely when the width is 1**, so an Acrobat-authored
  annotation with no `/BS` key *is* a 1 pt annotation, not a borderless one.
- A parallel investigation established 1 pt three independent ways from
  Acrobat-authored PDFs already on this machine.

⇒ **An absent key is evidence about the serialiser, not about the default.** The
two sessions that read this hive both reported "no width" and one of them nearly
reported it as *"Acrobat has no default width"*, which is false — it has one,
and it is 1 pt, and it is invisible in the registry by construction.

⚠ **This shell does not take it, and the reason is the operator's own use case.**
`canvas::markup::pen`'s 2 pt carries an argument that is about *his* documents:

> a hairline vanishes among the drawing's own 0.25 pt linework, which is the
> specific failure a markup on an engineering drawing has to avoid

Acrobat's 1 pt is tuned for a letter-size text document read at 100 %. A dense
CAD site plan is the case pdfcer exists for, and matching Adobe here would make
a comment shape harder to see on exactly the sheets this program is used on.
**2 pt stays**, both sides recorded, and it is now a per-kind value the operator
can change.

## Serialisation habits worth knowing

Observed in Acrobat-authored annotations, not in the registry. These matter for
*reading* Acrobat's files, not for authoring ours:

- **`/BS` is omitted at width 1.** Absent `/BS` ≠ absent border.
- **`/CA` is omitted at 100 %.** Absent `/CA` ≠ transparent.
- **`/RD` is written as W/2** on the subtypes that carry it.
- Colour and opacity components are **float32-quantised**, so `0.858826` rather
  than `0.86` — which is why the table above quotes the full float.
- **`/Border` is never written**; `/BS` supersedes it.

## Still open, and what would close it

No sourced value from a current Acrobat for: sticky-note `/C` as *written into a
file* (the hive says violet; nothing on this machine confirms it), Redact,
Cloud `/BE /I` intensity, Ink Highlighter, and the Stamp gallery's own colours.

**One action closes all of them:** draw one markup of each type in Acrobat, save,
and read the `/C`, `/CA` and `/BS` out of the file. That is a five-minute job
for whoever has Acrobat open, and it would also date the `#E52237` → `#DB3425`
red change precisely. It is not done here because it means driving Acrobat
against a real document.

## How to re-run this

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

`HKCU\Software\Adobe\Acrobat Reader\DC\Annots\cAnnots` exists too and is a
**subset** — it carries only the four subtypes Reader lets you author. Where the
two disagree, prefer the Acrobat hive: Reader's is stale, and a value that lags
is worse than one that might be an override, because nothing marks it as old.
