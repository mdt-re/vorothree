use crate::bounds::BoundingBox;
use crate::wall::Wall;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use crate::algo_grid::AlgorithmGrid;

/// A geometry-based Voronoi tessellation that unifies the [`Cell`], [`SpatialAlgorithm`], and [`Wall`] traits.
pub struct Tessellation<C: Cell, A: SpatialAlgorithm> {
    pub bounds: BoundingBox,
    pub generators: Vec<f64>,
    pub cells: Vec<C>,
    pub walls: Vec<Wall>,
    pub algorithm: A,
}

impl<C: Cell, A: SpatialAlgorithm> Tessellation<C, A> {
    pub fn new(bounds: BoundingBox, algorithm: A) -> Self {
        Self {
            bounds,
            generators: Vec::new(),
            cells: Vec::new(),
            walls: Vec::new(),
            algorithm,
        }
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
        self.algorithm.set_generators(&self.generators, &self.bounds);
    }

    /// Update the position of a single generator by index.
    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 >= self.generators.len() {
            return;
        }

        for wall in &self.walls {
            if !wall.contains(x, y, z) {
                return;
            }
        }

        let old_pos = [
            self.generators[offset],
            self.generators[offset + 1],
            self.generators[offset + 2],
        ];
        let new_pos = [x, y, z];

        self.algorithm
            .update_generator(index, &old_pos, &new_pos, &self.bounds);

        self.generators[offset] = x;
        self.generators[offset + 1] = y;
        self.generators[offset + 2] = z;
    }

    /// Generates random points within the bounds and sets them as generators.
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

            if self.walls.iter().all(|w| w.contains(x, y, z)) {
                points.push(x);
                points.push(y);
                points.push(z);
                found += 1;
            }
        }
        
        self.generators = points;
        self.algorithm.set_generators(&self.generators, &self.bounds);
    }

    /// Removes generators that are not inside the defined walls.
    /// Note: This changes the indices of the remaining generators.
    fn prune_outside_generators(&mut self) {
        let mut new_generators = Vec::with_capacity(self.generators.len());
        let count = self.generators.len() / 3;
        
        for i in 0..count {
            let x = self.generators[i * 3];
            let y = self.generators[i * 3 + 1];
            let z = self.generators[i * 3 + 2];
            
            if self.walls.iter().all(|w| w.contains(x, y, z)) {
                new_generators.push(x);
                new_generators.push(y);
                new_generators.push(z);
            }
        }
        
        if new_generators.len() != self.generators.len() {
            self.generators = new_generators;
            self.algorithm.set_generators(&self.generators, &self.bounds);
        }
    }

    /// Adds a wall to the tessellation to clip the Voronoi cells.
    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
        self.prune_outside_generators();
    }

    /// Removes all walls from the tessellation.
    pub fn clear_walls(&mut self) {
        self.walls.clear();
    }

    /// Calculates all cells based on the current generators.
    ///
    /// This method applies the SpatialAlgorithm to efficiently find the closest generators
    /// and clips the cells against the generators, the bounding box and any added walls.
    /// For the clipping it applies the algoritm as defined in the Cell implementation.
    /// It runs in parallel if the `rayon` feature is enabled (which is default).
    pub fn calculate(&mut self) {
        let count = self.generators.len() / 3;
        let generators = &self.generators;
        let bounds = &self.bounds;
        let walls = &self.walls;
        let algorithm = &self.algorithm;

        self.cells = (0..count)
            .into_par_iter()
            .map_init(
                || C::Scratch::default(),
                |scratch, i| Self::compute_cell(i, generators, bounds, walls, algorithm, scratch),
            )
            .collect();
    }

    fn compute_cell(
        i: usize,
        generators: &[f64],
        bounds: &BoundingBox,
        walls: &[Wall],
        algorithm: &A,
        scratch: &mut C::Scratch,
    ) -> C {
        let gx = generators[i * 3];
        let gy = generators[i * 3 + 1];
        let gz = generators[i * 3 + 2];
        let g_pos = [gx, gy, gz];

        let mut cell = C::new(i, *bounds);

        // 1. Clip against walls
        for wall in walls {
            wall.cut(&g_pos, &mut |point, normal| {
                cell.clip(&point, &normal, wall.id(), scratch, None);
            });
            if cell.is_empty() {
                return cell;
            }
        }

        let mut current_max_dist_sq = cell.max_radius_sq(&g_pos);

        // 2. Clip against neighbors found by the SpatialAlgorithm
        algorithm.visit_neighbors(
            generators,
            i,
            g_pos,
            &mut current_max_dist_sq,
            |j, n_pos, cur_dist| {
                let dx = n_pos[0] - gx;
                let dy = n_pos[1] - gy;
                let dz = n_pos[2] - gz;

                // Note: Some indices might not filter exact distance, so we check here
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if dist_sq > 4.0 * cur_dist {
                    return cur_dist;
                }

                let mx = gx + dx * 0.5;
                let my = gy + dy * 0.5;
                let mz = gz + dz * 0.5;

                if let (true, new_radius) =
                    cell.clip(&[mx, my, mz], &[dx, dy, dz], j as i32, scratch, Some(&g_pos))
                {
                    if cell.is_empty() {
                        return 0.0;
                    }
                    return new_radius;
                }
                cur_dist
            },
        );

        cell
    }

    /// Returns the number of generators in the tessellation.
    pub fn count_generators(&self) -> usize {
        self.generators.len() / 3
    }

    /// Returns the number of computed cells.
    pub fn count_cells(&self) -> usize {
        self.cells.len()
    }

    /// Retrieves the position as `[f64; 3]` of a generator by its index.
    pub fn get_generator(&self, index: usize) -> [f64; 3] {
        let offset = index * 3;
        [
            self.generators[offset],
            self.generators[offset + 1],
            self.generators[offset + 2],
        ]
    }

    /// Retrieves a cell by its index.
    pub fn get_cell(&self, index: usize) -> Option<C> {
        self.cells.get(index).cloned()
    }

    /// Returns a copy of all generator positions as a flat vector.
    pub fn generators(&self) -> Vec<f64> {
        self.generators.clone()
    }

    /// Returns a copy of all computed cells.
    pub fn get_cells(&self) -> Vec<C> {
        self.cells.clone()
    }

    /// Performs one step of Lloyd's relaxation.
    ///
    /// This moves each generator to the centroid of its calculated Voronoi cell,
    /// which tends to make the cells more uniform in size and shape. A calculation
    /// step must be invoked separately to get the new Voronoi cells.
    pub fn relax(&mut self) {
        let new_generators: Vec<f64> = self.cells.par_iter()
            .zip(self.generators.par_chunks(3))
            .flat_map(|(cell, original_pos)| {
                if cell.is_empty() {
                    original_pos.to_vec()
                } else {
                    cell.centroid()
                }
            })
            .collect();

        self.set_generators(&new_generators);
    }
}

