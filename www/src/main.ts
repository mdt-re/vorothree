/// <reference types="vite/client" />
import './style.css';
import init from 'voronoid';

const examples: Record<string, () => Promise<{ run: (app: HTMLElement) => Promise<void> }>> = {
    'moving_cell': () => import('./examples/moving_cell'),
    'walls': () => import('./examples/walls'),
    'benchmark': () => import('./examples/benchmark'),
    'relaxation': () => import('./examples/relaxation'),
    'transition': () => import('./examples/transition'),
    'granular_flow': () => import('./examples/granular_flow'),
    'pathfinding': () => import('./examples/pathfinding'),
    'distributions': () => import('./examples/distributions'),
    'vector_graphics': () => import('./examples/vector_graphics'),
    'dimension_two': () => import('./examples/dimension_two'),
};

const exampleDetails: Record<string, { desc: string, file: string }> = {
    'moving_cell': { desc: 'Dynamic updates with a moving cell', file: 'moving_cell.ts' },
    'walls': { desc: 'Various boundary wall shapes', file: 'walls.ts' },
    'benchmark': { desc: 'Performance benchmark: Grid vs Octree', file: 'benchmark.ts' },
    'relaxation': { desc: 'Lloyd\'s relaxation for regular cells', file: 'relaxation.ts' },
    'transition': { desc: 'Morphing between boundary shapes', file: 'transition.ts' },
    'granular_flow': { desc: 'Granular flow physics simulation', file: 'granular_flow.ts' },
    'pathfinding': { desc: 'A* pathfinding on Voronoi graph', file: 'pathfinding.ts' },
    'distributions': { desc: 'Different point distributions', file: 'distributions.ts' },
    'vector_graphics': { desc: 'Various boundary wall shapes', file: 'vector_graphics.ts' },
    'dimension_two': { desc: '2D Voronoi with selectable walls', file: 'dimension_two.ts' },
};

async function run() {
    await init();

    const app = document.querySelector<HTMLDivElement>('#app')!;
    
    const params = new URLSearchParams(window.location.search);
    const exampleName = params.get('example');

    if (exampleName && examples[exampleName]) {
        // Inject global styles
        const style = document.createElement('style');
        style.innerHTML = `
            :root {
                --bg-color: #1a1a1a;
                --overlay-bg: rgba(20, 20, 20, 0.8);
                --text-color: #e0e0e0;
                --accent-color: #4af;
            }
            body { 
                margin: 0; 
                overflow: hidden; 
                background-color: var(--bg-color); 
                color: var(--text-color); 
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                -webkit-font-smoothing: antialiased;
            }
            #app { width: 100%; height: 100%; }
            canvas { display: block; outline: none; }
            
            /* Shared Overlay Styles */
            .overlay {
                position: absolute;
                padding: 12px 16px;
                background: var(--overlay-bg);
                backdrop-filter: blur(8px);
                -webkit-backdrop-filter: blur(8px);
                border-radius: 8px;
                border: 1px solid rgba(255,255,255,0.08);
                box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                pointer-events: none;
                user-select: none;
                font-size: 13px;
                line-height: 1.5;
                transition: opacity 0.3s ease;
            }
            
            .overlay-interactive {
                pointer-events: auto;
                user-select: text;
            }

            .overlay-title {
                top: 16px;
                left: 16px;
                z-index: 1000;
                max-width: 300px;
            }
            
            .overlay-info {
                bottom: 16px;
                right: 16px;
                text-align: left;
                font-family: 'Fira Code', 'Consolas', monospace;
                font-size: 12px;
                color: #ccc;
            }

            .overlay-legend {
                bottom: 16px;
                left: 16px;
            }

            /* Link styling */
            .overlay a {
                color: var(--accent-color);
                text-decoration: none;
                transition: color 0.2s;
            }
            .overlay a:hover {
                color: #fff;
            }

            /* Lil-GUI Customization */
            .lil-gui.root {
                top: 16px !important;
                right: 16px !important;
            }
        `;
        document.head.appendChild(style);

        const module = await examples[exampleName]();
        
        const details = exampleDetails[exampleName];
        const infoDiv = document.createElement('div');
        infoDiv.className = 'overlay overlay-title overlay-interactive';

        const githubUrl = `https://github.com/mdt-re/voronoid/tree/main/www/src/examples/${details?.file || exampleName + '.ts'}`;
        
        infoDiv.innerHTML = `
            <div style="display: flex; align-items: center; margin-bottom: 8px;">
                <strong style="text-transform: capitalize; font-size: 1.1em;">${exampleName.replace(/_/g, ' ')}</strong>
            </div>
            <div style="margin-bottom: 12px; opacity: 0.8;">${details?.desc || ''}</div>
            <a href="${githubUrl}" target="_blank" style="display: flex; align-items: center;">
                <svg height="16" width="16" viewBox="0 0 16 16" fill="currentColor" style="margin-right: 6px; opacity: 0.8;">
                    <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>
                </svg>
                View Source
            </a>
        `;
        document.body.appendChild(infoDiv);

        await module.run(app);
    }
}

run().catch(console.error);