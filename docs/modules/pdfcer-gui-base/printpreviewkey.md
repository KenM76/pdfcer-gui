# `printpreviewkey` — what a print-preview bitmap is a picture of

`PreviewKey` names everything a cached preview raster depends on — page,
annotation scope, the whole `Settings`, and the fixed line width with its
placement scale — so a change to any of them misses the cache rather than
showing a stale sheet. `Overhang` is the verdict drawn from that raster, kept
beside the key because the caption and the hatch must read the same fact.
