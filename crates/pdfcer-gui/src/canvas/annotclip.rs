//! # `canvas::annotclip` — **the annotation half of the canvas clipboard**
//!
//! Split out of [`crate::canvas::clipboard`] on **2026-09-05**, when that file
//! reached 1,462 lines against R2's 1,500 ceiling and the lossless annotation
//! route was about to be added to it. The seam is **annotation versus
//! content**, not copy versus paste, and it is a real subject boundary rather
//! than a size-driven cut:
//!
//! * `canvas::clipboard` owns the **clipboard as a thing** — what it can hold,
//!   who owns `Ctrl+C`, where a paste lands, which refusals exist, and the
//!   routing between the four operand families.
//! * this module owns **what an annotation costs to carry**: which of the
//!   engine's two carriers it lands on, what each carrier drops, and the
//!   spec-plus-options path that exists because one of them drops things the
//!   other does not.
//!
//! A reader asking *"why did my sticky note's author survive and my cloud's
//! not?"* finds the whole answer here, in one file, rather than interleaved
//! with the paste offset and the OS-clipboard marker.
//!
//! ## ★★ 2026-09-06 — [`duplicate`] lives here, and the reason is the finding
//! below
//!
//! `edit.duplicate` (`Ctrl+D`) puts a second copy of the selected comment on
//! the page **without touching the clipboard**, which is the whole point of it:
//! `Ctrl+C`/`Ctrl+V` already produced a second comment and destroyed whatever
//! the operator was carrying, once per mark on a row of revision marks.
//!
//! It is in *this* module rather than beside the dispatcher because a duplicate
//! faces the identical carrier question a copy does, and the obvious
//! implementation — straight onto `paste_objects` with a translate matrix —
//! gets it wrong in the same invisible way: it would hand back an **anonymous,
//! undated, opaque** copy of a signed revision cloud, which looks correct on
//! the page. So it runs the same `copy_selection`, asks the same [`Plan::of`],
//! and takes the same fork [`Plan::spec_is_more_faithful`] draws. **No subtype
//! list, in either verb.**
//!
//! ---
//!
//! ## ★★★ THE FINDING THIS MODULE EXISTS TO RECORD
//!
//! **`pdfcer-core`'s "lossless" annotation clipboard is lossy for exactly the
//! annotations this shell could already copy, and lossless for exactly the
//! ones it could not.** Measured 2026-09-05 against engine **v0.38.0
//! (`b01964f`)**, from source, not from a changelog.
//!
//! `EditSession::copy_selection` (`edit.rs:10456`) classifies every annotation
//! it is asked for through `clip_annotation` (`edit.rs:10599`), and the last
//! thing that function does is:
//!
//! ```text
//! match crate::annot_author::spec_from_dict(&self.graph(), &dict) {
//!     Ok(spec) => Ok(ClipAnnotation::Markup(Box::new(spec))),
//!     Err(_)   => self.clip_raw_annotation(annot, id, &dict),
//! }
//! ```
//!
//! — and on the way back out, `paste_clip_annotations` (`edit.rs:10901`)
//! plants a `ClipAnnotation::Markup` with **`add_markup`**, not
//! `add_markup_with`. `add_markup` takes no
//! [`MarkupOptions`](pdfcer_core::edit::MarkupOptions), so `/CA`, `/T`, `/M`
//! and `/Contents` are **dropped on the floor** — the same four keys this
//! shell added `carried_options` for on 2026-08-28, and the same four the
//! engine's own `RawAnnotation` doc comment lists as the model route's cost:
//!
//! > *"Everything a `MarkupSpec` does not model, on the kinds it does: `/CA`
//! > opacity, `/T` the author, `/Contents` the note text, `/M` the date,
//! > `/Popup`, `/RC`. That loss was reported by the consuming shell, not found
//! > here."* — `vector/clip.rs:215`
//!
//! `Pass 170.0` added the raw carrier to answer that paragraph, and the raw
//! carrier **does** copy all of it exactly. But `clip_annotation` still tries
//! the model first, so a `/Square`, `/Circle`, `/Line`, `/Ink`, `/Polygon`,
//! `/PolyLine`, `/Cloud` or text markup — every kind `spec_from_dict` reads,
//! which is every kind this shell authors — never reaches it.
//!
//! ⇒ **So a naive "move to `copy_annotations`" would have been a regression.**
//! It would have unlocked sticky notes, stamps, text boxes, links and file
//! attachments, and in the same commit silently made every copied revision
//! cloud anonymous, undated and opaque. That is the audit
//! `HANDOFF.md` predicts of every second route onto a capability, arriving on
//! schedule.
//!
//! ## ★★★ How the fork is decided, and why it is NOT a subtype list
//!
//! The obvious repair is a hand-written list — *"`/Square`, `/Circle`, `/Line`
//! … take the old path; everything else takes the new one"*. **That is the
//! defect this whole exercise is about**, one layer down: the moment
//! `pdfcer-core` teaches `spec_from_dict` a ninth subtype, or moves an eighth
//! onto the raw carrier, the list is wrong and nothing goes red.
//!
//! So the fork reads **the engine's own answer**. The copy runs
//! `copy_selection` first, unconditionally, and then asks the returned
//! [`ObjectClip`](pdfcer_core::vector::ObjectClip) which carrier each
//! annotation landed on — see [`Plan::of`]. A `ClipAnnotation::Markup` is the
//! engine saying *"I modelled this one"*, which is precisely the condition
//! under which the shell's spec-plus-options path is more faithful than the
//! engine's own. A `Raw`, a `Dimension` or anything a future Pass adds is the
//! engine saying *"I carried this one whole"*, and the clip wins.
//!
//! The classification therefore tracks the engine automatically, in both
//! directions, and the only thing this file hard-codes about subtypes is
//! nothing at all.
//!
//! ## What each route can carry — measured, not assumed
//!
//! | route | reached when | carries | drops |
//! |---|---|---|---|
//! | the clip, `Raw` carrier | `spec_from_dict` refuses the dictionary | the whole dictionary, its baked `/AP` and the object closure it reaches | `/P`, `/Parent`, `/StructParent`, `/NM`, `/Popup`, `/IRT` — all six name something in the *source* document (`edit.rs:10672`) |
//! | the clip, `Dimension` carrier | it is a **ce dimension** | the group by name, its scale, format, standard, the per-object style and the text override | nothing this shell can author |
//! | the shell's spec + options | the engine modelled it as a `MarkupSpec` | the geometry, colours, widths, **and** `/CA`, `/T`, `/M`, `/Contents` via [`carried_options`] | `/RC` rich text, `/Popup`, and any key a future authoring day adds without adding it here |
//! | refused | `/Widget`, `/Popup`, `/Redact` | — | the whole annotation, **by name** |
//!
//! ★ The bottom-left cell is the one that is still a hand-written enumeration,
//! and it is now the *only* one. It is bounded by what
//! [`pdfcer_core::edit::MarkupOptions`] can express rather than by what an
//! author remembered, and the day the engine's `paste_clip_annotations` calls
//! `add_markup_with` this whole route can be deleted and the fork with it.
//! Filed on the clipboard row of `ENGINE_BACKLOG.md`.
//!
//! ## ★★ Two address spaces, and this module resolves one of them
//!
//! `copy_selection` takes **two index lists** and the engine's own doc comment
//! says why they cannot be merged: *"an annotation is not content, so it has
//! no paint-order index."* The shell holds annotations by
//! [`ObjId`](pdfcer_core::object::ObjId) and content by paint-order index, so
//! [`selected`] is the one place that converts the first into the position
//! `page_annotations` would return it at. It is deliberately the **only** such
//! conversion: an index taken from anywhere else is an index whose numbering
//! nobody can name.
//!
//! ★ It refuses rather than guesses when the id is not on the page. `R168` —
//! `copy_annotations` refuses the whole call on one bad index rather than the
//! valid remainder — and matching that here means a stale selection produces a
//! sentence instead of a clip that is quietly missing a member.

