use crate::bounds::BoundingBox;
use crate::wall::Wall;
use rayon::prelude::*;

/// A geometry-based Voronoi tessellation that unifies Cells, SpatialAlgorithms, and Walls.
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

    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
    }

    pub fn set_generators(&mut self, generators: Vec<f64>) {
        // Note: Pruning logic could be moved here or kept in specific implementations
        self.generators = generators;
        self.algorithm.set_generators(&self.generators, &self.bounds);
    }

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
                |scratch, i| {
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
                    }

                    let mut current_max_dist_sq = cell.max_radius_sq(&g_pos);

                    // 2. Clip against neighbors found by the SpatialAlgorithm
                    algorithm.visit_neighbors(generators, i, g_pos, &mut current_max_dist_sq, |j, n_pos, cur_dist| {
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
                            return new_radius;
                        }
                        cur_dist
                    });

                    cell
                },
            )
            .collect();
    }

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

        self.set_generators(new_generators);
    }
}


/// Trait defining the behavior of a Voronoi cell.
/// This allows swapping between simple Polygon cells (`Cell`) and Graph-based cells (`CellEdges`).
pub trait Cell: Send + Sync + Sized {
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