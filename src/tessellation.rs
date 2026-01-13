use crate::bounds::BoundingBox;
use crate::cell::Cell;
use crate::wall::Wall;
use wasm_bindgen::prelude::*;
use rayon::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_threads(n: usize) -> js_sys::Promise {
    init_thread_pool(n)
}

#[wasm_bindgen]
pub struct Tessellation {
    bounds: BoundingBox,
    generators: Vec<f64>,
    cells: Vec<Cell>,
    walls: Vec<Wall>,
    // Grid for spatial partitioning
    grid_res_x: usize,
    grid_res_y: usize,
    grid_res_z: usize,
    grid_scale_x: f64,
    grid_scale_y: f64,
    grid_scale_z: f64,
    grid_limit_x: f64,
    grid_limit_y: f64,
    grid_limit_z: f64,
    grid_bins: Vec<Vec<usize>>,
    generator_bin_ids: Vec<usize>,
}

#[wasm_bindgen]
impl Tessellation {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> Tessellation {
        let sx = (nx as f64) / (bounds.max_x - bounds.min_x);
        let sy = (ny as f64) / (bounds.max_y - bounds.min_y);
        let sz = (nz as f64) / (bounds.max_z - bounds.min_z);

        Tessellation {
            bounds,
            generators: Vec::new(),
            cells: Vec::new(),
            walls: Vec::new(),
            grid_res_x: nx,
            grid_res_y: ny,
            grid_res_z: nz,
            grid_scale_x: sx,
            grid_scale_y: sy,
            grid_scale_z: sz,
            grid_limit_x: (nx as f64) - 1e-5,
            grid_limit_y: (ny as f64) - 1e-5,
            grid_limit_z: (nz as f64) - 1e-5,
            grid_bins: vec![Vec::new(); nx * ny * nz],
            generator_bin_ids: Vec::new(),
        }
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
    }

    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize {
        self.cells.len()
    }

    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.generators.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize {
        self.generators.len() / 3
    }

    /// Calculates all cells based on the current generators.
    pub fn calculate(&mut self) {
        let count = self.generators.len() / 3;
        let generators = &self.generators;
        let bounds = &self.bounds;
        let walls = &self.walls;

        self.cells = (0..count).into_par_iter().map(|i| {
            let gx: f64 = generators[i * 3];
            let gy: f64 = generators[i * 3 + 1];
            let gz: f64 = generators[i * 3 + 2];

            // Initialize a new cell from the bounding box
            let mut cell: Cell = Cell::new(i, bounds.clone());

            // Apply walls
            for wall in walls {
                if let Some((point, normal)) = wall.cut(&[gx, gy, gz]) {
                    cell.clip(&point, &normal, wall.id());
                }
            }

            // TODO: implement the smarter logic to use the grid and its bins.
            for j in 0..count {
                if i == j { continue; }

                let ox: f64 = generators[j * 3];
                let oy: f64 = generators[j * 3 + 1];
                let oz: f64 = generators[j * 3 + 2];

                let dx: f64 = ox - gx;
                let dy: f64 = oy - gy;
                let dz: f64 = oz - gz;

                let mx: f64 = gx + dx * 0.5;
                let my: f64 = gy + dy * 0.5;
                let mz: f64 = gz + dz * 0.5;

                cell.clip(&[mx, my, mz], &[dx, dy, dz], j as i32);
            }
            cell
        }).collect();
    }

    /// Performs one step of Lloyd's relaxation.
    /// Moves each generator to the centroid of its cell.
    pub fn relax(&mut self) {
        if self.cells.len() != self.generators.len() / 3 {
            return;
        }

        let new_generators: Vec<f64> = self.cells.par_iter()
            .zip(self.generators.par_chunks(3))
            .flat_map(|(cell, original_pos)| {
                if cell.vertices.is_empty() {
                    original_pos.to_vec()
                } else {
                    cell.centroid()
                }
            })
            .collect();

        self.set_generators(&new_generators);
    }

    /// Update all generators at once.
    /// Expects a flat array [x, y, z, x, y, z, ...]
    pub fn set_generators(&mut self, generators: &[f64]) {
        self.generators = generators.to_vec();
        
        // Rebuild grid
        let total_bins = self.grid_res_x * self.grid_res_y * self.grid_res_z;
        self.grid_bins.iter_mut().for_each(|bin| bin.clear());
        if self.grid_bins.len() != total_bins {
            self.grid_bins = vec![Vec::new(); total_bins];
        }
        
        let count = self.generators.len() / 3;
        self.generator_bin_ids = vec![0; count];

        for i in 0..count {
            let x = self.generators[i * 3];
            let y = self.generators[i * 3 + 1];
            let z = self.generators[i * 3 + 2];
            let bin_idx = self.get_bin_index(x, y, z);
            
            self.grid_bins[bin_idx].push(i);
            self.generator_bin_ids[i] = bin_idx;
        }
    }

