use crate::bounds::BoundingBox;
use crate::bounds::{BOX_ID_BOTTOM, BOX_ID_TOP, BOX_ID_FRONT, BOX_ID_BACK, BOX_ID_LEFT, BOX_ID_RIGHT};
use crate::tessellation::Cell;
use wasm_bindgen::prelude::*;
use std::collections::HashSet;

/// Scratch buffer to reuse allocations during clipping for CellEdges.
#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct CellEdgesScratch {
    vertices: Vec<f64>,
    edge_buffer: Vec<u16>,
    neighbor_buffer: Vec<i32>,
    vertex_offsets: Vec<u16>,
    vertex_counts: Vec<u8>,
    
    dists: Vec<f64>,
    old_to_new: Vec<Option<u16>>,
    
    // (FaceID, new_vertex_idx, is_left_of_edge)
    face_cut_map: Vec<(i32, u16, bool)>,
    // (p_idx, u_idx, F_left, F_right)
    cut_infos: Vec<(u16, u16, i32, i32)>,
}

/// A Voronoi cell represented as a graph of vertices and edges.
/// This structure is optimized for clipping operations by maintaining
/// connectivity information, similar to Voro++.
#[wasm_bindgen]
#[derive(Clone)]
pub struct CellEdges {
    pub(crate) id: usize,
    // Flat array of vertices [x, y, z, x, y, z, ...]
    pub(crate) vertices: Vec<f64>,
    // Flat adjacency list. 
    // Edges for vertex i are at edge_buffer[vertex_offsets[i] .. vertex_offsets[i] + vertex_counts[i]]
    pub(crate) edge_buffer: Vec<u16>,
    // Corresponding face IDs. neighbor_buffer[k] is the face ID to the "left" of the edge at edge_buffer[k]
    pub(crate) neighbor_buffer: Vec<i32>,
    pub(crate) vertex_offsets: Vec<u16>,
    pub(crate) vertex_counts: Vec<u8>,
}

#[wasm_bindgen]
impl CellEdges {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, bounds: BoundingBox) -> CellEdges {
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

        // Initial topology for a box.
        // Each vertex has 3 neighbors.
        // Neighbors are ordered such that faces are traversed correctly.
        // neighbor_buffer[k] is the face ID between edge k and edge k+1 (cyclic).
        
        let edge_buffer: Vec<u16> = vec![
            1, 4, 3, // Vertex 0
            2, 5, 0, // Vertex 1
            3, 6, 1, // Vertex 2
            0, 7, 2, // Vertex 3
            5, 7, 0, // Vertex 4
            1, 6, 4, // Vertex 5
            2, 7, 5, // Vertex 6
            3, 4, 6, // Vertex 7
        ];

        let neighbor_buffer: Vec<i32> = vec![
            BOX_ID_FRONT, BOX_ID_LEFT, BOX_ID_BOTTOM, // Vertex 0
            BOX_ID_RIGHT, BOX_ID_FRONT, BOX_ID_BOTTOM, // Vertex 1
            BOX_ID_BACK, BOX_ID_RIGHT, BOX_ID_BOTTOM, // Vertex 2
            BOX_ID_LEFT, BOX_ID_BACK, BOX_ID_BOTTOM, // Vertex 3
            BOX_ID_TOP, BOX_ID_LEFT, BOX_ID_FRONT, // Vertex 4
            BOX_ID_RIGHT, BOX_ID_TOP, BOX_ID_FRONT, // Vertex 5
            BOX_ID_BACK, BOX_ID_TOP, BOX_ID_RIGHT, // Vertex 6
            BOX_ID_LEFT, BOX_ID_TOP, BOX_ID_BACK, // Vertex 7
        ];

        let vertex_offsets: Vec<u16> = vec![0, 3, 6, 9, 12, 15, 18, 21];
        let vertex_counts: Vec<u8> = vec![3; 8];

