const fs = require('fs');
const path = require('path');

const pkgPath = path.join(__dirname, 'pkg', 'package.json');
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));

pkg.files = pkg.files || [];
if (!pkg.files.includes('snippets')) {
    pkg.files.push('snippets');
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
    console.log('Successfully patched pkg/package.json to include "snippets"');
}

const readmeSrc = path.join(__dirname, 'README_WASM.md');
const readmeDest = path.join(__dirname, 'pkg', 'README.md');
fs.copyFileSync(readmeSrc, readmeDest);
console.log('Successfully copied README_WASM.md to pkg/README.md');