    /// Update a single generator by index.
    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 < self.generators.len() {
            // Update binning if position changed
            let new_bin_idx = self.get_bin_index(x, y, z);
            let old_bin_idx = self.generator_bin_ids[index];

            if new_bin_idx != old_bin_idx {
                // Remove from old bin
                if let Some(pos) = self.grid_bins[old_bin_idx].iter().position(|&id| id == index) {
                    // swap_remove is faster but changes order; order inside bin usually doesn't matter
                    self.grid_bins[old_bin_idx].swap_remove(pos);
                }
                // Add to new bin
                self.grid_bins[new_bin_idx].push(index);
                self.generator_bin_ids[index] = new_bin_idx;
            }

            self.generators[offset] = x;
            self.generators[offset + 1] = y;
            self.generators[offset + 2] = z;
        }
    }

    pub fn set_grid_shape(&mut self, nx: usize, ny: usize, nz: usize) {
        self.grid_res_x = nx;
        self.grid_res_y = ny;
        self.grid_res_z = nz;
        self.grid_scale_x = (nx as f64) / (self.bounds.max_x - self.bounds.min_x);
        self.grid_scale_y = (ny as f64) / (self.bounds.max_y - self.bounds.min_y);
        self.grid_scale_z = (nz as f64) / (self.bounds.max_z - self.bounds.min_z);
        self.grid_limit_x = (nx as f64) - 1e-5;
        self.grid_limit_y = (ny as f64) - 1e-5;
        self.grid_limit_z = (nz as f64) - 1e-5;
        // Re-bin existing generators
        // We can simply call set_generators with the current data to rebuild
        let current_gens = self.generators.clone();
        self.set_generators(&current_gens);
    }

    pub fn get(&self, index: usize) -> Option<Cell> {
        self.cells.get(index).cloned()
    }

    fn get_bin_index(&self, x: f64, y: f64, z: f64) -> usize {
        let nx = self.grid_res_x;
        let ny = self.grid_res_y;

        let ix = ((x - self.bounds.min_x) * self.grid_scale_x).clamp(0.0, self.grid_limit_x) as usize;
        let iy = ((y - self.bounds.min_y) * self.grid_scale_y).clamp(0.0, self.grid_limit_y) as usize;
        let iz = ((z - self.bounds.min_z) * self.grid_scale_z).clamp(0.0, self.grid_limit_z) as usize;

        ix + iy * nx + iz * nx * ny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let tess = Tessellation::new(bounds, 10, 10, 10);

        // Point at (5, 5, 5) should be in the first bin (0,0,0) -> index 0
        assert_eq!(tess.get_bin_index(5.0, 5.0, 5.0), 0);

        // Point at (15, 5, 5) should be in bin (1,0,0).
        // Index = x + y*nx + z*nx*ny = 1 + 0 + 0 = 1
        assert_eq!(tess.get_bin_index(15.0, 5.0, 5.0), 1);

        // Point at (5, 15, 5) should be in bin (0,1,0).
        // Index = 0 + 1*10 + 0 = 10
        assert_eq!(tess.get_bin_index(5.0, 15.0, 5.0), 10);
    }

    #[test]
    fn test_generator_updates() {
        let bounds = mock_bounds();
        let mut tess = Tessellation::new(bounds, 10, 10, 10);
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
            let mut tess = Tessellation::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Keep x > 5.0
            tess.add_wall(Wall::new_plane(5.0, 0.0, 0.0, 1.0, 0.0, 0.0, -11));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            assert!((vol - 500.0).abs() < 1e-3, "Plane wall volume incorrect: {}", vol);
        }

        // Test Sphere Wall
        {
            let mut tess = Tessellation::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Sphere at center, radius 4. Volume = 4/3 * pi * 4^3 = 268.08257
            tess.add_wall(Wall::new_sphere(5.0, 5.0, 5.0, 4.0, -11));
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
            let mut tess = Tessellation::new(bounds.clone(), 5, 5, 5);
            tess.set_generators(&generators);
            // Cylinder along Z, radius 4. Volume = pi * r^2 * h = pi * 16 * 10 = 502.6548
            tess.add_wall(Wall::new_cylinder(5.0, 5.0, 5.0, 0.0, 0.0, 1.0, 4.0, -11));
            tess.calculate();
            let vol: f64 = (0..tess.count_cells()).map(|i| tess.get(i).unwrap().volume()).sum();
            let expected = std::f64::consts::PI * 4.0f64.powi(2) * 10.0;
            assert!((vol - expected).abs() / expected < 0.05, "Cylinder wall volume incorrect: got {}, expected {}", vol, expected);
        }
    }
}
