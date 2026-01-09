import init, { Tessellation, BoundingBox } from 'vorothree';

async function run() {
    // 1. Initialize the WASM module
    await init();

    const app = document.querySelector<HTMLDivElement>('#app')!;
    app.innerHTML = `<h1>Vorothree Initialized</h1>`;

    // 2. Create BoundingBox and Tessellation
    const bounds = new BoundingBox(0, 0, 0, 100, 100, 100);
    const tess = new Tessellation(bounds);

    // 3. Set Generators
    const points = new Float64Array([
        10, 10, 10,
        50, 50, 50,
        90, 10, 10
    ]);
    
    tess.set_generators(points);

    app.innerHTML += `<p>Created tessellation with ${tess.count_generators} cells (generators).</p>`;
}

run().catch(console.error);