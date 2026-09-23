---
name: release-waits-for-next-engine-release
description: From 2026-09-23 the next pdfcer-gui build+release waits for the engine's next tagged release (after v0.55.0), which should carry the G028–G038 answers
metadata:
  type: project
---

Ken, 2026-09-23: *"Pdfcer is still in the process of completing your feature
requests. Once the new pdfcer engine is released you can go ahead and build on
it and release."* Commit 1731d5c (coloured icons O232 + zoom white-out O231) is
committed but NOT packaged or published.

**Why:** the engine session is mid-way through G028–G038 (four had FIXED
replies on 2026-09-23); a release built on a half-landed engine would ship
partial behaviour and need re-releasing.

**How to apply:** the trigger is a NEW engine tag after `v0.55.0`
(`git -C D:/Dev/pdfcer tag --sort=-creatordate | head -1`). When it appears:
`cargo update` the core/render/print pins, wire the FIXED replies, run gates
AND `cargo test`, smoke-launch, then package + publish to OneDrive and GitHub
without asking — this is the standing go-ahead. Until then, do not publish;
[[always-publish-the-latest-build-to-onedrive]] is suspended for this build.
