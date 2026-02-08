import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
import Stats from 'three/examples/jsm/libs/stats.module';
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

    const infoText = document.createElement('div');
    infoText.style.marginBottom = '10px';
    resultsDiv.appendChild(infoText);
    app.appendChild(resultsDiv);

    const stats = new Stats();
    stats.dom.style.position = 'static';
    stats.dom.style.pointerEvents = 'auto';
    resultsDiv.appendChild(stats.dom);

    const params = {
        count: 500,
        paths: 3,
        opacity: 0.2,
        pathOpacity: 0.9,
        relaxIterations: 5,
        torusRadius: 40,
        torusTube: 15,
        mode: 'Random Paths',
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
    const cellSequenceMap = new Map<number, number>(); // cellId -> sequenceIndex
    let graph: any;

    const pathColors = [
        new THREE.Color(0xff3333),
        new THREE.Color(0x33ff33),
        new THREE.Color(0x3333ff),
        new THREE.Color(0xffff33),
        new THREE.Color(0x33ffff),
        new THREE.Color(0xff33ff),
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
            const cell = tess.get(i);
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
        cellSequenceMap.clear();
        if (!graph) buildGraph();
        
        const nodeIds: number[] = [];
        graph.forEachNode((node: any) => {
            nodeIds.push(node.id);
        });

        if (nodeIds.length === 0) {
            console.warn("Graph has no nodes.");
            return;
        }
        console.log(`Graph has ${nodeIds.length} nodes available for pathfinding.`);
        
        if (params.mode === 'Longest Path') {
            calculateLongestPath(nodeIds);
            return;
        }

        const pathFinder = aStar(graph, {
            distance(a: any, b: any) {
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

        for (let p = 0; p < params.paths; p++) {
            let path: any[] = [];
            let attempts = 0;
            while (path.length === 0 && attempts < 50) {
                attempts++;
                const startId = nodeIds[Math.floor(Math.random() * nodeIds.length)];

                // BFS to find a reachable node at some distance
                const visited = new Set<number>();
                const queue: { id: number, dist: number }[] = [{ id: startId, dist: 0 }];
                visited.add(startId);
                const candidates: number[] = [];
                let head = 0;

                while (head < queue.length && candidates.length < 20 && head < 500) {
                    const { id, dist } = queue[head++];
                    if (dist > 8) candidates.push(id);
                    
                    graph.forEachLinkedNode(id, (node: any) => {
                        if (!visited.has(node.id)) {
                            visited.add(node.id);
                            queue.push({ id: node.id, dist: dist + 1 });
                        }
                    });
                }

                if (candidates.length > 0) {
                    const endId = candidates[Math.floor(Math.random() * candidates.length)];
                    path = pathFinder.find(startId, endId);
                    if (path.length > 0) {
                        console.log(`Path ${p} found with length ${path.length}`);
                    }
                }
            }
            
            if (path.length === 0) console.warn(`Failed to find path ${p} after ${attempts} attempts.`);
            
            for (const node of path) {
                cellPathMap.set(Number(node.id), p);
            }
        }
        
        infoText.innerText = `Paths generated: ${params.paths}`;
    }

    function calculateLongestPath(nodeIds: number[]) {
        let bestPath: number[] = [];
        
        // Try multiple starts to find a better path
        const attempts = 20;
        for(let i=0; i<attempts; i++) {
             const startId = nodeIds[Math.floor(Math.random() * nodeIds.length)];
             const path = [startId];
             const visited = new Set([startId]);
             let curr = startId;
             
             while(true) {
                 let neighbors: number[] = [];
                 graph.forEachLinkedNode(curr, (node: any) => {
                     if (!visited.has(node.id)) neighbors.push(node.id);
                 });
                 
                 if (neighbors.length === 0) break;
                 
                 // Warnsdorff's rule: choose neighbor with fewest unvisited neighbors
                 neighbors.sort((a: number, b: number) => {
                     let degA = 0;
                     graph.forEachLinkedNode(a, (n: any) => { if(!visited.has(n.id)) degA++; });
                     let degB = 0;
                     graph.forEachLinkedNode(b, (n: any) => { if(!visited.has(n.id)) degB++; });
                     return degA - degB;
                 });
                 
                 const next = neighbors[0];
                 path.push(next);
                 visited.add(next);
                 curr = next;
             }
             
             if (path.length > bestPath.length) bestPath = path;
             if (bestPath.length === nodeIds.length) break;
        }
        
        for (let i = 0; i < bestPath.length; i++) {
            const nodeId = bestPath[i];
            cellPathMap.set(nodeId, 0);
            cellSequenceMap.set(nodeId, i);
        }
        infoText.innerText = `Longest Path: ${bestPath.length} / ${nodeIds.length} cells\nCoverage: ${((bestPath.length/nodeIds.length)*100).toFixed(1)}%`;
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
            const cell = tess.get(i);
            if (!cell) continue;

            // Determine color
            if (params.mode === 'Longest Path' && cellSequenceMap.has(cell.id)) {
                const seqIdx = cellSequenceMap.get(cell.id)!;
                const maxSeq = cellSequenceMap.size > 1 ? cellSequenceMap.size - 1 : 1;
                const hue = (seqIdx / maxSeq) * 0.8; // 0.0 (Red) to 0.8 (Purple)
                color.setHSL(hue, 1.0, 0.5);
            } else if (cellPathMap.has(cell.id)) {
                const pathIdx = cellPathMap.get(cell.id)!;
                const c = pathColors[pathIdx % pathColors.length];
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
    gui.add(params, 'mode', ['Random Paths', 'Longest Path']).onChange(() => { calculatePaths(); updateVisualization(); });
    gui.add(params, 'paths', 1, 6, 1).onChange(() => { calculatePaths(); updateVisualization(); });
    gui.add(params, 'torusRadius', 10, 50).onChange(init);
    gui.add(params, 'torusTube', 5, 30).onChange(init);
    gui.add(params, 'opacity', 0, 1).name('Cell Opacity').onChange(updateVisualization);
    gui.add(params, 'pathOpacity', 0, 1).name('Path Opacity').onChange(updateVisualization);

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
