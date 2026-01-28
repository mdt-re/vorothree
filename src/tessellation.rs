use crate::bounds::BoundingBox;
use crate::cell::{Cell, ClipScratch};
use crate::wall::Wall;
use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

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
    bin_search_order: Vec<(isize, isize, isize)>,
}

#[wasm_bindgen]
impl Tessellation {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> Tessellation {
        let sx = (nx as f64) / (bounds.max_x - bounds.min_x);
        let sy = (ny as f64) / (bounds.max_y - bounds.min_y);
        let sz = (nz as f64) / (bounds.max_z - bounds.min_z);

        let cell_size_x = 1.0 / sx;
        let cell_size_y = 1.0 / sy;
        let cell_size_z = 1.0 / sz;

        let mut bin_search_order = Vec::new();
        let rx = nx as isize;
        let ry = ny as isize;
        let rz = nz as isize;
        for z in -rz..=rz {
            for y in -ry..=ry {
                for x in -rx..=rx {
                    bin_search_order.push((x, y, z));
                }
            }
        }
        bin_search_order.sort_unstable_by(|a, b| get_min_dist_sq(a.0, a.1, a.2, cell_size_x, cell_size_y, cell_size_z).partial_cmp(&get_min_dist_sq(b.0, b.1, b.2, cell_size_x, cell_size_y, cell_size_z)).unwrap_or(std::cmp::Ordering::Equal));

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
            bin_search_order,
        }
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
        self.prune_outside_generators();
    }

    pub fn clear_walls(&mut self) {
        self.walls.clear();
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
        let grid_res_x = self.grid_res_x;
        let grid_res_y = self.grid_res_y;
        let grid_res_z = self.grid_res_z;
        let grid_scale_x = self.grid_scale_x;
        let grid_scale_y = self.grid_scale_y;
        let grid_scale_z = self.grid_scale_z;
        let grid_bins = &self.grid_bins;
        let generator_bin_ids = &self.generator_bin_ids;
        let bin_search_order = &self.bin_search_order;

        self.cells = (0..count).into_par_iter().map_init(|| ClipScratch::default(), |scratch, i| {
            let gx: f64 = generators[i * 3];
            let gy: f64 = generators[i * 3 + 1];
            let gz: f64 = generators[i * 3 + 2];

            // Initialize a new cell from the bounding box
            let mut cell: Cell = Cell::new(i, bounds.clone());

            // Apply walls
            for wall in walls {
                wall.cut(&[gx, gy, gz], &mut |point, normal| {
                    cell.clip_with_scratch(&point, &normal, wall.id(), scratch, None);
                });
            }

            let bin_idx = generator_bin_ids[i];
            let idx_z = bin_idx / (grid_res_x * grid_res_y);
            let rem_z = bin_idx % (grid_res_x * grid_res_y);
            let idx_y = rem_z / grid_res_x;
            let idx_x = rem_z % grid_res_x;

            // Pre-calculate position within the bin (0.0 to 1.0)
            let rel_x = (gx - bounds.min_x) * grid_scale_x - idx_x as f64;
            let rel_y = (gy - bounds.min_y) * grid_scale_y - idx_y as f64;
            let rel_z = (gz - bounds.min_z) * grid_scale_z - idx_z as f64;

            let mut current_max_dist_sq = cell.max_radius_sq(&[gx, gy, gz]);

            let cell_size_x = 1.0 / grid_scale_x;
            let cell_size_y = 1.0 / grid_scale_y;
            let cell_size_z = 1.0 / grid_scale_z;

            // Helper closure to process a bin at relative offset (dx, dy, dz)
            let mut process_bin = |dx: isize, dy: isize, dz: isize, current_max_dist_sq: &mut f64, cell: &mut Cell| {
                let bx = idx_x as isize + dx;
                let by = idx_y as isize + dy;
                let bz = idx_z as isize + dz;

                if bx >= 0 && bx < grid_res_x as isize &&
                   by >= 0 && by < grid_res_y as isize &&
                   bz >= 0 && bz < grid_res_z as isize {
                    
                    // Optimized distance check using relative coordinates
                    // Distance from point (rel_x) to bin range [dx, dx+1]
                    let dx_dist = if dx > 0 {
                        (dx as f64 - rel_x) * cell_size_x
                    } else if dx < 0 {
                        (-(dx + 1) as f64 + rel_x) * cell_size_x
                    } else { 0.0 };

                    let dy_dist = if dy > 0 {
                        (dy as f64 - rel_y) * cell_size_y
                    } else if dy < 0 {
                        (-(dy + 1) as f64 + rel_y) * cell_size_y
                    } else { 0.0 };

                    let dz_dist = if dz > 0 {
                        (dz as f64 - rel_z) * cell_size_z
                    } else if dz < 0 {
                        (-(dz + 1) as f64 + rel_z) * cell_size_z
                    } else { 0.0 };

                    let dx_bin = dx_dist.max(0.0);
                    let dy_bin = dy_dist.max(0.0);
                    let dz_bin = dz_dist.max(0.0);
                    
                    if dx_bin * dx_bin + dy_bin * dy_bin + dz_bin * dz_bin <= 4.0 * *current_max_dist_sq {
                        let bin_index = (bx as usize) + (by as usize) * grid_res_x + (bz as usize) * grid_res_x * grid_res_y;
                        for &j in &grid_bins[bin_index] {
                            if i == j { continue; }
                            let ox = generators[j * 3];
                            let oy = generators[j * 3 + 1];
                            let oz = generators[j * 3 + 2];
                            let dx = ox - gx;
                            let dy = oy - gy;
                            let dz = oz - gz;
                            let dist_sq = dx * dx + dy * dy + dz * dz;
                            if dist_sq > 4.0 * *current_max_dist_sq { continue; }
                            if let (true, new_radius) = cell.clip_with_scratch(&[gx + dx * 0.5, gy + dy * 0.5, gz + dz * 0.5], &[dx, dy, dz], j as i32, scratch, Some(&[gx, gy, gz])) {
                                *current_max_dist_sq = new_radius;
                            }
                        }
                        return true; // Found bin in range
                    }
                }
                false
            };

            // Iterate using pre-calculated sorted offsets covering the full grid
            for &(dx, dy, dz) in bin_search_order {
                let min_d2 = get_min_dist_sq(dx, dy, dz, cell_size_x, cell_size_y, cell_size_z);
                if min_d2 > 4.0 * current_max_dist_sq {
                    break;
                }
                process_bin(dx, dy, dz, &mut current_max_dist_sq, &mut cell);
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
        let mut valid_generators = Vec::with_capacity(generators.len());
        let count = generators.len() / 3;

        for i in 0..count {
            let x = generators[i * 3];
            let y = generators[i * 3 + 1];
            let z = generators[i * 3 + 2];

            let mut inside = true;
            for wall in &self.walls {
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
        self.generators = valid_generators;
        
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
            for wall in &self.walls {
                if !wall.contains(x, y, z) {
                    return;
                }
            }

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
        
        let cell_size_x = 1.0 / self.grid_scale_x;
        let cell_size_y = 1.0 / self.grid_scale_y;
        let cell_size_z = 1.0 / self.grid_scale_z;

        self.bin_search_order.clear();
        let rx = nx as isize;
        let ry = ny as isize;
        let rz = nz as isize;
        for z in -rz..=rz {
            for y in -ry..=ry {
                for x in -rx..=rx {
                    self.bin_search_order.push((x, y, z));
                }
            }
        }
        self.bin_search_order.sort_unstable_by(|a, b| get_min_dist_sq(a.0, a.1, a.2, cell_size_x, cell_size_y, cell_size_z).partial_cmp(&get_min_dist_sq(b.0, b.1, b.2, cell_size_x, cell_size_y, cell_size_z)).unwrap_or(std::cmp::Ordering::Equal));

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

    pub fn random_generators(&mut self, count: usize) {
        // TODO: implement generator radii, to others and to walls. Use octree like website_infinity.
        let mut rng = StdRng::seed_from_u64(get_seed());
        let mut points = Vec::with_capacity(count * 3);
        let w = self.bounds.max_x - self.bounds.min_x;
        let h = self.bounds.max_y - self.bounds.min_y;
        let d = self.bounds.max_z - self.bounds.min_z;
        
        let mut found = 0;
        let max_attempts = count * 1000; // Safety limit
        let mut attempts = 0;

        while found < count && attempts < max_attempts {
            attempts += 1;
            let x = self.bounds.min_x + rng.r#gen::<f64>() * w;
            let y = self.bounds.min_y + rng.r#gen::<f64>() * h;
            let z = self.bounds.min_z + rng.r#gen::<f64>() * d;

            let mut inside = true;
            for wall in &self.walls {
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
        let mut new_generators = Vec::with_capacity(self.generators.len());
        let count = self.generators.len() / 3;
        
        for i in 0..count {
            let x = self.generators[i * 3];
            let y = self.generators[i * 3 + 1];
            let z = self.generators[i * 3 + 2];
            
            let mut inside = true;
            for wall in &self.walls {
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
        
        if new_generators.len() != self.generators.len() {
            self.set_generators(&new_generators);
        }
    }
}

fn get_min_dist_sq(dx: isize, dy: isize, dz: isize, cx: f64, cy: f64, cz: f64) -> f64 {
    let mx = if dx > 0 { (dx - 1) as f64 * cx } else if dx < 0 { (-dx - 1) as f64 * cx } else { 0.0 };
    let my = if dy > 0 { (dy - 1) as f64 * cy } else if dy < 0 { (-dy - 1) as f64 * cy } else { 0.0 };
    let mz = if dz > 0 { (dz - 1) as f64 * cz } else if dz < 0 { (-dz - 1) as f64 * cz } else { 0.0 };
    mx * mx + my * my + mz * mz
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
