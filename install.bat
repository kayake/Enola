@echo off
SETLOCAL ENABLEDELAYEDEXPANSION

SET BIN_NAME=enola.exe
SET BIN_INSTALL_DIR=%ProgramFiles%\Enola
SET DATA_DIR=%LOCALAPPDATA%\Enola
SET PROJECT_ROOT=%~dp0

IF "%PROJECT_ROOT:~-1%"=="\" SET PROJECT_ROOT=%PROJECT_ROOT:~0,-1%

where cargo >nul 2>nul
IF %ERRORLEVEL% NEQ 0 (
    echo Cargo not found. Installing Rust + Cargo...
    powershell -Command "Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile %TEMP%\rustup-init.exe"
    "%TEMP%\rustup-init.exe" -y
    SET PATH=%USERPROFILE%\.cargo\bin;%PATH%
)

IF NOT EXIST "%PROJECT_ROOT%\target\release\%BIN_NAME%" (
    echo Compiling %BIN_NAME%...
    pushd "%PROJECT_ROOT%"
    cargo build --release
    IF %ERRORLEVEL% NEQ 0 (
        echo Build failed!
        popd
        exit /b 1
    )
    popd
)

if not exist "%BIN_INSTALL_DIR%" mkdir "%BIN_INSTALL_DIR%"

copy /Y "%PROJECT_ROOT%\target\release\%BIN_NAME%" "%BIN_INSTALL_DIR%\%BIN_NAME%" >nul

if not exist "%DATA_DIR%" mkdir "%DATA_DIR%"

IF EXIST "%PROJECT_ROOT%\src\utils\" (
    xcopy "%PROJECT_ROOT%\src\utils\*" "%DATA_DIR%\" /E /I /Y >nul
)

echo.
echo %BIN_NAME% installed successfully!
echo Binary location: %BIN_INSTALL_DIR%\%BIN_NAME%
echo Data location: %DATA_DIR%
echo.
echo You can run Enola from any terminal.
pause
ENDLOCAL
