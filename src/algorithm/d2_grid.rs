use crate::bounds::BoundingBox;
use crate::algorithm::SpatialAlgorithm;

/// A spatial index based on a uniform grid for 2D space.
///
/// This structure divides the 2D space into a fixed number of bins.
/// It allows for O(1) insertion and update operations.
pub struct AlgorithmGrid2D {
    /// Number of bins along the X axis.
    pub grid_res_x: usize,
    /// Number of bins along the Y axis.
    pub grid_res_y: usize,
    /// Scale factor for X coordinate to grid index.
    pub grid_scale_x: f64,
    /// Scale factor for Y coordinate to grid index.
    pub grid_scale_y: f64,
    /// Maximum valid index for X.
    pub grid_limit_x: f64,
    /// Maximum valid index for Y.
    pub grid_limit_y: f64,
    /// Minimum X coordinate of the grid bounds.
    pub min_x: f64,
    /// Minimum Y coordinate of the grid bounds.
    pub min_y: f64,
    /// The grid bins, each containing a list of generator indices.
    pub grid_bins: Vec<Vec<usize>>,
    /// Map from generator index to its current bin index.
    pub generator_bin_ids: Vec<usize>,
    /// Precomputed search order for visiting neighboring bins.
    pub bin_search_order: Vec<(isize, isize, f64)>,
}

impl AlgorithmGrid2D {
    /// Creates a new `AlgorithmGrid2D` with the specified dimensions and bounds.
    pub fn new(nx: usize, ny: usize, bounds: &BoundingBox<2>) -> Self {
        let sx = (nx as f64) / (bounds.max[0] - bounds.min[0]);
        let sy = (ny as f64) / (bounds.max[1] - bounds.min[1]);

        let cell_size_x = 1.0 / sx;
        let cell_size_y = 1.0 / sy;

        let mut bin_search_order = Vec::new();
        let rx = nx as isize;
        let ry = ny as isize;

        let get_min_dist_sq = |dx: isize, dy: isize| {
            let mx = if dx > 0 { (dx - 1) as f64 * cell_size_x } else if dx < 0 { (-dx - 1) as f64 * cell_size_x } else { 0.0 };
            let my = if dy > 0 { (dy - 1) as f64 * cell_size_y } else if dy < 0 { (-dy - 1) as f64 * cell_size_y } else { 0.0 };
            mx * mx + my * my
        };

        for y in -ry..=ry {
            for x in -rx..=rx {
                let dist_sq = get_min_dist_sq(x, y);
                bin_search_order.push((x, y, dist_sq));
            }
        }
        bin_search_order.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        AlgorithmGrid2D {
            grid_res_x: nx,
            grid_res_y: ny,
            grid_scale_x: sx,
            grid_scale_y: sy,
            grid_limit_x: (nx as f64) - 1e-5,
            grid_limit_y: (ny as f64) - 1e-5,
            min_x: bounds.min[0],
            min_y: bounds.min[1],
            grid_bins: vec![Vec::new(); nx * ny],
            generator_bin_ids: Vec::new(),
            bin_search_order,
        }
    }

    /// Calculates the linear index of the bin corresponding to the given coordinates.
    pub fn get_bin_index(&self, x: f64, y: f64, bounds: &BoundingBox<2>) -> usize {
        let nx = self.grid_res_x;

        let ix = ((x - bounds.min[0]) * self.grid_scale_x).clamp(0.0, self.grid_limit_x) as usize;
        let iy = ((y - bounds.min[1]) * self.grid_scale_y).clamp(0.0, self.grid_limit_y) as usize;

        ix + iy * nx
    }
}

