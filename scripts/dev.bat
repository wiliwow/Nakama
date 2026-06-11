@echo off
echo Nakama Full Development Environment
echo ======================================

REM Load/create .env
if not exist .env (
    echo STASH_URL=http://localhost:8080 > .env
    echo STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash >> .env
    echo STASH_OLLAMA_URL=http://localhost:11434 >> .env
    echo STASH_OLLAMA_MODEL=deepseek-r1:1.5b >> .env
    echo Created .env
)

REM Create directories
if not exist "%USERPROFILE%\.nakama_config" mkdir "%USERPROFILE%\.nakama_config"
if not exist "%USERPROFILE%\.lancedb" mkdir "%USERPROFILE%\.lancedb"
if not exist "%USERPROFILE%\.nakama_logs" mkdir "%USERPROFILE%\.nakama_logs"

echo Directories ready

REM Start Stash Docker
echo.
echo [Docker] Starting Stash services...
docker-compose -f stash-docker-compose.yml up -d 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Stash Docker started
) else (
    echo Docker may already be running or not available
)

REM Check Ollama
echo.
echo [Ollama] Checking model...
ollama list | findstr "deepseek-r1" >nul
if %ERRORLEVEL% NEQ 0 (
    echo Pulling deepseek-r1:1.5b...
    ollama pull deepseek-r1:1.5b
)
echo Ollama ready

echo.
echo Starting Tauri development server...
pnpm tauri dev