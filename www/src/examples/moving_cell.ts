import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import Stats from 'three/examples/jsm/libs/stats.module';
import { Tessellation, BoundingBox } from 'vorothree';

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

    const infoText = document.createElement('div');
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        count: 50,
        speed: 1.0,
        opacity: 0.3
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

    // --- Vorothree Setup ---
    const boxSize = 100;
    const bounds = new BoundingBox(-boxSize / 2, -boxSize / 2, -boxSize / 2, boxSize / 2, boxSize / 2, boxSize / 2);
    const tess = new Tessellation(bounds, 10, 10, 10);

    let points: Float64Array;

    function initPoints() {
        // +1 for the moving point
        points = new Float64Array((params.count + 1) * 3);
        
        // Static points
        for (let i = 0; i < params.count; i++) {
            points[i * 3] = (Math.random() - 0.5) * boxSize;
            points[i * 3 + 1] = (Math.random() - 0.5) * boxSize;
            points[i * 3 + 2] = (Math.random() - 0.5) * boxSize;
        }
        
        // Moving point (last one) initialized at center
        const idx = params.count * 3;
        points[idx] = 0;
        points[idx + 1] = 0;
        points[idx + 2] = 0;
    }
    initPoints();

    // --- Visualization ---
    const material = new THREE.MeshStandardMaterial({
        color: 0x00aaff,
        roughness: 0.5,
        metalness: 0.1,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false 
    });

    const movingMaterial = new THREE.MeshStandardMaterial({
        color: 0xff3333,
        roughness: 0.5,
        metalness: 0.1,
        transparent: true,
        opacity: 2 * params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false 
    });

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    // Helper to visualize bounds
    const boxGeo = new THREE.BoxGeometry(boxSize, boxSize, boxSize);
    const boxEdges = new THREE.EdgesGeometry(boxGeo);
    const boxLines = new THREE.LineSegments(boxEdges, new THREE.LineBasicMaterial({ color: 0x888888 }));
    scene.add(boxLines);

    // Helper for the moving point position
    const pointGeo = new THREE.SphereGeometry(1, 16, 16);
    const pointMat = new THREE.MeshBasicMaterial({ color: 0xff0000 });
    const pointMesh = new THREE.Mesh(pointGeo, pointMat);
    scene.add(pointMesh);

    function updateVisualization() {
        // Dispose old geometries
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0] as THREE.Mesh;
            child.geometry.dispose();
            geometryGroup.remove(child);
        }

        const cellCount = tess.count_cells;
        
        let movingVolume = 0;
        let otherVolume = 0;

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get(i);
            if (!cell) continue;

            // Identify if this is the moving cell.
            // The moving point is at index `params.count`.
            const isMoving = cell.id === params.count;
            const vol = cell.volume();

            if (isMoving) {
                movingVolume = vol;
            } else {
                otherVolume += vol;
            }

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
            
            const mesh = new THREE.Mesh(geometry, isMoving ? movingMaterial : material);
            geometryGroup.add(mesh);
        }

        const totalVolume = movingVolume + otherVolume;
        infoText.innerText = `moving volume: ${movingVolume.toFixed(2)}\nother volume:  ${otherVolume.toFixed(2)}\ntotal volume:  ${totalVolume.toFixed(2)}`;
    }

    gui.add(params, 'count', 10, 200, 10).name('static points').onChange(initPoints);
    gui.add(params, 'speed', 0, 3).name('speed');
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'moving_cell.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        stats.update();

        const time = performance.now() * 0.001 * params.speed;
        
        // Update moving point position (Lissajous-like curve)
        const idx = params.count * 3;
        const x = Math.sin(time) * (boxSize * 0.4);
        const y = Math.cos(time * 1.3) * (boxSize * 0.4);
        const z = Math.sin(time * 0.7) * (boxSize * 0.4);

        points[idx] = x;
        points[idx + 1] = y;
        points[idx + 2] = z;

        // Update point helper
        pointMesh.position.set(x, y, z);

        // Recalculate Voronoi
        tess.set_generators(points);
        tess.calculate();

        updateVisualization();

        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}