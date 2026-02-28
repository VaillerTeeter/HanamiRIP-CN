/** 文件：clean.js | 用途：清理仓库构建产物与临时目录 | 关键对象：resolveTargetPath、removePath、targets */

// 引入删除目录与检测路径存在性的异步文件系统 API。
import { rm, stat } from "node:fs/promises";
// 引入路径拼接与标准化工具，避免硬编码分隔符问题。
import { resolve, dirname, sep } from "node:path";
// 引入 ESM 环境下获取当前文件路径的工具。
import { fileURLToPath } from "node:url";

// 变量：__dirname | 含义：当前脚本所在目录绝对路径 | 类型：string | 作用域：模块级
const __dirname = dirname(fileURLToPath(import.meta.url));
// 变量：repoRoot | 含义：仓库根目录绝对路径（scripts 的上一级） | 类型：string | 作用域：模块级
const repoRoot = resolve(__dirname, "..");
// 变量：args | 含义：命令行参数数组（去掉 node 与脚本名） | 类型：string[] | 作用域：模块级
const args = process.argv.slice(2);
// 变量：dryRun | 含义：是否只预览不执行删除 | 类型：boolean | 作用域：模块级
const dryRun = args.includes("--dry-run");
// 变量：includeNodeModules | 含义：是否额外包含 node_modules 清理 | 类型：boolean | 作用域：模块级
const includeNodeModules = args.includes("--node-modules") || args.includes("--all");
// 变量：showHelp | 含义：是否显示帮助信息 | 类型：boolean | 作用域：模块级
const showHelp = args.includes("--help") || args.includes("-h");
// 变量：baseTargets | 含义：默认清理目标集合 | 类型：string[] | 作用域：模块级
const baseTargets = ["build", "dist", "node_modules"];

// 变量：extraTargets | 含义：附加清理目录（代码生成产物） | 类型：string[] | 作用域：模块级
const extraTargets = ["apps/desktop/backend/gen"];

// 若请求帮助，则打印说明并立即退出。
if (showHelp) {
    console.log("\nHanamiRIP-CN clean 脚本\n");
    console.log("用法：node scripts/clean.js [--dry-run] [--node-modules] [--all]\n");
    console.log("--dry-run       仅打印将要删除的路径，不实际删除");
    console.log("--node-modules  额外删除 node_modules（耗时较长）");
    console.log("--all           等同于 --node-modules\n");
    process.exit(0);
}

// 变量：targets | 含义：最终要处理的清理路径列表 | 类型：string[] | 作用域：模块级
const targets = [...baseTargets, ...extraTargets];
// 用户显式传入 --node-modules 或 --all 时，再追加一次 node_modules 目标（兼容旧参数语义）。
if (includeNodeModules) {
    targets.push("node_modules");
}

/** 函数：resolveTargetPath | 输入：relativePath（相对仓库根路径） | 输出：受安全校验的绝对路径 | 可能失败：路径越界时抛出 Error */
function resolveTargetPath(relativePath) {
    // 变量：absPath | 含义：目标相对路径解析后的绝对路径 | 类型：string | 作用域：函数内
    const absPath = resolve(repoRoot, relativePath);
    // 变量：rootWithSep | 含义：带尾部分隔符的仓库根路径，用于 startsWith 安全比较 | 类型：string | 作用域：函数内
    const rootWithSep = repoRoot.endsWith(sep) ? repoRoot : repoRoot + sep;
    // 禁止删除仓库外路径，防止误删系统文件。
    if (absPath !== repoRoot && !absPath.startsWith(rootWithSep)) {
        throw new Error(`拒绝清理仓库外路径：${absPath}`);
    }
    // 返回通过校验的绝对路径。
    return absPath;
}

/** 函数：removePath | 输入：relativePath（相对路径） | 输出：Promise<void> | 可能失败：文件系统权限不足或路径被占用 */
async function removePath(relativePath) {
    // 变量：absPath | 含义：通过安全校验后的绝对删除路径 | 类型：string | 作用域：函数内
    const absPath = resolveTargetPath(relativePath);
    try {
        // 先探测路径是否存在，不存在直接跳过。
        await stat(absPath);
    } catch {
        console.log(`跳过（不存在）：${relativePath}`);
        return;
    }

    // dry-run 模式只打印目标，不执行实际删除。
    if (dryRun) {
        console.log(`将删除：${relativePath}`);
        return;
    }

    // 递归强制删除目录或文件。
    await rm(absPath, { recursive: true, force: true });
    console.log(`已删除：${relativePath}`);
}

// 顺序执行清理，降低并发删除导致的冲突概率。
for (const target of targets) {
    // eslint-disable-next-line no-await-in-loop
    await removePath(target);
}

// 根据模式输出结束信息。
console.log(dryRun ? "\n清理预览完成。" : "\n清理完成。");
