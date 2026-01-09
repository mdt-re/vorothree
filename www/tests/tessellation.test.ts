import { describe, it, expect, beforeAll } from 'vitest';
import fs from 'fs/promises';
import path from 'path';

describe('Vorothree WASM', () => {
    let init: any;
    let Tessellation: any;
    let BoundingBox: any;
    let Cell: any;

    beforeAll(async () => {
        // Polyfill self for wasm-bindgen-rayon
        if (typeof self === 'undefined') {
            // @ts-ignore
            global.self = global;
        }
        const module = await import('vorothree');
        init = module.default;
        Tessellation = module.Tessellation;
        BoundingBox = module.BoundingBox;
        Cell = module.Cell;

        // In a test environment, we initialize the module.
        // We load the WASM file manually because 'fetch' is not available/working 
        // for local files in the Node.js environment used by Vitest.
        const wasmPath = path.resolve(process.cwd(), '../pkg/vorothree_bg.wasm');
        const buffer = await fs.readFile(wasmPath);
        await init(buffer);
    });

    it('should create a tessellation and count generators', () => {
        const bounds = new BoundingBox(0, 0, 0, 100, 100, 100);
        const tess = new Tessellation(bounds);

        const points = new Float64Array([10, 10, 10, 20, 20, 20]);
        tess.set_generators(points);

        // We expect 2 cells because we passed 6 coordinates (2 points * 3 coords)
        expect(tess.count_generators).toBe(2);
    });

    it('should performantly add points to the tessellation', () => {
        const bounds = new BoundingBox(0, 0, 0, 1000, 1000, 1000);
        const tess = new Tessellation(bounds);

        const numPoints = 10000;
        const points = new Float64Array(numPoints * 3);
        for (let i = 0; i < points.length; i++) {
            points[i] = Math.random() * 1000;
        }

        const start = performance.now();
        tess.set_generators(points);
        const end = performance.now();

        expect(tess.count_generators).toBe(numPoints);
        console.log(`Added ${numPoints} points in ${(end - start).toFixed(2)}ms`);
        expect(end - start).toBeLessThan(2000); // Should be well under 2s
    });

    it('should have total cell volume equal to bounding box volume', () => {
        const width = 100;
        const height = 100;
        const depth = 100;
        const bounds = new BoundingBox(0, 0, 0, width, height, depth);
        const tess = new Tessellation(bounds);

        // Add some points to generate cells
        const points = new Float64Array([
            10, 10, 10,
            90, 90, 90,
            50, 50, 50
        ]);
        tess.set_generators(points);

        let totalVolume = 0;
        for (let i = 0; i < tess.count_cells; i++) {
            const cell = tess.get(i);
            if (cell) {
                totalVolume += cell.volume;
            }
        }

        const boxVolume = width * height * depth;
        expect(totalVolume).toBeCloseTo(boxVolume, 1);
    });
});