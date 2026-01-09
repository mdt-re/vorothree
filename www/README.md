# Vorothree Web Demo & Tests

This directory contains the web integration, end-to-end tests, and performance benchmarks for the `vorothree` WASM library.

## Setup

Before running any commands, ensure you have built the WASM package and installed the dependencies.

1. **Build the WASM package** (from the parent directory):
   ```bash
   cd ..
   rustup run nightly wasm-pack build --target web
   cd www
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

## Running the Demo

To start the development server and view the interactive demo:

```bash
npm run dev
```

## Running Tests

To run the integration tests (using Vitest):

```bash
npm test
```

## Running Benchmarks

To run the performance benchmarks (using Vitest Bench):

```bash
npx vitest bench
```