        CellEdges {
            id,
            vertices,
            edge_buffer,
            neighbor_buffer,
            vertex_offsets,
            vertex_counts,
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

    // Reconstruct face information for compatibility with Cell API
    #[wasm_bindgen(getter)]
    pub fn face_counts(&self) -> Vec<u32> {
        let (counts, _, _) = self.calculate_faces();
        counts
    }

    #[wasm_bindgen(getter)]
    pub fn face_indices(&self) -> Vec<u32> {
        let (_, indices, _) = self.calculate_faces();
        indices
    }

    #[wasm_bindgen(getter)]
    pub fn face_neighbors(&self) -> Vec<i32> {
        let (_, _, neighbors) = self.calculate_faces();
        neighbors
    }

    pub fn clip(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32) {
        let mut scratch = CellEdgesScratch::default();
        self.clip_with_scratch(point, normal, neighbor_id, &mut scratch, None);
    }

    pub fn volume(&self) -> f64 {
        let mut volume: f64 = 0.0;
        let (counts, indices, _) = self.calculate_faces();
        let mut index_offset = 0;

        for count in counts {
            let count = count as usize;
            if count < 3 {
                index_offset += count;
                continue;
            }

            let idx0 = indices[index_offset] as usize;
            let v0_x = self.vertices[idx0 * 3];
            let v0_y = self.vertices[idx0 * 3 + 1];
            let v0_z = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1 = indices[index_offset + i] as usize;
                let idx2 = indices[index_offset + i + 1] as usize;

                let v1_x = self.vertices[idx1 * 3];
                let v1_y = self.vertices[idx1 * 3 + 1];
                let v1_z = self.vertices[idx1 * 3 + 2];

                let v2_x = self.vertices[idx2 * 3];
                let v2_y = self.vertices[idx2 * 3 + 1];
                let v2_z = self.vertices[idx2 * 3 + 2];

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
        let (counts, indices, _) = self.calculate_faces();
        let mut index_offset = 0;

        for count in counts {
            let count = count as usize;
            if count < 3 {
                index_offset += count;
                continue;
            }

            let idx0 = indices[index_offset] as usize;
            let v0_x = self.vertices[idx0 * 3];
            let v0_y = self.vertices[idx0 * 3 + 1];
            let v0_z = self.vertices[idx0 * 3 + 2];

            for i in 1..count - 1 {
                let idx1 = indices[index_offset + i] as usize;
                let idx2 = indices[index_offset + i + 1] as usize;

                let v1_x = self.vertices[idx1 * 3];
                let v1_y = self.vertices[idx1 * 3 + 1];
                let v1_z = self.vertices[idx1 * 3 + 2];

                let v2_x = self.vertices[idx2 * 3];
                let v2_y = self.vertices[idx2 * 3 + 1];
                let v2_z = self.vertices[idx2 * 3 + 2];

                let cross_x = v1_y * v2_z - v1_z * v2_y;
                let cross_y = v1_z * v2_x - v1_x * v2_z;
                let cross_z = v1_x * v2_y - v1_y * v2_x;

                let det = v0_x * cross_x + v0_y * cross_y + v0_z * cross_z;
                total_volume += det;

                let tet_cx = v0_x + v1_x + v2_x;
                let tet_cy = v0_y + v1_y + v2_y;
                let tet_cz = v0_z + v1_z + v2_z;

                centroid_x += det * tet_cx;
                centroid_y += det * tet_cy;
                centroid_z += det * tet_cz;
            }
            index_offset += count;
        }

        if total_volume.abs() < 1e-9 {
            return vec![0.0, 0.0, 0.0];
        }

        let factor = 1.0 / (4.0 * total_volume);
        vec![
            centroid_x * factor,
            centroid_y * factor,
            centroid_z * factor,
        ]
    }

    pub fn face_area(&self, face_index: usize) -> f64 {
        // This is inefficient with on-the-fly face calculation, but necessary for compatibility
        let (counts, indices, _) = self.calculate_faces();
        if face_index >= counts.len() {
            return 0.0;
        }
        let mut offset = 0;
        for i in 0..face_index {
            offset += counts[i] as usize;
        }
        let count = counts[face_index] as usize;
        if count < 3 { return 0.0; }

        let mut area = 0.0;
        let p0_idx = indices[offset] as usize;
        let p0_x = self.vertices[p0_idx * 3];
        let p0_y = self.vertices[p0_idx * 3 + 1];
        let p0_z = self.vertices[p0_idx * 3 + 2];

        for i in 1..count - 1 {
            let p1_idx = indices[offset + i] as usize;
            let p2_idx = indices[offset + i + 1] as usize;

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
    pub fn wasm_faces(&self) -> js_sys::Array {
        let (counts, indices, _) = self.calculate_faces();
        let result = js_sys::Array::new_with_length(counts.len() as u32);
        let mut offset = 0;
        for (i, &count) in counts.iter().enumerate() {
            let count = count as usize;
            let end = offset + count;
            let face_slice = &indices[offset..end];
            let js_face = js_sys::Uint32Array::from(face_slice);
            result.set(i as u32, js_face.into());
            offset = end;
        }
        result
    }
}

impl CellEdges {
    pub fn faces(&self) -> Vec<Vec<usize>> {
        let (counts, indices, _) = self.calculate_faces();
        let mut faces = Vec::with_capacity(counts.len());
        let mut offset = 0;
        for count in counts {
            let count = count as usize;
            faces.push(indices[offset..offset + count].iter().map(|&i| i as usize).collect());
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

    pub fn clip_with_scratch(&mut self, point: &[f64], normal: &[f64], neighbor_id: i32, scratch: &mut CellEdgesScratch, generator: Option<&[f64]>) -> (bool, f64) {
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

        if all_inside { return (false, 0.0); }
        if all_outside {
            self.vertices.clear();
            self.edge_buffer.clear();
            self.neighbor_buffer.clear();
            self.vertex_offsets.clear();
            self.vertex_counts.clear();
            return (true, 0.0);
        }

        // Prepare scratch
        scratch.vertices.clear();
        scratch.edge_buffer.clear();
        scratch.neighbor_buffer.clear();
        scratch.vertex_offsets.clear();
        scratch.vertex_counts.clear();
        scratch.old_to_new.clear();
        scratch.old_to_new.resize(num_verts, None);
        scratch.face_cut_map.clear();
        scratch.cut_infos.clear();

        let mut max_d2 = 0.0;

        // 1. Keep inside vertices and identify cuts
        for i in 0..num_verts {
            if scratch.dists[i] <= 1e-9 {
                let new_idx = (scratch.vertices.len() / 3) as u16;
                scratch.vertices.push(self.vertices[i * 3]);
                scratch.vertices.push(self.vertices[i * 3 + 1]);
                scratch.vertices.push(self.vertices[i * 3 + 2]);
                scratch.old_to_new[i] = Some(new_idx);
                
                if let Some(g) = generator {
                    let dx = self.vertices[i * 3] - g[0];
                    let dy = self.vertices[i * 3 + 1] - g[1];
                    let dz = self.vertices[i * 3 + 2] - g[2];
                    let d2 = dx * dx + dy * dy + dz * dz;
                    if d2 > max_d2 { max_d2 = d2; }
                }

                // Process edges
                let start = self.vertex_offsets[i] as usize;
                let count = self.vertex_counts[i] as usize;
                scratch.vertex_offsets.push(scratch.edge_buffer.len() as u16);
                scratch.vertex_counts.push(0); // Placeholder
                
                for k in 0..count {
                    let neighbor_idx = self.edge_buffer[start + k] as usize;
                    let face_left = self.neighbor_buffer[start + k];
                    
                    if scratch.dists[neighbor_idx] <= 1e-9 {
                        // Neighbor is inside, will be processed later, just add placeholder or handle mapping later?
                        // We can't add index yet if neighbor not processed. 
                        // But we know neighbor is kept. We can use old index and remap later? 
                        // Or better: we rebuild edges after all vertices are created.
                        // But we need to create cut vertices now.
                    } else {
                        // Neighbor is outside -> Cut
                        let d_s = scratch.dists[i];
                        let d_e = scratch.dists[neighbor_idx];
                        let t = (d_s / (d_s - d_e)).clamp(0.0, 1.0);
                        
                        let ax = self.vertices[i * 3]; let ay = self.vertices[i * 3 + 1]; let az = self.vertices[i * 3 + 2];
                        let bx = self.vertices[neighbor_idx * 3]; let by = self.vertices[neighbor_idx * 3 + 1]; let bz = self.vertices[neighbor_idx * 3 + 2];
                        
                        let nx = ax + t * (bx - ax);
                        let ny = ay + t * (by - ay);
                        let nz = az + t * (bz - az);
                        
                        let p_idx = (scratch.vertices.len() / 3) as u16;
                        scratch.vertices.push(nx);
                        scratch.vertices.push(ny);
                        scratch.vertices.push(nz);

                        scratch.vertex_offsets.push(0);
                        scratch.vertex_counts.push(0);
                        
                        if let Some(g) = generator {
                            let dx = nx - g[0];
                            let dy = ny - g[1];
                            let dz = nz - g[2];
                            let d2 = dx * dx + dy * dy + dz * dz;
                            if d2 > max_d2 { max_d2 = d2; }
                        }

                        // Face to the right of edge (i, neighbor) is face to the left of edge (neighbor, i)?
                        // No, we need face ID.
                        // In `self`, `neighbor_buffer[start + k]` is face LEFT of `i->neighbor`.
                        // `neighbor_buffer[start + (k + count - 1) % count]` is face RIGHT of `i->neighbor`.
                        let face_right = self.neighbor_buffer[start + (k + count - 1) % count];
                        
                        // Check if faces already have 2 cuts to prevent malformed topology
                        let count_left = scratch.face_cut_map.iter().filter(|(f, _, _)| *f == face_left).count();
                        let count_right = scratch.face_cut_map.iter().filter(|(f, _, _)| *f == face_right).count();

                        if count_left < 2 && count_right < 2 {
                            scratch.face_cut_map.push((face_left, p_idx, true)); // Left of edge -> p is start of cut on face
                            scratch.face_cut_map.push((face_right, p_idx, false)); // Right of edge -> p is end of cut on face
                            
                            scratch.cut_infos.push((p_idx, new_idx, face_left, face_right));
                        }
                    }
                }
            }
        }

        // 2. Build edges for kept vertices
        for i in 0..num_verts {
            if let Some(new_idx) = scratch.old_to_new[i] {
                scratch.vertex_offsets[new_idx as usize] = scratch.edge_buffer.len() as u16;
                let start = self.vertex_offsets[i] as usize;
                let count = self.vertex_counts[i] as usize;
                let mut new_count = 0;
                
                for k in 0..count {
                    let neighbor_idx = self.edge_buffer[start + k] as usize;
                    let face_left = self.neighbor_buffer[start + k];
                    
                    if let Some(new_neighbor_idx) = scratch.old_to_new[neighbor_idx] {
                        scratch.edge_buffer.push(new_neighbor_idx);
                        scratch.neighbor_buffer.push(face_left);
                        new_count += 1;
                    } else {
                        // It was a cut. Find the cut vertex.
                        // We stored it in cut_infos.
                        // Linear search is fine for small N.
                        // We need p such that p connects to new_idx and corresponds to this edge.
                        // In cut_infos: (p_idx, u_idx, F_left, F_right). u_idx == new_idx. F_left == face_left.
                        for &(p_idx, u, f_l, _) in &scratch.cut_infos {
                            if u == new_idx && f_l == face_left {
                                scratch.edge_buffer.push(p_idx);
                                scratch.neighbor_buffer.push(face_left);
                                new_count += 1;
                                break;
                            }
                        }
                    }
                }
                scratch.vertex_counts[new_idx as usize] = new_count as u8;
            }
        }

        // 3. Build edges for new cut vertices
        // Need to link them to form the lid.
        for &(p_idx, u_idx, f_left, f_right) in &scratch.cut_infos {
            scratch.vertex_offsets[p_idx as usize] = scratch.edge_buffer.len() as u16;
            
            let mut count = 0;
            // 1. Connection back to u
            // Face to the left of p->u is f_right.
            scratch.edge_buffer.push(u_idx);
            scratch.neighbor_buffer.push(f_right);
            count += 1;
            
            // 2. Connection to p_prev (on f_right)
            // Find other cut on f_right.
            let mut p_prev = u16::MAX;
            for &(f, idx, _) in &scratch.face_cut_map {
                if f == f_right && idx != p_idx {
                    // On f_right, p is "right" (end). We need "left" (start).
                    // Wait, logic:
                    // Lid edge on f_right connects p_prev to p.
                    // So p connects to p_prev? No, p connects to p_next.
                    // Order around p: u, p_prev, p_next.
                    // p->p_prev has Lid to the left? No.
                    // p->p_prev has neighbor_id to the left?
                    // Let's check faces again.
                    // p->u: Left is f_right.
                    // p->p_prev: Left is neighbor_id (Lid).
                    // p->p_next: Left is f_left.
                    p_prev = idx;
                    break;
                }
            }
            if p_prev != u16::MAX {
                scratch.edge_buffer.push(p_prev);
                scratch.neighbor_buffer.push(neighbor_id);
                count += 1;
            }

            // 3. Connection to p_next (on f_left)
            let mut p_next = u16::MAX;
            for &(f, idx, _) in &scratch.face_cut_map {
                if f == f_left && idx != p_idx {
                    p_next = idx;
                    break;
                }
            }
            if p_next != u16::MAX {
                scratch.edge_buffer.push(p_next);
                scratch.neighbor_buffer.push(f_left);
                count += 1;
            }
            
            scratch.vertex_counts[p_idx as usize] = count as u8;
        }

        std::mem::swap(&mut self.vertices, &mut scratch.vertices);
        std::mem::swap(&mut self.edge_buffer, &mut scratch.edge_buffer);
        std::mem::swap(&mut self.neighbor_buffer, &mut scratch.neighbor_buffer);
        std::mem::swap(&mut self.vertex_offsets, &mut scratch.vertex_offsets);
        std::mem::swap(&mut self.vertex_counts, &mut scratch.vertex_counts);

        (true, max_d2)
    }

    fn calculate_faces(&self) -> (Vec<u32>, Vec<u32>, Vec<i32>) {
        let mut counts = Vec::new();
        let mut indices = Vec::new();
        let mut neighbors = Vec::new();
        
        let mut visited = HashSet::new(); // (u, v)
        
        for u in 0..self.vertex_counts.len() {
            let start = self.vertex_offsets[u] as usize;
            let count = self.vertex_counts[u] as usize;
            for k in 0..count {
                let v = self.edge_buffer[start + k] as usize;
                if visited.contains(&(u, v)) { continue; }
                
                let face_id = self.neighbor_buffer[start + k];
                
                // Traverse face
                let mut face_verts = Vec::new();
                let mut curr = u;
                let mut next = v;
                
                loop {
                    face_verts.push(curr as u32);
                    visited.insert((curr, next));
                    
                    // Find edge from next that has face_id on left
                    let n_start = self.vertex_offsets[next] as usize;
                    let n_count = self.vertex_counts[next] as usize;
                    let mut found = false;
                    for m in 0..n_count {
                        if self.neighbor_buffer[n_start + m] == face_id {
                            curr = next;
                            next = self.edge_buffer[n_start + m] as usize;
                            found = true;
                            break;
                        }
                    }
                    if !found || curr == u { break; }
                }
                
                if !face_verts.is_empty() {
                    counts.push(face_verts.len() as u32);
                    indices.extend(face_verts);
                    neighbors.push(face_id);
                }
            }
        }
        (counts, indices, neighbors)
    }
}

impl Cell for CellEdges {
    type Scratch = CellEdgesScratch;

    #[inline]
    fn new(id: usize, bounds: BoundingBox) -> Self {
        CellEdges::new(id, bounds)
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
    fn test_cell_edges_box() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let cell = CellEdges::new(0, bounds);
        
        assert!((cell.volume() - 1.0).abs() < 1e-6);
        let c = cell.centroid();
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cell_edges_clip() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mut cell = CellEdges::new(0, bounds);
        let mut scratch = CellEdgesScratch::default();
        
        cell.clip_with_scratch(&[0.5, 0.5, 0.5], &[1.0, 0.0, 0.0], 10, &mut scratch, None);
        assert!((cell.volume() - 0.5).abs() < 1e-6);
    }
}