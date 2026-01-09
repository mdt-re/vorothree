use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Function, Array};

/// Trait defining the geometry and logic of a wall.
/// Must be Send + Sync to support parallel execution in Tessellation.
pub trait WallGeometry: Send + Sync + std::fmt::Debug {
    /// Checks if a point is inside the valid region defined by the wall.
    fn contains(&self, point: &[f64; 3]) -> bool;

    /// Calculates the clipping plane for a given generator.
    /// Returns a tuple (point_on_plane, plane_normal).
    /// The normal should point OUT of the valid region (towards the region to be clipped).
    fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])>;
}

#[derive(Debug)]
struct PlaneGeometry {
    point: [f64; 3],
    normal: [f64; 3], // Points IN (towards valid region)
}

impl WallGeometry for PlaneGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.point[0];
        let dy = point[1] - self.point[1];
        let dz = point[2] - self.point[2];
        (dx * self.normal[0] + dy * self.normal[1] + dz * self.normal[2]) >= 0.0
    }

    fn cut(&self, _generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        // For a plane wall, the cut is the plane itself.
        // Our normal points IN, but clip expects normal pointing OUT.
        Some((self.point, [-self.normal[0], -self.normal[1], -self.normal[2]]))
    }
}

#[derive(Debug)]
struct SphereGeometry {
    center: [f64; 3],
    radius: f64,
}

impl WallGeometry for SphereGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        (dx * dx + dy * dy + dz * dz) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist == 0.0 { return None; }

        // Project generator to sphere surface
        let scale = self.radius / dist;
        let px = self.center[0] + dx * scale;
        let py = self.center[1] + dy * scale;
        let pz = self.center[2] + dz * scale;

        // Normal at surface pointing OUT of sphere (away from center)
        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;

        Some(([px, py, pz], [nx, ny, nz]))
    }
}

#[derive(Debug)]
struct CylinderGeometry {
    center: [f64; 3],
    axis: [f64; 3],
    radius: f64,
}

impl WallGeometry for CylinderGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        
        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];

        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        if dist == 0.0 { return None; }

        // Project to cylinder surface
        let scale = self.radius / dist;
        let px = self.center[0] + dot * self.axis[0] + perp_x * scale;
        let py = self.center[1] + dot * self.axis[1] + perp_y * scale;
        let pz = self.center[2] + dot * self.axis[2] + perp_z * scale;

        // Normal pointing OUT (away from axis)
        let nx = perp_x / dist;
        let ny = perp_y / dist;
        let nz = perp_z / dist;

        Some(([px, py, pz], [nx, ny, nz]))
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

    fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        if let Ok(func) = Reflect::get(&self.val, &"cut".into()).and_then(|f| f.dyn_into::<Function>()) {
            let args = Array::of3(&generator[0].into(), &generator[1].into(), &generator[2].into());
            if let Ok(res) = func.apply(&self.val, &args) {
                if res.is_null() || res.is_undefined() { return None; }
                
                let get_arr = |key: &str| -> Option<[f64; 3]> {
                    let v = Reflect::get(&res, &key.into()).ok()?;
                    let arr: Array = v.dyn_into().ok()?;
                    Some([arr.get(0).as_f64()?, arr.get(1).as_f64()?, arr.get(2).as_f64()?])
                };

                let p = get_arr("point")?;
                let n = get_arr("normal")?;
                return Some((p, n));
            }
        }
        None
    }
}

#[wasm_bindgen]
pub struct Wall {
    id: i32,
    inner: Box<dyn WallGeometry>,
}

#[wasm_bindgen]
impl Wall {
    pub fn new_plane(px: f64, py: f64, pz: f64, nx: f64, ny: f64, nz: f64, id: i32) -> Wall {
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let normal = if len == 0.0 { [0.0, 0.0, 1.0] } else { [nx / len, ny / len, nz / len] };
        Wall {
            id,
            inner: Box::new(PlaneGeometry { point: [px, py, pz], normal }),
        }
    }

    pub fn new_sphere(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall {
        Wall {
            id,
            inner: Box::new(SphereGeometry { center: [cx, cy, cz], radius }),
        }
    }

    pub fn new_cylinder(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, radius: f64, id: i32) -> Wall {
        let len = (ax * ax + ay * ay + az * az).sqrt();
        let axis = if len == 0.0 { [0.0, 0.0, 1.0] } else { [ax / len, ay / len, az / len] };
        Wall {
            id,
            inner: Box::new(CylinderGeometry { center: [cx, cy, cz], axis, radius }),
        }
    }

    #[wasm_bindgen(js_name = newCustom)]
    pub fn new_custom(val: JsValue, id: i32) -> Wall {
        Wall {
            id,
            inner: Box::new(JsWallGeometry { val }),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.inner.contains(&[x, y, z])
    }
}

impl Wall {
    pub fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        self.inner.cut(generator)
    }
}