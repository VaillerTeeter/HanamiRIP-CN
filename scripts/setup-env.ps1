# 文件：setup-env.ps1 | 用途：一键检查并安装 HanamiRIP-CN 在 Windows 上的开发与打包依赖 | 关键命令：winget、rustup、yarn/npm、Invoke-WebRequest、Write-Host
# 说明：本脚本通过分步骤函数执行环境准备，遇到关键失败会中止，非关键失败会给出手动修复提示。

# 变量：$ErrorActionPreference | 含义：设置 PowerShell 遇错后的默认处理策略 | 类型：String | 作用域：脚本全局
# 将默认错误策略设置为 Stop，确保未捕获错误会抛出异常并进入 catch。
$ErrorActionPreference = "Stop"

# 导入横幅脚本，用于在执行开始时输出项目标识。
.(Join-Path $PSScriptRoot "banner.ps1")

# 打印环境配置启动提示，方便区分日志阶段。
Write-Host "=== HanamiRIP-CN Windows Environment Setup ===" -ForegroundColor Cyan

# 函数：Test-Administrator | 输入：无 | 输出：布尔值（当前进程是否管理员） | 可能失败：Windows 身份对象创建异常
# 检测当前 PowerShell 会话是否拥有管理员权限。
function Test-Administrator {
    # 变量：$currentUser | 含义：当前登录用户的 Windows 身份对象 | 类型：WindowsIdentity | 作用域：函数内部
    # 读取当前线程关联的 Windows 身份，用于权限判断。
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    # 变量：$principal | 含义：基于当前身份构建的权限主体对象 | 类型：WindowsPrincipal | 作用域：函数内部
    # 使用主体对象检查是否属于系统管理员角色。
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    # 返回是否属于 Administrator 内置角色。
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# 函数：Install-NodeJS | 输入：无 | 输出：无（副作用：检查/安装 Node.js 并刷新 PATH） | 可能失败：winget 不可用、安装失败、网络中断
# 确保本机具备可用的 Node.js（优先 v24 LTS）。
function Install-NodeJS {
    # 输出 Node.js 检查阶段提示。
    Write-Host '[CHECK] Node.js ...' -ForegroundColor Yellow

    # 变量：$nodeInstalled | 含义：预留的安装状态标记 | 类型：Boolean | 作用域：函数内部
    # 先初始化标记，便于后续扩展条件控制。
    $nodeInstalled = $false
    # 尝试读取本机 node 版本。
    try {
        # 变量：$nodeVersion | 含义：node --version 的输出值 | 类型：String | 作用域：try 块内部
        # 执行 node 版本命令并抑制标准错误输出。
        $nodeVersion = node --version 2>$null
        # 通过正则匹配主版本号，格式例如 v24.1.0。
        if ($nodeVersion -match "^v(\d+)\.") {
            # 变量：$majorVersion | 含义：Node.js 主版本号 | 类型：Int32 | 作用域：if 块内部
            # 将匹配结果中的主版本转换为整数，便于比较。
            $majorVersion = [int] $matches[1]
            # 主版本等于 24，视为满足推荐版本。
            if ($majorVersion -eq 24) {
                # 输出已满足版本提示并结束函数。
                Write-Host ('[OK] Node.js ' + $nodeVersion + ' installed') -ForegroundColor Green
                # 直接返回，不执行安装流程。
                return
            # 主版本小于 24，提示建议升级但继续走安装分支。
            } elseif ($majorVersion -lt 24) {
                # 输出当前版本与升级建议。
                Write-Host (
                    '[INFO] Current Node.js version: ' + $nodeVersion + ', recommend upgrade to v24'
                ) -ForegroundColor Yellow
            # 主版本大于 24，视为兼容并直接使用。
            } else {
                # 输出可用版本提示并结束函数。
                Write-Host ('[OK] Node.js ' + $nodeVersion + ' installed (newer than v24)') -ForegroundColor Green
                # 直接返回，跳过安装。
                return
            }
        }
    # 捕获 node 命令不存在或执行失败的异常。
    } catch {
        # 输出未检测到 Node.js 的提示。
        Write-Host '[INFO] Node.js not detected' -ForegroundColor Yellow
    }

    # 输出开始安装 Node.js 的阶段提示。
    Write-Host '[INSTALL] Node.js 24 LTS ...' -ForegroundColor Cyan
    # 提示安装过程可能持续一段时间。
    Write-Host "Installing via winget, please wait..."

    # 尝试使用 winget 执行静默安装。
    try {
        # 使用 OpenJS 官方 LTS 包 ID 进行安装。
        winget install --id OpenJS.NodeJS.LTS -e --silent
        # 输出安装成功提示。
        Write-Host '[OK] Node.js 24 installed successfully' -ForegroundColor Green

        # 变量：$env:PATH | 含义：当前进程的 PATH 环境变量 | 类型：String | 作用域：进程级
        # 重新拼接用户与系统 PATH，尽量让新安装的 node 在当前会话立即可用。
        $env:PATH = [System.Environment]::GetEnvironmentVariable(
            "Path",
            "User"
        ) + ";" + [System.Environment]::GetEnvironmentVariable( "Path", "Machine" )
    # 捕获安装失败并给出手动安装指引。
    } catch {
        # 输出安装失败详情。
        Write-Host ('[ERROR] Node.js installation failed: ' + $_) -ForegroundColor Red
        # 提示用户手动下载安装地址。
        Write-Host "Please manually download and install Node.js 24 LTS from https://nodejs.org/" -ForegroundColor Yellow
        # 关键依赖失败，退出脚本。
        exit 1
    }
}

# 函数：Install-Yarn | 输入：无 | 输出：无（副作用：尝试启用 corepack 并提供 yarn） | 可能失败：无管理员权限启用 corepack、网络失败、yarn 命令不可用
# 检查并安装 Yarn；优先通过 Node.js 自带的 corepack 启用。
function Install-Yarn {
    # 输出 Yarn 检查提示。
    Write-Host '[CHECK] Yarn ...' -ForegroundColor Yellow

    # 尝试读取当前 Yarn 版本，存在则直接复用。
    try {
        # 变量：$yarnVersion | 含义：yarn --version 输出 | 类型：String | 作用域：try 块内部
        # 调用 yarn 版本命令并抑制标准错误。
        $yarnVersion = yarn --version 2>$null
        # 输出已安装版本。
        Write-Host ('[OK] Yarn ' + $yarnVersion + ' installed') -ForegroundColor Green
        # 已满足条件，结束函数。
        return
    # 捕获未安装或命令不可用异常。
    } catch {
        # 输出未检测到 Yarn 的提示。
        Write-Host '[INFO] Yarn not detected' -ForegroundColor Yellow
    }

    # 输出开始安装 Yarn 的提示。
    Write-Host '[INSTALL] Yarn (via corepack) ...' -ForegroundColor Cyan

    # 进入安装尝试流程。
    try {
        # 检查是否管理员权限（某些环境下 corepack enable 需要管理员）。
        if (Test-Administrator) {
            # 启用 corepack 以管理 Yarn 发行版本。
            corepack enable
        # 非管理员场景下给出手动处理提示。
        } else {
            # 输出权限不足说明。
            Write-Host '[INFO] Administrator privileges required to enable corepack' -ForegroundColor Yellow
            # 指导用户用管理员终端执行 corepack enable。
            Write-Host "Please run PowerShell as Administrator and execute: corepack enable" -ForegroundColor Yellow
            # 给出备选安装方式。
            Write-Host "Or manually install Yarn: npm install -g yarn" -ForegroundColor Yellow
        }

        # 变量：$yarnVersion | 含义：安装后再次检测的 Yarn 版本 | 类型：String | 作用域：try 块内部
        # 再次读取版本确认 Yarn 是否可用。
        $yarnVersion = yarn --version 2>$null
        # 输出可用版本。
        Write-Host ('[OK] Yarn ' + $yarnVersion + ' installed') -ForegroundColor Green
    # 捕获启用或检测失败，降级为 npm。
    } catch {
        # 输出警告，不中断脚本。
        Write-Host '[WARNING] Yarn not installed, will use npm instead' -ForegroundColor Yellow
    }
}

# 函数：Install-Rust | 输入：无 | 输出：无（副作用：检查/安装 Rust 并刷新 PATH） | 可能失败：winget 安装失败、网络中断
# 确保 Rust 工具链已安装，供 Tauri 与后端构建使用。
function Install-Rust {
    # 输出 Rust 检查提示。
    Write-Host '[CHECK] Rust ...' -ForegroundColor Yellow

    # 尝试读取 rustc 版本，存在则直接返回。
    try {
        # 变量：$rustVersion | 含义：rustc --version 输出值 | 类型：String | 作用域：try 块内部
        # 执行 rustc 版本查询。
        $rustVersion = rustc --version 2>$null
        # 输出已安装版本信息。
        Write-Host ('[OK] Rust ' + $rustVersion + ' installed') -ForegroundColor Green
        # Rust 可用，结束函数。
        return
    # 捕获 rustc 不存在等异常。
    } catch {
        # 输出未检测到 Rust 的提示。
        Write-Host '[INFO] Rust not detected' -ForegroundColor Yellow
    }

    # 输出 Rust 安装阶段提示。
    Write-Host '[INSTALL] Rust toolchain ...' -ForegroundColor Cyan
    # 提示安装过程等待。
    Write-Host "Installing via winget, please wait..."

    # 尝试通过 winget 安装 rustup。
    try {
        # 安装 Rust 官方工具链管理器 rustup。
        winget install --id Rustlang.Rustup -e --silent
        # 输出安装成功提示。
        Write-Host '[OK] Rust installed successfully' -ForegroundColor Green

        # 变量：$env:PATH | 含义：当前会话 PATH | 类型：String | 作用域：进程级
        # 更新当前进程 PATH，尽量无需重开终端即可使用 rustc/rustup。
        $env:PATH = [System.Environment]::GetEnvironmentVariable(
            "Path",
            "User"
        ) + ";" + [System.Environment]::GetEnvironmentVariable( "Path", "Machine" )

        # 变量：$rustVersion | 含义：安装后的 rustc 版本 | 类型：String | 作用域：try 块内部
        # 再次读取版本确认安装成功。
        $rustVersion = rustc --version 2>$null
        # 输出安装后的版本信息。
        Write-Host ('[OK] Rust ' + $rustVersion) -ForegroundColor Green
    # 捕获安装失败并终止脚本。
    } catch {
        # 输出失败详情。
        Write-Host ('[ERROR] Rust installation failed: ' + $_) -ForegroundColor Red
        # 提供手动安装地址。
        Write-Host "Please manually download and install Rust from https://rustup.rs/" -ForegroundColor Yellow
        # Rust 为关键依赖，失败时退出。
        exit 1
    }
}

# 函数：Install-RustTargets | 输入：无 | 输出：无（副作用：安装 Rust 目标三元组） | 可能失败：rustup 未安装、网络失败
# 确保同时具备 64 位和 32 位 Windows MSVC 目标，以支持多架构构建。
function Install-RustTargets {
    # 输出目标架构检查提示。
    Write-Host '[CHECK] Rust targets (x86_64, i686) ...' -ForegroundColor Yellow

    # 尝试读取当前已安装目标列表。
    try {
        # 变量：$targets | 含义：rustup target list 返回的目标列表 | 类型：String[] | 作用域：try 块内部
        # 获取所有目标状态文本。
        $targets = rustup target list 2>$null
        # 变量：$hasX64 | 含义：是否已安装 x86_64 目标 | 类型：Object/Boolean 语义 | 作用域：try 块内部
        # 通过筛选包含 installed 的目标行判断 64 位目标状态。
        $hasX64 = $targets
            | Where-Object {
                $_ -like "*x86_64-pc-windows-msvc*" -and $_ -like "*installed*"
            }
        # 变量：$hasX86 | 含义：是否已安装 i686 目标 | 类型：Object/Boolean 语义 | 作用域：try 块内部
        # 判断 32 位目标是否已安装。
        $hasX86 = $targets
            | Where-Object {
                $_ -like "*i686-pc-windows-msvc*" -and $_ -like "*installed*"
            }

        # 已安装 64 位目标则直接提示。
        if ($hasX64) {
            # 输出 64 位目标已存在提示。
            Write-Host '[OK] x86_64-pc-windows-msvc already installed' -ForegroundColor Green
        # 未安装则执行新增。
        } else {
            # 输出安装 64 位目标提示。
            Write-Host '[INSTALL] Adding x86_64-pc-windows-msvc target ...' -ForegroundColor Cyan
            # 通过 rustup 添加 64 位目标。
            rustup target add x86_64-pc-windows-msvc
            # 输出安装完成提示。
            Write-Host '[OK] x86_64-pc-windows-msvc installed' -ForegroundColor Green
        }

        # 已安装 32 位目标则直接提示。
        if ($hasX86) {
            # 输出 32 位目标已存在提示。
            Write-Host '[OK] i686-pc-windows-msvc already installed' -ForegroundColor Green
        # 未安装则执行新增。
        } else {
            # 输出安装 32 位目标提示。
            Write-Host '[INSTALL] Adding i686-pc-windows-msvc target (32-bit) ...' -ForegroundColor Cyan
            # 通过 rustup 添加 32 位目标。
            rustup target add i686-pc-windows-msvc
            # 输出安装完成提示。
            Write-Host '[OK] i686-pc-windows-msvc installed' -ForegroundColor Green
        }
    # 捕获目标安装异常并终止脚本。
    } catch {
        # 输出失败详情。
        Write-Host ('[ERROR] Rust target installation failed: ' + $_) -ForegroundColor Red
        # 提供手动添加目标命令。
        Write-Host 'Manual install: rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc' -ForegroundColor Yellow
        # 目标缺失会影响构建，退出脚本。
        exit 1
    }
}

# 函数：Ensure-MSVCForRust | 输入：无 | 输出：无(副作用：添加link.exe到PATH或安装Build Tools) | 可能失败：VS未安装、权限不足、winget失败
# 确保Rust能找到MSVC链接器(link.exe)，用于编译Windows原生程序
# Rust在Windows上需要MSVC工具链来链接生成的目标文件
function Ensure-MSVCForRust {
    # 输出检查提示，黄色字体
    Write-Host '[CHECK] MSVC linker (link.exe) for Rust ...' -ForegroundColor Yellow

    # 变量：$linkExe | 含义：link.exe命令的路径对象 | 类型：CommandInfo或$null | 作用域：函数内部
    # 尝试在PATH中查找link.exe，-ErrorAction SilentlyContinue表示找不到时不报错
    $linkExe = Get-Command link.exe -ErrorAction SilentlyContinue
    # 检查是否找到link.exe
    if ($linkExe) {
        # 输出找到信息，绿色字体，显示link.exe的完整路径
        Write-Host ('[OK] link.exe found: ' + $linkExe.Source) -ForegroundColor Green
        # 已找到，直接返回
        return
    }

    # 变量：$vsBasePaths | 含义：Visual Studio可能安装的基础路径列表 | 类型：字符串数组 | 作用域：函数内部
    # 定义Visual Studio 2022的各个版本可能的安装路径
    # BuildTools是仅命令行工具，Community/Professional/Enterprise是完整IDE
    # 同时检查Program Files (x86)和Program Files两个位置
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

    # 变量：$msvcBinPath | 含义：MSVC工具链bin目录的路径 | 类型：字符串或$null | 作用域：函数内部
    # 初始化为$null，后续找到时会更新
    $msvcBinPath = $null
    # 遍历每个可能的Visual Studio安装路径
    foreach ($base in $vsBasePaths) {
        # 检查当前路径是否存在
        if (-not (Test-Path $base)) {
            # 路径不存在，跳过本次循环
            continue
        }

        # 变量：$vcTools | 含义：MSVC工具链的根目录路径 | 类型：字符串 | 作用域：foreach块内部
        # 拼接VC\Tools\MSVC路径，这是MSVC编译器工具链的存放位置
        $vcTools = Join-Path $base "VC\Tools\MSVC"
        # 检查MSVC工具链目录是否存在
        if (-not (Test-Path $vcTools)) {
            # 目录不存在，跳过本次循环
            continue
        }

        # 变量：$dirs | 含义：MSVC版本目录列表 | 类型：DirectoryInfo数组 | 作用域：foreach块内部
        # 获取MSVC目录下的所有子目录（每个子目录代表一个MSVC版本）
        # -ErrorAction SilentlyContinue表示遇到错误时继续执行
        # Sort-Object按版本号降序排序，Select-Object选择最新版本
        $dirs = Get-ChildItem -Path $vcTools -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
        # 遍历每个MSVC版本目录
        foreach ($d in $dirs) {
            # 变量：$linkPath | 含义：link.exe的完整路径 | 类型：字符串 | 作用域：foreach块内部
            # 拼接link.exe的路径，Hostx64\x64表示在64位主机上编译64位程序
            $linkPath = Join-Path $d.FullName "bin\Hostx64\x64\link.exe"
            # 检查link.exe是否存在
            if (Test-Path $linkPath) {
                # 变量：$msvcBinPath | 含义：包含link.exe的bin目录路径 | 类型：字符串 | 作用域：函数内部
                # 提取link.exe所在的目录路径（去掉文件名）
                $msvcBinPath = (Split-Path $linkPath -Parent)
                # 找到link.exe，跳出内层循环
                break
            }
        }

        # 检查是否已找到MSVC bin目录
        if ($msvcBinPath) {
            # 已找到，跳出外层循环
            break
        }
    }

    # 检查是否成功找到MSVC bin目录
    if ($msvcBinPath) {
        # 变量：$userPath | 含义：用户级PATH环境变量 | 类型：字符串 | 作用域：if块内部
        # 获取当前用户的PATH环境变量
        $userPath = [System.Environment]::GetEnvironmentVariable( "Path", "User" )
        # 检查MSVC bin目录是否已在用户PATH中
        if ($userPath -notlike "*$msvcBinPath*") {
            # 将MSVC bin目录添加到用户级PATH环境变量（永久生效）
            # 使用分号(;)连接路径
            [System.Environment]::SetEnvironmentVariable( "Path", ($userPath + ';' + $msvcBinPath), "User" )
            # 输出添加成功信息，绿色字体
            Write-Host ('[OK] MSVC linker path added to user PATH: ' + $msvcBinPath) -ForegroundColor Green
        # MSVC bin目录已在PATH中
        } else {
            # 输出已存在信息，绿色字体
            Write-Host '[OK] MSVC linker dir already in user PATH' -ForegroundColor Green
        }

        # 变量：$env:PATH | 含义：当前进程的PATH环境变量 | 类型：字符串 | 作用域：进程级
        # 将MSVC bin目录添加到当前会话的PATH中（立即生效）
        $env:PATH = $env:PATH + ';' + $msvcBinPath

        # 配置完成，返回
        return
    }

    # 未找到link.exe，需要安装Visual Studio Build Tools
    # 检查是否具有管理员权限（Build Tools需要管理员权限安装）
    if (Test-Administrator) {
        # 输出开始安装提示，青色字体
        Write-Host '[INSTALL] Installing Visual Studio Build Tools (C++) ...' -ForegroundColor Cyan
        # 尝试安装Build Tools
        try {
            # 调用winget安装Visual Studio 2022 Build Tools
            # --override参数传递VS安装器的自定义参数：
            # --passive: 显示进度但不需要用户交互
            # --wait: 等待安装完成
            # --add: 添加VCTools工作负载（包含C++编译器和链接器）
            # includeRecommended: 包含推荐组件
            winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended'
            # 输出安装成功提示，绿色字体，提醒用户需要重启终端
            Write-Host '[OK] Build Tools installed. Close and reopen terminal, then run this script again to add link.exe to PATH.' -ForegroundColor Green
            # 以退出码0正常终止脚本（需要重启终端后再次运行）
            exit 0
        # 捕获winget安装失败的异常
        } catch {
            # 输出错误信息，红色字体
            Write-Host '[ERROR] MSVC linker (link.exe) not found and Build Tools install failed.' -ForegroundColor Red
            # 输出详细失败原因，红色字体，$_是异常对象
            Write-Host ('Install failed: ' + $_) -ForegroundColor Red
            # 输出手动安装建议，黄色字体，提供官方下载链接
            Write-Host 'Install manually: https://visualstudio.microsoft.com/visual-cpp-build-tools/' -ForegroundColor Yellow
            # 输出继续提示，灰色字体，说明其他步骤不受影响
            Write-Host 'Setup will continue; other steps (deps, FFmpeg, etc.) are not affected.' -ForegroundColor Gray
        }
    # 非管理员权限
    } else {
        # 输出错误信息，红色字体，说明需要管理员权限安装Build Tools
        Write-Host '[ERROR] MSVC linker (link.exe) not found. Build Tools require administrator to install (no user-level install).' -ForegroundColor Red
        # 输出修复建议，黄色字体
        Write-Host 'Fix: Run PowerShell as Administrator, then run this script again, or run:' -ForegroundColor Yellow
        # 输出手动安装命令，白色字体
        Write-Host '  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended"' -ForegroundColor White
        # 输出继续提示，灰色字体
        Write-Host 'Setup will continue; other steps (deps, FFmpeg, etc.) are not affected.' -ForegroundColor Gray
    }
}

# 函数：Ensure-NSIS | 输入：无 | 输出：无(副作用：安装NSIS或添加到PATH) | 可能失败：winget失败、NSIS未正确安装
# 确保NSIS（Nullsoft Scriptable Install System）安装器系统可用
# NSIS用于生成Windows安装程序，Tauri打包需要
function Ensure-NSIS {
    # 输出检查提示，黄色字体
    Write-Host '[CHECK] NSIS (makensis.exe) ...' -ForegroundColor Yellow

    # 变量：$makensis | 含义：makensis.exe命令的路径对象 | 类型：CommandInfo或$null | 作用域：函数内部
    # 尝试在PATH中查找makensis.exe，-ErrorAction SilentlyContinue表示找不到时不报错
    $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
    # 检查是否找到makensis.exe
    if ($makensis) {
        # 输出找到信息，绿色字体，显示makensis.exe的完整路径
        Write-Host ('[OK] NSIS found: ' + $makensis.Source) -ForegroundColor Green
        # 已找到，直接返回
        return
    }

    # 变量：$nsisCandidates | 含义：NSIS可能安装的目录列表 | 类型：字符串数组 | 作用域：函数内部
    # 定义NSIS的常见安装位置
    # 包括Program Files和Program Files (x86)，以及Bin子目录
    $nsisCandidates = @(
        "${env:ProgramFiles}\NSIS",
        "${env:ProgramFiles(x86)}\NSIS",
        "${env:ProgramFiles}\NSIS\Bin",
        "${env:ProgramFiles(x86)}\NSIS\Bin"
    )
    # 遍历每个可能的NSIS安装目录
    foreach ($dir in $nsisCandidates) {
        # 检查当前目录是否存在
        if (-not (Test-Path $dir)) {
            # 目录不存在，跳过本次循环
            continue
        }

        # 变量：$exePath | 含义：makensis.exe的完整路径 | 类型：字符串 | 作用域：foreach块内部
        # 拼接makensis.exe的完整路径
        $exePath = Join-Path $dir "makensis.exe"
        # 检查makensis.exe是否存在
        if (Test-Path $exePath) {
            # 变量：$userPath | 含义：用户级PATH环境变量 | 类型：字符串 | 作用域：foreach块内部
            # 获取当前用户的PATH环境变量
            $userPath = [System.Environment]::GetEnvironmentVariable( "Path", "User" )
            # 检查NSIS目录是否已在用户PATH中
            if ($userPath -notlike "*$dir*") {
                # 将NSIS目录添加到用户级PATH环境变量（永久生效）
                [System.Environment]::SetEnvironmentVariable( "Path", ($userPath + ';' + $dir), "User" )
                # 输出添加成功信息，绿色字体
                Write-Host ('[OK] NSIS path added to user PATH: ' + $dir) -ForegroundColor Green
            }

            # 变量：$env:PATH | 含义：当前进程的PATH环境变量 | 类型：字符串 | 作用域：进程级
            # 将NSIS目录添加到当前会话的PATH中（立即生效）
            $env:PATH = $env:PATH + ';' + $dir

            # 变量：$makensis | 含义：makensis.exe命令的路径对象（重新检测） | 类型：CommandInfo或$null | 作用域：函数内部
            # 再次尝试查找makensis.exe以验证PATH更新成功
            $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
            # 检查是否成功找到
            if ($makensis) {
                # 输出找到信息，绿色字体
                Write-Host ('[OK] NSIS found: ' + $makensis.Source) -ForegroundColor Green
                # 配置完成，返回
                return
            }
        }
    }

    # 未在常见位置找到NSIS，需要安装
    # 输出开始安装提示，青色字体
    Write-Host '[INSTALL] Installing NSIS ...' -ForegroundColor Cyan

    # 尝试安装NSIS
    try {
        # 调用winget安装NSIS
        # -e精确匹配，--silent静默安装
        winget install -e --id NSIS.NSIS --silent

        # 变量：$makensis | 含义：makensis.exe命令的路径对象（重新检测） | 类型：CommandInfo或$null | 作用域：函数内部
        # 安装后再次尝试查找makensis.exe
        $makensis = Get-Command makensis.exe -ErrorAction SilentlyContinue
        # 检查是否成功找到
        if ($makensis) {
            # 输出安装成功信息，绿色字体
            Write-Host ('[OK] NSIS installed: ' + $makensis.Source) -ForegroundColor Green
            # 安装完成，返回
            return
        }

        # makensis.exe未在PATH中，可能需要重启终端
        # 输出警告信息，黄色字体
        Write-Host '[WARNING] NSIS installed but makensis.exe not found in PATH. Please restart terminal.' -ForegroundColor Yellow
    # 捕获winget安装失败的异常
    } catch {
        # 输出错误信息，红色字体，$_是异常对象
        Write-Host ('[ERROR] NSIS installation failed: ' + $_) -ForegroundColor Red
        # 输出手动安装建议，黄色字体，提供官方下载链接
        Write-Host 'Install manually: https://nsis.sourceforge.io/Download' -ForegroundColor Yellow
    }
}

# 函数：Ensure-NSISTauriUtils | 输入：无 | 输出：无(副作用：下载nsis_tauri_utils.dll) | 可能失败：网络异常、下载失败
# 下载并放置Tauri专用的NSIS插件DLL，用于NSIS安装包生成
function Ensure-NSISTauriUtils {
    # 输出检查提示，黄色字体
    Write-Host '[CHECK] NSIS tauri utils (nsis_tauri_utils.dll) ...' -ForegroundColor Yellow

    # 变量：$downloadUrl | 含义：nsis_tauri_utils.dll的GitHub下载地址 | 类型：字符串 | 作用域：函数内部
    # 指定插件的下载URL（版本v0.5.3）
    $downloadUrl = "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll"
    # 变量：$destDir | 含义：插件的目标安装目录 | 类型：字符串 | 作用域：函数内部
    # 拼接目标目录路径，LOCALAPPDATA是用户本地应用数据目录
    # x86-unicode是NSIS插件的架构类型，additional是自定义插件子目录
    $destDir = Join-Path $env:LOCALAPPDATA "tauri\NSIS\Plugins\x86-unicode\additional"
    # 变量：$destPath | 含义：插件DLL的完整目标路径 | 类型：字符串 | 作用域：函数内部
    # 拼接插件文件的完整路径
    $destPath = Join-Path $destDir "nsis_tauri_utils.dll"

    # 检查目标目录是否存在
    if (-not (Test-Path $destDir)) {
        # 目录不存在，创建目录
        # -Force参数会递归创建所有父目录，Out-Null抑制输出
        New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    }

    # 检查插件DLL是否已存在
    if (Test-Path $destPath) {
        # 输出已存在信息，绿色字体
        Write-Host ('[OK] nsis_tauri_utils.dll found: ' + $destPath) -ForegroundColor Green
        # 已存在，直接返回
        return
    }

    # 插件DLL不存在，需要下载
    # 输出开始下载提示，青色字体
    Write-Host '[INSTALL] Download nsis_tauri_utils.dll ...' -ForegroundColor Cyan

    # 变量：$maxRetries | 含义：最大重试次数 | 类型：整数 | 作用域：函数内部
    # 设置最大重试次数为3，以应对网络波动
    $maxRetries = 3
    # 循环重试下载
    for ($i = 1; $i -le $maxRetries; $i++) {
        # 尝试下载
        try {
            # 调用Invoke-WebRequest下载文件
            # -Uri指定下载地址，-OutFile指定保存路径
            # -UseBasicParsing避免依赖IE的DOM解析器
            Invoke-WebRequest -Uri $downloadUrl -OutFile $destPath -UseBasicParsing
            # 检查下载的文件是否存在（验证下载成功）
            if (Test-Path $destPath) {
                # 输出下载成功信息，绿色字体
                Write-Host "[OK] nsis_tauri_utils.dll downloaded to $destPath" -ForegroundColor Green
                # 下载成功，返回
                return
            }
        # 捕获下载失败的异常
        } catch {
            # 输出警告信息，黄色字体，显示当前尝试次数和最大次数
            # -f是格式化字符串操作符，{0}和{1}是占位符
            Write-Host (
                "[WARN] Download failed (attempt {0}/{1}): {2}" -f $i,
                $maxRetries,
                $_
            ) -ForegroundColor Yellow
            # 等待2秒后重试，避免立即重试对服务器造成压力
            Start-Sleep -Seconds 2
        }
    }

    # 所有重试都失败
    # 输出错误信息，红色字体
    Write-Host "[ERROR] Failed to download nsis_tauri_utils.dll. Please download it manually:" -ForegroundColor Red
    # 输出下载地址，黄色字体
    Write-Host $downloadUrl -ForegroundColor Yellow
    # 输出目标路径，黄色字体
    Write-Host "Then place it at: $destPath" -ForegroundColor Yellow
}

# 函数：Install-ProjectDependencies | 输入：无 | 输出：无(副作用：安装项目依赖包) | 可能失败：yarn/npm失败、网络异常、package.json错误
# 安装项目的Node.js依赖包，优先使用yarn，否则使用npm
function Install-ProjectDependencies {
    # 输出开始安装提示，青色字体
    Write-Host '[INSTALL] Project dependencies ...' -ForegroundColor Cyan

    # 变量：$projectRoot | 含义：项目根目录的绝对路径 | 类型：字符串 | 作用域：函数内部
    # 获取脚本所在目录的父目录（即项目根目录）
    # $PSScriptRoot是脚本文件所在目录，Split-Path -Parent获取其父目录
    $projectRoot = Split-Path -Parent $PSScriptRoot
    # 切换当前工作目录到项目根目录
    # 确保后续命令在正确的目录下执行
    Set-Location $projectRoot

    # 尝试安装依赖包
    try {
        # 检查yarn命令是否可用
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            # yarn可用，使用yarn安装依赖
            # yarn install会读取package.json和yarn.lock安装依赖
            yarn install
        # yarn不可用
        } else {
            # 使用npm安装依赖
            # npm install会读取package.json和package-lock.json安装依赖
            npm install
        }
        # 输出安装成功信息，绿色字体
        Write-Host '[OK] Project dependencies installed successfully' -ForegroundColor Green
    # 捕获依赖安装失败的异常
    } catch {
        # 输出错误信息，红色字体，$_是异常对象
        Write-Host ('[ERROR] Dependencies installation failed: ' + $_) -ForegroundColor Red
        # 以退出码1终止脚本，表示安装失败
        exit 1
    }
}

