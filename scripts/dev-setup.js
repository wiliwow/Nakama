#!/usr/bin/env node
/**
 * Nakama Full Development Setup
 * One-command setup that initializes:
 * - Environment variables
 * - Configuration directories
 * - Docker services (Stash)
 * - Ollama model
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

console.log('🚀 Nakama Full Development Setup');
console.log('================================\n');

// 1. Create .env file
const envContent = `STASH_URL=http://localhost:8080
STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash
STASH_EMBEDDING_MODEL=openai:text-embedding-3-small
STASH_OPENAI_API_KEY=your-openai-api-key-here
STASH_OLLAMA_URL=http://localhost:11434
STASH_OLLAMA_MODEL=deepseek-r1:1.5b
`;

fs.writeFileSync('.env', envContent);
console.log('✓ Created .env with environment variables');

// 2. Create directories
const dirs = [
  path.join(os.homedir(), '.nakama_config'),
  path.join(os.homedir(), '.lancedb'),
  path.join(os.homedir(), '.nakama_logs')
];

dirs.forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
    console.log(`✓ Created ${dir}`);
  } else {
    console.log(`✓ ${dir} already exists`);
  }
});

// 3. Start Docker services
console.log('\n[Docker] Starting Stash services...');
try {
  const isWindows = process.platform === 'win32';
  const dockerCmd = isWindows 
    ? 'docker-compose -f stash-docker-compose.yml up -d'
    : 'docker-compose -f stash-docker-compose.yml up -d';
  execSync(dockerCmd, { stdio: 'pipe' });
  console.log('✓ Stash Docker started');
} catch (e) {
  console.log('⚠ Docker may already be running or not available');
}

// 4. Check Ollama
console.log('\n[Ollama] Checking model...');
try {
  const models = execSync('ollama list', { encoding: 'utf8' });
  if (!models.includes('deepseek-r1')) {
    console.log('Pulling deepseek-r1:1.5b (this may take a while)...');
    execSync('ollama pull deepseek-r1:1.5b', { stdio: 'inherit' });
  }
  console.log('✓ Ollama ready');
} catch (e) {
  console.log('⚠ Ollama not available - start manually: ollama run deepseek-r1:1.5b');
}

console.log('\n================================');
console.log('✅ Setup complete! Starting Tauri...');