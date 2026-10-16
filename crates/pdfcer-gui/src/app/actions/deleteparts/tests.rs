use super::*;

const A: ObjId = ObjId::new(5, 0);
const B: ObjId = ObjId::new(9, 0);

fn at(dx: f64) -> Matrix {
    Matrix::translate(dx, 0.0)
}

/// Each selected form is one command, in the order it was first selected, its
/// in-form indices unique.
#[test]
fn parts_group_by_form_in_first_seen_order() {
    let selected = [
        Some((B, at(0.0), 3)),
        Some((A, at(0.0), 1)),
        Some((B, at(0.0), 0)),
        Some((B, at(0.0), 3)),
        None,
    ];
    let forms = forms_of(selected.into_iter());
    let got: Vec<(ObjId, Vec<usize>)> = forms.into_iter().map(|f| (f.form, f.indices)).collect();
    assert_eq!(got, vec![(B, vec![3, 0]), (A, vec![1])]);
}

/// A form selected at two placements is one command at the first placement:
/// deleting from a form removes the part wherever the form is drawn, so a
/// second command would address parts already gone.
#[test]
fn a_form_selected_at_two_placements_is_deleted_once() {
    let selected = [Some((A, at(0.0), 2)), Some((A, at(300.0), 4))];
    let forms = forms_of(selected.into_iter());
    assert_eq!(forms.len(), 1);
    assert_eq!(forms[0].placement, at(0.0));
    assert_eq!(forms[0].indices, vec![2, 4]);
}

/// The parts are found again by form, placement and in-form index, wherever
/// an earlier command has moved them in the page's leaf list.
#[test]
fn parts_are_found_again_after_the_leaf_list_renumbers() {
    let parts = FormParts {
        form: B,
        placement: at(0.0),
        indices: vec![0, 3],
    };
    let now = [
        Some((B, at(0.0), 0)),
        None,
        Some((A, at(0.0), 3)),
        Some((B, at(300.0), 3)),
        Some((B, at(0.0), 1)),
        Some((B, at(0.0), 3)),
    ];
    assert_eq!(find(now.into_iter(), &parts), vec![0, 5]);
}
