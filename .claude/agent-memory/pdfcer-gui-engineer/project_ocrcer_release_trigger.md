---
name: ocrcer-release-trigger
description: Standing pre-approved release — when the engine reports it adopted a new OCRcer, wire its GUI options and publish without asking
metadata:
  type: project
---

On 2026-09-27 Ken pre-approved a release (O248): when pdfcer reports it has implemented the next OCRcer version, move the pin, surface every new or changed OCRcer option in the GUI, drive it, refresh FEATURES, and publish to OneDrive and GitHub.

**Why:** OCRcer and pdfcer ship in sequence. Ken wants the GUI to follow each OCRcer release without another round-trip to him.

**How to apply:** Check the request channel every session for an OCRcer adoption note. Its arrival is the go-ahead to publish, the one exception to "ask before publishing". Related: [[ocrcer-second-engine]], [[always-publish-the-latest-build-to-onedrive]].
