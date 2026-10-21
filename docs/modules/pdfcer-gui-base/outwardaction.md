# `pdfcer-gui-base/outwardaction`

**The acts of a link that leaves the document.** `OutwardAction::WebLink`
is raised by a click on a `/URI` link with the address exactly as the engine
reads it (`EditSession::link`); `WebAnswer` by the Security Warning. Both are
applied by `app::actions::outward` and are never edits.