# 函数：Install-FFmpegTools | 输入：无 | 输出：无(副作用：下载ffmpeg.exe和ffprobe.exe) | 可能失败：网络异常、解压失败
# 下载FFmpeg工具（ffmpeg和ffprobe），用于音视频处理
function Install-FFmpegTools {
    # 输出检查提示，黄色字体
    Write-Host '[CHECK] FFmpeg tools (ffmpeg/ffprobe) ...' -ForegroundColor Yellow

    # 变量：$projectRoot | 含义：项目根目录的绝对路径 | 类型：字符串 | 作用域：函数内部
    # 获取项目根目录路径
    $projectRoot = Split-Path -Parent $PSScriptRoot
    # 变量：$binDir | 含义：FFmpeg工具的目标存放目录 | 类型：字符串 | 作用域：函数内部
    # 拼接工具存放目录路径，位于desktop应用的public/tools下
    $binDir = Join-Path $projectRoot "apps\desktop\public\tools"
    # 变量：$ffmpegExe | 含义：ffmpeg.exe的完整目标路径 | 类型：字符串 | 作用域：函数内部
    # 拼接ffmpeg.exe的完整路径
    $ffmpegExe = Join-Path $binDir "ffmpeg.exe"
    # 变量：$ffprobeExe | 含义：ffprobe.exe的完整目标路径 | 类型：字符串 | 作用域：函数内部
    # 拼接ffprobe.exe的完整路径
    $ffprobeExe = Join-Path $binDir "ffprobe.exe"

    # 检查ffmpeg.exe和ffprobe.exe是否都已存在
    if ((Test-Path $ffmpegExe) -and (Test-Path $ffprobeExe)) {
        # 输出已存在信息，绿色字体
        Write-Host '[OK] FFmpeg already exists' -ForegroundColor Green
        # 已存在，直接返回
        return
    }

    # FFmpeg工具不存在，需要下载
    # 输出开始下载提示，青色字体
    Write-Host '[INSTALL] Download FFmpeg ...' -ForegroundColor Cyan
    # 确保目标目录存在，-Force会递归创建，Out-Null抑制输出
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    # 变量：$tempDir | 含义：临时下载目录 | 类型：字符串 | 作用域：函数内部
    # 拼接临时目录路径，使用系统临时目录
    $tempDir = Join-Path $env:TEMP "hanamirip-ffmpeg"
    # 检查临时目录是否存在
    if (Test-Path $tempDir) {
        # 目录存在，删除以确保干净的下载环境
        # -Recurse递归删除所有内容，-Force强制删除
        Remove-Item -Recurse -Force $tempDir
    }
    # 创建临时目录，Out-Null抑制输出
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    # 变量：$zipPath | 含义：FFmpeg压缩包的本地保存路径 | 类型：字符串 | 作用域：函数内部
    # 拼接压缩包路径
    $zipPath = Join-Path $tempDir "ffmpeg.zip"
    # 变量：$url | 含义：FFmpeg下载地址 | 类型：字符串 | 作用域：函数内部
    # 使用gyan.dev提供的FFmpeg预编译包（仅包含核心功能，体积较小）
    $url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"

    # 尝试下载和解压FFmpeg
    try {
        # 调用Invoke-WebRequest下载FFmpeg压缩包
        # -UseBasicParsing避免依赖IE的DOM解析器
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        # 解压压缩包到临时目录
        # -Force覆盖已存在的文件
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        # 变量：$extracted | 含义：解压后的FFmpeg目录对象 | 类型：DirectoryInfo或$null | 作用域：函数内部
        # 查找解压后的目录（通常名称以"ffmpeg-"开头）
        # Get-ChildItem -Directory获取所有子目录
        # Where-Object过滤名称匹配的目录
        # Select-Object -First 1选择第一个匹配项
        $extracted = Get-ChildItem -Path $tempDir -Directory
            | Where-Object {
                $_.Name -like "ffmpeg-*"
            }
            | Select-Object -First 1
        # 检查是否找到解压目录
        if (-not $extracted) {
            # 未找到，抛出异常
            throw "FFmpeg zip 解压失败"
        }

        # 复制ffmpeg.exe到目标目录
        # Join-Path拼接源文件路径，-Destination指定目标路径，-Force覆盖已存在文件
        Copy-Item (Join-Path $extracted.FullName "bin\ffmpeg.exe") -Destination $ffmpegExe -Force
        # 复制ffprobe.exe到目标目录
        Copy-Item (Join-Path $extracted.FullName "bin\ffprobe.exe") -Destination $ffprobeExe -Force

        # 输出下载成功信息，绿色字体
        Write-Host '[OK] FFmpeg download done' -ForegroundColor Green
    # 捕获下载或解压失败的异常
    } catch {
        # 输出错误信息，红色字体，$_是异常对象
        Write-Host ('[ERROR] FFmpeg download failed: ' + $_) -ForegroundColor Red
        # 输出手动下载建议，黄色字体
        Write-Host 'Download manually and put into apps\desktop\public\tools' -ForegroundColor Yellow
        # 输出下载地址，黄色字体
        Write-Host ('URL: ' + $url) -ForegroundColor Yellow
    # finally块无论是否异常都会执行，用于清理临时文件
    } finally {
        # 检查临时目录是否存在
        if (Test-Path $tempDir) {
            # 删除临时目录及其内容
            Remove-Item -Recurse -Force $tempDir
        }
    }
}