impl SpatialAlgorithm<2> for AlgorithmGrid2D {
    fn set_generators(&mut self, generators: &[f64], bounds: &BoundingBox<2>) {
        let total_bins = self.grid_res_x * self.grid_res_y;
        self.grid_bins.iter_mut().for_each(|bin| bin.clear());
        if self.grid_bins.len() != total_bins {
            self.grid_bins = vec![Vec::new(); total_bins];
        }
        
        let count = generators.len() / 2;
        self.generator_bin_ids = vec![0; count];

        for i in 0..count {
            let x = generators[i * 2];
            let y = generators[i * 2 + 1];
            let bin_idx = self.get_bin_index(x, y, bounds);
            
            self.grid_bins[bin_idx].push(i);
            self.generator_bin_ids[i] = bin_idx;
        }
    }

    fn update_generator(&mut self, index: usize, _old_pos: &[f64; 2], new_pos: &[f64; 2], bounds: &BoundingBox<2>) {
        let new_bin_idx = self.get_bin_index(new_pos[0], new_pos[1], bounds);
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
        pos: [f64; 2],
        max_dist_sq: &mut f64,
        mut visitor: F,
    ) where
        F: FnMut(usize, [f64; 2], f64) -> f64,
    {
        let bin_idx = self.generator_bin_ids[index];
        let idx_y = bin_idx / self.grid_res_x;
        let idx_x = bin_idx % self.grid_res_x;

        let cell_size_x = 1.0 / self.grid_scale_x;
        let cell_size_y = 1.0 / self.grid_scale_y;

        let rel_x = (pos[0] - self.min_x) * self.grid_scale_x - idx_x as f64;
        let rel_y = (pos[1] - self.min_y) * self.grid_scale_y - idx_y as f64;

        for &(dx, dy, min_d2) in &self.bin_search_order {
            if min_d2 > 4.0 * *max_dist_sq {
                break;
            }

            let bx = idx_x as isize + dx;
            let by = idx_y as isize + dy;

            if bx >= 0 && bx < self.grid_res_x as isize &&
               by >= 0 && by < self.grid_res_y as isize {
                
                let dx_dist = if dx > 0 { (dx as f64 - rel_x) * cell_size_x } else if dx < 0 { (-(dx + 1) as f64 + rel_x) * cell_size_x } else { 0.0 };
                let dy_dist = if dy > 0 { (dy as f64 - rel_y) * cell_size_y } else if dy < 0 { (-(dy + 1) as f64 + rel_y) * cell_size_y } else { 0.0 };

                let dx_bin = dx_dist.max(0.0);
                let dy_bin = dy_dist.max(0.0);
                
                if dx_bin * dx_bin + dy_bin * dy_bin <= 4.0 * *max_dist_sq {
                    let bin_index = (bx as usize) + (by as usize) * self.grid_res_x;
                    for &j in &self.grid_bins[bin_index] {
                        if index == j { continue; }
                        let ox = generators[j * 2];
                        let oy = generators[j * 2 + 1];
                        *max_dist_sq = visitor(j, [ox, oy], *max_dist_sq);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_indexing_2d() {
        let bounds = BoundingBox::new([0.0, 0.0], [10.0, 10.0]);
        let grid = AlgorithmGrid2D::new(10, 10, &bounds); // 1x1 cells

        let idx = grid.get_bin_index(0.5, 0.5, &bounds);
        assert_eq!(idx, 0);

        let idx = grid.get_bin_index(1.5, 0.5, &bounds);
        assert_eq!(idx, 1);
        
        let idx = grid.get_bin_index(0.5, 1.5, &bounds);
        assert_eq!(idx, 10);
    }

    #[test]
    fn test_grid_neighbors_2d() {
        let bounds = BoundingBox::new([0.0, 0.0], [3.0, 3.0]);
        let mut grid = AlgorithmGrid2D::new(3, 3, &bounds);
        let generators = vec![0.5, 0.5, 1.5, 0.5];
        
        grid.set_generators(&generators, &bounds);
        
        let mut neighbors = Vec::new();
        let mut max_dist_sq = 2.0;
        
        grid.visit_neighbors(&generators, 0, [0.5, 0.5], &mut max_dist_sq, |idx, _, d| {
            neighbors.push(idx);
            d
        });
        assert!(neighbors.contains(&1));
    }
}