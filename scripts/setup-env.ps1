# HanamiRIP-CN Windows Environment Setup Script
# Run with PowerShell 5.1+
# 目的：一次性检查并安装本项目在 Windows 上的所有必需依赖。

# 遇到任何错误就直接抛出，避免脚本继续执行导致半安装状态。
$ErrorActionPreference = "Stop"

# 引入 Banner 脚本（同目录），用于在控制台输出项目 Logo。
. (Join-Path $PSScriptRoot "banner.ps1")

Write-Host "=== HanamiRIP-CN Windows Environment Setup ===" -ForegroundColor Cyan

# 检查当前 PowerShell 是否具备管理员权限（安装系统级工具时需要）。
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# 安装 Node.js 24（前端构建/工具链依赖）。
function Install-NodeJS {
    Write-Host '[CHECK] Node.js ...' -ForegroundColor Yellow
    
    # 标记变量：用于表达“是否已满足要求”，便于阅读流程。
    $nodeInstalled = $false
    try {
        $nodeVersion = node --version 2>$null
        if ($nodeVersion -match "^v(\d+)\.") {
            $majorVersion = [int]$matches[1]
            if ($majorVersion -eq 24) {
                Write-Host ('[OK] Node.js ' + $nodeVersion + ' installed') -ForegroundColor Green
                return
            } elseif ($majorVersion -lt 24) {
                Write-Host ('[INFO] Current Node.js version: ' + $nodeVersion + ', recommend upgrade to v24') -ForegroundColor Yellow
            } else {
                Write-Host ('[OK] Node.js ' + $nodeVersion + ' installed (newer than v24)') -ForegroundColor Green
                return
            }
        }
    } catch {
        Write-Host '[INFO] Node.js not detected' -ForegroundColor Yellow
    }
    
    # 依赖外部命令 winget：若不可用，会在 Main 中提前报错。
    Write-Host '[INSTALL] Node.js 24 LTS ...' -ForegroundColor Cyan
    Write-Host "Installing via winget, please wait..."
    
    try {
        winget install --id OpenJS.NodeJS.LTS -e --silent
        Write-Host '[OK] Node.js 24 installed successfully' -ForegroundColor Green
        
        # Refresh environment variables
        $env:PATH = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
    } catch {
        Write-Host ('[ERROR] Node.js installation failed: ' + $_) -ForegroundColor Red
        Write-Host "Please manually download and install Node.js 24 LTS from https://nodejs.org/" -ForegroundColor Yellow
        exit 1
    }
}

# 安装 Yarn（优先使用 corepack，失败则提示手动安装）。
function Install-Yarn {
    Write-Host '[CHECK] Yarn ...' -ForegroundColor Yellow
    
    try {
        $yarnVersion = yarn --version 2>$null
        Write-Host ('[OK] Yarn ' + $yarnVersion + ' installed') -ForegroundColor Green
        return
    } catch {
        Write-Host '[INFO] Yarn not detected' -ForegroundColor Yellow
    }
    
    Write-Host '[INSTALL] Yarn (via corepack) ...' -ForegroundColor Cyan
    
    try {
        # Try to enable corepack
        # 启用 corepack 需要管理员权限，失败时会提醒用户手动操作。
        if (Test-Administrator) {
            corepack enable
        } else {
            Write-Host '[INFO] Administrator privileges required to enable corepack' -ForegroundColor Yellow
            Write-Host "Please run PowerShell as Administrator and execute: corepack enable" -ForegroundColor Yellow
            Write-Host "Or manually install Yarn: npm install -g yarn" -ForegroundColor Yellow
        }
        
        # Check if successful
        $yarnVersion = yarn --version 2>$null
        Write-Host ('[OK] Yarn ' + $yarnVersion + ' installed') -ForegroundColor Green
    } catch {
        Write-Host '[WARNING] Yarn not installed, will use npm instead' -ForegroundColor Yellow
    }
}

