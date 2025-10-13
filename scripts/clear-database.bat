@echo off
echo Clearing TrackEx Agent Database...
echo.

set "DB_PATH=%APPDATA%\TrackEx\agent.db"

if exist "%DB_PATH%" (
    echo Found database at: %DB_PATH%
    del "%DB_PATH%"
    if %ERRORLEVEL% EQU 0 (
        echo Database deleted successfully!
    ) else (
        echo Failed to delete database. Make sure the TrackEx agent is closed.
    )
) else (
    echo Database not found at: %DB_PATH%
    echo The database may not exist yet or is in a different location.
)

echo.
echo Database reset complete.
pause

