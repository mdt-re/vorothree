# Benchmarks

This folder contains performance benchmarks for the `vorothree` library, utilizing [Criterion.rs](https://crates.io/crates/criterion).

## Files

* `cell.rs`: Benchmarks for individual cell operations, like `volume` or `centroid`.
* `distributions.rs`: Compares the algorithms for different distributions for the generators.
* `scaling.rs`: Scaling of the tessellation algorithm with the input size of the generators.
* `capacity.rs`: Behavior of the tessellation algorithm with the number of cells per bin/leaf.
* `parallelism.rs`: Scaling of the tessellation algorithm with the number of threads.

## Usage

To run all benchmarks or a specific benchmark suite, run one of the following commands:

```bash
cargo bench
cargo bench --bench <suite_name>
```

## Results

TODO: show the results of the benchmarks.