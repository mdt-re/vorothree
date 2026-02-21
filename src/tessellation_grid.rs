use crate::bounds::BoundingBox;
use crate::cell_faces::{CellFaces};
use crate::tessellation::Tessellation;
use crate::tessellation::SpatialAlgorithm;
use crate::wall::Wall;
use wasm_bindgen::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use crate::algo_grid::AlgorithmGrid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_threads(n: usize) -> js_sys::Promise {
    init_thread_pool(n)
}

/// The main container for performing 3D Voronoi tessellations.
///
/// This struct manages:
/// - The **bounding box** of the simulation.
/// - The **generators** (points) that define the Voronoi cells.
/// - The **walls** that clip the cells.
/// - The **grid** used for spatial partitioning acceleration.
#[wasm_bindgen]
pub struct TessellationGrid {
    inner: Tessellation<CellFaces, AlgorithmGrid>,
}

#[wasm_bindgen]
impl TessellationGrid {
    /// Creates a new `TessellationGrid` instance with the specified bounds and grid resolution.
    ///
    /// The grid resolution (`nx`, `ny`, `nz`) determines the performance of the spatial lookup.
    /// A heuristic of `cbrt(N)` (cube root of the number of points) is often a good starting point.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial boundaries of the tessellation.
    /// * `nx` - Number of grid bins along the X axis.
    /// * `ny` - Number of grid bins along the Y axis.
    /// * `nz` - Number of grid bins along the Z axis.
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> TessellationGrid {
        let algorithm = AlgorithmGrid::new(nx, ny, nz, &bounds);
        TessellationGrid {
            inner: Tessellation::new(bounds, algorithm),
        }
    }

    /// Adds a wall to the tessellation to clip the Voronoi cells.
    pub fn add_wall(&mut self, wall: Wall) {
        self.inner.add_wall(wall);
        self.prune_outside_generators();
    }

    /// Removes all walls from the tessellation.
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

    /// Calculates all cells based on the current generators.
    ///
    /// This method uses the internal grid to efficiently find the closest generators
    /// and clips the cells against the bounding box and any added walls.
    /// It runs in parallel if the `rayon` feature is enabled (which is default).
    pub fn calculate(&mut self) {
        self.inner.calculate();
    }

    /// Performs one step of Lloyd's relaxation.
    ///
    /// This moves each generator to the centroid of its calculated Voronoi cell,
    /// which tends to make the cells more uniform in size and shape (centroidal Voronoi tessellation).
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

            let old_pos = [self.inner.generators[offset], self.inner.generators[offset+1], self.inner.generators[offset+2]];
            let new_pos = [x, y, z];
            
            self.inner.algorithm.update_generator(index, &old_pos, &new_pos, &self.inner.bounds);
            
