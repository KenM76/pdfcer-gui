use super::*;
use crate::canvas::selection::{AnnotKind, AnnotSelection};
use pdfcer_core::object::ObjId;

fn member(num: u32, x: f32, locked: bool) -> AnnotSelection {
    AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: ObjId::new(num, 0),
            kind: AnnotKind::Markup,
            // ui-text-exempt: a PDF /Subtype name in a test fixture.
            subtype: "Square".to_owned(),
            locked,
        },
        outline: Rect::from_min_size(egui::pos2(x, 20.0), egui::vec2(40.0, 30.0)),
        oriented: None,
    }
}

fn state(members: Vec<AnnotSelection>) -> SelectionState {
    let mut s = SelectionState::default();
    for (i, m) in members.into_iter().enumerate() {
        if i == 0 {
            s.select_annot(m);
        } else {
            s.toggle_annot(m);
        }
    }
    s
}

#[test]
fn one_annotation_is_not_a_group() {
    assert!(Group::annots(&state(vec![member(5, 10.0, false)])).is_none());
}

#[test]
fn every_unlocked_member_gets_a_ghost() {
    let s = state(vec![
        member(5, 10.0, false),
        member(7, 100.0, true),
        member(9, 200.0, false),
    ]);
    let mut actions = Vec::new();
    let ghosts = Group::annots(&s)
        .expect("a group")
        .drag(egui::vec2(3.0, 4.0), Phase::InFlight, None, &mut actions)
        .expect("ghosts");
    assert_eq!(ghosts.len(), 2);
    assert!(ghosts.iter().any(|r| r.min == egui::pos2(13.0, 24.0)));
    assert!(ghosts.iter().any(|r| r.min == egui::pos2(203.0, 24.0)));
    assert!(actions.is_empty());
}

#[test]
fn an_all_locked_group_does_not_drag() {
    let s = state(vec![member(5, 10.0, true), member(7, 100.0, true)]);
    assert!(Group::annots(&s).is_none());
}

#[test]
fn a_release_with_no_page_declines() {
    let s = state(vec![member(5, 10.0, false), member(7, 100.0, false)]);
    let mut actions = Vec::new();
    let out = Group::annots(&s).expect("a group").drag(
        egui::vec2(3.0, 4.0),
        Phase::Complete,
        None,
        &mut actions,
    );
    assert!(out.is_none());
    assert!(actions.is_empty());
}
