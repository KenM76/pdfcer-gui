//! # `annotmany` — several annotations resized or turned by one grip drag
//!
//! A grip drag with several markups selected scales or turns them all about
//! the selection box's corner or centre, so the group changes as one shape,
//! in one undo step.

use pdfcer_core::object::ObjId;

/// [`AnnotAction::ResizeMany`](crate::annotaction::AnnotAction::ResizeMany).
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeMany {
    /// The page they are on, for the trace.
    pub page: usize,
    /// The annotations.
    pub ids: Vec<ObjId>,
    /// The point that stays still, in PDF page space: the selection box's
    /// corner opposite the grip grabbed.
    pub anchor: (f64, f64),
    /// Horizontal scale factor, shared.
    pub sx: f64,
    /// Vertical scale factor, shared.
    pub sy: f64,
    /// Whether the two factors are equal.
    pub uniform: bool,
    /// The operator's Tool-row switches.
    pub modifiers: crate::scaling::Modifiers,
    /// Selected annotations that cannot be resized (locked, a ce dimension,
    /// another page), for the status line.
    pub left_out: usize,
}

/// [`AnnotAction::RotateMany`](crate::annotaction::AnnotAction::RotateMany).
#[derive(Debug, Clone, PartialEq)]
pub struct RotateMany {
    /// The page they are on, for the trace.
    pub page: usize,
    /// The markups.
    pub ids: Vec<ObjId>,
    /// The point they turn about, in PDF page space.
    pub pivot: (f64, f64),
    /// Degrees anticlockwise in PDF user space, shared.
    pub degrees: f64,
    /// Selected annotations that cannot be turned with them, for the status
    /// line.
    pub left_out: usize,
}
