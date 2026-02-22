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

    pub fn add_wall(&mut self, wall: Wall) {
        self.inner.add_wall(wall);
    }

    pub fn clear_walls(&mut self) {
        self.inner.clear_walls();
    }

    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize {
        self.inner.cells.len()
    }

    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.inner.generators.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize {
        self.inner.generators.len() / 3
    }

    pub fn calculate(&mut self) {
        self.inner.calculate();
    }

    pub fn relax(&mut self) {
        self.inner.relax();
    }

    pub fn set_generators(&mut self, generators: &[f64]) {
        self.inner.set_generators(generators);
    }

    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        self.inner.set_generator(index, x, y, z);
    }

    pub fn set_grid_shape(&mut self, nx: usize, ny: usize, nz: usize) {
        self.inner.algorithm = AlgorithmGrid::new(nx, ny, nz, &self.inner.bounds);
        let current_gens = self.inner.generators.clone();
        self.inner.set_generators(&current_gens);
    }

    pub fn get(&self, index: usize) -> Option<CellFaces> {
        self.inner.cells.get(index).cloned()
    }

    pub fn random_generators(&mut self, count: usize) {
        self.inner.random_generators(count);
    }
}