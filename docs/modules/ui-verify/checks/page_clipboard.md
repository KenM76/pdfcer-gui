# `ui-verify/checks/page_clipboard`

`pages_can_be_copied_and_pasted` — the driven proof of
`OPERATOR_REQUESTS.md` **O59**'s second item.

# What this is about

Ken, 2026-08-29, to the engine session: *"can you make sure we have cut,
copy, and paste available for everything and if not implement?"* The engine
shipped `copy_pages` / `cut_pages` / `paste_pages`; this shell consumed them
as three named commands on the Pages tab.

# ★★★ The oracle is the PAGE COUNT, and nothing smaller would do

Every cheaper thing a check could read here is a statement of *intent*:

| reading | what a green result would prove |
|---|---|
| `pageclip-copy` in the trace | the shell asked the engine for a clip |
| `pageclip-paste` in the trace | the shell raised an action |
| `insert-pages` applied | the engine returned `Ok` |
| **the document has more pages** | **the operator got what they pressed the button for** |

Only the last one distinguishes a working paste from a paste that inserted
zero pages successfully — which is exactly what a clip built from an empty
page list would do, silently, with every trace line present and correct.

★ So this counts pages before and after, from the application's own report
of how many it has, and asserts the arithmetic.

# The sequence

1. open a four-page fixture and read the page count;
2. invoke `pages.copy` — with nothing picked, the operand rule takes the
   **current** sheet, which is the ordinary case and the one an operator
   reaches first;
3. invoke `pages.paste`;
4. assert the document now has **one more page**.

★★ Steps 2 and 3 are invoked by **command id**, not by clicking the ribbon,
and that is deliberate rather than lazy. The subject here is whether the
page clipboard works; whether the Pages ▸ Clipboard band is drawn where the
manifest says is `shell::manifest`'s own assertion and a different question.
A check that clicked the band would fail for two unrelated reasons and its
message could not tell them apart.

# ★ What this does NOT prove, said out loud

That the **orphaned-widget disclosure** fires. That needs a source document
whose form fields straddle the copied sheets, and the honest way to get one
is the engine's own smoke fixture rather than a drawing improvised here. It
is the disclosure the engine flagged as *"the one that produces a document
that looks right and is not"*, so its absence from this check is a gap
rather than a decision — recorded so the next session finds it named.
