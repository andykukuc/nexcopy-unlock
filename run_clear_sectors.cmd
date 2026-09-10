@echo off
cd /d "%~dp0"
nexcopy_unlock.exe --clear-sectors > clear-sectors-result.txt 2>&1
echo EXIT_CODE=%ERRORLEVEL%>> clear-sectors-result.txt
type clear-sectors-result.txt
echo.
pause
