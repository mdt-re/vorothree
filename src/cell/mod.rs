use crate::bounds::BoundingBox;

pub mod cell_2d;
pub mod cell_3d_faces;

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

    /// Get the indices of neighboring cells. Negative values indicate bounding box or wall boundaries.
    fn neighbors(&self) -> &[i32];

    /// Returns the vertices shared by two neighboring boundaries.
    fn shared_vertices(&self, neighbor_a: i32, neighbor_b: i32) -> Vec<f64> {
        let _ = neighbor_a;
        let _ = neighbor_b;
        Vec::new()
    }
}