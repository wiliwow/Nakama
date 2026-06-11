@echo off
echo Nakama Development Environment Setup

REM Create .env file
(
echo STASH_URL=http://localhost:8080
echo STASH_DB_URL=postgres://stash:changeme@localhost:5432/stash
echo STASH_EMBEDDING_MODEL=openai:text-embedding-3-small
echo STASH_OPENAI_API_KEY=your-openai-api-key-here
echo STASH_OLLAMA_URL=http://localhost:11434
echo STASH_OLLAMA_MODEL=deepseek-r1:1.5b
) > .env

echo Created .env file

REM Create directories (Windows-friendly paths)
if not exist "%USERPROFILE%\.nakama_config" mkdir "%USERPROFILE%\.nakama_config"
if not exist "%USERPROFILE%\.lancedb" mkdir "%USERPROFILE%\.lancedb"
if not exist "%USERPROFILE%\.nakama_logs" mkdir "%USERPROFILE%\.nakama_logs"

echo Created config directories

REM Install Node deps
pnpm install

echo.
echo Done. Next steps:
echo   1. ollama run deepseek-r1:1.5b (if needed)
echo   2. pnpm tauri dev