use crate::bounds::BoundingBox;
use crate::tessellation::SpatialAlgorithm;

/// A spatial index based on a uniform grid.
///
/// This structure divides the 3D space into a fixed number of bins (voxels).
/// It is generally faster than an octree for uniform distributions and allows
/// for O(1) insertion and update operations, but may be less memory efficient
/// for highly clustered data or very large sparse domains.
pub struct AlgorithmGrid {
    /// Number of bins along the X axis.
    pub grid_res_x: usize,
    /// Number of bins along the Y axis.
    pub grid_res_y: usize,
    /// Number of bins along the Z axis.
    pub grid_res_z: usize,
    /// Scale factor for X coordinate to grid index.
    pub grid_scale_x: f64,
    /// Scale factor for Y coordinate to grid index.
    pub grid_scale_y: f64,
    /// Scale factor for Z coordinate to grid index.
    pub grid_scale_z: f64,
    /// Maximum valid index for X.
    pub grid_limit_x: f64,
    /// Maximum valid index for Y.
    pub grid_limit_y: f64,
    /// Maximum valid index for Z.
    pub grid_limit_z: f64,
    /// Minimum X coordinate of the grid bounds.
    pub min_x: f64,
    /// Minimum Y coordinate of the grid bounds.
    pub min_y: f64,
    /// Minimum Z coordinate of the grid bounds.
    pub min_z: f64,
    /// The grid bins, each containing a list of generator indices.
    pub grid_bins: Vec<Vec<usize>>,
    /// Map from generator index to its current bin index.
    pub generator_bin_ids: Vec<usize>,
    /// Precomputed search order for visiting neighboring bins.
    pub bin_search_order: Vec<(isize, isize, isize, f64)>,
}

impl AlgorithmGrid {
    /// Creates a new `AlgorithmGrid` with the specified dimensions and bounds.
    ///
    /// The grid resolution (`nx`, `ny`, `nz`) determines the granularity of the spatial partitioning.
    pub fn new(nx: usize, ny: usize, nz: usize, bounds: &BoundingBox) -> Self {
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

        AlgorithmGrid {
            grid_res_x: nx,
            grid_res_y: ny,
            grid_res_z: nz,
            grid_scale_x: sx,
            grid_scale_y: sy,
            grid_scale_z: sz,
            grid_limit_x: (nx as f64) - 1e-5,
            grid_limit_y: (ny as f64) - 1e-5,
            grid_limit_z: (nz as f64) - 1e-5,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            min_z: bounds.min_z,
            grid_bins: vec![Vec::new(); nx * ny * nz],
            generator_bin_ids: Vec::new(),
            bin_search_order,
        }
    }

    /// Calculates the linear index of the bin corresponding to the given coordinates.
    pub fn get_bin_index(&self, x: f64, y: f64, z: f64, bounds: &BoundingBox) -> usize {
        let nx = self.grid_res_x;
        let ny = self.grid_res_y;

        let ix = ((x - bounds.min_x) * self.grid_scale_x).clamp(0.0, self.grid_limit_x) as usize;
        let iy = ((y - bounds.min_y) * self.grid_scale_y).clamp(0.0, self.grid_limit_y) as usize;
        let iz = ((z - bounds.min_z) * self.grid_scale_z).clamp(0.0, self.grid_limit_z) as usize;

        ix + iy * nx + iz * nx * ny
    }
}

impl SpatialAlgorithm for AlgorithmGrid {
    fn set_generators(&mut self, generators: &[f64], bounds: &BoundingBox) {
        let total_bins = self.grid_res_x * self.grid_res_y * self.grid_res_z;
        self.grid_bins.iter_mut().for_each(|bin| bin.clear());
        if self.grid_bins.len() != total_bins {
            self.grid_bins = vec![Vec::new(); total_bins];
        }
        
        let count = generators.len() / 3;
        self.generator_bin_ids = vec![0; count];

        for i in 0..count {
            let x = generators[i * 3];
            let y = generators[i * 3 + 1];
            let z = generators[i * 3 + 2];
            let bin_idx = self.get_bin_index(x, y, z, bounds);
            
            self.grid_bins[bin_idx].push(i);
            self.generator_bin_ids[i] = bin_idx;
        }
    }

