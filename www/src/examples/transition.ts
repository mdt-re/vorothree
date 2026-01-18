import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { Tessellation, BoundingBox, Wall } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = '';

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    const params = {
        count: 1000,
        radius: 40,
        opacity: 0.3,
        animate: false,
        speed: 1.0
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
    controls.autoRotateSpeed = 0.5;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(50, 100, 50);
    scene.add(dirLight);

    // --- Vorothree Setup ---
    const boxSize = 120;
    const bounds = new BoundingBox(-boxSize/2, -boxSize/2, -boxSize/2, boxSize/2, boxSize/2, boxSize/2);
    const tess = new Tessellation(bounds, 15, 15, 15);

    // Store generators in JS to maintain them while wall changes
    let generators = new Float64Array(0);

    function initGenerators() {
        // Generate random points
        generators = new Float64Array(params.count * 3);
        for(let i = 0; i < params.count * 3; i++) {
            generators[i] = (Math.random() - 0.5) * boxSize;
        }
        tess.set_generators(generators);
    }

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

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    function updateVisualization() {
        // Clear previous meshes
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0] as THREE.Mesh;
            child.geometry.dispose();
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
            const mesh = new THREE.Mesh(geometry, material);
            geometryGroup.add(mesh);
        }
    }

    initGenerators();

    gui.add(params, 'count', 100, 5000, 100).onChange(initGenerators);
    gui.add(params, 'radius', 10, 60).name('Base Radius');
    gui.add(params, 'animate').name('Morph Shape');
    gui.add(params, 'speed', 0.1, 5.0).name('Anim Speed');
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);

    // --- Precompute Normals for Morphing ---
    const phi = (1 + Math.sqrt(5)) / 2;
    const one_over_phi = 1 / phi;

    // Dodecahedron Normals (12 faces) -> (0, ±1, ±phi) cyclic
    const dodecNormals: number[] = [];
    for(const y of [-1, 1]) for(const z of [-phi, phi]) dodecNormals.push(0, y, z);
    for(const x of [-phi, phi]) for(const z of [-1, 1]) dodecNormals.push(x, 0, z);
    for(const x of [-1, 1]) for(const y of [-phi, phi]) dodecNormals.push(x, y, 0);
    
    // Normalize Dodecahedron normals
    for(let i=0; i<dodecNormals.length; i+=3) {
        const l = Math.sqrt(dodecNormals[i]**2 + dodecNormals[i+1]**2 + dodecNormals[i+2]**2);
        dodecNormals[i]/=l; dodecNormals[i+1]/=l; dodecNormals[i+2]/=l;
    }

    // Icosahedron Normals (20 faces) -> (±1, ±1, ±1) and (0, ±phi, ±1/phi) cyclic
    const icoNormals: number[] = [];
    for(const x of [-1, 1]) for(const y of [-1, 1]) for(const z of [-1, 1]) icoNormals.push(x, y, z);
    for(const y of [-phi, phi]) for(const z of [-one_over_phi, one_over_phi]) icoNormals.push(0, y, z);
    for(const x of [-one_over_phi, one_over_phi]) for(const z of [-phi, phi]) icoNormals.push(x, 0, z);
    for(const x of [-phi, phi]) for(const y of [-one_over_phi, one_over_phi]) icoNormals.push(x, y, 0);

    // Normalize Icosahedron normals
    for(let i=0; i<icoNormals.length; i+=3) {
        const l = Math.sqrt(icoNormals[i]**2 + icoNormals[i+1]**2 + icoNormals[i+2]**2);
        icoNormals[i]/=l; icoNormals[i+1]/=l; icoNormals[i+2]/=l;
    }

    // Inradius factors
    const xi_dodec = Math.sqrt((5 + 2 * Math.sqrt(5)) / 15);
    const xi_ico = Math.sqrt((7 + 3 * Math.sqrt(5)) / 24);

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        const time = performance.now() * 0.001 * params.speed;
        
        // Morph parameter t: 0 = Dodecahedron, 1 = Icosahedron
        const t = params.animate ? (Math.sin(time) + 1) / 2 : 0;

        const R = params.radius;
        const large = R * Math.sqrt(2); // Distance for "inactive" planes

        // Calculate distances for both sets of planes
        // When t=0: D is active (dist ~ R), I is inactive (dist ~ large)
        // When t=1: D is inactive (dist ~ large), I is active (dist ~ R)
        const d_dodec_base = R * xi_dodec;
        const d_ico_base = R * xi_ico;

        const d_dodec = d_dodec_base + (large - d_dodec_base) * Math.pow(t, 2);
        const d_ico = d_ico_base + (large - d_ico_base) * Math.pow(1 - t, 2);

        // Build arrays for Rust
        const points = new Float64Array((12 + 20) * 3);
        const normals = new Float64Array((12 + 20) * 3);
        let ptr = 0;

        // Add Dodecahedron planes
        for(let i=0; i<dodecNormals.length; i+=3) {
            const nx = dodecNormals[i], ny = dodecNormals[i+1], nz = dodecNormals[i+2];
            normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
            points[ptr] = nx * d_dodec; points[ptr+1] = ny * d_dodec; points[ptr+2] = nz * d_dodec;
            ptr += 3;
        }
        // Add Icosahedron planes
        for(let i=0; i<icoNormals.length; i+=3) {
            const nx = icoNormals[i], ny = icoNormals[i+1], nz = icoNormals[i+2];
            normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
            points[ptr] = nx * d_ico; points[ptr+1] = ny * d_ico; points[ptr+2] = nz * d_ico;
            ptr += 3;
        }

        tess.clear_walls();
        // @ts-ignore
        tess.add_wall(Wall.new_convex_polyhedron(points, normals, -15));
        tess.calculate();

        updateVisualization();
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}