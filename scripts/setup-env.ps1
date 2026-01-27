# HanamiRIP-CN Windows Environment Setup Script
# Run with PowerShell 5.1+

$ErrorActionPreference = "Stop"

Write-Host "=== HanamiRIP-CN Windows Environment Setup ===" -ForegroundColor Cyan
Write-Host ""

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Install Node.js 24
function Install-NodeJS {
    Write-Host "[CHECK] Node.js ..." -ForegroundColor Yellow
    
    $nodeInstalled = $false
    try {
        $nodeVersion = node --version 2>$null
        if ($nodeVersion -match "^v(\d+)\.") {
            $majorVersion = [int]$matches[1]
            if ($majorVersion -eq 24) {
                Write-Host "[OK] Node.js $nodeVersion installed" -ForegroundColor Green
                return
            } elseif ($majorVersion -lt 24) {
                Write-Host "[INFO] Current Node.js version: $nodeVersion, recommend upgrade to v24" -ForegroundColor Yellow
            } else {
                Write-Host "[OK] Node.js $nodeVersion installed (newer than v24)" -ForegroundColor Green
                return
            }
        }
    } catch {
        Write-Host "[INFO] Node.js not detected" -ForegroundColor Yellow
    }
    
    Write-Host "[INSTALL] Node.js 24 LTS ..." -ForegroundColor Cyan
    Write-Host "Installing via winget, please wait..."
    
    try {
        winget install --id OpenJS.NodeJS.LTS -e --silent
        Write-Host "[OK] Node.js 24 installed successfully" -ForegroundColor Green
        
        # Refresh environment variables
        $env:PATH = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
    } catch {
        Write-Host "[ERROR] Node.js installation failed: $_" -ForegroundColor Red
        Write-Host "Please manually download and install Node.js 24 LTS from https://nodejs.org/" -ForegroundColor Yellow
        exit 1
    }
}

# Install Yarn
function Install-Yarn {
    Write-Host "[CHECK] Yarn ..." -ForegroundColor Yellow
    
    try {
        $yarnVersion = yarn --version 2>$null
        Write-Host "[OK] Yarn $yarnVersion installed" -ForegroundColor Green
        return
    } catch {
        Write-Host "[INFO] Yarn not detected" -ForegroundColor Yellow
    }
    
    Write-Host "[INSTALL] Yarn (via corepack) ..." -ForegroundColor Cyan
    
    try {
        # Try to enable corepack
        if (Test-Administrator) {
            corepack enable
        } else {
            Write-Host "[INFO] Administrator privileges required to enable corepack" -ForegroundColor Yellow
            Write-Host "Please run PowerShell as Administrator and execute: corepack enable" -ForegroundColor Yellow
            Write-Host "Or manually install Yarn: npm install -g yarn" -ForegroundColor Yellow
        }
        
        # Check if successful
        $yarnVersion = yarn --version 2>$null
        Write-Host "[OK] Yarn $yarnVersion installed" -ForegroundColor Green
    } catch {
        Write-Host "[WARNING] Yarn not installed, will use npm instead" -ForegroundColor Yellow
    }
}

# Install Rust
function Install-Rust {
    Write-Host "[CHECK] Rust ..." -ForegroundColor Yellow
    
    try {
        $rustVersion = rustc --version 2>$null
        Write-Host "[OK] Rust $rustVersion installed" -ForegroundColor Green
        return
    } catch {
        Write-Host "[INFO] Rust not detected" -ForegroundColor Yellow
    }
    
    Write-Host "[INSTALL] Rust toolchain ..." -ForegroundColor Cyan
    Write-Host "Installing via winget, please wait..."
    
    try {
        winget install --id Rustlang.Rustup -e --silent
        Write-Host "[OK] Rust installed successfully" -ForegroundColor Green
        
        # Refresh environment variables
        $env:PATH = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
        
        # Verify installation
        $rustVersion = rustc --version 2>$null
        Write-Host "[OK] Rust $rustVersion" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] Rust installation failed: $_" -ForegroundColor Red
        Write-Host "Please manually download and install Rust from https://rustup.rs/" -ForegroundColor Yellow
        exit 1
    }
}

