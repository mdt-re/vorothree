use crate::algorithm::d2_grid::AlgorithmGrid2D;
use crate::bounds::BoundingBox;
use crate::cell::d2::Cell2D;
use crate::tessellation::Tessellation;
use crate::wall::{Wall, WallGeometry};
use crate::wall::geometries::*;
use crate::wasm::utils::parse_js_point;
use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Function, Array};

// --- Bounding Box ---

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[wasm_bindgen]
impl BoundingBox2D {
    #[wasm_bindgen(constructor)]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> BoundingBox2D {
        BoundingBox2D { min_x, min_y, max_x, max_y }
    }
}

impl From<BoundingBox2D> for BoundingBox<2> {
    fn from(b: BoundingBox2D) -> Self {
        Self { min: [b.min_x, b.min_y], max: [b.max_x, b.max_y] }
    }
}

// --- Wall ---

#[wasm_bindgen]
pub struct Wall2D {
    inner: Option<Wall<2>>,
}

#[wasm_bindgen]
impl Wall2D {
    #[wasm_bindgen(js_name = newCustom)]
    pub fn new_custom(val: JsValue, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(JsWallGeometry2D { val }))) }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> i32 { self.inner.as_ref().unwrap().id() }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        self.inner.as_ref().unwrap().contains(&[x, y])
    }

    pub fn new_line(px: f64, py: f64, nx: f64, ny: f64, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(LineGeometry::new([px, py], [nx, ny])))) }
    }

    pub fn new_circle(cx: f64, cy: f64, radius: f64, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(CircleGeometry::new([cx, cy], radius)))) }
    }

    pub fn new_polygon(points: &[f64], normals: &[f64], id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(ConvexPolygonGeometry2D::new(points, normals)))) }
    }

    pub fn new_regular_polygon(cx: f64, cy: f64, radius: f64, sides: usize, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(ConvexPolygonGeometry2D::new_regular([cx, cy], radius, sides)))) }
    }

    pub fn new_annulus(cx: f64, cy: f64, inner_r: f64, outer_r: f64, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(AnnulusGeometry::new([cx, cy], inner_r, outer_r)))) }
    }

    pub fn new_bezier(p0x: f64, p0y: f64, p1x: f64, p1y: f64, p2x: f64, p2y: f64, p3x: f64, p3y: f64, radius: f64, resolution: usize, closed: bool, id: i32) -> Wall2D {
        Wall2D { inner: Some(Wall::new(id, Box::new(CubicBezierGeometry2D::new([p0x, p0y], [p1x, p1y], [p2x, p2y], [p3x, p3y], radius, resolution, closed)))) }
    }
}

impl Wall2D {
    pub fn take_inner(&mut self) -> Option<Wall<2>> { self.inner.take() }
}

struct JsWallGeometry2D {
    val: JsValue,
}

unsafe impl Send for JsWallGeometry2D {}
unsafe impl Sync for JsWallGeometry2D {}

impl std::fmt::Debug for JsWallGeometry2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JsWallGeometry2D")
    }
}

impl WallGeometry<2> for JsWallGeometry2D {
    fn contains(&self, point: &[f64; 2]) -> bool {
        if let Ok(func) = Reflect::get(&self.val, &"contains".into()).and_then(|f| f.dyn_into::<Function>()) {
            let args = Array::of2(&point[0].into(), &point[1].into());
            if let Ok(res) = func.apply(&self.val, &args) {
                return res.as_bool().unwrap_or(false);
            }
        }
        false
    }

    fn cut(&self, generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        if let Ok(func) = Reflect::get(&self.val, &"cut".into()).and_then(|f| f.dyn_into::<Function>()) {
            let args = Array::of2(&generator[0].into(), &generator[1].into());
            if let Ok(res) = func.apply(&self.val, &args) {
                if res.is_null() || res.is_undefined() { return; }
                
                let process_item = |item: &JsValue| -> Option<([f64; 2], [f64; 2])> {
                    let p = Reflect::get(item, &"point".into()).ok()?;
                    let n = Reflect::get(item, &"normal".into()).ok()?;
                    Some((parse_js_point(&p)?, parse_js_point(&n)?))
                };

                if Array::is_array(&res) {
                    let arr: Array = res.dyn_into().unwrap();
                    for i in 0..arr.length() {
                        if let Some((p, n)) = process_item(&arr.get(i)) {
                            callback(p, n);
                        }
                    }
                } else {
                    if let Some((p, n)) = process_item(&res) {
                        callback(p, n);
                    }
                }
            }
        }
    }
}

// --- Cell Wrapper ---

#[wasm_bindgen(js_name = Cell2D)]
pub struct Cell2DWASM {
    inner: Cell2D,
}

#[wasm_bindgen(js_class = Cell2D)]
impl Cell2DWASM {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> usize { self.inner.id() }
    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f64> { self.inner.vertices() }
    #[wasm_bindgen(getter)]
    pub fn edge_neighbors(&self) -> Vec<i32> { self.inner.edge_neighbors() }
    pub fn area(&self) -> f64 { self.inner.area() }
    pub fn centroid(&self) -> Vec<f64> { self.inner.centroid().to_vec() }
}

// --- Tessellation ---

#[wasm_bindgen(js_name = Tessellation2D)]
pub struct Tessellation2D {
    inner: Tessellation<2, Cell2D, AlgorithmGrid2D>,
}

#[wasm_bindgen(js_class = Tessellation2D)]
impl Tessellation2D {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox2D, nx: usize, ny: usize) -> Tessellation2D {
        let b: BoundingBox<2> = bounds.into();
        Tessellation2D { inner: Tessellation::new(b, AlgorithmGrid2D::new(nx, ny, &b)) }
    }
    pub fn set_generators(&mut self, generators: &[f64]) { self.inner.set_generators(generators); }
    pub fn set_generator(&mut self, index: usize, x: f64, y: f64) { self.inner.set_generator(index, &[x, y]); }
    pub fn random_generators(&mut self, count: usize) { self.inner.random_generators(count); }
    pub fn add_wall(&mut self, mut wall: Wall2D) { if let Some(w) = wall.take_inner() { self.inner.add_wall(w); } }
    pub fn clear_walls(&mut self) { self.inner.clear_walls(); }
    pub fn calculate(&mut self) { self.inner.calculate(); }
    pub fn relax(&mut self) { self.inner.relax(); }
    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize { self.inner.count_generators() }
    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize { self.inner.count_cells() }
    pub fn get_generator(&self, index: usize) -> Vec<f64> { self.inner.get_generator(index).to_vec() }
    pub fn get_cell(&self, index: usize) -> Option<Cell2DWASM> { self.inner.get_cell(index).map(|inner| Cell2DWASM { inner }) }
    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> { self.inner.generators() }
    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<Cell2DWASM> { self.inner.cells().into_iter().map(|inner| Cell2DWASM { inner }).collect() }
}