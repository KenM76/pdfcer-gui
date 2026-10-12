---
name: drive-holds-deleted-target
description: If deleting files under D:\Dev frees no disk space, check for a sync client holding hard links (Google Drive did, until D:\Dev was unsynced)
metadata:
  type: project
---

Google Drive for desktop used to sync `D:\Dev` and kept its upload queue as
hard links in `D:\Dev\.tmp.driveupload\` (244 GB, stuck because Drive had been
paused for 169 days). While that was so, deleting 13 GB of `target/` freed
0 bytes. At Ken's request (2026-10-12) the coordinator removed `D:\Dev` from
Drive's sync and deleted the queue. Deletions under `D:\Dev` now free space at
once.

**Why it matters:** a hard link elsewhere keeps the data, so a deletion only
removes this repo's link. A rebuild after that writes NEW bytes, which makes
the shortage worse.

**How to apply:** if a cleanup does not move the free-space reading, run
`fsutil hardlink list <file>` on a file that is still there before deleting
anything more. A sync client's queue sits outside the project, so it is Ken's
to fix: report it and do not touch it. Related: [[project_disk_tight_target]].
