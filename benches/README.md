# Benchmarks

This folder contains performance benchmarks for the `vorothree` library, utilizing Criterion.rs.

## Files

* `tessellation.rs`: Benchmarks for tessellation updates and bulk calculations.
* `cell.rs`: Benchmarks for individual cell operations.

## Usage

To run all benchmarks:

```bash
cargo bench
```

To run a specific benchmark suite (e.g., `tessellation`):

```bash
cargo bench --bench tessellation
```