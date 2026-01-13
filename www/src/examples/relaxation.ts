import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import { Tessellation, BoundingBox } from 'vorothree';

export async function run(app: HTMLElement) {
    app.innerHTML = '';

    // --- UI ---
    const ui = document.createElement('div');
    ui.style.position = 'absolute';
    ui.style.top = '10px';
    ui.style.right = '10px';
    ui.style.padding = '15px';
    ui.style.background = 'rgba(0,0,0,0.6)';
    ui.style.color = 'white';
    ui.style.borderRadius = '8px';
    ui.style.fontFamily = 'sans-serif';
    ui.style.pointerEvents = 'auto';
    ui.innerHTML = `
        <h3 style="margin: 0 0 10px 0;">Lloyd's Relaxation</h3>
        <p style="font-size: 0.9em; margin-bottom: 15px;">
            Iteratively moves generators to cell centroids.<br>
            Cells become more uniform (honeycomb-like).
        </p>
    `;
    app.appendChild(ui);

    const gui = new GUI({ container: app });
    gui.domElement.style.position = 'absolute';
    gui.domElement.style.top = '10px';
    gui.domElement.style.right = '10px';

    const params = {
        count: 100,
        autoRelax: true,
        relax: () => {
            tess.relax();
            tess.calculate();
            updateMesh();
        },
        reset: () => resetGenerators()
    };

    // --- Three.js ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x222222);

    const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 500);
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

    const light = new THREE.DirectionalLight(0xffffff, 1);
    light.position.set(50, 100, 50);
    scene.add(light);
    scene.add(new THREE.AmbientLight(0x404040));

    // --- Vorothree ---
    const size = 60;
    const bounds = new BoundingBox(-size/2, -size/2, -size/2, size/2, size/2, size/2);
    const tess = new Tessellation(bounds, 8, 8, 8);

    // Visuals
    const group = new THREE.Group();
    scene.add(group);

    const material = new THREE.MeshPhysicalMaterial({
        color: 0x00ff88,
        metalness: 0.1,
        roughness: 0.2,
        transmission: 0.2,
        transparent: true,
        opacity: 0.8,
        side: THREE.DoubleSide
    });

    // Wireframe box
    const boxGeo = new THREE.BoxGeometry(size, size, size);
    const boxMat = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true, opacity: 0.2, transparent: true });
    scene.add(new THREE.Mesh(boxGeo, boxMat));

    function resetGenerators() {
        const points = new Float64Array(params.count * 3);
        for(let i=0; i<params.count * 3; i++) {
            points[i] = (Math.random() - 0.5) * size;
        }
        tess.set_generators(points);
        tess.calculate();
        updateMesh();
    }

    function updateMesh() {
        // Cleanup
        while(group.children.length) {
            const m = group.children.pop() as THREE.Mesh;
            if (m.geometry) m.geometry.dispose();
        }

        const count = tess.count_cells;
        for(let i=0; i<count; i++) {
            const cell = tess.get(i);
            if(!cell) continue;

            const verts = cell.vertices;
            const faces = cell.faces();
            const positions: number[] = [];

            for(const face of faces) {
                if(face.length < 3) continue;
                const v0 = face[0];
                for(let k=1; k<face.length-1; k++) {
                    const v1 = face[k];
                    const v2 = face[k+1];
                    positions.push(
                        verts[v0*3], verts[v0*3+1], verts[v0*3+2],
                        verts[v1*3], verts[v1*3+1], verts[v1*3+2],
                        verts[v2*3], verts[v2*3+1], verts[v2*3+2]
                    );
                }
            }

            const geo = new THREE.BufferGeometry();
            geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
            geo.computeVertexNormals();
            
            const mesh = new THREE.Mesh(geo, material);
            group.add(mesh);
        }
    }

    resetGenerators();

    gui.add(params, 'count', 10, 1000, 10).name('Point Count').onChange(resetGenerators);
    gui.add(params, 'autoRelax').name('Auto Relax');
    gui.add(params, 'relax').name('Step Relax');
    gui.add(params, 'reset').name('Reset');

    let frame = 0;
    function animate() {
        if(!app.isConnected) return;
        requestAnimationFrame(animate);

        // Relax every 30 frames to visualize the steps
        if(params.autoRelax && frame % 30 === 0) {
            tess.relax();
            tess.calculate();
            updateMesh();
        }

        controls.update();
        renderer.render(scene, camera);
        frame++;
    }
    animate();
}