    fn update_generator(&mut self, index: usize, _old_pos: &[f64], new_pos: &[f64], bounds: &BoundingBox) {
        let new_bin_idx = self.get_bin_index(new_pos[0], new_pos[1], new_pos[2], bounds);
        let old_bin_idx = self.generator_bin_ids[index];

        if new_bin_idx != old_bin_idx {
            if let Some(pos) = self.grid_bins[old_bin_idx].iter().position(|&id| id == index) {
                self.grid_bins[old_bin_idx].swap_remove(pos);
            }
            self.grid_bins[new_bin_idx].push(index);
            self.generator_bin_ids[index] = new_bin_idx;
        }
    }

    fn visit_neighbors<F>(
        &self,
        generators: &[f64],
        index: usize,
        pos: [f64; 3],
        max_dist_sq: &mut f64,
        mut visitor: F,
    ) where
        F: FnMut(usize, [f64; 3], f64) -> f64,
    {
        let bin_idx = self.generator_bin_ids[index];
        let idx_z = bin_idx / (self.grid_res_x * self.grid_res_y);
        let rem_z = bin_idx % (self.grid_res_x * self.grid_res_y);
        let idx_y = rem_z / self.grid_res_x;
        let idx_x = rem_z % self.grid_res_x;

        let cell_size_x = 1.0 / self.grid_scale_x;
        let cell_size_y = 1.0 / self.grid_scale_y;
        let cell_size_z = 1.0 / self.grid_scale_z;

        let rel_x = (pos[0] - self.min_x) * self.grid_scale_x - idx_x as f64;
        let rel_y = (pos[1] - self.min_y) * self.grid_scale_y - idx_y as f64;
        let rel_z = (pos[2] - self.min_z) * self.grid_scale_z - idx_z as f64;

        for &(dx, dy, dz, min_d2) in &self.bin_search_order {
            if min_d2 > 4.0 * *max_dist_sq {
                break;
            }

            let bx = idx_x as isize + dx;
            let by = idx_y as isize + dy;
            let bz = idx_z as isize + dz;

            if bx >= 0 && bx < self.grid_res_x as isize &&
               by >= 0 && by < self.grid_res_y as isize &&
               bz >= 0 && bz < self.grid_res_z as isize {
                
                let dx_dist = if dx > 0 { (dx as f64 - rel_x) * cell_size_x } else if dx < 0 { (-(dx + 1) as f64 + rel_x) * cell_size_x } else { 0.0 };
                let dy_dist = if dy > 0 { (dy as f64 - rel_y) * cell_size_y } else if dy < 0 { (-(dy + 1) as f64 + rel_y) * cell_size_y } else { 0.0 };
                let dz_dist = if dz > 0 { (dz as f64 - rel_z) * cell_size_z } else if dz < 0 { (-(dz + 1) as f64 + rel_z) * cell_size_z } else { 0.0 };

                let dx_bin = dx_dist.max(0.0);
                let dy_bin = dy_dist.max(0.0);
                let dz_bin = dz_dist.max(0.0);
                
                if dx_bin * dx_bin + dy_bin * dy_bin + dz_bin * dz_bin <= 4.0 * *max_dist_sq {
                    let bin_index = (bx as usize) + (by as usize) * self.grid_res_x + (bz as usize) * self.grid_res_x * self.grid_res_y;
                    for &j in &self.grid_bins[bin_index] {
                        if index == j { continue; }
                        let ox = generators[j * 3];
                        let oy = generators[j * 3 + 1];
                        let oz = generators[j * 3 + 2];
                        *max_dist_sq = visitor(j, [ox, oy, oz], *max_dist_sq);
                    }
                }
            }
        }
    }
}

fn get_min_dist_sq(dx: isize, dy: isize, dz: isize, cx: f64, cy: f64, cz: f64) -> f64 {
    let mx = if dx > 0 { (dx - 1) as f64 * cx } else if dx < 0 { (-dx - 1) as f64 * cx } else { 0.0 };
    let my = if dy > 0 { (dy - 1) as f64 * cy } else if dy < 0 { (-dy - 1) as f64 * cy } else { 0.0 };
    let mz = if dz > 0 { (dz - 1) as f64 * cz } else if dz < 0 { (-dz - 1) as f64 * cz } else { 0.0 };
    mx * mx + my * my + mz * mz
}