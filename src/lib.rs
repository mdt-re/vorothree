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

mod algorithm;
mod cell;
mod bounds;
mod tessellation;
mod wall;

pub use algorithm::SpatialAlgorithm;
pub use algorithm::d2_grid::AlgorithmGrid2D;
pub use algorithm::d3_grid::AlgorithmGrid;
pub use algorithm::d3_octree::AlgorithmOctree;

pub use bounds::BoundingBox;
pub use bounds::box_side;

pub use cell::Cell;
pub use cell::d2::Cell2D;
pub use cell::d3_edges::CellEdges;
pub use cell::d3_faces::CellFaces;

pub use wall::Wall;
pub use wall::WallGeometry;
pub use wall::WALL_ID_START;
pub use wall::geometries;

pub use tessellation::Tessellation;


// The WebAssembly implementation is sourced out in these files.
pub mod wasm;