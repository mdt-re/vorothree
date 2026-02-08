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
};

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
        .gallery-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; max-width: 1200px; margin: 0 auto; }
        .card { background: #2a2a2a; border-radius: 8px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.3); transition: transform 0.2s, box-shadow 0.2s; text-decoration: none; color: white; display: flex; flex-direction: column; }
        .card:hover { transform: translateY(-4px); box-shadow: 0 10px 15px rgba(0,0,0,0.4); }
        .card-image { height: 200px; background: #333; display: flex; align-items: center; justify-content: center; overflow: hidden; position: relative; }
        .card-image img { width: 100%; height: 100%; object-fit: cover; position: absolute; top: 0; left: 0; }
        .card-placeholder { font-size: 4rem; color: #444; font-weight: bold; }
        .card-content { padding: 15px; }
        .card-title { margin: 0; font-size: 1.2rem; font-weight: 500; }
    `;
    document.head.appendChild(style);

    const app = document.querySelector<HTMLDivElement>('#app')!;
    
    const params = new URLSearchParams(window.location.search);
    const exampleName = params.get('example');

    if (exampleName && examples[exampleName]) {
        const module = await examples[exampleName]();
        
        // Add a "Back" button
        const backBtn = document.createElement('a');
        backBtn.href = '/';
        backBtn.textContent = '← back to examples';
        backBtn.style.position = 'absolute';
        backBtn.style.top = '10px';
        backBtn.style.left = '10px';
        backBtn.style.zIndex = '1000';
        backBtn.style.color = 'white';
        backBtn.style.background = 'rgba(0,0,0,0.5)';
        backBtn.style.padding = '5px 10px';
        backBtn.style.borderRadius = '4px';
        backBtn.style.textDecoration = 'none';
        document.body.appendChild(backBtn);

        await module.run(app);
    } else {
        app.innerHTML = `
            <div class="gallery-container">
                <h1 style="text-align: center; margin-bottom: 40px;">vorothree examples</h1>
                <div class="gallery-grid">
                    ${Object.keys(examples).map(key => `
                        <a href="?example=${key}" class="card">
                            <div class="card-image">
                                <span class="card-placeholder">${key}</span>
                                <img src="assets/${key}.png" onerror="this.style.display='none'" alt="${key}" />
                            </div>
                            <div class="card-content">
                                <h3 class="card-title">${key.replace(/_/g, ' ')}</h3>
                            </div>
                        </a>
                    `).join('')}
                </div>
            </div>
        `;
    }
}

run().catch(console.error);