import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { Tessellation, BoundingBox } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = ''; // Clear existing content

    // --- UI for Stats ---
    const statsDiv = document.createElement('div');
    statsDiv.style.position = 'absolute';
    statsDiv.style.top = '10px';
    statsDiv.style.left = '10px';
    statsDiv.style.color = 'white';
    statsDiv.style.backgroundColor = 'rgba(0, 0, 0, 0.5)';
    statsDiv.style.padding = '10px';
    statsDiv.style.fontFamily = 'monospace';
    statsDiv.style.pointerEvents = 'none';
    statsDiv.style.userSelect = 'none';
    app.appendChild(statsDiv);

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    const params = {
        count: 200,
        opacity: 0.3,
        wireframe: false,
        autoRotate: true
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
    controls.autoRotate = true;
    controls.autoRotateSpeed = 1.0;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(50, 100, 50);
    scene.add(dirLight);

    // --- Vorothree Setup ---
    const boxSize = 100;
    const bounds = new BoundingBox(-boxSize / 2, -boxSize / 2, -boxSize / 2, boxSize / 2, boxSize / 2, boxSize / 2);
    const tess = new Tessellation(bounds, 10, 10, 10);

    // Number of points
    let points: Float64Array;
    let velocities: Float64Array;

    function initPoints() {
        points = new Float64Array(params.count * 3);
        velocities = new Float64Array(params.count * 3);

        for (let i = 0; i < params.count; i++) {
            points[i * 3] = (Math.random() - 0.5) * boxSize;
            points[i * 3 + 1] = (Math.random() - 0.5) * boxSize;
            points[i * 3 + 2] = (Math.random() - 0.5) * boxSize;

            velocities[i * 3] = (Math.random() - 0.5) * 0.5;
            velocities[i * 3 + 1] = (Math.random() - 0.5) * 0.5;
            velocities[i * 3 + 2] = (Math.random() - 0.5) * 0.5;
        }
        tess.set_generators(points);
        tess.calculate();
    }
    initPoints();

    // --- Visualization ---
    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00aaff,
        metalness: 0.1,
        roughness: 0.5,
        transmission: 0.6,
        thickness: 1.0,
        transparent: true,
        opacity: params.opacity,
        wireframe: params.wireframe,
        side: THREE.DoubleSide
    });

    gui.add(params, 'count', 10, 1000, 10).name('Point Count').onChange(initPoints);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(params, 'wireframe').onChange((v: boolean) => material.wireframe = v);
    gui.add(params, 'autoRotate').onChange((v: boolean) => controls.autoRotate = v);

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    // Helper to visualize bounds
    const boxGeo = new THREE.BoxGeometry(boxSize, boxSize, boxSize);
    const boxMat = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true, transparent: true, opacity: 0.1 });
    const boxMesh = new THREE.Mesh(boxGeo, boxMat);
    boxMesh.position.set(0, 0, 0);
    scene.add(boxMesh);

    let lastTime = performance.now();
    let frameCount = 0;
    let calcTimeTotal = 0;
    let renderTimeTotal = 0;

    function updateVisualization() {
        // Dispose old geometries
        for (let i = geometryGroup.children.length - 1; i >= 0; i--) {
            const child = geometryGroup.children[i] as THREE.Mesh;
            if (child.geometry) child.geometry.dispose();
            geometryGroup.remove(child);
        }

        const cellCount = tess.count_cells;
        
        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get(i);
            if (!cell) continue;

            const vertices = cell.vertices;
            const faces = cell.faces();
            const positions: number[] = [];

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

            const geometry = new THREE.BufferGeometry();
            geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
            geometry.computeVertexNormals();
            geometry.scale(0.9, 0.9, 0.9);
            
            // Re-center for scaling effect
            geometry.computeBoundingBox();
            if (geometry.boundingBox) {
                const center = new THREE.Vector3();
                geometry.boundingBox.getCenter(center);
                const offset = center.clone().multiplyScalar(1 - 0.9);
                geometry.translate(offset.x, offset.y, offset.z);
            }

            const mesh = new THREE.Mesh(geometry, material);
            geometryGroup.add(mesh);
        }
    }

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        const now = performance.now();
        
        // Update positions
        for (let i = 0; i < params.count; i++) {
            points[i * 3] += velocities[i * 3];
            points[i * 3 + 1] += velocities[i * 3 + 1];
            points[i * 3 + 2] += velocities[i * 3 + 2];

            // Bounce
            const halfSize = boxSize / 2;
            if (points[i * 3] < -halfSize || points[i * 3] > halfSize) velocities[i * 3] *= -1;
            if (points[i * 3 + 1] < -halfSize || points[i * 3 + 1] > halfSize) velocities[i * 3 + 1] *= -1;
            if (points[i * 3 + 2] < -halfSize || points[i * 3 + 2] > halfSize) velocities[i * 3 + 2] *= -1;
        }

        // Calculate Voronoi
        const t0 = performance.now();
        tess.set_generators(points);
        tess.calculate();
        const t1 = performance.now();
        calcTimeTotal += (t1 - t0);

        // Render
        const t2 = performance.now();
        updateVisualization();
        const t3 = performance.now();
        renderTimeTotal += (t3 - t2);

        controls.update();
        renderer.render(scene, camera);

        frameCount++;
        if (now - lastTime >= 1000) {
            const avgCalcTime = calcTimeTotal / frameCount;
            const avgRenderTime = renderTimeTotal / frameCount;
            const fps = frameCount;
            
            statsDiv.innerHTML = `
                <strong>Vorothree Performance</strong><br>
                Points: ${params.count}<br>
                FPS: ${fps}<br>
                Calc Time: ${avgCalcTime.toFixed(2)} ms<br>
                Mesh Gen Time: ${avgRenderTime.toFixed(2)} ms
            `;
            
            frameCount = 0;
            calcTimeTotal = 0;
            renderTimeTotal = 0;
            lastTime = now;
        }
    }
    animate();
}