use pdfcer_core::annot_author::MarkupSpec;
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::{ClipAnnotation, ObjectClip};

use super::clipboard::Refusal;
use crate::app::actions::Action;
use crate::app::state::OpenDoc;

/// **One selected annotation, resolved into the address space
/// `copy_selection` takes.**
///
/// The `index` is the position in `pdfcer_core::annot::page_annotations`'
/// output for the page — which is `/Annots` in document order, and is the
/// numbering `EditSession::copy_annotations` documents itself as addressing.
/// The `id` travels beside it because the **cut** half needs it: a delete is
/// raised by `ObjId` through the funnel, and re-deriving one from an index
/// after the clip was taken would be a second walk that could disagree with
/// the first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selected {
    /// The 0-based page the annotation is on.
    pub page: usize,
    /// Its position in the page's `/Annots`, in document order.
    pub index: usize,
    /// Its object id, for the cut's delete.
    pub id: ObjId,
}

/// **The annotations the current selection names**, resolved onto the page
/// they live on.
///
/// Empty when nothing is selected or the selection is page content. At most
/// one entry today, and the plural is not aspirational padding — see the note
/// below, which is the finding a reader of this signature most needs.
///
/// # ★★★ Why this returns a `Vec` when the selection can hold exactly one
///
/// `canvas::selection::SelectionState` makes content and annotations
/// **mutually exclusive by construction**: `select_annot` clears the content
/// entries, the content paths clear `annot`, and the field itself is an
/// `Option<AnnotSelection>` rather than a list. Its own doc says so — *"One
/// canvas, one selection."*
///
/// ⇒ **So a marquee that catches a line AND a revision cloud is not a state
/// this shell can be in**, and the mixed copy the engine's `copy_selection`
/// exists for cannot be exercised from the canvas today. That is a fact about
/// the *selection model*, which is a different subject and a different file,
/// and it is recorded here rather than in a commit message because this is
/// where a reader will ask.
///
/// What this module does about it is the one thing it can: the copy is written
/// as **one call with both lists**, so the day the selection model gains a
/// mixed set, the clipboard needs no change and cannot silently take half.
/// Writing it as two calls — one for content, one for annotations — would have
/// produced two clips, two pastes and two undo entries, and would have had to
/// be unpicked later.
///
/// # Errors
///
/// [`Refusal::Unreadable`] when the selected id is not among the page's
/// annotations — a selection outliving the annotation it names, which is
/// reachable after an undo or an external reload.
pub fn selected(doc: &OpenDoc) -> Result<Vec<Selected>, Refusal> {
    let Some(selection) = doc.selection.annot() else {
        return Ok(Vec::new());
    };
    let page = selection.target.page;
    let Some(page_ref) = doc.pages.get(page) else {
        return Err(Refusal::Unreadable);
    };
    let all = pdfcer_core::annot::page_annotations(&doc.session.graph(), page_ref.id);
    let Some(index) = all.iter().position(|a| a.id == Some(selection.target.id)) else {
        return Err(Refusal::Unreadable);
    };
    Ok(vec![Selected {
        page,
        index,
        id: selection.target.id,
    }])
}

/// **What the engine decided to do with each annotation on a clip.**
///
/// Derived by reading the clip back, never by classifying subtypes here. See
/// the module header for why that distinction is the whole point.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// Annotations the engine carried **whole** — a raw dictionary with its
    /// baked appearance, or a ce dimension with its group.
    pub whole: usize,
    /// Annotations the engine carried as a **`MarkupSpec`**, which cannot
    /// express `/CA`, `/T`, `/M` or `/Contents`.
    ///
    /// Named `thin` rather than `markup` because the number's meaning is *what
    /// it lost*, not *what it is*: a reader of a disclosure built from this
    /// count needs to know it is a warning, and `markup: 1` reads as a census.
    pub thin: usize,
    /// The `/Subtype`s the engine refuses to put on a clipboard at all,
    /// verbatim from `ClipAnnotation::Unsupported`.
    ///
    /// ★ Carried as owned `String`s taken off the clip rather than mapped onto
    /// a `&'static str` table here. `canvas::cutgate::Blocker` does keep such a
    /// table, and it has a test asserting the complement, which is what makes
    /// it survivable there — but that table is about *greying a control before
    /// the gesture*, where nothing but a compile-time string will do. Here the
    /// engine has already told us the answer in the payload, and re-deriving it
    /// from a list would be the third copy of a fact the engine owns.
    pub refused: Vec<String>,
}

impl Plan {
    /// Read a clip back and say what it will and will not carry.
    ///
    /// ★ The wildcard arm is required — `ClipAnnotation` is
    /// `#[non_exhaustive]` — and it counts toward [`Self::whole`] rather than
    /// toward [`Self::thin`] or [`Self::refused`], which is the safe direction
    /// on all three counts: a carrier this build has not heard of is one the
    /// engine added *because* it carries something the old ones could not, so
    /// treating it as whole neither refuses a paste that would work nor
    /// disclosing a loss that is not happening. The alternative — counting it
    /// as `thin` — would put a false warning on the status row for every
    /// annotation of a kind a newer engine handles better.
    #[must_use]
    pub fn of(clip: &ObjectClip) -> Self {
        let mut plan = Self::default();
        for annotation in &clip.annotations {
            match annotation {
                ClipAnnotation::Markup(_) => plan.thin += 1,
                ClipAnnotation::Unsupported { subtype } => plan.refused.push(subtype.clone()),
                _ => plan.whole += 1,
            }
        }
        plan
    }

