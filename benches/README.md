# Benchmarks

This folder contains performance benchmarks for the `vorothree` library, utilizing [Criterion.rs](https://crates.io/crates/criterion).

## Usage

To run all benchmarks or a specific benchmark suite, run one of the following commands:
```bash
cargo bench
cargo bench --bench <benchmark>
```
with the following benchmarks available:
* `capacity`: Behavior of the tessellation algorithm with the number of cells per bin/leaf.
* `cell`: Benchmarks for individual cell operations, like `volume` or `centroid`.
* `distributions`: Compares the algorithms for different distributions for the generators.
* `parallelism`: Scaling of the tessellation algorithm with the number of threads.
* `scaling`: Scaling of the tessellation algorithm with the input size of the generators.

## Results

TODO: show the results of the benchmarks.