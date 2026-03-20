@echo off
title Ollama Launcher

echo Starting Docker if needed...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
"$p = Get-Process -Name 'Docker Desktop' -ErrorAction SilentlyContinue; ^
if (-not $p) { Start-Process 'C:\Program Files\Docker\Docker\Docker Desktop.exe' }; ^
do { Start-Sleep 2; docker version > $null 2>&1 } while ($LASTEXITCODE -ne 0)"

echo Docker ready.

echo Checking Ollama container...

FOR /F "tokens=*" %%i IN ('docker ps -a --format "{{.Names}}"') DO (
    if "%%i"=="ollama" set FOUND=1
)

if not defined FOUND (
    echo Creating container...
    docker run -d  --gpus all --name ollama -p 11434:11434 ollama/ollama
) else (
    echo Container exists, ensuring it's running...
    docker start ollama >nul 2>&1
)

echo Waiting for Ollama...
:waitloop
timeout /t 2 >nul
curl http://localhost:11434 >nul 2>&1
if errorlevel 1 goto waitloop

echo.
echo === STARTING CHAT ===
echo.

docker exec -it ollama ollama run qwen3:4b

echo.
echo === PROCESS EXITED ===
pause