impl<C: Cell> Tessellation<C, AlgorithmGrid> {
    /// Resizes the internal spatial partitioning grid.
    pub fn set_grid_shape(&mut self, nx: usize, ny: usize, nz: usize) {
        self.algorithm = AlgorithmGrid::new(nx, ny, nz, &self.bounds);
        let current_gens = self.generators.clone();
        self.set_generators(&current_gens);
    }
}

/// Trait defining the behavior of a Voronoi cell.
/// This allows swapping between simple Polygon cells (`Cell`) and Graph-based cells (`CellEdges`).
pub trait Cell: Send + Sync + Sized + Clone {
    /// Scratch buffer used to avoid allocations during clipping.
    type Scratch: Default + Clone + Send;

    /// Initialize a new cell for the given generator index and bounds.
    fn new(id: usize, bounds: BoundingBox) -> Self;

    /// Clip the cell by a plane defined by `point` and `normal`.
    /// Returns `(true, new_max_radius_sq)` if the cell was modified, or `(false, 0.0)` if not.
    fn clip(
        &mut self,
        point: &[f64],
        normal: &[f64],
        neighbor_id: i32,
        scratch: &mut Self::Scratch,
        generator: Option<&[f64]>,
    ) -> (bool, f64);

    /// Calculate the squared distance from the center to the furthest vertex.
    fn max_radius_sq(&self, center: &[f64]) -> f64;

    /// Calculate the centroid of the cell.
    fn centroid(&self) -> Vec<f64>;

    /// Check if the cell is empty (collapsed).
    fn is_empty(&self) -> bool;
}


/// Trait defining a spatial acceleration structure.
/// This allows swapping between Grid, Octree, or Linear Octree algorithms.
pub trait SpatialAlgorithm: Send + Sync {
    /// Rebuild the index with new generators.
    fn set_generators(&mut self, generators: &[f64], bounds: &BoundingBox);

    /// Update the position of a single generator.
    fn update_generator(&mut self, index: usize, old_pos: &[f64], new_pos: &[f64], bounds: &BoundingBox);

    /// Visit potential neighbors for a given generator.
    ///
    /// # Arguments
    /// * `generators` - The full list of generators (needed to retrieve neighbor positions).
    /// * `index` - The index of the generator we are processing.
    /// * `pos` - The [x, y, z] position of the generator.
    /// * `max_dist_sq` - A mutable reference to the current maximum search radius squared.
    ///                   The visitor can update this if the cell shrinks.
    /// * `visitor` - A closure called for each candidate neighbor. It receives the neighbor's index
    ///               and its position.
    fn visit_neighbors<F>(&self, generators: &[f64], index: usize, pos: [f64; 3], max_dist_sq: &mut f64, visitor: F)
    where
        F: FnMut(usize, [f64; 3], f64) -> f64;
}

fn get_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Math::random() * 4294967296.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        rand::thread_rng().next_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo_grid::AlgorithmGrid;
    use crate::cell_faces::CellFaces;

    #[test]
    fn test_tessellation_basic() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let algo = AlgorithmGrid::new(2, 2, 2, &bounds);
        let mut tess = Tessellation::<CellFaces, _>::new(bounds, algo);

        // 2 points
        tess.set_generators(&[0.25, 0.5, 0.5, 0.75, 0.5, 0.5]);
        tess.calculate();

        assert_eq!(tess.count_cells(), 2);
        let c1 = tess.get_cell(0).unwrap();
        let c2 = tess.get_cell(1).unwrap();

        // Total volume should be 1.0
        let vol = c1.volume() + c2.volume();
        assert!((vol - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_relax() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let algo = AlgorithmGrid::new(2, 2, 2, &bounds);
        let mut tess = Tessellation::<CellFaces, _>::new(bounds, algo);

        tess.set_generators(&[0.1, 0.1, 0.1, 0.9, 0.9, 0.9]);
        tess.calculate();
        tess.relax();
        assert_eq!(tess.count_generators(), 2);
    }
}