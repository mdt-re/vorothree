use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Function, Array};
use crate::geometries::{PlaneGeometry, SphereGeometry, CylinderGeometry, ConeGeometry, TorusGeometry, TrefoilKnotGeometry, ConvexPolyhedronGeometry};

/// TypeScript constant definitions for wall IDs.
#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS: &'static str = r#"
export const WALL_ID_START = -10;
"#;

/// The starting ID for walls. Wall IDs must be less than or equal to this value
/// to avoid conflicts with non-negative generator IDs and positive boundary IDs.
pub const WALL_ID_START: i32 = -10;

/// Trait defining the geometry and logic of a wall.
/// Must be Send + Sync to support parallel execution in Tessellation.
pub trait WallGeometry: Send + Sync + std::fmt::Debug {
    /// Checks if a point is inside the valid region defined by the wall.
    fn contains(&self, point: &[f64; 3]) -> bool;

    /// Calculates the clipping plane for a given generator.
    /// Returns a tuple (point_on_plane, plane_normal).
    /// The normal should point OUT of the valid region (towards the region to be clipped).
    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3]));
}

/// A clipping boundary for the Voronoi tessellation.
///
/// A `Wall` is a container for a `WallGeometry` implementation, giving it a unique
/// integer ID. This ID will be reported in the `face_neighbors` array of a `Cell`
/// for faces that lie on this wall.
#[wasm_bindgen]
pub struct Wall {
    id: i32,
    inner: Box<dyn WallGeometry>,
}

impl Wall {
    /// Creates a new `Wall` from a Rust struct that implements the `WallGeometry` trait.
    pub fn new(id: i32, geometry: Box<dyn WallGeometry>) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        Wall {
            id,
            inner: geometry,
        }
    }
}

#[wasm_bindgen]
impl Wall {
    /// Creates a new `Wall` from a JavaScript object.
    ///
    /// The JavaScript object must implement the `WallGeometry` interface:
    /// ```typescript
    /// interface JsWall {
    ///   contains(point: [number, number, number]): boolean;
    ///   cut(generator: [number, number, number]): { point: [number, number, number], normal: [number, number, number] } | { point: [number, number, number], normal: [number, number, number] }[];
    /// }
    /// ```
    ///
    /// **Safety**: Using JavaScript walls with parallel execution (`calculate_parallel`) is unsafe unless the host JavaScript environment supports thread-safe calls.
    #[wasm_bindgen(js_name = newCustom)]
    pub fn new_custom(val: JsValue, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        Wall {
            id,
            inner: Box::new(JsWallGeometry { val }),
        }
    }

    /// Returns the unique ID of the wall.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Checks if a 3D point is inside the valid region defined by the wall.
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.inner.contains(&[x, y, z])
    }

    pub fn new_plane(px: f64, py: f64, pz: f64, nx: f64, ny: f64, nz: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(PlaneGeometry::new([px, py, pz], [nx, ny, nz])))
    }

    pub fn new_sphere(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(SphereGeometry::new([cx, cy, cz], radius)))
    }

    pub fn new_cylinder(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, radius: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(CylinderGeometry::new([cx, cy, cz], [ax, ay, az], radius)))
    }

    pub fn new_cone(tx: f64, ty: f64, tz: f64, ax: f64, ay: f64, az: f64, angle: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(ConeGeometry::new([tx, ty, tz], [ax, ay, az], angle)))
    }

    pub fn new_torus(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, major: f64, minor: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(TorusGeometry::new([cx, cy, cz], [ax, ay, az], major, minor)))
    }

    pub fn new_trefoil(cx: f64, cy: f64, cz: f64, scale: f64, tube_radius: f64, resolution: usize, id: i32) -> Wall {
        Wall::new(id, Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, tube_radius, resolution)))
    }

    pub fn new_convex_polyhedron(points: &[f64], normals: &[f64], id: i32) -> Wall {
        Wall::new(id, Box::new(ConvexPolyhedronGeometry::new(points, normals)))
    }

    pub fn new_dodecahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([cx, cy, cz], radius)))
    }

    pub fn new_icosahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall {
        Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_icosahedron([cx, cy, cz], radius)))
    }
}

impl Wall {
    /// Internal method to perform the cut operation, delegating to the `WallGeometry`.
    pub fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        self.inner.cut(generator, callback)
    }
}


/// Wrapper for JS-defined walls.
/// SAFETY: This claims to be Send+Sync to satisfy Rayon, but calling JS from workers is dangerous.
/// Users should ensure they don't use parallel execution if using JS walls, or that the runtime supports it.
struct JsWallGeometry {
    val: JsValue,
}

unsafe impl Send for JsWallGeometry {}
unsafe impl Sync for JsWallGeometry {}

impl std::fmt::Debug for JsWallGeometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JsWallGeometry")
    }
}

impl WallGeometry for JsWallGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        if let Ok(func) = Reflect::get(&self.val, &"contains".into()).and_then(|f| f.dyn_into::<Function>()) {
            let args = Array::of3(&point[0].into(), &point[1].into(), &point[2].into());
            if let Ok(res) = func.apply(&self.val, &args) {
                return res.as_bool().unwrap_or(false);
            }
        }
        false
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        if let Ok(func) = Reflect::get(&self.val, &"cut".into()).and_then(|f| f.dyn_into::<Function>()) {
            let args = Array::of3(&generator[0].into(), &generator[1].into(), &generator[2].into());
            if let Ok(res) = func.apply(&self.val, &args) {
                if res.is_null() || res.is_undefined() { return; }
                
                if Array::is_array(&res) {
                    let arr: Array = res.dyn_into().unwrap();
                    for i in 0..arr.length() {
                        let item = arr.get(i);
                        if let Some((p, n)) = parse_js_cut_result(&item) {
                            callback(p, n);
                        }
                    }
                } else {
                    if let Some((p, n)) = parse_js_cut_result(&res) {
                        callback(p, n);
                    }
                }
            }
        }
    }
}

/// Parses the result from a JavaScript `cut` function call.
fn parse_js_cut_result(val: &JsValue) -> Option<([f64; 3], [f64; 3])> {
    let p = Reflect::get(val, &"point".into()).ok().and_then(|v| v.dyn_into::<Array>().ok())?;
    let n = Reflect::get(val, &"normal".into()).ok().and_then(|v| v.dyn_into::<Array>().ok())?;
    Some(([p.get(0).as_f64()?, p.get(1).as_f64()?, p.get(2).as_f64()?], [n.get(0).as_f64()?, n.get(1).as_f64()?, n.get(2).as_f64()?]))
}
