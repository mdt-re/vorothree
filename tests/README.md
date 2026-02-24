# Integration Tests

This folder contains all integration tests for the `vorothree` library.

## Usage

To run the tests:
```bash
cargo test --test <file>
```
with the following tests available:
* `comparisons`: Compares the output with the results from existing libraries.
* `integration`: Tests covering the public API and workflow of the library.
* `mappings`: Tests the map functionality for on-the-fly calculations.
* `neighbors`: Does test on the neighbor structure of the cells.
* `volumes`: compares the sum of all cell volumes to the theoretical value for different walls.
