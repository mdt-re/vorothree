use crate::bounds::BoundingBox;
use crate::cell_faces::CellFaces;
use crate::tessellation::Tessellation;
use crate::wall::Wall;
use crate::algo_grid::AlgorithmGrid;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_threads(n: usize) -> js_sys::Promise {
    init_thread_pool(n)
}

#[wasm_bindgen(js_name = Tessellation)]
pub struct TessellationWASM {
    inner: Tessellation<CellFaces, AlgorithmGrid>,
}

#[wasm_bindgen(js_class = Tessellation)]
impl TessellationWASM {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> TessellationWASM {
        let algorithm = AlgorithmGrid::new(nx, ny, nz, &bounds);
        TessellationWASM {
            inner: Tessellation::new(bounds, algorithm),
        }
    }

    pub fn set_generators(&mut self, generators: &[f64]) {
        self.inner.set_generators(generators);
    }

    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        self.inner.set_generator(index, x, y, z);
    }

    pub fn random_generators(&mut self, count: usize) {
        self.inner.random_generators(count);
    }

    pub fn import_generators(&mut self, path: &str) -> Result<(), JsValue> {
        self.inner.import_generators(path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.inner.add_wall(wall);
    }

    pub fn clear_walls(&mut self) {
        self.inner.clear_walls();
    }

    pub fn calculate(&mut self) {
        self.inner.calculate();
    }

//  TODO: needs a serial version of the map function to Tessellation.
//  The existing map function requires Send bounds for parallel execution (via Rayon), which JsValue and js_sys::Function do not satisfy.
//  pub fn map(&self, callback: js_sys::Function) -> js_sys::Array {
//
//  }

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

    pub fn get_cell(&self, index: usize) -> Option<CellFaces> {
        self.inner.get_cell(index)
    }

    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.inner.generators()
    }

    #[wasm_bindgen(getter)]
    pub fn cells(&self) -> Vec<CellFaces> {
        self.inner.cells()
    }
}