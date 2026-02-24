import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation, BoundingBox } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = '';

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
    infoText.innerText = 'Ready';
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        count: 100,
        autoRelax: true,
        relax: () => {
            const t0 = performance.now();
            tess.relax();
            tess.calculate();
            const dt = performance.now() - t0;
            updateStats(dt);
            updateMesh();
        },
        reset: () => resetGenerators()
    };

    // --- Three.js ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x222222);

    const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 500);
    camera.position.set(120, 120, 120);

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
    controls.autoRotate = true;
    controls.autoRotateSpeed = 0.5;

    const light = new THREE.DirectionalLight(0xffffff, 1);
    light.position.set(50, 100, 50);
    scene.add(light);
    scene.add(new THREE.AmbientLight(0x404040));

    // --- Vorothree ---
    const size = 100;
    const bounds = new BoundingBox(-size/2, -size/2, -size/2, size/2, size/2, size/2);
    const tess = new Tessellation(bounds, 8, 8, 8);

    // Visuals
    const group = new THREE.Group();
    scene.add(group);

    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00ff88,
        metalness: 0.1,
        roughness: 0.2,
        transmission: 0.2,
        transparent: true,
        opacity: 0.8,
        side: THREE.DoubleSide
    });

    // Helper to visualize bounds
    const boxGeo = new THREE.BoxGeometry(size, size, size);
    const boxEdges = new THREE.EdgesGeometry(boxGeo);
    const boxLines = new THREE.LineSegments(boxEdges, new THREE.LineBasicMaterial({ color: 0x888888 }));
    scene.add(boxLines);

    function updateStats(dt: number) {
        const volumes: number[] = [];
        const faceCounts: number[] = [];
        const vertexCounts: number[] = [];
        const faceAreas: number[] = [];
        const vertsPerFace: number[] = [];

        const count = tess.count_cells;
        for (let i = 0; i < count; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            volumes.push(cell.volume());
            
            const fCounts = cell.face_counts;
            faceCounts.push(fCounts.length);

            const vCount = cell.vertices.length / 3;
            vertexCounts.push(vCount);

            for(let j=0; j<fCounts.length; j++) {
                faceAreas.push(cell.face_area(j));
                vertsPerFace.push(fCounts[j]);
            }
        }

        const getStats = (data: number[]) => {
            if (data.length === 0) return { avg: 0, std: 0 };
            const sum = data.reduce((a, b) => a + b, 0);
            const avg = sum / data.length;
            const sqDiff = data.reduce((a, b) => a + (b - avg) ** 2, 0);
            const std = Math.sqrt(sqDiff / data.length);
            return { avg, std };
        };

        const sVol = getStats(volumes);
        const sFaces = getStats(faceCounts);
        const sVerts = getStats(vertexCounts);
        const sAreas = getStats(faceAreas);
        const sVertsPerFace = getStats(vertsPerFace);

        infoText.innerText = 
            `Relaxation:   ${dt.toFixed(2)} ms\n` +
            `--------------------------------------\n` +
            `Metric        | Avg       | Std Dev   \n` +
            `--------------------------------------\n` +
            `Volume        | ${sVol.avg.toFixed(2).padStart(9)} | ${sVol.std.toFixed(2).padStart(9)}\n` +
            `Faces/Cell    | ${sFaces.avg.toFixed(2).padStart(9)} | ${sFaces.std.toFixed(2).padStart(9)}\n` +
            `Verts/Cell    | ${sVerts.avg.toFixed(2).padStart(9)} | ${sVerts.std.toFixed(2).padStart(9)}\n` +
            `Verts/Face    | ${sVertsPerFace.avg.toFixed(2).padStart(9)} | ${sVertsPerFace.std.toFixed(2).padStart(9)}\n` +
            `Face Area     | ${sAreas.avg.toFixed(2).padStart(9)} | ${sAreas.std.toFixed(2).padStart(9)}`;
    }

    function resetGenerators() {
        const points = new Float64Array(params.count * 3);
        for(let i=0; i<params.count * 3; i++) {
            points[i] = (Math.random() - 0.5) * size;
        }
        tess.set_generators(points);
        tess.calculate();
        updateMesh();
        updateStats(0);
    }

    function updateMesh() {
        // Cleanup
        while(group.children.length) {
            const m = group.children.pop() as THREE.Mesh;
            if (m.geometry) m.geometry.dispose();
        }

        const count = tess.count_cells;
        for(let i=0; i<count; i++) {
            const cell = tess.get_cell(i);
            if(!cell) continue;

            const verts = cell.vertices;
            const faces = cell.faces();
            const positions: number[] = [];

            for(const face of faces) {
                if(face.length < 3) continue;
                const v0 = face[0];
                for(let k=1; k<face.length-1; k++) {
                    const v1 = face[k];
                    const v2 = face[k+1];
                    positions.push(
                        verts[v0*3], verts[v0*3+1], verts[v0*3+2],
                        verts[v1*3], verts[v1*3+1], verts[v1*3+2],
                        verts[v2*3], verts[v2*3+1], verts[v2*3+2]
                    );
                }
            }

            const geo = new THREE.BufferGeometry();
            geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
            geo.computeVertexNormals();
            
            const mesh = new THREE.Mesh(geo, material);
            group.add(mesh);
        }
    }

    resetGenerators();

    gui.add(params, 'count', 10, 1000, 10).name('Point Count').onChange(resetGenerators);
    gui.add(params, 'autoRelax').name('Auto Relax');
    gui.add(params, 'relax').name('Step Relax');
    gui.add(params, 'reset').name('Reset');

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'relaxation.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    let frame = 0;
    function animate() {
        if(!app.isConnected) return;
        requestAnimationFrame(animate);

        stats.update();

        // Relax every 30 frames to visualize the steps
        if(params.autoRelax && frame % 30 === 0) {
            const t0 = performance.now();
            tess.relax();
            tess.calculate();
            const dt = performance.now() - t0;
            updateStats(dt);
            updateMesh();
        }

        controls.update();
        renderer.render(scene, camera);
        frame++;
    }
    animate();
}