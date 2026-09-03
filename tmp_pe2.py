import io

p = 'crates/pdfcer-gui/src/panels/objects/provider/mod.rs'
s = io.open(p, encoding='utf-8').read()

# ---- field (the earlier run did not reach this) ---------------------------
if 'page_extent' not in s:
    old = """    page_index: usize,
    objects: PageObjects,
    to_canvas: Transform,
    to_pdf: Option<Transform>,
}"""
    new = """    page_index: usize,
    objects: PageObjects,
    to_canvas: Transform,
    to_pdf: Option<Transform>,
    /// **The page's own extent in canvas units**, or `None` when this provider
    /// was built from parts and nobody supplied one.
    ///
    /// ★★★ Held for exactly one question:
    /// [`crate::canvas::target::CanvasTargetProvider::container_is_worth_selecting`],
    /// which needs to know whether a form covers the whole sheet. It is
    /// `page_device_geometry(page, 1.0)`'s first two returns, which were
    /// discarded here until 2026-09-01 — the transform was wanted and the size
    /// was not.
    ///
    /// ★ `None` makes that predicate answer `true`, which is the behaviour
    /// before it existed. A provider that cannot measure must not guess.
    page_extent: Option<egui::Vec2>,
}"""
    assert old in s, 'struct tail not found'
    s = s.replace(old, new, 1)

# ---- build ----------------------------------------------------------------
old = """        let (_, _, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
        })"""
new = """        let (w, h, to_canvas) = page_device_geometry(page, 1.0);
        Ok(Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // ★ At scale 1.0 these ARE the page's canvas-space extent, which is
            // the space `bounds` answers in. Taken here rather than re-derived
            // from the crop box so the geometry has one source.
            page_extent: Some(egui::vec2(w as f32, h as f32)),
        })"""
assert old in s, 'build tail not found'
s = s.replace(old, new, 1)

# ---- from_parts -----------------------------------------------------------
old = """        Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
        }
    }"""
new = """        Self {
            page_index,
            objects,
            to_canvas,
            to_pdf: to_canvas.invert(),
            // ★ Headless tests construct from parts and have no page. `None`
            // makes `container_is_worth_selecting` answer `true`, which is what
            // those tests were written against — a unit test must not start
            // depending on a geometric judgement it never supplied the geometry
            // for.
            page_extent: None,
        }
    }"""
assert old in s, 'from_parts not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('ok')
