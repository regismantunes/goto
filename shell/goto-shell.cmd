@echo off
setlocal DisableDelayedExpansion
set "GOTO_FIRST=%~1"

if "%~1"=="" goto manage
if /I "%~1"=="-s" goto manage
if /I "%~1"=="--save" goto manage
if /I "%~1"=="-l" goto manage
if /I "%~1"=="--list" goto manage
if /I "%~1"=="-r" goto manage
if /I "%~1"=="--remove" goto manage
if /I "%~1"=="save" goto manage
if /I "%~1"=="list" goto manage
if /I "%~1"=="remove" goto manage
if /I "%~1"=="init" goto manage
if /I "%~1"=="-h" goto manage
if /I "%~1"=="--help" goto manage
if /I "%~1"=="-V" goto manage
if /I "%~1"=="--version" goto manage
if "%GOTO_FIRST:~0,1%"=="-" goto escaped
goto resolve

:manage
"%~dp0goto.exe" %*
exit /b %ERRORLEVEL%

:escaped
if /I not "%~1"=="--" goto manage
if "%~2"=="" goto manage
if not "%~3"=="" goto manage
set "GOTO_KEY=%~2"
goto resolve_key

:resolve
if not "%~2"=="" (
    "%~dp0goto.exe" __resolve -- %*
    exit /b %ERRORLEVEL%
)
set "GOTO_KEY=%~1"

:resolve_key
set "GOTO_TARGET="
for /f "usebackq delims=" %%D in (`""%~dp0goto.exe" __resolve -- "%GOTO_KEY%""`) do set "GOTO_TARGET=%%D"
if not defined GOTO_TARGET exit /b 1
endlocal & cd /d "%GOTO_TARGET%"
