@echo off
rem Generate NNUE training data (self-play).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\datagen.ps1" %*
echo.
pause