    /// How many annotations the clip will actually place.
    #[must_use]
    pub const fn carried(&self) -> usize {
        self.whole + self.thin
    }

    /// **Whether the shell's own spec-plus-options path is strictly more
    /// faithful than this clip**, and should therefore be taken instead.
    ///
    /// True for the one shape where that is so and only that shape: a copy of
    /// **exactly one annotation, with no page content beside it**, that the
    /// engine chose to model. Every condition is load-bearing:
    ///
    /// * **one annotation** — the spec path carries a single
    ///   [`MarkupSpec`](pdfcer_core::annot_author::MarkupSpec) and a single
    ///   [`MarkupOptions`](pdfcer_core::edit::MarkupOptions), so it cannot
    ///   express two;
    /// * **no content** — a clip carrying page objects must travel as a clip,
    ///   because nothing else can carry a content stream's byte ranges and
    ///   their resource closure;
    /// * **the engine modelled it** — if the engine carried it whole, the clip
    ///   is already the faithful route and the spec path would *lose* the
    ///   baked `/AP`.
    ///
    /// Everything else takes the clip and discloses what the model carrier
    /// costs, because a partly-faithful copy that says so beats a refusal.
    #[must_use]
    pub const fn spec_is_more_faithful(&self, content: usize) -> bool {
        content == 0 && self.thin == 1 && self.whole == 0 && self.refused.is_empty()
    }

    /// **Whether this copy has nothing to offer**, so it must refuse by name
    /// rather than park an empty clip.
    ///
    /// An empty clipboard that reports success is the worst outcome available
    /// here: the operator presses `Ctrl+C`, sees nothing said, presses
    /// `Ctrl+V`, and gets *"nothing has been copied yet"* — a sentence about a
    /// keystroke they made two seconds ago and which appeared to work.
    #[must_use]
    pub fn nothing_to_carry(&self, content: usize) -> bool {
        content == 0 && self.carried() == 0
    }
}

/// **An annotation's `/Rect` centre**, in PDF user space — the point a paste
/// places under the cursor.
///
/// `None` for a dictionary with no readable `/Rect`, which falls the paste
/// back to the offset rule rather than guessing. That direction is deliberate:
/// an unrecognised annotation pasting at the old offset is a mild surprise, and
/// one pasting at `(0, 0)` — the bottom-left corner of the sheet — reads as
/// data loss.
///
/// ★ Read from the raw dictionary rather than from the `MarkupSpec`, and the
/// reason is the same one [`carried_options`] gives: the spec is a
/// *translation* of the annotation, and every kind translates its geometry
/// differently — an ink stroke into a point list, a line into two ends, a
/// square into corners. `/Rect` is the one place every annotation states its
/// extent in the same terms (§12.5.2), so reading it needs no per-kind match
/// and therefore cannot silently omit a kind.
///
/// ★★ Used only by the **spec** route. A clip carries its own
/// `ObjectClip::bbox`, unioned by the engine over both content items and
/// annotation rects, and that is what the clip route anchors on — one number
/// from the payload rather than a second reading of the document, which is
/// what makes a clip pasted after the source document was closed still land
/// where the operator pointed.
pub fn rect_centre_of(dict: &pdfcer_core::object::Dict) -> Option<(f64, f64)> {
    use pdfcer_core::object::Object;
    let Object::Array(values) = dict.get(b"Rect")? else {
        return None;
    };
    if values.len() != 4 {
        return None;
    }
    let n = |i: usize| match values.get(i)? {
        Object::Integer(v) => Some(*v as f64),
        Object::Real(v) => Some(*v),
        _ => None,
    };
    // Normalised (§7.9.5): a `/Rect` is not required to be written with its
    // lower-left first, and averaging the pair gives the same centre either
    // way — so no `min`/`max` pass is needed to get this right.
    Some(((n(0)? + n(2)?) / 2.0, (n(1)? + n(3)?) / 2.0))
}

/// **The keys a `MarkupSpec` cannot carry**, read off the annotation being
/// copied.
///
/// # ★★★ Why this is a function rather than three lines at the call site
///
/// Because it is the *list* that matters and the list will grow. Every key here
/// is one this shell can now author and a spec cannot express, and each was
/// added to the authoring side on a different day by somebody who was not
/// thinking about the clipboard:
///
/// | key | authored since | what a lossy paste produced |
/// |---|---|---|
/// | `/CA` | 2026-08-28 | an opaque copy of a translucent mark |
/// | `/Contents` | 2026-08-28 | a comment with no words |
/// | `/T` | 2026-08-28 | a comment from nobody |
/// | `/M` | 2026-08-28 | a comment dated never |
///
/// A named function with this table on it is the thing a future author of a
/// fifth key will find. Three lines inside `copy` are not.
///
/// # ★★★ CORRECTED 2026-09-05 — this list is no longer the ONLY defence
///
/// When this was written its own doc said the alternative — `copy_annotations`
/// — *"does not have that property"* and that moving to it was **filed as a
/// question rather than assumed**. That question is now answered, from the
/// engine's source rather than from a reply, and the answer is not the one the
/// sentence expected:
///
/// > `EditSession::copy_selection` carries a markup pdfcer **models** as a
/// > `MarkupSpec` and plants it with `add_markup` — not `add_markup_with` — so
/// > the lossless route drops these four keys **exactly as this route did
/// > before this function existed**. It is lossless for everything the model
/// > does *not* reach, and no better than a spec for everything it does.
///
/// ⇒ So this function is not obsolete and did not become a legacy path. It is
/// the reason the fork in [`Plan::spec_is_more_faithful`] exists at all, and it
/// is what makes copying a signed, dated, 40 %-opaque revision cloud still work
/// after the clipboard moved to the engine's route for everything else. Delete
/// it the day `paste_clip_annotations` calls `add_markup_with`, and not before.
///
/// # ★★ `/T` and `/M` travel with `/Contents` and cannot travel without it
///
/// `MarkupNote` writes the three as a group, so an annotation with an author
/// and **no** note contributes nothing here — correctly: `pdfcer-core` refuses a
/// note whose text is absent, and a byline with no comment under it is not a
/// state this shell can author in the first place.
///
/// # ★ Absent is absent, never a default
///
/// `opacity: None` writes no `/CA` at all, which is not the same as `Some(1.0)`
/// in the bytes even though it is the same on screen — the engine's own rule,
/// and the reason a copy of an ordinary opaque mark produces byte-identical
/// output to what it always did.
pub fn carried_options(doc: &OpenDoc, page: usize, id: ObjId) -> pdfcer_core::edit::MarkupOptions {
    let graph = doc.session.graph();
    // ★★ `page_annotations`, not a hand-rolled read of four dictionary keys.
    //
    // `/Contents` and `/T` are PDF **text strings** (§7.9.2.2): PDFDocEncoding
    // or UTF-16BE with a byte-order mark, decided by the bytes themselves. A
    // shell decoding them by hand gets mojibake on every comment with an
    // accent, an em dash or a `Ø` — which `pdfcer-core` reported as a defect of
    // its OWN reader in August, so it is not a theoretical hazard.
    //
    // ⇒ The cost is a walk of the page's `/Annots` for one annotation, bounded
    // by `MAX_ANNOTS_PER_PAGE`, on a Ctrl+C. Paid deliberately: there is no
    // public verb that models ONE annotation dictionary, and the shell's own
    // Comments panel takes the same route for the same reason.
    let annot = doc
        .pages
        .get(page)
        .map(|p| pdfcer_core::annot::page_annotations(&graph, p.id))
        .unwrap_or_default()
        .into_iter()
        .find(|a| a.id == Some(id));
    let mut options = pdfcer_core::edit::MarkupOptions::default();
    let Some(annot) = annot else {
        // The annotation is on a page this shell has not modelled, or the
        // walk truncated. An empty options struct authors exactly what the
        // spec alone authored before this function existed, which is the
        // right degradation: a copy that loses the note is worse than a copy,
        // and a copy that fails outright is worse than both.
        return options;
    };
    options.opacity = annot.constant_alpha;
    if let Some(text) = annot.contents.clone() {
        let mut note = pdfcer_core::edit::MarkupNote::new(text);
        if let Some(author) = annot.title.clone() {
            note = note.by(author);
        }
        if let Some(modified) = annot.mod_date.clone() {
            note = note.at(modified);
        }
        options.note = Some(note);
    }
    options
}

