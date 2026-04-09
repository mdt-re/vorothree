use crate::bounds::BoundingBox;
use crate::algorithm::SpatialAlgorithm;
use crate::cell::Cell;
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
    pub seal_log: Vec<i32>,
    pub prune_log: Vec<i32>,
    pub prune_pos_log: Vec<f64>,
}

impl<const D: usize, C: Cell<D>, A: SpatialAlgorithm<D>> Tessellation<D, C, A> {
    pub fn new(bounds: BoundingBox<D>, algorithm: A) -> Self {
        Self {
            bounds,
            generators: Vec::new(),
            cells: Vec::new(),
            walls: Vec::new(),
            algorithm,
            seal_log: Vec::new(),
            prune_log: Vec::new(),
            prune_pos_log: Vec::new(),
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
    /// Each line should contain an id followed by D coordinate entries.
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

    /// Reads generators from a string.
    /// Each line should contain an id followed by D coordinate entries.
    /// The id is ignored.
    pub fn read_generators(&mut self, input: &str) {
        let mut raw_points = Vec::new();

        for line in input.lines() {
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
        self.seal_log.clear();
        self.prune_log.clear();
        self.prune_pos_log.clear();
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

    /// Calculates cells, and then runs a post-processing pass to share
    /// curved wall tangent planes between neighbors, guaranteeing watertight boundaries.
    pub fn calculate_sealed(&mut self) {
        self.calculate();

        let count = self.generators.len() / D;
        let generators = &self.generators;
        let walls = &self.walls;

        // 2. Extract neighbor topologies
        // We need to know who neighbors who before we start mutating cells.
        let mut topologies: Vec<Vec<usize>> = Vec::with_capacity(count);
        let mut cell_walls: Vec<Vec<i32>> = Vec::with_capacity(count);
        for cell in &self.cells {
            let mut neighbors = Vec::new();
            let mut non_planar_walls = Vec::new();
            for &face_neighbor in cell.neighbors() {
                if face_neighbor >= 0 && (face_neighbor as usize) < count {
                    neighbors.push(face_neighbor as usize);
                } else {
                    if let Some(wall) = walls.iter().find(|w| w.id() == face_neighbor) {
                        if !wall.is_planar() && !non_planar_walls.contains(&face_neighbor) {
                            non_planar_walls.push(face_neighbor);
                        }
                    }
                }
            }
            topologies.push(neighbors);
            cell_walls.push(non_planar_walls);
        }

        // 3. Post-Op Planar Consensus (Pass 2)
        // Note: Using `par_iter_mut` so we can clip cells in parallel!
        let logs: Vec<Vec<i32>> = self.cells.par_iter_mut().enumerate().filter_map(|(i, cell)| {
            let my_walls = &cell_walls[i];
            if my_walls.is_empty() {
                return None;
            }

            let mut scratch = C::Scratch::default();
            let offset_i = i * D;
            let g_pos: [f64; D] = generators[offset_i..offset_i + D].try_into().unwrap();
            let mut local_log = Vec::new();
            
            // For every neighbor this cell has...
            for &neighbor_idx in &topologies[i] {
                let neighbor_walls = &cell_walls[neighbor_idx];

                let offset_n = neighbor_idx * D;
                let n_slice = &generators[offset_n..offset_n + D];
                let n_pos: [f64; D] = n_slice.try_into().unwrap();

                // Check if the neighbor generated any tangent planes from curved walls
                for wall in walls {
                    if !wall.is_planar() && my_walls.contains(&wall.id()) && neighbor_walls.contains(&wall.id()) {
                        // We ask the wall for the tangent plane *as if we were the neighbor*
                        wall.cut(&n_pos, &mut |point, normal| {
                            // Validate the neighbor's tangent plane against our own generator.
                            // On concave curves (convex obstacles), a neighbor's plane points 
                            // outward and would severely truncate or destroy this cell.
                            let mut dot = 0.0;
                            for k in 0..D {
                                dot += (g_pos[k] - point[k]) * normal[k];
                            }
                            
                            // Only apply the plane if our generator is on the valid side of it
                            if dot <= 1e-9 {
                                let (modified, _) = cell.clip(&point, &normal, wall.id(), &mut scratch, None);
                                if modified {
                                    local_log.push(i as i32);
                                    local_log.push(neighbor_idx as i32);
                                    local_log.push(wall.id());
                                }
                            }
                        });
                    }
                }
            }
            if local_log.is_empty() {
                None
            } else {
                Some(local_log)
            }
        }).collect();

        self.seal_log = logs.into_iter().flatten().collect();
    }

    /// Runs a post-processing pass to prune the cells faces at the boundaries.
    /// It queries additional tangent planes from curved walls by interpolating
    /// between the cell's generator and its neighbors' generators, creating a beveled, smoother surface.
    pub fn prune_boundaries(&mut self) {
        let count = self.generators.len() / D;
        let walls = &self.walls;

        let mut topologies: Vec<Vec<usize>> = Vec::with_capacity(count);
        let mut cell_walls: Vec<Vec<i32>> = Vec::with_capacity(count);
        for cell in &self.cells {
            let mut neighbors = Vec::new();
            let mut non_planar_walls = Vec::new();
            for &face_neighbor in cell.neighbors() {
                if face_neighbor >= 0 && (face_neighbor as usize) < count {
                    if !neighbors.contains(&(face_neighbor as usize)) {
                        neighbors.push(face_neighbor as usize);
                    }
                } else {
                    if let Some(wall) = walls.iter().find(|w| w.id() == face_neighbor) {
                        if !wall.is_planar() && !non_planar_walls.contains(&face_neighbor) {
                            non_planar_walls.push(face_neighbor);
                        }
                    }
                }
            }
            topologies.push(neighbors);
            cell_walls.push(non_planar_walls);
        }

        let logs: Vec<(Vec<i32>, Vec<f64>)> = self.cells.par_iter().enumerate().filter_map(|(i, cell)| {
            let my_walls = &cell_walls[i];
            if my_walls.is_empty() {
                return None;
            }

            let mut local_log = Vec::new();
            let mut local_pos_log = Vec::new();
            
            for &neighbor_idx in &topologies[i] {
                let neighbor_walls = &cell_walls[neighbor_idx];

                for wall in walls {
                    if !wall.is_planar() && my_walls.contains(&wall.id()) && neighbor_walls.contains(&wall.id()) {
                        
                        let shared = cell.shared_vertices(wall.id(), neighbor_idx as i32);
                        if !shared.is_empty() {
                            local_log.push(i as i32);
                            local_log.push(neighbor_idx as i32);
                            local_log.push(wall.id());
                            local_log.push((shared.len() / D) as i32);
                            local_pos_log.extend_from_slice(&shared);
                        }
                    }
                }
            }
            if local_log.is_empty() {
                None
            } else {
                Some((local_log, local_pos_log))
            }
        }).collect();

        let (prune_logs, prune_pos_logs): (Vec<_>, Vec<_>) = logs.into_iter().unzip();
        self.prune_log = prune_logs.into_iter().flatten().collect();
        self.prune_pos_log = prune_pos_logs.into_iter().flatten().collect();
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