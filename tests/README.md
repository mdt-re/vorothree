# Integration Tests

This folder contains all integration tests for the `vorothree` library. Before commiting 

## Files

* `integration.rs`: Tests covering the public API and workflow of the library.
* `neighbors.rs`: Does test on the neighbor structure of the cells.
* `volumes.rs`: compares the sum of all cell volumes to the theoretical value for different walls.

## Usage

To run the tests:

```bash
cargo test --test <file>
```