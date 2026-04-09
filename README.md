# voronoid

[![crates.io](https://img.shields.io/crates/v/voronoid)](https://crates.io/crates/voronoid)
[![tests](https://github.com/mdt-re/voronoid/actions/workflows/test.yml/badge.svg)](https://github.com/mdt-re/voronoid/actions/workflows/test.yml)
[![examples](https://github.com/mdt-re/voronoid/actions/workflows/deploy.yml/badge.svg?branch=main)](https://github.com/mdt-re/voronoid/actions/workflows/deploy.yml)
[![docs](https://github.com/mdt-re/voronoid/actions/workflows/docs.yml/badge.svg?branch=main)](https://github.com/mdt-re/voronoid/actions/workflows/docs.yml)


Rust library for D-dimensional Voronoi tessellations, designed to be used in Rust as well as compiled to WebAssembly (with a TypeScript interface). It provides a flexible and feature-rich implementation to calculate the individual cells by a clipping procedure based on the generating points, the bounding box and possible walls. The tessellation struct takes a spatial algorithm to calculate the nearest neighbours efficiently and a cell struct which manages cell data and the clipping algorithm. The combination of spatial algorithm and cell can then be matched to the specific application and distribution of generators. A few [interactive examples](https://mdt-re.github.io/voronoid/) are shown below.
<p align="center">
  <a href="https://mdt-re.github.io/voronoid/?example=moving_cell">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/moving_cell.png" width="196px" alt="moving cell" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=walls">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/walls.png" width="196px" alt="walls" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=benchmark">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/benchmark.png" width="196px" alt="benchmark" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=relaxation">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/relaxation.png" width="196px" alt="relaxation" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=transition">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/transition.png" width="196px" alt="transition" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=granular_flow">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/granular_flow.png" width="196px" alt="granular flow" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=pathfinding">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/pathfinding.png" width="196px" alt="pathfinding" />
  </a>
  <a href="https://mdt-re.github.io/voronoid/?example=distributions">
    <img src="https://raw.githubusercontent.com/mdt-re/voronoid/refs/heads/main/www/src/assets/distributions.png" width="196px" alt="distributions" />
  </a>
</p>

## WebAssembly and TypeScript API

This library is designed to directly compile to WASM, using wasm-pack, and is compatible with TypeScript. The package is published on [npm](https://www.npmjs.com/package/voronoid) and can be installed with:
```bash
npm install voronoid
```
Consult the [www](https://github.com/mdt-re/voronoid/tree/main/www) folder for [interactive examples](https://mdt-re.github.io/voronoid/) and more details on how to use with TypeScript and in a web environment. 

To build the project for web usage:
```bash
wasm-pack build --target web
```
which can then be added as a local dependency. To prepare the generated package for publication we need to copy the `README_WASM.md` to the package folder.
```bash
cp README_WASM.md pkg/README.md
```

## Usage & Documentation

The library, with [documentation](https://docs.rs/crate/voronoid/latest), can also be direclty used in Rust by installing it with:
```bash
cargo add voronoid
```
For a small usage example we generate a Voronoi tessellation for a 3D box with randomly positioned generators and calculate the total volume.
```rust
use voronoid::{BoundingBox, Tessellation, Algorithm3DGrid, Cell3DFaces, Wall, WALL_ID_MAX};

fn main() {
  let size = 10.0;
  let nr_bins = 10;
  // Creates a bounding box of length, widht , height = size.
  let bounds = BoundingBox::new([0.0, 0.0, 0.0], [size, size, size]);
  // Initializes a 3D tessellation with a grid algorithm and the bounding box.
  let mut tess = Tessellation::<3, Cell3DFaces, _>::new(bounds.clone(), Algorithm3DGrid::new(nr_bins, nr_bins, nr_bins, &bounds));
  // Add a spherical wall that spans the box.
  let r = size / 2.0;
  tess.add_wall(Wall::new(WALL_ID_MAX, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
  // Fill the tessellation with random generators (automatically confined to the walls).
  tess.random_generators(1000);
  // Calculate the tessellation.
  tess.calculate();
  // Calculate the total volume of all cells.
  let total_volume: f64 = (0..tess.count_cells())
    .map(|i| tess.get_cell(i).unwrap().volume())
    .sum();
  // Compare the theoretical value.
  let mut sphere_volume = 04.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3);
  println!("total cell volume: {}", total_volume);
  println!("theoretical sphere volume: {}", sphere_volume);
}
```

## Development

More information on the [tests](https://github.com/mdt-re/voronoid/tree/main/tests), [benchmarks](https://github.com/mdt-re/voronoid/tree/main/benches) and [examples](https://github.com/mdt-re/voronoid/tree/main/examples) is in their respective directories. They can be run by:
```bash
cargo test
cargo bench
cargo example --example <example>
```
Contributing is highly appreciated via [issues](https://github.com/mdt-re/voronoid/issues) and [pull requests](https://github.com/mdt-re/voronoid/pulls).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.