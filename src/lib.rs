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
pub use algorithm::algo_2d_grid::Algorithm2DGrid;
pub use algorithm::algo_3d_grid::Algorithm3DGrid;
pub use algorithm::algo_3d_octree::Algorithm3DOctree;

pub use bounds::BoundingBox;
pub use bounds::box_side;

pub use cell::Cell;
pub use cell::cell_2d::Cell2D;
pub use cell::cell_3d_faces::Cell3DFaces;

pub use wall::Wall;
pub use wall::WallGeometry;
pub use wall::WALL_ID_MAX;
pub use wall::wall_2d;
pub use wall::wall_3d;

pub use tessellation::Tessellation;


// The WebAssembly implementation is sourced out in these files.
pub mod wasm;