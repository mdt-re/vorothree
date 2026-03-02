/// Generic bounding box for N-dimensional space.
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox<const D: usize> {
    pub min: [f64; D],
    pub max: [f64; D],
}

impl<const D: usize> BoundingBox<D> {
    pub fn new(min: [f64; D], max: [f64; D]) -> Self {
        Self { min, max }
    }
}

/// Calculates the ID for a bounding box wall based on the axis and direction.
///
/// The IDs start at -1 and decrease.
/// - Axis 0 (X) Min: -1
/// - Axis 0 (X) Max: -2
/// - Axis 1 (Y) Min: -3
/// - Axis 1 (Y) Max: -4
pub fn box_side(axis: usize, is_max: bool) -> i32 {
    -1 - (axis * 2 + if is_max { 1 } else { 0 }) as i32
}