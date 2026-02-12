import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { Tessellation, TessellationMoctree, BoundingBox } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = ''; // Clear existing content

    // --- UI for Results ---
    const resultsDiv = document.createElement('div');
    resultsDiv.style.position = 'absolute';
    resultsDiv.style.bottom = '10px';
    resultsDiv.style.right = '10px';
    resultsDiv.style.textAlign = 'left';
    resultsDiv.style.color = 'white';
    resultsDiv.style.backgroundColor = 'rgba(0, 0, 0, 0.5)';
    resultsDiv.style.padding = '10px';
    resultsDiv.style.fontFamily = 'monospace';
    resultsDiv.style.whiteSpace = 'pre';
    resultsDiv.style.pointerEvents = 'none';
    resultsDiv.style.userSelect = 'none';
    resultsDiv.style.textTransform = 'lowercase';
    resultsDiv.style.display = 'none';
    app.appendChild(resultsDiv);

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x111111);

    const aspect = window.innerWidth / window.innerHeight;
    const frustumSize = 60;

    const persCamera = new THREE.PerspectiveCamera(60, aspect, 0.1, 500);
    persCamera.position.set(28, 21, 28);

    const orthoCamera = new THREE.OrthographicCamera(
        frustumSize * aspect / -2, frustumSize * aspect / 2,
        frustumSize / 2, frustumSize / -2,
        0.1, 1000
    );
    orthoCamera.position.set(28, 21, 28);

    let activeCamera: THREE.Camera = persCamera;

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    app.appendChild(renderer.domElement);

    const controls = new OrbitControls(activeCamera, renderer.domElement);
    controls.enableDamping = true;
    controls.autoRotate = true;
    controls.autoRotateSpeed = 1.0;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(10, 20, 10);
    scene.add(dirLight);

    // Visualization Group
    const visGroup = new THREE.Group();
    scene.add(visGroup);

    // --- Benchmark Logic ---
    const params = {
        cameraType: 'Perspective',
        count: 1000,
        boxSize: 20,
        n: 10,
        capacity: 8,
        render: true,
        run: () => runBenchmark(),
        download: () => downloadResults()
    };

    let lastResults: any = null;

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    gui.add(params, 'cameraType', ['Perspective', 'Orthographic']).name('Camera').onChange((val: string) => {
        const prevCamera = activeCamera;
        if (val === 'Perspective') {
            activeCamera = persCamera;
        } else {
            activeCamera = orthoCamera;
        }
        activeCamera.position.copy(prevCamera.position);
        activeCamera.rotation.copy(prevCamera.rotation);
        controls.object = activeCamera;
    });
    gui.add(params, 'count', 100, 50000, 100).name('Particle Count');
    gui.add(params, 'boxSize', 10, 100).name('Box Size');
    gui.add(params, 'n', 1, 50, 1).name('Grid Size (n)');
    gui.add(params, 'capacity', 4, 20, 1).name('Octree Cap');
    gui.add(params, 'render').name('Render Result');
    gui.add(params, 'run').name('Run Benchmark');
    gui.add(params, 'download').name('Download CSV');

    function runBenchmark() {
        if (resultsDiv) {
            resultsDiv.style.display = 'block';
            resultsDiv.innerText = 'running...';
        }

        // Use setTimeout to allow UI to update before heavy processing
        setTimeout(() => {
            try {
                // 1. Data Generation (JS Side)
                const t0 = performance.now();
                const points = new Float64Array(params.count * 3);

                for(let i=0; i<params.count; i++) {
                    points[i * 3] = (Math.random() - 0.5) * params.boxSize;
                    points[i * 3 + 1] = (Math.random() - 0.5) * params.boxSize;
                    points[i * 3 + 2] = (Math.random() - 0.5) * params.boxSize;
                }
                const tGen = performance.now() - t0;

                // 2. Grid Benchmark
                const t1 = performance.now();
                const half = params.boxSize / 2;
                const bounds = new BoundingBox(-half, -half, -half, half, half, half);
                const tess = new Tessellation(bounds, params.n, params.n, params.n);
                
                tess.set_generators(points);
                const tInsert = performance.now() - t1;

                // Computation
                const t2 = performance.now();
                tess.calculate();
                const tCompute = performance.now() - t2;

                // Relaxation
                const tRelaxStart = performance.now();
                tess.relax();
                tess.calculate();
                const tRelax = performance.now() - tRelaxStart;

                // Extraction (iterating cells)
                const t3 = performance.now();
                const cellCount = tess.count_cells;
                let totalVertices = 0;
                for(let i = 0; i < cellCount; i++) {
                    const cell = tess.get(i);
                    if (cell) {
                        totalVertices += cell.vertices.length;
                    }
                }
                const tExtract = performance.now() - t3;
                const totalGrid = tInsert + tCompute + tRelax + tExtract;

                // 3. Moctree Benchmark
                const t1m = performance.now();
                const boundsMoc = new BoundingBox(-half, -half, -half, half, half, half);
                const tessMoc = new TessellationMoctree(boundsMoc, params.capacity);
                
                tessMoc.set_generators(points);
                const tInsertMoc = performance.now() - t1m;

                const t2m = performance.now();
                tessMoc.calculate();
                const tComputeMoc = performance.now() - t2m;

                const tRelaxStartMoc = performance.now();
                tessMoc.relax();
                tessMoc.calculate();
                const tRelaxMoc = performance.now() - tRelaxStartMoc;

                const t3m = performance.now();
                const cellCountMoc = tessMoc.count_cells;
                for(let i = 0; i < cellCountMoc; i++) {
                    const cell = tessMoc.get(i);
                    if (cell) {
                        const _ = cell.vertices.length;
                    }
                }
                const tExtractMoc = performance.now() - t3m;
                const totalMoc = tInsertMoc + tComputeMoc + tRelaxMoc + tExtractMoc;

                // 4. Visualization (using Grid result)
                visGroup.clear();
                if (params.render) {
                    if (params.count > 50000) {
                        // Render points only for performance
                        const geo = new THREE.BufferGeometry();
                        geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(points), 3));
                        const mat = new THREE.PointsMaterial({ color: 0x00ff88, size: 0.2 });
                        visGroup.add(new THREE.Points(geo, mat));
                    } else {
                        // Render wireframe cells
                        const vertices: number[] = [];
                        for(let i = 0; i < cellCount; i++) {
                            const cell = tess.get(i);
                            if (!cell) continue;
                            
                            const cVerts = cell.vertices;
                            const faces = cell.faces();
                            
                            for (const face of faces) {
                                for (let j = 0; j < face.length; j++) {
                                    const idx1 = face[j];
                                    const idx2 = face[(j + 1) % face.length];
                                    
                                    vertices.push(cVerts[idx1 * 3], cVerts[idx1 * 3 + 1], cVerts[idx1 * 3 + 2]);
                                    vertices.push(cVerts[idx2 * 3], cVerts[idx2 * 3 + 1], cVerts[idx2 * 3 + 2]);
                                }
                            }
                        }
                        
                        const geo = new THREE.BufferGeometry();
                        geo.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
                        const mat = new THREE.LineBasicMaterial({ color: 0x00ff88, transparent: true, opacity: 0.3 });
                        visGroup.add(new THREE.LineSegments(geo, mat));
                    }
                }

                // Report
                lastResults = {
                    count: params.count,
                    boxSize: params.boxSize,
                    n: params.n,
                    capacity: params.capacity,
                    gen: tGen,
                    grid: { insert: tInsert, compute: tCompute, relax: tRelax, extract: tExtract, total: totalGrid },
                    moctree: { insert: tInsertMoc, compute: tComputeMoc, relax: tRelaxMoc, extract: tExtractMoc, total: totalMoc }
                };

                if (resultsDiv) {
                    resultsDiv.innerText = 
                        `generators:   ${params.count}\n` +
                        `box size:     ${params.boxSize}\n` +
                        `grid N:       ${params.n}\n` +
                        `octree cap:   ${params.capacity}\n` +
                        `generation:   ${tGen.toFixed(2)} ms\n` +
                        `--------------------------------------\n` +
                        `metric        | grid (ms) | octree (ms)\n` +
                        `--------------------------------------\n` +
                        `insert        | ${tInsert.toFixed(2).padStart(9)} | ${tInsertMoc.toFixed(2).padStart(11)}\n` +
                        `compute       | ${tCompute.toFixed(2).padStart(9)} | ${tComputeMoc.toFixed(2).padStart(11)}\n` +
                        `relax         | ${tRelax.toFixed(2).padStart(9)} | ${tRelaxMoc.toFixed(2).padStart(11)}\n` +
                        `extract       | ${tExtract.toFixed(2).padStart(9)} | ${tExtractMoc.toFixed(2).padStart(11)}\n` +
                        `--------------------------------------\n` +
                        `total         | ${totalGrid.toFixed(2).padStart(9)} | ${totalMoc.toFixed(2).padStart(11)}\n` +
                        `fps (equiv)   | ${(1000/totalGrid).toFixed(1).padStart(9)} | ${(1000/totalMoc).toFixed(1).padStart(11)}`;
                }

            } catch (e: any) {
                console.error(e);
                if (resultsDiv) resultsDiv.innerText = "Error: " + e.message;
            }
        }, 10);
    }

    function downloadResults() {
        if (!lastResults) {
            alert("No results to download. Run the benchmark first.");
            return;
        }

        const headers = "Particles,Box Size,Grid N,Octree Cap,JS Gen (ms),Grid Insert,Grid Compute,Grid Relax,Grid Extract,Grid Total,Moc Insert,Moc Compute,Moc Relax,Moc Extract,Moc Total\n";
        const row = `${lastResults.count},${lastResults.boxSize},${lastResults.n},${lastResults.capacity},${lastResults.gen.toFixed(2)},` +
                    `${lastResults.grid.insert.toFixed(2)},${lastResults.grid.compute.toFixed(2)},${lastResults.grid.relax.toFixed(2)},${lastResults.grid.extract.toFixed(2)},${lastResults.grid.total.toFixed(2)},` +
                    `${lastResults.moctree.insert.toFixed(2)},${lastResults.moctree.compute.toFixed(2)},${lastResults.moctree.relax.toFixed(2)},${lastResults.moctree.extract.toFixed(2)},${lastResults.moctree.total.toFixed(2)}`;

        const blob = new Blob([headers + row], { type: 'text/csv;charset=utf-8;' });
        const link = document.createElement("a");
        link.href = URL.createObjectURL(blob);
        link.download = "voro_benchmark_results.csv";
        link.click();
    }

    // Animation Loop
    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        controls.update();
        renderer.render(scene, activeCamera);
    }
    animate();

    // Resize
    window.addEventListener('resize', () => {
        const aspect = window.innerWidth / window.innerHeight;
        
        persCamera.aspect = aspect;
        persCamera.updateProjectionMatrix();

        orthoCamera.left = -frustumSize * aspect / 2;
        orthoCamera.right = frustumSize * aspect / 2;
        orthoCamera.top = frustumSize / 2;
        orthoCamera.bottom = -frustumSize / 2;
        orthoCamera.updateProjectionMatrix();

        renderer.setSize(window.innerWidth, window.innerHeight);
    });

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, activeCamera);
            const link = document.createElement('a');
            link.download = 'benchmark.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    // Auto-run once
    runBenchmark();
}