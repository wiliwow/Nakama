#!/usr/bin/env node
/**
 * Quick Nakama Setup - just config and directories
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('Nakama Quick Setup\n');

// Create .env
fs.writeFileSync('.env', `STASH_URL=http://localhost:8080
STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash
STASH_OLLAMA_URL=http://localhost:11434
STASH_OLLAMA_MODEL=deepseek-r1:1.5b
`);

// Create directories
['.nakama_config', '.lancedb', '.nakama_logs'].forEach(d => {
  const dir = path.join(os.homedir(), d);
  fs.mkdirSync(dir, { recursive: true });
  console.log(`Created ${dir}`);
});

console.log('\nDone. Run: ollama pull deepseek-r1:1.5b && pnpm tauri dev');