//! # vorothree
//!
//! `vorothree` is a Rust library for 3D Voronoi tessellations, designed to be used in Rust
//! as well as compiled to WebAssembly (WASM). It provides efficient management of 3D generator
//! points and spatial partitioning to facilitate fast cellular computations.
//!
//! ## Features
//!
//! - **WASM-first**: Built with `wasm-bindgen` for seamless integration with JavaScript and TypeScript.
//! - **Spatial Partitioning**: Implements a configurable grid-based binning strategy for spatial lookups.
//! - **Dynamic Updates**: Supports updating individual generators or bulk setting of points with automatic grid re-binning.
//! - **Custom Walls**: Support for clipping cells against various geometries (Plane, Sphere, Cylinder, Torus, Custom).
//!
//! ## Example
//!
//! See the `examples/` directory for usage with SVG plotting and GLTF export.
//!
//! ## Main Interface
//!
//! The primary entry point is the [`Tessellation`] struct, which manages the grid and generators.

mod bounds;
mod cell_faces;
mod cell_edges;
pub mod geometries;
mod algo_grid;
mod algo_octree;
mod tessellation_grid;
mod tessellation_edges;
mod tessellation_moctree;
mod tessellation;
mod wall;


pub use algo_grid::AlgorithmGrid;
pub use algo_octree::AlgorithmOctree;
pub use bounds::BoundingBox;
pub use bounds::BOX_ID_BOTTOM;
pub use bounds::BOX_ID_TOP;
pub use bounds::BOX_ID_FRONT;
pub use bounds::BOX_ID_BACK;
pub use bounds::BOX_ID_LEFT;
pub use bounds::BOX_ID_RIGHT;
pub use cell_faces::CellFaces;
pub use cell_edges::CellEdges;
pub use tessellation_grid::TessellationGrid;
pub use tessellation_edges::TessellationEdges;
pub use tessellation_moctree::TessellationMoctree;
pub use tessellation::Tessellation;
pub use tessellation::Cell;
pub use tessellation::SpatialAlgorithm;
pub use wall::Wall;
pub use wall::WallGeometry;
pub use wall::WALL_ID_START;
