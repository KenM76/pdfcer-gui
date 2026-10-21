---
name: channel-consume-keeps-files
description: Consuming a reply in the request channel ADDS a done_*_CONSUMED.md; the request and reply stay in open/. The channel is not under git, so an rm is unrecoverable.
metadata:
  type: feedback
---
Consuming a reply = write `done_GNNN_CONSUMED.md` at the channel root and drop the INDEX row. The `request_*` and `reply_*_FIXED` files stay in `open/`.

**Why:** I rm'd G221/G223's request+reply while consuming them. `D:\Dev\FeatureRequests\` has no `.git`, and I had never read the G221 reply in full, so it could only be reconstructed from the engine commit.

**How to apply:** never `rm` in the channel. `ls open | grep G2xx` and confirm an older consumed id still has its files before acting on a guessed convention. Related: [[feedback_never_git_checkout]].