# 函数：Install-MkvToolNixTools | 输入：无 | 输出：无（副作用：下载并放置 mkvmerge/mkvinfo） | 可能失败：版本索引解析失败、下载失败、压缩包结构变化
# 确保项目内置工具目录包含 MKVToolNix 的核心命令。
function Install-MkvToolNixTools {
    # 输出 MKVToolNix 检查提示。
    Write-Host '[CHECK] MKVToolNix tools (mkvmerge/mkvinfo) ...' -ForegroundColor Yellow

    # 变量：$projectRoot | 含义：仓库根目录路径 | 类型：String | 作用域：函数内部
    # 定位仓库根目录。
    $projectRoot = Split-Path -Parent $PSScriptRoot
    # 变量：$binDir | 含义：工具目标目录 | 类型：String | 作用域：函数内部
    # 指向公共工具目录。
    $binDir = Join-Path $projectRoot "apps\desktop\public\tools"
    # 变量：$mkvmergeExe | 含义：mkvmerge.exe 目标路径 | 类型：String | 作用域：函数内部
    # 拼接 mkvmerge 的目标路径。
    $mkvmergeExe = Join-Path $binDir "mkvmerge.exe"
    # 变量：$mkvinfoExe | 含义：mkvinfo.exe 目标路径 | 类型：String | 作用域：函数内部
    # 拼接 mkvinfo 的目标路径。
    $mkvinfoExe = Join-Path $binDir "mkvinfo.exe"

    # 若两个工具都存在则直接复用。
    if ((Test-Path $mkvmergeExe) -and (Test-Path $mkvinfoExe)) {
        # 输出已存在提示。
        Write-Host '[OK] MKVToolNix already exists' -ForegroundColor Green
        # 结束函数。
        return
    }

    # 输出下载阶段提示。
    Write-Host '[INSTALL] Download MKVToolNix ...' -ForegroundColor Cyan
    # 确保目标目录存在。
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null

    # 变量：$tempDir | 含义：MKVToolNix 下载临时目录 | 类型：String | 作用域：函数内部
    # 创建专用临时目录。
    $tempDir = Join-Path $env:TEMP "hanamirip-mkvtoolnix"
    if (Test-Path $tempDir) {
        Remove-Item -Recurse -Force $tempDir
    }
    New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

    # 变量：$zipPath | 含义：zip 下载文件路径 | 类型：String | 作用域：函数内部
    # 设置压缩包文件名。
    $zipPath = Join-Path $tempDir "mkvtoolnix.zip"
    # 变量：$indexUrl | 含义：版本索引页地址 | 类型：String | 作用域：函数内部
    # 使用官方 releases 页面解析最新版本。
    $indexUrl = "https://mkvtoolnix.download/windows/releases/"
    # 变量：$url | 含义：最终选中的可下载 zip 地址 | 类型：String 或 $null | 作用域：函数内部
    # 初始化为 null，后续成功时赋值。
    $url = $null

    # 尝试解析版本并下载。
    try {
        # 变量：$index | 含义：索引页响应对象 | 类型：HtmlWebResponseObject | 作用域：try 块内部
        # 拉取版本索引页面。
        $index = Invoke-WebRequest -Uri $indexUrl -UseBasicParsing
        # 变量：$versions | 含义：提取出的版本号列表 | 类型：ArrayList/String[] | 作用域：try 块内部
        # 初始化版本集合。
        $versions = @()
        # 遍历索引中的链接，提取版本号。
        foreach ($link in $index.Links) {
            # 变量：$re | 含义：版本目录匹配正则 | 类型：String | 作用域：foreach 块内部
            # 定义匹配 windows/releases/{version}/ 的规则。
            $re = "windows/releases/(\d+(\.\d+)*)/"
            # 若链接匹配到版本目录则收集版本号。
            if ($link.href -match $re) {
                # 追加匹配到的版本号。
                $versions += $matches[1]
            }
        }

        # 没有提取到任何版本则抛错。
        if ($versions.Count -eq 0) {
            throw 'MKVToolNix version list parse failed'
        }

        # 变量：$latest | 含义：解析出的最新版本号 | 类型：String | 作用域：try 块内部
        # 将版本按语义版本降序排序并取最新。
        $latest = $versions
            | Sort-Object {
                [version] $_
            } -Descending
            | Select-Object -First 1
        # 变量：$baseUrl | 含义：最新版本目录基地址 | 类型：String | 作用域：try 块内部
        # 组合最新版本目录地址。
        $baseUrl = "https://mkvtoolnix.download/windows/releases/$latest/"
        # 变量：$candidates | 含义：按常见命名构造的候选 zip 文件名 | 类型：String[] | 作用域：try 块内部
        # 覆盖不同补位格式（x.y、x.y.0、x.y.0.0）。
        $candidates = @(
            "mkvtoolnix-64-bit-$latest.zip",
            "mkvtoolnix-64-bit-$latest.0.zip",
            "mkvtoolnix-64-bit-$latest.0.0.zip"
        )

        # 依次尝试候选下载地址直到成功。
        foreach ($name in $candidates) {
            # 变量：$candidateUrl | 含义：当前尝试的完整下载地址 | 类型：String | 作用域：foreach 块内部
            # 拼接本次候选地址。
            $candidateUrl = $baseUrl + $name
            try {
                # 尝试下载候选压缩包。
                Invoke-WebRequest -Uri $candidateUrl -OutFile $zipPath -UseBasicParsing
                # 下载成功后记录真实 URL。
                $url = $candidateUrl
                # 成功后跳出候选循环。
                break
            } catch {
                # 当前候选失败，重置 URL 继续下一个。
                $url = $null
            }
        }

        # 全部候选都失败则抛错。
        if (-not $url) {
            throw 'MKVToolNix zip not found'
        }

        # 解压下载的压缩包。
        Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

        # 变量：$mkvmergeFound | 含义：解压结果中的 mkvmerge.exe 文件对象 | 类型：FileInfo 或 $null | 作用域：try 块内部
        # 递归查找 mkvmerge 可执行文件。
        $mkvmergeFound = Get-ChildItem -Path $tempDir -Recurse -Filter 'mkvmerge.exe' | Select-Object -First 1
        # 变量：$mkvinfoFound | 含义：解压结果中的 mkvinfo.exe 文件对象 | 类型：FileInfo 或 $null | 作用域：try 块内部
        # 递归查找 mkvinfo 可执行文件。
        $mkvinfoFound = Get-ChildItem -Path $tempDir -Recurse -Filter 'mkvinfo.exe' | Select-Object -First 1
        # 任一文件缺失都视为解压结果异常。
        if (-not $mkvmergeFound -or -not $mkvinfoFound) {
            throw 'MKVToolNix zip extract failed'
        }

        # 复制 mkvmerge 到目标目录。
        Copy-Item $mkvmergeFound.FullName -Destination $mkvmergeExe -Force
        # 复制 mkvinfo 到目标目录。
        Copy-Item $mkvinfoFound.FullName -Destination $mkvinfoExe -Force

        # 输出下载完成提示。
        Write-Host '[OK] MKVToolNix download done' -ForegroundColor Green
    } catch {
        Write-Host ('[ERROR] MKVToolNix download failed: ' + $_) -ForegroundColor Red
        Write-Host 'Download manually and put into apps\desktop\public\tools' -ForegroundColor Yellow
        if ($url) {
            Write-Host ('URL: ' + $url) -ForegroundColor Yellow
        } else {
            Write-Host ('Index: ' + $indexUrl) -ForegroundColor Yellow
        }
    # 无论成功失败都清理临时目录。
    } finally {
        if (Test-Path $tempDir) {
            Remove-Item -Recurse -Force $tempDir
        }
    }
}

