import { describe, bench } from 'vitest';
import fs from 'fs/promises';
import path from 'path';

// Polyfill self for wasm-bindgen-rayon
if (typeof self === 'undefined') {
    // @ts-ignore
    global.self = global;
    // @ts-ignore
    self.addEventListener = () => {};
    // @ts-ignore
    self.removeEventListener = () => {};
}

const { default: init, Tessellation3D, BoundingBox3D, Wall3D, Tessellation2D, BoundingBox2D } = await import('voronoid');

// Initialize WASM module globally for the benchmarks
const wasmPath = path.resolve(process.cwd(), '../pkg/voronoid_bg.wasm');
const buffer = await fs.readFile(wasmPath);
await init({ module_or_path: buffer });

describe('Distributions', () => {
    const generator_cnt = [100, 1000, 10000];
    const box_size = 100;

    // Trefoil Knot parameters
    const cx = 50;
    const cy = 50;
    const cz = 50;
    const scale = 100.0 / 7.0;
    const tube_radius = scale * 0.3;

    for (const cnt of generator_cnt) {
        // Grid resolution heuristic: cube root of N
        const gridRes = Math.max(1, Math.ceil(Math.pow(cnt, 1/3)));

        // Uniform / Grid
        const tessUniformGrid = new Tessellation3D(new BoundingBox3D(0, 0, 0, box_size, box_size, box_size), gridRes, gridRes, gridRes);
        tessUniformGrid.random_generators(cnt);
        bench(`uniform/grid (${cnt})`, () => {
            tessUniformGrid.calculate();
        });

        // Trefoil / Grid
        const tessTrefoilGrid = new Tessellation3D(new BoundingBox3D(0, 0, 0, box_size, box_size, box_size), gridRes, gridRes, gridRes);
        tessTrefoilGrid.add_wall(Wall3D.new_trefoil(cx, cy, cz, scale, tube_radius, 100, -1000));
        tessTrefoilGrid.random_generators(cnt);
        bench(`trefoil/grid (${cnt})`, () => {
            tessTrefoilGrid.calculate();
        });

        // 2D Uniform / Grid
        const gridRes2D = Math.max(1, Math.ceil(Math.sqrt(cnt)));
        const tessUniformGrid2D = new Tessellation2D(new BoundingBox2D(0, 0, box_size, box_size), gridRes2D, gridRes2D);
        tessUniformGrid2D.random_generators(cnt);
        bench(`uniform/grid 2D (${cnt})`, () => {
            tessUniformGrid2D.calculate();
        });
    }
});