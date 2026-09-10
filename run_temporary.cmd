@echo off
cd /d "%~dp0"
nexcopy_unlock.exe --temporary > temporary-result.txt 2>&1
echo EXIT_CODE=%ERRORLEVEL%>> temporary-result.txt
type temporary-result.txt
echo.
pause