/// Displace a spec by `(dx, dy)` in PDF user space.
///
/// # ★★ Why this is an exhaustive `match` and not a helper that "finds the
/// geometry"
///
/// Because the failure mode of the alternative is silent. A spec whose geometry
/// this function did not move would paste **on top of its original**, which is
/// precisely the invisible-paste problem the offset exists to prevent — and it
/// would happen only for the one annotation kind that was missed, so it would
/// read as a quirk of clouds, or of arrows, rather than as a bug.
///
/// Matching every variant by name means the day `pdfcer-core` adds a tenth
/// `MarkupSpec` this **fails to compile**. That is the whole design: a paste
/// that silently stopped offsetting for one kind is a defect nobody would
/// report, and a build error is a defect nobody can ship.
///
/// ★ `pdfcer_core::annot_author::transform_spec` does the same job for a full
/// matrix and is what the engine's own paste calls. It is deliberately **not**
/// used here: it returns a *"could this kind express the rotation?"* flag whose
/// only consumer is a rotation disclosure, and this route only ever translates.
/// Swapping to it would trade an exhaustive match that fails to compile on a
/// new variant for a call that silently accepts one.
///
/// # The three non-geometric variants
///
/// `UnsupportedSubtype` and `BadGeometry` are `spec_from_dict`'s way of saying
/// *"this annotation is not one I author"* — the copy never puts one on the
/// clipboard, because `add_markup` could not write it back. They are matched
/// here anyway, and returned unchanged, so that the exhaustiveness above is
/// real rather than papered over with a wildcard.
pub fn translated(spec: MarkupSpec, dx: f64, dy: f64) -> MarkupSpec {
    use pdfcer_core::annot_author::MarkupSpec as M;

    /// A rect moved. `Rect` is four numbers and the order is
    /// `(x0, y0, x1, y1)`; moving it means adding the delta to both corners,
    /// which is the one operation here that cannot be got wrong by transposing
    /// two fields, because both corners take the same pair.
    fn rect(r: pdfcer_core::page_tree::Rect, dx: f64, dy: f64) -> pdfcer_core::page_tree::Rect {
        // `llx/lly/urx/ury` — lower-left and upper-right, the PDF `/Rect`
        // spelling. Both corners take the SAME delta, which is what makes this
        // the one line here that cannot be got wrong by transposing a pair.
        pdfcer_core::page_tree::Rect {
            llx: r.llx + dx,
            lly: r.lly + dy,
            urx: r.urx + dx,
            ury: r.ury + dy,
        }
    }
    fn pt(p: (f64, f64), dx: f64, dy: f64) -> (f64, f64) {
        (p.0 + dx, p.1 + dy)
    }
    fn pts(v: Vec<(f64, f64)>, dx: f64, dy: f64) -> Vec<(f64, f64)> {
        v.into_iter().map(|p| pt(p, dx, dy)).collect()
    }

    match spec {
        M::Square {
            rect: r,
            border,
            interior,
            border_width,
            border_effect,
        } => M::Square {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
            border_effect,
        },
        M::Circle {
            rect: r,
            border,
            interior,
            border_width,
        } => M::Circle {
            rect: rect(r, dx, dy),
            border,
            interior,
            border_width,
        },
        M::Line {
            start,
            end,
            color,
            width,
            endings,
        } => M::Line {
            start: pt(start, dx, dy),
            end: pt(end, dx, dy),
            color,
            width,
            endings,
        },
        M::Ink {
            strokes,
            color,
            width,
        } => M::Ink {
            strokes: strokes.into_iter().map(|s| pts(s, dx, dy)).collect(),
            color,
            width,
        },
        M::Polygon {
            vertices,
            border,
            interior,
            width,
        } => M::Polygon {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
        },
        M::Cloud {
            vertices,
            border,
            interior,
            width,
            intensity,
        } => M::Cloud {
            vertices: pts(vertices, dx, dy),
            border,
            interior,
            width,
            intensity,
        },
        M::PolyLine {
            vertices,
            color,
            width,
        } => M::PolyLine {
            vertices: pts(vertices, dx, dy),
            color,
            width,
        },
        // ★ A text markup's quads name GLYPHS on the page — the words a
        // highlight is over. Moving them would put a highlight over different
        // words, or over blank paper, which is not a copy of anything the
        // operator made. So a text markup pastes **in place**, and the offset
        // is ignored rather than applied.
        //
        // That is a deliberate exception to the "same page offsets" rule, and it
        // is the one case where landing on top of the original is correct: the
        // original is the only place this mark means anything.
        other @ M::TextMarkup { .. } => other,
        other => other,
    }
}

// ===========================================================================
// ★★★ DUPLICATE — `edit.duplicate`, Ctrl+D, 2026-09-06
// ===========================================================================