# Install project dependencies
function Install-ProjectDependencies {
    Write-Host "[INSTALL] Project dependencies ..." -ForegroundColor Cyan
    
    $projectRoot = Split-Path -Parent $PSScriptRoot
    Set-Location $projectRoot
    
    try {
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            yarn install
        } else {
            npm install
        }
        Write-Host "[OK] Project dependencies installed successfully" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] Dependencies installation failed: $_" -ForegroundColor Red
        exit 1
    }
}

# Install bundled FFmpeg tools (ffmpeg/ffprobe)
function Install-FFmpegTools {
    Write-Host "[CHECK] FFmpeg tools (ffmpeg/ffprobe) ..." -ForegroundColor Yellow

    $projectRoot = Split-Path -Parent $PSScriptRoot
    $binDir = Join-Path $projectRoot "src-tauri\bin"
    $ffmpegExe = Join-Path $binDir "ffmpeg.exe"
    $ffprobeExe = Join-Path $binDir "ffprobe.exe"

    if ((Test-Path $ffmpegExe) -and (Test-Path $ffprobeExe)) {
        Write-Host "[OK] 内置 FFmpeg 已存在" -ForegroundColor Green
        return
    }

    Write-Host "[INSTALL] 下载内置 FFmpeg ..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $tempDir = Join-Path $env:TEMP "hanamirip-ffmpeg"
    if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    $zipPath = Join-Path $tempDir "ffmpeg.zip"
    $url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        $extracted = Get-ChildItem -Path $tempDir -Directory | Where-Object { $_.Name -like "ffmpeg-*" } | Select-Object -First 1
        if (-not $extracted) { throw "FFmpeg zip 解压失败" }

        Copy-Item (Join-Path $extracted.FullName "bin\ffmpeg.exe") -Destination $ffmpegExe -Force
        Copy-Item (Join-Path $extracted.FullName "bin\ffprobe.exe") -Destination $ffprobeExe -Force

        Write-Host "[OK] 内置 FFmpeg 下载完成" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] 下载 FFmpeg 失败: $_" -ForegroundColor Red
        Write-Host "请手动下载并放入 src-tauri\\bin" -ForegroundColor Yellow
        Write-Host "下载地址：$url" -ForegroundColor Yellow
    } finally {
        if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    }
}

function Install-MkvToolNixTools {
    Write-Host "[CHECK] MKVToolNix tools (mkvmerge/mkvinfo) ..." -ForegroundColor Yellow

    $projectRoot = Split-Path -Parent $PSScriptRoot
    $binDir = Join-Path $projectRoot "src-tauri\bin"
    $mkvmergeExe = Join-Path $binDir "mkvmerge.exe"
    $mkvinfoExe = Join-Path $binDir "mkvinfo.exe"

    if ((Test-Path $mkvmergeExe) -and (Test-Path $mkvinfoExe)) {
        Write-Host "[OK] 内置 MKVToolNix 已存在" -ForegroundColor Green
        return
    }

    Write-Host "[INSTALL] 下载内置 MKVToolNix ..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $tempDir = Join-Path $env:TEMP "hanamirip-mkvtoolnix"
    if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    $zipPath = Join-Path $tempDir "mkvtoolnix.zip"
    $indexUrl = "https://mkvtoolnix.download/windows/releases/"
    $url = $null
    try {
        $index = Invoke-WebRequest -Uri $indexUrl -UseBasicParsing
        $versions = @()
        foreach ($link in $index.Links) {
            if ($link.href -match "/windows/releases/([0-9]+(\.[0-9]+)*)/") {
                $versions += $matches[1]
            }
        }

        if ($versions.Count -eq 0) { throw "无法解析 MKVToolNix 版本列表" }

        $latest = $versions | Sort-Object { [version]$_ } -Descending | Select-Object -First 1
        $baseUrl = "https://mkvtoolnix.download/windows/releases/$latest/"
        $candidates = @(
            "mkvtoolnix-64-bit-$latest.zip",
            "mkvtoolnix-64-bit-$latest.0.zip",
            "mkvtoolnix-64-bit-$latest.0.0.zip"
        )

        foreach ($name in $candidates) {
            $candidateUrl = $baseUrl + $name
            try {
                Invoke-WebRequest -Uri $candidateUrl -OutFile $zipPath -UseBasicParsing
                $url = $candidateUrl
                break
            } catch {
                $url = $null
            }
        }

        if (-not $url) { throw "未找到可用的 MKVToolNix zip 包" }

        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        $mkvmergeFound = Get-ChildItem -Path $tempDir -Recurse -Filter "mkvmerge.exe" | Select-Object -First 1
        $mkvinfoFound = Get-ChildItem -Path $tempDir -Recurse -Filter "mkvinfo.exe" | Select-Object -First 1
        if (-not $mkvmergeFound -or -not $mkvinfoFound) { throw "MKVToolNix zip 解压失败" }

        Copy-Item $mkvmergeFound.FullName -Destination $mkvmergeExe -Force
        Copy-Item $mkvinfoFound.FullName -Destination $mkvinfoExe -Force

        Write-Host "[OK] 内置 MKVToolNix 下载完成" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] 下载 MKVToolNix 失败: $_" -ForegroundColor Red
        Write-Host "请手动下载并放入 src-tauri\\bin" -ForegroundColor Yellow
        if ($url) {
            Write-Host "下载地址：$url" -ForegroundColor Yellow
        } else {
            Write-Host "下载列表：$indexUrl" -ForegroundColor Yellow
        }
    } finally {
        if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    }
}

