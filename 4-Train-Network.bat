@echo off
rem Train a new network on the GPU and publish it as a test branch.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\train.ps1" %*
echo.
pause
