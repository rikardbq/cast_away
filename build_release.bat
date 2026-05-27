@echo off & setlocal

set CAST_AWAY_DIR=%APPDATA%\CastAway
call :ResolvePath TARGET_DIR %~dpn0\..\target\release

for /F "tokens=1-9 delims= " %%a in ("%*") do (
    if %%a==-rd (
        set CAST_AWAY_DIR=%%~fb
    )
)

echo Starting build
set FFMPEG_SIDECAR=0
cargo build --release

dir /A:D %CAST_AWAY_DIR% >nul 2>&1 & if ERRORLEVEL 1 (
    mkdir %CAST_AWAY_DIR%
)
echo %TARGET_DIR%
echo %CAST_AWAY_DIR%
copy %TARGET_DIR%\*.exe %CAST_AWAY_DIR%
xcopy /s /e /h /i .\assets %CAST_AWAY_DIR%\assets

@REM echo Cleaning up...
@REM cargo clean -vv --release
echo Done!
exit /b

:ResolvePath
    set %1=%~f2
    exit /b
