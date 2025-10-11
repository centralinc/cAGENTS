#!/usr/bin/env node
const { run } = require('../dist/index.js');
process.exit(run(process.argv.slice(2)));
