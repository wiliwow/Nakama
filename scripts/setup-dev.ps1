# Nakama Development Environment Setup
# Quick setup: create .env and directories

Write-Host "Nakama Development Environment Setup" -ForegroundColor Cyan

# Create .env file
@"
STASH_URL=http://localhost:8080
STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash
STASH_EMBEDDING_MODEL=openai:text-embedding-3-small
STASH_OPENAI_API_KEY=your-openai-api-key-here
STASH_OLLAMA_URL=http://localhost:11434
STASH_OLLAMA_MODEL=deepseek-r1:1.5b
"@ | Out-File -FilePath ".env" -Encoding utf8

Write-Host "Created .env file"

# Create directories
New-Item -ItemType Directory -Force -Path "$HOME/.nakama_config" >$null
New-Item -ItemType Directory -Force -Path "$HOME/.lancedb" >$null
New-Item -ItemType Directory -Force -Path "$HOME/.nakama_logs" >$null

Write-Host "Created config directories"

# Install Node deps
pnpm install

Write-Host "Done. Next: ollama pull deepseek-r1:1.5b (if needed), then pnpm tauri dev"