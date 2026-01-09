import { defineConfig } from 'vite';

export default defineConfig({
  // Base relative path ensures assets work on GitHub Pages (e.g. user.github.io/repo/)
  base: './', 
  build: {
    target: 'esnext' // Top-level await support
  },
  server: {
    fs: {
      // Allow serving files from one level up to the project root (to access pkg)
      allow: ['..']
    }
  }
});