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
        speed: 0.5,
        radius: 12,
        opacity: 0.6,
        wireframe: false
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a1a);

    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(60, 60, 60);

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
    const pointLight = new THREE.PointLight(0x0088ff, 1, 100);
    pointLight.position.set(0, 20, 0);
    scene.add(pointLight);

    // --- Bezier Curve Definition ---
    const p0 = new THREE.Vector3(-50, -30, -30);
    const p1 = new THREE.Vector3(-20, 50, -10);
    const p2 = new THREE.Vector3(20, -50, 10);
    const p3 = new THREE.Vector3(50, 30, 30);
    const curve = new THREE.CubicBezierCurve3(p0, p1, p2, p3);

    // --- Vorothree Setup ---
    const boxSize = 150;
    const bounds = new BoundingBox(-boxSize/2, -boxSize/2, -boxSize/2, boxSize/2, boxSize/2, boxSize/2);
    const tess = new Tessellation(bounds, 15, 15, 15);

    // --- 3D Grid ---
    const gridVertices: number[] = [];
    const step = 30;
    const half = boxSize / 2;
    for (let i = -half; i <= half; i += step) {
        for (let j = -half; j <= half; j += step) {
            gridVertices.push(-half, i, j, half, i, j);
            gridVertices.push(i, -half, j, i, half, j);
            gridVertices.push(i, j, -half, i, j, half);
        }
    }
    const gridGeo = new THREE.BufferGeometry();
    gridGeo.setAttribute('position', new THREE.Float32BufferAttribute(gridVertices, 3));
    const gridMat = new THREE.LineBasicMaterial({ color: 0x888888, transparent: true, opacity: 0.15 });
    scene.add(new THREE.LineSegments(gridGeo, gridMat));

    // Add the Bezier Tube Wall
    const wallPoints = new Float64Array([
        p0.x, p0.y, p0.z,
        p1.x, p1.y, p1.z,
        p2.x, p2.y, p2.z,
        p3.x, p3.y, p3.z
    ]);
    
    // We'll update the wall if radius changes, so we keep a reference to the ID
    const WALL_ID = -10;
    tess.add_wall(Wall.new_bezier(wallPoints, params.radius, 100, false, WALL_ID));

    // --- Particle System ---
    let generators = new Float64Array(params.count * 3);
    
    // Particle state
    interface Particle {
        t: number;      // Position along curve (0..1)
        r: number;      // Radial offset
        theta: number;  // Angular offset
        speed: number;  // Individual speed variance
    }
    let particles: Particle[] = [];

    function initParticles() {
        generators = new Float64Array(params.count * 3);
        particles = [];
        
        for(let i = 0; i < params.count; i++) {
            particles.push({
                t: Math.random(),
                r: Math.sqrt(Math.random()) * (params.radius * 0.9), // Keep slightly inside
                theta: Math.random() * Math.PI * 2,
                speed: 0.5 + Math.random() * 0.5
            });
        }
        updateGenerators(0);
    }

    function updateGenerators(dt: number) {
        const up = new THREE.Vector3(0, 1, 0);
        
        for(let i = 0; i < params.count; i++) {
            const p = particles[i];
            
            // Advance particle
            p.t += dt * params.speed * p.speed * 0.1;
            if (p.t > 1.0) p.t -= 1.0;

            // Calculate position on curve
            const pos = curve.getPointAt(p.t);
            const tangent = curve.getTangentAt(p.t);

            // Calculate local frame (Normal and Binormal)
            // N = T x Up (or Forward if T is Up)
            let normal = new THREE.Vector3().crossVectors(tangent, up).normalize();
            if (normal.lengthSq() < 0.001) {
                normal = new THREE.Vector3().crossVectors(tangent, new THREE.Vector3(0, 0, 1)).normalize();
            }
            const binormal = new THREE.Vector3().crossVectors(tangent, normal).normalize();

            // Apply radial offset
            // P_final = P_curve + N * (r cos theta) + B * (r sin theta)
            const r = p.r * (params.radius / 12); // Scale r if radius param changes (base 12)
            
            const offsetX = normal.x * r * Math.cos(p.theta) + binormal.x * r * Math.sin(p.theta);
            const offsetY = normal.y * r * Math.cos(p.theta) + binormal.y * r * Math.sin(p.theta);
            const offsetZ = normal.z * r * Math.cos(p.theta) + binormal.z * r * Math.sin(p.theta);

            generators[i*3] = pos.x + offsetX;
            generators[i*3+1] = pos.y + offsetY;
            generators[i*3+2] = pos.z + offsetZ;
        }

        tess.set_generators(generators);
    }

    initParticles();

    // --- Visualization ---
    const material = new THREE.MeshPhysicalMaterial({
        vertexColors: true,
        metalness: 0.2,
        roughness: 0.2,
        transmission: 0.1,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false
    });

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    function updateVisualization() {
        // Clear previous
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0] as THREE.Mesh;
            child.geometry.dispose();
            geometryGroup.remove(child);
        }

        const cellCount = tess.count_cells;
        const positions: number[] = [];
        const colors: number[] = [];
        const color = new THREE.Color();

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get(i);
            if (!cell) continue;

            const vertices = cell.vertices;
            const faces = cell.faces();

            if (cell.id >= 0 && cell.id < particles.length) {
                const t = particles[cell.id].t;
                color.setHSL(t, 1.0, 0.5);
            } else {
                color.setHex(0xffffff);
            }

            for (const face of faces) {
                if (face.length < 3) continue;
                const v0 = face[0];
                for (let k = 1; k < face.length - 1; k++) {
                    const v1 = face[k];
                    const v2 = face[k + 1];
                    positions.push(vertices[v0*3], vertices[v0*3+1], vertices[v0*3+2]);
                    positions.push(vertices[v1*3], vertices[v1*3+1], vertices[v1*3+2]);
                    positions.push(vertices[v2*3], vertices[v2*3+1], vertices[v2*3+2]);

                    colors.push(color.r, color.g, color.b);
                    colors.push(color.r, color.g, color.b);
                    colors.push(color.r, color.g, color.b);
                }
            }
        }

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
        geometry.computeVertexNormals();
        const mesh = new THREE.Mesh(geometry, material);
        geometryGroup.add(mesh);
    }

    gui.add(params, 'count', 100, 3000, 100).onChange(initParticles);
    gui.add(params, 'speed', 0, 2);
    gui.add(params, 'radius', 5, 25).onChange(() => {
        tess.clear_walls();
        tess.add_wall(Wall.new_bezier(wallPoints, params.radius, 100, false, WALL_ID));
    });
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(material, 'wireframe');

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        
        updateGenerators(0.016); // Fixed dt for smoothness
        tess.calculate();
        updateVisualization();

        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}