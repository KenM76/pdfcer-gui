# `ui-verify/checks/object_clipboard`

`copy_and_paste_page_content` — **the operator's oldest open request**,
driven end to end.

# What this is for

> *"can you get cut copy and paste working for objects I select on the
> canvas?"* — asked in the first week and repeatedly since.

Until 2026-08-20 `Ctrl+C` on a shape put a **sentence** on the status row —
*"pdfcer can copy comments and markup, but it cannot yet put page content
back onto a page"* — which was honest and was still a refusal. `Pass 120.0`
shipped `ObjectClip` and this check is the wiring of it.

# Why this cannot be a unit test

Because the interesting failure is **silent and correct-looking**, and it is
the one the engine warned about in the reply that shipped the verb:

> `import_object` copies indirect objects. A page's content objects are byte
> ranges inside a content stream, and the operators in those bytes name
> their resources **by page-local name**. On the destination page, `/F1` is a
> different font. Paste the bytes verbatim and you get the right glyphs in
> the wrong typeface — or nothing at all. **Neither failure errors**, and
> neither is visible in a diff.

So the assertions here are about **counts the engine reports**, not about
whether a call returned `Ok`:

| # | link | its own test |
|---|---|---|
| 1 | `Ctrl+C` on page content reaches `copy_objects` rather than the old refusal | nothing — the refusal was in the shell |
| 2 | the clip is parked where a paste can find it, across a page change | nothing |
| 3 | `Ctrl+V` deserialises it and reaches `paste_objects` | nothing |
| 4 | **resources came with it** | `pdfcer-core` — and the count is the only thing this side can see |

Link 4 is the one that would ship. A clip that carried the operators and
dropped the resources pastes *something*, on the right page, at the right
place, in the wrong typeface. `resources_added=0` on a paste of text is the
tell, and it is on the trace for exactly that reason.

# What it does NOT assert, said rather than implied

**The pixels.** A paste offsets by 10 pt, which on a CAD sheet at 0.3× is
three screen pixels — below the noise floor of a window capture, and
`insert_image`'s own threshold repair is the record of what happens when a
pixel oracle is asked a question at that scale. What is asserted instead is
the engine's own count of what it wrote, which is a number a wrong build
gets wrong and a capture cannot resolve.
