use crate::bounds::BoundingBox;
use crate::wall::Wall;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;

/// A geometry-based Voronoi tessellation that unifies the [`Cell`], [`SpatialAlgorithm`], and [`Wall`] traits.
pub struct Tessellation<const D: usize, C: Cell<D>, A: SpatialAlgorithm<D>> {
    pub bounds: BoundingBox<D>,
    pub generators: Vec<f64>,
    pub cells: Vec<C>,
    pub walls: Vec<Wall<D>>,
    pub algorithm: A,
}

impl<const D: usize, C: Cell<D>, A: SpatialAlgorithm<D>> Tessellation<D, C, A> {
    pub fn new(bounds: BoundingBox<D>, algorithm: A) -> Self {
        Self {
            bounds,
            generators: Vec::new(),
            cells: Vec::new(),
            walls: Vec::new(),
            algorithm,
        }
    }

    /// Update all generators at once. Only accepts generators that are inside the
    /// bounding box and contained by the walls.
    /// 
    /// # Arguments
    /// * `generators` - A flat array of coordinates `[x, y, z, ..., x, y, z, ...]`.
    pub fn set_generators(&mut self, generators: &[f64]) {
        let mut valid_generators = Vec::with_capacity(generators.len());
        let count = generators.len() / D;

        for i in 0..count {
            let offset = i * D;
            let point_slice = &generators[offset..offset + D];
            if let Ok(point) = point_slice.try_into() {
                let mut inside = true;
                for wall in &self.walls {
                    if !wall.contains(point) {
                        inside = false;
                        break;
                    }
                }
                if inside {
                    valid_generators.extend_from_slice(point_slice);
                }
            }
        }

        valid_generators.shrink_to_fit();
        self.generators = valid_generators;
        self.algorithm.set_generators(&self.generators, &self.bounds);
    }

    /// Update the position of a single generator by index. Only sets the generator
    /// if it is inside the bounding box and contained by the walls.
    pub fn set_generator(&mut self, index: usize, generator: &[f64; D]) {
        let offset = index * D;
        if offset + D > self.generators.len() {
            return;
        }

        for wall in &self.walls {
            if !wall.contains(generator) {
                return;
            }
        }

        let old_slice = &self.generators[offset..offset + D];
        let old_pos: [f64; D] = old_slice.try_into().unwrap();

        self.algorithm
            .update_generator(index, &old_pos, generator, &self.bounds);

        for (i, &val) in generator.iter().enumerate() {
            self.generators[offset + i] = val;
        }
    }

    /// Generates random points within the boundaries of the bounding box
    /// and walls and sets them as generators.
    pub fn random_generators(&mut self, count: usize) {
        let mut rng = StdRng::seed_from_u64(get_seed());
        let mut points = Vec::with_capacity(count * D);
        
        let mut found = 0;
        let max_attempts = count * 1000;
        let mut attempts = 0;

        while found < count && attempts < max_attempts {
            attempts += 1;
            let mut point = [0.0; D];
            for i in 0..D {
                let min = self.bounds.min[i];
                let max = self.bounds.max[i];
                point[i] = min + rng.r#gen::<f64>() * (max - min);
            }

            if self.walls.iter().all(|w| w.contains(&point)) {
                points.extend_from_slice(&point);
                found += 1;
            }
        }
        
        self.generators = points;
        self.algorithm.set_generators(&self.generators, &self.bounds);
    }

