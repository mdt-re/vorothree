# vorothree

Rust library for 3D Voronoi tessellations, designed to be used in Rust as well as compiled to WebAssembly (TypeScript interface). It provides a flexible and feature-rich implementation to calculate the individual cells by a clipping procedure based on the generating points, the bounding box and possible walls. The tessellation struct takes a spatial algorithm to calculate the nearest neighbours efficiently and a cell struct which manages cell data and the clipping algorithm. The combination of spatial algorithm and cell can then be matched to the specific application and distribution of generators. A few [interactive examples](https://mdt-re.github.io/vorothree/) are shown below.
<p align="center">
  <a href="https://mdt-re.github.io/vorothree/?example=moving_cell">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/moving_cell.png" width="196px" alt="moving cell" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=walls">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/walls.png" width="196px" alt="walls" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=benchmark">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/benchmark.png" width="196px" alt="benchmark" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=relaxation">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/relaxation.png" width="196px" alt="relaxation" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=transition">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/transition.png" width="196px" alt="transition" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=granular_flow">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/granular_flow.png" width="196px" alt="granular flow" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=pathfinding">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/pathfinding.png" width="196px" alt="pathfinding" />
  </a>
  <a href="https://mdt-re.github.io/vorothree/?example=distributions">
    <img src="https://raw.githubusercontent.com/mdt-re/vorothree/refs/heads/main/www/src/assets/distributions.png" width="196px" alt="distributions" />
  </a>
</p>

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

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.