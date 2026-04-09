import { describe, bench } from 'vitest';
import fs from 'fs/promises';
import path from 'path';
import os from 'os';

// Polyfill self for wasm-bindgen-rayon
if (typeof self === 'undefined') {
    // @ts-ignore
    global.self = global;
    // @ts-ignore
    self.addEventListener = () => {};
    // @ts-ignore
    self.removeEventListener = () => {};
}

const { default: init, Tessellation3D, BoundingBox3D, Tessellation2D, BoundingBox2D, initThreadPool } = await import('voronoid');

// Initialize WASM module globally for the benchmarks
const wasmPath = path.resolve(process.cwd(), '../pkg/voronoid_bg.wasm');
const buffer = await fs.readFile(wasmPath);
await init({ module_or_path: buffer });

// To benchmark scaling, run this file with different THREADS env vars, e.g.: THREADS=4 npx vitest bench ...
// Initialize thread pool if available
const threads = process.env.THREADS ? parseInt(process.env.THREADS) : os.cpus().length;
if (typeof initThreadPool === 'function') {
    try {
        await initThreadPool(threads);
        console.log(`Initialized thread pool with ${threads} threads`);
    } catch (e) {
        // Thread pool might already be initialized
    }
} else {
    console.warn('initThreadPool is not exported. Running in single-threaded mode. Ensure WASM is built with threading support.');
}

describe('Parallelism (100k points)', () => {
    const count = 100000;
    const boxSize = 1000;

    // 2D Setup
    const gridRes2D = Math.ceil(Math.sqrt(count));
    const tess2D = new Tessellation2D(new BoundingBox2D(0, 0, boxSize, boxSize), gridRes2D, gridRes2D);

    bench(`2D Tessellation (${threads} threads)`, () => {
        tess2D.random_generators(count);
        tess2D.calculate();
    });

    // 3D Setup
    const gridRes3D = Math.ceil(Math.pow(count, 1/3));
    const tess3D = new Tessellation3D(new BoundingBox3D(0, 0, 0, boxSize, boxSize, boxSize), gridRes3D, gridRes3D, gridRes3D);

    bench(`3D Tessellation (${threads} threads)`, () => {
        tess3D.random_generators(count);
        tess3D.calculate();
    });
});