//! Which of the model's axes points up and which way the Front view looks.
//! Every named view is measured in this pair; a file's own opening view keeps
//! its own axes.

/// One signed model axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

/// Every axis, in the order the choices list them and `t::axis_names` names
/// them.
pub(crate) const ALL: [Axis; 6] = [
    Axis::PosX,
    Axis::PosY,
    Axis::PosZ,
    Axis::NegX,
    Axis::NegY,
    Axis::NegZ,
];

impl Axis {
    /// Its place in [`ALL`].
    pub(crate) fn index(self) -> usize {
        ALL.iter().position(|a| *a == self).unwrap_or(0)
    }

    /// The unit vector it names.
    pub(crate) fn vector(self) -> [f64; 3] {
        let i = self.index();
        let sign = if i < 3 { 1.0 } else { -1.0 };
        std::array::from_fn(|k| if k == i % 3 { sign } else { 0.0 })
    }

    /// Its trace token: `+x` … `-z`.
    pub(crate) fn token(self) -> &'static str {
        ["+x", "+y", "+z", "-x", "-y", "-z"][self.index()]
    }

    /// Whether `other` is a different line through the origin.
    pub(crate) fn perpendicular(self, other: Self) -> bool {
        self.index() % 3 != other.index() % 3
    }

    /// The Front view's look when this axis is up: along +y for z or x up
    /// (the CAD convention), along -z for y up (the convention of y-up
    /// exporters).
    fn default_front(self) -> Self {
        match self {
            Self::PosY | Self::NegY => Self::NegZ,
            _ => Self::PosY,
        }
    }
}

/// The up axis and the Front view's look direction; always perpendicular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Upright {
    pub up: Axis,
    pub front: Axis,
}

impl Default for Upright {
    /// z up, Front looking along +y.
    fn default() -> Self {
        Self {
            up: Axis::PosZ,
            front: Axis::PosY,
        }
    }
}

impl Upright {
    /// `up` as the up axis, keeping the Front look when it is still
    /// perpendicular, else that axis's conventional one.
    pub(crate) fn with_up(self, up: Axis) -> Self {
        let front = if up.perpendicular(self.front) {
            self.front
        } else {
            up.default_front()
        };
        Self { up, front }
    }

    /// `front` as the Front view's look, or unchanged when it lies along up.
    pub(crate) fn with_front(self, front: Axis) -> Self {
        if self.up.perpendicular(front) {
            Self { front, ..self }
        } else {
            self
        }
    }

    /// The Front looks the choice offers: the four perpendicular to up.
    pub(crate) fn fronts(self) -> impl Iterator<Item = Axis> {
        ALL.into_iter().filter(move |a| self.up.perpendicular(*a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_up_keeps_a_front_that_still_fits_and_replaces_one_that_does_not() {
        let start = Upright::default();
        assert_eq!(start.with_up(Axis::PosX).front, Axis::PosY);
        assert_eq!(start.with_up(Axis::PosY).front, Axis::NegZ);
        assert_eq!(start.with_front(Axis::NegZ), start);
        assert_eq!(start.fronts().count(), 4);
    }

    #[test]
    fn every_axis_is_a_signed_unit_vector() {
        assert_eq!(Axis::NegY.vector(), [0.0, -1.0, 0.0]);
        assert_eq!(Axis::PosZ.vector(), [0.0, 0.0, 1.0]);
    }
}
