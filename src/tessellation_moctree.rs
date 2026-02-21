use crate::bounds::BoundingBox;
use crate::cell_faces::CellFaces;
use crate::tessellation::Tessellation;
use crate::wall::Wall;
use crate::algo_octree::AlgorithmOctree;
use wasm_bindgen::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

/// A Voronoi tessellation container that uses an Octree for spatial partitioning.
///
/// This struct manages:
/// - The **bounding box** of the simulation.
/// - The **generators** (points) that define the Voronoi cells.
/// - The **walls** that clip the cells.
/// - The **octree** used for accelerating nearest-neighbor searches.
#[wasm_bindgen]
pub struct TessellationMoctree {
    inner: Tessellation<CellFaces, AlgorithmOctree>,
}

#[wasm_bindgen]
impl TessellationMoctree {
    /// Creates a new `TessellationMoctree` instance with the specified bounds and octree node capacity.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial boundaries of the tessellation.
    /// * `capacity` - The maximum number of points in a leaf node before subdivision occurs.
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, capacity: usize) -> TessellationMoctree {
        let algorithm = AlgorithmOctree::new(bounds, capacity);
        TessellationMoctree {
            inner: Tessellation::new(bounds, algorithm),
        }
    }

    /// Adds a wall to the tessellation to clip the Voronoi cells.
    pub fn add_wall(&mut self, wall: Wall) {
        self.inner.add_wall(wall);
    }

    /// Removes all walls from the tessellation.
    pub fn clear_walls(&mut self) {
        self.inner.walls.clear();
    }

    /// Returns the number of cells in the tessellation.
    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize {
        self.inner.cells.len()
    }

    /// Returns a flat array of all generator coordinates [x, y, z, x, y, z, ...].
    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.inner.generators.clone()
    }

    /// Returns the number of generators.
    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize {
        self.inner.generators.len() / 3
    }

    /// Calculates all cells based on the current generators.
    ///
    /// This method uses the internal octree to efficiently find the closest generators.
    pub fn calculate(&mut self) {
        self.inner.calculate();
    }

    /// Performs one step of Lloyd's relaxation.
    ///
    /// This moves each generator to the centroid of its calculated Voronoi cell,
    /// which tends to make the cells more uniform in size and shape.
    pub fn relax(&mut self) {
        self.inner.relax();
    }

    /// Update all generators at once.
    ///
    /// # Arguments
    /// * `generators` - A flat array of coordinates `[x, y, z, x, y, z, ...]`.
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

    /// Update the position of a single generator by index.
    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 < self.inner.generators.len() {
            for wall in &self.inner.walls {
                if !wall.contains(x, y, z) {
                    return;
                }
            }

            // Moctree update is expensive (rebuild), so we just rebuild for now via set_generators if needed,
            // or we can implement a smarter update. For now, let's just update the vector and rebuild.
            // Since GenericTessellation doesn't expose mutable generators directly for single update + index update easily,
            // we might need to access inner.
            
            // Actually, GenericTessellation doesn't have set_generator.
            // We can implement it on GenericTessellation or just do full update here.
            // Full update is safe.
            let mut gens = self.inner.generators.clone();
            gens[offset] = x;
            gens[offset+1] = y;
            gens[offset+2] = z;
            self.inner.set_generators(gens);
        }
    }

    /// Retrieves a calculated cell by index.
    pub fn get(&self, index: usize) -> Option<CellFaces> {
        self.inner.cells.get(index).cloned()
    }

    /// Generates random points within the bounds and sets them as generators.
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

    /// Removes generators that are not inside the defined walls.
    /// Note: This changes the indices of the remaining generators.
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