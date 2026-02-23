use crate::bounds::BoundingBox;
use crate::bounds::{BOX_ID_BOTTOM, BOX_ID_TOP, BOX_ID_FRONT, BOX_ID_BACK, BOX_ID_LEFT, BOX_ID_RIGHT};
use crate::tessellation::Cell;
use wasm_bindgen::prelude::*;

/// Scratch buffer to reuse allocations during clipping.
#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct CellFacesScratch {
    vertices: Vec<f64>,
    face_counts: Vec<u8>,
    face_indices: Vec<u16>,
    face_neighbors: Vec<i32>,
    dists: Vec<f64>,
    is_intersection: Vec<bool>,
    old_to_new: Vec<Option<u16>>,
    intersection_map: Vec<(u32, u16)>,
    lid_segments: Vec<(u16, u16)>,
    face_buffer: Vec<u16>,
    lid_buffer: Vec<u16>,
    lid_map: Vec<u16>,
}

/// A Voronoi cell containing vertices and face information.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CellFaces {
    pub(crate) id: usize,
    // Flat array of vertices [x, y, z, x, y, z, ...]
    pub(crate) vertices: Vec<f64>,
    // Number of vertices for each face
    pub(crate) face_counts: Vec<u8>,
    // Flattened indices for all faces
    pub(crate) face_indices: Vec<u16>,
    // Neighbor ID for each face. Negative values indicate walls/boundaries.
    pub(crate) face_neighbors: Vec<i32>,
}

#[wasm_bindgen]
impl CellFaces {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, bounds: BoundingBox) -> CellFaces {
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

        let face_counts: Vec<u8> = vec![4, 4, 4, 4, 4, 4];

        let face_indices: Vec<u16> = vec![
            3, 2, 1, 0, // Bottom (z-)
            4, 5, 6, 7, // Top (z+)
            0, 1, 5, 4, // Front (y-)
            2, 3, 7, 6, // Back (y+)
            0, 4, 7, 3, // Left (x-)
            1, 2, 6, 5, // Right (x+)
        ];

