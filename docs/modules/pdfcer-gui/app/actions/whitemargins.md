# `app::actions::whitemargins` — Pages ▸ Crop ▸ Remove white margins

The body of `PageAction::RemoveWhiteMargins { pages, margin_mm }`, raised by
the button in the Crop pages window ([`dialogs::page_crop`](../../dialogs/page_crop.md)).

**Contract.** Each page in scope is measured with
`pdfcer_render::white_margin_crop(view, page, margin_pt)`, and every page that
draws something gets `CropBoxEdit::Set(crop)`. All of them go through
`apply::vector_edit` as **one** undo step, through `pagesize::crop`, so the
overhang and inherited disclosures of a hand crop are raised here too.

| Engine answer | What happens to the page | Counted as |
|---|---|---|
| `Ok(Some(c))` | crop set to `c.crop` | cropped |
| `Ok(None)` | left alone — it draws nothing | blank |
| `Err(_)` | left alone; `white-margins-refused` traced | failed |

No page cropped → `record_note` posts `text::page_crop::white_nothing` and
nothing is edited.

**Disclosure (R8b).** The box is an inference, so the status sentence
(`white_cropped`) says how it was found. The engine's three limits are worded
in the button's tooltip: the edge is within half a point (render at 144 dpi),
a tinted background counts as drawing, and a page is never cropped larger
than it already shows.

**Units.** `margin_mm` is whole millimetres from the window; it becomes
points through `units::points_from_mm` before the call.

**Thread.** Measuring renders on the UI thread, as image export does:
`DocumentView` borrows the session and cannot leave it.

Trace: `white-margins-measured pages= cropped= blank= failed= margin_mm=
llx= lly= urx= ury=` (the first crop), then the funnel's `page-crop-applied`.

Driven by `ui-verify --check remove_white_margins_crops_to_the_ink`.
