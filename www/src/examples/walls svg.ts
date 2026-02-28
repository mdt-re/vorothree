import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation, BoundingBox, Wall } from 'vorothree';
import { SVGRenderer } from 'three/examples/jsm/renderers/SVGRenderer.js';

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
        wallType: 'sphere',
        radius: 40.0,
        radiusA: 40.0,
        radiusB: 30.0,
        radiusC: 20.0,
        height: 60.0,
        tube: 7.0,
        scale: 14.0,
        angle: 0.5,
        count: 2000,
        showEdges: true,
        boundaryEdgesOnly: true,
        edgeOpacity: 0.2,
        svgPrecision: 3,
        animFrames: 30,
        animDuration: 3,
        exportSVG: () => exportSVG(),
        exportAnimatedSVG: () => exportAnimatedSVG(),
        exportJSON: () => exportJSON(),
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xffffff);

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
            case 'catmull':
                const boxSize = 100;
                const spiralPoints: THREE.Vector3[] = [];
                const turns = 3;
                const helixRadius = params.radius;
                const start = new THREE.Vector3(-boxSize / 2, -boxSize / 2, -boxSize / 2);
                const end = new THREE.Vector3(boxSize / 2, boxSize / 2, boxSize / 2);

                const axis = new THREE.Vector3().subVectors(end, start);
                const axisNorm = axis.clone().normalize();

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
                    spiralPoints.push(pos);
                }
                const wallPoints = new Float64Array(spiralPoints.length * 3);
                for (let i = 0; i < spiralPoints.length; i++) {
                    wallPoints[i * 3] = spiralPoints[i].x;
                    wallPoints[i * 3 + 1] = spiralPoints[i].y;
                    wallPoints[i * 3 + 2] = spiralPoints[i].z;
                }
                // @ts-ignore
                tess.add_wall(Wall.new_catmull_rom(wallPoints, params.tube, 200, false, -15));
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
            case 'catmull':
                const c_boxSize = 100;
                const c_points: THREE.Vector3[] = [];
                const c_turns = 3;
                const c_helixRadius = p.radius;
                const c_start = new THREE.Vector3(-c_boxSize / 2, -c_boxSize / 2, -c_boxSize / 2);
                const c_end = new THREE.Vector3(c_boxSize / 2, c_boxSize / 2, c_boxSize / 2);

                const c_axis = new THREE.Vector3().subVectors(c_end, c_start);
                const c_axisNorm = c_axis.clone().normalize();

                const c_tmpVec = new THREE.Vector3(0, 1, 0);
                if (Math.abs(c_axisNorm.dot(c_tmpVec)) > 0.9) c_tmpVec.set(1, 0, 0);

                const c_basisX = new THREE.Vector3().crossVectors(c_axisNorm, c_tmpVec).normalize();
                const c_basisY = new THREE.Vector3().crossVectors(c_axisNorm, c_basisX).normalize();

                const c_numPoints = 50;
                for (let i = 0; i <= c_numPoints; i++) {
                    const t = i / c_numPoints;
                    const pos = new THREE.Vector3().copy(c_start).lerp(c_end, t);
                    const r = c_helixRadius * Math.sin(t * Math.PI);
                    const angle = t * c_turns * Math.PI * 2;
                    const offsetX = c_basisX.clone().multiplyScalar(r * Math.cos(angle));
                    const offsetY = c_basisY.clone().multiplyScalar(r * Math.sin(angle));
                    pos.add(offsetX).add(offsetY);
                    c_points.push(pos);
                }
                const curve = new THREE.CatmullRomCurve3(c_points);
                return curve.getLength() * Math.PI * Math.pow(p.tube, 2);
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
    const edgeMaterial = new THREE.LineBasicMaterial({
        color: 0x000000,
        transparent: true,
        opacity: params.edgeOpacity,
    });

    const edgeGroup = new THREE.Group();
    scene.add(edgeGroup);

    function updateVisualization() {
        // Clear previous meshes
        while (edgeGroup.children.length > 0) {
            const child = edgeGroup.children[0] as THREE.LineSegments;
            child.geometry.dispose();
            edgeGroup.remove(child);
        }

        edgeGroup.visible = params.showEdges;

        const cellCount = tess.count_cells;
        let totalVolume = 0;
        const edgeVertices: number[] = [];

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            totalVolume += cell.volume();

            const vertices = cell.vertices;
            const faces = cell.faces();
            // @ts-ignore
            const neighbors = cell.face_neighbors;

            // Triangulate faces (Fan triangulation for convex polygons)
            for (let j = 0; j < faces.length; j++) {
                const face = faces[j];
                if (face.length < 3) continue;

                if (params.showEdges) {
                    const isBoundary = neighbors && neighbors[j] < 0;
                    if (!params.boundaryEdgesOnly || isBoundary) {
                        for (let k = 0; k < face.length; k++) {
                            const v1Idx = face[k];
                            const v2Idx = face[(k + 1) % face.length];
                            edgeVertices.push(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                            edgeVertices.push(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);
                        }
                    }
                }
            }
        }

        if (params.showEdges && edgeVertices.length > 0) {
            const edgeGeometry = new THREE.BufferGeometry();
            edgeGeometry.setAttribute('position', new THREE.Float32BufferAttribute(edgeVertices, 3));
            const edges = new THREE.LineSegments(edgeGeometry, edgeMaterial);
            edgeGroup.add(edges);
        }

        const expected = getExpectedVolume();
        const deviation = expected > 0 ? ((totalVolume - expected) / expected) * 100 : 0;
        infoText.innerText =
            `total volume:     ${totalVolume.toFixed(2)}\n` + 
            `expected volume:  ${expected.toFixed(2)}\n` +
            `--------------------------------------\n` +
            `deviation:        ${deviation.toFixed(2)}%`;
    }

    function exportSVG() {
        const svgRenderer = new SVGRenderer();
        const size = 1024;
        svgRenderer.setSize(size, size);
        svgRenderer.setPrecision(params.svgPrecision);

        const originalBackground = scene.background;
        const originalAspect = camera.aspect;

        scene.background = null;
        camera.aspect = 1;
        camera.updateProjectionMatrix();

        svgRenderer.render(scene, camera);

        scene.background = originalBackground;
        camera.aspect = originalAspect;
        camera.updateProjectionMatrix();

        const result = new XMLSerializer().serializeToString(svgRenderer.domElement);

        const blob = new Blob([result], { type: 'image/svg+xml' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'voronoi_walls.svg';
        link.click();
    }

    function exportAnimatedSVG() {
        const svgRenderer = new SVGRenderer();
        const size = 1024;
        svgRenderer.setSize(size, size);
        svgRenderer.setPrecision(params.svgPrecision);
        
        const frames = params.animFrames;
        const duration = params.animDuration;
        
        // Save state
        const originalRotZEdge = edgeGroup.rotation.z;
        const originalBackground = scene.background;
        const originalAspect = camera.aspect;

        scene.background = null;
        camera.aspect = 1;
        camera.updateProjectionMatrix();

        const masterSVG = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        masterSVG.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
        masterSVG.setAttribute('xmlns:xlink', 'http://www.w3.org/1999/xlink');
        
        for (let i = 0; i < frames; i++) {
            const angle = (i / frames) * Math.PI * 2;
            edgeGroup.rotation.z = originalRotZEdge + angle;
            edgeGroup.updateMatrixWorld();
            
            svgRenderer.render(scene, camera);
            
            const frameSVG = svgRenderer.domElement;
            
            if (i === 0) {
                // Copy attributes from first frame to master
                for (let j = 0; j < frameSVG.attributes.length; j++) {
                    const attr = frameSVG.attributes[j];
                    masterSVG.setAttribute(attr.name, attr.value);
                }
            }

            // Fix IDs to be unique per frame to avoid collisions (e.g. clip paths)
            const idMap = new Map<string, string>();
            const nodesWithId = frameSVG.querySelectorAll('[id]');
            nodesWithId.forEach((node) => {
                const oldId = node.getAttribute('id');
                if (oldId) {
                    const newId = `frame${i}_${oldId}`;
                    node.setAttribute('id', newId);
                    idMap.set(oldId, newId);
                }
            });

            const allNodes = frameSVG.querySelectorAll('*');
            allNodes.forEach((node) => {
                for (let j = 0; j < node.attributes.length; j++) {
                    const attr = node.attributes[j];
                    const value = attr.value;
                    if (value.includes('url(#')) {
                        const newValue = value.replace(/url\(#([^)]+)\)/g, (match, id) => {
                            if (idMap.has(id)) return `url(#${idMap.get(id)})`;
                            return match;
                        });
                        if (newValue !== value) node.setAttribute(attr.name, newValue);
                    }
                    if ((attr.name === 'xlink:href' || attr.name === 'href') && value.startsWith('#')) {
                         const id = value.substring(1);
                         if (idMap.has(id)) node.setAttribute(attr.name, `#${idMap.get(id)}`);
                    }
                }
            });
            
            const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            g.setAttribute('visibility', 'hidden');
            
            // Move children
            while (frameSVG.childNodes.length > 0) {
                g.appendChild(frameSVG.childNodes[0]);
            }
            
            const animate = document.createElementNS('http://www.w3.org/2000/svg', 'animate');
            animate.setAttribute('attributeName', 'visibility');
            animate.setAttribute('calcMode', 'discrete');
            
            const values = new Array(frames).fill('hidden');
            values[i] = 'visible';
            animate.setAttribute('values', values.join(';'));
            animate.setAttribute('keyTimes', Array.from({ length: frames }, (_, k) => k / frames).join(';'));
            
            animate.setAttribute('dur', `${duration}s`);
            animate.setAttribute('repeatCount', 'indefinite');
            
            g.appendChild(animate);
            masterSVG.appendChild(g);
        }
        
        // Restore state
        edgeGroup.rotation.z = originalRotZEdge;
        edgeGroup.updateMatrixWorld();
        scene.background = originalBackground;
        camera.aspect = originalAspect;
        camera.updateProjectionMatrix();
        
        const serializer = new XMLSerializer();
        const result = serializer.serializeToString(masterSVG);
        
        const blob = new Blob([result], { type: 'image/svg+xml' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'voronoi_animated.svg';
        link.click();
    }

    function exportJSON() {
        const cells = [];
        const count = tess.count_cells;
        for (let i = 0; i < count; i++) {
            const cell = tess.get_cell(i);
            if (cell) {
                cells.push({
                    id: cell.id,
                    vertices: Array.from(cell.vertices),
                    faces: cell.faces(),
                    centroid: cell.centroid()
                });
            }
        }
        const blob = new Blob([JSON.stringify(cells)], { type: 'application/json' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'tessellation.json';
        link.click();
    }

    initTessellation();

    gui.add(params, 'count', 100, 5000, 100).onChange(initTessellation);
    const visFolder = gui.addFolder('Visualization');
    visFolder.add(params, 'showEdges').name('Show Edges').onChange(updateVisualization);
    visFolder.add(params, 'boundaryEdgesOnly').name('Boundary Edges Only').onChange(updateVisualization);
    visFolder.add(params, 'edgeOpacity', 0, 1).name('Edge Opacity').onChange((v: number) => edgeMaterial.opacity = v);

    const wallTypeCtrl = gui.add(params, 'wallType', ['sphere', 'cylinder', 'cone', 'torus', 'trefoil', 'tetrahedron', 'hexahedron', 'octahedron', 'dodecahedron', 'icosahedron', 'ellipsoid', 'bezier', 'catmull']).name('wall');

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
        if (t === 'torus' || t === 'trefoil' || t === 'bezier' || t === 'catmull') tubeCtrl.show(); else tubeCtrl.hide();
        if (t === 'trefoil') scaleCtrl.show(); else scaleCtrl.hide();
        if (t === 'cone') angleCtrl.show(); else angleCtrl.hide();
    };

    wallTypeCtrl.onChange(() => {
        updateVisibility();
        initTessellation();
    });
    updateVisibility();

    gui.add(params, 'svgPrecision', 1, 10, 1).name('SVG Precision');
    gui.add(params, 'exportSVG').name('Export to SVG');
    gui.add(params, 'exportJSON').name('Export to JSON');

    const animFolder = gui.addFolder('Animation Export');
    animFolder.add(params, 'animFrames', 2, 120, 1).name('Frames');
    animFolder.add(params, 'animDuration', 0.1, 60).name('Duration (s)');
    animFolder.add(params, 'exportAnimatedSVG').name('Export Animated SVG');

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