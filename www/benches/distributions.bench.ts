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

const { default: init, Tessellation, TessellationMoctree, BoundingBox, Wall } = await import('vorothree');

// Initialize WASM module globally for the benchmarks
const wasmPath = path.resolve(process.cwd(), '../pkg/vorothree_bg.wasm');
const buffer = await fs.readFile(wasmPath);
await init(buffer);

describe('Distributions', () => {
    const sizes = [100, 1000, 10000];

    // Trefoil Knot parameters
    const cx = 50;
    const cy = 50;
    const cz = 50;
    const scale = 100.0 / 7.0;
    const tube_radius = scale * 0.3;

    for (const size of sizes) {
        // Grid resolution heuristic: cube root of N
        const gridRes = Math.max(1, Math.ceil(Math.pow(size, 1/3)));

        // Generate random points for uniform distribution
        const uniformPoints = new Float64Array(size * 3);
        for (let i = 0; i < uniformPoints.length; i++) {
            uniformPoints[i] = Math.random() * 100;
        }

        // Uniform / Grid
        const tessUniformGrid = new Tessellation(new BoundingBox(0, 0, 0, 100, 100, 100), gridRes, gridRes, gridRes);
        bench(`uniform/grid (${size})`, () => {
            tessUniformGrid.set_generators(uniformPoints);
            tessUniformGrid.calculate();
        });

        // Uniform / Moctree
        const tessUniformMoctree = new TessellationMoctree(new BoundingBox(0, 0, 0, 100, 100, 100), 8);
        bench(`uniform/moctree (${size})`, () => {
            tessUniformMoctree.set_generators(uniformPoints);
            tessUniformMoctree.calculate();
        });

        // Trefoil / Grid
        const tessTrefoilGrid = new Tessellation(new BoundingBox(0, 0, 0, 100, 100, 100), gridRes, gridRes, gridRes);
        tessTrefoilGrid.add_wall(Wall.new_trefoil(cx, cy, cz, scale, tube_radius, 100, -10));
        bench(`trefoil/grid (${size})`, () => {
            tessTrefoilGrid.random_generators(size);
            tessTrefoilGrid.calculate();
        });

        // Trefoil / Moctree
        const tessTrefoilMoctree = new TessellationMoctree(new BoundingBox(0, 0, 0, 100, 100, 100), 8);
        tessTrefoilMoctree.add_wall(Wall.new_trefoil(cx, cy, cz, scale, tube_radius, 100, -10));
        bench(`trefoil/moctree (${size})`, () => {
            tessTrefoilMoctree.random_generators(size);
            tessTrefoilMoctree.calculate();
        });
    }
});