# 函数：Generate-Icons | 输入：无 | 输出：无（副作用：在缺失时生成应用图标） | 可能失败：tauri cli 不可用、命令参数错误
# 检查并生成应用图标资源（若 icon.ico 不存在）。
function Generate-Icons {
    # 输出图标检查提示。
    Write-Host '[CHECK] Application icons ...' -ForegroundColor Yellow

    # 变量：$iconPath | 含义：目标 ico 文件路径 | 类型：String | 作用域：函数内部
    # 拼接 icon.ico 的预期位置。
    $iconPath = Join-Path $PSScriptRoot '..\apps\desktop\public\icons\icon.ico'
    # 若图标已存在则跳过生成。
    if (Test-Path $iconPath) {
        # 输出已存在提示。
        Write-Host '[OK] Icon file exists' -ForegroundColor Green
        # 结束函数。
        return
    }

    # 输出开始生成图标提示。
    Write-Host '[GENERATE] Application icons ...' -ForegroundColor Cyan
    # 尝试调用包管理器脚本生成图标。
    try {
        # 优先使用 yarn 执行 tauri icon 命令。
        if (Get-Command yarn -ErrorAction SilentlyContinue) {
            yarn tauri icon apps / desktop / public / icons / icon.png
        # yarn 不可用时回退到 npm。
        } else {
            npm run tauri icon apps / desktop / public / icons / icon.png
        }
        # 输出生成成功提示。
        Write-Host '[OK] Icons generated successfully' -ForegroundColor Green
    # 图标失败不阻断主流程，仅给出警告。
    } catch {
        Write-Host ('[WARNING] Icon generation failed: ' + $_) -ForegroundColor Yellow
    }
}

