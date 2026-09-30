@echo off
rem One-time setup: installs Rust, builds and tests Trinity.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\setup.ps1" %*
echo.
pause
