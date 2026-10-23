# `overprint_default`

**Owns:** Settings ▸ Display ▸ *Use overprint preview* decides whether a
document opens with overprint preview on, and the Settings window writes the
choice ([`prefs::overprint`](../../pdfcer-gui-base/prefs/overprint.md)).

**Drive.** Seven launches, each with the choice planted in the sandbox's
preferences file and one fixture opened in Read mode; the oracle is the
`overprint-default choice=… on=…` line each open writes.

| Planted | Fixture | Owed |
|---|---|---|
| nothing | `checker-image.pdf` | `always`, on |
| `never` | `print-shop.pdf` | off |
| `never` | `print-shop-pdfx.pdf` | off |
| `pdfx` | `print-shop.pdf` | off |
| `pdfx` | `print-shop-pdfx.pdf` | on |
| `automatic` | `checker-image.pdf` | off |
| `automatic` | `print-shop.pdf` | on |

`print-shop.pdf` asks for overprint and claims no standard;
`print-shop-pdfx.pdf` is the same page with a `GTS_PDFX` output intent;
`checker-image.pdf` asks for neither. Every choice meets one fixture where a
wrong reading flips the answer, so neither a constant nor a choice ignored in
favour of another passes.

Then `automatic` is planted, Settings ▸ Display opened, the page body rolled
until `settings.display.overprint.never` is declared, Never pressed, Save
pressed (a new `prefs-saved` line required), and the file must read
`overprint_preview_default = never`.

**Does not own:** that the preview, once on, draws overprint correctly
(`overprint_preview`), or the Output Preview panel (`output_preview`).
