import { describe, bench } from 'vitest';
import fs from 'fs/promises';
import path from 'path';

// Polyfill self for wasm-bindgen-rayon
if (typeof self === 'undefined') {
    // @ts-ignore
    global.self = global;
    // @ts-ignore
    global.self.addEventListener = () => {};
    // @ts-ignore
    global.self.removeEventListener = () => {};
}

const { default: init, Tessellation, BoundingBox } = await import('vorothree');

// Initialize WASM module globally for the benchmarks
// We use top-level await here which Vitest supports
const wasmPath = path.resolve(process.cwd(), '../pkg/vorothree_bg.wasm');
const buffer = await fs.readFile(wasmPath);
await init(buffer);

describe('Tessellation Performance', () => {
    // Setup data outside the benchmark function to isolate the test subject
    const bounds = new BoundingBox(0, 0, 0, 1000, 1000, 1000);
    const numPoints = 10000;
    const points = new Float64Array(numPoints * 3);
    for (let i = 0; i < points.length; i++) {
        points[i] = Math.random() * 1000;
    }

    // We reuse the instance to match the Rust benchmark strategy
    // and measure the cost of the update/calculation specifically.
    const tess = new Tessellation(bounds, 10, 10, 10);

    bench('set_generators (10k points)', () => {
        tess.set_generators(points);
    });
});