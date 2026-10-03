# `text::replace` — the Replace row's strings

Every word the Find bar's Replace row shows, and every sentence a Replace
press leaves on the status line. The decline variant
`Declined::Replace(ReplaceRefusal)` renders through `ReplaceRefusal::line`, so
the status line and this catalog cannot disagree.

The summary is `Replaced N hits.` when every hit was replaced and `Replaced N
of M hits.` otherwise, followed by one `Left unchanged: …` sentence per
reason a hit was skipped, each with its count.
