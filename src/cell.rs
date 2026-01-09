use crate::bounds::BoundingBox;
use wasm_bindgen::prelude::*;

// Constants for boundary walls
#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS: &'static str = r#"
export const BOX_BOTTOM = -1;
export const BOX_TOP = -2;
export const BOX_FRONT = -3;
export const BOX_BACK = -4;
export const BOX_LEFT = -5;
export const BOX_RIGHT = -6;
"#;

pub const BOX_BOTTOM: i32 = -1;
pub const BOX_TOP: i32 = -2;
pub const BOX_FRONT: i32 = -3;
pub const BOX_BACK: i32 = -4;
pub const BOX_LEFT: i32 = -5;
pub const BOX_RIGHT: i32 = -6;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Cell {
    pub(crate) id: usize,
    // Flat array of vertices [x, y, z, x, y, z, ...]
    pub(crate) vertices: Vec<f64>,
    // Number of vertices for each face
    pub(crate) face_counts: Vec<u32>,
    // Flattened indices for all faces
    pub(crate) face_indices: Vec<u32>,
    // Neighbor ID for each face. Negative values indicate walls/boundaries.
    pub(crate) face_neighbors: Vec<i32>,
}

#[wasm_bindgen]
impl Cell {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, bounds: BoundingBox) -> Cell {
        let vertices: Vec<f64> = vec![
            bounds.min_x, bounds.min_y, bounds.min_z, // 0
            bounds.max_x, bounds.min_y, bounds.min_z, // 1
            bounds.max_x, bounds.max_y, bounds.min_z, // 2
            bounds.min_x, bounds.max_y, bounds.min_z, // 3
            bounds.min_x, bounds.min_y, bounds.max_z, // 4
            bounds.max_x, bounds.min_y, bounds.max_z, // 5
            bounds.max_x, bounds.max_y, bounds.max_z, // 6
            bounds.min_x, bounds.max_y, bounds.max_z, // 7
        ];

        let face_counts: Vec<u32> = vec![4, 4, 4, 4, 4, 4];

        let face_indices: Vec<u32> = vec![
            3, 2, 1, 0, // Bottom (z-)
            4, 5, 6, 7, // Top (z+)
            0, 1, 5, 4, // Front (y-)
            2, 3, 7, 6, // Back (y+)
            0, 4, 7, 3, // Left (x-)
            1, 2, 6, 5, // Right (x+)
        ];

