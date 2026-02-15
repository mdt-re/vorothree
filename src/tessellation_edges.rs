use crate::bounds::BoundingBox;
use crate::cell_edges::{CellEdges, CellEdgesScratch};
use crate::wall::Wall;
use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

/// The main container for performing 3D Voronoi tessellations using the graph-based CellEdges.
#[wasm_bindgen]
pub struct TessellationEdges {
    bounds: BoundingBox,
    generators: Vec<f64>,
    cells: Vec<CellEdges>,
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
    bin_search_order: Vec<(isize, isize, isize, f64)>,
}

#[wasm_bindgen]
impl TessellationEdges {
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, nx: usize, ny: usize, nz: usize) -> TessellationEdges {
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
                    let dist_sq = get_min_dist_sq(x, y, z, cell_size_x, cell_size_y, cell_size_z);
                    bin_search_order.push((x, y, z, dist_sq));
                }
            }
        }
        bin_search_order.sort_unstable_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        TessellationEdges {
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

        self.cells = (0..count).into_par_iter().map_init(|| CellEdgesScratch::default(), |scratch, i| {
            let gx: f64 = generators[i * 3];
            let gy: f64 = generators[i * 3 + 1];
            let gz: f64 = generators[i * 3 + 2];

            let mut cell = CellEdges::new(i, bounds.clone());

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

            let rel_x = (gx - bounds.min_x) * grid_scale_x - idx_x as f64;
            let rel_y = (gy - bounds.min_y) * grid_scale_y - idx_y as f64;
            let rel_z = (gz - bounds.min_z) * grid_scale_z - idx_z as f64;

            let mut current_max_dist_sq = cell.max_radius_sq(&[gx, gy, gz]);

            let cell_size_x = 1.0 / grid_scale_x;
            let cell_size_y = 1.0 / grid_scale_y;
            let cell_size_z = 1.0 / grid_scale_z;

            let mut process_bin = |dx: isize, dy: isize, dz: isize, current_max_dist_sq: &mut f64, cell: &mut CellEdges| {
                let bx = idx_x as isize + dx;
                let by = idx_y as isize + dy;
                let bz = idx_z as isize + dz;

                if bx >= 0 && bx < grid_res_x as isize &&
                   by >= 0 && by < grid_res_y as isize &&
                   bz >= 0 && bz < grid_res_z as isize {
                    
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
                        return true;
                    }
                }
                false
            };

            for &(dx, dy, dz, min_d2) in bin_search_order {
                if min_d2 > 4.0 * current_max_dist_sq {
                    break;
                }
                process_bin(dx, dy, dz, &mut current_max_dist_sq, &mut cell);
            }
            cell
        }).collect();
    }

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

    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 < self.generators.len() {
            for wall in &self.walls {
                if !wall.contains(x, y, z) {
                    return;
                }
            }

            let new_bin_idx = self.get_bin_index(x, y, z);
            let old_bin_idx = self.generator_bin_ids[index];

            if new_bin_idx != old_bin_idx {
                if let Some(pos) = self.grid_bins[old_bin_idx].iter().position(|&id| id == index) {
                    self.grid_bins[old_bin_idx].swap_remove(pos);
                }
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
                    let dist_sq = get_min_dist_sq(x, y, z, cell_size_x, cell_size_y, cell_size_z);
                    self.bin_search_order.push((x, y, z, dist_sq));
                }
            }
        }
        self.bin_search_order.sort_unstable_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

        let current_gens = self.generators.clone();
        self.set_generators(&current_gens);
    }

    pub fn get(&self, index: usize) -> Option<CellEdges> {
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
        let mut rng = StdRng::seed_from_u64(get_seed());
        let mut points = Vec::with_capacity(count * 3);
        let w = self.bounds.max_x - self.bounds.min_x;
        let h = self.bounds.max_y - self.bounds.min_y;
        let d = self.bounds.max_z - self.bounds.min_z;
        
        let mut found = 0;
        let max_attempts = count * 1000;
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