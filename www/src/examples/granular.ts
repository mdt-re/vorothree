import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import Stats from 'three/examples/jsm/libs/stats.module';
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

    const infoText = document.createElement('div');
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        count: 1000,
        speed: 1.5,
        radius: 12,
        opacity: 0.6,
        wireframe: false
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a1a);

    const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.set(-200, 160, 0);

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

    // --- Vorothree Setup ---
    const boxSize = 150;

    // --- Curve Definition ---
    const points: THREE.Vector3[] = [];
    const turns = 3;
    const helixRadius = 40;
    const start = new THREE.Vector3(-boxSize / 2, -boxSize / 2, -boxSize / 2);
    const end = new THREE.Vector3(boxSize / 2, boxSize / 2, boxSize / 2);

    // Basis for helix
    const axis = new THREE.Vector3().subVectors(end, start);
    const axisNorm = axis.clone().normalize();

    // Arbitrary vector not parallel to axis
    const tmpVec = new THREE.Vector3(0, 1, 0);
    if (Math.abs(axisNorm.dot(tmpVec)) > 0.9) tmpVec.set(1, 0, 0);

    const basisX = new THREE.Vector3().crossVectors(axisNorm, tmpVec).normalize();
    const basisY = new THREE.Vector3().crossVectors(axisNorm, basisX).normalize();

    const numPoints = 50;
    for (let i = 0; i <= numPoints; i++) {
        const t = i / numPoints;
        const pos = new THREE.Vector3().copy(start).lerp(end, t);
        const r = helixRadius * Math.sin(t * Math.PI);
        const angle = t * turns * Math.PI * 2;
        const offsetX = basisX.clone().multiplyScalar(r * Math.cos(angle));
        const offsetY = basisY.clone().multiplyScalar(r * Math.sin(angle));
        pos.add(offsetX).add(offsetY);
        points.push(pos);
    }
    const curve = new THREE.CatmullRomCurve3(points);

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

    // Add the Catmull-Rom Tube Wall
    const wallPoints = new Float64Array(points.length * 3);
    for (let i = 0; i < points.length; i++) {
        wallPoints[i * 3] = points[i].x;
        wallPoints[i * 3 + 1] = points[i].y;
        wallPoints[i * 3 + 2] = points[i].z;
    }
    
    // We'll update the wall if radius changes, so we keep a reference to the ID
    const WALL_ID = -10;
    // @ts-ignore
    tess.add_wall(Wall.new_catmull_rom(wallPoints, params.radius, 200, false, WALL_ID));

    // --- Particle System ---
    let generators = new Float64Array(params.count * 3);
    
    // Particle state
    interface Particle {
        t: number;      // Position along curve (0..1)
        r: number;      // Radial offset
        theta: number;  // Angular offset
        speed: number;  // Individual speed variance
        hue: number;
    }
    let particles: Particle[] = [];
    let currentPhase = 0;

    function initParticles() {
        generators = new Float64Array(params.count * 3);
        particles = [];
        currentPhase = 0;
        
        for(let i = 0; i < params.count; i++) {
            const t = Math.random();
            particles.push({
                t: t,
                r: Math.sqrt(Math.random()) * (params.radius * 0.9), // Keep slightly inside
                theta: Math.random() * Math.PI * 2,
                speed: 0.9 + Math.random() * 0.1,
                hue: (t < 0.5 ? t * 2 : (1 - t) * 2)
            });
        }
        updateGenerators(0);
    }

    function updateGenerators(dt: number) {
        const up = new THREE.Vector3(0, 1, 0);

        // Update phase for rainbow effect
        // Average speed factor is 0.75 (0.5 + 0.5/2)
        const avgSpeed = params.speed * 0.75 * 0.1;
        currentPhase -= dt * avgSpeed;
        if (currentPhase < 0) currentPhase += 1;
        
        for(let i = 0; i < params.count; i++) {
            const p = particles[i];
            
            // Advance particle
            p.t += dt * params.speed * p.speed * 0.1;
            if (p.t > 1.0) {
                p.t -= 1.0;
                p.hue = (currentPhase < 0.5 ? currentPhase * 2 : (1 - currentPhase) * 2);
            }

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
        const volumes: number[] = [];

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get(i);
            if (!cell) continue;

            const vertices = cell.vertices;
            const faces = cell.faces();

            volumes.push(cell.volume());

            if (cell.id >= 0 && cell.id < particles.length) {
                const hue = particles[cell.id].hue;
                color.setHSL(hue, 1.0, 0.5);
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

        if (volumes.length > 0) {
            const sum = volumes.reduce((a, b) => a + b, 0);
            const avg = sum / volumes.length;
            const sqDiff = volumes.reduce((a, b) => a + (b - avg) ** 2, 0);
            const std = Math.sqrt(sqDiff / volumes.length);
            infoText.innerText = `Avg Vol: ${avg.toFixed(2)}\nStd Dev: ${std.toFixed(2)}`;
        }

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
        geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
        geometry.computeVertexNormals();
        const mesh = new THREE.Mesh(geometry, material);
        geometryGroup.add(mesh);
    }

    gui.add(params, 'count', 100, 3000, 100).onChange(initParticles);
    gui.add(params, 'speed', 0, 5);
    gui.add(params, 'radius', 5, 25).onChange(() => {
        tess.clear_walls();
        // @ts-ignore
        tess.add_wall(Wall.new_catmull_rom(wallPoints, params.radius, 200, false, WALL_ID));
    });
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(material, 'wireframe');

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'granular.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        stats.update();
        
        updateGenerators(0.016); // Fixed dt for smoothness
        tess.calculate();
        updateVisualization();

        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}