# 安装 Rust 工具链（后端/构建依赖）。
function Install-Rust {
    Write-Host '[CHECK] Rust ...' -ForegroundColor Yellow
    
    try {
        $rustVersion = rustc --version 2>$null
        Write-Host ('[OK] Rust ' + $rustVersion + ' installed') -ForegroundColor Green
        return
    } catch {
        Write-Host '[INFO] Rust not detected' -ForegroundColor Yellow
    }
    
    Write-Host '[INSTALL] Rust toolchain ...' -ForegroundColor Cyan
    Write-Host "Installing via winget, please wait..."
    
    try {
        winget install --id Rustlang.Rustup -e --silent
        Write-Host '[OK] Rust installed successfully' -ForegroundColor Green
        
        # Refresh environment variables
        $env:PATH = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
        
        # Verify installation
        $rustVersion = rustc --version 2>$null
        Write-Host ('[OK] Rust ' + $rustVersion) -ForegroundColor Green
    } catch {
        Write-Host ('[ERROR] Rust installation failed: ' + $_) -ForegroundColor Red
        Write-Host "Please manually download and install Rust from https://rustup.rs/" -ForegroundColor Yellow
        exit 1
    }
}

# 安装 Rust 交叉目标（x64/x86），用于生成不同位数的 Windows 二进制。
function Install-RustTargets {
    Write-Host '[CHECK] Rust targets (x86_64, i686) ...' -ForegroundColor Yellow
    
    try {
        # Check if targets are installed
        $targets = rustup target list 2>$null
        $hasX64 = $targets | Where-Object { $_ -like "*x86_64-pc-windows-msvc*" -and $_ -like "*installed*" }
        $hasX86 = $targets | Where-Object { $_ -like "*i686-pc-windows-msvc*" -and $_ -like "*installed*" }
        
        if ($hasX64) {
            Write-Host '[OK] x86_64-pc-windows-msvc already installed' -ForegroundColor Green
        } else {
            Write-Host '[INSTALL] Adding x86_64-pc-windows-msvc target ...' -ForegroundColor Cyan
            rustup target add x86_64-pc-windows-msvc
            Write-Host '[OK] x86_64-pc-windows-msvc installed' -ForegroundColor Green
        }
        
        if ($hasX86) {
            Write-Host '[OK] i686-pc-windows-msvc already installed' -ForegroundColor Green
        } else {
            Write-Host '[INSTALL] Adding i686-pc-windows-msvc target (32-bit) ...' -ForegroundColor Cyan
            rustup target add i686-pc-windows-msvc
            Write-Host '[OK] i686-pc-windows-msvc installed' -ForegroundColor Green
        }
    } catch {
        Write-Host ('[ERROR] Rust target installation failed: ' + $_) -ForegroundColor Red
        Write-Host 'Manual install: rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc' -ForegroundColor Yellow
        exit 1
    }
}

# 确保 MSVC 链接器存在：Rust 在 Windows 上编译需要 link.exe。
function Ensure-MSVCForRust {
    Write-Host '[CHECK] MSVC linker (link.exe) for Rust ...' -ForegroundColor Yellow

    # 通过 Get-Command 查找可执行文件，避免硬编码路径。
    $linkExe = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($linkExe) {
        Write-Host ('[OK] link.exe found: ' + $linkExe.Source) -ForegroundColor Green
        return
    }

    # 在常见 Visual Studio/BuildTools 安装目录中寻找 link.exe。
    $vsBasePaths = @(
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Professional",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Enterprise",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise"
    )

    # 如果找到 link.exe，就把它所在目录加入用户 PATH。
    $msvcBinPath = $null
    foreach ($base in $vsBasePaths) {
        if (-not (Test-Path $base)) { continue }
        $vcTools = Join-Path $base "VC\Tools\MSVC"
        if (-not (Test-Path $vcTools)) { continue }
        $dirs = Get-ChildItem -Path $vcTools -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
        foreach ($d in $dirs) {
            $linkPath = Join-Path $d.FullName "bin\Hostx64\x64\link.exe"
            if (Test-Path $linkPath) {
                $msvcBinPath = (Split-Path $linkPath -Parent)
                break
            }
        }
        if ($msvcBinPath) { break }
    }

    if ($msvcBinPath) {
        $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$msvcBinPath*") {
            [System.Environment]::SetEnvironmentVariable("Path", ($userPath + ';' + $msvcBinPath), "User")
            Write-Host ('[OK] MSVC linker path added to user PATH: ' + $msvcBinPath) -ForegroundColor Green
        } else {
            Write-Host '[OK] MSVC linker dir already in user PATH' -ForegroundColor Green
        }
        $env:PATH = $env:PATH + ';' + $msvcBinPath
        return
    }

    # 如果未找到：管理员权限下尝试安装 Build Tools，否则给出手动安装提示。
    if (Test-Administrator) {
        Write-Host '[INSTALL] Installing Visual Studio Build Tools (C++) ...' -ForegroundColor Cyan
        try {
            winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended'
            Write-Host '[OK] Build Tools installed. Close and reopen terminal, then run this script again to add link.exe to PATH.' -ForegroundColor Green
            exit 0
        } catch {
            Write-Host '[ERROR] MSVC linker (link.exe) not found and Build Tools install failed.' -ForegroundColor Red
            Write-Host ('Install failed: ' + $_) -ForegroundColor Red
            Write-Host 'Install manually: https://visualstudio.microsoft.com/visual-cpp-build-tools/' -ForegroundColor Yellow
            Write-Host 'Setup will continue; other steps (deps, FFmpeg, etc.) are not affected.' -ForegroundColor Gray
        }
    } else {
        Write-Host '[ERROR] MSVC linker (link.exe) not found. Build Tools require administrator to install (no user-level install).' -ForegroundColor Red
        Write-Host 'Fix: Run PowerShell as Administrator, then run this script again, or run:' -ForegroundColor Yellow
        Write-Host '  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended"' -ForegroundColor White
        Write-Host 'Setup will continue; other steps (deps, FFmpeg, etc.) are not affected.' -ForegroundColor Gray
    }
}

