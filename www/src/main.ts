import './style.css';
import init from 'vorothree';

const examples: Record<string, () => Promise<{ run: (app: HTMLElement) => Promise<void> }>> = {
    'moving_cell': () => import('./examples/moving_cell'),
    'walls': () => import('./examples/walls'),
    'benchmark': () => import('./examples/benchmark'),
    'relaxation': () => import('./examples/relaxation'),
    'transition': () => import('./examples/transition'),
    'granular_flow': () => import('./examples/granular_flow'),
    'pathfinding': () => import('./examples/pathfinding'),
    'distributions': () => import('./examples/distributions'),
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
};

const thumbnails = import.meta.glob('./assets/*.png', { eager: true });

async function run() {
    await init();

    // Inject styles to ensure full screen canvas
    const style = document.createElement('style');
    style.innerHTML = `
        body { margin: 0; overflow: hidden; background-color: #1a1a1a; color: #ffffff; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; }
        #app { max-width: none; margin: 0; padding: 0; width: 100%; height: 100%; }
        canvas { display: block; }
        
        /* Gallery Styles */
        .gallery-container { height: 100%; overflow-y: auto; padding: 20px; box-sizing: border-box; }
        .gallery-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 20px; max-width: 1200px; margin: 0 auto; }
        .card { aspect-ratio: 1; position: relative; background: #2a2a2a; border-radius: 8px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.3); transition: transform 0.2s, box-shadow 0.2s; text-decoration: none; color: white; display: block; }
        .card:hover { transform: translateY(-4px); box-shadow: 0 10px 15px rgba(0,0,0,0.4); }
        .card-image { width: 100%; height: 100%; background: #333; display: flex; align-items: center; justify-content: center; overflow: hidden; position: relative; }
        .card-image img { width: 100%; height: 100%; object-fit: cover; position: absolute; top: 0; left: 0; }
        .card-placeholder { font-size: 4rem; color: #444; font-weight: bold; }
        .card-content { position: absolute; bottom: 0; left: 0; width: 100%; padding: 15px; background: rgba(0,0,0,0.6); box-sizing: border-box; backdrop-filter: blur(2px); }
        .card-title { margin: 0; font-size: 1.2rem; font-weight: 500; text-align: center; }
    `;
    document.head.appendChild(style);

    const app = document.querySelector<HTMLDivElement>('#app')!;
    
    const params = new URLSearchParams(window.location.search);
    const exampleName = params.get('example');

    if (exampleName && examples[exampleName]) {
        const module = await examples[exampleName]();
        
        const details = exampleDetails[exampleName];
        const infoDiv = document.createElement('div');
        Object.assign(infoDiv.style, {
            position: 'absolute',
            top: '10px',
            left: '10px',
            zIndex: '1000',
            color: 'white',
            background: 'rgba(0,0,0,0.6)',
            padding: '10px',
            borderRadius: '8px',
            backdropFilter: 'blur(4px)',
            maxWidth: '250px',
            fontFamily: 'sans-serif',
            fontSize: '14px'
        });

        const githubUrl = `https://github.com/mdt-re/vorothree/tree/main/www/src/examples/${details?.file || exampleName + '.ts'}`;
        
        infoDiv.innerHTML = `
            <div style="display: flex; align-items: center; margin-bottom: 5px;">
                <strong style="text-transform: capitalize;">${exampleName.replace(/_/g, ' ')}</strong>
            </div>
            <div style="margin-bottom: 8px; font-size: 0.9em; opacity: 0.8;">${details?.desc || ''}</div>
            <a href="${githubUrl}" target="_blank" style="color: #4af; text-decoration: none; display: flex; align-items: center; font-size: 0.9em;">
                <svg height="16" width="16" viewBox="0 0 16 16" fill="currentColor" style="margin-right: 5px;">
                    <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>
                </svg>
                View Source
            </a>
        `;
        document.body.appendChild(infoDiv);

        await module.run(app);
    } else {
        app.innerHTML = `
            <div class="gallery-container">
                <h1 style="text-align: center; margin-bottom: 40px;">vorothree examples</h1>
                <div class="gallery-grid">
                    ${Object.keys(examples).map(key => {
                        const path = `./assets/${key}.png`;
                        const mod = thumbnails[path] as { default: string };
                        const src = mod?.default || '';
                        return `
                        <a href="?example=${key}" class="card">
                            <div class="card-image">
                                <span class="card-placeholder">${key}</span>
                                <img src="${src}" onerror="this.style.display='none'" alt="${key}" />
                            </div>
                            <div class="card-content">
                                <h3 class="card-title">${key.replace(/_/g, ' ')}</h3>
                            </div>
                        </a>
                    `}).join('')}
                </div>
            </div>
        `;
    }
}

run().catch(console.error);