    /// Imports generators from a text file.
    /// Each line should contain an id followed by N coordinate entries.
    /// For now only the first 3 coordinates are used (x, y, z) and the id is ignored.
    pub fn import_generators<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut raw_points = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }

            let mut parts = line.split_whitespace();
            
            // Skip ID
            if parts.next().is_none() { continue; }

            for _ in 0..D {
                let val = parts.next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                raw_points.push(val);
            }
        }

        self.set_generators(&raw_points);
        Ok(())
    }

    /// Removes generators that are not inside the defined walls.
    /// Note: This changes the indices of the remaining generators.
    fn prune_outside_generators(&mut self) {
        let mut new_generators = Vec::with_capacity(self.generators.len());
        let count = self.generators.len() / D;
        
        for i in 0..count {
            let offset = i * D;
            let point_slice = &self.generators[offset..offset + D];
            if let Ok(point) = point_slice.try_into() {
                if self.walls.iter().all(|w| w.contains(point)) {
                    new_generators.extend_from_slice(point_slice);
                }
            }
        }
        
        if new_generators.len() != self.generators.len() {
            new_generators.shrink_to_fit();
            self.generators = new_generators;
            self.algorithm.set_generators(&self.generators, &self.bounds);
        }
    }

    /// Adds a wall to the tessellation to clip the Voronoi cells.
    pub fn add_wall(&mut self, wall: Wall<D>) {
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
        let count = self.generators.len() / D;
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

    /// Computes cells and applies a mapping function `f` to each cell, returning the collected results.
    ///
    /// This method is memory-efficient as it does not store the intermediate `Cell` objects.
    /// It runs in parallel if the `rayon` feature is enabled (which is default).
    pub fn map<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(C) -> T + Sync + Send,
        T: Send,
    {
        let count = self.generators.len() / D;
        let generators = &self.generators;
        let bounds = &self.bounds;
        let walls = &self.walls;
        let algorithm = &self.algorithm;

        (0..count)
            .into_par_iter()
            .map_init(
                || C::Scratch::default(),
                |scratch, i| {
                    let cell = Self::compute_cell(i, generators, bounds, walls, algorithm, scratch);
                    f(cell)
                },
            )
            .collect()
    }

    fn compute_cell(
        i: usize,
        generators: &[f64],
        bounds: &BoundingBox<D>,
        walls: &[Wall<D>],
        algorithm: &A,
        scratch: &mut C::Scratch,
    ) -> C {
        let offset = i * D;
        let g_slice = &generators[offset..offset + D];
        let g_pos: [f64; D] = g_slice.try_into().unwrap();

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
                let mut dist_sq = 0.0;
                let mut midpoint = [0.0; D];
                let mut normal = [0.0; D];

                for k in 0..D {
                    let d = n_pos[k] - g_pos[k];
                    dist_sq += d * d;
                    midpoint[k] = g_pos[k] + d * 0.5;
                    normal[k] = d;
                }

                if dist_sq > 4.0 * cur_dist {
                    return cur_dist;
                }

                if let (true, new_radius) =
                    cell.clip(&midpoint, &normal, j as i32, scratch, Some(&g_pos))
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
    
    /// Performs one step of Lloyd's relaxation.
    ///
    /// This moves each generator to the centroid of its calculated Voronoi cell,
    /// which tends to make the cells more uniform in size and shape. A calculation
    /// step must be invoked separately to get the new Voronoi cells.
    pub fn relax(&mut self) {
        let new_generators: Vec<f64> = self.cells.par_iter()
            .zip(self.generators.par_chunks(D))
            .flat_map(|(cell, original_pos)| {
                if cell.is_empty() {
                    original_pos.to_vec()
                } else {
                    cell.centroid().to_vec()
                }
            })
            .collect();

        self.set_generators(&new_generators);
    }

    /// Returns the number of generators in the tessellation.
    pub fn count_generators(&self) -> usize {
        self.generators.len() / D
    }

    /// Returns the number of computed cells.
    pub fn count_cells(&self) -> usize {
        self.cells.len()
    }

    /// Retrieves the position as `[f64; 3]` of a generator by its index.
    pub fn get_generator(&self, index: usize) -> [f64; D] {
        let offset = index * D;
        self.generators[offset..offset + D].try_into().unwrap()
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
    pub fn cells(&self) -> Vec<C> {
        self.cells.clone()
    }
}


/// Trait defining the behavior of a Voronoi cell.
/// This allows swapping between simple Polygon cells (`Cell`) and Graph-based cells (`CellEdges`).
pub trait Cell<const D: usize>: Send + Sync + Sized + Clone {
    /// Scratch buffer used to avoid allocations during clipping.
    type Scratch: Default + Clone + Send;

    /// Initialize a new cell for the given generator index and bounds.
    fn new(id: usize, bounds: BoundingBox<D>) -> Self;

    /// Clip the cell by a plane defined by `point` and `normal`.
    /// Returns `(true, new_max_radius_sq)` if the cell was modified, or `(false, 0.0)` if not.
    fn clip(
        &mut self,
        point: &[f64; D],
        normal: &[f64; D],
        neighbor_id: i32,
        scratch: &mut Self::Scratch,
        generator: Option<&[f64; D]>,
    ) -> (bool, f64);

    /// Calculate the squared distance from the center to the furthest vertex.
    fn max_radius_sq(&self, center: &[f64; D]) -> f64;

    /// Calculate the centroid of the cell.
    fn centroid(&self) -> [f64; D];

    /// Check if the cell is empty (collapsed).
    fn is_empty(&self) -> bool;
}


/// Trait defining a spatial acceleration structure.
/// This allows swapping between Grid, Octree, or Linear Octree algorithms.
pub trait SpatialAlgorithm<const D: usize>: Send + Sync {
    /// Rebuild the index with new generators.
    fn set_generators(&mut self, generators: &[f64], bounds: &BoundingBox<D>);

    /// Update the position of a single generator.
    fn update_generator(&mut self, index: usize, old_pos: &[f64; D], new_pos: &[f64; D], bounds: &BoundingBox<D>);

    /// Visit potential neighbors for a given generator.
    ///
    /// # Arguments
    /// * `generators` - The full list of generators (needed to retrieve neighbor positions).
    /// * `index` - The index of the generator we are processing.
    /// * `pos` - The position of the generator (array of size D).
    /// * `max_dist_sq` - A mutable reference to the current maximum search radius squared.
    ///                   The visitor can update this if the cell shrinks.
    /// * `visitor` - A closure called for each candidate neighbor. It receives the neighbor's index
    ///               and its position.
    fn visit_neighbors<F>(&self, generators: &[f64], index: usize, pos: [f64; D], max_dist_sq: &mut f64, visitor: F)
    where
        F: FnMut(usize, [f64; D], f64) -> f64;
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
        let bounds = BoundingBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let algo = AlgorithmGrid::new(2, 2, 2, &bounds);
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds, algo);

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
        let bounds = BoundingBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let algo = AlgorithmGrid::new(2, 2, 2, &bounds);
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds, algo);

        tess.set_generators(&[0.1, 0.1, 0.1, 0.9, 0.9, 0.9]);
        tess.calculate();
        tess.relax();
        assert_eq!(tess.count_generators(), 2);
    }
}