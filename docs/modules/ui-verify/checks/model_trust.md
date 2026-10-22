# `ui-verify/checks/model_trust`

`three_d_content_asks_while_it_is_off` — **while Play 3D content is off, View…
on a model raises the 3D prompt, and the prompt's answer is honoured.**

## What it drives

Seeds `play_3d = false` into the sandbox's preferences, launches on a copy of
the four-page document with `assembly.prc` as the model picker's answer
(`model_view_window::launch_with_model`), and places the model from the
ribbon. Clicks through the scripted pointer, so it runs under `--no-input`.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | View… traces `model3d-decision decision=ask` and `model_trust.body` is drawn | the model opens without asking, or the choke point is bypassed |
| 2 | Cancel traces `model3d-answer trust=no` and no `model-view-opened` | refusing showed the model |
| 3 | View… asks again; *one time only* traces `trust=once` and `model-view-opened` | the trusting answer does not reach the viewer |
| 4 | a third View… traces `decision=play` and draws no prompt | the one-time trust was forgotten while the document is open |
| 5 | the preferences hold no `trusted_3d_document` line | *one time only* was kept as *always* |

*Trust this document always* is not driven: once a document is trusted, the
same session cannot raise the prompt again. Its write is `TrustPrefs::trust_3d`,
covered by the preferences round-trip tests.
