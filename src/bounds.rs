/// Bounding box ID for the bottom face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_BOTTOM: i32 = -1;
/// Bounding box ID for the top face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_TOP: i32 = -2;
/// Bounding box ID for the front face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_FRONT: i32 = -3;
/// Bounding box ID for the back face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_BACK: i32 = -4;
/// Bounding box ID for the left face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_LEFT: i32 = -5;
/// Bounding box ID for the right face, it is negative to prevent conflicts with generator IDs.
pub const BOX_ID_RIGHT: i32 = -6;

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
