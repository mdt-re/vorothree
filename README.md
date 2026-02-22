# vorothree

Rust library for 3D Voronoi tessellations, designed to be used in Rust as well as compiled to WebAssembly (WASM). It provides a flexible and feature-rich implementation to calculate the individual cells by a clipping procedure based on the generating points, the bounding box and possible walls. The tessellation struct takes a spatial algorithm to calculate the nearest neighbours efficiently and a cell struct which manages cell data and clipping. The combination of algorithm and cell can then be chosen based on the application.

## WASM and Web Usage

This library is designed to directly compile to WASM. To build the project for web usage:
```bash
wasm-pack build --target web
```
Consult the [www](www/) folder for examples and more details on how to use with TypeScript and in an web environment.


## Development

More information on the [tests](tests/), [benchmarks](benches/) and [examples](examples/) is in their respective directories. They can be run by:
```bash
cargo test
cargo bench
cargo example --example <example>
```
Contributing is highly appreciated via [Issues](https://github.com/mdt-re/vorothree/issues) and [Pull Requests](https://github.com/mdt-re/vorothree/pulls).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.