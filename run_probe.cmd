@echo off
cd /d "%~dp0"
nexcopy_unlock.exe > probe-result.txt 2>&1
echo EXIT_CODE=%ERRORLEVEL%>> probe-result.txt

