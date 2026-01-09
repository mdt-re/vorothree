# vorothree

`vorothree` is a Rust library for 3D Voronoi tessellations, designed to be compiled to WebAssembly (WASM). It provides efficient management of 3D generator points and spatial partitioning to facilitate cellular computations.

## Features

- **WASM-first**: Built with `wasm-bindgen` for seamless integration with JavaScript and TypeScript.
- **Spatial Partitioning**: Implements a configurable grid-based binning strategy for spatial lookups.
- **Dynamic Updates**: Supports updating individual generators or bulk setting of points with automatic grid re-binning.

## Prerequisites

- Rust toolchain
- `wasm-pack`

## Building

To build the project for web usage:

```bash
wasm-pack build --target web
```

## Examples

This directory contains examples demonstrating how to use the `vorothree` library. Run them locally using:

```bash
cargo run --example <example>
```

## Tests

The test directory contains integrations tests. Run them using:

```bash
cargo test
```

## Benches

This directory contains the benchmarks for this library, results are found in the README. Run them using:

```bash
cargo bench
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.