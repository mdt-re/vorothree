use crate::bounds::BoundingBox;
use crate::cell::Cell;
use crate::wall::Wall;
use crate::moctree::Moctree;
use wasm_bindgen::prelude::*;
use rayon::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

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

    pub fn add_wall(&mut self, wall: Wall) {
        self.walls.push(wall);
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
        let octree = &self.octree;

        self.cells = (0..count).into_par_iter().map(|i| {
            let gx: f64 = generators[i * 3];
            let gy: f64 = generators[i * 3 + 1];
            let gz: f64 = generators[i * 3 + 2];

            let mut cell: Cell = Cell::new(i, bounds.clone());

            for wall in walls {
                wall.cut(&[gx, gy, gz], &mut |point, normal| {
                    cell.clip(&point, &normal, wall.id());
                });
            }

            for j in octree.nearest_iter(gx, gy, gz) {
                if i == j { continue; }

                let ox: f64 = generators[j * 3];
                let oy: f64 = generators[j * 3 + 1];
                let oz: f64 = generators[j * 3 + 2];

                let dx: f64 = ox - gx;
                let dy: f64 = oy - gy;
                let dz: f64 = oz - gz;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                // Optimization: check if we can stop
                let mut max_dist_sq = 0.0;
                for k in 0..cell.vertices.len() / 3 {
                    let vx = cell.vertices[k * 3] - gx;
                    let vy = cell.vertices[k * 3 + 1] - gy;
                    let vz = cell.vertices[k * 3 + 2] - gz;
                    let d2 = vx * vx + vy * vy + vz * vz;
                    if d2 > max_dist_sq {
                        max_dist_sq = d2;
                    }
                }

                if dist_sq > 4.0 * max_dist_sq {
                    break;
                }

                let mx: f64 = gx + dx * 0.5;
                let my: f64 = gy + dy * 0.5;
                let mz: f64 = gz + dz * 0.5;

                cell.clip(&[mx, my, mz], &[dx, dy, dz], j as i32);
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
        self.generators = generators.to_vec();
        
        self.octree.clear();
        let count = self.generators.len() / 3;
        for i in 0..count {
            self.octree.insert(i, self.generators[i*3], self.generators[i*3+1], self.generators[i*3+2]);
        }
    }

    pub fn set_generator(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let offset = index * 3;
        if offset + 2 < self.generators.len() {
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

    pub fn get(&self, index: usize) -> Option<Cell> {
        self.cells.get(index).cloned()
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