import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import Stats from 'three/examples/jsm/libs/stats.module';
import { Tessellation, BoundingBox, Wall } from 'vorothree';

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

    const infoText = document.createElement('div');
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        wallType: 'sphere',
        radius: 40.0,
        radiusA: 40.0,
        radiusB: 30.0,
        radiusC: 20.0,
        height: 60.0,
        tube: 10.0,
        scale: 15.0,
        angle: 0.5,
        count: 2000,
        opacity: 0.3,
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x242424);

    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(100, 100, 100);

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
            case 'cone':
                // @ts-ignore
                tess.add_wall(Wall.new_cone(0.0, 0.0, -50.0, 0.0, 0.0, 1.0, params.angle, -15));
                break;
            case 'torus':
                // @ts-ignore
                tess.add_wall(Wall.new_torus(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, params.radius, params.tube, -15));
                break;
            case 'trefoil':
                tess.add_wall(Wall.new_trefoil(0.0, 0.0, 0.0, params.scale, params.tube, 200, -15));
                break;
            case 'tetrahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_tetrahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'hexahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_hexahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'octahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_octahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'dodecahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_dodecahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'icosahedron':
                // @ts-ignore
                tess.add_wall(Wall.new_icosahedron(0.0, 0.0, 0.0, params.radius, -15));
                break;
            case 'ellipsoid':
                // Custom JS implementation of an Ellipsoid
                // x^2/a^2 + y^2/b^2 + z^2/c^2 <= 1
                const ra = params.radiusA;
                const rb = params.radiusB;
                const rc = params.radiusC;

                const jsWall = {
                    contains: (x: number, y: number, z: number) => {
                        return (x*x)/(ra*ra) + (y*y)/(rb*rb) + (z*z)/(rc*rc) <= 1.0;
                    },
                    cut: (x: number, y: number, z: number) => {
                        // Simple radial projection approximation for the cut point.
                        // This is not the exact closest point, but sufficient for convex walls in many cases.
                        
                        // 1. Map to unit sphere space
                        const mx = x / ra;
                        const my = y / rb;
                        const mz = z / rc;
                        const len = Math.sqrt(mx*mx + my*my + mz*mz);
                        
                        if (len === 0) return null;

                        // 2. Project to surface
                        const px = (mx / len) * ra;
                        const py = (my / len) * rb;
                        const pz = (mz / len) * rc;

                        // 3. Calculate normal at surface point (gradient of implicit function)
                        let nx = px / (ra*ra);
                        let ny = py / (rb*rb);
                        let nz = pz / (rc*rc);
                        const nLen = Math.sqrt(nx*nx + ny*ny + nz*nz);
                        
                        return {
                            point: [px, py, pz],
                            normal: [nx/nLen, ny/nLen, nz/nLen]
                        };
                    }
                };
                // @ts-ignore
                tess.add_wall(Wall.newCustom(jsWall, -15));
                break;
            case 'bezier':
                // Define 4 control points for a cubic bezier curve
                const points = new Float64Array([
                    -params.radius, -params.radius, -params.radius, // P0
                    -params.radius/2, params.radius, 0,             // P1
                    params.radius/2, -params.radius, 0,             // P2
                    params.radius, params.radius, params.radius     // P3
                ]);
                tess.add_wall(Wall.new_bezier(points, params.tube, 100, false, -15));
                break;
        }

        tess.random_generators(params.count);
        tess.calculate();
        updateVisualization();
    }

    function getExpectedVolume() {
        const p = params;
        switch (p.wallType) {
            case 'sphere':
                return (4/3) * Math.PI * Math.pow(p.radius, 3);
            case 'cylinder':
                // Infinite cylinder clipped by 100x100x100 box
                return Math.PI * Math.pow(p.radius, 2) * 100;
            case 'cone':
                const r_cone = 100.0 * Math.tan(p.angle);
                return (1.0/3.0) * Math.PI * Math.pow(r_cone, 2) * 100.0;
            case 'torus':
                return 2 * Math.pow(Math.PI, 2) * p.radius * Math.pow(p.tube, 2);
            case 'tetrahedron':
                return (8 / (9 * Math.sqrt(3))) * Math.pow(p.radius, 3);
            case 'hexahedron':
                return (8 / (3 * Math.sqrt(3))) * Math.pow(p.radius, 3);
            case 'octahedron':
                return (4 / 3) * Math.pow(p.radius, 3);
            case 'ellipsoid':
                return (4/3) * Math.PI * p.radiusA * p.radiusB * p.radiusC;
            case 'dodecahedron':
                const a_d = (4 * p.radius) / (Math.sqrt(3) * (1 + Math.sqrt(5)));
                return ((15 + 7 * Math.sqrt(5)) / 4) * Math.pow(a_d, 3);
            case 'icosahedron':
                const a_i = (4 * p.radius) / Math.sqrt(10 + 2 * Math.sqrt(5));
                return (5 * (3 + Math.sqrt(5)) / 12) * Math.pow(a_i, 3);
            case 'trefoil':
                return calculateTrefoilLength(p.scale) * Math.PI * Math.pow(p.tube, 2);
            case 'bezier':
                return calculateBezierLength(p.radius) * Math.PI * Math.pow(p.tube, 2);
        }
        return 0;
    }

    function calculateTrefoilLength(scale: number) {
        const steps = 10000;
        let len = 0;
        const dt = (2 * Math.PI) / steps;
        for(let i=0; i<steps; i++) {
            const t = i * dt;
            const dx = Math.cos(t) + 4 * Math.cos(2*t);
            const dy = -Math.sin(t) + 4 * Math.sin(2*t);
            const dz = -3 * Math.cos(3*t);
            len += Math.sqrt(dx*dx + dy*dy + dz*dz) * dt;
        }
        return len * scale;
    }

    function calculateBezierLength(r: number) {
        const p0 = new THREE.Vector3(-r, -r, -r);
        const p1 = new THREE.Vector3(-r/2, r, 0);
        const p2 = new THREE.Vector3(r/2, -r, 0);
        const p3 = new THREE.Vector3(r, r, r);
        const v0 = new THREE.Vector3().subVectors(p1, p0);
        const v1 = new THREE.Vector3().subVectors(p2, p1);
        const v2 = new THREE.Vector3().subVectors(p3, p2);
        const steps = 100;
        let len = 0;
        const dt = 1.0 / steps;
        for(let i=0; i<steps; i++) {
            const t = i * dt;
            const mt = 1-t;
            const d = new THREE.Vector3().addScaledVector(v0, 3*mt*mt).addScaledVector(v1, 6*mt*t).addScaledVector(v2, 3*t*t);
            len += d.length() * dt;
        }
        return len;
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
        let totalVolume = 0;

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get(i);
            if (!cell) continue;

            totalVolume += cell.volume();

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

        const expected = getExpectedVolume();
        const deviation = expected > 0 ? ((totalVolume - expected) / expected) * 100 : 0;
        infoText.innerText =
            `total volume:     ${totalVolume.toFixed(2)}\n` + 
            `expected volume:  ${expected.toFixed(2)}\n` +
            `--------------------------------------\n` +
            `deviation:        ${deviation.toFixed(2)}%`;
    }

    initTessellation();

    gui.add(params, 'count', 100, 5000, 100).onChange(initTessellation);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);

    const wallTypeCtrl = gui.add(params, 'wallType', ['sphere', 'cylinder', 'cone', 'torus', 'trefoil', 'tetrahedron', 'hexahedron', 'octahedron', 'dodecahedron', 'icosahedron', 'ellipsoid', 'bezier']).name('wall');

    const radiusCtrl = gui.add(params, 'radius', 5, 45).name('radius').onChange(initTessellation);
    const radiusACtrl = gui.add(params, 'radiusA', 5, 45).name('radius x').onChange(initTessellation);
    const radiusBCtrl = gui.add(params, 'radiusB', 5, 45).name('radius y').onChange(initTessellation);
    const radiusCCtrl = gui.add(params, 'radiusC', 5, 45).name('radius z').onChange(initTessellation);
    const heightCtrl = gui.add(params, 'height', 10, 100).name('height').onChange(initTessellation);
    const tubeCtrl = gui.add(params, 'tube', 1, 20).name('radius tube').onChange(initTessellation);
    const scaleCtrl = gui.add(params, 'scale', 5, 20).name('scale').onChange(initTessellation);
    const angleCtrl = gui.add(params, 'angle', 0.1, 1.0).name('angle').onChange(initTessellation);

    const updateVisibility = () => {
        const t = params.wallType;
        if (t === 'trefoil' || t === 'ellipsoid' || t === 'bezier' || t === 'cone') radiusCtrl.hide(); else radiusCtrl.show();

        if (t === 'ellipsoid') {
            radiusACtrl.show(); radiusBCtrl.show(); radiusCCtrl.show();
        } else {
            radiusACtrl.hide(); radiusBCtrl.hide(); radiusCCtrl.hide();
        }

        if (t === 'cylinder') heightCtrl.show(); else heightCtrl.hide();
        if (t === 'torus' || t === 'trefoil' || t === 'bezier') tubeCtrl.show(); else tubeCtrl.hide();
        if (t === 'trefoil') scaleCtrl.show(); else scaleCtrl.hide();
        if (t === 'cone') angleCtrl.show(); else angleCtrl.hide();
    };

    wallTypeCtrl.onChange(() => {
        updateVisibility();
        initTessellation();
    });
    updateVisibility();

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'walls.png';
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