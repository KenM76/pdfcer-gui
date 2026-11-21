# `pdfcer-gui/dialogs/export_image/plates`

The Export image window's *One file per ink (plates)* checkbox, drawn under
the text choice when EMF is the format. It sets `ImagePlan::plates`; the plan
carries it only for EMF. Region `export-image.plates`; toggling traces
`export-image-plates-chosen plates=`. Not remembered between exports, like the
rendering standard: plates are a property of one deliverable.
