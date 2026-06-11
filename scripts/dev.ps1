# Nakama Full Development Environment
# One-command setup: starts Ollama, Docker, and Tauri dev

param(
    [switch]$SkipDocker,
    [switch]$SkipOllama
)

Write-Host "Nakama Full Development Environment" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

# Load .env if exists
if (Test-Path ".env") {
    Get-Content ".env" | ForEach-Object {
        if ($_ -match "^([^=]+)=(.*)$") {
            $key, $value = $matches[1], $matches[2]
            $env:$key = $value
            Write-Host "Loaded $key"
        }
    }
} else {
    # Create base .env
    @"
STASH_URL=http://localhost:8080
STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash
STASH_OLLAMA_URL=http://localhost:11434
STASH_OLLAMA_MODEL=deepseek-r1:1.5b
"@ | Out-File -FilePath ".env" -Encoding utf8
}

# Ensure directories exist
@("$HOME/.nakama_config", "$HOME/.lancedb", "$HOME/.nakama_logs") | ForEach-Object {
    if (-not (Test-Path $_)) { New-Item -ItemType Directory -Path $_ -Force | Out-Null }
}

# Start Stash Docker service
if (-not $SkipDocker) {
    Write-Host ""
    Write-Host "[Docker] Starting Stash services..." -ForegroundColor Yellow
    if (Test-Path "stash-docker-compose.yml") {
        docker-compose -f "stash-docker-compose.yml" up -d 2>&1 | Out-Null
        Write-Host "Stash Docker started" -ForegroundColor Green
    }
}

# Start Ollama if needed
if (-not $SkipOllama) {
    Write-Host ""
    Write-Host "[Ollama] Checking model..." -ForegroundColor Yellow
    try {
        $running = ollama info 2>&1
        if (-not (ollama list 2>&1 | Select-String "deepseek-r1")) {
            Write-Host "Pulling deepseek-r1:1.5b..." -ForegroundColor Yellow
            ollama pull deepseek-r1:1.5b 2>&1 | Out-Null
        }
        Write-Host "Ollama ready" -ForegroundColor Green
    } catch {
        Write-Host "Ollama not available - start manually: ollama run deepseek-r1:1.5b" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Starting Tauri development server..." -ForegroundColor Cyan
pnpm tauri dev