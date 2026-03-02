use crate::algorithm::d3_grid::AlgorithmGrid;
use crate::bounds::BoundingBox;
use crate::cell::d3_faces::CellFaces;
use crate::tessellation::Tessellation;
use crate::wall::{Wall, WallGeometry};
use crate::wall::geometries::*;
use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Function, Array};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_threads(n: usize) -> js_sys::Promise {
    init_thread_pool(n)
}

// --- Bounding Box ---

#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS_BOUNDS: &'static str = r#"
export const BOX_ID_LEFT = -1;
export const BOX_ID_RIGHT = -2;
export const BOX_ID_FRONT = -3;
export const BOX_ID_BACK = -4;
export const BOX_ID_BOTTOM = -5;
export const BOX_ID_TOP = -6;
"#;

/// Represents an axis-aligned bounding box in 3D space.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox3D {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[wasm_bindgen]
impl BoundingBox3D {
    #[wasm_bindgen(constructor)]
    pub fn new(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> BoundingBox3D {
        BoundingBox3D {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}

impl From<BoundingBox3D> for BoundingBox<3> {
    fn from(b: BoundingBox3D) -> Self {
        Self {
            min: [b.min_x, b.min_y, b.min_z],
            max: [b.max_x, b.max_y, b.max_z],
        }
    }
}

// --- Wall ---

#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS_WALL: &'static str = r#"
export const WALL_ID_START = -10;
"#;

/// WASM wrapper for 3D Walls.
#[wasm_bindgen]
pub struct Wall3D {
    inner: Option<Wall<3>>,
}

#[wasm_bindgen]
impl Wall3D {
    #[wasm_bindgen(js_name = newCustom)]
    pub fn new_custom(val: JsValue, id: i32) -> Wall3D {
        Wall3D {
            inner: Some(Wall::new(id, Box::new(JsWallGeometry { val }))),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> i32 {
        self.inner.as_ref().unwrap().id()
    }

    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        self.inner.as_ref().unwrap().contains(&[x, y, z])
    }

    pub fn new_plane(px: f64, py: f64, pz: f64, nx: f64, ny: f64, nz: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(PlaneGeometry::new([px, py, pz], [nx, ny, nz])))) }
    }

    pub fn new_sphere(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(SphereGeometry::new([cx, cy, cz], radius)))) }
    }

    pub fn new_cylinder(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(CylinderGeometry::new([cx, cy, cz], [ax, ay, az], radius)))) }
    }

    pub fn new_cone(tx: f64, ty: f64, tz: f64, ax: f64, ay: f64, az: f64, angle: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConeGeometry::new([tx, ty, tz], [ax, ay, az], angle)))) }
    }

    pub fn new_torus(cx: f64, cy: f64, cz: f64, ax: f64, ay: f64, az: f64, major: f64, minor: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(TorusGeometry::new([cx, cy, cz], [ax, ay, az], major, minor)))) }
    }

    pub fn new_trefoil(cx: f64, cy: f64, cz: f64, scale: f64, tube_radius: f64, resolution: usize, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, tube_radius, resolution)))) }
    }

    pub fn new_convex_polyhedron(points: &[f64], normals: &[f64], id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new(points, normals)))) }
    }

    pub fn new_tetrahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_tetrahedron([cx, cy, cz], radius)))) }
    }

    pub fn new_hexahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_hexahedron([cx, cy, cz], radius)))) }
    }

    pub fn new_octahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_octahedron([cx, cy, cz], radius)))) }
    }

    pub fn new_dodecahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([cx, cy, cz], radius)))) }
    }

    pub fn new_icosahedron(cx: f64, cy: f64, cz: f64, radius: f64, id: i32) -> Wall3D {
        Wall3D { inner: Some(Wall::new(id, Box::new(ConvexPolyhedronGeometry::new_icosahedron([cx, cy, cz], radius)))) }
    }

    pub fn new_bezier(points: &[f64], radius: f64, resolution: usize, closed: bool, id: i32) -> Wall3D {
        if points.len() != 12 {
            panic!("Cubic Bezier curve requires exactly 4 control points (12 coordinates)");
        }
        let p0 = [points[0], points[1], points[2]];
        let p1 = [points[3], points[4], points[5]];
        let p2 = [points[6], points[7], points[8]];
        let p3 = [points[9], points[10], points[11]];
        Wall3D { inner: Some(Wall::new(id, Box::new(CubicBezierGeometry::new(p0, p1, p2, p3, radius, resolution, closed)))) }
    }

    pub fn new_catmull_rom(points: &[f64], radius: f64, resolution: usize, closed: bool, id: i32) -> Wall3D {
        if points.len() % 3 != 0 {
            panic!("Catmull-Rom curve points must be a multiple of 3 coordinates");
        }
        let mut control_points = Vec::with_capacity(points.len() / 3);
        for i in (0..points.len()).step_by(3) {
            control_points.push([points[i], points[i+1], points[i+2]]);
        }
        Wall3D { inner: Some(Wall::new(id, Box::new(CatmullRomGeometry::new(control_points, radius, resolution, closed)))) }
    }
}