            self.inner.generators[offset] = x;
            self.inner.generators[offset + 1] = y;
            self.inner.generators[offset + 2] = z;
        }
    }

    /// Resizes the internal spatial partitioning grid.
    pub fn set_grid_shape(&mut self, nx: usize, ny: usize, nz: usize) {
        self.inner.algorithm = AlgorithmGrid::new(nx, ny, nz, &self.inner.bounds);
        let current_gens = self.inner.generators.clone();
        self.set_generators(&current_gens);
    }

    /// Retrieves a calculated cell by index.
    pub fn get(&self, index: usize) -> Option<CellFaces> {
        self.inner.cells.get(index).cloned()
    }

    /// Generates random points within the bounds and sets them as generators.
    pub fn random_generators(&mut self, count: usize) {
        // TODO: implement generator radii, to others and to walls. Use octree like website_infinity.
        let mut rng = StdRng::seed_from_u64(get_seed());
        let mut points = Vec::with_capacity(count * 3);
        let w = self.inner.bounds.max_x - self.inner.bounds.min_x;
        let h = self.inner.bounds.max_y - self.inner.bounds.min_y;
        let d = self.inner.bounds.max_z - self.inner.bounds.min_z;
        
        let mut found = 0;
        let max_attempts = count * 1000; // Safety limit
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometries::{PlaneGeometry, SphereGeometry, CylinderGeometry, ConeGeometry};

    // Helper to create a simple bounding box for testing
    fn mock_bounds() -> BoundingBox {
        BoundingBox {
            min_x: 0.0, min_y: 0.0, min_z: 0.0,
            max_x: 100.0, max_y: 100.0, max_z: 100.0,
        }
    }

    #[test]
    fn test_grid_binning_indices() {
        let bounds = mock_bounds();
        let tess = TessellationGrid::new(bounds, 10, 10, 10);

        // Point at (5, 5, 5) should be in the first bin (0,0,0) -> index 0
        assert_eq!(tess.inner.algorithm.get_bin_index(5.0, 5.0, 5.0, &tess.inner.bounds), 0);

        // Point at (15, 5, 5) should be in bin (1,0,0).
        // Index = x + y*nx + z*nx*ny = 1 + 0 + 0 = 1
        assert_eq!(tess.inner.algorithm.get_bin_index(15.0, 5.0, 5.0, &tess.inner.bounds), 1);

        // Point at (5, 15, 5) should be in bin (0,1,0).
        // Index = 0 + 1*10 + 0 = 10
        assert_eq!(tess.inner.algorithm.get_bin_index(5.0, 15.0, 5.0, &tess.inner.bounds), 10);
    }

    #[test]
    fn test_generator_updates() {
        let bounds = mock_bounds();
        let mut tess = TessellationGrid::new(bounds, 10, 10, 10);
        tess.set_generators(&[10.0, 10.0, 10.0, 50.0, 50.0, 50.0]);
        
        assert_eq!(tess.generators().len(), 6);
    }

    #[test]
    fn test_wall_clipping_volume() {
        let bounds = BoundingBox {
            min_x: 0.0, min_y: 0.0, min_z: 0.0,
            max_x: 10.0, max_y: 10.0, max_z: 10.0,
        };

        // Generate 1000 points in a grid to ensure uniform filling
        let mut generators = Vec::new();
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    generators.push(x as f64 + 0.5);
                    generators.push(y as f64 + 0.5);
                    generators.push(z as f64 + 0.5);
                }
            }
        }

        // Test Plane Wall
        {
            let mut tess = TessellationGrid::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Keep x > 5.0
            tess.add_wall(Wall::new(-11, Box::new(PlaneGeometry::new([5.0, 0.0, 0.0], [1.0, 0.0, 0.0]))));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            assert!((vol - 500.0).abs() < 1e-3, "Plane wall volume incorrect: {}", vol);
        }

        // Test Sphere Wall
        {
            let mut tess = TessellationGrid::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Sphere at center, radius 4. Volume = 4/3 * pi * 4^3 = 268.08257
            tess.add_wall(Wall::new(-11, Box::new(SphereGeometry::new([5.0, 5.0, 5.0], 4.0))));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            let expected = 4.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3);
            // Voronoi approximation of a curved surface with 1000 cells might have some error
            // The error depends on the resolution (number of cells).
            // With 1000 cells, it should be reasonably close, maybe within 1-2%?
            assert!((vol - expected).abs() / expected < 0.05, "Sphere wall volume incorrect: got {}, expected {}", vol, expected);
        }

        // Test Cylinder Wall
        {
            let mut tess = TessellationGrid::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Cylinder along Z, radius 4. Volume = pi * r^2 * h = pi * 16 * 10 = 502.6548
            tess.add_wall(Wall::new(-11, Box::new(CylinderGeometry::new([5.0, 5.0, 5.0], [0.0, 0.0, 1.0], 4.0))));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            let expected = std::f64::consts::PI * 4.0f64.powi(2) * 10.0;
            assert!((vol - expected).abs() / expected < 0.05, "Cylinder wall volume incorrect: got {}, expected {}", vol, expected);
        }

        // Test Cone Wall
        {
            let mut tess = TessellationGrid::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Cone at 5,5,2, axis Z, angle atan(0.5).
            // h = z - 2. r = h * 0.5. At z=10, h=8, r=4. Fits in 10x10 box.
            // Volume = 1/3 * pi * r^2 * h = 1/3 * pi * 16 * 8 = 128/3 * pi.
            tess.add_wall(Wall::new(-11, Box::new(ConeGeometry::new([5.0, 5.0, 2.0], [0.0, 0.0, 1.0], 0.5f64.atan()))));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            let expected = 128.0 / 3.0 * std::f64::consts::PI;
            assert!((vol - expected).abs() / expected < 0.05, "Cone wall volume incorrect: got {}, expected {}", vol, expected);
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
        123456789 // Fixed seed for tests
    }
}