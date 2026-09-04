# GLYPH_ADOPTION.md — the 2026-09-03 icon proposal, adjudicated

An outside review of the shipped GUI arrived on 2026-09-03 carrying, among much
else, **sixty-five proposed icon glyphs** as path data in
`D:\Dev\FeatureRequests\pdfcer-gui\mockups\glyphs\new.json`. The handoff
said they *"can go straight into `crates/pdfcer-gui/src/icons/assets`"*.

They could not, and this file is the record of what happened instead.

---

## Why an adjudication was needed at all

Five things were true of the delivered set and none of them was visible from the
handoff sentence:

1. **Wrong format.** The assets here are individual `.svg` files behind
   `include_str!`, one `Icon` variant each, with a doc comment that is a *ruling*
   rather than a description. The delivery was one JSON object.
2. **Six glyphs used `stroke-dasharray`, which the parser silently ignored.**
   They parsed, rasterized, passed every icon test we own, and drew the WRONG
   PICTURE — `new-from-template` losing the dashed box that is the only thing
   separating it from `new-document`, `select-all` losing its marquee. Fixed
   first, on 2026-09-04, before any adoption: see the commit *"Dashed strokes
   render as dashes"*.
3. **Two used `fill`, and the fill set is closed by a test with a written reason
   per member.** Both turned out to be redaction-family glyphs and both were
   added to the set deliberately — see `icons::tests::fill_is_semantic_and_the_set_that_uses_it_is_closed`.
4. **None carried the rationale comment the house style requires** — what the
   glyph depicts and which neighbour it was drawn to stay distinguishable from.
   Every adopted asset now does.
5. **`thin-lines` was art for a command deleted six weeks ago.** Verified: there
   is no such field on `RenderOptions` and no such command anywhere in the tree.

---

## ★★★ The rule the adjudication used, and it is the only interesting decision

> **A glyph is adopted only when a command or role in this build would use it
> TODAY. A glyph that restyles an icon we already have is deferred. A glyph with
> no home is deferred.**

Not because the art is bad — most of it is good — but because an `Icon` variant
nobody names is dead weight that a future reader must still evaluate, and a
restyle is a different decision from filling a gap. The operator asked for
neither. **R9's spirit, applied to the icon set: an unavailable capability
renders nothing, and an unused glyph is a capability nobody has.**

Applied honestly, that rule took 36 and left 26. Both halves are recorded below,
because the deferred list is the part that will otherwise be re-proposed.

---

## Adopted — 36 glyphs

Each fills a gap that was **already written down**. Nine discharge a
registration carrying a multi-paragraph "No icon" refusal in prose. The rest
take a role off a glyph it was borrowing — and two of those borrows were
four-way:

| Borrowed glyph | Controls sharing it | Now |
|---|---|---|
| `form-field` | Check box, Radio button, Drop-down, Push button | four glyphs |
| `measure` | Length, Perimeter, Radius/diameter, Angle | four glyphs |
| `redact` | Redact selection, Apply redactions | two glyphs |
| `copy` | Copy page text, Copy document text | two glyphs |
| `fonts` | Embed fonts, Unembed fonts | two glyphs |
| `chevron-left` / `chevron-right` | Previous/next **document** — the same picture as previous/next **page** | two glyphs |

| Key | Home | Was |
|---|---|---|

| `apply-redactions` | `edit.redact_apply` | borrowed a neighbour |
| `attachment` | `edit.attachments` | a written refusal |
| `check` | `measure.finish` | a written refusal |
| `check-box` | `edit.form_check_box` | borrowed a neighbour |
| `close-others` | `view.close_other_documents` | a written refusal |
| `collapse` | `crates/pdfcer-gui/src/text/panels/bookmarks.rs:109` | a written refusal |
| `copy-document-text` | `file.copy_document_text` | borrowed a neighbour |
| `copy-page-text` | `file.copy_page_text` | borrowed a neighbour |
| `dimension-groups` | `measure.manage_groups` | borrowed a neighbour |
| `document-next` | `view.next_document` | borrowed a neighbour |
| `document-previous` | `view.previous_document` | borrowed a neighbour |
| `drop-down` | `edit.form_choice` | borrowed a neighbour |
| `embed-fonts` | `tools.embed_fonts` | borrowed a neighbour |
| `expand` | `crates/pdfcer-gui/src/text/panels/bookmarks.rs:99` | a written refusal |
| `finish-shape` | `markup.finish` | a written refusal |
| `lock` | `crates/pdfcer-gui/src/panels/layers.rs:297` | a written refusal |
| `measure-angle` | `measure.two_line` | borrowed a neighbour |
| `measure-length` | `measure.length` | borrowed a neighbour |
| `measure-perimeter` | `measure.perimeter` | borrowed a neighbour |
| `measure-radius` | `measure.radius_diameter` | borrowed a neighbour |
| `merge` | `pages.merge_into` | borrowed a neighbour |
| `new-document` | `file.new` | a written refusal |
| `new-from-template` | `file.new_from_template` | a written refusal |
| `push-button` | `edit.form_push_button` | borrowed a neighbour |
| `put-down` | `crates/pdfcer-gui/src/panels/tool/armed.rs:642` | a written refusal |
| `radio-button` | `edit.form_radio_button` | borrowed a neighbour |
| `recent` | `file.recent` | a written refusal |
| `recognise-text` | `file.ocr` | a written refusal |
| `redact-selection` | `edit.redact_selection` | borrowed a neighbour |
| `reflow` | `edit.reflow_block` | a written refusal |
| `render-diagnostics` | `tools.render_diagnostics` | borrowed a neighbour |
| `save-as` | `file.save_as` | a written refusal |
| `save-compact` | `file.save_compacted` | a written refusal |
| `save-copy` | `file.save_copy` | a written refusal |
| `unembed-fonts` | `tools.unembed_fonts` | borrowed a neighbour |
| `wheel-flip` | `crates/pdfcer-gui/src/app/status/page_box.rs:234` | a written refusal |

