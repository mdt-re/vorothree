import * as THREE from 'three';
import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import Stats from 'three/examples/jsm/libs/stats.module.js';
import { CSS2DRenderer, CSS2DObject } from 'three/examples/jsm/renderers/CSS2DRenderer.js';
// @ts-ignore
import { Tessellation2D, BoundingBox2D, Wall2D } from 'vorothree';

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

    const legendDiv = document.createElement('div');
    legendDiv.style.position = 'absolute';
    legendDiv.style.bottom = '10px';
    legendDiv.style.left = '10px';
    legendDiv.style.textAlign = 'left';
    legendDiv.style.color = 'white';
    legendDiv.style.backgroundColor = 'rgba(0, 0, 0, 0.5)';
    legendDiv.style.padding = '10px';
    legendDiv.style.fontFamily = 'monospace';
    legendDiv.style.pointerEvents = 'none';
    legendDiv.style.userSelect = 'none';
    legendDiv.style.display = 'none';
    app.appendChild(legendDiv);

    const params = {
        wallType: 'circle',
        radius: 40.0,
        innerRadius: 20.0,
        sides: 6,
        width: 60.0,
        height: 60.0,
        count: 500,
        opacity: 0.5,
        showEdges: true,
        showFaces: true,
        showLabels: false,
        showNeighborLabels: false,
        checkNeighbors: false,
        colorByVertexCount: false,
    };

    // --- Three.js Setup ---
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x242424);

    const aspect = window.innerWidth / window.innerHeight;
    const frustumSize = 120;
    const camera = new THREE.OrthographicCamera(
        frustumSize * aspect / -2, frustumSize * aspect / 2,
        frustumSize / 2, frustumSize / -2,
        0.1, 1000
    );
    camera.position.set(0, 0, 100);
    camera.lookAt(0, 0, 0);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    app.appendChild(renderer.domElement);

    const labelRenderer = new CSS2DRenderer();
    labelRenderer.setSize(window.innerWidth, window.innerHeight);
    labelRenderer.domElement.style.position = 'absolute';
    labelRenderer.domElement.style.top = '0px';
    labelRenderer.domElement.style.pointerEvents = 'none';
    app.appendChild(labelRenderer.domElement);

    window.addEventListener('resize', () => {
        const aspect = window.innerWidth / window.innerHeight;
        camera.left = -frustumSize * aspect / 2;
        camera.right = frustumSize * aspect / 2;
        camera.top = frustumSize / 2;
        camera.bottom = -frustumSize / 2;
        camera.updateProjectionMatrix();
        renderer.setSize(window.innerWidth, window.innerHeight);
        labelRenderer.setSize(window.innerWidth, window.innerHeight);
    });

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.enableRotate = false;
    controls.mouseButtons = {
        LEFT: THREE.MOUSE.PAN,
        MIDDLE: THREE.MOUSE.DOLLY,
        RIGHT: THREE.MOUSE.PAN
    };

    // Helper to visualize bounds
    const boxSize = 100;
    const boxGeo = new THREE.EdgesGeometry(new THREE.PlaneGeometry(boxSize, boxSize));
    const boxLines = new THREE.LineSegments(boxGeo, new THREE.LineBasicMaterial({ color: 0x888888 }));
    scene.add(boxLines);

    // --- Vorothree Setup ---
    let tess: any;

    function initTessellation() {
        const half = boxSize / 2;
        const bounds = new BoundingBox2D(-half, -half, half, half);
        tess = new Tessellation2D(bounds, 10, 10);

        switch (params.wallType) {
            case 'circle':
                // @ts-ignore
                tess.add_wall(Wall2D.new_circle(0.0, 0.0, params.radius, -1000));
                break;
            case 'annulus':
                // @ts-ignore
                tess.add_wall(Wall2D.new_annulus(0.0, 0.0, params.innerRadius, params.radius, -1000));
                break;
            case 'regular_polygon':
                // @ts-ignore
                tess.add_wall(Wall2D.new_regular_polygon(0.0, 0.0, params.radius, params.sides, -1000));
                break;
            case 'rectangle':
                const w = params.width / 2;
                const h = params.height / 2;
                const rectWall = {
                    contains: (x: number, y: number) => Math.abs(x) <= w && Math.abs(y) <= h,
                    cut: (x: number, y: number) => {
                        if (Math.abs(x) <= w && Math.abs(y) <= h) return null;
                        
                        let px = Math.max(-w, Math.min(w, x));
                        let py = Math.max(-h, Math.min(h, y));
                        
                        let nx = 0, ny = 0;
                        if (Math.abs(x) - w > Math.abs(y) - h) {
                            nx = Math.sign(x);
                        } else {
                            ny = Math.sign(y);
                        }
                        
                        return {
                            point: [px, py],
                            normal: [nx, ny]
                        };
                    }
                };
                // @ts-ignore
                tess.add_wall(Wall2D.newCustom(rectWall, -1000));
                break;
            case 'diamond':
                 const r = params.radius;
                 const diamondWall = {
                     contains: (x: number, y: number) => Math.abs(x) + Math.abs(y) <= r,
                     cut: (x: number, y: number) => {
                         if (Math.abs(x) + Math.abs(y) <= r) return null;
                         const sx = Math.sign(x) || 1;
                         const sy = Math.sign(y) || 1;
                         const nx = sx / Math.sqrt(2);
                         const ny = sy / Math.sqrt(2);
                         return {
                             point: [sx * r, 0],
                             normal: [nx, ny]
                         };
                     }
                 };
                 // @ts-ignore
                 tess.add_wall(Wall2D.newCustom(diamondWall, -1000));
                 break;
        }

        tess.random_generators(params.count);
        tess.calculate();
        updateVisualization();
    }

    // --- Visualization ---
    const material = new THREE.MeshBasicMaterial({
        color: 0x00aaff,
        transparent: true,
        opacity: params.opacity,
        side: THREE.DoubleSide,
    });
    
    const edgeMaterial = new THREE.LineBasicMaterial({ color: 0xffffff });

    const geometryGroup = new THREE.Group();
    scene.add(geometryGroup);

    function updateVisualization() {
        while (geometryGroup.children.length > 0) {
            const child = geometryGroup.children[0];
            if ((child as any).geometry) (child as any).geometry.dispose();
            geometryGroup.remove(child);
        }

        const cellCount = tess.count_cells;
        const positions: number[] = [];
        const edgePositions: number[] = [];
        const colors: number[] = [];

        let totalArea = 0;
        const presentCounts = new Set<number>();

        for (let i = 0; i < cellCount; i++) {
            const cell = tess.get_cell(i);
            if (!cell) continue;

            let cellColor = new THREE.Color(0x00aaff);
            const verts = cell.vertices; 
            if (!verts || verts.length < 6) continue;

            // Area
            let area = 0;
            const numVerts = verts.length / 2;
            for (let j = 0; j < numVerts; j++) {
                const x1 = verts[j * 2];
                const y1 = verts[j * 2 + 1];
                const x2 = verts[((j + 1) % numVerts) * 2];
                const y2 = verts[((j + 1) % numVerts) * 2 + 1];
                area += (x1 * y2 - x2 * y1);
            }
            totalArea += Math.abs(area) / 2;

            // Faces
            if (params.showFaces) {
                if (params.colorByVertexCount) {
                    presentCounts.add(numVerts);
                    cellColor.setHSL(((numVerts - 3) / 8.0) % 1.0, 1.0, 0.5);
                }
                if (params.checkNeighbors) {
                    for (let j = 0; j < numVerts; j++) {
                        const neighborId = cell.edge_neighbors[j];
                        if (neighborId >= 0) {
                            const neighborCell = tess.get_cell(neighborId);
                            if (neighborCell) {
                                const neighborNeighbors = neighborCell.edge_neighbors;
                                if (!neighborNeighbors.includes(i)) {
                                    cellColor = new THREE.Color(0xff0000);
                                    break;
                                }
                            }
                        }
                    }
                }

                const r = cellColor.r;
                const g = cellColor.g;
                const b = cellColor.b;

                const v0x = verts[0];
                const v0y = verts[1];
                for (let j = 1; j < numVerts - 1; j++) {
                    const v1x = verts[j * 2];
                    const v1y = verts[j * 2 + 1];
                    const v2x = verts[(j + 1) * 2];
                    const v2y = verts[(j + 1) * 2 + 1];
                    positions.push(v0x, v0y, 0, v1x, v1y, 0, v2x, v2y, 0);
                    colors.push(r, g, b, r, g, b, r, g, b);
                }

                // Labels
                if (params.showLabels || params.showNeighborLabels) {
                    let cx = 0, cy = 0;
                    for(let j=0; j<numVerts; j++) {
                        cx += verts[j*2];
                        cy += verts[j*2+1];
                    }
                    cx /= numVerts;
                    cy /= numVerts;

                    if (params.showLabels) {
                        const div = document.createElement('div');
                        div.textContent = i.toString();
                        div.style.color = 'white';
                        div.style.fontSize = '10px';
                        div.style.fontFamily = 'sans-serif';
                        div.style.textShadow = '1px 1px 1px #000';

                        const label = new CSS2DObject(div);
                        label.position.set(cx, cy, 0);
                        geometryGroup.add(label);
                    }

                    if (params.showNeighborLabels) {
                        for (let j = 0; j < numVerts; j++) {
                            const neighborId = cell.edge_neighbors[j];
                            const v1x = verts[j * 2];
                            const v1y = verts[j * 2 + 1];
                            const next = (j + 1) % numVerts;
                            const v2x = verts[next * 2];
                            const v2y = verts[next * 2 + 1];

                            const mx = (v1x + v2x) * 0.5;
                            const my = (v1y + v2y) * 0.5;
                            const lx = mx + (cx - mx) * 0.2;
                            const ly = my + (cy - my) * 0.2;

                            const div = document.createElement('div');
                            div.textContent = neighborId.toString();
                            div.style.color = '#ffaa00';
                            div.style.fontSize = '8px';
                            div.style.fontFamily = 'sans-serif';

                            const label = new CSS2DObject(div);
                            label.position.set(lx, ly, 0);
                            geometryGroup.add(label);
                        }
                    }
                }
            }

            // Edges
            if (params.showEdges) {
                for (let j = 0; j < numVerts; j++) {
                    edgePositions.push(verts[j * 2], verts[j * 2 + 1], 0);
                    const next = (j + 1) % numVerts;
                    edgePositions.push(verts[next * 2], verts[next * 2 + 1], 0);
                }
            }
        }

        if (positions.length > 0) {
            const geo = new THREE.BufferGeometry();
            geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
            geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
            const mesh = new THREE.Mesh(geo, material);
            geometryGroup.add(mesh);
        }


        if (edgePositions.length > 0) {
            const geo = new THREE.BufferGeometry();
            geo.setAttribute('position', new THREE.Float32BufferAttribute(edgePositions, 3));
            const lines = new THREE.LineSegments(geo, edgeMaterial);
            geometryGroup.add(lines);
        }

        infoText.innerText = `Cells: ${cellCount}\nTotal Area: ${totalArea.toFixed(2)}`;

        if (params.showFaces && params.colorByVertexCount) {
            legendDiv.style.display = 'block';
            const sortedCounts = Array.from(presentCounts).sort((a, b) => a - b);
            let html = '<div style="margin-bottom:5px; font-weight:bold; text-transform:lowercase;">vertices</div>';
            for (const count of sortedCounts) {
                const hue = ((count - 3) / 8.0) % 1.0;
                const color = new THREE.Color().setHSL(hue, 1.0, 0.5).getStyle();
                html += `<div style="display:flex; align-items:center; gap:8px; margin-bottom:2px;"><div style="width:12px; height:12px; background-color:${color};"></div><div>${count}</div></div>`;
            }
            legendDiv.innerHTML = html;
        } else {
            legendDiv.style.display = 'none';
        }
    }

    material.vertexColors = true;

    initTessellation();

    gui.add(params, 'count', 10, 2000, 10).onChange(initTessellation);
    const wallFolder = gui.addFolder('Wall Settings');
    const wallTypeCtrl = wallFolder.add(params, 'wallType', ['circle', 'annulus', 'regular_polygon', 'rectangle', 'diamond']);
    const radiusCtrl = wallFolder.add(params, 'radius', 10, 50).onChange(initTessellation);
    const innerRadiusCtrl = wallFolder.add(params, 'innerRadius', 5, 45).onChange(initTessellation);
    const sidesCtrl = wallFolder.add(params, 'sides', 3, 12, 1).onChange(initTessellation);
    const widthCtrl = wallFolder.add(params, 'width', 10, 100).onChange(initTessellation);
    const heightCtrl = wallFolder.add(params, 'height', 10, 100).onChange(initTessellation);

    const updateVisibility = () => {
        const type = params.wallType;
        radiusCtrl.show(['circle', 'annulus', 'regular_polygon', 'diamond'].includes(type));
        innerRadiusCtrl.show(type === 'annulus');
        sidesCtrl.show(type === 'regular_polygon');
        widthCtrl.show(type === 'rectangle');
        heightCtrl.show(type === 'rectangle');
    };

    wallTypeCtrl.onChange(() => {
        updateVisibility();
        initTessellation();
    });
    updateVisibility();
    
    gui.add(params, 'opacity', 0, 1).onChange((v: number) => material.opacity = v);
    gui.add(params, 'showEdges').onChange(updateVisualization);
    gui.add(params, 'checkNeighbors').onChange(updateVisualization);
    gui.add(params, 'colorByVertexCount').name('Color by Vertices').onChange(updateVisualization);
    gui.add(params, 'showLabels').onChange(updateVisualization);
    gui.add(params, 'showNeighborLabels').name('Show Neighbors').onChange(updateVisualization);
    gui.add(params, 'showFaces').onChange(updateVisualization);

    function animate() {
        if (!app.isConnected) return;
        requestAnimationFrame(animate);
        stats.update();
        controls.update();
        labelRenderer.render(scene, camera);
        renderer.render(scene, camera);
    }
    animate();
}