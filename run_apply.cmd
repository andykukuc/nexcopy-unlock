@echo off
cd /d "%~dp0"
nexcopy_unlock.exe --apply > apply-result.txt 2>&1
echo EXIT_CODE=%ERRORLEVEL%>> apply-result.txt
type apply-result.txt
echo.
pause
