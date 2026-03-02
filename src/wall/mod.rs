pub mod geometries;

/// The maximum ID for walls. Wall IDs must be less than or equal to this value
/// to avoid conflicts with non-negative generator IDs and the bounding box IDs.
/// The number of D-1 dimensional faces of a hypercube is 2*D so with walls
/// starting at -1000 we allow for D < 500, which should be plenty.
pub const WALL_ID_MAX: i32 = -1000;

/// A clipping boundary for the Voronoi tessellation.
///
/// A `Wall` is a container for a `WallGeometry` implementation, giving it a unique
/// integer ID. This ID will be reported in the `face_neighbors` array of a `Cell`
/// for faces that have been clipped by this wall.
pub struct Wall<const D: usize> {
    id: i32,
    // FIXME: Storing the geometry as a Box<dyn ...> forces dynamic dispatch on every call,
    // possibly contributing to significant performance regression.
    inner: Box<dyn WallGeometry<D>>,
}

impl<const D: usize> Wall<D> {
    /// Creates a new `Wall` from a Rust struct that implements the `WallGeometry` trait.
    pub fn new(id: i32, geometry: Box<dyn WallGeometry<D>>) -> Self {
        if id > WALL_ID_MAX {
            panic!("Wall ID must be <= {}", WALL_ID_MAX);
        }
        Self {
            id,
            inner: geometry,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn contains(&self, point: &[f64; D]) -> bool {
        self.inner.contains(point)
    }

    pub fn cut(&self, generator: &[f64; D], callback: &mut dyn FnMut([f64; D], [f64; D])) {
        self.inner.cut(generator, callback)
    }
}

/// Trait defining the geometry and logic of a wall.
/// Must be Send + Sync to support parallel execution in Tessellation.
pub trait WallGeometry<const D: usize>: Send + Sync + std::fmt::Debug {
    /// Checks if a point is inside the valid region defined by the wall.
    fn contains(&self, point: &[f64; D]) -> bool;

    /// Calculates the clipping plane for a given generator.
    /// Returns a tuple (point_on_plane, plane_normal).
    /// The normal should point OUT of the valid region (towards the region to be clipped).
    // FIXME: The use of `&mut dyn FnMut` here forces dynamic dispatch, which has possibly been observed
    // to cause a 50-130% performance regression in benchmarks (CellEdges) compared to static dispatch.
    fn cut(&self, generator: &[f64; D], callback: &mut dyn FnMut([f64; D], [f64; D]));
}