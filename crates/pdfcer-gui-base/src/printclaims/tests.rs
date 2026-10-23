//! # `printclaims::tests` — each claim found where it hides, and absent where it is not
#![cfg(test)]

use super::*;
use pdfcer_core::object::Name;

/// Loose objects plus a trailer.
struct Loose {
    objects: Vec<(ObjId, Object)>,
    root: Object,
    info: Option<Object>,
}

impl ObjectGraph for Loose {
    fn value(&self, id: ObjId) -> Option<&Object> {
        self.objects.iter().find(|(o, _)| *o == id).map(|(_, v)| v)
    }
    fn trailer_entry(&self, key: &[u8]) -> Option<&Object> {
        match key {
            b"Root" => Some(&self.root),
            b"Info" => self.info.as_ref(),
            _ => None,
        }
    }
}

fn id(n: u32) -> ObjId {
    ObjId::new(n, 0)
}

fn r(n: u32) -> Object {
    Object::Reference(id(n))
}

fn name(s: &[u8]) -> Object {
    Object::Name(Name(s.to_vec()))
}

fn dict(entries: Vec<(&[u8], Object)>) -> Object {
    let mut d = Dict::new();
    for (k, v) in entries {
        d.insert(Name(k.to_vec()), v);
    }
    Object::Dict(d)
}

/// Page 3 under pages node 2, with `gs` as graphics state 5 drawn from a
/// form 4 when `in_form`, else directly; resources on the PARENT when
/// `inherited`.
fn doc(gs: Object, in_form: bool, inherited: bool, catalog: Object) -> Loose {
    let states = dict(vec![(b"GS0", r(5))]);
    let page_res = if in_form {
        dict(vec![(b"XObject", dict(vec![(b"Fm0", r(4))]))])
    } else {
        dict(vec![(b"ExtGState", states.clone())])
    };
    let form = dict(vec![
        (b"Subtype", name(b"Form")),
        (b"Resources", dict(vec![(b"ExtGState", states)])),
    ]);
    let (node, page) = if inherited {
        (
            dict(vec![(b"Type", name(b"Pages")), (b"Resources", page_res)]),
            dict(vec![(b"Parent", r(2))]),
        )
    } else {
        (
            dict(vec![(b"Type", name(b"Pages"))]),
            dict(vec![(b"Parent", r(2)), (b"Resources", page_res)]),
        )
    };
    Loose {
        objects: vec![
            (id(1), catalog),
            (id(2), node),
            (id(3), page),
            (id(4), form),
            (id(5), gs),
        ],
        root: r(1),
        info: None,
    }
}

fn plain_catalog() -> Object {
    dict(vec![(b"Type", name(b"Catalog"))])
}

#[test]
fn overprint_is_found_directly_inherited_and_inside_a_form() {
    let on = || dict(vec![(b"op", Object::Boolean(true))]);
    for (in_form, inherited) in [(false, false), (false, true), (true, false), (true, true)] {
        let g = doc(on(), in_form, inherited, plain_catalog());
        assert!(
            asks_overprint(&g, &[id(3)]),
            "form={in_form} inherited={inherited}"
        );
    }
}

#[test]
fn overprint_off_or_absent_is_not_a_claim() {
    let off = dict(vec![
        (b"OP", Object::Boolean(false)),
        (b"CA", Object::Real(0.5)),
    ]);
    let g = doc(off, true, true, plain_catalog());
    assert!(!asks_overprint(&g, &[id(3)]));
    assert!(!read(&g, &[id(3)]).pdfx);
}

#[test]
fn a_gts_pdfx_intent_or_the_info_version_is_the_pdfx_claim() {
    let intent = |s: &[u8]| {
        dict(vec![(
            b"OutputIntents",
            Object::Array(vec![dict(vec![(b"S", name(s))])]),
        )])
    };
    let gs = dict(vec![]);
    assert!(claims_pdfx(&doc(
        gs.clone(),
        false,
        false,
        intent(b"GTS_PDFX")
    )));
    assert!(!claims_pdfx(&doc(
        gs.clone(),
        false,
        false,
        intent(b"GTS_PDFA1")
    )));
    let mut g = doc(gs, false, false, plain_catalog());
    g.info = Some(dict(vec![(
        b"GTS_PDFXVersion",
        Object::String(b"PDF/X-4".to_vec()),
    )]));
    assert!(claims_pdfx(&g));
}

#[test]
fn a_form_drawing_itself_terminates() {
    let mut g = doc(dict(vec![]), true, false, plain_catalog());
    g.objects[3].1 = dict(vec![
        (b"Subtype", name(b"Form")),
        (
            b"Resources",
            dict(vec![(b"XObject", dict(vec![(b"Fm0", r(4))]))]),
        ),
    ]);
    assert!(!asks_overprint(&g, &[id(3)]));
}
