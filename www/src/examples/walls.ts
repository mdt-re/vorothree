import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { Tessellation, BoundingBox, Wall } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = ''; // Clear existing content

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    const params = {
        wallType: 'sphere',
        radius: 40.0,
        height: 60.0,
        tube: 10.0,
        scale: 12.0,
        count: 2000,
        opacity: 0.3,

            };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x242424);

    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(150, 150, 150);

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
    let tess: Tessellation;

    function initTessellation() {
        const bounds = new BoundingBox(-50, -50, -50, 50, 50, 50);
        tess = new Tessellation(bounds, 10, 10, 10);
        
        switch (params.wallType) {
            case 'sphere':
                tess.add_wall(Wall.new_sphere(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'cylinder':
                // @ts-ignore
                tess.add_wall(Wall.new_cylinder(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, params.radius, -15));
                break;
            case 'torus':
                // @ts-ignore
                tess.add_wall(Wall.new_torus(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, params.radius, params.tube, -15));
                break;
            case 'trefoil':
                tess.add_wall(Wall.new_trefoil(0.0, 0.0, 0.0, params.scale, params.tube, 200, -15));
                break;
            case 'dodecahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_dodecahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'icosahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_icosahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
        }

        tess.random_generators(params.count);
        tess.calculate();
        updateVisualization();
    }

    // --- Visualization ---
    
    // 2. Create Meshes for Cells
    const material = new THREE.MeshStandardMaterial({
        color: 0xffffff,
        roughness: 0.5,
        metalness: 0.1,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false // Helps with transparency
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

            const mesh = new THREE.Mesh(geometry, material);
            geometryGroup.add(mesh);
        }
    }

    initTessellation();

    gui.add(params, 'wallType', ['sphere', 'cylinder', 'torus', 'trefoil', 'dodecahedron', 'icosahedron']).name('Wall Type').onChange(initTessellation);
    gui.add(params, 'radius', 5, 45).name('Radius (Sph/Cyl/Tor)').onChange(initTessellation);
    gui.add(params, 'height', 10, 100).name('Height (Cyl)').onChange(() => { if(params.wallType === 'cylinder') initTessellation(); });
    gui.add(params, 'tube', 1, 20).name('Tube (Tor/Tref)').onChange(() => { if(params.wallType === 'torus' || params.wallType === 'trefoil') initTessellation(); });
    gui.add(params, 'scale', 1, 20).name('Scale (Tref)').onChange(() => { if(params.wallType === 'trefoil') initTessellation(); });

    gui.add(params, 'count', 100, 5000, 100).onChange(initTessellation);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);

    // Animation Loop
    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}