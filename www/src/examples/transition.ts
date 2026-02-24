import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation, BoundingBox, Wall } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = '';

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
        transition: 'Dodec <-> Ico',
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
        let totalVolume = 0;
        let totalArea = 0;

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            totalVolume += cell.volume();

            const vertices = cell.vertices;
            const faces = cell.faces();
            const positions: number[] = [];

            for (const face of faces) {
                if (face.length < 3) continue;
                const v0Idx = face[0];
                const v0x = vertices[v0Idx * 3];
                const v0y = vertices[v0Idx * 3 + 1];
                const v0z = vertices[v0Idx * 3 + 2];

                // Calculate face area for stats
                let faceArea = 0;
                // Simple polygon area for convex faces (fan from v0)
                const v0 = new THREE.Vector3(v0x, v0y, v0z);
                const v1 = new THREE.Vector3();
                const v2 = new THREE.Vector3();
                const cross = new THREE.Vector3();

                for (let k = 1; k < face.length - 1; k++) {
                    const v1Idx = face[k];
                    const v2Idx = face[k + 1];

                    positions.push(v0x, v0y, v0z);
                    positions.push(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                    positions.push(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);

                    v1.set(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                    v2.set(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);
                    
                    cross.crossVectors(v1.sub(v0), v2.sub(v0));
                    faceArea += cross.length() * 0.5;
                    v1.copy(v0); // Reset for next triangle if needed, though v0 is constant here
                }
                totalArea += faceArea;
            }

            const geometry = new THREE.BufferGeometry();
            geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
            geometry.computeVertexNormals();
            const mesh = new THREE.Mesh(geometry, material);
            geometryGroup.add(mesh);
        }

        const avgVolume = cellCount > 0 ? totalVolume / cellCount : 0;
        infoText.innerText = `Total Volume:   ${totalVolume.toFixed(0)}\n` +
                             `Total Area:     ${totalArea.toFixed(0)}\n` +
                             `Avg Cell Vol:   ${avgVolume.toFixed(1)}`;
    }

    initGenerators();

    gui.add(params, 'transition', ['Dodec <-> Ico', 'Cube <-> Octa', 'Sphere <-> Cube', 'Cylinder <-> Cone']);
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

    // Cube Normals (6 faces)
    const cubeNormals: number[] = [];
    for(const s of [-1, 1]) { cubeNormals.push(s, 0, 0); cubeNormals.push(0, s, 0); cubeNormals.push(0, 0, s); }

    // Octahedron Normals (8 faces)
    const octaNormals: number[] = [];
    const invSqrt3 = 1 / Math.sqrt(3);
    for(const x of [-1, 1]) for(const y of [-1, 1]) for(const z of [-1, 1]) 
        octaNormals.push(x*invSqrt3, y*invSqrt3, z*invSqrt3);

    // Inradius factors
    const xi_dodec = Math.sqrt((5 + 2 * Math.sqrt(5)) / 15);
    const xi_ico = Math.sqrt((5 + 2 * Math.sqrt(5)) / 15);

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        stats.update();

        const time = performance.now() * 0.001 * params.speed;
        
        // Morph parameter t: 0 = Dodecahedron, 1 = Icosahedron
        const t = params.animate ? (Math.sin(time) + 1) / 2 : (params.transition === 'Dodec <-> Ico' ? 0 : 0.5);

        const R = params.radius;
        tess.clear_walls();

        if (params.transition === 'Dodec <-> Ico') {
            const large = R * 2.0; 
            const d_dodec_base = R * xi_dodec;
            const d_ico_base = R * xi_ico;

            const d_dodec = d_dodec_base + (large - d_dodec_base) * Math.pow(t, 2);
            const d_ico = d_ico_base + (large - d_ico_base) * Math.pow(1 - t, 2);

            const points = new Float64Array((12 + 20) * 3);
            const normals = new Float64Array((12 + 20) * 3);
            let ptr = 0;

            for(let i=0; i<dodecNormals.length; i+=3) {
                const nx = dodecNormals[i], ny = dodecNormals[i+1], nz = dodecNormals[i+2];
                normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
                points[ptr] = nx * d_dodec; points[ptr+1] = ny * d_dodec; points[ptr+2] = nz * d_dodec;
                ptr += 3;
            }
            for(let i=0; i<icoNormals.length; i+=3) {
                const nx = icoNormals[i], ny = icoNormals[i+1], nz = icoNormals[i+2];
                normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
                points[ptr] = nx * d_ico; points[ptr+1] = ny * d_ico; points[ptr+2] = nz * d_ico;
                ptr += 3;
            }
            // @ts-ignore
            tess.add_wall(Wall.new_convex_polyhedron(points, normals, -15));

        } else if (params.transition === 'Cube <-> Octa') {
            const large = R * 2.0;
            // Cube inradius = R. Octahedron inradius = R.
            const d_cube = R + (large - R) * Math.pow(t, 2);
            const d_octa = R + (large - R) * Math.pow(1 - t, 2);

            const points = new Float64Array((6 + 8) * 3);
            const normals = new Float64Array((6 + 8) * 3);
            let ptr = 0;

            for(let i=0; i<cubeNormals.length; i+=3) {
                const nx = cubeNormals[i], ny = cubeNormals[i+1], nz = cubeNormals[i+2];
                normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
                points[ptr] = nx * d_cube; points[ptr+1] = ny * d_cube; points[ptr+2] = nz * d_cube;
                ptr += 3;
            }
            for(let i=0; i<octaNormals.length; i+=3) {
                const nx = octaNormals[i], ny = octaNormals[i+1], nz = octaNormals[i+2];
                normals[ptr] = nx; normals[ptr+1] = ny; normals[ptr+2] = nz;
                points[ptr] = nx * d_octa; points[ptr+1] = ny * d_octa; points[ptr+2] = nz * d_octa;
                ptr += 3;
            }
            // @ts-ignore
            tess.add_wall(Wall.new_convex_polyhedron(points, normals, -15));

        } else if (params.transition === 'Sphere <-> Cube') {
            // Superquadric: |x|^p + |y|^p + |z|^p <= R^p
            // p=2 is sphere, p->inf is cube
            const p = 2 + t * 18; // 2 to 20
            
            const superquadric = {
                contains: (x: number, y: number, z: number) => {
                    return Math.pow(Math.abs(x), p) + Math.pow(Math.abs(y), p) + Math.pow(Math.abs(z), p) <= Math.pow(R, p);
                },
                cut: (x: number, y: number, z: number) => {
                    // Radial projection approximation
                    const distP = Math.pow(Math.abs(x), p) + Math.pow(Math.abs(y), p) + Math.pow(Math.abs(z), p);
                    if (distP === 0) return null;
                    
                    // Scale factor to surface
                    const scale = R / Math.pow(distP, 1/p);
                    
                    // Normal is gradient: p*|x|^(p-1)*sgn(x)
                    const nx = Math.sign(x) * Math.pow(Math.abs(x), p-1);
                    const ny = Math.sign(y) * Math.pow(Math.abs(y), p-1);
                    const nz = Math.sign(z) * Math.pow(Math.abs(z), p-1);
                    const len = Math.sqrt(nx*nx + ny*ny + nz*nz);

                    return {
                        point: [x * scale, y * scale, z * scale],
                        normal: [nx/len, ny/len, nz/len]
                    };
                }
            };
            // @ts-ignore
            tess.add_wall(Wall.newCustom(superquadric, -15));

        } else if (params.transition === 'Cylinder <-> Cone') {
            // Morph radius profile along Y
            // Cylinder: r(y) = R
            // Cone: r(y) tapers. Let's taper the top (y > 0) more than bottom? 
            // Or simple linear taper: r(y) = R * (1 - t * (y + H/2)/H)
            const H = 100;
            const yMin = -H/2;
            
            const coneCyl = {
                contains: (x: number, y: number, z: number) => {
                    if (y < -H/2 || y > H/2) return false;
                    const normalizedY = (y - yMin) / H; // 0 to 1
                    const r = R * (1 - t * 0.8 * normalizedY); // Taper to 20% at top when t=1
                    return x*x + z*z <= r*r;
                },
                cut: (x: number, y: number, z: number) => {
                    const normalizedY = (y - yMin) / H;
                    const r = R * (1 - t * 0.8 * normalizedY);
                    const d = Math.sqrt(x*x + z*z);
                    if (d === 0) return null;
                    
                    // Normal: slope is dr/dy = -R * t * 0.8 / H
                    const slope = -R * t * 0.8 / H;
                    const nx = x/d;
                    const nz = z/d;
                    const ny = -slope;
                    const len = Math.sqrt(nx*nx + ny*ny + nz*nz);
                    
                    const factor = r / d;
                    return {
                        point: [x * factor, y, z * factor],
                        normal: [nx/len, ny/len, nz/len]
                    };
                }
            };
            // @ts-ignore
            tess.add_wall(Wall.newCustom(coneCyl, -15));
        }

        tess.calculate();

        updateVisualization();
        controls.update();
        renderer.render(scene, camera);
    }
    animate();

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'transition.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });
}