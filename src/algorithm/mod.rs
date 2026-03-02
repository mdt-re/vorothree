use crate::bounds::BoundingBox;

pub mod d2_grid;
pub mod d3_grid;
pub mod d3_octree;


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