# 确保 NSIS 打包工具可用（生成 Windows 安装包需要）。
function Ensure-NSIS {
    Write-Host '[CHECK] NSIS (makensis.exe) ...' -ForegroundColor Yellow

    # makensis.exe 是 NSIS 的核心命令，存在即代表已安装。
    $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
    if ($makensis) {
        Write-Host ('[OK] NSIS found: ' + $makensis.Source) -ForegroundColor Green
        return
    }

    # 尝试常见安装路径，若存在则添加到 PATH，避免要求重装。
    $nsisCandidates = @(
        "${env:ProgramFiles}\NSIS",
        "${env:ProgramFiles(x86)}\NSIS",
        "${env:ProgramFiles}\NSIS\Bin",
        "${env:ProgramFiles(x86)}\NSIS\Bin"
    )
    foreach ($dir in $nsisCandidates) {
        if (-not (Test-Path $dir)) { continue }
        $exePath = Join-Path $dir "makensis.exe"
        if (Test-Path $exePath) {
            $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
            if ($userPath -notlike "*$dir*") {
                [System.Environment]::SetEnvironmentVariable("Path", ($userPath + ';' + $dir), "User")
                Write-Host ('[OK] NSIS path added to user PATH: ' + $dir) -ForegroundColor Green
            }
            $env:PATH = $env:PATH + ';' + $dir
            $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
            if ($makensis) {
                Write-Host ('[OK] NSIS found: ' + $makensis.Source) -ForegroundColor Green
                return
            }
        }
    }

    Write-Host '[INSTALL] Installing NSIS ...' -ForegroundColor Cyan
    try {
        winget install -e --id NSIS.NSIS --silent
        $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
        if ($makensis) {
            Write-Host ('[OK] NSIS installed: ' + $makensis.Source) -ForegroundColor Green
            return
        }
        Write-Host '[WARNING] NSIS installed but makensis.exe not found in PATH. Please restart terminal.' -ForegroundColor Yellow
    } catch {
        Write-Host ('[ERROR] NSIS installation failed: ' + $_) -ForegroundColor Red
        Write-Host 'Install manually: https://nsis.sourceforge.io/Download' -ForegroundColor Yellow
    }
}

