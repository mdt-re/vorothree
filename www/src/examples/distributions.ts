import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation3D, BoundingBox3D, Wall3D } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = ''; // Clear existing content

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

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

    const infoText = document.createElement('div');
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        distribution: 'uniform',
        count: 1000,
        opacity: 0.3,
        boxSize: 100,
        sphereRadius: 33,
        centralFraction: 0.1,
        trefoilTube: 4.0,
        calculate: () => initTessellation(),
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x242424);

    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(80, 80, 80);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    app.appendChild(renderer.domElement);

    window.addEventListener('resize', () => {
        camera.aspect = window.innerWidth / window.innerHeight;
        camera.updateProjectionMatrix();
        renderer.setSize(window.innerWidth, window.innerHeight);
    });

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(50, 100, 50);
    scene.add(dirLight);

    // Helper to visualize bounds
    const boxGeo = new THREE.BoxGeometry(params.boxSize, params.boxSize, params.boxSize);
    const boxEdges = new THREE.EdgesGeometry(boxGeo);
    const boxLines = new THREE.LineSegments(boxEdges, new THREE.LineBasicMaterial({ color: 0x888888 }));
    scene.add(boxLines);

    // --- Vorothree Setup ---
    let tess: Tessellation3D;

    function generateAxesPoints(count: number, size: number): Float64Array {
        const points = new Float64Array(count * 3);
        const transform = (val: number) => 0.5 + 4.0 * Math.pow(val - 0.5, 3);
        
        for (let i = 0; i < count; i++) {
            const x = transform(Math.random());
            const y = transform(Math.random());
            const z = transform(Math.random());
            
            points[i * 3] = (x - 0.5) * size;
            points[i * 3 + 1] = (y - 0.5) * size;
            points[i * 3 + 2] = (z - 0.5) * size;
        }
        return points;
    }

    function generateCentralBoxPoints(count: number, size: number, fraction: number): Float64Array {
        const points = new Float64Array(count * 3);
        // Scale factor for volume fraction: s = cbrt(fraction)
        const s = Math.pow(fraction, 1.0/3.0);
        
        for (let i = 0; i < count; i++) {
            points[i * 3] = (Math.random() - 0.5) * size * s;
            points[i * 3 + 1] = (Math.random() - 0.5) * size * s;
            points[i * 3 + 2] = (Math.random() - 0.5) * size * s;
        }
        return points;
    }

    function generateSphereSurfacePoints(count: number, radius: number): Float64Array {
        const points = new Float64Array(count * 3);

        // Center point
        points[0] = 0; points[1] = 0; points[2] = 0;
        
        for (let i = 1; i < count; i++) {
            let x, y, z, lenSq;
            do {
                x = Math.random() * 2.0 - 1.0;
                y = Math.random() * 2.0 - 1.0;
                z = Math.random() * 2.0 - 1.0;
                lenSq = x*x + y*y + z*z;
            } while (lenSq <= 0.0001 || lenSq > 1.0);
            
            const len = Math.sqrt(lenSq);
            points[i * 3] = (x / len) * radius;
            points[i * 3 + 1] = (y / len) * radius;
            points[i * 3 + 2] = (z / len) * radius;
        }
        return points;
    }

    function initTessellation() {
        const half = params.boxSize / 2;
        const bounds = new BoundingBox3D(-half, -half, -half, half, half, half);
        const gridRes = Math.max(1, Math.ceil(Math.pow(params.count, 1/3)));
        tess = new Tessellation3D(bounds, gridRes, gridRes, gridRes);
        
        if (params.distribution === 'trefoil') {
             const scale = params.boxSize / 7.0;
             const tubeRadius = params.trefoilTube;
             tess.add_wall(Wall3D.new_trefoil(0, 0, 0, scale, tubeRadius, 100, -1000));
             tess.random_generators(params.count);
        } else if (params.distribution === 'axes') {
            const points = generateAxesPoints(params.count, params.boxSize);
            tess.set_generators(points);
        } else if (params.distribution === 'central') {
            const points = generateCentralBoxPoints(params.count, params.boxSize, params.centralFraction);
            tess.set_generators(points);
        } else if (params.distribution === 'sphere') {
            const points = generateSphereSurfacePoints(params.count, params.sphereRadius);
            tess.set_generators(points);
        } else {
            // Uniform
            tess.random_generators(params.count);
        }

        const t0 = performance.now();
        tess.calculate();
        const dt = performance.now() - t0;
        
        updateVisualization(dt);
    }

    // --- Visualization ---
    const material = new THREE.MeshStandardMaterial({
        color: 0xffffff,
        roughness: 0.5,
        metalness: 0.1,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false
    });

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    function updateVisualization(computeTime: number) {
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0] as THREE.Mesh;
            child.geometry.dispose();
            geometryGroup.remove(child);
        }

        const cellCount = tess.count_cells;
        let totalVolume = 0;
        const positions: number[] = [];

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            totalVolume += cell.volume();

            const vertices = cell.vertices;
            const faces = cell.faces();

            for (const face of faces) {
                if (face.length < 3) continue;

                const v0Idx = face[0];
                const v0x = vertices[v0Idx * 3];
                const v0y = vertices[v0Idx * 3 + 1];
                const v0z = vertices[v0Idx * 3 + 2];

                for (let k = 1; k < face.length - 1; k++) {
                    const v1Idx = face[k];
                    const v2Idx = face[k + 1];

                    positions.push(v0x, v0y, v0z);
                    positions.push(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                    positions.push(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);
                }
            }
        }

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
        geometry.computeVertexNormals();

        const mesh = new THREE.Mesh(geometry, material);
        geometryGroup.add(mesh);

        infoText.innerText =
            `Distribution: ${params.distribution}\n` +
            `Cells:        ${cellCount}\n` +
            `Compute Time: ${computeTime.toFixed(2)} ms\n` +
            `Total Volume: ${totalVolume.toFixed(2)}`;
    }

    initTessellation();

    gui.add(params, 'distribution', ['uniform', 'trefoil', 'axes', 'central', 'sphere']).onChange(updateGuiVisibility);
    gui.add(params, 'count', 100, 10000, 100);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);

    const sphereCtrl = gui.add(params, 'sphereRadius', 5, params.boxSize).name('sphere radius');
    const centralCtrl = gui.add(params, 'centralFraction', 0.01, 0.5).name('volume fraction');
    const trefoilCtrl = gui.add(params, 'trefoilTube', 2, 10).name('tube radius');

    gui.add(params, 'calculate').name('Calculate');

    function updateGuiVisibility() {
        sphereCtrl.show(params.distribution === 'sphere');
        centralCtrl.show(params.distribution === 'central');
        trefoilCtrl.show(params.distribution === 'trefoil');
    }
    updateGuiVisibility();

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'distributions.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    // Animation Loop
    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        stats.update();
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}