# `document_properties_list_page_box_sizes`

**Defect it guards.** Document properties ▸ Page box sizes is missing, cannot
be opened, merges pages whose boxes differ, or does not say which boxes the
file writes.

**Fixture.** `fixtures/page-box-sizes.pdf`, built by
`page-box-sizes.PROVENANCE.py`. Pages 1–2 are 600 × 800 with a written bleed
`[10 10 590 790]` and trim `[20 20 580 780]`. Page 3 is the same sheet with no
boxes written. Page 4 is 1000 × 700 with a written art box `[100 100 900 600]`.

**Steps.** Launch with Document properties open. The control: the
`docprops.box-sizes.header` region is live and no `box-sizes` line has been
traced, so the section is closed and the line after the press is the press's
doing. Press the header. The line must say `unit=mm groups=3`, `g1=1-2` with
bleed `580x780:written`, trim `560x760:written`, crop and art
`600x800:default`; `g2=3` with trim `600x800:default`; `g3=4` with media
`1000x700:written` and art `800x500:written`. Sizes in the trace are always
points. Press `docprops.box-sizes.unit.in`; the next line must say `unit=in`
with the same boxes.

**Falsified** by grouping on the media box alone: `groups=2`, FAIL.

**What it does not prove.** The displayed numbers after unit conversion and
their wording, read in `text::panels::docprops::box_size`.
