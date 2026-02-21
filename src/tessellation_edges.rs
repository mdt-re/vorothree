use crate::bounds::BoundingBox;
use crate::cell_edges::CellEdges;
use crate::tessellation::Tessellation;
use crate::tessellation::SpatialAlgorithm;
use crate::wall::Wall;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

use crate::algo_grid::AlgorithmGrid;

/// The main container for performing 3D Voronoi tessellations using the graph-based CellEdges.
#[wasm_bindgen]
pub struct TessellationEdges {
    inner: Tessellation<CellEdges, AlgorithmGrid>,
}

#[wasm_bindgen]
impl TessellationEdges {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> TessellationEdges {
        let algorithm = AlgorithmGrid::new(nx, ny, nz, &bounds);
        TessellationEdges {
            inner: Tessellation::new(bounds, algorithm),
        }
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.inner.add_wall(wall);
    }

    pub fn clear_walls(&mut self) {
        self.inner.walls.clear();
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
        let offset = index * 3;
        if offset + 2 < self.inner.generators.len() {
            for wall in &self.inner.walls {
                if !wall.contains(x, y, z) {
                    return;
                }
            }

            let old_pos = [self.inner.generators[offset], self.inner.generators[offset+1], self.inner.generators[offset+2]];
            let new_pos = [x, y, z];
            
            self.inner.algorithm.update_generator(index, &old_pos, &new_pos, &self.inner.bounds);
            
            self.inner.generators[offset] = x;
            self.inner.generators[offset + 1] = y;
            self.inner.generators[offset + 2] = z;
        }
    }

    pub fn set_grid_shape(&mut self, nx: usize, ny: usize, nz: usize) {
        self.inner.algorithm = AlgorithmGrid::new(nx, ny, nz, &self.inner.bounds);
        let current_gens = self.inner.generators.clone();
        self.inner.set_generators(&current_gens);
    }

    pub fn get(&self, index: usize) -> Option<CellEdges> {
        self.inner.cells.get(index).cloned()
    }

    pub fn random_generators(&mut self, count: usize) {
        self.inner.random_generators(count);
    }

    pub fn prune_outside_generators(&mut self) {
        self.inner.prune_outside_generators();
    }
}