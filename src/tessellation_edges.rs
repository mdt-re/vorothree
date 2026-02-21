use crate::bounds::BoundingBox;
use crate::cell_edges::CellEdges;
use crate::tessellation::Tessellation;
use crate::tessellation::SpatialAlgorithm;
use crate::wall::Wall;
use wasm_bindgen::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

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
        self.prune_outside_generators();
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
        let mut valid_generators = Vec::with_capacity(generators.len());
        let count = generators.len() / 3;

        for i in 0..count {
            let x = generators[i * 3];
            let y = generators[i * 3 + 1];
            let z = generators[i * 3 + 2];

            let mut inside = true;
            for wall in &self.inner.walls {
                if !wall.contains(x, y, z) {
                    inside = false;
                    break;
                }
            }

            if inside {
                valid_generators.push(x);
                valid_generators.push(y);
                valid_generators.push(z);
            }
        }
        self.inner.set_generators(valid_generators);
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
        self.set_generators(&current_gens);
    }

    pub fn get(&self, index: usize) -> Option<CellEdges> {
        self.inner.cells.get(index).cloned()
    }

    pub fn random_generators(&mut self, count: usize) {
        let mut rng = StdRng::seed_from_u64(get_seed());
        let mut points = Vec::with_capacity(count * 3);
        let w = self.inner.bounds.max_x - self.inner.bounds.min_x;
        let h = self.inner.bounds.max_y - self.inner.bounds.min_y;
        let d = self.inner.bounds.max_z - self.inner.bounds.min_z;
        
        let mut found = 0;
        let max_attempts = count * 1000;
        let mut attempts = 0;

        while found < count && attempts < max_attempts {
            attempts += 1;
            let x = self.inner.bounds.min_x + rng.r#gen::<f64>() * w;
            let y = self.inner.bounds.min_y + rng.r#gen::<f64>() * h;
            let z = self.inner.bounds.min_z + rng.r#gen::<f64>() * d;

            let mut inside = true;
            for wall in &self.inner.walls {
                if !wall.contains(x, y, z) {
                    inside = false;
                    break;
                }
            }

            if inside {
                points.push(x);
                points.push(y);
                points.push(z);
                found += 1;
            }
        }
        
        self.set_generators(&points);
    }

    pub fn prune_outside_generators(&mut self) {
        let mut new_generators = Vec::with_capacity(self.inner.generators.len());
        let count = self.inner.generators.len() / 3;
        
        for i in 0..count {
            let x = self.inner.generators[i * 3];
            let y = self.inner.generators[i * 3 + 1];
            let z = self.inner.generators[i * 3 + 2];
            
            let mut inside = true;
            for wall in &self.inner.walls {
                if !wall.contains(x, y, z) {
                    inside = false;
                    break;
                }
            }
            
            if inside {
                new_generators.push(x);
                new_generators.push(y);
                new_generators.push(z);
            }
        }
        
        if new_generators.len() != self.inner.generators.len() {
            self.set_generators(&new_generators);
        }
    }
}

fn get_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Math::random() * 4294967296.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        123456789
    }
}