# `app::actions::textstyle::donor`

The operator's font files offered to Bold and Italic as donor faces: the
ladder's rung 3 (`StyleRung::SuppliedFaceEmbedded`).

- `add(doc)` — asks for a file through `files::pick_font_file` and appends
  it to `OpenDoc::style_donors` if new. Traces
  `text-style-donor-added donors= file=`. The list lasts the session and is
  never saved.
- `plans(doc, wanted)` — one `FontEmbedPlan` per offered file that covers
  `wanted`, through `plan`. A file that cannot be read or does not cover the
  characters is skipped and traced `text-style-donor-unusable file= detail=`.
- `plan(path, wanted)` — `plan_subset` over the file's bytes. The base name
  is the file stem, which is what the engine matches against the run's
  family stem and the wanted style; the subset tag is `subset_tag_for` of
  the stem and characters, so one file and one set of characters always
  plan the same tag.
- `chars_of(texts)` — the characters of `texts`, sorted and deduplicated.

Callers: the Bold and Italic commits on whole runs (`textstyle::restyle`)
and on a span (`textstyle::span::apply`), and the Properties forecast
(`panels::properties::text::style`). Only `StyleChange::Weight` carries
donors; the engine ignores them for every other change.
