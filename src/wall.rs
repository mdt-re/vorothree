use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Function, Array};
use crate::geometries::TrefoilKnotGeometry;

#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS: &'static str = r#"
export const WALL_ID_START = -10;
"#;

pub const WALL_ID_START: i32 = -10;

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

#[derive(Debug)]
struct TorusGeometry {
    center: [f64; 3],
    axis: [f64; 3],
    major_radius: f64,
    minor_radius: f64,
}

impl WallGeometry for TorusGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        
        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist_perp = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        
        // Distance to the tube center (which is at distance major_radius from axis)
        let dist_tube = ((dist_perp - self.major_radius).powi(2) + dot.powi(2)).sqrt();
        
        dist_tube <= self.minor_radius
    }

    fn cut(&self, generator: &[f64; 3]) -> Option<([f64; 3], [f64; 3])> {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];

        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist_perp = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        
        // Determine direction from axis to projected point
        let (dir_x, dir_y, dir_z) = if dist_perp < 1e-9 {
            // Singularity on axis: pick an arbitrary perpendicular vector
            let mut tx = 1.0; let mut ty = 0.0; let tz = 0.0;
            if self.axis[0].abs() > 0.9 { tx = 0.0; ty = 1.0; }
            let t_dot = tx * self.axis[0] + ty * self.axis[1] + tz * self.axis[2];
            let ax = tx - t_dot * self.axis[0];
            let ay = ty - t_dot * self.axis[1];
            let az = tz - t_dot * self.axis[2];
            let len = (ax*ax + ay*ay + az*az).sqrt();
            if len == 0.0 { return None; }
            (ax/len, ay/len, az/len)
        } else {
            (perp_x / dist_perp, perp_y / dist_perp, perp_z / dist_perp)
        };

        // Closest point on the major circle
        let cx = self.center[0] + dir_x * self.major_radius;
        let cy = self.center[1] + dir_y * self.major_radius;
        let cz = self.center[2] + dir_z * self.major_radius;

        // Vector from C to Generator
        let v_cx = generator[0] - cx;
        let v_cy = generator[1] - cy;
        let v_cz = generator[2] - cz;
        let dist_c = (v_cx*v_cx + v_cy*v_cy + v_cz*v_cz).sqrt();

        if dist_c == 0.0 { return None; }

        // Normal pointing OUT (away from C)
        let nx = v_cx / dist_c;
        let ny = v_cy / dist_c;
        let nz = v_cz / dist_c;

        // Point on surface
        let px = cx + nx * self.minor_radius;
        let py = cy + ny * self.minor_radius;
        let pz = cz + nz * self.minor_radius;

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

impl Wall {
    /// Creates a new Wall with a custom Rust implementation of WallGeometry.
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
    pub fn new_plane(px: f64, py: f64, pz: f64, nx: f64, ny: f64, nz: f64, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        let normal = if len == 0.0 { [0.0, 0.0, 1.0] } else { [nx / len, ny / len, nz / len] };
        Wall {
            id,
            inner: Box::new(PlaneGeometry { point: [px, py, pz], normal }),
        }
    }

    pub fn new_sphere(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        Wall {
            id,
            inner: Box::new(SphereGeometry { center: [cx, cy, cz], radius }),
        }
    }

    pub fn new_cylinder(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, radius: f64, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        let len = (ax * ax + ay * ay + az * az).sqrt();
        let axis = if len == 0.0 { [0.0, 0.0, 1.0] } else { [ax / len, ay / len, az / len] };
        Wall {
            id,
            inner: Box::new(CylinderGeometry { center: [cx, cy, cz], axis, radius }),
        }
    }

    pub fn new_torus(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, major: f64, minor: f64, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        let len = (ax * ax + ay * ay + az * az).sqrt();
        let axis = if len == 0.0 { [0.0, 0.0, 1.0] } else { [ax / len, ay / len, az / len] };
        Wall {
            id,
            inner: Box::new(TorusGeometry { center: [cx, cy, cz], axis, major_radius: major, minor_radius: minor }),
        }
    }

    pub fn new_trefoil(cx: f64, cy: f64, cz: f64, scale: f64, radius: f64, resolution: usize, id: i32) -> Wall {
        if id > WALL_ID_START {
            panic!("Wall ID must be <= {}", WALL_ID_START);
        }
        Wall {
            id,
            inner: Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, radius, resolution)),
        }
    }

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