        CellFaces {
            id,
            vertices,
            face_counts,
            face_indices,
            face_neighbors: vec![
                BOX_ID_BOTTOM, // z-
                BOX_ID_TOP,    // z+
                BOX_ID_FRONT,  // y-
                BOX_ID_BACK,   // y+
                BOX_ID_LEFT,   // x-
                BOX_ID_RIGHT,  // x+
            ],
        }
    }

    /// The ID of the generator associated with this cell.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Flat array of vertices [x, y, z, x, y, z, ...].
    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f64> {
        self.vertices.clone()
    }

    /// Number of vertices for each face.
    #[wasm_bindgen(getter)]
    pub fn face_counts(&self) -> Vec<u32> {
        self.face_counts.iter().map(|&c| c as u32).collect()
    }

    /// Flattened indices for all faces.
    #[wasm_bindgen(getter)]
    pub fn face_indices(&self) -> Vec<u32> {
        self.face_indices.iter().map(|&i| i as u32).collect()
    }

    /// Neighbor ID for each face. Negative values indicate walls/boundaries.
    #[wasm_bindgen(getter)]
    pub fn face_neighbors(&self) -> Vec<i32> {
        self.face_neighbors.clone()
    }

    pub fn clip(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32) {
        let mut scratch = CellFacesScratch::default();
        self.clip_with_scratch(point, normal, neighbor_id, &mut scratch, None);
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
            let idx0 = self.face_indices[index_offset] as usize;
            let v0_x: f64 = self.vertices[idx0 * 3];
            let v0_y: f64 = self.vertices[idx0 * 3 + 1];
            let v0_z: f64 = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1 = self.face_indices[index_offset + i] as usize;
                let idx2 = self.face_indices[index_offset + i + 1] as usize;

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

            let idx0 = self.face_indices[index_offset] as usize;
            let v0_x: f64 = self.vertices[idx0 * 3];
            let v0_y: f64 = self.vertices[idx0 * 3 + 1];
            let v0_z: f64 = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1 = self.face_indices[index_offset + i] as usize;
                let idx2 = self.face_indices[index_offset + i + 1] as usize;

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

    pub fn face_area(&self, face_index: usize) -> f64 {
        if face_index >= self.face_counts.len() {
            return 0.0;
        }
        let mut offset = 0;
        for i in 0..face_index {
            offset += self.face_counts[i] as usize;
        }
        let count = self.face_counts[face_index] as usize;

        if count < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        let p0_idx = self.face_indices[offset] as usize;
        let p0_x = self.vertices[p0_idx * 3];
        let p0_y = self.vertices[p0_idx * 3 + 1];
        let p0_z = self.vertices[p0_idx * 3 + 2];

        for i in 1..count - 1 {
            let p1_idx = self.face_indices[offset + i] as usize;
            let p2_idx = self.face_indices[offset + i + 1] as usize;

            let p1_x = self.vertices[p1_idx * 3];
            let p1_y = self.vertices[p1_idx * 3 + 1];
            let p1_z = self.vertices[p1_idx * 3 + 2];

            let p2_x = self.vertices[p2_idx * 3];
            let p2_y = self.vertices[p2_idx * 3 + 1];
            let p2_z = self.vertices[p2_idx * 3 + 2];

            let v1_x = p1_x - p0_x;
            let v1_y = p1_y - p0_y;
            let v1_z = p1_z - p0_z;

            let v2_x = p2_x - p0_x;
            let v2_y = p2_y - p0_y;
            let v2_z = p2_z - p0_z;

            let cross_x = v1_y * v2_z - v1_z * v2_y;
            let cross_y = v1_z * v2_x - v1_x * v2_z;
            let cross_z = v1_x * v2_y - v1_y * v2_x;

            area += 0.5 * (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        }
        area
    }

    #[wasm_bindgen(js_name = faces)]
    // Workaround for the fact that wasm-bindgen does not support nested vectors directly
    pub fn wasm_faces(&self) -> js_sys::Array {
        let result = js_sys::Array::new_with_length(self.face_counts.len() as u32);
        let mut offset = 0;
        for (i, &count) in self.face_counts.iter().enumerate() {
            let count = count as usize;
            let end = offset + count;
            let face_slice = &self.face_indices[offset..end];
            let js_face = js_sys::Uint16Array::from(face_slice);
            result.set(i as u32, js_face.into());
            offset = end;
        }
        result
    }
}

impl CellFaces {
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

    pub fn max_radius_sq(&self, center: &[f64]) -> f64 {   
        let gx = center[0];
        let gy = center[1];
        let gz = center[2];
        let mut max_d2 = 0.0;
        for k in 0..self.vertices.len() / 3 {
            let dx = self.vertices[k * 3] - gx;
            let dy = self.vertices[k * 3 + 1] - gy;
            let dz = self.vertices[k * 3 + 2] - gz;
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 > max_d2 {
                max_d2 = d2;
            }
        }
        max_d2
    }

    pub fn clip_with_scratch(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32, scratch: &mut CellFacesScratch, generator: Option<&[f64]>) -> (bool, f64) {
        let px = point[0];
        let py = point[1];
        let pz = point[2];
        let nx = normal[0];
        let ny = normal[1];
        let nz = normal[2];

        let num_verts = self.vertices.len() / 3;
        scratch.dists.clear();
        scratch.dists.reserve(num_verts);
        let mut all_inside = true;
        let mut all_outside = true;

        // 1. Calculate distances
        for i in 0..num_verts {
            let vx = self.vertices[i * 3];
            let vy = self.vertices[i * 3 + 1];
            let vz = self.vertices[i * 3 + 2];
            let d = (vx - px) * nx + (vy - py) * ny + (vz - pz) * nz;
            scratch.dists.push(d);

            if d > 1e-9 {
                all_inside = false;
            } else if d < -1e-9 {
                all_outside = false;
            }
        }

        if all_inside {
            return (false, 0.0);
        }
        if all_outside {
            self.vertices.clear();
            self.face_counts.clear();
            self.face_indices.clear();
            self.face_neighbors.clear();
            return (true, 0.0);
        }

        // 2. Prepare new data structures
        scratch.vertices.clear();
        scratch.face_counts.clear();
        scratch.face_indices.clear();
        scratch.face_neighbors.clear();
        scratch.is_intersection.clear();
        
        scratch.old_to_new.clear();
        scratch.old_to_new.resize(num_verts, None);

        scratch.intersection_map.clear();
        scratch.lid_segments.clear();
        scratch.lid_map.clear();

        let mut max_d2 = 0.0;

        // Keep existing vertices that are inside
        for i in 0..num_verts {
            if scratch.dists[i] <= 1e-9 {
                let new_idx = (scratch.vertices.len() / 3) as u16;
                scratch.vertices.push(self.vertices[i * 3]);
                scratch.vertices.push(self.vertices[i * 3 + 1]);
                scratch.vertices.push(self.vertices[i * 3 + 2]);
                scratch.old_to_new[i] = Some(new_idx);
                scratch.is_intersection.push(false);

                if let Some(g) = generator {
                    let dx = self.vertices[i * 3] - g[0];
                    let dy = self.vertices[i * 3 + 1] - g[1];
                    let dz = self.vertices[i * 3 + 2] - g[2];
                    let d2 = dx * dx + dy * dy + dz * dz;
                    if d2 > max_d2 { max_d2 = d2; }
                }
            }
        }

        // Map edge (min, max) to new vertex index for intersections
        // scratch.intersection_map used here
        // scratch.lid_segments used here
        let mut index_offset = 0;

        // 3. Clip each face
        for (face_idx, &count_u8) in self.face_counts.iter().enumerate() {
            let count = count_u8 as usize;
            let face_neighbor = self.face_neighbors[face_idx];
            let current_indices = &self.face_indices[index_offset..index_offset + count];
            index_offset += count;

            scratch.face_buffer.clear();

            for i in 0..count {
                let idx_s = current_indices[i] as usize;
                let idx_e = current_indices[(i + 1) % count] as usize;
                let d_s = scratch.dists[idx_s];
                let d_e = scratch.dists[idx_e];
                let s_in = d_s <= 1e-9;
                let e_in = d_e <= 1e-9;

                if s_in {
                    if e_in {
                        if let Some(idx) = scratch.old_to_new[idx_e] { scratch.face_buffer.push(idx); }
                    } else {
                        // Start In, End Out -> Intersection
                        let key = if idx_s < idx_e { (idx_s as u32) << 16 | (idx_e as u32) } else { (idx_e as u32) << 16 | (idx_s as u32) };
                        let idx = if let Some(&(_, id)) = scratch.intersection_map.iter().find(|&&(k, _)| k == key) {
                            id
                        } else {
                            let t = (d_s / (d_s - d_e)).clamp(0.0, 1.0);
                            let ax = self.vertices[idx_s * 3]; let ay = self.vertices[idx_s * 3 + 1]; let az = self.vertices[idx_s * 3 + 2];
                            let bx = self.vertices[idx_e * 3]; let by = self.vertices[idx_e * 3 + 1]; let bz = self.vertices[idx_e * 3 + 2];
                            let new_idx = (scratch.vertices.len() / 3) as u16;
                            let nx = ax + t * (bx - ax);
                            let ny = ay + t * (by - ay);
                            let nz = az + t * (bz - az);
                            scratch.vertices.push(nx);
                            scratch.vertices.push(ny);
                            scratch.vertices.push(nz);

                            if let Some(g) = generator {
                                let dx = nx - g[0];
                                let dy = ny - g[1];
                                let dz = nz - g[2];
                                let d2 = dx * dx + dy * dy + dz * dz;
                                if d2 > max_d2 { max_d2 = d2; }
                            }

                            scratch.is_intersection.push(true);
                            scratch.intersection_map.push((key, new_idx));
                            new_idx
                        };
                        scratch.face_buffer.push(idx);
                    }
                } else if e_in {
                    // Start Out, End In -> Intersection then End
                    let key = if idx_s < idx_e { (idx_s as u32) << 16 | (idx_e as u32) } else { (idx_e as u32) << 16 | (idx_s as u32) };
                    let idx = if let Some(&(_, id)) = scratch.intersection_map.iter().find(|&&(k, _)| k == key) {
                        id
                    } else {
                        let t = (d_s / (d_s - d_e)).clamp(0.0, 1.0);
                        let ax = self.vertices[idx_s * 3]; let ay = self.vertices[idx_s * 3 + 1]; let az = self.vertices[idx_s * 3 + 2];
                        let bx = self.vertices[idx_e * 3]; let by = self.vertices[idx_e * 3 + 1]; let bz = self.vertices[idx_e * 3 + 2];
                        let new_idx = (scratch.vertices.len() / 3) as u16;
                        let nx = ax + t * (bx - ax);
                        let ny = ay + t * (by - ay);
                        let nz = az + t * (bz - az);
                        scratch.vertices.push(nx);
                        scratch.vertices.push(ny);
                        scratch.vertices.push(nz);

                        if let Some(g) = generator {
                            let dx = nx - g[0];
                            let dy = ny - g[1];
                            let dz = nz - g[2];
                            let d2 = dx * dx + dy * dy + dz * dz;
                            if d2 > max_d2 { max_d2 = d2; }
                        }

                        scratch.is_intersection.push(true);
                        scratch.intersection_map.push((key, new_idx));
                        new_idx
                    };
                    scratch.face_buffer.push(idx);
                    if let Some(idx) = scratch.old_to_new[idx_e] { scratch.face_buffer.push(idx); }
                }
            }

            if scratch.face_buffer.len() >= 3 {
                scratch.face_counts.push(scratch.face_buffer.len() as u8);
                scratch.face_neighbors.push(face_neighbor);
                
                // Identify the segment on the clipping plane (connecting two intersection points)
                for i in 0..scratch.face_buffer.len() {
                    let u = scratch.face_buffer[i];
                    let v = scratch.face_buffer[(i + 1) % scratch.face_buffer.len()];
                    if scratch.is_intersection[u as usize] && scratch.is_intersection[v as usize] {
                        scratch.lid_segments.push((v, u)); // Reverse order for the lid face
                    }
                }
                scratch.face_indices.extend_from_slice(&scratch.face_buffer);
            }
        }

        // 4. Reconstruct the "lid" face from segments
        if !scratch.lid_segments.is_empty() {
            scratch.lid_buffer.clear();
            
            // Build adjacency map for O(1) lookup
            scratch.lid_map.resize(scratch.vertices.len() / 3, u16::MAX);
            for &(u, v) in &scratch.lid_segments {
                scratch.lid_map[u as usize] = v;
            }

            let (start, next) = scratch.lid_segments[0];
            scratch.lid_buffer.push(start);
            
            let mut current = next;
            while current != start && scratch.lid_buffer.len() <= scratch.lid_segments.len() {
                scratch.lid_buffer.push(current);
                current = scratch.lid_map[current as usize];
                if current == u16::MAX { break; } // Should not happen for convex poly
            }
            
            if scratch.lid_buffer.len() >= 3 {
                scratch.face_counts.push(scratch.lid_buffer.len() as u8);
                scratch.face_indices.extend_from_slice(&scratch.lid_buffer);
                scratch.face_neighbors.push(neighbor_id);
            }
        }

        std::mem::swap(&mut self.vertices, &mut scratch.vertices);
        std::mem::swap(&mut self.face_counts, &mut scratch.face_counts);
        std::mem::swap(&mut self.face_indices, &mut scratch.face_indices);
        std::mem::swap(&mut self.face_neighbors, &mut scratch.face_neighbors);

        (true, max_d2)
    }
}

impl Cell for CellFaces {
    type Scratch = CellFacesScratch;

    #[inline]
    fn new(id: usize, bounds: BoundingBox) -> Self {
        CellFaces::new(id, bounds)
    }

    #[inline]
    fn clip(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32, scratch: &mut Self::Scratch, generator: Option<&[f64]>) -> (bool, f64) {
        self.clip_with_scratch(point, normal, neighbor_id, scratch, generator)
    }

    #[inline]
    fn max_radius_sq(&self, center: &[f64]) -> f64 {
        self.max_radius_sq(center)
    }

    fn centroid(&self) -> Vec<f64> {
        self.centroid()
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_faces_box() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let cell = CellFaces::new(0, bounds);
        
        assert!((cell.volume() - 1.0).abs() < 1e-6);
        let c = cell.centroid();
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cell_faces_clip() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mut cell = CellFaces::new(0, bounds);
        let mut scratch = CellFacesScratch::default();
        
        cell.clip_with_scratch(&[0.5, 0.5, 0.5], &[1.0, 0.0, 0.0], 10, &mut scratch, None);
        assert!((cell.volume() - 0.5).abs() < 1e-6);
    }
}