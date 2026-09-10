@echo off
setlocal
set "TESTFILE=I:\__nexcopy_write_test.tmp"
if exist "%TESTFILE%" del /f /q "%TESTFILE%" >nul 2>&1
> "%TESTFILE%" echo Nexcopy write test %DATE% %TIME%
if exist "%TESTFILE%" (
  echo WRITE_SUCCEEDED=Yes
  echo WRITE_SUCCEEDED=Yes> "%~dp0verify-result.txt"
  del /f /q "%TESTFILE%"
  if exist "%TESTFILE%" (
    echo DELETE_SUCCEEDED=No
    echo DELETE_SUCCEEDED=No>> "%~dp0verify-result.txt"
  ) else (
    echo DELETE_SUCCEEDED=Yes
    echo DELETE_SUCCEEDED=Yes>> "%~dp0verify-result.txt"
  )
 ) else (
  echo WRITE_SUCCEEDED=No
  echo WRITE_SUCCEEDED=No> "%~dp0verify-result.txt"
)
echo.
pause