---

## Deferred — 26 glyphs, each with the reason it was not taken

★ **This list is not a rejection of the art.** Most of these become adoptable
the moment the command they name exists, and several name a capability the
roadmap already intends. What they lack today is a home.

Three groups, and the distinction matters when re-reading this:

- **`no-home`** — nothing in this build would draw it. Adopting it would add an
  `Icon` variant nobody names.
- **`has-own`** — the role already has a dedicated glyph. Taking this one would
  be a restyle, which is a separate decision nobody has asked for.
- **`none`** — the role exists and draws no icon, but something other than
  supply is in the way; read the reason.

| Key | Verdict | Why not, in short |
|---|---|---|
| `merge-files` | has-own | DEFERRED — this is the collision trap the brief warned about, wearing a different name. The command already has DEDICATED art: Icon::Combine, whose catalog doc comment is literally "Combine files…. Sc… |
| `move-page-down` | has-own | DEFERRED with its twin, and deferred as a PAIR on purpose. `pages.move_down` (catalog/pages.rs:83) draws with `chevron-down`, whose own doc comment gives its primary role as the menu-disclosure marker… |
| `move-page-up` | has-own | DEFERRED, and this is the trap the briefing warned about wearing a different name. `pages.move_up` (catalog/pages.rs:80) draws with `chevron-up`, and `chevron-up` is NOT another role's art borrowed fo… |
| `bring-forward` | no-home | DEFERRED as dead art, and blocked below the GUI. `edit.bring_forward` is a registered absence whose stated blocker is an engine capability — "needs a content-stream reordering primitive that does not … |
| `eye` | no-home | Every visibility role in the build is already answered, and none of them is answered by a glyph. The two document-visibility controls are native `egui::Checkbox` widgets, which have no icon slot; the … |
| `fill-colour` | no-home | DEFER -- the strongest no-home of the seven. Fill is not a gap this project has; it is a decision this project made AGAINST, recorded in three separate files, and reserved to the operator personally. … |
| `first-page` | no-home | There is NO on-screen first-page control anywhere in the tree. The status bar's page group is `⏴ ⟨n⟩ / ⟨N⟩ ⏵` and nothing else (crates/pdfcer-gui/src/app/status/page_box.rs:167-182); a grep for a firs… |
| `help` | no-home | There is no Help command, no help dialog, and no documentation target anywhere in the build. The group caption at manifest/file.rs:273 uses the WORD help, but the three controls under it are Settings,… |
| `last-page` | no-home | Identical verdict to `first-page`, and for the identical reason: `End` is a keystroke owned by `app::keyboard`, not a command, and the module's own header says so — the viewer navigation keys are "del… |
| `line-width` | no-home | DEFER -- the role is live but has no icon slot, and the ids that would carry a key are absent. The control ships: swatch.rs:31 records "| **Line width** | ✅ | a drag value in points, over the pen's ow… |
| `opacity` | no-home | DEFER -- the control exists, in a surface that draws no icons; the ids that would carry a key do not exist. The live opacity editor is panels/properties/markup.rs:381 `opacity_row`, which is `ui.label… |
| `paste-in-place` | no-home | DEFERRED as dead art. There is no `edit.paste_in_place` command and there is deliberately not going to be one: it sits in the registered-absence table marked "N — and deliberately so, rather than pend… |
| `pin` | no-home | The push-pin's role in a docked application is tear-out / keep-floating, and that capability was UNREGISTERED on 2026-08-17 with its token retired: "`egui-shell`'s dock has no floating mode at all — i… |
| `send-backward` | no-home | DEFERRED with its twin and for its twin's reason: a registered absence pointing at `edit.bring_forward`, which is itself blocked on a content-stream reordering primitive that does not exist in `pdfcer… |
| `stroke-colour` | no-home | DEFER -- no icon-bearing home, and one that was refused in writing. The colour control DOES exist and ships. It is not a command and cannot take an icon. manifest/markup.rs:27-33: "Colour is the only … |
| `thin-lines` | no-home | DEFERRED — VERIFIED DEAD, exactly as warned. There is no such command and no such capability. Three independent records agree: (a) catalog/view.rs:106-122 records all five Render knobs unregistered on… |
| `actual-size` | none | DEFERRED, and the refusal names the delivered shape verbatim. `crates/pdfcer-gui/src/shell/commands/catalog/view.rs:137` registers `view.zoom_actual` with no icon, so state is "none" — but the icon ui… |
| `calculated` | none | DEFERRED for the same structural reason as tab-order, and again the state is reported as it is rather than as the verdict rule would prefer. The role exists: `egui::CollapsingHeader::new(t::recompute_… |
| `flip-pages` | none | Deferred on two independent grounds. (1) DUPLICATE HOME: this glyph and `wheel-flip` are proposals for the SAME single control — the O30 wheel-paging toggle, which is the only place in the build where… |
| `modified` | none | DEFERRED, and this is the one glyph in the batch whose alternative the codebase rejects BY NAME. The unsaved-edits marker on a document tab is a settled copy decision with three stated grounds — ASCII… |
| `note` | none | DEFERRED. The art is a caution triangle with an exclamation, and every candidate home for it is a place this project has already ruled against an icon, in writing. (1) The status bar's rule-4 disclosu… |
| `overflow` | none | DEFERRED, and the state is "none" only in the narrow sense that the control draws text rather than art. The role is not a gap — it is a working, argued, three-surface marker. Three-dots would be a SEC… |
| `reset` | none | DEFERRED, and this one has a SECOND, independent reason that would stand even if the draw site existed. (1) No draw site: the role is `egui::CollapsingHeader::new(t::reset_heading())` — the "Reset to … |
| `select-all` | none | DEFERRED, and the refusal names the delivered shape. `crates/pdfcer-gui/src/shell/commands/catalog/edit.rs:164` registers `edit.select_all` with no icon, so state is "none" and the adopt gate is open … |
| `tab-order` | none | DEFERRED, and the state is reported honestly as "none" rather than bent to fit the verdict: the role genuinely exists and genuinely draws no icon. It is `egui::CollapsingHeader::new(t::tab_order_headi… |
| `zoom-readout` | none | The art is a percent sign — two rings and a slash — and the role it names is the one control in this application with a WRITTEN, ARGUED refusal that anticipated a completeness drive and refused it in … |

### The one that is verified dead

`thin-lines` — art for a command **deleted six weeks ago with evidence**.
`RenderOptions` has no such field. It was carried into the proposal sheet marked
*"DRAWN FOR THIS MOCK"*, which is exactly how a deleted feature comes back: as a
picture of itself in a document nobody re-checked against the code.

---

## How this was decided, and why that is worth recording

Seven agents, one per command domain, each given the same contract: find the
concrete home in this codebase for every glyph in your batch, quote any written
refusal you find, judge `none` / `shared` / `has-own` / `no-home`, and emit the
finished asset only for the ones you would adopt.

Two things came out of that which a single reader would have missed:

- **Two agents independently landed `check` and `finish-shape` on the same
  command**, and both flagged the collision rather than papering over it. The
  settlement — `finish-shape` to `markup.finish` because it closes a drawn
  shape, `check` to `measure.finish` because it accepts a result — is recorded
  at both sites so the next reader does not wonder why two near-identical
  commands took different glyphs.
- **One agent reported that the gate cited by two doc comments
  (`redaction_is_the_only_filled_icon`) did not exist anywhere in the tree.** It
  grepped correctly. The test exists; it was RENAMED on 2026-08-19 when the
  arrow pair joined the filled set, and **four doc comments went on citing the
  dead name for sixteen days.** A rename that leaves its citations behind blinds
  a reader exactly as thoroughly as a deletion. All of them now name the live
  test.

---

## Provenance

`crates/pdfcer-gui/src/icons/assets/PROVENANCE.md` covers this whole directory
and its terms are **unchanged** by this batch: the glyphs are constructed from
primitives — lines, arcs, rectangles — in the same 48×48 / stroke-2.5 / round
caps and joins contract as the rest of the set, drawn for pdfcer, under the
project's MIT grant. No asset was traced, imported or adapted from any icon
pack, vendor mark or screenshot. Metaphor-level resemblance to what Acrobat and
Inkscape use for the same command is intended and is what makes a ribbon legible
to somebody arriving from those tools; asset-level copying is forbidden outright
and none occurred.

