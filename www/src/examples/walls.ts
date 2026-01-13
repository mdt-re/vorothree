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
        wireframe: false
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
        }

        // Generate Random Points
        const points = new Float64Array(params.count * 3);
        for (let i = 0; i < points.length; i++) {
            points[i] = (Math.random() - 0.5) * 100;
        }
        tess.set_generators(points);
        tess.calculate();
        updateVisualization();
    }

    // --- Visualization ---
    
    // 1. Visualize the Sphere "Ghost" (Wireframe)
    let ghostMesh: THREE.Mesh;
    function updateGhost() {
        if (ghostMesh) scene.remove(ghostMesh);
        
        let geometry;
        switch (params.wallType) {
            case 'cylinder':
                geometry = new THREE.CylinderGeometry(params.radius, params.radius, params.height, 32);
                break;
            case 'torus':
                geometry = new THREE.TorusGeometry(params.radius, params.tube, 16, 100);
                break;
            case 'trefoil':
                const knotCurve = new THREE.Curve<THREE.Vector3>();
                knotCurve.getPoint = function(t: number) {
                    const angle = t * Math.PI * 2;
                    const x = Math.sin(angle) + 2.0 * Math.sin(2.0 * angle);
                    const y = Math.cos(angle) - 2.0 * Math.cos(2.0 * angle);
                    const z = -Math.sin(3.0 * angle);
                    const s = params.scale;
                    return new THREE.Vector3(x * s, y * s, z * s);
                };
                geometry = new THREE.TubeGeometry(knotCurve, 200, params.tube, 16, true);
                break;
            case 'sphere':
            default:
                geometry = new THREE.SphereGeometry(params.radius, 32, 32);
        }

        const mat = new THREE.MeshBasicMaterial({ 
            color: 0xff0000, 
            wireframe: true, 
            transparent: true, 
            opacity: 0.1 
        });
        ghostMesh = new THREE.Mesh(geometry, mat);
        scene.add(ghostMesh);
    }

    // 2. Create Meshes for Cells
    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00aaff,
        metalness: 0.1,
        roughness: 0.5,
        transmission: 0.6, // Glass-like
        thickness: 1.0,
        transparent: true,
        opacity: params.opacity,
        wireframe: params.wireframe,
        side: THREE.DoubleSide
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
    updateGhost();

    gui.add(params, 'wallType', ['sphere', 'cylinder', 'torus', 'trefoil']).name('Wall Type').onChange(() => {
        initTessellation();
        updateGhost();
    });
    gui.add(params, 'radius', 5, 45).name('Radius (Sph/Cyl/Tor)').onChange(() => {
        initTessellation();
        updateGhost();
    });
    gui.add(params, 'height', 10, 100).name('Height (Cyl)').onChange(() => { if(params.wallType === 'cylinder') { initTessellation(); updateGhost(); }});
    gui.add(params, 'tube', 1, 20).name('Tube (Tor/Tref)').onChange(() => { if(params.wallType === 'torus' || params.wallType === 'trefoil') { initTessellation(); updateGhost(); }});
    gui.add(params, 'scale', 1, 20).name('Scale (Tref)').onChange(() => { if(params.wallType === 'trefoil') { initTessellation(); updateGhost(); }});

    gui.add(params, 'count', 100, 5000, 100).onChange(initTessellation);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(params, 'wireframe').onChange((v: boolean) => material.wireframe = v);

    // Animation Loop
    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}