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

mod bounds;
mod cell;
pub mod geometries;
mod kdtree;
mod moctree;
mod tessellation;
mod tessellation_moctree;
mod tessellation_kdtree;
mod wall;


pub use bounds::BoundingBox;
pub use cell::Cell;
pub use tessellation::Tessellation;
pub use tessellation_moctree::TessellationMoctree;
pub use tessellation_kdtree::TessellationKdTree;
pub use wall::Wall;
pub use wall::WallGeometry;