impl Wall3D {
    pub fn take_inner(&mut self) -> Option<Wall<3>> {
        self.inner.take()
    }
}

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

impl WallGeometry<3> for JsWallGeometry {
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

fn parse_js_cut_result(val: &JsValue) -> Option<([f64; 3], [f64; 3])> {
    let p = Reflect::get(val, &"point".into()).ok().and_then(|v| v.dyn_into::<Array>().ok())?;
    let n = Reflect::get(val, &"normal".into()).ok().and_then(|v| v.dyn_into::<Array>().ok())?;
    Some(([p.get(0).as_f64()?, p.get(1).as_f64()?, p.get(2).as_f64()?], [n.get(0).as_f64()?, n.get(1).as_f64()?, n.get(2).as_f64()?]))
}

// --- CellFaces Wrapper ---

#[wasm_bindgen(js_name = CellFaces)]
pub struct CellFacesWASM {
    inner: CellFaces,
}

#[wasm_bindgen(js_class = CellFaces)]
impl CellFacesWASM {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> usize {
        self.inner.id()
    }

    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f64> {
        self.inner.vertices()
    }

    #[wasm_bindgen(getter)]
    pub fn face_counts(&self) -> Vec<u32> {
        self.inner.face_counts()
    }

    #[wasm_bindgen(getter)]
    pub fn face_indices(&self) -> Vec<u32> {
        self.inner.face_indices()
    }

    #[wasm_bindgen(getter)]
    pub fn face_neighbors(&self) -> Vec<i32> {
        self.inner.face_neighbors()
    }

    pub fn volume(&self) -> f64 {
        self.inner.volume()
    }

    pub fn centroid(&self) -> Vec<f64> {
        let c = self.inner.centroid();
        vec![c[0], c[1], c[2]]
    }

    pub fn face_area(&self, face_index: usize) -> f64 {
        self.inner.face_area(face_index)
    }
}

// --- Tessellation ---

#[wasm_bindgen(js_name = Tessellation)]
pub struct TessellationWASM {
    inner: Tessellation<3, CellFaces, AlgorithmGrid>,
}

#[wasm_bindgen(js_class = Tessellation)]
impl TessellationWASM {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox3D, nx: usize, ny: usize, nz: usize) -> TessellationWASM {
        let b: BoundingBox<3> = bounds.into();
        let algorithm = AlgorithmGrid::new(nx, ny, nz, &b);
        TessellationWASM {
            inner: Tessellation::new(b, algorithm),
        }
    }

    pub fn set_generators(&mut self, generators: &[f64]) {
        self.inner.set_generators(generators);
    }

    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        self.inner.set_generator(index, &[x, y, z]);
    }

    pub fn random_generators(&mut self, count: usize) {
        self.inner.random_generators(count);
    }

    pub fn import_generators(&mut self, path: &str) -> Result<(), JsValue> {
        self.inner.import_generators(path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn add_wall(&mut self, mut wall: Wall3D) {
        if let Some(w) = wall.take_inner() {
            self.inner.add_wall(w);
        }
    }

    pub fn clear_walls(&mut self) {
        self.inner.clear_walls();
    }

    pub fn calculate(&mut self) {
        self.inner.calculate();
    }

    pub fn relax(&mut self) {
        self.inner.relax();
    }

    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize {
        self.inner.count_generators()
    }

    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize {
        self.inner.count_cells()
    }

    pub fn get_generator(&self, index: usize) -> Vec<f64> {
        self.inner.get_generator(index).to_vec()
    }

    pub fn get_cell(&self, index: usize) -> Option<CellFacesWASM> {
        self.inner.get_cell(index).map(|inner| CellFacesWASM { inner })
    }

    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.inner.generators()
    }

    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<CellFacesWASM> {
        self.inner.cells().into_iter().map(|inner| CellFacesWASM { inner }).collect()
    }
}