# vorothree WASM

This directory contains the web integration, end-to-end tests, and performance benchmarks for the `vorothree` WASM library. 

## Setup

Before running any commands, ensure you have built the WASM package and installed the dependencies.
```bash
wasm-pack build --target web
npm install
npm run dev
```
For tests and benchmarks run:
```bash
npm test
npx vitest bench
```

##