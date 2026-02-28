// ========================================================
// CSS / HTML / JSON / Vue / TS 格式化规则 (Prettier 配置)
// 文档：https://prettier.io/docs/en/options.html
// 兼容版本：Prettier 3.0+
// ========================================================
// 使用示例：
// npx prettier --write xxx
// ========================================================

/** @type {import("prettier").Config} */
export default {
	// ==================== 行长与缩进 ====================
	// printWidth 说明：单行最大字符数，超过此长度会自动换行
	// 与 rustfmt 的 max_width = 150 保持一致
	printWidth: 150,

	// tabWidth 说明：一个制表符对应的空格数
	// 与 rustfmt 的 tab_spaces = 4 保持一致
	tabWidth: 4,

	// useTabs 说明：false = 使用空格；true = 使用制表符
	// 与 rustfmt 的 hard_tabs = false 保持一致
	useTabs: false,

	// ==================== 分号与引号 ====================
	// semi 说明：true = 在语句末尾添加分号；false = 不添加
	// CSS 样式表中的属性不需要分号后的换行，此项主要影响 JS 部分
	semi: true,

	// singleQuote 说明：false = 使用双引号；true = 使用单引号
	// CSS 推荐使用双引号作为标准
	singleQuote: false,

	// ==================== 引号在属性值中的处理 ====================
	// quoteProps 说明：'as-needed' = 仅在必要时添加引号（推荐用于 CSS）
	quoteProps: 'as-needed',

	// ==================== JSX 和箭头函数 ====================
	// bracketSpacing 说明：true = 在对象字面量的括号内添加空格
	// 示例：{ foo: bar } 而不是 {foo:bar}
	bracketSpacing: true,

	// bracketSameLine 说明：false = 右括号放在新行（取代已弃用的 jsxBracketSameLine）
	// 这对 XML/HTML/JSX 类内容的可读性更好
	bracketSameLine: false,

	// arrowParens 说明：'always' = 箭头函数参数总是用括号包裹
	// 示例：(x) => x 而不是 x => x
	arrowParens: 'always',

	// ==================== 数组与对象的尾逗号 ====================
	// trailingComma 说明：
	// - 'es5'：在多行 ES5 有效的位置（对象、数组）添加尾逗号
	// - 'none'：不添加尾逗号
	// - 'all'：尽可能在所有位置添加尾逗号（需要 ES8+）
	// 推荐使用 'es5' 以保持良好兼容性
	trailingComma: 'es5',

	// ==================== CSS 专用选项 ====================
	// singleAttributePerLine 说明：
	// - false：多个 CSS 属性可在同一行（根据 printWidth 自动换行）
	// - true：每个 CSS 属性单独占一行（更易读，但文件更大）
	// 推荐 false 以保持紧凑性，与rustfmt保持一致
	singleAttributePerLine: false,

	// ==================== HTML 专用选项 ====================
	// htmlWhitespaceSensitivity 说明：
	// - 'css'：根据 CSS display 规则处理空白（推荐，默认值）
	// - 'strict'：保留所有空白（可能导致格式不佳）
	// - 'ignore'：忽略空白处理（最紧凑，适合生成的 HTML）
	// 推荐使用 'css' 以保持正确的空白语义
	htmlWhitespaceSensitivity: 'css',

	// ==================== HTML 文本换行 ====================
	// proseWrap 说明：控制如何格式化 Markdown/HTML 中的长文本行
	// - 'preserve'：保持原样（不重新格式化长段落）
	// - 'always'：超过 printWidth 时自动换行（对 Markdown 友好）
	// - 'never'：不换行（即使超过 printWidth）
	// 推荐 'preserve' 以保留原始文本结构
	proseWrap: 'preserve',

	// ==================== 属性换行 ====================
	// bracketSameLine（已在 JSX 部分定义，对 HTML 也适用）
	// false = <div\n  class="..."\n> (多行属性)
	// true  = <div class="..." > (属性与标签在同一行)
	// 推荐 false 以提高可读性，特别是属性较多时

	// ==================== JSON 专用选项 ====================
	// trailingComma 对 JSON 的影响：
	// - 当 trailingComma 为 'es5' 时，Prettier 智能处理 JSON
	// - JSON 文件中：自动设为 'none'（JSON 标准不支持尾逗号）
	// - JS/TS 文件中：保持 'es5'（可添加尾逗号）
	//
	// bracketSpacing 对 JSON 的影响：
	// - true（当前设置）：{ "key": "value" } 格式化为 { "key": "value" }
	// - false：会格式化为 {"key": "value"}（稍紧凑）
	// 推荐保持 true 以提高可读性
	//
	// printWidth 和 tabWidth 对 JSON 的影响：
	// - printWidth: 150 - JSON 对象/数组超过 150 字符自动换行
	// - tabWidth: 4 - JSON 嵌套缩进使用 4 个空格
	//
	// 示例格式化：
	// 输入：
	//   {"name":"John","age":30,"address":{"street":"123 Main St","city":"New York"},"hobbies":["reading","coding"]}
	// 输出（超过 150 字符，自动换行）：
	//   {
	//       "name": "John",
	//       "age": 30,
	//       "address": {
	//           "street": "123 Main St",
	//           "city": "New York"
	//       },
	//       "hobbies": ["reading", "coding"]
	//   }

	// ==================== 换行符风格 ====================
	// endOfLine 说明：
	// - 'lf'：LF 换行符（Unix/Linux/macOS 标准）
	// - 'crlf'：CRLF 换行符（Windows 标准）
	// - 'cr'：CR 换行符（旧 Mac 标准，很少使用）
	// - 'auto'：自动检测（基于现有文件）
	// 推荐使用 'lf' 保持跨平台一致性
	endOfLine: 'lf',

	// ==================== 支持的文件类型 ====================
	// 该配置自动适用于以下文件：
	// - *.css：标准 CSS 文件
	// - *.html：HTML 文件（模板、静态页面）
	// - *.json：JSON 配置文件（package.json、tsconfig.json 等）
	// - *.vue：Vue 单文件组件（<template><style> 块）
	// - *.ts：TypeScript 文件
	// - *.ps1：PowerShell 脚本文件（需要额外配置，见下文）

	// ==================== 注释处理 ====================
	// Prettier 自动保留并格式化注释：
	// - /* ... */ 多行注释：格式化但保留内容
	// - // 单行注释：JavaScript/SCSS 中保留
	// - 注释后的代码：自动换行处理

	// ==================== 集成建议 ====================
	// 1. VS Code 集成：
	//    - 安装 "Prettier - Code formatter" 扩展（esbenp.prettier-vscode）
	//    - 在 settings.json 中设置：
	//      {
	//        "editor.defaultFormatter": "esbenp.prettier-vscode",
	//        "[css]": { "editor.defaultFormatter": "esbenp.prettier-vscode" },
	//        "[html]": { "editor.defaultFormatter": "esbenp.prettier-vscode" },
	//        "[json]": { "editor.defaultFormatter": "esbenp.prettier-vscode" },
	//        "[vue]": { "editor.defaultFormatter": "esbenp.prettier-vscode" },
	//        "[typescript]": { "editor.defaultFormatter": "esbenp.prettier-vscode" },
	//        "[powershell]": { "editor.defaultFormatter": "ms-vscode.PowerShell" },
	//        "editor.formatOnSave": true
	//      }
	//
	//    特别说明：PowerShell 格式化需单独配置，见下文
	//
	// 2. 命令行使用（CSS 文件）：
	//    npx prettier --write "apps/**/*.css"
	//    npx prettier --check "apps/**/*.css"
	//
	// 3. 命令行使用（HTML 文件）：
	//    npx prettier --write "apps/**/*.html"
	//    npx prettier --check "apps/**/*.html"
	//
	// 4. 命令行使用（JSON 文件）：
	//    npx prettier --write "*.json"
	//    npx prettier --write "apps/**/*.json"
	//    npx prettier --check "*.json"
	//    示例：对项目中的所有 JSON 配置文件进行格式化
	//      npx prettier --write "package.json" "tsconfig.json" "tauri.conf.json"
	//
	// 5. 命令行使用（TS 文件）：
	//    npx prettier --write "apps/**/*.ts"
	//
	// 6. 命令行使用（所有支持的文件）：
	//    npx prettier --write "apps/"
	//    npx prettier --write "."  （格式化当前目录所有支持文件）
	//
	// 7. 命令行使用（PowerShell 文件，需要专用工具）：
	//    A. 使用 VS Code PowerShell 扩展（最简单，推荐）：
	//       - 安装 "PowerShell" 官方扩展（ms-vscode.powershell）
	//       - 在 .ps1 文件中按 Shift+Alt+F 快速格式化
	//       - 或在终端运行：code -r scripts/banner.ps1
	//       - VS Code 会自动调用扩展进行格式化
	//       - 优点：无需 npm 依赖，内置 VS Code，最稳定可靠
	//
	//    B. 使用 prettier-plugin-powershell（完全自动化）：
	//       第一步：安装插件
	//         npm install --save-dev prettier-plugin-powershell
	//       第二步：清除缓存
	//         rm -rf node_modules/.cache  （Linux/macOS）
	//         rmdir /s /q node_modules\.cache  （Windows PowerShell）
	//       第三步：格式化 PowerShell 文件
	//         npx prettier --write "scripts/*.ps1"
	//         npx prettier --write scripts/banner.ps1
	//         npx prettier --write scripts/setup-env.ps1
	//       重要：如果仍然提示 "No parser could be inferred"，说明插件未生效
	//       解决方法：
	//         1. 检查是否成功安装：npm list prettier-plugin-powershell
	//         2. 尝试重新安装：npm reinstall prettier-plugin-powershell
	//         3. 强制清空 Prettier 缓存：npm run prettier -- --cache-strategy=content --write scripts/
	//
	//    C. 使用 PSScriptAnalyzer（代码质量检查，不是格式化）：
	//       - Install-Module -Name PSScriptAnalyzer -Force
	//       - Invoke-ScriptAnalyzer -Path scripts/banner.ps1 -Severity Warning
	//       - 输出规则违规列表，开发者手动修复
	//
	// 8. Git Hook 集成（pre-commit）：
	//    使用 husky + lint-staged 配置自动格式化。
	//    .lintstagedrc.json 示例：
	//    {
	//      "*.css": "prettier --write",
	//      "*.html": "prettier --write",
	//      "*.json": "prettier --write",
	//      "*.vue": "prettier --write",
	//      "*.ts": "prettier --write",
	//      "*.ps1": ["powershell -Command \"Invoke-Formatter -Path {} || true\""]
	//    }
	//    注意：PowerShell 格式化无法通过 Prettier 自动化，建议使用 VS Code 或 PSScriptAnalyzer

	// ==================== 性能与缓存 ====================
	// - Prettier 会自动生成 .prettier-cache 目录
	// - 首次格式化扫描文件，后续使用缓存加速
	// - 建议在 .gitignore 中添加：.prettier-cache/
	// - 清空缓存：rm -rf .prettier-cache/（或在 Windows 中使用 rmdir）

	// ==================== HTML 格式化行为示例 ====================
	// 当前配置对 HTML 文件的影响：
	//
	// 1. 属性换行（bracketSameLine: false）：
	//    输入：
	//      <div class="container" id="main" data-value="test">
	//    输出（属性较多或超过 printWidth 时）：
	//      <div
	//        class="container"
	//        id="main"
	//        data-value="test"
	//      >
	//
	// 2. 空白处理（htmlWhitespaceSensitivity: 'css'）：
	//    <p>Hello   world</p> 会被格式化为 <p>Hello world</p>
	//    （多余空白被合并，保持 CSS 规则）
	//
	// 3. 嵌套元素缩进（tabWidth: 4）：
	//    <div>
	//        <p>
	//            嵌套内容
	//        </p>
	//    </div>
	//
	// 4. 自闭合标签：
	//    <img src="..." /> 或 <br /> 格式化取决于 HTML 版本
	//    HTML5 推荐：<img src="..." > 或 <br >
	//
	// ==================== HTML 最佳实践 ====================
	// 1. 保持代码一致：
	//    使用 formatOnSave，确保所有 HTML 文件自动格式化
	//
	// 2. 处理大文件：
	//    如果 HTML 文件特别大（>1MB），考虑分割或使用模板引擎
	//
	// 3. 特殊情况处理：
	//    - 内联脚本（<script>）：Prettier 会格式化其中的 JavaScript
	//    - 内联样式（<style>）：Prettier 会格式化其中的 CSS
	//    - 模板语法（如 Vue/Handlebars）：需要对应的插件支持
	//
	// 4. 忽略特定块：
	//    在 HTML 中添加注释禁用格式化：
	//      <!-- prettier-ignore -->
	//      <div class="   weird   spacing   "></div>
	//
	// 5. 行长设置验证：
	//    printWidth: 150 适合大多数编辑器和屏幕宽度
	//    如无特殊需求，不建议改动

	// ==================== JSON 格式化行为示例 ====================
	// 当前配置对 JSON 文件的影响：
	//
	// 1. 对象格式化（bracketSpacing: true）：
	//    输入（紧凑格式）：
	//      {"name":"John","email":"john@example.com"}
	//    输出（标准格式）：
	//      { "name": "John", "email": "john@example.com" }
	//    如果超过 150 字符，会自动换行为多行：
	//      {
	//          "name": "John",
	//          "email": "john@example.com",
	//          "address": "123 Main Street"
	//      }
	//
	// 2. 数组格式化（缩进和换行）：
	//    输入：
	//      {"items":[1,2,3,4,5],"total":5}
	//    输出（短数组保持一行）：
	//      { "items": [1, 2, 3, 4, 5], "total": 5 }
	//    输出（长数组自动换行）：
	//      {
	//          "items": [
	//              1,
	//              2,
	//              3,
	//              4,
	//              5
	//          ],
	//          "total": 5
	//      }
	//
	// 3. 嵌套对象缩进（tabWidth: 4）：
	//    {
	//        "config": {
	//            "database": {
	//                "host": "localhost",
	//                "port": 5432
	//            }
	//        }
	//    }
	//
	// 4. 尾逗号处理（trailingComma 对 JSON 的影响）：
	//    JSON 标准不允许尾逗号，Prettier 自动处理：
	//    - 在 JSON 文件中：无尾逗号
	//    - 在 JavaScript 文件中：根据 trailingComma 设置添加尾逗号
	//
	// ==================== JSON 最佳实践 ====================
	// 1. 配置文件格式化：
	//    使用 formatOnSave 确保项目配置文件（package.json、tsconfig.json）自动格式化
	//    推荐命令：npx prettier --write "package.json" "tsconfig.json"
	//
	// 2. 处理 JSON 注释：
	//    - 标准 JSON：不支持注释，使用 *.json 文件
	//    - 如需注释，建议把该配置迁移到 .js/.ts 配置文件
	//
	// 3. 常见 JSON 文件的格式化：
	//    - package.json：项目元数据和依赖
	//    - tsconfig.json：TypeScript 配置
	//    - tauri.conf.json：Tauri 应用配置（本项目）
	//    - .vscode/settings.json：VS Code 工作区设置
	//    - .prettierrc.json：Prettier 配置（如果不使用 .js 或 .mjs）
	//    示例：npx prettier --write "*.json" ".vscode/*.json"
	//
	// 4. 忽略 JSON 格式化：
	//    如果某个 JSON 文件需要保持特定格式（如最小化），使用 .prettierignore：
	//      # .prettierignore
	//      dist/
	//      build.json
	//    或在文件顶部禁用格式化：
	//      // 此方法对 JSON 不适用，JSON 不支持注释
	//      // 建议使用 .prettierignore 文件或在命令行中排除
	//
	// 5. 批量格式化 JSON 文件：
	//    新增所有 JSON 配置：
	//      npx prettier --write "*.json" "apps/**/*.json"
	//    检查 JSON 格式是否符合规范：
	//      npx prettier --check "*.json"
	//    仅列出需要格式化的文件：
	//      npx prettier --list-different "*.json"
	//
	// 6. PowerShell 文件格式化（脚本）：
	//    项目中的 .ps1 文件位置：
	//    - scripts/banner.ps1：构建横幅大小写生成脚本
	//    - scripts/clean.js：项目清理脚本
	//    - scripts/setup-env.ps1：开发环境初始化脚本
	//
	//    a) 使用 VS Code PowerShell 扩展（推荐）：
	//       - 在 VS Code 中打开 .ps1 文件
	//       - 按 Shift+Alt+F（或 Cmd+Shift+P → "Format Document"）
	//       - VS Code 会自动格式化 PowerShell 文件
	//
	//    b) 使用 PSScriptAnalyzer 检查代码质量：
	//       - 在 PowerShell 中运行：
	//         Invoke-ScriptAnalyzer -Path scripts/banner.ps1 -Severity Warning
	//       - 查看警告和建议，手动修复或忽略
	//
	//    c) 如需自动格式化，安装 prettier-plugin-powershell：
	//       npm install --save-dev prettier-plugin-powershell
	//       然后运行：npx prettier --write "scripts/*.ps1"
	//
	// 6. 与 package.json 集成：
	//    在 package.json 中添加格式化脚本：
	//      "scripts": {
	//          "format": "prettier --write .",
	//          "format:json": "prettier --write \"*.json\"",
	//          "format:check": "prettier --check .",
	//          "format:ps1": "powershell scripts/setup-env.ps1"
	//      }
	//    使用：npm run format 或 npm run format:json 或 npm run format:ps1

	// ==================== PowerShell 格式化行为详解 ====================
	// 本项目中的 PowerShell 脚本文件（scripts/ 目录）：
	// - scripts/banner.ps1：构建项目横幅标题（彩色输出）
	// - scripts/setup-env.ps1：初始化开发环境（Node、npm、Rust、VSCode）
	// - scripts/clean.js：JavaScript 脚本（使用 Prettier 标准格式化）
	//
	// PowerShell 与 Prettier 的兼容性说明：
	// - Prettier 核心功能仅支持 Web 技术（JS/CSS/HTML/JSON/TS）
	// - PowerShell 不在 Prettier 的原生支持范围内
	// - 需要使用第三方插件：prettier-plugin-powershell
	//
	// 错误解决：
	// 执行 npx prettier --write scripts/banner.ps1 时出现：
	//   [error] No parser could be inferred for file
	// 原因：prettier-plugin-powershell 未安装或未正确加载
	//
	// 推荐解决方案（按优先级，推荐第 1 项）：
	//
	// 方案 1：VS Code PowerShell 扩展（最简单、最推荐）
	// ======================================================
	// 优点：
	//   - 无需额外 npm 依赖
	//   - 与 VS Code 原生集成
	//   - 使用官方扩展，稳定可靠
	//   - 适合团队开发，无需共享 npm 配置
	//
	// 安装步骤：
	//   1. 打开 VS Code 扩展市场（Ctrl+Shift+X 或 Cmd+Shift+X）
	//   2. 搜索 "PowerShell"
	//   3. 安装 "PowerShell" 扩展（发布者：Microsoft）
	//
	// 配置 .vscode/settings.json（可选，使用官方扩展自动检测）：
	//   {
	//       "[powershell]": {
	//           "editor.defaultFormatter": "ms-vscode.PowerShell",
	//           "editor.formatOnSave": true,
	//           "editor.tabSize": 4,
	//           "editor.insertSpaces": true
	//       }
	//   }
	//
	// 使用方法：
	//   - 在 VS Code 中打开 .ps1 文件
	//   - 按 Shift+Alt+F（或 Shift+Command+F on macOS）
	//   - 或右键选择 "Format Document"
	//   - 或 Cmd+Shift+P 搜索 "Format Document"
	//   - 启用 formatOnSave 后会自动格式化保存文件
	//
	// 方案 2：prettier-plugin-powershell（完全自动化）
	// ======================================================
	// 优点：
	//   - 与 Prettier 生态完全集成
	//   - 支持命令行批量格式化
	//   - 支持 Git Hook 自动化
	//
	// 安装步骤：
	//   npm install --save-dev prettier-plugin-powershell
	//
	// 清除缓存（重要，否则可能仍然无法识别）：
	//   # Linux / macOS
	//   rm -rf node_modules/.cache
	//   rm -rf .prettier-cache
	//   npx prettier --cache-strategy=content --clear-cache
	//
	//   # Windows PowerShell
	//   Remove-Item -Path node_modules\.cache -Recurse -Force
	//   Remove-Item -Path .prettier-cache -Recurse -Force
	//   npx prettier --cache-strategy=content --clear-cache
	//
	// 验证插件是否安装：
	//   npm list prettier-plugin-powershell
	//
	// 使用方法（格式化单个文件）：
	//   npx prettier --write scripts/banner.ps1
	//   npx prettier --write scripts/setup-env.ps1
	//
	// 使用方法（格式化整个目录）：
	//   npx prettier --write scripts/
	//   npx prettier --write \"scripts/**/*.ps1\"
	//
	// 使用方法（检查格式是否符合规范，不进行修改）：
	//   npx prettier --check scripts/
	//
	// 故障排除：
	// 如果仍然提示 "No parser could be inferred"，尝试：
	//   1. 重新安装插件：
	//      npm uninstall prettier-plugin-powershell
	//      npm install --save-dev prettier-plugin-powershell
	//   2. 清除所有缓存：
	//      rm -rf node_modules .prettier-cache
	//      npm install
	//   3. 验证 prettier 版本（需要 3.0+）：
	//      npx prettier --version
	//   4. 显式指定插件（编辑此文件，取消注释 plugins 行）：
	//      plugins: ['prettier-plugin-powershell']
	//   5. 检查 PowerShell 文件的文件后缀名是否为 .ps1
	//
	// 方案 3：PSScriptAnalyzer（代码质量检查）
	// ======================================================
	// 说明：
	//   - PSScriptAnalyzer 用于代码质量检查，不是代码格式化
	//   - 可检查出常见的 PowerShell 编程错误和最佳实践违规
	//
	// 安装：
	//   Install-Module -Name PSScriptAnalyzer -Repository PSGallery -Force
	//
	// 使用：
	//   Invoke-ScriptAnalyzer -Path scripts/banner.ps1 -Severity Warning
	//   Invoke-ScriptAnalyzer -Path scripts/setup-env.ps1 -Severity Error
	//
	// 输出示例：
	//   RuleName                         Severity   Line  Message
	//   --------                         --------   ----  -------
	//   PSAvoidAssignmentToAutomaticVar  Warning    10    Variable 'error' is an automatic variable...
	//   PSUseShouldProcessForStateChange Error      25    Cmdlet should use ShouldProcess...
	//
	// 提示：
	//   - 修复输出的错误和警告需要手动编辑 .ps1 文件
	//   - 不提供自动修正功能（与 Prettier 不同）
	// ======================================================
	//
	// 格式化结果示例（使用方案 1 或 2）：
	// 输入（未格式化）：
	//   function Test { write-host "Hello World" }
	//   if( $var -eq $true ){
	//   Write-Output \"Test\"
	//   }
	//   \$array=@(1,2,3  , 4)
	//
	// 输出（格式化后）：
	//   function Test {
	//       Write-Host \"Hello World\"
	//   }
	//   if ($var -eq $true) {
	//       Write-Output \"Test\"
	//   }
	//   \$array = @(1, 2, 3, 4)
	//
	// 格式化规则：
	//   - 函数与条件块的缩进标准化（4 个空格）
	//   - Cmdlet 首字母大写（Write-Host 而非 write-host）
	//   - 运算符前后空格规范（\$var -eq \$true，=两侧各一空格）
	//   - 括号内空格处理（(condition) 而非 ( condition )）
	//   - 数组元素逗号后加空格
	//   - 代码块换行与对齐

	// ==================== 插件配置 ====================
	// plugins 说明：指定 Prettier 使用的扩展插件
	// 这是使 Prettier 支持非原生格式（如 PowerShell）的唯一方式
	//
	// 注意：plugins 数组必须明确指定，Prettier 不会自动发现 node_modules 中的插件
	// 只有在 plugins 数组中声明的插件才会被加载
	//
	// prettier-plugin-powershell 功能说明：
	// - 支持 *.ps1 文件的格式化
	// - 为 PowerShell 代码提供 Cmdlet 大小写标准、运算符间距、缩进规范
	// - 与 Prettier 的 printWidth、tabWidth 等选项集成
	//
	// 使用前提：
	//   npm install --save-dev prettier-plugin-powershell
	//
	// 安装后首次使用时需要清除缓存：
	//   # Linux/macOS：
	//   rm -rf node_modules/.cache
	//   
	//   # Windows PowerShell：
	//   Remove-Item -Path node_modules\.cache -Recurse -Force
	plugins: ['prettier-plugin-powershell']
};