# 确保 tauri 打包所需的 NSIS 插件 DLL 存在，缺失则自动下载。
function Ensure-NSISTauriUtils {
    Write-Host '[CHECK] NSIS tauri utils (nsis_tauri_utils.dll) ...' -ForegroundColor Yellow

    # 固定指向官方发布地址，避免下载到不可信来源。
    $downloadUrl = "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll"
    $destDir = Join-Path $env:LOCALAPPDATA "tauri\NSIS\Plugins\x86-unicode\additional"
    $destPath = Join-Path $destDir "nsis_tauri_utils.dll"

    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    }

    if (Test-Path $destPath) {
        Write-Host ('[OK] nsis_tauri_utils.dll found: ' + $destPath) -ForegroundColor Green
        return
    }

    Write-Host '[INSTALL] Download nsis_tauri_utils.dll ...' -ForegroundColor Cyan

    # 简单重试机制，降低偶发网络失败的影响。
    $maxRetries = 3
    for ($i = 1; $i -le $maxRetries; $i++) {
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing
            if (Test-Path $destPath) {
                Write-Host "[OK] nsis_tauri_utils.dll downloaded to $destPath" -ForegroundColor Green
                return
            }
        } catch {
            Write-Host ("[WARN] Download failed (attempt {0}/{1}): {2}" -f $i, $maxRetries, $_) -ForegroundColor Yellow
            Start-Sleep -Seconds 2
        }
    }

    Write-Host "[ERROR] Failed to download nsis_tauri_utils.dll. Please download it manually:" -ForegroundColor Red
    Write-Host $downloadUrl -ForegroundColor Yellow
    Write-Host "Then place it at: $destPath" -ForegroundColor Yellow
}

# 安装前端/脚本依赖（优先 Yarn，其次 npm）。
function Install-ProjectDependencies {
    Write-Host '[INSTALL] Project dependencies ...' -ForegroundColor Cyan
    
    # 项目根目录 = scripts 的上级目录。
    $projectRoot = Split-Path -Parent $PSScriptRoot
    Set-Location $projectRoot
    
    try {
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            yarn install
        } else {
            npm install
        }
        Write-Host '[OK] Project dependencies installed successfully' -ForegroundColor Green
    } catch {
        Write-Host ('[ERROR] Dependencies installation failed: ' + $_) -ForegroundColor Red
        exit 1
    }
}

# 安装 FFmpeg 工具（用于媒体处理，打包到 public/tools）。
function Install-FFmpegTools {
    Write-Host '[CHECK] FFmpeg tools (ffmpeg/ffprobe) ...' -ForegroundColor Yellow

    $projectRoot = Split-Path -Parent $PSScriptRoot
    # 统一把工具放在 public/tools，应用运行时直接读取。
    $binDir = Join-Path $projectRoot "apps\desktop\public\tools"
    $ffmpegExe = Join-Path $binDir "ffmpeg.exe"
    $ffprobeExe = Join-Path $binDir "ffprobe.exe"

    if ((Test-Path $ffmpegExe) -and (Test-Path $ffprobeExe)) {
        Write-Host '[OK] FFmpeg already exists' -ForegroundColor Green
        return
    }

    Write-Host '[INSTALL] Download FFmpeg ...' -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $tempDir = Join-Path $env:TEMP "hanamirip-ffmpeg"
    if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    $zipPath = Join-Path $tempDir "ffmpeg.zip"
    # 下载官方 Windows 预编译版本，避免自行编译成本。
    $url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        $extracted = Get-ChildItem -Path $tempDir -Directory | Where-Object { $_.Name -like "ffmpeg-*" } | Select-Object -First 1
        if (-not $extracted) { throw "FFmpeg zip 解压失败" }

        Copy-Item (Join-Path $extracted.FullName "bin\ffmpeg.exe") -Destination $ffmpegExe -Force
        Copy-Item (Join-Path $extracted.FullName "bin\ffprobe.exe") -Destination $ffprobeExe -Force

        Write-Host '[OK] FFmpeg download done' -ForegroundColor Green
    } catch {
        Write-Host ('[ERROR] FFmpeg download failed: ' + $_) -ForegroundColor Red
        Write-Host 'Download manually and put into apps\desktop\public\tools' -ForegroundColor Yellow
        Write-Host ('URL: ' + $url) -ForegroundColor Yellow
    } finally {
        if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    }
}

