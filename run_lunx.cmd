@echo off
cd /d "%~dp0"
nexcopy_unlock.exe --lunx > lunx-result.txt 2>&1
echo EXIT_CODE=%ERRORLEVEL%>> lunx-result.txt
type lunx-result.txt
echo.
pause
