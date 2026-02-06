// 清理脚本：删除构建产物与打包输出，避免仓库被临时文件污染。
// 用法：node scripts/clean.js [--dry-run] [--node-modules] [--all]

import { rm, stat } from "node:fs/promises";
import { resolve, dirname, sep } from "node:path";
import { fileURLToPath } from "node:url";

// 解析当前脚本目录（ESM 环境下没有 __dirname，需要手动计算）。
const __dirname = dirname(fileURLToPath(import.meta.url));
// 仓库根目录 = scripts 的上一级。
const repoRoot = resolve(__dirname, "..");

// 解析命令行参数：支持 dry-run 与是否清理 node_modules。
const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const includeNodeModules = args.includes("--node-modules") || args.includes("--all");
const showHelp = args.includes("--help") || args.includes("-h");

// 需要清理的基础路径：构建输出与打包目录。
const baseTargets = ["build", "dist", "node_modules"]; // build 下包含 tauri-target / frontend 等。
// 额外目标：防止用户使用默认 target 目录时遗留产物。
const extraTargets = [
    "apps/desktop/backend/gen",
	// "apps/desktop/backend/target",
	// "apps/desktop/frontend/dist"
];

if (showHelp) {
	console.log("\nHanamiRIP-CN clean 脚本\n");
	console.log("用法：node scripts/clean.js [--dry-run] [--node-modules] [--all]\n");
	console.log("--dry-run       仅打印将要删除的路径，不实际删除");
	console.log("--node-modules  额外删除 node_modules（耗时较长）");
	console.log("--all           等同于 --node-modules\n");
	process.exit(0);
}

// 拼出最终清理列表。
const targets = [...baseTargets, ...extraTargets];
if (includeNodeModules) {
	targets.push("node_modules");
}

// 确保目标路径在仓库内，避免误删系统目录。
function resolveTargetPath(relativePath) {
	const absPath = resolve(repoRoot, relativePath);
	const rootWithSep = repoRoot.endsWith(sep) ? repoRoot : repoRoot + sep;
	if (absPath !== repoRoot && !absPath.startsWith(rootWithSep)) {
		throw new Error(`拒绝清理仓库外路径：${absPath}`);
	}
	return absPath;
}

// 删除单个路径：存在则删除，不存在则跳过。
async function removePath(relativePath) {
	const absPath = resolveTargetPath(relativePath);
	try {
		await stat(absPath);
	} catch {
		console.log(`跳过（不存在）：${relativePath}`);
		return;
	}

	if (dryRun) {
		console.log(`将删除：${relativePath}`);
		return;
	}

	await rm(absPath, { recursive: true, force: true });
	console.log(`已删除：${relativePath}`);
}

// 主流程：按顺序清理所有目标路径。
for (const target of targets) {
	// eslint-disable-next-line no-await-in-loop
	await removePath(target);
}

console.log(dryRun ? "\n清理预览完成。" : "\n清理完成。");

