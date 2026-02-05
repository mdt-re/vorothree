import './style.css';
import init from 'vorothree';

const examples: Record<string, () => Promise<{ run: (app: HTMLElement) => Promise<void> }>> = {
    'moving_cell': () => import('./examples/moving_cell'),
    'walls': () => import('./examples/walls'),
    'performance': () => import('./examples/performance'),
    'relaxation': () => import('./examples/relaxation'),
    'transition': () => import('./examples/transition'),
    'granular': () => import('./examples/granular'),
};

async function run() {
    await init();

    // Inject styles to ensure full screen canvas
    const style = document.createElement('style');
    style.innerHTML = `
        body { margin: 0; overflow: hidden; }
        #app { max-width: none; margin: 0; padding: 0; width: 100%; height: 100%; }
        canvas { display: block; }
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
            <h1>vorothree examples</h1>
            <ul style="list-style: none; padding: 0;">
                ${Object.keys(examples).map(key => `
                    <li style="margin: 10px;">
                        <a href="?example=${key}" style="font-size: 1.2em; color: #646cff;">${key} example</a>
                    </li>
                `).join('')}
            </ul>
        `;
    }
}

run().catch(console.error);