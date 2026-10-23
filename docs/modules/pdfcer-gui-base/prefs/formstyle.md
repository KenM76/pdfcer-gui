# `prefs::formstyle` — Forms ▸ Style, kept across sessions

`FormStylePrefs::look`, an `Option<FieldLook>`: `None` until Style, *Use as
default look* or an accepted placement sets it. Keys:

| Key | Value |
|---|---|
| `form_style_border_width` | points, ≥ 0 |
| `form_style_border_style` | `BorderStyle::token` (`solid`, `dashed`, …) |
| `form_style_fill`, `form_style_outline` | `none`, `gray V`, `rgb R G B`, `cmyk C M Y K`; absent = unstated |
| `form_style_text_font` | a `Std14` variant as `Debug` spells it (`TimesRoman`) |
| `form_style_text_size` | points; 0 is the engine's auto size |
| `form_style_text_colour` | `gray`, `rgb` or `cmyk` as above |

Components are 0–1; a value outside that, an unknown face or a negative
number is refused as a bad value. The first key read starts from a text
field's defaults, so a file carrying only some keys still yields a whole look.

Nothing is written for an unset look, so *unset* round-trips as absence rather
than as the defaults — the placement dialog then opens each kind on its own.

The application side, `app::prefs::formstyle::keep`, runs once a frame:
it compares `Remembered::chosen` with the stored look and saves on a
difference, waiting until no pointer button is down so a drag saves once.