        Cell {
            id,
            vertices,
            face_counts,
            face_indices,
            face_neighbors: vec![
                BOX_BOTTOM, // z-
                BOX_TOP,    // z+
                BOX_FRONT,  // y-
                BOX_BACK,   // y+
                BOX_LEFT,   // x-
                BOX_RIGHT,  // x+
            ],
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> usize {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f64> {
        self.vertices.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn face_counts(&self) -> Vec<u32> {
        self.face_counts.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn face_indices(&self) -> Vec<u32> {
        self.face_indices.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn face_neighbors(&self) -> Vec<i32> {
        self.face_neighbors.clone()
    }

    pub fn clip(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32) {
        let px = point[0];
        let py = point[1];
        let pz = point[2];
        let nx = normal[0];
        let ny = normal[1];
        let nz = normal[2];

        let num_verts = self.vertices.len() / 3;
        let mut dists = Vec::with_capacity(num_verts);
        let mut all_inside = true;
        let mut all_outside = true;

        // 1. Calculate distances
        for i in 0..num_verts {
            let vx = self.vertices[i * 3];
            let vy = self.vertices[i * 3 + 1];
            let vz = self.vertices[i * 3 + 2];
            let d = (vx - px) * nx + (vy - py) * ny + (vz - pz) * nz;
            dists.push(d);

            if d > 1e-9 {
                all_inside = false;
            } else if d < -1e-9 {
                all_outside = false;
            }
        }

        if all_inside {
            return;
        }
        if all_outside {
            self.vertices.clear();
            self.face_counts.clear();
            self.face_indices.clear();
            self.face_neighbors.clear();
            return;
        }

        // 2. Prepare new data structures
        let mut new_vertices = Vec::new();
        let mut new_face_counts = Vec::new();
        let mut new_face_indices = Vec::new();
        let mut new_face_neighbors = Vec::new();
        let mut is_intersection = Vec::new();

        let mut old_to_new = vec![None; num_verts];

        // Keep existing vertices that are inside
        for i in 0..num_verts {
            if dists[i] <= 1e-9 {
                let new_idx = (new_vertices.len() / 3) as u32;
                new_vertices.push(self.vertices[i * 3]);
                new_vertices.push(self.vertices[i * 3 + 1]);
                new_vertices.push(self.vertices[i * 3 + 2]);
                old_to_new[i] = Some(new_idx);
                is_intersection.push(false);
            }
        }

        // Map edge (min, max) to new vertex index for intersections
        let mut intersection_map: Vec<((usize, usize), u32)> = Vec::new();
        let mut lid_segments = Vec::new();
        let mut index_offset = 0;

        // 3. Clip each face
        for (face_idx, &count) in self.face_counts.iter().enumerate() {
            let count = count as usize;
            let face_neighbor = self.face_neighbors[face_idx];
            let current_indices = &self.face_indices[index_offset..index_offset + count];
            index_offset += count;

            let mut next_face = Vec::new();

            for i in 0..count {
                let idx_s = current_indices[i] as usize;
                let idx_e = current_indices[(i + 1) % count] as usize;
                let d_s = dists[idx_s];
                let d_e = dists[idx_e];
                let s_in = d_s <= 1e-9;
                let e_in = d_e <= 1e-9;

                if s_in {
                    if e_in {
                        if let Some(idx) = old_to_new[idx_e] { next_face.push(idx); }
                    } else {
                        // Start In, End Out -> Intersection
                        let key = if idx_s < idx_e { (idx_s, idx_e) } else { (idx_e, idx_s) };
                        let idx = if let Some(&(_, id)) = intersection_map.iter().find(|&&(k, _)| k == key) {
                            id
                        } else {
                            let t = d_s / (d_s - d_e);
                            let ax = self.vertices[idx_s * 3]; let ay = self.vertices[idx_s * 3 + 1]; let az = self.vertices[idx_s * 3 + 2];
                            let bx = self.vertices[idx_e * 3]; let by = self.vertices[idx_e * 3 + 1]; let bz = self.vertices[idx_e * 3 + 2];
                            let new_idx = (new_vertices.len() / 3) as u32;
                            new_vertices.push(ax + t * (bx - ax));
                            new_vertices.push(ay + t * (by - ay));
                            new_vertices.push(az + t * (bz - az));
                            is_intersection.push(true);
                            intersection_map.push((key, new_idx));
                            new_idx
                        };
                        next_face.push(idx);
                    }
                } else if e_in {
                    // Start Out, End In -> Intersection then End
                    let key = if idx_s < idx_e { (idx_s, idx_e) } else { (idx_e, idx_s) };
                    let idx = if let Some(&(_, id)) = intersection_map.iter().find(|&&(k, _)| k == key) {
                        id
                    } else {
                        let t = d_s / (d_s - d_e);
                        let ax = self.vertices[idx_s * 3]; let ay = self.vertices[idx_s * 3 + 1]; let az = self.vertices[idx_s * 3 + 2];
                        let bx = self.vertices[idx_e * 3]; let by = self.vertices[idx_e * 3 + 1]; let bz = self.vertices[idx_e * 3 + 2];
                        let new_idx = (new_vertices.len() / 3) as u32;
                        new_vertices.push(ax + t * (bx - ax));
                        new_vertices.push(ay + t * (by - ay));
                        new_vertices.push(az + t * (bz - az));
                        is_intersection.push(true);
                        intersection_map.push((key, new_idx));
                        new_idx
                    };
                    next_face.push(idx);
                    if let Some(idx) = old_to_new[idx_e] { next_face.push(idx); }
                }
            }

            if next_face.len() >= 3 {
                new_face_counts.push(next_face.len() as u32);
                new_face_neighbors.push(face_neighbor);
                
                // Identify the segment on the clipping plane (connecting two intersection points)
                for i in 0..next_face.len() {
                    let u = next_face[i];
                    let v = next_face[(i + 1) % next_face.len()];
                    if is_intersection[u as usize] && is_intersection[v as usize] {
                        lid_segments.push((v, u)); // Reverse order for the lid face
                    }
                }
                new_face_indices.extend(next_face);
            }
        }

        // 4. Reconstruct the "lid" face from segments
        if !lid_segments.is_empty() {
            let mut lid_ordered = Vec::new();
            let (start, next) = lid_segments[0];
            lid_ordered.push(start);
            
            let mut current = next;
            while current != start && lid_ordered.len() <= lid_segments.len() {
                lid_ordered.push(current);
                if let Some(&(_, v)) = lid_segments.iter().find(|&&(u, _)| u == current) {
                    current = v;
                } else { break; } // Should not happen for convex poly
            }
            
            if lid_ordered.len() >= 3 {
                new_face_counts.push(lid_ordered.len() as u32);
                new_face_indices.extend(lid_ordered);
                new_face_neighbors.push(neighbor_id);
            }
        }

        self.vertices = new_vertices;
        self.face_counts = new_face_counts;
        self.face_indices = new_face_indices;
        self.face_neighbors = new_face_neighbors;
    }

    pub fn volume(&self) -> f64 {
        let mut volume: f64 = 0.0;
        let mut index_offset: usize = 0;

        for &count in &self.face_counts {
            let count: usize = count as usize;
            if count < 3 {
                index_offset += count;
                continue;
            }

            // Use the first vertex of the face as a pivot for fan triangulation
            let idx0: usize = self.face_indices[index_offset] as usize;
            let v0_x: f64 = self.vertices[idx0 * 3];
            let v0_y: f64 = self.vertices[idx0 * 3 + 1];
            let v0_z: f64 = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1: usize = self.face_indices[index_offset + i] as usize;
                let idx2: usize = self.face_indices[index_offset + i + 1] as usize;

                let v1_x: f64 = self.vertices[idx1 * 3];
                let v1_y: f64 = self.vertices[idx1 * 3 + 1];
                let v1_z: f64 = self.vertices[idx1 * 3 + 2];

                let v2_x: f64 = self.vertices[idx2 * 3];
                let v2_y: f64 = self.vertices[idx2 * 3 + 1];
                let v2_z: f64 = self.vertices[idx2 * 3 + 2];

                volume += v0_x * (v1_y * v2_z - v1_z * v2_y)
                    + v0_y * (v1_z * v2_x - v1_x * v2_z)
                    + v0_z * (v1_x * v2_y - v1_y * v2_x);
            }
            index_offset += count;
        }

        (volume / 6.0).abs()
    }

    pub fn centroid(&self) -> Vec<f64> {
        let mut centroid_x: f64 = 0.0;
        let mut centroid_y: f64 = 0.0;
        let mut centroid_z: f64 = 0.0;
        let mut total_volume: f64 = 0.0;
        let mut index_offset: usize = 0;

        for &count in &self.face_counts {
            let count: usize = count as usize;
            if count < 3 {
                index_offset += count;
                continue;
            }

            let idx0: usize = self.face_indices[index_offset] as usize;
            let v0_x: f64 = self.vertices[idx0 * 3];
            let v0_y: f64 = self.vertices[idx0 * 3 + 1];
            let v0_z: f64 = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1: usize = self.face_indices[index_offset + i] as usize;
                let idx2: usize = self.face_indices[index_offset + i + 1] as usize;

                let v1_x: f64 = self.vertices[idx1 * 3];
                let v1_y: f64 = self.vertices[idx1 * 3 + 1];
                let v1_z: f64 = self.vertices[idx1 * 3 + 2];

                let v2_x: f64 = self.vertices[idx2 * 3];
                let v2_y: f64 = self.vertices[idx2 * 3 + 1];
                let v2_z: f64 = self.vertices[idx2 * 3 + 2];

                let cross_x: f64 = v1_y * v2_z - v1_z * v2_y;
                let cross_y: f64 = v1_z * v2_x - v1_x * v2_z;
                let cross_z: f64 = v1_x * v2_y - v1_y * v2_x;

                let det: f64 = v0_x * cross_x + v0_y * cross_y + v0_z * cross_z;

                total_volume += det;

                let tet_cx: f64 = v0_x + v1_x + v2_x;
                let tet_cy: f64 = v0_y + v1_y + v2_y;
                let tet_cz: f64 = v0_z + v1_z + v2_z;

                centroid_x += det * tet_cx;
                centroid_y += det * tet_cy;
                centroid_z += det * tet_cz;
            }
            index_offset += count;
        }

        if total_volume.abs() < 1e-9 {
            return vec![0.0, 0.0, 0.0];
        }

        let factor: f64 = 1.0 / (4.0 * total_volume);
        vec![
            centroid_x * factor,
            centroid_y * factor,
            centroid_z * factor,
        ]
    }
}

impl Cell {
    pub fn faces(&self) -> Vec<Vec<usize>> {
        let mut faces: Vec<Vec<usize>> = Vec::with_capacity(self.face_counts.len());
        let mut offset: usize = 0;
        for &count in &self.face_counts {
            let count: usize = count as usize;
            let face: Vec<usize> = self.face_indices[offset..offset + count]
                .iter()
                .map(|&i| i as usize)
                .collect();
            faces.push(face);
            offset += count;
        }
        faces
    }
}
