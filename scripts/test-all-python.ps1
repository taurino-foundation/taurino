$ErrorActionPreference = "Stop"

$versions = @("3.9", "3.10", "3.11", "3.12", "3.13", "3.14")

foreach ($version in $versions) {

    Write-Host "`n=================================" -ForegroundColor Cyan
    Write-Host "Starting Taurino - Python $version" -ForegroundColor Green
    Write-Host "=================================" -ForegroundColor Cyan

    $venvPath = ".venvs\python-$version"

    try {
        # 1. Create virtual environment
        uv venv $venvPath --python $version

        if ($LASTEXITCODE -ne 0) {
            throw "Environment creation failed: Python $version"
        }

        $env:UV_PROJECT_ENVIRONMENT = $venvPath
        $env:VIRTUAL_ENV = (Resolve-Path $venvPath).Path

        # 2. Install dependencies
        uv sync --python $version --no-install-project

        if ($LASTEXITCODE -ne 0) {
            throw "Dependency installation failed: Python $version"
        }

        # 3. Build Taurino with Maturin
        Write-Host "`nBuilding Taurino..." -ForegroundColor Yellow

        uvx --from "maturin>=1.14.1,<2.0" maturin develop --release --uv

        if ($LASTEXITCODE -ne 0) {
            throw "Maturin build failed: Python $version"
        }

        # 4. Run Taurino example
        Write-Host "`nRunning Taurino..." -ForegroundColor Yellow

        uv run --no-sync --python $version python examples/runtime.py

        if ($LASTEXITCODE -ne 0) {
            throw "Runtime failed: Python $version"
        }

        Write-Host "`nPython $version completed successfully!" -ForegroundColor Green
    }
    finally {
        # 5. Deactivate environment variables
        Remove-Item Env:\VIRTUAL_ENV -ErrorAction SilentlyContinue
        Remove-Item Env:\UV_PROJECT_ENVIRONMENT -ErrorAction SilentlyContinue

        # 6. Delete virtual environment
        if (Test-Path $venvPath) {
            Write-Host "`nDeleting environment: $venvPath" -ForegroundColor Yellow

            Remove-Item -LiteralPath $venvPath -Recurse -Force

            Write-Host "Environment deleted successfully!" -ForegroundColor Green
        }
    }
}

Write-Host "`nAll Python versions completed successfully!" -ForegroundColor Green



 #  .\scripts\test-all-python.ps1
