import './style.css';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import init, { Tessellation, BoundingBox, Wall } from 'vorothree';

async function run() {
    await init();

    const app = document.querySelector<HTMLDivElement>('#app')!;
    app.innerHTML = ''; // Clear existing content

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x242424);

    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(150, 150, 150);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    app.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(50, 100, 50);
    scene.add(dirLight);

    // --- Vorothree Setup ---
    const bounds = new BoundingBox(0, 0, 0, 100, 100, 100);
    const tess = new Tessellation(bounds, 10, 10, 10);

    // Add Sphere Wall
    // Center (50,50,50), Radius 40.0, ID -15
    tess.add_wall(Wall.new_sphere(50.0, 50.0, 50.0, 40.0, -15));

    // Generate Random Points
    const numPoints = 2000;
    const points = new Float64Array(numPoints * 3);
    for (let i = 0; i < points.length; i++) {
        points[i] = Math.random() * 100;
    }
    tess.set_generators(points);
    tess.calculate();

    // --- Visualization ---
    
    // 1. Visualize the Sphere "Ghost" (Wireframe)
    const sphereGeom = new THREE.SphereGeometry(40.0, 32, 32);
    sphereGeom.translate(50.0, 50.0, 50.0);

    const sphereMat = new THREE.MeshBasicMaterial({ 
        color: 0xff0000, 
        wireframe: true, 
        transparent: true, 
        opacity: 0.1 
    });
    scene.add(new THREE.Mesh(sphereGeom, sphereMat));

    // 2. Create Meshes for Cells
    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00aaff,
        metalness: 0.1,
        roughness: 0.5,
        transmission: 0.6, // Glass-like
        thickness: 1.0,
        transparent: true,
        opacity: 0.3,
        side: THREE.DoubleSide
    });

    const cellCount = tess.count_cells;
    const geometryGroup = new THREE.Group();

    for (let i = 0; i < cellCount; i++) {
        const cell = tess.get(i);
        if (!cell) continue;

        const vertices = cell.vertices;
        const faces = cell.faces();

        const positions: number[] = [];
        
        // Triangulate faces (Fan triangulation for convex polygons)
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

        // Scale down slightly to see gaps
        geometry.scale(0.95, 0.95, 0.95);
        // Re-center after scaling (approximate)
        geometry.computeBoundingBox();
        const center = new THREE.Vector3();
        geometry.boundingBox!.getCenter(center);
        const offset = center.clone().multiplyScalar(1 - 0.95);
        geometry.translate(offset.x, offset.y, offset.z);

        const mesh = new THREE.Mesh(geometry, material);
        geometryGroup.add(mesh);
    }

    scene.add(geometryGroup);

    // Animation Loop
    function animate() {
        requestAnimationFrame(animate);
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}

run().catch(console.error);