# 函数：Main | 输入：无 | 输出：无（副作用：顺序执行全部环境准备步骤） | 可能失败：winget 缺失、关键依赖安装失败
# 脚本主入口：先检查 winget，再按既定顺序执行各安装函数。
function Main {
    # 先验证 winget 可用性。
    try {
        # 调用 winget 版本命令，输出重定向到空设备。
        winget --version | Out-Null
    # winget 不可用则直接报错并退出。
    } catch {
        # 输出错误信息。
        Write-Host '[ERROR] winget (Windows Package Manager) not detected' -ForegroundColor Red
        # 输出系统版本建议。
        Write-Host 'Please ensure you are using Windows 10 1809+ or Windows 11' -ForegroundColor Yellow
        # 提示安装 App Installer。
        Write-Host 'Install App Installer from Microsoft Store' -ForegroundColor Yellow
        # 无法继续执行，退出。
        exit 1
    }

    # 安装或检查 Node.js。
    Install-NodeJS
    # 安装或检查 Yarn。
    Install-Yarn
    # 安装或检查 Rust。
    Install-Rust
    # 安装 Rust 多目标支持。
    Install-RustTargets
    # 确保 MSVC 链接器可用。
    Ensure-MSVCForRust
    # 确保 NSIS 可用。
    Ensure-NSIS
    # 确保 NSIS tauri 插件可用。
    Ensure-NSISTauriUtils
    # 安装 Node 依赖。
    Install-ProjectDependencies
    # 安装 FFmpeg 工具。
    Install-FFmpegTools
    # 安装 MKVToolNix 工具。
    Install-MkvToolNixTools
    # 生成应用图标。
    Generate-Icons

    # 输出完成提示。
    Write-Host "=== Environment Setup Complete! ===" -ForegroundColor Green
}

# 调用主入口函数。
Main
