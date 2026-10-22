# `pdfcer-gui/dialogs/model_trust`

**The prompt a 3D model raises while Play 3D content is off.** Acrobat's
document message bar as a window: *3D content has been disabled. Enable this
feature if you trust this document.*, with *Trust this document one time
only*, *Trust this document always* and Cancel.

# Contract

- Raised from the one 3D choke point, the `AttachmentAction::ViewModel` arm of
  `app::actions::apply`, which both the canvas click and the Attachments
  panel's View… reach. It is raised when `TrustPrefs::plays_3d` says no for the
  document's path and the document has not been trusted once.
- Enter or *one time only*: `OutwardAction::ModelAnswer { trust: Once }`, then
  the `ViewModel` that raised it. `OpenDoc::models_trusted_once` holds the
  answer until the document closes; nothing is written to the preferences.
- *Always*: `trust: Always`, then the `ViewModel`. The document's path is kept
  as `trusted_3d_document` in the preferences.
- Escape, Cancel or the close button: `trust: No` and nothing else; the status
  line says the model was not shown and why.
- Regions: `model_trust.body` and the dialog host's `dialog.buttons.accept`
  (once), `dialog.buttons.keep` (always) and `dialog.buttons.cancel`. Traces:
  `model3d-decision decision=ask|play page=…` from the choke point,
  `model3d-answer trust=no|once|always` from the prompt.
- Driven by `ui-verify` check `three_d_content_asks_while_it_is_off`.
