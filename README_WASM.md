# vorothree

3D Voronoi tessellation in Rust: a fast, feature-rich grid-based approach ready for WASM and TypeScript.

## Features

- **WASM-first**: Built with `wasm-bindgen` for seamless integration with JavaScript and TypeScript.
- **Spatial Partitioning**: Implements a configurable grid-based binning strategy for spatial lookups.
- **Dynamic Updates**: Supports updating individual generators or bulk setting of points with automatic grid re-binning.
- **Custom Walls**: Support for clipping cells against various geometries (Plane, Sphere, Cylinder, Torus, Custom).

## Installation

```bash
npm install vorothree
```

## Usage

```typescript
import init, { initThreadPool, Tessellation, BoundingBox, Wall } from 'vorothree';

async function run() {
    await init();
    await initThreadPool(navigator.hardwareConcurrency);

    // 1. Define bounds
    const bounds = new BoundingBox(0, 0, 0, 100, 100, 100);
    
    // 2. Create tessellation with grid size (10x10x10)
    const tess = new Tessellation(bounds, 10, 10, 10);

    // 3. Add a wall (optional)
    tess.add_wall(Wall.new_sphere(50, 50, 50, 40, -1000));

    // 4. Set generators
    const points = new Float64Array([
        10, 10, 10,
        50, 50, 50,
        90, 90, 90
    ]);
    tess.set_generators(points);

    // 5. Calculate
    tess.calculate();

    // 6. Access results
    console.log(`Calculated ${tess.count_cells} cells`);
    const cell = tess.get_cell(0);
    console.log(cell);
}

run();
```

## License

MIT/Apache-2.0