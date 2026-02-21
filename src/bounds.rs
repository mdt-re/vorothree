use wasm_bindgen::prelude::*;

// Constants for boundary walls
#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS: &'static str = r#"
export const BOX_ID_BOTTOM = -1;
export const BOX_ID_TOP = -2;
export const BOX_ID_FRONT = -3;
export const BOX_ID_BACK = -4;
export const BOX_ID_LEFT = -5;
export const BOX_ID_RIGHT = -6;
"#;

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
