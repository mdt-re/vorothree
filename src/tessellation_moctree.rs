use crate::bounds::BoundingBox;
use crate::cell_faces::CellFaces;
use crate::tessellation::Tessellation;
use crate::wall::Wall;
use crate::algo_octree::AlgorithmOctree;
use wasm_bindgen::prelude::*;

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
        self.inner.set_generators(generators);
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
            self.inner.set_generators(&gens);
        }
    }

    /// Retrieves a calculated cell by index.
    pub fn get(&self, index: usize) -> Option<CellFaces> {
        self.inner.cells.get(index).cloned()
    }

    /// Generates random points within the bounds and sets them as generators.
    pub fn random_generators(&mut self, count: usize) {
        self.inner.random_generators(count);
    }

    /// Removes generators that are not inside the defined walls.
    /// Note: This changes the indices of the remaining generators.
    pub fn prune_outside_generators(&mut self) {
        self.inner.prune_outside_generators();
    }
}