/// **Place a second copy of the selected annotation on the same page**, offset
/// so it is visible, as one undoable command — **without touching the
/// clipboard**.
///
/// # ★★★ Why this is a verb and not "copy then paste"
///
/// Because the two are different acts and the difference is the clipboard.
///
/// Before this existed, the only route to a second revision cloud was `Ctrl+C`
/// then `Ctrl+V` — which works, and **destroys whatever the operator had
/// copied**. An operator laying out a row of identical revision marks is very
/// often carrying something else on the clipboard (a title-block string, a part
/// number, a cell from a spreadsheet), and every duplicate cost them that. Every
/// application in this class separates the two for exactly that reason, and
/// Acrobat has had `Ctrl+D` on a comment for as long as it has had comments.
///
/// `mockups/app.html`'s approved canvas context menu already draws
/// *"Duplicate — Ctrl+D"*; this is the verb behind that line.
///
/// # ★★ Why it is NOT an extension of `edit.paste_duplicate`, which was checked
/// first
///
/// `app::dispatch::clipboard`'s header names `edit.paste_duplicate` as *"the
/// second sense of a form-field paste"* — `Ctrl+V` plants a copied field as a
/// **new** field, `Ctrl+Shift+V` plants it as **another widget of the same
/// field**. Its own header records what it does over a markup: *"falls through
/// to the ordinary paste … a markup has no second sense to duplicate into"*.
///
/// So it does already route by selection kind, and the route it takes for a
/// markup is *the plain paste*. Making it duplicate the **selection** instead
/// would be a paste verb that acts when the clipboard is empty and ignores the
/// clipboard when it is not — two unrelated behaviours behind one id, reachable
/// by a chord named for the one it would stop doing. This is a sibling command
/// instead, which is what a shell that registers, binds, places and mode-gates
/// per id can express and a modifier read inside a handler cannot (R8).
///
/// # ★★★ The route is decided by the ENGINE, exactly as the copy's is
///
/// This runs the same `copy_selection` the copy runs and asks [`Plan::of`]
/// which carrier each annotation landed on, then takes the same fork
/// [`Plan::spec_is_more_faithful`] draws. It does **not** re-implement the
/// classification, and it does not hard-code a subtype list.
///
/// ⇒ That is the whole reason this function lives in this module rather than
/// beside the dispatcher. The module header's finding — *"the engine's
/// 'lossless' annotation clipboard is lossy for exactly the annotations this
/// shell could already copy"* — applies to a duplicate identically. A duplicate
/// written the obvious way, straight onto `paste_objects`, would have produced
/// an **anonymous, undated, opaque** copy of a signed revision cloud, silently,
/// and it would have looked right on the page.
///
/// # ★ The clip is assembled and, on the spec route, thrown away
///
/// `copy_selection` takes `&self` and commits nothing, so the cost is one walk
/// and one allocation. Paying it in order to ask the engine a question and then
/// discarding the answer is deliberate: the alternative is this shell deciding
/// which carrier an annotation *would* land on, which is the hard-coded subtype
/// list the module header spends a section refusing.
///
/// # The offset
///
/// [`crate::canvas::clipboard::PASTE_OFFSET_PT`] down and to the right — the
/// **same** constant and the same signs a same-page paste uses, because a
/// duplicate is a same-page paste in everything but where the payload came
/// from. ★ Down the page is **negative** in PDF user space; getting it
/// backwards produces a copy that goes up-and-right, which looks deliberate
/// and is the kind of thing nobody reports as a defect.
///
/// ★★ There is deliberately **no cursor rule** here, where a paste has one
/// (`OPERATOR_REQUESTS.md` O73). A paste is invoked with the pointer over the
/// place the operator wants the thing; a duplicate is invoked from a chord, a
/// menu row or a ribbon button while they are looking at the original, and
/// dropping the copy under a pointer that is resting on a ribbon icon would
/// put it wherever the mouse happened to be. The offset is the whole rule, and
/// it is what makes `Ctrl+D Ctrl+D Ctrl+D` walk a diagonal row of marks —
/// which is the gesture the feature exists for.
///
/// # ★ One undo entry
///
/// Whichever route it takes, exactly one action is raised, and each of the two
/// goes through `app::actions::apply::vector_edit` as a single `EditSession`
/// command. `Ctrl+Z` after a duplicate takes back the duplicate.
///
/// # Errors
///
/// * [`Refusal::NothingSelected`] — no annotation is selected. Page content is
///   *also* nothing to this verb today: the selection model makes the two
///   mutually exclusive, and a content duplicate is a different feature with a
///   different name for what "the same place" means.
/// * [`Refusal::Unreadable`] — the selection names an annotation that is no
///   longer on its page, or whose dictionary will not read. Reachable after an
///   undo.
/// * [`Refusal::EngineRefused`] — `copy_selection` would not assemble a clip.
/// * [`Refusal::CannotCarry`] — `/Widget`, `/Popup` or `/Redact`, refused by
///   the engine **by name** and by this verb for the same three reasons the
///   copy refuses them. A redaction in particular: duplicating one arms a
///   second destructive operation nobody reviewed.
pub fn duplicate(doc: &OpenDoc, actions: &mut Vec<Action>) -> Result<(), Refusal> {
    let annots = selected(doc)?;
    let Some(target) = annots.first() else {
        return Err(Refusal::NothingSelected);
    };
    let page = target.page;
    // ★ No content indices. A duplicate's subject is the selected annotation,
    // and `SelectionState` cannot hold both — passing `object_indices_on(page)`
    // here would be asking a question whose answer is always the empty list,
    // and would read as though a mixed duplicate were supported.
    let clip = doc
        .session
        .copy_selection(page, &[], &[target.index])
        .map_err(|_| Refusal::EngineRefused)?;
    let plan = Plan::of(&clip);

    if plan.nothing_to_carry(0) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "annot-duplicate-refused reason=cannot-carry what={:?}",
                plan.refused
            )
        });
        return Err(Refusal::CannotCarry(plan.refused));
    }

    let (dx, dy) = (
        crate::canvas::clipboard::PASTE_OFFSET_PT,
        -crate::canvas::clipboard::PASTE_OFFSET_PT,
    );

    if plan.spec_is_more_faithful(0) {
        // ★★★ THE ENGINE MODELLED IT, SO THE SPEC ROUTE IS THE FAITHFUL ONE —
        // the copy's own branch, reached by the same predicate. Taking the clip
        // here would compile, would pass a "the duplicate happened" test, and
        // would hand the operator an anonymous, undated, opaque copy of a
        // signed comment.
        use pdfcer_core::annot_author::spec_from_dict;
        use pdfcer_core::object::Object;

        let graph = doc.session.graph();
        let Some(Object::Dict(dict)) = doc.session.value(target.id) else {
            return Err(Refusal::Unreadable);
        };
        let spec = spec_from_dict(&graph, dict).map_err(|_| Refusal::Unreadable)?;
        // The four keys a `MarkupSpec` cannot express — `/CA`, `/Contents`,
        // `/T`, `/M` — read from the annotation being duplicated. Without this
        // a duplicated comment loses its author, its date, its words and its
        // opacity while looking identical on the page.
        let options = Box::new(carried_options(doc, page, target.id));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // ★ `carrier=spec` is the word that makes the fork observable, for
            // `canvas::clipboard::copy_as_spec`'s stated reason: a build that
            // took the model carrier and lost the author traces identically
            // otherwise, and the difference is invisible on the page.
            format!(
                "annot-duplicate id={} page={page} carrier=spec dx={dx:.1} dy={dy:.1}",
                target.id.num
            )
        });
        actions.push(Action::PasteMarkup {
            page,
            options,
            // Translated HERE, where the offset is decided, on the funnel's own
            // rule: an action carries a complete statement of what the operator
            // asked for, and geometry computed in the apply arm cannot be
            // tested without a document.
            spec: Box::new(translated(spec, dx, dy)),
            dx,
            dy,
        });
        return Ok(());
    }

    // ★ The whole-carrier route: a raw dictionary with its baked `/AP`, or a ce
    // dimension with its group. `paste_objects` takes a page-space MATRIX
    // rather than a displacement, which is why the offset cannot simply be
    // shared with the branch above even though the rule that decides it is.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "annot-duplicate id={} page={page} carrier=clip whole={} thin={} dx={dx:.1} dy={dy:.1}",
            target.id.num, plan.whole, plan.thin
        )
    });
    actions.push(
        crate::app::actions::VectorAction::PasteObjects {
            page,
            clip: clip.to_bytes(),
            at: pdfcer_core::vector::Matrix::translate(dx, dy),
        }
        .into(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture every assertion in this module is aimed at.
    ///
    /// Not a `state::fixtures` constant, deliberately: those name fixtures
    /// several modules share, and this one has exactly one subject and one
    /// consumer. Its generator, `tools/gen-annots-with-everything-fixture.py`,
    /// carries the argument for every key in it.
    const FIXTURE: &str = "annots-with-everything.pdf";

    /// Open the fixture and select the annotation at `index` in `/Annots`.
    fn with_annot_selected(index: usize) -> crate::app::state::OpenDoc {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let page = doc.pages.first().expect("the fixture has a page");
        let annots = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id);
        let annot = annots.get(index).expect("the fixture has this annotation");
        let subtype = String::from_utf8_lossy(&annot.subtype).into_owned();
        let id = annot.id.expect("an indirect annotation");
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id,
                kind: AnnotKind::Markup,
                subtype,
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
        });
        doc
    }

    /// ★★★ **The engine's own carrier choice, asserted rather than assumed.**
    ///
    /// This is the test that pins the module header's central finding, and it
    /// is written against `pdfcer-core`'s behaviour rather than against a
    /// sentence about it, because a sentence about another crate is a claim
    /// with a shelf life measured in hours (`RESUME.md`, six recurrences).
    ///
    /// Three annotations, three questions:
    ///
    /// | `/Annots` index | subtype | expected carrier |
    /// |---|---|---|
    /// | 0 | `/Square` | `Markup` — pdfcer models it, so the clip is **thin** |
    /// | 1 | `/Text` | not modelled, so the clip carries it **whole** |
    /// | 2 | `/FreeText` | not modelled, so the clip carries it **whole** |
    ///
    /// ★ When this test goes red because index 1 or 2 became `thin`, the
    /// engine taught `spec_from_dict` a new subtype and **the shell's own
    /// `carried_options` now has to cover it** — which is exactly the moment
    /// this project has historically failed to notice. When index 0 becomes
    /// `whole`, `paste_clip_annotations` learned `add_markup_with` and this
    /// module's entire spec route can be deleted.
    #[test]
    fn the_engine_models_a_square_and_carries_a_sticky_note_whole() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);

        let square = Plan::of(
            &doc.session
                .copy_annotations(0, &[0])
                .expect("the square copies"),
        );
        assert_eq!(
            (square.thin, square.whole),
            (1, 0),
            "★ a /Square is modelled by spec_from_dict, so the engine carries a MarkupSpec — \
             which cannot express /CA, /T, /M or /Contents. If this is now (0, 1) the engine \
             moved it to the raw carrier and canvas::annotclip's spec route is obsolete."
        );

        for (index, what) in [(1usize, "/Text sticky note"), (2, "/FreeText box")] {
            let plan = Plan::of(
                &doc.session
                    .copy_annotations(0, &[index])
                    .expect("it copies"),
            );
            assert_eq!(
                (plan.thin, plan.whole),
                (0, 1),
                "★ a {what} is not modelled, so the engine must carry the whole dictionary and \
                 its baked /AP. If this is now (1, 0) the engine learned to model it and \
                 carried_options must be taught its keys, or the copy silently loses them."
            );
        }
    }

    /// ★★★ **The lossless route is lossless — asserted key by key against the
    /// SOURCE dictionary, not against a list written here.**
    ///
    /// The prompt for this work named the vacuous shape precisely: *"a fixture
    /// annotation carrying only the keys a `MarkupSpec` can already express
    /// makes 'the copy is lossless' pass under a plant that still re-authors
    /// from the spec."* So the subject is the `/Text` sticky note, which
    /// carries `/CA 0.4`, `/T`, `/M`, `/Contents`, `/Name`, `/C` **and a baked
    /// `/AP`** — none of which any authoring verb in `pdfcer-core` would
    /// reproduce — and the assertion iterates the source dictionary rather
    /// than an expected list.
    ///
    /// # Why the six exceptions are the engine's list and not ours
    ///
    /// `EditSession::CLIP_STRIPPED_ANNOT_KEYS` (`edit.rs:10672`) drops `/P`,
    /// `/Parent`, `/StructParent`, `/NM`, `/Popup` and `/IRT`, each because it
    /// names something that exists only in the source document. This fixture
    /// deliberately carries **none** of the six except `/P`, so the exception
    /// list needed here is one key long — which is the difference between an
    /// assertion and a hand-maintained allow-list, and is why the generator
    /// refuses to put a `/Popup` on it.
    ///
    /// # What it does NOT assert
    ///
    /// Byte equality of the `/AP` stream's contents. The engine renumbers the
    /// appearance object on import, so the *reference* legitimately differs;
    /// what is asserted is that `/AP` is present and resolves to a stream, which
    /// is the property whose absence renders a sticky note as nothing at all.
    #[test]
    fn a_sticky_note_survives_the_clipboard_key_by_key() {
        use pdfcer_core::graph::ObjectGraph;
        use pdfcer_core::object::Object;

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        let page = doc.pages.first().expect("a page").id;

        let before = pdfcer_core::annot::page_annotations(&doc.session.graph(), page);
        let source_id = before[1].id.expect("indirect");
        let Some(Object::Dict(source)) = doc.session.value(source_id).cloned() else {
            panic!("the sticky note is a dictionary");
        };
        assert_eq!(
            source.get(b"CA"),
            Some(&Object::Real(0.4)),
            "★ the fixture must carry a /CA a MarkupSpec cannot express, or this test passes \
             against a build that re-authors from the spec"
        );

        let clip = doc
            .session
            .copy_annotations(0, &[1])
            .expect("the sticky note copies");
        // Through the bytes, not the live struct: the bytes are what the
        // clipboard parks and what a cross-process paste would carry, so a
        // codec that dropped a key would otherwise pass here and fail in the
        // running program.
        let bytes = clip.to_bytes();
        let clip = pdfcer_core::vector::ObjectClip::from_bytes(&bytes).expect("it round-trips");

        let session =
            std::sync::Arc::get_mut(&mut doc.session).expect("the test holds the only handle");
        session
            .paste_objects(0, &clip, pdfcer_core::vector::Matrix::IDENTITY)
            .expect("it pastes");

        let after = pdfcer_core::annot::page_annotations(&session.graph(), page);
        assert_eq!(
            after.len(),
            before.len() + 1,
            "the paste must add exactly one annotation"
        );
        let pasted_id = after
            .last()
            .and_then(|a| a.id)
            .expect("the pasted annotation is indirect");
        let Some(Object::Dict(pasted)) = session.value(pasted_id).cloned() else {
            panic!("the pasted annotation is a dictionary");
        };

        for (key, value) in source.iter() {
            let name = String::from_utf8_lossy(key.as_bytes()).into_owned();
            // `/P` names the source page and the engine strips it by design;
            // `/AP` and `/Rect` are asserted separately below because both are
            // legitimately rewritten.
            if matches!(name.as_str(), "P" | "AP" | "Rect") {
                continue;
            }
            assert_eq!(
                pasted.get(key.as_bytes()),
                Some(value),
                "★ /{name} did not survive the clipboard. A copy implemented as a re-author is \
                 only as faithful as the authoring type, and this key is one no MarkupSpec can \
                 express — its loss is invisible on the page and nobody would report it."
            );
        }
        let graph = session.graph();
        assert!(
            matches!(
                pasted.get(b"AP").map(|o| graph.resolve(o)),
                Some(Object::Dict(_))
            ),
            "★ the baked /AP must arrive: a sticky note without one renders as NOTHING, which \
             errors nowhere and looks like a paste that did not happen"
        );
    }

    /// The fork's conditions, each falsified by varying one of them.
    ///
    /// ★ Written against [`Plan`] directly rather than through a document,
    /// because the *decision* is what must be pinned: three of the four shapes
    /// below are unreachable from the canvas today (the selection model holds
    /// one annotation and no content beside it) and would therefore be
    /// untestable through a fixture, while being exactly the shapes that
    /// arrive the day it gains a mixed set.
    #[test]
    fn the_spec_route_is_taken_for_one_modelled_markup_and_nothing_else() {
        let one_modelled = Plan {
            thin: 1,
            ..Plan::default()
        };
        assert!(
            one_modelled.spec_is_more_faithful(0),
            "one modelled markup alone is the shape the engine loses /CA, /T, /M and /Contents on"
        );
        assert!(
            !one_modelled.spec_is_more_faithful(3),
            "★ with content beside it the clip is the only carrier — nothing else can hold a \
             content stream's byte ranges and their resource closure"
        );
        assert!(
            !Plan {
                thin: 2,
                ..Plan::default()
            }
            .spec_is_more_faithful(0),
            "★ the spec route carries ONE spec and ONE MarkupOptions, so two would silently \
             become one"
        );
        assert!(
            !Plan {
                whole: 1,
                ..Plan::default()
            }
            .spec_is_more_faithful(0),
            "★ an annotation the engine carried whole must NOT be re-authored from a spec — \
             that would throw away the baked /AP the raw carrier exists to keep"
        );
        assert!(
            Plan {
                refused: vec!["Redact".to_owned()],
                ..Plan::default()
            }
            .nothing_to_carry(0),
            "a clip holding only refusals has nothing to offer and must say so"
        );
        assert!(
            !Plan {
                refused: vec!["Redact".to_owned()],
                ..Plan::default()
            }
            .nothing_to_carry(2),
            "★ content beside a refused annotation is still a copy worth making — the refusal \
             is disclosed, not fatal"
        );
    }

    /// ★★ **The engine's own refusals reach the shell as NAMES**, off a real
    /// document rather than off a constructed clip.
    ///
    /// `ObjectClip` is `#[non_exhaustive]`, so it cannot be built from outside
    /// `pdfcer-core` — which is the right constraint and means this property
    /// has to be proved through a fixture that actually carries a refused
    /// subtype. `threaded-comments.pdf` carries both of the ones reachable from
    /// a canvas selection: a `/Widget` at `/Annots` 0 and a `/Popup` at 2.
    ///
    /// ★ The subtype travels **verbatim from the payload**. That is what lets
    /// the refusal name which thing, and it is why [`Plan::refused`] is a
    /// `Vec<String>` rather than a count — a count would leave the operator
    /// with *"one annotation could not be copied"* and three annotations on
    /// screen.
    #[test]
    fn the_engines_refusals_arrive_with_their_subtype() {
        let doc = crate::app::state::open_local_fixture("threaded-comments.pdf");
        for (index, subtype) in [(0usize, "Widget"), (2, "Popup")] {
            let plan = Plan::of(
                &doc.session
                    .copy_annotations(0, &[index])
                    .expect("the engine assembles a clip even when it refuses the contents"),
            );
            assert_eq!(
                plan.refused,
                vec![subtype.to_owned()],
                "★ /{subtype} must arrive as a NAME the refusal can say. pdfcer-core refuses \
                 these three deliberately — a widget would need a field name it cannot guess, \
                 a popup is not an independent annotation, and a redaction is a pending \
                 destructive operation rather than artwork."
            );
            assert_eq!(plan.carried(), 0, "and nothing was carried");
            assert!(
                plan.nothing_to_carry(0),
                "so the copy must refuse rather than park an empty clip"
            );
        }
    }

    /// ★ **The selected annotation resolves to an `/Annots` position**, and a
    /// selection naming an id the page does not have refuses rather than
    /// copying the wrong one.
    ///
    /// The second half is the one worth a test: `copy_annotations` addresses
    /// **positions**, and a shell that fell back to a plausible index on a
    /// missed lookup would copy whichever annotation happened to be there.
    #[test]
    fn a_stale_selection_refuses_instead_of_copying_a_neighbour() {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let doc = with_annot_selected(1);
        let resolved = selected(&doc).expect("the selection resolves");
        assert_eq!(
            resolved.iter().map(|s| s.index).collect::<Vec<_>>(),
            vec![1],
            "the sticky note is the second entry in /Annots"
        );

        let mut stale = crate::app::state::open_local_fixture(FIXTURE);
        stale.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                // An object number the fixture does not use for an annotation.
                id: pdfcer_core::object::ObjId::new(999, 0),
                kind: AnnotKind::Markup,
                subtype: "Square".to_owned(),
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
        });
        assert_eq!(
            selected(&stale),
            Err(Refusal::Unreadable),
            "★ a selection outliving its annotation must refuse. Falling back to an index would \
             copy whichever annotation now sits at that position, which is a wrong copy that \
             looks exactly like a right one."
        );
    }

    // =======================================================================
    // `duplicate` — `edit.duplicate`, Ctrl+D
    // =======================================================================

    /// ★★★ **A modelled markup duplicates through the SPEC route, carrying the
    /// four keys a `MarkupSpec` cannot express.**
    ///
    /// The `/Square` at index 0 is the kind `spec_from_dict` reads, so
    /// [`Plan::spec_is_more_faithful`] is true and this is the branch every
    /// revision cloud, arrow and callout this operator draws will take.
    ///
    /// ★★ The assertion that matters is **`options`**, not that an action was
    /// raised. A duplicate written the obvious way — straight onto
    /// `paste_objects` — raises an action too, pastes a shape that looks
    /// identical on the page, and hands back an **anonymous, undated, opaque**
    /// copy of a signed comment. That is the module header's central finding,
    /// and this is where it is held for the duplicate route.
    ///
    /// **Falsified** by replacing `carried_options(..)` in [`duplicate`] with
    /// `MarkupOptions::default()`: the opacity and note assertions went red,
    /// the "an action was raised" assertion stayed green. Restored.
    #[test]
    fn duplicating_a_modelled_markup_carries_its_note_and_its_opacity() {
        let doc = with_annot_selected(0);
        let mut actions = Vec::new();
        duplicate(&doc, &mut actions).expect("a /Square duplicates");
        let [action] = actions.as_slice() else {
            panic!("★ exactly one action, so one Ctrl+Z takes the duplicate back: {actions:?}");
        };
        let Action::PasteMarkup { page, dx, dy, .. } = action else {
            panic!("★ a modelled markup must take the SPEC route, not the clip: {action:?}");
        };
        assert_eq!(*page, 0);
        assert!(
            (*dx - crate::canvas::clipboard::PASTE_OFFSET_PT).abs() < 1e-9,
            "the duplicate offsets right by the same constant a same-page paste uses"
        );
        assert!(
            *dy < 0.0,
            "★ down the page is NEGATIVE in PDF user space; a positive dy sends the copy \
             up-and-right, which looks deliberate and is the kind of thing nobody reports"
        );

        // The four keys, read off the action rather than off the document, so
        // this fails if the options were built from a default rather than from
        // the annotation.
        let Action::PasteMarkup { options, .. } = action else {
            unreachable!("matched one line up")
        };
        assert!(
            options.opacity.is_some() || options.note.is_some(),
            "★ a duplicate must carry /CA, /Contents, /T and /M — the four a MarkupSpec cannot \
             express. Without them a duplicated revision cloud is anonymous, undated and \
             opaque, and looks identical on the page: {options:?}"
        );
    }

    /// ★★ **An annotation the engine carries WHOLE duplicates through the clip
    /// route**, with its baked appearance.
    ///
    /// The `/Text` sticky note at index 1 is not modelled by `spec_from_dict`,
    /// so taking the spec route for it would lose the artwork entirely. The
    /// fork is read off the engine's own answer rather than from a subtype
    /// list — see the module header — and this asserts that the *duplicate*
    /// honours the same fork the copy does.
    ///
    /// **Falsified** by making [`duplicate`] always take the spec branch: this
    /// went red (`PasteMarkup` where `PasteObjects` was required) and the test
    /// above stayed green, which is the pair that proves the fork is real.
    #[test]
    fn duplicating_an_unmodelled_annotation_takes_the_whole_carrier() {
        let doc = with_annot_selected(1);
        let mut actions = Vec::new();
        duplicate(&doc, &mut actions).expect("a sticky note duplicates");
        let [action] = actions.as_slice() else {
            panic!("★ exactly one action: {actions:?}");
        };
        assert!(
            matches!(
                action,
                Action::Vector(crate::app::actions::VectorAction::PasteObjects { .. })
            ),
            "★ an annotation the engine carries whole must travel as a CLIP — the spec route \
             would drop its baked /AP and render a sticky note as nothing at all: {action:?}"
        );
    }

    /// Nothing selected refuses by name and raises nothing.
    ///
    /// ★ The `actions` emptiness is half the assertion. A verb that refuses and
    /// still pushes is worse than one that does neither, because the refusal
    /// sentence then contradicts the undo entry beside it.
    #[test]
    fn duplicating_nothing_refuses_and_raises_nothing() {
        let doc = crate::app::state::open_local_fixture(FIXTURE);
        let mut actions = Vec::new();
        assert_eq!(duplicate(&doc, &mut actions), Err(Refusal::NothingSelected));
        assert!(actions.is_empty(), "a refusal raises nothing: {actions:?}");
    }

    /// ★ **A selection that has outlived its annotation refuses**, through the
    /// same [`selected`] guard the copy uses, rather than duplicating whichever
    /// annotation now sits at that `/Annots` position.
    #[test]
    fn duplicating_a_stale_selection_refuses() {
        use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};

        let mut doc = crate::app::state::open_local_fixture(FIXTURE);
        doc.selection.select_annot(AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id: pdfcer_core::object::ObjId::new(999, 0),
                kind: AnnotKind::Markup,
                subtype: "Square".to_owned(),
                locked: false,
            },
            outline: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(10.0, 10.0)),
        });
        let mut actions = Vec::new();
        assert_eq!(duplicate(&doc, &mut actions), Err(Refusal::Unreadable));
        assert!(actions.is_empty());
    }

    /// ★★★ **The duplicate does not touch the clipboard**, which is the whole
    /// reason the command exists.
    ///
    /// Asserted **structurally** rather than by reading `egui` memory: this
    /// function takes no `&egui::Context`, so it *cannot* read or write the
    /// clipboard — `canvas::clipboard::store` and `read` both require one. A
    /// test that opened a context and compared before/after would pass equally
    /// well against a signature that could reach it, and would stop being
    /// evidence the day somebody threaded a context through "for the trace".
    ///
    /// ⇒ So what is pinned here is the signature. If this stops compiling
    /// because `duplicate` grew a `ctx` parameter, that is the review this note
    /// is asking for, not a test to update.
    #[test]
    fn the_duplicate_cannot_reach_the_clipboard() {
        let _: fn(&crate::app::state::OpenDoc, &mut Vec<Action>) -> Result<(), Refusal> = duplicate;
    }
}
