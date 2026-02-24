import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation, BoundingBox, Wall } from 'vorothree';
import RAPIER from '@dimforge/rapier3d-compat';

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
        geometry: 'helix',
        count: 400,
        speed: 1.5,
        radius: 12,
        opacity: 0.6,
        particleRadius: 4.0,
        showCells: true,
        showParticles: true
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

    // --- Curve Definition ---
    let boxSize = 150;
    const points: THREE.Vector3[] = [];
    const turns = 3;
    const helixRadius = 40;
    const start = new THREE.Vector3(-boxSize / 2, -boxSize / 2, -boxSize / 2);
    const end = new THREE.Vector3(boxSize / 2, boxSize / 2, boxSize / 2);

    // Basis for helix
    const axis = new THREE.Vector3().subVectors(end, start);
    const axisNorm = axis.clone().normalize();
    const axisLenSq = axis.lengthSq();

    // --- Rapier Setup ---
    await RAPIER.init();
    let gravity = { x: axisNorm.x * 9.81, y: axisNorm.y * 9.81, z: axisNorm.z * 9.81 };
    let world = new RAPIER.World(gravity);

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
    let curve = new THREE.CatmullRomCurve3(points);

    // --- Physics Boundaries (Tube) ---
    let wallBody: RAPIER.RigidBody;

    // --- Vorothree Setup ---
    const bounds = new BoundingBox(-boxSize/2, -boxSize/2, -boxSize/2, boxSize/2, boxSize/2, boxSize/2);
    let tess = new Tessellation(bounds, 15, 15, 15);

    // --- Particles ---
    const bodies: RAPIER.RigidBody[] = [];
    let generators = new Float64Array(params.count * 3);

    // --- Particle Visualization (Spheres) ---
    const particleGeo = new THREE.SphereGeometry(1, 16, 16);
    const particleMat = new THREE.MeshStandardMaterial({ color: 0xffaa00, roughness: 0.4, metalness: 0.1 });
    let particleMesh: THREE.InstancedMesh;
    const dummy = new THREE.Object3D();

    function initParticleMesh() {
        if (particleMesh) {
            scene.remove(particleMesh);
            particleMesh.dispose();
        }
        particleMesh = new THREE.InstancedMesh(particleGeo, particleMat, params.count);
        particleMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
        particleMesh.visible = params.showParticles;
        scene.add(particleMesh);
    }
    
    const WALL_ID = -10;

    function spawnParticle(t: number, checkOverlap = false) {
        let pos = new THREE.Vector3();
        
        if (params.geometry === 'helix') {
            pos = curve.getPointAt(t);
            // Random offset inside radius
            const r = Math.sqrt(Math.random()) * (params.radius - params.particleRadius * 1.5);
            const theta = Math.random() * Math.PI * 2;
            
            // Local frame at t
            const tangent = curve.getTangentAt(t);
            let normal = new THREE.Vector3().crossVectors(tangent, new THREE.Vector3(0,1,0)).normalize();
            if (normal.lengthSq() < 0.1) normal = new THREE.Vector3().crossVectors(tangent, new THREE.Vector3(1,0,0)).normalize();
            const binormal = new THREE.Vector3().crossVectors(tangent, normal).normalize();
            
            pos.addScaledVector(normal, r * Math.cos(theta));
            pos.addScaledVector(binormal, r * Math.sin(theta));
        } else if (params.geometry === 'cone') {
            // Spawn at top (+z)
            const z = 75 - Math.random() * 10;
            const rMax = 40 - params.particleRadius * 1.5;
            const r = Math.sqrt(Math.random()) * rMax;
            const theta = Math.random() * Math.PI * 2;
            pos.set(r * Math.cos(theta), r * Math.sin(theta), z);
        } else if (params.geometry === 'torus') {
            // Spawn inside torus
            const u = Math.random() * Math.PI * 2;
            const v = Math.random() * Math.PI * 2;
            const R = 60;
            const r = Math.random() * (15 - params.particleRadius * 1.5);
            pos.x = (R + r * Math.cos(v)) * Math.cos(u);
            pos.y = (R + r * Math.cos(v)) * Math.sin(u);
            pos.z = r * Math.sin(v);
        }

        if (checkOverlap) {
            const thresholdSq = (params.particleRadius * 2.0) ** 2;
            for (const body of bodies) {
                const bPos = body.translation();
                const dx = pos.x - bPos.x;
                const dy = pos.y - bPos.y;
                const dz = pos.z - bPos.z;
                if (dx * dx + dy * dy + dz * dz < thresholdSq) return null;
            }
        }

        const bodyDesc = RAPIER.RigidBodyDesc.dynamic()
            .setTranslation(pos.x, pos.y, pos.z)
            .setLinearDamping(0.5); // Damping to stabilize flow
        const body = world.createRigidBody(bodyDesc);
        const colliderDesc = RAPIER.ColliderDesc.ball(params.particleRadius)
            .setRestitution(0.5)
            .setFriction(0.0); // Low friction for flow
        world.createCollider(colliderDesc, body);
        
        return body;
    }

    function initParticles() {
        // Clear existing
        for(const b of bodies) world.removeRigidBody(b);
        bodies.length = 0;
        generators = new Float64Array(params.count * 3);

        initParticleMesh();

        for(let i = 0; i < params.count; i++) {
            // Spread initial positions
            const b = spawnParticle(Math.random());
            if (b) bodies.push(b);
        }
    }

    let wireframeMesh: THREE.Mesh;

    function initScene() {
        // Cleanup
        if (world) {
            bodies.forEach(b => world.removeRigidBody(b));
            if (wallBody) world.removeRigidBody(wallBody);
            world.free();
        }
        bodies.length = 0;
        if (wireframeMesh) {
            scene.remove(wireframeMesh);
            wireframeMesh.geometry.dispose();
        }
        tess.clear_walls();

        // Setup based on geometry type
        if (params.geometry === 'helix') {
            gravity = { x: axisNorm.x * 9.81, y: axisNorm.y * 9.81, z: axisNorm.z * 9.81 };
            world = new RAPIER.World(gravity);

            // Create a static mesh collider for the tube
            const tubeGeo = new THREE.TubeGeometry(curve, 64, params.radius, 8, false);
            const tubeVerts = new Float32Array(tubeGeo.attributes.position.array);
            const tubeIndices = new Uint32Array(tubeGeo.index!.array);
            
            const wallBodyDesc = RAPIER.RigidBodyDesc.fixed();
            wallBody = world.createRigidBody(wallBodyDesc);
            const wallColliderDesc = RAPIER.ColliderDesc.trimesh(tubeVerts, tubeIndices);
            world.createCollider(wallColliderDesc, wallBody);

            // Vorothree Wall
            const wallPoints = new Float64Array(points.length * 3);
            for (let i = 0; i < points.length; i++) {
                wallPoints[i * 3] = points[i].x;
                wallPoints[i * 3 + 1] = points[i].y;
                wallPoints[i * 3 + 2] = points[i].z;
            }
            // @ts-ignore
            tess.add_wall(Wall.new_catmull_rom(wallPoints, params.radius, 200, false, WALL_ID));

            // Visuals
            const wireframeMat = new THREE.MeshBasicMaterial({ color: 0x444444, wireframe: true, transparent: true, opacity: 0.1 });
            wireframeMesh = new THREE.Mesh(tubeGeo, wireframeMat);
            scene.add(wireframeMesh);

        } else if (params.geometry === 'cone') {
            gravity = { x: 0, y: 0, z: -9.81 };
            world = new RAPIER.World(gravity);

            // Cone: R_small=20 at -z, R_big=40 at +z. Height 150.
            // ThreeJS Cylinder is Y-up. We need to rotate it to align with Z.
            const coneGeo = new THREE.CylinderGeometry(40, 20, 150, 32, 1, true);
            coneGeo.rotateX(Math.PI / 2);

            const coneVerts = new Float32Array(coneGeo.attributes.position.array);
            const coneIndices = new Uint32Array(coneGeo.index!.array);

            const wallBodyDesc = RAPIER.RigidBodyDesc.fixed();
            wallBody = world.createRigidBody(wallBodyDesc);
            const wallColliderDesc = RAPIER.ColliderDesc.trimesh(coneVerts, coneIndices);
            world.createCollider(wallColliderDesc, wallBody);

            // Custom Vorothree Wall for Cone
            const coneWall = {
                contains: (x: number, y: number, z: number) => {
                    if (z < -75 || z > 75) return false;
                    const t = (z + 75) / 150;
                    const r = 20 + t * 20;
                    return x*x + y*y < r*r;
                },
                cut: (x: number, y: number, z: number) => {
                    const t = (z + 75) / 150;
                    const r = 20 + t * 20;
                    const d = Math.sqrt(x*x + y*y);
                    if (d === 0) return null;
                    const factor = r / d;
                    // Normal pointing outwards
                    const nx = x / d;
                    const ny = y / d;
                    const nz = -0.133; // Slope approx
                    const len = Math.sqrt(nx*nx + ny*ny + nz*nz);
                    return {
                        point: [x * factor, y * factor, z],
                        normal: [nx/len, ny/len, nz/len]
                    };
                }
            };
            // @ts-ignore
            tess.add_wall(Wall.newCustom(coneWall, WALL_ID));

            const wireframeMat = new THREE.MeshBasicMaterial({ color: 0x444444, wireframe: true, transparent: true, opacity: 0.1 });
            wireframeMesh = new THREE.Mesh(coneGeo, wireframeMat);
            scene.add(wireframeMesh);

        } else if (params.geometry === 'torus') {
            gravity = { x: 0, y: 0, z: 0 };
            world = new RAPIER.World(gravity);

            const torusGeo = new THREE.TorusGeometry(60, 15, 16, 64);
            const torusVerts = new Float32Array(torusGeo.attributes.position.array);
            const torusIndices = new Uint32Array(torusGeo.index!.array);

            const wallBodyDesc = RAPIER.RigidBodyDesc.fixed();
            wallBody = world.createRigidBody(wallBodyDesc);
            const wallColliderDesc = RAPIER.ColliderDesc.trimesh(torusVerts, torusIndices);
            world.createCollider(wallColliderDesc, wallBody);

            // @ts-ignore
            tess.add_wall(Wall.new_torus(0, 0, 0, 0, 0, 1, 60, 15, WALL_ID));

            const wireframeMat = new THREE.MeshBasicMaterial({ color: 0x444444, wireframe: true, transparent: true, opacity: 0.1 });
            wireframeMesh = new THREE.Mesh(torusGeo, wireframeMat);
            scene.add(wireframeMesh);
        }

        initParticles();
    }

    initScene();

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
            const cell = tess.get_cell(i);
            if (!cell) continue;

            const vertices = cell.vertices;
            const faces = cell.faces();

            volumes.push(cell.volume());

            // Color based on position along curve
            if (cell.id >= 0 && cell.id < bodies.length && bodies[cell.id]) {
                const body = bodies[cell.id];
                const pos = body.translation();
                let hue = 0;

                if (params.geometry === 'helix') {
                    const v = new THREE.Vector3(pos.x, pos.y, pos.z).sub(start);
                    let t = v.dot(axis) / axisLenSq;
                    t = Math.max(0, Math.min(1, t));
                    hue = (t < 0.5 ? t * 2 : (1 - t) * 2);
                } else if (params.geometry === 'cone') {
                    // Color by Z height
                    hue = (pos.z + 75) / 150;
                } else if (params.geometry === 'torus') {
                    // Color by angle around Z
                    const angle = Math.atan2(pos.y, pos.x);
                    hue = (angle + Math.PI) / (2 * Math.PI);
                }
                
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

    // GUI
    gui.add(params, 'geometry', ['helix', 'cone', 'torus']).onChange(initScene);
    gui.add(params, 'count', 50, 800, 10).onChange(initParticles);
    gui.add(params, 'speed', 0, 5);
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(params, 'showCells').name('Show Cells').onChange((v: boolean) => geometryGroup.visible = v);
    gui.add(params, 'showParticles').name('Show Particles').onChange((v: boolean) => { if(particleMesh) particleMesh.visible = v; });

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'granular_flow.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);

        stats.update();

        // Remove bodies outside bounds
        const halfBox = boxSize / 2;
        const limit = halfBox + 10; // little buffer
        for (let i = bodies.length - 1; i >= 0; i--) {
            const body = bodies[i];
            if (!body) continue;
            const pos = body.translation();
            
            // Check bounds based on geometry
            let out = false;
            if (params.geometry === 'helix' || params.geometry === 'cone') {
                if (Math.abs(pos.z) > limit) out = true;
            }
            if (out) {
                 world.removeRigidBody(body);
                 bodies.splice(i, 1);
            }
        }

        // Refill particles
        let attempts = 0;
        while (bodies.length < params.count && attempts < 10) {
            const body = spawnParticle(Math.random() * 0.05, true);
            if (body) {
                bodies.push(body);
            }
            attempts++;
        }

        // Physics Step
        // Apply forces to drive flow along the curve
        for (let i = 0; i < bodies.length; i++) {
            if (!bodies[i]) continue;
            const body = bodies[i];
            const pos = body.translation();
            
            if (params.geometry === 'helix') {
                const p = new THREE.Vector3(pos.x, pos.y, pos.z);
                // Estimate t based on projection onto helix axis
                const v = new THREE.Vector3().subVectors(p, start);
                let t = v.dot(axis) / axisLenSq;
                
                // Force along tangent
                t = Math.max(0, Math.min(1, t));
                const tangent = curve.getTangentAt(t);
                
                // Apply impulse to drive flow
                const forceMag = params.speed * 160.0;
                body.applyImpulse(tangent.multiplyScalar(forceMag * 0.016), true);
            } else if (params.geometry === 'torus') {
                // Flow around Z axis
                const x = pos.x;
                const y = pos.y;
                const len = Math.sqrt(x*x + y*y);
                if (len > 0.001) {
                    const tx = -y / len;
                    const ty = x / len;
                    const forceMag = params.speed * 100.0;
                    body.applyImpulse({ x: tx * forceMag * 0.016, y: ty * forceMag * 0.016, z: 0 }, true);
                }
            }
        }

        world.step();

        // Update generators and visualization
        if (generators.length !== bodies.length * 3) {
            generators = new Float64Array(bodies.length * 3);
        }
        particleMesh.count = bodies.length;

        for (let i = 0; i < bodies.length; i++) {
            const body = bodies[i];
            if (!body) continue;
            const pos = body.translation();

            // Update particle mesh
            dummy.position.set(pos.x, pos.y, pos.z);
            dummy.scale.setScalar(params.particleRadius);
            dummy.updateMatrix();
            particleMesh.setMatrixAt(i, dummy.matrix);

            // Update generator for Voronoi
            generators[i*3] = pos.x;
            generators[i*3+1] = pos.y;
            generators[i*3+2] = pos.z;
        }
        particleMesh.instanceMatrix.needsUpdate = true;

        tess.set_generators(generators);
        tess.calculate();
        updateVisualization();

        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}
