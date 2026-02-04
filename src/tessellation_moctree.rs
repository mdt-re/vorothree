use crate::bounds::BoundingBox;
use crate::cell::{Cell, ClipScratch};
use crate::wall::Wall;
use crate::moctree::Moctree;
use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

/// A Voronoi tessellation container that uses an Octree for spatial partitioning.
///
/// This struct manages:
/// - The **bounding box** of the simulation.
/// - The **generators** (points) that define the Voronoi cells.
/// - The **walls** that clip the cells.
/// - The **octree** used for accelerating nearest-neighbor searches.
#[wasm_bindgen]
pub struct TessellationMoctree {
    bounds: BoundingBox,
    generators: Vec<f64>,
    cells: Vec<Cell>,
    walls: Vec<Wall>,
    octree: Moctree,
}

#[wasm_bindgen]
impl TessellationMoctree {
    /// Creates a new `TessellationMoctree` instance with the specified bounds and octree node capacity.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial boundaries of the tessellation.
    /// * `capacity` - The maximum number of points in a leaf node before subdivision occurs.
    #[wasm_bindgen(constructor)]
    pub fn new(bounds: BoundingBox, capacity: usize) -> TessellationMoctree {
        TessellationMoctree {
            bounds: bounds.clone(),
            generators: Vec::new(),
            cells: Vec::new(),
            walls: Vec::new(),
            octree: Moctree::new(bounds, capacity),
        }
    }

    /// Adds a wall to the tessellation to clip the Voronoi cells.
    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
    }

    /// Removes all walls from the tessellation.
    pub fn clear_walls(&mut self) {
        self.walls.clear();
    }

    /// Returns the number of cells in the tessellation.
    #[wasm_bindgen(getter)]
    pub fn count_cells(&self) -> usize {
        self.cells.len()
    }

    /// Returns a flat array of all generator coordinates [x, y, z, x, y, z, ...].
    #[wasm_bindgen(getter)]
    pub fn generators(&self) -> Vec<f64> {
        self.generators.clone()
    }

    /// Returns the number of generators.
    #[wasm_bindgen(getter)]
    pub fn count_generators(&self) -> usize {
        self.generators.len() / 3
    }

    /// Calculates all cells based on the current generators.
    ///
    /// This method uses the internal octree to efficiently find the closest generators.
    pub fn calculate(&mut self) {
        let count = self.generators.len() / 3;
        let generators = &self.generators;
        let bounds = &self.bounds;
        let walls = &self.walls;
        let octree = &self.octree;

        self.cells = (0..count).into_par_iter().map(|i| {
            let gx: f64 = generators[i * 3];
            let gy: f64 = generators[i * 3 + 1];
            let gz: f64 = generators[i * 3 + 2];

            let mut cell: Cell = Cell::new(i, bounds.clone());
            let mut scratch = ClipScratch::default();

            for wall in walls {
                wall.cut(&[gx, gy, gz], &mut |point, normal| {
                    cell.clip_with_scratch(&point, &normal, wall.id(), &mut scratch, None);
                });
            }

            // Calculate initial max_dist_sq after walls
            let mut current_max_dist_sq = cell.max_radius_sq(&[gx, gy, gz]);

            for j in octree.nearest_iter(gx, gy, gz) {
                if i == j { continue; }

                let ox: f64 = generators[j * 3];
                let oy: f64 = generators[j * 3 + 1];
                let oz: f64 = generators[j * 3 + 2];

                let dx: f64 = ox - gx;
                let dy: f64 = oy - gy;
                let dz: f64 = oz - gz;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq > 4.0 * current_max_dist_sq {
                    break;
                }

                let mx: f64 = gx + dx * 0.5;
                let my: f64 = gy + dy * 0.5;
                let mz: f64 = gz + dz * 0.5;

                if let (true, new_radius) = cell.clip_with_scratch(&[mx, my, mz], &[dx, dy, dz], j as i32, &mut scratch, Some(&[gx, gy, gz])) {
                    current_max_dist_sq = new_radius;
                }
            }
            cell
        }).collect();
    }

    /// Performs one step of Lloyd's relaxation.
    ///
    /// This moves each generator to the centroid of its calculated Voronoi cell,
    /// which tends to make the cells more uniform in size and shape.
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
        
        self.octree.clear();
        let count = self.generators.len() / 3;
        for i in 0..count {
            self.octree.insert(i, self.generators[i*3], self.generators[i*3+1], self.generators[i*3+2]);
        }
    }

    /// Update the position of a single generator by index.
    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 < self.generators.len() {
            for wall in &self.walls {
                if !wall.contains(x, y, z) {
                    return;
                }
            }

            self.generators[offset] = x;
            self.generators[offset + 1] = y;
            self.generators[offset + 2] = z;

            // Rebuild octree
            self.octree.clear();
            let count = self.generators.len() / 3;
            for i in 0..count {
                self.octree.insert(i, self.generators[i*3], self.generators[i*3+1], self.generators[i*3+2]);
            }
        }
    }

    /// Retrieves a calculated cell by index.
    pub fn get(&self, index: usize) -> Option<Cell> {
        self.cells.get(index).cloned()
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