import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { Tessellation, BoundingBox, Wall } from 'vorothree';
// @ts-ignore
import createGraph from 'ngraph.graph';
// @ts-ignore
import { aStar } from 'ngraph.path';

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
        count: 1000,
        paths: 5,
        opacity: 0.1,
        pathOpacity: 0.8,
        relaxIterations: 5,
        torusRadius: 40,
        torusTube: 15,
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x222222);

    const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
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
    controls.autoRotate = true;
    controls.autoRotateSpeed = 0.5;

    // Lights
    const ambientLight = new THREE.AmbientLight(0x404040, 10);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(50, 100, 50);
    scene.add(dirLight);

    // --- Vorothree Setup ---
    const boxSize = 150;
    const bounds = new BoundingBox(-boxSize/2, -boxSize/2, -boxSize/2, boxSize/2, boxSize/2, boxSize/2);
    const tess = new Tessellation(bounds, 15, 15, 15);

    let generators = new Float64Array(0);
    const cellPathMap = new Map<number, number>(); // cellId -> pathIndex
    let graph: any;

    const pathColors = [
        { name: 'dark red',  color: new THREE.Color(0xff6666) },
        { name: 'light red', color: new THREE.Color(0xffcccc) },
        { name: 'orange', color: new THREE.Color(0xffcc99) },
        { name: 'yellow', color: new THREE.Color(0xffff99) },
        { name: 'green',  color: new THREE.Color(0x99ff99) },
        { name: 'cyan',   color: new THREE.Color(0x99ffff) },
        { name: 'blue',   color: new THREE.Color(0x9999ff) },
        { name: 'purple', color: new THREE.Color(0xcc99ff) },
    ];
    const whiteColor = new THREE.Color(0xffffff);


    function init() {
        tess.clear_walls();
        // @ts-ignore
        tess.add_wall(Wall.new_torus(0, 0, 0, 0, 0, 1, params.torusRadius, params.torusTube, -10));

        // 1. Calculate tessellation for random points inside the wall.
        tess.random_generators(params.count);
        tess.calculate();

        // 2. Relax tessellation a few times.
        for (let i = 0; i < params.relaxIterations; i++) {
            tess.relax();
            tess.calculate();
        }

        buildGraph();

        // 3. Calculate Paths
        calculatePaths();

        // 4. Update Visualization
        updateVisualization();
    }

    function buildGraph() {
        console.time("buildGraph");
        const cellCount = tess.count_cells;
        graph = createGraph();
        let linkCount = 0;
        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;
            
            const c = cell.centroid();
            graph.addNode(cell.id, { x: c[0], y: c[1], z: c[2] });

            // @ts-ignore
            const neighbors = cell.face_neighbors;
            for (let j = 0; j < neighbors.length; j++) {
                const nId = neighbors[j];
                if (nId >= 0) {
                    graph.addLink(cell.id, nId);
                    linkCount++;
                }
            }
        }
        console.timeEnd("buildGraph");
        console.log(`Graph built. Nodes: ${cellCount}, Links: ${linkCount}`);
    }

    function calculatePaths() {
        console.log("Calculating paths...");
        cellPathMap.clear();
        if (!graph) buildGraph();
        
        const nodeIds: number[] = [];
        const nodePositions = new Map<number, {x: number, y: number, z: number}>();
        graph.forEachNode((node: any) => {
            nodeIds.push(node.id);
            nodePositions.set(node.id, node.data);
        });

        if (nodeIds.length === 0) {
            console.warn("Graph has no nodes.");
            return;
        }
        console.log(`Graph has ${nodeIds.length} nodes available for pathfinding.`);
        
        const usedCells = new Set<number>();

        const pathFinder = aStar(graph, {
            distance(a: any, b: any) {
                if (usedCells.has(b.id)) return Infinity;
                const dx = a.data.x - b.data.x;
                const dy = a.data.y - b.data.y;
                const dz = a.data.z - b.data.z;
                return Math.sqrt(dx * dx + dy * dy + dz * dz);
            },
            heuristic(a: any, b: any) {
                const dx = a.data.x - b.data.x;
                const dy = a.data.y - b.data.y;
                const dz = a.data.z - b.data.z;
                return Math.sqrt(dx * dx + dy * dy + dz * dz);
            }
        });

        let statsText = `paths:        ${params.paths}\n` +
                        `--------------------------------------\n` +
                        `color         | cells     | length (px)\n` +
                        `--------------------------------------\n`;

        const R = params.torusRadius;

        for (let p = 0; p < params.paths; p++) {
            let fullPath: any[] = [];
            let success = false;
            let attempts = 0;
            
            while (!success && attempts < 20) {
                attempts++;
                fullPath = [];
                
                // Generate 6 waypoints
                const segments = 6;
                const angleOffset = Math.random() * Math.PI * 2;
                const waypoints: number[] = [];
                let waypointsFound = true;

                for (let i = 0; i < segments; i++) {
                    const theta = angleOffset + (i / segments) * Math.PI * 2;
                    const tx = R * Math.cos(theta);
                    const ty = R * Math.sin(theta);
                    const tz = 0; // Center of tube cross section is at z=0

                    // Find closest unused node
                    let closest = -1;
                    let minD2 = Infinity;
                    
                    for (const id of nodeIds) {
                        if (usedCells.has(id)) continue;
                        // Avoid picking same node as previous waypoint
                        if (waypoints.length > 0 && waypoints[waypoints.length-1] === id) continue;

                        const pos = nodePositions.get(id)!;
                        const d2 = (pos.x - tx)**2 + (pos.y - ty)**2 + (pos.z - tz)**2;
                        if (d2 < minD2) {
                            minD2 = d2;
                            closest = id;
                        }
                    }

                    if (closest !== -1) {
                        waypoints.push(closest);
                    } else {
                        waypointsFound = false;
                        break;
                    }
                }

                if (!waypointsFound) continue;

                // Close the loop
                waypoints.push(waypoints[0]);

                // Connect waypoints
                let pathNodes: any[] = [];
                let connectionFailed = false;

                for (let i = 0; i < segments; i++) {
                    const start = waypoints[i];
                    const end = waypoints[i+1];
                    
                    const segment = pathFinder.find(start, end);
                    if (segment.length === 0) {
                        connectionFailed = true;
                        break;
                    }

                    // ngraph.path returns path in reverse order (end -> start)
                    const segmentNodes = segment.reverse();
                    
                    // If not the first segment, remove the first node (it's the same as last of previous)
                    if (i > 0) {
                        segmentNodes.shift();
                    }
                    
                    for (const node of segmentNodes) {
                        pathNodes.push(node);
                    }
                }

                if (!connectionFailed) {
                    // Check for duplicates in pathNodes (self-intersection)
                    const seen = new Set<number>();
                    let selfIntersect = false;
                    for (let i = 0; i < pathNodes.length - 1; i++) {
                        const id = pathNodes[i].id;
                        if (seen.has(id)) {
                            selfIntersect = true;
                            break;
                        }
                        seen.add(id);
                    }
                    
                    if (!selfIntersect) {
                        fullPath = pathNodes;
                        success = true;
                    }
                }
            }
            
            if (success) {
                let len = 0;
                for(let k=0; k<fullPath.length-1; k++) {
                    const a = fullPath[k].data;
                    const b = fullPath[k+1].data;
                    const dx = a.x - b.x;
                    const dy = a.y - b.y;
                    const dz = a.z - b.z;
                    len += Math.sqrt(dx * dx + dy * dy + dz * dz);
                }

                const colorEntry = pathColors[p % pathColors.length];
                const colorName = colorEntry.name.padEnd(13, ' ');
                statsText += `${colorName} | ${(fullPath.length - 1).toString().padStart(9)} | ${len.toFixed(2).padStart(11)}\n`;

                // Register used cells
                for (let i=0; i<fullPath.length - 1; i++) {
                    const id = Number(fullPath[i].id);
                    cellPathMap.set(id, p);
                    usedCells.add(id);
                }
            } else {
                console.warn(`Failed to find path ${p}`);
            }
        }
        
        infoText.innerText = statsText;
    }

    // --- Visualization ---
    const bgMaterial = new THREE.MeshStandardMaterial({
        vertexColors: true,
        roughness: 0.2,
        metalness: 0.1,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
        depthWrite: false
    });

    const pathMaterial = new THREE.MeshStandardMaterial({
        vertexColors: true,
        roughness: 0.2,
        metalness: 0.1,
        transparent: true,
        opacity: params.pathOpacity,
        side: THREE.DoubleSide
    });

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    function updateVisualization() {
        console.log(`Updating visualization. Path cells: ${cellPathMap.size}`);
        // Clear previous meshes
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0] as THREE.Mesh;
            child.geometry.dispose();
            geometryGroup.remove(child);
        }

        bgMaterial.opacity = params.opacity;
        pathMaterial.opacity = params.pathOpacity;

        const cellCount = tess.count_cells;
        const bgPositions: number[] = [];
        const bgColors: number[] = [];
        const pathPositions: number[] = [];
        const pathColorBuffer: number[] = [];
        const color = new THREE.Color();

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            // Determine color
            if (cellPathMap.has(cell.id)) {
                const pathIdx = cellPathMap.get(cell.id)!;
                const c = pathColors[pathIdx % pathColors.length].color;
                if (c) color.copy(c); else color.copy(whiteColor);
            } else {
                color.copy(whiteColor);
            }

            const vertices = cell.vertices;
            const faces = cell.faces();

            for (const face of faces) {
                if (face.length < 3) continue;
                const v0Idx = face[0];
                const v0x = vertices[v0Idx * 3];
                const v0y = vertices[v0Idx * 3 + 1];
                const v0z = vertices[v0Idx * 3 + 2];

                for (let k = 1; k < face.length - 1; k++) {
                    const v1Idx = face[k];
                    const v2Idx = face[k + 1];

                    if (cellPathMap.has(cell.id)) {
                        pathPositions.push(v0x, v0y, v0z);
                        pathPositions.push(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                        pathPositions.push(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);
                        pathColorBuffer.push(color.r, color.g, color.b);
                        pathColorBuffer.push(color.r, color.g, color.b);
                        pathColorBuffer.push(color.r, color.g, color.b);
                    } else {
                        bgPositions.push(v0x, v0y, v0z);
                        bgPositions.push(vertices[v1Idx * 3], vertices[v1Idx * 3 + 1], vertices[v1Idx * 3 + 2]);
                        bgPositions.push(vertices[v2Idx * 3], vertices[v2Idx * 3 + 1], vertices[v2Idx * 3 + 2]);
                        bgColors.push(color.r, color.g, color.b);
                        bgColors.push(color.r, color.g, color.b);
                        bgColors.push(color.r, color.g, color.b);
                    }
                }
            }
        }

        if (bgPositions.length > 0) {
            const bgGeo = new THREE.BufferGeometry();
            bgGeo.setAttribute('position', new THREE.Float32BufferAttribute(bgPositions, 3));
            bgGeo.setAttribute('color', new THREE.Float32BufferAttribute(bgColors, 3));
            bgGeo.computeVertexNormals();
            geometryGroup.add(new THREE.Mesh(bgGeo, bgMaterial));
        }

        if (pathPositions.length > 0) {
            const pathGeo = new THREE.BufferGeometry();
            pathGeo.setAttribute('position', new THREE.Float32BufferAttribute(pathPositions, 3));
            pathGeo.setAttribute('color', new THREE.Float32BufferAttribute(pathColorBuffer, 3));
            pathGeo.computeVertexNormals();
            geometryGroup.add(new THREE.Mesh(pathGeo, pathMaterial));
        }
    }

    init();

    gui.add(params, 'count', 100, 2000, 100).onChange(init);
    gui.add(params, 'paths', 1, 8, 1).onChange(() => { calculatePaths(); updateVisualization(); });
    gui.add(params, 'torusRadius', 10, 50).name('torus radius').onChange(init);
    gui.add(params, 'torusTube', 5, 30).name('tube radius').onChange(init);
    gui.add(params, 'opacity', 0, 1).name('cell opacity').onChange(updateVisualization);
    gui.add(params, 'pathOpacity', 0, 1).name('path opacity').onChange(updateVisualization);

    // Handle screenshot
    window.addEventListener('keydown', (event) => {
        if (event.key === 'p') {
            renderer.render(scene, camera);
            const link = document.createElement('a');
            link.download = 'pathfinding.png';
            link.href = renderer.domElement.toDataURL('image/png');
            link.click();
        }
    });

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        stats.update();
        controls.update();
        renderer.render(scene, camera);
    }
    animate();
}
