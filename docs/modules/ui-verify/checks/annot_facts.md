# `ui-verify/checks/annot_facts`

One check on `fixtures/annot-facts.pdf`: one 400 × 400 page carrying a
`/FreeText` at `/Rect [100 100 200 140]` with no `/DA`. Without `/DA`,
`text_spec_from_dict` declines it, and `spec_from_dict` has no arm for a
text box, so neither restyle verb reads the mark. It carries `/C [1 0 0]` and
`/Name /Comment`. Only `annot::Annotation::color` and
`annot::Annotation::icon` report those for this mark.

# `an_annotations_colour_and_icon_are_read_for_any_subtype`

Driven with `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.

1. Click (150, 120). Require `annot-select` with `subtype=FreeText`.
2. Take a screenshot, `annot-facts-after.png`, of the Properties row.
3. Require `annot-facts-shown` carrying `colour=1,0,0` and `icon=Comment`.
   A row that reads neither key traces `none`. A row drawn only for the
   subtypes a restyle verb reads draws nothing here, and traces nothing.