# Generate icon files
function Generate-Icons {
    Write-Host "[CHECK] Application icons ..." -ForegroundColor Yellow
    
    $iconPath = Join-Path $PSScriptRoot "..\src-tauri\icons\icon.ico"
    if (Test-Path $iconPath) {
        Write-Host "[OK] Icon file exists" -ForegroundColor Green
        return
    }
    
    Write-Host "[GENERATE] Application icons ..." -ForegroundColor Cyan
    try {
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            yarn tauri icon src-tauri/icons/icon.png
        } else {
            npm run tauri icon src-tauri/icons/icon.png
        }
        Write-Host "[OK] Icons generated successfully" -ForegroundColor Green
    } catch {
        Write-Host "[WARNING] Icon generation failed, will auto-generate on first build" -ForegroundColor Yellow
    }
}

# Main function
function Main {
    Write-Host ""
    
    # Check winget availability
    try {
        winget --version | Out-Null
    } catch {
        Write-Host "[ERROR] winget (Windows Package Manager) not detected" -ForegroundColor Red
        Write-Host "Please ensure you are using Windows 10 1809+ or Windows 11" -ForegroundColor Yellow
        Write-Host "Install 'App Installer' from Microsoft Store" -ForegroundColor Yellow
        exit 1
    }
    
    Install-NodeJS
    Install-Yarn
    Install-Rust
    Install-ProjectDependencies
    Install-FFmpegTools
    Install-MkvToolNixTools
    Generate-Icons
    
    Write-Host ""
    Write-Host "=== Environment Setup Complete! ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "Available commands:" -ForegroundColor Cyan
    Write-Host "  yarn tauri dev    # Start Tauri desktop app in dev mode" -ForegroundColor White
    Write-Host "  yarn dev          # Start Vite frontend server only" -ForegroundColor White
    Write-Host "  yarn build        # Build desktop application" -ForegroundColor White
    Write-Host ""
    Write-Host "To enable Baidu Translation feature, configure environment variables:" -ForegroundColor Yellow
    Write-Host "  `$env:BAIDU_TRANSLATE_APP_ID = ''your-app-id''" -ForegroundColor White
    Write-Host "  `$env:BAIDU_TRANSLATE_API_KEY = ''your-api-key''" -ForegroundColor White
    Write-Host "  cd src-tauri\baidu_verify" -ForegroundColor White
    Write-Host "  cargo build --release" -ForegroundColor White
    Write-Host ""
}

# Run main function
Main