# vorothree WASM

This directory contains the web integration for the `vorothree` WASM library, including interactive examples, tests, and benchmarks. The live version is hosted on GitHub Pages.

## Setup

Before running any commands, ensure you have built the WASM package and installed the dependencies.
```bash
# From the root of the repository
wasm-pack build --target web
cd www
npm install
```

## Development

To run the interactive examples locally, use the Vite development server. To include the API documentation, generate it first:

```bash
# Generate API documentation into the `docs/` directory
npm run doc

# Start the local development server
npm run dev
```
For tests and benchmarks run:
```bash
npm test
npx vitest bench
```