/** 文件：useExternalLink.ts | 用途：封装统一外链打开逻辑（优先后端命令，失败回退浏览器） | 关键对象：useExternalLink, openExternalLink */
// 本行目的：引入 Tauri invoke，用于调用后端外链打开命令。
import { invoke } from "@tauri-apps/api/core";

// 变量：useExternalLink | 含义：外链能力组合式函数入口 | 类型：() => { openExternalLink: (url?: string | null) => Promise<void> } | 作用域：模块级
/** 函数：useExternalLink | 输入：无 | 输出：openExternalLink 方法 | 可能失败：后端命令异常时回退浏览器窗口打开 */
// 本行目的：导出外链打开能力。
export const useExternalLink = () => {
    // 变量：openExternalLink | 含义：打开外部链接的统一方法 | 类型：(url?: string | null) => Promise<void> | 作用域：useExternalLink 内部
    /** 函数：openExternalLink | 输入：可选 URL 字符串 | 输出：无 | 可能失败：invoke 失败时记录错误并回退 window.open */
    // 本行目的：定义外链打开流程。
    const openExternalLink = async (url?: string | null) => {
        // 本行目的：URL 为空时直接返回。
        if (!url) return;

        try {
            // 本行目的：优先通过后端命令打开外链。
            await invoke("open_external_link", { url });
        } catch (err) {
            // 本行目的：记录后端打开失败日志。
            console.error("openExternalLink failed", err);
            // 本行目的：回退到浏览器默认新窗口打开。
            window.open(url, "_blank");
        }
    };

    // 本行目的：返回外链方法供页面复用。
    return { openExternalLink };
};
