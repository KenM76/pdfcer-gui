# `overprint_default`

**Owns:** Settings ▸ Display ▸ *Use overprint preview* decides whether a
document opens with overprint preview on, and the Settings window writes the
choice ([`prefs::overprint`](../../pdfcer-gui-base/prefs/overprint.md)).

**Drive.** Nine launches, each with the choice planted in the sandbox's
preferences file and one fixture opened in Read mode; the oracle is the
last `overprint-default choice=… on=…` line whose `asks` is not `?`.
*Automatic* answers from a thread, so its first line says `asks=?` and the
check waits for the second; a walk that never answers fails.

| Planted | Fixture | Owed |
|---|---|---|
| nothing | `checker-image.pdf` | `always`, on |
| `never` | `print-shop.pdf` | off |
| `never` | `print-shop-pdfx.pdf` | off |
| `pdfx` | `print-shop.pdf` | off |
| `pdfx` | `print-shop-pdfx.pdf` | on |
| `pdfx` | `print-shop-pdfx4.pdf` | on |
| `automatic` | `checker-image.pdf` | off |
| `automatic` | `print-shop.pdf` | on |
| `automatic` | `print-shop-listed.pdf` | off |

`print-shop.pdf` asks for overprint and claims no standard;
`print-shop-pdfx.pdf` is the same page with a `GTS_PDFX` output intent;
`print-shop-pdfx4.pdf` claims PDF/X-4 in XMP only, with no intent;
`print-shop-listed.pdf` lists an overprinting `/GS0` that no `gs` invokes;
`checker-image.pdf` asks for neither. Every choice meets one fixture where a
wrong reading flips the answer, so neither a constant nor a choice ignored in
favour of another passes.

Then `automatic` is planted, Settings ▸ Display opened, the page body rolled
until `settings.display.overprint.never` is declared, Never pressed, Save
pressed (a new `prefs-saved` line required), and the file must read
`overprint_preview_default = never`.

**Does not own:** that the preview, once on, draws overprint correctly
(`overprint_preview`), or the Output Preview panel (`output_preview`).
