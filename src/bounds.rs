use wasm_bindgen::prelude::*;

/// Represents an axis-aligned bounding box in 3D space.
///
/// This struct defines the spatial limits for the Voronoi tessellation.
/// All generators and resulting cells are clipped to be within these bounds.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    /// The minimum x-coordinate of the bounding box.
    pub min_x: f64,
    /// The minimum y-coordinate of the bounding box.
    pub min_y: f64,
    /// The minimum z-coordinate of the bounding box.
    pub min_z: f64,
    /// The maximum x-coordinate of the bounding box.
    pub max_x: f64,
    /// The maximum y-coordinate of the bounding box.
    pub max_y: f64,
    /// The maximum z-coordinate of the bounding box.
    pub max_z: f64,
}

#[wasm_bindgen]
impl BoundingBox {
    /// Creates a new `BoundingBox` with the specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `min_x` - The minimum x-coordinate.
    /// * `min_y` - The minimum y-coordinate.
    /// * `min_z` - The minimum z-coordinate.
    /// * `max_x` - The maximum x-coordinate.
    /// * `max_y` - The maximum y-coordinate.
    /// * `max_z` - The maximum z-coordinate.
    #[wasm_bindgen(constructor)]
    pub fn new(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> BoundingBox {
        BoundingBox {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}