# 安装 MKVToolNix（封装/信息读取所需）。
function Install-MkvToolNixTools {
    Write-Host '[CHECK] MKVToolNix tools (mkvmerge/mkvinfo) ...' -ForegroundColor Yellow

    $projectRoot = Split-Path -Parent $PSScriptRoot
    $binDir = Join-Path $projectRoot "apps\desktop\public\tools"
    $mkvmergeExe = Join-Path $binDir "mkvmerge.exe"
    $mkvinfoExe = Join-Path $binDir "mkvinfo.exe"

    if ((Test-Path $mkvmergeExe) -and (Test-Path $mkvinfoExe)) {
        Write-Host '[OK] MKVToolNix already exists' -ForegroundColor Green
        return
    }

    Write-Host '[INSTALL] Download MKVToolNix ...' -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    $tempDir = Join-Path $env:TEMP "hanamirip-mkvtoolnix"
    if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    $zipPath = Join-Path $tempDir "mkvtoolnix.zip"
    # 先访问版本索引页，再解析最新版本号。
    $indexUrl = "https://mkvtoolnix.download/windows/releases/"
    $url = $null
    try {
        $index = Invoke-WebRequest -Uri $indexUrl -UseBasicParsing
        $versions = @()
        foreach ($link in $index.Links) {
            $re = "windows/releases/(\d+(\.\d+)*)/"; if ($link.href -match $re) {
                $versions += $matches[1]
            }
        }

        if ($versions.Count -eq 0) { throw 'MKVToolNix version list parse failed' }

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

        if (-not $url) { throw 'MKVToolNix zip not found' }

        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        $mkvmergeFound = Get-ChildItem -Path $tempDir -Recurse -Filter 'mkvmerge.exe' | Select-Object -First 1
        $mkvinfoFound = Get-ChildItem -Path $tempDir -Recurse -Filter 'mkvinfo.exe' | Select-Object -First 1
        if (-not $mkvmergeFound -or -not $mkvinfoFound) { throw 'MKVToolNix zip extract failed' }

        Copy-Item $mkvmergeFound.FullName -Destination $mkvmergeExe -Force
        Copy-Item $mkvinfoFound.FullName -Destination $mkvinfoExe -Force

        Write-Host '[OK] MKVToolNix download done' -ForegroundColor Green
    } catch {
        Write-Host ('[ERROR] MKVToolNix download failed: ' + $_) -ForegroundColor Red
        Write-Host 'Download manually and put into apps\desktop\public\tools' -ForegroundColor Yellow
        if ($url) {
            Write-Host ('URL: ' + $url) -ForegroundColor Yellow
        } else {
            Write-Host ('Index: ' + $indexUrl) -ForegroundColor Yellow
        }
    } finally {
        if (Test-Path $tempDir) { Remove-Item -Recurse -Force $tempDir }
    }
}

# 生成应用图标（tauri icon 工具），只在缺失时执行。
function Generate-Icons {
    Write-Host '[CHECK] Application icons ...' -ForegroundColor Yellow
    
    $iconPath = Join-Path $PSScriptRoot '..\apps\desktop\public\icons\icon.ico'
    if (Test-Path $iconPath) {
        Write-Host '[OK] Icon file exists' -ForegroundColor Green
        return
    }
    
    Write-Host '[GENERATE] Application icons ...' -ForegroundColor Cyan
    try {
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            yarn tauri icon apps/desktop/public/icons/icon.png
        } else {
            npm run tauri icon apps/desktop/public/icons/icon.png
        }
        Write-Host '[OK] Icons generated successfully' -ForegroundColor Green
    } catch {
        Write-Host ('[WARNING] Icon generation failed: ' + $_) -ForegroundColor Yellow
    }
}

# 主流程：按顺序执行所有检查/安装步骤，保证依赖完整。
function Main {
    # 检查 winget 是否可用，后续安装步骤都依赖它。
    try {
        winget --version | Out-Null
    } catch {
        Write-Host '[ERROR] winget (Windows Package Manager) not detected' -ForegroundColor Red
        Write-Host 'Please ensure you are using Windows 10 1809+ or Windows 11' -ForegroundColor Yellow
        Write-Host 'Install App Installer from Microsoft Store' -ForegroundColor Yellow
        exit 1
    }
    
    # 依赖按顺序执行，避免后续步骤缺少前置工具。
    Install-NodeJS
    Install-Yarn
    Install-Rust
    Install-RustTargets
    Ensure-MSVCForRust
    Ensure-NSIS
    Ensure-NSISTauriUtils
    Install-ProjectDependencies
    Install-FFmpegTools
    Install-MkvToolNixTools
    Generate-Icons
    
    Write-Host "=== Environment Setup Complete! ===" -ForegroundColor Green
}

# Run main function
Main